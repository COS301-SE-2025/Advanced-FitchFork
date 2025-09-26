#[cfg(test)]
mod tests {
    use crate::helpers::app::make_test_app_with_storage;
    use api::auth::generate_jwt;
    use axum::{
        body::Body,
        http::{Request, StatusCode},
    };
    use chrono::{TimeZone, Utc};
    use db::models::{
        assignment::{AssignmentType, Model as AssignmentModel},
        assignment_file::{FileType, Model as AssignmentFileModel},
        assignment_memo_output::Model as AssignmentMemoOutputModel,
        assignment_submission::Model as AssignmentSubmissionModel,
        assignment_task::{Model as AssignmentTaskModel, TaskType},
        module::{ActiveModel as ModuleActiveModel, Model as ModuleModel},
        user::Model as UserModel,
        user_module_role::{Model as UserModuleRoleModel, Role},
    };
    use sea_orm::{ActiveModelTrait, EntityTrait, Set};
    use serde_json::{Value, json};
    use tower::ServiceExt;

    struct TestData {
        admin_user: UserModel,
        lecturer_user: UserModel,
        student_user: UserModel,
        forbidden_user: UserModel,
        module: ModuleModel,
        empty_module: ModuleModel,
        assignments: Vec<AssignmentModel>,
        dummy_module_id: i64,
        file_id: i64,
        task_id: i64,
        memo_output_id: i64,
        submission_id: i64,
    }

    async fn setup_test_data(db: &sea_orm::DatabaseConnection) -> TestData {
        let module = ModuleModel::create(db, "COS101", 2024, Some("Test Module"), 16)
            .await
            .unwrap();
        let empty_module = ModuleModel::create(db, "EMPTY101", 2024, Some("Empty Module"), 16)
            .await
            .unwrap();
        let admin_user = UserModel::create(db, "admin1", "admin1@test.com", "password", true)
            .await
            .unwrap();
        let lecturer_user =
            UserModel::create(db, "lecturer1", "lecturer1@test.com", "password1", false)
                .await
                .unwrap();
        let student_user =
            UserModel::create(db, "student1", "student1@test.com", "password2", false)
                .await
                .unwrap();
        let forbidden_user =
            UserModel::create(db, "forbidden", "forbidden@test.com", "password3", false)
                .await
                .unwrap();
        UserModuleRoleModel::assign_user_to_module(db, lecturer_user.id, module.id, Role::Lecturer)
            .await
            .unwrap();
        UserModuleRoleModel::assign_user_to_module(
            db,
            lecturer_user.id,
            empty_module.id,
            Role::Lecturer,
        )
        .await
        .unwrap();
        UserModuleRoleModel::assign_user_to_module(db, student_user.id, module.id, Role::Student)
            .await
            .unwrap();
        let dummy_module = ModuleActiveModel {
            id: Set(9999),
            code: Set("DUMMY9999".to_string()),
            year: Set(2024),
            description: Set(Some("Dummy module for not found test".to_string())),
            created_at: Set(Utc::now()),
            updated_at: Set(Utc::now()),
            ..Default::default()
        }
        .insert(db)
        .await
        .unwrap();
        UserModuleRoleModel::assign_user_to_module(
            db,
            lecturer_user.id,
            dummy_module.id,
            Role::Lecturer,
        )
        .await
        .unwrap();
        let a1 = AssignmentModel::create(
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
        let a2 = AssignmentModel::create(
            db,
            module.id,
            "Assignment 2",
            Some("Desc 2"),
            AssignmentType::Practical,
            Utc.with_ymd_and_hms(2024, 2, 1, 0, 0, 0).unwrap(),
            Utc.with_ymd_and_hms(2024, 2, 28, 23, 59, 59).unwrap(),
        )
        .await
        .unwrap();
        let a3 = AssignmentModel::create(
            db,
            module.id,
            "Assignment 3",
            Some("Desc 3"),
            AssignmentType::Assignment,
            Utc.with_ymd_and_hms(2024, 3, 1, 0, 0, 0).unwrap(),
            Utc.with_ymd_and_hms(2024, 3, 31, 23, 59, 59).unwrap(),
        )
        .await
        .unwrap();
        let file = AssignmentFileModel::save_file(
            db,
            a1.id,
            module.id,
            FileType::Spec,
            "spec.txt",
            b"spec",
        )
        .await
        .unwrap();
        let task =
            AssignmentTaskModel::create(db, a1.id, 1, "Task 1", "echo Hello", TaskType::Normal)
                .await
                .unwrap();
        let memo_output =
            AssignmentMemoOutputModel::save_file(db, a1.id, task.id, "memo.txt", b"memo")
                .await
                .unwrap();
        let submission = AssignmentSubmissionModel::save_file(
            db,
            a1.id,
            student_user.id,
            1,
            10.0,
            10.0,
            false,
            "sub.txt",
            "hash123#",
            b"sub",
        )
        .await
        .unwrap();

        TestData {
            admin_user,
            lecturer_user,
            student_user,
            forbidden_user,
            module,
            empty_module,
            assignments: vec![a1, a2, a3],
            dummy_module_id: dummy_module.id,
            file_id: file.id,
            task_id: task.id,
            memo_output_id: memo_output.id,
            submission_id: submission.id,
        }
    }

    #[tokio::test]
    async fn test_delete_assignment_success_as_lecturer() {
        let (app, app_state, _tmp) = make_test_app_with_storage().await;
        let data = setup_test_data(app_state.db()).await;

        let (token, _) = generate_jwt(data.lecturer_user.id, data.lecturer_user.admin);
        let uri = format!(
            "/api/modules/{}/assignments/{}",
            data.module.id, data.assignments[0].id
        );
        let req = Request::builder()
            .method("DELETE")
            .uri(&uri)
            .header("Authorization", format!("Bearer {}", token))
            .body(Body::empty())
            .unwrap();

        let response = app.clone().oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let json: Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(json["success"], true);
        let found = db::models::assignment::Entity::find_by_id(data.assignments[0].id)
            .one(app_state.db())
            .await
            .unwrap();
        assert!(found.is_none());
    }

    #[tokio::test]
    async fn test_delete_assignment_success_as_admin() {
        let (app, app_state, _tmp) = make_test_app_with_storage().await;
        let data = setup_test_data(app_state.db()).await;

        let (token, _) = generate_jwt(data.admin_user.id, data.admin_user.admin);
        let uri = format!(
            "/api/modules/{}/assignments/{}",
            data.module.id, data.assignments[1].id
        );
        let req = Request::builder()
            .method("DELETE")
            .uri(&uri)
            .header("Authorization", format!("Bearer {}", token))
            .body(Body::empty())
            .unwrap();

        let response = app.clone().oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let json: Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(json["success"], true);
        let found = db::models::assignment::Entity::find_by_id(data.assignments[1].id)
            .one(app_state.db())
            .await
            .unwrap();
        assert!(found.is_none());
    }

    #[tokio::test]
    async fn test_delete_assignment_forbidden_for_student() {
        let (app, app_state, _tmp) = make_test_app_with_storage().await;
        let data = setup_test_data(app_state.db()).await;

        let (token, _) = generate_jwt(data.student_user.id, data.student_user.admin);
        let uri = format!(
            "/api/modules/{}/assignments/{}",
            data.module.id, data.assignments[0].id
        );
        let req = Request::builder()
            .method("DELETE")
            .uri(&uri)
            .header("Authorization", format!("Bearer {}", token))
            .body(Body::empty())
            .unwrap();

        let response = app.clone().oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::FORBIDDEN);
    }

