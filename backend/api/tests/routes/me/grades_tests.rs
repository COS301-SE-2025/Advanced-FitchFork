#[cfg(test)]
mod tests {
    use crate::helpers::app::make_test_app_with_storage;
    use api::auth::generate_jwt;
    use axum::{
        body::Body as AxumBody,
        http::{Request, StatusCode},
    };
    use chrono::Utc;
    use db::models::{
        assignment::Model as AssignmentModel,
        assignment_submission::Model as SubmissionModel,
        module::Model as ModuleModel,
        user::Model as UserModel,
        user_module_role::{Model as UserModuleRoleModel, Role},
    };
    use serde_json::Value;
    use serial_test::serial;
    use tower::ServiceExt;

    struct TestData {
        student1: UserModel,
        lecturer: UserModel,
        submission1: SubmissionModel,
    }

    async fn setup_test_data(db: &sea_orm::DatabaseConnection) -> TestData {
        let student1 = UserModel::create(db, "student1", "s1@test.com", "p1", false)
            .await
            .unwrap();
        let student2 = UserModel::create(db, "student2", "s2@test.com", "p2", false)
            .await
            .unwrap();
        let lecturer = UserModel::create(db, "lecturer", "l1@test.com", "p3", false)
            .await
            .unwrap();

        let module1 = ModuleModel::create(db, "COS101", 2024, None, 15)
            .await
            .unwrap();
        let module2 = ModuleModel::create(db, "COS212", 2025, None, 15)
            .await
            .unwrap();

        UserModuleRoleModel::assign_user_to_module(db, student1.id, module1.id, Role::Student)
            .await
            .unwrap();
        UserModuleRoleModel::assign_user_to_module(db, student2.id, module1.id, Role::Student)
            .await
            .unwrap();
        UserModuleRoleModel::assign_user_to_module(db, lecturer.id, module1.id, Role::Lecturer)
            .await
            .unwrap();

        let assignment1 = AssignmentModel::create(
            db,
            module1.id,
            "A01",
            None,
            db::models::assignment::AssignmentType::Practical,
            Utc::now(),
            Utc::now(),
        )
        .await
        .unwrap();
        let _assignment2 = AssignmentModel::create(
            db,
            module2.id,
            "A02",
            None,
            db::models::assignment::AssignmentType::Practical,
            Utc::now(),
            Utc::now(),
        )
        .await
        .unwrap();

        let submission1 = SubmissionModel::save_file(
            db,
            assignment1.id,
            student1.id,
            1,
            80.0,
            100.0,
            false,
            "s1.zip",
            "hash1",
            "output1".as_bytes(),
        )
        .await
        .unwrap();
        let _submission2 = SubmissionModel::save_file(
            db,
            assignment1.id,
            student2.id,
            1,
            90.0,
            100.0,
            false,
            "s2.zip",
            "hash2",
            "output2".as_bytes(),
        )
        .await
        .unwrap();

        TestData {
            student1,
            lecturer,
            submission1,
        }
    }

    #[tokio::test]
    #[serial]
    async fn test_get_grades_as_student_success() {
        let (app, app_state, _tmp) = make_test_app_with_storage().await;
        let data = setup_test_data(app_state.db()).await;

        let (token, _) = generate_jwt(data.student1.id, false);
        let req = Request::builder()
            .method("GET")
            .uri("/api/me/grades")
            .header("Authorization", format!("Bearer {}", token))
            .body(AxumBody::empty())
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let json: Value = serde_json::from_slice(&body).unwrap();

        assert_eq!(json["success"], true);
        let grades = json["data"]["grades"].as_array().unwrap();
        assert_eq!(grades.len(), 1);
        assert_eq!(grades[0]["id"], data.submission1.id);
    }

    #[tokio::test]
    #[serial]
    async fn test_get_grades_as_lecturer_success() {
        let (app, app_state, _tmp) = make_test_app_with_storage().await;
        let data = setup_test_data(app_state.db()).await;

        let (token, _) = generate_jwt(data.lecturer.id, false);
        let req = Request::builder()
            .method("GET")
            .uri("/api/me/grades?role=lecturer")
            .header("Authorization", format!("Bearer {}", token))
            .body(AxumBody::empty())
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let json: Value = serde_json::from_slice(&body).unwrap();

        assert_eq!(json["success"], true);
        let grades = json["data"]["grades"].as_array().unwrap();
        assert_eq!(grades.len(), 2);
    }

