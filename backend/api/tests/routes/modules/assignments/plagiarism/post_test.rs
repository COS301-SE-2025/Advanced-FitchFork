#[cfg(test)]
mod create_plagiarism_tests {
    use crate::helpers::app::make_test_app_with_storage;
    use api::auth::generate_jwt;
    use api::routes::modules::assignments::plagiarism::post::CreatePlagiarismCasePayload;
    use axum::{
        body::Body as AxumBody,
        http::{Request, StatusCode},
    };
    use chrono::{Datelike, TimeZone, Utc};
    use db::models::{
        assignment::{AssignmentType, Model as AssignmentModel},
        assignment_submission::Model as SubmissionModel,
        module::Model as ModuleModel,
        plagiarism_case::{Entity as PlagiarismCaseEntity, Status},
        user::Model as UserModel,
        user_module_role::{Model as UserModuleRoleModel, Role},
    };
    use sea_orm::{DatabaseConnection, EntityTrait};
    use serde_json::{Value, json};
    use tower::ServiceExt;

    // Small helper for float compare
    fn approx_eq_f64(a: f64, b: f64, eps: f64) -> bool {
        (a - b).abs() <= eps
    }

    struct TestData {
        lecturer_user: UserModel,
        assistant_user: UserModel,
        tutor_user: UserModel,
        student_user1: UserModel,
        module: ModuleModel,
        assignment: AssignmentModel,
        submission1: SubmissionModel,
        submission2: SubmissionModel,
    }

    async fn setup_test_data(db: &DatabaseConnection) -> TestData {
        dotenvy::dotenv().ok();

        let module = ModuleModel::create(db, "CS101", Utc::now().year(), Some("Intro to CS"), 5)
            .await
            .expect("Failed to create test module");

        let lecturer_user =
            UserModel::create(db, "lecturer", "lecturer@test.com", "password", false)
                .await
                .expect("Failed to create lecturer user");
        let assistant_user =
            UserModel::create(db, "assistant", "assistant@test.com", "password", false)
                .await
                .expect("Failed to create assistant user");
        let tutor_user = UserModel::create(db, "tutor", "tutor@test.com", "password", false)
            .await
            .expect("Failed to create tutor user");
        let student_user1 =
            UserModel::create(db, "student1", "student1@test.com", "password", false)
                .await
                .expect("Failed to create student1 user");
        let student_user2 =
            UserModel::create(db, "student2", "student2@test.com", "password", false)
                .await
                .expect("Failed to create student2 user");

        UserModuleRoleModel::assign_user_to_module(db, lecturer_user.id, module.id, Role::Lecturer)
            .await
            .unwrap();
        UserModuleRoleModel::assign_user_to_module(
            db,
            assistant_user.id,
            module.id,
            Role::AssistantLecturer,
        )
        .await
        .unwrap();
        UserModuleRoleModel::assign_user_to_module(db, tutor_user.id, module.id, Role::Tutor)
            .await
            .unwrap();
        UserModuleRoleModel::assign_user_to_module(db, student_user1.id, module.id, Role::Student)
            .await
            .unwrap();
        UserModuleRoleModel::assign_user_to_module(db, student_user2.id, module.id, Role::Student)
            .await
            .unwrap();

        let assignment = AssignmentModel::create(
            db,
            module.id,
            "Assignment 1",
            Some("Desc 1"),
            AssignmentType::Assignment,
            Utc.with_ymd_and_hms(2024, 1, 1, 0, 0, 0).unwrap(),
            Utc.with_ymd_and_hms(2024, 1, 31, 23, 59, 59).unwrap(),
        )
        .await
        .unwrap();

        let submission1 = SubmissionModel::save_file(
            db,
            assignment.id,
            student_user1.id,
            1,
            10,
            10,
            false,
            "sub1.txt",
            "hash123#",
            b"ontime",
        )
        .await
        .unwrap();

        let submission2 = SubmissionModel::save_file(
            db,
            assignment.id,
            student_user2.id,
            1,
            10,
            10,
            false,
            "sub2.txt",
            "hash123#",
            b"ontime",
        )
        .await
        .unwrap();

        TestData {
            lecturer_user,
            assistant_user,
            tutor_user,
            student_user1,
            module,
            assignment,
            submission1,
            submission2,
        }
    }

    fn make_post_request(
        user: &UserModel,
        module_id: i64,
        assignment_id: i64,
        payload: CreatePlagiarismCasePayload,
    ) -> Request<AxumBody> {
        let (token, _) = generate_jwt(user.id, user.admin);
        let uri = format!(
            "/api/modules/{}/assignments/{}/plagiarism",
            module_id, assignment_id
        );
        let body = AxumBody::from(serde_json::to_string(&payload).unwrap());

        Request::builder()
            .method("POST")
            .uri(&uri)
            .header("Authorization", format!("Bearer {}", token))
            .header("Content-Type", "application/json")
            .body(body)
            .unwrap()
    }

