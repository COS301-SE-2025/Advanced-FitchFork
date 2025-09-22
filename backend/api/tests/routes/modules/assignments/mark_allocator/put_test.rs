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
    use serde_json::json;
    use serial_test::serial;
    use tower::ServiceExt;

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
    async fn test_put_mark_allocator_success_as_lecturer() {
        use serde_json::json;

        let (app, app_state, _tmp) = make_test_app_with_storage().await;
        let data = setup_test_data(app_state.db()).await;

        let (token, _) = generate_jwt(data.lecturer_user.id, data.lecturer_user.admin);
        let uri = format!(
            "/api/modules/{}/assignments/{}/mark_allocator",
            data.module.id, data.assignment.id
        );

        // Normalized allocator: must include generated_at + total_value
        let payload = json!({
            "generated_at": Utc::now().to_rfc3339(),
            "tasks": [
                {
                    "task_number": 1,
                    "name": "Task 1",
                    "value": 1,
                    "subsections": [
                        { "name": "Correctness", "value": 1 }
                    ]
                }
            ],
            "total_value": 1
        });

        let req = Request::builder()
            .uri(&uri)
            .method("PUT")
            .header("Authorization", format!("Bearer {}", token))
            .header("Content-Type", "application/json")
            .body(Body::from(payload.to_string()))
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    #[serial]
    async fn test_put_mark_allocator_validation_error_values() {
        use serde_json::json;

        let (app, app_state, _tmp) = make_test_app_with_storage().await;
        let data = setup_test_data(app_state.db()).await;

        let (token, _) = generate_jwt(data.lecturer_user.id, data.lecturer_user.admin);
        let uri = format!(
            "/api/modules/{}/assignments/{}/mark_allocator",
            data.module.id, data.assignment.id
        );

        // Subsections sum to 1 but task.value is 2 -> 400
        let bad_payload = json!({
            "generated_at": Utc::now().to_rfc3339(),
            "tasks": [
                {
                    "task_number": 1,
                    "name": "Task 1",
                    "value": 2,
                    "subsections": [
                        { "name": "Correctness", "value": 1 }
                    ]
                }
            ],
            "total_value": 2
        });

        let req = Request::builder()
            .method("PUT")
            .uri(&uri)
            .header("Authorization", format!("Bearer {}", token))
            .header("Content-Type", "application/json")
            .body(Body::from(bad_payload.to_string()))
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    #[serial]
    async fn test_put_mark_allocator_not_found_on_nonexistent() {
        use serde_json::json;

        let (app, app_state, _tmp) = make_test_app_with_storage().await;
        let data = setup_test_data(app_state.db()).await;

        let (token, _) = generate_jwt(data.lecturer_user.id, data.lecturer_user.admin);
        let uri = format!(
            "/api/modules/{}/assignments/{}/mark_allocator",
            data.module.id, 9999
        );

        let payload = json!({
            "generated_at": Utc::now().to_rfc3339(),
            "tasks": [
                {
                    "task_number": 1,
                    "name": "Task 1",
                    "value": 1,
                    "subsections": [
                        { "name": "Correctness", "value": 1 }
                    ]
                }
            ],
            "total_value": 1
        });

        let req = Request::builder()
            .uri(&uri)
            .method("PUT")
            .header("Authorization", format!("Bearer {}", token))
            .header("Content-Type", "application/json")
            .body(Body::from(payload.to_string()))
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    #[serial]
    async fn test_put_mark_allocator_forbidden_for_student() {
        use serde_json::json;

        let (app, app_state, _tmp) = make_test_app_with_storage().await;
        let data = setup_test_data(app_state.db()).await;

        let (token, _) = generate_jwt(data.student_user.id, data.student_user.admin);
        let uri = format!(
            "/api/modules/{}/assignments/{}/mark_allocator",
            data.module.id, data.assignment.id
        );

        let payload = json!({
            "generated_at": Utc::now().to_rfc3339(),
            "tasks": [
                {
                    "task_number": 1,
                    "name": "Task 1",
                    "value": 1,
                    "subsections": [
                        { "name": "Correctness", "value": 1 }
                    ]
                }
            ],
            "total_value": 1
        });

        let req = Request::builder()
            .method("PUT")
            .uri(&uri)
            .header("Authorization", format!("Bearer {}", token))
            .header("Content-Type", "application/json")
            .body(Body::from(payload.to_string()))
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::FORBIDDEN);
    }

    #[tokio::test]
    #[serial]
    async fn test_put_mark_allocator_forbidden_for_unassigned_user() {
        use serde_json::json;

        let (app, app_state, _tmp) = make_test_app_with_storage().await;
        let data = setup_test_data(app_state.db()).await;

        let (token, _) = generate_jwt(data.forbidden_user.id, data.forbidden_user.admin);
        let uri = format!(
            "/api/modules/{}/assignments/{}/mark_allocator",
            data.module.id, data.assignment.id
        );

        let payload = json!({
            "generated_at": Utc::now().to_rfc3339(),
            "tasks": [
                {
                    "task_number": 1,
                    "name": "Task 1",
                    "value": 1,
                    "subsections": [
                        { "name": "Correctness", "value": 1 }
                    ]
                }
            ],
            "total_value": 1
        });

        let req = Request::builder()
            .method("PUT")
            .uri(&uri)
            .header("Authorization", format!("Bearer {}", token))
            .header("Content-Type", "application/json")
            .body(Body::from(payload.to_string()))
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::FORBIDDEN);
    }

    #[tokio::test]
    #[serial]
    async fn test_put_mark_allocator_unauthorized() {
        use serde_json::json;

        let (app, app_state, _tmp) = make_test_app_with_storage().await;
        let data = setup_test_data(app_state.db()).await;

        let uri = format!(
            "/api/modules/{}/assignments/{}/mark_allocator",
            data.module.id, data.assignment.id
        );

        let payload = json!({
            "generated_at": Utc::now().to_rfc3339(),
            "tasks": [
                {
                    "task_number": 1,
                    "name": "Task 1",
                    "value": 1,
                    "subsections": [
                        { "name": "Correctness", "value": 1 }
                    ]
                }
            ],
            "total_value": 1
        });

        let req = Request::builder()
            .method("PUT")
            .uri(&uri)
            .header("Content-Type", "application/json")
            .body(Body::from(payload.to_string()))
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    #[serial]
    async fn test_put_mark_allocator_invalid_json() {
        let (app, app_state, _tmp) = make_test_app_with_storage().await;
        let data = setup_test_data(app_state.db()).await;

        let (token, _) = generate_jwt(data.lecturer_user.id, data.lecturer_user.admin);
        let uri = format!(
            "/api/modules/{}/assignments/{}/mark_allocator",
            data.module.id, data.assignment.id
        );
        let req = Request::builder()
            .uri(&uri)
            .method("PUT")
            .header("Authorization", format!("Bearer {}", token))
            .header("Content-Type", "application/json")
            .body(Body::from(r#"{"invalid": "json"}"#))
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
    }

    #[tokio::test]
    #[serial]
    async fn test_put_then_get_mark_allocator() {
        use axum::body::to_bytes;
        use chrono::{DateTime, Utc};
        use serde_json::{Value, json};

        let (app, app_state, _tmp) = make_test_app_with_storage().await;
        let data = setup_test_data(app_state.db()).await;

        let (token, _) = generate_jwt(data.lecturer_user.id, data.lecturer_user.admin);
        let uri = format!(
            "/api/modules/{}/assignments/{}/mark_allocator",
            data.module.id, data.assignment.id
        );

        // PUT payload (normalized shape). We won't compare it byte-for-byte later.
        let payload = json!({
            "generated_at": Utc::now().to_rfc3339(), // "+00:00" form is fine
            "tasks": [
                {
                    "task_number": 1,
                    "name": "Task 1",
                    "value": 1,
                    "subsections": [
                        { "name": "Correctness", "value": 1 }
                    ]
                }
            ],
            "total_value": 1
        });

        // PUT
        let put_req = Request::builder()
            .uri(&uri)
            .method("PUT")
            .header("Authorization", format!("Bearer {}", token))
            .header("Content-Type", "application/json")
            .body(Body::from(payload.to_string()))
            .unwrap();
        let put_response = app.clone().oneshot(put_req).await.unwrap();
        assert_eq!(put_response.status(), StatusCode::OK);

        // GET
        let get_req = Request::builder()
            .uri(&uri)
            .method("GET")
            .header("Authorization", format!("Bearer {}", token))
            .body(Body::empty())
            .unwrap();
        let get_response = app.oneshot(get_req).await.unwrap();
        assert_eq!(get_response.status(), StatusCode::OK);

        // Parse response
        let body_bytes = to_bytes(get_response.into_body(), usize::MAX)
            .await
            .unwrap();
        let body_json: Value = serde_json::from_slice(&body_bytes).unwrap();

        // ---- Assertions that tolerate "Z" vs "+00:00" and optional null fields ----

        // Top-level structure exists
        let data = body_json.get("data").expect("missing data");
        // generated_at is a valid RFC3339 timestamp
        let ga_str = data
            .get("generated_at")
            .and_then(|v| v.as_str())
            .expect("missing generated_at");
        let _: DateTime<Utc> = ga_str.parse().expect("generated_at not RFC3339");

        // total_value
        assert_eq!(data.get("total_value").and_then(|v| v.as_i64()), Some(1));

        // tasks
        let tasks = data
            .get("tasks")
            .and_then(|v| v.as_array())
            .expect("tasks not array");
        assert_eq!(tasks.len(), 1);

        let t0 = &tasks[0];
        assert_eq!(t0.get("task_number").and_then(|v| v.as_i64()), Some(1));
        assert_eq!(t0.get("name").and_then(|v| v.as_str()), Some("Task 1"));
        assert_eq!(t0.get("value").and_then(|v| v.as_i64()), Some(1));

        // code_coverage is optional; if present, allow null or bool.
        if let Some(cc) = t0.get("code_coverage") {
            assert!(
                cc.is_null() || cc.is_boolean(),
                "code_coverage must be null or bool"
            );
        }

        // subsections
        let subs = t0
            .get("subsections")
            .and_then(|v| v.as_array())
            .expect("subsections not array");
        assert_eq!(subs.len(), 1);

        let s0 = &subs[0];
        assert_eq!(s0.get("name").and_then(|v| v.as_str()), Some("Correctness"));
        assert_eq!(s0.get("value").and_then(|v| v.as_i64()), Some(1));

        // regex/feedback are optional; if present, allow null or correct types
        if let Some(r) = s0.get("regex") {
            assert!(r.is_null() || r.is_array(), "regex must be null or array");
        }
        if let Some(fb) = s0.get("feedback") {
            assert!(
                fb.is_null() || fb.is_string(),
                "feedback must be null or string"
            );
        }
    }
}
