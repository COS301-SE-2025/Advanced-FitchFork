#[cfg(test)]
mod tests {
    use api::{
        auth::{
            generate_jwt,
            guards::{
                Empty, allow_admin, allow_assigned_to_module, allow_assistant_lecturer,
                allow_authenticated, allow_lecturer, allow_ready_assignment, allow_student,
                allow_tutor, validate_known_ids,
            },
        },
        response::ApiResponse,
    };
    use axum::{
        Json, Router,
        body::Body,
        extract::{Path, State},
        http::{Request, StatusCode, header},
        middleware,
        middleware::Next,
        response::Response,
        routing::get,
    };
    use chrono::Utc;
    use db::models::{
        assignment::{AssignmentType, Model as AssignmentModel},
        assignment_file::{FileType, Model as FileModel},
        assignment_submission::Model as SubmissionModel,
        assignment_task::Model as TaskModel,
        module::Model as ModuleModel,
        user::Model as UserModel,
        user_module_role::{Model as UserModuleRoleModel, Role},
    };
    use dotenvy::dotenv;
    use std::collections::HashMap;
    use std::future::Future;
    use tower::ServiceExt;
    use util::state::AppState;

    use crate::helpers::app::make_test_app;

    struct TestContext {
        app_state: AppState,
        admin_token: String,
        lecturer_token: String,
        assistant_lecturer_token: String,
        tutor_token: String,
        student_token: String,
        unassigned_user_token: String,
        module_id: i64,
    }

    async fn setup() -> TestContext {
        dotenv().expect("Failed to load .env file for testing");
        let (_, app_state) = make_test_app().await;
        let db = app_state.db();

        let module = ModuleModel::create(db, "TST101", 2025, Some("Guard Test Module"), 1)
            .await
            .unwrap();
        let admin = UserModel::create(db, "guard_admin", "ga@test.com", "pw", true)
            .await
            .unwrap();
        let lecturer = UserModel::create(db, "guard_lecturer", "gl@test.com", "pw", false)
            .await
            .unwrap();
        let assistant_lecturer =
            UserModel::create(db, "guard_assistant_lecturer", "gal@test.com", "pw", false)
                .await
                .unwrap();
        let tutor = UserModel::create(db, "guard_tutor", "gt@test.com", "pw", false)
            .await
            .unwrap();
        let student = UserModel::create(db, "guard_student", "gs@test.com", "pw", false)
            .await
            .unwrap();
        let unassigned = UserModel::create(db, "guard_unassigned", "gu@test.com", "pw", false)
            .await
            .unwrap();

        UserModuleRoleModel::assign_user_to_module(db, lecturer.id, module.id, Role::Lecturer)
            .await
            .unwrap();
        UserModuleRoleModel::assign_user_to_module(
            db,
            assistant_lecturer.id,
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

        let (admin_token, _) = generate_jwt(admin.id, admin.admin);
        let (lecturer_token, _) = generate_jwt(lecturer.id, lecturer.admin);
        let (assistant_lecturer_token, _) =
            generate_jwt(assistant_lecturer.id, assistant_lecturer.admin);
        let (tutor_token, _) = generate_jwt(tutor.id, tutor.admin);
        let (student_token, _) = generate_jwt(student.id, student.admin);
        let (unassigned_user_token, _) = generate_jwt(unassigned.id, unassigned.admin);

        TestContext {
            app_state,
            module_id: module.id,
            admin_token,
            lecturer_token,
            assistant_lecturer_token,
            tutor_token,
            student_token,
            unassigned_user_token,
        }
    }

    fn create_test_router<G, Fut>(app_state: AppState, guard: G) -> Router
    where
        G: Clone + Send + Sync + 'static,
        Fut: Future<Output = Result<Response, (StatusCode, Json<ApiResponse<Empty>>)>>
            + Send
            + 'static,
        G: Fn(State<AppState>, Path<HashMap<String, String>>, Request<Body>, Next) -> Fut,
    {
        async fn test_handler() -> &'static str {
            "OK"
        }

        Router::new()
            .route(
                "/test/{module_id}",
                get(test_handler)
                    .route_layer(middleware::from_fn_with_state(app_state.clone(), guard)),
            )
            .layer(middleware::from_fn(allow_authenticated))
            .with_state(app_state)
    }

    fn build_request(uri: &str, token: Option<&str>) -> Request<Body> {
        let mut builder = Request::builder().uri(uri);
        if let Some(t) = token {
            builder = builder.header(header::AUTHORIZATION, format!("Bearer {}", t));
        }
        builder.body(Body::empty()).unwrap()
    }

