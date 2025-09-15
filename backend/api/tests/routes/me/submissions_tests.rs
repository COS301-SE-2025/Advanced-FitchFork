#[cfg(test)]
mod tests {
    use api::auth::generate_jwt;
    use axum::{
        body::Body as AxumBody,
        http::{Request, StatusCode},
    };
    use chrono::{Duration, Utc};
    use db::models::{
        assignment::Model as AssignmentModel,
        assignment_submission::Model as SubmissionModel,
        module::Model as ModuleModel,
        user::Model as UserModel,
        user_module_role::{Model as UserModuleRoleModel, Role},
    };
    use sea_orm::ActiveModelTrait;
    use serial_test::serial;
    use serde_json::Value;
    use tower::ServiceExt;

    use crate::helpers::app::make_test_app_with_storage;

    struct TestData {
        student1: UserModel,
        lecturer: UserModel,
        submission_late: SubmissionModel,
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
        UserModuleRoleModel::assign_user_to_module(db, student1.id, module2.id, Role::Student)
            .await
            .unwrap();
        UserModuleRoleModel::assign_user_to_module(db, lecturer.id, module2.id, Role::Tutor)
            .await
            .unwrap();

        let now = Utc::now();
        let past_time = now + Duration::hours(1);
        let due_date_a1 = now + Duration::days(1);

        let assignment1 = AssignmentModel::create(
            db,
            module1.id,
            "A01",
            None,
            db::models::assignment::AssignmentType::Practical,
            now,
            due_date_a1,
        )
        .await
        .unwrap();

        let _assignment2 = AssignmentModel::create(
            db,
            module2.id,
            "A02",
            None,
            db::models::assignment::AssignmentType::Practical,
            now,
            due_date_a1,
        )
        .await
        .unwrap();

        let submission1 = SubmissionModel::save_file(
            db,
            assignment1.id,
            student1.id,
            1,
            80,
            100,
            false,
            "s1.zip",
            "hash1",
            "output1".as_bytes(),
        )
        .await
        .unwrap();
        let mut active_submission1: db::models::assignment_submission::ActiveModel = submission1.into();
        active_submission1.created_at = sea_orm::ActiveValue::Set(past_time);
        let _submission1 = active_submission1.update(db).await.unwrap();

        let _submission2 = SubmissionModel::save_file(
            db,
            assignment1.id,
            student2.id,
            1,
            90,
            100,
            false,
            "s2.zip",
            "hash2",
            "output2".as_bytes(),
        )
        .await
        .unwrap();

        let submission3 = SubmissionModel::save_file(
            db,
            assignment1.id,
            student1.id,
            1,
            85,
            100,
            false,
            "s1_v2.zip",
            "hash4",
            "output4".as_bytes(),
        )
        .await
        .unwrap();
        let mut active_submission3: db::models::assignment_submission::ActiveModel = submission3.into();
        active_submission3.created_at = sea_orm::ActiveValue::Set(past_time + Duration::minutes(1));
        let _submission3 = active_submission3.update(db).await.unwrap();

        let submission4 = SubmissionModel::save_file(
            db,
            assignment1.id,
            student1.id,
            1,
            75,
            100,
            false,
            "s1_v3.zip",
            "hash5",
            "output5".as_bytes(),
        )
        .await
        .unwrap();
        let mut active_submission4: db::models::assignment_submission::ActiveModel = submission4.into();
        active_submission4.created_at = sea_orm::ActiveValue::Set(past_time + Duration::minutes(2));
        let _submission4 = active_submission4.update(db).await.unwrap();

        let submission5 = SubmissionModel::save_file(
            db,
            assignment1.id,
            student1.id,
            1,
            95,
            100,
            false,
            "s1_v4.zip",
            "hash6",
            "output6".as_bytes(),
        )
        .await
        .unwrap();
        let mut active_submission5: db::models::assignment_submission::ActiveModel = submission5.into();
        active_submission5.created_at = sea_orm::ActiveValue::Set(past_time + Duration::minutes(3));
        let _submission5 = active_submission5.update(db).await.unwrap();

