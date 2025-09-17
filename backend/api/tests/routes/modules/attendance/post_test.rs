#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]

    use api::{auth::generate_jwt, ws::attendance::topics::attendance_session_topic};
    use axum::{
        body::Body as AxumBody,
        extract::ConnectInfo,
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

    // ---------------------------
    // Shared setup
    // ---------------------------

    struct TestCtx {
        lecturer: UserModel,
        student: UserModel,
        tutor: UserModel,
        _forbidden: UserModel,
        module: ModuleModel,
        sess_active: SessionModel, // seeded, active
    }

    async fn setup(db: &sea_orm::DatabaseConnection) -> TestCtx {
        dotenvy::dotenv().ok();

        let module = ModuleModel::create(
            db,
            "ATT201",
            Utc::now().year(),
            Some("Attendance Post Tests"),
            8,
        )
        .await
        .expect("create module");

        let lecturer = UserModel::create(db, "post_lect", "post_lect@test.com", "password", false)
            .await
            .unwrap();
        let student = UserModel::create(
            db,
            "post_student",
            "post_student@test.com",
            "password",
            false,
        )
        .await
        .unwrap();
        let tutor = UserModel::create(db, "post_tutor", "post_tutor@test.com", "password", false)
            .await
            .unwrap();
        let _forbidden = UserModel::create(
            db,
            "post_forbidden",
            "post_forbidden@test.com",
            "password",
            false,
        )
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

        // seed an active session for mark-attendance tests
        let secret_hex = "00112233445566778899aabbccddeeff00112233445566778899aabbccddeeff";
        let sess_active = SessionModel::create(
            db,
            module.id,
            lecturer.id,
            "Seeded Active",
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
            _forbidden,
            module,
            sess_active,
        }
    }

    // Utility: attach a ConnectInfo<SocketAddr> to a Request after building.
    fn with_connect_info(mut req: Request<AxumBody>, ip: [u8; 4]) -> Request<AxumBody> {
        use std::net::{IpAddr, Ipv4Addr, SocketAddr};
        let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::from(ip)), 43210);
        req.extensions_mut().insert(ConnectInfo(addr));
        req
    }

    // ---------------------------
    // create_session
    // ---------------------------

    #[tokio::test]
    async fn test_create_session_as_lecturer_ok_minimal() {
        let (app, app_state, _tmp) = make_test_app_with_storage().await;
        let ctx = setup(app_state.db()).await;

        let (token, _) = generate_jwt(ctx.lecturer.id, ctx.lecturer.admin);
        let uri = format!("/api/modules/{}/attendance/sessions", ctx.module.id);

        let body = serde_json::json!({
            "title": "New Session",
            // rely on defaults: active=false, rotation_seconds=30, restrict=false
            "pin_to_creator_ip": true, // should capture client ip
        });

        let req = Request::builder()
            .method("POST")
            .uri(&uri)
            .header("Authorization", format!("Bearer {}", token))
            .header("Content-Type", "application/json")
            .body(AxumBody::from(body.to_string()))
            .unwrap();

        // attach client ip -> 198.51.100.9
        let req = with_connect_info(req, [198, 51, 100, 9]);

        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::CREATED);

        let bytes = axum::body::to_bytes(resp.into_body(), usize::MAX)
            .await
            .unwrap();
        let json: Value = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(json["success"], true);
        assert_eq!(json["message"], "Attendance session created");

        // sanity: fetch from DB and confirm it belongs to our module and IP was pinned
        let id = json["data"]["id"].as_i64().expect("id present");
        let sess = SessionEntity::find()
            .filter(SessionCol::Id.eq(id))
            .one(app_state.db())
            .await
            .unwrap()
            .expect("session created");
        assert_eq!(sess.module_id, ctx.module.id);
        // if pin_to_creator_ip was true, created_from_ip must match the provided ip
        assert_eq!(sess.created_from_ip.as_deref(), Some("198.51.100.9"));
    }

    #[tokio::test]
    async fn test_create_session_forbidden_for_student() {
        let (app, app_state, _tmp) = make_test_app_with_storage().await;
        let ctx = setup(app_state.db()).await;

        let (token, _) = generate_jwt(ctx.student.id, ctx.student.admin);
        let uri = format!("/api/modules/{}/attendance/sessions", ctx.module.id);
        let body = serde_json::json!({ "title": "Student Should Not Create" });

        let req = Request::builder()
            .method("POST")
            .uri(&uri)
            .header("Authorization", format!("Bearer {}", token))
            .header("Content-Type", "application/json")
            .body(AxumBody::from(body.to_string()))
            .unwrap();

        let req = with_connect_info(req, [203, 0, 113, 10]);
        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::FORBIDDEN);
    }

    // ---------------------------
    // mark_attendance + websocket broadcast
    // ---------------------------

    #[tokio::test]
    async fn test_mark_attendance_student_ok_and_broadcast_received() {
        let (app, app_state, _tmp) = make_test_app_with_storage().await;
        let ctx = setup(app_state.db()).await;

        // Create a fresh active session with no records to make count deterministic
        let fresh = SessionModel::create(
            app_state.db(),
            ctx.module.id,
            ctx.lecturer.id,
            "Mark/Broadcast",
            true,
            30,
            false,
            None,
            None,
            None, // random secret ok
        )
        .await
        .unwrap();

        // Subscribe to the topic BEFORE marking
        let topic = attendance_session_topic(fresh.id);
        let mut rx = app_state.ws().subscribe(&topic).await;

        // Student token + compute current code
        let (token, _) = generate_jwt(ctx.student.id, ctx.student.admin);
        let code = fresh.current_code(Utc::now());

        let uri = format!(
            "/api/modules/{}/attendance/sessions/{}/mark",
            ctx.module.id, fresh.id
        );
        let body = serde_json::json!({ "code": code });

        let req = Request::builder()
            .method("POST")
            .uri(&uri)
            .header("Authorization", format!("Bearer {}", token))
            .header("Content-Type", "application/json")
            .body(AxumBody::from(body.to_string()))
            .unwrap();

        let req = with_connect_info(req, [198, 51, 100, 7]);
        let resp = app.clone().oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);

        // Ensure broadcast arrived
        use tokio::time::{Duration, timeout};
        let msg = timeout(Duration::from_millis(300), rx.recv())
            .await
            .expect("broadcast not timed out")
            .expect("broadcast ok");

        let v: Value = serde_json::from_str(&msg).unwrap();
        assert_eq!(v["event"], "attendance_marked");
        assert_eq!(v["payload"]["session_id"], fresh.id);
        assert_eq!(v["payload"]["user_id"], ctx.student.id);
        assert_eq!(v["payload"]["count"], 1); // first mark for this fresh session
    }

    #[tokio::test]
    async fn test_mark_attendance_forbidden_for_lecturer() {
        let (app, app_state, _tmp) = make_test_app_with_storage().await;
        let ctx = setup(app_state.db()).await;

        let (token, _) = generate_jwt(ctx.lecturer.id, ctx.lecturer.admin);
        let uri = format!(
            "/api/modules/{}/attendance/sessions/{}/mark",
            ctx.module.id, ctx.sess_active.id
        );
        let body = serde_json::json!({ "code": "000000" });

        let req = Request::builder()
            .method("POST")
            .uri(&uri)
            .header("Authorization", format!("Bearer {}", token))
            .header("Content-Type", "application/json")
            .body(AxumBody::from(body.to_string()))
            .unwrap();

        let req = with_connect_info(req, [203, 0, 113, 20]);
        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::FORBIDDEN);

        let bytes = axum::body::to_bytes(resp.into_body(), usize::MAX)
            .await
            .unwrap();
        let json: Value = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(json["success"], false);
        assert!(json["message"].as_str().unwrap().contains("Only students"));
    }

    #[tokio::test]
    async fn test_mark_attendance_invalid_code_bad_request() {
        let (app, app_state, _tmp) = make_test_app_with_storage().await;
        let ctx = setup(app_state.db()).await;

        let (token, _) = generate_jwt(ctx.student.id, ctx.student.admin);
        let uri = format!(
            "/api/modules/{}/attendance/sessions/{}/mark",
            ctx.module.id, ctx.sess_active.id
        );

        let body = serde_json::json!({ "code": "999999" }); // intentionally wrong

        let req = Request::builder()
            .method("POST")
            .uri(&uri)
            .header("Authorization", format!("Bearer {}", token))
            .header("Content-Type", "application/json")
            .body(AxumBody::from(body.to_string()))
            .unwrap();

        let req = with_connect_info(req, [198, 51, 100, 8]);
        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::BAD_REQUEST);

        let bytes = axum::body::to_bytes(resp.into_body(), usize::MAX)
            .await
            .unwrap();
        let json: Value = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(json["success"], false);
        assert!(
            json["message"].as_str().unwrap().contains("Invalid")
                || json["message"].as_str().unwrap().contains("expired")
        );
    }

    #[tokio::test]
    async fn test_mark_attendance_ip_restricted_bad_request() {
        let (app, app_state, _tmp) = make_test_app_with_storage().await;
        let ctx = setup(app_state.db()).await;

        // Create session restricted to creator IP (198.51.100.50), then try mark from a different IP
        let restricted = SessionModel::create(
            app_state.db(),
            ctx.module.id,
            ctx.lecturer.id,
            "IP Restricted",
            true,
            30,
            true, // restrict_by_ip = true
            None,
            Some("198.51.100.50"), // created_from_ip pinned
            None,
        )
        .await
        .unwrap();

        let (token, _) = generate_jwt(ctx.student.id, ctx.student.admin);
        let code = restricted.current_code(Utc::now());
        let uri = format!(
            "/api/modules/{}/attendance/sessions/{}/mark",
            ctx.module.id, restricted.id
        );
        let body = serde_json::json!({ "code": code });

        let req = Request::builder()
            .method("POST")
            .uri(&uri)
            .header("Authorization", format!("Bearer {}", token))
            .header("Content-Type", "application/json")
            .body(AxumBody::from(body.to_string()))
            .unwrap();

        // different IP -> should be rejected by ip_permitted
        let req = with_connect_info(req, [203, 0, 113, 77]);
        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::BAD_REQUEST);

        let bytes = axum::body::to_bytes(resp.into_body(), usize::MAX)
            .await
            .unwrap();
        let json: Value = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(json["success"], false);
        assert!(
            json["message"]
                .as_str()
                .unwrap()
                .contains("IP not permitted")
        );
    }

    #[tokio::test]
    async fn test_mark_attendance_duplicate_bad_request() {
        let (app, app_state, _tmp) = make_test_app_with_storage().await;
        let ctx = setup(app_state.db()).await;

        // Fresh active session
        let sess = SessionModel::create(
            app_state.db(),
            ctx.module.id,
            ctx.lecturer.id,
            "Duplicate Mark",
            true,
            30,
            false,
            None,
            None,
            None,
        )
        .await
        .unwrap();

        let (token, _) = generate_jwt(ctx.student.id, ctx.student.admin);
        let code = sess.current_code(Utc::now());
        let uri = format!(
            "/api/modules/{}/attendance/sessions/{}/mark",
            ctx.module.id, sess.id
        );
        let body = serde_json::json!({ "code": code });

        // first mark OK
        let req1 = Request::builder()
            .method("POST")
            .uri(&uri)
            .header("Authorization", format!("Bearer {}", token))
            .header("Content-Type", "application/json")
            .body(AxumBody::from(body.to_string()))
            .unwrap();
        let req1 = with_connect_info(req1, [198, 51, 100, 10]);
        let resp1 = app.clone().oneshot(req1).await.unwrap();
        assert_eq!(resp1.status(), StatusCode::OK);

        // second mark -> BAD_REQUEST "Attendance already recorded"
        let req2 = Request::builder()
            .method("POST")
            .uri(&uri)
            .header("Authorization", format!("Bearer {}", token))
            .header("Content-Type", "application/json")
            .body(AxumBody::from(body.to_string()))
            .unwrap();
        let req2 = with_connect_info(req2, [198, 51, 100, 10]);
        let resp2 = app.oneshot(req2).await.unwrap();
        assert_eq!(resp2.status(), StatusCode::BAD_REQUEST);

        let bytes = axum::body::to_bytes(resp2.into_body(), usize::MAX)
            .await
            .unwrap();
        let json: Value = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(json["success"], false);
        assert!(
            json["message"]
                .as_str()
                .unwrap()
                .contains("already recorded")
        );
    }

    #[tokio::test]
    async fn test_mark_attendance_inactive_session_bad_request() {
        let (app, app_state, _tmp) = make_test_app_with_storage().await;
        let ctx = setup(app_state.db()).await;

        // Make an inactive session
        let sess = SessionModel::create(
            app_state.db(),
            ctx.module.id,
            ctx.lecturer.id,
            "Inactive",
            false, // inactive
            30,
            false,
            None,
            None,
            None,
        )
        .await
        .unwrap();

        let (token, _) = generate_jwt(ctx.student.id, ctx.student.admin);
        let code = sess.current_code(Utc::now());
        let uri = format!(
            "/api/modules/{}/attendance/sessions/{}/mark",
            ctx.module.id, sess.id
        );
        let body = serde_json::json!({ "code": code });

        let req = Request::builder()
            .method("POST")
            .uri(&uri)
            .header("Authorization", format!("Bearer {}", token))
            .header("Content-Type", "application/json")
            .body(AxumBody::from(body.to_string()))
            .unwrap();
        let req = with_connect_info(req, [198, 51, 100, 21]);

        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::BAD_REQUEST);

        let bytes = axum::body::to_bytes(resp.into_body(), usize::MAX)
            .await
            .unwrap();
        let json: Value = serde_json::from_slice(&bytes).unwrap();
        assert!(json["message"].as_str().unwrap().contains("not active"));
    }

    #[tokio::test]
    async fn test_mark_attendance_student_not_assigned_forbidden() {
        let (app, app_state, _tmp) = make_test_app_with_storage().await;
        let ctx = setup(app_state.db()).await;

        // Create a second module + student not assigned to ctx.module
        let _ = ModuleModel::create(
            app_state.db(),
            "ATT202",
            Utc::now().year(),
            Some("Other"),
            8,
        )
        .await
        .unwrap();
        let stranger = UserModel::create(
            app_state.db(),
            "stranger",
            "stranger@test.com",
            "pwd",
            false,
        )
        .await
        .unwrap();
        // (No role assignment for stranger on ctx.module)

        let (token, _) = generate_jwt(stranger.id, false);
        let code = ctx.sess_active.current_code(Utc::now());
        let uri = format!(
            "/api/modules/{}/attendance/sessions/{}/mark",
            ctx.module.id, ctx.sess_active.id
        );
        let body = serde_json::json!({ "code": code });

        let req = Request::builder()
            .method("POST")
            .uri(&uri)
            .header("Authorization", format!("Bearer {}", token))
            .header("Content-Type", "application/json")
            .body(AxumBody::from(body.to_string()))
            .unwrap();
        let req = with_connect_info(req, [203, 0, 113, 31]);

        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::FORBIDDEN);
    }

    #[tokio::test]
    async fn test_mark_attendance_session_not_in_module_404() {
        let (app, app_state, _tmp) = make_test_app_with_storage().await;
        let ctx = setup(app_state.db()).await;

        // Make another module and put a session there
        let other_mod = ModuleModel::create(
            app_state.db(),
            "ATT203",
            Utc::now().year(),
            Some("Mismatched"),
            8,
        )
        .await
        .unwrap();

        let sess_other = SessionModel::create(
            app_state.db(),
            other_mod.id,
            ctx.lecturer.id,
            "Elsewhere",
            true,
            30,
            false,
            None,
            None,
            None,
        )
        .await
        .unwrap();

        let (token, _) = generate_jwt(ctx.student.id, ctx.student.admin);
        let code = sess_other.current_code(Utc::now());

        // Path uses ctx.module.id but session_id from other module → handler should 404
        let uri = format!(
            "/api/modules/{}/attendance/sessions/{}/mark",
            ctx.module.id, sess_other.id
        );
        let body = serde_json::json!({ "code": code });

        let req = Request::builder()
            .method("POST")
            .uri(&uri)
            .header("Authorization", format!("Bearer {}", token))
            .header("Content-Type", "application/json")
            .body(AxumBody::from(body.to_string()))
            .unwrap();
        let req = with_connect_info(req, [198, 51, 100, 33]);

        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn test_mark_attendance_tolerance_window_boundaries() {
        let (app, app_state, _tmp) = make_test_app_with_storage().await;
        let ctx = setup(app_state.db()).await;

        // Fresh session with rotation=30s
        let sess = SessionModel::create(
            app_state.db(),
            ctx.module.id,
            ctx.lecturer.id,
            "Tolerance",
            true,
            30,
            false,
            None,
            None,
            None,
        )
        .await
        .unwrap();

        let (token, _) = generate_jwt(ctx.student.id, ctx.student.admin);

        // Pick a fixed time and compute code for window w-1 (edge inside tolerance=1)
        let now = Utc::now();
        let w = sess.window(now);
        let code_prev = sess.code_for_window(w - 1);

        // submit code for previous window with tolerance=1 -> accepted by handler (uses 1)
        let uri = format!(
            "/api/modules/{}/attendance/sessions/{}/mark",
            ctx.module.id, sess.id
        );
        let body_ok = serde_json::json!({ "code": code_prev });

        let req_ok = Request::builder()
            .method("POST")
            .uri(&uri)
            .header("Authorization", format!("Bearer {}", token))
            .header("Content-Type", "application/json")
            .body(AxumBody::from(body_ok.to_string()))
            .unwrap();
        let req_ok = with_connect_info(req_ok, [198, 51, 100, 40]);
        let resp_ok = app.clone().oneshot(req_ok).await.unwrap();
        assert_eq!(resp_ok.status(), StatusCode::OK);

        // Now try w-2 (outside tolerance) -> BAD_REQUEST
        let (_token2, _) = generate_jwt(ctx.tutor.id, ctx.tutor.admin); // use another user to avoid duplicate
        // but tutor is not allowed to mark; use new student:
        let student2 = UserModel::create(
            app_state.db(),
            "post_student2",
            "post_student2@test.com",
            "pwd",
            false,
        )
        .await
        .unwrap();
        UserModuleRoleModel::assign_user_to_module(
            app_state.db(),
            student2.id,
            ctx.module.id,
            Role::Student,
        )
        .await
        .unwrap();
        let (token2, _) = generate_jwt(student2.id, false);

        let code_too_old = sess.code_for_window(w - 2);
        let body_bad = serde_json::json!({ "code": code_too_old });

        let req_bad = Request::builder()
            .method("POST")
            .uri(&uri)
            .header("Authorization", format!("Bearer {}", token2))
            .header("Content-Type", "application/json")
            .body(AxumBody::from(body_bad.to_string()))
            .unwrap();
        let req_bad = with_connect_info(req_bad, [198, 51, 100, 41]);
        let resp_bad = app.oneshot(req_bad).await.unwrap();
        assert_eq!(resp_bad.status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn test_mark_attendance_preserves_leading_zeros_and_trims() {
        let (app, app_state, _tmp) = make_test_app_with_storage().await;
        let ctx = setup(app_state.db()).await;

        let sess = SessionModel::create(
            app_state.db(),
            ctx.module.id,
            ctx.lecturer.id,
            "Zeros",
            true,
            30,
            false,
            None,
            None,
            Some("0000000000000000000000000000000000000000000000000000000000000000"),
        )
        .await
        .unwrap();

        let (token, _) = generate_jwt(ctx.student.id, ctx.student.admin);

        let now = Utc::now();
        let code = sess.current_code(now);
        assert_eq!(code.len(), 6); // could start with zeros

        let uri = format!(
            "/api/modules/{}/attendance/sessions/{}/mark",
            ctx.module.id, sess.id
        );
        // add whitespace around code in payload -> mark() trims internally
        let body = serde_json::json!({ "code": format!("  {code}  ") });

        let req = Request::builder()
            .method("POST")
            .uri(&uri)
            .header("Authorization", format!("Bearer {}", token))
            .header("Content-Type", "application/json")
            .body(AxumBody::from(body.to_string()))
            .unwrap();
        let req = with_connect_info(req, [198, 51, 100, 60]);

        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_mark_attendance_two_students_broadcast_counts_1_then_2() {
        let (app, app_state, _tmp) = make_test_app_with_storage().await;
        let ctx = setup(app_state.db()).await;

        let sess = SessionModel::create(
            app_state.db(),
            ctx.module.id,
            ctx.lecturer.id,
            "Multi",
            true,
            30,
            false,
            None,
            None,
            None,
        )
        .await
        .unwrap();

        // second student
        let student2 = UserModel::create(
            app_state.db(),
            "post_student2b",
            "post_student2b@test.com",
            "pwd",
            false,
        )
        .await
        .unwrap();
        UserModuleRoleModel::assign_user_to_module(
            app_state.db(),
            student2.id,
            ctx.module.id,
            Role::Student,
        )
        .await
        .unwrap();

        let topic = attendance_session_topic(sess.id);
        let mut rx = app_state.ws().subscribe(&topic).await;

        // first mark
        let (t1, _) = generate_jwt(ctx.student.id, false);
        let code = sess.current_code(Utc::now());
        let uri = format!(
            "/api/modules/{}/attendance/sessions/{}/mark",
            ctx.module.id, sess.id
        );
        let body = serde_json::json!({ "code": code });

        let req1 = Request::builder()
            .method("POST")
            .uri(&uri)
            .header("Authorization", format!("Bearer {}", t1))
            .header("Content-Type", "application/json")
            .body(AxumBody::from(body.to_string()))
            .unwrap();
        let req1 = with_connect_info(req1, [198, 51, 100, 70]);
        let _ = app.clone().oneshot(req1).await.unwrap();

        use tokio::time::{Duration, timeout};
        let msg1 = timeout(Duration::from_millis(300), rx.recv())
            .await
            .unwrap()
            .unwrap();
        let v1: Value = serde_json::from_str(&msg1).unwrap();
        assert_eq!(v1["payload"]["count"], 1);

        // second mark
        let (t2, _) = generate_jwt(student2.id, false);
        let req2 = Request::builder()
            .method("POST")
            .uri(&uri)
            .header("Authorization", format!("Bearer {}", t2))
            .header("Content-Type", "application/json")
            .body(AxumBody::from(body.to_string()))
            .unwrap();
        let req2 = with_connect_info(req2, [198, 51, 100, 71]);
        let _ = app.clone().oneshot(req2).await.unwrap();

        let msg2 = timeout(Duration::from_millis(300), rx.recv())
            .await
            .unwrap()
            .unwrap();
        let v2: Value = serde_json::from_str(&msg2).unwrap();
        assert_eq!(v2["payload"]["count"], 2);
    }

    #[tokio::test]
    async fn test_mark_attendance_missing_body_is_422() {
        let (app, app_state, _tmp) = make_test_app_with_storage().await;
        let ctx = setup(app_state.db()).await;

        let (token, _) = generate_jwt(ctx.student.id, false);
        let uri = format!(
            "/api/modules/{}/attendance/sessions/{}/mark",
            ctx.module.id, ctx.sess_active.id
        );

        // Empty JSON object -> missing required "code" -> Axum JSON extractor returns 422
        let req = Request::builder()
            .method("POST")
            .uri(&uri)
            .header("Authorization", format!("Bearer {}", token))
            .header("Content-Type", "application/json")
            .body(AxumBody::from("{}"))
            .unwrap();
        let req = with_connect_info(req, [198, 51, 100, 88]);

        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::UNPROCESSABLE_ENTITY);
    }

    #[tokio::test]
    async fn test_ws_guard_auth_via_query_param_token() {
        let (app, app_state, _tmp) = make_test_app_with_storage().await;
        let ctx = setup(app_state.db()).await;

        let (token, _) = generate_jwt(ctx.lecturer.id, ctx.lecturer.admin);
        let uri = format!(
            "/ws/attendance/sessions/{}?token={}",
            ctx.sess_active.id, token
        );

        // No Authorization header on purpose; guard should accept ?token=
        let req = Request::builder()
            .method("GET")
            .uri(&uri)
            .body(AxumBody::empty())
            .unwrap();

        let resp = app.oneshot(req).await.unwrap();
        // It will pass middleware and reach the handler; since we didn't do an upgrade,
        // we only assert it isn't rejected by auth/guard:
        assert_ne!(resp.status(), StatusCode::UNAUTHORIZED);
        assert_ne!(resp.status(), StatusCode::FORBIDDEN);
        assert_ne!(resp.status(), StatusCode::NOT_FOUND);
    }

    // ---------------------------
    // (optional) websocket guard surface checks
    // ---------------------------
    // We don't perform a full WS upgrade here; instead, we can quickly sanity-check
    // the guard behavior for missing auth or unknown session.

    #[tokio::test]
    async fn test_ws_guard_unknown_session_yields_404() {
        let (app, app_state, _tmp) = make_test_app_with_storage().await;
        let ctx = setup(app_state.db()).await;
        let (token, _) = generate_jwt(ctx.lecturer.id, ctx.lecturer.admin);

        let unknown_session_id = ctx.sess_active.id + 999_999;
        let uri = format!("/ws/attendance/sessions/{}", unknown_session_id);

        let req = Request::builder()
            .method("GET")
            .uri(&uri)
            .header("Authorization", format!("Bearer {}", token))
            .body(AxumBody::empty())
            .unwrap();

        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn test_ws_guard_forbidden_for_tutor() {
        let (app, app_state, _tmp) = make_test_app_with_storage().await;
        let ctx = setup(app_state.db()).await;
        let (token, _) = generate_jwt(ctx.tutor.id, ctx.tutor.admin);

        let uri = format!("/ws/attendance/sessions/{}", ctx.sess_active.id);
        let req = Request::builder()
            .method("GET")
            .uri(&uri)
            .header("Authorization", format!("Bearer {}", token))
            .body(AxumBody::empty())
            .unwrap();

        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::FORBIDDEN);
    }
}
