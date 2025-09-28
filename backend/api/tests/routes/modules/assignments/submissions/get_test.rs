#[cfg(test)]
mod tests {
    use crate::helpers::app::make_test_app_with_storage;
    use api::auth::generate_jwt;
    use axum::{
        body::Body,
        http::{Request, StatusCode},
    };
    use chrono::{Duration, Utc};
    use db::models::{
        assignment::Model as AssignmentModel,
        assignment_submission::Model as AssignmentSubmissionModel,
        module::Model as ModuleModel,
        plagiarism_case::{self, Entity as PlagiarismCaseEntity, Status},
        user::Model as UserModel,
        user_module_role::{Model as UserModuleRoleModel, Role},
    };
    use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, IntoActiveModel, QueryFilter, Set};
    use serde_json::{Value, json};
    use serial_test::serial;
    use std::fs;
    use tower::ServiceExt;
    use util::paths::submission_report_path;

    struct TestData {
        lecturer_user: UserModel,
        student_user: UserModel,
        forbidden_user: UserModel,
        module: ModuleModel,
        assignment: AssignmentModel,
        submissions: Vec<AssignmentSubmissionModel>,
    }

    async fn setup_test_data(db: &sea_orm::DatabaseConnection) -> TestData {
        let module = ModuleModel::create(db, "COS101", 2024, Some("Test Module"), 16)
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
            Utc::now(),
            Utc::now() + Duration::days(30),
        )
        .await
        .unwrap();

        let sub1 = AssignmentSubmissionModel::save_file(
            db,
            assignment.id,
            student_user.id,
            1,
            10.0,
            10.0,
            false,
            "ontime.txt",
            "hash123#",
            b"ontime",
        )
        .await
        .unwrap();
        let sub1_time = assignment.due_date - Duration::days(1);
        update_submission_time(db, sub1.id, sub1_time).await;
        write_submission_report(
            module.id,
            assignment.id,
            student_user.id,
            1,
            &sub1,
            false,
            false,
            Some(json!({"earned": 80, "total": 100})),
            sub1_time,
        );

        let sub2 = AssignmentSubmissionModel::save_file(
            db,
            assignment.id,
            student_user.id,
            2,
            10.0,
            10.0,
            false,
            "late.txt",
            "hash123#",
            b"late",
        )
        .await
        .unwrap();
        let sub2_time = assignment.due_date + Duration::days(1);
        update_submission_time(db, sub2.id, sub2_time).await;
        write_submission_report(
            module.id,
            assignment.id,
            student_user.id,
            2,
            &sub2,
            false,
            false,
            Some(json!({"earned": 50, "total": 100})),
            sub2_time,
        );

        let sub3 = AssignmentSubmissionModel::save_file(
            db,
            assignment.id,
            student_user.id,
            3,
            10.0,
            10.0,
            false,
            "practice.txt",
            "hash123#",
            b"practice",
        )
        .await
        .unwrap();
        let sub3_time = assignment.due_date - Duration::days(2);
        update_submission_time(db, sub3.id, sub3_time).await;
        write_submission_report(
            module.id,
            assignment.id,
            student_user.id,
            3,
            &sub3,
            true,
            false,
            Some(json!({"earned": 100, "total": 100})),
            sub3_time,
        );

        let sub4 = AssignmentSubmissionModel::save_file(
            db,
            assignment.id,
            forbidden_user.id,
            1,
            10.0,
            10.0,
            false,
            "forbidden.txt",
            "hash123#",
            b"forbidden",
        )
        .await
        .unwrap();
        let sub4_time = assignment.due_date - Duration::days(1);
        update_submission_time(db, sub4.id, sub4_time).await;
        write_submission_report(
            module.id,
            assignment.id,
            forbidden_user.id,
            1,
            &sub4,
            false,
            false,
            Some(json!({"earned": 0, "total": 100})),
            sub4_time,
        );

        // mark one submission as ignored so we can assert both states
        AssignmentSubmissionModel::set_ignored(db, sub2.id, true)
            .await
            .unwrap();

        let submissions = [sub1, sub2, sub3, sub4].to_vec();

        TestData {
            lecturer_user,
            student_user,
            forbidden_user,
            module,
            assignment,
            submissions,
        }
    }

    async fn update_submission_time(
        db: &sea_orm::DatabaseConnection,
        submission_id: i64,
        new_time: chrono::DateTime<Utc>,
    ) {
        use db::models::assignment_submission::{ActiveModel, Entity};
        use sea_orm::EntityTrait;
        if let Some(model) = Entity::find_by_id(submission_id).one(db).await.unwrap() {
            let mut active: ActiveModel = model.into();
            active.created_at = Set(new_time);
            active.updated_at = Set(new_time);
            let _ = active.update(db).await;
        }
    }

    fn write_submission_report(
        module_id: i64,
        assignment_id: i64,
        user_id: i64,
        attempt: i64,
        submission: &AssignmentSubmissionModel,
        is_practice: bool,
        is_late: bool,
        mark: Option<Value>,
        created_at: chrono::DateTime<Utc>,
    ) {
        let path = submission_report_path(module_id, assignment_id, user_id, attempt);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        let mut report = json!({
            "id": submission.id,
            "attempt": attempt,
            "filename": submission.filename,
            "created_at": created_at.to_rfc3339(),
            "updated_at": created_at.to_rfc3339(),
            "is_practice": is_practice,
            "is_late": is_late,
        });
        if let Some(m) = mark {
            report["mark"] = m;
        }
        fs::write(&path, serde_json::to_string_pretty(&report).unwrap()).unwrap();
    }

    // --- GET /api/modules/{module_id}/assignments/{assignment_id}/submissions ---

    #[tokio::test]
    #[serial]
    async fn test_student_sees_only_own_submissions() {
        let (app, app_state, _tmp) = make_test_app_with_storage().await;
        let data = setup_test_data(app_state.db()).await;

        let (token, _) = generate_jwt(data.student_user.id, data.student_user.admin);
        let uri = format!(
            "/api/modules/{}/assignments/{}/submissions",
            data.module.id, data.assignment.id
        );
        let req = Request::builder()
            .uri(&uri)
            .header("Authorization", format!("Bearer {}", token))
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let json: Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(json["success"], true);

        let submissions = json["data"]["submissions"].as_array().unwrap();
        assert_eq!(submissions.len(), 3);

        // each item has boolean `ignored`
        assert!(submissions.iter().all(|s| s["ignored"].is_boolean()));
        // we set one (sub2) to true
        assert!(submissions.iter().any(|s| s["ignored"] == true));
    }

    #[tokio::test]
    #[serial]
    async fn test_lecturer_sees_all_submissions_with_pagination() {
        let (app, app_state, _tmp) = make_test_app_with_storage().await;
        let data = setup_test_data(app_state.db()).await;

        let (token, _) = generate_jwt(data.lecturer_user.id, data.lecturer_user.admin);
        let uri = format!(
            "/api/modules/{}/assignments/{}/submissions?per_page=2&page=1",
            data.module.id, data.assignment.id
        );
        let req = Request::builder()
            .uri(&uri)
            .header("Authorization", format!("Bearer {}", token))
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let json: Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(json["success"], true);
        assert_eq!(json["data"]["per_page"], 2);
        assert_eq!(json["data"]["page"], 1);
        assert_eq!(json["data"]["total"], 4);

        let subs = json["data"]["submissions"].as_array().unwrap();
        assert_eq!(subs.len(), 2);
        assert!(subs.iter().all(|s| s["ignored"].is_boolean()));
    }

    #[tokio::test]
    #[serial]
    async fn test_query_by_username_returns_only_that_user() {
        let (app, app_state, _tmp) = make_test_app_with_storage().await;
        let data = setup_test_data(app_state.db()).await;

        let (token, _) = generate_jwt(data.lecturer_user.id, data.lecturer_user.admin);
        let uri = format!(
            "/api/modules/{}/assignments/{}/submissions?username=student1",
            data.module.id, data.assignment.id
        );
        let req = Request::builder()
            .uri(&uri)
            .header("Authorization", format!("Bearer {}", token))
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let json: Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(json["success"], true);

        let subs = json["data"]["submissions"].as_array().unwrap();
        assert_eq!(subs.len(), 3);
        assert!(subs.iter().all(|s| s["ignored"].is_boolean()));
    }

    #[tokio::test]
    #[serial]
    async fn test_filter_by_late_status() {
        let (app, app_state, _tmp) = make_test_app_with_storage().await;
        let data = setup_test_data(app_state.db()).await;

        let (token, _) = generate_jwt(data.lecturer_user.id, data.lecturer_user.admin);
        let uri = format!(
            "/api/modules/{}/assignments/{}/submissions?late=true",
            data.module.id, data.assignment.id
        );
        let req = Request::builder()
            .uri(&uri)
            .header("Authorization", format!("Bearer {}", token))
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let json: Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(json["success"], true);

        let subs = json["data"]["submissions"].as_array().unwrap();
        assert_eq!(subs.len(), 1);
        assert!(subs.iter().all(|s| s["ignored"].is_boolean()));
    }

    #[tokio::test]
    #[serial]
    async fn test_forbidden_user_gets_403() {
        let (app, app_state, _tmp) = make_test_app_with_storage().await;
        let data = setup_test_data(app_state.db()).await;

        let (token, _) = generate_jwt(data.forbidden_user.id, data.forbidden_user.admin);
        let uri = format!(
            "/api/modules/{}/assignments/{}/submissions",
            data.module.id, data.assignment.id
        );
        let req = Request::builder()
            .uri(&uri)
            .header("Authorization", format!("Bearer {}", token))
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::FORBIDDEN);
    }

    #[tokio::test]
    #[serial]
    async fn test_filter_by_ignored_true_lecturer() {
        let (app, app_state, _tmp) = make_test_app_with_storage().await;
        let data = setup_test_data(app_state.db()).await;

        let (token, _) = generate_jwt(data.lecturer_user.id, data.lecturer_user.admin);
        let uri = format!(
            "/api/modules/{}/assignments/{}/submissions?ignored=true",
            data.module.id, data.assignment.id
        );
        let req = Request::builder()
            .uri(&uri)
            .header("Authorization", format!("Bearer {}", token))
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let json: Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(json["success"], true);

        let subs = json["data"]["submissions"].as_array().unwrap();
        // Only sub2 is ignored in the fixture
        assert_eq!(subs.len(), 1);
        assert!(subs.iter().all(|s| s["ignored"] == true));
    }

    #[tokio::test]
    #[serial]
    async fn test_filter_by_ignored_false_lecturer() {
        let (app, app_state, _tmp) = make_test_app_with_storage().await;
        let data = setup_test_data(app_state.db()).await;

        let (token, _) = generate_jwt(data.lecturer_user.id, data.lecturer_user.admin);
        let uri = format!(
            "/api/modules/{}/assignments/{}/submissions?ignored=false",
            data.module.id, data.assignment.id
        );
        let req = Request::builder()
            .uri(&uri)
            .header("Authorization", format!("Bearer {}", token))
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let json: Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(json["success"], true);

        let subs = json["data"]["submissions"].as_array().unwrap();
        // 4 total, 1 ignored (sub2) → 3 non-ignored
        assert_eq!(subs.len(), 3);
        assert!(subs.iter().all(|s| s["ignored"] == false));
    }

    #[tokio::test]
    #[serial]
    async fn test_filter_by_ignored_true_student_scope() {
        let (app, app_state, _tmp) = make_test_app_with_storage().await;
        let data = setup_test_data(app_state.db()).await;

        let (token, _) = generate_jwt(data.student_user.id, data.student_user.admin);
        let uri = format!(
            "/api/modules/{}/assignments/{}/submissions?ignored=true",
            data.module.id, data.assignment.id
        );
        let req = Request::builder()
            .uri(&uri)
            .header("Authorization", format!("Bearer {}", token))
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let json: Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(json["success"], true);

        let subs = json["data"]["submissions"].as_array().unwrap();
        // Student has 3 submissions; only sub2 is ignored → expect 1
        assert_eq!(subs.len(), 1);
        assert!(subs.iter().all(|s| s["ignored"] == true));
    }

    // --- GET /api/modules/{module_id}/assignments/{assignment_id}/submissions/{submission_id} ---

    #[tokio::test]
    #[serial]
    async fn test_student_gets_own_submission() {
        let (app, app_state, _tmp) = make_test_app_with_storage().await;
        let data = setup_test_data(app_state.db()).await;

        let (token, _) = generate_jwt(data.student_user.id, data.student_user.admin);
        let sub = &data.submissions[0];
        let uri = format!(
            "/api/modules/{}/assignments/{}/submissions/{}",
            data.module.id, data.assignment.id, sub.id
        );
        let req = Request::builder()
            .uri(&uri)
            .header("Authorization", format!("Bearer {}", token))
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let json: Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(json["success"], true);
        assert_eq!(json["data"]["id"], sub.id);
        assert_eq!(json["data"]["plagiarism"]["flagged"], false);
        assert_eq!(json["data"]["plagiarism"]["similarity"], 0.0);
        assert_eq!(json["data"]["plagiarism"]["lines_matched"], 0);
        assert_eq!(json["data"]["plagiarism"]["description"], "");
    }

    #[tokio::test]
    #[serial]
    async fn test_lecturer_gets_any_submission_with_user_info() {
        let (app, app_state, _tmp) = make_test_app_with_storage().await;
        let data = setup_test_data(app_state.db()).await;

        let (token, _) = generate_jwt(data.lecturer_user.id, data.lecturer_user.admin);
        let sub = &data.submissions[0];
        let uri = format!(
            "/api/modules/{}/assignments/{}/submissions/{}",
            data.module.id, data.assignment.id, sub.id
        );
        let req = Request::builder()
            .uri(&uri)
            .header("Authorization", format!("Bearer {}", token))
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let json: Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(json["success"], true);
        assert_eq!(json["data"]["id"], sub.id);
        assert!(json["data"]["user"].is_object());
        assert_eq!(json["data"]["plagiarism"]["flagged"], false);
        assert_eq!(json["data"]["plagiarism"]["similarity"], 0.0);
        assert_eq!(json["data"]["plagiarism"]["lines_matched"], 0);
        assert_eq!(json["data"]["plagiarism"]["description"], "");
    }

    #[tokio::test]
    #[serial]
    async fn test_forbidden_user_gets_403_on_submission() {
        let (app, app_state, _tmp) = make_test_app_with_storage().await;
        let data = setup_test_data(app_state.db()).await;

        let (token, _) = generate_jwt(data.forbidden_user.id, data.forbidden_user.admin);
        let sub = &data.submissions[0];
        let uri = format!(
            "/api/modules/{}/assignments/{}/submissions/{}",
            data.module.id, data.assignment.id, sub.id
        );
        let req = Request::builder()
            .uri(&uri)
            .header("Authorization", format!("Bearer {}", token))
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::FORBIDDEN);
    }

    #[tokio::test]
    #[serial]
    async fn test_submission_not_found_returns_404() {
        let (app, app_state, _tmp) = make_test_app_with_storage().await;
        let data = setup_test_data(app_state.db()).await;

        let (token, _) = generate_jwt(data.lecturer_user.id, data.lecturer_user.admin);
        let uri = format!(
            "/api/modules/{}/assignments/{}/submissions/999999",
            data.module.id, data.assignment.id
        );
        let req = Request::builder()
            .uri(&uri)
            .header("Authorization", format!("Bearer {}", token))
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    #[serial]
    async fn test_submission_report_missing_returns_404() {
        let (app, app_state, _tmp) = make_test_app_with_storage().await;
        let data = setup_test_data(app_state.db()).await;

        let sub = &data.submissions[0];
        let path = submission_report_path(
            data.module.id,
            data.assignment.id,
            data.student_user.id,
            sub.attempt,
        );
        let _ = fs::remove_file(&path);

        let (token, _) = generate_jwt(data.lecturer_user.id, data.lecturer_user.admin);
        let uri = format!(
            "/api/modules/{}/assignments/{}/submissions/{}",
            data.module.id, data.assignment.id, sub.id
        );
        let req = Request::builder()
            .uri(&uri)
            .header("Authorization", format!("Bearer {}", token))
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    #[serial]
    async fn test_submission_list_contains_status_field_student() {
        let (app, app_state, _tmp) = make_test_app_with_storage().await;
        let data = setup_test_data(app_state.db()).await;

        // Update one submission to have a different status
        use db::models::assignment_submission::Model as SubmissionModel;
        let _ = SubmissionModel::set_running(app_state.db(), data.submissions[1].id).await;

        let (token, _) = generate_jwt(data.student_user.id, data.student_user.admin);
        let uri = format!(
            "/api/modules/{}/assignments/{}/submissions",
            data.module.id, data.assignment.id
        );
        let req = Request::builder()
            .uri(&uri)
            .header("Authorization", format!("Bearer {}", token))
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let json: Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(json["success"], true);

        let submissions = json["data"]["submissions"].as_array().unwrap();
        assert_eq!(submissions.len(), 3); // Student only sees their own submissions

        // Every submission should have a status field
        for submission in submissions {
            assert!(
                submission["status"].is_string(),
                "Submission should have status field"
            );
            let status = submission["status"].as_str().unwrap();
            // Status should be one of the valid submission statuses
            assert!(
                matches!(
                    status,
                    "queued"
                        | "running"
                        | "grading"
                        | "graded"
                        | "failed_upload"
                        | "failed_compile"
                        | "failed_execution"
                        | "failed_grading"
                        | "failed_internal"
                ),
                "Invalid status value: {}",
                status
            );
        }

        // Check that we have at least one submission with "running" status
        let has_running = submissions.iter().any(|s| s["status"] == "running");
        assert!(
            has_running,
            "Should have at least one submission with running status"
        );
    }

    #[tokio::test]
    #[serial]
    async fn test_submission_list_contains_status_field_lecturer() {
        let (app, app_state, _tmp) = make_test_app_with_storage().await;
        let data = setup_test_data(app_state.db()).await;

        // Set different statuses for different submissions to test variety
        use db::models::assignment_submission::{Model as SubmissionModel, SubmissionStatus};
        let _ = SubmissionModel::set_grading(app_state.db(), data.submissions[0].id).await;
        let _ = SubmissionModel::set_graded(app_state.db(), data.submissions[1].id).await;
        let _ = SubmissionModel::set_failed(
            app_state.db(),
            data.submissions[2].id,
            SubmissionStatus::FailedCompile,
        )
        .await;

        let (token, _) = generate_jwt(data.lecturer_user.id, data.lecturer_user.admin);
        let uri = format!(
            "/api/modules/{}/assignments/{}/submissions",
            data.module.id, data.assignment.id
        );
        let req = Request::builder()
            .uri(&uri)
            .header("Authorization", format!("Bearer {}", token))
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let json: Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(json["success"], true);

        let submissions = json["data"]["submissions"].as_array().unwrap();
        assert_eq!(submissions.len(), 4); // Lecturer sees all submissions

        // Every submission should have a status field
        for submission in submissions {
            assert!(
                submission["status"].is_string(),
                "Submission should have status field"
            );
        }

        // Collect all status values
        let statuses: Vec<&str> = submissions
            .iter()
            .map(|s| s["status"].as_str().unwrap())
            .collect();

        // We should have different statuses due to our updates above
        assert!(
            statuses.contains(&"grading"),
            "Should have at least one grading status"
        );
        assert!(
            statuses.contains(&"graded"),
            "Should have at least one graded status"
        );
        assert!(
            statuses.contains(&"failed_compile"),
            "Should have at least one failed_compile status"
        );
    }

    #[tokio::test]
    #[serial]
    async fn test_submission_status_values_are_correct() {
        let (app, app_state, _tmp) = make_test_app_with_storage().await;
        let data = setup_test_data(app_state.db()).await;

        // Test various status transitions and verify they appear correctly in API
        use db::models::assignment_submission::{Model as SubmissionModel, SubmissionStatus};

        // Test each status type
        let test_cases = vec![
            (data.submissions[0].id, SubmissionStatus::Running, "running"),
            (data.submissions[1].id, SubmissionStatus::Grading, "grading"),
            (data.submissions[2].id, SubmissionStatus::Graded, "graded"),
        ];

        for (submission_id, db_status, expected_api_status) in test_cases {
            // Update submission to specific status using the same app_state
            match db_status {
                SubmissionStatus::Running => {
                    let _ = SubmissionModel::set_running(app_state.db(), submission_id).await;
                }
                SubmissionStatus::Grading => {
                    let _ = SubmissionModel::set_grading(app_state.db(), submission_id).await;
                }
                SubmissionStatus::Graded => {
                    let _ = SubmissionModel::set_graded(app_state.db(), submission_id).await;
                }
                _ => {} // Skip other statuses for this test
            }

            // Fetch submissions and verify status
            let (token, _) = generate_jwt(data.lecturer_user.id, data.lecturer_user.admin);
            let uri = format!(
                "/api/modules/{}/assignments/{}/submissions",
                data.module.id, data.assignment.id
            );
            let req = Request::builder()
                .uri(&uri)
                .header("Authorization", format!("Bearer {}", token))
                .body(Body::empty())
                .unwrap();

            let response = app.clone().oneshot(req).await.unwrap();
            assert_eq!(response.status(), StatusCode::OK);

            let body = axum::body::to_bytes(response.into_body(), usize::MAX)
                .await
                .unwrap();
            let json: Value = serde_json::from_slice(&body).unwrap();

            let submissions = json["data"]["submissions"].as_array().unwrap();
            let target_submission = submissions
                .iter()
                .find(|s| s["id"].as_i64().unwrap() == submission_id)
                .expect("Should find submission");

            assert_eq!(
                target_submission["status"].as_str().unwrap(),
                expected_api_status,
                "Status mismatch for submission {}",
                submission_id
            );
        }
    }

    #[tokio::test]
    #[serial]
    async fn test_get_submission_with_flagged_plagiarism_case() {
        let (app, app_state, _tmp) = make_test_app_with_storage().await;
        let data = setup_test_data(app_state.db()).await;

        let sub1 = &data.submissions[0];
        let sub2 = &data.submissions[1];

        let _ = plagiarism_case::Model::create_case(
            app_state.db(),
            data.assignment.id,
            sub1.id,
            sub2.id,
            "Plagiarism detected between sub1 and sub2",
            75.5,
            150,
            None,
        )
        .await
        .unwrap();

        let mut active_case = PlagiarismCaseEntity::find()
            .filter(plagiarism_case::Column::SubmissionId1.eq(sub1.id))
            .one(app_state.db())
            .await
            .unwrap()
            .unwrap()
            .into_active_model();
        active_case.status = Set(Status::Flagged);
        active_case.update(app_state.db()).await.unwrap();

        let (token, _) = generate_jwt(data.lecturer_user.id, data.lecturer_user.admin);
        let uri = format!(
            "/api/modules/{}/assignments/{}/submissions/{}",
            data.module.id, data.assignment.id, sub1.id
        );
        let req = Request::builder()
            .uri(&uri)
            .header("Authorization", format!("Bearer {}", token))
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let json: Value = serde_json::from_slice(&body).unwrap();

        assert_eq!(json["success"], true);
        assert_eq!(json["data"]["id"], sub1.id);
        assert_eq!(json["data"]["plagiarism"]["flagged"], true);
        assert_eq!(json["data"]["plagiarism"]["similarity"], 75.5);
        assert_eq!(json["data"]["plagiarism"]["lines_matched"], 150);
        assert_eq!(
            json["data"]["plagiarism"]["description"],
            "Plagiarism detected between sub1 and sub2"
        );
    }

    #[tokio::test]
    #[serial]
    async fn test_get_submission_with_multiple_flagged_plagiarism_cases_highest_similarity() {
        let (app, app_state, _tmp) = make_test_app_with_storage().await;
        let data = setup_test_data(app_state.db()).await;

        let sub1 = &data.submissions[0];
        let sub2 = &data.submissions[1];
        let sub3 = &data.submissions[2];

        let case1 = plagiarism_case::Model::create_case(
            app_state.db(),
            data.assignment.id,
            sub1.id,
            sub2.id,
            "Plagiarism case 1",
            75.5,
            150,
            None,
        )
        .await
        .unwrap();
        let mut active_case1 = case1.into_active_model();
        active_case1.status = Set(Status::Flagged);
        active_case1.update(app_state.db()).await.unwrap();

        let case2 = plagiarism_case::Model::create_case(
            app_state.db(),
            data.assignment.id,
            sub1.id,
            sub3.id,
            "Plagiarism case 2 (higher similarity)",
            85.0,
            200,
            None,
        )
        .await
        .unwrap();
        let mut active_case2 = case2.into_active_model();
        active_case2.status = Set(Status::Flagged);
        active_case2.update(app_state.db()).await.unwrap();

        let case3 = plagiarism_case::Model::create_case(
            app_state.db(),
            data.assignment.id,
            sub1.id,
            data.submissions[3].id,
            "Plagiarism case 3 (lower similarity)",
            60.0,
            100,
            None,
        )
        .await
        .unwrap();
        let mut active_case3 = case3.into_active_model();
        active_case3.status = Set(Status::Flagged);
        active_case3.update(app_state.db()).await.unwrap();

        let (token, _) = generate_jwt(data.lecturer_user.id, data.lecturer_user.admin);
        let uri = format!(
            "/api/modules/{}/assignments/{}/submissions/{}",
            data.module.id, data.assignment.id, sub1.id
        );
        let req = Request::builder()
            .uri(&uri)
            .header("Authorization", format!("Bearer {}", token))
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let json: Value = serde_json::from_slice(&body).unwrap();

        assert_eq!(json["success"], true);
        assert_eq!(json["data"]["id"], sub1.id);
        assert_eq!(json["data"]["plagiarism"]["flagged"], true);
        assert_eq!(json["data"]["plagiarism"]["similarity"], 85.0);
        assert_eq!(json["data"]["plagiarism"]["lines_matched"], 200);
        assert_eq!(
            json["data"]["plagiarism"]["description"],
            "Plagiarism case 2 (higher similarity)"
        );
    }
}
