#[cfg(test)]
mod tests {
    use axum::{
        body::Body,
        http::{Request, StatusCode},
    };
    use chrono::{Duration, TimeZone, Utc};
    use db::models::{
        assignment::{AssignmentType, Model as AssignmentModel},
        assignment_submission,
        module::Model as ModuleModel,
        user::Model as UserModel,
        user_module_role::{Model as UserModuleRoleModel, Role},
    };
    use serde_json::{json, Value};
    use tower::ServiceExt;

    use api::auth::generate_jwt;
    use crate::helpers::app::make_test_app;

    struct TestData {
        admin_user: UserModel,
        lecturer_user: UserModel,
        student_user: UserModel,
        forbidden_user: UserModel,
        module: ModuleModel,
        empty_module: ModuleModel,
        assignments: Vec<AssignmentModel>,
    }

    async fn setup_test_data(db: &sea_orm::DatabaseConnection) -> TestData {
        let module =
            ModuleModel::create(db, "COS101", 2024, Some("Test Module"), 16).await.unwrap();
        let empty_module =
            ModuleModel::create(db, "EMPTY101", 2024, Some("Empty Module"), 16).await.unwrap();

        let admin_user =
            UserModel::create(db, "admin1", "admin1@test.com", "password", true).await.unwrap();
        let lecturer_user = UserModel::create(
            db,
            "lecturer1",
            "lecturer1@test.com",
            "password1",
            false,
        )
        .await
        .unwrap();
        let student_user =
            UserModel::create(db, "student1", "student1@test.com", "password2", false)
                .await
                .unwrap();
        let forbidden_user = UserModel::create(
            db,
            "forbidden",
            "forbidden@test.com",
            "password3",
            false,
        )
        .await
        .unwrap();

        UserModuleRoleModel::assign_user_to_module(db, lecturer_user.id, module.id, Role::Lecturer)
            .await
            .unwrap();
        UserModuleRoleModel::assign_user_to_module(db, student_user.id, module.id, Role::Student)
            .await
            .unwrap();

        let a1 = AssignmentModel::create(
            db,
            module.id,
            "Assignment 1",
            Some("Desc 1"),
            AssignmentType::Assignment,
            Utc.with_ymd_and_hms(2024, 1, 1, 0, 0, 0).unwrap(),
            Utc.with_ymd_and_hms(2024, 1, 31, 23, 59, 59).unwrap(),
        )
        .await
        .unwrap();
        let a2 = AssignmentModel::create(
            db,
            module.id,
            "Assignment 2",
            Some("Desc 2"),
            AssignmentType::Practical,
            Utc.with_ymd_and_hms(2024, 2, 1, 0, 0, 0).unwrap(),
            Utc.with_ymd_and_hms(2024, 2, 28, 23, 59, 59).unwrap(),
        )
        .await
        .unwrap();
        let a3 = AssignmentModel::create(
            db,
            module.id,
            "Assignment 3",
            Some("Desc 3"),
            AssignmentType::Assignment,
            Utc.with_ymd_and_hms(2024, 3, 1, 0, 0, 0).unwrap(),
            Utc.with_ymd_and_hms(2024, 3, 31, 23, 59, 59).unwrap(),
        )
        .await
        .unwrap();

        TestData {
            admin_user,
            lecturer_user,
            student_user,
            forbidden_user,
            module,
            empty_module,
            assignments: vec![a1, a2, a3],
        }
    }

    // --- GET /api/modules/{module_id}/assignments (List) ---

    #[tokio::test]
    async fn test_get_assignments_success_as_admin() {
        let (app, app_state) = make_test_app().await;
        let data = setup_test_data(app_state.db()).await;

        let (token, _) = generate_jwt(data.admin_user.id, data.admin_user.admin);
        let uri = format!("/api/modules/{}/assignments", data.module.id);
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
        let names: Vec<_> = json["data"]["assignments"]
            .as_array()
            .unwrap()
            .iter()
            .map(|a| a["name"].as_str().unwrap())
            .collect();
        assert!(names.contains(&"Assignment 1"));
        assert!(names.contains(&"Assignment 2"));
        assert!(names.contains(&"Assignment 3"));
    }

    #[tokio::test]
    async fn test_get_assignments_success_as_lecturer() {
        let (app, app_state) = make_test_app().await;
        let data = setup_test_data(app_state.db()).await;

        let (token, _) = generate_jwt(data.lecturer_user.id, data.lecturer_user.admin);
        let uri = format!("/api/modules/{}/assignments", data.module.id);
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
    }

