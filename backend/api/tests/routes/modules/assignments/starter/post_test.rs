#[cfg(test)]
mod tests {
    use axum::{body::Body, http::{Request, StatusCode, header::CONTENT_TYPE}};
    use tower::ServiceExt;
    use serde_json::json;
    use serial_test::serial;

    use api::auth::generate_jwt;
    use crate::helpers::app::make_test_app_with_storage;

    use db::models::{
        assignment::{AssignmentType, Model as AssignmentModel},
        module::Model as ModuleModel,
        user::Model as UserModel,
        user_module_role::{Model as UserModuleRoleModel, Role},
    };
    use chrono::Utc;

    struct TestData {
        admin: UserModel,
        lecturer: UserModel,
        assistant: UserModel,
        tutor: UserModel,
        student: UserModel,
        outsider: UserModel,
        module: ModuleModel,
        assignment: AssignmentModel,
    }

    async fn setup(db: &sea_orm::DatabaseConnection) -> TestData {
        let module = ModuleModel::create(db, "MOD200", 2024, Some("Module"), 16)
            .await
            .unwrap();

        let admin = UserModel::create(db, "admin2", "admin2@test.com", "pw", true)
            .await
            .unwrap();
        let lecturer = UserModel::create(db, "lect2", "lect2@test.com", "pw", false)
            .await
            .unwrap();
        let assistant = UserModel::create(db, "al2", "al2@test.com", "pw", false)
            .await
            .unwrap();
        let tutor = UserModel::create(db, "tut2", "tut2@test.com", "pw", false)
            .await
            .unwrap();
        let student = UserModel::create(db, "stud2", "stud2@test.com", "pw", false)
            .await
            .unwrap();
        let outsider = UserModel::create(db, "out2", "out2@test.com", "pw", false)
            .await
            .unwrap();

        UserModuleRoleModel::assign_user_to_module(db, lecturer.id, module.id, Role::Lecturer)
            .await
            .unwrap();
        UserModuleRoleModel::assign_user_to_module(db, assistant.id, module.id, Role::AssistantLecturer)
            .await
            .unwrap();
        UserModuleRoleModel::assign_user_to_module(db, tutor.id, module.id, Role::Tutor)
            .await
            .unwrap();
        UserModuleRoleModel::assign_user_to_module(db, student.id, module.id, Role::Student)
            .await
            .unwrap();

        let assignment = AssignmentModel::create(
            db,
            module.id,
            "A2",
            Some("Desc"),
            AssignmentType::Assignment,
            Utc::now(),
            Utc::now(),
        )
        .await
        .unwrap();

        TestData { admin, lecturer, assistant, tutor, student, outsider, module, assignment }
    }

    fn starter_uri(module_id: i64, assignment_id: i64) -> String {
        format!("/api/modules/{}/assignments/{}/starter", module_id, assignment_id)
    }

    #[tokio::test]
    #[serial]
    async fn install_starter_as_admin_ok() {
        let (app, app_state, _tmp) = make_test_app_with_storage().await;
        let data = setup(app_state.db()).await;

        let (token, _) = generate_jwt(data.admin.id, data.admin.admin);
        let req = Request::builder()
            .method("POST")
            .uri(starter_uri(data.module.id, data.assignment.id))
            .header("Authorization", format!("Bearer {}", token))
            .header(CONTENT_TYPE, "application/json")
            .body(Body::from(json!({"id": "cpp-linkedlist"}).to_string()))
            .unwrap();

        let res = app.oneshot(req).await.unwrap();
        assert_eq!(res.status(), StatusCode::CREATED);
    }

    #[tokio::test]
    #[serial]
    async fn install_starter_as_lecturer_ok() {
        let (app, app_state, _tmp) = make_test_app_with_storage().await;
        let data = setup(app_state.db()).await;

        let (token, _) = generate_jwt(data.lecturer.id, data.lecturer.admin);
        let req = Request::builder()
            .method("POST")
            .uri(starter_uri(data.module.id, data.assignment.id))
            .header("Authorization", format!("Bearer {}", token))
            .header(CONTENT_TYPE, "application/json")
            .body(Body::from(json!({"id": "cpp-linkedlist"}).to_string()))
            .unwrap();

        let res = app.oneshot(req).await.unwrap();
        assert_eq!(res.status(), StatusCode::CREATED);
    }

