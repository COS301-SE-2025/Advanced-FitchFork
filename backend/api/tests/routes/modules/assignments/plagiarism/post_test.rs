#[cfg(test)]
mod create_plagiarism_tests {
    use db::{
        models::{
            assignment::{Model as AssignmentModel, AssignmentType},
            assignment_submission::Model as SubmissionModel,
            module::Model as ModuleModel,
            plagiarism_case::{Status, Entity as PlagiarismCaseEntity},
            user::Model as UserModel,
            user_module_role::{Model as UserModuleRoleModel, Role},
        },
        repositories::user_repository::UserRepository,
    };
    use axum::{
        body::Body as AxumBody,
        http::{Request, StatusCode},
    };
    use services::{
        service::Service,
        user_service::{CreateUser, UserService},
    };
    use sea_orm::{EntityTrait, DatabaseConnection};
    use tower::ServiceExt;
    use serde_json::{Value, json};
    use api::auth::generate_jwt;
    use crate::helpers::app::make_test_app;
    use chrono::{Datelike, TimeZone, Utc};
    use api::routes::modules::assignments::plagiarism::post::CreatePlagiarismCasePayload;

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

        let module = ModuleModel::create(db, "CS101", Utc::now().year(), Some("Intro to CS"), 5).await.expect("Failed to create test module");
        let service = UserService::new(UserRepository::new(db.clone()));
        let lecturer_user = service.create(CreateUser { username: "lecturer".into(), email: "lecturer@test.com".into(), password: "password".into(), admin: false }).await.expect("Failed to create lecturer user");
        let assistant_user = service.create(CreateUser { username: "assistant".into(), email: "assistant@test.com".into(), password: "password".into(), admin: false }).await.expect("Failed to create assistant user");
        let tutor_user = service.create(CreateUser { username: "tutor".into(), email: "tutor@test.com".into(), password: "password".into(), admin: false }).await.expect("Failed to create tutor user");
        let student_user1 = service.create(CreateUser { username: "student1".into(), email: "student1@test.com".into(), password: "password".into(), admin: false }).await.expect("Failed to create student1 user");
        let student_user2 = service.create(CreateUser { username: "student2".into(), email: "student2@test.com".into(), password: "password".into(), admin: false }).await.expect("Failed to create student2 user");
        UserModuleRoleModel::assign_user_to_module(db, lecturer_user.id, module.id, Role::Lecturer).await.expect("Failed to assign lecturer role");
        UserModuleRoleModel::assign_user_to_module(db, assistant_user.id, module.id, Role::AssistantLecturer).await.expect("Failed to assign assistant lecturer role");
        UserModuleRoleModel::assign_user_to_module(db, tutor_user.id, module.id, Role::Tutor).await.expect("Failed to assign tutor role");
        UserModuleRoleModel::assign_user_to_module(db, student_user1.id, module.id, Role::Student).await.expect("Failed to assign student role");
        UserModuleRoleModel::assign_user_to_module(db, student_user2.id, module.id, Role::Student).await.expect("Failed to assign student role");
        let assignment = AssignmentModel::create(
            db, 
            module.id, 
            "Assignment 1", 
            Some("Desc 1"), 
            AssignmentType::Assignment, 
            Utc.with_ymd_and_hms(2024, 1, 1, 0, 0, 0).unwrap(), 
            Utc.with_ymd_and_hms(2024, 1, 31, 23, 59, 59).unwrap()
        ).await.unwrap();
        let submission1 = SubmissionModel::save_file(
            db, 
            assignment.id, 
            student_user1.id, 
            1, 
            false, 
            "sub1.txt", 
            "hash123#", 
            b"ontime"
        ).await.unwrap();
        let submission2 = SubmissionModel::save_file(
            db, 
            assignment.id, 
            student_user2.id, 
            1, 
            false, 
            "sub2.txt", 
            "hash123#", 
            b"ontime"
        ).await.unwrap();

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
        let uri = format!("/api/modules/{}/assignments/{}/plagiarism", module_id, assignment_id);
        let body = AxumBody::from(serde_json::to_string(&payload).unwrap());
        