    #[tokio::test]
    async fn test_get_assignments_forbidden_for_unassigned_user() {
        let (app, app_state) = make_test_app().await;
        let data = setup_test_data(app_state.db()).await;

        let (token, _) = generate_jwt(data.forbidden_user.id, data.forbidden_user.admin);
        let uri = format!("/api/modules/{}/assignments", data.module.id);
        let req = Request::builder()
            .uri(&uri)
            .header("Authorization", format!("Bearer {}", token))
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::FORBIDDEN);
    }

    #[tokio::test]
    async fn test_get_assignments_module_not_found() {
        let (app, _) = make_test_app().await;
        let (token, _) = generate_jwt(1, false);

        let uri = format!("/api/modules/{}/assignments", 9999);
        let req = Request::builder()
            .uri(&uri)
            .header("Authorization", format!("Bearer {}", token))
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn test_get_assignments_filtering_and_sorting() {
        let (app, app_state) = make_test_app().await;
        let data = setup_test_data(app_state.db()).await;

        let (token, _) = generate_jwt(data.admin_user.id, data.admin_user.admin);
        let uri = format!(
            "/api/modules/{}/assignments?query=Assignment&sort=-name",
            data.module.id
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
        let names: Vec<_> = json["data"]["assignments"]
            .as_array()
            .unwrap()
            .iter()
            .map(|a| a["name"].as_str().unwrap())
            .collect();
        assert_eq!(names, vec!["Assignment 3", "Assignment 2", "Assignment 1"]);
    }

    #[tokio::test]
    async fn test_get_assignments_pagination() {
        let (app, app_state) = make_test_app().await;
        let data = setup_test_data(app_state.db()).await;

        let (token, _) = generate_jwt(data.admin_user.id, data.admin_user.admin);
        let uri = format!("/api/modules/{}/assignments?page=2&per_page=2", data.module.id);
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
        assert_eq!(json["data"]["page"], 2);
        assert_eq!(json["data"]["per_page"], 2);
        let assignments = json["data"]["assignments"].as_array().unwrap();
        assert_eq!(assignments.len(), 1);
    }

    #[tokio::test]
    async fn test_get_assignments_invalid_sort_field() {
        let (app, app_state) = make_test_app().await;
        let data = setup_test_data(app_state.db()).await;

        let (token, _) = generate_jwt(data.admin_user.id, data.admin_user.admin);
        let uri = format!(
            "/api/modules/{}/assignments?sort=invalid_field",
            data.module.id
        );
        let req = Request::builder()
            .uri(&uri)
            .header("Authorization", format!("Bearer {}", token))
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn test_get_assignments_invalid_assignment_type() {
        let (app, app_state) = make_test_app().await;
        let data = setup_test_data(app_state.db()).await;

        let (token, _) = generate_jwt(data.admin_user.id, data.admin_user.admin);
        let uri = format!(
            "/api/modules/{}/assignments?assignment_type=invalid",
            data.module.id
        );
        let req = Request::builder()
            .uri(&uri)
            .header("Authorization", format!("Bearer {}", token))
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn test_get_assignments_no_assignments_in_module() {
        let (app, app_state) = make_test_app().await;
        let data = setup_test_data(app_state.db()).await;

        let (token, _) = generate_jwt(data.admin_user.id, data.admin_user.admin);
        let uri = format!("/api/modules/{}/assignments", data.empty_module.id);
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
        assert_eq!(json["data"]["total"], 0);
    }

    #[tokio::test]
    async fn test_get_assignments_unauthorized() {
        let (app, app_state) = make_test_app().await;
        let data = setup_test_data(app_state.db()).await;

        let uri = format!("/api/modules/{}/assignments", data.module.id);
        let req = Request::builder().uri(&uri).body(Body::empty()).unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    // --- GET /api/modules/{module_id}/assignments/{assignment_id} (Detail) ---

    #[tokio::test]
    async fn test_get_assignment_detail_success_as_admin_best_mark_absent() {
        // Admin can view, but `best_mark` must be omitted for non-students
        let (app, app_state) = make_test_app().await;
        let data = setup_test_data(app_state.db()).await;

        let (token, _) = generate_jwt(data.admin_user.id, data.admin_user.admin);
        let uri = format!(
            "/api/modules/{}/assignments/{}",
            data.module.id, data.assignments[0].id
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
        assert_eq!(json["data"]["assignment"]["id"], data.assignments[0].id);
        assert!(json["data"].get("best_mark").is_none());
    }

    #[tokio::test]
    async fn test_get_assignment_detail_success_as_lecturer_best_mark_absent() {
        // Lecturer can view, but `best_mark` must be omitted for non-students
        let (app, app_state) = make_test_app().await;
        let data = setup_test_data(app_state.db()).await;

        let (token, _) = generate_jwt(data.lecturer_user.id, data.lecturer_user.admin);
        let uri = format!(
            "/api/modules/{}/assignments/{}",
            data.module.id, data.assignments[0].id
        );
        let req = Request::builder()
            .uri(&uri)
            .header("Authorization", format!("Bearer {}", token))
            .body(Body::empty())
            .unwrap();

        let response: axum::http::Response<Body> = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let json: Value = serde_json::from_slice(&body).unwrap();
        assert!(json["data"].get("best_mark").is_none());
    }

    #[tokio::test]
    async fn test_get_assignment_detail_success_as_student_no_submissions_best_mark_absent() {
        let (app, app_state) = make_test_app().await;
        let data = setup_test_data(app_state.db()).await;

        // Student, but no submissions yet → best_mark omitted
        let (token, _) = generate_jwt(data.student_user.id, data.student_user.admin);
        let uri = format!(
            "/api/modules/{}/assignments/{}",
            data.module.id, data.assignments[0].id
        );
        let req = Request::builder()
            .uri(&uri)
            .header("Authorization", format!("Bearer {}", token))
            .body(Body::empty())
            .unwrap();

        let response: axum::http::Response<Body> = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let json: Value = serde_json::from_slice(&body).unwrap();
        assert!(json["data"].get("best_mark").is_none());
    }

    #[tokio::test]
    async fn test_get_assignment_detail_best_mark_for_student_grading_policy_best() {
        // Create multiple submissions; policy=best → highest (earned/total) should be returned
        use sea_orm::{ActiveModelTrait, Set};

        let (app, app_state) = make_test_app().await;
        let data = setup_test_data(app_state.db()).await;
        let db = app_state.db();

        // Set grading policy to "best" on assignment 1
        {
            use sea_orm::IntoActiveModel;
            let mut am = data.assignments[0].clone().into_active_model();
            am.config = Set(Some(json!({
                "marking": { "grading_policy": "best" }
            })));
            am.update(db).await.unwrap();
        }

        // Insert 3 non-practice, non-ignored submissions for the student with different marks
        // attempts 1..=3, earned: 10, 15, 12 out of 20
        let base = Utc::now();
        let _ = assignment_submission::ActiveModel {
            assignment_id: Set(data.assignments[0].id),
            user_id: Set(data.student_user.id),
            attempt: Set(1),
            earned: Set(10),
            total: Set(20),
            filename: Set("a1.zip".into()),
            file_hash: Set("h1".into()),
            path: Set("p1".into()),
            is_practice: Set(false),
            ignored: Set(false),
            created_at: Set(base),
            updated_at: Set(base),
            ..Default::default()
        }
        .insert(db)
        .await
        .unwrap();

        let s2 = assignment_submission::ActiveModel {
            assignment_id: Set(data.assignments[0].id),
            user_id: Set(data.student_user.id),
            attempt: Set(2),
            earned: Set(15), // best
            total: Set(20),
            filename: Set("a2.zip".into()),
            file_hash: Set("h2".into()),
            path: Set("p2".into()),
            is_practice: Set(false),
            ignored: Set(false),
            created_at: Set(base + Duration::minutes(10)),
            updated_at: Set(base + Duration::minutes(10)),
            ..Default::default()
        }
        .insert(db)
        .await
        .unwrap();

        let _s3 = assignment_submission::ActiveModel {
            assignment_id: Set(data.assignments[0].id),
            user_id: Set(data.student_user.id),
            attempt: Set(3),
            earned: Set(12),
            total: Set(20),
            filename: Set("a3.zip".into()),
            file_hash: Set("h3".into()),
            path: Set("p3".into()),
            is_practice: Set(false),
            ignored: Set(false),
            created_at: Set(base + Duration::minutes(20)),
            updated_at: Set(base + Duration::minutes(20)),
            ..Default::default()
        }
        .insert(db)
        .await
        .unwrap();

        // Call as the student
        let (token, _) = generate_jwt(data.student_user.id, data.student_user.admin);
        let uri = format!(
            "/api/modules/{}/assignments/{}",
            data.module.id, data.assignments[0].id
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

        let bm = json["data"]["best_mark"].as_object().expect("best_mark missing");
        assert_eq!(bm["earned"], 15);
        assert_eq!(bm["total"], 20);
        assert_eq!(bm["attempt"], 2);
        assert_eq!(bm["submission_id"], s2.id);
    }

    #[tokio::test]
    async fn test_get_assignment_detail_best_mark_for_student_grading_policy_last() {
        // Create multiple submissions; policy=last → most recent should be returned
        use sea_orm::{ActiveModelTrait, Set};

        let (app, app_state) = make_test_app().await;
        let data = setup_test_data(app_state.db()).await;
        let db = app_state.db();

        // Set grading policy to "last" on assignment 2
        {
            use sea_orm::IntoActiveModel;
            let mut am = data.assignments[1].clone().into_active_model();
            am.config = Set(Some(json!({
                "marking": { "grading_policy": "last" }
            })));
            am.update(db).await.unwrap();
        }

        let base = Utc::now();

        let _s1 = assignment_submission::ActiveModel {
            assignment_id: Set(data.assignments[1].id),
            user_id: Set(data.student_user.id),
            attempt: Set(1),
            earned: Set(10),
            total: Set(20),
            filename: Set("b1.zip".into()),
            file_hash: Set("bh1".into()),
            path: Set("bp1".into()),
            is_practice: Set(false),
            ignored: Set(false),
            created_at: Set(base),
            updated_at: Set(base),
            ..Default::default()
        }
        .insert(db)
        .await
        .unwrap();

        let s2 = assignment_submission::ActiveModel {
            assignment_id: Set(data.assignments[1].id),
            user_id: Set(data.student_user.id),
            attempt: Set(2),
            earned: Set(11),
            total: Set(20),
            filename: Set("b2.zip".into()),
            file_hash: Set("bh2".into()),
            path: Set("bp2".into()),
            is_practice: Set(false),
            ignored: Set(false),
            created_at: Set(base + Duration::minutes(30)), // latest
            updated_at: Set(base + Duration::minutes(30)),
            ..Default::default()
        }
        .insert(db)
        .await
        .unwrap();

        // Call as the student
        let (token, _) = generate_jwt(data.student_user.id, data.student_user.admin);
        let uri = format!(
            "/api/modules/{}/assignments/{}",
            data.module.id, data.assignments[1].id
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

        let bm = json["data"]["best_mark"].as_object().expect("best_mark missing");
        assert_eq!(bm["earned"], 11);
        assert_eq!(bm["total"], 20);
        assert_eq!(bm["attempt"], 2);
        assert_eq!(bm["submission_id"], s2.id);
    }

    #[tokio::test]
    async fn test_get_assignment_detail_not_found() {
        let (app, app_state) = make_test_app().await;
        let data = setup_test_data(app_state.db()).await;

        let (token, _) = generate_jwt(data.admin_user.id, data.admin_user.admin);
        let uri = format!("/api/modules/{}/assignments/9999", data.module.id);
        let req = Request::builder()
            .uri(&uri)
            .header("Authorization", format!("Bearer {}", token))
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn test_get_assignment_detail_forbidden_for_unassigned_user() {
        let (app, app_state) = make_test_app().await;
        let data = setup_test_data(app_state.db()).await;

        let (token, _) = generate_jwt(data.forbidden_user.id, data.forbidden_user.admin);
        let uri = format!(
            "/api/modules/{}/assignments/{}",
            data.module.id, data.assignments[0].id
        );
        let req = Request::builder()
            .uri(&uri)
            .header("Authorization", format!("Bearer {}", token))
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::FORBIDDEN);
    }

    #[tokio::test]
    async fn test_get_assignment_detail_wrong_module() {
        let (app, app_state) = make_test_app().await;
        let data = setup_test_data(app_state.db()).await;

        let (token, _) = generate_jwt(data.admin_user.id, data.admin_user.admin);
        let uri = format!(
            "/api/modules/{}/assignments/{}",
            data.empty_module.id, data.assignments[0].id
        );
        let req = Request::builder()
            .uri(&uri)
            .header("Authorization", format!("Bearer {}", token))
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn test_get_assignment_detail_unauthorized() {
        let (app, app_state) = make_test_app().await;
        let data = setup_test_data(app_state.db()).await;

        let uri = format!(
            "/api/modules/{}/assignments/{}",
            data.module.id, data.assignments[0].id
        );
        let req = Request::builder().uri(&uri).body(Body::empty()).unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    // --- GET /api/modules/{module_id}/assignments/{assignment_id}/readiness ---

    #[tokio::test]
    async fn test_get_assignment_readiness_success_as_admin() {
        let (app, app_state) = make_test_app().await;
        let data = setup_test_data(app_state.db()).await;

        let (token, _) = generate_jwt(data.admin_user.id, data.admin_user.admin);
        let uri = format!(
            "/api/modules/{}/assignments/{}/readiness",
            data.module.id, data.assignments[0].id
        );
        let req = Request::builder()
            .uri(&uri)
            .header("Authorization", format!("Bearer {}", token))
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert!(response.status() == StatusCode::OK);
    }

    #[tokio::test]
    async fn test_get_assignment_readiness_success_as_lecturer() {
        let (app, app_state) = make_test_app().await;
        let data = setup_test_data(app_state.db()).await;

        let (token, _) = generate_jwt(data.lecturer_user.id, data.lecturer_user.admin);
        let uri = format!(
            "/api/modules/{}/assignments/{}/readiness",
            data.module.id, data.assignments[0].id
        );
        let req = Request::builder()
            .uri(&uri)
            .header("Authorization", format!("Bearer {}", token))
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert!(response.status() == StatusCode::OK);
    }

    #[tokio::test]
    async fn test_get_assignment_readiness_success_as_student() {
        let (app, app_state) = make_test_app().await;
        let data = setup_test_data(app_state.db()).await;

        let (token, _) = generate_jwt(data.student_user.id, data.student_user.admin);
        let uri = format!(
            "/api/modules/{}/assignments/{}/readiness",
            data.module.id, data.assignments[0].id
        );
        let req = Request::builder()
            .uri(&uri)
            .header("Authorization", format!("Bearer {}", token))
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_get_assignment_readiness_not_found() {
        let (app, _) = make_test_app().await;

        let (token, _) = generate_jwt(1, false);
        let uri = format!("/api/modules/{}/assignments/9999/readiness", 1234);
        let req = Request::builder()
            .uri(&uri)
            .header("Authorization", format!("Bearer {}", token))
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert!(response.status() == StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn test_get_assignment_readiness_module_not_found() {
        let (app, app_state) = make_test_app().await;
        let data = setup_test_data(app_state.db()).await;

        let (token, _) = generate_jwt(data.admin_user.id, data.admin_user.admin);
        let uri = format!(
            "/api/modules/{}/assignments/{}/readiness",
            9999, data.assignments[0].id
        );
        let req = Request::builder()
            .uri(&uri)
            .header("Authorization", format!("Bearer {}", token))
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    // --- GET /api/modules/{module_id}/assignments/{assignment_id}/stats ---

    #[tokio::test]
    async fn test_get_assignment_stats_success_as_lecturer() {
        let (app, app_state) = make_test_app().await;
        let data = setup_test_data(app_state.db()).await;

        let (token, _) = generate_jwt(data.lecturer_user.id, data.lecturer_user.admin);
        let uri = format!(
            "/api/modules/{}/assignments/{}/stats",
            data.module.id, data.assignments[0].id
        );
        let req = Request::builder()
            .uri(&uri)
            .header("Authorization", format!("Bearer {}", token))
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert!(response.status() == StatusCode::OK);
    }

    #[tokio::test]
    async fn test_get_assignment_stats_success_as_admin() {
        let (app, app_state) = make_test_app().await;
        let data = setup_test_data(app_state.db()).await;

        let (token, _) = generate_jwt(data.admin_user.id, data.admin_user.admin);
        let uri = format!(
            "/api/modules/{}/assignments/{}/stats",
            data.module.id, data.assignments[0].id
        );
        let req = Request::builder()
            .uri(&uri)
            .header("Authorization", format!("Bearer {}", token))
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_get_assignment_stats_forbidden_for_student() {
        let (app, app_state) = make_test_app().await;
        let data = setup_test_data(app_state.db()).await;

        let (token, _) = generate_jwt(data.student_user.id, data.student_user.admin);
        let uri = format!(
            "/api/modules/{}/assignments/{}/stats",
            data.module.id, data.assignments[0].id
        );
        let req = Request::builder()
            .uri(&uri)
            .header("Authorization", format!("Bearer {}", token))
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::FORBIDDEN);
    }

    #[tokio::test]
    async fn test_get_assignment_stats_not_found() {
        let (app, app_state) = make_test_app().await;
        let data = setup_test_data(app_state.db()).await;

        let (token, _) = generate_jwt(data.lecturer_user.id, data.lecturer_user.admin);
        let uri = format!(
            "/api/modules/{}/assignments/9999/stats",
            data.module.id
        );
        let req = Request::builder()
            .uri(&uri)
            .header("Authorization", format!("Bearer {}", token))
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert!(response.status() == StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn test_get_assignment_stats_module_not_found() {
        let (app, app_state) = make_test_app().await;
        let data = setup_test_data(app_state.db()).await;

        let (token, _) = generate_jwt(data.admin_user.id, data.admin_user.admin);
        let uri = format!(
            "/api/modules/{}/assignments/{}/stats",
            9999, data.assignments[0].id
        );
        let req = Request::builder()
            .uri(&uri)
            .header("Authorization", format!("Bearer {}", token))
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }
}
