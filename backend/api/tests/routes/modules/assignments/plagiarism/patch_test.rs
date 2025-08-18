#[cfg(test)]
mod patch_plagiarism_tests {
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
        plagiarism_case::{Entity as PlagiarismCaseEntity, Model as PlagiarismCaseModel, Status},
        user::Model as UserModel,
        user_module_role::{Model as UserModuleRoleModel, Role},
    };
    use sea_orm::{DatabaseConnection, EntityTrait};
    use serde_json::Value;
    use tower::ServiceExt;

    use crate::helpers::app::make_test_app;

    pub struct TestData {
        pub lecturer_user: UserModel,
        pub assistant_user: UserModel,
        pub tutor_user: UserModel,
        pub student_user: UserModel,
        pub module: ModuleModel,
        pub assignment: AssignmentModel,
        pub plagiarism_case: PlagiarismCaseModel,
    }

    pub async fn setup_test_data(db: &DatabaseConnection) -> TestData {
        dotenvy::dotenv().ok();

        // Create module
        let module = ModuleModel::create(db, "CS101", Utc::now().year(), Some("Intro to CS"), 5)
            .await
            .expect("Failed to create test module");

        // Create users
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
        let student_user =
            UserModel::create(db, "student", "student@test.com", "password", false)
                .await
                .expect("Failed to create student user");

        // Assign roles
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
        UserModuleRoleModel::assign_user_to_module(db, student_user.id, module.id, Role::Student)
            .await
            .expect("Failed to assign student role");

        // Create assignment
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

        // Create submissions
        let submission1 = SubmissionModel::save_file(
            db,
            assignment.id,
            student_user.id,
            1,
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
            student_user.id,
            1,
            false,
            "sub2.txt",
            "hash456#",
            b"ontime",
        )
        .await
        .unwrap();

        // Create plagiarism case
        let plagiarism_case = PlagiarismCaseModel::create_case(
            db,
            assignment.id,
            submission1.id,
            submission2.id,
            "Initial description",
            0.0
        )
        .await
        .unwrap();

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

    fn make_patch_request(
        user: &UserModel,
        module_id: i64,
        assignment_id: i64,
        case_id: i64,
    ) -> Request<AxumBody> {
        let (token, _) = generate_jwt(user.id, user.admin);
        let uri = format!(
            "/api/modules/{}/assignments/{}/plagiarism/{}/flag",
            module_id, assignment_id, case_id
        );

        Request::builder()
            .method("PATCH")
            .uri(&uri)
            .header("Authorization", format!("Bearer {}", token))
            .body(AxumBody::empty())
            .unwrap()
    }

    #[tokio::test]
    async fn test_flag_plagiarism_case_success_as_lecturer() {
        let (app, app_state) = make_test_app().await;
        let data = setup_test_data(app_state.db()).await;

        let req = make_patch_request(
            &data.lecturer_user,
            data.module.id,
            data.assignment.id,
            data.plagiarism_case.id,
        );

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let json: Value = serde_json::from_slice(&body).unwrap();

        assert_eq!(json["success"], true);
        assert_eq!(json["message"], "Plagiarism case flagged");
        assert_eq!(json["data"]["status"], "Flagged");

        let updated_case = PlagiarismCaseEntity::find_by_id(data.plagiarism_case.id)
            .one(app_state.db())
            .await
            .unwrap()
            .unwrap();
        assert_eq!(updated_case.status, Status::Flagged);
    }

    #[tokio::test]
    async fn test_flag_plagiarism_case_success_as_assistant() {
        let (app, app_state) = make_test_app().await;
        let data = setup_test_data(app_state.db()).await;

        let req = make_patch_request(
            &data.assistant_user,
            data.module.id,
            data.assignment.id,
            data.plagiarism_case.id,
        );

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);

        let updated_case = PlagiarismCaseEntity::find_by_id(data.plagiarism_case.id)
            .one(app_state.db())
            .await
            .unwrap()
            .unwrap();
        assert_eq!(updated_case.status, Status::Flagged);
    }

    #[tokio::test]
    async fn test_flag_plagiarism_case_forbidden_roles() {
        let (app, app_state) = make_test_app().await;
        let data = setup_test_data(app_state.db()).await;

        // Test tutor
        let req = make_patch_request(
            &data.tutor_user,
            data.module.id,
            data.assignment.id,
            data.plagiarism_case.id,
        );
        let response = app.clone().oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::FORBIDDEN);

        // Test student
        let req = make_patch_request(
            &data.student_user,
            data.module.id,
            data.assignment.id,
            data.plagiarism_case.id,
        );
        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::FORBIDDEN);
    }

    #[tokio::test]
    async fn test_flag_plagiarism_case_not_found() {
        let (app, app_state) = make_test_app().await;
        let data = setup_test_data(app_state.db()).await;

        let req = make_patch_request(
            &data.lecturer_user,
            data.module.id,
            data.assignment.id,
            999999, // Non-existent case ID
        );

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn test_flag_plagiarism_case_unauthorized() {
        let (app, app_state) = make_test_app().await;
        let data = setup_test_data(app_state.db()).await;

        let uri = format!(
            "/api/modules/{}/assignments/{}/plagiarism/{}/flag",
            data.module.id, data.assignment.id, data.plagiarism_case.id
        );

        // Case 1: Missing authorization header
        let req = Request::builder()
            .method("PATCH")
            .uri(&uri)
            .body(AxumBody::empty())
            .unwrap();
        let response = app.clone().oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);

        // Case 2: Invalid token
        let req = Request::builder()
            .method("PATCH")
            .uri(&uri)
            .header("Authorization", "Bearer invalid.token.here")
            .body(AxumBody::empty())
            .unwrap();
        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }
}

