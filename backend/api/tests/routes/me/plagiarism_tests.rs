#[cfg(test)]
mod tests {
    use crate::helpers::app::make_test_app_with_storage;
    use api::auth::generate_jwt;
    use axum::{
        body::Body as AxumBody,
        http::{Request, StatusCode},
    };
    use chrono::Utc;
    use db::models::{
        assignment::Model as AssignmentModel,
        assignment_submission::Model as SubmissionModel,
        module::Model as ModuleModel,
        plagiarism_case::Model as PlagiarismModel,
        user::Model as UserModel,
        user_module_role::{Model as UserModuleRoleModel, Role},
    };
    use serde_json::Value;
    use serial_test::serial;
    use tower::ServiceExt;

    struct TestData {
        lecturer: UserModel,
        assistant: UserModel,
        student: UserModel,
        _module: ModuleModel,
        assignment: AssignmentModel,
        case: PlagiarismModel,
    }

    async fn setup_test_data(db: &sea_orm::DatabaseConnection) -> TestData {
        let lecturer = UserModel::create(db, "lecturer", "lecturer@test.com", "secret", false)
            .await
            .unwrap();
        let assistant = UserModel::create(db, "assistant", "assistant@test.com", "secret", false)
            .await
            .unwrap();
        let student_a = UserModel::create(db, "student_a", "student_a@test.com", "secret", false)
            .await
            .unwrap();
        let student_b = UserModel::create(db, "student_b", "student_b@test.com", "secret", false)
            .await
            .unwrap();

        let module = ModuleModel::create(db, "COS700", 2024, None, 15)
            .await
            .unwrap();

        UserModuleRoleModel::assign_user_to_module(db, lecturer.id, module.id, Role::Lecturer)
            .await
            .unwrap();
        UserModuleRoleModel::assign_user_to_module(
            db,
            assistant.id,
            module.id,
            Role::AssistantLecturer,
        )
        .await
        .unwrap();
        UserModuleRoleModel::assign_user_to_module(db, student_a.id, module.id, Role::Student)
            .await
            .unwrap();
        UserModuleRoleModel::assign_user_to_module(db, student_b.id, module.id, Role::Student)
            .await
            .unwrap();

        let assignment = AssignmentModel::create(
            db,
            module.id,
            "Assignment 1",
            None,
            db::models::assignment::AssignmentType::Practical,
            Utc::now(),
            Utc::now(),
        )
        .await
        .unwrap();

        let submission_a = SubmissionModel::save_file(
            db,
            assignment.id,
            student_a.id,
            1,
            80,
            100,
            false,
            "a.zip",
            "hash-a",
            b"content-a",
        )
        .await
        .unwrap();
        let submission_b = SubmissionModel::save_file(
            db,
            assignment.id,
            student_b.id,
            1,
            85,
            100,
            false,
            "b.zip",
            "hash-b",
            b"content-b",
        )
        .await
        .unwrap();

        let case = PlagiarismModel::create_case(
            db,
            assignment.id,
            submission_a.id,
            submission_b.id,
            "Matched code blocks",
            92.5,
            120,
            None,
        )
        .await
        .unwrap();

        TestData {
            lecturer,
            assistant,
            student: student_a,
            _module: module,
            assignment,
            case,
        }
    }

    #[tokio::test]
    #[serial]
    async fn lecturer_can_fetch_plagiarism_cases() {
        let (app, app_state, _tmp) = make_test_app_with_storage().await;
        let data = setup_test_data(app_state.db()).await;

        let (token, _) = generate_jwt(data.lecturer.id, false);
        let req = Request::builder()
            .method("GET")
            .uri("/api/me/plagiarism")
            .header("Authorization", format!("Bearer {}", token))
            .body(AxumBody::empty())
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let json: Value = serde_json::from_slice(&body).unwrap();

        assert_eq!(json["success"], true);
        let cases = json["data"]["cases"].as_array().unwrap();
        assert_eq!(cases.len(), 1);
        assert_eq!(cases[0]["id"].as_i64().unwrap(), data.case.id);
        assert_eq!(
            cases[0]["assignment"]["id"].as_i64().unwrap(),
            data.assignment.id
        );
    }

    #[tokio::test]
    #[serial]
    async fn assistant_lecturer_can_fetch_plagiarism_cases() {
        let (app, app_state, _tmp) = make_test_app_with_storage().await;
        let data = setup_test_data(app_state.db()).await;

        let (token, _) = generate_jwt(data.assistant.id, false);
        let req = Request::builder()
            .method("GET")
            .uri("/api/me/plagiarism")
            .header("Authorization", format!("Bearer {}", token))
            .body(AxumBody::empty())
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let json: Value = serde_json::from_slice(&body).unwrap();

        assert_eq!(json["success"], true);
        assert_eq!(json["data"]["cases"].as_array().unwrap().len(), 1);
    }

    #[tokio::test]
    #[serial]
    async fn student_cannot_fetch_plagiarism_cases() {
        let (app, app_state, _tmp) = make_test_app_with_storage().await;
        let data = setup_test_data(app_state.db()).await;

        let (token, _) = generate_jwt(data.student.id, false);
        let req = Request::builder()
            .method("GET")
            .uri("/api/me/plagiarism")
            .header("Authorization", format!("Bearer {}", token))
            .body(AxumBody::empty())
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::FORBIDDEN);
    }
}
