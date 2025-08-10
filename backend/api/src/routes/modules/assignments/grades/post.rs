use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use sea_orm::{
   ColumnTrait,EntityTrait, QueryFilter, QueryOrder,
    Set, TransactionTrait, QuerySelect, DbErr,
};
use serde::Serialize;

use crate::response::ApiResponse;
use util::state::AppState;
use sea_orm::ConnectionTrait;

use db::models::{
    assignment::{Entity as AssignmentEntity, Column as AssignmentCol},
    grade::{Entity as GradeEntity, ActiveModel as GradeActive, Column as GradeCol},
    user_module_role::{Entity as UMR, Column as UMRCol},
};

use util::execution_config::ExecutionConfig;



#[derive(Clone, Copy, Debug)]
enum GradingPolicy {
    Best,
    Last,
}


#[derive(Serialize)]
pub struct ComputeGradesData {
    assignment_id: i64,
    graded_count: usize,
}
 
pub async fn post_compute_grades(
    State(app): State<AppState>,
    Path((module_id, assignment_id)): Path<(i64, i64)>,
) -> (StatusCode, Json<ApiResponse<ComputeGradesData>>) {
    let db = app.db();

    // ensure assignment exists in this module
    let _assignment = match AssignmentEntity::find()
        .filter(AssignmentCol::Id.eq(assignment_id as i32))
        .filter(AssignmentCol::ModuleId.eq(module_id as i32))
        .one(db)
        .await
    {
        Ok(Some(a)) => a,
        Ok(None) => {
            return (
                StatusCode::NOT_FOUND,
                Json(ApiResponse::<ComputeGradesData>::error("Assignment not found")),
            );
        }
        Err(_) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::<ComputeGradesData>::error("Database error loading assignment")),
            );
        }
    };

    //  grading policy
    let policy = match get_grading_policy(module_id, assignment_id).as_str() {
        "last" => GradingPolicy::Last,
        _ => GradingPolicy::Best,
    };

    //  transaction
    let trans = match db.begin().await {
        Ok(t) => t,
        Err(_) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::<ComputeGradesData>::error("Failed to start transaction")),
            );
        }
    };

    //) delete existing grades for this assignment
    if let Err(_) = GradeEntity::delete_many()
        .filter(GradeCol::AssignmentId.eq(assignment_id))
        .exec(&trans)
        .await
    {
        let _ = trans.rollback().await;
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse::<ComputeGradesData>::error("Failed clearing previous grades")),
        );
    }

    // fetch ALL enrolled students (role = "student") for this module
    let student_ids: Vec<i32> = match UMR::find()
        .filter(UMRCol::ModuleId.eq(module_id as i32))
        .filter(UMRCol::Role.eq("student".to_string()))
        .select_only()
        .column(UMRCol::UserId)
        .distinct()
        .into_tuple::<i32>()
        .all(&trans)
        .await
    {
        Ok(v) => v,
        Err(_) => {
            let _ = trans.rollback().await;
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::<ComputeGradesData>::error("Failed to fetch enrolled students")),
            );
        }
    };

    // compute submission per policy & stage grade rows (null + 0.0 for non-submitters)
    let mut staged = Vec::with_capacity(student_ids.len());
    for student_id in student_ids.iter().copied() {
        let picked = match pick_submission_for_student(&trans, assignment_id, student_id, policy).await
        {
            Ok(p) => p,
            Err(_) => {
                let _ = trans.rollback().await;
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(ApiResponse::<ComputeGradesData>::error("Failed selecting submissions")),
                );
            }
        };

        let (submission_id, score) = match picked {
            Some((sid, sc)) => (Some(sid), sc),
            None => (None, 0.0_f32),
        };

        staged.push(GradeActive {
            assignment_id: Set(assignment_id),
            student_id: Set(student_id as i64),
            submission_id: Set(submission_id),
            score: Set(score),
            ..Default::default()
        });
    }

    // insert new grades
    if !staged.is_empty() {
        if let Err(_) = GradeEntity::insert_many(staged).exec(&trans).await {
            let _ = trans.rollback().await;
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::<ComputeGradesData>::error("Failed inserting grades")),
            );
        }
    }

    // commit
    if let Err(_) = trans.commit().await {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse::<ComputeGradesData>::error("Failed to commit transaction")),
        );
    }

    // success
    (
        StatusCode::CREATED,
        Json(ApiResponse::success(
            ComputeGradesData {
                assignment_id,
                graded_count: student_ids.len(),
            },
            "Grades computed successfully",
        )),
    )
}



fn get_grading_policy(module_id: i64, assignment_id: i64) -> String {
    ExecutionConfig::get_execution_config(module_id, assignment_id)
        .ok()
        .and_then(|cfg| cfg.grading.and_then(|g| g.policy))
        .map(|s| s.to_ascii_lowercase())
        .filter(|s| s == "best" || s == "last")
        .unwrap_or_else(|| "best".to_string())
}

async fn pick_submission_for_student<C>(
    db: &C,
    assignment_id: i64,
    student_id: i32,
    policy: GradingPolicy,
) -> Result<Option<(i64, f32)>, DbErr>
where
    C: ConnectionTrait,
{
    use db::models::assignment_submission::{
        Entity as SubmissionEntity, Column as SubmissionCol, Model as Submission,
    };

    let q = SubmissionEntity::find()
        .filter(SubmissionCol::AssignmentId.eq(assignment_id as i32))
        .filter(SubmissionCol::UserId.eq(student_id));

    let rec = match policy {
        GradingPolicy::Best => {
            q.filter(SubmissionCol::Score.is_not_null())
                .order_by_desc(SubmissionCol::Score)
                .order_by_desc(SubmissionCol::CreatedAt)
                .one(db)
                .await?
        }
        GradingPolicy::Last => q.order_by_desc(SubmissionCol::CreatedAt).one(db).await?,
    };

    Ok(rec.map(|r: Submission| (r.id as i64, r.score.unwrap_or(0.0))))
}