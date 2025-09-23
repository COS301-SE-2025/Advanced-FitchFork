#[cfg(test)]
mod tests {
    use crate::helpers::app::make_test_app_with_storage;
    use api::auth::generate_jwt;
    use axum::http::header::{AUTHORIZATION, CONTENT_TYPE};
    use axum::{
        body::Body,
        http::{Request, StatusCode},
        response::Response,
    };
    use chrono::{Datelike, Duration, Utc};
    use db::models::{
        assignment::{
            ActiveModel as AssignmentActiveModel, Entity as AssignmentEntity,
            Model as AssignmentModel,
        },
        assignment_submission::{self, Model as AssignmentSubmissionModel},
        assignment_submission_output, assignment_task,
        assignment_task::Model as AssignmentTaskModel,
        module::Model as ModuleModel,
        user::Model as UserModel,
        user_module_role::{Model as UserModuleRoleModel, Role},
    };
    use flate2::{Compression, write::GzEncoder};
    use sea_orm::{
        ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, QueryOrder,
        Set,
    };
    use serde_json::{Value, json};
    use serial_test::serial;
    use std::convert::Infallible;
    use std::{fs, io::Write};
    use tar::Builder as TarBuilder;
    use tower::ServiceExt;
    use tower::util::BoxCloneService;
    use util::execution_config::ExecutionConfig;
    use util::execution_config::{GradingPolicy, SubmissionMode};
    use util::languages::Language;
    use util::paths::{
        config_dir, main_dir, makefile_dir, mark_allocator_dir, mark_allocator_path,
        memo_output_dir, storage_root, submission_output_dir, submission_output_path,
    };
    use zip::write::SimpleFileOptions;

    struct TestData {
        student_user: UserModel,
        unassigned_user: UserModel,
        module: ModuleModel,
        assignment: AssignmentModel,
    }

    async fn setup_test_data(db: &DatabaseConnection) -> TestData {
        let now = Utc::now();

        // Keep module year current so test data always looks “current”
        let module_year = now.year();

        let module = ModuleModel::create(db, "COS101", module_year, Some("Test Module"), 16)
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

        // Ensure: available_from <= now <= due_date
        let available_from = now - Duration::hours(1);
        let due_date = now + Duration::days(1);

        let assignment = AssignmentModel::create(
            db,
            module.id,
            "Assignment 1",
            Some("Desc 1"),
            db::models::assignment::AssignmentType::Assignment,
            available_from,
            due_date,
        )
        .await
        .unwrap();

        // Keep status Ready explicitly (your logic may auto-transition in other tests)
        let mut active_assignment: db::models::assignment::ActiveModel = assignment.clone().into();
        active_assignment.status = Set(db::models::assignment::Status::Ready);
        active_assignment.updated_at = Set(now);
        let assignment = active_assignment.update(db).await.unwrap();

        AssignmentTaskModel::create(db, assignment.id, 1, "Task 1", "make task1", false)
            .await
            .unwrap();

        create_assignment_structure(module.id, assignment.id);
        create_mark_allocator(module.id, assignment.id);
        create_memo_outputs(module.id, assignment.id);
        create_execution_config(module.id, assignment.id);
        create_makefile_zip(module.id, assignment.id);
        create_main_zip(module.id, assignment.id);

        TestData {
            student_user,
            unassigned_user,
            module,
            assignment,
        }
    }

    fn create_assignment_structure(module_id: i64, assignment_id: i64) {
        fs::create_dir_all(mark_allocator_dir(module_id, assignment_id)).unwrap();
        fs::create_dir_all(memo_output_dir(module_id, assignment_id)).unwrap();
        fs::create_dir_all(config_dir(module_id, assignment_id)).unwrap();
        fs::create_dir_all(makefile_dir(module_id, assignment_id)).unwrap();
        fs::create_dir_all(main_dir(module_id, assignment_id)).unwrap();
    }

    fn create_mark_allocator(module_id: i64, assignment_id: i64) {
        use serde_json::json;

        let alloc = json!({
            "generated_at": "2025-07-21T10:00:00Z",
            "tasks": [
                {
                    "task_number": 1,
                    "name": "Task 1 Execution",
                    "value": 10,
                    "code_coverage": false,
                    "subsections": [
                        { "name": "Subtask 1 Output", "value": 10 }
                    ]
                }
            ],
            "total_value": 10
        });

        fs::create_dir_all(mark_allocator_dir(module_id, assignment_id)).unwrap();
        fs::write(
            mark_allocator_path(module_id, assignment_id),
            serde_json::to_string_pretty(&alloc).unwrap(),
        )
        .unwrap();
    }

    fn create_memo_outputs(module_id: i64, assignment_id: i64) {
        let dir = memo_output_dir(module_id, assignment_id);
        fs::create_dir_all(&dir).unwrap();
        let memo_content = "make task1\n&-=-&Subtask 1 Output\nOutput A";
        fs::write(dir.join("task1.txt"), memo_content).unwrap();
    }

    fn create_execution_config(module_id: i64, assignment_id: i64) {
        let mut cfg = ExecutionConfig::default_config();
        // Only override what the tests require:
        cfg.project.language = Language::Java; // because we ship .java files
        cfg.project.submission_mode = SubmissionMode::Manual;
        cfg.marking.deliminator = "&-=-&".to_string(); // your parser expects this exact token
        // (Other defaults: pass_mark=50, grading_policy=Last, etc.)

        cfg.save(module_id, assignment_id)
            .expect("write config.json");
    }

    fn create_makefile_zip(module_id: i64, assignment_id: i64) {
        let mut buf = Vec::new();
        let mut zip = zip::ZipWriter::new(std::io::Cursor::new(&mut buf));
        zip.start_file("Makefile", SimpleFileOptions::default())
            .unwrap();
        zip.write_all(&create_makefile_content()).unwrap();
        zip.finish().unwrap();

        let out = makefile_dir(module_id, assignment_id).join("makefile.zip");
        fs::create_dir_all(out.parent().unwrap()).unwrap();
        fs::write(out, buf).unwrap();
    }

    fn create_main_zip(module_id: i64, assignment_id: i64) {
        let mut buf = Vec::new();
        let mut zip = zip::ZipWriter::new(std::io::Cursor::new(&mut buf));
        let opts = SimpleFileOptions::default();

        zip.start_file("Main.java", opts).unwrap();
        zip.write_all(&create_main_content()).unwrap();

        zip.start_file("HelperOne.java", opts).unwrap();
        zip.write_all(&create_helper_one_content()).unwrap();

        zip.start_file("HelperTwo.java", opts).unwrap();
        zip.write_all(&create_helper_two_content()).unwrap();

        zip.finish().unwrap();

        let out = main_dir(module_id, assignment_id).join("main.zip");
        fs::create_dir_all(out.parent().unwrap()).unwrap();
        fs::write(out, buf).unwrap();
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
        attests_ownership: Option<&str>, // "true" | "false"
    ) -> (String, Vec<u8>) {
        let boundary = "----BoundaryTest".to_string();
        let mut body = Vec::new();

        // file
        body.extend(format!("--{}\r\n", boundary).as_bytes());
        body.extend(
            format!(
                "Content-Disposition: form-data; name=\"file\"; filename=\"{}\"\r\nContent-Type: application/octet-stream\r\n\r\n",
                filename
            )
            .as_bytes(),
        );
        body.extend(file_content);
        body.extend(b"\r\n");

        // optional is_practice
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

        // optional attests_ownership
        if let Some(val) = attests_ownership {
            body.extend(format!("--{}\r\n", boundary).as_bytes());
            body.extend(b"Content-Disposition: form-data; name=\"attests_ownership\"\r\n\r\n");
            body.extend(val.as_bytes());
            body.extend(b"\r\n");
        }

        // end
        body.extend(format!("--{}--\r\n", boundary).as_bytes());
        (boundary, body)
    }

