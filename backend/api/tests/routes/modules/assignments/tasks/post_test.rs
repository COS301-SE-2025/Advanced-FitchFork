#[cfg(test)]
mod tests {
    use crate::helpers::app::make_test_app_with_storage;
    use api::auth::generate_jwt;
    use axum::{
        body::Body as AxumBody,
        http::{Request, StatusCode, header::CONTENT_TYPE},
    };
    use chrono::{TimeZone, Utc};
    use db::models::{
        assignment::{AssignmentType, Model as AssignmentModel},
        module::Model as ModuleModel,
        user::Model as UserModel,
        user_module_role::{Model as UserModuleRoleModel, Role},
    };
    use serde_json::{Value, json};
    use serial_test::serial;
    use tower::ServiceExt;

    struct TestData {
        admin_user: UserModel,
        forbidden_user: UserModel,
        lecturer1: UserModel,
        module: ModuleModel,
        assignment: AssignmentModel,
    }

    async fn setup_test_data(db: &sea_orm::DatabaseConnection) -> TestData {
        let module = ModuleModel::create(db, "TASK101", 2024, Some("Test Task Module"), 16)
            .await
            .expect("Failed to create test module");
        let admin_user =
            UserModel::create(db, "task_admin", "task_admin@test.com", "password", true)
                .await
                .expect("Failed to create admin user");
        let forbidden_user = UserModel::create(
            db,
            "task_unauthed",
            "task_unauthed@test.com",
            "password",
            false,
        )
        .await
        .expect("Failed to create forbidden user");
        let lecturer1 = UserModel::create(
            db,
            "task_lecturer1",
            "task_lecturer1@test.com",
            "password1",
            false,
        )
        .await
        .expect("Failed to create lecturer1");
        UserModuleRoleModel::assign_user_to_module(db, lecturer1.id, module.id, Role::Lecturer)
            .await
            .expect("Failed to assign lecturer1 to module");
        let assignment = AssignmentModel::create(
            db,
            module.id,
            "Test Assignment",
            Some("Description"),
            AssignmentType::Assignment,
            Utc.with_ymd_and_hms(2024, 1, 1, 0, 0, 0).unwrap(),
            Utc.with_ymd_and_hms(2024, 12, 31, 23, 59, 59).unwrap(),
        )
        .await
        .expect("Failed to create test assignment");

        TestData {
            admin_user,
            forbidden_user,
            lecturer1,
            module,
            assignment,
        }
    }

    /// Test Case: Successful Creation of a New Task as Admin
    #[tokio::test]
    #[serial]
    async fn test_create_task_success_as_admin() {
        let (app, app_state, _tmp) = make_test_app_with_storage().await;
        let data = setup_test_data(app_state.db()).await;

        let (token, _) = generate_jwt(data.admin_user.id, data.admin_user.admin);
        let payload = json!({
            "task_number": 1,
            "name": "Hello Task",
            "command": "echo 'Hello, World!'"
        });
        let uri = format!(
            "/api/modules/{}/assignments/{}/tasks",
            data.module.id, data.assignment.id
        );
        let req = Request::builder()
            .method("POST")
            .uri(&uri)
            .header("Authorization", format!("Bearer {}", token))
            .header(CONTENT_TYPE, "application/json")
            .body(AxumBody::from(serde_json::to_vec(&payload).unwrap()))
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::CREATED);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let json: Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(json["success"], true);
        assert_eq!(json["message"], "Task created successfully");
        let task_data = &json["data"];