        Request::builder()
            .method("POST")
            .uri(&uri)
            .header("Authorization", format!("Bearer {}", token))
            .header("Content-Type", "application/json")
            .body(body)
            .unwrap()
    }

    /// Test Case: Successful Creation by Lecturer
    #[tokio::test]
    async fn test_create_plagiarism_case_success_as_lecturer() {
        let (app, app_state) = make_test_app().await;
        let data = setup_test_data(app_state.db()).await;

        let payload = CreatePlagiarismCasePayload {
            submission_id_1: data.submission1.id,
            submission_id_2: data.submission2.id,
            description: "Code similarity detected".to_string(),
        };

        let req = make_post_request(
            &data.lecturer_user,
            data.module.id,
            data.assignment.id,
            payload,
        );
        
        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::CREATED);
        
        let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let json: Value = serde_json::from_slice(&body).unwrap();
        
        assert_eq!(json["success"], true);
        assert_eq!(json["message"], "Plagiarism case created successfully");
        
        let case_data = &json["data"];
        assert!(case_data["id"].is_i64());
        assert_eq!(case_data["assignment_id"], data.assignment.id);
        assert_eq!(case_data["submission_id_1"], data.submission1.id);
        assert_eq!(case_data["submission_id_2"], data.submission2.id);
        assert_eq!(case_data["description"], "Code similarity detected");
        assert_eq!(case_data["status"], "Review");
        assert!(case_data["created_at"].is_string());
        assert!(case_data["updated_at"].is_string());
        
        // Verify case exists in database
        let case = PlagiarismCaseEntity::find_by_id(case_data["id"].as_i64().unwrap())
            .one(app_state.db())
            .await
            .unwrap()
            .expect("Plagiarism case should exist");
        assert_eq!(case.status, Status::Review);
    }

    /// Test Case: Successful Creation by Assistant Lecturer
    #[tokio::test]
    async fn test_create_plagiarism_case_success_as_assistant() {
        let (app, app_state) = make_test_app().await;
        let data = setup_test_data(app_state.db()).await;

        let payload = CreatePlagiarismCasePayload {
            submission_id_1: data.submission1.id,
            submission_id_2: data.submission2.id,
            description: "Similar solution structure".to_string(),
        };

        let req = make_post_request(
            &data.assistant_user,
            data.module.id,
            data.assignment.id,
            payload,
        );
        
        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::CREATED);
    }

    /// Test Case: Forbidden Access for Tutor
    #[tokio::test]
    async fn test_create_plagiarism_case_forbidden_as_tutor() {
        let (app, app_state) = make_test_app().await;
        let data = setup_test_data(app_state.db()).await;

        let payload = CreatePlagiarismCasePayload {
            submission_id_1: data.submission1.id,
            submission_id_2: data.submission2.id,
            description: "Tutor should not access".to_string(),
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
    async fn test_create_plagiarism_case_same_submission_ids() {
        let (app, app_state) = make_test_app().await;
        let data = setup_test_data(app_state.db()).await;

        let payload = CreatePlagiarismCasePayload {
            submission_id_1: data.submission1.id,
            submission_id_2: data.submission1.id, // Same submission
            description: "Invalid same submission".to_string(),
        };

        let req = make_post_request(
            &data.lecturer_user,
            data.module.id,
            data.assignment.id,
            payload,
        );
        
        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
        
        let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let json: Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(json["success"], false);
        assert_eq!(json["message"], "Submissions cannot be the same");
    }

    /// Test Case: Submission Not Found Validation
    #[tokio::test]
    async fn test_create_plagiarism_case_submission_not_found() {
        let (app, app_state) = make_test_app().await;
        let data = setup_test_data(app_state.db()).await;

        let payload = CreatePlagiarismCasePayload {
            submission_id_1: data.submission1.id,
            submission_id_2: 999999, // Non-existent submission
            description: "Invalid submission".to_string(),
        };

        let req = make_post_request(
            &data.lecturer_user,
            data.module.id,
            data.assignment.id,
            payload,
        );
        
        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
        
        let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let json: Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(json["success"], false);
        assert_eq!(
            json["message"], 
            "One or both submissions do not exist or belong to a different assignment"
        );
    }

    /// Test Case: Submission from Different Assignment
    #[tokio::test]
    async fn test_create_plagiarism_case_wrong_assignment() {
        let (app, app_state) = make_test_app().await;
        let data = setup_test_data(app_state.db()).await;

        // Create another assignment and submission
        let other_assignment = AssignmentModel::create(
            app_state.db(),
            data.module.id,
            "Other Assignment",
            None,
            AssignmentType::Assignment,
            Utc.with_ymd_and_hms(2024, 2, 1, 0, 0, 0).unwrap(),
            Utc.with_ymd_and_hms(2024, 2, 28, 23, 59, 59).unwrap(),
        ).await.unwrap();
        
        let other_submission = SubmissionModel::save_file(
            app_state.db(),
            other_assignment.id,
            data.student_user1.id,
            1,
            false,
            "other.txt",
            "hash456#",
            b"ontime",
        ).await.unwrap();

        let payload = CreatePlagiarismCasePayload {
            submission_id_1: data.submission1.id,
            submission_id_2: other_submission.id, // From different assignment
            description: "Cross-assignment submission".to_string(),
        };

        let req = make_post_request(
            &data.lecturer_user,
            data.module.id,
            data.assignment.id, // Current assignment
            payload,
        );
        
        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
        
        let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let json: Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(json["success"], false);
        assert_eq!(
            json["message"], 
            "One or both submissions do not exist or belong to a different assignment"
        );
    }

    /// Test Case: Missing Authorization Header
    #[tokio::test]
    async fn test_create_plagiarism_case_unauthorized() {
        let (app, app_state) = make_test_app().await;
        let data = setup_test_data(app_state.db()).await;

        let payload = CreatePlagiarismCasePayload {
            submission_id_1: data.submission1.id,
            submission_id_2: data.submission2.id,
            description: "Unauthorized attempt".to_string(),
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

    /// Test Case: Invalid Payload Format
    #[tokio::test]
    async fn test_create_plagiarism_case_invalid_payload() {
        let (app, app_state) = make_test_app().await;
        let data = setup_test_data(app_state.db()).await;

        let invalid_payload = json!({
            "submission_id_1": "not_a_number",
            "description": "Missing fields"
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
}