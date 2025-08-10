#[cfg(test)]
mod common {
    use api::auth::generate_jwt;
    use axum::{
        body::Body as AxumBody,
        http::Request,
    };
    use chrono::{Datelike, TimeZone, Utc};
    use db::models::{
        assignment::{AssignmentType, Model as AssignmentModel},
        assignment_submission::Model as SubmissionModel,
        module::Model as ModuleModel,
        plagiarism_case::{Model as PlagiarismCaseModel},
        user::Model as UserModel,
        user_module_role::{Model as UserModuleRoleModel, Role},
    };
    use sea_orm::{DatabaseConnection};
    
    pub struct TestData {
        pub lecturer_user: UserModel,
        pub assistant_user: UserModel,
        pub tutor_user: UserModel,
        pub student_user: UserModel,
        pub module: ModuleModel,
        pub assignment: AssignmentModel,
        pub plagiarism_case: PlagiarismCaseModel,
        pub submission1: SubmissionModel,
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
            submission1,
        }
    }

    pub fn make_delete_request(
        user: &UserModel,
        module_id: i64,
        assignment_id: i64,
        case_id: i64,
    ) -> Request<AxumBody> {
        let (token, _) = generate_jwt(user.id, user.admin);
        let uri = format!(
            "/api/modules/{}/assignments/{}/plagiarism/{}",
            module_id, assignment_id, case_id
        );

        Request::builder()
            .method("DELETE")
            .uri(&uri)
            .header("Authorization", format!("Bearer {}", token))
            .body(AxumBody::empty())
            .unwrap()
    }
}


#[cfg(test)]
mod delete_plagiarism_tests {
    use super::common::*;
    use crate::helpers::app::make_test_app;
    use axum::http::StatusCode;
    use db::models::plagiarism_case::Entity as PlagiarismCaseEntity;
    use sea_orm::EntityTrait;
    use serde_json::Value;
    use tower::ServiceExt;

    /// Test Case: Successful Deletion by Lecturer
    #[tokio::test]
    async fn test_delete_plagiarism_case_success_as_lecturer() {
        let (app, app_state) = make_test_app().await;
        let data = setup_test_data(app_state.db()).await;

        let req = make_delete_request(
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
        assert_eq!(json["message"], "Plagiarism case deleted successfully");

        // Verify database deletion
        let deleted_case = PlagiarismCaseEntity::find_by_id(data.plagiarism_case.id)
            .one(app_state.db())
            .await
            .unwrap();
        assert!(deleted_case.is_none());
    }

    /// Test Case: Successful Deletion by Assistant Lecturer
    #[tokio::test]
    async fn test_delete_plagiarism_case_success_as_assistant() {
        let (app, app_state) = make_test_app().await;
        let data = setup_test_data(app_state.db()).await;

        let req = make_delete_request(
            &data.assistant_user,
            data.module.id,
            data.assignment.id,
            data.plagiarism_case.id,
        );

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);

        // Verify database deletion
        let deleted_case = PlagiarismCaseEntity::find_by_id(data.plagiarism_case.id)
            .one(app_state.db())
            .await
            .unwrap();
        assert!(deleted_case.is_none());
    }

    /// Test Case: Forbidden Access for Non-Permitted Roles
    #[tokio::test]
    async fn test_delete_plagiarism_case_forbidden_roles() {
        let (app, app_state) = make_test_app().await;
        let data = setup_test_data(app_state.db()).await;

        // Test tutor
        let req = make_delete_request(
            &data.tutor_user,
            data.module.id,
            data.assignment.id,
            data.plagiarism_case.id,
        );
        let response = app.clone().oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::FORBIDDEN);

        // Test student
        let req = make_delete_request(
            &data.student_user,
            data.module.id,
            data.assignment.id,
            data.plagiarism_case.id,
        );
        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::FORBIDDEN);
    }

    /// Test Case: Case Not Found
    #[tokio::test]
    async fn test_delete_plagiarism_case_not_found() {
        let (app, app_state) = make_test_app().await;
        let data = setup_test_data(app_state.db()).await;

        let req = make_delete_request(
            &data.lecturer_user,
            data.module.id,
            data.assignment.id,
            999999, // Non-existent case ID
        );

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::NOT_FOUND);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let json: Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(json["success"], false);
        assert_eq!(json["message"], "Plagiarism case 999999 in Assignment 1 not found.");
    }

    /// Test Case: Unauthorized Access
    #[tokio::test]
    async fn test_delete_plagiarism_case_unauthorized() {
        let (app, app_state) = make_test_app().await;
        let data = setup_test_data(app_state.db()).await;

        let uri = format!(
            "/api/modules/{}/assignments/{}/plagiarism/{}",
            data.module.id, data.assignment.id, data.plagiarism_case.id
        );

        // Case 1: Missing authorization header
        let req = axum::http::Request::builder()
            .method("DELETE")
            .uri(&uri)
            .body(axum::body::Body::empty())
            .unwrap();
        let response = app.clone().oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);

        // Case 2: Invalid token
        let req = axum::http::Request::builder()
            .method("DELETE")
            .uri(&uri)
            .header("Authorization", "Bearer invalid.token.here")
            .body(axum::body::Body::empty())
            .unwrap();
        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }
}

