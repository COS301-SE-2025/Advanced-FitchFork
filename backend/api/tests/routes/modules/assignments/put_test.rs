#[cfg(test)]
mod tests {
    use db::{test_utils::setup_test_db, models::{user::Model as UserModel, module::{Model as ModuleModel, ActiveModel as ModuleActiveModel}, assignment::{Model as AssignmentModel, AssignmentType, Status}, user_module_role::{Model as UserModuleRoleModel, Role}}};
    use axum::{body::Body, http::{Request, StatusCode}};
    use tower::ServiceExt;
    use serde_json::{json, Value};
    use api::auth::generate_jwt;
    use dotenvy;
    use chrono::{Utc, TimeZone, DateTime};
    use sea_orm::{Set, ActiveModelTrait, EntityTrait};
    use crate::test_helpers::make_app;

    struct TestData {
        admin_user: UserModel,
        lecturer_user: UserModel,
        student_user: UserModel,
        forbidden_user: UserModel,
        module: ModuleModel,
        empty_module: ModuleModel,
        assignments: Vec<AssignmentModel>,
        dummy_module_id: i64,
    }

    async fn setup_test_data(db: &sea_orm::DatabaseConnection) -> TestData {
        dotenvy::dotenv().expect("Failed to load .env");

        let module = ModuleModel::create(db, "COS101", 2024, Some("Test Module"), 16).await.unwrap();
        let empty_module = ModuleModel::create(&db, "EMPTY101", 2024, Some("Empty Module"), 16).await.unwrap();
        let admin_user = UserModel::create(db, "admin1", "admin1@test.com", "password", true).await.unwrap();
        let lecturer_user = UserModel::create(db, "lecturer1", "lecturer1@test.com", "password1", false).await.unwrap();
        let student_user = UserModel::create(db, "student1", "student1@test.com", "password2", false).await.unwrap();
        let forbidden_user = UserModel::create(db, "forbidden", "forbidden@test.com", "password3", false).await.unwrap();
        UserModuleRoleModel::assign_user_to_module(db, lecturer_user.id, module.id, Role::Lecturer).await.unwrap();
        UserModuleRoleModel::assign_user_to_module(db, lecturer_user.id, empty_module.id, Role::Lecturer).await.unwrap();
        UserModuleRoleModel::assign_user_to_module(db, student_user.id, module.id, Role::Student).await.unwrap();
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
        UserModuleRoleModel::assign_user_to_module(db, lecturer_user.id, dummy_module.id, Role::Lecturer).await.unwrap();
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
        let a3 = AssignmentModel::create(
            db,
            module.id,
            "Assignment 3",
            Some("Desc 3"),
            AssignmentType::Assignment,
            Utc.with_ymd_and_hms(2024, 3, 1, 0, 0, 0).unwrap(),
            Utc.with_ymd_and_hms(2024, 3, 31, 23, 59, 59).unwrap(),
        ).await.unwrap();

        TestData {
            admin_user,
            lecturer_user,
            student_user,
            forbidden_user,
            module,
            empty_module,
            assignments: vec![a1, a2, a3],
            dummy_module_id: dummy_module.id,
        }
    }

    #[tokio::test]
    async fn test_put_assignment_shouldnt_update_status() {
        let db = setup_test_db().await;
        let data = setup_test_data(&db).await;

        let app = make_app(db.clone());
        let (token, _) = generate_jwt(data.admin_user.id, data.admin_user.admin);
        let uri = format!("/api/modules/{}/assignments/{}", data.module.id, data.assignments[1].id);
        let body = json!({
            "name": "Admin Updated",
            "description": "Admin update desc",
            "assignment_type": "practical",
            "status": "ready",
            "available_from": "2024-02-05T00:00:00Z",
            "due_date": "2024-02-25T23:59:59Z"
        });
        let req = Request::builder()
            .method("PUT")
            .uri(&uri)
            .header("Authorization", format!("Bearer {}", token))
            .header("Content-Type", "application/json")
            .body(Body::from(body.to_string()))
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(json["success"], true);
        assert_eq!(json["data"]["name"], "Admin Updated");
        assert_eq!(json["data"]["status"], "setup");
    }

    #[tokio::test]
    async fn test_put_assignment_optional_description_none() {
        let db = setup_test_db().await;
        let data = setup_test_data(&db).await;

        let app = make_app(db.clone());
        let (token, _) = generate_jwt(data.lecturer_user.id, data.lecturer_user.admin);
        let uri = format!("/api/modules/{}/assignments/{}", data.module.id, data.assignments[2].id);
        let body = json!({
            "name": "No Desc",
            "description": null,
            "assignment_type": "assignment",
            "status": "closed",
            "available_from": "2024-03-05T00:00:00Z",
            "due_date": "2024-03-25T23:59:59Z"
        });
        let req = Request::builder()
            .method("PUT")
            .uri(&uri)
            .header("Authorization", format!("Bearer {}", token))
            .header("Content-Type", "application/json")
            .body(Body::from(body.to_string()))
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(json["success"], true);
        assert_eq!(json["data"]["description"], serde_json::Value::Null);
    }

