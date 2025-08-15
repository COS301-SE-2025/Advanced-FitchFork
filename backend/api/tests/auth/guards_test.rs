#[cfg(test)]
mod tests {
    use axum::{
        extract::Path,
        routing::get,
        Router,
        http::{header, Request, StatusCode},
        middleware,
        body::Body,
        response::Response,
        middleware::Next,
        Json,
    };
    use db::{
        models::{
            module::Model as ModuleModel,
            user_module_role::{Model as UserModuleRoleModel, Role},
            assignment::{Model as AssignmentModel, AssignmentType},
            assignment_task::Model as TaskModel,
            assignment_submission::Model as SubmissionModel,
            assignment_file::{Model as FileModel, FileType},
        },
        repositories::user_repository::UserRepository,
    };
    use api::{
        auth::{
            generate_jwt,
            guards::{
                require_authenticated, require_admin, require_lecturer, require_assistant_lecturer, require_tutor, require_student,
                require_lecturer_or_assistant_lecturer, require_lecturer_or_tutor, require_assigned_to_module, require_ready_assignment,
                Empty, validate_known_ids,
            }
        },
        response::ApiResponse
    };
    use services::{
        service::Service,
        user_service::{UserService, CreateUser}
    };
    use dotenvy::dotenv;
    use tower::ServiceExt;
    use std::future::Future;
    use std::collections::HashMap;
    use chrono::Utc;
    use services::util_service::UtilService;
    use serial_test::serial;

    struct TestContext {
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
        UtilService::clean_db().await.expect("Failed to clean db"); // TODO: Make this better IDK
        let db = db::get_connection().await;
        let service = UserService::new(UserRepository::new(db.clone()));

        let module = ModuleModel::create(db, "TST101", 2025, Some("Guard Test Module"), 1).await.unwrap();
        let admin = service.create(CreateUser{ username: "guard_admin".to_string(), email: "ga@test.com".to_string(), password: "pw".to_string(), admin: true }).await.unwrap();
        let lecturer = service.create(CreateUser{ username: "guard_lecturer".to_string(), email: "gl@test.com".to_string(), password: "pw".to_string(), admin: false }).await.unwrap();
        let assistant_lecturer = service.create(CreateUser{ username: "guard_assistant_lecturer".to_string(), email: "gal@test.com".to_string(), password: "pw".to_string(), admin: false }).await.unwrap();
        let tutor = service.create(CreateUser{ username: "guard_tutor".to_string(), email: "gt@test.com".to_string(), password: "pw".to_string(), admin: false }).await.unwrap();
        let student = service.create(CreateUser{ username: "guard_student".to_string(), email: "gs@test.com".to_string(), password: "pw".to_string(), admin: false }).await.unwrap();
        let unassigned = service.create(CreateUser{ username: "guard_unassigned".to_string(), email: "gu@test.com".to_string(), password: "pw".to_string(), admin: false }).await.unwrap();

        UserModuleRoleModel::assign_user_to_module(db, lecturer.id, module.id, Role::Lecturer).await.unwrap();
        UserModuleRoleModel::assign_user_to_module(db, assistant_lecturer.id, module.id, Role::AssistantLecturer).await.unwrap();
        UserModuleRoleModel::assign_user_to_module(db, tutor.id, module.id, Role::Tutor).await.unwrap();
        UserModuleRoleModel::assign_user_to_module(db, student.id, module.id, Role::Student).await.unwrap();

        let (admin_token, _) = generate_jwt(admin.id, admin.admin);
        let (lecturer_token, _) = generate_jwt(lecturer.id, lecturer.admin);
        let (assistant_lecturer_token, _) = generate_jwt(assistant_lecturer.id, assistant_lecturer.admin);
        let (tutor_token, _) = generate_jwt(tutor.id, tutor.admin);
        let (student_token, _) = generate_jwt(student.id, student.admin);
        let (unassigned_user_token, _) = generate_jwt(unassigned.id, unassigned.admin);

        TestContext {
            module_id: module.id,
            admin_token,
            lecturer_token,
            assistant_lecturer_token,
            tutor_token,
            student_token,
            unassigned_user_token,
        }
    }

