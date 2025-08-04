#[cfg(test)]
mod tests {
    use crate::helpers::app::make_test_app;
    use api::auth::generate_jwt;
    use axum::http::header::{AUTHORIZATION, CONTENT_TYPE};
    use axum::{
        body::Body,
        http::{Request, StatusCode},
        response::Response,
    };
    use chrono::{Duration, TimeZone, Utc};
    use db::{
        models::{
            assignment::{
                ActiveModel as AssignmentActiveModel, Entity as AssignmentEntity,
                Model as AssignmentModel,
            },
            assignment_submission::{
                self,
                Model as AssignmentSubmissionModel,
            },
            assignment_task::Model as AssignmentTaskModel,
            module::Model as ModuleModel,
            user::Model as UserModel,
            user_module_role::{Model as UserModuleRoleModel, Role},
        }
    };
    use flate2::{Compression, write::GzEncoder};
    use sea_orm::{ActiveModelTrait, DatabaseConnection, EntityTrait, Set};
    use serde_json::{json, Value};
    use serial_test::serial;
    use std::path::Path;
    use std::{fs, io::Write};
    use tar::Builder as TarBuilder;
    use tempfile::{TempDir, tempdir};
    use tower::ServiceExt;
    use zip::write::SimpleFileOptions;
    use tower::util::BoxCloneService;
    use std::convert::Infallible;

    struct TestData {
        student_user: UserModel,
        unassigned_user: UserModel,
        module: ModuleModel,
        assignment: AssignmentModel,
    }

    fn setup_assignment_storage_root() -> TempDir {
        let temp_dir = tempdir().expect("Failed to create temporary directory");
        unsafe {
            std::env::set_var("ASSIGNMENT_STORAGE_ROOT", temp_dir.path());
        }
        temp_dir
    }

    async fn setup_test_data(db: &DatabaseConnection, temp_dir: &TempDir) -> TestData {
        let module = ModuleModel::create(db, "COS101", 2024, Some("Test Module"), 16)
            .await
            .unwrap();
        let student_user =
            UserModel::create(db, "student1", "student1@test.com", "password2", false)
                .await
                .unwrap();
        let unassigned_user =
            UserModel::create(&db, "unassigned", "unassigned@test.com", "password", false)
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

        let assignment_base_path = temp_dir
            .path()
            .join(format!("module_{}", module.id))
            .join(format!("assignment_{}", assignment.id));
        create_assignment_structure(&assignment_base_path);
        create_mark_allocator(&assignment_base_path);
        create_memo_outputs(&assignment_base_path);
        create_execution_config(&assignment_base_path);
        create_makefile_zip(&assignment_base_path);
        create_main_zip(&assignment_base_path);

        TestData {
            student_user,
            unassigned_user,
            module,
            assignment,
        }
    }

    fn create_assignment_structure(base_path: &Path) {
        fs::create_dir_all(base_path.join("mark_allocator")).unwrap();
        fs::create_dir_all(base_path.join("memo_output")).unwrap();
        fs::create_dir_all(base_path.join("assignment_submissions")).unwrap();
        fs::create_dir_all(base_path.join("config")).unwrap();
        fs::create_dir_all(base_path.join("makefile")).unwrap();
        fs::create_dir_all(base_path.join("main")).unwrap();
    }

    fn create_mark_allocator(base_path: &Path) {
        let path = base_path.join("mark_allocator");
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
        fs::write(path.join("allocator.json"), allocator_content).unwrap();
    }

    fn create_memo_outputs(base_path: &std::path::Path) {
        let path = base_path.join("memo_output");
        let memo_content = "make task1\n&-=-&Subtask 1 Output\nOutput A";
        fs::write(path.join("task1.txt"), memo_content).unwrap();
    }

    fn create_execution_config(base_path: &Path) {
        let path = base_path.join("config");
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
        fs::write(path.join("config.json"), config_content).unwrap();
    }

    fn create_makefile_zip(base_path: &Path) {
        let mut buf = Vec::new();
        let mut zip = zip::ZipWriter::new(std::io::Cursor::new(&mut buf));
        let options = SimpleFileOptions::default();

        zip.start_file("Makefile", options).unwrap();
        zip.write_all(&create_makefile_content()).unwrap();

        zip.finish().unwrap();
        fs::write(base_path.join("makefile").join("makefile.zip"), buf).unwrap();
    }

    fn create_main_zip(base_path: &Path) {
        let mut buf = Vec::new();
        let mut zip = zip::ZipWriter::new(std::io::Cursor::new(&mut buf));
        let options = SimpleFileOptions::default();

        zip.start_file("Main.java", options).unwrap();
        zip.write_all(&create_main_content()).unwrap();

        zip.start_file("HelperOne.java", options).unwrap();
        zip.write_all(&create_helper_one_content()).unwrap();

        zip.start_file("HelperTwo.java", options).unwrap();
        zip.write_all(&create_helper_two_content()).unwrap();

        zip.finish().unwrap();
        fs::write(base_path.join("main").join("main.zip"), buf).unwrap();
    }

    fn create_submission_zip() -> Vec<u8> {
        let mut buf = Vec::new();
        let mut zip = zip::ZipWriter::new(std::io::Cursor::new(&mut buf));
        let options = SimpleFileOptions::default();

        zip.start_file("Main.java", options).unwrap();
        zip.write_all(&create_main_content()).unwrap();

        zip.start_file("HelperOne.java", options).unwrap();
        zip.write_all(&create_helper_one_content()).unwrap();

        zip.start_file("HelperTwo.java", options).unwrap();
        zip.write_all(&create_helper_two_content()).unwrap();

        zip.finish().unwrap();
        buf
    }

    fn create_submission_tar() -> Vec<u8> {
        let mut buf = Vec::new();
        {
            let mut tar = TarBuilder::new(&mut buf);
            let mut header = tar::Header::new_gnu();

            let main_content = create_main_content();
            header.set_path("Main.java").unwrap();
            header.set_size(main_content.len() as u64);
            header.set_mode(0o644);
            header.set_cksum();
            tar.append(&header, &main_content[..]).unwrap();

            let helper1_content = create_helper_one_content();
            header.set_path("HelperOne.java").unwrap();
            header.set_size(helper1_content.len() as u64);
            header.set_mode(0o644);
            header.set_cksum();
            tar.append(&header, &helper1_content[..]).unwrap();

            let helper2_content = create_helper_two_content();
            header.set_path("HelperTwo.java").unwrap();
            header.set_size(helper2_content.len() as u64);
            header.set_mode(0o644);
            header.set_cksum();
            tar.append(&header, &helper2_content[..]).unwrap();

            tar.finish().unwrap();
            drop(tar);
        }
        buf
    }