    /// Test Case: Successful Creation by Lecturer (explicit similarity)
    #[tokio::test]
    #[serial]
    async fn test_create_plagiarism_case_success_as_lecturer() {
        let (app, app_state, _tmp) = make_test_app_with_storage().await;
        let data = setup_test_data(app_state.db()).await;

        let payload = CreatePlagiarismCasePayload {
            submission_id_1: data.submission1.id,
            submission_id_2: data.submission2.id,
            description: "Code similarity detected".to_string(),
            similarity: 67.5,
            lines_matched: 0,
            report_id: None,
        };

        let req = make_post_request(
            &data.lecturer_user,
            data.module.id,
            data.assignment.id,
            payload,
        );

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::CREATED);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let json: Value = serde_json::from_slice(&body).unwrap();

        assert_eq!(json["success"], true);
        assert_eq!(json["message"], "Plagiarism case created successfully");

        let case_data = &json["data"];
        assert!(case_data["id"].is_i64());
        assert_eq!(case_data["assignment_id"], data.assignment.id);
        assert_eq!(case_data["submission_id_1"], data.submission1.id);
        assert_eq!(case_data["submission_id_2"], data.submission2.id);
        assert_eq!(case_data["description"], "Code similarity detected");
        assert_eq!(case_data["status"], "review");
        assert!(case_data["created_at"].is_string());
        assert!(case_data["updated_at"].is_string());
        assert!(case_data["similarity"].is_number());
        assert!(approx_eq_f64(
            case_data["similarity"].as_f64().unwrap(),
            67.5,
            1e-6
        ));
        assert_eq!(case_data["lines_matched"], 0); // server defaults to 0
        assert!(case_data["report_id"].is_null()); // manually created case has no report

