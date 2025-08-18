#[cfg(test)]
mod tests {
    use axum::{
        body::Body,
        http::{Request, StatusCode},
    };
    use tower::ServiceExt;
    use serde_json::Value;
    use chrono::{Utc, TimeZone};
    use db::models::{
        user::Model as UserModel,
        module::Model as ModuleModel,
        assignment::Model as AssignmentModel,
        user_module_role::{Model as UserModuleRoleModel, Role},
        tickets::Model as TicketModel,
    };
    use api::auth::generate_jwt;
    use crate::helpers::app::make_test_app;

    async fn setup_test_data(db: &sea_orm::DatabaseConnection) -> (UserModel, UserModel, UserModel, ModuleModel, AssignmentModel, TicketModel) {
        let module = ModuleModel::create(db, "COS133", 2025, Some("Testing Module"), 12)
            .await
            .unwrap();

        let assignment = AssignmentModel::create(
            db,
            module.id,
            "Test Assignment",
            Some("desc"),
            db::models::assignment::AssignmentType::Assignment,
            Utc.with_ymd_and_hms(2025, 1, 1, 0, 0, 0).unwrap(),
            Utc.with_ymd_and_hms(2025, 12, 31, 23, 59, 59).unwrap(),
        )
        .await
        .unwrap();

        let author = UserModel::create(db, "student33", "student33@test.com", "pass", false)
            .await
            .unwrap();

        let lecturer = UserModel::create(db, "lect33", "lect@test.com", "pass", false)
            .await
            .unwrap();

        let outsider = UserModel::create(db, "outsider", "nope@test.com", "pass", false)
            .await
            .unwrap();

        UserModuleRoleModel::assign_user_to_module(db, author.id, module.id, Role::Student)
            .await
            .unwrap();
        UserModuleRoleModel::assign_user_to_module(db, lecturer.id, module.id, Role::Lecturer)
            .await
            .unwrap();

        let ticket = TicketModel::create(
            db,
            assignment.id,
            author.id,
            "Help me!",
            "I am confused by Q2",
        )
        .await
        .unwrap();

        (author, lecturer, outsider, module, assignment, ticket)
    }

    #[tokio::test]
    async fn test_get_ticket_with_user_success_as_admin() {
        let (app, app_state) = make_test_app().await;
        let (_author, _lecturer, _outsider, module, assignment, ticket) =
            setup_test_data(app_state.db()).await;

        let admin = UserModel::create(app_state.db(), "admin", "admin@test.com", "pass", true)
            .await
            .unwrap();

        let (token, _) = generate_jwt(admin.id, admin.admin);
        let uri = format!(
            "/api/modules/{}/assignments/{}/tickets/{}",
            module.id, assignment.id, ticket.id
        );

        let req = Request::builder()
            .uri(uri)
            .header("Authorization", format!("Bearer {}", token))
            .body(Body::empty())
            .unwrap();

        let res = app.oneshot(req).await.unwrap();
        assert_eq!(res.status(), StatusCode::OK);
    }


    #[tokio::test]
    async fn test_get_ticket_with_user_success_as_lecturer() {
        let (app, app_state) = make_test_app().await;
        let (_author, lecturer, _, module, assignment, ticket) =
            setup_test_data(app_state.db()).await;

        let (token, _) = generate_jwt(lecturer.id, lecturer.admin);
        let uri = format!(
            "/api/modules/{}/assignments/{}/tickets/{}",
            module.id, assignment.id, ticket.id
        );

        let req = Request::builder()
            .uri(uri)
            .header("Authorization", format!("Bearer {}", token))
            .body(Body::empty())
            .unwrap();

        let res = app.oneshot(req).await.unwrap();
        assert_eq!(res.status(), StatusCode::OK);

        let body = axum::body::to_bytes(res.into_body(), usize::MAX).await.unwrap();
        let json: Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(json["success"], true);
        assert_eq!(json["data"]["ticket"]["id"], ticket.id);
        assert_eq!(json["data"]["user"]["email"].as_str().unwrap().ends_with("@test.com"), true);
    }