    fn create_submission_tgz() -> Vec<u8> {
        let tar_buf = create_submission_tar();
        let mut gz_buf = Vec::new();
        {
            let mut encoder = GzEncoder::new(&mut gz_buf, Compression::default());
            encoder.write_all(&tar_buf).unwrap();
            encoder.finish().unwrap();
        }
        gz_buf
    }

    fn create_submission_gz() -> Vec<u8> {
        let tar_buf = create_submission_tar();
        let mut gz_buf = Vec::new();
        {
            let mut encoder = GzEncoder::new(&mut gz_buf, Compression::default());
            encoder.write_all(&tar_buf).unwrap();
            encoder.finish().unwrap();
        }
        gz_buf
    }

    fn create_large_zip_file(size: usize) -> Vec<u8> {
        let mut buf = Vec::new();
        {
            let mut zip = zip::ZipWriter::new(std::io::Cursor::new(&mut buf));
            let options = SimpleFileOptions::default();
            zip.start_file("large.txt", options).unwrap();
            zip.write_all(&vec![b'a'; size]).unwrap();
            zip.finish().unwrap();
        }
        buf
    }

    fn create_corrupted_zip() -> Vec<u8> {
        let mut buf = Vec::new();
        buf.extend_from_slice(b"PK\x03\x04");
        buf.extend_from_slice(b"This is definitely not valid ZIP file data after the header. It lacks the proper local file header structure, central directory, and end of central directory record. The ZIP library should fail to read this properly.");
        buf.extend(std::iter::repeat(0xFF).take(1000));
        buf
    }

    fn create_exe_file() -> Vec<u8> {
        b"invalid executable content".to_vec()
    }

    fn create_makefile_content() -> Vec<u8> {
        b"task1:\n\tjavac -d /output Main.java HelperOne.java HelperTwo.java && java -cp /output Main task1\n\ntask2:\n\tjavac -d /output Main.java HelperOne.java HelperTwo.java && java -cp /output Main task2\n".to_vec()
    }

    fn create_main_content() -> Vec<u8> {
        br#"
        public class Main {
            public static void main(String[] args) {
                String task = args.length > 0 ? args[0] : "task1";
                switch (task) {
                    case "task1": runTask1(); break;
                    case "task2": runTask2(); break;
                    default: System.out.println(task + " is not a valid task");
                }
            }
            static void runTask1() {
                System.out.println("&-=-&Task1Subtask1");
                System.out.println(HelperOne.subtaskA());
            }
            static void runTask2() {
                System.out.println("&-=-&Task2Subtask1");
                System.out.println(HelperTwo.subtaskX());
            }
        }"#
        .to_vec()
    }

    fn create_helper_one_content() -> Vec<u8> {
        b"public class HelperOne { public static String subtaskA() { return \"Output A\"; } }"
            .to_vec()
    }

    fn create_helper_two_content() -> Vec<u8> {
        b"public class HelperTwo { public static String subtaskX() { return \"Output X\"; } }"
            .to_vec()
    }

    fn multipart_body(
        filename: &str,
        file_content: &[u8],
        is_practice: Option<&str>,
    ) -> (String, Vec<u8>) {
        let boundary = "----BoundaryTest".to_string();
        let mut body = Vec::new();
        body.extend(format!("--{}\r\n", boundary).as_bytes());
        body.extend(format!("Content-Disposition: form-data; name=\"file\"; filename=\"{}\"\r\nContent-Type: application/octet-stream\r\n\r\n", filename).as_bytes());
        body.extend(file_content);
        body.extend(b"\r\n");
        if let Some(val) = is_practice {
            body.extend(format!("--{}\r\n", boundary).as_bytes());
            body.extend(
                format!(
                    "Content-Disposition: form-data; name=\"is_practice\"\r\n\r\n{}\r\n",
                    val
                )
                .as_bytes(),
            );
        }
        body.extend(format!("--{}--\r\n", boundary).as_bytes());
        (boundary, body)
    }

    #[tokio::test]
    #[serial]
    async fn test_valid_submission_zip() {
        let temp_dir = setup_assignment_storage_root();
        let (app, app_state) = make_test_app().await;
        let data = setup_test_data(app_state.db(), &temp_dir).await;
        let file = create_submission_zip();

        let (boundary, body) = multipart_body("solution.zip", &file, None);
        let (token, _) = generate_jwt(data.student_user.id, data.student_user.admin);
        let uri = format!(
            "/api/modules/{}/assignments/{}/submissions",
            data.module.id, data.assignment.id
        );
        let req = Request::builder()
            .method("POST")
            .uri(&uri)
            .header(AUTHORIZATION, format!("Bearer {}", token))
            .header(
                CONTENT_TYPE,
                format!("multipart/form-data; boundary={}", boundary),
            )
            .body(Body::from(body))
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let json: Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(json["success"], true);
        assert_eq!(json["data"]["filename"], "solution.zip");
    }

    #[tokio::test]
    #[serial]
    async fn test_valid_submission_tar() {
        let temp_dir =setup_assignment_storage_root();
        let (app, app_state) = make_test_app().await;
        let data = setup_test_data(app_state.db(), &temp_dir).await;
        let file = create_submission_tar();

        let (boundary, body) = multipart_body("solution.tar", &file, None);
        let (token, _) = generate_jwt(data.student_user.id, data.student_user.admin);
        let uri = format!(
            "/api/modules/{}/assignments/{}/submissions",
            data.module.id, data.assignment.id
        );
        let req = Request::builder()
            .method("POST")
            .uri(&uri)
            .header(AUTHORIZATION, format!("Bearer {}", token))
            .header(
                CONTENT_TYPE,
                format!("multipart/form-data; boundary={}", boundary),
            )
            .body(Body::from(body))
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let json: Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(json["success"], true);
        assert_eq!(json["data"]["filename"], "solution.tar");
    }

    #[tokio::test]
    #[serial]
    async fn test_valid_submission_tgz() {
        let temp_dir =setup_assignment_storage_root();
        let (app, app_state) = make_test_app().await;
        let data = setup_test_data(app_state.db(), &temp_dir).await;
        let file = create_submission_tgz();

        let (boundary, body) = multipart_body("solution.tgz", &file, None);
        let (token, _) = generate_jwt(data.student_user.id, data.student_user.admin);
        let uri = format!(
            "/api/modules/{}/assignments/{}/submissions",
            data.module.id, data.assignment.id
        );
        let req = Request::builder()
            .method("POST")
            .uri(&uri)
            .header(AUTHORIZATION, format!("Bearer {}", token))
            .header(
                CONTENT_TYPE,
                format!("multipart/form-data; boundary={}", boundary),
            )
            .body(Body::from(body))
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let json: Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(json["success"], true);
        assert_eq!(json["data"]["filename"], "solution.tgz");
    }

