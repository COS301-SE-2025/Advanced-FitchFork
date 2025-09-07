#[cfg(test)]
mod tests {
    use crate::helpers::app::make_test_app;
    use api::auth::generate_jwt;
    use axum::{
        body::Body,
        http::{Request, StatusCode},
    };
    use chrono::{TimeZone, Utc};
    use db::models::{
        assignment::{AssignmentType, Model as AssignmentModel},
        assignment_task::Model as AssignmentTaskModel,
        module::Model as ModuleModel,
        user::Model as UserModel,
        user_module_role::{Model as UserModuleRoleModel, Role},
    };
    use dotenvy;
    use serde_json::Value;
    use serde_json::json;
    use serial_test::serial;
    use std::fs;
    use tempfile::{TempDir, tempdir};
    use tower::ServiceExt;
    use util::mark_allocator::mark_allocator::save_allocator;

    struct TestData {
        admin_user: UserModel,
        forbidden_user: UserModel,
        lecturer1: UserModel,
        module: ModuleModel,
        assignment: AssignmentModel,
        task1: AssignmentTaskModel,
        task2: AssignmentTaskModel,
    }

    async fn setup_test_data(db: &sea_orm::DatabaseConnection) -> (TestData, TempDir) {
        dotenvy::dotenv().expect("Failed to load .env");
        let temp_dir = tempdir().expect("Failed to create temporary directory");
        unsafe {
            std::env::set_var("ASSIGNMENT_STORAGE_ROOT", temp_dir.path().to_str().unwrap());
        }

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

        let task1 = AssignmentTaskModel::create(
            db,
            assignment.id,
            1,
            "echo 'Task 1'",
            "Task 1 Name",
            false,
        )
        .await
        .expect("Failed to create task 1");

        let task2 = AssignmentTaskModel::create(
            db,
            assignment.id,
            2,
            "echo 'Task 2'",
            "Task 2 Name",
            false,
        )
        .await
        .expect("Failed to create task 2");

        let allocator_data = json!({
            "allocatorVersion": "1.0",
            "tasks": [
                {
                    "task1": {
                        "name": "Task 1 Name",
                        "value": 25,
                        "subsections": [
                            { "name": "Subsection A", "value": 10 },
                            { "name": "Subsection B", "value": 15 }
                        ]
                    }
                },
                {
                    "task2": {
                        "name": "Task 2 Name",
                        "value": 30,
                        "subsections": [
                            { "name": "Part 1", "value": 20 },
                            { "name": "Part 2", "value": 10 }
                        ]
                    }
                }
            ]
        });
        let _ = save_allocator(module.id, assignment.id, allocator_data).await;

        let memo_dir = temp_dir
            .path()
            .join(format!("module_{}", module.id))
            .join(format!("assignment_{}", assignment.id))
            .join("memo_output");
        fs::create_dir_all(&memo_dir).expect("Failed to create memo output directory");

        let memo_file_path = memo_dir.join("task_1.txt");
        let memo_content =
            "Overall Feedback\n&-=-&\nFeedback for Subsection A\n&-=-&\nFeedback for Subsection B";
        fs::write(&memo_file_path, memo_content).expect("Failed to write task memo file");

        (
            TestData {
                admin_user,
                forbidden_user,
                lecturer1,
                module,
                assignment,
                task1,
                task2,
            },
            temp_dir,
        )
    }

    // --- Tests for GET /api/modules/{module_id}/assignments/{assignment_id}/tasks (list_tasks) ---

    /// Test Case: Successful Retrieval of Tasks List as Admin
    #[tokio::test]
    #[serial]
    async fn test_list_tasks_success_as_admin() {
        let (app, app_state) = make_test_app().await;
        let (data, _temp_dir) = setup_test_data(app_state.db()).await;

        let (token, _) = generate_jwt(data.admin_user.id, data.admin_user.admin);
        let uri = format!(
            "/api/modules/{}/assignments/{}/tasks",
            data.module.id, data.assignment.id
        );
        let req = Request::builder()
            .uri(&uri)
            .header("Authorization", format!("Bearer {}", token))
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let json: Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(json["success"], true);
        assert_eq!(json["message"], "Tasks retrieved successfully");
        let tasks_array = json["data"].as_array().expect("Data should be an array");
        assert_eq!(tasks_array.len(), 2);

        let task1_data = &tasks_array[0];
        assert_eq!(task1_data["id"], data.task1.id);
        assert_eq!(task1_data["task_number"], data.task1.task_number);
        assert_eq!(task1_data["name"], data.task1.name);
        assert_eq!(task1_data["command"], data.task1.command);

        let task2_data = &tasks_array[1];
        assert_eq!(task2_data["id"], data.task2.id);
        assert_eq!(task2_data["task_number"], data.task2.task_number);
        assert_eq!(task2_data["name"], data.task2.name);
        assert_eq!(task2_data["command"], data.task2.command);
    }

