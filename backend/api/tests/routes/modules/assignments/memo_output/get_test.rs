#[cfg(test)]
mod tests {
    use crate::helpers::app::make_test_app_with_storage;
    use api::auth::generate_jwt;
    use axum::{
        body::{Body, to_bytes},
        http::{Request, StatusCode},
    };
    use chrono::{TimeZone, Utc};
    use db::models::{
        assignment::Model as AssignmentModel,
        assignment_memo_output, assignment_task,
        module::Model as ModuleModel,
        user::Model as UserModel,
        user_module_role::{Model as UserModuleRoleModel, Role},
    };
    use serde_json::Value;
    use serial_test::serial;
    use std::fs;
    use tower::ServiceExt;
    use util::paths::memo_output_dir;

    struct TestData {
        admin_user: UserModel,
        lecturer_user: UserModel,
        student_user: UserModel,
        forbidden_user: UserModel,
        module: ModuleModel,
        assignment: AssignmentModel,
    }

    async fn setup_test_data(db: &sea_orm::DatabaseConnection) -> TestData {
        let module = ModuleModel::create(db, "COS101", 2024, Some("Test Module"), 16)
            .await
            .unwrap();
        let service = UserService::new(UserRepository::new(db.clone()));
        let admin_user = service
            .create(CreateUser {
                username: "admin1".to_string(),
                email: "admin1@test.com".to_string(),
                password: "password".to_string(),
                admin: true,
            })
            .await
            .unwrap();
        let lecturer_user = service
            .create(CreateUser {
                username: "lecturer1".to_string(),
                email: "lecturer1@test.com".to_string(),
                password: "password1".to_string(),
                admin: false,
            })
            .await
            .unwrap();
        let student_user = service
            .create(CreateUser {
                username: "student1".to_string(),
                email: "student1@test.com".to_string(),
                password: "password2".to_string(),
                admin: false,
            })
            .await
            .unwrap();
        let forbidden_user = service
            .create(CreateUser {
                username: "forbidden".to_string(),
                email: "forbidden@test.com".to_string(),
                password: "password3".to_string(),
                admin: false,
            })
            .await
            .unwrap();
        UserModuleRoleModel::assign_user_to_module(db, lecturer_user.id, module.id, Role::Lecturer)
            .await
            .unwrap();
        UserModuleRoleModel::assign_user_to_module(db, student_user.id, module.id, Role::Student)
            .await
            .unwrap();
        let assignment = AssignmentModel::create(
            db,
            module.id,
            "Assignment 1",
            Some("Desc 1"),
            db::models::assignment::AssignmentType::Assignment,
            Utc.with_ymd_and_hms(2024, 1, 1, 0, 0, 0).unwrap(),
            Utc.with_ymd_and_hms(2024, 1, 31, 23, 59, 59).unwrap(),
        )
        .await
        .unwrap();

        TestData {
            admin_user,
            lecturer_user,
            student_user,
            forbidden_user,
            module,
            assignment,
        }
    }

    async fn setup_memo_output_file(
        db: &sea_orm::DatabaseConnection,
        module_id: i64,
        assignment_id: i64,
        task_number: i64,
    ) -> assignment_memo_output::Model {
        // 1. Ensure directory exists
        let memo_output_directory = memo_output_dir(module_id, assignment_id);
        fs::create_dir_all(&memo_output_directory).unwrap();

        // 2. Create a task in DB
        let task = assignment_task::Model::create(
            db,
            assignment_id,
            task_number as i64,
            &format!("Task {}", task_number),
            "echo Hello", // dummy command
            false,
        )
        .await
        .unwrap();

        // 3. Create a memo output record in DB + write file using `save_file`
        let contents = b"This is a test memo output.";
        assignment_memo_output::Model::save_file(
            db,
            assignment_id,
            task.id,
            &format!("{}.txt", task_number),
            contents,
        )
        .await
        .unwrap()
    }