    fn create_test_router<G, Fut>(guard: G) -> Router
    where
        G: Clone + Send + Sync + 'static,
        Fut: Future<Output = Result<Response, (StatusCode, Json<ApiResponse<Empty>>)>> + Send + 'static,
        G: Fn(Path<HashMap<String, String>>, Request<Body>, Next) -> Fut,
    {
        async fn test_handler() -> &'static str { "OK" }

        Router::new()
            .route(
                "/test/{module_id}",
                get(test_handler).route_layer(middleware::from_fn(guard)),
            )
            .layer(middleware::from_fn(require_authenticated))
    }

    
    fn build_request(uri: &str, token: Option<&str>) -> Request<Body> {
        let mut builder = Request::builder().uri(uri);
        if let Some(t) = token {
            builder = builder.header(header::AUTHORIZATION, format!("Bearer {}", t));
        }
        builder.body(Body::empty()).unwrap()
    }

    mod test_require_admin {
        use super::*;

        fn create_admin_router<G, Fut>(guard: G) -> Router
        where
            G: Clone + Send + Sync + 'static,
            Fut: Future<Output = Result<Response, (StatusCode, Json<ApiResponse<Empty>>)>> + Send + 'static,
            G: Fn(Request<Body>, Next) -> Fut,
        {
            async fn test_handler() -> &'static str { "OK" }

            Router::new()
                .route(
                    "/test/{module_id}",
                    get(test_handler).route_layer(middleware::from_fn(guard)),
                )
                .layer(middleware::from_fn(require_authenticated))
        }

        #[tokio::test]
        #[serial]
        async fn succeeds_for_admin() {
            let ctx = setup().await;
            let app = create_admin_router(require_admin);
            let req = build_request(&format!("/test/{}", ctx.module_id), Some(&ctx.admin_token));
            let res = app.oneshot(req).await.unwrap();
            assert_eq!(res.status(), StatusCode::OK);
        }

        #[tokio::test]
        #[serial]
        async fn fails_for_non_admin() {
            let ctx = setup().await;
            let app = create_admin_router(require_admin);
            let req = build_request(&format!("/test/{}", ctx.module_id), Some(&ctx.lecturer_token));
            let res = app.oneshot(req).await.unwrap();
            assert_eq!(res.status(), StatusCode::FORBIDDEN);
        }

        #[tokio::test]
        #[serial]
        async fn fails_for_unauthenticated() {
            let ctx = setup().await;
            let app = create_admin_router(require_admin);
            let req = build_request(&format!("/test/{}", ctx.module_id), None);
            let res = app.oneshot(req).await.unwrap();
            assert_eq!(res.status(), StatusCode::UNAUTHORIZED);
        }
    }

    mod test_require_lecturer {
        use super::*;

        #[tokio::test]
        #[serial]
        async fn succeeds_for_lecturer() {
            let ctx = setup().await;
            let app = create_test_router(require_lecturer);
            let req = build_request(&format!("/test/{}", ctx.module_id), Some(&ctx.lecturer_token));
            let res = app.oneshot(req).await.unwrap();
            assert_eq!(res.status(), StatusCode::OK);
        }

        #[tokio::test]
        #[serial]
        async fn succeeds_for_admin() {
            let ctx = setup().await;
            let app = create_test_router(require_lecturer);
            let req = build_request(&format!("/test/{}", ctx.module_id), Some(&ctx.admin_token));
            let res = app.oneshot(req).await.unwrap();
            assert_eq!(res.status(), StatusCode::OK);
        }

        #[tokio::test]
        #[serial]
        async fn fails_for_tutor() {
            let ctx = setup().await;
            let app = create_test_router(require_lecturer);
            let req = build_request(&format!("/test/{}", ctx.module_id), Some(&ctx.tutor_token));
            let res = app.oneshot(req).await.unwrap();
            assert_eq!(res.status(), StatusCode::FORBIDDEN);
        }
    }

    mod test_require_assistant_lecturer {
        use super::*;

