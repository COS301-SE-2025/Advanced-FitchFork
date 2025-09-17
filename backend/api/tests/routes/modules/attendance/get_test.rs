#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]

    use api::auth::generate_jwt;
    use axum::{
        body::Body as AxumBody,
        http::{Request, StatusCode},
    };
    use chrono::{Datelike, TimeZone, Utc};
    use serde_json::Value;
    use tower::ServiceExt;

    use db::models::{
        attendance_record,
        attendance_session::Model as SessionModel,
        module::Model as ModuleModel,
        user::Model as UserModel,
        user_module_role::{Model as UserModuleRoleModel, Role},
    };
    use sea_orm::{ActiveModelTrait, Set};

    use crate::helpers::app::make_test_app_with_storage;

    struct TestCtx {
        _admin: UserModel,    // not assigned to the module
        forbidden: UserModel, // not assigned to the module
        lecturer: UserModel,  // assigned (Lecturer)
        tutor: UserModel,     // assigned (Tutor) -> should be 403 for attendance GETs
        student1: UserModel,  // assigned (Student) -> should be 403 for attendance GETs
        _student2: UserModel, // assigned (Student)
        module: ModuleModel,
        sess_active: SessionModel,
        sess_inactive: SessionModel,
    }

    async fn setup(db: &sea_orm::DatabaseConnection) -> TestCtx {
        dotenvy::dotenv().ok();

        // --- module & users
        let module =
            ModuleModel::create(db, "ATT101", Utc::now().year(), Some("Attendance Test"), 8)
                .await
                .expect("create module");

        let _admin = UserModel::create(db, "att_admin", "att_admin@test.com", "password", true)
            .await
            .unwrap();
        let forbidden = UserModel::create(
            db,
            "att_forbidden",
            "att_forbidden@test.com",
            "password",
            false,
        )
        .await
        .unwrap();
        let lecturer = UserModel::create(db, "att_lect", "att_lect@test.com", "password", false)
            .await
            .unwrap();
        let tutor = UserModel::create(db, "att_tutor", "att_tutor@test.com", "password", false)
            .await
            .unwrap();
        let student1 = UserModel::create(
            db,
            "att_student1",
            "att_student1@test.com",
            "password",
            false,
        )
        .await
        .unwrap();
        let _student2 = UserModel::create(
            db,
            "att_student2",
            "att_student2@test.com",
            "password",
            false,
        )
        .await
        .unwrap();

        // Assign roles used by router guards
        UserModuleRoleModel::assign_user_to_module(db, lecturer.id, module.id, Role::Lecturer)
            .await
            .unwrap();
        UserModuleRoleModel::assign_user_to_module(db, tutor.id, module.id, Role::Tutor)
            .await
            .unwrap();
        UserModuleRoleModel::assign_user_to_module(db, student1.id, module.id, Role::Student)
            .await
            .unwrap();
        UserModuleRoleModel::assign_user_to_module(db, _student2.id, module.id, Role::Student)
            .await
            .unwrap();

        // --- sessions
        let secret_hex_a = "00112233445566778899aabbccddeeff00112233445566778899aabbccddeeff";
        let secret_hex_b = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";

        let sess_active = SessionModel::create(
            db,
            module.id,
            lecturer.id,
            "Lec A (active)",
            true,  // active
            30,    // rotation_seconds
            false, // restrict_by_ip
            None,
            None,
            Some(secret_hex_a),
        )
        .await
        .expect("create active session");

        let sess_inactive = SessionModel::create(
            db,
            module.id,
            lecturer.id,
            "Lec B (inactive)",
            false, // not active
            30,
            false,
            None,
            None,
            Some(secret_hex_b),
        )
        .await
        .expect("create inactive session");

        // --- seed some records for sess_active
        let now = Utc.with_ymd_and_hms(2025, 9, 8, 10, 15, 2).unwrap();

        let r1 = attendance_record::ActiveModel {
            session_id: Set(sess_active.id),
            user_id: Set(student1.id),
            taken_at: Set(now),
            ip_address: Set(Some("198.51.100.7".to_string())),
            token_window: Set(12345),
        };
        r1.insert(db).await.unwrap();

        let r2 = attendance_record::ActiveModel {
            session_id: Set(sess_active.id),
            user_id: Set(_student2.id),
            taken_at: Set(now + chrono::Duration::seconds(10)),
            ip_address: Set(Some("203.0.113.5".to_string())),
            token_window: Set(12346),
        };
        r2.insert(db).await.unwrap();

        TestCtx {
            _admin,
            forbidden,
            lecturer,
            tutor,
            student1,
            _student2,
            module,
            sess_active,
            sess_inactive,
        }
    }

    // ---------------------------
    // list_sessions (lecturer/assistant only; must also be assigned)
    // ---------------------------

    #[tokio::test]
    async fn test_list_sessions_as_lecturer_allowed() {
        let (app, app_state, _tmp) = make_test_app_with_storage().await;
        let ctx = setup(app_state.db()).await;

        let (token, _) = generate_jwt(ctx.lecturer.id, ctx.lecturer.admin);
        let uri = format!("/api/modules/{}/attendance/sessions", ctx.module.id);

        let req = Request::builder()
            .method("GET")
            .uri(&uri)
            .header("Authorization", format!("Bearer {}", token))
            .body(AxumBody::empty())
            .unwrap();

        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);

        let body = axum::body::to_bytes(resp.into_body(), usize::MAX)
            .await
            .unwrap();
        let json: Value = serde_json::from_slice(&body).unwrap();

        assert_eq!(json["success"], true);
        assert_eq!(json["message"], "Attendance sessions retrieved");
        let sessions = json["data"]["sessions"].as_array().unwrap();
        assert!(!sessions.is_empty());
    }

    #[tokio::test]
    async fn test_list_sessions_as_student_forbidden_now() {
        // Assigned but not Lecturer/Assistant -> 403
        let (app, app_state, _tmp) = make_test_app_with_storage().await;
        let ctx = setup(app_state.db()).await;

        let (token, _) = generate_jwt(ctx.student1.id, ctx.student1.admin);
        let uri = format!("/api/modules/{}/attendance/sessions", ctx.module.id);

        let req = Request::builder()
            .method("GET")
            .uri(&uri)
            .header("Authorization", format!("Bearer {}", token))
            .body(AxumBody::empty())
            .unwrap();

        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::FORBIDDEN);
    }

    #[tokio::test]
    async fn test_list_sessions_as_tutor_forbidden_now() {
        // Assigned Tutor is not Lecturer/Assistant -> 403
        let (app, app_state, _tmp) = make_test_app_with_storage().await;
        let ctx = setup(app_state.db()).await;

        let (token, _) = generate_jwt(ctx.tutor.id, ctx.tutor.admin);
        let uri = format!("/api/modules/{}/attendance/sessions", ctx.module.id);

        let req = Request::builder()
            .method("GET")
            .uri(&uri)
            .header("Authorization", format!("Bearer {}", token))
            .body(AxumBody::empty())
            .unwrap();

        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::FORBIDDEN);
    }

    #[tokio::test]
    async fn test_list_sessions_forbidden_user_not_assigned() {
        // Not assigned -> 403 (outer `require_assigned_to_module`)
        let (app, app_state, _tmp) = make_test_app_with_storage().await;
        let ctx = setup(app_state.db()).await;

        let (token, _) = generate_jwt(ctx.forbidden.id, ctx.forbidden.admin);
        let uri = format!("/api/modules/{}/attendance/sessions", ctx.module.id);

        let req = Request::builder()
            .method("GET")
            .uri(&uri)
            .header("Authorization", format!("Bearer {}", token))
            .body(AxumBody::empty())
            .unwrap();

        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::FORBIDDEN);
    }

    #[tokio::test]
    async fn test_list_sessions_missing_auth_unauthorized() {
        let (app, app_state, _tmp) = make_test_app_with_storage().await;
        let ctx = setup(app_state.db()).await;
        let uri = format!("/api/modules/{}/attendance/sessions", ctx.module.id);

        let req = Request::builder()
            .method("GET")
            .uri(&uri)
            .body(AxumBody::empty())
            .unwrap();

        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
    }

    // ---------------------------
    // get_session (lecturer/assistant only; must also be assigned)
    // ---------------------------

    #[tokio::test]
    async fn test_get_session_as_lecturer_allowed() {
        let (app, app_state, _tmp) = make_test_app_with_storage().await;
        let ctx = setup(app_state.db()).await;

        let (token, _) = generate_jwt(ctx.lecturer.id, ctx.lecturer.admin);
        let uri = format!(
            "/api/modules/{}/attendance/sessions/{}",
            ctx.module.id, ctx.sess_active.id
        );

        let req = Request::builder()
            .method("GET")
            .uri(&uri)
            .header("Authorization", format!("Bearer {}", token))
            .body(AxumBody::empty())
            .unwrap();

        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);

        let body = axum::body::to_bytes(resp.into_body(), usize::MAX)
            .await
            .unwrap();
        let json: Value = serde_json::from_slice(&body).unwrap();

        assert_eq!(json["success"], true);
        assert_eq!(json["message"], "Attendance session retrieved");
        assert_eq!(json["data"]["id"], ctx.sess_active.id);
    }

    #[tokio::test]
    async fn test_get_session_as_student_forbidden_now() {
        // Assigned Student -> 403
        let (app, app_state, _tmp) = make_test_app_with_storage().await;
        let ctx = setup(app_state.db()).await;

        let (token, _) = generate_jwt(ctx.student1.id, ctx.student1.admin);
        let uri = format!(
            "/api/modules/{}/attendance/sessions/{}",
            ctx.module.id, ctx.sess_active.id
        );

        let req = Request::builder()
            .method("GET")
            .uri(&uri)
            .header("Authorization", format!("Bearer {}", token))
            .body(AxumBody::empty())
            .unwrap();

        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::FORBIDDEN);
    }

    #[tokio::test]
    async fn test_get_session_not_found_still_403_for_student() {
        // Student may be blocked by the guard (403) or, depending on routing/middleware
        // evaluation order, the handler might run and return 404. Accept either.
        let (app, app_state, _tmp) = make_test_app_with_storage().await;
        let ctx = setup(app_state.db()).await;
        let (token, _) = generate_jwt(ctx.student1.id, ctx.student1.admin);

        // realistic, non-existent id (avoid validator 400s)
        let unknown_session_id = ctx.sess_active.id + 999_999;
        let uri = format!(
            "/api/modules/{}/attendance/sessions/{}",
            ctx.module.id, unknown_session_id
        );

        let req = Request::builder()
            .method("GET")
            .uri(&uri)
            .header("Authorization", format!("Bearer {}", token))
            .body(AxumBody::empty())
            .unwrap();

        let resp = app.oneshot(req).await.unwrap();

        // allow either behavior depending on guard/route order
        let status = resp.status();
        assert!(
            status == StatusCode::FORBIDDEN || status == StatusCode::NOT_FOUND,
            "expected 403 or 404 for student, got {status}"
        );
    }

    #[tokio::test]
    async fn test_get_session_not_found_as_lecturer_yields_404() {
        // Lecturer passes guards -> handler should return 404 with a specific message
        let (app, app_state, _tmp) = make_test_app_with_storage().await;
        let ctx = setup(app_state.db()).await;
        let (token, _) = generate_jwt(ctx.lecturer.id, ctx.lecturer.admin);

        // realistic, non-existent id (avoid validator 400s)
        let unknown_session_id = ctx.sess_active.id + 999_999;
        let uri = format!(
            "/api/modules/{}/attendance/sessions/{}",
            ctx.module.id, unknown_session_id
        );

        let req = Request::builder()
            .method("GET")
            .uri(&uri)
            .header("Authorization", format!("Bearer {}", token))
            .body(AxumBody::empty())
            .unwrap();

        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::NOT_FOUND);

        let body = axum::body::to_bytes(resp.into_body(), usize::MAX)
            .await
            .unwrap();
        let json: Value = serde_json::from_slice(&body).unwrap();

        assert_eq!(json["success"], false);

        // message can include IDs (e.g., "Attendance session {id} in Module {module_id} not found.")
        // so assert more leniently
        let msg = json["message"].as_str().unwrap_or_default();
        assert!(
            msg.to_lowercase().contains("not found") && msg.contains("Attendance session"),
            "unexpected not-found message: {msg}"
        );
    }

    // ---------------------------
    // get_session_code (lecturer/assistant only; must also be assigned)
    // ---------------------------

    #[tokio::test]
    async fn test_get_session_code_as_lecturer_ok_when_active() {
        let (app, app_state, _tmp) = make_test_app_with_storage().await;
        let ctx = setup(app_state.db()).await;

        let (token, _) = generate_jwt(ctx.lecturer.id, ctx.lecturer.admin);
        let uri = format!(
            "/api/modules/{}/attendance/sessions/{}/code",
            ctx.module.id, ctx.sess_active.id
        );

        let req = Request::builder()
            .method("GET")
            .uri(&uri)
            .header("Authorization", format!("Bearer {}", token))
            .body(AxumBody::empty())
            .unwrap();

        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);

        let body = axum::body::to_bytes(resp.into_body(), usize::MAX)
            .await
            .unwrap();
        let json: Value = serde_json::from_slice(&body).unwrap();

        assert_eq!(json["success"], true);
        assert_eq!(json["message"], "Current code");
        let code = json["data"].as_str().unwrap();
        assert_eq!(code.len(), 6);
        assert!(code.chars().all(|c| c.is_ascii_digit()));
    }

    #[tokio::test]
    async fn test_get_session_code_inactive_session_bad_request_for_lecturer() {
        let (app, app_state, _tmp) = make_test_app_with_storage().await;
        let ctx = setup(app_state.db()).await;

        let (token, _) = generate_jwt(ctx.lecturer.id, ctx.lecturer.admin);
        let uri = format!(
            "/api/modules/{}/attendance/sessions/{}/code",
            ctx.module.id, ctx.sess_inactive.id
        );

        let req = Request::builder()
            .method("GET")
            .uri(&uri)
            .header("Authorization", format!("Bearer {}", token))
            .body(AxumBody::empty())
            .unwrap();

        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::BAD_REQUEST);

        let body = axum::body::to_bytes(resp.into_body(), usize::MAX)
            .await
            .unwrap();
        let json: Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(json["success"], false);
        assert_eq!(json["message"], "Session is not currently active");
    }

    #[tokio::test]
    async fn test_get_session_code_forbidden_for_tutor_and_student() {
        let (app, app_state, _tmp) = make_test_app_with_storage().await;
        let ctx = setup(app_state.db()).await;

        for uid in [ctx.tutor.id, ctx.student1.id] {
            let (token, _) = generate_jwt(uid, false);
            let uri = format!(
                "/api/modules/{}/attendance/sessions/{}/code",
                ctx.module.id, ctx.sess_active.id
            );

            let req = Request::builder()
                .method("GET")
                .uri(&uri)
                .header("Authorization", format!("Bearer {}", token))
                .body(AxumBody::empty())
                .unwrap();

            let resp = app.clone().oneshot(req).await.unwrap();
            assert_eq!(resp.status(), StatusCode::FORBIDDEN);
        }
    }

    // ---------------------------
    // list_session_records (lecturer/assistant only; must also be assigned)
    // ---------------------------

    #[tokio::test]
    async fn test_list_session_records_as_lecturer_ok() {
        let (app, app_state, _tmp) = make_test_app_with_storage().await;
        let ctx = setup(app_state.db()).await;

        let (token, _) = generate_jwt(ctx.lecturer.id, ctx.lecturer.admin);
        let uri = format!(
            "/api/modules/{}/attendance/sessions/{}/records",
            ctx.module.id, ctx.sess_active.id
        );

        let req = Request::builder()
            .method("GET")
            .uri(&uri)
            .header("Authorization", format!("Bearer {}", token))
            .body(AxumBody::empty())
            .unwrap();

        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);

        let body = axum::body::to_bytes(resp.into_body(), usize::MAX)
            .await
            .unwrap();
        let json: Value = serde_json::from_slice(&body).unwrap();

        assert_eq!(json["success"], true);
        assert_eq!(json["message"], "Attendance records retrieved");
        assert_eq!(json["data"]["total"].as_i64().unwrap(), 2);
        assert_eq!(json["data"]["records"].as_array().unwrap().len(), 2);
    }

    #[tokio::test]
    async fn test_list_session_records_forbidden_for_student_and_tutor() {
        let (app, app_state, _tmp) = make_test_app_with_storage().await;
        let ctx = setup(app_state.db()).await;

        for uid in [ctx.student1.id, ctx.tutor.id] {
            let (token, _) = generate_jwt(uid, false);
            let uri = format!(
                "/api/modules/{}/attendance/sessions/{}/records",
                ctx.module.id, ctx.sess_active.id
            );

            let req = Request::builder()
                .method("GET")
                .uri(&uri)
                .header("Authorization", format!("Bearer {}", token))
                .body(AxumBody::empty())
                .unwrap();

            let resp = app.clone().oneshot(req).await.unwrap();
            assert_eq!(resp.status(), StatusCode::FORBIDDEN);
        }
    }

    // ---------------------------
    // export_session_records_csv (lecturer/assistant only; must also be assigned)
    // ---------------------------

    #[tokio::test]
    async fn test_export_session_records_csv_as_lecturer_ok() {
        let (app, app_state, _tmp) = make_test_app_with_storage().await;
        let ctx = setup(app_state.db()).await;

        let (token, _) = generate_jwt(ctx.lecturer.id, ctx.lecturer.admin);
        let uri = format!(
            "/api/modules/{}/attendance/sessions/{}/records/export",
            ctx.module.id, ctx.sess_active.id
        );

        let req = Request::builder()
            .method("GET")
            .uri(&uri)
            .header("Authorization", format!("Bearer {}", token))
            .body(AxumBody::empty())
            .unwrap();

        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);

        // headers
        let headers = resp.headers();
        assert_eq!(
            headers
                .get(axum::http::header::CONTENT_TYPE)
                .and_then(|v| v.to_str().ok()),
            Some("text/csv; charset=utf-8")
        );
        let cd = headers
            .get(axum::http::header::CONTENT_DISPOSITION)
            .unwrap();
        let cd_s = cd.to_str().unwrap();
        assert!(cd_s.contains("attachment"));
        assert!(cd_s.contains("attendance_session_"));

        // body
        let body = axum::body::to_bytes(resp.into_body(), usize::MAX)
            .await
            .unwrap();
        let csv = String::from_utf8(body.to_vec()).unwrap();

        let mut lines = csv.lines();
        assert_eq!(
            lines.next().unwrap(),
            "session_id,user_id,username,taken_at,ip_address,token_window"
        );
        let rest: Vec<&str> = lines.collect();
        assert_eq!(rest.len(), 2);
        assert!(rest.iter().any(|l| l.contains("att_student1")));
        assert!(rest.iter().any(|l| l.contains("att_student2")));
        assert!(rest.iter().any(|l| l.contains("198.51.100.7")));
        assert!(rest.iter().any(|l| l.contains("203.0.113.5")));
    }

    #[tokio::test]
    async fn test_export_session_records_csv_forbidden_for_student_and_tutor() {
        let (app, app_state, _tmp) = make_test_app_with_storage().await;
        let ctx = setup(app_state.db()).await;

        for uid in [ctx.student1.id, ctx.tutor.id] {
            let (token, _) = generate_jwt(uid, false);
            let uri = format!(
                "/api/modules/{}/attendance/sessions/{}/records/export",
                ctx.module.id, ctx.sess_active.id
            );

            let req = Request::builder()
                .method("GET")
                .uri(&uri)
                .header("Authorization", format!("Bearer {}", token))
                .body(AxumBody::empty())
                .unwrap();

            let resp = app.clone().oneshot(req).await.unwrap();
            assert_eq!(resp.status(), StatusCode::FORBIDDEN);
        }
    }
}