    #[tokio::test]
    #[serial]
    async fn test_get_grades_unauthorized() {
        let (app, _app_state, _tmp) = make_test_app_with_storage().await;
        let req = Request::builder()
            .method("GET")
            .uri("/api/me/grades")
            .body(AxumBody::empty())
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    #[serial]
    async fn test_get_grades_filter_by_year() {
        let (app, app_state, _tmp) = make_test_app_with_storage().await;
        let data = setup_test_data(app_state.db()).await;

        let (token, _) = generate_jwt(data.lecturer.id, false);
        let req = Request::builder()
            .method("GET")
            .uri("/api/me/grades?role=lecturer&year=2024")
            .header("Authorization", format!("Bearer {}", token))
            .body(AxumBody::empty())
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let json: Value = serde_json::from_slice(&body).unwrap();
        let grades = json["data"]["grades"].as_array().unwrap();
        assert_eq!(grades.len(), 2);
    }

    #[tokio::test]
    #[serial]
    async fn test_get_grades_filter_by_query() {
        let (app, app_state, _tmp) = make_test_app_with_storage().await;
        let data = setup_test_data(app_state.db()).await;

        let (token, _) = generate_jwt(data.lecturer.id, false);
        let req = Request::builder()
            .method("GET")
            .uri("/api/me/grades?role=lecturer&query=student1")
            .header("Authorization", format!("Bearer {}", token))
            .body(AxumBody::empty())
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let json: Value = serde_json::from_slice(&body).unwrap();
        let grades = json["data"]["grades"].as_array().unwrap();
        assert_eq!(grades.len(), 1);
        assert_eq!(grades[0]["user"]["username"], "student1");
    }

    #[tokio::test]
    #[serial]
    async fn test_get_grades_sort_by_score_asc() {
        let (app, app_state, _tmp) = make_test_app_with_storage().await;
        let data = setup_test_data(app_state.db()).await;

        let (token, _) = generate_jwt(data.lecturer.id, false);
        let req = Request::builder()
            .method("GET")
            .uri("/api/me/grades?role=lecturer&sort=score")
            .header("Authorization", format!("Bearer {}", token))
            .body(AxumBody::empty())
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let json: Value = serde_json::from_slice(&body).unwrap();
        let grades = json["data"]["grades"].as_array().unwrap();
        assert_eq!(grades[0]["score"]["earned"], 80);
        assert_eq!(grades[1]["score"]["earned"], 90);
    }

    #[tokio::test]
    #[serial]
    async fn test_get_grades_sort_by_score_desc() {
        let (app, app_state, _tmp) = make_test_app_with_storage().await;
        let data = setup_test_data(app_state.db()).await;

        let (token, _) = generate_jwt(data.lecturer.id, false);
        let req = Request::builder()
            .method("GET")
            .uri("/api/me/grades?role=lecturer&sort=-score")
            .header("Authorization", format!("Bearer {}", token))
            .body(AxumBody::empty())
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let json: Value = serde_json::from_slice(&body).unwrap();
        let grades = json["data"]["grades"].as_array().unwrap();
        assert_eq!(grades[0]["score"]["earned"], 90);
        assert_eq!(grades[1]["score"]["earned"], 80);
    }

    #[tokio::test]
    #[serial]
    async fn test_get_grades_pagination() {
        let (app, app_state, _tmp) = make_test_app_with_storage().await;
        let data = setup_test_data(app_state.db()).await;

        let (token, _) = generate_jwt(data.lecturer.id, false);
        let req = Request::builder()
            .method("GET")
            .uri("/api/me/grades?role=lecturer&page=2&per_page=1")
            .header("Authorization", format!("Bearer {}", token))
            .body(AxumBody::empty())
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let json: Value = serde_json::from_slice(&body).unwrap();
        let grades = json["data"]["grades"].as_array().unwrap();
        assert_eq!(grades.len(), 1);
        assert_eq!(json["data"]["page"], 2);
        assert_eq!(json["data"]["per_page"], 1);
    }

    #[tokio::test]
    #[serial]
    async fn test_get_grades_invalid_page() {
        let (app, app_state, _tmp) = make_test_app_with_storage().await;
        let data = setup_test_data(app_state.db()).await;

        let (token, _) = generate_jwt(data.lecturer.id, false);
        let req = Request::builder()
            .method("GET")
            .uri("/api/me/grades?role=lecturer&page=0")
            .header("Authorization", format!("Bearer {}", token))
            .body(AxumBody::empty())
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    #[serial]
    async fn test_get_grades_no_grades_found() {
        let (app, app_state, _tmp) = make_test_app_with_storage().await;
        let data = setup_test_data(app_state.db()).await;

        let (token, _) = generate_jwt(data.lecturer.id, false);
        let req = Request::builder()
            .method("GET")
            .uri("/api/me/grades?role=lecturer&year=2026")
            .header("Authorization", format!("Bearer {}", token))
            .body(AxumBody::empty())
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let json: Value = serde_json::from_slice(&body).unwrap();
        let grades = json["data"]["grades"].as_array().unwrap();
        assert_eq!(grades.len(), 0);
    }
}