        #[tokio::test]
        #[serial]
        async fn succeeds_for_assistant_lecturer() {
            let ctx = setup().await;
            let app = create_test_router(require_assistant_lecturer);
            let req = build_request(&format!("/test/{}", ctx.module_id), Some(&ctx.assistant_lecturer_token));
            let res = app.oneshot(req).await.unwrap();
            assert_eq!(res.status(), StatusCode::OK);
        }

        #[tokio::test]
        #[serial]
        async fn succeeds_for_admin() {
            let ctx = setup().await;
            let app = create_test_router(require_assistant_lecturer);
            let req = build_request(&format!("/test/{}", ctx.module_id), Some(&ctx.admin_token));
            let res = app.oneshot(req).await.unwrap();
            assert_eq!(res.status(), StatusCode::OK);
        }

        #[tokio::test]
        #[serial]
        async fn fails_for_lecturer() {
            let ctx = setup().await;
            let app = create_test_router(require_assistant_lecturer);
            let req = build_request(&format!("/test/{}", ctx.module_id), Some(&ctx.lecturer_token));
            let res = app.oneshot(req).await.unwrap();
            assert_eq!(res.status(), StatusCode::FORBIDDEN);
        }

        #[tokio::test]
        #[serial]
        async fn fails_for_tutor() {
            let ctx = setup().await;
            let app = create_test_router(require_assistant_lecturer);
            let req = build_request(&format!("/test/{}", ctx.module_id), Some(&ctx.tutor_token));
            let res = app.oneshot(req).await.unwrap();
            assert_eq!(res.status(), StatusCode::FORBIDDEN);
        }

        #[tokio::test]
        #[serial]
        async fn fails_for_student() {
            let ctx = setup().await;
            let app = create_test_router(require_assistant_lecturer);
            let req = build_request(&format!("/test/{}", ctx.module_id), Some(&ctx.student_token));
            let res = app.oneshot(req).await.unwrap();
            assert_eq!(res.status(), StatusCode::FORBIDDEN);
        }
    }
    
    mod test_require_tutor {
        use super::*;

        #[tokio::test]
        #[serial]
        async fn succeeds_for_tutor() {
            let ctx = setup().await;
            let app = create_test_router(require_tutor);
            let req = build_request(&format!("/test/{}", ctx.module_id), Some(&ctx.tutor_token));
            let res = app.oneshot(req).await.unwrap();
            assert_eq!(res.status(), StatusCode::OK);
        }

        #[tokio::test]
        #[serial]
        async fn succeeds_for_admin() {
            let ctx = setup().await;
            let app = create_test_router(require_tutor);
            let req = build_request(&format!("/test/{}", ctx.module_id), Some(&ctx.admin_token));
            let res = app.oneshot(req).await.unwrap();
            assert_eq!(res.status(), StatusCode::OK);
        }
        
        #[tokio::test]
        #[serial]
        async fn fails_for_lecturer() {
            let ctx = setup().await;
            let app = create_test_router(require_tutor);
            let req = build_request(&format!("/test/{}", ctx.module_id), Some(&ctx.lecturer_token));
            let res = app.oneshot(req).await.unwrap();
            assert_eq!(res.status(), StatusCode::FORBIDDEN);
        }
    }
    
    mod test_require_student {
        use super::*;

        #[tokio::test]
        #[serial]
        async fn succeeds_for_student() {
            let ctx = setup().await;
            let app = create_test_router(require_student);
            let req = build_request(&format!("/test/{}", ctx.module_id), Some(&ctx.student_token));
            let res = app.oneshot(req).await.unwrap();
            assert_eq!(res.status(), StatusCode::OK);
        }

        #[tokio::test]
        #[serial]
        async fn succeeds_for_admin() {
            let ctx = setup().await;
            let app = create_test_router(require_student);
            let req = build_request(&format!("/test/{}", ctx.module_id), Some(&ctx.admin_token));
            let res = app.oneshot(req).await.unwrap();
            assert_eq!(res.status(), StatusCode::OK);
        }