    /// Test Case: Successful Retrieval of Tasks List as Lecturer
    #[tokio::test]
    #[serial]
    async fn test_list_tasks_success_as_lecturer() {
        let (app, app_state) = make_test_app().await;
        let (data, _temp_dir) = setup_test_data(app_state.db()).await;

        let (token, _) = generate_jwt(data.lecturer1.id, data.lecturer1.admin);
        let uri = format!(
            "/api/modules/{}/assignments/{}/tasks",
            data.module.id, data.assignment.id
        );
        let req = Request::builder()
            .uri(&uri)
            .header("Authorization", format!("Bearer {}", token))
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let json: Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(json["success"], true);
        assert_eq!(json["message"], "Tasks retrieved successfully");
        let tasks_array = json["data"].as_array().expect("Data should be an array");
        assert_eq!(tasks_array.len(), 2);
    }

    /// Test Case: Listing Tasks for Non-Existent Assignment
    #[tokio::test]
    #[serial]
    async fn test_list_tasks_assignment_not_found() {
        let (app, app_state) = make_test_app().await;
        let (data, _temp_dir) = setup_test_data(app_state.db()).await;

        let (token, _) = generate_jwt(data.admin_user.id, data.admin_user.admin);
        let uri = format!("/api/modules/{}/assignments/{}/tasks", data.module.id, 9999);
        let req = Request::builder()
            .uri(&uri)
            .header("Authorization", format!("Bearer {}", token))
            .body(Body::empty())
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

    /// Test Case: Listing Tasks for Non-Existent Module
    #[tokio::test]
    #[serial]
    async fn test_list_tasks_module_not_found() {
        let (app, app_state) = make_test_app().await;
        let (data, _temp_dir) = setup_test_data(app_state.db()).await;

        let (token, _) = generate_jwt(data.admin_user.id, data.admin_user.admin);
        let uri = format!(
            "/api/modules/{}/assignments/{}/tasks",
            9999, data.assignment.id
        );
        let req = Request::builder()
            .uri(&uri)
            .header("Authorization", format!("Bearer {}", token))
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::NOT_FOUND);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let json: Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(json["success"], false);
        assert_eq!(json["message"], "Module 9999 not found.");
    }