    mod test_allow_admin {
        use super::*;
        async fn admin_guard_adapter(
            _s: State<AppState>,
            _p: Path<HashMap<String, String>>,
            req: Request<Body>,
            next: Next,
        ) -> Result<Response, (StatusCode, Json<ApiResponse<Empty>>)> {
            allow_admin(req, next).await
        }

        #[tokio::test]
        async fn succeeds_for_admin() {
            let ctx = setup().await;
            let app = create_test_router(ctx.app_state.clone(), admin_guard_adapter);
            let req = build_request(&format!("/test/{}", ctx.module_id), Some(&ctx.admin_token));
            let res = app.oneshot(req).await.unwrap();
            assert_eq!(res.status(), StatusCode::OK);
        }

        #[tokio::test]
        async fn fails_for_non_admin() {
            let ctx = setup().await;
            let app = create_test_router(ctx.app_state.clone(), admin_guard_adapter);
            let req = build_request(
                &format!("/test/{}", ctx.module_id),
                Some(&ctx.lecturer_token),
            );
            let res = app.oneshot(req).await.unwrap();
            assert_eq!(res.status(), StatusCode::FORBIDDEN);
        }

        #[tokio::test]
        async fn fails_for_unauthenticated() {
            let ctx = setup().await;
            let app = create_test_router(ctx.app_state.clone(), admin_guard_adapter);
            let req = build_request(&format!("/test/{}", ctx.module_id), None);
            let res = app.oneshot(req).await.unwrap();
            assert_eq!(res.status(), StatusCode::UNAUTHORIZED);
        }
    }

    mod test_allow_lecturer {
        use super::*;

        #[tokio::test]
        async fn succeeds_for_lecturer() {
            let ctx = setup().await;
            let app = create_test_router(ctx.app_state.clone(), allow_lecturer);
            let req = build_request(
                &format!("/test/{}", ctx.module_id),
                Some(&ctx.lecturer_token),
            );
            let res = app.oneshot(req).await.unwrap();
            assert_eq!(res.status(), StatusCode::OK);
        }

        #[tokio::test]
        async fn succeeds_for_admin() {
            let ctx = setup().await;
            let app = create_test_router(ctx.app_state.clone(), allow_lecturer);
            let req = build_request(&format!("/test/{}", ctx.module_id), Some(&ctx.admin_token));
            let res = app.oneshot(req).await.unwrap();
            assert_eq!(res.status(), StatusCode::OK);
        }

        #[tokio::test]
        async fn fails_for_tutor() {
            let ctx = setup().await;
            let app = create_test_router(ctx.app_state.clone(), allow_lecturer);
            let req = build_request(&format!("/test/{}", ctx.module_id), Some(&ctx.tutor_token));
            let res = app.oneshot(req).await.unwrap();
            assert_eq!(res.status(), StatusCode::FORBIDDEN);
        }
        #[tokio::test]
        async fn fails_for_student() {
            let ctx = setup().await;
            let app = create_test_router(ctx.app_state.clone(), allow_lecturer);
            let req = build_request(
                &format!("/test/{}", ctx.module_id),
                Some(&ctx.student_token),
            );
            let res = app.oneshot(req).await.unwrap();
            assert_eq!(res.status(), StatusCode::FORBIDDEN);
        }
    }

    mod test_allow_assistant_lecturer {
        use super::*;

        #[tokio::test]
        async fn succeeds_for_assistant_lecturer() {
            let ctx = setup().await;
            let app = create_test_router(ctx.app_state.clone(), allow_assistant_lecturer);
            let req = build_request(
                &format!("/test/{}", ctx.module_id),
                Some(&ctx.assistant_lecturer_token),
            );
            let res = app.oneshot(req).await.unwrap();
            assert_eq!(res.status(), StatusCode::OK);
        }

        #[tokio::test]
        async fn succeeds_for_admin() {
            let ctx = setup().await;
            let app = create_test_router(ctx.app_state.clone(), allow_assistant_lecturer);
            let req = build_request(&format!("/test/{}", ctx.module_id), Some(&ctx.admin_token));
            let res = app.oneshot(req).await.unwrap();
            assert_eq!(res.status(), StatusCode::OK);
        }

        #[tokio::test]
        async fn succeeds_for_lecturer() {
            let ctx = setup().await;
            let app = create_test_router(ctx.app_state.clone(), allow_assistant_lecturer);
            let req = build_request(
                &format!("/test/{}", ctx.module_id),
                Some(&ctx.lecturer_token),
            );
            let res = app.oneshot(req).await.unwrap();
            assert_eq!(res.status(), StatusCode::OK);
        }

