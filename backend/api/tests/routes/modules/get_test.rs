#[cfg(test)]
mod tests {
    use db::{
        test_utils::setup_test_db,
        models::{
            user::Model as UserModel,
            module::Model as ModuleModel,
            user_module_role::{Model as UserModuleRoleModel, Role},
        },
    };
    use axum::{
        body::Body as AxumBody,
        http::{Request, StatusCode},
    };
    use tower::ServiceExt;
    use serde_json::Value;
    use api::auth::generate_jwt;
    use crate::test_helpers::make_app;
    use chrono::{Datelike, Utc};

    struct TestData {
        admin_user: UserModel,
        forbidden_user: UserModel,
        lecturer_user: UserModel,
        tutor_user: UserModel,
        student_user: UserModel,
        module: ModuleModel,
    }

    async fn setup_test_data(db: &sea_orm::DatabaseConnection) -> TestData {
        dotenvy::dotenv().expect("Failed to load .env");

        let module = ModuleModel::create(db, "MOD101", Utc::now().year() - 1, Some("Test Module Description"), 16).await.expect("Failed to create test module");
        let admin_user = UserModel::create(db, "module_admin", "module_admin@test.com", "password", true).await.expect("Failed to create admin user");
        let forbidden_user = UserModel::create(db, "module_forbidden", "module_forbidden@test.com", "password", false).await.expect("Failed to create forbidden user");
        let lecturer_user = UserModel::create(db, "module_lecturer", "module_lecturer@test.com", "password", false).await.expect("Failed to create lecturer user");
        let tutor_user = UserModel::create(db, "module_tutor", "module_tutor@test.com", "password", false).await.expect("Failed to create tutor user");
        let student_user = UserModel::create(db, "module_student", "module_student@test.com", "password", false).await.expect("Failed to create student user");
        UserModuleRoleModel::assign_user_to_module(db, lecturer_user.id, module.id, Role::Lecturer).await.expect("Failed to assign lecturer role");
        UserModuleRoleModel::assign_user_to_module(db, tutor_user.id, module.id, Role::Tutor).await.expect("Failed to assign tutor role");
        UserModuleRoleModel::assign_user_to_module(db, student_user.id, module.id, Role::Student).await.expect("Failed to assign student role");

        TestData {
            admin_user,
            forbidden_user,
            lecturer_user,
            tutor_user,
            student_user,
            module,
        }
    }

    /// Test Case: Successful Retrieval of Module Info as Admin
    #[tokio::test]
    async fn test_get_module_success_as_admin() {
        let db = setup_test_db().await;
        let data = setup_test_data(&db).await;

        let app = make_app(db.clone());
        let (token, _) = generate_jwt(data.admin_user.id, data.admin_user.admin);
        let uri = format!("/api/modules/{}", data.module.id);
        let req = Request::builder()
            .method("GET")
            .uri(&uri)
            .header("Authorization", format!("Bearer {}", token))
            .body(AxumBody::empty())
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let json: Value = serde_json::from_slice(&body).unwrap();

        assert_eq!(json["success"], true);
        assert_eq!(json["message"], "Module retrieved successfully");
        let module_data = &json["data"];

        assert_eq!(module_data["id"], data.module.id);
        assert_eq!(module_data["code"], data.module.code);
        assert_eq!(module_data["year"], data.module.year);
        assert_eq!(module_data["description"], data.module.description.unwrap_or_default());
        assert_eq!(module_data["credits"], data.module.credits);
        assert!(module_data["created_at"].as_str().is_some());
        assert!(module_data["updated_at"].as_str().is_some());

        let lecturers = module_data["lecturers"].as_array().expect("Lecturers should be an array");
        assert_eq!(lecturers.len(), 1);
        assert_eq!(lecturers[0]["id"], data.lecturer_user.id);

        let tutors = module_data["tutors"].as_array().expect("Tutors should be an array");
        assert_eq!(tutors.len(), 1);
        assert_eq!(tutors[0]["id"], data.tutor_user.id);

        let students = module_data["students"].as_array().expect("Students should be an array");
        assert_eq!(students.len(), 1);
        assert_eq!(students[0]["id"], data.student_user.id);
    }

    /// Test Case: Successful Retrieval of Module Info as Lecturer
    #[tokio::test]
    async fn test_get_module_success_as_lecturer() {
        let db = setup_test_db().await;
        let data = setup_test_data(&db).await;

        let app = make_app(db.clone());
        let (token, _) = generate_jwt(data.lecturer_user.id, data.lecturer_user.admin);
        let uri = format!("/api/modules/{}", data.module.id);
        let req = Request::builder()
            .method("GET")
            .uri(&uri)
            .header("Authorization", format!("Bearer {}", token))
            .body(AxumBody::empty())
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
    }