    #[tokio::test]
    #[serial]
    async fn test_get_memo_output_success_as_lecturer() {
        let (app, app_state, _tmp) = make_test_app_with_storage().await;
        let data = setup_test_data(app_state.db()).await;
        setup_memo_output_file(app_state.db(), data.module.id, data.assignment.id, 1).await;

        let (token, _) = generate_jwt(data.lecturer_user.id, data.lecturer_user.admin);
        let uri = format!(
            "/api/modules/{}/assignments/{}/memo_output",
            data.module.id, data.assignment.id
        );
        let req = Request::builder()
            .uri(&uri)
            .header("Authorization", format!("Bearer {}", token))
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);

        let body_bytes = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let v: Value = serde_json::from_slice(&body_bytes).unwrap();
        let data_arr = v
            .get("data")
            .and_then(|d| d.as_array())
            .expect("data not array");
        //TODO this failed for some reason once on my side (could not replicate again) - Richard
        assert_eq!(data_arr.len(), 1);
        assert_eq!(
            data_arr[0].get("raw").and_then(|r| r.as_str()),
            Some("This is a test memo output.")
        );
    }

    #[tokio::test]
    #[serial]
    async fn test_get_memo_output_success_as_admin() {
        let (app, app_state, _tmp) = make_test_app_with_storage().await;
        let data = setup_test_data(app_state.db()).await;
        setup_memo_output_file(app_state.db(), data.module.id, data.assignment.id, 1).await;

        let (token, _) = generate_jwt(data.admin_user.id, data.admin_user.admin);
        let uri = format!(
            "/api/modules/{}/assignments/{}/memo_output",
            data.module.id, data.assignment.id
        );
        let req = Request::builder()
            .uri(&uri)
            .header("Authorization", format!("Bearer {}", token))
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    #[serial]
    async fn test_get_memo_output_forbidden_for_student() {
        let (app, app_state, _tmp) = make_test_app_with_storage().await;
        let data = setup_test_data(app_state.db()).await;
        setup_memo_output_file(app_state.db(), data.module.id, data.assignment.id, 1).await;

        let (token, _) = generate_jwt(data.student_user.id, data.student_user.admin);
        let uri = format!(
            "/api/modules/{}/assignments/{}/memo_output",
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

    #[tokio::test]
    #[serial]
    async fn test_get_memo_output_forbidden_for_unassigned_user() {
        let (app, app_state, _tmp) = make_test_app_with_storage().await;
        let data = setup_test_data(app_state.db()).await;
        setup_memo_output_file(app_state.db(), data.module.id, data.assignment.id, 1).await;

        let (token, _) = generate_jwt(data.forbidden_user.id, data.forbidden_user.admin);
        let uri = format!(
            "/api/modules/{}/assignments/{}/memo_output",
            data.module.id, data.assignment.id
        );
        let req = Request::builder()
            .uri(&uri)
            .header("Authorization", format!("Bearer {}", token))
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::FORBIDDEN)
    }

    #[tokio::test]
    #[serial]
    async fn test_get_memo_output_not_found_if_file_doesnt_exist() {
        let (app, app_state, _tmp) = make_test_app_with_storage().await;
        let data = setup_test_data(app_state.db()).await;

        let (token, _) = generate_jwt(data.lecturer_user.id, data.lecturer_user.admin);
        let uri = format!(
            "/api/modules/{}/assignments/{}/memo_output",
            data.module.id, data.assignment.id
        );
        let req = Request::builder()
            .uri(&uri)
            .header("Authorization", format!("Bearer {}", token))
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    #[serial]
    async fn test_get_memo_output_assignment_not_found() {
        let (app, app_state, _tmp) = make_test_app_with_storage().await;
        let data = setup_test_data(app_state.db()).await;

        let (token, _) = generate_jwt(data.lecturer_user.id, data.lecturer_user.admin);
        let uri = format!(
            "/api/modules/{}/assignments/{}/memo_output",
            data.module.id, 9999
        );
        let req = Request::builder()
            .uri(&uri)
            .header("Authorization", format!("Bearer {}", token))
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    #[serial]
    async fn test_get_memo_output_unauthorized() {
        let (app, app_state, _tmp) = make_test_app_with_storage().await;
        let data = setup_test_data(app_state.db()).await;

        let uri = format!(
            "/api/modules/{}/assignments/{}/memo_output",
            data.module.id, data.assignment.id
        );
        let req = Request::builder().uri(&uri).body(Body::empty()).unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }
}
