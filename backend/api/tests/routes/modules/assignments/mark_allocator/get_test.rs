#[cfg(test)]
mod tests {
    use db::{
        models::{
            user::Model as UserModel,
            module::Model as ModuleModel,
            assignment::Model as AssignmentModel,
            user_module_role::{Model as UserModuleRoleModel,Role}
        },
        repositories::user_repository::UserRepository,
    };
    use axum::{
        body::Body,
        http::{Request, StatusCode}
    };
    use services::{
        service::Service,
        user::{CreateUser, UserService}
    };
    use tower::ServiceExt;
    use serde_json::Value;
    use api::auth::generate_jwt;
    use chrono::{Utc, TimeZone};
    use std::{fs, path::PathBuf};
    use serial_test::serial;
    use crate::helpers::app::make_test_app;

    struct TestData {
        lecturer_user: UserModel,
        student_user: UserModel,
        forbidden_user: UserModel,
        module: ModuleModel,
        assignment: AssignmentModel,
    }

    fn set_test_assignment_root() -> String {
        let tmp_dir = "./tmp".to_string();
        unsafe {
            std::env::set_var("ASSIGNMENT_STORAGE_ROOT", &tmp_dir);
        }

        tmp_dir
    }

    async fn setup_test_data(db: &sea_orm::DatabaseConnection) -> TestData {
        let module = ModuleModel::create(db, "COS101", 2024, Some("Test Module"), 16).await.unwrap();
        let service = UserService::new(UserRepository::new(db.clone()));
        let lecturer_user = service.create(CreateUser { username: "lecturer1".to_string(), email: "lecturer1@test.com".to_string(), password: "password1".to_string(), admin: false }).await.unwrap();
        let student_user = service.create(CreateUser { username: "student1".to_string(), email: "student1@test.com".to_string(), password: "password2".to_string(), admin: false }).await.unwrap();
        let forbidden_user = service.create(CreateUser { username: "forbidden".to_string(), email: "forbidden@test.com".to_string(), password: "password3".to_string(), admin: false }).await.unwrap();
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
        set_test_assignment_root();
        let app = make_test_app().await;
        let data = setup_test_data(db::get_connection().await).await;
        
        let allocator_path = PathBuf::from("./tmp")
            .join(format!("module_{}", data.module.id))
            .join(format!("assignment_{}", data.assignment.id))
            .join("mark_allocator")
            .join("allocator.json");
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
        
        let _ = fs::remove_dir_all("./tmp");
    }

    #[tokio::test]
    #[serial]
    async fn test_get_mark_allocator_not_found() {
        let app = make_test_app().await;
        let data = setup_test_data(db::get_connection().await).await;
        
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
        let app = make_test_app().await;
        let data = setup_test_data(db::get_connection().await).await;
        
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
        let app = make_test_app().await;
        let data = setup_test_data(db::get_connection().await).await;
        
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
        let app = make_test_app().await;
        let data = setup_test_data(db::get_connection().await).await;
        
        let uri = format!("/api/modules/{}/assignments/{}/mark_allocator", data.module.id, data.assignment.id);
        let req = Request::builder()
            .uri(&uri)
            .body(Body::empty())
            .unwrap();
        
        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }
}