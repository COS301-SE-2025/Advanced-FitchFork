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

        let module = ModuleModel::create(db, "CS101", Utc::now().year(), Some("Intro to CS"), 5).await.expect("Failed to create test module");
        let service = UserService::new(UserRepository::new(db.clone()));
        let lecturer_user = service.create(CreateUser{ username: "lecturer".to_string(), email: "lecturer@test.com".to_string(), password: "password".to_string(), admin: false }).await.expect("Failed to create lecturer user");
        let assistant_user = service.create(CreateUser{ username: "assistant".to_string(), email: "assistant@test.com".to_string(), password: "password".to_string(), admin: false }).await.expect("Failed to create assistant user");
        let tutor_user = service.create(CreateUser{ username: "tutor".to_string(), email: "tutor@test.com".to_string(), password: "password".to_string(), admin: false }).await.expect("Failed to create tutor user");
        let student_user = service.create(CreateUser{ username: "student".to_string(), email: "student@test.com".to_string(), password: "password".to_string(), admin: false }).await.expect("Failed to create student user");
        UserModuleRoleModel::assign_user_to_module(db, lecturer_user.id, module.id, Role::Lecturer).await.expect("Failed to assign lecturer role");
        UserModuleRoleModel::assign_user_to_module(db, assistant_user.id, module.id, Role::AssistantLecturer).await.expect("Failed to assign assistant lecturer role");
        UserModuleRoleModel::assign_user_to_module(db, tutor_user.id, module.id, Role::Tutor).await.expect("Failed to assign tutor role");
        UserModuleRoleModel::assign_user_to_module(db, student_user.id, module.id, Role::Student).await.expect("Failed to assign student role");
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
            false, 
            "sub2.txt", 
            "hash456#", 
            b"ontime"
        ).await.unwrap();
        let plagiarism_case = PlagiarismCaseModel::create_case(
            db,
            assignment.id,
            submission1.id,
            submission2.id,
            "Initial description"
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

    /// Test Case: Successful Update by Lecturer
    #[tokio::test]
    async fn test_update_plagiarism_case_success_as_lecturer() {
        let (app, app_state) = make_test_app().await;
        let data = setup_test_data(app_state.db()).await;

        let original_updated_at = data.plagiarism_case.updated_at;
        tokio::time::sleep(std::time::Duration::from_millis(10)).await;

        let payload = UpdatePlagiarismCasePayload {
            description: Some("Updated description".to_string()),
            status: Some("flagged".to_string()),
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
        assert_eq!(case_data["status"], "Flagged");
        assert!(*case_data["updated_at"].as_str().unwrap() > *original_updated_at.to_rfc3339());

        let updated_case = PlagiarismCaseEntity::find_by_id(data.plagiarism_case.id)
            .one(app_state.db())
            .await
            .unwrap()
            .expect("Case should exist");
        assert_eq!(updated_case.description, "Updated description");
        assert_eq!(updated_case.status, Status::Flagged);
        assert!(updated_case.updated_at > original_updated_at);
    }

    /// Test Case: Partial Update by Assistant Lecturer
    #[tokio::test]
    async fn test_update_plagiarism_case_partial_update() {
        let (app, app_state) = make_test_app().await;
        let data = setup_test_data(app_state.db()).await;
        
        let original_updated_at = data.plagiarism_case.updated_at;
        tokio::time::sleep(std::time::Duration::from_millis(10)).await;

        let payload = UpdatePlagiarismCasePayload {
            description: Some("Assistant updated description".to_string()),
            status: None,
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
        assert_eq!(case_data["status"], "Review");
        assert!(*case_data["updated_at"].as_str().unwrap() > *original_updated_at.to_rfc3339());
        
        let updated_case = PlagiarismCaseEntity::find_by_id(data.plagiarism_case.id)
            .one(app_state.db())
            .await
            .unwrap()
            .expect("Case should exist");
        assert_eq!(updated_case.description, "Assistant updated description");
        assert_eq!(updated_case.status, Status::Review);
    }

    /// Test Case: Forbidden Access for Non-Permitted Roles
    #[tokio::test]
    async fn test_update_plagiarism_case_forbidden_roles() {
        let (app, app_state) = make_test_app().await;
        let data = setup_test_data(app_state.db()).await;

        let payload = UpdatePlagiarismCasePayload {
            description: Some("Unauthorized update".to_string()),
            status: Some("reviewed".to_string()),
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

    /// Test Case: Validation Failures
    #[tokio::test]
    async fn test_update_plagiarism_case_validation_errors() {
        let (app, app_state) = make_test_app().await;
        let data = setup_test_data(app_state.db()).await;

        let payload = UpdatePlagiarismCasePayload {
            description: None,
            status: None,
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
        assert_eq!(json["message"], "At least one field (description or status) must be provided");

        let payload = UpdatePlagiarismCasePayload {
            description: None,
            status: Some("invalid_status".to_string()),
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
        let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let json: Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(
            json["message"], 
            "Invalid status value. Must be one of: 'review', 'flagged', 'reviewed'"
        );
    }

    /// Test Case: Case Not Found
    #[tokio::test]
    async fn test_update_plagiarism_case_not_found() {
        let (app, app_state) = make_test_app().await;
        let data = setup_test_data(app_state.db()).await;

        let payload = UpdatePlagiarismCasePayload {
            description: Some("Update non-existent case".to_string()),
            status: Some("reviewed".to_string()),
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
        assert_eq!(json["success"], false);
        assert_eq!(json["message"], "Plagiarism case 999999 in Assignment 1 not found.");
    }

    /// Test Case: Unauthorized Access
    #[tokio::test]
    async fn test_update_plagiarism_case_unauthorized() {
        let (app, app_state) = make_test_app().await;
        let data = setup_test_data(app_state.db()).await;

        let payload = UpdatePlagiarismCasePayload {
            description: Some("Unauthorized update".to_string()),
            status: Some("reviewed".to_string()),
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