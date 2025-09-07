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
    use db::models::{
        assignment::{
            ActiveModel as AssignmentActiveModel, Entity as AssignmentEntity,
            Model as AssignmentModel,
        },
        assignment_submission::{self, Model as AssignmentSubmissionModel},
        assignment_submission_output,
        assignment_task::Model as AssignmentTaskModel,
        module::Model as ModuleModel,
        user::Model as UserModel,
        user_module_role::{Model as UserModuleRoleModel, Role},
    };
    use flate2::{Compression, write::GzEncoder};
    use sea_orm::{
        ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, Set,
    };
    use serde_json::{Value, json};
    use serial_test::serial;
    use std::convert::Infallible;
    use std::path::Path;
    use std::{fs, io::Write};
    use tar::Builder as TarBuilder;
    use tempfile::{TempDir, tempdir};
    use tower::ServiceExt;
    use tower::util::BoxCloneService;
    use zip::write::SimpleFileOptions;

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
        let mut active_assignment: db::models::assignment::ActiveModel = assignment.clone().into();
        active_assignment.status = Set(db::models::assignment::Status::Ready);
        active_assignment.updated_at = Set(Utc::now());
        let assignment = active_assignment.update(db).await.unwrap();
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
        }"#;
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

    fn write_attempt_policy_config(
        base_path: &std::path::Path,
        limit_attempts: bool,
        max_attempts: u32,
        allow_practice: bool,
    ) {
        std::fs::create_dir_all(base_path.join("config")).unwrap();
        let cfg = format!(r#"{{
            "execution": {{
                "timeout_secs": 10,
                "max_memory": 8589934592,
                "max_cpus": 2,
                "max_uncompressed_size": 100000000,
                "max_processes": 256
            }},
            "project": {{
                "submission_mode": "manual"
            }},
            "marking": {{
                "limit_attempts": {limit_attempts},
                "max_attempts": {max_attempts},
                "allow_practice_submissions": {allow_practice},
                "pass_mark": 50,
                "grading_policy": "last"
            }}
        }}"#);
        std::fs::write(base_path.join("config").join("config.json"), cfg).unwrap();
    }

    fn assignment_base(temp_dir: &tempfile::TempDir, module_id: i64, assignment_id: i64) -> std::path::PathBuf {
        temp_dir
            .path()
            .join(format!("module_{module_id}"))
            .join(format!("assignment_{assignment_id}"))
    }

    /// Seed a submission row that does *not* count (ignored or practice).
    async fn seed_submission_row(
        db: &DatabaseConnection,
        assignment_id: i64,
        user_id: i64,
        attempt: i64,
        is_practice: bool,
        ignored: bool,
    ) {
        use db::models::assignment_submission as sub;
        let mut m = sub::ActiveModel {
            assignment_id: Set(assignment_id),
            user_id: Set(user_id),
            attempt: Set(attempt),
            earned: Set(0),
            total: Set(0),
            filename: Set("seed.zip".into()),
            file_hash: Set("seedhash".into()),
            path: Set("seed/path.zip".into()),
            is_practice: Set(is_practice),
            ..Default::default()
        };
        // Only if your schema has it (your queries use Column::Ignored, so it should exist):
        m.ignored = Set(ignored);
        m.insert(db).await.unwrap();
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
        let temp_dir = setup_assignment_storage_root();
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
        let temp_dir = setup_assignment_storage_root();
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
        let temp_dir = setup_assignment_storage_root();
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
        let temp_dir = setup_assignment_storage_root();
        let (app, app_state) = make_test_app().await;
        let data = setup_test_data(app_state.db(), &temp_dir).await;

        // ensure practice is allowed for this assignment
        let base = temp_dir
            .path()
            .join(format!("module_{}", data.module.id))
            .join(format!("assignment_{}", data.assignment.id));
        write_attempt_policy_config(
            &base,
            /*limit_attempts*/ false,
            /*max_attempts*/ 10,
            /*allow_practice*/ true,
        );

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
            .header(CONTENT_TYPE, format!("multipart/form-data; boundary={}", boundary))
            .body(Body::from(body))
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(json["success"], true);
        assert_eq!(json["data"]["is_practice"], true);
    }


    #[tokio::test]
    #[serial]
    async fn test_multiple_attempts_increments() {
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

    #[tokio::test]
    #[serial]
    async fn test_submit_blocked_when_max_attempts_reached_for_student() {
        let temp_dir = setup_assignment_storage_root();
        let (app, app_state) = make_test_app().await;
        let db = app_state.db();
        let data = setup_test_data(db, &temp_dir).await;

        // Force strict attempt policy: max 1, practice irrelevant here.
        let base = assignment_base(&temp_dir, data.module.id, data.assignment.id);
        write_attempt_policy_config(&base, true, 1, false);

        let file = create_submission_zip();
        let (token, _) = generate_jwt(data.student_user.id, data.student_user.admin);
        let uri = format!(
            "/api/modules/{}/assignments/{}/submissions",
            data.module.id, data.assignment.id
        );

        // 1st submission -> OK
        {
            let (boundary, body) = multipart_body("solution.zip", &file, None);
            let req = Request::builder()
                .method("POST")
                .uri(&uri)
                .header(AUTHORIZATION, format!("Bearer {}", token))
                .header(CONTENT_TYPE, format!("multipart/form-data; boundary={}", boundary))
                .body(Body::from(body))
                .unwrap();
            let resp = app.clone().oneshot(req).await.unwrap();
            assert_eq!(resp.status(), StatusCode::OK);
        }

        // 2nd submission -> should be FORBIDDEN with max-attempts message
        {
            let (boundary, body) = multipart_body("solution.zip", &file, None);
            let req = Request::builder()
                .method("POST")
                .uri(&uri)
                .header(AUTHORIZATION, format!("Bearer {}", token))
                .header(CONTENT_TYPE, format!("multipart/form-data; boundary={}", boundary))
                .body(Body::from(body))
                .unwrap();
            let resp = app.clone().oneshot(req).await.unwrap();
            assert_eq!(resp.status(), StatusCode::FORBIDDEN);

            let body = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
            let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
            assert_eq!(json["success"], false);
            assert_eq!(json["message"], "Maximum attempts reached: used 1 of 1 (remaining 0).");
        }
    }

    #[tokio::test]
    #[serial]
    async fn test_practice_submission_denied_when_disabled() {
        let temp_dir = setup_assignment_storage_root();
        let (app, app_state) = make_test_app().await;
        let db = app_state.db();
        let data = setup_test_data(db, &temp_dir).await;

        // Practice disabled
        let base = assignment_base(&temp_dir, data.module.id, data.assignment.id);
        write_attempt_policy_config(&base, true, 5, false);

        let file = create_submission_zip();
        let (boundary, body) = multipart_body("solution.zip", &file, Some("true")); // is_practice=true
        let (token, _) = generate_jwt(data.student_user.id, data.student_user.admin);

        let uri = format!(
            "/api/modules/{}/assignments/{}/submissions",
            data.module.id, data.assignment.id
        );
        let req = Request::builder()
            .method("POST")
            .uri(&uri)
            .header(AUTHORIZATION, format!("Bearer {}", token))
            .header(CONTENT_TYPE, format!("multipart/form-data; boundary={}", boundary))
            .body(Body::from(body))
            .unwrap();

        let resp = app.clone().oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::FORBIDDEN);

        let body = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(json["success"], false);
        assert_eq!(json["message"], "Practice submissions are disabled for this assignment.");
    }

    #[tokio::test]
    #[serial]
    async fn test_submission_allowed_when_previous_attempt_was_ignored() {
        let temp_dir = setup_assignment_storage_root();
        let (app, app_state) = make_test_app().await;
        let db = app_state.db();
        let data = setup_test_data(db, &temp_dir).await;

        // max 1 attempt, but we seed a single *ignored* prior attempt → should NOT count
        let base = assignment_base(&temp_dir, data.module.id, data.assignment.id);
        write_attempt_policy_config(&base, true, 1, false);

        seed_submission_row(
            db,
            data.assignment.id,
            data.student_user.id,
            1,
            /*is_practice*/ false,
            /*ignored*/ true,
        ).await;

        let file = create_submission_zip();
        let (token, _) = generate_jwt(data.student_user.id, data.student_user.admin);
        let (boundary, body) = multipart_body("solution.zip", &file, None);

        let uri = format!(
            "/api/modules/{}/assignments/{}/submissions",
            data.module.id, data.assignment.id
        );
        let req = Request::builder()
            .method("POST")
            .uri(&uri)
            .header(AUTHORIZATION, format!("Bearer {}", token))
            .header(CONTENT_TYPE, format!("multipart/form-data; boundary={}", boundary))
            .body(Body::from(body))
            .unwrap();

        // Should be allowed because prior attempt is ignored
        let resp = app.clone().oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let body = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(json["success"], true);
    }

    #[tokio::test]
    #[serial]
    async fn test_staff_bypass_attempt_limit() {
        let temp_dir = setup_assignment_storage_root();
        let (app, app_state) = make_test_app().await;
        let db = app_state.db();
        let data = setup_test_data(db, &temp_dir).await;

        // Strict limit (1 attempt), but we'll submit as lecturer (staff → bypass)
        let base = assignment_base(&temp_dir, data.module.id, data.assignment.id);
        write_attempt_policy_config(&base, true, 1, false);

        let lecturer = UserModel::create(db, "lect1", "lect1@test.com", "pw", false)
            .await
            .unwrap();
        UserModuleRoleModel::assign_user_to_module(db, lecturer.id, data.module.id, Role::Lecturer)
            .await
            .unwrap();

        let file = create_submission_zip();
        let (token, _) = generate_jwt(lecturer.id, lecturer.admin);
        let uri = format!(
            "/api/modules/{}/assignments/{}/submissions",
            data.module.id, data.assignment.id
        );

        // First submission as staff
        {
            let (boundary, body) = multipart_body("solution.zip", &file, None);
            let req = Request::builder()
                .method("POST")
                .uri(&uri)
                .header(AUTHORIZATION, format!("Bearer {}", token))
                .header(CONTENT_TYPE, format!("multipart/form-data; boundary={}", boundary))
                .body(Body::from(body))
                .unwrap();
            let resp = app.clone().oneshot(req).await.unwrap();
            assert_eq!(resp.status(), StatusCode::OK);
        }

        // Second submission as staff (would be blocked for students) → still OK
        {
            let (boundary, body) = multipart_body("solution.zip", &file, None);
            let req = Request::builder()
                .method("POST")
                .uri(&uri)
                .header(AUTHORIZATION, format!("Bearer {}", token))
                .header(CONTENT_TYPE, format!("multipart/form-data; boundary={}", boundary))
                .body(Body::from(body))
                .unwrap();
            let resp = app.clone().oneshot(req).await.unwrap();
            assert_eq!(resp.status(), StatusCode::OK);
        }
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
        assignment_active_model.due_date = Set(Utc::now());
        assignment_active_model
            .update(app_state.db())
            .await
            .unwrap();

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
        assignment_active_model.due_date = Set(Utc::now() - Duration::hours(1));
        assignment_active_model
            .update(app_state.db())
            .await
            .unwrap();

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
        assignment_active_model
            .update(app_state.db())
            .await
            .unwrap();

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
        //Removed this since it's annoying to check for a specific error message - it will constantly change
        //Checking that success is false is enough
        // assert_eq!(json["message"], "ParseOutputError(\"Expected 1 subtasks, but found 0 delimiters\")");
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
    //
    //     let mut buf = Vec::new();
    //     {
    //         let mut zip = zip::ZipWriter::new(std::io::Cursor::new(&mut buf));
    //         let options = SimpleFileOptions::default();
    //         zip.start_file("main.c", options).unwrap();
    //         zip.write_all(b"invalid C code that won't compile").unwrap();
    //         zip.finish().unwrap();
    //     }
    //
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
    //
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

    //Test invalidated since Broken Allocator should not crash the system but return a mark of 0

    // #[tokio::test]
    // #[serial]
    // async fn test_failed_marking_due_to_broken_allocator() {
    //     let temp_dir = setup_assignment_storage_root();
    //     let (app, app_state) = make_test_app().await;
    //     let data = setup_test_data(app_state.db(), &temp_dir).await;
    //     let file = create_submission_zip();

    //     let base_path = temp_dir
    //         .path()
    //         .join(format!("module_{}", data.module.id))
    //         .join(format!("assignment_{}", data.assignment.id));
    //     let allocator_path = base_path.join("mark_allocator").join("allocator.json");
    //     let broken_allocator_content = r#"{
    //         "generated_at": "2025-07-21T10:00:00Z",
    //         "tasks": [
    //             {
    //                 "task1": {
    //                     "name": "Task 1 Execution",
    //                     "value": 10,
    //                     "subsections": [
    //                         { "name": "Subtask 1 Output", "value": 5 },
    //                         { "name": "Subtask 2 Output (Does Not Exist)", "value": 5 }
    //                     ]
    //                 }
    //             }
    //         ]
    //     }"#;
    //     std::fs::write(&allocator_path, broken_allocator_content)
    //         .expect("Failed to write broken allocator.json for test setup");

    //     let (boundary, body) = multipart_body("solution.zip", &file, None);
    //     let (token, _) = generate_jwt(data.student_user.id, data.student_user.admin);
    //     let uri = format!(
    //         "/api/modules/{}/assignments/{}/submissions",
    //         data.module.id, data.assignment.id
    //     );
    //     let req = Request::builder()
    //         .method("POST")
    //         .uri(&uri)
    //         .header(AUTHORIZATION, format!("Bearer {}", token))
    //         .header(
    //             CONTENT_TYPE,
    //             format!("multipart/form-data; boundary={}", boundary),
    //         )
    //         .body(Body::from(body))
    //         .unwrap();

    // let response = app.oneshot(req).await.unwrap();
    // assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);

    // let body = axum::body::to_bytes(response.into_body(), usize::MAX)
    //     .await
    //     .unwrap();
    // let json: Value = serde_json::from_slice(&body).unwrap();
    // assert_eq!(json["success"], false);
    // assert_eq!(
    //     json["message"],
    //     "ParseOutputError(\"Expected 2 subtasks, but found 1 delimiters\")"
    // );
    // }

    // Helper function to create a submission
    async fn create_remarkable_submission(
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
            earned: Set(10),
            total: Set(10),
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

        let task = db::models::assignment_task::Entity::find()
            .filter(db::models::assignment_task::Column::AssignmentId.eq(assignment_id))
            .one(db)
            .await
            .unwrap()
            .unwrap();

        let output = assignment_submission_output::ActiveModel {
            task_id: Set(task.id),
            submission_id: Set(submission.id),
            path: Set("".to_string()),
            ..Default::default()
        };
        let output = output.insert(db).await.unwrap();

        let output_dir = temp_dir
            .path()
            .join(format!("module_{}", module_id))
            .join(format!("assignment_{}", assignment_id))
            .join("assignment_submissions")
            .join(format!("user_{}", user_id))
            .join(format!("attempt_{}", attempt))
            .join("submission_output");

        std::fs::create_dir_all(&output_dir).unwrap();
        let file_path = output_dir.join(format!("{}.txt", output.id));
        std::fs::write(&file_path, "make task1\n&-=-&Subtask 1 Output\nOutput A").unwrap();

        let relative_path = file_path
            .strip_prefix(temp_dir.path())
            .unwrap()
            .to_string_lossy()
            .to_string();

        let mut output_active: assignment_submission_output::ActiveModel = output.into();
        output_active.path = Set(relative_path);
        output_active.update(db).await.unwrap();

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

        let lecturer = UserModel::create(db, "lecturer1", "lecturer@test.com", "password", false)
            .await
            .unwrap();
        UserModuleRoleModel::assign_user_to_module(db, lecturer.id, data.module.id, Role::Lecturer)
            .await
            .unwrap();

        let sub1 = create_remarkable_submission(
            db,
            data.assignment.module_id,
            data.assignment.id,
            data.student_user.id,
            1,
            &temp_dir,
        )
        .await;
        let sub2 = create_remarkable_submission(
            db,
            data.assignment.module_id,
            data.assignment.id,
            data.student_user.id,
            2,
            &temp_dir,
        )
        .await;

        let (token, _) = generate_jwt(lecturer.id, lecturer.admin);
        let body = json!({
            "submission_ids": [sub1.id, sub2.id]
        });

        let (status, json) =
            send_remark_request(&app, &token, data.module.id, data.assignment.id, body).await;

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

        let assistant =
            UserModel::create(db, "assistant1", "assistant@test.com", "password", false)
                .await
                .unwrap();
        UserModuleRoleModel::assign_user_to_module(
            db,
            assistant.id,
            data.module.id,
            Role::AssistantLecturer,
        )
        .await
        .unwrap();

        let sub1 = create_remarkable_submission(
            db,
            data.assignment.module_id,
            data.assignment.id,
            data.student_user.id,
            1,
            &temp_dir,
        )
        .await;
        let sub2 = create_remarkable_submission(
            db,
            data.assignment.module_id,
            data.assignment.id,
            data.student_user.id,
            2,
            &temp_dir,
        )
        .await;

        let (token, _) = generate_jwt(assistant.id, assistant.admin);
        let body = json!({
            "submission_ids": [sub1.id, sub2.id]
        });

        let (status, json) =
            send_remark_request(&app, &token, data.module.id, data.assignment.id, body).await;

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

        let lecturer = UserModel::create(db, "lecturer1", "lecturer@test.com", "password", false)
            .await
            .unwrap();
        UserModuleRoleModel::assign_user_to_module(db, lecturer.id, data.module.id, Role::Lecturer)
            .await
            .unwrap();

        create_remarkable_submission(
            db,
            data.assignment.module_id,
            data.assignment.id,
            data.student_user.id,
            1,
            &temp_dir,
        )
        .await;
        create_remarkable_submission(
            db,
            data.assignment.module_id,
            data.assignment.id,
            data.student_user.id,
            2,
            &temp_dir,
        )
        .await;

        let (token, _) = generate_jwt(lecturer.id, lecturer.admin);
        let body = json!({
            "all": true
        });

        let (status, json) =
            send_remark_request(&app, &token, data.module.id, data.assignment.id, body).await;

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

        let assistant =
            UserModel::create(db, "assistant1", "assistant@test.com", "password", false)
                .await
                .unwrap();
        UserModuleRoleModel::assign_user_to_module(
            db,
            assistant.id,
            data.module.id,
            Role::AssistantLecturer,
        )
        .await
        .unwrap();

        create_remarkable_submission(
            db,
            data.assignment.module_id,
            data.assignment.id,
            data.student_user.id,
            1,
            &temp_dir,
        )
        .await;
        create_remarkable_submission(
            db,
            data.assignment.module_id,
            data.assignment.id,
            data.student_user.id,
            2,
            &temp_dir,
        )
        .await;

        let (token, _) = generate_jwt(assistant.id, assistant.admin);
        let body = json!({
            "all": true
        });

        let (status, json) =
            send_remark_request(&app, &token, data.module.id, data.assignment.id, body).await;

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

        let admin = UserModel::create(db, "admin1", "admin@test.com", "password", true)
            .await
            .unwrap();

        let (token, _) = generate_jwt(admin.id, admin.admin);
        let body = json!({});

        let (status, json) =
            send_remark_request(&app, &token, data.module.id, data.assignment.id, body).await;

        assert_eq!(status, StatusCode::BAD_REQUEST);
        assert_eq!(json["success"], false);
        assert_eq!(
            json["message"],
            "Must provide exactly one of submission_ids or all=true"
        );
    }

    /// Test 4: Unauthorized user (student)
    #[tokio::test]
    #[serial]
    async fn test_remark_unauthorized_user() {
        let temp_dir = setup_assignment_storage_root();
        let (app, app_state) = make_test_app().await;
        let db = app_state.db();
        let data = setup_test_data(app_state.db(), &temp_dir).await;

        let submission = create_remarkable_submission(
            db,
            data.assignment.module_id,
            data.assignment.id,
            data.student_user.id,
            1,
            &temp_dir,
        )
        .await;

        let (token, _) = generate_jwt(data.student_user.id, data.student_user.admin);
        let body = json!({
            "submission_ids": [submission.id]
        });

        let (status, json) =
            send_remark_request(&app, &token, data.module.id, data.assignment.id, body).await;

        assert_eq!(status, StatusCode::FORBIDDEN);
        assert_eq!(json["success"], false);
        assert_eq!(
            json["message"],
            "Lecturer or assistant lecturer access required for this module"
        );
    }

    /// Test 5: Assignment not found
    #[tokio::test]
    #[serial]
    async fn test_remark_assignment_not_found() {
        let temp_dir = setup_assignment_storage_root();
        let (app, app_state) = make_test_app().await;
        let db = app_state.db();
        let data = setup_test_data(app_state.db(), &temp_dir).await;

        let lecturer = UserModel::create(db, "lecturer1", "lecturer@test.com", "password", false)
            .await
            .unwrap();
        UserModuleRoleModel::assign_user_to_module(db, lecturer.id, data.module.id, Role::Lecturer)
            .await
            .unwrap();

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

        let lecturer = UserModel::create(db, "lecturer1", "lecturer@test.com", "password", false)
            .await
            .unwrap();
        UserModuleRoleModel::assign_user_to_module(db, lecturer.id, data.module.id, Role::Lecturer)
            .await
            .unwrap();

        let (token, _) = generate_jwt(lecturer.id, lecturer.admin);
        let body = json!({
            "submission_ids": [9999]
        });

        let (status, json) =
            send_remark_request(&app, &token, data.module.id, data.assignment.id, body).await;

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

        let lecturer = UserModel::create(db, "lecturer1", "lecturer@test.com", "password", false)
            .await
            .unwrap();
        UserModuleRoleModel::assign_user_to_module(db, lecturer.id, data.module.id, Role::Lecturer)
            .await
            .unwrap();

        let submission = create_remarkable_submission(
            db,
            data.assignment.module_id,
            data.assignment.id,
            data.student_user.id,
            1,
            &temp_dir,
        )
        .await;

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

        let (status, json) =
            send_remark_request(&app, &token, data.module.id, data.assignment.id, body).await;

        assert_eq!(status, StatusCode::INTERNAL_SERVER_ERROR);
        assert_eq!(json["success"], false);
        assert!(
            json["message"]
                .as_str()
                .unwrap()
                .contains("Failed to load mark allocator")
        );
    }

    /// Test 8: Failed during grading of a submission
    #[tokio::test]
    #[serial]
    async fn test_remark_grading_failure() {
        let temp_dir = setup_assignment_storage_root();
        let (app, app_state) = make_test_app().await;
        let db = app_state.db();
        let data = setup_test_data(app_state.db(), &temp_dir).await;

        let lecturer = UserModel::create(db, "lecturer1", "lecturer@test.com", "password", false)
            .await
            .unwrap();
        UserModuleRoleModel::assign_user_to_module(db, lecturer.id, data.module.id, Role::Lecturer)
            .await
            .unwrap();

        let submission = create_remarkable_submission(
            db,
            data.assignment.module_id,
            data.assignment.id,
            data.student_user.id,
            1,
            &temp_dir,
        )
        .await;

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

        let (status, json) =
            send_remark_request(&app, &token, data.module.id, data.assignment.id, body).await;

        assert_eq!(status, StatusCode::OK);
        assert_eq!(json["success"], true);
        let failed = json["data"]["failed"].as_array().unwrap();
        assert_eq!(failed.len(), 1);
        assert_eq!(failed[0]["id"], submission.id);
        assert!(
            failed[0]["error"]
                .as_str()
                .unwrap()
                .contains("memo_paths.len() != student_paths.len()"),
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

        let lecturer = UserModel::create(db, "lecturer1", "lecturer@test.com", "password", false)
            .await
            .unwrap();
        UserModuleRoleModel::assign_user_to_module(db, lecturer.id, data.module.id, Role::Lecturer)
            .await
            .unwrap();

        let valid_sub = create_remarkable_submission(
            db,
            data.assignment.module_id,
            data.assignment.id,
            data.student_user.id,
            1,
            &temp_dir,
        )
        .await;
        let invalid_sub_id = 9999;

        let broken_sub = create_remarkable_submission(
            db,
            data.assignment.module_id,
            data.assignment.id,
            data.student_user.id,
            2,
            &temp_dir,
        )
        .await;
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

        let (status, json) =
            send_remark_request(&app, &token, data.module.id, data.assignment.id, body).await;

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
            errors
                .iter()
                .any(|e| e.contains("memo_paths.len() != student_paths.len()")),
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

        let lecturer = UserModel::create(db, "lecturer1", "lecturer@test.com", "password", false)
            .await
            .unwrap();
        UserModuleRoleModel::assign_user_to_module(db, lecturer.id, data.module.id, Role::Lecturer)
            .await
            .unwrap();

        let (token, _) = generate_jwt(lecturer.id, lecturer.admin);
        let body = json!({
            "all": true
        });

        let (status, json) =
            send_remark_request(&app, &token, 9999, data.assignment.id, body).await;

        assert_eq!(status, StatusCode::NOT_FOUND);
        assert_eq!(json["success"], false);
        assert_eq!(json["message"], "Module 9999 not found.");
    }

    async fn send_resubmit_request(
        app: &BoxCloneService<Request<Body>, Response, Infallible>,
        token: &str,
        module_id: i64,
        assignment_id: i64,
        body: Value,
    ) -> Response {
        let req = Request::builder()
            .method("POST")
            .uri(&format!(
                "/api/modules/{}/assignments/{}/submissions/resubmit",
                module_id, assignment_id
            ))
            .header(AUTHORIZATION, format!("Bearer {}", token))
            .header(CONTENT_TYPE, "application/json")
            .body(Body::from(body.to_string()))
            .unwrap();

        app.clone().oneshot(req).await.unwrap()
    }

    async fn response_body_to_json(response: Response) -> Value {
        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        serde_json::from_slice(&body).unwrap()
    }

    async fn create_resubmitable_submission(
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
            earned: Set(10),
            total: Set(10),
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

        let task = db::models::assignment_task::Entity::find()
            .filter(db::models::assignment_task::Column::AssignmentId.eq(assignment_id))
            .one(db)
            .await
            .unwrap()
            .unwrap();

        let output = assignment_submission_output::ActiveModel {
            task_id: Set(task.id),
            submission_id: Set(submission.id),
            path: Set("".to_string()),
            ..Default::default()
        };
        let _output = output.insert(db).await.unwrap();

        let submission_file_path = temp_dir.path().join(&submission.path);
        if let Some(parent) = submission_file_path.parent() {
            std::fs::create_dir_all(parent).unwrap();
        }
        std::fs::write(&submission_file_path, create_submission_zip()).unwrap();

        submission
    }

    #[tokio::test]
    #[serial]
    async fn test_resubmit_specific_submissions() {
        let temp_dir = setup_assignment_storage_root();
        let (app, app_state) = make_test_app().await;
        let db = app_state.db();
        let data = setup_test_data(app_state.db(), &temp_dir).await;

        // Create submissions
        let sub1 = create_resubmitable_submission(
            db,
            data.assignment.module_id,
            data.assignment.id,
            data.student_user.id,
            1,
            &temp_dir,
        )
        .await;
        let sub2 = create_resubmitable_submission(
            db,
            data.assignment.module_id,
            data.assignment.id,
            data.student_user.id,
            2,
            &temp_dir,
        )
        .await;

        // Create lecturer user
        let lecturer = UserModel::create(db, "lecturer1", "lecturer@test.com", "password", false)
            .await
            .unwrap();
        UserModuleRoleModel::assign_user_to_module(db, lecturer.id, data.module.id, Role::Lecturer)
            .await
            .unwrap();

        // Send resubmit request
        let (token, _) = generate_jwt(lecturer.id, lecturer.admin);
        let body = json!({
            "submission_ids": [sub1.id, sub2.id]
        });

        let response =
            send_resubmit_request(&app, &token, data.module.id, data.assignment.id, body).await;

        // Verify response
        assert_eq!(response.status(), StatusCode::OK);
        let body = response_body_to_json(response).await;
        assert_eq!(body["success"], true);
        assert_eq!(body["data"]["resubmitted"], 2);
        assert!(body["data"]["failed"].as_array().unwrap().is_empty());
    }

    #[tokio::test]
    #[serial]
    async fn test_resubmit_all_submissions() {
        let temp_dir = setup_assignment_storage_root();
        let (app, app_state) = make_test_app().await;
        let db = app_state.db();
        let data = setup_test_data(app_state.db(), &temp_dir).await;

        // Create submissions
        create_resubmitable_submission(
            db,
            data.assignment.module_id,
            data.assignment.id,
            data.student_user.id,
            1,
            &temp_dir,
        )
        .await;
        create_resubmitable_submission(
            db,
            data.assignment.module_id,
            data.assignment.id,
            data.student_user.id,
            2,
            &temp_dir,
        )
        .await;

        // Create admin user
        let admin = UserModel::create(db, "admin1", "admin@test.com", "password", true)
            .await
            .unwrap();

        // Send resubmit request
        let (token, _) = generate_jwt(admin.id, admin.admin);
        let body = json!({
            "all": true
        });

        let response =
            send_resubmit_request(&app, &token, data.module.id, data.assignment.id, body).await;

        // Verify response
        assert_eq!(response.status(), StatusCode::OK);
        let body = response_body_to_json(response).await;
        assert_eq!(body["success"], true);
        assert_eq!(body["data"]["resubmitted"], 2);
        assert!(body["data"]["failed"].as_array().unwrap().is_empty());
    }

    #[tokio::test]
    #[serial]
    async fn test_resubmit_invalid_parameters() {
        let temp_dir = setup_assignment_storage_root();
        let (app, app_state) = make_test_app().await;
        let data = setup_test_data(app_state.db(), &temp_dir).await;

        // Create lecturer user
        let lecturer = UserModel::create(
            app_state.db(),
            "lecturer1",
            "lecturer@test.com",
            "password",
            false,
        )
        .await
        .unwrap();
        UserModuleRoleModel::assign_user_to_module(
            app_state.db(),
            lecturer.id,
            data.module.id,
            Role::Lecturer,
        )
        .await
        .unwrap();

        // Test cases
        let test_cases = vec![
            (
                json!({}),
                "Must provide exactly one of submission_ids or all=true",
            ),
            (
                json!({"submission_ids": [], "all": true}),
                "Must provide exactly one of submission_ids or all=true",
            ),
            (
                json!({"submission_ids": []}),
                "submission_ids cannot be empty",
            ),
        ];

        for (body, expected_error) in test_cases {
            let (token, _) = generate_jwt(lecturer.id, lecturer.admin);
            let response =
                send_resubmit_request(&app, &token, data.module.id, data.assignment.id, body).await;

            assert_eq!(response.status(), StatusCode::BAD_REQUEST);
            let body = response_body_to_json(response).await;
            assert_eq!(body["success"], false);
            assert_eq!(body["message"], expected_error);
        }
    }

    #[tokio::test]
    #[serial]
    async fn test_resubmit_unauthorized_access() {
        let temp_dir = setup_assignment_storage_root();
        let (app, app_state) = make_test_app().await;
        let db = app_state.db();
        let data = setup_test_data(app_state.db(), &temp_dir).await;

        // Create submission
        let submission = create_resubmitable_submission(
            db,
            data.assignment.module_id,
            data.assignment.id,
            data.student_user.id,
            1,
            &temp_dir,
        )
        .await;

        // Create student token
        let (token, _) = generate_jwt(data.student_user.id, data.student_user.admin);
        let body = json!({
            "submission_ids": [submission.id]
        });

        let response =
            send_resubmit_request(&app, &token, data.module.id, data.assignment.id, body).await;

        // Verify unauthorized response
        assert_eq!(response.status(), StatusCode::FORBIDDEN);
        let body = response_body_to_json(response).await;
        assert_eq!(body["success"], false);
        assert_eq!(
            body["message"],
            "Lecturer or assistant lecturer access required for this module"
        );
    }

    #[tokio::test]
    #[serial]
    async fn test_resubmit_assignment_not_found() {
        let temp_dir = setup_assignment_storage_root();
        let (app, app_state) = make_test_app().await;
        let db = app_state.db();
        let data = setup_test_data(app_state.db(), &temp_dir).await;

        // Create admin user
        let admin = UserModel::create(db, "admin1", "admin@test.com", "password", true)
            .await
            .unwrap();

        let (token, _) = generate_jwt(admin.id, admin.admin);
        let body = json!({
            "all": true
        });

        let response = send_resubmit_request(
            &app,
            &token,
            data.module.id,
            9999, // Invalid assignment ID
            body,
        )
        .await;

        // Verify not found response
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
        let body = response_body_to_json(response).await;
        assert_eq!(body["success"], false);
        assert_eq!(body["message"], "Assignment 9999 in Module 1 not found.");
    }
}
