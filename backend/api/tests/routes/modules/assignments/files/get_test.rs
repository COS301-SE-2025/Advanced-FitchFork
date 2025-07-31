#[cfg(test)]
mod tests {
    use db::{test_utils::setup_test_db, models::{user::Model as UserModel, module::Model as ModuleModel, assignment::Model as AssignmentModel, user_module_role::{Model as UserModuleRoleModel, Role}, assignment_file::{Model as AssignmentFileModel, FileType}}};
    use axum::{body::Body, http::{Request, StatusCode}};
    use tower::ServiceExt;
    use serde_json::Value;
    use api::auth::generate_jwt;
    use dotenvy;
    use chrono::{Utc, TimeZone};
    use sea_orm::DatabaseConnection;
    use crate::test_helpers::make_app;

    struct TestData {
        lecturer_user: UserModel,
        student_user: UserModel,
        forbidden_user: UserModel,
        module: ModuleModel,
        assignment: AssignmentModel,
        empty_assignment: AssignmentModel,
        file: AssignmentFileModel,
    }

    async fn setup_test_data(db: &DatabaseConnection) -> TestData {
        let module = ModuleModel::create(db, "COS101", 2024, Some("Test Module"), 16).await.unwrap();
        let lecturer_user = UserModel::create(db, "lecturer1", "lecturer1@test.com", "password1", false).await.unwrap();
        let student_user = UserModel::create(db, "student1", "student1@test.com", "password2", false).await.unwrap();
        let forbidden_user = UserModel::create(db, "forbidden", "forbidden@test.com", "password3", false).await.unwrap();
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
        let empty_assignment = AssignmentModel::create(
            &db,
            module.id,
            "Assignment 2",
            Some("No files"),
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

        TestData {
            lecturer_user,
            student_user,
            forbidden_user,
            module,
            assignment,
            empty_assignment,
            file,
        }
    }

    #[tokio::test]
    async fn test_list_files_success_as_lecturer() {
        dotenvy::dotenv().expect("Failed to load .env");
        let db = setup_test_db().await;
        let data = setup_test_data(&db).await;

        let app = make_app(db.clone());
        let (token, _) = generate_jwt(data.lecturer_user.id, data.lecturer_user.admin);
        let uri = format!("/api/modules/{}/assignments/{}/files", data.module.id, data.assignment.id);
        let req = Request::builder()
            .uri(&uri)
            .header("Authorization", format!("Bearer {}", token))
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let json: Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(json["success"], true);
        assert_eq!(json["data"].as_array().unwrap().len(), 1);
        assert_eq!(json["data"][0]["filename"], "spec.txt");
    }

    #[tokio::test]
    async fn test_list_files_success_as_student() {
        dotenvy::dotenv().expect("Failed to load .env");
        let db = setup_test_db().await;
        let data = setup_test_data(&db).await;

        let app = make_app(db.clone());
        let (token, _) = generate_jwt(data.student_user.id, data.student_user.admin);
        let uri = format!("/api/modules/{}/assignments/{}/files", data.module.id, data.assignment.id);
        let req = Request::builder()
            .uri(&uri)
            .header("Authorization", format!("Bearer {}", token))
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_list_files_forbidden_for_unassigned_user() {
        dotenvy::dotenv().expect("Failed to load .env");
        let db = setup_test_db().await;
        let data = setup_test_data(&db).await;

        let app = make_app(db.clone());
        let (token, _) = generate_jwt(data.forbidden_user.id, data.forbidden_user.admin);
        let uri = format!("/api/modules/{}/assignments/{}/files", data.module.id, data.assignment.id);
        let req = Request::builder()
            .uri(&uri)
            .header("Authorization", format!("Bearer {}", token))
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::FORBIDDEN);
    }

    #[tokio::test]
    async fn test_list_files_assignment_not_found() {
        dotenvy::dotenv().expect("Failed to load .env");
        let db = setup_test_db().await;
        let data = setup_test_data(&db).await;

        let app = make_app(db.clone());
        let (token, _) = generate_jwt(data.lecturer_user.id, data.lecturer_user.admin);
        let uri = format!("/api/modules/{}/assignments/{}/files", data.module.id, 9999);
        let req = Request::builder()
            .uri(&uri)
            .header("Authorization", format!("Bearer {}", token))
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn test_list_files_unauthorized() {
        dotenvy::dotenv().expect("Failed to load .env");
        let db = setup_test_db().await;
        let data = setup_test_data(&db).await;

        let app = make_app(db.clone());
        let uri = format!("/api/modules/{}/assignments/{}/files", data.module.id, data.assignment.id);
        let req = Request::builder()
            .uri(&uri)
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn test_download_file_success_as_lecturer() {
        dotenvy::dotenv().expect("Failed to load .env");
        let db = setup_test_db().await;
        let data = setup_test_data(&db).await;

        let app = make_app(db.clone());
        let (token, _) = generate_jwt(data.lecturer_user.id, data.lecturer_user.admin);
        let uri = format!("/api/modules/{}/assignments/{}/files/{}", data.module.id, data.assignment.id, data.file.id);
        let req = Request::builder()
            .uri(&uri)
            .header("Authorization", format!("Bearer {}", token))
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_download_file_not_found() {
        dotenvy::dotenv().expect("Failed to load .env");
        let db = setup_test_db().await;
        let data = setup_test_data(&db).await;

        let app = make_app(db.clone());
        let (token, _) = generate_jwt(data.lecturer_user.id, data.lecturer_user.admin);
        let uri = format!("/api/modules/{}/assignments/{}/files/9999", data.module.id, data.assignment.id);
        let req = Request::builder()
            .uri(&uri)
            .header("Authorization", format!("Bearer {}", token))
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn test_download_file_forbidden_for_unassigned_user() {
        dotenvy::dotenv().expect("Failed to load .env");
        let db = setup_test_db().await;
        let data = setup_test_data(&db).await;

        let app = make_app(db.clone());
        let (token, _) = generate_jwt(data.forbidden_user.id, data.forbidden_user.admin);
        let uri = format!("/api/modules/{}/assignments/{}/files/{}", data.module.id, data.assignment.id, data.file.id);
        let req = Request::builder()
            .uri(&uri)
            .header("Authorization", format!("Bearer {}", token))
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::FORBIDDEN);
    }

    #[tokio::test]
    async fn test_download_file_unauthorized() {
        dotenvy::dotenv().expect("Failed to load .env");
        let db = setup_test_db().await;
        let data = setup_test_data(&db).await;

        let app = make_app(db.clone());
        let uri = format!("/api/modules/{}/assignments/{}/files/{}", data.module.id, data.assignment.id, data.file.id);
        let req = Request::builder()
            .uri(&uri)
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn test_list_files_empty_assignment() {
        dotenvy::dotenv().expect("Failed to load .env");
        let db = setup_test_db().await;
        let data = setup_test_data(&db).await;

        let app = make_app(db.clone());
        let (token, _) = generate_jwt(data.lecturer_user.id, data.lecturer_user.admin);
        let uri = format!("/api/modules/{}/assignments/{}/files", data.module.id, data.empty_assignment.id);
        let req = Request::builder()
            .uri(&uri)
            .header("Authorization", format!("Bearer {}", token))
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let json: Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(json["success"], true);
        assert_eq!(json["data"].as_array().unwrap().len(), 0);
    }

    #[tokio::test]
    async fn test_download_file_wrong_assignment() {
        dotenvy::dotenv().expect("Failed to load .env");
        let db = setup_test_db().await;
        let data = setup_test_data(&db).await;

        let app = make_app(db.clone());
        let (token, _) = generate_jwt(data.lecturer_user.id, data.lecturer_user.admin);
        let uri = format!("/api/modules/{}/assignments/{}/files/{}", data.module.id, data.empty_assignment.id, data.file.id);
        let req = Request::builder()
            .uri(&uri)
            .header("Authorization", format!("Bearer {}", token))
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }
}