    /// Test Case: Forbidden Access to List Tasks
    #[tokio::test]
    #[serial]
    async fn test_list_tasks_forbidden() {
        let (app, app_state) = make_test_app().await;
        let (data, _temp_dir) = setup_test_data(app_state.db()).await;

        let (token, _) = generate_jwt(data.forbidden_user.id, data.forbidden_user.admin);
        let uri = format!(
            "/api/modules/{}/assignments/{}/tasks",
            data.module.id, data.assignment.id
        );
        let req = Request::builder()
            .uri(&uri)
            .header("Authorization", format!("Bearer {}", token))
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::FORBIDDEN);
    }

    // --- Tests for GET /api/modules/{module_id}/assignments/{assignment_id}/tasks/{task_id} (get_task_details) ---

    /// Test Case: Successful Retrieval of Task Details as Admin
    #[tokio::test]
    #[serial]
    async fn test_get_task_details_success_as_admin() {
        let (app, app_state) = make_test_app().await;
        let (data, _temp_dir) = setup_test_data(app_state.db()).await;

        let (token, _) = generate_jwt(data.admin_user.id, data.admin_user.admin);
        let uri = format!(
            "/api/modules/{}/assignments/{}/tasks/{}",
            data.module.id, data.assignment.id, data.task1.id
        );
        let req = Request::builder()
            .uri(&uri)
            .header("Authorization", format!("Bearer {}", token))
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let json: Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(json["success"], true);
        assert_eq!(json["message"], "Task details retrieved successfully");
        let task_data = &json["data"];

        assert_eq!(task_data["id"], data.task1.id);
        assert_eq!(task_data["task_id"], data.task1.id);
        assert_eq!(task_data["name"], data.task1.name);
        assert_eq!(task_data["command"], data.task1.command);
        assert!(task_data["created_at"].as_str().is_some());
        assert!(task_data["updated_at"].as_str().is_some());

        let subsections_array = task_data["subsections"]
            .as_array()
            .expect("Subsections should be an array");
        assert_eq!(subsections_array.len(), 2);

        let subsec1 = &subsections_array[0];
        assert_eq!(subsec1["name"], "Subsection A");
        assert_eq!(subsec1["value"], 10);
        assert_eq!(subsec1["memo_output"], "Feedback for Subsection A");

        let subsec2 = &subsections_array[1];
        assert_eq!(subsec2["name"], "Subsection B");
        assert_eq!(subsec2["value"], 15);
        assert_eq!(subsec2["memo_output"], "Feedback for Subsection B");
    }

    /// Test Case: Successful Retrieval of Task Details as Lecturer
    #[tokio::test]
    #[serial]
    async fn test_get_task_details_success_as_lecturer() {
        let (app, app_state) = make_test_app().await;
        let (data, _temp_dir) = setup_test_data(app_state.db()).await;

        let (token, _) = generate_jwt(data.lecturer1.id, data.lecturer1.admin);
        let uri = format!(
            "/api/modules/{}/assignments/{}/tasks/{}",
            data.module.id, data.assignment.id, data.task1.id
        );
        let req = Request::builder()
            .uri(&uri)
            .header("Authorization", format!("Bearer {}", token))
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
    }

    /// Test Case: Retrieving Details for Non-Existent Task
    #[tokio::test]
    #[serial]
    async fn test_get_task_details_task_not_found() {
        let (app, app_state) = make_test_app().await;
        let (data, _temp_dir) = setup_test_data(app_state.db()).await;

        let (token, _) = generate_jwt(data.admin_user.id, data.admin_user.admin);
        let uri = format!(
            "/api/modules/{}/assignments/{}/tasks/{}",
            data.module.id, data.assignment.id, 99999
        );
        let req = Request::builder()
            .uri(&uri)
            .header("Authorization", format!("Bearer {}", token))
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::NOT_FOUND);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let json: Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(json["success"], false);
        assert_eq!(json["message"], "Task 99999 in Assignment 1 not found.");
    }

    /// Test Case: Retrieving Task Details for Task Not Belonging to Assignment
    #[tokio::test]
    #[serial]
    async fn test_get_task_details_task_wrong_assignment() {
        let (app, app_state) = make_test_app().await;
        let (data, _temp_dir) = setup_test_data(app_state.db()).await;

        let open_date = Utc.with_ymd_and_hms(2024, 1, 1, 0, 0, 0).unwrap();
        let due_date = Utc.with_ymd_and_hms(2024, 12, 31, 23, 59, 59).unwrap();
        let assignment2 = AssignmentModel::create(
            app_state.db(),
            data.module.id,
            "Assignment 2",
            Some("Desc 2"),
            AssignmentType::Assignment,
            open_date,
            due_date,
        )
        .await
        .expect("Failed to create second assignment");
        let task_in_assignment2 = AssignmentTaskModel::create(
            app_state.db(),
            assignment2.id,
            1,
            "echo 'Other'",
            "Task in Ass 2",
            false,
        )
        .await
        .expect("Failed to create task in second assignment");

        let (token, _) = generate_jwt(data.admin_user.id, data.admin_user.admin);
        let uri = format!(
            "/api/modules/{}/assignments/{}/tasks/{}",
            data.module.id, data.assignment.id, task_in_assignment2.id
        );
        let req = Request::builder()
            .uri(&uri)
            .header("Authorization", format!("Bearer {}", token))
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::NOT_FOUND);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let json: Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(json["success"], false);
        assert_eq!(json["message"], "Task 3 in Assignment 1 not found.");
    }

    /// Test Case: Forbidden Access to Get Task Details
    #[tokio::test]
    #[serial]
    async fn test_get_task_details_forbidden() {
        let (app, app_state) = make_test_app().await;
        let (data, _temp_dir) = setup_test_data(app_state.db()).await;

        let (token, _) = generate_jwt(data.forbidden_user.id, data.forbidden_user.admin);
        let uri = format!(
            "/api/modules/{}/assignments/{}/tasks/{}",
            data.module.id, data.assignment.id, data.task1.id
        );
        let req = Request::builder()
            .uri(&uri)
            .header("Authorization", format!("Bearer {}", token))
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::FORBIDDEN);
    }
}
