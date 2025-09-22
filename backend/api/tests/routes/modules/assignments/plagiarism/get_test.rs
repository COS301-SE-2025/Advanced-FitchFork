#[cfg(test)]
mod plagiarism_tests {
    use crate::helpers::app::make_test_app_with_storage;
    use api::auth::generate_jwt;
    use axum::{
        body::Body as AxumBody,
        http::{Request, StatusCode},
    };
    use chrono::{Datelike, TimeZone, Utc};
    use db::models::{
        assignment::{AssignmentType, Model as AssignmentModel},
        assignment_submission::Model as SubmissionModel,
        module::Model as ModuleModel,
        plagiarism_case::{Model as PlagiarismCaseModel, Status},
        user::Model as UserModel,
        user_module_role::{Model as UserModuleRoleModel, Role},
    };
    use sea_orm::{ActiveModelTrait, DatabaseConnection, IntoActiveModel, Set};
    use serde_json::Value;
    use tower::ServiceExt;

    struct TestData {
        admin_user: UserModel,
        lecturer_user: UserModel,
        assistant_user: UserModel,
        tutor_user: UserModel,
        student_user1: UserModel,
        student_user2: UserModel,
        module: ModuleModel,
        assignment: AssignmentModel,
        submission1: SubmissionModel,
        submission2: SubmissionModel,
        plagiarism_case: PlagiarismCaseModel,
    }

    async fn setup_test_data(db: &DatabaseConnection) -> TestData {
        dotenvy::dotenv().ok();

        let module = ModuleModel::create(db, "CS101", Utc::now().year(), Some("Intro to CS"), 5)
            .await
            .expect("Failed to create test module");
        let admin_user = UserModel::create(db, "admin", "admin@test.com", "password", true)
            .await
            .expect("Failed to create admin user");
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
            .expect("Failed to assign lecturer role");
        UserModuleRoleModel::assign_user_to_module(
            db,
            assistant_user.id,
            module.id,
            Role::AssistantLecturer,
        )
        .await
        .expect("Failed to assign assistant lecturer role");
        UserModuleRoleModel::assign_user_to_module(db, tutor_user.id, module.id, Role::Tutor)
            .await
            .expect("Failed to assign tutor role");
        UserModuleRoleModel::assign_user_to_module(db, student_user1.id, module.id, Role::Student)
            .await
            .expect("Failed to assign student role");
        UserModuleRoleModel::assign_user_to_module(db, student_user2.id, module.id, Role::Student)
            .await
            .expect("Failed to assign student role");
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
        let plagiarism_case = PlagiarismCaseModel::create_case(
            db,
            assignment.id,
            submission1.id,
            submission2.id,
            "High similarity detected",
            0.0,  // similarity
            0,    // lines_matched
            None, // report_id
        )
        .await
        .unwrap();

        TestData {
            admin_user,
            lecturer_user,
            assistant_user,
            tutor_user,
            student_user1,
            student_user2,
            module,
            assignment,
            submission1,
            submission2,
            plagiarism_case,
        }
    }

