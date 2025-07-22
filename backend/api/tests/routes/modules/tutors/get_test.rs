#[cfg(test)]
mod tests {
    use db::{test_utils::setup_test_db, models::{user::Model as UserModel, module::Model as ModuleModel, user_module_role::{Model as UserModuleRoleModel, Role}}};
    use axum::{body::Body, http::{Request, StatusCode}};
    use tower::ServiceExt;
    use serde_json::Value;
    use api::auth::generate_jwt;
    use dotenvy;
    use crate::test_helpers::make_app;

    struct TestData {
        admin_user: UserModel,
        forbidden_user: UserModel,
        module: ModuleModel,
        student1: UserModel,
        lecturer1: UserModel,
    }

    async fn setup_test_data(db: &sea_orm::DatabaseConnection) -> TestData {
        let module = ModuleModel::create(db, "COS601", 2024, Some("Test Module"), 16).await.unwrap();
        let admin_user = UserModel::create(db, "admin1", "admin1@test.com", "password", true).await.unwrap();
        let forbidden_user = UserModel::create(db, "unauthed", "unauthed@test.com", "password", false).await.unwrap();
        let tutor1 = UserModel::create(db, "tutor1", "tutor1@test.com", "password1", false).await.unwrap();
        let tutor2 = UserModel::create(db, "tutor2", "tutor2@test.com", "password2", false).await.unwrap();
        let tutor3 = UserModel::create(db, "tutor3", "tutor3@test.com", "password3", false).await.unwrap();
        let student1 = UserModel::create(db, "student1", "student1@test.com", "password4", false).await.unwrap();
        let lecturer1 = UserModel::create(db, "lecturer1", "lecturer1@test.com", "password5", false).await.unwrap();
        UserModuleRoleModel::assign_user_to_module(db, tutor1.id, module.id, Role::Tutor).await.unwrap();
        UserModuleRoleModel::assign_user_to_module(db, tutor2.id, module.id, Role::Tutor).await.unwrap();
        UserModuleRoleModel::assign_user_to_module(db, tutor3.id, module.id, Role::Tutor).await.unwrap();
        UserModuleRoleModel::assign_user_to_module(db, student1.id, module.id, Role::Student).await.unwrap();
        UserModuleRoleModel::assign_user_to_module(db, lecturer1.id, module.id, Role::Lecturer).await.unwrap();

        TestData {
            admin_user,
            forbidden_user,
            module,
            student1,
            lecturer1,
        }
    }

    /// Test Case 1: Successful Retrieval of Tutors as Admin
    #[tokio::test]
    async fn test_get_tutors_success_as_admin() {
        dotenvy::dotenv().expect("Failed to load .env");
        let db = setup_test_db().await;
        let data = setup_test_data(&db).await;

        let app = make_app(db.clone());
        let (token, _) = generate_jwt(data.admin_user.id, data.admin_user.admin);
        let uri = format!("/api/modules/{}/tutors", data.module.id);
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
        let usernames: Vec<_> = json["data"]["users"].as_array().unwrap().iter().map(|u| u["username"].as_str().unwrap()).collect();
        assert!(usernames.contains(&"tutor1"));
        assert!(usernames.contains(&"tutor2"));
        assert!(usernames.contains(&"tutor3"));
    }

    /// Test Case 2: Forbidden Retrieval of Tutors as Student
    #[tokio::test]
    async fn test_get_tutors_forbidden_as_student() {
        dotenvy::dotenv().expect("Failed to load .env");
        let db = setup_test_db().await;
        let data = setup_test_data(&db).await;

        let app = make_app(db.clone());
        let (token, _) = generate_jwt(data.student1.id, data.student1.admin);
        let uri = format!("/api/modules/{}/tutors", data.module.id);
        let req = Request::builder()
            .uri(&uri)
            .header("Authorization", format!("Bearer {}", token))
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::FORBIDDEN);
    }

    /// Test Case 3: Forbidden Retrieval of Tutors as Lecturer
    #[tokio::test]
    async fn test_get_tutors_forbidden_as_lecturer() {
        dotenvy::dotenv().expect("Failed to load .env");
        let db = setup_test_db().await;
        let data = setup_test_data(&db).await;

        let app = make_app(db.clone());
        let (token, _) = generate_jwt(data.lecturer1.id, data.lecturer1.admin);
        let uri = format!("/api/modules/{}/tutors", data.module.id);
        let req = Request::builder()
            .uri(&uri)
            .header("Authorization", format!("Bearer {}", token))
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::FORBIDDEN);
    }

    /// Test Case 4: Module 9999 not found.
    #[tokio::test]
    async fn test_get_tutors_not_found() {
        dotenvy::dotenv().expect("Failed to load .env");
        let db = setup_test_db().await;
        let data = setup_test_data(&db).await;

        let app = make_app(db.clone());
        let (token, _) = generate_jwt(data.admin_user.id, data.admin_user.admin);
        let uri = format!("/api/modules/{}/tutors", 9999);
        let req = Request::builder()
            .uri(&uri)
            .header("Authorization", format!("Bearer {}", token))
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::NOT_FOUND);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let json: Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(json["success"], false);
        assert_eq!(json["message"], "Module 9999 not found.");
    }

    /// Test Case 5: Forbidden Access (Unassigned User)
    #[tokio::test]
    async fn test_get_tutors_forbidden() {
        dotenvy::dotenv().expect("Failed to load .env");
        let db = setup_test_db().await;
        let data = setup_test_data(&db).await;

        let app = make_app(db.clone());
        let (token, _) = generate_jwt(data.forbidden_user.id, data.forbidden_user.admin);
        let uri = format!("/api/modules/{}/tutors", data.module.id);
        let req = Request::builder()
            .uri(&uri)
            .header("Authorization", format!("Bearer {}", token))
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::FORBIDDEN);
    }

    /// Test Case 6: Filtering and Sorting
    #[tokio::test]
    async fn test_get_tutors_with_query_params() {
        dotenvy::dotenv().expect("Failed to load .env");
        let db = setup_test_db().await;
        let data = setup_test_data(&db).await;

        let app = make_app(db.clone());
        let (token, _) = generate_jwt(data.admin_user.id, data.admin_user.admin);
        let uri = format!("/api/modules/{}/tutors?query=tutor&sort=-email", data.module.id);
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
        let emails: Vec<_> = json["data"]["users"].as_array().unwrap().iter().map(|u| u["email"].as_str().unwrap()).collect();
        let mut expected = vec!["tutor3@test.com", "tutor2@test.com", "tutor1@test.com"];
        expected.sort_by(|a, b| b.cmp(a));
        assert_eq!(emails, expected);
    }

    /// Test Case 7: Pagination
    #[tokio::test]
    async fn test_get_tutors_pagination() {
        dotenvy::dotenv().expect("Failed to load .env");
        let db = setup_test_db().await;
        let data = setup_test_data(&db).await;

        let app = make_app(db.clone());
        let (token, _) = generate_jwt(data.admin_user.id, data.admin_user.admin);
        let uri = format!("/api/modules/{}/tutors?page=2&per_page=2", data.module.id);
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
        assert_eq!(json["data"]["page"], 2);
        assert_eq!(json["data"]["per_page"], 2);
        let users = json["data"]["users"].as_array().unwrap();
        assert_eq!(users.len(), 1);
        let usernames: Vec<_> = users.iter().map(|u| u["username"].as_str().unwrap()).collect();
        assert!(usernames[0] == "tutor3" || usernames[0] == "tutor2" || usernames[0] == "tutor1");
    }
}