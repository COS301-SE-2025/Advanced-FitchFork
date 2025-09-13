#[cfg(test)]
mod tests {
    use crate::helpers::app::make_test_app_with_storage;
    use api::auth::generate_jwt;
    use axum::{body::Body, http::{Request, StatusCode}};
    use chrono::{TimeZone, Utc};
    use db::models::{
        assignment::Model as AssignmentModel,
        assignment_task::Model as AssignmentTaskModel,
        module::Model as ModuleModel,
        user::Model as UserModel,
        user_module_role::{Model as UserModuleRoleModel, Role},
    };
    use serial_test::serial;
    use std::{fs, io::Write};
    use tower::ServiceExt;
    use zip::write::SimpleFileOptions;
    use util::{execution_config::{execution_config::SubmissionMode, ExecutionConfig}, languages::Language, paths::{
        main_dir, makefile_dir, mark_allocator_dir, memo_dir, memo_output_dir
    }};

    fn create_makefile_zip_stub() -> Vec<u8> {
        let mut buf = Vec::new();
        let mut zip = zip::ZipWriter::new(std::io::Cursor::new(&mut buf));
        zip.start_file("Makefile", SimpleFileOptions::default()).unwrap();
        zip.write_all(b"task1:\n\t@echo 'Memo for task 1'\n\t@mkdir -p /output\n\t@echo 'Memo for task 1' > /output/1.txt\n").unwrap();
        zip.finish().unwrap();
        buf
    }

    fn create_main_java_zip_stub() -> Vec<u8> {
        let mut buf = Vec::new();
        let mut zip = zip::ZipWriter::new(std::io::Cursor::new(&mut buf));
        zip.start_file("Main.java", SimpleFileOptions::default()).unwrap();
        zip.write_all(b"public class Main { public static void main(String[] args) { System.out.println(\"Stub\"); } }").unwrap();
        zip.finish().unwrap();
        buf
    }

    fn create_execution_config_stub(module_id: i64, assignment_id: i64) {
        // Start from the defaulted struct and only override what we need for the test
        let mut cfg = ExecutionConfig::default_config();
        cfg.project.language = Language::Java;                 // we’re providing Main.java
        cfg.project.submission_mode = SubmissionMode::Manual;  // matches your route expectations
        cfg.marking.deliminator = "&-=-&".to_string();         // keep existing spelling
        
        cfg.save(module_id, assignment_id).expect("write config.json");
    }