    #[tokio::test]
    #[serial]
    async fn install_starter_as_assistant_ok() {
        let (app, app_state, _tmp) = make_test_app_with_storage().await;
        let data = setup(app_state.db()).await;

        let (token, _) = generate_jwt(data.assistant.id, data.assistant.admin);
        let req = Request::builder()
            .method("POST")
            .uri(starter_uri(data.module.id, data.assignment.id))
            .header("Authorization", format!("Bearer {}", token))
            .header(CONTENT_TYPE, "application/json")
            .body(Body::from(json!({"id": "cpp-linkedlist"}).to_string()))
            .unwrap();

        let res = app.oneshot(req).await.unwrap();
        assert_eq!(res.status(), StatusCode::CREATED);
    }

    #[tokio::test]
    #[serial]
    async fn install_starter_as_tutor_forbidden() {
        let (app, app_state, _tmp) = make_test_app_with_storage().await;
        let data = setup(app_state.db()).await;

        let (token, _) = generate_jwt(data.tutor.id, data.tutor.admin);
        let req = Request::builder()
            .method("POST")
            .uri(starter_uri(data.module.id, data.assignment.id))
            .header("Authorization", format!("Bearer {}", token))
            .header(CONTENT_TYPE, "application/json")
            .body(Body::from(json!({"id": "cpp-linkedlist"}).to_string()))
            .unwrap();

        let res = app.oneshot(req).await.unwrap();
        assert_eq!(res.status(), StatusCode::FORBIDDEN);
    }

    #[tokio::test]
    #[serial]
    async fn install_starter_as_student_forbidden() {
        let (app, app_state, _tmp) = make_test_app_with_storage().await;
        let data = setup(app_state.db()).await;

        let (token, _) = generate_jwt(data.student.id, data.student.admin);
        let req = Request::builder()
            .method("POST")
            .uri(starter_uri(data.module.id, data.assignment.id))
            .header("Authorization", format!("Bearer {}", token))
            .header(CONTENT_TYPE, "application/json")
            .body(Body::from(json!({"id": "cpp-linkedlist"}).to_string()))
            .unwrap();

        let res = app.oneshot(req).await.unwrap();
        assert_eq!(res.status(), StatusCode::FORBIDDEN);
    }

    #[tokio::test]
    #[serial]
    async fn install_starter_unassigned_forbidden() {
        let (app, app_state, _tmp) = make_test_app_with_storage().await;
        let data = setup(app_state.db()).await;

        let (token, _) = generate_jwt(data.outsider.id, data.outsider.admin);
        let req = Request::builder()
            .method("POST")
            .uri(starter_uri(data.module.id, data.assignment.id))
            .header("Authorization", format!("Bearer {}", token))
            .body(Body::from(json!({"id": "cpp-linkedlist"}).to_string()))
            .unwrap();

        let res = app.oneshot(req).await.unwrap();
        assert_eq!(res.status(), StatusCode::FORBIDDEN);
    }

    #[tokio::test]
    #[serial]
    async fn install_starter_unknown_pack_422() {
        let (app, app_state, _tmp) = make_test_app_with_storage().await;
        let data = setup(app_state.db()).await;

        let (token, _) = generate_jwt(data.lecturer.id, data.lecturer.admin);
        let req = Request::builder()
            .method("POST")
            .uri(starter_uri(data.module.id, data.assignment.id))
            .header("Authorization", format!("Bearer {}", token))
            .header(CONTENT_TYPE, "application/json")
            .body(Body::from(json!({"id": "does-not-exist"}).to_string()))
            .unwrap();

        let res = app.oneshot(req).await.unwrap();
        assert_eq!(res.status(), StatusCode::UNPROCESSABLE_ENTITY);
    }

    #[tokio::test]
    #[serial]
    async fn install_starter_assignment_not_found_404() {
        let (app, app_state, _tmp) = make_test_app_with_storage().await;
        let data = setup(app_state.db()).await;

        let (token, _) = generate_jwt(data.lecturer.id, data.lecturer.admin);
        // valid module, bogus assignment id
        let req = Request::builder()
            .method("POST")
            .uri(starter_uri(data.module.id, 999999))
            .header("Authorization", format!("Bearer {}", token))
            .header(CONTENT_TYPE, "application/json")
            .body(Body::from(json!({"id": "cpp-linkedlist"}).to_string()))
            .unwrap();

        let res = app.oneshot(req).await.unwrap();
        // validate_known_ids or the handler should report not found
        assert!(res.status() == StatusCode::NOT_FOUND || res.status() == StatusCode::BAD_REQUEST);
    }
}