        #[tokio::test]
        async fn fails_for_tutor() {
            let ctx = setup().await;
            let app = create_test_router(ctx.app_state.clone(), allow_assistant_lecturer);
            let req = build_request(&format!("/test/{}", ctx.module_id), Some(&ctx.tutor_token));
            let res = app.oneshot(req).await.unwrap();
            assert_eq!(res.status(), StatusCode::FORBIDDEN);
        }

        #[tokio::test]
        async fn fails_for_student() {
            let ctx = setup().await;
            let app = create_test_router(ctx.app_state.clone(), allow_assistant_lecturer);
            let req = build_request(
                &format!("/test/{}", ctx.module_id),
                Some(&ctx.student_token),
            );
            let res = app.oneshot(req).await.unwrap();
            assert_eq!(res.status(), StatusCode::FORBIDDEN);
        }
    }

    mod test_allow_tutor {
        use super::*;

        #[tokio::test]
        async fn succeeds_for_tutor() {
            let ctx = setup().await;
            let app = create_test_router(ctx.app_state.clone(), allow_tutor);
            let req = build_request(&format!("/test/{}", ctx.module_id), Some(&ctx.tutor_token));
            let res = app.oneshot(req).await.unwrap();
            assert_eq!(res.status(), StatusCode::OK);
        }

        #[tokio::test]
        async fn succeeds_for_admin() {
            let ctx = setup().await;
            let app = create_test_router(ctx.app_state.clone(), allow_tutor);
            let req = build_request(&format!("/test/{}", ctx.module_id), Some(&ctx.admin_token));
            let res = app.oneshot(req).await.unwrap();
            assert_eq!(res.status(), StatusCode::OK);
        }

        #[tokio::test]
        async fn succeeds_for_lecturer() {
            let ctx = setup().await;
            let app = create_test_router(ctx.app_state.clone(), allow_tutor);
            let req = build_request(
                &format!("/test/{}", ctx.module_id),
                Some(&ctx.lecturer_token),
            );
            let res = app.oneshot(req).await.unwrap();
            assert_eq!(res.status(), StatusCode::OK);
        }
        #[tokio::test]
        async fn fails_for_student() {
            let ctx = setup().await;
            let app = create_test_router(ctx.app_state.clone(), allow_tutor);
            let req = build_request(
                &format!("/test/{}", ctx.module_id),
                Some(&ctx.student_token),
            );
            let res = app.oneshot(req).await.unwrap();
            assert_eq!(res.status(), StatusCode::FORBIDDEN);
        }
    }

    mod test_allow_student {
        use super::*;

        #[tokio::test]
        async fn succeeds_for_student() {
            let ctx = setup().await;
            let app = create_test_router(ctx.app_state.clone(), allow_student);
            let req = build_request(
                &format!("/test/{}", ctx.module_id),
                Some(&ctx.student_token),
            );
            let res = app.oneshot(req).await.unwrap();
            assert_eq!(res.status(), StatusCode::OK);
        }

        #[tokio::test]
        async fn succeeds_for_admin() {
            let ctx = setup().await;
            let app = create_test_router(ctx.app_state.clone(), allow_student);
            let req = build_request(&format!("/test/{}", ctx.module_id), Some(&ctx.admin_token));
            let res = app.oneshot(req).await.unwrap();
            assert_eq!(res.status(), StatusCode::OK);
        }

        #[tokio::test]
        async fn succeeds_for_tutor() {
            let ctx = setup().await;
            let app = create_test_router(ctx.app_state.clone(), allow_student);
            let req = build_request(&format!("/test/{}", ctx.module_id), Some(&ctx.tutor_token));
            let res = app.oneshot(req).await.unwrap();
            assert_eq!(res.status(), StatusCode::OK);
        }
        #[tokio::test]
        async fn succeeds_for_lecturer() {
            let ctx = setup().await;
            let app = create_test_router(ctx.app_state.clone(), allow_student);
            let req = build_request(
                &format!("/test/{}", ctx.module_id),
                Some(&ctx.lecturer_token),
            );
            let res = app.oneshot(req).await.unwrap();
            assert_eq!(res.status(), StatusCode::OK);
        }
    }

    mod test_allow_lecturer_or_assistant_lecturer {
        use super::*;

        #[tokio::test]
        async fn succeeds_for_lecturer() {
            let ctx = setup().await;
            let app = create_test_router(ctx.app_state.clone(), allow_assistant_lecturer);
            let req = build_request(
                &format!("/test/{}", ctx.module_id),
                Some(&ctx.lecturer_token),
            );
            let res = app.oneshot(req).await.unwrap();
            assert_eq!(res.status(), StatusCode::OK);
        }