    /// Test Case: Successful Retrieval of Module Info as Tutor
    #[tokio::test]
    async fn test_get_module_success_as_tutor() {
        let db = setup_test_db().await;
        let data = setup_test_data(&db).await;

        let app = make_app(db.clone());
        let (token, _) = generate_jwt(data.tutor_user.id, data.tutor_user.admin);
        let uri = format!("/api/modules/{}", data.module.id);
        let req = Request::builder()
            .method("GET")
            .uri(&uri)
            .header("Authorization", format!("Bearer {}", token))
            .body(AxumBody::empty())
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
    }

    /// Test Case: Successful Retrieval of Module Info as Student
    #[tokio::test]
    async fn test_get_module_success_as_student() {
        let db = setup_test_db().await;
        let data = setup_test_data(&db).await;

        let app = make_app(db.clone());
        let (token, _) = generate_jwt(data.student_user.id, data.student_user.admin);
        let uri = format!("/api/modules/{}", data.module.id);
        let req = Request::builder()
            .method("GET")
            .uri(&uri)
            .header("Authorization", format!("Bearer {}", token))
            .body(AxumBody::empty())
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
    }

    /// Test Case: Retrieving Non-Existent Module
    #[tokio::test]
    async fn test_get_module_not_found() {
        let db = setup_test_db().await;
        let data = setup_test_data(&db).await;

        let app = make_app(db.clone());
        let (token, _) = generate_jwt(data.admin_user.id, data.admin_user.admin);
        let uri = format!("/api/modules/{}", 99999);
        let req = Request::builder()
            .method("GET")
            .uri(&uri)
            .header("Authorization", format!("Bearer {}", token))
            .body(AxumBody::empty())
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::NOT_FOUND);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let json: Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(json["success"], false);
        assert_eq!(json["message"], format!("Module 99999 not found."));
    }

    /// Test Case: Accessing Module without Required Role (Forbidden)
    #[tokio::test]
    async fn test_get_module_forbidden_user() {
        let db = setup_test_db().await;
        let data = setup_test_data(&db).await;

        let app = make_app(db.clone());
        let (token, _) = generate_jwt(data.forbidden_user.id, data.forbidden_user.admin);
        let uri = format!("/api/modules/{}", data.module.id);
        let req = Request::builder()
            .method("GET")
            .uri(&uri)
            .header("Authorization", format!("Bearer {}", token))
            .body(AxumBody::empty())
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::FORBIDDEN);
    }

    /// Test Case: Accessing Module without Authorization Header
    #[tokio::test]
    async fn test_get_module_missing_auth_header() {
        let db = setup_test_db().await;
        let data = setup_test_data(&db).await;

        let app = make_app(db.clone());
        let uri = format!("/api/modules/{}", data.module.id);
        let req = Request::builder()
            .method("GET")
            .uri(&uri)
            .body(AxumBody::empty())
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    /// Test Case: Accessing Module with Invalid JWT Token
    #[tokio::test]
    async fn test_get_module_invalid_token() {
        let db = setup_test_db().await;
        let data = setup_test_data(&db).await;

        let app = make_app(db.clone());
        let uri = format!("/api/modules/{}", data.module.id);
        let req = Request::builder()
            .method("GET")
            .uri(&uri)
            .header("Authorization", "Bearer invalid.token.here")
            .body(AxumBody::empty())
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    /// Test Case: Module Info Includes Correct User Details in Personnel
    #[tokio::test]
    async fn test_get_module_personnel_user_details() {
        let db = setup_test_db().await;
        let data = setup_test_data(&db).await;

        let app = make_app(db.clone());
        let (token, _) = generate_jwt(data.admin_user.id, data.admin_user.admin);
        let uri = format!("/api/modules/{}", data.module.id);
        let req = Request::builder()
            .method("GET")
            .uri(&uri)
            .header("Authorization", format!("Bearer {}", token))
            .body(AxumBody::empty())
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let json: Value = serde_json::from_slice(&body).unwrap();
        let module_data = &json["data"];

        let lecturers = module_data["lecturers"].as_array().unwrap();
        assert_eq!(lecturers.len(), 1);
        let lecturer = &lecturers[0];
        assert_eq!(lecturer["id"], data.lecturer_user.id);
        assert_eq!(lecturer["username"], data.lecturer_user.username);
        assert_eq!(lecturer["email"], data.lecturer_user.email);

        let tutors = module_data["tutors"].as_array().unwrap();
        assert_eq!(tutors.len(), 1);
        let tutor = &tutors[0];
        assert_eq!(tutor["id"], data.tutor_user.id);
        assert_eq!(tutor["username"], data.tutor_user.username);
        assert_eq!(tutor["email"], data.tutor_user.email);

        let students = module_data["students"].as_array().unwrap();
        assert_eq!(students.len(), 1);
        let student = &students[0];
        assert_eq!(student["id"], data.student_user.id);
        assert_eq!(student["username"], data.student_user.username);
        assert_eq!(student["email"], data.student_user.email);
    }

    /// Test Case: Module with No Assigned Personnel
    #[tokio::test]
    async fn test_get_module_no_personnel() {
        let db = setup_test_db().await;
        dotenvy::dotenv().expect("Failed to load .env");

        let empty_module = ModuleModel::create(&db, "EMPTY101", Utc::now().year() - 1, Some("Empty Module"), 10).await.expect("Failed to create empty module");
        let admin_user = UserModel::create(&db, "empty_admin", "empty_admin@test.com", "password", true).await.expect("Failed to create admin user for empty module test");

        let app = make_app(db.clone());
        let (token, _) = generate_jwt(admin_user.id, admin_user.admin);
        let uri = format!("/api/modules/{}", empty_module.id);
        let req = Request::builder()
            .method("GET")
            .uri(&uri)
            .header("Authorization", format!("Bearer {}", token))
            .body(AxumBody::empty())
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let json: Value = serde_json::from_slice(&body).unwrap();

        assert_eq!(json["success"], true);
        let module_data = &json["data"];

        assert_eq!(module_data["id"], empty_module.id);
        assert_eq!(module_data["code"], empty_module.code);

        let lecturers = module_data["lecturers"].as_array().expect("Lecturers should be an array");
        assert_eq!(lecturers.len(), 0);

        let tutors = module_data["tutors"].as_array().expect("Tutors should be an array");
        assert_eq!(tutors.len(), 0);

        let students = module_data["students"].as_array().expect("Students should be an array");
        assert_eq!(students.len(), 0);
    }

    /// Test Case: Module with Multiple Users per Role
    #[tokio::test]
    async fn test_get_module_multiple_personnel_per_role() {
        let db = setup_test_db().await;
        let data = setup_test_data(&db).await;

        let lecturer2 = UserModel::create(&db, "module_lecturer2", "module_lecturer2@test.com", "password", false).await.expect("Failed to create second lecturer");
        let tutor2 = UserModel::create(&db, "module_tutor2", "module_tutor2@test.com", "password", false).await.expect("Failed to create second tutor");
        let student2 = UserModel::create(&db, "module_student2", "module_student2@test.com", "password", false).await.expect("Failed to create second student");
        let student3 = UserModel::create(&db, "module_student3", "module_student3@test.com", "password", false).await.expect("Failed to create third student");
        UserModuleRoleModel::assign_user_to_module(&db, lecturer2.id, data.module.id, Role::Lecturer).await.expect("Failed to assign second lecturer");
        UserModuleRoleModel::assign_user_to_module(&db, tutor2.id, data.module.id, Role::Tutor).await.expect("Failed to assign second tutor");
        UserModuleRoleModel::assign_user_to_module(&db, student2.id, data.module.id, Role::Student).await.expect("Failed to assign second student");
        UserModuleRoleModel::assign_user_to_module(&db, student3.id, data.module.id, Role::Student).await.expect("Failed to assign third student");

        let app = make_app(db.clone());
        let (token, _) = generate_jwt(data.admin_user.id, data.admin_user.admin);
        let uri = format!("/api/modules/{}", data.module.id);
        let req = Request::builder()
            .method("GET")
            .uri(&uri)
            .header("Authorization", format!("Bearer {}", token))
            .body(AxumBody::empty())
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let json: Value = serde_json::from_slice(&body).unwrap();
        let module_data = &json["data"];

        let lecturers = module_data["lecturers"].as_array().unwrap();
        assert_eq!(lecturers.len(), 2);

        let tutors = module_data["tutors"].as_array().unwrap();
        assert_eq!(tutors.len(), 2);

        let students = module_data["students"].as_array().unwrap();
        assert_eq!(students.len(), 3);

        let lecturer_ids: Vec<i64> = lecturers.iter().map(|l| l["id"].as_i64().unwrap()).collect();
        assert!(lecturer_ids.contains(&data.lecturer_user.id));
        assert!(lecturer_ids.contains(&lecturer2.id));

        let tutor_ids: Vec<i64> = tutors.iter().map(|t| t["id"].as_i64().unwrap()).collect();
        assert!(tutor_ids.contains(&data.tutor_user.id));
        assert!(tutor_ids.contains(&tutor2.id));

        let student_ids: Vec<i64> = students.iter().map(|s| s["id"].as_i64().unwrap()).collect();
        assert!(student_ids.contains(&data.student_user.id));
        assert!(student_ids.contains(&student2.id));
        assert!(student_ids.contains(&student3.id));
    }
}