        #[tokio::test]
        #[serial]
        async fn fails_for_tutor() {
            let ctx = setup().await;
            let app = create_test_router(require_student);
            let req = build_request(&format!("/test/{}", ctx.module_id), Some(&ctx.tutor_token));
            let res = app.oneshot(req).await.unwrap();
            assert_eq!(res.status(), StatusCode::FORBIDDEN);
        }
    }

    mod test_require_lecturer_or_assistant_lecturer {
        use super::*;

        #[tokio::test]
        #[serial]
        async fn succeeds_for_lecturer() {
            let ctx = setup().await;
            let app = create_test_router(require_lecturer_or_assistant_lecturer);
            let req = build_request(&format!("/test/{}", ctx.module_id), Some(&ctx.lecturer_token));
            let res = app.oneshot(req).await.unwrap();
            assert_eq!(res.status(), StatusCode::OK);
        }

        #[tokio::test]
        #[serial]
        async fn succeeds_for_assistant_lecturer() {
            let ctx = setup().await;
            let app = create_test_router(require_lecturer_or_assistant_lecturer);
            let req = build_request(&format!("/test/{}", ctx.module_id), Some(&ctx.assistant_lecturer_token));
            let res = app.oneshot(req).await.unwrap();
            assert_eq!(res.status(), StatusCode::OK);
        }

        #[tokio::test]
        #[serial]
        async fn succeeds_for_admin() {
            let ctx = setup().await;
            let app = create_test_router(require_lecturer_or_assistant_lecturer);
            let req = build_request(&format!("/test/{}", ctx.module_id), Some(&ctx.admin_token));
            let res = app.oneshot(req).await.unwrap();
            assert_eq!(res.status(), StatusCode::OK);
        }

        #[tokio::test]
        #[serial]
        async fn fails_for_tutor() {
            let ctx = setup().await;
            let app = create_test_router(require_lecturer_or_assistant_lecturer);
            let req = build_request(&format!("/test/{}", ctx.module_id), Some(&ctx.tutor_token));
            let res = app.oneshot(req).await.unwrap();
            assert_eq!(res.status(), StatusCode::FORBIDDEN);
        }

        #[tokio::test]
        #[serial]
        async fn fails_for_student() {
            let ctx = setup().await;
            let app = create_test_router(require_lecturer_or_assistant_lecturer);
            let req = build_request(&format!("/test/{}", ctx.module_id), Some(&ctx.student_token));
            let res = app.oneshot(req).await.unwrap();
            assert_eq!(res.status(), StatusCode::FORBIDDEN);
        }
    }

    mod test_require_lecturer_or_tutor {
        use super::*;

        #[tokio::test]
        #[serial]
        async fn succeeds_for_lecturer() {
            let ctx = setup().await;
            let app = create_test_router(require_lecturer_or_tutor);
            let req = build_request(&format!("/test/{}", ctx.module_id), Some(&ctx.lecturer_token));
            let res = app.oneshot(req).await.unwrap();
            assert_eq!(res.status(), StatusCode::OK);
        }

        #[tokio::test]
        #[serial]
        async fn succeeds_for_tutor() {
            let ctx = setup().await;
            let app = create_test_router(require_lecturer_or_tutor);
            let req = build_request(&format!("/test/{}", ctx.module_id), Some(&ctx.tutor_token));
            let res = app.oneshot(req).await.unwrap();
            assert_eq!(res.status(), StatusCode::OK);
        }
        
        #[tokio::test]
        #[serial]
        async fn succeeds_for_admin() {
            let ctx = setup().await;
            let app = create_test_router(require_lecturer_or_tutor);
            let req = build_request(&format!("/test/{}", ctx.module_id), Some(&ctx.admin_token));
            let res = app.oneshot(req).await.unwrap();
            assert_eq!(res.status(), StatusCode::OK);
        }

        #[tokio::test]
        #[serial]
        async fn fails_for_student() {
            let ctx = setup().await;
            let app = create_test_router(require_lecturer_or_tutor);
            let req = build_request(&format!("/test/{}", ctx.module_id), Some(&ctx.student_token));
            let res = app.oneshot(req).await.unwrap();
            assert_eq!(res.status(), StatusCode::FORBIDDEN);
        }
    }
    
