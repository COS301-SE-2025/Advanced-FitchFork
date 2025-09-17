#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]

    use axum::{
        body::Body as AxumBody,
        http::{Request, StatusCode},
    };
    use chrono::{Datelike, Utc};
    use serde_json::Value;
    use tower::ServiceExt;

    use api::auth::generate_jwt;

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
        lecturer: UserModel,
        student: UserModel,
        tutor: UserModel,
        module: ModuleModel,
        session: SessionModel,
    }

    async fn setup(db: &sea_orm::DatabaseConnection) -> TestCtx {
        dotenvy::dotenv().ok();

        // --- module & users
        let module = ModuleModel::create(
            db,
            "ATT301",
            Utc::now().year(),
            Some("Attendance PUT Tests"),
            8,
        )
        .await
        .expect("create module");

        let lecturer = UserModel::create(db, "put_lect", "put_lect@test.com", "password", false)
            .await
            .unwrap();
        let student =
            UserModel::create(db, "put_student", "put_student@test.com", "password", false)
                .await
                .unwrap();
        let tutor = UserModel::create(db, "put_tutor", "put_tutor@test.com", "password", false)
            .await
            .unwrap();

        UserModuleRoleModel::assign_user_to_module(db, lecturer.id, module.id, Role::Lecturer)
            .await
            .unwrap();
        UserModuleRoleModel::assign_user_to_module(db, student.id, module.id, Role::Student)
            .await
            .unwrap();
        UserModuleRoleModel::assign_user_to_module(db, tutor.id, module.id, Role::Tutor)
            .await
            .unwrap();

        // seed a session we will edit
        let session = SessionModel::create(
            db,
            module.id,
            lecturer.id,
            "Original Title",
            false, // inactive
            30,    // rotation
            false, // restrict_by_ip
            None,
            None,
            Some("00112233445566778899aabbccddeeff00112233445566778899aabbccddeeff"),
        )
        .await
        .unwrap();

        TestCtx {
            lecturer,
            student,
            tutor,
            module,
            session,
        }
    }

    // -----------------------------------
    // Happy path: lecturer edits settings
    // -----------------------------------

    #[tokio::test]
    async fn test_edit_session_ok_by_lecturer_updates_fields_and_clamps_rotation() {
        let (app, app_state, _tmp) = make_test_app_with_storage().await;
        let ctx = setup(app_state.db()).await;

        // send rotation_seconds=2 (should clamp to 5)
        // toggle active=true, restrict_by_ip=true, set both CIDR and created_from_ip
        let body = serde_json::json!({
            "title": "Updated Title",
            "active": true,
            "rotation_seconds": 2,
            "restrict_by_ip": true,
            "allowed_ip_cidr": "198.51.100.0/24",
            "created_from_ip": "198.51.100.7"
        });

        let (token, _) = generate_jwt(ctx.lecturer.id, ctx.lecturer.admin);
        let uri = format!(
            "/api/modules/{}/attendance/sessions/{}",
            ctx.module.id, ctx.session.id
        );

        let req = Request::builder()
            .method("PUT")
            .uri(&uri)
            .header("Authorization", format!("Bearer {}", token))
            .header("Content-Type", "application/json")
            .body(AxumBody::from(body.to_string()))
            .unwrap();

        let resp = app.clone().oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);

        let bytes = axum::body::to_bytes(resp.into_body(), usize::MAX)
            .await
            .unwrap();
        let json: Value = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(json["success"], true);
        assert_eq!(json["message"], "Attendance session updated");

        // response echoes updates
        assert_eq!(json["data"]["title"], "Updated Title");
        assert_eq!(json["data"]["active"], true);
        assert_eq!(json["data"]["rotation_seconds"], 5); // clamped
        assert_eq!(json["data"]["restrict_by_ip"], true);
        assert_eq!(json["data"]["allowed_ip_cidr"], "198.51.100.0/24");
        assert_eq!(json["data"]["created_from_ip"], "198.51.100.7");

        // verify persisted state
        let row = SessionEntity::find()
            .filter(SessionCol::Id.eq(ctx.session.id))
            .one(app_state.db())
            .await
            .unwrap()
            .expect("row exists");
        assert_eq!(row.title, "Updated Title");
        assert!(row.active);
        assert_eq!(row.rotation_seconds, 5);
        assert!(row.restrict_by_ip);
        assert_eq!(row.allowed_ip_cidr.as_deref(), Some("198.51.100.0/24"));
        assert_eq!(row.created_from_ip.as_deref(), Some("198.51.100.7"));
    }

    // ---------------------------------------------------
    // Forbidden: student and tutor cannot edit the session
    // ---------------------------------------------------

    #[tokio::test]
    async fn test_edit_session_forbidden_for_student_and_tutor() {
        let (app, app_state, _tmp) = make_test_app_with_storage().await;
        let ctx = setup(app_state.db()).await;

        for uid in [ctx.student.id, ctx.tutor.id] {
            let (token, _) = generate_jwt(uid, false);
            let uri = format!(
                "/api/modules/{}/attendance/sessions/{}",
                ctx.module.id, ctx.session.id
            );
            let body = serde_json::json!({ "title": "Should Not Update" });

            let req = Request::builder()
                .method("PUT")
                .uri(&uri)
                .header("Authorization", format!("Bearer {}", token))
                .header("Content-Type", "application/json")
                .body(AxumBody::from(body.to_string()))
                .unwrap();

            let resp = app.clone().oneshot(req).await.unwrap();
            assert_eq!(resp.status(), StatusCode::FORBIDDEN);
        }
    }

    // -------------------------------
    // Not found: wrong/nonexistent id
    // -------------------------------

    #[tokio::test]
    async fn test_edit_session_not_found_returns_404() {
        let (app, app_state, _tmp) = make_test_app_with_storage().await;
        let ctx = setup(app_state.db()).await;

        let (token, _) = generate_jwt(ctx.lecturer.id, ctx.lecturer.admin);
        let uri = format!(
            "/api/modules/{}/attendance/sessions/{}",
            ctx.module.id,
            ctx.session.id + 1_000_000
        );

        let req = Request::builder()
            .method("PUT")
            .uri(&uri)
            .header("Authorization", format!("Bearer {}", token))
            .header("Content-Type", "application/json")
            .body(AxumBody::from(r#"{"title":"X"}"#))
            .unwrap();

        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::NOT_FOUND);

        let bytes = axum::body::to_bytes(resp.into_body(), usize::MAX)
            .await
            .unwrap();
        let json: serde_json::Value = serde_json::from_slice(&bytes).unwrap();

        assert_eq!(json["success"], false);
        let msg = json["message"]
            .as_str()
            .unwrap_or_default()
            .to_ascii_lowercase();
        assert!(msg.contains("not found")); // middleware or handler message variants
    }

    // --------------------
    // Missing authorization
    // --------------------

    #[tokio::test]
    async fn test_edit_session_missing_auth_unauthorized() {
        let (app, app_state, _tmp) = make_test_app_with_storage().await;
        let ctx = setup(app_state.db()).await;

        let uri = format!(
            "/api/modules/{}/attendance/sessions/{}",
            ctx.module.id, ctx.session.id
        );

        let req = Request::builder()
            .method("PUT")
            .uri(&uri)
            .header("Content-Type", "application/json")
            .body(AxumBody::from(r#"{"title":"No Auth"}"#))
            .unwrap();

        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
    }

    // ---------------------------------------------
    // Clamp upper bound on rotation (e.g. > 300 -> 300)
    // ---------------------------------------------

    #[tokio::test]
    async fn test_edit_session_rotation_upper_clamp() {
        let (app, app_state, _tmp) = make_test_app_with_storage().await;
        let ctx = setup(app_state.db()).await;

        let (token, _) = generate_jwt(ctx.lecturer.id, ctx.lecturer.admin);
        let uri = format!(
            "/api/modules/{}/attendance/sessions/{}",
            ctx.module.id, ctx.session.id
        );

        let body = serde_json::json!({ "rotation_seconds": 9999 });

        let req = Request::builder()
            .method("PUT")
            .uri(&uri)
            .header("Authorization", format!("Bearer {}", token))
            .header("Content-Type", "application/json")
            .body(AxumBody::from(body.to_string()))
            .unwrap();

        let resp = app.clone().oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);

        // confirm clamped in DB
        let row = SessionEntity::find()
            .filter(SessionCol::Id.eq(ctx.session.id))
            .one(app_state.db())
            .await
            .unwrap()
            .expect("row exists");
        assert_eq!(row.rotation_seconds, 300);
    }
}
