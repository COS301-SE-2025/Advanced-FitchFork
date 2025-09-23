#[cfg(test)]
mod tests {
    use std::convert::Infallible;

    use crate::helpers::app::make_test_app_with_storage;
    use api::auth::generate_jwt;
    use axum::{
        body::Body,
        http::{Request, Response, StatusCode},
    };
    use chrono::{TimeZone, Utc};
    use db::models::{
        assignment::{AssignmentType, Model as AssignmentModel},
        module::Model as ModuleModel,
        tickets::Model as TicketModel,
        user::Model as UserModel,
        user_module_role::{Model as UserModuleRole, Role},
    };
    use serde_json::json;
    use tower::{ServiceExt, util::BoxCloneService};

    struct TestData {
        user: UserModel,
        unauthorised_user: UserModel,
        invalid_user: UserModel,
        lecturer: UserModel,
        tutor: UserModel,
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
        let lecturer =
            UserModel::create(db, "lecturer_user", "lecturer@example.com", "pass3", false)
                .await
                .unwrap();
        let tutor = UserModel::create(db, "tutor_user", "tutor@example.com", "pass4", false)
            .await
            .unwrap();
        let unauthorised_user = UserModel::create(
            db,
            "unauthorised_user",
            "unauthorised@example.com",
            "pass5",
            false,
        )
        .await
        .unwrap();

        let module = ModuleModel::create(db, "330", 2025, Some("test description"), 16)
            .await
            .unwrap();

        UserModuleRole::assign_user_to_module(db, user.id, module.id, Role::Student)
            .await
            .unwrap();
        UserModuleRole::assign_user_to_module(db, invalid_user.id, module.id, Role::Student)
            .await
            .unwrap();
        UserModuleRole::assign_user_to_module(db, lecturer.id, module.id, Role::Lecturer)
            .await
            .unwrap();
        UserModuleRole::assign_user_to_module(db, tutor.id, module.id, Role::Tutor)
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
            unauthorised_user,
            invalid_user,
            lecturer,
            tutor,
            ticket,
            module,
            assignment,
        }
    }

    async fn post_message_request(
        app: BoxCloneService<Request<Body>, Response<Body>, Infallible>,
        token: &str,
        module_id: i64,
        assignment_id: i64,
        ticket_id: i64,
        content: String,
    ) -> axum::http::Response<axum::body::Body> {
        let uri = format!(
            "/api/modules/{}/assignments/{}/tickets/{}/messages",
            module_id, assignment_id, ticket_id
        );

        let body = json!({
            "content": content,
        });

        let req = Request::builder()
            .method("POST")
            .uri(&uri)
            .header("Authorization", format!("Bearer {}", token))
            .header("Content-Type", "application/json")
            .body(Body::from(body.to_string()))
            .unwrap();

        app.clone().oneshot(req).await.unwrap()
    }

    #[tokio::test]
    async fn post_not_found_hierarchy() {
        let (app, app_state, _tmp) = make_test_app_with_storage().await;
        let data = setup_test_data(app_state.db()).await;
        let (token, _) = generate_jwt(data.user.id, data.user.admin);

        let response = post_message_request(
            app.clone(),
            &token,
            data.module.id,
            3,
            data.ticket.id,
            "Updated Message Content".to_string(),
        )
        .await;

        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn post_message_as_student() {
        let (app, app_state, _tmp) = make_test_app_with_storage().await;
        let data = setup_test_data(app_state.db()).await;
        let (token, _) = generate_jwt(data.user.id, data.user.admin);

        let response = post_message_request(
            app.clone(),
            &token,
            data.module.id,
            data.assignment.id,
            data.ticket.id,
            "Updated Message Content".to_string(),
        )
        .await;

        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn post_message_as_invalid_student() {
        let (app, app_state, _tmp) = make_test_app_with_storage().await;
        let data = setup_test_data(app_state.db()).await;
        let (token, _) = generate_jwt(data.invalid_user.id, data.invalid_user.admin);

        let response = post_message_request(
            app.clone(),
            &token,
            data.module.id,
            data.assignment.id,
            data.ticket.id,
            "Updated Message Content".to_string(),
        )
        .await;

        assert_eq!(response.status(), StatusCode::FORBIDDEN);
    }

    #[tokio::test]
    async fn post_message_as_unauthorised_user() {
        let (app, app_state, _tmp) = make_test_app_with_storage().await;
        let data = setup_test_data(app_state.db()).await;
        let (token, _) = generate_jwt(data.unauthorised_user.id, data.unauthorised_user.admin);

        let response = post_message_request(
            app.clone(),
            &token,
            data.module.id,
            data.assignment.id,
            data.ticket.id,
            "Updated Message Content".to_string(),
        )
        .await;

        assert_eq!(response.status(), StatusCode::FORBIDDEN);
    }

    #[tokio::test]
    async fn post_message_as_lecturer() {
        let (app, app_state, _tmp) = make_test_app_with_storage().await;
        let data = setup_test_data(app_state.db()).await;
        let (token, _) = generate_jwt(data.lecturer.id, data.lecturer.admin);

        let response = post_message_request(
            app.clone(),
            &token,
            data.module.id,
            data.assignment.id,
            data.ticket.id,
            "Updated Message Content".to_string(),
        )
        .await;

        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn post_message_as_tutor() {
        let (app, app_state, _tmp) = make_test_app_with_storage().await;
        let data = setup_test_data(app_state.db()).await;
        let (token, _) = generate_jwt(data.tutor.id, data.tutor.admin);

        let response = post_message_request(
            app.clone(),
            &token,
            data.module.id,
            data.assignment.id,
            data.ticket.id,
            "Updated Message Content".to_string(),
        )
        .await;

        assert_eq!(response.status(), StatusCode::OK);
    }
}
