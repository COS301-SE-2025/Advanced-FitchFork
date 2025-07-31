#[cfg(test)]
mod tests {
    use crate::test_helpers::make_app;
    use api::auth::generate_jwt;
    use axum::{
        body::Body,
        http::{Request, StatusCode},
    };
    use chrono::{TimeZone, Utc};
    use db::{
        models::{
            assignment::Model as AssignmentModel,
            assignment_task::Model as AssignmentTaskModel,
            module::Model as ModuleModel,
            user::Model as UserModel,
            user_module_role::{Model as UserModuleRoleModel, Role},
        },
        test_utils::setup_test_db,
    };
    use serial_test::serial;
    use std::{fs, io::Write, path::PathBuf};
    use tempfile::{TempDir, tempdir};
    use tower::ServiceExt;
    use zip::write::SimpleFileOptions;

    fn create_makefile_content_stub() -> Vec<u8> {
        b"task1:\n\t@echo 'Memo for task 1'\n\t@mkdir -p /output\n\t@echo 'Memo for task 1' > /output/1.txt\n".to_vec()
    }

    fn create_makefile_zip_stub() -> Vec<u8> {
        let mut buf = Vec::new();
        let mut zip = zip::ZipWriter::new(std::io::Cursor::new(&mut buf));
        let options = SimpleFileOptions::default();
        zip.start_file("Makefile", options).unwrap();
        zip.write_all(&create_makefile_content_stub()).unwrap();
        zip.finish().unwrap();
        buf
    }

    fn create_main_java_zip_stub() -> Vec<u8> {
        let mut buf = Vec::new();
        let mut zip = zip::ZipWriter::new(std::io::Cursor::new(&mut buf));
        let options = SimpleFileOptions::default();
        zip.start_file("Main.java", options).unwrap();
        zip.write_all(b"public class Main { public static void main(String[] args) { System.out.println(\"Stub\"); } }").unwrap();
        zip.finish().unwrap();
        buf
    }

    fn create_execution_config_stub(base_path: &PathBuf) {
        let config_dir = base_path.join("config");
        fs::create_dir_all(&config_dir).unwrap();
        let config_content = r#"{
  "execution": {
    "timeout_secs": 10,
    "max_memory": 8589934592,
    "max_cpus": 2,
    "max_uncompressed_size": 100000000,
    "max_processes": 256
  },
  "marking": {
    "marking_scheme": "exact",
    "feedback_scheme": "auto",
    "deliminator": "&-=-&"
  }
}
"#;
        fs::write(config_dir.join("config.json"), config_content).unwrap();
    }

    fn create_memo_zip_stub() -> Vec<u8> {
        let mut buf = Vec::new();
        let mut zip = zip::ZipWriter::new(std::io::Cursor::new(&mut buf));
        let options = SimpleFileOptions::default();
        zip.start_file("memo_content.txt", options).unwrap();
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

    async fn setup_test_data(db: &sea_orm::DatabaseConnection) -> (TestData, TempDir) {
        dotenvy::dotenv().expect("Failed to load .env");
        let temp_dir = tempdir().expect("Failed to create temporary directory");
        unsafe {
            std::env::set_var("ASSIGNMENT_STORAGE_ROOT", temp_dir.path().to_str().unwrap());
        }

        let module = ModuleModel::create(db, "COS101", 2024, Some("Test Module"), 16)
            .await
            .unwrap();
        let admin_user = UserModel::create(db, "admin1", "admin1@test.com", "password", true)
            .await
            .unwrap();
        let lecturer_user =
            UserModel::create(db, "lecturer1", "lecturer1@test.com", "password1", false)
                .await
                .unwrap();
        let student_user =
            UserModel::create(db, "student1", "student1@test.com", "password2", false)
                .await
                .unwrap();
        let forbidden_user =
            UserModel::create(db, "forbidden", "forbidden@test.com", "password3", false)
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
        AssignmentTaskModel::create(db, assignment.id, 1, "Task 1", "make task1")
            .await
            .unwrap();

        (
            TestData {
                admin_user,
                lecturer_user,
                student_user,
                forbidden_user,
                module,
                assignment,
            },
            temp_dir,
        )
    }

    fn setup_input_dirs(module_id: i64, assignment_id: i64, storage_root: &std::path::Path) {
        let base = storage_root
            .join(format!("module_{}", module_id))
            .join(format!("assignment_{}", assignment_id));

        fs::create_dir_all(&base.join("memo")).unwrap();
        fs::create_dir_all(&base.join("config")).unwrap();
        fs::create_dir_all(&base.join("makefile")).unwrap();
        fs::create_dir_all(&base.join("main")).unwrap();
        fs::create_dir_all(&base.join("assignment_submissions")).unwrap();
        fs::create_dir_all(&base.join("mark_allocator")).unwrap();

        fs::write(base.join("memo").join("memo.zip"), create_memo_zip_stub()).unwrap();

        create_execution_config_stub(&base);

        fs::write(
            base.join("makefile").join("makefile.zip"),
            create_makefile_zip_stub(),
        )
        .unwrap();
        fs::write(
            base.join("main").join("main.zip"),
            create_main_java_zip_stub(),
        )
        .unwrap();

        let allocator_content = r#"{
            "generated_at": "2025-07-21T10:00:00Z",
            "tasks": [
                {
                    "task1": {
                        "name": "Task 1 Execution",
                        "value": 10,
                        "subsections": [
                            { "name": "Subtask 1 Output", "value": 10 }
                        ]
                    }
                }
            ]
        }"#;
        fs::write(
            base.join("mark_allocator").join("allocator.json"),
            allocator_content,
        )
        .unwrap();
    }

    #[tokio::test]
    #[serial]
    async fn test_post_memo_output_success_as_lecturer() {
        let db = setup_test_db().await;
        let (data, temp_dir) = setup_test_data(&db).await;
        setup_input_dirs(data.module.id, data.assignment.id, temp_dir.path());

        let app = make_app(db.clone());
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

        let memo_path = temp_dir
            .path()
            .join(format!("module_{}", data.module.id))
            .join(format!("assignment_{}", data.assignment.id))
            .join(format!("memo_output"))
            .join(format!("1.txt"));

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
        let db = setup_test_db().await;
        let (data, temp_dir) = setup_test_data(&db).await;
        setup_input_dirs(data.module.id, data.assignment.id, temp_dir.path());

        let app = make_app(db.clone());
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

        let memo_path = temp_dir
            .path()
            .join(format!("module_{}", data.module.id))
            .join(format!("assignment_{}", data.assignment.id))
            .join("memo_output")
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
        let db = setup_test_db().await;
        let (data, _temp_dir) = setup_test_data(&db).await;

        let app = make_app(db.clone());
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
        let db = setup_test_db().await;
        let (data, _temp_dir) = setup_test_data(&db).await;

        let app = make_app(db.clone());
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
        let db = setup_test_db().await;
        let (data, _temp_dir) = setup_test_data(&db).await;

        let app = make_app(db.clone());
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
        let db = setup_test_db().await;
        let (data, _temp_dir) = setup_test_data(&db).await;

        let app = make_app(db.clone());
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