        let submission6 = SubmissionModel::save_file(
            db,
            assignment1.id,
            student1.id,
            1,
            60,
            100,
            false,
            "s1_v5.zip",
            "hash7",
            "output7".as_bytes(),
        )
        .await
        .unwrap();
        let mut active_submission6: db::models::assignment_submission::ActiveModel = submission6.into();
        active_submission6.created_at = sea_orm::ActiveValue::Set(past_time + Duration::minutes(4));
        let _submission6 = active_submission6.update(db).await.unwrap();

        let late_submission_time = now + Duration::days(2);
        let submission_late = SubmissionModel::save_file(
            db,
            assignment1.id,
            student1.id,
            2,
            70,
            100,
            false,
            "s1_late.zip",
            "hash3",
            "output3".as_bytes(),
        )
        .await
        .unwrap();
        let mut active_submission: db::models::assignment_submission::ActiveModel = submission_late.into();
        active_submission.created_at = sea_orm::ActiveValue::Set(late_submission_time);
        let submission_late = active_submission.update(db).await.unwrap();

        TestData {
            student1,
            lecturer,
            submission_late,
        }
    }
    
    #[tokio::test]
    #[serial]
    async fn test_get_submissions_per_page() {
        let (app, app_state, _tmp) = make_test_app_with_storage().await;
        let data = setup_test_data(app_state.db()).await;

        let (token, _) = generate_jwt(data.student1.id, false);
        let req = Request::builder()
            .method("GET")
            .uri("/api/me/submissions?per_page=1")
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
        let submissions = json["data"]["submissions"].as_array().unwrap();
        assert_eq!(submissions.len(), 1);
        assert_eq!(json["data"]["per_page"], 1);
        assert_eq!(json["data"]["total"], 6); // student1 has 6 submissions
    }

    #[tokio::test]
    #[serial]
    async fn test_get_submissions_pagination() {
        let (app, app_state, _tmp) = make_test_app_with_storage().await;
        let data = setup_test_data(app_state.db()).await;

        let (token, _) = generate_jwt(data.student1.id, false);
        let req = Request::builder()
            .method("GET")
            .uri("/api/me/submissions?page=2&per_page=2")
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
        let submissions = json["data"]["submissions"].as_array().unwrap();
        assert_eq!(submissions.len(), 2);
        assert_eq!(json["data"]["page"], 2);
        assert_eq!(json["data"]["per_page"], 2);
        assert_eq!(json["data"]["total"], 6);
    }

    #[tokio::test]
    #[serial]
    async fn test_get_submissions_filter_by_module_code() {
        let (app, app_state, _tmp) = make_test_app_with_storage().await;
        let data = setup_test_data(app_state.db()).await;

        let (token, _) = generate_jwt(data.lecturer.id, false);
        let req = Request::builder()
            .method("GET")
            .uri("/api/me/submissions?role=lecturer&query=COS101")
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
        let submissions = json["data"]["submissions"].as_array().unwrap();
        assert_eq!(submissions.len(), 7); // All submissions are in COS101 (student1: 6, student2: 1)
        assert!(submissions
            .iter()
            .all(|s| s["module"]["code"] == "COS101"));
    }

    #[tokio::test]
    #[serial]
    async fn test_get_submissions_filter_by_username() {
        let (app, app_state, _tmp) = make_test_app_with_storage().await;
        let data = setup_test_data(app_state.db()).await;

        let (token, _) = generate_jwt(data.lecturer.id, false);
        let req = Request::builder()
            .method("GET")
            .uri("/api/me/submissions?role=lecturer&query=student2")
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
        let submissions = json["data"]["submissions"].as_array().unwrap();
        assert_eq!(submissions.len(), 1);
        assert!(submissions
            .iter()
            .all(|s| s["user"]["username"] == "student2"));
    }