    async fn create_additional_plagiarism_cases(
        db: &DatabaseConnection,
        assignment_id: i64,
        user1: i64,
        user2: i64,
    ) -> Vec<PlagiarismCaseModel> {
        let mut cases = Vec::new();

        let sub3 = SubmissionModel::save_file(
            db,
            assignment_id,
            user1,
            1,
            10,
            10,
            false,
            "sub3.txt",
            "hash123#",
            b"ontime",
        )
        .await
        .unwrap();
        let sub4 = SubmissionModel::save_file(
            db,
            assignment_id,
            user2,
            1,
            10,
            10,
            false,
            "sub4.txt",
            "hash123#",
            b"ontime",
        )
        .await
        .unwrap();

        let mut case1 = PlagiarismCaseModel::create_case(
            db,
            assignment_id,
            sub3.id,
            sub4.id,
            "Resolved case",
            0.0,  // similarity
            0,    // lines_matched
            None, // report_id
        )
        .await
        .unwrap();

        let mut active_case1 = case1.into_active_model();
        active_case1.status = Set(Status::Flagged);
        active_case1.updated_at = Set(Utc::now());

        case1 = active_case1.update(db).await.unwrap();
        cases.push(case1);

        let mut case2 = PlagiarismCaseModel::create_case(
            db,
            assignment_id,
            sub3.id,
            sub4.id,
            "Pending case",
            0.0,   // similarity
            0_i64, // lines_matched
            None,  // report_id
        )
        .await
        .unwrap();

        let mut active_case2 = case2.into_active_model();
        active_case2.status = Set(Status::Reviewed);
        active_case2.updated_at = Set(Utc::now());

        case2 = active_case2.update(db).await.unwrap();
        cases.push(case2);

        cases
    }

    fn make_request(
        user: &UserModel,
        module_id: i64,
        assignment_id: i64,
        query_params: Option<Vec<(&str, &str)>>,
    ) -> Request<AxumBody> {
        let (token, _) = generate_jwt(user.id, user.admin);
        let mut uri = format!(
            "/api/modules/{}/assignments/{}/plagiarism",
            module_id, assignment_id
        );

        if let Some(params) = query_params {
            let query_string = params
                .iter()
                .map(|(k, v)| format!("{}={}", k, v))
                .collect::<Vec<_>>()
                .join("&");
            uri = format!("{}?{}", uri, query_string);
        }

        Request::builder()
            .method("GET")
            .uri(&uri)
            .header("Authorization", format!("Bearer {}", token))
            .body(AxumBody::empty())
            .unwrap()
    }

    /// Test Case: Successful Retrieval of Plagiarism Cases as Admin
    #[tokio::test]
    #[serial]
    async fn test_list_plagiarism_cases_success_as_admin() {
        let (app, app_state, _tmp) = make_test_app_with_storage().await;
        let data = setup_test_data(app_state.db()).await;

        let req = make_request(&data.admin_user, data.module.id, data.assignment.id, None);
        let response = app.oneshot(req).await.unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let json: Value = serde_json::from_slice(&body).unwrap();

        assert_eq!(json["success"], true);
        assert_eq!(json["message"], "Plagiarism cases retrieved successfully");

        let cases = &json["data"]["cases"];
        assert!(cases.is_array());
        assert_eq!(cases.as_array().unwrap().len(), 1);

        let case_data = &cases[0];
        assert_eq!(case_data["id"], data.plagiarism_case.id);
        assert_eq!(case_data["status"], "review");
        assert_eq!(case_data["description"], "High similarity detected");

        let sub1 = &case_data["submission_1"];
        assert_eq!(sub1["id"], data.submission1.id);
        assert_eq!(sub1["filename"], "sub1.txt");
        assert_eq!(sub1["user"]["username"], "student1");

        let sub2 = &case_data["submission_2"];
        assert_eq!(sub2["id"], data.submission2.id);
        assert_eq!(sub2["filename"], "sub2.txt");
        assert_eq!(sub2["user"]["username"], "student2");
    }

    /// Test Case: Successful Retrieval as Lecturer
    #[tokio::test]
    #[serial]
    async fn test_list_plagiarism_cases_success_as_lecturer() {
        let (app, app_state, _tmp) = make_test_app_with_storage().await;
        let data = setup_test_data(app_state.db()).await;

        let req = make_request(
            &data.lecturer_user,
            data.module.id,
            data.assignment.id,
            None,
        );
        let response = app.oneshot(req).await.unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let json: Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(json["data"]["cases"].as_array().unwrap().len(), 1);
    }