    #[tokio::test]
    #[serial]
    async fn test_valid_submission_gz() {
        let temp_dir =setup_assignment_storage_root();
        let (app, app_state) = make_test_app().await;
        let data = setup_test_data(app_state.db(), &temp_dir).await;
        let file = create_submission_gz();

        let (boundary, body) = multipart_body("solution.gz", &file, None);
        let (token, _) = generate_jwt(data.student_user.id, data.student_user.admin);
        let uri = format!(
            "/api/modules/{}/assignments/{}/submissions",
            data.module.id, data.assignment.id
        );
        let req = Request::builder()
            .method("POST")
            .uri(&uri)
            .header(AUTHORIZATION, format!("Bearer {}", token))
            .header(
                CONTENT_TYPE,
                format!("multipart/form-data; boundary={}", boundary),
            )
            .body(Body::from(body))
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let json: Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(json["success"], true);
        assert_eq!(json["data"]["filename"], "solution.gz");
    }

    #[tokio::test]
    #[serial]
    async fn test_practice_submission() {
        let temp_dir =setup_assignment_storage_root();
        let (app, app_state) = make_test_app().await;
        let data = setup_test_data(app_state.db(), &temp_dir).await;
        let file = create_submission_zip();

        let (boundary, body) = multipart_body("solution.zip", &file, Some("true"));
        let (token, _) = generate_jwt(data.student_user.id, data.student_user.admin);
        let uri = format!(
            "/api/modules/{}/assignments/{}/submissions",
            data.module.id, data.assignment.id
        );
        let req = Request::builder()
            .method("POST")
            .uri(&uri)
            .header(AUTHORIZATION, format!("Bearer {}", token))
            .header(
                CONTENT_TYPE,
                format!("multipart/form-data; boundary={}", boundary),
            )
            .body(Body::from(body))
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let json: Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(json["success"], true);
        assert_eq!(json["data"]["is_practice"], true);
    }

    #[tokio::test]
    #[serial]
    async fn test_multiple_attempts_increments() {
        let temp_dir =setup_assignment_storage_root();
        let (app, app_state) = make_test_app().await;
        let data = setup_test_data(app_state.db(), &temp_dir).await;
        let file = create_submission_zip();

        let (boundary, body) = multipart_body("solution.zip", &file, None);
        let (token, _) = generate_jwt(data.student_user.id, data.student_user.admin);
        let uri = format!(
            "/api/modules/{}/assignments/{}/submissions",
            data.module.id, data.assignment.id
        );
        let req1 = Request::builder()
            .method("POST")
            .uri(&uri)
            .header(AUTHORIZATION, format!("Bearer {}", token))
            .header(
                CONTENT_TYPE,
                format!("multipart/form-data; boundary={}", boundary.clone()),
            )
            .body(Body::from(body.clone()))
            .unwrap();

        let response1 = app.clone().oneshot(req1).await.unwrap();
        assert_eq!(response1.status(), StatusCode::OK);

        let body1 = axum::body::to_bytes(response1.into_body(), usize::MAX)
            .await
            .unwrap();
        let json1: Value = serde_json::from_slice(&body1).unwrap();
        assert_eq!(json1["success"], true);
        assert_eq!(json1["data"]["attempt"], 1);

        let req2 = Request::builder()
            .method("POST")
            .uri(&uri)
            .header(AUTHORIZATION, format!("Bearer {}", token))
            .header(
                CONTENT_TYPE,
                format!("multipart/form-data; boundary={}", boundary.clone()),
            )
            .body(Body::from(body.clone()))
            .unwrap();

        let response2 = app.oneshot(req2).await.unwrap();
        assert_eq!(response2.status(), StatusCode::OK);

        let body2 = axum::body::to_bytes(response2.into_body(), usize::MAX)
            .await
            .unwrap();
        let json2: Value = serde_json::from_slice(&body2).unwrap();
        assert_eq!(json2["success"], true);
        assert_eq!(json2["data"]["attempt"], 2);
    }

    // TODO: Once coverage and complexity have been implemented, uncomment this test.
    //
    // #[tokio::test]
    // #[serial]
    // async fn test_submission_with_code_coverage_and_complexity_fields() {
    //     let db = setup_test_db().await;
    //     let data = setup_test_data(&db).await;
    //     let app = make_app(db.clone());
    //     let file = create_submission_zip();
    //     let (boundary, body) = multipart_body("solution.zip", &file, None);
    //     let (token, _) = generate_jwt(data.student_user.id, data.student_user.admin);
    //     let uri = format!("/api/modules/{}/assignments/{}/submissions", data.module.id, data.assignment.id);
    //     let req = Request::builder()
    //         .method("POST")
    //         .uri(&uri)
    //         .header(AUTHORIZATION, format!("Bearer {}", token))
    //         .header(CONTENT_TYPE, format!("multipart/form-data; boundary={}", boundary))
    //         .body(Body::from(body))
    //         .unwrap();
    //     let response = app.oneshot(req).await.unwrap();
    //     assert_eq!(response.status(), StatusCode::OK);
    //     let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    //     let json: Value = serde_json::from_slice(&body).unwrap();
    //     assert_eq!(json["success"], true);
    //     assert!(json["data"].get("code_coverage").is_some());
    //     assert!(json["data"].get("code_complexity").is_some());
    // }

    #[tokio::test]
    #[serial]
    async fn test_submission_exactly_at_due_date() {
        let temp_dir =setup_assignment_storage_root();
        let (app, app_state) = make_test_app().await;
        let data = setup_test_data(app_state.db(), &temp_dir).await;
        let file = create_submission_zip();

        let mut assignment_active_model: AssignmentActiveModel =
            AssignmentEntity::find_by_id(data.assignment.id)
                .one(app_state.db())
                .await
                .unwrap()
                .unwrap()
                .into();
        assignment_active_model.due_date = Set(Utc::now());
        assignment_active_model.update(app_state.db()).await.unwrap();

        let (boundary, body) = multipart_body("solution.zip", &file, None);
        let (token, _) = generate_jwt(data.student_user.id, data.student_user.admin);
        let uri = format!(
            "/api/modules/{}/assignments/{}/submissions",
            data.module.id, data.assignment.id
        );
        let req = Request::builder()
            .method("POST")
            .uri(&uri)
            .header(AUTHORIZATION, format!("Bearer {}", token))
            .header(
                CONTENT_TYPE,
                format!("multipart/form-data; boundary={}", boundary),
            )
            .body(Body::from(body))
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let json: Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(json["success"], true);
        assert_eq!(json["data"]["is_late"], true);
    }

