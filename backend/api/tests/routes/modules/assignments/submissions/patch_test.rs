#[cfg(test)]
mod patch_submission_ignore_tests {
    use axum::{
        body::Body,
        http::{Request, StatusCode},
    };
    use tower::ServiceExt;
    use serde_json::{json, Value};
    use serial_test::serial;
    use chrono::Utc;

    use api::auth::generate_jwt;

    use db::models::{
        assignment::{AssignmentType, Model as AssignmentModel},
        assignment_submission::Model as AssignmentSubmissionModel,
        module::Model as ModuleModel,
        user::Model as UserModel,
        user_module_role::{Model as UserModuleRoleModel, Role},
    };

    use crate::helpers::app:: make_test_app_with_storage;

    struct TestData {
        lecturer: UserModel,
        assistant: UserModel,
        tutor: UserModel,   
        student: UserModel,
        module: ModuleModel,
        assignment: AssignmentModel,
        other_assignment: AssignmentModel,
        sub1: AssignmentSubmissionModel,
        sub2: AssignmentSubmissionModel,
    }

    async fn setup_data(db: &sea_orm::DatabaseConnection) -> TestData {
        // users
        let lecturer = UserModel::create(db, "lect1", "lect1@test.com", "pw", false).await.unwrap();
        let assistant = UserModel::create(db, "al1", "al1@test.com", "pw", false).await.unwrap();
        let tutor = UserModel::create(db, "tutor1", "tutor1@test.com", "pw", false).await.unwrap();
        let student = UserModel::create(db, "stud1", "stud1@test.com", "pw", false).await.unwrap();

        // module + roles
        let module = ModuleModel::create(db, "COS777", 2025, Some("Test"), 16).await.unwrap();
        UserModuleRoleModel::assign_user_to_module(db, lecturer.id, module.id, Role::Lecturer).await.unwrap();
        UserModuleRoleModel::assign_user_to_module(db, assistant.id, module.id, Role::AssistantLecturer).await.unwrap();
        UserModuleRoleModel::assign_user_to_module(db, tutor.id, module.id, Role::Tutor).await.unwrap();
        UserModuleRoleModel::assign_user_to_module(db, student.id, module.id, Role::Student).await.unwrap();

        // assignments
        let now = Utc::now();
        let assignment = AssignmentModel::create(
            db,
            module.id,
            "A1",
            Some("desc"),
            AssignmentType::Assignment,
            now,
            now,
        )
        .await
        .unwrap();

        let other_assignment = AssignmentModel::create(
            db,
            module.id,
            "A2",
            Some("desc"),
            AssignmentType::Assignment,
            now,
            now,
        )
        .await
        .unwrap();

        // submissions (two under A1)
        let sub1 = AssignmentSubmissionModel::save_file(
            db,
            assignment.id,
            student.id,
            1,
            10,
            10,
            false,
            "one.txt",
            "h1",
            b"x",
        )
        .await
        .unwrap();

        let sub2 = AssignmentSubmissionModel::save_file(
            db,
            assignment.id,
            student.id,
            2,
            10,
            10,
            false,
            "two.txt",
            "h2",
            b"y",
        )
        .await
        .unwrap();

        TestData {
            lecturer,
            assistant,
            tutor,
            student,
            module,
            assignment,
            other_assignment,
            sub1,
            sub2,
        }
    }

    // ---------------- SINGLE: /submissions/{submission_id}/ignore ----------------

    #[tokio::test]
    #[serial]
    async fn lecturer_can_set_single_ignored_and_unignored() {
        let (app, app_state, _tmp) = make_test_app_with_storage().await;
        let data = setup_data(app_state.db()).await;

        let (token, _) = generate_jwt(data.lecturer.id, data.lecturer.admin);
        let uri = format!(
            "/api/modules/{}/assignments/{}/submissions/{}/ignore",
            data.module.id, data.assignment.id, data.sub1.id
        );

        // set ignored = true
        let req = Request::builder()
            .method("PATCH")
            .uri(&uri)
            .header("Authorization", format!("Bearer {}", token))
            .header("content-type", "application/json")
            .body(Body::from(json!({ "ignored": true }).to_string()))
            .unwrap();
        let resp = app.clone().oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let body = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
        let v: Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(v["success"], true);
        assert_eq!(v["data"]["id"], data.sub1.id);
        assert_eq!(v["data"]["ignored"], true);
        assert!(v["data"]["updated_at"].is_string());

        // set ignored = false
        let req2 = Request::builder()
            .method("PATCH")
            .uri(&uri)
            .header("Authorization", format!("Bearer {}", token))
            .header("content-type", "application/json")
            .body(Body::from(json!({ "ignored": false }).to_string()))
            .unwrap();
        let resp2 = app.clone().oneshot(req2).await.unwrap();
        assert_eq!(resp2.status(), StatusCode::OK);
        let body2 = axum::body::to_bytes(resp2.into_body(), usize::MAX).await.unwrap();
        let v2: Value = serde_json::from_slice(&body2).unwrap();
        assert_eq!(v2["success"], true);
        assert_eq!(v2["data"]["id"], data.sub1.id);
        assert_eq!(v2["data"]["ignored"], false);
    }

