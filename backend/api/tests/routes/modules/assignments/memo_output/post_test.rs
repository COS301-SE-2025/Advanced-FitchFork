#[cfg(test)]
mod tests {
    use db::{test_utils::setup_test_db, models::{user::Model as UserModel, module::Model as ModuleModel, assignment::Model as AssignmentModel, user_module_role::{Model as UserModuleRoleModel, Role}}};
    use axum::{body::Body, http::{Request, StatusCode}};
    use tower::ServiceExt;
    use api::{routes::routes, auth::generate_jwt};
    use dotenvy;
    use chrono::{Utc, TimeZone};
    use std::{fs, path::PathBuf};
    use serial_test::serial;

    struct TestData {
        admin_user: UserModel,
        lecturer_user: UserModel,
        student_user: UserModel,
        forbidden_user: UserModel,
        module: ModuleModel,
        assignment: AssignmentModel,
    }

    async fn setup_test_data(db: &sea_orm::DatabaseConnection) -> TestData {
        let module = ModuleModel::create(db, "COS101", 2024, Some("Test Module"), 16).await.unwrap();
        let admin_user = UserModel::create(db, "admin1", "admin1@test.com", "password", true).await.unwrap();
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
            Some(db::models::assignment::Status::Setup),
        ).await.unwrap();
        