    /// Test Case: Successful Retrieval as Assistant Lecturer
    #[tokio::test]
    #[serial]
    async fn test_list_plagiarism_cases_success_as_assistant_lecturer() {
        let (app, app_state, _tmp) = make_test_app_with_storage().await;
        let data = setup_test_data(app_state.db()).await;

        let req = make_request(
            &data.assistant_user,
            data.module.id,
            data.assignment.id,
            None,
        );
        let response = app.oneshot(req).await.unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let json: Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(json["data"]["cases"].as_array().unwrap().len(), 1);
    }

    /// Test Case: Forbidden Access for Unauthorized Tutor
    #[tokio::test]
    #[serial]
    async fn test_list_plagiarism_cases_forbidden_for_tutor() {
        let (app, app_state, _tmp) = make_test_app_with_storage().await;
        let data = setup_test_data(app_state.db()).await;

        let req = make_request(&data.tutor_user, data.module.id, data.assignment.id, None);
        let response = app.oneshot(req).await.unwrap();

        assert_eq!(response.status(), StatusCode::FORBIDDEN);
    }

    /// Test Case: Forbidden Access for Unauthorized Student
    #[tokio::test]
    #[serial]
    async fn test_list_plagiarism_cases_forbidden_for_student() {
        let (app, app_state, _tmp) = make_test_app_with_storage().await;
        let data = setup_test_data(app_state.db()).await;

        let req = make_request(
            &data.student_user1,
            data.module.id,
            data.assignment.id,
            None,
        );
        let response = app.oneshot(req).await.unwrap();

        assert_eq!(response.status(), StatusCode::FORBIDDEN);
    }

    /// Test Case: Empty List for Assignment Without Plagiarism Cases
    #[tokio::test]
    #[serial]
    async fn test_list_plagiarism_cases_empty() {
        let (app, app_state, _tmp) = make_test_app_with_storage().await;
        let data = setup_test_data(app_state.db()).await;

        let new_assignment = AssignmentModel::create(
            db::get_connection().await,
            data.module.id,
            "Empty Assignment",
            Some("Empty Description"),
            AssignmentType::Assignment,
            Utc.with_ymd_and_hms(2024, 1, 1, 0, 0, 0).unwrap(),
            Utc.with_ymd_and_hms(2024, 1, 31, 23, 59, 59).unwrap(),
        )
        .await
        .unwrap();

        let req = make_request(&data.admin_user, data.module.id, new_assignment.id, None);
        let response = app.oneshot(req).await.unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let json: Value = serde_json::from_slice(&body).unwrap();

        assert_eq!(json["success"], true);
        assert_eq!(json["data"]["cases"].as_array().unwrap().len(), 0);
        assert_eq!(json["data"]["total"], 0);
    }

    /// Test Case: Filtering by `review` Status
    #[tokio::test]
    #[serial]
    async fn test_list_plagiarism_cases_filter_by_review_status() {
        let (app, app_state, _tmp) = make_test_app_with_storage().await;
        let data = setup_test_data(app_state.db()).await;

        let _ = create_additional_plagiarism_cases(
            db::get_connection().await,
            data.assignment.id,
            data.student_user1.id,
            data.student_user2.id,
        )
        .await;

        let req = make_request(
            &data.admin_user,
            data.module.id,
            data.assignment.id,
            Some(vec![("status", "review")]),
        );
        let response = app.oneshot(req).await.unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let json: Value = serde_json::from_slice(&body).unwrap();

        let cases = json["data"]["cases"].as_array().unwrap();
        assert_eq!(cases.len(), 1);
        assert_eq!(cases[0]["status"], "review");
    }

    /// Test Case: Filtering by `Flagged` Status
    #[tokio::test]
    #[serial]
    async fn test_list_plagiarism_cases_filter_by_flagged_status() {
        let (app, app_state, _tmp) = make_test_app_with_storage().await;
        let data = setup_test_data(app_state.db()).await;

        let _ = create_additional_plagiarism_cases(
            db::get_connection().await,
            data.assignment.id,
            data.student_user1.id,
            data.student_user2.id,
        )
        .await;

        let req = make_request(
            &data.admin_user,
            data.module.id,
            data.assignment.id,
            Some(vec![("status", "Flagged")]),
        );
        let response = app.oneshot(req).await.unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let json: Value = serde_json::from_slice(&body).unwrap();

        let cases = json["data"]["cases"].as_array().unwrap();
        assert_eq!(cases.len(), 1);
        assert_eq!(cases[0]["status"], "flagged");
    }

