#[cfg(test)]
mod tests {
    use crate::helpers::make_test_app;
    use api::auth::generate_jwt;
    use chrono::{TimeZone, Utc};
    use db::{
        models::{
            assignment::{AssignmentType, Model as AssignmentModel},
            module::Model as ModuleModel,
            tickets::Model as TicketModel,
            user::Model as UserModel,
            user_module_role::{Model as UserModuleRole, Role},
        },
        repositories::user_repository::UserRepository,
    };
    use axum::{
        body::Body,
        http::{Request, StatusCode},
    };
    use services::{
        service::Service,
        user_service::{CreateUser, UserService}
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
        let service = UserService::new(UserRepository::new(db.clone()));
        let user = service.create(CreateUser{ username: "test_user".to_string(), email: "test@example.com".to_string(), password: "pass".to_string(), admin: false }).await.unwrap();
        let invalid_user = service.create(CreateUser{ username: "invalid_user".to_string(), email: "invalid@email.com".to_string(), password: "pass2".to_string(), admin: false }).await.unwrap();
        let module = ModuleModel::create(db, "330", 2025, Some("test description"), 16).await.unwrap();
        UserModuleRole::assign_user_to_module(db, user.id, module.id, Role::Student).await.unwrap();
        let assignment = AssignmentModel::create(
            db,
            module.id,
            "Assignment 1",
            Some("Desc 1"),
            AssignmentType::Assignment,
            Utc.with_ymd_and_hms(2024, 1, 1, 0, 0, 0).unwrap(),
            Utc.with_ymd_and_hms(2024, 1, 31, 23, 59, 59).unwrap(),
        ).await.unwrap();
        let ticket = TicketModel::create(
            db,
            assignment.id,
            user.id,
            "Test Ticket",
            "This is a test ticket",
        ).await.unwrap();

        TestData {
            user,
            invalid_user,
            ticket,
            module,
            assignment,
        }
    }

    #[tokio::test]
    async fn delete_ticket_test() {
        let (app, app_state) = make_test_app().await;
        let data = setup_test_data(app_state.db()).await;
        let (token, _) = generate_jwt(data.user.id, data.user.admin);
        let uri = format!(
            "/api/modules/{}/assignments/{}/tickets/{}",
            data.module.id, data.assignment.id, data.ticket.id
        );
        let req = Request::builder()
            .method("DELETE")
            .uri(&uri)
            .header("Authorization", format!("Bearer {}", token))
            .body(Body::empty())
            .unwrap();
        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(json["message"], "Ticket deleted successfully");
    }

    #[tokio::test]
    async fn delete_invalid_ticket_test() {
        let (app, app_state) = make_test_app().await;
        let data = setup_test_data(app_state.db()).await;
        let (token, _) = generate_jwt(data.invalid_user.id, data.invalid_user.admin);
        let uri = format!(
            "/api/modules/{}/assignments/{}/tickets/{}",
            data.module.id, data.assignment.id, data.ticket.id
        );
        let req = Request::builder()
            .method("DELETE")
            .uri(&uri)
            .header("Authorization", format!("Bearer {}", token))
            .body(Body::empty())
            .unwrap();
        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::FORBIDDEN);
        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(json["message"], "User not assigned to this module");
    }
}
