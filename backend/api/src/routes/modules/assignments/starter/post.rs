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
use db::models::{assignment_memo_output, assignment_task};
use util::{
    execution_config::ExecutionConfig,
    mark_allocator::{TaskInfo, generate_allocator, save_allocator},
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
    #[serde(default)]
    valgrind: bool,
}

// TODO
pub async fn create(
    State(app_state): State<AppState>,
    Path((module_id, assignment_id)): Path<(i64, i64)>,
    Json(body): Json<StarterReq>,
) -> impl IntoResponse {
    let db = app_state.db();

    println!("Starting starter installation: {}", &body.id);

    // 1) Ensure assignment exists
    match AssignmentEntity::find()
        .filter(AssignmentCol::Id.eq(assignment_id))
        .filter(AssignmentCol::ModuleId.eq(module_id))
        .one(db)
        .await
    {
        Ok(Some(_)) => println!("Assignment found: {}", assignment_id),
        Ok(None) => {
            println!("Assignment not found: {}", assignment_id);
            return (
                StatusCode::NOT_FOUND,
                Json(ApiResponse::<JsonValue>::error("Assignment not found")),
            );
        }
        Err(e) => {
            println!("Failed to fetch assignment: {:?}", e);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::<JsonValue>::error(
                    "Failed to fetch assignment",
                )),
            );
        }
    }

    // 2) Resolve pack & check embedded assets
    let Some(pack) = find_pack(&body.id) else {
        println!("Unknown starter id: {}", &body.id);
        return (
            StatusCode::UNPROCESSABLE_ENTITY,
            Json(ApiResponse::<JsonValue>::error("Unknown starter id")),
        );
    };
    println!("Pack found: {}", pack.id);

    if STARTERS_ROOT.get_dir(pack.id).is_none() {
        println!("Starter assets missing for pack: {}", pack.id);
        return (
            StatusCode::UNPROCESSABLE_ENTITY,
            Json(ApiResponse::<JsonValue>::error("Starter assets missing")),
        );
    }

    // 3) Wipe starter artifacts
    if wipe_assignment_starter(module_id, assignment_id, db)
        .await
        .is_err()
    {
        println!(
            "Failed to reset starter artifacts for assignment {}",
            assignment_id
        );
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse::<JsonValue>::error(
                "Failed to reset existing starter artifacts",
            )),
        );
    }
    println!("Starter artifacts wiped for assignment {}", assignment_id);

    // 4) Write config
    let mut cfg = ExecutionConfig::default_config();
    cfg.project.language = pack.language;
    if cfg.save(module_id, assignment_id).is_err() {
        println!(
            "Failed to save execution config for assignment {}",
            assignment_id
        );
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse::<JsonValue>::error(
                "Failed to save execution configuration",
            )),
        );
    }
    println!("Execution config saved");

    // 5) Install zips
    let zip_steps = ["main", "makefile", "memo", "spec"];
    for subdir in zip_steps {
        if let Err(e) = install_zip_if_present_root(
            pack.id,
            subdir,
            module_id,
            assignment_id,
            match subdir {
                "main" => FileType::Main,
                "makefile" => FileType::Makefile,
                "memo" => FileType::Memo,
                "spec" => FileType::Spec,
                _ => FileType::Main,
            },
            &format!("{}.zip", subdir),
            db,
        )
        .await
        {
            println!("Failed to install {} starter files: {}", subdir, e);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::<JsonValue>::error(&format!(
                    "Failed to install the '{}' starter files",
                    subdir
                ))),
            );
        }
        println!("Installed {} starter files", subdir);
    }

    // 6) Create tasks
    match create_tasks_from_assets_root(&pack.id, db, assignment_id).await {
        Ok(n) => println!("Created {} tasks", n),
        Err(e) => {
            println!("Failed to create tasks: {}", e);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::<JsonValue>::error(
                    "Failed to create tasks from assets",
                )),
            );
        }
    }

    // 7) Best-effort generators
    let _ = code_runner::create_memo_outputs_for_all_tasks(db, assignment_id).await;
    let _ = try_generate_allocator(module_id, assignment_id, db).await;

    // 8) Flip to ready
    let _ = AssignmentModel::try_transition_to_ready(db, module_id, assignment_id).await;

    println!(
        "Starter installation complete for assignment {}",
        assignment_id
    );
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
            t.valgrind,
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
            valgrind: t.valgrind,
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

    // Generate normalized allocator (this pads regex if scheme == Regex)
    let alloc = generate_allocator(module_id, assignment_id, &pairs)
        .await
        .map_err(|e| format!("Failed to generate allocator: {e}"))?;

    // Persist normalized JSON (no legacy wire shape)
    save_allocator(module_id, assignment_id, &alloc)
        .map_err(|e| format!("Failed to save allocator: {e}"))?;

    Ok(())
}