        // Verify DB row
        let case = PlagiarismCaseEntity::find_by_id(case_data["id"].as_i64().unwrap())
            .one(db::get_connection().await)
            .await
            .unwrap()
            .expect("Plagiarism case should exist");
        assert_eq!(case.status, Status::Review);
        assert!((case.similarity as f64 - 67.5).abs() < 1e-6);
    }

    /// Test Case: Successful Creation by Assistant Lecturer (explicit 0.0 similarity)
    #[tokio::test]
    #[serial]
    async fn test_create_plagiarism_case_success_as_assistant() {
        let (app, app_state, _tmp) = make_test_app_with_storage().await;
        let data = setup_test_data(app_state.db()).await;

        let payload = CreatePlagiarismCasePayload {
            submission_id_1: data.submission1.id,
            submission_id_2: data.submission2.id,
            description: "Similar solution structure".to_string(),
            similarity: 0.0,
            lines_matched: 0,
            report_id: None,
        };

        let req = make_post_request(
            &data.assistant_user,
            data.module.id,
            data.assignment.id,
            payload,
        );

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::CREATED);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let json: Value = serde_json::from_slice(&body).unwrap();
        let case_data = &json["data"];
        assert!(approx_eq_f64(
            case_data["similarity"].as_f64().unwrap(),
            0.0,
            1e-6
        ));
    }

    /// Test Case: Forbidden Access for Tutor
    #[tokio::test]
    #[serial]
    async fn test_create_plagiarism_case_forbidden_as_tutor() {
        let (app, app_state, _tmp) = make_test_app_with_storage().await;
        let data = setup_test_data(app_state.db()).await;

        let payload = CreatePlagiarismCasePayload {
            submission_id_1: data.submission1.id,
            submission_id_2: data.submission2.id,
            description: "Tutor should not access".to_string(),
            similarity: 10.0,
            lines_matched: 0,
            report_id: None,
        };

        let req = make_post_request(
            &data.tutor_user,
            data.module.id,
            data.assignment.id,
            payload,
        );

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::FORBIDDEN);
    }

    /// Test Case: Same Submission IDs Validation
    #[tokio::test]
    #[serial]
    async fn test_create_plagiarism_case_same_submission_ids() {
        let (app, app_state, _tmp) = make_test_app_with_storage().await;
        let data = setup_test_data(app_state.db()).await;

        let payload = CreatePlagiarismCasePayload {
            submission_id_1: data.submission1.id,
            submission_id_2: data.submission1.id, // Same submission
            description: "Invalid same submission".to_string(),
            similarity: 50.0,
            lines_matched: 0,
            report_id: None,
        };

        let req = make_post_request(
            &data.lecturer_user,
            data.module.id,
            data.assignment.id,
            payload,
        );

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let json: Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(json["success"], false);
        assert_eq!(json["message"], "Submissions cannot be the same");
    }

    /// Test Case: Submission Not Found Validation
    #[tokio::test]
    #[serial]
    async fn test_create_plagiarism_case_submission_not_found() {
        let (app, app_state, _tmp) = make_test_app_with_storage().await;
        let data = setup_test_data(app_state.db()).await;

        let payload = CreatePlagiarismCasePayload {
            submission_id_1: data.submission1.id,
            submission_id_2: 999999, // Non-existent submission
            description: "Invalid submission".to_string(),
            similarity: 15.0,
            lines_matched: 0,
            report_id: None,
        };

        let req = make_post_request(
            &data.lecturer_user,
            data.module.id,
            data.assignment.id,
            payload,
        );

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let json: Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(json["success"], false);
        assert_eq!(
            json["message"],
            "One or both submissions do not exist or belong to a different assignment"
        );
    }

    /// Test Case: Submission from Different Assignment
    #[tokio::test]
    #[serial]
    async fn test_create_plagiarism_case_wrong_assignment() {
        let (app, app_state, _tmp) = make_test_app_with_storage().await;
        let data = setup_test_data(app_state.db()).await;

        // Create another assignment and submission
        let other_assignment = AssignmentModel::create(
            db::get_connection().await,
            data.module.id,
            "Other Assignment",
            None,
            AssignmentType::Assignment,
            Utc.with_ymd_and_hms(2024, 2, 1, 0, 0, 0).unwrap(),
            Utc.with_ymd_and_hms(2024, 2, 28, 23, 59, 59).unwrap(),
        )
        .await
        .unwrap();

        let other_submission = SubmissionModel::save_file(
            db::get_connection().await,
            other_assignment.id,
            data.student_user1.id,
            1,
            10,
            10,
            false,
            "other.txt",
            "hash456#",
            b"ontime",
        )
        .await
        .unwrap();

        let payload = CreatePlagiarismCasePayload {
            submission_id_1: data.submission1.id,
            submission_id_2: other_submission.id, // From different assignment
            description: "Cross-assignment submission".to_string(),
            similarity: 88.0,
            lines_matched: 0,
            report_id: None,
        };

        let req = make_post_request(
            &data.lecturer_user,
            data.module.id,
            data.assignment.id, // Current assignment
            payload,
        );

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let json: Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(json["success"], false);
        assert_eq!(
            json["message"],
            "One or both submissions do not exist or belong to a different assignment"
        );
    }

    /// Test Case: Missing Authorization Header
    #[tokio::test]
    #[serial]
    async fn test_create_plagiarism_case_unauthorized() {
        let (app, app_state, _tmp) = make_test_app_with_storage().await;
        let data = setup_test_data(app_state.db()).await;

        let payload = CreatePlagiarismCasePayload {
            submission_id_1: data.submission1.id,
            submission_id_2: data.submission2.id,
            description: "Unauthorized attempt".to_string(),
            similarity: 10.0,
            lines_matched: 0,
            report_id: None,
        };

        let uri = format!(
            "/api/modules/{}/assignments/{}/plagiarism",
            data.module.id, data.assignment.id
        );

        let req = Request::builder()
            .method("POST")
            .uri(&uri)
            .header("Content-Type", "application/json")
            .body(AxumBody::from(serde_json::to_string(&payload).unwrap()))
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    /// Test Case: Invalid Payload Format (serde 422)
    #[tokio::test]
    #[serial]
    async fn test_create_plagiarism_case_invalid_payload() {
        let (app, app_state, _tmp) = make_test_app_with_storage().await;
        let data = setup_test_data(app_state.db()).await;

        // Wrong types + missing required fields => 422
        let invalid_payload = json!({
            "submission_id_1": "not_a_number",
            "description": "Missing numeric fields"
            // submission_id_2 missing, similarity missing, etc.
        });

        let (token, _) = generate_jwt(data.lecturer_user.id, data.lecturer_user.admin);
        let uri = format!(
            "/api/modules/{}/assignments/{}/plagiarism",
            data.module.id, data.assignment.id
        );

        let req = Request::builder()
            .method("POST")
            .uri(&uri)
            .header("Authorization", format!("Bearer {}", token))
            .header("Content-Type", "application/json")
            .body(AxumBody::from(invalid_payload.to_string()))
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
    }

    /// NEW: Missing similarity alone should 422 (required field)
    #[tokio::test]
    async fn test_create_plagiarism_case_missing_similarity_is_422() {
        let (app, app_state, _tmp) = make_test_app_with_storage().await;
        let data = setup_test_data(app_state.db()).await;

        // Build JSON manually without "similarity"
        let invalid_payload = json!({
            "submission_id_1": data.submission1.id,
            "submission_id_2": data.submission2.id,
            "description": "No similarity provided"
        });

        let (token, _) = generate_jwt(data.lecturer_user.id, data.lecturer_user.admin);
        let uri = format!(
            "/api/modules/{}/assignments/{}/plagiarism",
            data.module.id, data.assignment.id
        );
        let req = Request::builder()
            .method("POST")
            .uri(&uri)
            .header("Authorization", format!("Bearer {}", token))
            .header("Content-Type", "application/json")
            .body(AxumBody::from(invalid_payload.to_string()))
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
    }

    /// Test Case: Similarity boundary 0.0 and 100.0 are accepted
    #[tokio::test]
    async fn test_create_plagiarism_case_similarity_boundaries() {
        let (app, app_state, _tmp) = make_test_app_with_storage().await;
        let data = setup_test_data(app_state.db()).await;

        // 0.0
        let payload0 = CreatePlagiarismCasePayload {
            submission_id_1: data.submission1.id,
            submission_id_2: data.submission2.id,
            description: "Boundary 0".to_string(),
            similarity: 0.0,
            lines_matched: 0,
            report_id: None,
        };
        let req0 = make_post_request(
            &data.lecturer_user,
            data.module.id,
            data.assignment.id,
            payload0,
        );
        let resp0 = app.clone().oneshot(req0).await.unwrap();
        assert_eq!(resp0.status(), StatusCode::CREATED);
        let body0 = axum::body::to_bytes(resp0.into_body(), usize::MAX)
            .await
            .unwrap();
        let json0: Value = serde_json::from_slice(&body0).unwrap();
        assert!(approx_eq_f64(
            json0["data"]["similarity"].as_f64().unwrap(),
            0.0,
            1e-6
        ));

        // 100.0
        let payload100 = CreatePlagiarismCasePayload {
            submission_id_1: data.submission1.id,
            submission_id_2: data.submission2.id,
            description: "Boundary 100".to_string(),
            similarity: 100.0,
            lines_matched: 0,
            report_id: None,
        };
        let req100 = make_post_request(
            &data.lecturer_user,
            data.module.id,
            data.assignment.id,
            payload100,
        );
        let resp100 = app.oneshot(req100).await.unwrap();
        assert_eq!(resp100.status(), StatusCode::CREATED);
        let body100 = axum::body::to_bytes(resp100.into_body(), usize::MAX)
            .await
            .unwrap();
        let json100: Value = serde_json::from_slice(&body100).unwrap();
        assert!(approx_eq_f64(
            json100["data"]["similarity"].as_f64().unwrap(),
            100.0,
            1e-6
        ));
    }

    /// Test Case: Similarity out of range (negative) -> 400
    #[tokio::test]
    async fn test_create_plagiarism_case_similarity_too_low() {
        let (app, app_state, _tmp) = make_test_app_with_storage().await;
        let data = setup_test_data(app_state.db()).await;

        let payload = CreatePlagiarismCasePayload {
            submission_id_1: data.submission1.id,
            submission_id_2: data.submission2.id,
            description: "Too low".to_string(),
            similarity: -1.0,
            lines_matched: 0,
            report_id: None,
        };

        let req = make_post_request(
            &data.lecturer_user,
            data.module.id,
            data.assignment.id,
            payload,
        );
        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
    }

    /// Test Case: Similarity out of range (>100) -> 400
    #[tokio::test]
    async fn test_create_plagiarism_case_similarity_too_high() {
        let (app, app_state, _tmp) = make_test_app_with_storage().await;
        let data = setup_test_data(app_state.db()).await;

        let payload = CreatePlagiarismCasePayload {
            submission_id_1: data.submission1.id,
            submission_id_2: data.submission2.id,
            description: "Too high".to_string(),
            similarity: 120.0,
            lines_matched: 0,
            report_id: None,
        };

        let req = make_post_request(
            &data.lecturer_user,
            data.module.id,
            data.assignment.id,
            payload,
        );
        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
    }

    /// NEW: Similarity with fractional value is preserved (precision check)
    #[tokio::test]
    async fn test_create_plagiarism_case_similarity_fractional_precision() {
        let (app, app_state, _tmp) = make_test_app_with_storage().await;
        let data = setup_test_data(app_state.db()).await;

        let sim = 33.333_f32;
        let payload = CreatePlagiarismCasePayload {
            submission_id_1: data.submission1.id,
            submission_id_2: data.submission2.id,
            description: "Fractional".to_string(),
            similarity: sim,
            lines_matched: 0,
            report_id: None,
        };

        let req = make_post_request(
            &data.lecturer_user,
            data.module.id,
            data.assignment.id,
            payload,
        );
        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::CREATED);

        let body = axum::body::to_bytes(resp.into_body(), usize::MAX)
            .await
            .unwrap();
        let json: Value = serde_json::from_slice(&body).unwrap();
        let v = json["data"]["similarity"].as_f64().unwrap();
        assert!(approx_eq_f64(v, sim as f64, 1e-3)); // allow small float error
    }
}

