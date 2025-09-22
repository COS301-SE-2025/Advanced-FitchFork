#[cfg(test)]
mod tests {
    use crate::helpers::app::make_test_app_with_storage;
    use api::auth::generate_jwt;
    use axum::{
        body::Body,
        http::{Request, StatusCode},
    };
    use chrono::{TimeZone, Utc};
    use db::models::assignment_file::{FileType, Model as AssignmentFile};
    use db::{
        models::{
            assignment::{AssignmentType, Model as AssignmentModel},
            module::Model as ModuleModel,
            user::Model as UserModel,
            user_module_role::{Model as UserModuleRoleModel, Role},
        },
        repositories::user_repository::UserRepository,
    };
    use serde_json::Value;
    use services::{
        service::Service,
        user::{CreateUser, UserService},
    };
    use tower::ServiceExt;

    struct TestData {
        admin_user: UserModel,
        lecturer_user: UserModel,
        student_user: UserModel,
        forbidden_user: UserModel,
        module: ModuleModel,
        assignments: Vec<AssignmentModel>,
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

        TestData {
            admin_user,
            lecturer_user,
            student_user,
            forbidden_user,
            module,
            assignments: vec![a1, a2],
        }
    }

    #[tokio::test]
    #[serial]
    async fn test_get_config_success_as_admin() {
        let (app, app_state, _tmp) = make_test_app_with_storage().await;
        let data = setup_test_data(app_state.db()).await;

        let config = serde_json::json!({
            "execution": {
                "timeout_secs": 123,
                "max_memory": 8589934592u64,
                "max_cpus": 2,
                "max_uncompressed_size": 100_000_000u64,
                "max_processes": 256
            },
            "marking": {
                "marking_scheme": "exact",
                "feedback_scheme": "auto",
                "deliminator": "&-=-&"
            }
        });

        let config_bytes = serde_json::to_vec_pretty(&config).unwrap();

        AssignmentFile::save_file(
            db::get_connection().await,
            data.assignments[0].id,
            data.module.id,
            FileType::Config,
            "config.json",
            &config_bytes,
        )
        .await
        .unwrap();

        // Make request as admin
        let (token, _) = generate_jwt(data.admin_user.id, data.admin_user.admin);
        let uri = format!(
            "/api/modules/{}/assignments/{}/config",
            data.module.id, data.assignments[0].id
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
        assert_eq!(json["data"]["execution"]["timeout_secs"], 123);
        assert_eq!(json["data"]["marking"]["marking_scheme"], "exact");
    }

    #[tokio::test]
    #[serial]
    async fn test_get_config_success_as_lecturer() {
        let (app, app_state, _tmp) = make_test_app_with_storage().await;
        let data = setup_test_data(app_state.db()).await;

        // Save a valid config using disk-backed storage
        let config = serde_json::json!({
            "execution": {
                "timeout_secs": 10,
                "max_memory": 512_000_000u64,
                "max_cpus": 2,
                "max_uncompressed_size": 100_000_000u64,
                "max_processes": 256
            },
            "marking": {
                "marking_scheme": "exact",
                "feedback_scheme": "auto",
                "deliminator": "&-=-&"
            }
        });

        let config_bytes = serde_json::to_vec_pretty(&config).unwrap();

        AssignmentFile::save_file(
            db::get_connection().await,
            data.assignments[1].id,
            data.module.id,
            FileType::Config,
            "config.json",
            &config_bytes,
        )
        .await
        .unwrap();

        let (token, _) = generate_jwt(data.lecturer_user.id, data.lecturer_user.admin);
        let uri = format!(
            "/api/modules/{}/assignments/{}/config",
            data.module.id, data.assignments[1].id
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
        assert_eq!(json["data"]["execution"]["max_memory"], 512_000_000);
        assert_eq!(json["data"]["execution"]["timeout_secs"], 10);
        assert_eq!(json["data"]["marking"]["feedback_scheme"], "auto");
    }

    #[tokio::test]
    #[serial]
    async fn test_get_config_forbidden_for_student() {
        let (app, app_state, _tmp) = make_test_app_with_storage().await;
        let data = setup_test_data(app_state.db()).await;

        let (token, _) = generate_jwt(data.student_user.id, data.student_user.admin);
        let uri = format!(
            "/api/modules/{}/assignments/{}/config",
            data.module.id, data.assignments[0].id
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
    async fn test_get_config_forbidden_for_unassigned_user() {
        let (app, app_state, _tmp) = make_test_app_with_storage().await;
        let data = setup_test_data(app_state.db()).await;

        let (token, _) = generate_jwt(data.forbidden_user.id, data.forbidden_user.admin);
        let uri = format!(
            "/api/modules/{}/assignments/{}/config",
            data.module.id, data.assignments[0].id
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
    async fn test_get_config_not_found() {
        let (app, app_state, _tmp) = make_test_app_with_storage().await;
        let data = setup_test_data(app_state.db()).await;

        let (token, _) = generate_jwt(data.admin_user.id, data.admin_user.admin);
        let uri = format!("/api/modules/{}/assignments/9999/config", data.module.id);
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
    async fn test_get_config_unauthorized() {
        let (app, app_state, _tmp) = make_test_app_with_storage().await;
        let data = setup_test_data(app_state.db()).await;

        let uri = format!(
            "/api/modules/{}/assignments/{}/config",
            data.module.id, data.assignments[0].id
        );
        let req = Request::builder().uri(&uri).body(Body::empty()).unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    #[serial]
    async fn test_get_config_no_config_set() {
        let (app, app_state, _tmp) = make_test_app_with_storage().await;
        let data = setup_test_data(app_state.db()).await;

        let (token, _) = generate_jwt(data.admin_user.id, data.admin_user.admin);
        let uri = format!(
            "/api/modules/{}/assignments/{}/config",
            data.module.id, data.assignments[0].id
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
        assert!(json["data"].as_object().unwrap().is_empty());
    }

    #[tokio::test]
    #[serial]
    async fn test_get_config_invalid_config_format() {
        let (app, app_state, _tmp) = make_test_app_with_storage().await;
        let data = setup_test_data(app_state.db()).await;

        // Intentionally save invalid JSON (e.g., a primitive instead of object)
        let bad_json = b"12345";

        AssignmentFile::save_file(
            db::get_connection().await,
            data.assignments[0].id,
            data.module.id,
            FileType::Config,
            "config.json",
            bad_json,
        )
        .await
        .unwrap();

        let (token, _) = generate_jwt(data.admin_user.id, data.admin_user.admin);
        let uri = format!(
            "/api/modules/{}/assignments/{}/config",
            data.module.id, data.assignments[0].id
        );

        let req = Request::builder()
            .uri(&uri)
            .header("Authorization", format!("Bearer {}", token))
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK); // No longer 400

        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let json: Value = serde_json::from_slice(&body).unwrap();

        assert_eq!(json["success"], true);
        assert!(
            json["message"]
                .as_str()
                .unwrap()
                .contains("No configuration set for this assignment")
        );
        assert!(json["data"].as_object().unwrap().is_empty());
    }
}
