#[cfg(test)]
mod tests {
    use db::{models::{user::Model as UserModel, module::Model as ModuleModel, assignment::Model as AssignmentModel, user_module_role::{Model as UserModuleRoleModel, Role}}};
    use axum::{body::Body, http::{Request, StatusCode}};
    use tower::ServiceExt;
    use serde_json::Value;
    use api::auth::generate_jwt;
    use chrono::{Utc, TimeZone};
    use util::paths::mark_allocator_path;
    use std::fs;
    use serial_test::serial;
    use crate::helpers::app::make_test_app_with_storage;
        
    struct TestData {
        lecturer_user: UserModel,
        student_user: UserModel,
        forbidden_user: UserModel,
        module: ModuleModel,
        assignment: AssignmentModel,
    }

    async fn setup_test_data(db: &sea_orm::DatabaseConnection) -> TestData {
        let module = ModuleModel::create(db, "COS101", 2024, Some("Test Module"), 16).await.unwrap();
        let lecturer_user = UserModel::create(db, "lecturer1", "lecturer1@test.com", "password1", false).await.unwrap();
        let student_user = UserModel::create(db, "student1", "student1@test.com", "password2", false).await.unwrap();
        let forbidden_user = UserModel::create(db, "forbidden", "forbidden@test.com", "password3", false).await.unwrap();
        UserModuleRoleModel::assign_user_to_module(db, lecturer_user.id, module.id, Role::Lecturer).await.unwrap();
        UserModuleRoleModel::assign_user_to_module(db, student_user.id, module.id, Role::Student).await.unwrap();
        let assignment = AssignmentModel::create(
            db,
            module.id,
            "Assignment 1",
            Some("Desc 1"),
            db::models::assignment::AssignmentType::Assignment,
            Utc.with_ymd_and_hms(2024, 1, 1, 0, 0, 0).unwrap(),
            Utc.with_ymd_and_hms(2024, 1, 31, 23, 59, 59).unwrap(),
        ).await.unwrap();
        
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
    async fn test_get_mark_allocator_success_as_lecturer() {
        let (app, app_state, _tmp) = make_test_app_with_storage().await;
        let data = setup_test_data(app_state.db()).await;
        
        let allocator_path = mark_allocator_path(data.module.id, data.assignment.id);
        fs::create_dir_all(allocator_path.parent().unwrap()).unwrap();
        fs::write(&allocator_path, r#"{"tasks":[{"task_number":1,"weight":1.0,"criteria":[{"name":"Correctness","weight":1.0}]}],"total_weight":1.0}"#).unwrap();

        let (token, _) = generate_jwt(data.lecturer_user.id, data.lecturer_user.admin);
        let uri = format!("/api/modules/{}/assignments/{}/mark_allocator", data.module.id, data.assignment.id);
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
        assert_eq!(json["data"]["total_weight"], 1.0);
    }

    #[tokio::test]
    #[serial]
    async fn test_get_mark_allocator_not_found() {
        let (app, app_state, _tmp) = make_test_app_with_storage().await;
        let data = setup_test_data(app_state.db()).await;
        
        let (token, _) = generate_jwt(data.lecturer_user.id, data.lecturer_user.admin);
        let uri = format!("/api/modules/{}/assignments/{}/mark_allocator", data.module.id, data.assignment.id);
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
    async fn test_get_mark_allocator_forbidden_for_student() {
        let (app, app_state, _tmp) = make_test_app_with_storage().await;
        let data = setup_test_data(app_state.db()).await;
        
        let (token, _) = generate_jwt(data.student_user.id, data.student_user.admin);
        let uri = format!("/api/modules/{}/assignments/{}/mark_allocator", data.module.id, data.assignment.id);
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
    async fn test_get_mark_allocator_forbidden_for_unassigned_user() {
        let (app, app_state, _tmp) = make_test_app_with_storage().await;
        let data = setup_test_data(app_state.db()).await;
        
        let (token, _) = generate_jwt(data.forbidden_user.id, data.forbidden_user.admin);
        let uri = format!("/api/modules/{}/assignments/{}/mark_allocator", data.module.id, data.assignment.id);
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
    async fn test_get_mark_allocator_unauthorized() {
        let (app, app_state, _tmp) = make_test_app_with_storage().await;
        let data = setup_test_data(app_state.db()).await;
        
        let uri = format!("/api/modules/{}/assignments/{}/mark_allocator", data.module.id, data.assignment.id);
        let req = Request::builder()
            .uri(&uri)
            .body(Body::empty())
            .unwrap();
        
        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }
}