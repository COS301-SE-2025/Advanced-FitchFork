use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
};
use include_dir::{Dir, include_dir};
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};
use serde::Deserialize;
use serde_json::Value as JsonValue;
use std::{fs, io::Write};
use zip::write::FileOptions;

use super::common::find_pack;
use crate::response::ApiResponse;
use db::models::{
    assignment::{Column as AssignmentCol, Entity as AssignmentEntity, Model as AssignmentModel},
    assignment_file::{
        Column as AssignmentFileCol, Entity as AssignmentFileEntity, FileType,
        Model as AssignmentFileModel,
    },
    assignment_memo_output::{Column as MemoOutCol, Entity as MemoOutEntity},
    assignment_task::{Column as TaskCol, Entity as TaskEntity, Model as TaskModel},
};
use util::{
    execution_config::ExecutionConfig,
    paths::{
        assignment_dir, config_dir, ensure_dir, main_dir, makefile_dir, mark_allocator_dir,
        mark_allocator_path, memo_dir, memo_output_dir, spec_dir, storage_root,
    },
    state::AppState,
};

static STARTERS_ROOT: Dir<'static> = include_dir!("$CARGO_MANIFEST_DIR/assets/starters");

#[derive(Deserialize)]
pub struct StarterReq {
    /// Starter pack ID (matches directory under assets/starters/<id>/)
    pub id: String,
}

#[derive(Deserialize)]
struct TaskSeed {
    task_number: i64,
    name: String,
    command: String,
    #[serde(default)]
    code_coverage: bool,
}

// TODO
pub async fn create(
    State(app_state): State<AppState>,
    Path((module_id, assignment_id)): Path<(i64, i64)>,
    Json(body): Json<StarterReq>,
) -> impl IntoResponse {
    let db = app_state.db();

    // 1) Ensure assignment exists and belongs to module.
    match AssignmentEntity::find()
        .filter(AssignmentCol::Id.eq(assignment_id))
        .filter(AssignmentCol::ModuleId.eq(module_id))
        .one(db)
        .await
    {
        Ok(Some(_)) => {}
        Ok(None) => {
            return (
                StatusCode::NOT_FOUND,
                Json(ApiResponse::<JsonValue>::error("Assignment not found")),
            );
        }
        Err(_) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::<JsonValue>::error(
                    "Failed to fetch assignment",
                )),
            );
        }
    }

    // 2) Resolve pack & ensure embedded assets exist.
    let Some(pack) = find_pack(&body.id) else {
        return (
            StatusCode::UNPROCESSABLE_ENTITY,
            Json(ApiResponse::<JsonValue>::error("Unknown starter id")),
        );
    };
    if STARTERS_ROOT.get_dir(pack.id).is_none() {
        return (
            StatusCode::UNPROCESSABLE_ENTITY,
            Json(ApiResponse::<JsonValue>::error("Starter assets missing")),
        );
    }

    // 3) Full reset of starter artifacts (DB + FS).
    if wipe_assignment_starter(module_id, assignment_id, db)
        .await
        .is_err()
    {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse::<JsonValue>::error(
                "Failed to reset existing starter artifacts for this assignment.",
            )),
        );
    }

    // 4) Write config (language from pack).
    let mut cfg = ExecutionConfig::default_config();
    cfg.project.language = pack.language;
    if cfg.save(module_id, assignment_id).is_err() {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse::<JsonValue>::error(
                "Failed to save execution configuration.",
            )),
        );
    }

    // 5) Save asset subfolders as flat zips (lookup via ROOT using pack.id + subfolder).
    if install_zip_if_present_root(
        &pack.id,
        "main",
        module_id,
        assignment_id,
        FileType::Main,
        "main.zip",
        db,
    )
    .await
    .is_err()
    {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse::<JsonValue>::error(
                "Failed to install the 'main' starter files.",
            )),
        );
    }
    if install_zip_if_present_root(
        &pack.id,
        "makefile",
        module_id,
        assignment_id,
        FileType::Makefile,
        "makefile.zip",
        db,
    )
    .await
    .is_err()
    {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse::<JsonValue>::error(
                "Failed to install the 'makefile' starter files.",
            )),
        );
    }
    if install_zip_if_present_root(
        &pack.id,
        "memo",
        module_id,
        assignment_id,
        FileType::Memo,
        "memo.zip",
        db,
    )
    .await
    .is_err()
    {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse::<JsonValue>::error(
                "Failed to install the 'memo' starter files.",
            )),
        );
    }
    if install_zip_if_present_root(
        &pack.id,
        "spec",
        module_id,
        assignment_id,
        FileType::Spec,
        "spec.zip",
        db,
    )
    .await
    .is_err()
    {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse::<JsonValue>::error(
                "Failed to install the 'spec' starter files.",
            )),
        );
    }

    // 6) Seed tasks (tasks.json under pack root, also via ROOT).
    if create_tasks_from_assets_root(&pack.id, db, assignment_id)
        .await
        .is_err()
    {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse::<JsonValue>::error(
                "Failed to create tasks from assets.",
            )),
        );
    }

    // 7) Best-effort generators (don’t fail request).
    let _ = code_runner::create_memo_outputs_for_all_tasks(db, assignment_id).await;
    let _ = try_generate_allocator(module_id, assignment_id, db).await;

    // 8) Flip to ready (ignore result).
    let _ = AssignmentModel::try_transition_to_ready(db, module_id, assignment_id).await;

    (
        StatusCode::CREATED,
        Json(ApiResponse::<JsonValue>::success_without_data(
            "Starter installed.",
        )),
    )
}