        #[tokio::test]
        async fn succeeds_for_assistant_lecturer() {
            let ctx = setup().await;
            let app = create_test_router(ctx.app_state.clone(), allow_assistant_lecturer);
            let req = build_request(
                &format!("/test/{}", ctx.module_id),
                Some(&ctx.assistant_lecturer_token),
            );
            let res = app.oneshot(req).await.unwrap();
            assert_eq!(res.status(), StatusCode::OK);
        }

        #[tokio::test]
        async fn succeeds_for_admin() {
            let ctx = setup().await;
            let app = create_test_router(ctx.app_state.clone(), allow_assistant_lecturer);
            let req = build_request(&format!("/test/{}", ctx.module_id), Some(&ctx.admin_token));
            let res = app.oneshot(req).await.unwrap();
            assert_eq!(res.status(), StatusCode::OK);
        }

        #[tokio::test]
        async fn fails_for_tutor() {
            let ctx = setup().await;
            let app = create_test_router(ctx.app_state.clone(), allow_assistant_lecturer);
            let req = build_request(&format!("/test/{}", ctx.module_id), Some(&ctx.tutor_token));
            let res = app.oneshot(req).await.unwrap();
            assert_eq!(res.status(), StatusCode::FORBIDDEN);
        }

        #[tokio::test]
        async fn fails_for_student() {
            let ctx = setup().await;
            let app = create_test_router(ctx.app_state.clone(), allow_assistant_lecturer);
            let req = build_request(
                &format!("/test/{}", ctx.module_id),
                Some(&ctx.student_token),
            );
            let res = app.oneshot(req).await.unwrap();
            assert_eq!(res.status(), StatusCode::FORBIDDEN);
        }
    }

    mod test_allow_lecturer_or_tutor {
        use super::*;

        #[tokio::test]
        async fn succeeds_for_lecturer() {
            let ctx = setup().await;
            let app = create_test_router(ctx.app_state.clone(), allow_tutor);
            let req = build_request(
                &format!("/test/{}", ctx.module_id),
                Some(&ctx.lecturer_token),
            );
            let res = app.oneshot(req).await.unwrap();
            assert_eq!(res.status(), StatusCode::OK);
        }

        #[tokio::test]
        async fn succeeds_for_assistant_lecturer() {
            let ctx = setup().await;
            let app = create_test_router(ctx.app_state.clone(), allow_tutor);
            let req = build_request(
                &format!("/test/{}", ctx.module_id),
                Some(&ctx.assistant_lecturer_token),
            );
            let res = app.oneshot(req).await.unwrap();
            assert_eq!(res.status(), StatusCode::OK);
        }

        #[tokio::test]
        async fn succeeds_for_tutor() {
            let ctx = setup().await;
            let app = create_test_router(ctx.app_state.clone(), allow_tutor);
            let req = build_request(&format!("/test/{}", ctx.module_id), Some(&ctx.tutor_token));
            let res = app.oneshot(req).await.unwrap();
            assert_eq!(res.status(), StatusCode::OK);
        }

        #[tokio::test]
        async fn succeeds_for_admin() {
            let ctx = setup().await;
            let app = create_test_router(ctx.app_state.clone(), allow_tutor);
            let req = build_request(&format!("/test/{}", ctx.module_id), Some(&ctx.admin_token));
            let res = app.oneshot(req).await.unwrap();
            assert_eq!(res.status(), StatusCode::OK);
        }

        #[tokio::test]
        async fn fails_for_student() {
            let ctx = setup().await;
            let app = create_test_router(ctx.app_state.clone(), allow_tutor);
            let req = build_request(
                &format!("/test/{}", ctx.module_id),
                Some(&ctx.student_token),
            );
            let res = app.oneshot(req).await.unwrap();
            assert_eq!(res.status(), StatusCode::FORBIDDEN);
        }
    }

    mod test_allow_assigned_to_module {
        use super::*;

        async fn guard_adapter(
            s: State<AppState>,
            p: Path<HashMap<String, String>>,
            req: Request<Body>,
            next: Next,
        ) -> Result<Response, (StatusCode, Json<ApiResponse<Empty>>)> {
            allow_assigned_to_module(s, p, req, next).await
        }

        #[tokio::test]
        async fn succeeds_for_lecturer() {
            let ctx = setup().await;
            let app = create_test_router(ctx.app_state.clone(), guard_adapter);
            let req = build_request(
                &format!("/test/{}", ctx.module_id),
                Some(&ctx.lecturer_token),
            );
            let res = app.oneshot(req).await.unwrap();
            assert_eq!(res.status(), StatusCode::OK);
        }