    #[tokio::test]
    async fn test_get_ticket_with_user_success_as_assistant_lecturer() {
        let (app, app_state) = make_test_app().await;
        let (_author, _lecturer, _outsider, module, assignment, ticket) =
            setup_test_data(app_state.db()).await;

        let assistant = UserModel::create(app_state.db(), "assistant", "assistant@test.com", "pass", false)
            .await
            .unwrap();

        UserModuleRoleModel::assign_user_to_module(
            app_state.db(),
            assistant.id,
            module.id,
            Role::AssistantLecturer,
        )
        .await
        .unwrap();

        let (token, _) = generate_jwt(assistant.id, assistant.admin);
        let uri = format!(
            "/api/modules/{}/assignments/{}/tickets/{}",
            module.id, assignment.id, ticket.id
        );

        let req = Request::builder()
            .uri(uri)
            .header("Authorization", format!("Bearer {}", token))
            .body(Body::empty())
            .unwrap();

        let res = app.oneshot(req).await.unwrap();
        assert_eq!(res.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_get_ticket_with_user_success_as_tutor() {
        let (app, app_state) = make_test_app().await;
        let (_author, _lecturer, _outsider, module, assignment, ticket) =
            setup_test_data(app_state.db()).await;

        let tutor = UserModel::create(app_state.db(), "tutor", "tutor@test.com", "pass", false)
            .await
            .unwrap();

        UserModuleRoleModel::assign_user_to_module(
            app_state.db(),
            tutor.id,
            module.id,
            Role::Tutor,
        )
        .await
        .unwrap();

        let (token, _) = generate_jwt(tutor.id, tutor.admin);
        let uri = format!(
            "/api/modules/{}/assignments/{}/tickets/{}",
            module.id, assignment.id, ticket.id
        );

        let req = Request::builder()
            .uri(uri)
            .header("Authorization", format!("Bearer {}", token))
            .body(Body::empty())
            .unwrap();

        let res = app.oneshot(req).await.unwrap();
        assert_eq!(res.status(), StatusCode::OK);
    }


    #[tokio::test]
    async fn test_get_ticket_with_user_success_as_author() {
        let (app, app_state) = make_test_app().await;
        let (author, _, _, module, assignment, ticket) =
            setup_test_data(app_state.db()).await;

        let (token, _) = generate_jwt(author.id, author.admin);
        let uri = format!(
            "/api/modules/{}/assignments/{}/tickets/{}",
            module.id, assignment.id, ticket.id
        );

        let req = Request::builder()
            .uri(uri)
            .header("Authorization", format!("Bearer {}", token))
            .body(Body::empty())
            .unwrap();

        let res = app.oneshot(req).await.unwrap();
        assert_eq!(res.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_get_ticket_with_user_forbidden() {
        let (app, app_state) = make_test_app().await;
        let (_, _, outsider, module, assignment, ticket) =
            setup_test_data(app_state.db()).await;

        let (token, _) = generate_jwt(outsider.id, outsider.admin);
        let uri = format!(
            "/api/modules/{}/assignments/{}/tickets/{}",
            module.id, assignment.id, ticket.id
        );

        let req = Request::builder()
            .uri(uri)
            .header("Authorization", format!("Bearer {}", token))
            .body(Body::empty())
            .unwrap();

        let res = app.oneshot(req).await.unwrap();
        assert_eq!(res.status(), StatusCode::FORBIDDEN);
    }

    #[tokio::test]
    async fn test_get_ticket_with_user_not_found() {
        let (app, app_state) = make_test_app().await;
        let (_author, lecturer, _, module, assignment, _) =
            setup_test_data(app_state.db()).await;

        let (token, _) = generate_jwt(lecturer.id, lecturer.admin);
        let uri = format!(
            "/api/modules/{}/assignments/{}/tickets/{}",
            module.id, assignment.id, 99999
        );

        let req = Request::builder()
            .uri(uri)
            .header("Authorization", format!("Bearer {}", token))
            .body(Body::empty())
            .unwrap();

        let res = app.oneshot(req).await.unwrap();
        assert_eq!(res.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn test_get_ticket_with_user_unauthorized() {
        let (app, app_state) = make_test_app().await;
        let (_, _, _, module, assignment, ticket) =
            setup_test_data(app_state.db()).await;

        let uri = format!(
            "/api/modules/{}/assignments/{}/tickets/{}",
            module.id, assignment.id, ticket.id
        );

        let req = Request::builder()
            .uri(uri)
            .body(Body::empty())
            .unwrap();

        let res = app.oneshot(req).await.unwrap();
        assert_eq!(res.status(), StatusCode::UNAUTHORIZED);
    }

        #[tokio::test]
    async fn test_get_ticket_with_user_student_not_author_forbidden() {
        let (app, app_state) = make_test_app().await;
        let (_, _, _, module, assignment, ticket) =
            setup_test_data(app_state.db()).await;

        // Create second student in same module who is NOT the ticket author
        let another_student = UserModel::create(app_state.db(), "studentB", "sb@test.com", "pass", false)
            .await
            .unwrap();

        UserModuleRoleModel::assign_user_to_module(
            app_state.db(),
            another_student.id,
            module.id,
            Role::Student,
        )
        .await
        .unwrap();

        let (token, _) = generate_jwt(another_student.id, another_student.admin);
        let uri = format!(
            "/api/modules/{}/assignments/{}/tickets/{}",
            module.id, assignment.id, ticket.id
        );

        let req = Request::builder()
            .uri(uri)
            .header("Authorization", format!("Bearer {}", token))
            .body(Body::empty())
            .unwrap();

        let res = app.oneshot(req).await.unwrap();
        assert_eq!(res.status(), StatusCode::FORBIDDEN);
    }

}