#[cfg(test)]
mod tests {
    use db::{
        models::{user::Model as UserModel,
            module::Model as ModuleModel,
            assignment::Model as AssignmentModel,
            user_module_role::{Model as UserModuleRoleModel, Role}
        },
        repositories::user_repository::UserRepository,
    };
    use axum::{
        body::Body,
        http::{Request, StatusCode}
    };
    use services::{
        service::Service,
        user_service::{UserService, CreateUser},
    };
    use tower::ServiceExt;
    use serde_json::Value;
    use api::auth::generate_jwt;
    use chrono::{Utc, TimeZone};
    use db::models::assignment::AssignmentType;
    use crate::helpers::app::make_test_app;

    struct TestData {
        admin_user: UserModel,
        lecturer_user: UserModel,
        student_user: UserModel,
        forbidden_user: UserModel,
        module: ModuleModel,
        empty_module: ModuleModel,
        assignments: Vec<AssignmentModel>,
    }

    async fn setup_test_data(db: &sea_orm::DatabaseConnection) -> TestData {
        let module = ModuleModel::create(db, "COS101", 2024, Some("Test Module"), 16).await.unwrap();
        let empty_module = ModuleModel::create(db, "EMPTY101", 2024, Some("Empty Module"), 16).await.unwrap();
        let service = UserService::new(UserRepository::new(db.clone()));
        let admin_user = service.create(CreateUser { username: "admin1".to_string(), email: "admin1@test.com".to_string(), password: "password".to_string(), admin: true }).await.unwrap();
        let lecturer_user = service.create(CreateUser { username: "lecturer1".to_string(), email: "lecturer1@test.com".to_string(), password: "password1".to_string(), admin: false }).await.unwrap();
        let student_user = service.create(CreateUser { username: "student1".to_string(), email: "student1@test.com".to_string(), password: "password2".to_string(), admin: false }).await.unwrap();
        let forbidden_user = service.create(CreateUser { username: "forbidden".to_string(), email: "forbidden@test.com".to_string(), password: "password3".to_string(), admin: false }).await.unwrap();
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
        }
    }

    // --- GET /api/modules/{module_id}/assignments (List) ---

    #[tokio::test]
    async fn test_get_assignments_success_as_admin() {
        let (app, app_state) = make_test_app().await;
        let data = setup_test_data(app_state.db()).await;

        let (token, _) = generate_jwt(data.admin_user.id, data.admin_user.admin);
        let uri = format!("/api/modules/{}/assignments", data.module.id);
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
        assert_eq!(json["data"]["total"], 3);
        let names: Vec<_> = json["data"]["assignments"].as_array().unwrap().iter().map(|a| a["name"].as_str().unwrap()).collect();
        assert!(names.contains(&"Assignment 1"));
        assert!(names.contains(&"Assignment 2"));
        assert!(names.contains(&"Assignment 3"));
    }

    #[tokio::test]
    async fn test_get_assignments_success_as_lecturer() {
        let (app, app_state) = make_test_app().await;
        let data = setup_test_data(app_state.db()).await;

        let (token, _) = generate_jwt(data.lecturer_user.id, data.lecturer_user.admin);
        let uri = format!("/api/modules/{}/assignments", data.module.id);
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
        assert_eq!(json["data"]["total"], 3);
    }

    #[tokio::test]
    async fn test_get_assignments_forbidden_for_unassigned_user() {
        let (app, app_state) = make_test_app().await;
        let data = setup_test_data(app_state.db()).await;

        let (token, _) = generate_jwt(data.forbidden_user.id, data.forbidden_user.admin);
        let uri = format!("/api/modules/{}/assignments", data.module.id);
        let req = Request::builder()
            .uri(&uri)
            .header("Authorization", format!("Bearer {}", token))
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::FORBIDDEN);
    }

    #[tokio::test]
    async fn test_get_assignments_module_not_found() {
        let (app, app_state) = make_test_app().await;
        let data = setup_test_data(app_state.db()).await;

        let (token, _) = generate_jwt(data.admin_user.id, data.admin_user.admin);
        let uri = format!("/api/modules/{}/assignments", 9999);
        let req = Request::builder()
            .uri(&uri)
            .header("Authorization", format!("Bearer {}", token))
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn test_get_assignments_filtering_and_sorting() {
        let (app, app_state) = make_test_app().await;
        let data = setup_test_data(app_state.db()).await;

        let (token, _) = generate_jwt(data.admin_user.id, data.admin_user.admin);
        let uri = format!("/api/modules/{}/assignments?query=Assignment&sort=-name", data.module.id);
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
        let names: Vec<_> = json["data"]["assignments"].as_array().unwrap().iter().map(|a| a["name"].as_str().unwrap()).collect();
        assert_eq!(names, vec!["Assignment 3", "Assignment 2", "Assignment 1"]);
    }

    #[tokio::test]
    async fn test_get_assignments_pagination() {
        let (app, app_state) = make_test_app().await;
        let data = setup_test_data(app_state.db()).await;

        let (token, _) = generate_jwt(data.admin_user.id, data.admin_user.admin);
        let uri = format!("/api/modules/{}/assignments?page=2&per_page=2", data.module.id);
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
        assert_eq!(json["data"]["page"], 2);
        assert_eq!(json["data"]["per_page"], 2);
        let assignments = json["data"]["assignments"].as_array().unwrap();
        assert_eq!(assignments.len(), 1);
    }

    #[tokio::test]
    async fn test_get_assignments_invalid_sort_field() {
        let (app, app_state) = make_test_app().await;
        let data = setup_test_data(app_state.db()).await;

        let (token, _) = generate_jwt(data.admin_user.id, data.admin_user.admin);
        let uri = format!("/api/modules/{}/assignments?sort=invalid_field", data.module.id);
        let req = Request::builder()
            .uri(&uri)
            .header("Authorization", format!("Bearer {}", token))
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn test_get_assignments_invalid_assignment_type() {
        let (app, app_state) = make_test_app().await;
        let data = setup_test_data(app_state.db()).await;

        let (token, _) = generate_jwt(data.admin_user.id, data.admin_user.admin);
        let uri = format!("/api/modules/{}/assignments?assignment_type=invalid", data.module.id);
        let req = Request::builder()
            .uri(&uri)
            .header("Authorization", format!("Bearer {}", token))
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn test_get_assignments_no_assignments_in_module() {
        let (app, app_state) = make_test_app().await;
        let data = setup_test_data(app_state.db()).await;

        let (token, _) = generate_jwt(data.admin_user.id, data.admin_user.admin);
        let uri = format!("/api/modules/{}/assignments", data.empty_module.id);
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
        assert_eq!(json["data"]["total"], 0);
    }

    #[tokio::test]
    async fn test_get_assignments_unauthorized() {
        let (app, app_state) = make_test_app().await;
        let data = setup_test_data(app_state.db()).await;

        let uri = format!("/api/modules/{}/assignments", data.module.id);
        let req = Request::builder()
            .uri(&uri)
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    // --- GET /api/modules/{module_id}/assignments/{assignment_id} (Detail) ---

    #[tokio::test]
    async fn test_get_assignment_detail_success_as_admin() {
        let (app, app_state) = make_test_app().await;
        let data = setup_test_data(app_state.db()).await;

        let (token, _) = generate_jwt(data.admin_user.id, data.admin_user.admin);
        let uri = format!("/api/modules/{}/assignments/{}", data.module.id, data.assignments[0].id);
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
        assert_eq!(json["data"]["assignment"]["id"], data.assignments[0].id);
    }

    #[tokio::test]
    async fn test_get_assignment_detail_success_as_lecturer() {
        let (app, app_state) = make_test_app().await;
        let data = setup_test_data(app_state.db()).await;

        let (token, _) = generate_jwt(data.lecturer_user.id, data.lecturer_user.admin);
        let uri = format!("/api/modules/{}/assignments/{}", data.module.id, data.assignments[0].id);
        let req = Request::builder()
            .uri(&uri)
            .header("Authorization", format!("Bearer {}", token))
            .body(Body::empty())
            .unwrap();

        let response: axum::http::Response<Body> = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_get_assignment_detail_not_found() {
        let (app, app_state) = make_test_app().await;
        let data = setup_test_data(app_state.db()).await;

        let (token, _) = generate_jwt(data.admin_user.id, data.admin_user.admin);
        let uri = format!("/api/modules/{}/assignments/9999", data.module.id);
        let req = Request::builder()
            .uri(&uri)
            .header("Authorization", format!("Bearer {}", token))
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn test_get_assignment_detail_forbidden_for_unassigned_user() {
        let (app, app_state) = make_test_app().await;
        let data = setup_test_data(app_state.db()).await;

        let (token, _) = generate_jwt(data.forbidden_user.id, data.forbidden_user.admin);
        let uri = format!("/api/modules/{}/assignments/{}", data.module.id, data.assignments[0].id);
        let req = Request::builder()
            .uri(&uri)
            .header("Authorization", format!("Bearer {}", token))
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::FORBIDDEN);
    }

    #[tokio::test]
    async fn test_get_assignment_detail_wrong_module() {
        let (app, app_state) = make_test_app().await;
        let data = setup_test_data(app_state.db()).await;

        let (token, _) = generate_jwt(data.admin_user.id, data.admin_user.admin);
        let uri = format!("/api/modules/{}/assignments/{}", data.empty_module.id, data.assignments[0].id);
        let req = Request::builder()
            .uri(&uri)
            .header("Authorization", format!("Bearer {}", token))
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn test_get_assignment_detail_unauthorized() {
        let (app, app_state) = make_test_app().await;
        let data = setup_test_data(app_state.db()).await;

        let uri = format!("/api/modules/{}/assignments/{}", data.module.id, data.assignments[0].id);
        let req = Request::builder()
            .uri(&uri)
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn test_get_assignment_detail_module_not_found() {
        let (app, app_state) = make_test_app().await;
        let data = setup_test_data(app_state.db()).await;

        let (token, _) = generate_jwt(data.admin_user.id, data.admin_user.admin);
        let uri = format!("/api/modules/{}/assignments/{}", 9999, data.assignments[0].id);
        let req = Request::builder()
            .uri(&uri)
            .header("Authorization", format!("Bearer {}", token))
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    // --- GET /api/modules/{module_id}/assignments/{assignment_id}/readiness ---

    #[tokio::test]
    async fn test_get_assignment_readiness_success_as_admin() {
        let (app, app_state) = make_test_app().await;
        let data = setup_test_data(app_state.db()).await;

        let (token, _) = generate_jwt(data.admin_user.id, data.admin_user.admin);
        let uri = format!("/api/modules/{}/assignments/{}/readiness", data.module.id, data.assignments[0].id);
        let req = Request::builder()
            .uri(&uri)
            .header("Authorization", format!("Bearer {}", token))
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert!(response.status() == StatusCode::OK);
    }

    #[tokio::test]
    async fn test_get_assignment_readiness_success_as_lecturer() {
        let (app, app_state) = make_test_app().await;
        let data = setup_test_data(app_state.db()).await;

        let (token, _) = generate_jwt(data.lecturer_user.id, data.lecturer_user.admin);
        let uri = format!("/api/modules/{}/assignments/{}/readiness", data.module.id, data.assignments[0].id);
        let req = Request::builder()
            .uri(&uri)
            .header("Authorization", format!("Bearer {}", token))
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert!(response.status() == StatusCode::OK);
    }

    #[tokio::test]
    async fn test_get_assignment_readiness_forbidden_for_student() {
        let (app, app_state) = make_test_app().await;
        let data = setup_test_data(app_state.db()).await;

        let (token, _) = generate_jwt(data.student_user.id, data.student_user.admin);
        let uri = format!("/api/modules/{}/assignments/{}/readiness", data.module.id, data.assignments[0].id);
        let req = Request::builder()
            .uri(&uri)
            .header("Authorization", format!("Bearer {}", token))
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::FORBIDDEN);
    }

    #[tokio::test]
    async fn test_get_assignment_readiness_not_found() {
        let (app, app_state) = make_test_app().await;
        let data = setup_test_data(app_state.db()).await;

        let (token, _) = generate_jwt(data.admin_user.id, data.admin_user.admin);
        let uri = format!("/api/modules/{}/assignments/9999/readiness", data.module.id);
        let req = Request::builder()
            .uri(&uri)
            .header("Authorization", format!("Bearer {}", token))
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert!(response.status() == StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn test_get_assignment_readiness_module_not_found() {
        let (app, app_state) = make_test_app().await;
        let data = setup_test_data(app_state.db()).await;

        let (token, _) = generate_jwt(data.admin_user.id, data.admin_user.admin);
        let uri = format!("/api/modules/{}/assignments/{}/readiness", 9999, data.assignments[0].id);
        let req = Request::builder()
            .uri(&uri)
            .header("Authorization", format!("Bearer {}", token))
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    // --- GET /api/modules/{module_id}/assignments/{assignment_id}/stats ---

    #[tokio::test]
    async fn test_get_assignment_stats_success_as_lecturer() {
        let (app, app_state) = make_test_app().await;
        let data = setup_test_data(app_state.db()).await;

        let (token, _) = generate_jwt(data.lecturer_user.id, data.lecturer_user.admin);
        let uri = format!("/api/modules/{}/assignments/{}/stats", data.module.id, data.assignments[0].id);
        let req = Request::builder()
            .uri(&uri)
            .header("Authorization", format!("Bearer {}", token))
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert!(response.status() == StatusCode::OK);
    }

    #[tokio::test]
    async fn test_get_assignment_stats_success_as_admin() {
        let (app, app_state) = make_test_app().await;
        let data = setup_test_data(app_state.db()).await;

        let (token, _) = generate_jwt(data.admin_user.id, data.admin_user.admin);
        let uri = format!("/api/modules/{}/assignments/{}/stats", data.module.id, data.assignments[0].id);
        let req = Request::builder()
            .uri(&uri)
            .header("Authorization", format!("Bearer {}", token))
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_get_assignment_stats_forbidden_for_student() {
        let (app, app_state) = make_test_app().await;
        let data = setup_test_data(app_state.db()).await;

        let (token, _) = generate_jwt(data.student_user.id, data.student_user.admin);
        let uri = format!("/api/modules/{}/assignments/{}/stats", data.module.id, data.assignments[0].id);
        let req = Request::builder()
            .uri(&uri)
            .header("Authorization", format!("Bearer {}", token))
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::FORBIDDEN);
    }

    #[tokio::test]
    async fn test_get_assignment_stats_not_found() {
        let (app, app_state) = make_test_app().await;
        let data = setup_test_data(app_state.db()).await;

        let (token, _) = generate_jwt(data.lecturer_user.id, data.lecturer_user.admin);
        let uri = format!("/api/modules/{}/assignments/9999/stats", data.module.id);
        let req = Request::builder()
            .uri(&uri)
            .header("Authorization", format!("Bearer {}", token))
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert!(response.status() == StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn test_get_assignment_stats_module_not_found() {
        let (app, app_state) = make_test_app().await;
        let data = setup_test_data(app_state.db()).await;

        let (token, _) = generate_jwt(data.admin_user.id, data.admin_user.admin);
        let uri = format!("/api/modules/{}/assignments/{}/stats", 9999, data.assignments[0].id);
        let req = Request::builder()
            .uri(&uri)
            .header("Authorization", format!("Bearer {}", token))
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }
}