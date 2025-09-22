#[cfg(test)]
mod tests {
    use crate::helpers::app::make_test_app_with_storage;
    use api::auth::generate_jwt;
    use axum::{
        body::Body,
        http::{Request, StatusCode},
    };
    use db::models::{
        module::Model as ModuleModel,
        user::Model as UserModel,
        user_module_role::{Model as UserModuleRoleModel, Role},
    };
    use serde_json::Value;
    use tower::ServiceExt;

    struct TestData {
        admin: UserModel,
        lecturer: UserModel,
        tutor: UserModel,
        outsider: UserModel,
        module: ModuleModel,
    }

    async fn setup_data(db: &sea_orm::DatabaseConnection) -> TestData {
        let module = ModuleModel::create(db, "COS999", 2025, Some("Test Module"), 12)
            .await
            .unwrap();
        let service = UserService::new(UserRepository::new(db.clone()));
        let admin = service
            .create(CreateUser {
                username: "admin".to_string(),
                email: "admin@test.com".to_string(),
                password: "pw".to_string(),
                admin: true,
            })
            .await
            .unwrap();
        let lecturer = service
            .create(CreateUser {
                username: "lect1".to_string(),
                email: "lect@test.com".to_string(),
                password: "pw".to_string(),
                admin: false,
            })
            .await
            .unwrap();
        let tutor = service
            .create(CreateUser {
                username: "tut1".to_string(),
                email: "tut@test.com".to_string(),
                password: "pw".to_string(),
                admin: false,
            })
            .await
            .unwrap();
        let outsider = service
            .create(CreateUser {
                username: "out".to_string(),
                email: "out@test.com".to_string(),
                password: "pw".to_string(),
                admin: false,
            })
            .await
            .unwrap();

        UserModuleRoleModel::assign_user_to_module(db, lecturer.id, module.id, Role::Lecturer)
            .await
            .unwrap();
        UserModuleRoleModel::assign_user_to_module(db, tutor.id, module.id, Role::Tutor)
            .await
            .unwrap();

        TestData {
            admin,
            lecturer,
            tutor,
            outsider,
            module,
        }
    }

    #[tokio::test]
    #[serial]
    async fn get_personnel_as_admin_success() {
        let (app, app_state, _tmp) = make_test_app_with_storage().await;
        let data = setup_data(app_state.db()).await;

        let (token, _) = generate_jwt(data.admin.id, true);
        let uri = format!("/api/modules/{}/personnel?role=tutor", data.module.id);

        let req = Request::builder()
            .method("GET")
            .uri(&uri)
            .header("Authorization", format!("Bearer {}", token))
            .body(Body::empty())
            .unwrap();

        let res = app.oneshot(req).await.unwrap();
        assert_eq!(res.status(), StatusCode::OK);

        let body = axum::body::to_bytes(res.into_body(), 1024 * 1024)
            .await
            .unwrap();
        let json: Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(json["success"], true);
        assert!(
            json["data"]["users"]
                .as_array()
                .unwrap()
                .iter()
                .any(|u| u["id"] == data.tutor.id)
        );
    }

    #[tokio::test]
    #[serial]
    async fn get_personnel_as_lecturer_for_tutors_success() {
        let (app, app_state, _tmp) = make_test_app_with_storage().await;
        let data = setup_data(app_state.db()).await;

        let (token, _) = generate_jwt(data.lecturer.id, false);
        let uri = format!("/api/modules/{}/personnel?role=tutor", data.module.id);

        let req = Request::builder()
            .method("GET")
            .uri(&uri)
            .header("Authorization", format!("Bearer {}", token))
            .body(Body::empty())
            .unwrap();

        let res = app.oneshot(req).await.unwrap();
        assert_eq!(res.status(), StatusCode::OK);
    }

    #[tokio::test]
    #[serial]
    async fn get_personnel_as_lecturer_for_lecturer_role_forbidden() {
        let (app, app_state, _tmp) = make_test_app_with_storage().await;
        let data = setup_data(app_state.db()).await;

        let (token, _) = generate_jwt(data.lecturer.id, false);
        let uri = format!("/api/modules/{}/personnel?role=lecturer", data.module.id);

        let req = Request::builder()
            .method("GET")
            .uri(&uri)
            .header("Authorization", format!("Bearer {}", token))
            .body(Body::empty())
            .unwrap();

        let res = app.oneshot(req).await.unwrap();
        assert_eq!(res.status(), StatusCode::FORBIDDEN);
    }

    #[tokio::test]
    #[serial]
    async fn get_personnel_as_non_member_forbidden() {
        let (app, app_state, _tmp) = make_test_app_with_storage().await;
        let data = setup_data(app_state.db()).await;

        let (token, _) = generate_jwt(data.outsider.id, false);
        let uri = format!("/api/modules/{}/personnel?role=student", data.module.id);

        let req = Request::builder()
            .method("GET")
            .uri(&uri)
            .header("Authorization", format!("Bearer {}", token))
            .body(Body::empty())
            .unwrap();

        let res = app.oneshot(req).await.unwrap();
        assert_eq!(res.status(), StatusCode::FORBIDDEN);
    }

    #[tokio::test]
    #[serial]
    async fn get_eligible_users_as_admin_success() {
        let (app, app_state, _tmp) = make_test_app_with_storage().await;
        let data = setup_data(app_state.db()).await;

        let (token, _) = generate_jwt(data.admin.id, true);
        let uri = format!("/api/modules/{}/personnel/eligible", data.module.id);

        let req = Request::builder()
            .method("GET")
            .uri(&uri)
            .header("Authorization", format!("Bearer {}", token))
            .body(Body::empty())
            .unwrap();

        let res = app.oneshot(req).await.unwrap();
        assert_eq!(res.status(), StatusCode::OK);

        let body = axum::body::to_bytes(res.into_body(), 1024 * 1024)
            .await
            .unwrap();
        let json: Value = serde_json::from_slice(&body).unwrap();

        assert_eq!(json["success"], true);
        assert_eq!(json["message"], "Eligible users fetched");
        let user_ids: Vec<i64> = json["data"]["users"]
            .as_array()
            .unwrap()
            .iter()
            .map(|u| u["id"].as_i64().unwrap())
            .collect();

        assert!(user_ids.contains(&data.outsider.id));
        assert!(!user_ids.contains(&data.lecturer.id));
    }

    #[tokio::test]
    #[serial]
    async fn get_eligible_users_pagination_and_filtering() {
        let (app, app_state, _tmp) = make_test_app_with_storage().await;
        let data = setup_data(app_state.db()).await;

        let (token, _) = generate_jwt(data.admin.id, true);
        let uri = format!(
            "/api/modules/{}/personnel/eligible?page=1&per_page=1&username=out",
            data.module.id
        );

        let req = Request::builder()
            .method("GET")
            .uri(&uri)
            .header("Authorization", format!("Bearer {}", token))
            .body(Body::empty())
            .unwrap();

        let res = app.oneshot(req).await.unwrap();
        assert_eq!(res.status(), StatusCode::OK);

        let body = axum::body::to_bytes(res.into_body(), 1024 * 1024)
            .await
            .unwrap();
        let json: Value = serde_json::from_slice(&body).unwrap();

        assert_eq!(json["data"]["page"], 1);
        assert_eq!(json["data"]["per_page"], 1);
        assert!(
            json["data"]["users"]
                .as_array()
                .unwrap()
                .iter()
                .any(|u| u["username"] == "out")
        );
    }
}
