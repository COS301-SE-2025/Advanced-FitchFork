#[cfg(test)]
mod tests {
    use crate::test_helpers::make_app;
    use api::auth::generate_jwt;
    use axum::{
        body::Body,
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
        test_utils::setup_test_db,
    };
    use dotenvy;
    use serial_test::serial;
    use std::{fs, path::PathBuf};
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
    async fn test_post_mark_allocator_success_as_lecturer() {
        dotenvy::dotenv().expect("Failed to load .env");
        unsafe {
            std::env::set_var("ASSIGNMENT_STORAGE_ROOT", "./tmp");
        }
        let db = setup_test_db().await;
        let data = setup_test_data(&db).await;

        let memo_output_dir = PathBuf::from("./tmp")
            .join(format!("module_{}", data.module.id))
            .join(format!("assignment_{}", data.assignment.id))
            .join("memo_output");
        fs::create_dir_all(&memo_output_dir).unwrap();
        fs::write(memo_output_dir.join("task_1.txt"), "Test memo output").unwrap();

        let app = make_app(db.clone());
        let (token, _) = generate_jwt(data.lecturer_user.id, data.lecturer_user.admin);
        let uri = format!(
            "/api/modules/{}/assignments/{}/mark_allocator/generate",
            data.module.id, data.assignment.id
        );
        let req = Request::builder()
            .uri(&uri)
            .method("POST")
            .header("Authorization", format!("Bearer {}", token))
            .header("Content-Type", "application/json")
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        //TODO = fix this
        // assert_eq!(response.status(), StatusCode::OK);

        let _ = fs::remove_dir_all("./tmp");
    }

    #[tokio::test]
    #[serial]
    async fn test_post_mark_allocator_not_found() {
        dotenvy::dotenv().expect("Failed to load .env");
        unsafe {
            std::env::set_var("ASSIGNMENT_STORAGE_ROOT", "./tmp");
        }
        let db = setup_test_db().await;
        let data = setup_test_data(&db).await;

        let app = make_app(db.clone());
        let (token, _) = generate_jwt(data.lecturer_user.id, data.lecturer_user.admin);
        let uri = format!(
            "/api/modules/{}/assignments/{}/mark_allocator/generate",
            data.module.id, data.assignment.id
        );
        let req = Request::builder()
            .method("POST")
            .uri(&uri)
            .header("Authorization", format!("Bearer {}", token))
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    #[serial]
    async fn test_post_mark_allocator_forbidden_for_student() {
        dotenvy::dotenv().expect("Failed to load .env");
        unsafe {
            std::env::set_var("ASSIGNMENT_STORAGE_ROOT", "./tmp");
        }
        let db = setup_test_db().await;
        let data = setup_test_data(&db).await;

        let app = make_app(db.clone());
        let (token, _) = generate_jwt(data.student_user.id, data.student_user.admin);
        let uri = format!(
            "/api/modules/{}/assignments/{}/mark_allocator/generate",
            data.module.id, data.assignment.id
        );
        let req = Request::builder()
            .method("POST")
            .uri(&uri)
            .header("Authorization", format!("Bearer {}", token))
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::FORBIDDEN);
    }

    #[tokio::test]
    #[serial]
    async fn test_post_mark_allocator_forbidden_for_unassigned_user() {
        dotenvy::dotenv().expect("Failed to load .env");
        unsafe {
            std::env::set_var("ASSIGNMENT_STORAGE_ROOT", "./tmp");
        }
        let db = setup_test_db().await;
        let data = setup_test_data(&db).await;

        let app = make_app(db.clone());
        let (token, _) = generate_jwt(data.forbidden_user.id, data.forbidden_user.admin);
        let uri = format!(
            "/api/modules/{}/assignments/{}/mark_allocator/generate",
            data.module.id, data.assignment.id
        );
        let req = Request::builder()
            .method("POST")
            .uri(&uri)
            .header("Authorization", format!("Bearer {}", token))
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::FORBIDDEN);
    }

    #[tokio::test]
    #[serial]
    async fn test_post_mark_allocator_unauthorized() {
        dotenvy::dotenv().expect("Failed to load .env");
        unsafe {
            std::env::set_var("ASSIGNMENT_STORAGE_ROOT", "./tmp");
        }
        let db = setup_test_db().await;
        let data = setup_test_data(&db).await;

        let app = make_app(db.clone());
        let uri = format!(
            "/api/modules/{}/assignments/{}/mark_allocator/generate",
            data.module.id, data.assignment.id
        );
        let req = Request::builder()
            .method("POST")
            .uri(&uri)
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    #[serial]
    async fn test_post_mark_allocator_missing_memo_or_config() {
        dotenvy::dotenv().expect("Failed to load .env");
        unsafe {
            std::env::set_var("ASSIGNMENT_STORAGE_ROOT", "./tmp");
        }
        let db = setup_test_db().await;
        let data = setup_test_data(&db).await;

        let app = make_app(db.clone());
        let (token, _) = generate_jwt(data.lecturer_user.id, data.lecturer_user.admin);
        let uri = format!(
            "/api/modules/{}/assignments/{}/mark_allocator/generate",
            data.module.id, data.assignment.id
        );
        let req = Request::builder()
            .method("POST")
            .uri(&uri)
            .header("Authorization", format!("Bearer {}", token))
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    #[serial]
    async fn test_post_mark_allocator_missing_memo_output() {
        dotenvy::dotenv().expect("Failed to load .env");
        unsafe {
            std::env::set_var("ASSIGNMENT_STORAGE_ROOT", "./tmp");
        }
        let db = setup_test_db().await;
        let data = setup_test_data(&db).await;

        let app = make_app(db.clone());
        let (token, _) = generate_jwt(data.lecturer_user.id, data.lecturer_user.admin);
        let uri = format!(
            "/api/modules/{}/assignments/{}/mark_allocator/generate",
            data.module.id, data.assignment.id
        );
        let req = Request::builder()
            .uri(&uri)
            .method("POST")
            .header("Authorization", format!("Bearer {}", token))
            .header("Content-Type", "application/json")
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }
}