#[cfg(test)]
mod hash_scan_tests {
    use crate::helpers::app::make_test_app_with_storage;
    use api::auth::generate_jwt;
    use api::routes::modules::assignments::plagiarism::post::{HashScanPayload, HashScanResponse};
    use axum::{
        body::Body as AxumBody,
        http::{Request, StatusCode},
    };
    use chrono::{Datelike, TimeZone, Utc};
    use db::models::{
        assignment::{AssignmentType, Model as AssignmentModel},
        assignment_submission::Model as SubmissionModel,
        module::Model as ModuleModel,
        plagiarism_case::{Entity as PlagiarismCaseEntity, Model as PlagiarismCaseModel, Status},
        user::Model as UserModel,
        user_module_role::{Model as UserModuleRoleModel, Role},
    };
    use sea_orm::{DatabaseConnection, EntityTrait};
    use serde_json::Value;
    use tower::ServiceExt;

    struct TestData {
        lecturer_user: UserModel,
        tutor_user: UserModel,
        student_user1: UserModel,
        student_user2: UserModel,
        module: ModuleModel,
        assignment: AssignmentModel,
    }

    async fn setup_test_data(db: &DatabaseConnection) -> TestData {
        dotenvy::dotenv().ok();

        let module = ModuleModel::create(db, "CS101", Utc::now().year(), Some("Intro to CS"), 5)
            .await
            .expect("Failed to create test module");

        let lecturer_user =
            UserModel::create(db, "lecturer", "lecturer@test.com", "password", false)
                .await
                .expect("Failed to create lecturer user");
        let assistant_user =
            UserModel::create(db, "assistant", "assistant@test.com", "password", false)
                .await
                .expect("Failed to create assistant user");
        let tutor_user = UserModel::create(db, "tutor", "tutor@test.com", "password", false)
            .await
            .expect("Failed to create tutor user");
        let student_user1 =
            UserModel::create(db, "student1", "student1@test.com", "password", false)
                .await
                .expect("Failed to create student1 user");
        let student_user2 =
            UserModel::create(db, "student2", "student2@test.com", "password", false)
                .await
                .expect("Failed to create student2 user");
        let student_user3 =
            UserModel::create(db, "student3", "student3@test.com", "password", false)
                .await
                .expect("Failed to create student3 user");

        UserModuleRoleModel::assign_user_to_module(db, lecturer_user.id, module.id, Role::Lecturer)
            .await
            .unwrap();
        UserModuleRoleModel::assign_user_to_module(
            db,
            assistant_user.id,
            module.id,
            Role::AssistantLecturer,
        )
        .await
        .unwrap();
        UserModuleRoleModel::assign_user_to_module(db, tutor_user.id, module.id, Role::Tutor)
            .await
            .unwrap();
        UserModuleRoleModel::assign_user_to_module(db, student_user1.id, module.id, Role::Student)
            .await
            .unwrap();
        UserModuleRoleModel::assign_user_to_module(db, student_user2.id, module.id, Role::Student)
            .await
            .unwrap();
        UserModuleRoleModel::assign_user_to_module(db, student_user3.id, module.id, Role::Student)
            .await
            .unwrap();

        let assignment = AssignmentModel::create(
            db,
            module.id,
            "Assignment 1",
            Some("Desc 1"),
            AssignmentType::Assignment,
            Utc.with_ymd_and_hms(2024, 1, 1, 0, 0, 0).unwrap(),
            Utc.with_ymd_and_hms(2024, 1, 31, 23, 59, 59).unwrap(),
        )
        .await
        .unwrap();

        TestData {
            lecturer_user,
            tutor_user,
            student_user1,
            student_user2,
            module,
            assignment,
        }
    }