// ────────────────────────────── helpers ──────────────────────────────

async fn wipe_assignment_starter(
    module_id: i64,
    assignment_id: i64,
    db: &sea_orm::DatabaseConnection,
) -> Result<(), String> {
    TaskEntity::delete_many()
        .filter(TaskCol::AssignmentId.eq(assignment_id))
        .exec(db)
        .await
        .map_err(|e| e.to_string())?;

    AssignmentFileEntity::delete_many()
        .filter(AssignmentFileCol::AssignmentId.eq(assignment_id))
        .filter(AssignmentFileCol::FileType.is_in(vec![
            FileType::Main.to_string(),
            FileType::Makefile.to_string(),
            FileType::Memo.to_string(),
            FileType::Spec.to_string(),
            FileType::Config.to_string(),
            FileType::MarkAllocator.to_string(),
        ]))
        .exec(db)
        .await
        .map_err(|e| e.to_string())?;

    MemoOutEntity::delete_many()
        .filter(MemoOutCol::AssignmentId.eq(assignment_id))
        .exec(db)
        .await
        .map_err(|e| e.to_string())?;

    let dirs = [
        assignment_dir(module_id, assignment_id),
        config_dir(module_id, assignment_id),
        main_dir(module_id, assignment_id),
        makefile_dir(module_id, assignment_id),
        memo_dir(module_id, assignment_id),
        spec_dir(module_id, assignment_id),
        memo_output_dir(module_id, assignment_id),
        mark_allocator_dir(module_id, assignment_id),
    ];
    for d in dirs {
        if d.exists() {
            let _ = fs::remove_dir_all(&d);
        }
        ensure_dir(&d).map_err(|e| e.to_string())?;
    }

    let alloc_path = mark_allocator_path(module_id, assignment_id);
    if alloc_path.exists() {
        let _ = fs::remove_file(alloc_path);
    }

    Ok(())
}

/// Look up a subdir as ROOT path: starters/<pack_id>/<subdir>
async fn install_zip_if_present_root(
    pack_id: &str,
    subdir: &str,
    module_id: i64,
    assignment_id: i64,
    file_type: FileType,
    out_name: &str,
    db: &sea_orm::DatabaseConnection,
) -> Result<(), String> {
    let rel = format!("{}/{}", pack_id, subdir);
    let Some(dir) = STARTERS_ROOT.get_dir(&rel) else {
        return Ok(());
    };

    let bytes = zip_dir_flat(dir).map_err(|e| format!("zip failed: {e}"))?;

    AssignmentFileModel::save_file(db, assignment_id, module_id, file_type, out_name, &bytes)
        .await
        .map_err(|e| format!("DB save_file failed: {e}"))?;

    Ok(())
}