#[cfg(test)]
mod bulk_delete_plagiarism_tests {
    use super::common::*;
    use crate::helpers::app::make_test_app;
    use api::auth::generate_jwt;
    use axum::{
        body::Body as AxumBody,
        http::{Request, StatusCode},
    };
    use db::models::{
        assignment_submission::Model as SubmissionModel,
        plagiarism_case::{Entity as PlagiarismCaseEntity, Model as PlagiarismCaseModel},
        user::Model as UserModel,
    };
    use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter};
    use serde_json::{json, Value};
    use tower::ServiceExt;

    async fn setup_bulk_test_data(
        db: &DatabaseConnection,
    ) -> (TestData, Vec<PlagiarismCaseModel>) {
        let data = setup_test_data(db).await;
        let mut extra_cases = Vec::new();

        // Create a few more submissions and cases for bulk testing
        let submission3 = SubmissionModel::save_file(
            db,
            data.assignment.id,
            data.student_user.id,
            1,
            false,
            "sub3.txt",
            "hash789#",
            b"ontime",
        )
        .await
        .unwrap();
        let submission4 = SubmissionModel::save_file(
            db,
            data.assignment.id,
            data.student_user.id,
            1,
            false,
            "sub4.txt",
            "hash101#",
            b"ontime",
        )
        .await
        .unwrap();

        let case2 =
            PlagiarismCaseModel::create_case(db, data.assignment.id, submission3.id, submission4.id, "Case 2")
                .await
                .unwrap();
        let case3 = PlagiarismCaseModel::create_case(
            db,
            data.assignment.id,
            data.submission1.id,
            submission3.id,
            "Case 3",
        )
        .await
        .unwrap();

        extra_cases.push(case2);
        extra_cases.push(case3);

        (data, extra_cases)
    }

    fn make_bulk_delete_request(
        user: &UserModel,
        module_id: i64,
        assignment_id: i64,
        case_ids: &[i64],
    ) -> Request<AxumBody> {
        let (token, _) = generate_jwt(user.id, user.admin);
        let uri = format!(
            "/api/modules/{}/assignments/{}/plagiarism/bulk",
            module_id, assignment_id
        );
        let payload = json!({ "case_ids": case_ids });
        let body = AxumBody::from(serde_json::to_string(&payload).unwrap());

        Request::builder()
            .method("DELETE")
            .uri(&uri)
            .header("Authorization", format!("Bearer {}", token))
            .header("Content-Type", "application/json")
            .body(body)
            .unwrap()
    }

    #[tokio::test]
    async fn test_bulk_delete_success() {
        let (app, app_state) = make_test_app().await;
        let (data, extra_cases) = setup_bulk_test_data(app_state.db()).await;
        let req = make_bulk_delete_request(
            &data.lecturer_user,
            data.module.id,
            data.assignment.id,
            &[
                data.plagiarism_case.id,
                extra_cases[0].id,
                extra_cases[1].id,
            ],
        );
        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let json: Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(json["success"], true);
        assert_eq!(json["message"], "3 plagiarism cases deleted successfully");

        let remaining_cases = PlagiarismCaseEntity::find()
            .filter(<PlagiarismCaseEntity as EntityTrait>::Column::Id.is_in(vec![
                data.plagiarism_case.id,
                extra_cases[0].id,
                extra_cases[1].id,
            ]))
            .all(app_state.db())
            .await
            .unwrap();
        assert!(remaining_cases.is_empty());
    }

    #[tokio::test]
    async fn test_bulk_delete_empty_list() {
        let (app, app_state) = make_test_app().await;
        let data = setup_test_data(app_state.db()).await;

        let req =
            make_bulk_delete_request(&data.lecturer_user, data.module.id, data.assignment.id, &[]);
        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let json: Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(json["message"], "case_ids cannot be empty");
    }

    #[tokio::test]
    async fn test_bulk_delete_not_found() {
        let (app, app_state) = make_test_app().await;
        let (data, _) = setup_bulk_test_data(app_state.db()).await;
        let case_ids_to_delete = vec![data.plagiarism_case.id, 999999];

        let req = make_bulk_delete_request(
            &data.lecturer_user,
            data.module.id,
            data.assignment.id,
            &case_ids_to_delete,
        );
        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let json: Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(
            json["message"],
            "Some plagiarism cases not found or not in assignment: [999999]",
        );
    }

    #[tokio::test]
    async fn test_bulk_delete_forbidden() {
        let (app, app_state) = make_test_app().await;
        let (data, extra_cases) = setup_bulk_test_data(app_state.db()).await;
        let case_ids_to_delete = vec![data.plagiarism_case.id, extra_cases[0].id];

        let req = make_bulk_delete_request(
            &data.student_user,
            data.module.id,
            data.assignment.id,
            &case_ids_to_delete,
        );
        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::FORBIDDEN);
    }
}