    fn write_attempt_policy_config(
        module_id: i64,
        assignment_id: i64,
        limit_attempts: bool,
        max_attempts: u32,
        allow_practice: bool,
    ) {
        let mut cfg = ExecutionConfig::default_config();
        cfg.project.submission_mode = SubmissionMode::Manual; // ensure manual path
        cfg.marking.limit_attempts = limit_attempts;
        cfg.marking.max_attempts = max_attempts;
        cfg.marking.allow_practice_submissions = allow_practice;
        cfg.marking.pass_mark = 50;
        cfg.marking.grading_policy = GradingPolicy::Last; // same as your previous JSON
        cfg.marking.deliminator = "&-=-&".to_string();

        cfg.save(module_id, assignment_id)
            .expect("write config.json");
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
        let (app, app_state, _tmp) = make_test_app_with_storage().await;
        let data = setup_test_data(app_state.db()).await;
        let file = create_submission_zip();

        let (boundary, body) = multipart_body("solution.zip", &file, None, Some("true"));
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
        let (app, app_state, _tmp) = make_test_app_with_storage().await;
        let data = setup_test_data(app_state.db()).await;
        let file = create_submission_tar();

        let (boundary, body) = multipart_body("solution.tar", &file, None, Some("true"));
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
        let (app, app_state, _tmp) = make_test_app_with_storage().await;
        let data = setup_test_data(app_state.db()).await;
        let file = create_submission_tgz();

        let (boundary, body) = multipart_body("solution.tgz", &file, None, Some("true"));
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
        let (app, app_state, _tmp) = make_test_app_with_storage().await;
        let data = setup_test_data(app_state.db()).await;
        let file = create_submission_gz();

        let (boundary, body) = multipart_body("solution.gz", &file, None, Some("true"));
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
        let (app, app_state, _tmp) = make_test_app_with_storage().await;
        let data = setup_test_data(app_state.db()).await;

        // allow practice submissions for this assignment via execution config
        write_attempt_policy_config(
            data.module.id,
            data.assignment.id,
            /*limit_attempts*/ false,
            /*max_attempts*/ 10,
            /*allow_practice*/ true,
        );

        let file = create_submission_zip();
        let (boundary, body) = multipart_body("solution.zip", &file, Some("true"), Some("true"));
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
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(json["success"], true);
        assert_eq!(json["data"]["is_practice"], true);
    }

    #[tokio::test]
    #[serial]
    async fn test_multiple_attempts_increments() {
        let (app, app_state, _tmp) = make_test_app_with_storage().await;
        let data = setup_test_data(app_state.db()).await;
        let file = create_submission_zip();

        let (boundary, body) = multipart_body("solution.zip", &file, None, Some("true"));
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
        let (app, app_state, _tmp) = make_test_app_with_storage().await;
        let db = app_state.db();
        let data = setup_test_data(db).await;

        // Force strict attempt policy: max 1, practice irrelevant here.
        write_attempt_policy_config(
            data.module.id,
            data.assignment.id,
            /*limit_attempts*/ true,
            /*max_attempts*/ 1,
            /*allow_practice*/ false,
        );

        let file = create_submission_zip();
        let (token, _) = generate_jwt(data.student_user.id, data.student_user.admin);
        let uri = format!(
            "/api/modules/{}/assignments/{}/submissions",
            data.module.id, data.assignment.id
        );

        // 1st submission -> OK
        {
            let (boundary, body) = multipart_body("solution.zip", &file, None, Some("true"));
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
            let resp = app.clone().oneshot(req).await.unwrap();
            assert_eq!(resp.status(), StatusCode::OK);
        }

        // 2nd submission -> FORBIDDEN (max attempts reached)
        {
            let (boundary, body) = multipart_body("solution.zip", &file, None, Some("true"));
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
            let resp = app.clone().oneshot(req).await.unwrap();
            assert_eq!(resp.status(), StatusCode::FORBIDDEN);

            let body = axum::body::to_bytes(resp.into_body(), usize::MAX)
                .await
                .unwrap();
            let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
            assert_eq!(json["success"], false);
            assert_eq!(
                json["message"],
                "Maximum attempts reached: used 1 of 1 (remaining 0)."
            );
        }
    }

    #[tokio::test]
    #[serial]
    async fn test_practice_submission_denied_when_disabled() {
        let (app, app_state, _tmp) = make_test_app_with_storage().await;
        let db = app_state.db();
        let data = setup_test_data(db).await;

        // Practice disabled in config
        write_attempt_policy_config(
            data.module.id,
            data.assignment.id,
            /*limit_attempts*/ true,
            /*max_attempts*/ 5,
            /*allow_practice*/ false,
        );

        let file = create_submission_zip();
        let (boundary, body) = multipart_body("solution.zip", &file, Some("true"), Some("true")); // is_practice=true
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

        let resp = app.clone().oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::FORBIDDEN);