    fn create_memo_zip_stub() -> Vec<u8> {
        let mut buf = Vec::new();
        let mut zip = zip::ZipWriter::new(std::io::Cursor::new(&mut buf));
        zip.start_file("memo_content.txt", SimpleFileOptions::default()).unwrap();
        zip.write_all(b"This is the memo content.").unwrap();
        zip.finish().unwrap();
        buf
    }

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
        ).await.unwrap();

        AssignmentTaskModel::create(db, assignment.id, 1, "Task 1", "make task1", false).await.unwrap();

        TestData { admin_user, lecturer_user, student_user, forbidden_user, module, assignment }
    }

    fn setup_input_dirs(module_id: i64, assignment_id: i64) {
        fs::create_dir_all(&memo_dir(module_id, assignment_id)).unwrap();
        fs::create_dir_all(&memo_output_dir(module_id, assignment_id)).unwrap();
        fs::create_dir_all(&makefile_dir(module_id, assignment_id)).unwrap();
        fs::create_dir_all(&main_dir(module_id, assignment_id)).unwrap();
        fs::create_dir_all(&mark_allocator_dir(module_id, assignment_id)).unwrap();
        fs::write(memo_dir(module_id, assignment_id).join("memo.zip"), create_memo_zip_stub()).unwrap();
        create_execution_config_stub(module_id, assignment_id);
        fs::write(makefile_dir(module_id, assignment_id).join("makefile.zip"), create_makefile_zip_stub()).unwrap();
        fs::write(main_dir(module_id, assignment_id).join("main.zip"), create_main_java_zip_stub()).unwrap();
        fs::write(mark_allocator_dir(module_id, assignment_id).join("allocator.json"), r#"{
            "generated_at": "2025-07-21T10:00:00Z",
            "tasks": [
                { "task1": { "name": "Task 1 Execution", "value": 10,
                    "subsections": [{ "name": "Subtask 1 Output", "value": 10 }] } }
            ]
        }"#).unwrap();
    }


    #[tokio::test]
    #[serial]
    async fn test_post_memo_output_success_as_lecturer() {
        let (app, app_state, _tmp) = make_test_app_with_storage().await;
        let data = setup_test_data(app_state.db()).await;
        setup_input_dirs(data.module.id, data.assignment.id);

        let (token, _) = generate_jwt(data.lecturer_user.id, data.lecturer_user.admin);
        let uri = format!(
            "/api/modules/{}/assignments/{}/memo_output/generate",
            data.module.id, data.assignment.id
        );
        let req = Request::builder()
            .method("POST")
            .uri(&uri)
            .header("Authorization", format!("Bearer {}", token))
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);

       let memo_path = memo_output_dir(data.module.id, data.assignment.id)
            .join("1.txt");

        assert!(
            memo_path.exists(),
            "Expected memo output file not found at {:?}",
            memo_path
        );
        let content = fs::read_to_string(&memo_path).unwrap();
        assert!(
            content.contains("Memo for task 1"),
            "Memo content does not match expectation. Found: {}",
            content
        );
    }

    #[tokio::test]
    #[serial]
    async fn test_post_memo_output_success_as_admin() {
        let (app, app_state, _tmp) = make_test_app_with_storage().await;
        let data = setup_test_data(app_state.db()).await;
        setup_input_dirs(data.module.id, data.assignment.id);

        let (token, _) = generate_jwt(data.admin_user.id, data.admin_user.admin);
        let uri = format!(
            "/api/modules/{}/assignments/{}/memo_output/generate",
            data.module.id, data.assignment.id
        );
        let req = Request::builder()
            .method("POST")
            .uri(&uri)
            .header("Authorization", format!("Bearer {}", token))
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);

        let memo_path = memo_output_dir(data.module.id, data.assignment.id)
            .join("1.txt");

        assert!(
            memo_path.exists(),
            "Expected memo output file not found at {:?}",
            memo_path
        );
        let content = fs::read_to_string(&memo_path).unwrap();
        assert!(
            content.contains("Memo for task 1"),
            "Memo content does not match expectation. Found: {}",
            content
        );
    }

    #[tokio::test]
    #[serial]
    async fn test_post_memo_output_forbidden_for_student() {
        let (app, app_state, _tmp) = make_test_app_with_storage().await;
        let data = setup_test_data(app_state.db()).await;

        let (token, _) = generate_jwt(data.student_user.id, data.student_user.admin);
        let uri = format!(
            "/api/modules/{}/assignments/{}/memo_output/generate",
            data.module.id, data.assignment.id
        );
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
        let (app, app_state, _tmp) = make_test_app_with_storage().await;
        let data = setup_test_data(app_state.db()).await;

        let (token, _) = generate_jwt(data.forbidden_user.id, data.forbidden_user.admin);
        let uri = format!(
            "/api/modules/{}/assignments/{}/memo_output/generate",
            data.module.id, data.assignment.id
        );
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
        let (app, app_state, _tmp) = make_test_app_with_storage().await;
        let data = setup_test_data(app_state.db()).await;

        let uri = format!(
            "/api/modules/{}/assignments/{}/memo_output/generate",
            data.module.id, data.assignment.id
        );
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
        let (app, app_state, _tmp) = make_test_app_with_storage().await;
        let data = setup_test_data(app_state.db()).await;

        let (token, _) = generate_jwt(data.lecturer_user.id, data.lecturer_user.admin);
        let uri = format!(
            "/api/modules/{}/assignments/{}/memo_output/generate",
            data.module.id, 9999
        );
        let req = Request::builder()
            .method("POST")
            .uri(&uri)
            .header("Authorization", format!("Bearer {}", token))
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }
}
