#[cfg(test)]
mod tests {
    use db::{
        test_utils::setup_test_db,
        models::{
            user::Model as UserModel,
            module::Model as ModuleModel,
            assignment::{Model as AssignmentModel, AssignmentType},
            assignment_task::Model as AssignmentTaskModel,
            user_module_role::{Model as UserModuleRoleModel, Role},
        },
    };
    use axum::{
        body::Body as AxumBody,
        http::{Request, StatusCode, header::CONTENT_TYPE},
    };
    use tower::ServiceExt;
    use serde_json::{Value, json};
    use api::auth::generate_jwt;
    use crate::test_helpers::make_app;
    use chrono::{Utc, TimeZone};
    use tempfile::{TempDir, tempdir};
    use serial_test::serial;

    struct TestData {
        admin_user: UserModel,
        forbidden_user: UserModel,
        lecturer1: UserModel,
        module: ModuleModel,
        assignment: AssignmentModel,
        task1: AssignmentTaskModel,
    }

    async fn setup_test_data(db: &sea_orm::DatabaseConnection) -> (TestData, TempDir) {
        dotenvy::dotenv().expect("Failed to load .env");
        let temp_dir = tempdir().expect("Failed to create temporary directory");
        unsafe { std::env::set_var("ASSIGNMENT_STORAGE_ROOT", temp_dir.path().to_str().unwrap()); }

        let module = ModuleModel::create(db, "TASK101", 2024, Some("Test Task Module"), 16).await.expect("Failed to create test module");
        let admin_user = UserModel::create(db, "task_admin", "task_admin@test.com", "password", true).await.expect("Failed to create admin user");
        let forbidden_user = UserModel::create(db, "task_unauthed", "task_unauthed@test.com", "password", false).await.expect("Failed to create forbidden user");
        let lecturer1 = UserModel::create(db, "task_lecturer1", "task_lecturer1@test.com", "password1", false).await.expect("Failed to create lecturer1");
        UserModuleRoleModel::assign_user_to_module(db, lecturer1.id, module.id, Role::Lecturer).await.expect("Failed to assign lecturer1 to module");
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
        let task1 = AssignmentTaskModel::create(
             db,
             assignment.id,
             1,
             "echo 'Initial Command'",
             "Initial Task Name",
         )
         .await
         .expect("Failed to create initial task");

        (
            TestData {
                admin_user,
                forbidden_user,
                lecturer1,
                module,
                assignment,
                task1,
            },
            temp_dir,
        )
    }

    /// Test Case: Successful Update of Task Command and Name as Admin
    #[tokio::test]
    #[serial]
    async fn test_edit_task_success_as_admin() {
        let db = setup_test_db().await;
        let (data, _temp_dir) = setup_test_data(&db).await;

        let app = make_app(db.clone());
        let (token, _) = generate_jwt(data.admin_user.id, data.admin_user.admin);
        let payload = json!({"name": "Updated Task Name", "command": "echo 'Updated Command'"});
        let uri = format!("/api/modules/{}/assignments/{}/tasks/{}", data.module.id, data.assignment.id, data.task1.id);
        let req = Request::builder()
            .method("PUT")
            .uri(&uri)
            .header("Authorization", format!("Bearer {}", token))
            .header(CONTENT_TYPE, "application/json")
            .body(AxumBody::from(serde_json::to_vec(&payload).unwrap()))
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let json: Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(json["success"], true);
        assert_eq!(json["message"], "Task updated successfully");
        let task_data = &json["data"];

        assert_eq!(task_data["id"], data.task1.id);
        assert_eq!(task_data["task_number"], data.task1.task_number);
        assert_eq!(task_data["name"], "Updated Task Name");
        assert_eq!(task_data["command"], "echo 'Updated Command'");
        assert_eq!(task_data["created_at"], data.task1.created_at.to_rfc3339());

        assert!(task_data["updated_at"].as_str().is_some());
        assert!(task_data["updated_at"].as_str().unwrap() >= task_data["created_at"].as_str().unwrap());
    }

    /// Test Case: Successful Update of Task Command and Name as Lecturer
    #[tokio::test]
    #[serial]
    async fn test_edit_task_success_as_lecturer() {
        let db = setup_test_db().await;
        let (data, _temp_dir) = setup_test_data(&db).await;

        let app = make_app(db.clone());
        let (token, _) = generate_jwt(data.lecturer1.id, data.lecturer1.admin);
        let payload = json!({"name": "Lecturer Updated Name", "command": "ls -l"});
        let uri = format!("/api/modules/{}/assignments/{}/tasks/{}", data.module.id, data.assignment.id, data.task1.id);
        let req = Request::builder()
            .method("PUT")
            .uri(&uri)
            .header("Authorization", format!("Bearer {}", token))
            .header(CONTENT_TYPE, "application/json")
            .body(AxumBody::from(serde_json::to_vec(&payload).unwrap()))
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
    }