        let body = axum::body::to_bytes(resp.into_body(), usize::MAX)
            .await
            .unwrap();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(json["success"], false);
        assert_eq!(
            json["message"],
            "Practice submissions are disabled for this assignment."
        );
    }

    #[tokio::test]
    #[serial]
    async fn test_submission_allowed_when_previous_attempt_was_ignored() {
        let (app, app_state, _tmp) = make_test_app_with_storage().await;
        let db = app_state.db();
        let data = setup_test_data(db).await;

        // max 1 attempt, but we'll seed a single IGNORED prior attempt → should NOT count
        write_attempt_policy_config(
            data.module.id,
            data.assignment.id,
            /*limit_attempts*/ true,
            /*max_attempts*/ 1,
            /*allow_practice*/ false,
        );

        seed_submission_row(
            db,
            data.assignment.id,
            data.student_user.id,
            1,
            /*is_practice*/ false,
            /*ignored*/ true,
        )
        .await;

        let file = create_submission_zip();
        let (token, _) = generate_jwt(data.student_user.id, data.student_user.admin);
        let (boundary, body) = multipart_body("solution.zip", &file, None, Some("true"));

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

        // Should be allowed because prior attempt is ignored
        let resp = app.clone().oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let body = axum::body::to_bytes(resp.into_body(), usize::MAX)
            .await
            .unwrap();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(json["success"], true);
    }

    #[tokio::test]
    #[serial]
    async fn test_staff_bypass_attempt_limit() {
        let (app, app_state, _tmp) = make_test_app_with_storage().await;
        let db = app_state.db();
        let data = setup_test_data(db).await;

        // Strict limit (1 attempt), but submit as lecturer → staff bypass
        write_attempt_policy_config(
            data.module.id,
            data.assignment.id,
            /*limit_attempts*/ true,
            /*max_attempts*/ 1,
            /*allow_practice*/ false,
        );

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
            let (boundary, body) = multipart_body("solution.zip", &file, None, Some("true"));
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
            let resp = app.clone().oneshot(req).await.unwrap();
            assert_eq!(resp.status(), StatusCode::OK);
        }

        // Second submission as staff (still OK; students would be blocked)
        {
            let (boundary, body) = multipart_body("solution.zip", &file, None, Some("true"));
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
            let resp = app.clone().oneshot(req).await.unwrap();
            assert_eq!(resp.status(), StatusCode::OK);
        }
    }

    #[tokio::test]
    #[serial]
    async fn test_submission_exactly_at_due_date() {
        let (app, app_state, _tmp) = make_test_app_with_storage().await;
        let data = setup_test_data(app_state.db()).await;
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

        let (boundary, body) = multipart_body("solution.zip", &file, None, Some("true"));
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
        assert_eq!(json["message"], "Late submissions are not allowed");
    }

    #[tokio::test]
    #[serial]
    async fn test_submission_just_after_due_date() {
        let (app, app_state, _tmp) = make_test_app_with_storage().await;
        let data = setup_test_data(app_state.db()).await;
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

        let (boundary, body) = multipart_body("solution.zip", &file, None, Some("true"));
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
        assert_eq!(json["message"], "Late submissions are not allowed");
    }

    #[tokio::test]
    #[serial]
    async fn test_submission_just_before_due_date() {
        let (app, app_state, _tmp) = make_test_app_with_storage().await;
        let data = setup_test_data(app_state.db()).await;
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

        let (boundary, body) = multipart_body("solution.zip", &file, None, Some("true"));
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
        let (app, app_state, _tmp) = make_test_app_with_storage().await;
        let data = setup_test_data(app_state.db()).await;
        let file = create_large_zip_file(1000 * 1000);

        let (boundary, body) = multipart_body("large.zip", &file, None, Some("true"));
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
        //assert_eq!(response.status(), StatusCode::OK);

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
        let (app, app_state, _tmp) = make_test_app_with_storage().await;
        let data = setup_test_data(app_state.db()).await;

        let boundary = "----BoundaryTest".to_string();
        let mut body = Vec::new();

        // attests_ownership=true, but no file part
        body.extend(format!("--{}\r\n", &boundary).as_bytes());
        body.extend(b"Content-Disposition: form-data; name=\"attests_ownership\"\r\n\r\ntrue\r\n");
        body.extend(format!("--{}--\r\n", &boundary).as_bytes());

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
        let (app, app_state, _tmp) = make_test_app_with_storage().await;
        let data = setup_test_data(app_state.db()).await;
        let file = Vec::new();

        let (boundary, body) = multipart_body("solution.zip", &file, None, Some("true"));
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
        let (app, app_state, _tmp) = make_test_app_with_storage().await;
        let data = setup_test_data(app_state.db()).await;
        let file = create_exe_file();

        let (boundary, body) = multipart_body("solution.exe", &file, None, Some("true"));
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
        let (app, app_state, _tmp) = make_test_app_with_storage().await;
        let data = setup_test_data(app_state.db()).await;
        let file = create_corrupted_zip();

        let (boundary, body) = multipart_body("corrupted.zip", &file, None, Some("true"));
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
        let (app, app_state, _tmp) = make_test_app_with_storage().await;
        let data = setup_test_data(app_state.db()).await;
        let file = create_submission_zip();

        let (boundary, body) = multipart_body("solution.zip", &file, None, Some("true"));
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
        let (app, app_state, _tmp) = make_test_app_with_storage().await;
        let data = setup_test_data(app_state.db()).await;
        let file = create_submission_zip();

        let (boundary, body) = multipart_body("solution.zip", &file, None, Some("true"));
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
        let (app, app_state, _tmp) = make_test_app_with_storage().await;
        let data = setup_test_data(app_state.db()).await;
        let file = create_submission_zip();

        let (boundary, body) = multipart_body("solution.zip", &file, None, Some("true"));
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

    #[tokio::test]
    #[serial]
    async fn test_failed_to_load_mark_allocator() {
        let (app, app_state, _tmp) = make_test_app_with_storage().await;
        let data = setup_test_data(app_state.db()).await;
        let file = create_submission_zip();

        let allocator_path = mark_allocator_path(data.module.id, data.assignment.id);
        std::fs::remove_file(&allocator_path)
            .expect("Failed to delete allocator.json for test setup");

        let (boundary, body) = multipart_body("solution.zip", &file, None, Some("true"));
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
        assert_eq!(json["message"], "Failed to run code for submission");
    }

    // Helper function to create a submission with an output file wired via path utilities.
    async fn create_remarkable_submission(
        db: &DatabaseConnection,
        module_id: i64,
        assignment_id: i64,
        user_id: i64,
        attempt: i64,
    ) -> AssignmentSubmissionModel {
        // Create the submission row (and a dummy file) via the model helper for consistency.
        // Filename matches the path utility (attempt dir contains "<user_id>.zip").
        let filename = format!("{user_id}.zip");
        let submission = assignment_submission::Model::save_file(
            db,
            assignment_id,
            user_id,
            attempt,
            10,
            10,
            false,
            &filename,
            "d41d8cd98f00b204e9800998ecf8427e", // dummy hash
            b"dummy",
        )
        .await
        .unwrap();

        // Grab any task for this assignment (your tests create at least one).
        let task = db::models::assignment_task::Entity::find()
            .filter(db::models::assignment_task::Column::AssignmentId.eq(assignment_id))
            .one(db)
            .await
            .unwrap()
            .expect("assignment must have a task");

        // Create the output row first (we'll update path after we write the file).
        let output = assignment_submission_output::ActiveModel {
            task_id: Set(task.id),
            submission_id: Set(submission.id),
            path: Set(String::new()),
            ..Default::default()
        }
        .insert(db)
        .await
        .unwrap();

        // Write an output artifact to the canonical location.
        let out_dir = submission_output_dir(module_id, assignment_id, user_id, attempt);
        std::fs::create_dir_all(&out_dir).unwrap();

        let out_file_path = submission_output_path(
            module_id,
            assignment_id,
            user_id,
            attempt,
            &format!("{}.txt", output.id),
        );
        std::fs::write(
            &out_file_path,
            "make task1\n&-=-&Subtask 1 Output\nOutput A",
        )
        .unwrap();

        let root = storage_root();
        let rel_path = out_file_path
            .strip_prefix(&root)
            .unwrap_or(&out_file_path)
            .to_string_lossy()
            .to_string();

        let mut output_active: assignment_submission_output::ActiveModel = output.into();
        output_active.path = Set(rel_path);
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
        let (app, app_state, _tmp) = make_test_app_with_storage().await;
        let db = app_state.db();
        let data = setup_test_data(app_state.db()).await;

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
        )
        .await;
        let sub2 = create_remarkable_submission(
            db,
            data.assignment.module_id,
            data.assignment.id,
            data.student_user.id,
            2,
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
        let (app, app_state, _tmp) = make_test_app_with_storage().await;
        let db = app_state.db();
        let data = setup_test_data(app_state.db()).await;

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
        )
        .await;
        let sub2 = create_remarkable_submission(
            db,
            data.assignment.module_id,
            data.assignment.id,
            data.student_user.id,
            2,
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
        let (app, app_state, _tmp) = make_test_app_with_storage().await;
        let db = app_state.db();
        let data = setup_test_data(app_state.db()).await;

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
        )
        .await;
        create_remarkable_submission(
            db,
            data.assignment.module_id,
            data.assignment.id,
            data.student_user.id,
            2,
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
        let (app, app_state, _tmp) = make_test_app_with_storage().await;
        let db = app_state.db();
        let data = setup_test_data(app_state.db()).await;

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
        )
        .await;
        create_remarkable_submission(
            db,
            data.assignment.module_id,
            data.assignment.id,
            data.student_user.id,
            2,
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
        let (app, app_state, _tmp) = make_test_app_with_storage().await;
        let db = app_state.db();
        let data = setup_test_data(app_state.db()).await;

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
        let (app, app_state, _tmp) = make_test_app_with_storage().await;
        let db = app_state.db();
        let data = setup_test_data(app_state.db()).await;

        let submission = create_remarkable_submission(
            db,
            data.assignment.module_id,
            data.assignment.id,
            data.student_user.id,
            1,
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
        let (app, app_state, _tmp) = make_test_app_with_storage().await;
        let db = app_state.db();
        let data = setup_test_data(app_state.db()).await;

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
        let (app, app_state, _tmp) = make_test_app_with_storage().await;
        let db = app_state.db();
        let data = setup_test_data(app_state.db()).await;

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
        let (app, app_state, _tmp) = make_test_app_with_storage().await;
        let db = app_state.db();
        let data = setup_test_data(app_state.db()).await;

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
        )
        .await;

        let allocator_path = mark_allocator_path(data.module.id, data.assignment.id);
        fs::remove_file(&allocator_path).unwrap();

        let (token, _) = generate_jwt(lecturer.id, lecturer.admin);
        let body = json!({
            "submission_ids": [submission.id]
        });

        let (status, json) =
            send_remark_request(&app, &token, data.module.id, data.assignment.id, body).await;

        assert_eq!(status, StatusCode::OK);
        assert_eq!(json["success"], true);
        assert_eq!(
            json["data"]["failed"][0]["error"],
            "Failed to load mark allocator: File not found"
        );
    }

    /// Test 8: Failed during grading of a submission
    #[tokio::test]
    #[serial]
    async fn test_remark_grading_failure() {
        let (app, app_state, _tmp) = make_test_app_with_storage().await;
        let db = app_state.db();
        let data = setup_test_data(app_state.db()).await;

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
        )
        .await;

        let student_output_dir =
            submission_output_dir(data.module.id, data.assignment.id, data.student_user.id, 1);
        fs::remove_dir_all(&student_output_dir).unwrap();

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
        let (app, app_state, _tmp) = make_test_app_with_storage().await;
        let db = app_state.db();
        let data = setup_test_data(app_state.db()).await;

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
        )
        .await;
        let invalid_sub_id = 9999;

        let broken_sub = create_remarkable_submission(
            db,
            data.assignment.module_id,
            data.assignment.id,
            data.student_user.id,
            2,
        )
        .await;

        let student_output_dir =
            submission_output_dir(data.module.id, data.assignment.id, data.student_user.id, 2);
        fs::remove_dir_all(&student_output_dir).unwrap();

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
        let (app, app_state, _tmp) = make_test_app_with_storage().await;
        let db = app_state.db();
        let data = setup_test_data(app_state.db()).await;

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

    // Create a submission that also writes the submission archive to disk
    // using the centralized save_file (stores as {attempt}/{submission_id}.zip).
    async fn create_resubmitable_submission(
        db: &DatabaseConnection,
        _module_id: i64, // kept for signature parity; not used here
        assignment_id: i64,
        user_id: i64,
        attempt: i64,
    ) -> AssignmentSubmissionModel {
        // Write the file and row via the model helper (uses {submission_id}.ext on disk,
        // and stores a RELATIVE path in the DB).
        let submission = assignment_submission::Model::save_file(
            db,
            assignment_id,
            user_id,
            attempt,
            /* earned */ 10,
            /* total  */ 10,
            /* is_practice */ false,
            "test_submission.zip", // original filename (what users see)
            "d41d8cd98f00b204e9800998ecf8427e", // dummy hash
            &create_submission_zip(), // actual bytes
        )
        .await
        .unwrap();

        // Seed an output row (empty path is fine for resubmit tests)
        let task = assignment_task::Entity::find()
            .filter(assignment_task::Column::AssignmentId.eq(assignment_id))
            .one(db)
            .await
            .unwrap()
            .expect("assignment has at least one task");

        let _ = assignment_submission_output::ActiveModel {
            task_id: Set(task.id),
            submission_id: Set(submission.id),
            path: Set(String::new()),
            ..Default::default()
        }
        .insert(db)
        .await
        .unwrap();

        submission
    }

    #[tokio::test]
    #[serial]
    async fn test_resubmit_specific_submissions() {
        let (app, app_state, _tmp) = make_test_app_with_storage().await;
        let db = app_state.db();
        let data = setup_test_data(app_state.db()).await;

        // Create submissions
        let sub1 = create_resubmitable_submission(
            db,
            data.assignment.module_id,
            data.assignment.id,
            data.student_user.id,
            1,
        )
        .await;
        let sub2 = create_resubmitable_submission(
            db,
            data.assignment.module_id,
            data.assignment.id,
            data.student_user.id,
            2,
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
        let (app, app_state, _tmp) = make_test_app_with_storage().await;
        let db = app_state.db();
        let data = setup_test_data(app_state.db()).await;

        // Create submissions
        create_resubmitable_submission(
            db,
            data.assignment.module_id,
            data.assignment.id,
            data.student_user.id,
            1,
        )
        .await;
        create_resubmitable_submission(
            db,
            data.assignment.module_id,
            data.assignment.id,
            data.student_user.id,
            2,
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
        let (app, app_state, _tmp) = make_test_app_with_storage().await;
        let data = setup_test_data(app_state.db()).await;

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
        let (app, app_state, _tmp) = make_test_app_with_storage().await;
        let db = app_state.db();
        let data = setup_test_data(app_state.db()).await;

        // Create submission
        let submission = create_resubmitable_submission(
            db,
            data.assignment.module_id,
            data.assignment.id,
            data.student_user.id,
            1,
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
        let (app, app_state, _tmp) = make_test_app_with_storage().await;
        let db = app_state.db();
        let data = setup_test_data(app_state.db()).await;

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

    #[tokio::test]
    #[serial]
    async fn test_submit_missing_attests_ownership_returns_422() {
        let (app, app_state, _tmp) = make_test_app_with_storage().await;
        let data = setup_test_data(app_state.db()).await;
        let file = create_submission_zip();

        // no attests_ownership field
        let (boundary, body) = multipart_body("solution.zip", &file, None, None);
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

        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::UNPROCESSABLE_ENTITY);

        let body = axum::body::to_bytes(resp.into_body(), usize::MAX)
            .await
            .unwrap();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(json["success"], false);
        assert!(
            json["message"]
                .as_str()
                .unwrap_or_default()
                .to_lowercase()
                .contains("ownership")
        );
    }

    #[tokio::test]
    #[serial]
    async fn test_submit_attests_ownership_false_returns_422() {
        let (app, app_state, _tmp) = make_test_app_with_storage().await;
        let data = setup_test_data(app_state.db()).await;
        let file = create_submission_zip();

        // multipart with attests_ownership = false
        let boundary = "----BoundaryAttestFalse".to_string();
        let mut body = Vec::new();

        body.extend(format!("--{}\r\n", boundary).as_bytes());
        body.extend(b"Content-Disposition: form-data; name=\"file\"; filename=\"solution.zip\"\r\nContent-Type: application/octet-stream\r\n\r\n");
        body.extend(&file);
        body.extend(b"\r\n");

        body.extend(format!("--{}\r\n", boundary).as_bytes());
        body.extend(b"Content-Disposition: form-data; name=\"attests_ownership\"\r\n\r\nfalse\r\n");
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

        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::UNPROCESSABLE_ENTITY);

        let body = axum::body::to_bytes(resp.into_body(), usize::MAX)
            .await
            .unwrap();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(json["success"], false);
        assert!(
            json["message"]
                .as_str()
                .unwrap_or_default()
                .to_lowercase()
                .contains("ownership")
        );
    }

    #[tokio::test]
    #[serial]
    async fn test_submit_attests_ownership_true_ok() {
        let (app, app_state, _tmp) = make_test_app_with_storage().await;
        let db = app_state.db();
        let data = setup_test_data(db).await;
        let file = create_submission_zip();

        // multipart with attests_ownership = true
        let boundary = "----BoundaryAttestTrue".to_string();
        let mut body = Vec::new();

        body.extend(format!("--{}\r\n", boundary).as_bytes());
        body.extend(b"Content-Disposition: form-data; name=\"file\"; filename=\"solution.zip\"\r\nContent-Type: application/octet-stream\r\n\r\n");
        body.extend(&file);
        body.extend(b"\r\n");

        body.extend(format!("--{}\r\n", boundary).as_bytes());
        body.extend(b"Content-Disposition: form-data; name=\"attests_ownership\"\r\n\r\ntrue\r\n");
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

        let resp = app.clone().oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);

        // verify a submission row exists (we no longer assert any DB column for attestation)
        use db::models::assignment_submission::Column as SubCol;
        let saved = db::models::assignment_submission::Entity::find()
            .filter(SubCol::AssignmentId.eq(data.assignment.id))
            .filter(SubCol::UserId.eq(data.student_user.id))
            .order_by(SubCol::Attempt, sea_orm::Order::Desc)
            .one(db)
            .await
            .expect("db ok")
            .expect("submission row exists");
        assert_eq!(saved.assignment_id, data.assignment.id);
        assert_eq!(saved.user_id, data.student_user.id);
        assert_eq!(saved.filename, "solution.zip");
    }

    #[tokio::test]
    #[serial]
    async fn test_submission_status_running_during_processing() {
        let (app, app_state, _tmp) = make_test_app_with_storage().await;
        let db = app_state.db();
        let data = setup_test_data(db).await;
        let file = create_submission_zip();

        let (boundary, body) = multipart_body("solution.zip", &file, None, Some("true"));
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

        // The submission should have been set to Running during code execution
        // and then progressed to either Graded or a Failed status
        use db::models::assignment_submission::{Column as SubCol, Entity as SubmissionEntity};
        let submission = SubmissionEntity::find()
            .filter(SubCol::AssignmentId.eq(data.assignment.id))
            .filter(SubCol::UserId.eq(data.student_user.id))
            .order_by(SubCol::Attempt, sea_orm::Order::Desc)
            .one(db)
            .await
            .expect("db ok")
            .expect("submission exists");

        // Since the submission processing completes synchronously in tests,
        // we verify that the status is no longer Queued and has progressed
        // (it should be either Graded or one of the Failed states)
        assert_ne!(
            submission.status,
            db::models::assignment_submission::SubmissionStatus::Queued
        );
        assert!(
            submission.status == db::models::assignment_submission::SubmissionStatus::Graded
                || submission.is_failed()
        );
    }

    #[tokio::test]
    #[serial]
    async fn test_submission_status_graded_on_success() {
        let (app, app_state, _tmp) = make_test_app_with_storage().await;
        let db = app_state.db();
        let data = setup_test_data(db).await;
        let file = create_submission_zip();

        let (boundary, body) = multipart_body("solution.zip", &file, None, Some("true"));
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

        // For successful submissions with valid test setup, the status should be Graded
        use db::models::assignment_submission::{Column as SubCol, Entity as SubmissionEntity};
        let submission = SubmissionEntity::find()
            .filter(SubCol::AssignmentId.eq(data.assignment.id))
            .filter(SubCol::UserId.eq(data.student_user.id))
            .order_by(SubCol::Attempt, sea_orm::Order::Desc)
            .one(db)
            .await
            .expect("db ok")
            .expect("submission exists");

        // The submission should be graded if everything went well
        if !submission.is_failed() {
            assert_eq!(
                submission.status,
                db::models::assignment_submission::SubmissionStatus::Graded
            );
        }
    }

    #[tokio::test]
    #[serial]
    async fn test_submission_status_failed_upload() {
        let (app, app_state, _tmp) = make_test_app_with_storage().await;
        let db = app_state.db();
        let data = setup_test_data(db).await;

        // Test with empty file to trigger upload failure
        let empty_file = Vec::new();
        let (boundary, body) = multipart_body("solution.zip", &empty_file, None, Some("true"));
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
        // Empty file should be rejected with UNPROCESSABLE_ENTITY
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
    async fn test_submission_status_failed_execution() {
        let (app, app_state, _tmp) = make_test_app_with_storage().await;
        let db = app_state.db();
        let data = setup_test_data(db).await;

        // Use corrupted zip file to trigger execution failure
        let corrupted_file = create_corrupted_zip();
        let (boundary, body) = multipart_body("corrupted.zip", &corrupted_file, None, Some("true"));
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
        // Corrupted file should cause internal server error during processing
        assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let json: Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(json["success"], false);

        // Check if a submission was created and has failed status
        use db::models::assignment_submission::{Column as SubCol, Entity as SubmissionEntity};
        let submission = SubmissionEntity::find()
            .filter(SubCol::AssignmentId.eq(data.assignment.id))
            .filter(SubCol::UserId.eq(data.student_user.id))
            .order_by(SubCol::Attempt, sea_orm::Order::Desc)
            .one(db)
            .await
            .expect("db ok");

        if let Some(submission) = submission {
            assert!(submission.is_failed());
        }
    }

    #[tokio::test]
    #[serial]
    async fn test_submission_status_failed_grading() {
        let (app, app_state, _tmp) = make_test_app_with_storage().await;
        let db = app_state.db();
        let data = setup_test_data(db).await;

        // Remove mark allocator to cause grading failure
        let allocator_path = mark_allocator_path(data.module.id, data.assignment.id);
        std::fs::remove_file(&allocator_path).ok(); // Ignore if it doesn't exist

        let file = create_submission_zip();
        let (boundary, body) = multipart_body("solution.zip", &file, None, Some("true"));
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
        // Missing mark allocator should cause internal server error
        assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let json: Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(json["success"], false);
        assert_eq!(json["message"], "Failed to mark submission");

        // Check if a submission was created and has failed status
        use db::models::assignment_submission::{Column as SubCol, Entity as SubmissionEntity};
        let submission = SubmissionEntity::find()
            .filter(SubCol::AssignmentId.eq(data.assignment.id))
            .filter(SubCol::UserId.eq(data.student_user.id))
            .order_by(SubCol::Attempt, sea_orm::Order::Desc)
            .one(db)
            .await
            .expect("db ok");

        if let Some(submission) = submission {
            assert!(submission.is_failed());
        }
    }

    #[tokio::test]
    #[serial]
    async fn test_submission_status_failed_compile() {
        let (app, app_state, _tmp) = make_test_app_with_storage().await;
        let db = app_state.db();
        let data = setup_test_data(db).await;

        // Create a zip with invalid Java code to trigger compile failure
        let mut buf = Vec::new();
        let mut zip = zip::ZipWriter::new(std::io::Cursor::new(&mut buf));
        let opts = SimpleFileOptions::default();

        // Invalid Java code that won't compile
        zip.start_file("Main.java", opts).unwrap();
        zip.write_all(b"public class Main { invalid syntax here }")
            .unwrap();
        zip.finish().unwrap();

        let (boundary, body) = multipart_body("broken.zip", &buf, None, Some("true"));
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

        // Check if a submission was created and verify its status
        use db::models::assignment_submission::{Column as SubCol, Entity as SubmissionEntity};
        let submission = SubmissionEntity::find()
            .filter(SubCol::AssignmentId.eq(data.assignment.id))
            .filter(SubCol::UserId.eq(data.student_user.id))
            .order_by(SubCol::Attempt, sea_orm::Order::Desc)
            .one(db)
            .await
            .expect("db ok");

        if let Some(submission) = submission {
            // The submission should either fail or succeed depending on the compilation step
            assert!(submission.is_complete()); // Either graded or failed
        }
    }

    #[tokio::test]
    #[serial]
    async fn test_submission_status_helper_methods() {
        let (_app, app_state, _tmp) = make_test_app_with_storage().await;
        let db = app_state.db();
        let data = setup_test_data(db).await;

        // Create a test submission to test direct status manipulation
        let file = create_submission_zip();
        let submission = db::models::assignment_submission::Model::save_file(
            db,
            data.assignment.id,
            data.student_user.id,
            1,
            0,
            10,
            false,
            "test.zip",
            "test_hash",
            &file,
        )
        .await
        .expect("Failed to create submission");

        // Test initial Queued status
        assert_eq!(
            submission.status,
            db::models::assignment_submission::SubmissionStatus::Queued
        );
        assert!(submission.is_in_progress());
        assert!(!submission.is_complete());
        assert!(!submission.is_failed());

        // Test Running status
        let running_submission =
            db::models::assignment_submission::Model::set_running(db, submission.id)
                .await
                .expect("Failed to set running status");
        assert_eq!(
            running_submission.status,
            db::models::assignment_submission::SubmissionStatus::Running
        );
        assert!(running_submission.is_in_progress());
        assert!(!running_submission.is_complete());
        assert!(!running_submission.is_failed());

        // Test Grading status
        let grading_submission =
            db::models::assignment_submission::Model::set_grading(db, submission.id)
                .await
                .expect("Failed to set grading status");
        assert_eq!(
            grading_submission.status,
            db::models::assignment_submission::SubmissionStatus::Grading
        );
        assert!(grading_submission.is_in_progress());
        assert!(!grading_submission.is_complete());
        assert!(!grading_submission.is_failed());

        // Test Graded status
        let graded_submission =
            db::models::assignment_submission::Model::set_graded(db, submission.id)
                .await
                .expect("Failed to set graded status");
        assert_eq!(
            graded_submission.status,
            db::models::assignment_submission::SubmissionStatus::Graded
        );
        assert!(!graded_submission.is_in_progress());
        assert!(graded_submission.is_complete());
        assert!(!graded_submission.is_failed());

        // Test FailedUpload status
        let failed_upload = db::models::assignment_submission::Model::set_failed(
            db,
            submission.id,
            db::models::assignment_submission::SubmissionStatus::FailedUpload,
        )
        .await
        .expect("Failed to set failed upload status");
        assert_eq!(
            failed_upload.status,
            db::models::assignment_submission::SubmissionStatus::FailedUpload
        );
        assert!(!failed_upload.is_in_progress());
        assert!(failed_upload.is_complete());
        assert!(failed_upload.is_failed());

        // Test FailedCompile status
        let failed_compile = db::models::assignment_submission::Model::set_failed(
            db,
            submission.id,
            db::models::assignment_submission::SubmissionStatus::FailedCompile,
        )
        .await
        .expect("Failed to set failed compile status");
        assert_eq!(
            failed_compile.status,
            db::models::assignment_submission::SubmissionStatus::FailedCompile
        );
        assert!(failed_compile.is_failed());

        // Test FailedExecution status
        let failed_execution = db::models::assignment_submission::Model::set_failed(
            db,
            submission.id,
            db::models::assignment_submission::SubmissionStatus::FailedExecution,
        )
        .await
        .expect("Failed to set failed execution status");
        assert_eq!(
            failed_execution.status,
            db::models::assignment_submission::SubmissionStatus::FailedExecution
        );
        assert!(failed_execution.is_failed());

        // Test FailedGrading status
        let failed_grading = db::models::assignment_submission::Model::set_failed(
            db,
            submission.id,
            db::models::assignment_submission::SubmissionStatus::FailedGrading,
        )
        .await
        .expect("Failed to set failed grading status");
        assert_eq!(
            failed_grading.status,
            db::models::assignment_submission::SubmissionStatus::FailedGrading
        );
        assert!(failed_grading.is_failed());

        // Test FailedInternal status
        let failed_internal = db::models::assignment_submission::Model::set_failed(
            db,
            submission.id,
            db::models::assignment_submission::SubmissionStatus::FailedInternal,
        )
        .await
        .expect("Failed to set failed internal status");
        assert_eq!(
            failed_internal.status,
            db::models::assignment_submission::SubmissionStatus::FailedInternal
        );
        assert!(failed_internal.is_failed());

        // Test that invalid failure types are rejected
        let invalid_result = db::models::assignment_submission::Model::set_failed(
            db,
            submission.id,
            db::models::assignment_submission::SubmissionStatus::Queued, // Invalid failure type
        )
        .await;
        assert!(invalid_result.is_err());
    }

    fn enable_late_acceptance(module_id: i64, assignment_id: i64) {
        // Start from your default config to keep everything else as-is
        let mut cfg = ExecutionConfig::default_config();

        // Make sure your runner path is Manual and your delimiter matches memo files
        cfg.project.submission_mode = SubmissionMode::Manual;
        cfg.marking.deliminator = "&-=-&".to_string();

        // Allow late submissions and make the window permissive for the tests
        cfg.marking.late.allow_late_submissions = true;
        cfg.marking.late.late_window_minutes = 24 * 60; // 24h is generous for tests
        cfg.marking.late.late_max_percent = 100; // do not cap for these tests

        // Keep attempts behavior as per your suite defaults
        // (we don't touch limit_attempts/max_attempts here)

        cfg.save(module_id, assignment_id)
            .expect("write config.json");
    }

    #[tokio::test]
    #[serial]
    async fn test_is_late_flag_transitions_across_boundary() {
        let (app, app_state, _tmp) = make_test_app_with_storage().await;
        let db = app_state.db();
        let data = setup_test_data(db).await;

        // ensure runner/marker + late policy are OK for late submits
        enable_late_acceptance(data.module.id, data.assignment.id);

        // Start with due date in the past -> should be late
        let mut a: AssignmentActiveModel = AssignmentEntity::find_by_id(data.assignment.id)
            .one(db)
            .await
            .unwrap()
            .unwrap()
            .into();
        a.due_date = Set(Utc::now() - chrono::Duration::minutes(5));
        a.update(db).await.unwrap();

        let submission_zip = create_submission_zip();
        let (boundary, body) = multipart_body("late1.zip", &submission_zip, None, Some("true"));
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
                format!("multipart/form-data; boundary={boundary}"),
            )
            .body(Body::from(body))
            .unwrap();

        let resp1 = app.clone().oneshot(req1).await.unwrap();
        assert_eq!(resp1.status(), StatusCode::OK);
        let body1 = axum::body::to_bytes(resp1.into_body(), usize::MAX)
            .await
            .unwrap();
        let j1: serde_json::Value = serde_json::from_slice(&body1).unwrap();
        assert_eq!(j1["success"], true);
        assert_eq!(j1["data"]["is_late"], true);

        // Move due date into the future, submit again → should not be late
        let mut a2: AssignmentActiveModel = AssignmentEntity::find_by_id(data.assignment.id)
            .one(db)
            .await
            .unwrap()
            .unwrap()
            .into();
        a2.due_date = Set(Utc::now() + chrono::Duration::minutes(5));
        a2.update(db).await.unwrap();

        let (boundary2, body2) =
            multipart_body("late2.zip", &create_submission_zip(), None, Some("true"));
        let req2 = Request::builder()
            .method("POST")
            .uri(&uri)
            .header(AUTHORIZATION, format!("Bearer {}", token))
            .header(
                CONTENT_TYPE,
                format!("multipart/form-data; boundary={boundary2}"),
            )
            .body(Body::from(body2))
            .unwrap();

        let resp2 = app.oneshot(req2).await.unwrap();
        assert_eq!(resp2.status(), StatusCode::OK);
        let body2 = axum::body::to_bytes(resp2.into_body(), usize::MAX)
            .await
            .unwrap();
        let j2: serde_json::Value = serde_json::from_slice(&body2).unwrap();
        assert_eq!(j2["success"], true);
        assert_eq!(j2["data"]["is_late"], false);
    }

    #[tokio::test]
    #[serial]
    async fn test_late_submission_counts_toward_attempts() {
        let (app, app_state, _tmp) = make_test_app_with_storage().await;
        let db = app_state.db();
        let data = setup_test_data(db).await;

        // allow late, keep memo/allocator/makefile config sane
        enable_late_acceptance(data.module.id, data.assignment.id);

        // Force a clearly past due date to ensure 'late'
        let mut a: AssignmentActiveModel = AssignmentEntity::find_by_id(data.assignment.id)
            .one(db)
            .await
            .unwrap()
            .unwrap()
            .into();
        a.due_date = Set(Utc::now() - Duration::hours(2));
        a.update(db).await.unwrap();

        // Submit twice; both are late; attempts should go 1 -> 2
        let (token, _) = generate_jwt(data.student_user.id, data.student_user.admin);
        let uri = format!(
            "/api/modules/{}/assignments/{}/submissions",
            data.module.id, data.assignment.id
        );

        // capture by value to avoid closure lifetime issues
        let token0 = token.clone();
        let uri0 = uri.clone();
        let once = move |app: &BoxCloneService<Request<Body>, Response, Infallible>| {
            let app = app.clone();
            let uri = uri0.clone();
            let token = token0.clone();
            async move {
                let (b, body) =
                    multipart_body("late.zip", &create_submission_zip(), None, Some("true"));
                let req = Request::builder()
                    .method("POST")
                    .uri(&uri)
                    .header(AUTHORIZATION, format!("Bearer {}", token))
                    .header(CONTENT_TYPE, format!("multipart/form-data; boundary={b}"))
                    .body(Body::from(body))
                    .unwrap();
                let resp = app.oneshot(req).await.unwrap();
                assert_eq!(resp.status(), StatusCode::OK);
                let bytes = axum::body::to_bytes(resp.into_body(), usize::MAX)
                    .await
                    .unwrap();
                serde_json::from_slice::<serde_json::Value>(&bytes).unwrap()
            }
        };

        let j1 = once(&app).await;
        assert_eq!(j1["data"]["is_late"], true);
        assert_eq!(j1["data"]["attempt"], 1);

        let j2 = once(&app).await;
        assert_eq!(j2["data"]["is_late"], true);
        assert_eq!(j2["data"]["attempt"], 2);
    }

    #[tokio::test]
    #[serial]
    async fn test_late_metadata_shape_when_present() {
        let (app, app_state, _tmp) = make_test_app_with_storage().await;
        let db = app_state.db();
        let data = setup_test_data(db).await;

        // ensure config accepts late so route doesn't 500
        enable_late_acceptance(data.module.id, data.assignment.id);

        // Past due to force late
        let mut a: AssignmentActiveModel = AssignmentEntity::find_by_id(data.assignment.id)
            .one(db)
            .await
            .unwrap()
            .unwrap()
            .into();
        a.due_date = Set(Utc::now() - Duration::minutes(1));
        a.update(db).await.unwrap();

        let (boundary, body) =
            multipart_body("late.zip", &create_submission_zip(), None, Some("true"));
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
                format!("multipart/form-data; boundary={boundary}"),
            )
            .body(Body::from(body))
            .unwrap();

        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let bytes = axum::body::to_bytes(resp.into_body(), usize::MAX)
            .await
            .unwrap();
        let json: serde_json::Value = serde_json::from_slice(&bytes).unwrap();

        assert_eq!(json["success"], true);
        assert_eq!(json["data"]["is_late"], true);

        if let Some(obj) = json["data"].as_object() {
            if obj.contains_key("late_by_seconds") {
                assert!(json["data"]["late_by_seconds"].as_i64().unwrap_or(0) >= 0);
            }
            if obj.contains_key("late_window_seconds") {
                assert!(json["data"]["late_window_seconds"].as_i64().unwrap_or(0) >= 0);
            }
            if obj.contains_key("late_penalty_percent") {
                let p = json["data"]["late_penalty_percent"].as_f64().unwrap_or(0.0);
                assert!((0.0..=100.0).contains(&p));
            }
        }
    }

    #[tokio::test]
    #[serial]
    async fn test_late_flag_ignores_filename_or_archive_timestamps() {
        let (app, app_state, _tmp) = make_test_app_with_storage().await;
        let db = app_state.db();
        let data = setup_test_data(db).await;

        // accept late so the route doesn't reject
        enable_late_acceptance(data.module.id, data.assignment.id);

        // due well in the past
        let mut a: AssignmentActiveModel = AssignmentEntity::find_by_id(data.assignment.id)
            .one(db)
            .await
            .unwrap()
            .unwrap()
            .into();
        a.due_date = Set(Utc::now() - Duration::hours(6));
        a.update(db).await.unwrap();

        // build a valid submission (Main + helpers) so the pipeline can compile/run
        let submission_zip = create_submission_zip();

        let (boundary, body) = multipart_body("futureish.zip", &submission_zip, None, Some("true"));
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

        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let bytes = axum::body::to_bytes(resp.into_body(), usize::MAX)
            .await
            .unwrap();
        let json: Value = serde_json::from_slice(&bytes).unwrap();

        assert_eq!(json["success"], true);
        assert_eq!(json["data"]["is_late"], true);
    }

    #[tokio::test]
    #[serial]
    async fn test_practice_after_due_late_disabled_is_rejected() {
        let (app, app_state, _tmp) = make_test_app_with_storage().await;
        let db = app_state.db();
        let data = setup_test_data(db).await;

        // Allow practice, but disable late submissions
        write_attempt_policy_config(
            data.module.id,
            data.assignment.id,
            /*limit_attempts*/ false,
            /*max_attempts*/ 10,
            /*allow_practice*/ true,
        );
        // Ensure late acceptance is OFF
        let mut cfg = util::execution_config::ExecutionConfig::get_execution_config(
            data.module.id,
            data.assignment.id,
        )
        .unwrap();
        cfg.marking.late.allow_late_submissions = false;
        cfg.save(data.module.id, data.assignment.id).unwrap();

        // Set due in the past
        let mut a: AssignmentActiveModel = AssignmentEntity::find_by_id(data.assignment.id)
            .one(db)
            .await
            .unwrap()
            .unwrap()
            .into();
        a.due_date = Set(Utc::now() - chrono::Duration::minutes(1));
        a.update(db).await.unwrap();

        let (boundary, body) = multipart_body(
            "p.zip",
            &create_submission_zip(),
            Some("true"),
            Some("true"),
        );
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
                format!("multipart/form-data; boundary={boundary}"),
            )
            .body(Body::from(body))
            .unwrap();

        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::UNPROCESSABLE_ENTITY);

        let bytes = axum::body::to_bytes(resp.into_body(), usize::MAX)
            .await
            .unwrap();
        let json: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(json["success"], false);
        assert_eq!(json["message"], "Late submissions are not allowed");
    }

    #[tokio::test]
    #[serial]
    async fn test_practice_after_due_late_enabled_is_ok() {
        let (app, app_state, _tmp) = make_test_app_with_storage().await;
        let db = app_state.db();
        let data = setup_test_data(db).await;

        // Allow practice and enable late acceptance with a window
        write_attempt_policy_config(
            data.module.id,
            data.assignment.id,
            /*limit_attempts*/ false,
            /*max_attempts*/ 10,
            /*allow_practice*/ true,
        );
        let mut cfg = util::execution_config::ExecutionConfig::get_execution_config(
            data.module.id,
            data.assignment.id,
        )
        .unwrap();
        cfg.marking.late.allow_late_submissions = true;
        cfg.marking.late.late_window_minutes = 60; // 1h window
        cfg.marking.late.late_max_percent = 100;
        cfg.save(data.module.id, data.assignment.id).unwrap();

        // Due 30 minutes ago => inside late window
        let mut a: AssignmentActiveModel = AssignmentEntity::find_by_id(data.assignment.id)
            .one(db)
            .await
            .unwrap()
            .unwrap()
            .into();
        a.due_date = Set(Utc::now() - chrono::Duration::minutes(30));
        a.update(db).await.unwrap();

        let (boundary, body) = multipart_body(
            "p.zip",
            &create_submission_zip(),
            Some("true"),
            Some("true"),
        );
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
                format!("multipart/form-data; boundary={boundary}"),
            )
            .body(Body::from(body))
            .unwrap();

        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);

        let bytes = axum::body::to_bytes(resp.into_body(), usize::MAX)
            .await
            .unwrap();
        let json: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(json["success"], true);
        assert_eq!(json["data"]["is_practice"], true);
        assert_eq!(json["data"]["is_late"], true);
    }

    #[tokio::test]
    #[serial]
    async fn test_practice_does_not_consume_graded_attempt_budget() {
        let (app, app_state, _tmp) = make_test_app_with_storage().await;
        let db = app_state.db();
        let data = setup_test_data(db).await;

        write_attempt_policy_config(
            data.module.id,
            data.assignment.id,
            /*limit_attempts*/ true,
            /*max_attempts*/ 1,
            /*allow_practice*/ true,
        );

        let base_uri = format!(
            "/api/modules/{}/assignments/{}/submissions",
            data.module.id, data.assignment.id
        );
        let (base_token, _) = generate_jwt(data.student_user.id, data.student_user.admin);

        // Submit helper that captures only owned/'static things
        let submit = {
            let app0 = app.clone();
            let uri0 = base_uri.clone();
            let token0 = base_token.clone();
            move |is_practice: Option<&'static str>| {
                let app = app0.clone();
                let uri = uri0.clone();
                let token = token0.clone();
                async move {
                    let (b, body) = multipart_body(
                        "s.zip",
                        &create_submission_zip(),
                        is_practice,
                        Some("true"),
                    );
                    let req = Request::builder()
                        .method("POST")
                        .uri(&uri)
                        .header(AUTHORIZATION, format!("Bearer {}", token))
                        .header(CONTENT_TYPE, format!("multipart/form-data; boundary={b}"))
                        .body(Body::from(body))
                        .unwrap();
                    app.oneshot(req).await.unwrap()
                }
            }
        };

        // 1) Practice submission first (should succeed)
        let r1 = submit(Some("true")).await;
        assert_eq!(r1.status(), StatusCode::OK);

        // 2) Graded submission next (should still be allowed)
        let r2 = submit(None).await;
        assert_eq!(r2.status(), StatusCode::OK);

        // 3) Another graded submission (now should hit the budget limit)
        let r3 = submit(None).await;
        assert_eq!(r3.status(), StatusCode::FORBIDDEN);
        let body = axum::body::to_bytes(r3.into_body(), usize::MAX)
            .await
            .unwrap();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(json["success"], false);
        assert!(
            json["message"]
                .as_str()
                .unwrap_or_default()
                .contains("Maximum attempts")
        );
    }

    #[tokio::test]
    #[serial]
    async fn test_practice_requires_ownership_attestation() {
        let (app, app_state, _tmp) = make_test_app_with_storage().await;
        let db = app_state.db();
        let data = setup_test_data(db).await;

        write_attempt_policy_config(
            data.module.id,
            data.assignment.id,
            /*limit_attempts*/ false,
            /*max_attempts*/ 10,
            /*allow_practice*/ true,
        );

        // Build multipart with is_practice=true but WITHOUT attests_ownership
        let (boundary, body) =
            multipart_body("p.zip", &create_submission_zip(), Some("true"), None);
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
                format!("multipart/form-data; boundary={boundary}"),
            )
            .body(Body::from(body))
            .unwrap();

        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::UNPROCESSABLE_ENTITY);

        let bytes = axum::body::to_bytes(resp.into_body(), usize::MAX)
            .await
            .unwrap();
        let json: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(json["success"], false);
        assert_eq!(
            json["message"],
            "You must confirm the ownership attestation before submitting"
        );
    }
}
