#[cfg(test)]
mod tests {
    use db::{
        models::{
            user::Model as UserModel,
            module::Model as ModuleModel,
            assignment::{Model as AssignmentModel, AssignmentType},
            assignment_task::Model as AssignmentTaskModel,
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
        user_service::{UserService, CreateUser},
    };
    use tower::ServiceExt;
    use serde_json::Value;
    use api::auth::generate_jwt;
    use crate::helpers::app::make_test_app;
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
        let service = UserService::new(UserRepository::new(db.clone()));
        let admin_user = service.create(CreateUser{ username: "task_admin".to_string(), email: "task_admin@test.com".to_string(), password: "password".to_string(), admin: true }).await.expect("Failed to create admin user");
        let forbidden_user = service.create(CreateUser{ username: "task_unauthed".to_string(), email: "task_unauthed@test.com".to_string(), password: "password".to_string(), admin: false }).await.expect("Failed to create forbidden user");
        let lecturer1 = service.create(CreateUser{ username: "task_lecturer1".to_string(), email: "task_lecturer1@test.com".to_string(), password: "password1".to_string(), admin: false }).await.expect("Failed to create lecturer1");
        UserModuleRoleModel::assign_user_to_module(db, lecturer1.id, module.id, Role::Lecturer).await.expect("Failed to assign lecturer1 to module");
        let assignment = AssignmentModel::create(
            db,
            module.id,
            "Test Assignment",
            Some("Description"),
            AssignmentType::Assignment,
            Utc.with_ymd_and_hms(2024, 1, 1, 0, 0, 0).unwrap(),
            Utc.with_ymd_and_hms(2024, 12, 31, 23, 59, 59).unwrap(),
        ).await.expect("Failed to create test assignment");
        let task1 = AssignmentTaskModel::create(
             db,
             assignment.id,
             1,
             "echo 'Task to Delete'",
             "Task To Delete Name",
         ).await.expect("Failed to create task to delete");

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

    /// Test Case: Successful Deletion of a Task as Admin
    #[tokio::test]
    #[serial]
    async fn test_delete_task_success_as_admin() {
        let (app, app_state) = make_test_app().await;
        let (data, _temp_dir) = setup_test_data(app_state.db()).await;

        let (token, _) = generate_jwt(data.admin_user.id, data.admin_user.admin);
        let uri = format!("/api/modules/{}/assignments/{}/tasks/{}", data.module.id, data.assignment.id, data.task1.id);
        let req = Request::builder()
            .method("DELETE")
            .uri(&uri)
            .header("Authorization", format!("Bearer {}", token))
            .body(AxumBody::empty())
            .unwrap();

        let response = app.clone().oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let json: Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(json["success"], true);
        assert_eq!(json["message"], "Task deleted successfully");
        assert_eq!(json["data"], serde_json::Value::Null);

        let list_uri = format!("/api/modules/{}/assignments/{}/tasks", data.module.id, data.assignment.id);
        let list_req = Request::builder()
            .uri(&list_uri)
            .header("Authorization", format!("Bearer {}", token))
            .body(AxumBody::empty())
            .unwrap();

        let list_response = app.oneshot(list_req).await.unwrap();
        assert_eq!(list_response.status(), StatusCode::OK);

        let list_body = axum::body::to_bytes(list_response.into_body(), usize::MAX).await.unwrap();
        let list_json: Value = serde_json::from_slice(&list_body).unwrap();
        let tasks_array = list_json["data"].as_array().expect("Data should be an array");

        let task_ids: Vec<i64> = tasks_array.iter().map(|t| t["id"].as_i64().unwrap()).collect();
        assert!(!task_ids.contains(&data.task1.id));
    }

    /// Test Case: Successful Deletion of a Task as Lecturer
    #[tokio::test]
    #[serial]
    async fn test_delete_task_success_as_lecturer() {
        let (app, app_state) = make_test_app().await;
        let (data, _temp_dir) = setup_test_data(app_state.db()).await;

        let (token, _) = generate_jwt(data.lecturer1.id, data.lecturer1.admin);
        let uri = format!("/api/modules/{}/assignments/{}/tasks/{}", data.module.id, data.assignment.id, data.task1.id);
        let req = Request::builder()
            .method("DELETE")
            .uri(&uri)
            .header("Authorization", format!("Bearer {}", token))
            .body(AxumBody::empty())
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
    }

    /// Test Case: Attempting to Delete a Non-Existent Task
    #[tokio::test]
    #[serial]
    async fn test_delete_task_not_found() {
        let (app, app_state) = make_test_app().await;
        let (data, _temp_dir) = setup_test_data(app_state.db()).await;

        let (token, _) = generate_jwt(data.admin_user.id, data.admin_user.admin);
        let uri = format!("/api/modules/{}/assignments/{}/tasks/{}", data.module.id, data.assignment.id, 99999);
        let req = Request::builder()
            .method("DELETE")
            .uri(&uri)
            .header("Authorization", format!("Bearer {}", token))
            .body(AxumBody::empty())
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::NOT_FOUND);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let json: Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(json["success"], false);
        assert_eq!(json["message"], "Task 99999 in Assignment 1 not found.");
    }

    /// Test Case: Forbidden Access to Delete Task
    #[tokio::test]
    #[serial]
    async fn test_delete_task_forbidden() {
        let (app, app_state) = make_test_app().await;
        let (data, _temp_dir) = setup_test_data(app_state.db()).await;

        let (token, _) = generate_jwt(data.forbidden_user.id, data.forbidden_user.admin);
        let uri = format!("/api/modules/{}/assignments/{}/tasks/{}", data.module.id, data.assignment.id, data.task1.id);
        let req = Request::builder()
            .method("DELETE")
            .uri(&uri)
            .header("Authorization", format!("Bearer {}", token))
            .body(AxumBody::empty())
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::FORBIDDEN);
    }
}