    fn make_hash_scan_request(
        user: &UserModel,
        module_id: i64,
        assignment_id: i64,
        payload: HashScanPayload,
    ) -> Request<AxumBody> {
        let (token, _) = generate_jwt(user.id, user.admin);
        let uri = format!(
            "/api/modules/{}/assignments/{}/plagiarism/hash-scan",
            module_id, assignment_id
        );
        let body = AxumBody::from(serde_json::to_string(&payload).unwrap());

        Request::builder()
            .method("POST")
            .uri(&uri)
            .header("Authorization", format!("Bearer {}", token))
            .header("Content-Type", "application/json")
            .body(body)
            .unwrap()
    }

    /// Test Case: Successful hash scan with no case creation
    #[tokio::test]
    async fn test_hash_scan_success_no_create_cases() {
        let (app, app_state, _tmp) = make_test_app_with_storage().await;
        let data = setup_test_data(app_state.db()).await;

        let mut cfg = util::execution_config::ExecutionConfig::default_config();
        cfg.marking.grading_policy = util::execution_config::GradingPolicy::Last;
        cfg.save(data.module.id, data.assignment.id).unwrap();

        let _collision_submission_user1 = SubmissionModel::save_file(
            app_state.db(),
            data.assignment.id,
            data.student_user1.id,
            3,
            10,
            10,
            false,
            "sub3_u1_collision.txt",
            "hash_abc",
            b"content_abc",
        )
        .await
        .unwrap();

        let _collision_submission_user2 = SubmissionModel::save_file(
            app_state.db(),
            data.assignment.id,
            data.student_user2.id,
            3,
            10,
            10,
            false,
            "sub3_u2_collision.txt",
            "hash_abc",
            b"content_abc",
        )
        .await
        .unwrap();

        let payload = HashScanPayload {
            create_cases: false,
        };

        let req = make_hash_scan_request(
            &data.lecturer_user,
            data.module.id,
            data.assignment.id,
            payload,
        );

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let json: Value = serde_json::from_slice(&body).unwrap();
        let res: HashScanResponse = serde_json::from_value(json["data"].clone()).unwrap();

        assert_eq!(json["success"], true);
        assert_eq!(json["message"], "Hash scan complete.");
        assert_eq!(res.assignment_id, data.assignment.id);
        assert_eq!(res.policy_used, "Last");
        assert_eq!(res.group_count, 1);
        assert_eq!(res.groups.len(), 1);
        assert_eq!(res.groups[0].file_hash, "hash_abc");
        assert_eq!(res.groups[0].submissions.len(), 2);

        let sub_ids: Vec<i64> = res.groups[0]
            .submissions
            .iter()
            .map(|s| s.submission_id)
            .collect();
        assert!(sub_ids.contains(&_collision_submission_user1.id));
        assert!(sub_ids.contains(&_collision_submission_user2.id));

        assert_eq!(res.cases.created.len(), 0);
        assert_eq!(res.cases.skipped_existing, 0);
    }