    #[tokio::test]
    #[serial]
    async fn test_submission_just_after_due_date() {
        let temp_dir =setup_assignment_storage_root();
        let (app, app_state) = make_test_app().await;
        let data = setup_test_data(app_state.db(), &temp_dir).await;
        let file = create_submission_zip();

        let mut assignment_active_model: AssignmentActiveModel =
            AssignmentEntity::find_by_id(data.assignment.id)
                .one(app_state.db())
                .await
                .unwrap()
                .unwrap()
                .into();
        assignment_active_model.due_date = Set(Utc::now() - Duration::hours(1));
        assignment_active_model.update(app_state.db()).await.unwrap();

        let (boundary, body) = multipart_body("solution.zip", &file, None);
        let (token, _) = generate_jwt(data.student_user.id, data.student_user.admin);
        let uri = format!(
            "/api/modules/{}/assignments/{}/submissions",
            data.module.id, data.assignment.id
        );
        let req = Request::builder()
            .method("POST")
            .uri(&uri)
            .header(AUTHORIZATION, format!("Bearer {}", token))
            .header(
                CONTENT_TYPE,
                format!("multipart/form-data; boundary={}", boundary),
            )
            .body(Body::from(body))
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let json: Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(json["success"], true);
        assert_eq!(json["data"]["is_late"], true);
    }

    #[tokio::test]
    #[serial]
    async fn test_submission_just_before_due_date() {
        let temp_dir = setup_assignment_storage_root();
        let (app, app_state) = make_test_app().await;
        let data = setup_test_data(app_state.db(), &temp_dir).await;
        let file = create_submission_zip();

        let mut assignment_active_model: AssignmentActiveModel =
            AssignmentEntity::find_by_id(data.assignment.id)
                .one(app_state.db())
                .await
                .unwrap()
                .unwrap()
                .into();
        assignment_active_model.due_date = Set(Utc::now() + Duration::hours(1));
        assignment_active_model.update(app_state.db()).await.unwrap();

        let (boundary, body) = multipart_body("solution.zip", &file, None);
        let (token, _) = generate_jwt(data.student_user.id, data.student_user.admin);
        let uri = format!(
            "/api/modules/{}/assignments/{}/submissions",
            data.module.id, data.assignment.id
        );
        let req = Request::builder()
            .method("POST")
            .uri(&uri)
            .header(AUTHORIZATION, format!("Bearer {}", token))
            .header(
                CONTENT_TYPE,
                format!("multipart/form-data; boundary={}", boundary),
            )
            .body(Body::from(body))
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let json: Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(json["success"], true);
        assert_eq!(json["data"]["is_late"], false);
    }

    //TODO - Reece this test passes but it should be a negative test
    #[tokio::test]
    #[serial]
    async fn test_large_file_submission() {
        let temp_dir = setup_assignment_storage_root();
        let (app, app_state) = make_test_app().await;
        let data = setup_test_data(app_state.db(), &temp_dir).await;
        let file = create_large_zip_file(1000 * 1000);

        let (boundary, body) = multipart_body("large.zip", &file, None);
        let (token, _) = generate_jwt(data.student_user.id, data.student_user.admin);
        let uri = format!(
            "/api/modules/{}/assignments/{}/submissions",
            data.module.id, data.assignment.id
        );
        let req = Request::builder()
            .method("POST")
            .uri(&uri)
            .header(AUTHORIZATION, format!("Bearer {}", token))
            .header(
                CONTENT_TYPE,
                format!("multipart/form-data; boundary={}", boundary),
            )
            .body(Body::from(body))
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let json: Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(json["success"], true);
        assert_eq!(json["data"]["filename"], "large.zip");
    }

    #[tokio::test]
    #[serial]
    async fn test_missing_file() {
        let temp_dir = setup_assignment_storage_root();
        let (app, app_state) = make_test_app().await;
        let data = setup_test_data(app_state.db(), &temp_dir).await;

        let boundary = "----BoundaryTest".to_string();
        let mut body = Vec::new();
        body.extend(format!("--{}\r\n", boundary).as_bytes());
        body.extend(format!("--{}--\r\n", boundary).as_bytes());

        let (token, _) = generate_jwt(data.student_user.id, data.student_user.admin);
        let uri = format!(
            "/api/modules/{}/assignments/{}/submissions",
            data.module.id, data.assignment.id
        );
        let req = Request::builder()
            .method("POST")
            .uri(&uri)
            .header(AUTHORIZATION, format!("Bearer {}", token))
            .header(
                CONTENT_TYPE,
                format!("multipart/form-data; boundary={}", boundary),
            )
            .body(Body::from(body))
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let json: Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(json["success"], false);
        assert_eq!(json["message"], "No file provided");
    }

    #[tokio::test]
    #[serial]
    async fn test_empty_file() {
        let temp_dir = setup_assignment_storage_root();
        let (app, app_state) = make_test_app().await;
        let data = setup_test_data(app_state.db(), &temp_dir).await;
        let file = Vec::new();

        let (boundary, body) = multipart_body("solution.zip", &file, None);
        let (token, _) = generate_jwt(data.student_user.id, data.student_user.admin);
        let uri = format!(
            "/api/modules/{}/assignments/{}/submissions",
            data.module.id, data.assignment.id
        );
        let req = Request::builder()
            .method("POST")
            .uri(&uri)
            .header(AUTHORIZATION, format!("Bearer {}", token))
            .header(
                CONTENT_TYPE,
                format!("multipart/form-data; boundary={}", boundary),
            )
            .body(Body::from(body))
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let json: Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(json["success"], false);
        assert_eq!(json["message"], "Empty file provided");
    }

    #[tokio::test]
    #[serial]
    async fn test_invalid_file_extension() {
        let temp_dir = setup_assignment_storage_root();
        let (app, app_state) = make_test_app().await;
        let data = setup_test_data(app_state.db(), &temp_dir).await;
        let file = create_exe_file();

        let (boundary, body) = multipart_body("solution.exe", &file, None);
        let (token, _) = generate_jwt(data.student_user.id, data.student_user.admin);
        let uri = format!(
            "/api/modules/{}/assignments/{}/submissions",
            data.module.id, data.assignment.id
        );
        let req = Request::builder()
            .method("POST")
            .uri(&uri)
            .header(AUTHORIZATION, format!("Bearer {}", token))
            .header(
                CONTENT_TYPE,
                format!("multipart/form-data; boundary={}", boundary),
            )
            .body(Body::from(body))
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let json: Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(json["success"], false);
        assert_eq!(
            json["message"],
            "Only .tgz, .gz, .tar, and .zip files are allowed"
        );
    }

