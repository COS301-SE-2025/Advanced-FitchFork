#![allow(clippy::unwrap_used)]

#[cfg(test)]
mod delete_submission_tests {
    use axum::{
        body::Body,
        http::{Request, StatusCode},
    };
    use chrono::Utc;
    use serde_json::{json, Value};
    use serial_test::serial;
    use tower::ServiceExt;

    use api::auth::generate_jwt;

    use db::models::{
        assignment::{AssignmentType, Model as AssignmentModel},
        assignment_submission::{Entity as SubmissionEntity, Model as SubmissionModel},
        module::Model as ModuleModel,
        user::Model as UserModel,
        user_module_role::{Model as UserModuleRoleModel, Role},
    };
    use sea_orm::EntityTrait;

    use crate::helpers::app::{make_test_app_with_storage};

    struct TestData {
        lecturer: UserModel,
        assistant: UserModel,
        tutor: UserModel,
        student: UserModel,
        module: ModuleModel,
        assignment: AssignmentModel,
        other_assignment: AssignmentModel,
        sub1: SubmissionModel,
        sub2: SubmissionModel,
    }

    async fn setup_data(db: &sea_orm::DatabaseConnection) -> TestData {
        // users
        let lecturer =
            UserModel::create(db, "lect_del", "lect_del@test.com", "pw", false).await.unwrap();
        let assistant =
            UserModel::create(db, "al_del", "al_del@test.com", "pw", false).await.unwrap();
        let tutor =
            UserModel::create(db, "tutor_del", "tutor_del@test.com", "pw", false).await.unwrap();
        let student =
            UserModel::create(db, "stud_del", "stud_del@test.com", "pw", false).await.unwrap();

        // module + roles
        let module = ModuleModel::create(db, "COSDEL", 2025, Some("DeleteTests"), 16)
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
        UserModuleRoleModel::assign_user_to_module(db, tutor.id, module.id, Role::Tutor)
            .await
            .unwrap();
        UserModuleRoleModel::assign_user_to_module(db, student.id, module.id, Role::Student)
            .await
            .unwrap();

        // assignments
        let now = Utc::now();
        let assignment = AssignmentModel::create(
            db,
            module.id,
            "A1-del",
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
            "A2-del",
            Some("desc"),
            AssignmentType::Assignment,
            now,
            now,
        )
        .await
        .unwrap();

        // submissions under main assignment
        let sub1 = SubmissionModel::save_file(
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

        let sub2 = SubmissionModel::save_file(
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

    // ---------------- SINGLE DELETE: /submissions/{id} ----------------

    #[tokio::test]
    #[serial]
    async fn lecturer_can_delete_single_submission() {
        let (app, app_state, _tmp) = make_test_app_with_storage().await;
        let data = setup_data(app_state.db()).await;

        // Ensure present
        assert!(SubmissionEntity::find_by_id(data.sub1.id)
            .one(app_state.db())
            .await
            .unwrap()
            .is_some());

        let (token, _) = generate_jwt(data.lecturer.id, data.lecturer.admin);
        let uri = format!(
            "/api/modules/{}/assignments/{}/submissions/{}",
            data.module.id, data.assignment.id, data.sub1.id
        );

        let req = Request::builder()
            .method("DELETE")
            .uri(&uri)
            .header("Authorization", format!("Bearer {}", token))
            .body(Body::empty())
            .unwrap();

        let resp = app.clone().oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);

        let body = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
        let v: Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(v["success"], true);

        // gone
        assert!(SubmissionEntity::find_by_id(data.sub1.id)
            .one(app_state.db())
            .await
            .unwrap()
            .is_none());
    }

    #[tokio::test]
    #[serial]
    async fn assistant_can_delete_single_submission() {
        let (app, app_state, _tmp) = make_test_app_with_storage().await;
        let data = setup_data(app_state.db()).await;

        let (token, _) = generate_jwt(data.assistant.id, data.assistant.admin);
        let uri = format!(
            "/api/modules/{}/assignments/{}/submissions/{}",
            data.module.id, data.assignment.id, data.sub2.id
        );

        let req = Request::builder()
            .method("DELETE")
            .uri(&uri)
            .header("Authorization", format!("Bearer {}", token))
            .body(Body::empty())
            .unwrap();

        let resp = app.clone().oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);

        // gone
        assert!(SubmissionEntity::find_by_id(data.sub2.id)
            .one(app_state.db())
            .await
            .unwrap()
            .is_none());
    }

    #[tokio::test]
    #[serial]
    async fn tutor_cannot_delete_single_submission_forbidden() {
        let (app, app_state, _tmp) = make_test_app_with_storage().await;
        let data = setup_data(app_state.db()).await;

        let (token, _) = generate_jwt(data.tutor.id, data.tutor.admin);
        let uri = format!(
            "/api/modules/{}/assignments/{}/submissions/{}",
            data.module.id, data.assignment.id, data.sub1.id
        );

        let req = Request::builder()
            .method("DELETE")
            .uri(&uri)
            .header("Authorization", format!("Bearer {}", token))
            .body(Body::empty())
            .unwrap();

        let resp = app.clone().oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::FORBIDDEN);

        // still there
        assert!(SubmissionEntity::find_by_id(data.sub1.id)
            .one(app_state.db())
            .await
            .unwrap()
            .is_some());
    }

