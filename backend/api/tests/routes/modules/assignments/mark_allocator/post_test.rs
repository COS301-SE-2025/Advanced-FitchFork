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
        assignment::Model as AssignmentModel,
        module::Model as ModuleModel,
        user::Model as UserModel,
        user_module_role::{Model as UserModuleRoleModel, Role},
    };
    use db::models::{assignment_memo_output, assignment_task};
    use sea_orm::{ActiveModelTrait, Set};
    use serde_json::Value;
    use serial_test::serial;
    use tower::ServiceExt;
    use util::paths::{mark_allocator_path, memo_output_dir};

    struct TestData {
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
            lecturer_user,
            student_user,
            forbidden_user,
            module,
            assignment,
        }
    }

    #[tokio::test]
    #[serial]
    async fn test_post_mark_allocator_success_as_lecturer() {
        let (app, app_state, _tmp) = make_test_app_with_storage().await;
        let data = setup_test_data(app_state.db()).await;

        // 1) Create a task with a non-empty command (NOT NULL)
        let task = assignment_task::Model::create(
            app_state.db(),
            data.assignment.id,
            1,         // task_number
            "",        // name -> will default to "Task 1" in allocator
            "echo ok", // <-- provide any command string (NOT NULL)
            false,     // code_coverage
            false,     // valgrind
        )
        .await
        .unwrap();

        // 2) Write memo file to disk
        let memo_dir = memo_output_dir(data.module.id, data.assignment.id);
        std::fs::create_dir_all(&memo_dir).unwrap();
        let memo_file_name = "task_1.txt";
        std::fs::write(
            memo_dir.join(memo_file_name),
            "&-=-& Sub1\nline A\nline B\n",
        )
        .unwrap();

        // 3) Insert memo-output DB row pointing to the file (path relative to storage_root)
        let rel_path = format!(
            "module_{}/assignment_{}/memo_output/{}",
            data.module.id, data.assignment.id, memo_file_name
        );

        // If your assignment_memo_output has NOT NULL timestamps, set them:
        let _ = assignment_memo_output::ActiveModel {
            assignment_id: Set(data.assignment.id),
            task_id: Set(task.id),
            path: Set(rel_path),
            // created_at / updated_at: include these if your schema requires NOT NULL
            // created_at: Set(Utc::now()),
            // updated_at: Set(Utc::now()),
            ..Default::default()
        }
        .insert(app_state.db())
        .await
        .unwrap();

        // 4) Hit the endpoint
        let (token, _) = generate_jwt(data.lecturer_user.id, data.lecturer_user.admin);
        let uri = format!(
            "/api/modules/{}/assignments/{}/mark_allocator/generate",
            data.module.id, data.assignment.id
        );
        let req = Request::builder()
            .uri(&uri)
            .method("POST")
            .header("Authorization", format!("Bearer {}", token))
            .header("Content-Type", "application/json")
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert!(memo_dir.exists(), "Memo output folder not created!");
        assert_eq!(response.status(), StatusCode::OK);

        // 5) Read generated allocator and assert normalized shape/values
        let alloc_path = mark_allocator_path(data.module.id, data.assignment.id);
        assert!(alloc_path.exists(), "allocator.json was not created");
        let raw = std::fs::read_to_string(&alloc_path).unwrap();
        let json: Value = serde_json::from_str(&raw).unwrap();

        // top-level
        assert!(json.get("generated_at").is_some(), "missing generated_at");
        assert_eq!(
            json["total_value"], 2.0,
            "total_value should match sum of task values"
        );

        // tasks
        let tasks = json["tasks"].as_array().expect("tasks not array");
        assert_eq!(tasks.len(), 1);
        assert_eq!(tasks[0]["task_number"], 1);
        assert_eq!(tasks[0]["name"], "Task 1"); // defaulted
        assert_eq!(tasks[0]["value"], 2.0);
        assert_eq!(tasks[0]["code_coverage"], false);

        // subsections
        let subs = tasks[0]["subsections"]
            .as_array()
            .expect("subsections not array");
        assert_eq!(subs.len(), 1);
        assert_eq!(subs[0]["name"], "Sub1");
        assert_eq!(subs[0]["value"], 2.0);

        // Note: regex/feedback may be omitted (None) unless scheme=Regex; don't assert presence.
    }

    //Commented out due to change in mark_allocator functionality - test no longer applies

    // #[tokio::test]
    // #[serial]
    // async fn test_post_mark_allocator_not_found() {
    //     setup_assignment_storage_root();
    //     let (app, app_state) = make_test_app().await;
    //     let data = setup_test_data(app_state.db()).await;

    //     let (token, _) = generate_jwt(data.lecturer_user.id, data.lecturer_user.admin);
    //     let uri = format!(
    //         "/api/modules/{}/assignments/{}/mark_allocator/generate",
    //         data.module.id, data.assignment.id
    //     );
    //     let req = Request::builder()
    //         .method("POST")
    //         .uri(&uri)
    //         .header("Authorization", format!("Bearer {}", token))
    //         .body(Body::empty())
    //         .unwrap();

    //     let response = app.oneshot(req).await.unwrap();
    //     assert_eq!(response.status(), StatusCode::NOT_FOUND);
    // }

    #[tokio::test]
    #[serial]
    async fn test_post_mark_allocator_forbidden_for_student() {
        let (app, app_state, _tmp) = make_test_app_with_storage().await;
        let data = setup_test_data(app_state.db()).await;

        let (token, _) = generate_jwt(data.student_user.id, data.student_user.admin);
        let uri = format!(
            "/api/modules/{}/assignments/{}/mark_allocator/generate",
            data.module.id, data.assignment.id
        );
        let req = Request::builder()
            .method("POST")
            .uri(&uri)
            .header("Authorization", format!("Bearer {}", token))
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::FORBIDDEN);
    }

    #[tokio::test]
    #[serial]
    async fn test_post_mark_allocator_forbidden_for_unassigned_user() {
        let (app, app_state, _tmp) = make_test_app_with_storage().await;
        let data = setup_test_data(app_state.db()).await;

        let (token, _) = generate_jwt(data.forbidden_user.id, data.forbidden_user.admin);
        let uri = format!(
            "/api/modules/{}/assignments/{}/mark_allocator/generate",
            data.module.id, data.assignment.id
        );
        let req = Request::builder()
            .method("POST")
            .uri(&uri)
            .header("Authorization", format!("Bearer {}", token))
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::FORBIDDEN);
    }

    #[tokio::test]
    #[serial]
    async fn test_post_mark_allocator_unauthorized() {
        let (app, app_state, _tmp) = make_test_app_with_storage().await;
        let data = setup_test_data(app_state.db()).await;

        let uri = format!(
            "/api/modules/{}/assignments/{}/mark_allocator/generate",
            data.module.id, data.assignment.id
        );
        let req = Request::builder()
            .method("POST")
            .uri(&uri)
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    //Commented out due to change in mark_allocator functionality - test no longer applies

    // #[tokio::test]
    // #[serial]
    // async fn test_post_mark_allocator_missing_memo_or_config() {
    //     setup_assignment_storage_root();
    //     let (app, app_state) = make_test_app().await;
    //     let data = setup_test_data(app_state.db()).await;

    //     let (token, _) = generate_jwt(data.lecturer_user.id, data.lecturer_user.admin);
    //     let uri = format!(
    //         "/api/modules/{}/assignments/{}/mark_allocator/generate",
    //         data.module.id, data.assignment.id
    //     );
    //     let req = Request::builder()
    //         .method("POST")
    //         .uri(&uri)
    //         .header("Authorization", format!("Bearer {}", token))
    //         .body(Body::empty())
    //         .unwrap();

    //     let response = app.oneshot(req).await.unwrap();
    //     assert_eq!(response.status(), StatusCode::NOT_FOUND);
    // }

    // #[tokio::test]
    // #[serial]
    // async fn test_post_mark_allocator_missing_memo_output() {
    //     setup_assignment_storage_root();
    //     let (app, app_state) = make_test_app().await;
    //     let data = setup_test_data(app_state.db()).await;

    //     let (token, _) = generate_jwt(data.lecturer_user.id, data.lecturer_user.admin);
    //     let uri = format!(
    //         "/api/modules/{}/assignments/{}/mark_allocator/generate",
    //         data.module.id, data.assignment.id
    //     );
    //     let req = Request::builder()
    //         .uri(&uri)
    //         .method("POST")
    //         .header("Authorization", format!("Bearer {}", token))
    //         .header("Content-Type", "application/json")
    //         .body(Body::empty())
    //         .unwrap();

    //     let response = app.oneshot(req).await.unwrap();
    //     assert_eq!(response.status(), StatusCode::NOT_FOUND);
    // }
}