    mod test_require_assigned_to_module {
        use super::*;

        #[tokio::test]
        #[serial]
        async fn succeeds_for_lecturer() {
            let ctx = setup().await;
            let app = create_test_router(require_assigned_to_module);
            let req = build_request(&format!("/test/{}", ctx.module_id), Some(&ctx.lecturer_token));
            let res = app.oneshot(req).await.unwrap();
            assert_eq!(res.status(), StatusCode::OK);
        }

        #[tokio::test]
        #[serial]
        async fn succeeds_for_assistant_lecturer() {
            let ctx = setup().await;
            let app = create_test_router(require_assigned_to_module);
            let req = build_request(&format!("/test/{}", ctx.module_id), Some(&ctx.assistant_lecturer_token));
            let res = app.oneshot(req).await.unwrap();
            assert_eq!(res.status(), StatusCode::OK);
        }

        #[tokio::test]
        #[serial]
        async fn succeeds_for_student() {
            let ctx = setup().await;
            let app = create_test_router(require_assigned_to_module);
            let req = build_request(&format!("/test/{}", ctx.module_id), Some(&ctx.student_token));
            let res = app.oneshot(req).await.unwrap();
            assert_eq!(res.status(), StatusCode::OK);
        }

        #[tokio::test]
        #[serial]
        async fn succeeds_for_admin() {
            let ctx = setup().await;
            let app = create_test_router(require_assigned_to_module);
            let req = build_request(&format!("/test/{}", ctx.module_id), Some(&ctx.admin_token));
            let res = app.oneshot(req).await.unwrap();
            assert_eq!(res.status(), StatusCode::OK);
        }

        #[tokio::test]
        #[serial]
        async fn fails_for_unassigned_user() {
            let ctx = setup().await;
            let app = create_test_router(require_assigned_to_module);
            let req = build_request(&format!("/test/{}", ctx.module_id), Some(&ctx.unassigned_user_token));
            let res = app.oneshot(req).await.unwrap();
            assert_eq!(res.status(), StatusCode::FORBIDDEN);
        }
    }

    mod test_require_ready_assignment {
        use super::*;
        use sea_orm::{ActiveModelTrait, Set};

        fn create_assignment_router<G, Fut>(guard: G) -> Router
        where
            G: Fn(
                    Path<HashMap<String, String>>,
                    Request<Body>,
                    Next,
                ) -> Fut
                + Clone
                + Send
                + Sync
                + 'static,
            Fut: Future<Output = Result<Response, (StatusCode, Json<ApiResponse<Empty>>)>> + Send + 'static,
        {
            async fn test_handler() -> &'static str {
                "OK"
            }

            Router::new()
                .route(
                    "/modules/{module_id}/assignments/{assignment_id}",
                    get(test_handler)
                        .route_layer(middleware::from_fn(guard)),
                )
                .layer(middleware::from_fn(require_authenticated))
        }

        #[tokio::test]
        #[serial]
        async fn forbids_when_assignment_in_setup() {
            let ctx = setup().await;
            let db = db::get_connection().await;

            let assignment = AssignmentModel::create(db, ctx.module_id, "RS-SETUP", None, AssignmentType::Assignment, Utc::now(), Utc::now()).await.unwrap();
            let mut am: db::models::assignment::ActiveModel = assignment.clone().into();
            am.status = Set(db::models::assignment::Status::Setup);
            am.update(db).await.unwrap();

            let app = create_assignment_router(require_ready_assignment);
            let uri = format!("/modules/{}/assignments/{}", ctx.module_id, assignment.id);
            let res = app.oneshot(build_request(&uri, Some(&ctx.admin_token))).await.unwrap();
            assert_eq!(res.status(), StatusCode::FORBIDDEN);
        }