        #[tokio::test]
        async fn succeeds_for_assistant_lecturer() {
            let ctx = setup().await;
            let app = create_test_router(ctx.app_state.clone(), guard_adapter);
            let req = build_request(
                &format!("/test/{}", ctx.module_id),
                Some(&ctx.assistant_lecturer_token),
            );
            let res = app.oneshot(req).await.unwrap();
            assert_eq!(res.status(), StatusCode::OK);
        }

        #[tokio::test]
        async fn succeeds_for_tutor() {
            let ctx = setup().await;
            let app = create_test_router(ctx.app_state.clone(), guard_adapter);
            let req = build_request(&format!("/test/{}", ctx.module_id), Some(&ctx.tutor_token));
            let res = app.oneshot(req).await.unwrap();
            assert_eq!(res.status(), StatusCode::OK);
        }

        #[tokio::test]
        async fn succeeds_for_student() {
            let ctx = setup().await;
            let app = create_test_router(ctx.app_state.clone(), guard_adapter);
            let req = build_request(
                &format!("/test/{}", ctx.module_id),
                Some(&ctx.student_token),
            );
            let res = app.oneshot(req).await.unwrap();
            assert_eq!(res.status(), StatusCode::OK);
        }

        #[tokio::test]
        async fn succeeds_for_admin() {
            let ctx = setup().await;
            let app = create_test_router(ctx.app_state.clone(), guard_adapter);
            let req = build_request(&format!("/test/{}", ctx.module_id), Some(&ctx.admin_token));
            let res = app.oneshot(req).await.unwrap();
            assert_eq!(res.status(), StatusCode::OK);
        }

        #[tokio::test]
        async fn fails_for_unassigned_user() {
            let ctx = setup().await;
            let app = create_test_router(ctx.app_state.clone(), guard_adapter);
            let req = build_request(
                &format!("/test/{}", ctx.module_id),
                Some(&ctx.unassigned_user_token),
            );
            let res = app.oneshot(req).await.unwrap();
            assert_eq!(res.status(), StatusCode::FORBIDDEN);
        }
    }

    mod test_allow_ready_assignment {
        use super::*;
        use sea_orm::{ActiveModelTrait, Set};

        fn create_assignment_router<G, Fut>(app_state: AppState, guard: G) -> Router
        where
            G: Fn(State<AppState>, Path<HashMap<String, String>>, Request<Body>, Next) -> Fut
                + Clone
                + Send
                + Sync
                + 'static,
            Fut: Future<Output = Result<Response, (StatusCode, Json<ApiResponse<Empty>>)>>
                + Send
                + 'static,
        {
            async fn test_handler() -> &'static str {
                "OK"
            }

            Router::new()
                .route(
                    "/modules/{module_id}/assignments/{assignment_id}",
                    get(test_handler)
                        .route_layer(middleware::from_fn_with_state(app_state.clone(), guard)),
                )
                .layer(middleware::from_fn(allow_authenticated))
                .with_state(app_state)
        }

        #[tokio::test]
        async fn forbids_when_assignment_in_setup() {
            let ctx = setup().await;
            let db = ctx.app_state.db();

            let assignment = AssignmentModel::create(
                db,
                ctx.module_id,
                "RS-SETUP",
                None,
                AssignmentType::Assignment,
                Utc::now(),
                Utc::now(),
            )
            .await
            .unwrap();
            let mut am: db::models::assignment::ActiveModel = assignment.clone().into();
            am.status = Set(db::models::assignment::Status::Setup);
            am.update(db).await.unwrap();

            let app = create_assignment_router(ctx.app_state.clone(), allow_ready_assignment);
            let uri = format!("/modules/{}/assignments/{}", ctx.module_id, assignment.id);
            let res = app
                .oneshot(build_request(&uri, Some(&ctx.admin_token)))
                .await
                .unwrap();
            assert_eq!(res.status(), StatusCode::FORBIDDEN);
        }

        #[tokio::test]
        async fn allows_when_assignment_ready() {
            let ctx = setup().await;
            let db = ctx.app_state.db();

            let assignment = AssignmentModel::create(
                db,
                ctx.module_id,
                "RS-READY",
                None,
                AssignmentType::Assignment,
                Utc::now(),
                Utc::now(),
            )
            .await
            .unwrap();
            let mut am: db::models::assignment::ActiveModel = assignment.clone().into();
            am.status = Set(db::models::assignment::Status::Ready);
            am.update(db).await.unwrap();

            let app = create_assignment_router(ctx.app_state.clone(), allow_ready_assignment);
            let uri = format!("/modules/{}/assignments/{}", ctx.module_id, assignment.id);
            let res = app
                .oneshot(build_request(&uri, Some(&ctx.admin_token)))
                .await
                .unwrap();
            assert_eq!(res.status(), StatusCode::OK);
        }