    /// Test Case: Editing Task Not Found
    #[tokio::test]
    #[serial]
    async fn test_edit_task_not_found() {
        let db = setup_test_db().await;
        let (data, _temp_dir) = setup_test_data(&db).await;

        let app = make_app(db.clone());
        let (token, _) = generate_jwt(data.admin_user.id, data.admin_user.admin);
        let payload = json!({"name": "Any Name", "command": "echo 'Any Command'"});
        let uri = format!("/api/modules/{}/assignments/{}/tasks/{}", data.module.id, data.assignment.id, 99999);
        let req = Request::builder()
            .method("PUT")
            .uri(&uri)
            .header("Authorization", format!("Bearer {}", token))
            .header(CONTENT_TYPE, "application/json")
            .body(AxumBody::from(serde_json::to_vec(&payload).unwrap()))
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::NOT_FOUND);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let json: Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(json["success"], false);
        assert_eq!(json["message"], "Task 99999 in Assignment 1 not found.");
    }

     /// Test Case: Editing Task for Non-Existent Assignment (Path Mismatch)
    #[tokio::test]
    #[serial]
    async fn test_edit_task_assignment_not_found_path_mismatch() {
        let db = setup_test_db().await;
        let (data, _temp_dir) = setup_test_data(&db).await;

        let assignment2 = AssignmentModel::create(
            &db,
            data.module.id,
            "Assignment 2",
            Some("Desc 2"),
            AssignmentType::Assignment,
            Utc.with_ymd_and_hms(2024, 1, 1, 0, 0, 0).unwrap(),
            Utc.with_ymd_and_hms(2024, 12, 31, 23, 59, 59).unwrap(),
        )
        .await
        .expect("Failed to create second assignment");
        let task_in_assignment2 = AssignmentTaskModel::create(
            &db,
            assignment2.id,
            1,
            "echo 'Other Assignment'",
            "Other Task",
        )
        .await
        .expect("Failed to create task in second assignment");

        let app = make_app(db.clone());
        let (token, _) = generate_jwt(data.admin_user.id, data.admin_user.admin);
        let payload = json!({"name": "Mismatched Name", "command": "echo 'Mismatched Command'"});
        let uri = format!("/api/modules/{}/assignments/{}/tasks/{}", data.module.id, data.assignment.id, task_in_assignment2.id);
        let req = Request::builder()
            .method("PUT")
            .uri(&uri)
            .header("Authorization", format!("Bearer {}", token))
            .header(CONTENT_TYPE, "application/json")
            .body(AxumBody::from(serde_json::to_vec(&payload).unwrap()))
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::NOT_FOUND);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let json: Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(json["success"], false);
        assert_eq!(json["message"], "Task 2 in Assignment 1 not found.");
    }


    /// Test Case: Editing Task with Non-Existent Module ID in Path
    #[tokio::test]
    #[serial]
    async fn test_edit_task_module_not_found() {
        let db = setup_test_db().await;
        let (data, _temp_dir) = setup_test_data(&db).await;

        let app = make_app(db.clone());
        let (token, _) = generate_jwt(data.admin_user.id, data.admin_user.admin);
        let payload = json!({"name": "Any Name", "command": "echo 'Any Command'"});
        let uri = format!("/api/modules/{}/assignments/{}/tasks/{}", 9999, data.assignment.id, data.task1.id);
        let req = Request::builder()
            .method("PUT")
            .uri(&uri)
            .header("Authorization", format!("Bearer {}", token))
            .header(CONTENT_TYPE, "application/json")
            .body(AxumBody::from(serde_json::to_vec(&payload).unwrap()))
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::NOT_FOUND);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let json: Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(json["success"], false);
        assert_eq!(json["message"], "Module 9999 not found.");
    }


    /// Test Case: Forbidden Access to Edit Task
    #[tokio::test]
    #[serial]
    async fn test_edit_task_forbidden() {
        let db = setup_test_db().await;
        let (data, _temp_dir) = setup_test_data(&db).await;

        let app = make_app(db.clone());
        let (token, _) = generate_jwt(data.forbidden_user.id, data.forbidden_user.admin);
        let payload = json!({"name": "Forbidden Name", "command": "echo 'Forbidden Command'"});
        let uri = format!("/api/modules/{}/assignments/{}/tasks/{}", data.module.id, data.assignment.id, data.task1.id);
        let req = Request::builder()
            .method("PUT")
            .uri(&uri)
            .header("Authorization", format!("Bearer {}", token))
            .header(CONTENT_TYPE, "application/json")
            .body(AxumBody::from(serde_json::to_vec(&payload).unwrap()))
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::FORBIDDEN);
    }

    /// Test Case: Invalid Request Body - Empty Name
    #[tokio::test]
    #[serial]
    async fn test_edit_task_invalid_name_empty() {
        let db = setup_test_db().await;
        let (data, _temp_dir) = setup_test_data(&db).await;

        let app = make_app(db.clone());
        let (token, _) = generate_jwt(data.admin_user.id, data.admin_user.admin);
        let payload = json!({"name": "", "command": "echo 'Valid Command'"});
        let uri = format!("/api/modules/{}/assignments/{}/tasks/{}", data.module.id, data.assignment.id, data.task1.id);
        let req = Request::builder()
            .method("PUT")
            .uri(&uri)
            .header("Authorization", format!("Bearer {}", token))
            .header(CONTENT_TYPE, "application/json")
            .body(AxumBody::from(serde_json::to_vec(&payload).unwrap()))
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let json: Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(json["success"], false);
        assert_eq!(json["message"], "'name' and 'command' must be non-empty strings");
    }

     /// Test Case: Invalid Request Body - Whitespace Only Name
    #[tokio::test]
    #[serial]
    async fn test_edit_task_invalid_name_whitespace() {
        let db = setup_test_db().await;
        let (data, _temp_dir) = setup_test_data(&db).await;

        let app = make_app(db.clone());
        let (token, _) = generate_jwt(data.admin_user.id, data.admin_user.admin);
        let payload = json!({"name": "   \n\t  ", "command": "echo 'Valid Command'"});
        let uri = format!("/api/modules/{}/assignments/{}/tasks/{}", data.module.id, data.assignment.id, data.task1.id);
        let req = Request::builder()
            .method("PUT")
            .uri(&uri)
            .header("Authorization", format!("Bearer {}", token))
            .header(CONTENT_TYPE, "application/json")
            .body(AxumBody::from(serde_json::to_vec(&payload).unwrap()))
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let json: Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(json["success"], false);
        assert_eq!(json["message"], "'name' and 'command' must be non-empty strings");
    }

    /// Test Case: Invalid Request Body - Empty Command
    #[tokio::test]
    #[serial]
    async fn test_edit_task_invalid_command_empty() {
        let db = setup_test_db().await;
        let (data, _temp_dir) = setup_test_data(&db).await;

        let app = make_app(db.clone());
        let (token, _) = generate_jwt(data.admin_user.id, data.admin_user.admin);
        let payload = json!({"name": "Valid Name", "command": ""});
        let uri = format!("/api/modules/{}/assignments/{}/tasks/{}", data.module.id, data.assignment.id, data.task1.id);
        let req = Request::builder()
            .method("PUT")
            .uri(&uri)
            .header("Authorization", format!("Bearer {}", token))
            .header(CONTENT_TYPE, "application/json")
            .body(AxumBody::from(serde_json::to_vec(&payload).unwrap()))
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let json: Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(json["success"], false);
        assert_eq!(json["message"], "'name' and 'command' must be non-empty strings");
    }

    /// Test Case: Invalid Request Body - Whitespace Only Command
    #[tokio::test]
    #[serial]
    async fn test_edit_task_invalid_command_whitespace() {
        let db = setup_test_db().await;
        let (data, _temp_dir) = setup_test_data(&db).await;

        let app = make_app(db.clone());
        let (token, _) = generate_jwt(data.admin_user.id, data.admin_user.admin);
        let payload = json!({"name": "Valid Name", "command": "   \n\t  "});
        let uri = format!("/api/modules/{}/assignments/{}/tasks/{}", data.module.id, data.assignment.id, data.task1.id);
        let req = Request::builder()
            .method("PUT")
            .uri(&uri)
            .header("Authorization", format!("Bearer {}", token))
            .header(CONTENT_TYPE, "application/json")
            .body(AxumBody::from(serde_json::to_vec(&payload).unwrap()))
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let json: Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(json["success"], false);
        assert_eq!(json["message"], "'name' and 'command' must be non-empty strings");
    }
}