    /// Test Case: Filtering by `reviewed` Status
    #[tokio::test]
    #[serial]
    async fn test_list_plagiarism_cases_filter_by_reviewed_status() {
        let (app, app_state, _tmp) = make_test_app_with_storage().await;
        let data = setup_test_data(app_state.db()).await;

        let _ = create_additional_plagiarism_cases(
            db::get_connection().await,
            data.assignment.id,
            data.student_user1.id,
            data.student_user2.id,
        )
        .await;

        let req = make_request(
            &data.admin_user,
            data.module.id,
            data.assignment.id,
            Some(vec![("status", "reviewed")]),
        );
        let response = app.oneshot(req).await.unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let json: Value = serde_json::from_slice(&body).unwrap();

        let cases = json["data"]["cases"].as_array().unwrap();
        assert_eq!(cases.len(), 1);
        assert_eq!(cases[0]["status"], "reviewed");
    }

    /// Test Case: Search by Username
    #[tokio::test]
    #[serial]
    async fn test_list_plagiarism_cases_search_by_username() {
        let (app, app_state, _tmp) = make_test_app_with_storage().await;
        let data = setup_test_data(app_state.db()).await;

        let req = make_request(
            &data.admin_user,
            data.module.id,
            data.assignment.id,
            Some(vec![("query", "student1")]),
        );
        let response = app.oneshot(req).await.unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let json: Value = serde_json::from_slice(&body).unwrap();

        let cases = json["data"]["cases"].as_array().unwrap();
        assert!(cases.len() >= 1);
        let case = &cases[0];
        assert!(
            case["submission_1"]["user"]["username"] == "student1"
                || case["submission_2"]["user"]["username"] == "student1"
        );
    }

    /// Test Case: Sorting by Created At
    #[tokio::test]
    #[serial]
    async fn test_list_plagiarism_cases_sorting() {
        let (app, app_state, _tmp) = make_test_app_with_storage().await;
        let data = setup_test_data(app_state.db()).await;

        let _ = create_additional_plagiarism_cases(
            db::get_connection().await,
            data.assignment.id,
            data.student_user1.id,
            data.student_user2.id,
        )
        .await;

        let req = make_request(
            &data.admin_user,
            data.module.id,
            data.assignment.id,
            Some(vec![("sort", "created_at")]),
        );
        let response = app.clone().oneshot(req).await.unwrap();
        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let json: Value = serde_json::from_slice(&body).unwrap();
        let cases = json["data"]["cases"].as_array().unwrap();
        let first_created = cases[0]["created_at"].as_str().unwrap();
        let last_created = cases[cases.len() - 1]["created_at"].as_str().unwrap();
        assert!(first_created < last_created);

        let req = make_request(
            &data.admin_user,
            data.module.id,
            data.assignment.id,
            Some(vec![("sort", "-created_at")]),
        );
        let response = app.oneshot(req).await.unwrap();
        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let json: Value = serde_json::from_slice(&body).unwrap();
        let cases = json["data"]["cases"].as_array().unwrap();
        let first_created = cases[0]["created_at"].as_str().unwrap();
        let last_created = cases[cases.len() - 1]["created_at"].as_str().unwrap();
        assert!(first_created > last_created);
    }

