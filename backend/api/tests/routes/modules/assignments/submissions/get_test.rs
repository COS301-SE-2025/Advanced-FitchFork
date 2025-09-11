#[cfg(test)]
mod tests {
    use db::{
        models::{
            user::Model as UserModel,
            module::Model as ModuleModel,
            assignment::Model as AssignmentModel,
            assignment_submission::Model as AssignmentSubmissionModel,
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
        user::{CreateUser, UserService}
    };
    use tower::ServiceExt;
    use serde_json::{json, Value};
    use api::auth::generate_jwt;
    use dotenvy;
    use chrono::{Duration, Utc};
    use std::{fs, path::PathBuf};
    use crate::helpers::app::make_test_app;
    use serial_test::serial;
    use sea_orm::{Set, ActiveModelTrait};
    use tempfile::{TempDir, tempdir};

    struct TestData {
        lecturer_user: UserModel,
        student_user: UserModel,
        forbidden_user: UserModel,
        module: ModuleModel,
        assignment: AssignmentModel,
        submissions: Vec<AssignmentSubmissionModel>,
    }

    async fn setup_test_data(db: &sea_orm::DatabaseConnection) -> (TestData, TempDir) {
        dotenvy::dotenv().expect("Failed to load .env");
        let temp_dir = tempdir().expect("Failed to create temporary directory");
        unsafe{ std::env::set_var("ASSIGNMENT_STORAGE_ROOT", temp_dir.path().to_str().unwrap()); }

        let module = ModuleModel::create(db, "COS101", 2024, Some("Test Module"), 16).await.unwrap();
        let service = UserService::new(UserRepository::new(db.clone()));
        let lecturer_user = service.create(CreateUser{ username: "lecturer1".to_string(), email: "lecturer1@test.com".to_string(), password: "password1".to_string(), admin: false }).await.unwrap();
        let student_user = service.create(CreateUser{ username: "student1".to_string(), email: "student1@test.com".to_string(), password: "password2".to_string(), admin: false }).await.unwrap();
        let forbidden_user = service.create(CreateUser{ username: "forbidden".to_string(), email: "forbidden@test.com".to_string(), password: "password3".to_string(), admin: false }).await.unwrap();
        UserModuleRoleModel::assign_user_to_module(db, lecturer_user.id, module.id, Role::Lecturer).await.unwrap();
        UserModuleRoleModel::assign_user_to_module(db, student_user.id, module.id, Role::Student).await.unwrap();
        let assignment = AssignmentModel::create(
            db,
            module.id,
            "Assignment 1",
            Some("Desc 1"),
            db::models::assignment::AssignmentType::Assignment,
            Utc::now(),
            Utc::now() + Duration::days(30)
        ).await.unwrap();

        let sub1 = AssignmentSubmissionModel::save_file(db, assignment.id, student_user.id, 1, 10, 10, false, "ontime.txt", "hash123#", b"ontime").await.unwrap();
        let sub1_time = assignment.due_date - Duration::days(1);
        update_submission_time(db, sub1.id, sub1_time).await;
        write_submission_report(temp_dir.path().to_str().unwrap(), module.id, assignment.id, student_user.id, 1, &sub1, false, false, Some(json!({"earned": 80, "total": 100})), sub1_time);

        let sub2 = AssignmentSubmissionModel::save_file(db, assignment.id, student_user.id, 2, 10, 10, false, "late.txt", "hash123#", b"late").await.unwrap();
        let sub2_time = assignment.due_date + Duration::days(1);
        update_submission_time(db, sub2.id, sub2_time).await;
        write_submission_report(temp_dir.path().to_str().unwrap(), module.id, assignment.id, student_user.id, 2, &sub2, false, false, Some(json!({"earned": 50, "total": 100})), sub2_time);

        let sub3 = AssignmentSubmissionModel::save_file(db, assignment.id, student_user.id, 3, 10, 10, false, "practice.txt", "hash123#", b"practice").await.unwrap();
        let sub3_time = assignment.due_date - Duration::days(2);
        update_submission_time(db, sub3.id, sub3_time).await;
        write_submission_report(temp_dir.path().to_str().unwrap(), module.id, assignment.id, student_user.id, 3, &sub3, true, false, Some(json!({"earned": 100, "total": 100})), sub3_time);

        let sub4 = AssignmentSubmissionModel::save_file(db, assignment.id, forbidden_user.id, 1, 10, 10, false, "forbidden.txt", "hash123#", b"forbidden").await.unwrap();
        let sub4_time = assignment.due_date - Duration::days(1);
        update_submission_time(db, sub4.id, sub4_time).await;
        write_submission_report(temp_dir.path().to_str().unwrap(), module.id, assignment.id, forbidden_user.id, 1, &sub4, false, false, Some(json!({"earned": 0, "total": 100})), sub4_time);

        // mark one submission as ignored so we can assert both states
        AssignmentSubmissionModel::set_ignored(db, sub2.id, true).await.unwrap();

        let submissions = [sub1, sub2, sub3, sub4].to_vec();

        (
            TestData {
                lecturer_user,
                student_user,
                forbidden_user,
                module,
                assignment,
                submissions,
            },
            temp_dir
        )
    }

    async fn update_submission_time(db: &sea_orm::DatabaseConnection, submission_id: i64, new_time: chrono::DateTime<Utc>) {
        use db::models::assignment_submission::{ActiveModel, Entity};
        use sea_orm::EntityTrait;
        if let Some(model) = Entity::find_by_id(submission_id).one(db).await.unwrap() {
            let mut active: ActiveModel = model.into();
            active.created_at = Set(new_time);
            active.updated_at = Set(new_time);
            let _ = active.update(db).await;
        }
    }

    fn write_submission_report(base: &str, module_id: i64, assignment_id: i64, user_id: i64, attempt: i64, submission: &AssignmentSubmissionModel, is_practice: bool, is_late: bool, mark: Option<Value>, created_at: chrono::DateTime<Utc>) {
        let path = PathBuf::from(base)
            .join(format!("module_{}", module_id))
            .join(format!("assignment_{}", assignment_id))
            .join("assignment_submissions")
            .join(format!("user_{}", user_id))
            .join(format!("attempt_{}", attempt))
            .join("submission_report.json");
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        let mut report = json!({
            "id": submission.id,
            "attempt": attempt,
            "filename": submission.filename,
            "created_at": created_at.to_rfc3339(),
            "updated_at": created_at.to_rfc3339(),
            "is_practice": is_practice,
            "is_late": is_late,
        });
        if let Some(m) = mark {
            report["mark"] = m;
        }
        fs::write(&path, serde_json::to_string_pretty(&report).unwrap()).unwrap();
    }

    // --- GET /api/modules/{module_id}/assignments/{assignment_id}/submissions ---

    #[tokio::test]
    #[serial]
    async fn test_student_sees_only_own_submissions() {
        let app = make_test_app().await;
        let (data, _temp_dir) = setup_test_data(db::get_connection().await).await;

        let (token, _) = generate_jwt(data.student_user.id, data.student_user.admin);
        let uri = format!("/api/modules/{}/assignments/{}/submissions", data.module.id, data.assignment.id);
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

        let submissions = json["data"]["submissions"].as_array().unwrap();
        assert_eq!(submissions.len(), 3);

        // each item has boolean `ignored`
        assert!(submissions.iter().all(|s| s["ignored"].is_boolean()));
        // we set one (sub2) to true
        assert!(submissions.iter().any(|s| s["ignored"] == true));
    }

    #[tokio::test]
    #[serial]
    async fn test_lecturer_sees_all_submissions_with_pagination() {
        let app = make_test_app().await;
        let (data, _temp_dir) = setup_test_data(db::get_connection().await).await;

        let (token, _) = generate_jwt(data.lecturer_user.id, data.lecturer_user.admin);
        let uri = format!("/api/modules/{}/assignments/{}/submissions?per_page=2&page=1", data.module.id, data.assignment.id);
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
        assert_eq!(json["data"]["per_page"], 2);
        assert_eq!(json["data"]["page"], 1);
        assert_eq!(json["data"]["total"], 4);

        let subs = json["data"]["submissions"].as_array().unwrap();
        assert_eq!(subs.len(), 2);
        assert!(subs.iter().all(|s| s["ignored"].is_boolean()));
    }

    #[tokio::test]
    #[serial]
    async fn test_query_by_username_returns_only_that_user() {
        let app = make_test_app().await;
        let (data, _temp_dir) = setup_test_data(db::get_connection().await).await;

        let (token, _) = generate_jwt(data.lecturer_user.id, data.lecturer_user.admin);
        let uri = format!("/api/modules/{}/assignments/{}/submissions?username=student1", data.module.id, data.assignment.id);
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

        let subs = json["data"]["submissions"].as_array().unwrap();
        assert_eq!(subs.len(), 3);
        assert!(subs.iter().all(|s| s["ignored"].is_boolean()));
    }

    #[tokio::test]
    #[serial]
    async fn test_filter_by_late_status() {
        let app = make_test_app().await;
        let (data, _temp_dir) = setup_test_data(db::get_connection().await).await;

        let (token, _) = generate_jwt(data.lecturer_user.id, data.lecturer_user.admin);
        let uri = format!("/api/modules/{}/assignments/{}/submissions?late=true", data.module.id, data.assignment.id);
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

        let subs = json["data"]["submissions"].as_array().unwrap();
        assert_eq!(subs.len(), 1);
        assert!(subs.iter().all(|s| s["ignored"].is_boolean()));
    }

    #[tokio::test]
    #[serial]
    async fn test_forbidden_user_gets_403() {
        let app = make_test_app().await;
        let (data, _temp_dir) = setup_test_data(db::get_connection().await).await;

        let (token, _) = generate_jwt(data.forbidden_user.id, data.forbidden_user.admin);
        let uri = format!("/api/modules/{}/assignments/{}/submissions", data.module.id, data.assignment.id);
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
    async fn test_filter_by_ignored_true_lecturer() {
        let (app, app_state) = make_test_app().await;
        let (data, _temp_dir) = setup_test_data(app_state.db()).await;

        let (token, _) = generate_jwt(data.lecturer_user.id, data.lecturer_user.admin);
        let uri = format!(
            "/api/modules/{}/assignments/{}/submissions?ignored=true",
            data.module.id, data.assignment.id
        );
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

        let subs = json["data"]["submissions"].as_array().unwrap();
        // Only sub2 is ignored in the fixture
        assert_eq!(subs.len(), 1);
        assert!(subs.iter().all(|s| s["ignored"] == true));
    }

    #[tokio::test]
    #[serial]
    async fn test_filter_by_ignored_false_lecturer() {
        let (app, app_state) = make_test_app().await;
        let (data, _temp_dir) = setup_test_data(app_state.db()).await;

        let (token, _) = generate_jwt(data.lecturer_user.id, data.lecturer_user.admin);
        let uri = format!(
            "/api/modules/{}/assignments/{}/submissions?ignored=false",
            data.module.id, data.assignment.id
        );
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

        let subs = json["data"]["submissions"].as_array().unwrap();
        // 4 total, 1 ignored (sub2) → 3 non-ignored
        assert_eq!(subs.len(), 3);
        assert!(subs.iter().all(|s| s["ignored"] == false));
    }

    #[tokio::test]
    #[serial]
    async fn test_filter_by_ignored_true_student_scope() {
        let (app, app_state) = make_test_app().await;
        let (data, _temp_dir) = setup_test_data(app_state.db()).await;

        let (token, _) = generate_jwt(data.student_user.id, data.student_user.admin);
        let uri = format!(
            "/api/modules/{}/assignments/{}/submissions?ignored=true",
            data.module.id, data.assignment.id
        );
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

        let subs = json["data"]["submissions"].as_array().unwrap();
        // Student has 3 submissions; only sub2 is ignored → expect 1
        assert_eq!(subs.len(), 1);
        assert!(subs.iter().all(|s| s["ignored"] == true));
    }


    // --- GET /api/modules/{module_id}/assignments/{assignment_id}/submissions/{submission_id} ---

    #[tokio::test]
    #[serial]
    async fn test_student_gets_own_submission() {
        let app = make_test_app().await;
        let (data, _temp_dir) = setup_test_data(db::get_connection().await).await;

        let (token, _) = generate_jwt(data.student_user.id, data.student_user.admin);
        let sub = &data.submissions[0];
        let uri = format!("/api/modules/{}/assignments/{}/submissions/{}", data.module.id, data.assignment.id, sub.id);
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
        assert_eq!(json["data"]["id"], sub.id);

        // If your single-item endpoint injects `ignored`, assert it:
        // assert!(json["data"]["ignored"].is_boolean());
    }

    #[tokio::test]
    #[serial]
    async fn test_lecturer_gets_any_submission_with_user_info() {
        let app = make_test_app().await;
        let (data, _temp_dir) = setup_test_data(db::get_connection().await).await;

        let (token, _) = generate_jwt(data.lecturer_user.id, data.lecturer_user.admin);
        let sub = &data.submissions[0];
        let uri = format!("/api/modules/{}/assignments/{}/submissions/{}", data.module.id, data.assignment.id, sub.id);
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
        assert_eq!(json["data"]["id"], sub.id);
        assert!(json["data"]["user"].is_object());

        // If your single-item endpoint injects `ignored`, assert it:
        // assert!(json["data"]["ignored"].is_boolean());
    }

    #[tokio::test]
    #[serial]
    async fn test_forbidden_user_gets_403_on_submission() {
        let app = make_test_app().await;
        let (data, _temp_dir) = setup_test_data(db::get_connection().await).await;

        let (token, _) = generate_jwt(data.forbidden_user.id, data.forbidden_user.admin);
        let sub = &data.submissions[0];
        let uri = format!("/api/modules/{}/assignments/{}/submissions/{}", data.module.id, data.assignment.id, sub.id);
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
    async fn test_submission_not_found_returns_404() {
        let app = make_test_app().await;
        let (data, _temp_dir) = setup_test_data(db::get_connection().await).await;

        let (token, _) = generate_jwt(data.lecturer_user.id, data.lecturer_user.admin);
        let uri = format!("/api/modules/{}/assignments/{}/submissions/999999", data.module.id, data.assignment.id);
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
    async fn test_submission_report_missing_returns_404() {
        let app = make_test_app().await;
        let (data, temp_dir) = setup_test_data(db::get_connection().await).await;

        let sub = &data.submissions[0];
        let path = PathBuf::from(temp_dir.path())
            .join(format!("module_{}", data.module.id))
            .join(format!("assignment_{}", data.assignment.id))
            .join("assignment_submissions")
            .join(format!("user_{}", data.student_user.id))
            .join(format!("attempt_{}", sub.attempt))
            .join("submission_report.json");
        let _ = fs::remove_file(&path);

        let (token, _) = generate_jwt(data.lecturer_user.id, data.lecturer_user.admin);
        let uri = format!("/api/modules/{}/assignments/{}/submissions/{}", data.module.id, data.assignment.id, sub.id);
        let req = Request::builder()
            .uri(&uri)
            .header("Authorization", format!("Bearer {}", token))
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }
}
