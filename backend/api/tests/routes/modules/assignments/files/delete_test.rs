#[cfg(test)]
mod tests {
    use db::{
        models::{
            user::Model as UserModel,
            module::Model as ModuleModel,
            assignment::Model as AssignmentModel,
            user_module_role::{Model as UserModuleRoleModel, Role},
            assignment_file::{Model as AssignmentFileModel, FileType}
        },
        repositories::user_repository::UserRepository,
    };
    use axum::{
        body::Body,
        http::{Request, StatusCode}
    };
    use services::{
        service::Service,
        user_service::{CreateUser, UserService}
    };
    use tower::ServiceExt;
    use serde_json::json;
    use api::auth::generate_jwt;
    use chrono::{Utc, TimeZone};
    use sea_orm::DatabaseConnection;
    use crate::helpers::app::make_test_app;

    struct TestData {
        lecturer_user: UserModel,
        student_user: UserModel,
        module: ModuleModel,
        assignment: AssignmentModel,
        file: AssignmentFileModel,
        other_file: AssignmentFileModel,
    }

    async fn setup_test_data(db: &DatabaseConnection) -> TestData {
        let module = ModuleModel::create(db, "COS101", 2024, Some("Test Module"), 16).await.unwrap();
        let service = UserService::new(UserRepository::new(db.clone()));
        let lecturer_user = service.create(CreateUser { username: "lecturer1".to_string(), email: "lecturer1@test.com".to_string(), password: "password1".to_string(), admin: false }).await.unwrap();
        let student_user = service.create(CreateUser { username: "student1".to_string(), email: "student1@test.com".to_string(), password: "password2".to_string(), admin: false }).await.unwrap();
        UserModuleRoleModel::assign_user_to_module(db, lecturer_user.id, module.id, Role::Lecturer).await.unwrap();
        UserModuleRoleModel::assign_user_to_module(db, student_user.id, module.id, Role::Student).await.unwrap();
        let assignment = AssignmentModel::create(
            db,
            module.id,
            "Assignment 1",
            Some("Desc 1"),
            db::models::assignment::AssignmentType::Assignment,
            Utc.with_ymd_and_hms(2024, 1, 1, 0, 0, 0).unwrap(),
            Utc.with_ymd_and_hms(2024, 1, 31, 23, 59, 59).unwrap(),
        ).await.unwrap();
        let other_assignment = AssignmentModel::create(
            &db,
            module.id,
            "Assignment 2",
            Some("Other assignment"),
            db::models::assignment::AssignmentType::Assignment,
            Utc.with_ymd_and_hms(2024, 2, 1, 0, 0, 0).unwrap(),
            Utc.with_ymd_and_hms(2024, 2, 28, 23, 59, 59).unwrap(),
        ).await.unwrap();
        let file = AssignmentFileModel::save_file(
            db,
            assignment.id,
            module.id,
            FileType::Spec,
            "spec.txt",
            b"spec file content",
        ).await.unwrap();
        let other_file = AssignmentFileModel::save_file(
            &db,
            other_assignment.id,
            module.id,
            FileType::Spec,
            "other_spec.txt",
            b"other file content",
        ).await.unwrap();

        TestData {
            lecturer_user,
            student_user,
            module,
            assignment,
            file,
            other_file,
        }
    }

    #[tokio::test]
    async fn test_delete_file_success_as_lecturer() {
        let (app, app_state) = make_test_app().await;
        let data = setup_test_data(app_state.db()).await;

        let (token, _) = generate_jwt(data.lecturer_user.id, data.lecturer_user.admin);
        let uri = format!("/api/modules/{}/assignments/{}/files", data.module.id, data.assignment.id);
        let req = Request::builder()
            .method("DELETE")
            .uri(&uri)
            .header("Authorization", format!("Bearer {}", token))
            .header("Content-Type", "application/json")
            .body(Body::from(json!({"file_ids": [data.file.id]}).to_string()))
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_delete_file_forbidden_for_student() {
        let (app, app_state) = make_test_app().await;
        let data = setup_test_data(app_state.db()).await;

        let (token, _) = generate_jwt(data.student_user.id, data.student_user.admin);
        let uri = format!("/api/modules/{}/assignments/{}/files", data.module.id, data.assignment.id);
        let req = Request::builder()
            .method("DELETE")
            .uri(&uri)
            .header("Authorization", format!("Bearer {}", token))
            .header("Content-Type", "application/json")
            .body(Body::from(json!({"file_ids": [data.file.id]}).to_string()))
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::FORBIDDEN);
    }

    #[tokio::test]
    async fn test_delete_file_assignment_not_found() {
        let (app, app_state) = make_test_app().await;
        let data = setup_test_data(app_state.db()).await;

        let (token, _) = generate_jwt(data.lecturer_user.id, data.lecturer_user.admin);
        let uri = format!("/api/modules/{}/assignments/{}/files", data.module.id, 9999);
        let req = Request::builder()
            .method("DELETE")
            .uri(&uri)
            .header("Authorization", format!("Bearer {}", token))
            .header("Content-Type", "application/json")
            .body(Body::from(json!({"file_ids": [data.file.id]}).to_string()))
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn test_delete_file_bad_request_empty_file_ids() {
        let (app, app_state) = make_test_app().await;
        let data = setup_test_data(app_state.db()).await;

        let (token, _) = generate_jwt(data.lecturer_user.id, data.lecturer_user.admin);
        let uri = format!("/api/modules/{}/assignments/{}/files", data.module.id, data.assignment.id);
        let req = Request::builder()
            .method("DELETE")
            .uri(&uri)
            .header("Authorization", format!("Bearer {}", token))
            .header("Content-Type", "application/json")
            .body(Body::from(json!({"file_ids": []}).to_string()))
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn test_delete_file_unauthorized() {
        let (app, app_state) = make_test_app().await;
        let data = setup_test_data(app_state.db()).await;

        let uri = format!("/api/modules/{}/assignments/{}/files", data.module.id, data.assignment.id);
        let req = Request::builder()
            .method("DELETE")
            .uri(&uri)
            .header("Content-Type", "application/json")
            .body(Body::from(json!({"file_ids": [data.file.id]}).to_string()))
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn test_delete_file_id_not_exist() {
        let (app, app_state) = make_test_app().await;
        let data = setup_test_data(app_state.db()).await;

        let (token, _) = generate_jwt(data.lecturer_user.id, data.lecturer_user.admin);
        let uri = format!("/api/modules/{}/assignments/{}/files", data.module.id, data.assignment.id);
        let req = Request::builder()
            .method("DELETE")
            .uri(&uri)
            .header("Authorization", format!("Bearer {}", token))
            .header("Content-Type", "application/json")
            .body(Body::from(json!({"file_ids": [99999]}).to_string()))
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert!(response.status() == StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn test_delete_file_id_belongs_to_another_assignment() {
        let (app, app_state) = make_test_app().await;
        let data = setup_test_data(app_state.db()).await;

        let (token, _) = generate_jwt(data.lecturer_user.id, data.lecturer_user.admin);
        let uri = format!("/api/modules/{}/assignments/{}/files", data.module.id, data.assignment.id);
        let req = Request::builder()
            .method("DELETE")
            .uri(&uri)
            .header("Authorization", format!("Bearer {}", token))
            .header("Content-Type", "application/json")
            .body(Body::from(json!({"file_ids": [data.other_file.id]}).to_string()))
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert!(response.status() == StatusCode::NOT_FOUND);
    }
}