    /// Test Case: Successful hash scan with case creation
    #[tokio::test]
    async fn test_hash_scan_success_create_cases() {
        let (app, app_state, _tmp) = make_test_app_with_storage().await;
        let data = setup_test_data(app_state.db()).await;

        let mut cfg = util::execution_config::ExecutionConfig::default_config();
        cfg.marking.grading_policy = util::execution_config::GradingPolicy::Last;
        cfg.save(data.module.id, data.assignment.id).unwrap();

        let _collision_submission_user1 = SubmissionModel::save_file(
            app_state.db(),
            data.assignment.id,
            data.student_user1.id,
            3,
            10,
            10,
            false,
            "sub3_u1_collision.txt",
            "hash_abc",
            b"content_abc",
        )
        .await
        .unwrap();

        let _collision_submission_user2 = SubmissionModel::save_file(
            app_state.db(),
            data.assignment.id,
            data.student_user2.id,
            3,
            10,
            10,
            false,
            "sub3_u2_collision.txt",
            "hash_abc",
            b"content_abc",
        )
        .await
        .unwrap();

        let payload = HashScanPayload { create_cases: true };

        let req = make_hash_scan_request(
            &data.lecturer_user,
            data.module.id,
            data.assignment.id,
            payload,
        );

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let json: Value = serde_json::from_slice(&body).unwrap();
        let res: HashScanResponse = serde_json::from_value(json["data"].clone()).unwrap();

        assert_eq!(json["success"], true);
        assert_eq!(json["message"], "Hash scan complete.");
        assert_eq!(res.assignment_id, data.assignment.id);
        assert_eq!(res.policy_used, "Last");
        assert_eq!(res.group_count, 1);
        assert_eq!(res.groups.len(), 1);
        assert_eq!(res.groups[0].file_hash, "hash_abc");
        assert_eq!(res.groups[0].submissions.len(), 2);

        assert_eq!(res.cases.created.len(), 1);
        assert_eq!(res.cases.skipped_existing, 0);

        let created_case = &res.cases.created[0];
        assert_eq!(created_case.assignment_id, data.assignment.id);
        assert_eq!(created_case.status, "review");
        assert_eq!(created_case.similarity, 100.0);
        assert_eq!(created_case.lines_matched, 0);
        assert_eq!(created_case.description, "Identical file hash collision");
        assert!(created_case.report_id.is_none());

        let db_case = PlagiarismCaseEntity::find_by_id(created_case.id)
            .one(app_state.db())
            .await
            .unwrap()
            .expect("Created plagiarism case not found in DB");
        assert_eq!(db_case.status, Status::Review);
        assert_eq!(db_case.similarity, 100.0);
    }