#[cfg(test)]
mod review_plagiarism_tests {
    use super::patch_plagiarism_tests::setup_test_data;
    use api::auth::generate_jwt;
    use axum::{
        body::Body as AxumBody,
        http::{Request, StatusCode},
    };
    use db::models::{
        plagiarism_case::{Entity as PlagiarismCaseEntity, Status},
        user::Model as UserModel,
    };
    use sea_orm::EntityTrait;
    use serde_json::Value;
    use tower::ServiceExt;

    use crate::helpers::app::make_test_app;

    fn make_review_request(
        user: &UserModel,
        module_id: i64,
        assignment_id: i64,
        case_id: i64,
    ) -> Request<AxumBody> {
        let (token, _) = generate_jwt(user.id, user.admin);
        let uri = format!(
            "/api/modules/{}/assignments/{}/plagiarism/{}/review",
            module_id, assignment_id, case_id
        );

        Request::builder()
            .method("PATCH")
            .uri(&uri)
            .header("Authorization", format!("Bearer {}", token))
            .body(AxumBody::empty())
            .unwrap()
    }

    #[tokio::test]
    async fn test_review_plagiarism_case_success_as_lecturer() {
        let (app, app_state) = make_test_app().await;
        let data = setup_test_data(app_state.db()).await;

        let req = make_review_request(
            &data.lecturer_user,
            data.module.id,
            data.assignment.id,
            data.plagiarism_case.id,
        );

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let json: Value = serde_json::from_slice(&body).unwrap();

        assert_eq!(json["success"], true);
        assert_eq!(json["message"], "Plagiarism case marked as reviewed");
        assert_eq!(json["data"]["status"], "Reviewed");

        let updated_case = PlagiarismCaseEntity::find_by_id(data.plagiarism_case.id)
            .one(app_state.db())
            .await
            .unwrap()
            .unwrap();
        assert_eq!(updated_case.status, Status::Reviewed);
    }

    #[tokio::test]
    async fn test_review_plagiarism_case_success_as_assistant() {
        let (app, app_state) = make_test_app().await;
        let data = setup_test_data(app_state.db()).await;

        let req = make_review_request(
            &data.assistant_user,
            data.module.id,
            data.assignment.id,
            data.plagiarism_case.id,
        );

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);

        let updated_case = PlagiarismCaseEntity::find_by_id(data.plagiarism_case.id)
            .one(app_state.db())
            .await
            .unwrap()
            .unwrap();
        assert_eq!(updated_case.status, Status::Reviewed);
    }

    #[tokio::test]
    async fn test_review_plagiarism_case_forbidden_roles() {
        let (app, app_state) = make_test_app().await;
        let data = setup_test_data(app_state.db()).await;

        // Test tutor
        let req = make_review_request(
            &data.tutor_user,
            data.module.id,
            data.assignment.id,
            data.plagiarism_case.id,
        );
        let response = app.clone().oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::FORBIDDEN);

        // Test student
        let req = make_review_request(
            &data.student_user,
            data.module.id,
            data.assignment.id,
            data.plagiarism_case.id,
        );
        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::FORBIDDEN);
    }

    #[tokio::test]
    async fn test_review_plagiarism_case_not_found() {
        let (app, app_state) = make_test_app().await;
        let data = setup_test_data(app_state.db()).await;

        let req = make_review_request(
            &data.lecturer_user,
            data.module.id,
            data.assignment.id,
            999999, // Non-existent case ID
        );

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn test_review_plagiarism_case_unauthorized() {
        let (app, app_state) = make_test_app().await;
        let data = setup_test_data(app_state.db()).await;

        let uri = format!(
            "/api/modules/{}/assignments/{}/plagiarism/{}/review",
            data.module.id, data.assignment.id, data.plagiarism_case.id
        );

        // Case 1: Missing authorization header
        let req = Request::builder()
            .method("PATCH")
            .uri(&uri)
            .body(AxumBody::empty())
            .unwrap();
        let response = app.clone().oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);

        // Case 2: Invalid token
        let req = Request::builder()
            .method("PATCH")
            .uri(&uri)
            .header("Authorization", "Bearer invalid.token.here")
            .body(AxumBody::empty())
            .unwrap();
        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }
}