use axum::{extract::Query, Json, response::IntoResponse};
use db::{
    connect,
    models::{
        assignment_submission::Entity as SubmissionEntity,
        plagiarism_case::{self, Entity as PlagiarismEntity},
        user::Entity as UserEntity,
    },
};
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub struct PlagiarismQuery {
    pub status: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct Link {
    pub source: String,
    pub target: String,
}

#[derive(Debug, Serialize)]
pub struct LinksResponse {
    pub links: Vec<Link>,
}

pub async fn get_graph(Query(query): Query<PlagiarismQuery>) -> impl IntoResponse {
    let db = connect().await;

    let mut query_builder = PlagiarismEntity::find();

    if let Some(status_str) = query.status {
        if let Ok(status) = status_str.parse::<plagiarism_case::Status>() {
            query_builder = query_builder.filter(plagiarism_case::Column::Status.eq(status));
        }
    }

    let plagiarism_cases = match query_builder.all(&db).await {
        Ok(cases) => cases,
        Err(_) => {
            return (
                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": "Failed to fetch plagiarism cases"})),
            );
        }
    };

    let mut links = Vec::new();

    for case in plagiarism_cases {
        let submission1 = SubmissionEntity::find_by_id(case.submission_id_1)
            .one(&db)
            .await
            .ok()
            .flatten();

        let submission2 = SubmissionEntity::find_by_id(case.submission_id_2)
            .one(&db)
            .await
            .ok()
            .flatten();

        if let (Some(sub1), Some(sub2)) = (submission1, submission2) {
            let user1 = UserEntity::find_by_id(sub1.user_id)
                .one(&db)
                .await
                .ok()
                .flatten();

            let user2 = UserEntity::find_by_id(sub2.user_id)
                .one(&db)
                .await
                .ok()
                .flatten();

            if let (Some(u1), Some(u2)) = (user1, user2) {
                links.push(Link {
                    source: u1.username,
                    target: u2.username,
                });
            }
        }
    }

    // Step 4: Return response with links
    (
        axum::http::StatusCode::OK,
        Json(serde_json::json!({ "links": links })),
    )
}