    /// Test Case: Hash scan with existing cases
    #[tokio::test]
    async fn test_hash_scan_skipped_existing_cases() {
        let (app, app_state, _tmp) = make_test_app_with_storage().await;
        let data = setup_test_data(app_state.db()).await;

        let mut cfg = util::execution_config::ExecutionConfig::default_config();
        cfg.marking.grading_policy = util::execution_config::GradingPolicy::Last;
        cfg.save(data.module.id, data.assignment.id).unwrap();

        let collision_submission_user1 = SubmissionModel::save_file(
            app_state.db(),
            data.assignment.id,
            data.student_user1.id,
            3,
            10,
            10,
            false,
            "sub3_u1_collision.txt",
            "hash_abc",
            b"content_abc",
        )
        .await
        .unwrap();

        let collision_submission_user2 = SubmissionModel::save_file(
            app_state.db(),
            data.assignment.id,
            data.student_user2.id,
            3,
            10,
            10,
            false,
            "sub3_u2_collision.txt",
            "hash_abc",
            b"content_abc",
        )
        .await
        .unwrap();

        let (sub_id1, sub_id2) = (
            std::cmp::min(collision_submission_user1.id, collision_submission_user2.id),
            std::cmp::max(collision_submission_user1.id, collision_submission_user2.id),
        );
        PlagiarismCaseModel::create_case(
            app_state.db(),
            data.assignment.id,
            sub_id1,
            sub_id2,
            "Existing hash collision",
            99.0,
            5,
            None,
        )
        .await
        .unwrap();

        let payload = HashScanPayload { create_cases: true };

        let req = make_hash_scan_request(
            &data.lecturer_user,
            data.module.id,
            data.assignment.id,
            payload,
        );

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let json: Value = serde_json::from_slice(&body).unwrap();
        let res: HashScanResponse = serde_json::from_value(json["data"].clone()).unwrap();

        assert_eq!(res.cases.created.len(), 0);
        assert_eq!(res.cases.skipped_existing, 1);
    }