        #[tokio::test]
        #[serial]
        async fn allows_when_assignment_ready() {
            let ctx = setup().await;
            let db = db::get_connection().await;

            let assignment = AssignmentModel::create(db, ctx.module_id, "RS-READY", None, AssignmentType::Assignment, Utc::now(), Utc::now()).await.unwrap();
            let mut am: db::models::assignment::ActiveModel = assignment.clone().into();
            am.status = Set(db::models::assignment::Status::Ready);
            am.update(db).await.unwrap();

            let app = create_assignment_router(require_ready_assignment);
            let uri = format!("/modules/{}/assignments/{}", ctx.module_id, assignment.id);
            let res = app.oneshot(build_request(&uri, Some(&ctx.admin_token))).await.unwrap();
            assert_eq!(res.status(), StatusCode::OK);
        }

        #[tokio::test]
        #[serial]
        async fn allows_when_assignment_open_or_closed() {
            let ctx = setup().await;
            let db = db::get_connection().await;

            // Open
            let assignment_open = AssignmentModel::create(db, ctx.module_id, "RS-OPEN", None, AssignmentType::Assignment, Utc::now(), Utc::now()).await.unwrap();
            let mut am_open: db::models::assignment::ActiveModel = assignment_open.clone().into();
            am_open.status = Set(db::models::assignment::Status::Open);
            am_open.update(db).await.unwrap();

            let app = create_assignment_router(require_ready_assignment);
            let uri_open = format!("/modules/{}/assignments/{}", ctx.module_id, assignment_open.id);
            let res_open = app.clone().oneshot(build_request(&uri_open, Some(&ctx.admin_token))).await.unwrap();
            assert_eq!(res_open.status(), StatusCode::OK);

            // Closed
            let assignment_closed = AssignmentModel::create(db, ctx.module_id, "RS-CLOSED", None, AssignmentType::Assignment, Utc::now(), Utc::now()).await.unwrap();
            let mut am_closed: db::models::assignment::ActiveModel = assignment_closed.clone().into();
            am_closed.status = Set(db::models::assignment::Status::Closed);
            am_closed.update(db).await.unwrap();

            let uri_closed = format!("/modules/{}/assignments/{}", ctx.module_id, assignment_closed.id);
            let res_closed = app.oneshot(build_request(&uri_closed, Some(&ctx.admin_token))).await.unwrap();
            assert_eq!(res_closed.status(), StatusCode::OK);
        }

