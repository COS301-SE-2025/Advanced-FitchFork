#[cfg(test)]
mod tests {
    use db::{test_utils::setup_test_db, models::{user::Model as UserModel, module::Model as ModuleModel, assignment::Model as AssignmentModel, user_module_role::{Model as UserModuleRoleModel, Role}}};
    use axum::{body::Body, http::{Request, StatusCode}};
    use tower::ServiceExt;
    use serde_json::json;
    use api::auth::generate_jwt;
    use dotenvy;
    use chrono::{Utc, TimeZone};
    use std::fs;
    use serial_test::serial;
    use crate::test_helpers::make_app;

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
    async fn test_put_mark_allocator_success_as_lecturer() {
        dotenvy::dotenv().expect("Failed to load .env");
        unsafe { std::env::set_var("ASSIGNMENT_STORAGE_ROOT", "./tmp"); }
        let db = setup_test_db().await;
        let data = setup_test_data(&db).await;
        
        let app = make_app(db.clone());
        let (token, _) = generate_jwt(data.lecturer_user.id, data.lecturer_user.admin);
        let uri = format!("/api/modules/{}/assignments/{}/mark_allocator", data.module.id, data.assignment.id);
        let req = Request::builder()
            .uri(&uri)
            .method("PUT")
            .header("Authorization", format!("Bearer {}", token))
            .header("Content-Type", "application/json")
            .body(Body::from(r#"{"tasks":[{"task_number":1,"weight":1.0,"criteria":[{"name":"Correctness","weight":1.0}]}],"total_weight":1.0}"#))
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        
        let _ = fs::remove_dir_all("./tmp");
    }

    #[tokio::test]
    #[serial]
    async fn test_put_mark_allocator_validation_error_weights() {
        dotenvy::dotenv().expect("Failed to load .env");
        unsafe { std::env::set_var("ASSIGNMENT_STORAGE_ROOT", "./tmp"); }
        let db = setup_test_db().await;
        let data = setup_test_data(&db).await;
        
        let app = make_app(db.clone());
        let (token, _) = generate_jwt(data.lecturer_user.id, data.lecturer_user.admin);
        let uri = format!("/api/modules/{}/assignments/{}/mark_allocator", data.module.id, data.assignment.id);

        let body = json!({
            "tasks": [
                {
                    "task_number": 1,
                    "weight": 0.5,
                    "criteria": [
                        { "name": "Correctness", "weight": 0.5 }
                    ]
                }
            ],
            "total_weight": 1.0
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
    }

    #[tokio::test]
    #[serial]
    async fn test_put_mark_allocator_not_found_on_nonexistent() {
        dotenvy::dotenv().expect("Failed to load .env");
        unsafe { std::env::set_var("ASSIGNMENT_STORAGE_ROOT", "./tmp"); }
        let db = setup_test_db().await;
        let data = setup_test_data(&db).await;
        
        let app = make_app(db.clone());
        let (token, _) = generate_jwt(data.lecturer_user.id, data.lecturer_user.admin);
        let uri = format!("/api/modules/{}/assignments/{}/mark_allocator", data.module.id, 9999);
        let body = json!({
            "tasks": [{"task_number":1,"weight":1.0,"criteria":[{"name":"Correctness","weight":1.0}]}],"total_weight":1.0
        });
        let req = Request::builder()
            .uri(&uri)
            .method("PUT")
            .header("Authorization", format!("Bearer {}", token))
            .header("Content-Type", "application/json")
            .body(Body::from(body.to_string()))
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    #[serial]
    async fn test_put_mark_allocator_forbidden_for_student() {
        dotenvy::dotenv().expect("Failed to load .env");
        unsafe { std::env::set_var("ASSIGNMENT_STORAGE_ROOT", "./tmp"); }
        let db = setup_test_db().await;
        let data = setup_test_data(&db).await;
        
        let app = make_app(db.clone());
        let (token, _) = generate_jwt(data.student_user.id, data.student_user.admin);
        let uri = format!("/api/modules/{}/assignments/{}/mark_allocator", data.module.id, data.assignment.id);
        let body = json!({
            "tasks": [{"task_number":1,"weight":1.0,"criteria":[{"name":"Correctness","weight":1.0}]}],"total_weight":1.0
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
    #[serial]
    async fn test_put_mark_allocator_forbidden_for_unassigned_user() {
        dotenvy::dotenv().expect("Failed to load .env");
        unsafe { std::env::set_var("ASSIGNMENT_STORAGE_ROOT", "./tmp"); }
        let db = setup_test_db().await;
        let data = setup_test_data(&db).await;
        
        let app = make_app(db.clone());
        let (token, _) = generate_jwt(data.forbidden_user.id, data.forbidden_user.admin);
        let uri = format!("/api/modules/{}/assignments/{}/mark_allocator", data.module.id, data.assignment.id);
        let body = json!({
            "tasks": [{"task_number":1,"weight":1.0,"criteria":[{"name":"Correctness","weight":1.0}]}],"total_weight":1.0
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
    #[serial]
    async fn test_put_mark_allocator_unauthorized() {
        dotenvy::dotenv().expect("Failed to load .env");
        unsafe { std::env::set_var("ASSIGNMENT_STORAGE_ROOT", "./tmp"); }
        let db = setup_test_db().await;
        let data = setup_test_data(&db).await;
        
        let app = make_app(db.clone());
        let uri = format!("/api/modules/{}/assignments/{}/mark_allocator", data.module.id, data.assignment.id);
        let body = json!({
            "tasks": [{"task_number":1,"weight":1.0,"criteria":[{"name":"Correctness","weight":1.0}]}],"total_weight":1.0
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

    #[tokio::test]
    #[serial]
    async fn test_put_mark_allocator_invalid_json() {
        dotenvy::dotenv().expect("Failed to load .env");
        unsafe { std::env::set_var("ASSIGNMENT_STORAGE_ROOT", "./tmp"); }
        let db = setup_test_db().await;
        let data = setup_test_data(&db).await;
        
        let app = make_app(db.clone());
        let (token, _) = generate_jwt(data.lecturer_user.id, data.lecturer_user.admin);
        let uri = format!("/api/modules/{}/assignments/{}/mark_allocator", data.module.id, data.assignment.id);
        let req = Request::builder()
            .uri(&uri)
            .method("PUT")
            .header("Authorization", format!("Bearer {}", token))
            .header("Content-Type", "application/json")
            .body(Body::from(r#"{"invalid": "json"}"#))
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }
}