        #[tokio::test]
        async fn allows_when_assignment_open_or_closed() {
            let ctx = setup().await;
            let db = ctx.app_state.db();

            // Open
            let assignment_open = AssignmentModel::create(
                db,
                ctx.module_id,
                "RS-OPEN",
                None,
                AssignmentType::Assignment,
                Utc::now(),
                Utc::now(),
            )
            .await
            .unwrap();
            let mut am_open: db::models::assignment::ActiveModel = assignment_open.clone().into();
            am_open.status = Set(db::models::assignment::Status::Open);
            am_open.update(db).await.unwrap();

            let app = create_assignment_router(ctx.app_state.clone(), allow_ready_assignment);
            let uri_open = format!(
                "/modules/{}/assignments/{}",
                ctx.module_id, assignment_open.id
            );
            let res_open = app
                .clone()
                .oneshot(build_request(&uri_open, Some(&ctx.admin_token)))
                .await
                .unwrap();
            assert_eq!(res_open.status(), StatusCode::OK);

            // Closed
            let assignment_closed = AssignmentModel::create(
                db,
                ctx.module_id,
                "RS-CLOSED",
                None,
                AssignmentType::Assignment,
                Utc::now(),
                Utc::now(),
            )
            .await
            .unwrap();
            let mut am_closed: db::models::assignment::ActiveModel =
                assignment_closed.clone().into();
            am_closed.status = Set(db::models::assignment::Status::Closed);
            am_closed.update(db).await.unwrap();

            let uri_closed = format!(
                "/modules/{}/assignments/{}",
                ctx.module_id, assignment_closed.id
            );
            let res_closed = app
                .oneshot(build_request(&uri_closed, Some(&ctx.admin_token)))
                .await
                .unwrap();
            assert_eq!(res_closed.status(), StatusCode::OK);
        }

