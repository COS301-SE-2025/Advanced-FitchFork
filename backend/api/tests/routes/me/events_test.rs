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
        module::Model as ModuleModel,
        user::Model as UserModel,
        user_module_role::{Model as UserModuleRoleModel, Role},
    };
    use serde_json::Value;
    use serial_test::serial;
    use tower::ServiceExt;

    use crate::helpers::app::make_test_app_with_storage;

    struct TestData {
        student1: UserModel,
        student2: UserModel,
    }

    async fn setup_events_test_data(db: &sea_orm::DatabaseConnection) -> TestData {
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
        UserModuleRoleModel::assign_user_to_module(db, student1.id, module2.id, Role::Student)
            .await
            .unwrap();
        UserModuleRoleModel::assign_user_to_module(db, student2.id, module1.id, Role::Student)
            .await
            .unwrap();
        UserModuleRoleModel::assign_user_to_module(db, lecturer.id, module1.id, Role::Lecturer)
            .await
            .unwrap();

        let now = Utc::now();

        let _assignment1 = AssignmentModel::create(
            db,
            module1.id,
            "Assignment 1",
            Some("First assignment"),
            db::models::assignment::AssignmentType::Practical,
            now,
            now + Duration::days(7),
        )
        .await
        .unwrap();

        let _assignment2 = AssignmentModel::create(
            db,
            module1.id,
            "Assignment 2",
            Some("Second assignment"),
            db::models::assignment::AssignmentType::Practical,
            now + Duration::days(3),
            now + Duration::days(10),
        )
        .await
        .unwrap();

        let _assignment3 = AssignmentModel::create(
            db,
            module2.id,
            "Assignment 3",
            Some("Past assignment"),
            db::models::assignment::AssignmentType::Practical,
            now - Duration::days(30),
            now - Duration::days(23),
        )
        .await
        .unwrap();

        TestData { student1, student2 }
    }

    #[tokio::test]
    #[serial]
    async fn test_get_events_success_no_filters() {
        let (app, app_state, _tmp) = make_test_app_with_storage().await;
        let data = setup_events_test_data(app_state.db()).await;

        let (token, _) = generate_jwt(data.student1.id, false);
        let req = Request::builder()
            .method("GET")
            .uri("/api/me/events")
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
        assert_eq!(json["message"], "Events retrieved successfully");

        let events = json["data"]["events"].as_object().unwrap();
        // Should have 6 events total: 3 assignments × 2 events each (available + due)
        assert_eq!(events.len(), 6);
    }

    #[tokio::test]
    #[serial]
    async fn test_get_events_with_date_range_filter() {
        let (app, app_state, _tmp) = make_test_app_with_storage().await;
        let data = setup_events_test_data(app_state.db()).await;

        let now = Utc::now();
        let from_date = (now + Duration::days(2)).format("%Y-%m-%d").to_string();
        let to_date = (now + Duration::days(8)).format("%Y-%m-%d").to_string();

        let (token, _) = generate_jwt(data.student1.id, false);
        let req = Request::builder()
            .method("GET")
            .uri(&format!("/api/me/events?from={}&to={}", from_date, to_date))
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
        let events = json["data"]["events"].as_object().unwrap();

        // Should include:
        // - Assignment 1 due date (day 7)
        // - Assignment 2 available date (day 3)
        // Should exclude events outside the date range
        assert!(events.len() >= 2);

        // Verify all events are within the date range
        for (date_key, _) in events {
            let event_date = chrono::DateTime::parse_from_rfc3339(&format!("{}Z", date_key))
                .unwrap()
                .with_timezone(&Utc);
            assert!(event_date >= now + Duration::days(2));
            assert!(event_date <= now + Duration::days(8) + Duration::days(1)); // Add buffer for end of day
        }
    }

    #[tokio::test]
    #[serial]
    async fn test_get_events_with_from_date_only() {
        let (app, app_state, _tmp) = make_test_app_with_storage().await;
        let data = setup_events_test_data(app_state.db()).await;

        let now = Utc::now();
        let from_date = (now + Duration::days(5)).format("%Y-%m-%d").to_string();

        let (token, _) = generate_jwt(data.student1.id, false);
        let req = Request::builder()
            .method("GET")
            .uri(&format!("/api/me/events?from={}", from_date))
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
        let events = json["data"]["events"].as_object().unwrap();

        // Should only include future events (from day 5 onwards)
        // Assignment 1 due (day 7) and Assignment 2 due (day 10)
        assert!(events.len() >= 2);
    }

    #[tokio::test]
    #[serial]
    async fn test_get_events_with_to_date_only() {
        let (app, app_state, _tmp) = make_test_app_with_storage().await;
        let data = setup_events_test_data(app_state.db()).await;

        let now = Utc::now();
        let to_date = (now + Duration::days(5)).format("%Y-%m-%d").to_string();

        let (token, _) = generate_jwt(data.student1.id, false);
        let req = Request::builder()
            .method("GET")
            .uri(&format!("/api/me/events?to={}", to_date))
            .header("Authorization", format!("Bearer {}", token))
            .body(AxumBody::empty())
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let json: Value = serde_json::from_slice(&body).unwrap();

        assert_eq!(json["success"], true);
        let events = json["data"]["events"].as_object().unwrap();

        // Should include past events and near-future events up to day 5
        // Assignment 3 events (past), Assignment 1 available (now), Assignment 2 available (day 3)
        assert!(events.len() >= 3);
    }

    #[tokio::test]
    #[serial]
    async fn test_get_events_datetime_format_support() {
        let (app, app_state, _tmp) = make_test_app_with_storage().await;
        let data = setup_events_test_data(app_state.db()).await;

        let now = Utc::now();
        let from_datetime = (now + Duration::days(2))
            .format("%Y-%m-%dT%H:%M:%S")
            .to_string();
        let to_datetime = (now + Duration::days(8))
            .format("%Y-%m-%dT%H:%M:%S")
            .to_string();

        let (token, _) = generate_jwt(data.student1.id, false);
        let req = Request::builder()
            .method("GET")
            .uri(&format!(
                "/api/me/events?from={}&to={}",
                from_datetime, to_datetime
            ))
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
        let events = json["data"]["events"].as_object().unwrap();
        assert!(events.len() >= 1);
    }

    #[tokio::test]
    #[serial]
    async fn test_get_events_user_scoping() {
        let (app, app_state, _tmp) = make_test_app_with_storage().await;
        let data = setup_events_test_data(app_state.db()).await;

        // Test that student1 sees their events
        let (token1, _) = generate_jwt(data.student1.id, false);
        let req1 = Request::builder()
            .method("GET")
            .uri("/api/me/events")
            .header("Authorization", format!("Bearer {}", token1))
            .body(AxumBody::empty())
            .unwrap();

        let response1 = app.clone().oneshot(req1).await.unwrap();
        let body1 = axum::body::to_bytes(response1.into_body(), usize::MAX)
            .await
            .unwrap();
        let json1: Value = serde_json::from_slice(&body1).unwrap();
        let events1 = json1["data"]["events"].as_object().unwrap();

        // Test that student2 sees different events (only from module1, not module2)
        let (token2, _) = generate_jwt(data.student2.id, false);
        let req2 = Request::builder()
            .method("GET")
            .uri("/api/me/events")
            .header("Authorization", format!("Bearer {}", token2))
            .body(AxumBody::empty())
            .unwrap();

        let response2 = app.oneshot(req2).await.unwrap();
        let body2 = axum::body::to_bytes(response2.into_body(), usize::MAX)
            .await
            .unwrap();
        let json2: Value = serde_json::from_slice(&body2).unwrap();
        let events2 = json2["data"]["events"].as_object().unwrap();

        // student1 should have more events (enrolled in both modules)
        // student2 should have fewer events (only enrolled in module1)
        assert!(events1.len() > events2.len());
    }

    #[tokio::test]
    #[serial]
    async fn test_get_events_event_types_and_content() {
        let (app, app_state, _tmp) = make_test_app_with_storage().await;
        let data = setup_events_test_data(app_state.db()).await;

        let (token, _) = generate_jwt(data.student1.id, false);
        let req = Request::builder()
            .method("GET")
            .uri("/api/me/events")
            .header("Authorization", format!("Bearer {}", token))
            .body(AxumBody::empty())
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let json: Value = serde_json::from_slice(&body).unwrap();

        let events = json["data"]["events"].as_object().unwrap();

        let mut found_warning = false;
        let mut found_error = false;

        for (_, event_array) in events {
            let events_list = event_array.as_array().unwrap();
            for event in events_list {
                let event_type = event["type"].as_str().unwrap();
                let content = event["content"].as_str().unwrap();

                if event_type == "warning" {
                    found_warning = true;
                    assert!(content.contains("available"));
                }
                if event_type == "error" {
                    found_error = true;
                    assert!(content.contains("due"));
                }
            }
        }

        assert!(found_warning, "Should find at least one 'warning' event");
        assert!(found_error, "Should find at least one 'error' event");
    }

    #[tokio::test]
    #[serial]
    async fn test_get_events_invalid_date_format() {
        let (app, app_state, _tmp) = make_test_app_with_storage().await;
        let data = setup_events_test_data(app_state.db()).await;

        let (token, _) = generate_jwt(data.student1.id, false);
        let req = Request::builder()
            .method("GET")
            .uri("/api/me/events?from=invalid-date")
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
        assert!(
            json["message"]
                .as_str()
                .unwrap()
                .contains("Invalid date format")
        );
    }

    #[tokio::test]
    #[serial]
    async fn test_get_events_invalid_date_range() {
        let (app, app_state, _tmp) = make_test_app_with_storage().await;
        let data = setup_events_test_data(app_state.db()).await;

        let now = Utc::now();
        let from_date = (now + Duration::days(10)).format("%Y-%m-%d").to_string();
        let to_date = (now + Duration::days(5)).format("%Y-%m-%d").to_string();

        let (token, _) = generate_jwt(data.student1.id, false);
        let req = Request::builder()
            .method("GET")
            .uri(&format!("/api/me/events?from={}&to={}", from_date, to_date))
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
        assert!(
            json["message"]
                .as_str()
                .unwrap()
                .contains("'from' date must be before 'to' date")
        );
    }

    #[tokio::test]
    #[serial]
    async fn test_get_events_unauthorized() {
        let (app, _app_state, _tmp) = make_test_app_with_storage().await;

        let req = Request::builder()
            .method("GET")
            .uri("/api/me/events")
            .body(AxumBody::empty())
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }
}
