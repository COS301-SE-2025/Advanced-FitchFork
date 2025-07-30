#[cfg(test)]
mod tests {
    use db::{test_utils::setup_test_db, models::{user::Model as UserModel, module::Model as ModuleModel, assignment::{Model as AssignmentModel, AssignmentType}, user_module_role::{Model as UserModuleRoleModel, Role}}};
    use axum::{body::Body, http::{Request, StatusCode}};
    use tower::ServiceExt;
    use serde_json::{Value};
    use api::auth::generate_jwt;
    use dotenvy;
    use chrono::{Utc, TimeZone};
    use db::models::assignment_file::{FileType, Model as AssignmentFile};
    use crate::test_helpers::make_app;

    struct TestData {
        admin_user: UserModel,
        lecturer_user: UserModel,
        student_user: UserModel,
        forbidden_user: UserModel,
        module: ModuleModel,
        assignments: Vec<AssignmentModel>,
    }

    async fn setup_test_data(db: &sea_orm::DatabaseConnection) -> TestData {
        let module = ModuleModel::create(db, "COS101", 2024, Some("Test Module"), 16).await.unwrap();
        let admin_user = UserModel::create(db, "admin1", "admin1@test.com", "password", true).await.unwrap();
        let lecturer_user = UserModel::create(db, "lecturer1", "lecturer1@test.com", "password1", false).await.unwrap();
        let student_user = UserModel::create(db, "student1", "student1@test.com", "password2", false).await.unwrap();
        let forbidden_user = UserModel::create(db, "forbidden", "forbidden@test.com", "password3", false).await.unwrap();
        UserModuleRoleModel::assign_user_to_module(db, lecturer_user.id, module.id, Role::Lecturer).await.unwrap();
        UserModuleRoleModel::assign_user_to_module(db, student_user.id, module.id, Role::Student).await.unwrap();
        let a1 = AssignmentModel::create(
            db,
            module.id,
            "Assignment 1",
            Some("Desc 1"),
            AssignmentType::Assignment,
            Utc.with_ymd_and_hms(2024, 1, 1, 0, 0, 0).unwrap(),
            Utc.with_ymd_and_hms(2024, 1, 31, 23, 59, 59).unwrap(),
        ).await.unwrap();
        let a2 = AssignmentModel::create(
            db,
            module.id,
            "Assignment 2",
            Some("Desc 2"),
            AssignmentType::Practical,
            Utc.with_ymd_and_hms(2024, 2, 1, 0, 0, 0).unwrap(),
            Utc.with_ymd_and_hms(2024, 2, 28, 23, 59, 59).unwrap(),
        ).await.unwrap();

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
    async fn test_get_config_success_as_admin() {
        dotenvy::dotenv().expect("Failed to load .env");
        let db = setup_test_db().await;
        let data = setup_test_data(&db).await;

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
            &db,
            data.assignments[0].id,
            data.module.id,
            FileType::Config,
            "config.json",
            &config_bytes,
        )
        .await
        .unwrap();

        // Make request as admin
        let app = make_app(db.clone());
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
    async fn test_get_config_success_as_lecturer() {
        dotenvy::dotenv().expect("Failed to load .env");
        let db = setup_test_db().await;
        let data = setup_test_data(&db).await;

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
            &db,
            data.assignments[1].id,
            data.module.id,
            FileType::Config,
            "config.json",
            &config_bytes,
        )
        .await
        .unwrap();

        let app = make_app(db.clone());
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
    async fn test_get_config_forbidden_for_student() {
        dotenvy::dotenv().expect("Failed to load .env");
        let db = setup_test_db().await;
        let data = setup_test_data(&db).await;

        let app = make_app(db.clone());
        let (token, _) = generate_jwt(data.student_user.id, data.student_user.admin);
        let uri = format!("/api/modules/{}/assignments/{}/config", data.module.id, data.assignments[0].id);
        let req = Request::builder()
            .uri(&uri)
            .header("Authorization", format!("Bearer {}", token))
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::FORBIDDEN);
    }

    #[tokio::test]
    async fn test_get_config_forbidden_for_unassigned_user() {
        dotenvy::dotenv().expect("Failed to load .env");
        let db = setup_test_db().await;
        let data = setup_test_data(&db).await;

        let app = make_app(db.clone());
        let (token, _) = generate_jwt(data.forbidden_user.id, data.forbidden_user.admin);
        let uri = format!("/api/modules/{}/assignments/{}/config", data.module.id, data.assignments[0].id);
        let req = Request::builder()
            .uri(&uri)
            .header("Authorization", format!("Bearer {}", token))
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::FORBIDDEN);
    }

    #[tokio::test]
    async fn test_get_config_not_found() {
        dotenvy::dotenv().expect("Failed to load .env");
        let db = setup_test_db().await;
        let data = setup_test_data(&db).await;

        let app = make_app(db.clone());
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
    async fn test_get_config_unauthorized() {
        dotenvy::dotenv().expect("Failed to load .env");
        let db = setup_test_db().await;
        let data = setup_test_data(&db).await;

        let app = make_app(db.clone());
        let uri = format!("/api/modules/{}/assignments/{}/config", data.module.id, data.assignments[0].id);
        let req = Request::builder()
            .uri(&uri)
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn test_get_config_no_config_set() {
        dotenvy::dotenv().expect("Failed to load .env");
        let db = setup_test_db().await;
        let data = setup_test_data(&db).await;

        let app = make_app(db.clone());
        let (token, _) = generate_jwt(data.admin_user.id, data.admin_user.admin);
        let uri = format!("/api/modules/{}/assignments/{}/config", data.module.id, data.assignments[0].id);
        let req = Request::builder()
            .uri(&uri)
            .header("Authorization", format!("Bearer {}", token))
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let json: Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(json["success"], true);
        assert!(json["data"].as_object().unwrap().is_empty());
    }

    #[tokio::test]
    async fn test_get_config_invalid_config_format() {
        dotenvy::dotenv().expect("Failed to load .env");
        let db = setup_test_db().await;
        let data = setup_test_data(&db).await;

        // Intentionally save invalid JSON (e.g., a primitive instead of object)
        let bad_json = b"12345";

        AssignmentFile::save_file(
            &db,
            data.assignments[0].id,
            data.module.id,
            FileType::Config,
            "config.json",
            bad_json,
        )
        .await
        .unwrap();

        let app = make_app(db.clone());
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

        let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let json: Value = serde_json::from_slice(&body).unwrap();

        assert_eq!(json["success"], true);
        assert!(json["message"]
            .as_str()
            .unwrap()
            .contains("No configuration set for this assignment"));
        assert!(json["data"].as_object().unwrap().is_empty());
    }
}