    /// Test Case: Pagination Works Correctly
    #[tokio::test]
    #[serial]
    async fn test_list_plagiarism_cases_pagination() {
        let (app, app_state, _tmp) = make_test_app_with_storage().await;
        let data = setup_test_data(app_state.db()).await;

        for _ in 0..15 {
            let sub = SubmissionModel::save_file(
                db::get_connection().await,
                data.assignment.id,
                data.student_user1.id,
                1,
                10,
                10,
                false,
                "sub.txt",
                "hash123#",
                b"ontime",
            )
            .await
            .unwrap();

            PlagiarismCaseModel::create_case(
                db::get_connection().await,
                data.assignment.id,
                data.submission1.id,
                sub.id,
                "Test case description",
                0.0,  // similarity
                0,    // lines_matched
                None, // report_id
            )
            .await
            .unwrap();
        }

        let req = make_request(
            &data.admin_user,
            data.module.id,
            data.assignment.id,
            Some(vec![("per_page", "10"), ("page", "1")]),
        );
        let response = app.clone().oneshot(req).await.unwrap();
        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let json: Value = serde_json::from_slice(&body).unwrap();
        let cases = json["data"]["cases"].as_array().unwrap();
        assert_eq!(cases.len(), 10);
        assert_eq!(json["data"]["page"], 1);
        assert_eq!(json["data"]["per_page"], 10);
        assert_eq!(json["data"]["total"], 16);

        let req = make_request(
            &data.admin_user,
            data.module.id,
            data.assignment.id,
            Some(vec![("per_page", "10"), ("page", "2")]),
        );
        let response = app.oneshot(req).await.unwrap();
        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let json: Value = serde_json::from_slice(&body).unwrap();
        let cases = json["data"]["cases"].as_array().unwrap();
        assert_eq!(cases.len(), 6);
        assert_eq!(json["data"]["page"], 2);
    }

    /// Test Case: Missing Authorization Header
    #[tokio::test]
    #[serial]
    async fn test_list_plagiarism_cases_unauthorized_missing_header() {
        let (app, app_state, _tmp) = make_test_app_with_storage().await;
        let data = setup_test_data(app_state.db()).await;

        let uri = format!(
            "/api/modules/{}/assignments/{}/plagiarism",
            data.module.id, data.assignment.id
        );
        let req = Request::builder()
            .method("GET")
            .uri(&uri)
            .body(AxumBody::empty())
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    /// Test Case: Invalid JWT Token
    #[tokio::test]
    #[serial]
    async fn test_list_plagiarism_cases_unauthorized_invalid_token() {
        let (app, app_state, _tmp) = make_test_app_with_storage().await;
        let data = setup_test_data(app_state.db()).await;

        let uri = format!(
            "/api/modules/{}/assignments/{}/plagiarism",
            data.module.id, data.assignment.id
        );
        let req = Request::builder()
            .method("GET")
            .uri(&uri)
            .header("Authorization", "Bearer invalid.token.here")
            .body(AxumBody::empty())
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    /// Test Case: Accessing Non-Existent Assignment
    #[tokio::test]
    #[serial]
    async fn test_list_plagiarism_cases_non_existent_assignment() {
        let (app, app_state, _tmp) = make_test_app_with_storage().await;
        let data = setup_test_data(app_state.db()).await;

        let req = make_request(&data.admin_user, data.module.id, 999999, None);
        let response = app.oneshot(req).await.unwrap();

        assert_eq!(response.status(), StatusCode::NOT_FOUND);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let json: Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(json["success"], false);
        assert_eq!(json["message"], "Assignment 999999 in Module 1 not found.");
    }

    /// Test Case: Invalid Status Filter
    #[tokio::test]
    #[serial]
    async fn test_list_plagiarism_cases_invalid_status() {
        let (app, app_state, _tmp) = make_test_app_with_storage().await;
        let data = setup_test_data(app_state.db()).await;

        let req = make_request(
            &data.admin_user,
            data.module.id,
            data.assignment.id,
            Some(vec![("status", "InvalidStatus")]),
        );
        let response = app.oneshot(req).await.unwrap();

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }
}