        #[tokio::test]
        async fn returns_not_found_when_assignment_missing() {
            let ctx = setup().await;
            let non_existent_assignment_id = 99999;
            let app = create_assignment_router(ctx.app_state.clone(), allow_ready_assignment);
            let uri = format!(
                "/modules/{}/assignments/{}",
                ctx.module_id, non_existent_assignment_id
            );
            let res = app
                .oneshot(build_request(&uri, Some(&ctx.admin_token)))
                .await
                .unwrap();
            assert_eq!(res.status(), StatusCode::NOT_FOUND);
        }
    }

    mod test_multi_param {
        use super::*;

        fn create_multi_param_router<G, Fut>(app_state: AppState, guard: G) -> Router
        where
            G: Fn(State<AppState>, Path<HashMap<String, String>>, Request<Body>, Next) -> Fut
                + Clone
                + Send
                + Sync
                + 'static,
            Fut: Future<Output = Result<Response, (StatusCode, Json<ApiResponse<Empty>>)>>
                + Send
                + 'static,
        {
            async fn test_handler() -> &'static str {
                "OK"
            }

            Router::new()
                .route(
                    "/test/{module_id}/other/{other_id}",
                    get(test_handler)
                        .route_layer(middleware::from_fn_with_state(app_state.clone(), guard)),
                )
                .layer(middleware::from_fn(allow_authenticated))
                .with_state(app_state)
        }

        #[tokio::test]
        async fn succeeds_for_lecturer_with_multi_param() {
            let ctx = setup().await;
            let app = create_multi_param_router(ctx.app_state.clone(), allow_lecturer);
            let req = build_request(
                &format!("/test/{}/other/42", ctx.module_id),
                Some(&ctx.lecturer_token),
            );
            let res = app.oneshot(req).await.unwrap();
            assert_eq!(res.status(), StatusCode::OK);
        }

        #[tokio::test]
        async fn fails_for_unassigned_user_with_multi_param() {
            let ctx = setup().await;
            let app = create_multi_param_router(ctx.app_state.clone(), allow_lecturer);
            let req = build_request(
                &format!("/test/{}/other/42", ctx.module_id),
                Some(&ctx.unassigned_user_token),
            );
            let res = app.oneshot(req).await.unwrap();
            assert_eq!(res.status(), StatusCode::FORBIDDEN);
        }

        #[tokio::test]
        async fn succeeds_for_admin_with_multi_param() {
            let ctx = setup().await;
            let app = create_multi_param_router(ctx.app_state.clone(), allow_lecturer);
            let req = build_request(
                &format!("/test/{}/other/42", ctx.module_id),
                Some(&ctx.admin_token),
            );
            let res = app.oneshot(req).await.unwrap();
            assert_eq!(res.status(), StatusCode::OK);
        }
    }

    mod test_validate_known_ids {
        use super::*;

        fn create_router(app_state: AppState, path: &'static str) -> Router {
            async fn handler() -> &'static str {
                "OK"
            }

            Router::new()
                .route(path, get(handler))
                .layer(middleware::from_fn(allow_authenticated))
                .with_state(app_state.clone())
                .route_layer(middleware::from_fn_with_state(
                    app_state,
                    validate_known_ids,
                ))
        }

        #[tokio::test]
        async fn succeeds_when_module_exists() {
            let ctx = setup().await;
            let app = create_router(ctx.app_state.clone(), "/test/{module_id}");
            let uri = format!("/test/{}", ctx.module_id);
            let res = app
                .oneshot(build_request(&uri, Some(&ctx.admin_token)))
                .await
                .unwrap();
            assert_eq!(res.status(), StatusCode::OK);
        }

        #[tokio::test]
        async fn fails_with_not_found_when_module_missing() {
            let ctx = setup().await;
            let app = create_router(ctx.app_state.clone(), "/test/{module_id}");
            let res = app
                .oneshot(build_request("/test/99999", Some(&ctx.admin_token)))
                .await
                .unwrap();
            assert_eq!(res.status(), StatusCode::NOT_FOUND);
        }

        #[tokio::test]
        async fn fails_with_bad_request_on_parse_error() {
            let ctx = setup().await;
            let app = create_router(ctx.app_state.clone(), "/test/{module_id}");
            let res = app
                .oneshot(build_request("/test/abc", Some(&ctx.admin_token)))
                .await
                .unwrap();
            assert_eq!(res.status(), StatusCode::BAD_REQUEST);
        }

        #[tokio::test]
        async fn fails_with_bad_request_on_unknown_param() {
            let ctx = setup().await;
            let app = create_router(ctx.app_state.clone(), "/test/{unknown_id}");
            let res = app
                .oneshot(build_request("/test/123", Some(&ctx.admin_token)))
                .await
                .unwrap();
            assert_eq!(res.status(), StatusCode::BAD_REQUEST);
        }

        #[tokio::test]
        async fn fails_when_assignment_not_in_module() {
            let ctx = setup().await;
            let other = ModuleModel::create(ctx.app_state.db(), "OTHER", 2025, None, 1)
                .await
                .unwrap();
            let assignment = AssignmentModel::create(
                ctx.app_state.db(),
                other.id,
                "A1",
                None,
                AssignmentType::Assignment,
                Utc::now(),
                Utc::now(),
            )
            .await
            .unwrap();
            let path = "/modules/{module_id}/assignments/{assignment_id}";
            let app = create_router(ctx.app_state.clone(), path);
            let uri = format!("/modules/{}/assignments/{}", ctx.module_id, assignment.id);
            let res = app
                .oneshot(build_request(&uri, Some(&ctx.admin_token)))
                .await
                .unwrap();
            assert_eq!(res.status(), StatusCode::NOT_FOUND);
        }

        #[tokio::test]
        async fn succeeds_for_assignment_in_module() {
            let ctx = setup().await;
            let assignment = AssignmentModel::create(
                ctx.app_state.db(),
                ctx.module_id,
                "A2",
                None,
                AssignmentType::Assignment,
                Utc::now(),
                Utc::now(),
            )
            .await
            .unwrap();
            let path = "/modules/{module_id}/assignments/{assignment_id}";
            let app = create_router(ctx.app_state.clone(), path);
            let uri = format!("/modules/{}/assignments/{}", ctx.module_id, assignment.id);
            let res = app
                .oneshot(build_request(&uri, Some(&ctx.admin_token)))
                .await
                .unwrap();
            assert_eq!(res.status(), StatusCode::OK);
        }

        #[tokio::test]
        async fn succeeds_for_task_hierarchy() {
            let ctx = setup().await;
            let assignment = AssignmentModel::create(
                ctx.app_state.db(),
                ctx.module_id,
                "A3",
                None,
                AssignmentType::Assignment,
                Utc::now(),
                Utc::now(),
            )
            .await
            .unwrap();
            let task = TaskModel::create(ctx.app_state.db(), assignment.id, 1, "T1", "echo", false)
                .await
                .unwrap();
            let path = "/modules/{module_id}/assignments/{assignment_id}/tasks/{task_id}";
            let app = create_router(ctx.app_state.clone(), path);
            let uri = format!(
                "/modules/{}/assignments/{}/tasks/{}",
                ctx.module_id, assignment.id, task.id
            );
            let res = app
                .oneshot(build_request(&uri, Some(&ctx.admin_token)))
                .await
                .unwrap();
            assert_eq!(res.status(), StatusCode::OK);
        }

        #[tokio::test]
        async fn fails_for_task_not_in_assignment() {
            let ctx = setup().await;
            let assignment1 = AssignmentModel::create(
                ctx.app_state.db(),
                ctx.module_id,
                "A4",
                None,
                AssignmentType::Assignment,
                Utc::now(),
                Utc::now(),
            )
            .await
            .unwrap();
            let assignment2 = AssignmentModel::create(
                ctx.app_state.db(),
                ctx.module_id,
                "A5",
                None,
                AssignmentType::Assignment,
                Utc::now(),
                Utc::now(),
            )
            .await
            .unwrap();
            let task =
                TaskModel::create(ctx.app_state.db(), assignment2.id, 1, "T2", "echo", false)
                    .await
                    .unwrap();
            let path = "/modules/{module_id}/assignments/{assignment_id}/tasks/{task_id}";
            let app = create_router(ctx.app_state.clone(), path);
            let uri = format!(
                "/modules/{}/assignments/{}/tasks/{}",
                ctx.module_id, assignment1.id, task.id
            );
            let res = app
                .oneshot(build_request(&uri, Some(&ctx.admin_token)))
                .await
                .unwrap();
            assert_eq!(res.status(), StatusCode::NOT_FOUND);
        }

        #[tokio::test]
        async fn succeeds_for_submission_and_file_hierarchy() {
            let ctx = setup().await;
            let assignment = AssignmentModel::create(
                ctx.app_state.db(),
                ctx.module_id,
                "A6",
                None,
                AssignmentType::Assignment,
                Utc::now(),
                Utc::now(),
            )
            .await
            .unwrap();
            let submission = SubmissionModel::save_file(
                ctx.app_state.db(),
                assignment.id,
                1,
                1,
                10.0,
                10.0,
                false,
                "f.txt",
                "hash123#",
                b"test",
            )
            .await
            .unwrap();
            let file = FileModel::save_file(
                ctx.app_state.db(),
                assignment.id,
                ctx.module_id,
                FileType::Spec,
                "f.txt",
                b"test",
            )
            .await
            .unwrap();

            let sub_path =
                "/modules/{module_id}/assignments/{assignment_id}/submissions/{submission_id}";
            let sub_app = create_router(ctx.app_state.clone(), sub_path);
            let sub_uri = format!(
                "/modules/{}/assignments/{}/submissions/{}",
                ctx.module_id, assignment.id, submission.id
            );
            assert_eq!(
                sub_app
                    .oneshot(build_request(&sub_uri, Some(&ctx.admin_token)))
                    .await
                    .unwrap()
                    .status(),
                StatusCode::OK
            );

            let file_path = "/modules/{module_id}/assignments/{assignment_id}/files/{file_id}";
            let file_app = create_router(ctx.app_state.clone(), file_path);
            let file_uri = format!(
                "/modules/{}/assignments/{}/files/{}",
                ctx.module_id, assignment.id, file.id
            );
            assert_eq!(
                file_app
                    .oneshot(build_request(&file_uri, Some(&ctx.admin_token)))
                    .await
                    .unwrap()
                    .status(),
                StatusCode::OK
            );
        }

        #[tokio::test]
        async fn fails_for_submission_not_in_assignment() {
            let ctx = setup().await;
            let other_assignment = AssignmentModel::create(
                ctx.app_state.db(),
                ctx.module_id,
                "A7",
                None,
                AssignmentType::Assignment,
                Utc::now(),
                Utc::now(),
            )
            .await
            .unwrap();
            let submission = SubmissionModel::save_file(
                ctx.app_state.db(),
                other_assignment.id,
                1,
                1,
                10.0,
                10.0,
                false,
                "f.txt",
                "hash123#",
                b"test",
            )
            .await
            .unwrap();
            let path =
                "/modules/{module_id}/assignments/{assignment_id}/submissions/{submission_id}";
            let app = create_router(ctx.app_state.clone(), path);
            let uri = format!(
                "/modules/{}/assignments/{}/submissions/{}",
                ctx.module_id, 0, submission.id
            );
            let res = app
                .oneshot(build_request(&uri, Some(&ctx.admin_token)))
                .await
                .unwrap();
            assert_eq!(res.status(), StatusCode::NOT_FOUND);
        }
    }
}