    /// Test Case: Forbidden access for Tutor
    #[tokio::test]
    async fn test_hash_scan_forbidden_as_tutor() {
        let (app, app_state, _tmp) = make_test_app_with_storage().await;
        let data = setup_test_data(app_state.db()).await;

        let payload = HashScanPayload {
            create_cases: false,
        };

        let req = make_hash_scan_request(
            &data.tutor_user,
            data.module.id,
            data.assignment.id,
            payload,
        );

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::FORBIDDEN);
    }

    /// Test Case: Assignment not found
    #[tokio::test]
    async fn test_hash_scan_assignment_not_found() {
        let (app, app_state, _tmp) = make_test_app_with_storage().await;
        let data = setup_test_data(app_state.db()).await;

        let payload = HashScanPayload {
            create_cases: false,
        };

        let req = make_hash_scan_request(&data.lecturer_user, data.module.id, 999999, payload);

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::NOT_FOUND);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let json: Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(json["success"], false);
        assert_eq!(json["message"], "Assignment 999999 in Module 1 not found.");
    }

    /// Test Case: No collisions found
    #[tokio::test]
    async fn test_hash_scan_no_collisions() {
        let (app, app_state, _tmp) = make_test_app_with_storage().await;
        let data = setup_test_data(app_state.db()).await;

        let assignment = AssignmentModel::create(
            app_state.db(),
            data.module.id,
            "Assignment No Collisions",
            None,
            AssignmentType::Assignment,
            Utc.with_ymd_and_hms(2024, 1, 1, 0, 0, 0).unwrap(),
            Utc.with_ymd_and_hms(2024, 1, 31, 23, 59, 59).unwrap(),
        )
        .await
        .unwrap();

        SubmissionModel::save_file(
            app_state.db(),
            assignment.id,
            data.student_user1.id,
            1,
            10,
            10,
            false,
            "sub_u1.txt",
            "unique_hash_1",
            b"content_1",
        )
        .await
        .unwrap();
        SubmissionModel::save_file(
            app_state.db(),
            assignment.id,
            data.student_user2.id,
            1,
            10,
            10,
            false,
            "sub_u2.txt",
            "unique_hash_2",
            b"content_2",
        )
        .await
        .unwrap();

        let payload = HashScanPayload {
            create_cases: false,
        };

        let req =
            make_hash_scan_request(&data.lecturer_user, data.module.id, assignment.id, payload);

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let json: Value = serde_json::from_slice(&body).unwrap();
        let res: HashScanResponse = serde_json::from_value(json["data"].clone()).unwrap();

        assert_eq!(json["success"], true);
        assert_eq!(json["message"], "Hash scan complete.");
        assert_eq!(res.group_count, 0);
        assert!(res.groups.is_empty());
        assert_eq!(res.cases.created.len(), 0);
        assert_eq!(res.cases.skipped_existing, 0);
    }

    /// Test Case: Policy used is 'best'
    #[tokio::test]
    async fn test_hash_scan_policy_best() {
        let (app, app_state, _tmp) = make_test_app_with_storage().await;

        dotenvy::dotenv().ok();

        let module = ModuleModel::create(
            app_state.db(),
            "CS102_Best",
            Utc::now().year(),
            Some("Best Policy Test"),
            5,
        )
        .await
        .expect("Failed to create test module");

        let lecturer_user = UserModel::create(
            app_state.db(),
            "lecturer_best",
            "lecturer_best@test.com",
            "password",
            false,
        )
        .await
        .expect("Failed to create lecturer user");
        let student_user1 = UserModel::create(
            app_state.db(),
            "student1_best",
            "student1_best@test.com",
            "password",
            false,
        )
        .await
        .expect("Failed to create student1 user");

        UserModuleRoleModel::assign_user_to_module(
            app_state.db(),
            lecturer_user.id,
            module.id,
            Role::Lecturer,
        )
        .await
        .unwrap();
        UserModuleRoleModel::assign_user_to_module(
            app_state.db(),
            student_user1.id,
            module.id,
            Role::Student,
        )
        .await
        .unwrap();

        let assignment = AssignmentModel::create(
            app_state.db(),
            module.id,
            "Assignment Best",
            Some("Best policy test assignment"),
            AssignmentType::Assignment,
            Utc.with_ymd_and_hms(2024, 1, 1, 0, 0, 0).unwrap(),
            Utc.with_ymd_and_hms(2024, 1, 31, 23, 59, 59).unwrap(),
        )
        .await
        .unwrap();

        let mut cfg = util::execution_config::ExecutionConfig::default_config();
        cfg.marking.grading_policy = util::execution_config::GradingPolicy::Best;
        cfg.save(module.id, assignment.id).unwrap();

        SubmissionModel::save_file(
            app_state.db(),
            assignment.id,
            student_user1.id,
            3,
            10,
            100,
            false,
            "sub3_u1_best.txt",
            "hash_best_u1",
            b"content_best_u1",
        )
        .await
        .unwrap();

        let payload = HashScanPayload {
            create_cases: false,
        };

        let req = make_hash_scan_request(&lecturer_user, module.id, assignment.id, payload);

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let json: Value = serde_json::from_slice(&body).unwrap();
        let res: HashScanResponse = serde_json::from_value(json["data"].clone()).unwrap();

        assert_eq!(json["success"], true);
        assert_eq!(res.policy_used, "Best");
        assert_eq!(res.group_count, 0);
        assert!(res.groups.is_empty());
    }

    /// Test Case: Policy used is 'last'
    #[tokio::test]
    async fn test_hash_scan_policy_last() {
        let (app, app_state, _tmp) = make_test_app_with_storage().await;
        let data = setup_test_data(app_state.db()).await;

        let mut cfg = util::execution_config::ExecutionConfig::default_config();
        cfg.marking.grading_policy = util::execution_config::GradingPolicy::Last;
        cfg.save(data.module.id, data.assignment.id).unwrap();

        let _late_submission_user1 = SubmissionModel::save_file(
            app_state.db(),
            data.assignment.id,
            data.student_user1.id,
            3,
            10,
            10,
            false,
            "sub3_u1_late.txt",
            "hash_late_u1",
            b"content_late_u1",
        )
        .await
        .unwrap();

        let late_collision_submission_user2 = SubmissionModel::save_file(
            app_state.db(),
            data.assignment.id,
            data.student_user2.id,
            3,
            10,
            10,
            false,
            "sub3_u2_late.txt",
            "hash_late_u1",
            b"content_late_u1",
        )
        .await
        .unwrap();

        let payload = HashScanPayload {
            create_cases: false,
        };

        let req = make_hash_scan_request(
            &data.lecturer_user,
            data.module.id,
            data.assignment.id,
            payload,
        );

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let json: Value = serde_json::from_slice(&body).unwrap();
        let res: HashScanResponse = serde_json::from_value(json["data"].clone()).unwrap();

        assert_eq!(json["success"], true);
        assert_eq!(res.policy_used, "Last");
        assert_eq!(res.group_count, 1);
        assert_eq!(res.groups.len(), 1);
        assert_eq!(res.groups[0].file_hash, "hash_late_u1");
        assert_eq!(res.groups[0].submissions.len(), 2);

        let sub_ids: Vec<i64> = res.groups[0]
            .submissions
            .iter()
            .map(|s| s.submission_id)
            .collect();
        assert!(sub_ids.contains(&_late_submission_user1.id));
        assert!(sub_ids.contains(&late_collision_submission_user2.id));
    }
}
