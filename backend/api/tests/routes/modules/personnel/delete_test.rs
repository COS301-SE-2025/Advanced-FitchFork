#[cfg(test)]
mod tests {
    use axum::{body::Body, http::{Request, StatusCode}};
    use tower::ServiceExt;
    use serde_json::json;
    use api::auth::generate_jwt;
    use db::{
        models::{
            user::Model as UserModel,
            module::Model as ModuleModel,
            user_module_role::{Model as UserModuleRoleModel, Role},
        },
    };
    use crate::helpers::app::make_test_app;

    struct TestData {
        admin: UserModel,
        lecturer: UserModel,
        student: UserModel,
        outsider: UserModel,
        module: ModuleModel,
    }

    async fn setup_data(db: &sea_orm::DatabaseConnection) -> TestData {
        let module = ModuleModel::create(db, "COS999", 2025, Some("Test Module"), 12).await.unwrap();
        let admin = UserModel::create(db, "admin", "admin@test.com", "pw", true).await.unwrap();
        let lecturer = UserModel::create(db, "lect1", "lect@test.com", "pw", false).await.unwrap();
        let student = UserModel::create(db, "stud1", "stud@test.com", "pw", false).await.unwrap();
        let outsider = UserModel::create(db, "outsider", "out@test.com", "pw", false).await.unwrap();
        UserModuleRoleModel::assign_user_to_module(db, lecturer.id, module.id, Role::Lecturer).await.unwrap();
        UserModuleRoleModel::assign_user_to_module(db, student.id, module.id, Role::Student).await.unwrap();
        TestData { admin, lecturer, student, outsider, module }
    }

    #[tokio::test]
    async fn remove_personnel_as_admin_success() {
        let (app, app_state) = make_test_app().await;
        let data = setup_data(app_state.db()).await;

        let (token, _) = generate_jwt(data.admin.id, true);
        let uri = format!("/api/modules/{}/personnel", data.module.id);

        let req = Request::builder()
            .method("DELETE")
            .uri(&uri)
            .header("Authorization", format!("Bearer {}", token))
            .header("Content-Type", "application/json")
            .body(Body::from(json!({
                "user_ids": [data.student.id],
                "role": "student"
            }).to_string()))
            .unwrap();

        let res = app.oneshot(req).await.unwrap();
        assert_eq!(res.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn remove_personnel_as_lecturer_success() {
        let (app, app_state) = make_test_app().await;
        let data = setup_data(app_state.db()).await;

        let (token, _) = generate_jwt(data.lecturer.id, false);
        let uri = format!("/api/modules/{}/personnel", data.module.id);

        let req = Request::builder()
            .method("DELETE")
            .uri(&uri)
            .header("Authorization", format!("Bearer {}", token))
            .header("Content-Type", "application/json")
            .body(Body::from(json!({
                "user_ids": [data.student.id],
                "role": "student"
            }).to_string()))
            .unwrap();

        let res = app.oneshot(req).await.unwrap();
        assert_eq!(res.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn remove_personnel_as_lecturer_forbidden_target_lecturer() {
        let (app, app_state) = make_test_app().await;
        let data = setup_data(app_state.db()).await;

        let (token, _) = generate_jwt(data.lecturer.id, false);
        let uri = format!("/api/modules/{}/personnel", data.module.id);

        let req = Request::builder()
            .method("DELETE")
            .uri(&uri)
            .header("Authorization", format!("Bearer {}", token))
            .header("Content-Type", "application/json")
            .body(Body::from(json!({
                "user_ids": [data.admin.id],
                "role": "lecturer"
            }).to_string()))
            .unwrap();

        let res = app.oneshot(req).await.unwrap();
        assert_eq!(res.status(), StatusCode::FORBIDDEN);
    }

    #[tokio::test]
    async fn remove_personnel_as_outsider_forbidden() {
        let (app, app_state) = make_test_app().await;
        let data = setup_data(app_state.db()).await;

        let (token, _) = generate_jwt(data.outsider.id, false);
        let uri = format!("/api/modules/{}/personnel", data.module.id);

        let req = Request::builder()
            .method("DELETE")
            .uri(&uri)
            .header("Authorization", format!("Bearer {}", token))
            .header("Content-Type", "application/json")
            .body(Body::from(json!({
                "user_ids": [data.student.id],
                "role": "student"
            }).to_string()))
            .unwrap();

        let res = app.oneshot(req).await.unwrap();
        assert_eq!(res.status(), StatusCode::FORBIDDEN);
    }

    #[tokio::test]
    async fn remove_personnel_user_not_found() {
        let (app, app_state) = make_test_app().await;
        let data = setup_data(app_state.db()).await;

        let (token, _) = generate_jwt(data.admin.id, true);
        let uri = format!("/api/modules/{}/personnel", data.module.id);

        let req = Request::builder()
            .method("DELETE")
            .uri(&uri)
            .header("Authorization", format!("Bearer {}", token))
            .header("Content-Type", "application/json")
            .body(Body::from(json!({
                "user_ids": [99999999],
                "role": "tutor"
            }).to_string()))
            .unwrap();

        let res = app.oneshot(req).await.unwrap();
        assert_eq!(res.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn remove_personnel_empty_user_ids() {
        let (app, app_state) = make_test_app().await;
        let data = setup_data(app_state.db()).await;

        let (token, _) = generate_jwt(data.admin.id, true);
        let uri = format!("/api/modules/{}/personnel", data.module.id);

        let req = Request::builder()
            .method("DELETE")
            .uri(&uri)
            .header("Authorization", format!("Bearer {}", token))
            .header("Content-Type", "application/json")
            .body(Body::from(json!({
                "user_ids": [],
                "role": "student"
            }).to_string()))
            .unwrap();

        let res = app.oneshot(req).await.unwrap();
        assert_eq!(res.status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn remove_personnel_conflict_user_not_assigned() {
        let (app, app_state) = make_test_app().await;
        let data = setup_data(app_state.db()).await;

        let (token, _) = generate_jwt(data.admin.id, true);
        let uri = format!("/api/modules/{}/personnel", data.module.id);

        let req = Request::builder()
            .method("DELETE")
            .uri(&uri)
            .header("Authorization", format!("Bearer {}", token))
            .header("Content-Type", "application/json")
            .body(Body::from(json!({
                "user_ids": [data.outsider.id],
                "role": "tutor"
            }).to_string()))
            .unwrap();

        let res = app.oneshot(req).await.unwrap();
        assert_eq!(res.status(), StatusCode::CONFLICT);
    }
}
