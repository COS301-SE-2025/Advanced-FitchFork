#[cfg(test)]
mod tests {
    use crate::helpers::app::make_test_app_with_storage;
    use api::auth::generate_jwt;
    use chrono::{TimeZone, Utc};
    use db::models::{
        assignment::{AssignmentType, Model as AssignmentModel},
        module::Model as ModuleModel,
        tickets::Model as TicketModel,
        user::Model as UserModel,
        user_module_role::{Model as UserModuleRole, Role},
    };

    use axum::{
        body::Body,
        http::{Request, StatusCode},
    };
    use tower::ServiceExt;
    struct TestData {
        user: UserModel,
        invalid_user: UserModel,
        ticket: TicketModel,
        module: ModuleModel,
        assignment: AssignmentModel,
    }

    async fn setup_test_data(db: &sea_orm::DatabaseConnection) -> TestData {
        let user = UserModel::create(db, "test_user", "test@example.com", "pass", false)
            .await
            .unwrap();

        let invalid_user =
            UserModel::create(db, "invalid_user", "invalid@email.com", "pass2", false)
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

        let ticket = TicketModel::create(
            db,
            assignment.id,
            user.id,
            "Test Ticket",
            "This is a test ticket",
        )
        .await
        .unwrap();

        TestData {
            user,
            invalid_user,
            ticket,
            module,
            assignment,
        }
    }

    #[tokio::test]
    async fn set_invalid_open_ticket_test() {
        let (app, app_state, _tmp) = make_test_app_with_storage().await;
        let data = setup_test_data(app_state.db()).await;

        let (token, _) = generate_jwt(data.invalid_user.id, data.invalid_user.admin);
        let uri = format!(
            "/api/modules/{}/assignments/{}/tickets/{}/open",
            data.module.id, data.assignment.id, data.ticket.id
        );
        let req = Request::builder()
            .method("PUT")
            .uri(uri)
            .header("Authorization", format!("Bearer {}", token))
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::FORBIDDEN);
    }

    #[tokio::test]
    async fn set_close_ticket_test() {
        let (app, app_state, _tmp) = make_test_app_with_storage().await;
        let data = setup_test_data(app_state.db()).await;

        let (token, _) = generate_jwt(data.user.id, data.user.admin);
        let uri = format!(
            "/api/modules/{}/assignments/{}/tickets/{}/close",
            data.module.id, data.assignment.id, data.ticket.id
        );
        let req = Request::builder()
            .method("PUT")
            .uri(uri)
            .header("Authorization", format!("Bearer {}", token))
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();

        let response_data: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(response_data["data"]["status"], "closed");
        assert_eq!(response_data["message"], "Ticket closed successfully");
    }

    #[tokio::test]
    async fn set_invalid_close_ticket_test() {
        let (app, app_state, _tmp) = make_test_app_with_storage().await;
        let data = setup_test_data(app_state.db()).await;

        let (token, _) = generate_jwt(data.invalid_user.id, data.invalid_user.admin);
        let uri = format!(
            "/api/modules/{}/assignments/{}/tickets/{}/close",
            data.module.id, data.assignment.id, data.ticket.id
        );
        let req = Request::builder()
            .method("PUT")
            .uri(uri)
            .header("Authorization", format!("Bearer {}", token))
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::FORBIDDEN);
    }
}