    #[tokio::test]
    #[serial]
    async fn test_corrupted_archive() {
        let temp_dir = setup_assignment_storage_root();
        let (app, app_state) = make_test_app().await;
        let data = setup_test_data(app_state.db(), &temp_dir).await;
        let file = create_corrupted_zip();

        let (boundary, body) = multipart_body("corrupted.zip", &file, None);
        let (token, _) = generate_jwt(data.student_user.id, data.student_user.admin);
        let uri = format!(
            "/api/modules/{}/assignments/{}/submissions",
            data.module.id, data.assignment.id
        );
        let req = Request::builder()
            .method("POST")
            .uri(&uri)
            .header(AUTHORIZATION, format!("Bearer {}", token))
            .header(
                CONTENT_TYPE,
                format!("multipart/form-data; boundary={}", boundary),
            )
            .body(Body::from(body))
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let json: Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(json["success"], false);
        assert_eq!(json["message"], "ParseOutputError(\"Expected 1 subtasks, but found 0 delimiters\")");
    }

    #[tokio::test]
    #[serial]
    async fn test_assignment_not_found() {
        let temp_dir = setup_assignment_storage_root();
        let (app, app_state) = make_test_app().await;
        let data = setup_test_data(app_state.db(), &temp_dir).await;
        let file = create_submission_zip();

        let (boundary, body) = multipart_body("solution.zip", &file, None);
        let (token, _) = generate_jwt(data.student_user.id, data.student_user.admin);
        let invalid_assignment_id = 9999;
        let uri = format!(
            "/api/modules/{}/assignments/{}/submissions",
            data.module.id, invalid_assignment_id
        );
        let req = Request::builder()
            .method("POST")
            .uri(&uri)
            .header(AUTHORIZATION, format!("Bearer {}", token))
            .header(
                CONTENT_TYPE,
                format!("multipart/form-data; boundary={}", boundary),
            )
            .body(Body::from(body))
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::NOT_FOUND);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let json: Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(json["success"], false);
        assert_eq!(json["message"], "Assignment 9999 in Module 1 not found.");
    }

    #[tokio::test]
    #[serial]
    async fn test_user_not_assigned() {
        let temp_dir = setup_assignment_storage_root();
        let (app, app_state) = make_test_app().await;
        let data = setup_test_data(app_state.db(), &temp_dir).await;
        let file = create_submission_zip();

        let (boundary, body) = multipart_body("solution.zip", &file, None);
        let (token, _) = generate_jwt(data.unassigned_user.id, data.unassigned_user.admin);
        let uri = format!(
            "/api/modules/{}/assignments/{}/submissions",
            data.module.id, data.assignment.id
        );
        let req = Request::builder()
            .method("POST")
            .uri(&uri)
            .header(AUTHORIZATION, format!("Bearer {}", token))
            .header(
                CONTENT_TYPE,
                format!("multipart/form-data; boundary={}", boundary),
            )
            .body(Body::from(body))
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::FORBIDDEN);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let json: Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(json["success"], false);
        assert_eq!(json["message"], "User not assigned to this module");
    }

    #[tokio::test]
    #[serial]
    async fn test_submitting_to_invalid_assignment() {
        let temp_dir = setup_assignment_storage_root();
        let (app, app_state) = make_test_app().await;
        let data = setup_test_data(app_state.db(), &temp_dir).await;
        let file = create_submission_zip();

        let (boundary, body) = multipart_body("solution.zip", &file, None);
        let (token, _) = generate_jwt(data.student_user.id, data.student_user.admin);
        let uri = format!(
            "/api/modules/{}/assignments/{}/submissions",
            data.module.id, -1
        );
        let req = Request::builder()
            .method("POST")
            .uri(&uri)
            .header(AUTHORIZATION, format!("Bearer {}", token))
            .header(
                CONTENT_TYPE,
                format!("multipart/form-data; boundary={}", boundary),
            )
            .body(Body::from(body))
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::NOT_FOUND);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let json: Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(json["success"], false);
        assert_eq!(json["message"], "Assignment -1 in Module 1 not found.");
    }

    // TODO: Fix this test - I dont think the code runner checks for invalid code yet, but once it does, uncomment this
    //
    // #[tokio::test]
    // #[serial]
    // async fn test_code_runner_failure() {
    //     let db = setup_test_db().await;
    //     let data = setup_test_data(&db).await;
    //     let app = make_app(db.clone());

    //     let mut buf = Vec::new();
    //     {
    //         let mut zip = zip::ZipWriter::new(std::io::Cursor::new(&mut buf));
    //         let options = SimpleFileOptions::default();
    //         zip.start_file("main.c", options).unwrap();
    //         zip.write_all(b"invalid C code that won't compile").unwrap();
    //         zip.finish().unwrap();
    //     }

    //     let (boundary, body) = multipart_body("solution.zip", &buf, None);
    //     let (token, _) = generate_jwt(data.student_user.id, data.student_user.admin);
    //     let uri = format!("/api/modules/{}/assignments/{}/submissions", data.module.id, data.assignment.id);
    //     let req = Request::builder()
    //         .method("POST")
    //         .uri(&uri)
    //         .header(AUTHORIZATION, format!("Bearer {}", token))
    //         .header(CONTENT_TYPE, format!("multipart/form-data; boundary={}", boundary))
    //         .body(Body::from(body))
    //         .unwrap();

    //     let response = app.oneshot(req).await.unwrap();
    //     assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
    //     let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    //     let json: Value = serde_json::from_slice(&body).unwrap();
    //     assert_eq!(json["success"], false);
    //     assert_eq!(json["message"], "Failed to run code for submission");
    // }

    #[tokio::test]
    #[serial]
    async fn test_failed_to_load_mark_allocator() {
        let temp_dir = setup_assignment_storage_root();
        let (app, app_state) = make_test_app().await;
        let data = setup_test_data(app_state.db(), &temp_dir).await;
        let file = create_submission_zip();

        let allocator_path = std::path::PathBuf::from(temp_dir.path())
            .join(format!("module_{}", data.module.id))
            .join(format!("assignment_{}", data.assignment.id))
            .join("mark_allocator")
            .join("allocator.json");
        std::fs::remove_file(&allocator_path)
            .expect("Failed to delete allocator.json for test setup");

        let (boundary, body) = multipart_body("solution.zip", &file, None);
        let (token, _) = generate_jwt(data.student_user.id, data.student_user.admin);
        let uri = format!(
            "/api/modules/{}/assignments/{}/submissions",
            data.module.id, data.assignment.id
        );
        let req = Request::builder()
            .method("POST")
            .uri(&uri)
            .header(AUTHORIZATION, format!("Bearer {}", token))
            .header(
                CONTENT_TYPE,
                format!("multipart/form-data; boundary={}", boundary),
            )
            .body(Body::from(body))
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let json: Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(json["success"], false);
        assert_eq!(json["message"], "Failed to load mark allocator");
    }

    #[tokio::test]
    #[serial]
    async fn test_failed_marking_due_to_broken_allocator() {
        let temp_dir = setup_assignment_storage_root();
        let (app, app_state) = make_test_app().await;
        let data = setup_test_data(app_state.db(), &temp_dir).await;
        let file = create_submission_zip();

        let base_path = temp_dir
            .path()
            .join(format!("module_{}", data.module.id))
            .join(format!("assignment_{}", data.assignment.id));
        let allocator_path = base_path.join("mark_allocator").join("allocator.json");
        let broken_allocator_content = r#"{
            "generated_at": "2025-07-21T10:00:00Z",
            "tasks": [
                {
                    "task1": {
                        "name": "Task 1 Execution",
                        "value": 10,
                        "subsections": [
                            { "name": "Subtask 1 Output", "value": 5 },
                            { "name": "Subtask 2 Output (Does Not Exist)", "value": 5 } 
                        ]
                    }
                }
            ]
        }"#;
        std::fs::write(&allocator_path, broken_allocator_content)
            .expect("Failed to write broken allocator.json for test setup");

        let (boundary, body) = multipart_body("solution.zip", &file, None);
        let (token, _) = generate_jwt(data.student_user.id, data.student_user.admin);
        let uri = format!(
            "/api/modules/{}/assignments/{}/submissions",
            data.module.id, data.assignment.id
        );
        let req = Request::builder()
            .method("POST")
            .uri(&uri)
            .header(AUTHORIZATION, format!("Bearer {}", token))
            .header(
                CONTENT_TYPE,
                format!("multipart/form-data; boundary={}", boundary),
            )
            .body(Body::from(body))
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let json: Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(json["success"], false);
        assert_eq!(json["message"], "ParseOutputError(\"Expected 2 subtasks, but found 1 delimiters\")");
    }

    // Helper function to create a submission
    async fn create_submission(
        db: &DatabaseConnection,
        module_id: i64,
        assignment_id: i64,
        user_id: i64,
        attempt: i64,
        temp_dir: &TempDir,
    ) -> AssignmentSubmissionModel {
        let submission = assignment_submission::ActiveModel {
            assignment_id: Set(assignment_id),
            user_id: Set(user_id),
            attempt: Set(attempt),
            filename: Set("test_submission.zip".to_string()),
            file_hash: Set("d41d8cd98f00b204e9800998ecf8427e".to_string()),
            path: Set(format!(
                "module_{}/assignment_{}/assignment_submissions/user_{}/attempt_{}/submission.zip",
                module_id, assignment_id, user_id, attempt
            )),
            is_practice: Set(false),
            ..Default::default()
        };
        let submission = submission.insert(db).await.unwrap();

        let output_dir = temp_dir.path()
            .join(format!("module_{}", module_id))
            .join(format!("assignment_{}", assignment_id))
            .join("assignment_submissions")
            .join(format!("user_{}", user_id))
            .join(format!("attempt_{}", attempt))
            .join("submission_output");
        
        let task_1_dir = output_dir.join("task_1");
        std::fs::create_dir_all(&task_1_dir).unwrap();

        let task_1_output = task_1_dir.join("1001.txt");
        std::fs::write(task_1_output, "make task1\n&-=-&Subtask 1 Output\nOutput A").unwrap();

        submission
    }

    // Helper to send remark request
    async fn send_remark_request(
        app: &BoxCloneService<Request<Body>, Response, Infallible>,
        token: &str,
        module_id: i64,
        assignment_id: i64,
        body: Value,
    ) -> (StatusCode, Value) {
        let req = Request::builder()
            .method("POST")
            .uri(&format!(
                "/api/modules/{}/assignments/{}/submissions/remark",
                module_id, assignment_id
            ))
            .header(AUTHORIZATION, format!("Bearer {}", token))
            .header(CONTENT_TYPE, "application/json")
            .body(Body::from(body.to_string()))
            .unwrap();

        let response = app.clone().oneshot(req).await.unwrap();
        let status = response.status();
        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let json: Value = serde_json::from_slice(&body).unwrap();
        (status, json)
    }

    /// Test 1: Successful remark of specific submissions as lecturer
    #[tokio::test]
    #[serial]
    async fn test_remark_specific_submissions_as_lec() {
        let temp_dir = setup_assignment_storage_root();
        let (app, app_state) = make_test_app().await;
        let db = app_state.db();
        let data = setup_test_data(app_state.db(), &temp_dir).await;
        
        let lecturer = UserModel::create(db, "lecturer1", "lecturer@test.com", "password", false).await.unwrap();
        UserModuleRoleModel::assign_user_to_module(db, lecturer.id, data.module.id, Role::Lecturer).await.unwrap();
        
        let sub1 = create_submission(db, data.assignment.module_id, data.assignment.id, data.student_user.id, 1, &temp_dir).await;
        let sub2 = create_submission(db, data.assignment.module_id, data.assignment.id, data.student_user.id, 2, &temp_dir).await;
        
        let (token, _) = generate_jwt(lecturer.id, lecturer.admin);
        let body = json!({
            "submission_ids": [sub1.id, sub2.id]
        });
        
        let (status, json) = send_remark_request(&app, &token, data.module.id, data.assignment.id, body).await;
        
        assert_eq!(status, StatusCode::OK);
        assert_eq!(json["success"], true);
        assert_eq!(json["data"]["regraded"], 2);
        assert_eq!(json["data"]["failed"].as_array().unwrap().len(), 0);
    }

     /// Test 1: Successful remark of specific submissions as assistant lecturer
    #[tokio::test]
    #[serial]
    async fn test_remark_specific_submissions_as_al() {
        let temp_dir = setup_assignment_storage_root();
        let (app, app_state) = make_test_app().await;
        let db = app_state.db();
        let data = setup_test_data(app_state.db(), &temp_dir).await;
        
        let assistant = UserModel::create(db, "assistant1", "assistant@test.com", "password", false).await.unwrap();
        UserModuleRoleModel::assign_user_to_module(db, assistant.id, data.module.id, Role::AssistantLecturer).await.unwrap();
        
        let sub1 = create_submission(db, data.assignment.module_id, data.assignment.id, data.student_user.id, 1, &temp_dir).await;
        let sub2 = create_submission(db, data.assignment.module_id, data.assignment.id, data.student_user.id, 2, &temp_dir).await;
        
        let (token, _) = generate_jwt(assistant.id, assistant.admin);
        let body = json!({
            "submission_ids": [sub1.id, sub2.id]
        });
        
        let (status, json) = send_remark_request(&app, &token, data.module.id, data.assignment.id, body).await;
        
        assert_eq!(status, StatusCode::OK);
        assert_eq!(json["success"], true);
        assert_eq!(json["data"]["regraded"], 2);
        assert_eq!(json["data"]["failed"].as_array().unwrap().len(), 0);
    }

    /// Test 2: Successful remark of all submissions as lecturer
    #[tokio::test]
    #[serial]
    async fn test_remark_all_submissions_as_lec() {
        let temp_dir = setup_assignment_storage_root();
        let (app, app_state) = make_test_app().await;
        let db = app_state.db();
        let data = setup_test_data(app_state.db(), &temp_dir).await;
    
        let lecturer = UserModel::create(db, "lecturer1", "lecturer@test.com", "password", false).await.unwrap();
        UserModuleRoleModel::assign_user_to_module(db, lecturer.id, data.module.id, Role::Lecturer).await.unwrap();
        
        create_submission(db, data.assignment.module_id, data.assignment.id, data.student_user.id, 1, &temp_dir).await;
        create_submission(db, data.assignment.module_id, data.assignment.id, data.student_user.id, 2, &temp_dir).await;
        
        let (token, _) = generate_jwt(lecturer.id, lecturer.admin);
        let body = json!({
            "all": true
        });
        
        let (status, json) = send_remark_request(&app, &token, data.module.id, data.assignment.id, body).await;
        
        assert_eq!(status, StatusCode::OK);
        assert_eq!(json["success"], true);
        assert_eq!(json["data"]["regraded"], 2);
        assert_eq!(json["data"]["failed"].as_array().unwrap().len(), 0);
    }

    /// Test 2: Successful remark of all submissions as assistant lecturer
    #[tokio::test]
    #[serial]
    async fn test_remark_all_submissions() {
        let temp_dir = setup_assignment_storage_root();
        let (app, app_state) = make_test_app().await;
        let db = app_state.db();
        let data = setup_test_data(app_state.db(), &temp_dir).await;
        
        let assistant = UserModel::create(db, "assistant1", "assistant@test.com", "password", false).await.unwrap();
        UserModuleRoleModel::assign_user_to_module(db, assistant.id, data.module.id, Role::AssistantLecturer).await.unwrap();
        
        create_submission(db, data.assignment.module_id, data.assignment.id, data.student_user.id, 1, &temp_dir).await;
        create_submission(db, data.assignment.module_id, data.assignment.id, data.student_user.id, 2, &temp_dir).await;
        
        let (token, _) = generate_jwt(assistant.id, assistant.admin);
        let body = json!({
            "all": true
        });
        
        let (status, json) = send_remark_request(&app, &token, data.module.id, data.assignment.id, body).await;
        
        assert_eq!(status, StatusCode::OK);
        assert_eq!(json["success"], true);
        assert_eq!(json["data"]["regraded"], 2);
        assert_eq!(json["data"]["failed"].as_array().unwrap().len(), 0);
    }

    /// Test 3: Missing both submission_ids and all=true
    #[tokio::test]
    #[serial]
    async fn test_remark_missing_parameters() {
        let temp_dir = setup_assignment_storage_root();
        let (app, app_state) = make_test_app().await;
        let db = app_state.db();
        let data = setup_test_data(app_state.db(), &temp_dir).await;
        
        let admin = UserModel::create(db, "admin1", "admin@test.com", "password", true).await.unwrap();
        
        let (token, _) = generate_jwt(admin.id, admin.admin);
        let body = json!({});
        
        let (status, json) = send_remark_request(&app, &token, data.module.id, data.assignment.id, body).await;
        
        assert_eq!(status, StatusCode::BAD_REQUEST);
        assert_eq!(json["success"], false);
        assert_eq!(json["message"], "Must provide either submission_ids or all=true");
    }

    /// Test 4: Unauthorized user (student)
    #[tokio::test]
    #[serial]
    async fn test_remark_unauthorized_user() {
        let temp_dir = setup_assignment_storage_root();
        let (app, app_state) = make_test_app().await;
        let db = app_state.db();
        let data = setup_test_data(app_state.db(), &temp_dir).await;
        
        let submission = create_submission(db, data.assignment.module_id, data.assignment.id, data.student_user.id, 1, &temp_dir).await;
        
        let (token, _) = generate_jwt(data.student_user.id, data.student_user.admin);
        let body = json!({
            "submission_ids": [submission.id]
        });
        
        let (status, json) = send_remark_request(&app, &token, data.module.id, data.assignment.id, body).await;
        
        assert_eq!(status, StatusCode::FORBIDDEN);
        assert_eq!(json["success"], false);
        assert_eq!(json["message"], "Lecturer or assistant lecturer access required for this module");
    }

    /// Test 5: Assignment not found
    #[tokio::test]
    #[serial]
    async fn test_remark_assignment_not_found() {
        let temp_dir = setup_assignment_storage_root();
        let (app, app_state) = make_test_app().await;
        let db = app_state.db();
        let data = setup_test_data(app_state.db(), &temp_dir).await;
        
        let lecturer = UserModel::create(db, "lecturer1", "lecturer@test.com", "password", false).await.unwrap();
        UserModuleRoleModel::assign_user_to_module(db, lecturer.id, data.module.id, Role::Lecturer).await.unwrap();
        
        let (token, _) = generate_jwt(lecturer.id, lecturer.admin);
        let body = json!({
            "all": true
        });
        
        let (status, json) = send_remark_request(&app, &token, data.module.id, 9999, body).await;
        
        assert_eq!(status, StatusCode::NOT_FOUND);
        assert_eq!(json["success"], false);
        assert_eq!(json["message"], "Assignment 9999 in Module 1 not found.");
    }

    /// Test 6: Submission not found in assignment
    #[tokio::test]
    #[serial]
    async fn test_remark_submission_not_found() {
        let temp_dir = setup_assignment_storage_root();
        let (app, app_state) = make_test_app().await;
        let db = app_state.db();
        let data = setup_test_data(app_state.db(), &temp_dir).await;

        let lecturer = UserModel::create(db, "lecturer1", "lecturer@test.com", "password", false).await.unwrap();
        UserModuleRoleModel::assign_user_to_module(db, lecturer.id, data.module.id, Role::Lecturer).await.unwrap();
        
        let (token, _) = generate_jwt(lecturer.id, lecturer.admin);
        let body = json!({
            "submission_ids": [9999]
        });
        
        let (status, json) = send_remark_request(&app, &token, data.module.id, data.assignment.id, body).await;
        
        assert_eq!(status, StatusCode::OK);
        assert_eq!(json["success"], true);
        let failed = json["data"]["failed"].as_array().unwrap();
        assert_eq!(failed.len(), 1);
        assert_eq!(failed[0]["id"], 9999);
        assert_eq!(failed[0]["error"], "Submission not found");
    }

    /// Test 7: Failed to load mark allocator
    #[tokio::test]
    #[serial]
    async fn test_remark_failed_allocator() {
        let temp_dir = setup_assignment_storage_root();
        let (app, app_state) = make_test_app().await;
        let db = app_state.db();
        let data = setup_test_data(app_state.db(), &temp_dir).await;
    
        let lecturer = UserModel::create(db, "lecturer1", "lecturer@test.com", "password", false).await.unwrap();
        UserModuleRoleModel::assign_user_to_module(db, lecturer.id, data.module.id, Role::Lecturer).await.unwrap();
    
        let submission = create_submission(db, data.assignment.module_id, data.assignment.id, data.student_user.id, 1, &temp_dir).await;

        let allocator_path = temp_dir
            .path()
            .join(format!("module_{}", data.module.id))
            .join(format!("assignment_{}", data.assignment.id))
            .join("mark_allocator")
            .join("allocator.json");
        fs::remove_file(allocator_path).unwrap();
        
        let (token, _) = generate_jwt(lecturer.id, lecturer.admin);
        let body = json!({
            "submission_ids": [submission.id]
        });
        
        let (status, json) = send_remark_request(&app, &token, data.module.id, data.assignment.id, body).await;
        
        assert_eq!(status, StatusCode::INTERNAL_SERVER_ERROR);
        assert_eq!(json["success"], false);
        assert!(json["message"].as_str().unwrap().contains("Failed to load mark allocator"));
    }

    /// Test 8: Failed during grading of a submission
    #[tokio::test]
    #[serial]
    async fn test_remark_grading_failure() {
        let temp_dir = setup_assignment_storage_root();
        let (app, app_state) = make_test_app().await;
        let db = app_state.db();
        let data = setup_test_data(app_state.db(), &temp_dir).await;
        
        let lecturer = UserModel::create(db, "lecturer1", "lecturer@test.com", "password", false).await.unwrap();
        UserModuleRoleModel::assign_user_to_module(db, lecturer.id, data.module.id, Role::Lecturer).await.unwrap();
        
        let submission = create_submission(db, data.assignment.module_id, data.assignment.id, data.student_user.id, 1, &temp_dir).await;
        
        let student_output_dir = temp_dir
            .path()
            .join(format!("module_{}", data.module.id))
            .join(format!("assignment_{}", data.assignment.id))
            .join("assignment_submissions")
            .join(format!("user_{}", data.student_user.id))
            .join("attempt_1")
            .join("submission_output");
        fs::remove_dir_all(student_output_dir).unwrap();
        
        let (token, _) = generate_jwt(lecturer.id, lecturer.admin);
        let body = json!({
            "submission_ids": [submission.id]
        });
        
        let (status, json) = send_remark_request(&app, &token, data.module.id, data.assignment.id, body).await;
        
        assert_eq!(status, StatusCode::OK);
        assert_eq!(json["success"], true);
        let failed = json["data"]["failed"].as_array().unwrap();
        assert_eq!(failed.len(), 1);
        assert_eq!(failed[0]["id"], submission.id);
        assert!(
            failed[0]["error"].as_str().unwrap().contains("memo_paths.len() != student_paths.len()"),
            "Actual error: {}",
            failed[0]["error"]
        );
    }

    /// Test 9: Mix of successful and failed regrades
    #[tokio::test]
    #[serial]
    async fn test_remark_mixed_results() {
        let temp_dir = setup_assignment_storage_root();
        let (app, app_state) = make_test_app().await;
        let db = app_state.db();
        let data = setup_test_data(app_state.db(), &temp_dir).await;
    
        let lecturer = UserModel::create(db, "lecturer1", "lecturer@test.com", "password", false).await.unwrap();
        UserModuleRoleModel::assign_user_to_module(db, lecturer.id, data.module.id, Role::Lecturer).await.unwrap();
        
        let valid_sub = create_submission(db, data.assignment.module_id, data.assignment.id, data.student_user.id, 1, &temp_dir).await;
        let invalid_sub_id = 9999;
        
        let broken_sub = create_submission(db, data.assignment.module_id, data.assignment.id, data.student_user.id, 2, &temp_dir).await;
        let student_output_dir = temp_dir
            .path()
            .join(format!("module_{}", data.module.id))
            .join(format!("assignment_{}", data.assignment.id))
            .join("assignment_submissions")
            .join(format!("user_{}", data.student_user.id))
            .join("attempt_2")
            .join("submission_output");
        fs::remove_dir_all(student_output_dir).unwrap();
        
        let (token, _) = generate_jwt(lecturer.id, lecturer.admin);
        let body = json!({
            "submission_ids": [valid_sub.id, invalid_sub_id, broken_sub.id]
        });
        
        let (status, json) = send_remark_request(&app, &token, data.module.id, data.assignment.id, body).await;
        
        assert_eq!(status, StatusCode::OK);
        assert_eq!(json["success"], true);
        assert_eq!(json["data"]["regraded"], 1);
        let failed = json["data"]["failed"].as_array().unwrap();
        assert_eq!(failed.len(), 2);

        let errors: Vec<&str> = failed
            .iter()
            .map(|f| f["error"].as_str().unwrap())
            .collect();
        assert!(errors.contains(&"Submission not found"));
        assert!(
            errors.iter().any(|e| e.contains("memo_paths.len() != student_paths.len()")),
            "Errors: {:?}",
            errors
        );
    }

    /// Test 10: Invalid module ID
    #[tokio::test]
    #[serial]
    async fn test_remark_invalid_module() {
        let temp_dir = setup_assignment_storage_root();
        let (app, app_state) = make_test_app().await;
        let db = app_state.db();
        let data = setup_test_data(app_state.db(), &temp_dir).await;

        let lecturer = UserModel::create(db, "lecturer1", "lecturer@test.com", "password", false).await.unwrap();
        UserModuleRoleModel::assign_user_to_module(db, lecturer.id, data.module.id, Role::Lecturer).await.unwrap();
        
        let (token, _) = generate_jwt(lecturer.id, lecturer.admin);
        let body = json!({
            "all": true
        });

        let (status, json) = send_remark_request(&app, &token, 9999, data.assignment.id, body).await;
        
        assert_eq!(status, StatusCode::NOT_FOUND);
        assert_eq!(json["success"], false);
        assert_eq!(json["message"], "Module 9999 not found.");
    }
}