        #[tokio::test]
        #[serial]
        async fn returns_not_found_when_assignment_missing() {
            let ctx = setup().await;
            let non_existent_assignment_id = 99999;
            let app = create_assignment_router(require_ready_assignment);
            let uri = format!("/modules/{}/assignments/{}", ctx.module_id, non_existent_assignment_id);
            let res = app.oneshot(build_request(&uri, Some(&ctx.admin_token))).await.unwrap();
            assert_eq!(res.status(), StatusCode::NOT_FOUND);
        }
    }

    mod test_multi_param {
        use super::*;

        fn create_multi_param_router<G, Fut>( guard: G) -> Router
        where
            G: Fn(
                    Path<HashMap<String, String>>,
                    Request<Body>,
                    Next,
                ) -> Fut
                + Clone
                + Send
                + Sync
                + 'static,
            Fut: Future<Output = Result<Response, (StatusCode, Json<ApiResponse<Empty>>)>> + Send + 'static,
        {
            async fn test_handler() -> &'static str {
                "OK"
            }

            Router::new()
                .route(
                    "/test/{module_id}/other/{other_id}",
                    get(test_handler)
                        .route_layer(middleware::from_fn(guard)),
                )
                .layer(middleware::from_fn(require_authenticated))
        }


        #[tokio::test]
        #[serial]
        async fn succeeds_for_lecturer_with_multi_param() {
            let ctx = setup().await;
            let app = create_multi_param_router(require_lecturer);
            let req = build_request(&format!("/test/{}/other/42", ctx.module_id), Some(&ctx.lecturer_token));
            let res = app.oneshot(req).await.unwrap();
            assert_eq!(res.status(), StatusCode::OK);
        }

        #[tokio::test]
        #[serial]
        async fn fails_for_unassigned_user_with_multi_param() {
            let ctx = setup().await;
            let app = create_multi_param_router(require_lecturer);
            let req = build_request(&format!("/test/{}/other/42", ctx.module_id), Some(&ctx.unassigned_user_token));
            let res = app.oneshot(req).await.unwrap();
            assert_eq!(res.status(), StatusCode::FORBIDDEN);
        }

        #[tokio::test]
        #[serial]
        async fn succeeds_for_admin_with_multi_param() {
            let ctx = setup().await;
            let app = create_multi_param_router(require_lecturer);
            let req = build_request(&format!("/test/{}/other/42", ctx.module_id), Some(&ctx.admin_token));
            let res = app.oneshot(req).await.unwrap();
            assert_eq!(res.status(), StatusCode::OK);
        }
    }

    mod test_validate_known_ids {
        use super::*;
    
        fn create_router(path: &'static str) -> Router {
            async fn handler() -> &'static str {
                "OK"
            }

            Router::new()
                .route(path, get(handler))
                .layer(middleware::from_fn(require_authenticated))
                .route_layer(middleware::from_fn(validate_known_ids))
        }
    
        #[tokio::test]
        #[serial]
        async fn succeeds_when_module_exists() {
            let ctx = setup().await;
            let app = create_router("/test/{module_id}");
            let uri = format!("/test/{}", ctx.module_id);
            let res = app.oneshot(build_request(&uri, Some(&ctx.admin_token))).await.unwrap();
            assert_eq!(res.status(), StatusCode::OK);
        }

        #[tokio::test]
        #[serial]
        async fn fails_with_not_found_when_module_missing() {
            let ctx = setup().await;
            let app = create_router("/test/{module_id}");
            let res = app.oneshot(build_request("/test/99999", Some(&ctx.admin_token))).await.unwrap();
            assert_eq!(res.status(), StatusCode::NOT_FOUND);
        }

        #[tokio::test]
        #[serial]
        async fn fails_with_bad_request_on_parse_error() {
            let ctx = setup().await;
            let app = create_router("/test/{module_id}");
            let res = app.oneshot(build_request("/test/abc", Some(&ctx.admin_token))).await.unwrap();
            assert_eq!(res.status(), StatusCode::BAD_REQUEST);
        }

        #[tokio::test]
        #[serial]
        async fn fails_with_bad_request_on_unknown_param() {
            let ctx = setup().await;
            let app = create_router("/test/{unknown_id}");
            let res = app.oneshot(build_request("/test/123", Some(&ctx.admin_token))).await.unwrap();
            assert_eq!(res.status(), StatusCode::BAD_REQUEST);
        }

        #[tokio::test]
        #[serial]
        async fn fails_when_assignment_not_in_module() {
            let ctx = setup().await;
            let other = ModuleModel::create(db::get_connection().await, "OTHER", 2025, None, 1).await.unwrap();
            let assignment = AssignmentModel::create(db::get_connection().await, other.id, "A1", None, AssignmentType::Assignment, Utc::now(), Utc::now()).await.unwrap();
            let path = "/modules/{module_id}/assignments/{assignment_id}";
            let app = create_router(path);
            let uri = format!("/modules/{}/assignments/{}", ctx.module_id, assignment.id);
            let res = app.oneshot(build_request(&uri, Some(&ctx.admin_token))).await.unwrap();
            assert_eq!(res.status(), StatusCode::NOT_FOUND);
        }

        #[tokio::test]
        #[serial]
        async fn succeeds_for_assignment_in_module() {
            let ctx = setup().await;
            let assignment = AssignmentModel::create(db::get_connection().await, ctx.module_id, "A2", None, AssignmentType::Assignment, Utc::now(), Utc::now()).await.unwrap();
            let path = "/modules/{module_id}/assignments/{assignment_id}";
            let app = create_router(path);
            let uri = format!("/modules/{}/assignments/{}", ctx.module_id, assignment.id);
            let res = app.oneshot(build_request(&uri, Some(&ctx.admin_token))).await.unwrap();
            assert_eq!(res.status(), StatusCode::OK);
        }

        #[tokio::test]
        #[serial]
        async fn succeeds_for_task_hierarchy() {
            let ctx = setup().await;
            let assignment = AssignmentModel::create(db::get_connection().await, ctx.module_id, "A3", None, AssignmentType::Assignment, Utc::now(), Utc::now()).await.unwrap();
            let task = TaskModel::create(db::get_connection().await, assignment.id, 1, "T1", "echo").await.unwrap();
            let path = "/modules/{module_id}/assignments/{assignment_id}/tasks/{task_id}";
            let app = create_router(path);
            let uri = format!("/modules/{}/assignments/{}/tasks/{}", ctx.module_id, assignment.id, task.id);
            let res = app.oneshot(build_request(&uri, Some(&ctx.admin_token))).await.unwrap();
            assert_eq!(res.status(), StatusCode::OK);
        }

        #[tokio::test]
        #[serial]
        async fn fails_for_task_not_in_assignment() {
            let ctx = setup().await;
            let assignment1 = AssignmentModel::create(db::get_connection().await, ctx.module_id, "A4", None, AssignmentType::Assignment, Utc::now(), Utc::now()).await.unwrap();
            let assignment2 = AssignmentModel::create(db::get_connection().await, ctx.module_id, "A5", None, AssignmentType::Assignment, Utc::now(), Utc::now()).await.unwrap();
            let task = TaskModel::create(db::get_connection().await, assignment2.id, 1, "T2", "echo").await.unwrap();
            let path = "/modules/{module_id}/assignments/{assignment_id}/tasks/{task_id}";
            let app = create_router(path);
            let uri = format!("/modules/{}/assignments/{}/tasks/{}", ctx.module_id, assignment1.id, task.id);
            let res = app.oneshot(build_request(&uri, Some(&ctx.admin_token))).await.unwrap();
            assert_eq!(res.status(), StatusCode::NOT_FOUND);
        }

        #[tokio::test]
        #[serial]
        async fn succeeds_for_submission_and_file_hierarchy() {
            let ctx = setup().await;
            let assignment = AssignmentModel::create(db::get_connection().await, ctx.module_id, "A6", None, AssignmentType::Assignment, Utc::now(), Utc::now()).await.unwrap();
            let submission = SubmissionModel::save_file(db::get_connection().await, assignment.id, 1, 1, false, "f.txt", "hash123#", b"test").await.unwrap();
            let file = FileModel::save_file(db::get_connection().await, assignment.id, ctx.module_id, FileType::Spec, "f.txt", b"test").await.unwrap();

            let sub_path = "/modules/{module_id}/assignments/{assignment_id}/submissions/{submission_id}";
            let sub_app = create_router(sub_path);
            let sub_uri = format!("/modules/{}/assignments/{}/submissions/{}", ctx.module_id, assignment.id, submission.id);
            assert_eq!(sub_app.oneshot(build_request(&sub_uri, Some(&ctx.admin_token))).await.unwrap().status(), StatusCode::OK);

            let file_path = "/modules/{module_id}/assignments/{assignment_id}/files/{file_id}";
            let file_app = create_router(file_path);
            let file_uri = format!("/modules/{}/assignments/{}/files/{}", ctx.module_id, assignment.id, file.id);
            assert_eq!(file_app.oneshot(build_request(&file_uri, Some(&ctx.admin_token))).await.unwrap().status(), StatusCode::OK);
        }

        #[tokio::test]
        #[serial]
        async fn fails_for_submission_not_in_assignment() {
            let ctx = setup().await;
            let other_assignment = AssignmentModel::create(db::get_connection().await, ctx.module_id, "A7", None, AssignmentType::Assignment, Utc::now(), Utc::now()).await.unwrap();
            let submission = SubmissionModel::save_file(db::get_connection().await, other_assignment.id, 1, 1, false, "f.txt", "hash123#", b"test").await.unwrap();
            let path = "/modules/{module_id}/assignments/{assignment_id}/submissions/{submission_id}";
            let app = create_router(path);
            let uri = format!("/modules/{}/assignments/{}/submissions/{}", ctx.module_id, 0, submission.id);
            let res = app.oneshot(build_request(&uri, Some(&ctx.admin_token))).await.unwrap();
            assert_eq!(res.status(), StatusCode::NOT_FOUND);
        }
    }
}