    #[tokio::test]
    #[serial]
    async fn assistant_lecturer_can_set_single_ignored() {
        let (app, app_state, _tmp) = make_test_app_with_storage().await;
        let data = setup_data(app_state.db()).await;

        let (token, _) = generate_jwt(data.assistant.id, data.assistant.admin);
        let uri = format!(
            "/api/modules/{}/assignments/{}/submissions/{}/ignore",
            data.module.id, data.assignment.id, data.sub2.id
        );

        let req = Request::builder()
            .method("PATCH")
            .uri(&uri)
            .header("Authorization", format!("Bearer {}", token))
            .header("content-type", "application/json")
            .body(Body::from(json!({ "ignored": true }).to_string()))
            .unwrap();
        let resp = app.clone().oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);

        let body = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
        let v: Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(v["success"], true);
        assert_eq!(v["data"]["id"], data.sub2.id);
        assert_eq!(v["data"]["ignored"], true);
    }

    #[tokio::test]
    #[serial]
    async fn tutor_cannot_set_single_ignored_forbidden() {
        let (app, app_state, _tmp) = make_test_app_with_storage().await;
        let data = setup_data(app_state.db()).await;

        let (token, _) = generate_jwt(data.tutor.id, data.tutor.admin);
        let uri = format!(
            "/api/modules/{}/assignments/{}/submissions/{}/ignore",
            data.module.id, data.assignment.id, data.sub1.id
        );

        let req = Request::builder()
            .method("PATCH")
            .uri(&uri)
            .header("Authorization", format!("Bearer {}", token))
            .header("content-type", "application/json")
            .body(Body::from(serde_json::json!({ "ignored": true }).to_string()))
            .unwrap();

        let resp = app.clone().oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::FORBIDDEN);
    }

    #[tokio::test]
    #[serial]
    async fn student_cannot_set_single_ignored_forbidden() {
        let (app, app_state, _tmp) = make_test_app_with_storage().await;
        let data = setup_data(app_state.db()).await;

        let (token, _) = generate_jwt(data.student.id, data.student.admin);
        let uri = format!(
            "/api/modules/{}/assignments/{}/submissions/{}/ignore",
            data.module.id, data.assignment.id, data.sub1.id
        );

        let req = Request::builder()
            .method("PATCH")
            .uri(&uri)
            .header("Authorization", format!("Bearer {}", token))
            .header("content-type", "application/json")
            .body(Body::from(json!({ "ignored": true }).to_string()))
            .unwrap();
        let resp = app.clone().oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::FORBIDDEN);
    }

    #[tokio::test]
    #[serial]
    async fn single_ignored_404_when_submission_not_in_assignment() {
        let (app, app_state, _tmp) = make_test_app_with_storage().await;
        let data = setup_data(app_state.db()).await;

        // sub1 belongs to `assignment`; call with `other_assignment` → 404
        let (token, _) = generate_jwt(data.lecturer.id, data.lecturer.admin);
        let uri = format!(
            "/api/modules/{}/assignments/{}/submissions/{}/ignore",
            data.module.id, data.other_assignment.id, data.sub1.id
        );

        let req = Request::builder()
            .method("PATCH")
            .uri(&uri)
            .header("Authorization", format!("Bearer {}", token))
            .header("content-type", "application/json")
            .body(Body::from(json!({ "ignored": true }).to_string()))
            .unwrap();
        let resp = app.clone().oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::NOT_FOUND);
    }
}
