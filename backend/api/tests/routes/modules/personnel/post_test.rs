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
    use crate::helpers::app::make_test_app_with_storage;

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
    async fn assign_personnel_as_admin_success() {
        let (app, app_state, _tmp) = make_test_app_with_storage().await;
        let data = setup_data(app_state.db()).await;

        let (token, _) = generate_jwt(data.admin.id, data.admin.admin);
        let uri = format!("/api/modules/{}/personnel", data.module.id);

        let body = serde_json::json!({
            "user_ids": [data.outsider.id],
            "role": "tutor"
        });

        let req = Request::builder()
            .method("POST")
            .uri(&uri)
            .header("Authorization", format!("Bearer {}", token))
            .header("Content-Type", "application/json")
            .body(Body::from(body.to_string()))
            .unwrap();

        let res = app.oneshot(req).await.unwrap();
        let status = res.status();
        let _ = axum::body::to_bytes(res.into_body(), 1024 * 1024).await.unwrap(); // consume body for completeness

        assert_eq!(status, StatusCode::OK);
    }

    #[tokio::test]
    async fn assign_personnel_as_lecturer_success_for_non_lecturer_roles() {
        let (app, app_state, _tmp) = make_test_app_with_storage().await;
        let data = setup_data(app_state.db()).await;

        let (token, _) = generate_jwt(data.lecturer.id, data.lecturer.admin);
        let uri = format!("/api/modules/{}/personnel", data.module.id);

        let req = Request::builder()
            .method("POST")
            .uri(&uri)
            .header("Authorization", format!("Bearer {}", token))
            .header("Content-Type", "application/json")
            .body(Body::from(json!({ "user_ids": [data.outsider.id], "role": "assistant_lecturer" }).to_string()))
            .unwrap();

        let res = app.oneshot(req).await.unwrap();
        assert_eq!(res.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn assign_lecturer_role_as_lecturer_forbidden() {
        let (app, app_state, _tmp) = make_test_app_with_storage().await;
        let data = setup_data(app_state.db()).await;

        let (token, _) = generate_jwt(data.lecturer.id, data.lecturer.admin);
        let uri = format!("/api/modules/{}/personnel", data.module.id);

        let req = Request::builder()
            .method("POST")
            .uri(&uri)
            .header("Authorization", format!("Bearer {}", token))
            .header("Content-Type", "application/json")
            .body(Body::from(json!({ "user_ids": [data.outsider.id], "role": "lecturer" }).to_string()))
            .unwrap();

        let res = app.oneshot(req).await.unwrap();
        assert_eq!(res.status(), StatusCode::FORBIDDEN);
    }

    #[tokio::test]
    async fn assign_personnel_as_non_lecturer_forbidden() {
        let (app, app_state, _tmp) = make_test_app_with_storage().await;
        let data = setup_data(app_state.db()).await;

        let (token, _) = generate_jwt(data.outsider.id, false);
        let uri = format!("/api/modules/{}/personnel", data.module.id);

        let req = Request::builder()
            .method("POST")
            .uri(&uri)
            .header("Authorization", format!("Bearer {}", token))
            .header("Content-Type", "application/json")
            .body(Body::from(json!({ "user_ids": [data.lecturer.id], "role": "tutor" }).to_string()))
            .unwrap();

        let res = app.oneshot(req).await.unwrap();
        assert_eq!(res.status(), StatusCode::FORBIDDEN);
    }

    #[tokio::test]
    async fn assign_personnel_as_student_forbidden() {
        let (app, app_state, _tmp) = make_test_app_with_storage().await;
        let data = setup_data(app_state.db()).await;

        // Student is assigned to the module, but still shouldn't have assign permissions
        let (token, _) = generate_jwt(data.student.id, false);
        let uri = format!("/api/modules/{}/personnel", data.module.id);

        let req = Request::builder()
            .method("POST")
            .uri(&uri)
            .header("Authorization", format!("Bearer {}", token))
            .header("Content-Type", "application/json")
            .body(Body::from(json!({ "user_ids": [data.outsider.id], "role": "tutor" }).to_string()))
            .unwrap();

        let res = app.oneshot(req).await.unwrap();
        assert_eq!(res.status(), StatusCode::FORBIDDEN);
    }

    #[tokio::test]
    async fn assign_personnel_user_not_found() {
        let (app, app_state, _tmp) = make_test_app_with_storage().await;
        let data = setup_data(app_state.db()).await;

        let (token, _) = generate_jwt(data.admin.id, true);
        let uri = format!("/api/modules/{}/personnel", data.module.id);

        let req = Request::builder()
            .method("POST")
            .uri(&uri)
            .header("Authorization", format!("Bearer {}", token))
            .header("Content-Type", "application/json")
            .body(Body::from(json!({ "user_ids": [9999999], "role": "student" }).to_string()))
            .unwrap();

        let res = app.oneshot(req).await.unwrap();
        assert_eq!(res.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn assign_personnel_empty_user_ids() {
        let (app, app_state, _tmp) = make_test_app_with_storage().await;
        let data = setup_data(app_state.db()).await;

        let (token, _) = generate_jwt(data.admin.id, true);
        let uri = format!("/api/modules/{}/personnel", data.module.id);

        let req = Request::builder()
            .method("POST")
            .uri(&uri)
            .header("Authorization", format!("Bearer {}", token))
            .header("Content-Type", "application/json")
            .body(Body::from(json!({ "user_ids": [], "role": "student" }).to_string()))
            .unwrap();

        let res = app.oneshot(req).await.unwrap();
        assert_eq!(res.status(), StatusCode::BAD_REQUEST);
    }
}