    #[tokio::test]
    async fn test_delete_assignment_forbidden_for_unassigned_user() {
        let (app, app_state, _tmp) = make_test_app_with_storage().await;
        let data = setup_test_data(app_state.db()).await;

        let (token, _) = generate_jwt(data.forbidden_user.id, data.forbidden_user.admin);
        let uri = format!(
            "/api/modules/{}/assignments/{}",
            data.module.id, data.assignments[0].id
        );
        let req = Request::builder()
            .method("DELETE")
            .uri(&uri)
            .header("Authorization", format!("Bearer {}", token))
            .body(Body::empty())
            .unwrap();

        let response = app.clone().oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::FORBIDDEN);
    }

    #[tokio::test]
    async fn test_delete_assignment_unauthorized() {
        let (app, app_state, _tmp) = make_test_app_with_storage().await;
        let data = setup_test_data(app_state.db()).await;

        let uri = format!(
            "/api/modules/{}/assignments/{}",
            data.module.id, data.assignments[0].id
        );
        let req = Request::builder()
            .method("DELETE")
            .uri(&uri)
            .body(Body::empty())
            .unwrap();

        let response = app.clone().oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn test_delete_assignment_not_found() {
        let (app, app_state, _tmp) = make_test_app_with_storage().await;
        let data = setup_test_data(app_state.db()).await;

        let (token, _) = generate_jwt(data.lecturer_user.id, data.lecturer_user.admin);
        let uri = format!("/api/modules/{}/assignments/9999", data.module.id);
        let req = Request::builder()
            .method("DELETE")
            .uri(&uri)
            .header("Authorization", format!("Bearer {}", token))
            .body(Body::empty())
            .unwrap();

        let response = app.clone().oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn test_delete_assignment_wrong_module() {
        let (app, app_state, _tmp) = make_test_app_with_storage().await;
        let data = setup_test_data(app_state.db()).await;

        let (token, _) = generate_jwt(data.lecturer_user.id, data.lecturer_user.admin);
        let uri = format!(
            "/api/modules/{}/assignments/{}",
            data.empty_module.id, data.assignments[0].id
        );
        let req = Request::builder()
            .method("DELETE")
            .uri(&uri)
            .header("Authorization", format!("Bearer {}", token))
            .body(Body::empty())
            .unwrap();

        let response = app.clone().oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn test_delete_assignment_module_not_found() {
        let (app, app_state, _tmp) = make_test_app_with_storage().await;
        let data = setup_test_data(app_state.db()).await;

        let (token, _) = generate_jwt(data.lecturer_user.id, data.lecturer_user.admin);
        let uri = format!(
            "/api/modules/{}/assignments/{}",
            data.dummy_module_id, data.assignments[0].id
        );
        let req = Request::builder()
            .method("DELETE")
            .uri(&uri)
            .header("Authorization", format!("Bearer {}", token))
            .body(Body::empty())
            .unwrap();

        let response = app.clone().oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn test_delete_assignment_with_related_data_cleanup() {
        let (app, app_state, _tmp) = make_test_app_with_storage().await;
        let db = app_state.db();
        let data = setup_test_data(db).await;

        let (token, _) = generate_jwt(data.lecturer_user.id, data.lecturer_user.admin);
        let uri = format!(
            "/api/modules/{}/assignments/{}",
            data.module.id, data.assignments[0].id
        );
        let req = Request::builder()
            .method("DELETE")
            .uri(&uri)
            .header("Authorization", format!("Bearer {}", token))
            .body(Body::empty())
            .unwrap();

        let response = app.clone().oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);

        let found = db::models::assignment::Entity::find_by_id(data.assignments[0].id)
            .one(db)
            .await
            .unwrap();
        assert!(found.is_none());
        let file = db::models::assignment_file::Entity::find_by_id(data.file_id)
            .one(db)
            .await
            .unwrap();
        assert!(file.is_none());
        let task = db::models::assignment_task::Entity::find_by_id(data.task_id)
            .one(db)
            .await
            .unwrap();
        assert!(task.is_none());
        let memo = db::models::assignment_memo_output::Entity::find_by_id(data.memo_output_id)
            .one(db)
            .await
            .unwrap();
        assert!(memo.is_none());
        let sub = db::models::assignment_submission::Entity::find_by_id(data.submission_id)
            .one(db)
            .await
            .unwrap();
        assert!(sub.is_none());
    }

    #[tokio::test]
    async fn test_delete_assignment_already_deleted() {
        let (app, app_state, _tmp) = make_test_app_with_storage().await;
        let data = setup_test_data(app_state.db()).await;

        let (token, _) = generate_jwt(data.lecturer_user.id, data.lecturer_user.admin);
        let uri = format!(
            "/api/modules/{}/assignments/{}",
            data.module.id, data.assignments[0].id
        );
        let req = Request::builder()
            .method("DELETE")
            .uri(&uri)
            .header("Authorization", format!("Bearer {}", token))
            .body(Body::empty())
            .unwrap();

        let response = app.clone().oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);

        let req2 = Request::builder()
            .method("DELETE")
            .uri(&uri)
            .header("Authorization", format!("Bearer {}", token))
            .body(Body::empty())
            .unwrap();

        let response2 = app.clone().oneshot(req2).await.unwrap();
        assert_eq!(response2.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn test_delete_assignment_cross_module_forbidden() {
        let (app, app_state, _tmp) = make_test_app_with_storage().await;
        let data = setup_test_data(app_state.db()).await;

        let (token, _) = generate_jwt(data.lecturer_user.id, data.lecturer_user.admin);
        let uri = format!(
            "/api/modules/{}/assignments/{}",
            data.empty_module.id, data.assignments[0].id
        );
        let req = Request::builder()
            .method("DELETE")
            .uri(&uri)
            .header("Authorization", format!("Bearer {}", token))
            .body(Body::empty())
            .unwrap();

        let response = app.clone().oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    /// Test Case: Successful Bulk Delete by Lecturer
    #[tokio::test]
    async fn test_bulk_delete_assignments_success_lecturer() {
        let (app, app_state, _tmp) = make_test_app_with_storage().await;
        let data = setup_test_data(app_state.db()).await;

        let (token, _) = generate_jwt(data.lecturer_user.id, data.lecturer_user.admin);
        let uri = format!("/api/modules/{}/assignments/bulk", data.module.id);

        let ids_to_delete = vec![data.assignments[0].id, data.assignments[1].id];
        let req_body = json!({ "assignment_ids": ids_to_delete });

        let req = Request::builder()
            .method("DELETE")
            .uri(&uri)
            .header("Authorization", format!("Bearer {}", token))
            .header("Content-Type", "application/json")
            .body(Body::from(serde_json::to_string(&req_body).unwrap()))
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let json: Value = serde_json::from_slice(&body).unwrap();

        assert_eq!(json["success"], true);
        assert_eq!(json["message"], "Deleted 2/2 assignments");
        assert_eq!(json["data"]["deleted"], 2);
        assert!(json["data"]["failed"].as_array().unwrap().is_empty());
    }

    /// Test Case: Successful Bulk Delete by Admin
    #[tokio::test]
    async fn test_bulk_delete_assignments_success_admin() {
        let (app, app_state, _tmp) = make_test_app_with_storage().await;
        let data = setup_test_data(app_state.db()).await;

        let (token, _) = generate_jwt(data.admin_user.id, data.admin_user.admin);
        let uri = format!("/api/modules/{}/assignments/bulk", data.module.id);

        let ids_to_delete = vec![data.assignments[2].id];
        let req_body = json!({ "assignment_ids": ids_to_delete });

        let req = Request::builder()
            .method("DELETE")
            .uri(&uri)
            .header("Authorization", format!("Bearer {}", token))
            .header("Content-Type", "application/json")
            .body(Body::from(serde_json::to_string(&req_body).unwrap()))
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let json: Value = serde_json::from_slice(&body).unwrap();

        assert_eq!(json["success"], true);
        assert_eq!(json["message"], "Deleted 1/1 assignments");
        assert_eq!(json["data"]["deleted"], 1);
    }

    /// Test Case: Mixed Success/Failure with Invalid IDs
    #[tokio::test]
    async fn test_bulk_delete_assignments_mixed_results() {
        let (app, app_state, _tmp) = make_test_app_with_storage().await;
        let data = setup_test_data(app_state.db()).await;

        let (token, _) = generate_jwt(data.lecturer_user.id, data.lecturer_user.admin);
        let uri = format!("/api/modules/{}/assignments/bulk", data.module.id);

        let ids_to_delete = vec![data.assignments[0].id, 9999, data.assignments[2].id];
        let req_body = json!({ "assignment_ids": ids_to_delete });

        let req = Request::builder()
            .method("DELETE")
            .uri(&uri)
            .header("Authorization", format!("Bearer {}", token))
            .header("Content-Type", "application/json")
            .body(Body::from(serde_json::to_string(&req_body).unwrap()))
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let json: Value = serde_json::from_slice(&body).unwrap();

        assert_eq!(json["success"], true);
        assert_eq!(json["message"], "Deleted 2/3 assignments");
        assert_eq!(json["data"]["deleted"], 2);

        let failed = json["data"]["failed"].as_array().unwrap();
        assert_eq!(failed.len(), 1);
        assert_eq!(failed[0]["id"], 9999);
        assert!(failed[0]["error"].as_str().unwrap().contains("not found"));
    }

    /// Test Case: Forbidden for Student
    #[tokio::test]
    async fn test_bulk_delete_assignments_forbidden_student() {
        let (app, app_state, _tmp) = make_test_app_with_storage().await;
        let data = setup_test_data(app_state.db()).await;

        let (token, _) = generate_jwt(data.student_user.id, data.student_user.admin);
        let uri = format!("/api/modules/{}/assignments/bulk", data.module.id);
        let req_body = json!({ "assignment_ids": [data.assignments[0].id] });
        let req = Request::builder()
            .method("DELETE")
            .uri(&uri)
            .header("Authorization", format!("Bearer {}", token))
            .header("Content-Type", "application/json")
            .body(Body::from(serde_json::to_string(&req_body).unwrap()))
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::FORBIDDEN);
    }

    /// Test Case: Empty Assignment IDs
    #[tokio::test]
    async fn test_bulk_delete_assignments_empty_ids() {
        let (app, app_state, _tmp) = make_test_app_with_storage().await;
        let data = setup_test_data(app_state.db()).await;

        let (token, _) = generate_jwt(data.lecturer_user.id, data.lecturer_user.admin);
        let uri = format!("/api/modules/{}/assignments/bulk", data.module.id);
        let req_body = json!({ "assignment_ids": [] });
        let req = Request::builder()
            .method("DELETE")
            .uri(&uri)
            .header("Authorization", format!("Bearer {}", token))
            .header("Content-Type", "application/json")
            .body(Body::from(serde_json::to_string(&req_body).unwrap()))
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let json: Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(json["success"], false);
        assert_eq!(json["message"], "No assignment IDs provided");
    }
}
