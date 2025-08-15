#[cfg(test)]
mod tests {
    use crate::helpers::app::make_test_app;
    use api::auth::generate_jwt;
    use axum::{
        body::{Body, to_bytes},
        http::{Request, StatusCode},
    };
    use chrono::{TimeZone, Utc};
    use db::{
        models::{
            assignment::Model as AssignmentModel,
            module::Model as ModuleModel,
            user::Model as UserModel,
            user_module_role::{Model as UserModuleRoleModel, Role},
        },
        repositories::user_repository::UserRepository,
    };
    use services::{
        service::Service,
        user_service::{CreateUser, UserService}
    };
    use serde_json::Value;
    use serial_test::serial;
    use std::{fs, path::PathBuf};
    use tower::ServiceExt;

    struct TestData {
        admin_user: UserModel,
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
        let admin_user = service.create(CreateUser { username: "admin1".to_string(), email: "admin1@test.com".to_string(), password: "password".to_string(), admin: true }).await.unwrap();
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
            admin_user,
            lecturer_user,
            student_user,
            forbidden_user,
            module,
            assignment,
        }
    }

    fn setup_memo_output_file(module_id: i64, assignment_id: i64, task_number: i32) {
        let memo_output_path = PathBuf::from("./tmp")
            .join(format!("module_{}", module_id))
            .join(format!("assignment_{}", assignment_id))
            .join("memo_output")
            .join(format!("{}.txt", task_number));
        fs::create_dir_all(memo_output_path.parent().unwrap()).unwrap();
        fs::write(&memo_output_path, "This is a test memo output.").unwrap();
    }

    fn cleanup_tmp() {
        let _ = fs::remove_dir_all("./tmp");
    }

    #[tokio::test]
    #[serial]
    async fn test_get_memo_output_success_as_lecturer() {
        set_test_assignment_root();
        let app = make_test_app().await;
        let data = setup_test_data(db::get_connection().await).await;
        setup_memo_output_file(data.module.id, data.assignment.id, 1);

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

        cleanup_tmp();
    }

    #[tokio::test]
    #[serial]
    async fn test_get_memo_output_success_as_admin() {
        set_test_assignment_root();
        let app = make_test_app().await;
        let data = setup_test_data(db::get_connection().await).await;
        setup_memo_output_file(data.module.id, data.assignment.id, 1);

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

        cleanup_tmp();
    }

    #[tokio::test]
    #[serial]
    async fn test_get_memo_output_forbidden_for_student() {
        let app = make_test_app().await;
        let data = setup_test_data(db::get_connection().await).await;
        setup_memo_output_file(data.module.id, data.assignment.id, 1);

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

        cleanup_tmp();
    }

    #[tokio::test]
    #[serial]
    async fn test_get_memo_output_forbidden_for_unassigned_user() {
        let app = make_test_app().await;
        let data = setup_test_data(db::get_connection().await).await;
        setup_memo_output_file(data.module.id, data.assignment.id, 1);

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
        assert_eq!(response.status(), StatusCode::FORBIDDEN);
        cleanup_tmp();
    }

    #[tokio::test]
    #[serial]
    async fn test_get_memo_output_not_found_if_file_doesnt_exist() {
        let app = make_test_app().await;
        let data = setup_test_data(db::get_connection().await).await;

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

        cleanup_tmp();
    }

    #[tokio::test]
    #[serial]
    async fn test_get_memo_output_assignment_not_found() {
        let app = make_test_app().await;
        let data = setup_test_data(db::get_connection().await).await;

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

        cleanup_tmp();
    }

    #[tokio::test]
    #[serial]
    async fn test_get_memo_output_unauthorized() {
        let app = make_test_app().await;
        let data = setup_test_data(db::get_connection().await).await;

        let uri = format!(
            "/api/modules/{}/assignments/{}/memo_output",
            data.module.id, data.assignment.id
        );
        let req = Request::builder().uri(&uri).body(Body::empty()).unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);

        cleanup_tmp();
    }
}