    #[tokio::test]
    async fn test_put_assignment_success_status_variants() {
        let db = setup_test_db().await;
        let data = setup_test_data(&db).await;

        let app = make_app(db.clone());
        let (token, _) = generate_jwt(data.lecturer_user.id, data.lecturer_user.admin);
        let statuses = ["setup", "ready", "open", "closed", "archived"];
        for status in statuses.iter() {
            let body = json!({
                "name": "Status Test",
                "description": "Status variant",
                "assignment_type": "assignment",
                "status": status,
                "available_from": "2024-01-01T00:00:00Z",
                "due_date": "2024-01-31T23:59:59Z"
            });
            let uri = format!("/api/modules/{}/assignments/{}", data.module.id, data.assignments[0].id);
            let req = Request::builder()
                .method("PUT")
                .uri(&uri)
                .header("Authorization", format!("Bearer {}", token))
                .header("Content-Type", "application/json")
                .body(Body::from(body.to_string()))
                .unwrap();

            let response = app.clone().oneshot(req).await.unwrap();
            assert_eq!(response.status(), StatusCode::OK);
        }
    }

    #[tokio::test]
    async fn test_put_assignment_forbidden_for_student() {
        let db = setup_test_db().await;
        let data = setup_test_data(&db).await;

        let app = make_app(db.clone());
        let (token, _) = generate_jwt(data.student_user.id, data.student_user.admin);
        let uri = format!("/api/modules/{}/assignments/{}", data.module.id, data.assignments[0].id);
        let body = json!({
            "name": "Student Update",
            "description": "Should not be allowed",
            "assignment_type": "assignment",
            "status": "setup",
            "available_from": "2024-01-01T00:00:00Z",
            "due_date": "2024-01-31T23:59:59Z"
        });
        let req = Request::builder()
            .method("PUT")
            .uri(&uri)
            .header("Authorization", format!("Bearer {}", token))
            .header("Content-Type", "application/json")
            .body(Body::from(body.to_string()))
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::FORBIDDEN);
    }

    #[tokio::test]
    async fn test_put_assignment_forbidden_for_unassigned_user() {
        let db = setup_test_db().await;
        let data = setup_test_data(&db).await;

        let app = make_app(db.clone());
        let (token, _) = generate_jwt(data.forbidden_user.id, data.forbidden_user.admin);
        let uri = format!("/api/modules/{}/assignments/{}", data.module.id, data.assignments[0].id);
        let body = json!({
            "name": "Forbidden Update",
            "description": "Should not be allowed",
            "assignment_type": "assignment",
            "status": "setup",
            "available_from": "2024-01-01T00:00:00Z",
            "due_date": "2024-01-31T23:59:59Z"
        });
        let req = Request::builder()
            .method("PUT")
            .uri(&uri)
            .header("Authorization", format!("Bearer {}", token))
            .header("Content-Type", "application/json")
            .body(Body::from(body.to_string()))
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::FORBIDDEN);
    }

    #[tokio::test]
    async fn test_put_assignment_invalid_assignment_type() {
        let db = setup_test_db().await;
        let data = setup_test_data(&db).await;

        let app = make_app(db.clone());
        let (token, _) = generate_jwt(data.lecturer_user.id, data.lecturer_user.admin);
        let uri = format!("/api/modules/{}/assignments/{}", data.module.id, data.assignments[0].id);
        let body = json!({
            "name": "Invalid Type",
            "description": "Invalid assignment_type",
            "assignment_type": "invalid_type",
            "status": "setup",
            "available_from": "2024-01-01T00:00:00Z",
            "due_date": "2024-01-31T23:59:59Z"
        });
        let req = Request::builder()
            .method("PUT")
            .uri(&uri)
            .header("Authorization", format!("Bearer {}", token))
            .header("Content-Type", "application/json")
            .body(Body::from(body.to_string()))
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(json["success"], false);
        assert!(json["message"].as_str().unwrap().contains("assignment_type"));
    }

    #[tokio::test]
    async fn test_put_assignment_invalid_dates() {
        let db = setup_test_db().await;
        let data = setup_test_data(&db).await;

        let app = make_app(db.clone());
        let (token, _) = generate_jwt(data.lecturer_user.id, data.lecturer_user.admin);
        let uri = format!("/api/modules/{}/assignments/{}", data.module.id, data.assignments[0].id);
        let body = json!({
            "name": "Bad Dates",
            "description": "Invalid dates",
            "assignment_type": "assignment",
            "status": "setup",
            "available_from": "not-a-date",
            "due_date": "also-not-a-date"
        });
        let req = Request::builder()
            .method("PUT")
            .uri(&uri)
            .header("Authorization", format!("Bearer {}", token))
            .header("Content-Type", "application/json")
            .body(Body::from(body.to_string()))
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(json["success"], false);
        assert!(json["message"].as_str().unwrap().contains("datetime"));
    }

    #[tokio::test]
    async fn test_put_assignment_missing_fields() {
        let db = setup_test_db().await;
        let data = setup_test_data(&db).await;

        let app = make_app(db.clone());
        let (token, _) = generate_jwt(data.lecturer_user.id, data.lecturer_user.admin);
        let uri = format!("/api/modules/{}/assignments/{}", data.module.id, data.assignments[0].id);
        let body = json!({
            "name": "Missing Fields"
        });
        let req = Request::builder()
            .method("PUT")
            .uri(&uri)
            .header("Authorization", format!("Bearer {}", token))
            .header("Content-Type", "application/json")
            .body(Body::from(body.to_string()))
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert!(response.status() == StatusCode::UNPROCESSABLE_ENTITY);
    }

    #[tokio::test]
    async fn test_put_assignment_not_found() {
        let db = setup_test_db().await;
        let data = setup_test_data(&db).await;

        let app = make_app(db.clone());
        let (token, _) = generate_jwt(data.lecturer_user.id, data.lecturer_user.admin);
        let uri = format!("/api/modules/{}/assignments/9999", data.module.id);
        let body = json!({
            "name": "Not Found",
            "description": "Should not exist",
            "assignment_type": "assignment",
            "status": "setup",
            "available_from": "2024-01-01T00:00:00Z",
            "due_date": "2024-01-31T23:59:59Z"
        });
        let req = Request::builder()
            .method("PUT")
            .uri(&uri)
            .header("Authorization", format!("Bearer {}", token))
            .header("Content-Type", "application/json")
            .body(Body::from(body.to_string()))
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn test_put_assignment_wrong_module() {
        let db = setup_test_db().await;
        let data = setup_test_data(&db).await;

        let (token, _) = generate_jwt(data.lecturer_user.id, data.lecturer_user.admin);
        let app = make_app(db.clone());
        let uri = format!("/api/modules/{}/assignments/{}", data.empty_module.id, data.assignments[0].id);
        let body = json!({
            "name": "Wrong Module",
            "description": "Assignment not in this module",
            "assignment_type": "assignment",
            "status": "setup",
            "available_from": "2024-01-01T00:00:00Z",
            "due_date": "2024-01-31T23:59:59Z"
        });
        let req = Request::builder()
            .method("PUT")
            .uri(&uri)
            .header("Authorization", format!("Bearer {}", token))
            .header("Content-Type", "application/json")
            .body(Body::from(body.to_string()))
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn test_put_assignment_module_not_found() {
        let db = setup_test_db().await;
        let data = setup_test_data(&db).await;

        let (token, _) = generate_jwt(data.lecturer_user.id, data.lecturer_user.admin);
        let app = make_app(db.clone());
        let uri = format!("/api/modules/{}/assignments/{}", data.dummy_module_id, data.assignments[0].id);
        let body = json!({
            "name": "Module 9999 not found.",
            "description": "Module does not exist",
            "assignment_type": "assignment",
            "status": "setup",
            "available_from": "2024-01-01T00:00:00Z",
            "due_date": "2024-01-31T23:59:59Z"
        });
        let req = Request::builder()
            .method("PUT")
            .uri(&uri)
            .header("Authorization", format!("Bearer {}", token))
            .header("Content-Type", "application/json")
            .body(Body::from(body.to_string()))
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn test_put_assignment_unauthorized() {
        let db = setup_test_db().await;
        let data = setup_test_data(&db).await;

        let app = make_app(db.clone());
        let uri = format!("/api/modules/{}/assignments/{}", data.module.id, data.assignments[0].id);
        let body = json!({
            "name": "No Auth",
            "description": "No token",
            "assignment_type": "assignment",
            "status": "setup",
            "available_from": "2024-01-01T00:00:00Z",
            "due_date": "2024-01-31T23:59:59Z"
        });
        let req = Request::builder()
            .method("PUT")
            .uri(&uri)
            .header("Content-Type", "application/json")
            .body(Body::from(body.to_string()))
            .unwrap();
        
        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    /// Test Case: Successful Transition from Ready to Open
    #[tokio::test]
    async fn test_open_assignment_success_from_ready() {
        let db = setup_test_db().await;
        let data = setup_test_data(&db).await;

        let mut active_assignment: db::models::assignment::ActiveModel = data.assignments[0].clone().into();
        active_assignment.status = Set(Status::Ready);
        active_assignment.updated_at = Set(Utc::now());
        let assignment = active_assignment.update(&db).await.unwrap();

        let app = make_app(db.clone());
        let (token, _) = generate_jwt(data.lecturer_user.id, data.lecturer_user.admin);
        let uri = format!("/api/modules/{}/assignments/{}/open", data.module.id, assignment.id);
        let req = Request::builder()
            .method("PUT")
            .uri(&uri)
            .header("Authorization", format!("Bearer {}", token))
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);

        let updated = db::models::assignment::Entity::find_by_id(assignment.id)
            .one(&db)
            .await
            .unwrap()
            .unwrap();
        assert_eq!(updated.status, Status::Open);
    }

    /// Test Case: Successful Transition from Closed to Open
    #[tokio::test]
    async fn test_open_assignment_success_from_closed() {
        let db = setup_test_db().await;
        let data = setup_test_data(&db).await;

        let mut active_assignment: db::models::assignment::ActiveModel = data.assignments[0].clone().into();
        active_assignment.status = Set(Status::Closed);
        active_assignment.updated_at = Set(Utc::now());
        let assignment = active_assignment.update(&db).await.unwrap();

        let app = make_app(db.clone());
        let (token, _) = generate_jwt(data.admin_user.id, data.admin_user.admin);
        let uri = format!("/api/modules/{}/assignments/{}/open", data.module.id, assignment.id);
        let req = Request::builder()
            .method("PUT")
            .uri(&uri)
            .header("Authorization", format!("Bearer {}", token))
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
    }

    /// Test Case: Successful Transition from Archived to Open
    #[tokio::test]
    async fn test_open_assignment_success_from_archived() {
        let db = setup_test_db().await;
        let data = setup_test_data(&db).await;

        let mut active_assignment: db::models::assignment::ActiveModel = data.assignments[0].clone().into();
        active_assignment.status = Set(Status::Archived);
        active_assignment.updated_at = Set(Utc::now());
        let assignment = active_assignment.update(&db).await.unwrap();

        let app = make_app(db.clone());
        let (token, _) = generate_jwt(data.lecturer_user.id, data.lecturer_user.admin);
        let uri = format!("/api/modules/{}/assignments/{}/open", data.module.id, assignment.id);
        let req = Request::builder()
            .method("PUT")
            .uri(&uri)
            .header("Authorization", format!("Bearer {}", token))
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
    }

    /// Test Case: Forbidden When Already Open
    #[tokio::test]
    async fn test_open_assignment_already_open() {
        let db = setup_test_db().await;
        let data = setup_test_data(&db).await;

        let mut active_assignment: db::models::assignment::ActiveModel = data.assignments[0].clone().into();
        active_assignment.status = Set(Status::Open);
        active_assignment.updated_at = Set(Utc::now());
        let assignment = active_assignment.update(&db).await.unwrap();

        let app = make_app(db.clone());
        let (token, _) = generate_jwt(data.lecturer_user.id, data.lecturer_user.admin);
        let uri = format!("/api/modules/{}/assignments/{}/open", data.module.id, assignment.id);
        let req = Request::builder()
            .method("PUT")
            .uri(&uri)
            .header("Authorization", format!("Bearer {}", token))
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    /// Test Case: Forbidden for Student
    #[tokio::test]
    async fn test_open_assignment_forbidden_student() {
        let db = setup_test_db().await;
        let data = setup_test_data(&db).await;

        let mut active_assignment: db::models::assignment::ActiveModel = data.assignments[0].clone().into();
        active_assignment.status = Set(Status::Ready);
        active_assignment.updated_at = Set(Utc::now());
        let assignment = active_assignment.update(&db).await.unwrap();

        let app = make_app(db.clone());
        let (token, _) = generate_jwt(data.student_user.id, data.student_user.admin);
        let uri = format!("/api/modules/{}/assignments/{}/open", data.module.id, assignment.id);
        let req = Request::builder()
            .method("PUT")
            .uri(&uri)
            .header("Authorization", format!("Bearer {}", token))
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::FORBIDDEN);
    }

    /// Test Case: Forbidden When in Setup State
    #[tokio::test]
    async fn test_open_assignment_invalid_setup_state() {
        let db = setup_test_db().await;
        let data = setup_test_data(&db).await;

        let assignment = data.assignments[0].clone();
        assert_eq!(assignment.status, Status::Setup);

        let app = make_app(db.clone());
        let (token, _) = generate_jwt(data.lecturer_user.id, data.lecturer_user.admin);
        let uri = format!("/api/modules/{}/assignments/{}/open", data.module.id, assignment.id);
        let req = Request::builder()
            .method("PUT")
            .uri(&uri)
            .header("Authorization", format!("Bearer {}", token))
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    /// Test Case: Not Found for Wrong Module
    #[tokio::test]
    async fn test_open_assignment_wrong_module() {
        let db = setup_test_db().await;
        let data = setup_test_data(&db).await;

        let mut active_assignment: db::models::assignment::ActiveModel = data.assignments[0].clone().into();
        active_assignment.status = Set(Status::Ready);
        active_assignment.updated_at = Set(Utc::now());
        let assignment = active_assignment.update(&db).await.unwrap();

        let app = make_app(db.clone());
        let (token, _) = generate_jwt(data.lecturer_user.id, data.lecturer_user.admin);
        let uri = format!("/api/modules/{}/assignments/{}/open", data.empty_module.id, assignment.id);
        let req = Request::builder()
            .method("PUT")
            .uri(&uri)
            .header("Authorization", format!("Bearer {}", token))
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    /// Test Case: Unauthorized Access
    #[tokio::test]
    async fn test_open_assignment_unauthorized() {
        let db = setup_test_db().await;
        let data = setup_test_data(&db).await;

        let mut active_assignment: db::models::assignment::ActiveModel = data.assignments[0].clone().into();
        active_assignment.status = Set(Status::Ready);
        active_assignment.updated_at = Set(Utc::now());
        let assignment = active_assignment.update(&db).await.unwrap();

        let app = make_app(db.clone());
        let uri = format!("/api/modules/{}/assignments/{}/open", data.module.id, assignment.id);
        let req = Request::builder()
            .method("PUT")
            .uri(&uri)
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    /// Test Case: Admin Can Open Without Module Role
    #[tokio::test]
    async fn test_open_assignment_admin_without_role() {
        let db = setup_test_db().await;
        let data = setup_test_data(&db).await;

        let mut active_assignment: db::models::assignment::ActiveModel = data.assignments[0].clone().into();
        active_assignment.status = Set(Status::Ready);
        active_assignment.updated_at = Set(Utc::now());
        let assignment = active_assignment.update(&db).await.unwrap();

        let app = make_app(db.clone());
        let (token, _) = generate_jwt(data.admin_user.id, data.admin_user.admin);
        let uri = format!("/api/modules/{}/assignments/{}/open", data.module.id, assignment.id);
        let req = Request::builder()
            .method("PUT")
            .uri(&uri)
            .header("Authorization", format!("Bearer {}", token))
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
    }

    /// Test Case: Assignment Not Found
    #[tokio::test]
    async fn test_open_assignment_not_found() {
        let db = setup_test_db().await;
        let data = setup_test_data(&db).await;

        let app = make_app(db.clone());
        let (token, _) = generate_jwt(data.lecturer_user.id, data.lecturer_user.admin);
        let uri = format!("/api/modules/{}/assignments/9999/open", data.module.id);
        let req = Request::builder()
            .method("PUT")
            .uri(&uri)
            .header("Authorization", format!("Bearer {}", token))
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    /// Test Case: Successful Transition from Open to Closed (Lecturer)
    #[tokio::test]
    async fn test_close_assignment_success_lecturer() {
        let db = setup_test_db().await;
        let data = setup_test_data(&db).await;

        let mut active_assignment: db::models::assignment::ActiveModel = data.assignments[0].clone().into();
        active_assignment.status = Set(Status::Open);
        active_assignment.updated_at = Set(Utc::now());
        let assignment = active_assignment.update(&db).await.unwrap();

        let app = make_app(db.clone());
        let (token, _) = generate_jwt(data.lecturer_user.id, data.lecturer_user.admin);
        let uri = format!("/api/modules/{}/assignments/{}/close", data.module.id, assignment.id);
        let req = Request::builder()
            .method("PUT")
            .uri(&uri)
            .header("Authorization", format!("Bearer {}", token))
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);

        let updated = db::models::assignment::Entity::find_by_id(assignment.id)
            .one(&db)
            .await
            .unwrap()
            .unwrap();
        assert_eq!(updated.status, Status::Closed);
    }

    /// Test Case: Successful Transition from Open to Closed (Admin)
    #[tokio::test]
    async fn test_close_assignment_success_admin() {
        let db = setup_test_db().await;
        let data = setup_test_data(&db).await;

        let mut active_assignment: db::models::assignment::ActiveModel = data.assignments[0].clone().into();
        active_assignment.status = Set(Status::Open);
        active_assignment.updated_at = Set(Utc::now());
        let assignment = active_assignment.update(&db).await.unwrap();

        let app = make_app(db.clone());
        let (token, _) = generate_jwt(data.admin_user.id, data.admin_user.admin);
        let uri = format!("/api/modules/{}/assignments/{}/close", data.module.id, assignment.id);
        let req = Request::builder()
            .method("PUT")
            .uri(&uri)
            .header("Authorization", format!("Bearer {}", token))
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
    }

    /// Test Case: Forbidden When Not Open
    #[tokio::test]
    async fn test_close_assignment_invalid_state() {
        let db = setup_test_db().await;
        let data = setup_test_data(&db).await;
        let app = make_app(db.clone());
        let (token, _) = generate_jwt(data.lecturer_user.id, data.lecturer_user.admin);

        for state in &[Status::Setup, Status::Ready, Status::Closed, Status::Archived] {
            let mut active_assignment: db::models::assignment::ActiveModel = data.assignments[0].clone().into();
            active_assignment.status = Set(state.clone());
            active_assignment.updated_at = Set(Utc::now());
            let assignment = active_assignment.update(&db).await.unwrap();

            let uri = format!("/api/modules/{}/assignments/{}/close", data.module.id, assignment.id);
            let req = Request::builder()
                .method("PUT")
                .uri(&uri)
                .header("Authorization", format!("Bearer {}", token))
                .body(Body::empty())
                .unwrap();

            let response = app.clone().oneshot(req).await.unwrap();
            assert_eq!(response.status(), StatusCode::BAD_REQUEST);

            let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
            let json: Value = serde_json::from_slice(&body).unwrap();
            assert_eq!(json["message"], "Assignment can only be closed if it is in Open state");
        }
    }

    /// Test Case: Forbidden for Student
    #[tokio::test]
    async fn test_close_assignment_forbidden_student() {
        let db = setup_test_db().await;
        let data = setup_test_data(&db).await;

        let mut active_assignment: db::models::assignment::ActiveModel = data.assignments[0].clone().into();
        active_assignment.status = Set(Status::Open);
        active_assignment.updated_at = Set(Utc::now());
        let assignment = active_assignment.update(&db).await.unwrap();

        let app = make_app(db.clone());
        let (token, _) = generate_jwt(data.student_user.id, data.student_user.admin);
        let uri = format!("/api/modules/{}/assignments/{}/close", data.module.id, assignment.id);
        let req = Request::builder()
            .method("PUT")
            .uri(&uri)
            .header("Authorization", format!("Bearer {}", token))
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::FORBIDDEN);
    }

    /// Test Case: Not Found for Wrong Module
    #[tokio::test]
    async fn test_close_assignment_wrong_module() {
        let db = setup_test_db().await;
        let data = setup_test_data(&db).await;

        let mut active_assignment: db::models::assignment::ActiveModel = data.assignments[0].clone().into();
        active_assignment.status = Set(Status::Open);
        active_assignment.updated_at = Set(Utc::now());
        let assignment = active_assignment.update(&db).await.unwrap();

        let app = make_app(db.clone());
        let (token, _) = generate_jwt(data.lecturer_user.id, data.lecturer_user.admin);
        let uri = format!("/api/modules/{}/assignments/{}/close", data.empty_module.id, assignment.id);
        let req = Request::builder()
            .method("PUT")
            .uri(&uri)
            .header("Authorization", format!("Bearer {}", token))
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    /// Test Case: Unauthorized Access
    #[tokio::test]
    async fn test_close_assignment_unauthorized() {
        let db = setup_test_db().await;
        let data = setup_test_data(&db).await;

        let mut active_assignment: db::models::assignment::ActiveModel = data.assignments[0].clone().into();
        active_assignment.status = Set(Status::Open);
        active_assignment.updated_at = Set(Utc::now());
        let assignment = active_assignment.update(&db).await.unwrap();

        let app = make_app(db.clone());
        let uri = format!("/api/modules/{}/assignments/{}/close", data.module.id, assignment.id);
        let req = Request::builder()
            .method("PUT")
            .uri(&uri)
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    /// Test Case: Admin Can Close Without Module Role
    #[tokio::test]
    async fn test_close_assignment_admin_without_role() {
        let db = setup_test_db().await;
        let data = setup_test_data(&db).await;

        let mut active_assignment: db::models::assignment::ActiveModel = data.assignments[0].clone().into();
        active_assignment.status = Set(Status::Open);
        active_assignment.updated_at = Set(Utc::now());
        let assignment = active_assignment.update(&db).await.unwrap();

        let app = make_app(db.clone());
        let (token, _) = generate_jwt(data.admin_user.id, data.admin_user.admin);
        let uri = format!("/api/modules/{}/assignments/{}/close", data.module.id, assignment.id);
        let req = Request::builder()
            .method("PUT")
            .uri(&uri)
            .header("Authorization", format!("Bearer {}", token))
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
    }

    /// Test Case: Assignment Not Found
    #[tokio::test]
    async fn test_close_assignment_not_found() {
        let db = setup_test_db().await;
        let data = setup_test_data(&db).await;

        let app = make_app(db.clone());
        let (token, _) = generate_jwt(data.lecturer_user.id, data.lecturer_user.admin);
        let uri = format!("/api/modules/{}/assignments/9999/close", data.module.id);
        let req = Request::builder()
            .method("PUT")
            .uri(&uri)
            .header("Authorization", format!("Bearer {}", token))
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    /// Test Case: Forbidden for Unassigned User
    #[tokio::test]
    async fn test_close_assignment_forbidden_unassigned() {
        let db = setup_test_db().await;
        let data = setup_test_data(&db).await;

        let mut active_assignment: db::models::assignment::ActiveModel = data.assignments[0].clone().into();
        active_assignment.status = Set(Status::Open);
        active_assignment.updated_at = Set(Utc::now());
        let assignment = active_assignment.update(&db).await.unwrap();

        let app = make_app(db.clone());
        let (token, _) = generate_jwt(data.forbidden_user.id, data.forbidden_user.admin);
        let uri = format!("/api/modules/{}/assignments/{}/close", data.module.id, assignment.id);
        let req = Request::builder()
            .method("PUT")
            .uri(&uri)
            .header("Authorization", format!("Bearer {}", token))
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::FORBIDDEN);
    }

    /// Test Case: Module Not Found
    #[tokio::test]
    async fn test_close_assignment_module_not_found() {
        let db = setup_test_db().await;
        let data = setup_test_data(&db).await;

        let mut active_assignment: db::models::assignment::ActiveModel = data.assignments[0].clone().into();
        active_assignment.status = Set(Status::Open);
        active_assignment.updated_at = Set(Utc::now());
        let assignment = active_assignment.update(&db).await.unwrap();

        let app = make_app(db.clone());
        let (token, _) = generate_jwt(data.lecturer_user.id, data.lecturer_user.admin);
        let uri = format!("/api/modules/99999/assignments/{}/close", assignment.id);
        let req = Request::builder()
            .method("PUT")
            .uri(&uri)
            .header("Authorization", format!("Bearer {}", token))
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    /// Test Case: Successful Bulk Update by Lecturer
    #[tokio::test]
    async fn test_bulk_update_assignments_success_lecturer() {
        let db = setup_test_db().await;
        let data = setup_test_data(&db).await;

        let app = make_app(db.clone());
        let (token, _) = generate_jwt(data.lecturer_user.id, data.lecturer_user.admin);
        let uri = format!("/api/modules/{}/assignments/bulk", data.module.id);

        let ids_to_update = vec![data.assignments[0].id, data.assignments[1].id];
        let new_available_from = DateTime::parse_from_rfc3339("2025-01-01T00:00:00Z").unwrap().with_timezone(&Utc);
        let new_due_date = DateTime::parse_from_rfc3339("2025-02-01T00:00:00Z").unwrap().with_timezone(&Utc);
        
        let req_body = json!({
            "assignment_ids": ids_to_update,
            "available_from": new_available_from,
            "due_date": new_due_date
        });
        let req = Request::builder()
            .method("PUT")
            .uri(&uri)
            .header("Authorization", format!("Bearer {}", token))
            .header("Content-Type", "application/json")
            .body(Body::from(serde_json::to_string(&req_body).unwrap()))
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let json: Value = serde_json::from_slice(&body).unwrap();
        
        assert_eq!(json["success"], true);
        assert_eq!(json["message"], "Updated 2/2 assignments");
        assert_eq!(json["data"]["updated"], 2);
        assert!(json["data"]["failed"].as_array().unwrap().is_empty());

        for id in ids_to_update {
            let assignment = db::models::assignment::Entity::find_by_id(id)
                .one(&db)
                .await
                .unwrap()
                .unwrap();
                
            assert_eq!(
                assignment.available_from,
                new_available_from
            );
            assert_eq!(
                assignment.due_date,
                new_due_date
            );
        }
    }

    /// Test Case: Successful Bulk Update by Admin
    #[tokio::test]
    async fn test_bulk_update_assignments_success_admin() {
        let db = setup_test_db().await;
        let data = setup_test_data(&db).await;

        let app = make_app(db.clone());
        let (token, _) = generate_jwt(data.admin_user.id, data.admin_user.admin);
        let uri = format!("/api/modules/{}/assignments/bulk", data.module.id);
    
        let ids_to_update = vec![data.assignments[2].id];
        let new_due_date = DateTime::parse_from_rfc3339("2025-01-01T00:00:00Z").unwrap().with_timezone(&Utc);
        
        let req_body = json!({
            "assignment_ids": ids_to_update,
            "due_date": new_due_date
        });
        let req = Request::builder()
            .method("PUT")
            .uri(&uri)
            .header("Authorization", format!("Bearer {}", token))
            .header("Content-Type", "application/json")
            .body(Body::from(serde_json::to_string(&req_body).unwrap()))
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);

        let assignment = db::models::assignment::Entity::find_by_id(ids_to_update[0])
            .one(&db)
            .await
            .unwrap()
            .unwrap();
            
        assert_eq!(
            assignment.due_date,
            new_due_date
        );
    }

    /// Test Case: Partial Success with Some Failures
    #[tokio::test]
    async fn test_bulk_update_assignments_partial_success() {
        let db = setup_test_db().await;
        let data = setup_test_data(&db).await;

        let app = make_app(db.clone());
        let (token, _) = generate_jwt(data.lecturer_user.id, data.lecturer_user.admin);
        let uri = format!("/api/modules/{}/assignments/bulk", data.module.id);
        
        let ids_to_update = vec![data.assignments[0].id, 9999, data.assignments[2].id];
        let new_available_from = DateTime::parse_from_rfc3339("2025-01-01T00:00:00Z").unwrap().with_timezone(&Utc);
        
        let req_body = json!({
            "assignment_ids": ids_to_update,
            "available_from": new_available_from
        });
        let req = Request::builder()
            .method("PUT")
            .uri(&uri)
            .header("Authorization", format!("Bearer {}", token))
            .header("Content-Type", "application/json")
            .body(Body::from(serde_json::to_string(&req_body).unwrap()))
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let json: Value = serde_json::from_slice(&body).unwrap();
        
        assert_eq!(json["success"], true);
        assert_eq!(json["message"], "Updated 2/3 assignments");
        assert_eq!(json["data"]["updated"], 2);
        
        let failed = json["data"]["failed"].as_array().unwrap();
        assert_eq!(failed.len(), 1);
        assert_eq!(failed[0]["id"], 9999);
        assert_eq!(failed[0]["error"], "Assignment not found");

        for id in [data.assignments[0].id, data.assignments[2].id] {
            let assignment = db::models::assignment::Entity::find_by_id(id)
                .one(&db)
                .await
                .unwrap()
                .unwrap();
                
            assert_eq!(
                assignment.available_from,
                new_available_from
            );
        }
    }

    /// Test Case: No Assignment IDs Provided
    #[tokio::test]
    async fn test_bulk_update_assignments_no_ids() {
        let db = setup_test_db().await;
        let data = setup_test_data(&db).await;

        let app = make_app(db.clone());
        let (token, _) = generate_jwt(data.lecturer_user.id, data.lecturer_user.admin);
        let uri = format!("/api/modules/{}/assignments/bulk", data.module.id);
        let req_body = json!({
            "assignment_ids": [],
            "due_date": "2025-01-01T00:00:00Z"
        });
        let req = Request::builder()
            .method("PUT")
            .uri(&uri)
            .header("Authorization", format!("Bearer {}", token))
            .header("Content-Type", "application/json")
            .body(Body::from(serde_json::to_string(&req_body).unwrap()))
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let json: Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(json["success"], false);
        assert_eq!(json["message"], "No assignment IDs provided");
    }

    /// Test Case: Forbidden for Student
    #[tokio::test]
    async fn test_bulk_update_assignments_forbidden_student() {
        let db = setup_test_db().await;
        let data = setup_test_data(&db).await;

        let app = make_app(db.clone());
        let (token, _) = generate_jwt(data.student_user.id, data.student_user.admin);
        let uri = format!("/api/modules/{}/assignments/bulk", data.module.id);
        let req_body = json!({
            "assignment_ids": [data.assignments[0].id],
            "due_date": "2025-01-01T00:00:00Z"
        });
        let req = Request::builder()
            .method("PUT")
            .uri(&uri)
            .header("Authorization", format!("Bearer {}", token))
            .header("Content-Type", "application/json")
            .body(Body::from(serde_json::to_string(&req_body).unwrap()))
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::FORBIDDEN);
    }

    /// Test Case: Update Only Available From
    #[tokio::test]
    async fn test_bulk_update_only_available_from() {
        let db = setup_test_db().await;
        let data = setup_test_data(&db).await;

        let app = make_app(db.clone());
        let (token, _) = generate_jwt(data.lecturer_user.id, data.lecturer_user.admin);
        let uri = format!("/api/modules/{}/assignments/bulk", data.module.id);
        
        let ids_to_update = vec![data.assignments[0].id];
        let new_available_from = DateTime::parse_from_rfc3339("2025-01-01T00:00:00Z").unwrap().with_timezone(&Utc);
        let original_due_date = data.assignments[0].due_date;
        
        let req_body = json!({
            "assignment_ids": ids_to_update,
            "available_from": new_available_from
        });
        let req = Request::builder()
            .method("PUT")
            .uri(&uri)
            .header("Authorization", format!("Bearer {}", token))
            .header("Content-Type", "application/json")
            .body(Body::from(serde_json::to_string(&req_body).unwrap()))
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);

        let assignment = db::models::assignment::Entity::find_by_id(data.assignments[0].id)
            .one(&db)
            .await
            .unwrap()
            .unwrap();
            
        assert_eq!(
            assignment.available_from,
            new_available_from
        );
        assert_eq!(
            assignment.due_date,
            original_due_date
        );
    }

    /// Test Case: Update Only Due Date
    #[tokio::test]
    async fn test_bulk_update_only_due_date() {
        let db = setup_test_db().await;
        let data = setup_test_data(&db).await;

        let app = make_app(db.clone());
        let (token, _) = generate_jwt(data.lecturer_user.id, data.lecturer_user.admin);
        let uri = format!("/api/modules/{}/assignments/bulk", data.module.id);
        
        let ids_to_update = vec![data.assignments[1].id];
        let new_due_date = DateTime::parse_from_rfc3339("2025-01-01T00:00:00Z").unwrap().with_timezone(&Utc);
        let original_available_from = data.assignments[1].available_from;
        
        let req_body = json!({
            "assignment_ids": ids_to_update,
            "due_date": new_due_date
        });
        let req = Request::builder()
            .method("PUT")
            .uri(&uri)
            .header("Authorization", format!("Bearer {}", token))
            .header("Content-Type", "application/json")
            .body(Body::from(serde_json::to_string(&req_body).unwrap()))
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);

        let assignment = db::models::assignment::Entity::find_by_id(data.assignments[1].id)
            .one(&db)
            .await
            .unwrap()
            .unwrap();
            
        assert_eq!(
            assignment.available_from,
            original_available_from
        );
        assert_eq!(
            assignment.due_date,
            new_due_date
        );
    }

    /// Test Case: Assignment in Wrong Module
    #[tokio::test]
    async fn test_bulk_update_wrong_module() {
        let db = setup_test_db().await;
        let data = setup_test_data(&db).await;

        let other_assignment = AssignmentModel::create(
            &db,
            data.empty_module.id,
            "Other Module Assignment",
            Some("Should not be updated"),
            AssignmentType::Assignment,
            Utc::now(),
            Utc::now() + chrono::Duration::days(30),
        )
        .await
        .unwrap();

        let app = make_app(db.clone());
        let (token, _) = generate_jwt(data.lecturer_user.id, data.lecturer_user.admin);
        let uri = format!("/api/modules/{}/assignments/bulk", data.module.id);
        let ids_to_update = vec![data.assignments[0].id, other_assignment.id];
        let req_body = json!({
            "assignment_ids": ids_to_update,
            "due_date": "2025-01-01T00:00:00Z"
        });
        let req = Request::builder()
            .method("PUT")
            .uri(&uri)
            .header("Authorization", format!("Bearer {}", token))
            .header("Content-Type", "application/json")
            .body(Body::from(serde_json::to_string(&req_body).unwrap()))
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let json: Value = serde_json::from_slice(&body).unwrap();
        
        assert_eq!(json["success"], true);
        assert_eq!(json["message"], "Updated 1/2 assignments");
        assert_eq!(json["data"]["updated"], 1);
        
        let failed = json["data"]["failed"].as_array().unwrap();
        assert_eq!(failed.len(), 1);
        assert_eq!(failed[0]["id"], other_assignment.id);
        assert_eq!(failed[0]["error"], "Assignment not found");

        let updated_assignment = db::models::assignment::Entity::find_by_id(data.assignments[0].id)
            .one(&db)
            .await
            .unwrap()
            .unwrap();
        assert_eq!(
            updated_assignment.due_date.to_rfc3339(),
            "2025-01-01T00:00:00+00:00"
        );
        
        let not_updated = db::models::assignment::Entity::find_by_id(other_assignment.id)
            .one(&db)
            .await
            .unwrap()
            .unwrap();
        assert_ne!(
            not_updated.due_date.to_rfc3339(),
            "2025-01-01T00:00:00+00:00"
        );
    }

    /// Test Case: Unauthorized Access
    #[tokio::test]
    async fn test_bulk_update_assignments_unauthorized() {
        let db = setup_test_db().await;
        let data = setup_test_data(&db).await;

        let app = make_app(db.clone());
        let uri = format!("/api/modules/{}/assignments/bulk", data.module.id);
        let req_body = json!({
            "assignment_ids": [data.assignments[0].id],
            "due_date": "2025-01-01T00:00:00Z"
        });
        let req = Request::builder()
            .method("PUT")
            .uri(&uri)
            .header("Content-Type", "application/json")
            .body(Body::from(serde_json::to_string(&req_body).unwrap()))
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    /// Test Case: Forbidden for Unassigned User
    #[tokio::test]
    async fn test_bulk_update_assignments_forbidden_unassigned() {
        let db = setup_test_db().await;
        let data = setup_test_data(&db).await;

        let app = make_app(db.clone());
        let (token, _) = generate_jwt(data.forbidden_user.id, data.forbidden_user.admin);
        let uri = format!("/api/modules/{}/assignments/bulk", data.module.id);
        let req_body = json!({
            "assignment_ids": [data.assignments[0].id],
            "due_date": "2025-01-01T00:00:00Z"
        });
        let req = Request::builder()
            .method("PUT")
            .uri(&uri)
            .header("Authorization", format!("Bearer {}", token))
            .header("Content-Type", "application/json")
            .body(Body::from(serde_json::to_string(&req_body).unwrap()))
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::FORBIDDEN);
    }
}