#[cfg(test)]
mod tests {
    use db::{
        models::{
            user::Model as UserModel,
            module::Model as ModuleModel,
            assignment::Model as AssignmentModel,
            user_module_role::{Model as UserModuleRoleModel, Role}
        },
        repositories::user_repository::UserRepository,
    };
    use services::{
        service::Service,
        user::{CreateUser, UserService},
    };
    use axum::{body::Body, http::{Request, StatusCode}};
    use tower::ServiceExt;
    use api::auth::generate_jwt;
    use chrono::{Utc, TimeZone};
    use sea_orm::{DatabaseConnection, EntityTrait, ColumnTrait, QueryFilter};
    use axum::http::header::{CONTENT_TYPE, AUTHORIZATION};
    use crate::helpers::app::make_test_app_with_storage;

    struct TestData {
        lecturer_user: UserModel,
        student_user: UserModel,
        module: ModuleModel,
        assignment: AssignmentModel,
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
        
        TestData {
            lecturer_user,
            student_user,
            module,
            assignment,
        }
    }

    fn multipart_body(file_type: &str, filename: &str, file_content: &[u8]) -> (String, Vec<u8>) {
        let boundary = "----BoundaryTest".to_string();
        let mut body = Vec::new();
        body.extend(format!("--{}\r\n", boundary).as_bytes());
        body.extend(format!("Content-Disposition: form-data; name=\"file_type\"\r\n\r\n{}\r\n", file_type).as_bytes());
        body.extend(format!("--{}\r\n", boundary).as_bytes());
        body.extend(format!("Content-Disposition: form-data; name=\"file\"; filename=\"{}\"\r\nContent-Type: application/octet-stream\r\n\r\n", filename).as_bytes());
        body.extend(file_content);
        body.extend(b"\r\n");
        body.extend(format!("--{}--\r\n", boundary).as_bytes());
        (boundary, body)
    }

