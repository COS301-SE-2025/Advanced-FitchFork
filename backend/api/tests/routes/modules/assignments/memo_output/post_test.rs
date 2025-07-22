#[cfg(test)]
mod tests {
    use db::{test_utils::setup_test_db, models::{user::Model as UserModel, module::Model as ModuleModel, assignment::Model as AssignmentModel, user_module_role::{Model as UserModuleRoleModel, Role}}};
    use axum::{body::{to_bytes, Body}, http::{Request, StatusCode}};
    use tower::ServiceExt;
    use api::auth::generate_jwt;
    use dotenvy;
    use chrono::{Utc, TimeZone};
    use std::{fs, path::PathBuf};
    use serial_test::serial;
    use crate::test_helpers::make_app;

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

    fn setup_input_dirs(module_id: i64, assignment_id: i64) {
        let base = PathBuf::from("./tmp")
            .join(format!("module_{}", module_id))
            .join(format!("assignment_{}", assignment_id));
        // memo dir
        let memo_dir = base.join("memo");
        fs::create_dir_all(&memo_dir).unwrap();
        fs::write(memo_dir.join("stub.txt"), "stub memo").unwrap();
        // config dir
        let config_dir = base.join("config");
        fs::create_dir_all(&config_dir).unwrap();
        fs::write(config_dir.join("stub.conf"), "stub config").unwrap();
    }

    fn cleanup_tmp() {
        let _ = fs::remove_dir_all("./tmp");
    }

    #[tokio::test]
    #[serial]
    async fn test_post_memo_output_success_as_lecturer() {
        dotenvy::dotenv().expect("Failed to load .env");
        unsafe { std::env::set_var("ASSIGNMENT_STORAGE_ROOT", "./tmp"); }
        let db = setup_test_db().await;
        let data = setup_test_data(&db).await;
        setup_input_dirs(data.module.id, data.assignment.id);

        let app = make_app(db.clone());
        let (token, _) = generate_jwt(data.lecturer_user.id, data.lecturer_user.admin);
        let uri = format!("/api/modules/{}/assignments/{}/memo_output/generate", data.module.id, data.assignment.id);
        
        let req = Request::builder()
            .method("POST")
            .uri(&uri)
            .header("Authorization", format!("Bearer {}", token))
            .body(Body::empty())
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
        setup_input_dirs(data.module.id, data.assignment.id);

        let app = make_app(db.clone());
        let (token, _) = generate_jwt(data.admin_user.id, data.admin_user.admin);
        let uri = format!("/api/modules/{}/assignments/{}/memo_output/generate", data.module.id, data.assignment.id);
        let req = Request::builder()
            .method("POST")
            .uri(&uri)
            .header("Authorization", format!("Bearer {}", token))
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(req).await.unwrap();

        // --- DEBUGGING: print status and body ---
        let status = response.status();
        let bytes = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let body_str = String::from_utf8(bytes.to_vec()).unwrap();
        println!("Response status: {}", status);
        println!("Response body: {}", body_str);
        // ----------------------------------------

        assert_eq!(status, StatusCode::OK);

        cleanup_tmp();
    }
    
    #[tokio::test]
    #[serial]
    async fn test_post_memo_output_forbidden_for_student() {
        dotenvy::dotenv().expect("Failed to load .env");
        let db = setup_test_db().await;
        let data = setup_test_data(&db).await;

        let app = make_app(db.clone());
        let (token, _) = generate_jwt(data.student_user.id, data.student_user.admin);
        let uri = format!("/api/modules/{}/assignments/{}/memo_output/generate", data.module.id, data.assignment.id);
        let req = Request::builder()
            .method("POST")
            .uri(&uri)
            .header("Authorization", format!("Bearer {}", token))
            .body(Body::empty())
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

        let app = make_app(db.clone());
        let (token, _) = generate_jwt(data.forbidden_user.id, data.forbidden_user.admin);
        let uri = format!("/api/modules/{}/assignments/{}/memo_output/generate", data.module.id, data.assignment.id);
        let req = Request::builder()
            .method("POST")
            .uri(&uri)
            .header("Authorization", format!("Bearer {}", token))
            .body(Body::empty())
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

        let app = make_app(db.clone());
        let uri = format!("/api/modules/{}/assignments/{}/memo_output/generate", data.module.id, data.assignment.id);
        let req = Request::builder()
            .method("POST")
            .uri(&uri)
            .body(Body::empty())
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

        let app = make_app(db.clone());
        let (token, _) = generate_jwt(data.lecturer_user.id, data.lecturer_user.admin);
        let uri = format!("/api/modules/{}/assignments/{}/memo_output/generate", data.module.id, 9999);
        let req = Request::builder()
            .method("POST")
            .uri(&uri)
            .header("Authorization", format!("Bearer {}", token))
            .body(Body::empty())
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

        let app = make_app(db.clone());
        let (token, _) = generate_jwt(data.lecturer_user.id, data.lecturer_user.admin);
        let uri = format!("/api/modules/{}/assignments/{}/memo_output/generate", data.module.id, data.assignment.id);
        let req = Request::builder()
            .method("POST")
            .uri(&uri)
            .header("Authorization", format!("Bearer {}", token))
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);

        cleanup_tmp();
    }
}