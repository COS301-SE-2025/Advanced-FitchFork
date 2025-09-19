#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]

    use api::auth::generate_jwt;
    use axum::{
        body::Body as AxumBody,
        http::{Request, StatusCode},
    };
    use chrono::{Datelike, Utc};
    use serde_json::Value;
    use tower::ServiceExt;

    use db::models::{
        attendance_session::{
            Column as SessionCol, Entity as SessionEntity, Model as SessionModel,
        },
        module::Model as ModuleModel,
        user::Model as UserModel,
        user_module_role::{Model as UserModuleRoleModel, Role},
    };
    use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};

    use crate::helpers::app::make_test_app_with_storage;

    struct TestCtx {
        lecturer: UserModel,  // lecturer of `module`
        student: UserModel,   // student of `module`
        tutor: UserModel,     // tutor of `module`
        forbidden: UserModel, // not assigned to `module`
        module: ModuleModel,
        _other_module: ModuleModel, // separate module the main lecturer is not assigned to
        other_lecturer: UserModel,  // lecturer of other_module
        session: SessionModel,      // a session in `module`
    }

    async fn setup(db: &sea_orm::DatabaseConnection) -> TestCtx {
        dotenvy::dotenv().ok();

        // Create two modules
        let module = ModuleModel::create(
            db,
            "ATT301",
            Utc::now().year(),
            Some("Attendance Delete Tests"),
            8,
        )
        .await
        .expect("create module");

        let _other_module =
            ModuleModel::create(db, "ATT302", Utc::now().year(), Some("Other Module"), 8)
                .await
                .expect("create other module");

        // Users
        let lecturer = UserModel::create(db, "del_lect", "del_lect@test.com", "password", false)
            .await
            .unwrap();
        let other_lecturer =
            UserModel::create(db, "del_lect2", "del_lect2@test.com", "password", false)
                .await
                .unwrap();
        let student =
            UserModel::create(db, "del_student", "del_student@test.com", "password", false)
                .await
                .unwrap();
        let tutor = UserModel::create(db, "del_tutor", "del_tutor@test.com", "password", false)
            .await
            .unwrap();
        let forbidden = UserModel::create(
            db,
            "del_forbidden",
            "del_forbidden@test.com",
            "password",
            false,
        )
        .await
        .unwrap();

        // Assign roles for primary module
        UserModuleRoleModel::assign_user_to_module(db, lecturer.id, module.id, Role::Lecturer)
            .await
            .unwrap();
        UserModuleRoleModel::assign_user_to_module(db, student.id, module.id, Role::Student)
            .await
            .unwrap();
        UserModuleRoleModel::assign_user_to_module(db, tutor.id, module.id, Role::Tutor)
            .await
            .unwrap();

        // Assign roles for other module (separately)
        UserModuleRoleModel::assign_user_to_module(
            db,
            other_lecturer.id,
            _other_module.id,
            Role::Lecturer,
        )
        .await
        .unwrap();

        // Seed a session in the primary module
        let secret_hex = "00112233445566778899aabbccddeeff00112233445566778899aabbccddeeff";
        let session = SessionModel::create(
            db,
            module.id,
            lecturer.id,
            "To be deleted",
            true,  // active
            30,    // rotation
            false, // restrict_by_ip
            None,
            None,
            Some(secret_hex),
        )
        .await
        .unwrap();

        TestCtx {
            lecturer,
            student,
            tutor,
            forbidden,
            module,
            _other_module,
            other_lecturer,
            session,
        }
    }

    #[tokio::test]
    async fn test_delete_session_as_lecturer_ok() {
        let (app, app_state, _tmp) = make_test_app_with_storage().await;
        let ctx = setup(app_state.db()).await;

        // sanity: exists
        let before = SessionEntity::find()
            .filter(SessionCol::Id.eq(ctx.session.id))
            .one(app_state.db())
            .await
            .unwrap();
        assert!(before.is_some());

        let (token, _) = generate_jwt(ctx.lecturer.id, ctx.lecturer.admin);
        let uri = format!(
            "/api/modules/{}/attendance/sessions/{}",
            ctx.module.id, ctx.session.id
        );

        let req = Request::builder()
            .method("DELETE")
            .uri(&uri)
            .header("Authorization", format!("Bearer {}", token))
            .body(AxumBody::empty())
            .unwrap();

        let resp = app.clone().oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);

        let bytes = axum::body::to_bytes(resp.into_body(), usize::MAX)
            .await
            .unwrap();
        let json: Value = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(json["success"], true);
        assert_eq!(json["message"], "Attendance session deleted");

        // confirm gone
        let after = SessionEntity::find()
            .filter(SessionCol::Id.eq(ctx.session.id))
            .one(app_state.db())
            .await
            .unwrap();
        assert!(after.is_none());
    }

    #[tokio::test]
    async fn test_delete_session_not_found_returns_404() {
        let (app, app_state, _tmp) = make_test_app_with_storage().await;
        let ctx = setup(app_state.db()).await;

        let unknown = ctx.session.id + 1_000_001;
        let (token, _) = generate_jwt(ctx.lecturer.id, ctx.lecturer.admin);
        let uri = format!(
            "/api/modules/{}/attendance/sessions/{}",
            ctx.module.id, unknown
        );

        let req = Request::builder()
            .method("DELETE")
            .uri(&uri)
            .header("Authorization", format!("Bearer {}", token))
            .body(AxumBody::empty())
            .unwrap();

        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::NOT_FOUND);

        let bytes = axum::body::to_bytes(resp.into_body(), usize::MAX)
            .await
            .unwrap();
        let json: Value = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(json["success"], false);
        let msg = json["message"]
            .as_str()
            .unwrap_or_default()
            .to_ascii_lowercase();
        assert!(msg.contains("not found"));
    }

    #[tokio::test]
    async fn test_delete_session_forbidden_for_student_and_tutor() {
        let (app, app_state, _tmp) = make_test_app_with_storage().await;
        let ctx = setup(app_state.db()).await;

        for uid in [ctx.student.id, ctx.tutor.id] {
            let (token, _) = generate_jwt(uid, false);
            let uri = format!(
                "/api/modules/{}/attendance/sessions/{}",
                ctx.module.id, ctx.session.id
            );

            let req = Request::builder()
                .method("DELETE")
                .uri(&uri)
                .header("Authorization", format!("Bearer {}", token))
                .body(AxumBody::empty())
                .unwrap();

            let resp = app.clone().oneshot(req).await.unwrap();
            assert_eq!(resp.status(), StatusCode::FORBIDDEN);
        }
    }

    #[tokio::test]
    async fn test_delete_session_forbidden_for_unassigned_user() {
        let (app, app_state, _tmp) = make_test_app_with_storage().await;
        let ctx = setup(app_state.db()).await;

        let (token, _) = generate_jwt(ctx.forbidden.id, ctx.forbidden.admin);
        let uri = format!(
            "/api/modules/{}/attendance/sessions/{}",
            ctx.module.id, ctx.session.id
        );

        let req = Request::builder()
            .method("DELETE")
            .uri(&uri)
            .header("Authorization", format!("Bearer {}", token))
            .body(AxumBody::empty())
            .unwrap();

        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::FORBIDDEN);
    }

    #[tokio::test]
    async fn test_delete_session_lecturer_of_other_module_forbidden() {
        // Lecturer of another module should be blocked by the guard (not assigned to THIS module)
        let (app, app_state, _tmp) = make_test_app_with_storage().await;
        let ctx = setup(app_state.db()).await;

        let (token, _) = generate_jwt(ctx.other_lecturer.id, ctx.other_lecturer.admin);
        let uri = format!(
            "/api/modules/{}/attendance/sessions/{}",
            ctx.module.id, ctx.session.id
        );

        let req = Request::builder()
            .method("DELETE")
            .uri(&uri)
            .header("Authorization", format!("Bearer {}", token))
            .body(AxumBody::empty())
            .unwrap();

        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::FORBIDDEN);
    }

    #[tokio::test]
    async fn test_delete_session_missing_auth_unauthorized() {
        let (app, app_state, _tmp) = make_test_app_with_storage().await;
        let ctx = setup(app_state.db()).await;

        let uri = format!(
            "/api/modules/{}/attendance/sessions/{}",
            ctx.module.id, ctx.session.id
        );

        let req = Request::builder()
            .method("DELETE")
            .uri(&uri)
            .body(AxumBody::empty())
            .unwrap();

        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn test_delete_session_emits_session_deleted_broadcast() {
        use api::ws::attendance::topics::attendance_session_topic;
        use tokio::time::{Duration, timeout};

        let (app, app_state, _tmp) = make_test_app_with_storage().await;
        let ctx = setup(app_state.db()).await;

        // Subscribe to the WS topic BEFORE the delete
        let topic = attendance_session_topic(ctx.session.id);
        let mut rx = app_state.ws().subscribe(&topic).await;

        // Perform delete as lecturer
        let (token, _) = generate_jwt(ctx.lecturer.id, ctx.lecturer.admin);
        let uri = format!(
            "/api/modules/{}/attendance/sessions/{}",
            ctx.module.id, ctx.session.id
        );
        let req = Request::builder()
            .method("DELETE")
            .uri(&uri)
            .header("Authorization", format!("Bearer {}", token))
            .body(AxumBody::empty())
            .unwrap();

        let resp = app.clone().oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);

        // Expect a "session_deleted" broadcast with matching session_id
        let msg = timeout(Duration::from_millis(300), rx.recv())
            .await
            .expect("broadcast not timed out")
            .expect("broadcast ok");

        let v: Value = serde_json::from_str(&msg).unwrap();
        assert_eq!(v["event"], "session_deleted");
        assert_eq!(v["payload"]["session_id"], ctx.session.id);
        // (Optional) If you broadcast more fields (e.g., title), assert them here:
        // assert_eq!(v["payload"]["title"], ctx.session.title);
    }
}