    #[tokio::test]
    #[serial]
    async fn test_upload_file_success_as_lecturer() {
        let (app, app_state, _tmp) = make_test_app_with_storage().await;
        let data = setup_test_data(app_state.db()).await;

        let (boundary, body) = multipart_body("spec", "spec.txt", b"spec file content");
        let (token, _) = generate_jwt(data.lecturer_user.id, data.lecturer_user.admin);
        let uri = format!("/api/modules/{}/assignments/{}/files", data.module.id, data.assignment.id);
        let req = Request::builder()
            .method("POST")
            .uri(&uri)
            .header(AUTHORIZATION, format!("Bearer {}", token))
            .header(CONTENT_TYPE, format!("multipart/form-data; boundary={}", boundary))
            .body(Body::from(body))
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::CREATED);
    }

    #[tokio::test]
    #[serial]
    async fn test_upload_file_forbidden_for_student() {
        let (app, app_state, _tmp) = make_test_app_with_storage().await;
        let data = setup_test_data(app_state.db()).await;

        let (boundary, body) = multipart_body("spec", "spec.txt", b"spec file content");
        let (token, _) = generate_jwt(data.student_user.id, data.student_user.admin);
        let uri = format!("/api/modules/{}/assignments/{}/files", data.module.id, data.assignment.id);
        let req = Request::builder()
            .method("POST")
            .uri(&uri)
            .header(AUTHORIZATION, format!("Bearer {}", token))
            .header(CONTENT_TYPE, format!("multipart/form-data; boundary={}", boundary))
            .body(Body::from(body))
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::FORBIDDEN);
    }

    #[tokio::test]
    #[serial]
    async fn test_upload_file_assignment_not_found() {
        let (app, app_state, _tmp) = make_test_app_with_storage().await;
        let data = setup_test_data(app_state.db()).await;

        let (boundary, body) = multipart_body("spec", "spec.txt", b"spec file content");
        let (token, _) = generate_jwt(data.lecturer_user.id, data.lecturer_user.admin);
        let uri = format!("/api/modules/{}/assignments/{}/files", data.module.id, 9999);
        let req = Request::builder()
            .method("POST")
            .uri(&uri)
            .header(AUTHORIZATION, format!("Bearer {}", token))
            .header(CONTENT_TYPE, format!("multipart/form-data; boundary={}", boundary))
            .body(Body::from(body))
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    #[serial]
    async fn test_upload_file_missing_file_type() {
        let (app, app_state, _tmp) = make_test_app_with_storage().await;
        let data = setup_test_data(app_state.db()).await;

        let boundary = "----BoundaryTest".to_string();
        let mut body = Vec::new();
        body.extend(format!("--{}\r\n", boundary).as_bytes());
        body.extend(format!("Content-Disposition: form-data; name=\"file\"; filename=\"spec.txt\"\r\nContent-Type: application/octet-stream\r\n\r\nspec file content\r\n").as_bytes());
        body.extend(format!("--{}--\r\n", boundary).as_bytes());
        let (token, _) = generate_jwt(data.lecturer_user.id, data.lecturer_user.admin);
        let uri = format!("/api/modules/{}/assignments/{}/files", data.module.id, data.assignment.id);
        let req = Request::builder()
            .method("POST")
            .uri(&uri)
            .header(AUTHORIZATION, format!("Bearer {}", token))
            .header(CONTENT_TYPE, format!("multipart/form-data; boundary={}", boundary))
            .body(Body::from(body))
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    #[serial]
    async fn test_upload_file_empty_file() {
        let (app, app_state, _tmp) = make_test_app_with_storage().await;
        let data = setup_test_data(app_state.db()).await;

        let (boundary, body) = multipart_body("spec", "spec.txt", b"");
        let (token, _) = generate_jwt(data.lecturer_user.id, data.lecturer_user.admin);
        let uri = format!("/api/modules/{}/assignments/{}/files", data.module.id, data.assignment.id);
        let req = Request::builder()
            .method("POST")
            .uri(&uri)
            .header(AUTHORIZATION, format!("Bearer {}", token))
            .header(CONTENT_TYPE, format!("multipart/form-data; boundary={}", boundary))
            .body(Body::from(body))
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    #[serial]
    async fn test_upload_file_unauthorized() {
        let (app, app_state, _tmp) = make_test_app_with_storage().await;
        let data = setup_test_data(app_state.db()).await;

        let (boundary, body) = multipart_body("spec", "spec.txt", b"spec file content");
        let uri = format!("/api/modules/{}/assignments/{}/files", data.module.id, data.assignment.id);
        let req = Request::builder()
            .method("POST")
            .uri(&uri)
            .header(CONTENT_TYPE, format!("multipart/form-data; boundary={}", boundary))
            .body(Body::from(body))
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    #[serial]
    async fn test_upload_file_invalid_file_type() {
        let (app, app_state, _tmp) = make_test_app_with_storage().await;
        let data = setup_test_data(app_state.db()).await;

        let (boundary, body) = multipart_body("not_a_type", "spec.txt", b"spec file content");
        let (token, _) = generate_jwt(data.lecturer_user.id, data.lecturer_user.admin);
        let uri = format!("/api/modules/{}/assignments/{}/files", data.module.id, data.assignment.id);
        let req = Request::builder()
            .method("POST")
            .uri(&uri)
            .header(AUTHORIZATION, format!("Bearer {}", token))
            .header(CONTENT_TYPE, format!("multipart/form-data; boundary={}", boundary))
            .body(Body::from(body))
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    #[serial]
    async fn test_upload_file_duplicate_file_type_replaces() {
        let (app, app_state, _tmp) = make_test_app_with_storage().await;
        let data = setup_test_data(app_state.db()).await;

        let (token, _) = generate_jwt(data.lecturer_user.id, data.lecturer_user.admin);
        let uri = format!("/api/modules/{}/assignments/{}/files", data.module.id, data.assignment.id);
        let (boundary1, body1) = multipart_body("spec", "spec1.txt", b"first content");
        let req1 = Request::builder()
            .method("POST")
            .uri(&uri)
            .header(AUTHORIZATION, format!("Bearer {}", token))
            .header(CONTENT_TYPE, format!("multipart/form-data; boundary={}", boundary1))
            .body(Body::from(body1))
            .unwrap();

        let response1 = app.clone().oneshot(req1).await.unwrap();
        assert_eq!(response1.status(), StatusCode::CREATED);

        let (boundary2, body2) = multipart_body("spec", "spec2.txt", b"second content");
        let req2 = Request::builder()
            .method("POST")
            .uri(&uri)
            .header(AUTHORIZATION, format!("Bearer {}", token))
            .header(CONTENT_TYPE, format!("multipart/form-data; boundary={}", boundary2))
            .body(Body::from(body2))
            .unwrap();

        let response2 = app.clone().oneshot(req2).await.unwrap();
        assert_eq!(response2.status(), StatusCode::CREATED);

        let files = db::models::assignment_file::Entity::find()
            .filter(db::models::assignment_file::Column::AssignmentId.eq(data.assignment.id))
            .filter(db::models::assignment_file::Column::FileType.eq(db::models::assignment_file::FileType::Spec))
            .all(db::get_connection().await)
            .await
            .unwrap();
        assert_eq!(files.len(), 1);
        assert_eq!(files[0].filename, "spec2.txt");
    }
}