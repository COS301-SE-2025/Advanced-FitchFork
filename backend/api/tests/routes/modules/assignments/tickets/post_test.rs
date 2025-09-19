#[cfg(test)]
mod tests {
    use api::auth::generate_jwt;
    use axum::{
        body::Body,
        http::{Request, StatusCode},
    };
    use chrono::{TimeZone, Utc};
    use db::models::{
        assignment::{AssignmentType, Model as AssignmentModel},
        module::Model as ModuleModel,
        user::Model as UserModel,
        user_module_role::{Model as UserModuleRole, Role},
    };
    use serde_json::json;
    use tower::ServiceExt;

    use crate::helpers::app::make_test_app_with_storage;

    struct TestData {
        user: UserModel,
        module: ModuleModel,
        assignment: AssignmentModel,
    }

    async fn setup_test_data(db: &sea_orm::DatabaseConnection) -> TestData {
        let user = UserModel::create(db, "test_user", "test@example.com", "pass", false)
            .await
            .unwrap();

        let module = ModuleModel::create(db, "330", 2025, Some("test description"), 16)
            .await
            .unwrap();

        UserModuleRole::assign_user_to_module(db, user.id, module.id, Role::Student)
            .await
            .unwrap();

        let assignment = AssignmentModel::create(
            db,
            module.id,
            "Assignment 1",
            Some("Desc 1"),
            AssignmentType::Assignment,
            Utc.with_ymd_and_hms(2024, 1, 1, 0, 0, 0).unwrap(),
            Utc.with_ymd_and_hms(2024, 1, 31, 23, 59, 59).unwrap(),
        )
        .await
        .unwrap();

        TestData {
            user,
            module,
            assignment,
        }
    }

    #[tokio::test]
    async fn create_ticket_test() {
        let (app, app_state, _tmp) = make_test_app_with_storage().await;
        let data = setup_test_data(app_state.db()).await;
        let (token, _) = generate_jwt(data.user.id, data.user.admin);
        let uri = format!(
            "/api/modules/{}/assignments/{}/tickets",
            data.module.id, data.assignment.id
        );
        let body = json!({
          "title": "Clarification needed on Task 2",
          "description": "Should we include error handling for edge cases?"
        });

        let req = Request::builder()
            .method("POST")
            .uri(&uri)
            .header("Authorization", format!("Bearer {}", token))
            .header("Content-Type", "application/json")
            .body(Body::from(body.to_string()))
            .unwrap();

        let response = app.oneshot(req).await.unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();

        assert_eq!(json["message"], "Ticket created successfully");
    }

    #[tokio::test]
    async fn create_invalid_ticket_test() {
        let (app, app_state, _tmp) = make_test_app_with_storage().await;
        let data = setup_test_data(app_state.db()).await;
        let uri = format!(
            "/api/modules/{}/assignments/{}/tickets",
            data.module.id, data.assignment.id
        );
        let body = json!({
          "title": "Clarification needed on Task 2",
          "description": "Should we include error handling for edge cases?"
        });

        let req = Request::builder()
            .method("POST")
            .uri(&uri)
            .header("Content-Type", "application/json")
            .body(Body::from(body.to_string()))
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();

        assert_eq!(json["message"], "Authentication required");
    }
}
