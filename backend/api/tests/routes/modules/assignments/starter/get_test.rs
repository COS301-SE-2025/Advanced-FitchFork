#[cfg(test)]
mod tests {
    use axum::{body::Body, http::{Request, StatusCode}};
    use tower::ServiceExt;
    use serde_json::Value;
    use serial_test::serial;

    use api::auth::generate_jwt;
    use api::routes::modules::assignments::starter::common::STARTER_PACKS;
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
        let module = ModuleModel::create(db, "MOD100", 2024, Some("Module"), 16)
            .await
            .unwrap();

        let admin = UserModel::create(db, "admin", "admin@test.com", "pw", true)
            .await
            .unwrap();
        let lecturer = UserModel::create(db, "lect", "lect@test.com", "pw", false)
            .await
            .unwrap();
        let assistant = UserModel::create(db, "al", "al@test.com", "pw", false)
            .await
            .unwrap();
        let tutor = UserModel::create(db, "tut", "tut@test.com", "pw", false)
            .await
            .unwrap();
        let student = UserModel::create(db, "stud", "stud@test.com", "pw", false)
            .await
            .unwrap();
        let outsider = UserModel::create(db, "out", "out@test.com", "pw", false)
            .await
            .unwrap();

        UserModuleRoleModel::assign_user_to_module(db, lecturer.id, module.id, Role::Lecturer)
            .await
            .unwrap();
        UserModuleRoleModel::assign_user_to_module(
            db,
            assistant.id,
            module.id,
            Role::AssistantLecturer,
        )
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
            "A1",
            Some("Desc"),
            AssignmentType::Assignment,
            Utc::now(),
            Utc::now(),
        )
        .await
        .unwrap();

        TestData {
            admin,
            lecturer,
            assistant,
            tutor,
            student,
            outsider,
            module,
            assignment,
        }
    }

    fn starter_uri(module_id: i64, assignment_id: i64) -> String {
        format!(
            "/api/modules/{}/assignments/{}/starter",
            module_id, assignment_id
        )
    }

    #[tokio::test]
    #[serial]
    async fn list_starter_packs_as_admin_ok() {
        let (app, app_state, _tmp) = make_test_app_with_storage().await;
        let data = setup(app_state.db()).await;

        let (token, _) = generate_jwt(data.admin.id, data.admin.admin);
        let req = Request::builder()
            .uri(starter_uri(data.module.id, data.assignment.id))
            .header("Authorization", format!("Bearer {}", token))
            .body(Body::empty())
            .unwrap();

        let res = app.oneshot(req).await.unwrap();
        assert_eq!(res.status(), StatusCode::OK);

        let body = axum::body::to_bytes(res.into_body(), usize::MAX).await.unwrap();
        let json: Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(json["success"], true);

        let items = json["data"].as_array().unwrap();
        assert_eq!(items.len(), STARTER_PACKS.len());
        let ids: Vec<&str> = items
            .iter()
            .map(|v| v["id"].as_str().unwrap())
            .collect();
        assert!(ids.contains(&STARTER_PACKS[0].id));
    }

    #[tokio::test]
    #[serial]
    async fn list_starter_packs_as_lecturer_ok() {
        let (app, app_state, _tmp) = make_test_app_with_storage().await;
        let data = setup(app_state.db()).await;

        let (token, _) = generate_jwt(data.lecturer.id, data.lecturer.admin);
        let req = Request::builder()
            .uri(starter_uri(data.module.id, data.assignment.id))
            .header("Authorization", format!("Bearer {}", token))
            .body(Body::empty())
            .unwrap();

        let res = app.oneshot(req).await.unwrap();
        assert_eq!(res.status(), StatusCode::OK);
    }

    #[tokio::test]
    #[serial]
    async fn list_starter_packs_as_assistant_ok() {
        let (app, app_state, _tmp) = make_test_app_with_storage().await;
        let data = setup(app_state.db()).await;

        let (token, _) = generate_jwt(data.assistant.id, data.assistant.admin);
        let req = Request::builder()
            .uri(starter_uri(data.module.id, data.assignment.id))
            .header("Authorization", format!("Bearer {}", token))
            .body(Body::empty())
            .unwrap();

        let res = app.oneshot(req).await.unwrap();
        assert_eq!(res.status(), StatusCode::OK);
    }

    #[tokio::test]
    #[serial]
    async fn list_starter_packs_as_tutor_forbidden() {
        let (app, app_state, _tmp) = make_test_app_with_storage().await;
        let data = setup(app_state.db()).await;

        let (token, _) = generate_jwt(data.tutor.id, data.tutor.admin);
        let req = Request::builder()
            .uri(starter_uri(data.module.id, data.assignment.id))
            .header("Authorization", format!("Bearer {}", token))
            .body(Body::empty())
            .unwrap();

        let res = app.oneshot(req).await.unwrap();
        assert_eq!(res.status(), StatusCode::FORBIDDEN);
    }

    #[tokio::test]
    #[serial]
    async fn list_starter_packs_as_student_forbidden() {
        let (app, app_state, _tmp) = make_test_app_with_storage().await;
        let data = setup(app_state.db()).await;

        let (token, _) = generate_jwt(data.student.id, data.student.admin);
        let req = Request::builder()
            .uri(starter_uri(data.module.id, data.assignment.id))
            .header("Authorization", format!("Bearer {}", token))
            .body(Body::empty())
            .unwrap();

        let res = app.oneshot(req).await.unwrap();
        assert_eq!(res.status(), StatusCode::FORBIDDEN);
    }

    #[tokio::test]
    #[serial]
    async fn list_starter_packs_unassigned_forbidden() {
        let (app, app_state, _tmp) = make_test_app_with_storage().await;
        let data = setup(app_state.db()).await;

        let (token, _) = generate_jwt(data.outsider.id, data.outsider.admin);
        let req = Request::builder()
            .uri(starter_uri(data.module.id, data.assignment.id))
            .header("Authorization", format!("Bearer {}", token))
            .body(Body::empty())
            .unwrap();

        let res = app.oneshot(req).await.unwrap();
        assert_eq!(res.status(), StatusCode::FORBIDDEN);
    }
}
