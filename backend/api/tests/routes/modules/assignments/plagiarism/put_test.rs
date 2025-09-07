#[cfg(test)]
mod update_plagiarism_tests {
    use db::{
        models::{
            assignment::{Model as AssignmentModel, AssignmentType},
            assignment_submission::Model as SubmissionModel,
            module::Model as ModuleModel,
            plagiarism_case::{Status, Entity as PlagiarismCaseEntity, Model as PlagiarismCaseModel},
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
    use serde_json::Value;
    use api::auth::generate_jwt;
    use crate::helpers::app::make_test_app;
    use chrono::{Datelike, TimeZone, Utc};
    use api::routes::modules::assignments::plagiarism::put::UpdatePlagiarismCasePayload;

    // small helper for float compares in JSON
    fn approx_eq_f64(a: f64, b: f64, eps: f64) -> bool { (a - b).abs() <= eps }

    struct TestData {
        lecturer_user: UserModel,
        assistant_user: UserModel,
        tutor_user: UserModel,
        student_user: UserModel,
        module: ModuleModel,
        assignment: AssignmentModel,
        plagiarism_case: PlagiarismCaseModel,
    }

    async fn setup_test_data(db: &DatabaseConnection) -> TestData {
        dotenvy::dotenv().ok();

        let module = ModuleModel::create(db, "CS101", Utc::now().year(), Some("Intro to CS"), 5)
            .await
            .expect("Failed to create test module");
        
        let lecturer_user = UserModel::create(db, "lecturer", "lecturer@test.com", "password", false)
            .await
            .expect("Failed to create lecturer user");
        let assistant_user = UserModel::create(db, "assistant", "assistant@test.com", "password", false)
            .await
            .expect("Failed to create assistant user");
        let tutor_user = UserModel::create(db, "tutor", "tutor@test.com", "password", false)
            .await
            .expect("Failed to create tutor user");
        let student_user = UserModel::create(db, "student", "student@test.com", "password", false)
            .await
            .expect("Failed to create student user");
        
        UserModuleRoleModel::assign_user_to_module(db, lecturer_user.id, module.id, Role::Lecturer).await.unwrap();
        UserModuleRoleModel::assign_user_to_module(db, assistant_user.id, module.id, Role::AssistantLecturer).await.unwrap();
        UserModuleRoleModel::assign_user_to_module(db, tutor_user.id, module.id, Role::Tutor).await.unwrap();
        UserModuleRoleModel::assign_user_to_module(db, student_user.id, module.id, Role::Student).await.unwrap();
        
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
            student_user.id, 
            1, 
            10,
            10,
            false, 
            "sub1.txt", 
            "hash123#", 
            b"ontime"
        ).await.unwrap();
        let submission2 = SubmissionModel::save_file(
            db, 
            assignment.id, 
            student_user.id, 
            1, 
            10,
            10,
            false, 
            "sub2.txt", 
            "hash456#", 
            b"ontime"
        ).await.unwrap();

        // NOTE: create_case signature now accepts similarity; seed with 25.0%
        let plagiarism_case = PlagiarismCaseModel::create_case(
            db,
            assignment.id,
            submission1.id,
            submission2.id,
            "Initial description",
            25.0_f32,
        ).await.unwrap();

        TestData {
            lecturer_user,
            assistant_user,
            tutor_user,
            student_user,
            module,
            assignment,
            plagiarism_case,
        }
    }

    fn make_put_request(
        user: &UserModel,
        module_id: i64,
        assignment_id: i64,
        case_id: i64,
        payload: &UpdatePlagiarismCasePayload,
    ) -> Request<AxumBody> {
        let (token, _) = generate_jwt(user.id, user.admin);
        let uri = format!(
            "/api/modules/{}/assignments/{}/plagiarism/{}", 
            module_id, assignment_id, case_id
        );
        let body = AxumBody::from(serde_json::to_string(&payload).unwrap());
        
        Request::builder()
            .method("PUT")
            .uri(&uri)
            .header("Authorization", format!("Bearer {}", token))
            .header("Content-Type", "application/json")
            .body(body)
            .unwrap()
    }

    /// Successful Update by Lecturer (description + status + similarity)
    #[tokio::test]
    #[serial]
    async fn test_update_plagiarism_case_success_as_lecturer() {
        let app = make_test_app().await;
        let data = setup_test_data(db::get_connection().await).await;

        let original_updated_at = data.plagiarism_case.updated_at;
        tokio::time::sleep(std::time::Duration::from_millis(10)).await;

        let payload = UpdatePlagiarismCasePayload {
            description: Some("Updated description".to_string()),
            status: Some("flagged".to_string()),
            similarity: Some(80.25),
        };

        let req = make_put_request(
            &data.lecturer_user,
            data.module.id,
            data.assignment.id,
            data.plagiarism_case.id,
            &payload,
        );
        
        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        
        let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let json: Value = serde_json::from_slice(&body).unwrap();
        
        assert_eq!(json["success"], true);
        assert_eq!(json["message"], "Plagiarism case updated successfully");
        
        let case_data = &json["data"];
        assert_eq!(case_data["id"], data.plagiarism_case.id);
        assert_eq!(case_data["description"], "Updated description");
        // status serialized as lowercase string
        assert_eq!(case_data["status"], "flagged");
        assert!(approx_eq_f64(case_data["similarity"].as_f64().unwrap(), 80.25, 1e-6));
        assert!(*case_data["updated_at"].as_str().unwrap() > *original_updated_at.to_rfc3339());

        let updated_case = PlagiarismCaseEntity::find_by_id(data.plagiarism_case.id)
            .one(db::get_connection().await)
            .await
            .unwrap()
            .expect("Case should exist");
        assert_eq!(updated_case.description, "Updated description");
        assert_eq!(updated_case.status, Status::Flagged);
        assert!((updated_case.similarity as f64 - 80.25).abs() < 1e-6);
        assert!(updated_case.updated_at > original_updated_at);
    }

    /// Partial Update by Assistant Lecturer (description only; similarity & status unchanged)
    #[tokio::test]
    #[serial]
    async fn test_update_plagiarism_case_partial_update() {
        let app = make_test_app().await;
        let data = setup_test_data(db::get_connection().await).await;
        
        let original_updated_at = data.plagiarism_case.updated_at;
        tokio::time::sleep(std::time::Duration::from_millis(10)).await;

        let payload = UpdatePlagiarismCasePayload {
            description: Some("Assistant updated description".to_string()),
            status: None,
            similarity: None,
        };

        let req = make_put_request(
            &data.assistant_user,
            data.module.id,
            data.assignment.id,
            data.plagiarism_case.id,
            &payload,
        );
        
        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        
        let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let json: Value = serde_json::from_slice(&body).unwrap();
        
        let case_data = &json["data"];
        assert_eq!(case_data["description"], "Assistant updated description");
        // stays review by default
        assert_eq!(case_data["status"], "review");
        // similarity should remain at the seeded 25.0
        assert!(approx_eq_f64(case_data["similarity"].as_f64().unwrap(), 25.0, 1e-6));
        assert!(*case_data["updated_at"].as_str().unwrap() > *original_updated_at.to_rfc3339());
        
        let updated_case = PlagiarismCaseEntity::find_by_id(data.plagiarism_case.id)
            .one(db::get_connection().await)
            .await
            .unwrap()
            .expect("Case should exist");
        assert_eq!(updated_case.description, "Assistant updated description");
        assert_eq!(updated_case.status, Status::Review);
        assert!((updated_case.similarity as f64 - 25.0).abs() < 1e-6);
    }

    /// Update similarity only
    #[tokio::test]
    async fn test_update_plagiarism_case_similarity_only() {
        let (app, app_state) = make_test_app().await;
        let data = setup_test_data(app_state.db()).await;

        let payload = UpdatePlagiarismCasePayload {
            description: None,
            status: None,
            similarity: Some(42.0),
        };

        let req = make_put_request(
            &data.lecturer_user,
            data.module.id,
            data.assignment.id,
            data.plagiarism_case.id,
            &payload,
        );

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let json: Value = serde_json::from_slice(&body).unwrap();
        assert!(approx_eq_f64(json["data"]["similarity"].as_f64().unwrap(), 42.0, 1e-6));

        let updated_case = PlagiarismCaseEntity::find_by_id(data.plagiarism_case.id)
            .one(app_state.db())
            .await
            .unwrap()
            .expect("Case should exist");
        assert!((updated_case.similarity as f64 - 42.0).abs() < 1e-6);
    }

    /// Forbidden Access for Non-Permitted Roles
    #[tokio::test]
    async fn test_update_plagiarism_case_forbidden_roles() {
        let (app, app_state) = make_test_app().await;
        let data = setup_test_data(app_state.db()).await;

        let payload = UpdatePlagiarismCasePayload {
            description: Some("Unauthorized update".to_string()),
            status: Some("reviewed".to_string()),
            similarity: Some(12.0),
        };

        let req = make_put_request(
            &data.tutor_user,
            data.module.id,
            data.assignment.id,
            data.plagiarism_case.id,
            &payload,
        );
        let response = app.clone().oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::FORBIDDEN);

        let req = make_put_request(
            &data.student_user,
            data.module.id,
            data.assignment.id,
            data.plagiarism_case.id,
            &payload,
        );
        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::FORBIDDEN);
    }

    /// Validation Failures
    #[tokio::test]
    #[serial]
    async fn test_update_plagiarism_case_validation_errors() {
        let app = make_test_app().await;
        let data = setup_test_data(db::get_connection().await).await;

        // No fields -> 400
        let payload = UpdatePlagiarismCasePayload {
            description: None,
            status: None,
            similarity: None,
        };
        let req = make_put_request(
            &data.lecturer_user,
            data.module.id,
            data.assignment.id,
            data.plagiarism_case.id,
            &payload,
        );
        let response = app.clone().oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
        let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let json: Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(json["message"], "At least one field (description, status, or similarity) must be provided");

        // Invalid status -> 400
        let payload = UpdatePlagiarismCasePayload {
            description: None,
            status: Some("invalid_status".to_string()),
            similarity: None,
        };
        let req = make_put_request(
            &data.lecturer_user,
            data.module.id,
            data.assignment.id,
            data.plagiarism_case.id,
            &payload,
        );
        let response = app.clone().oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
        let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let json: Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(
            json["message"], 
            "Invalid status value. Must be one of: 'review', 'flagged', 'reviewed'"
        );

        // similarity < 0 -> 400
        let payload = UpdatePlagiarismCasePayload {
            description: None,
            status: None,
            similarity: Some(-0.1),
        };
        let req = make_put_request(
            &data.lecturer_user,
            data.module.id,
            data.assignment.id,
            data.plagiarism_case.id,
            &payload,
        );
        let response = app.clone().oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);

        // similarity > 100 -> 400
        let payload = UpdatePlagiarismCasePayload {
            description: None,
            status: None,
            similarity: Some(120.0),
        };
        let req = make_put_request(
            &data.lecturer_user,
            data.module.id,
            data.assignment.id,
            data.plagiarism_case.id,
            &payload,
        );
        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    /// Case Not Found
    #[tokio::test]
    #[serial]
    async fn test_update_plagiarism_case_not_found() {
        let app = make_test_app().await;
        let data = setup_test_data(db::get_connection().await).await;

        let payload = UpdatePlagiarismCasePayload {
            description: Some("Update non-existent case".to_string()),
            status: Some("reviewed".to_string()),
            similarity: Some(33.0),
        };

        let req = make_put_request(
            &data.lecturer_user,
            data.module.id,
            data.assignment.id,
            999999,
            &payload,
        );
        
        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
        
        let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let json: Value = serde_json::from_slice(&body).unwrap();

        // If your handler returns a simple "Plagiarism case not found", assert that instead:
        // assert_eq!(json["message"], "Plagiarism case not found");
        // Keeping original custom message if your route uses it:
        assert_eq!(json["success"], false);
        assert_eq!(json["message"], "Plagiarism case 999999 in Assignment 1 not found.");
    }

    /// Unauthorized Access
    #[tokio::test]
    #[serial]
    async fn test_update_plagiarism_case_unauthorized() {
        let app = make_test_app().await;
        let data = setup_test_data(db::get_connection().await).await;

        let payload = UpdatePlagiarismCasePayload {
            description: Some("Unauthorized update".to_string()),
            status: Some("reviewed".to_string()),
            similarity: Some(10.0),
        };

        let uri = format!(
            "/api/modules/{}/assignments/{}/plagiarism/{}", 
            data.module.id, data.assignment.id, data.plagiarism_case.id
        );
        let req = Request::builder()
            .method("PUT")
            .uri(&uri)
            .header("Content-Type", "application/json")
            .body(AxumBody::from(serde_json::to_string(&payload).unwrap()))
            .unwrap();
        let response = app.clone().oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);

        let req = Request::builder()
            .method("PUT")
            .uri(&uri)
            .header("Authorization", "Bearer invalid.token.here")
            .header("Content-Type", "application/json")
            .body(AxumBody::from(serde_json::to_string(&payload).unwrap()))
            .unwrap();
        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }
}