    #[tokio::test]
    #[serial]
    async fn test_get_submissions_filter_by_assignment_name() {
        let (app, app_state, _tmp) = make_test_app_with_storage().await;
        let data = setup_test_data(app_state.db()).await;

        let (token, _) = generate_jwt(data.lecturer.id, false);
        let req = Request::builder()
            .method("GET")
            .uri("/api/me/submissions?role=lecturer&query=A01")
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
        let submissions = json["data"]["submissions"].as_array().unwrap();
        assert_eq!(submissions.len(), 7); // All submissions are for A01 (student1: 6, student2: 1)
        assert!(submissions
            .iter()
            .all(|s| s["assignment"]["name"] == "A01"));
    }

    #[tokio::test]
    #[serial]
    async fn test_get_submissions_filter_by_year() {
        let (app, app_state, _tmp) = make_test_app_with_storage().await;
        let data = setup_test_data(app_state.db()).await;

        let (token, _) = generate_jwt(data.lecturer.id, false);
        let req = Request::builder()
            .method("GET")
            .uri("/api/me/submissions?role=lecturer&year=2024")
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
        let submissions = json["data"]["submissions"].as_array().unwrap();
        assert_eq!(submissions.len(), 7); // All submissions are in 2024 module (student1: 6, student2: 1)
        assert!(submissions
            .iter()
            .all(|s| s["module"]["code"] == "COS101")); // COS101 is 2024
    }

    #[tokio::test]
    #[serial]
    async fn test_get_submissions_filter_is_late_true() {
        let (app, app_state, _tmp) = make_test_app_with_storage().await;
        let data = setup_test_data(app_state.db()).await;

        let (token, _) = generate_jwt(data.student1.id, false);
        let req = Request::builder()
            .method("GET")
            .uri("/api/me/submissions?is_late=true")
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
        let submissions = json["data"]["submissions"].as_array().unwrap();
        assert_eq!(submissions.len(), 1);
        assert_eq!(submissions[0]["id"], data.submission_late.id);
        assert_eq!(submissions[0]["is_late"], true);
    }

    #[tokio::test]
    #[serial]
    async fn test_get_submissions_filter_is_late_false() {
        let (app, app_state, _tmp) = make_test_app_with_storage().await;
        let data = setup_test_data(app_state.db()).await;

        let (token, _) = generate_jwt(data.student1.id, false);
        let req = Request::builder()
            .method("GET")
            .uri("/api/me/submissions?is_late=false")
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
        let submissions = json["data"]["submissions"].as_array().unwrap();
        assert_eq!(submissions.len(), 5);
        assert!(submissions.iter().all(|s| s["is_late"] == false));
    }

    #[tokio::test]
    #[serial]
    async fn test_get_submissions_sort_by_created_at_asc() {
        let (app, app_state, _tmp) = make_test_app_with_storage().await;
        let data = setup_test_data(app_state.db()).await;

        let (token, _) = generate_jwt(data.student1.id, false);
        let req = Request::builder()
            .method("GET")
            .uri("/api/me/submissions?sort=created_at")
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
        let submissions = json["data"]["submissions"].as_array().unwrap();
        assert_eq!(submissions.len(), 6);

        // Verify sorting by created_at ascending
        let mut sorted_submissions = submissions.clone();
        sorted_submissions.sort_by(|a, b| {
            let created_at_a = a["created_at"].as_str().unwrap();
            let created_at_b = b["created_at"].as_str().unwrap();
            created_at_a.cmp(created_at_b)
        });
        assert_eq!(submissions, &sorted_submissions);
    }