        assert_eq!(task_data["task_number"], 1);
        assert_eq!(task_data["name"], "Hello Task");
        assert_eq!(task_data["command"], "echo 'Hello, World!'");
        assert!(task_data["id"].as_i64().is_some());
        assert!(task_data["created_at"].as_str().is_some());
        assert!(task_data["updated_at"].as_str().is_some());
    }

    /// Test Case: Successful Creation of a New Task as Lecturer
    #[tokio::test]
    #[serial]
    async fn test_create_task_success_as_lecturer() {
        let (app, app_state, _tmp) = make_test_app_with_storage().await;
        let data = setup_test_data(app_state.db()).await;

        let (token, _) = generate_jwt(data.lecturer1.id, data.lecturer1.admin);
        let payload = json!({
            "task_number": 2,
            "name": "List Files",
            "command": "ls -la"
        });
        let uri = format!(
            "/api/modules/{}/assignments/{}/tasks",
            data.module.id, data.assignment.id
        );
        let req = Request::builder()
            .method("POST")
            .uri(&uri)
            .header("Authorization", format!("Bearer {}", token))
            .header(CONTENT_TYPE, "application/json")
            .body(AxumBody::from(serde_json::to_vec(&payload).unwrap()))
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::CREATED);
    }

    /// Test Case: Creating Task for Non-Existent Assignment
    #[tokio::test]
    #[serial]
    async fn test_create_task_assignment_not_found() {
        let (app, app_state, _tmp) = make_test_app_with_storage().await;
        let data = setup_test_data(app_state.db()).await;

        let (token, _) = generate_jwt(data.admin_user.id, data.admin_user.admin);
        let payload = json!({
            "task_number": 1,
            "name": "Should Fail",
            "command": "echo 'Should fail'"
        });
        let uri = format!("/api/modules/{}/assignments/{}/tasks", data.module.id, 9999);
        let req = Request::builder()
            .method("POST")
            .uri(&uri)
            .header("Authorization", format!("Bearer {}", token))
            .header(CONTENT_TYPE, "application/json")
            .body(AxumBody::from(serde_json::to_vec(&payload).unwrap()))
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::NOT_FOUND);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let json: Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(json["success"], false);
        assert_eq!(json["message"], "Assignment 9999 in Module 1 not found.");
    }

    /// Test Case: Creating Task for Non-Existent Module (via path mismatch)
    #[tokio::test]
    #[serial]
    async fn test_create_task_module_assignment_mismatch() {
        let (app, app_state, _tmp) = make_test_app_with_storage().await;
        let data = setup_test_data(app_state.db()).await;

        let module2 = ModuleModel::create(
            app_state.db(),
            "MISMTCH101",
            2024,
            Some("Mismatch Module"),
            16,
        )
        .await
        .expect("Failed to create second module");
        let assignment2_in_module2 = AssignmentModel::create(
            app_state.db(),
            module2.id,
            "Assignment in Module 2",
            Some("Desc"),
            AssignmentType::Assignment,
            Utc.with_ymd_and_hms(2024, 1, 1, 0, 0, 0).unwrap(),
            Utc.with_ymd_and_hms(2024, 12, 31, 23, 59, 59).unwrap(),
        )
        .await
        .expect("Failed to create assignment in second module");

        let (token, _) = generate_jwt(data.admin_user.id, data.admin_user.admin);
        let payload = json!({
            "task_number": 1,
            "name": "Mismatch",
            "command": "echo 'Mismatch'"
        });
        let uri = format!(
            "/api/modules/{}/assignments/{}/tasks",
            data.module.id, assignment2_in_module2.id
        );
        let req = Request::builder()
            .method("POST")
            .uri(&uri)
            .header("Authorization", format!("Bearer {}", token))
            .header(CONTENT_TYPE, "application/json")
            .body(AxumBody::from(serde_json::to_vec(&payload).unwrap()))
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::NOT_FOUND);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let json: Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(json["success"], false);
        assert_eq!(json["message"], "Assignment 2 in Module 1 not found.");
    }

    /// Test Case: Forbidden Access to Create Task
    #[tokio::test]
    #[serial]
    async fn test_create_task_forbidden() {
        let (app, app_state, _tmp) = make_test_app_with_storage().await;
        let data = setup_test_data(app_state.db()).await;

        let (token, _) = generate_jwt(data.forbidden_user.id, data.forbidden_user.admin);

        let payload = json!({
            "task_number": 1,
            "name": "Forbidden",
            "command": "echo 'Forbidden'"
        });
        let uri = format!(
            "/api/modules/{}/assignments/{}/tasks",
            data.module.id, data.assignment.id
        );
        let req = Request::builder()
            .method("POST")
            .uri(&uri)
            .header("Authorization", format!("Bearer {}", token))
            .header(CONTENT_TYPE, "application/json")
            .body(AxumBody::from(serde_json::to_vec(&payload).unwrap()))
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::FORBIDDEN);
    }

    /// Test Case: Invalid task_number (Zero)
    #[tokio::test]
    #[serial]
    async fn test_create_task_invalid_task_number_zero() {
        let (app, app_state, _tmp) = make_test_app_with_storage().await;
        let data = setup_test_data(app_state.db()).await;

        let (token, _) = generate_jwt(data.admin_user.id, data.admin_user.admin);
        let payload = json!({
            "task_number": 0,
            "name": "Invalid number",
            "command": "echo 'Invalid number'"
        });
        let uri = format!(
            "/api/modules/{}/assignments/{}/tasks",
            data.module.id, data.assignment.id
        );
        let req = Request::builder()
            .method("POST")
            .uri(&uri)
            .header("Authorization", format!("Bearer {}", token))
            .header(CONTENT_TYPE, "application/json")
            .body(AxumBody::from(serde_json::to_vec(&payload).unwrap()))
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let json: Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(json["success"], false);
        assert_eq!(json["message"], "Invalid task_number, name, or command");
    }

    /// Test Case: Invalid task_number (Negative)
    #[tokio::test]
    #[serial]
    async fn test_create_task_invalid_task_number_negative() {
        let (app, app_state, _tmp) = make_test_app_with_storage().await;
        let data = setup_test_data(app_state.db()).await;

        let (token, _) = generate_jwt(data.admin_user.id, data.admin_user.admin);
        let payload = json!({
            "task_number": -5,
            "name": "Negative number",
            "command": "echo 'Negative number'"
        });
        let uri = format!(
            "/api/modules/{}/assignments/{}/tasks",
            data.module.id, data.assignment.id
        );
        let req = Request::builder()
            .method("POST")
            .uri(&uri)
            .header("Authorization", format!("Bearer {}", token))
            .header(CONTENT_TYPE, "application/json")
            .body(AxumBody::from(serde_json::to_vec(&payload).unwrap()))
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let json: Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(json["success"], false);
        assert_eq!(json["message"], "Invalid task_number, name, or command");
    }

    /// Test Case: Invalid command (Empty String)
    #[tokio::test]
    #[serial]
    async fn test_create_task_invalid_command_empty() {
        let (app, app_state, _tmp) = make_test_app_with_storage().await;
        let data = setup_test_data(app_state.db()).await;

        let (token, _) = generate_jwt(data.admin_user.id, data.admin_user.admin);
        let payload = json!({
            "task_number": 1,
            "name": "Empty Command",
            "command": ""
        });
        let uri = format!(
            "/api/modules/{}/assignments/{}/tasks",
            data.module.id, data.assignment.id
        );
        let req = Request::builder()
            .method("POST")
            .uri(&uri)
            .header("Authorization", format!("Bearer {}", token))
            .header(CONTENT_TYPE, "application/json")
            .body(AxumBody::from(serde_json::to_vec(&payload).unwrap()))
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let json: Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(json["success"], false);
        assert_eq!(json["message"], "Invalid task_number, name, or command");
    }

    /// Test Case: Invalid command (Whitespace Only)
    #[tokio::test]
    #[serial]
    async fn test_create_task_invalid_command_whitespace() {
        let (app, app_state, _tmp) = make_test_app_with_storage().await;
        let data = setup_test_data(app_state.db()).await;

        let (token, _) = generate_jwt(data.admin_user.id, data.admin_user.admin);
        let payload = json!({
            "task_number": 1,
            "name": "Whitespace Command",
            "command": "   \n\t  "
        });
        let uri = format!(
            "/api/modules/{}/assignments/{}/tasks",
            data.module.id, data.assignment.id
        );
        let req = Request::builder()
            .method("POST")
            .uri(&uri)
            .header("Authorization", format!("Bearer {}", token))
            .header(CONTENT_TYPE, "application/json")
            .body(AxumBody::from(serde_json::to_vec(&payload).unwrap()))
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let json: Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(json["success"], false);
        assert_eq!(json["message"], "Invalid task_number, name, or command");
    }

    /// Test Case: Non-Unique task_number (Duplicate)
    #[tokio::test]
    #[serial]
    async fn test_create_task_duplicate_task_number() {
        let (app, app_state, _tmp) = make_test_app_with_storage().await;
        let data = setup_test_data(app_state.db()).await;

        let (token, _) = generate_jwt(data.admin_user.id, data.admin_user.admin);
        let initial_payload = json!({
            "task_number": 1,
            "name": "First Task",
            "command": "echo 'First Task'"
        });
        let uri = format!(
            "/api/modules/{}/assignments/{}/tasks",
            data.module.id, data.assignment.id
        );
        let req1 = Request::builder()
            .method("POST")
            .uri(&uri)
            .header("Authorization", format!("Bearer {}", token))
            .header(CONTENT_TYPE, "application/json")
            .body(AxumBody::from(
                serde_json::to_vec(&initial_payload).unwrap(),
            ))
            .unwrap();

        let response1 = app.clone().oneshot(req1).await.unwrap();
        assert_eq!(response1.status(), StatusCode::CREATED);

        let duplicate_payload = json!({
            "task_number": 1,
            "name": "Duplicate Task",
            "command": "echo 'Duplicate Task'"
        });
        let req2 = Request::builder()
            .method("POST")
            .uri(&uri)
            .header("Authorization", format!("Bearer {}", token))
            .header(CONTENT_TYPE, "application/json")
            .body(AxumBody::from(
                serde_json::to_vec(&duplicate_payload).unwrap(),
            ))
            .unwrap();

        let response2 = app.oneshot(req2).await.unwrap();
        assert_eq!(response2.status(), StatusCode::UNPROCESSABLE_ENTITY);

        let body = axum::body::to_bytes(response2.into_body(), usize::MAX)
            .await
            .unwrap();
        let json: Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(json["success"], false);
        assert_eq!(json["message"], "task_number must be unique");
    }
}