    #[tokio::test]
    #[serial]
    async fn student_cannot_delete_single_submission_forbidden() {
        let (app, app_state, _tmp) = make_test_app_with_storage().await;
        let data = setup_data(app_state.db()).await;

        let (token, _) = generate_jwt(data.student.id, data.student.admin);
        let uri = format!(
            "/api/modules/{}/assignments/{}/submissions/{}",
            data.module.id, data.assignment.id, data.sub1.id
        );

        let req = Request::builder()
            .method("DELETE")
            .uri(&uri)
            .header("Authorization", format!("Bearer {}", token))
            .body(Body::empty())
            .unwrap();

        let resp = app.clone().oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::FORBIDDEN);

        // still there
        assert!(SubmissionEntity::find_by_id(data.sub1.id)
            .one(app_state.db())
            .await
            .unwrap()
            .is_some());
    }

    #[tokio::test]
    #[serial]
    async fn single_delete_404_when_submission_not_in_assignment() {
        let (app, app_state, _tmp) = make_test_app_with_storage().await;
        let data = setup_data(app_state.db()).await;

        // call with wrong assignment id
        let (token, _) = generate_jwt(data.lecturer.id, data.lecturer.admin);
        let uri = format!(
            "/api/modules/{}/assignments/{}/submissions/{}",
            data.module.id, data.other_assignment.id, data.sub1.id
        );

        let req = Request::builder()
            .method("DELETE")
            .uri(&uri)
            .header("Authorization", format!("Bearer {}", token))
            .body(Body::empty())
            .unwrap();

        let resp = app.clone().oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::NOT_FOUND);

        // still there
        assert!(SubmissionEntity::find_by_id(data.sub1.id)
            .one(app_state.db())
            .await
            .unwrap()
            .is_some());
    }

    // ---------------- BULK DELETE: /submissions/bulk ----------------

    #[tokio::test]
    #[serial]
    async fn bulk_delete_all_ok_as_lecturer() {
        let (app, app_state, _tmp) = make_test_app_with_storage().await;
        let mut data = setup_data(app_state.db()).await;

        // create a third one to delete
        let sub3 = SubmissionModel::save_file(
            app_state.db(),
            data.assignment.id,
            data.student.id,
            3,
            10,
            10,
            false,
            "three.txt",
            "h3",
            b"z",
        )
        .await
        .unwrap();

        let (token, _) = generate_jwt(data.lecturer.id, data.lecturer.admin);
        let uri = format!(
            "/api/modules/{}/assignments/{}/submissions/bulk",
            data.module.id, data.assignment.id
        );

        let req = Request::builder()
            .method("DELETE")
            .uri(&uri)
            .header("Authorization", format!("Bearer {}", token))
            .header("content-type", "application/json")
            .body(Body::from(
                json!({ "submission_ids": [data.sub1.id, data.sub2.id, sub3.id] }).to_string(),
            ))
            .unwrap();

        let resp = app.clone().oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);

        let b = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
        let v: Value = serde_json::from_slice(&b).unwrap();
        assert_eq!(v["success"], true);
        assert_eq!(v["data"]["deleted"], 3);
        assert!(v["data"]["failed"].as_array().unwrap().is_empty());

        for id in [data.sub1.id, data.sub2.id, sub3.id] {
            assert!(SubmissionEntity::find_by_id(id)
                .one(app_state.db())
                .await
                .unwrap()
                .is_none());
        }

        // refresh sub1/sub2 in struct not to use stale ids in later tests
        data.sub1 = SubmissionModel { id: -1, ..data.sub1 };
        data.sub2 = SubmissionModel { id: -1, ..data.sub2 };
        let _ = data; // silence warning
    }

    #[tokio::test]
    #[serial]
    async fn bulk_delete_mixed_some_fail() {
        let (app, app_state, _tmp) = make_test_app_with_storage().await;
        let data = setup_data(app_state.db()).await;

        // create a submission in OTHER assignment to force a per-ID failure
        let wrong_sub = SubmissionModel::save_file(
            app_state.db(),
            data.other_assignment.id,
            data.student.id,
            99,
            10,
            10,
            false,
            "other.txt",
            "h99",
            b"q",
        )
        .await
        .unwrap();

        let (token, _) = generate_jwt(data.lecturer.id, data.lecturer.admin);
        let uri = format!(
            "/api/modules/{}/assignments/{}/submissions/bulk",
            data.module.id, data.assignment.id
        );

        let req = Request::builder()
            .method("DELETE")
            .uri(&uri)
            .header("Authorization", format!("Bearer {}", token))
            .header("content-type", "application/json")
            .body(Body::from(
                json!({ "submission_ids": [data.sub1.id, wrong_sub.id] }).to_string(),
            ))
            .unwrap();

        let resp = app.clone().oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);

        let b = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
        let v: Value = serde_json::from_slice(&b).unwrap();
        assert_eq!(v["success"], true);
        assert_eq!(v["data"]["deleted"], 1);

        let failed = v["data"]["failed"].as_array().unwrap();
        assert_eq!(failed.len(), 1);
        assert_eq!(failed[0]["id"], wrong_sub.id);

        // sub1 removed, wrong_sub remains (belongs to other assignment)
        assert!(SubmissionEntity::find_by_id(data.sub1.id)
            .one(app_state.db())
            .await
            .unwrap()
            .is_none());
        assert!(SubmissionEntity::find_by_id(wrong_sub.id)
            .one(app_state.db())
            .await
            .unwrap()
            .is_some());
    }

    #[tokio::test]
    #[serial]
    async fn bulk_delete_empty_ids_400() {
        let (app, app_state, _tmp) = make_test_app_with_storage().await;
        let data = setup_data(app_state.db()).await;

        let (token, _) = generate_jwt(data.lecturer.id, data.lecturer.admin);
        let uri = format!(
            "/api/modules/{}/assignments/{}/submissions/bulk",
            data.module.id, data.assignment.id
        );

        let req = Request::builder()
            .method("DELETE")
            .uri(&uri)
            .header("Authorization", format!("Bearer {}", token))
            .header("content-type", "application/json")
            .body(Body::from(json!({ "submission_ids": [] }).to_string()))
            .unwrap();

        let resp = app.clone().oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    #[serial]
    async fn tutor_cannot_bulk_delete_forbidden() {
        let (app, app_state, _tmp) = make_test_app_with_storage().await;
        let data = setup_data(app_state.db()).await;

        let (token, _) = generate_jwt(data.tutor.id, data.tutor.admin);
        let uri = format!(
            "/api/modules/{}/assignments/{}/submissions/bulk",
            data.module.id, data.assignment.id
        );

        let req = Request::builder()
            .method("DELETE")
            .uri(&uri)
            .header("Authorization", format!("Bearer {}", token))
            .header("content-type", "application/json")
            .body(Body::from(json!({ "submission_ids": [data.sub1.id] }).to_string()))
            .unwrap();

        let resp = app.clone().oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::FORBIDDEN);

        // still there
        assert!(SubmissionEntity::find_by_id(data.sub1.id)
            .one(app_state.db())
            .await
            .unwrap()
            .is_some());
    }

    #[tokio::test]
    #[serial]
    async fn student_cannot_bulk_delete_forbidden() {
        let (app, app_state, _tmp) = make_test_app_with_storage().await;
        let data = setup_data(app_state.db()).await;

        let (token, _) = generate_jwt(data.student.id, data.student.admin);
        let uri = format!(
            "/api/modules/{}/assignments/{}/submissions/bulk",
            data.module.id, data.assignment.id
        );

        let req = Request::builder()
            .method("DELETE")
            .uri(&uri)
            .header("Authorization", format!("Bearer {}", token))
            .header("content-type", "application/json")
            .body(Body::from(json!({ "submission_ids": [data.sub2.id] }).to_string()))
            .unwrap();

        let resp = app.clone().oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::FORBIDDEN);

        // still there
        assert!(SubmissionEntity::find_by_id(data.sub2.id)
            .one(app_state.db())
            .await
            .unwrap()
            .is_some());
    }
}
