#[cfg(test)]
mod tests {
    use axum::{body::{Body, to_bytes}, http::{Request, StatusCode}};
    use db::{
        test_utils::setup_test_db,
        models::{
            user::Model as UserModel,
            module::Model as ModuleModel,
            assignment::Model as AssignmentModel,
            user_module_role::{Model as UserModuleRoleModel, Role},
        },
    };
    use dotenvy;
    use tower::ServiceExt;
    use api::auth::generate_jwt;
    use chrono::{Utc, TimeZone};
    use serde_json::Value;
    use serial_test::serial;
    use std::{fs, path::PathBuf};
    use crate::test_helpers::make_app;

    struct TestData {
        admin_user: UserModel,
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
        let admin_user = UserModel::create(db, "admin1", "admin1@test.com", "password", true)
            .await
            .unwrap();
        let lecturer_user = UserModel::create(db, "lecturer1", "lecturer1@test.com", "password1", false)
            .await
            .unwrap();
        let student_user = UserModel::create(db, "student1", "student1@test.com", "password2", false)
            .await
            .unwrap();
        let forbidden_user = UserModel::create(db, "forbidden", "forbidden@test.com", "password3", false)
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
            Some(db::models::assignment::Status::Setup),
        )
        .await
        .unwrap();

        TestData {
            admin_user,
            lecturer_user,
            student_user,
            forbidden_user,
            module,
            assignment,
        }
    }

    /// Seeds a single memo output file under ./tmp
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
        dotenvy::dotenv().ok();
        unsafe { std::env::set_var("ASSIGNMENT_STORAGE_ROOT", "./tmp"); }
        let db = setup_test_db().await;
        let data = setup_test_data(&db).await;
        setup_memo_output_file(data.module.id, data.assignment.id, 1);

        let app = make_app(db.clone());
        let (token, _) = generate_jwt(data.lecturer_user.id, data.lecturer_user.admin);
        let uri = format!("/api/modules/{}/assignments/{}/memo_output", data.module.id, data.assignment.id);
        let req = Request::builder()
            .uri(&uri)
            .header("Authorization", format!("Bearer {}", token))
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);

        let body_bytes = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let v: Value = serde_json::from_slice(&body_bytes).unwrap();
        let data_arr = v.get("data").and_then(|d| d.as_array()).expect("data not array");
        assert_eq!(data_arr.len(), 1);
        assert_eq!(data_arr[0].get("raw").and_then(|r| r.as_str()), Some("This is a test memo output."));

        cleanup_tmp();
    }

    #[tokio::test]
    #[serial]
    async fn test_get_memo_output_success_as_admin() {
        dotenvy::dotenv().ok();
        unsafe { std::env::set_var("ASSIGNMENT_STORAGE_ROOT", "./tmp"); }
        let db = setup_test_db().await;
        let data = setup_test_data(&db).await;
        setup_memo_output_file(data.module.id, data.assignment.id, 1);

        let app = make_app(db.clone());
        let (token, _) = generate_jwt(data.admin_user.id, data.admin_user.admin);
        let uri = format!("/api/modules/{}/assignments/{}/memo_output", data.module.id, data.assignment.id);
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
        dotenvy::dotenv().ok();
        unsafe { std::env::set_var("ASSIGNMENT_STORAGE_ROOT", "./tmp"); }
        let db = setup_test_db().await;
        let data = setup_test_data(&db).await;
        setup_memo_output_file(data.module.id, data.assignment.id, 1);

        let app = make_app(db.clone());
        let (token, _) = generate_jwt(data.student_user.id, data.student_user.admin);
        let uri = format!("/api/modules/{}/assignments/{}/memo_output", data.module.id, data.assignment.id);
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
        dotenvy::dotenv().ok();
        unsafe { std::env::set_var("ASSIGNMENT_STORAGE_ROOT", "./tmp"); }
        let db = setup_test_db().await;
        let data = setup_test_data(&db).await;
        setup_memo_output_file(data.module.id, data.assignment.id, 1);

        let app = make_app(db.clone());
        let (token, _) = generate_jwt(data.forbidden_user.id, data.forbidden_user.admin);
        let uri = format!("/api/modules/{}/assignments/{}/memo_output", data.module.id, data.assignment.id);
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
        dotenvy::dotenv().ok();
        unsafe { std::env::set_var("ASSIGNMENT_STORAGE_ROOT", "./tmp"); }
        let db = setup_test_db().await;
        let data = setup_test_data(&db).await;

        let app = make_app(db.clone());
        let (token, _) = generate_jwt(data.lecturer_user.id, data.lecturer_user.admin);
        let uri = format!("/api/modules/{}/assignments/{}/memo_output", data.module.id, data.assignment.id);
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
        dotenvy::dotenv().ok();
        unsafe { std::env::set_var("ASSIGNMENT_STORAGE_ROOT", "./tmp"); }
        let db = setup_test_db().await;
        let data = setup_test_data(&db).await;

        let app = make_app(db.clone());
        let (token, _) = generate_jwt(data.lecturer_user.id, data.lecturer_user.admin);
        // Use correct header name here
        let uri = format!("/api/modules/{}/assignments/{}/memo_output", data.module.id, 9999);
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
        dotenvy::dotenv().ok();
        unsafe { std::env::set_var("ASSIGNMENT_STORAGE_ROOT", "./tmp"); }
        let db = setup_test_db().await;
        let data = setup_test_data(&db).await;

        let app = make_app(db.clone());
        let uri = format!("/api/modules/{}/assignments/{}/memo_output", data.module.id, data.assignment.id);
        let req = Request::builder()
            .uri(&uri)
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
        cleanup_tmp();
    }
}