        TestData {
            admin_user,
            lecturer_user,
            student_user,
            forbidden_user,
            module,
            assignment,
        }
    }

    fn cleanup_tmp() {
        let _ = fs::remove_dir_all("./tmp");
    }

    fn create_multipart_body(file_name: &str, file_content: &str) -> (Body, String) {
        let boundary = "------------------------_boundary";
        let body = format!(
            "--{boundary}\r\n\
            Content-Disposition: form-data; name=\"files\"; filename=\"{}\"\r\n\
            Content-Type: text/plain\r\n\
            \r\n\
            {}\r\n\
            --{boundary}--\r\n",
            file_name, file_content
        );
        let content_type = format!("multipart/form-data; boundary={}", boundary);
        (Body::from(body), content_type)
    }

    #[tokio::test]
    #[serial]
    async fn test_post_memo_output_success_as_lecturer() {
        dotenvy::dotenv().expect("Failed to load .env");
        unsafe { std::env::set_var("ASSIGNMENT_STORAGE_ROOT", "./tmp"); }
        let db = setup_test_db().await;
        let data = setup_test_data(&db).await;

        let (body, content_type) = create_multipart_body("1.txt", "Memo for task 1");
        let app = axum::Router::new().nest("/api", routes(db.clone())).with_state(db.clone());
        let (token, _) = generate_jwt(data.lecturer_user.id, data.lecturer_user.admin);
        let uri = format!("/api/modules/{}/assignments/{}/memo_output", data.module.id, data.assignment.id);
        
        let req = Request::builder()
            .method("POST")
            .uri(&uri)
            .header("Authorization", format!("Bearer {}", token))
            .header("Content-Type", &content_type)
            .body(body)
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);

        let memo_path = PathBuf::from("./tmp")
            .join(format!("module_{}", data.module.id))
            .join(format!("assignment_{}", data.assignment.id))
            .join("memo_output")
            .join("1.txt");
        assert!(memo_path.exists());
        assert_eq!(fs::read_to_string(memo_path).unwrap(), "Memo for task 1");

        cleanup_tmp();
    }

    #[tokio::test]
    #[serial]
    async fn test_post_memo_output_success_as_admin() {
        dotenvy::dotenv().expect("Failed to load .env");
        unsafe { std::env::set_var("ASSIGNMENT_STORAGE_ROOT", "./tmp"); }
        let db = setup_test_db().await;
        let data = setup_test_data(&db).await;

        let (body, content_type) = create_multipart_body("1.txt", "Admin memo");
        let app = axum::Router::new().nest("/api", routes(db.clone())).with_state(db.clone());
        let (token, _) = generate_jwt(data.admin_user.id, data.admin_user.admin);
        let uri = format!("/api/modules/{}/assignments/{}/memo_output", data.module.id, data.assignment.id);

        let req = Request::builder()
            .method("POST")
            .uri(&uri)
            .header("Authorization", format!("Bearer {}", token))
            .header("Content-Type", &content_type)
            .body(body)
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);

        cleanup_tmp();
    }
    
    #[tokio::test]
    #[serial]
    async fn test_post_memo_output_forbidden_for_student() {
        dotenvy::dotenv().expect("Failed to load .env");
        let db = setup_test_db().await;
        let data = setup_test_data(&db).await;

        let (body, content_type) = create_multipart_body("1.txt", "Student memo");
        let app = axum::Router::new().nest("/api", routes(db.clone())).with_state(db.clone());
        let (token, _) = generate_jwt(data.student_user.id, data.student_user.admin);
        let uri = format!("/api/modules/{}/assignments/{}/memo_output", data.module.id, data.assignment.id);

        let req = Request::builder()
            .method("POST")
            .uri(&uri)
            .header("Authorization", format!("Bearer {}", token))
            .header("Content-Type", &content_type)
            .body(body)
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::FORBIDDEN);
    }
    
    #[tokio::test]
    #[serial]
    async fn test_post_memo_output_forbidden_for_unassigned_user() {
        dotenvy::dotenv().expect("Failed to load .env");
        let db = setup_test_db().await;
        let data = setup_test_data(&db).await;

        let (body, content_type) = create_multipart_body("1.txt", "Forbidden memo");
        let app = axum::Router::new().nest("/api", routes(db.clone())).with_state(db.clone());
        let (token, _) = generate_jwt(data.forbidden_user.id, data.forbidden_user.admin);
        let uri = format!("/api/modules/{}/assignments/{}/memo_output", data.module.id, data.assignment.id);

        let req = Request::builder()
            .method("POST")
            .uri(&uri)
            .header("Authorization", format!("Bearer {}", token))
            .header("Content-Type", &content_type)
            .body(body)
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::FORBIDDEN);
    }

    #[tokio::test]
    #[serial]
    async fn test_post_memo_output_unauthorized() {
        dotenvy::dotenv().expect("Failed to load .env");
        let db = setup_test_db().await;
        let data = setup_test_data(&db).await;

        let (body, content_type) = create_multipart_body("1.txt", "Unauthorized memo");
        let app = axum::Router::new().nest("/api", routes(db.clone())).with_state(db.clone());
        let uri = format!("/api/modules/{}/assignments/{}/memo_output", data.module.id, data.assignment.id);

        let req = Request::builder()
            .method("POST")
            .uri(&uri)
            .header("Content-Type", &content_type)
            .body(body)
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }
    
    #[tokio::test]
    #[serial]
    async fn test_post_memo_output_assignment_not_found() {
        dotenvy::dotenv().expect("Failed to load .env");
        let db = setup_test_db().await;
        let data = setup_test_data(&db).await;

        let (body, content_type) = create_multipart_body("1.txt", "Bad memo");
        let app = axum::Router::new().nest("/api", routes(db.clone())).with_state(db.clone());
        let (token, _) = generate_jwt(data.lecturer_user.id, data.lecturer_user.admin);
        let uri = format!("/api/modules/{}/assignments/{}/memo_output", data.module.id, 9999);

        let req = Request::builder()
            .method("POST")
            .uri(&uri)
            .header("Authorization", format!("Bearer {}", token))
            .header("Content-Type", &content_type)
            .body(body)
            .unwrap();
        
        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    #[serial]
    async fn test_post_memo_output_bad_request_on_invalid_filename() {
        dotenvy::dotenv().expect("Failed to load .env");
        unsafe { std::env::set_var("ASSIGNMENT_STORAGE_ROOT", "./tmp"); }
        let db = setup_test_db().await;
        let data = setup_test_data(&db).await;

        let (body, content_type) = create_multipart_body("invalid.txt", "Invalid");
        let app = axum::Router::new().nest("/api", routes(db.clone())).with_state(db.clone());
        let (token, _) = generate_jwt(data.lecturer_user.id, data.lecturer_user.admin);
        let uri = format!("/api/modules/{}/assignments/{}/memo_output", data.module.id, data.assignment.id);
        
        let req = Request::builder()
            .method("POST")
            .uri(&uri)
            .header("Authorization", format!("Bearer {}", token))
            .header("Content-Type", &content_type)
            .body(body)
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);

        cleanup_tmp();
    }
} 