/// Zip the embedded dir so files are at ZIP ROOT (no extra top-level folder).
fn zip_dir_flat(d: &Dir<'_>) -> Result<Vec<u8>, std::io::Error> {
    use zip::{CompressionMethod, ZipWriter};

    let mut buf = std::io::Cursor::new(Vec::<u8>::new());
    let mut zip = ZipWriter::new(&mut buf);
    let opts = FileOptions::<()>::default().compression_method(CompressionMethod::Deflated);

    fn add(
        dir: &Dir<'_>,
        prefix: &str,
        zip: &mut zip::ZipWriter<&mut std::io::Cursor<Vec<u8>>>,
        opts: FileOptions<()>,
    ) -> std::io::Result<()> {
        for f in dir.files() {
            let name = if prefix.is_empty() {
                f.path().file_name().unwrap().to_string_lossy().into_owned()
            } else {
                format!(
                    "{}/{}",
                    prefix,
                    f.path().file_name().unwrap().to_string_lossy()
                )
            };
            zip.start_file(name, opts)?;
            zip.write_all(f.contents())?;
        }
        for sub in dir.dirs() {
            let sub_name = sub.path().file_name().unwrap().to_string_lossy();
            let next_prefix = if prefix.is_empty() {
                sub_name.into_owned()
            } else {
                format!("{}/{}", prefix, sub_name)
            };
            add(sub, &next_prefix, zip, opts)?;
        }
        Ok(())
    }

    add(d, "", &mut zip, opts)?;
    zip.finish()?;
    Ok(buf.into_inner())
}

async fn create_tasks_from_assets_root(
    pack_id: &str,
    db: &sea_orm::DatabaseConnection,
    assignment_id: i64,
) -> Result<usize, String> {
    let rel = format!("{}/tasks.json", pack_id);
    let Some(tasks_file) = STARTERS_ROOT.get_file(&rel) else {
        return Ok(0);
    };

    let seeds: Vec<TaskSeed> = serde_json::from_slice(tasks_file.contents())
        .map_err(|e| format!("Invalid tasks.json: {e}"))?;

    let mut created = 0usize;
    for t in seeds {
        TaskModel::create(
            db,
            assignment_id,
            t.task_number,
            &t.name,
            &t.command,
            t.code_coverage,
        )
        .await
        .map_err(|e| format!("Failed to create task {}: {}", t.task_number, e))?;
        created += 1;
    }
    Ok(created)
}

async fn try_generate_allocator(
    module_id: i64,
    assignment_id: i64,
    db: &sea_orm::DatabaseConnection,
) -> Result<(), String> {
    use db::models::{assignment_memo_output, assignment_task};
    use util::mark_allocator::{SaveError, TaskInfo, generate_allocator};

    let tasks = assignment_task::Entity::find()
        .filter(assignment_task::Column::AssignmentId.eq(assignment_id))
        .all(db)
        .await
        .map_err(|e| e.to_string())?;

    let mo = assignment_memo_output::Entity::find()
        .filter(assignment_memo_output::Column::AssignmentId.eq(assignment_id))
        .all(db)
        .await
        .map_err(|e| e.to_string())?;

    let base = memo_output_dir(module_id, assignment_id);
    let mut pairs = Vec::new();

    for t in tasks {
        let info = TaskInfo {
            id: t.id,
            task_number: t.task_number,
            code_coverage: t.code_coverage,
            name: if t.name.trim().is_empty() {
                format!("Task {}", t.task_number)
            } else {
                t.name.clone()
            },
        };

        let memo_path = mo
            .iter()
            .find(|m| m.task_id == t.id)
            .map(|m| storage_root().join(&m.path))
            .unwrap_or_else(|| base.join(format!("no_memo_for_task_{}", t.id)));

        pairs.push((info, memo_path));
    }

    match generate_allocator(module_id, assignment_id, &pairs).await {
        Ok(_) => Ok(()),
        Err(SaveError::DirectoryNotFound) => Err(
            "Mark allocator directory not found. Ensure the assignment storage folder exists and memo outputs were generated."
                .to_string(),
        ),
        Err(_) => Err("Failed to generate the mark allocator configuration.".to_string()),
    }
}