    #[tokio::test]
    #[serial]
    async fn test_get_submissions_sort_by_score_desc() {
        let (app, app_state, _tmp) = make_test_app_with_storage().await;
        let data = setup_test_data(app_state.db()).await;

        let (token, _) = generate_jwt(data.student1.id, false);
        let req = Request::builder()
            .method("GET")
            .uri("/api/me/submissions?sort=-score")
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
        let submissions = json["data"]["submissions"].as_array().unwrap();
        assert_eq!(submissions.len(), 6);

        // Verify sorting by score descending
        let mut sorted_submissions = submissions.clone();
        sorted_submissions.sort_by(|a, b| {
            let score_a = a["score"]["earned"].as_i64().unwrap();
            let score_b = b["score"]["earned"].as_i64().unwrap();
            score_b.cmp(&score_a) // Descending
        });
        assert_eq!(submissions, &sorted_submissions);
    }

    #[tokio::test]
    #[serial]
    async fn test_get_submissions_invalid_page_parameter() {
        let (app, app_state, _tmp) = make_test_app_with_storage().await;
        let data = setup_test_data(app_state.db()).await;

        let (token, _) = generate_jwt(data.student1.id, false);
        let req = Request::builder()
            .method("GET")
            .uri("/api/me/submissions?page=0")
            .header("Authorization", format!("Bearer {}", token))
            .body(AxumBody::empty())
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let json: Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(json["success"], false);
    }

    #[tokio::test]
    #[serial]
    async fn test_get_submissions_invalid_per_page_parameter() {
        let (app, app_state, _tmp) = make_test_app_with_storage().await;
        let data = setup_test_data(app_state.db()).await;

        let (token, _) = generate_jwt(data.student1.id, false);
        let req = Request::builder()
            .method("GET")
            .uri("/api/me/submissions?per_page=0")
            .header("Authorization", format!("Bearer {}", token))
            .body(AxumBody::empty())
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let json: Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(json["success"], false);
    }

    #[tokio::test]
    #[serial]
    async fn test_get_submissions_no_submissions_found() {
        let (app, app_state, _tmp) = make_test_app_with_storage().await;
        // Create a user with no submissions
        let no_submission_student = UserModel::create(app_state.db(), "no_sub_student", "no_sub@test.com", "p4", false)
            .await
            .unwrap();

        let (token, _) = generate_jwt(no_submission_student.id, false);
        let req = Request::builder()
            .method("GET")
            .uri("/api/me/submissions")
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
        let submissions = json["data"]["submissions"].as_array().unwrap();
        assert!(submissions.is_empty());
        assert_eq!(json["data"]["total"], 0);
        assert_eq!(json["message"], "No submissions found");
    }

    #[tokio::test]
    #[serial]
    async fn test_get_submissions_as_student_success() {
        let (app, app_state, _tmp) = make_test_app_with_storage().await;
        let data = setup_test_data(app_state.db()).await;

        let (token, _) = generate_jwt(data.student1.id, false);
        let req = Request::builder()
            .method("GET")
            .uri("/api/me/submissions")
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
        let submissions = json["data"]["submissions"].as_array().unwrap();
        assert_eq!(submissions.len(), 6); // student1 has 6 submissions (submission1, submission3, submission4, submission5, submission6, submission_late)
        assert!(submissions
            .iter()
            .all(|s| s["user"]["id"] == data.student1.id));
    }

    #[tokio::test]
    #[serial]
    async fn test_get_submissions_as_lecturer_success() {
        let (app, app_state, _tmp) = make_test_app_with_storage().await;
        let data = setup_test_data(app_state.db()).await;

        let (token, _) = generate_jwt(data.lecturer.id, false);
        let req = Request::builder()
            .method("GET")
            .uri("/api/me/submissions?role=lecturer")
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
        let submissions = json["data"]["submissions"].as_array().unwrap();
        assert_eq!(submissions.len(), 7); // lecturer should see all 7 submissions in module1
    }

    #[tokio::test]
    #[serial]
    async fn test_get_submissions_unauthorized() {
        let (app, _app_state, _tmp) = make_test_app_with_storage().await;
        let req = Request::builder()
            .method("GET")
            .uri("/api/me/submissions")
            .body(AxumBody::empty())
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }
}