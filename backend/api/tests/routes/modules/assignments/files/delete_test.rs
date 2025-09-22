#[cfg(test)]
mod tests {
    use crate::helpers::app::make_test_app_with_storage;
    use api::auth::generate_jwt;
    use axum::{
        body::Body,
        http::{Request, StatusCode},
    };
    use chrono::{TimeZone, Utc};
    use db::{
        models::{
            assignment::Model as AssignmentModel,
            assignment_file::{FileType, Model as AssignmentFileModel},
            module::Model as ModuleModel,
            user::Model as UserModel,
            user_module_role::{Model as UserModuleRoleModel, Role},
        },
        repositories::user_repository::UserRepository,
    };
    use sea_orm::DatabaseConnection;
    use serde_json::json;
    use services::{
        service::Service,
        user::{CreateUser, UserService},
    };
    use tower::ServiceExt;

    struct TestData {
        lecturer_user: UserModel,
        student_user: UserModel,
        module: ModuleModel,
        assignment: AssignmentModel,
        file: AssignmentFileModel,
        other_file: AssignmentFileModel,
    }

    async fn setup_test_data(db: &DatabaseConnection) -> TestData {
        let module = ModuleModel::create(db, "COS101", 2024, Some("Test Module"), 16)
            .await
            .unwrap();
        let service = UserService::new(UserRepository::new(db.clone()));
        let lecturer_user = service
            .create(CreateUser {
                username: "lecturer1".to_string(),
                email: "lecturer1@test.com".to_string(),
                password: "password1".to_string(),
                admin: false,
            })
            .await
            .unwrap();
        let student_user = service
            .create(CreateUser {
                username: "student1".to_string(),
                email: "student1@test.com".to_string(),
                password: "password2".to_string(),
                admin: false,
            })
            .await
            .unwrap();
        UserModuleRoleModel::assign_user_to_module(db, lecturer_user.id, module.id, Role::Lecturer)
            .await
            .unwrap();
        UserModuleRoleModel::assign_user_to_module(db, student_user.id, module.id, Role::Student)
            .await
            .unwrap();
        let assignment = AssignmentModel::create(
            db,
            module.id,
            "Assignment 1",
            Some("Desc 1"),
            db::models::assignment::AssignmentType::Assignment,
            Utc.with_ymd_and_hms(2024, 1, 1, 0, 0, 0).unwrap(),
            Utc.with_ymd_and_hms(2024, 1, 31, 23, 59, 59).unwrap(),
        )
        .await
        .unwrap();
        let other_assignment = AssignmentModel::create(
            &db,
            module.id,
            "Assignment 2",
            Some("Other assignment"),
            db::models::assignment::AssignmentType::Assignment,
            Utc.with_ymd_and_hms(2024, 2, 1, 0, 0, 0).unwrap(),
            Utc.with_ymd_and_hms(2024, 2, 28, 23, 59, 59).unwrap(),
        )
        .await
        .unwrap();
        let file = AssignmentFileModel::save_file(
            db,
            assignment.id,
            module.id,
            FileType::Spec,
            "spec.txt",
            b"spec file content",
        )
        .await
        .unwrap();
        let other_file = AssignmentFileModel::save_file(
            &db,
            other_assignment.id,
            module.id,
            FileType::Spec,
            "other_spec.txt",
            b"other file content",
        )
        .await
        .unwrap();

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
    #[serial]
    async fn test_delete_file_success_as_lecturer() {
        let (app, app_state, _tmp) = make_test_app_with_storage().await;
        let data = setup_test_data(app_state.db()).await;

        let (token, _) = generate_jwt(data.lecturer_user.id, data.lecturer_user.admin);
        let uri = format!(
            "/api/modules/{}/assignments/{}/files",
            data.module.id, data.assignment.id
        );
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
    #[serial]
    async fn test_delete_file_forbidden_for_student() {
        let (app, app_state, _tmp) = make_test_app_with_storage().await;
        let data = setup_test_data(app_state.db()).await;

        let (token, _) = generate_jwt(data.student_user.id, data.student_user.admin);
        let uri = format!(
            "/api/modules/{}/assignments/{}/files",
            data.module.id, data.assignment.id
        );
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
    #[serial]
    async fn test_delete_file_assignment_not_found() {
        let (app, app_state, _tmp) = make_test_app_with_storage().await;
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
    #[serial]
    async fn test_delete_file_bad_request_empty_file_ids() {
        let (app, app_state, _tmp) = make_test_app_with_storage().await;
        let data = setup_test_data(app_state.db()).await;

        let (token, _) = generate_jwt(data.lecturer_user.id, data.lecturer_user.admin);
        let uri = format!(
            "/api/modules/{}/assignments/{}/files",
            data.module.id, data.assignment.id
        );
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
    #[serial]
    async fn test_delete_file_unauthorized() {
        let (app, app_state, _tmp) = make_test_app_with_storage().await;
        let data = setup_test_data(app_state.db()).await;

        let uri = format!(
            "/api/modules/{}/assignments/{}/files",
            data.module.id, data.assignment.id
        );
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
    #[serial]
    async fn test_delete_file_id_not_exist() {
        let (app, app_state, _tmp) = make_test_app_with_storage().await;
        let data = setup_test_data(app_state.db()).await;

        let (token, _) = generate_jwt(data.lecturer_user.id, data.lecturer_user.admin);
        let uri = format!(
            "/api/modules/{}/assignments/{}/files",
            data.module.id, data.assignment.id
        );
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
    #[serial]
    async fn test_delete_file_id_belongs_to_another_assignment() {
        let (app, app_state, _tmp) = make_test_app_with_storage().await;
        let data = setup_test_data(app_state.db()).await;

        let (token, _) = generate_jwt(data.lecturer_user.id, data.lecturer_user.admin);
        let uri = format!(
            "/api/modules/{}/assignments/{}/files",
            data.module.id, data.assignment.id
        );
        let req = Request::builder()
            .method("DELETE")
            .uri(&uri)
            .header("Authorization", format!("Bearer {}", token))
            .header("Content-Type", "application/json")
            .body(Body::from(
                json!({"file_ids": [data.other_file.id]}).to_string(),
            ))
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert!(response.status() == StatusCode::NOT_FOUND);
    }
}
