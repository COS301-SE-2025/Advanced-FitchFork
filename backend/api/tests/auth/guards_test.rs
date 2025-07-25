#[cfg(test)]
mod tests {
    use axum::{
        extract::{Path, State},
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
            module::Model as ModuleModel, user::Model as UserModel,
            user_module_role::{Model as UserModuleRoleModel, Role},
            assignment::{Model as AssignmentModel, AssignmentType},
            assignment_task::Model as TaskModel,
            assignment_submission::Model as SubmissionModel,
            assignment_file::{Model as FileModel, FileType},
        },
        test_utils::setup_test_db,
    };
    use api::{
        auth::{
            generate_jwt,
            guards::{
                require_authenticated, require_admin, require_lecturer, require_tutor, require_student,
                require_lecturer_or_tutor, require_assigned_to_module, Empty, validate_known_ids
            }
        },
        response::ApiResponse
    };
    use dotenvy::dotenv;
    use sea_orm::DatabaseConnection;
    use tower::ServiceExt;
    use std::future::Future;
    use std::collections::HashMap;
    use chrono::Utc;

    struct TestContext {
        db: DatabaseConnection,
        admin_token: String,
        lecturer_token: String,
        tutor_token: String,
        student_token: String,
        unassigned_user_token: String,
        module_id: i64,
    }

    async fn setup() -> TestContext {
        dotenv().expect("Failed to load .env file for testing");
        let db = setup_test_db().await;

        let module = ModuleModel::create(&db, "TST101", 2025, Some("Guard Test Module"), 1).await.unwrap();
        let admin = UserModel::create(&db, "guard_admin", "ga@test.com", "pw", true).await.unwrap();
        let lecturer = UserModel::create(&db, "guard_lecturer", "gl@test.com", "pw", false).await.unwrap();
        let tutor = UserModel::create(&db, "guard_tutor", "gt@test.com", "pw", false).await.unwrap();
        let student = UserModel::create(&db, "guard_student", "gs@test.com", "pw", false).await.unwrap();
        let unassigned = UserModel::create(&db, "guard_unassigned", "gu@test.com", "pw", false).await.unwrap();

        UserModuleRoleModel::assign_user_to_module(&db, lecturer.id, module.id, Role::Lecturer).await.unwrap();
        UserModuleRoleModel::assign_user_to_module(&db, tutor.id, module.id, Role::Tutor).await.unwrap();
        UserModuleRoleModel::assign_user_to_module(&db, student.id, module.id, Role::Student).await.unwrap();

        let (admin_token, _) = generate_jwt(admin.id, admin.admin);
        let (lecturer_token, _) = generate_jwt(lecturer.id, lecturer.admin);
        let (tutor_token, _) = generate_jwt(tutor.id, tutor.admin);
        let (student_token, _) = generate_jwt(student.id, student.admin);
        let (unassigned_user_token, _) = generate_jwt(unassigned.id, unassigned.admin);

        TestContext {
            db,
            module_id: module.id,
            admin_token,
            lecturer_token,
            tutor_token,
            student_token,
            unassigned_user_token,
        }
    }

    fn create_test_router<G, Fut>(db: DatabaseConnection, guard: G) -> Router
    where
        G: Fn(State<DatabaseConnection>, Path<HashMap<String, String>>, Request<Body>, Next) -> Fut + Clone + Send + Sync + 'static,
        Fut: Future<Output = Result<Response, (StatusCode, Json<ApiResponse<Empty>>)>> + Send + 'static,
    {
        async fn test_handler() -> &'static str { "OK" }

        Router::new()
            .route(
                "/test/{module_id}",
                get(test_handler).route_layer(middleware::from_fn_with_state(db.clone(), guard)),
            )
            .layer(middleware::from_fn(require_authenticated))
            .with_state(db)
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
        async fn admin_guard_adapter(
            _s: State<DatabaseConnection>, _p: Path<HashMap<String, String>>, req: Request<Body>, next: Next
        ) -> Result<Response, (StatusCode, Json<ApiResponse<Empty>>)> {
            require_admin(req, next).await
        }

        #[tokio::test]
        async fn succeeds_for_admin() {
            let ctx = setup().await;
            let app = create_test_router(ctx.db.clone(), admin_guard_adapter);
            let req = build_request(&format!("/test/{}", ctx.module_id), Some(&ctx.admin_token));
            let res = app.oneshot(req).await.unwrap();
            assert_eq!(res.status(), StatusCode::OK);
        }

        #[tokio::test]
        async fn fails_for_non_admin() {
            let ctx = setup().await;
            let app = create_test_router(ctx.db.clone(), admin_guard_adapter);
            let req = build_request(&format!("/test/{}", ctx.module_id), Some(&ctx.lecturer_token));
            let res = app.oneshot(req).await.unwrap();
            assert_eq!(res.status(), StatusCode::FORBIDDEN);
        }

        #[tokio::test]
        async fn fails_for_unauthenticated() {
            let ctx = setup().await;
            let app = create_test_router(ctx.db.clone(), admin_guard_adapter);
            let req = build_request(&format!("/test/{}", ctx.module_id), None);
            let res = app.oneshot(req).await.unwrap();
            assert_eq!(res.status(), StatusCode::UNAUTHORIZED);
        }
    }

    mod test_require_lecturer {
        use super::*;

        #[tokio::test]
        async fn succeeds_for_lecturer() {
            let ctx = setup().await;
            let app = create_test_router(ctx.db.clone(), require_lecturer);
            let req = build_request(&format!("/test/{}", ctx.module_id), Some(&ctx.lecturer_token));
            let res = app.oneshot(req).await.unwrap();
            assert_eq!(res.status(), StatusCode::OK);
        }

        #[tokio::test]
        async fn succeeds_for_admin() {
            let ctx = setup().await;
            let app = create_test_router(ctx.db.clone(), require_lecturer);
            let req = build_request(&format!("/test/{}", ctx.module_id), Some(&ctx.admin_token));
            let res = app.oneshot(req).await.unwrap();
            assert_eq!(res.status(), StatusCode::OK);
        }

        #[tokio::test]
        async fn fails_for_tutor() {
            let ctx = setup().await;
            let app = create_test_router(ctx.db.clone(), require_lecturer);
            let req = build_request(&format!("/test/{}", ctx.module_id), Some(&ctx.tutor_token));
            let res = app.oneshot(req).await.unwrap();
            assert_eq!(res.status(), StatusCode::FORBIDDEN);
        }
    }
    
    mod test_require_tutor {
        use super::*;

        #[tokio::test]
        async fn succeeds_for_tutor() {
            let ctx = setup().await;
            let app = create_test_router(ctx.db.clone(), require_tutor);
            let req = build_request(&format!("/test/{}", ctx.module_id), Some(&ctx.tutor_token));
            let res = app.oneshot(req).await.unwrap();
            assert_eq!(res.status(), StatusCode::OK);
        }

        #[tokio::test]
        async fn succeeds_for_admin() {
            let ctx = setup().await;
            let app = create_test_router(ctx.db.clone(), require_tutor);
            let req = build_request(&format!("/test/{}", ctx.module_id), Some(&ctx.admin_token));
            let res = app.oneshot(req).await.unwrap();
            assert_eq!(res.status(), StatusCode::OK);
        }
        
        #[tokio::test]
        async fn fails_for_lecturer() {
            let ctx = setup().await;
            let app = create_test_router(ctx.db.clone(), require_tutor);
            let req = build_request(&format!("/test/{}", ctx.module_id), Some(&ctx.lecturer_token));
            let res = app.oneshot(req).await.unwrap();
            assert_eq!(res.status(), StatusCode::FORBIDDEN);
        }
    }
    
    mod test_require_student {
        use super::*;

        #[tokio::test]
        async fn succeeds_for_student() {
            let ctx = setup().await;
            let app = create_test_router(ctx.db.clone(), require_student);
            let req = build_request(&format!("/test/{}", ctx.module_id), Some(&ctx.student_token));
            let res = app.oneshot(req).await.unwrap();
            assert_eq!(res.status(), StatusCode::OK);
        }

        #[tokio::test]
        async fn succeeds_for_admin() {
            let ctx = setup().await;
            let app = create_test_router(ctx.db.clone(), require_student);
            let req = build_request(&format!("/test/{}", ctx.module_id), Some(&ctx.admin_token));
            let res = app.oneshot(req).await.unwrap();
            assert_eq!(res.status(), StatusCode::OK);
        }

        #[tokio::test]
        async fn fails_for_tutor() {
            let ctx = setup().await;
            let app = create_test_router(ctx.db.clone(), require_student);
            let req = build_request(&format!("/test/{}", ctx.module_id), Some(&ctx.tutor_token));
            let res = app.oneshot(req).await.unwrap();
            assert_eq!(res.status(), StatusCode::FORBIDDEN);
        }
    }

    mod test_require_lecturer_or_tutor {
        use super::*;

        #[tokio::test]
        async fn succeeds_for_lecturer() {
            let ctx = setup().await;
            let app = create_test_router(ctx.db.clone(), require_lecturer_or_tutor);
            let req = build_request(&format!("/test/{}", ctx.module_id), Some(&ctx.lecturer_token));
            let res = app.oneshot(req).await.unwrap();
            assert_eq!(res.status(), StatusCode::OK);
        }

        #[tokio::test]
        async fn succeeds_for_tutor() {
            let ctx = setup().await;
            let app = create_test_router(ctx.db.clone(), require_lecturer_or_tutor);
            let req = build_request(&format!("/test/{}", ctx.module_id), Some(&ctx.tutor_token));
            let res = app.oneshot(req).await.unwrap();
            assert_eq!(res.status(), StatusCode::OK);
        }
        
        #[tokio::test]
        async fn succeeds_for_admin() {
            let ctx = setup().await;
            let app = create_test_router(ctx.db.clone(), require_lecturer_or_tutor);
            let req = build_request(&format!("/test/{}", ctx.module_id), Some(&ctx.admin_token));
            let res = app.oneshot(req).await.unwrap();
            assert_eq!(res.status(), StatusCode::OK);
        }

        #[tokio::test]
        async fn fails_for_student() {
            let ctx = setup().await;
            let app = create_test_router(ctx.db.clone(), require_lecturer_or_tutor);
            let req = build_request(&format!("/test/{}", ctx.module_id), Some(&ctx.student_token));
            let res = app.oneshot(req).await.unwrap();
            assert_eq!(res.status(), StatusCode::FORBIDDEN);
        }
    }
    
    mod test_require_assigned_to_module {
        use super::*;

        #[tokio::test]
        async fn succeeds_for_lecturer() {
            let ctx = setup().await;
            let app = create_test_router(ctx.db.clone(), require_assigned_to_module);
            let req = build_request(&format!("/test/{}", ctx.module_id), Some(&ctx.lecturer_token));
            let res = app.oneshot(req).await.unwrap();
            assert_eq!(res.status(), StatusCode::OK);
        }

        #[tokio::test]
        async fn succeeds_for_student() {
            let ctx = setup().await;
            let app = create_test_router(ctx.db.clone(), require_assigned_to_module);
            let req = build_request(&format!("/test/{}", ctx.module_id), Some(&ctx.student_token));
            let res = app.oneshot(req).await.unwrap();
            assert_eq!(res.status(), StatusCode::OK);
        }

        #[tokio::test]
        async fn succeeds_for_admin() {
            let ctx = setup().await;
            let app = create_test_router(ctx.db.clone(), require_assigned_to_module);
            let req = build_request(&format!("/test/{}", ctx.module_id), Some(&ctx.admin_token));
            let res = app.oneshot(req).await.unwrap();
            assert_eq!(res.status(), StatusCode::OK);
        }

        #[tokio::test]
        async fn fails_for_unassigned_user() {
            let ctx = setup().await;
            let app = create_test_router(ctx.db.clone(), require_assigned_to_module);
            let req = build_request(&format!("/test/{}", ctx.module_id), Some(&ctx.unassigned_user_token));
            let res = app.oneshot(req).await.unwrap();
            assert_eq!(res.status(), StatusCode::FORBIDDEN);
        }
    }

    mod test_multi_param {
        use super::*;

        fn create_multi_param_router<G, Fut>(db: DatabaseConnection, guard: G) -> Router
        where
            G: Fn(State<DatabaseConnection>, Path<HashMap<String, String>>, Request<Body>, Next) -> Fut + Clone + Send + Sync + 'static,
            Fut: Future<Output = Result<Response, (StatusCode, Json<ApiResponse<Empty>>)>> + Send + 'static,
        {
            async fn test_handler() -> &'static str { "OK" }
            Router::new()
                .route(
                    "/test/{module_id}/other/{other_id}",
                    get(test_handler).route_layer(middleware::from_fn_with_state(db.clone(), guard)),
                )
                .layer(middleware::from_fn(require_authenticated))
                .with_state(db)
        }

        #[tokio::test]
        async fn succeeds_for_lecturer_with_multi_param() {
            let ctx = setup().await;
            let app = create_multi_param_router(ctx.db.clone(), require_lecturer);
            let req = build_request(&format!("/test/{}/other/42", ctx.module_id), Some(&ctx.lecturer_token));
            let res = app.oneshot(req).await.unwrap();
            assert_eq!(res.status(), StatusCode::OK);
        }

        #[tokio::test]
        async fn fails_for_unassigned_user_with_multi_param() {
            let ctx = setup().await;
            let app = create_multi_param_router(ctx.db.clone(), require_lecturer);
            let req = build_request(&format!("/test/{}/other/42", ctx.module_id), Some(&ctx.unassigned_user_token));
            let res = app.oneshot(req).await.unwrap();
            assert_eq!(res.status(), StatusCode::FORBIDDEN);
        }

        #[tokio::test]
        async fn succeeds_for_admin_with_multi_param() {
            let ctx = setup().await;
            let app = create_multi_param_router(ctx.db.clone(), require_lecturer);
            let req = build_request(&format!("/test/{}/other/42", ctx.module_id), Some(&ctx.admin_token));
            let res = app.oneshot(req).await.unwrap();
            assert_eq!(res.status(), StatusCode::OK);
        }
    }

    mod test_validate_known_ids {
        use super::*;
    
        fn create_router(
            db: DatabaseConnection,
            path: &'static str,
        ) -> Router {
            async fn handler() -> &'static str { "OK" }
            Router::new()
                .route(path, get(handler))
                .layer(middleware::from_fn(require_authenticated))
                .with_state(db.clone())
                .route_layer(middleware::from_fn_with_state(db, validate_known_ids))
        }
    
        #[tokio::test]
        async fn succeeds_when_module_exists() {
            let ctx = setup().await;
            let app = create_router(ctx.db.clone(), "/test/{module_id}");
            let uri = format!("/test/{}", ctx.module_id);
            let res = app.oneshot(build_request(&uri, Some(&ctx.admin_token))).await.unwrap();
            assert_eq!(res.status(), StatusCode::OK);
        }

        #[tokio::test]
        async fn fails_with_not_found_when_module_missing() {
            let ctx = setup().await;
            let app = create_router(ctx.db.clone(), "/test/{module_id}");
            let res = app.oneshot(build_request("/test/99999", Some(&ctx.admin_token))).await.unwrap();
            assert_eq!(res.status(), StatusCode::NOT_FOUND);
        }

        #[tokio::test]
        async fn fails_with_bad_request_on_parse_error() {
            let ctx = setup().await;
            let app = create_router(ctx.db.clone(), "/test/{module_id}");
            let res = app.oneshot(build_request("/test/abc", Some(&ctx.admin_token))).await.unwrap();
            assert_eq!(res.status(), StatusCode::BAD_REQUEST);
        }

        #[tokio::test]
        async fn fails_with_bad_request_on_unknown_param() {
            let ctx = setup().await;
            let app = create_router(ctx.db.clone(), "/test/{unknown_id}");
            let res = app.oneshot(build_request("/test/123", Some(&ctx.admin_token))).await.unwrap();
            assert_eq!(res.status(), StatusCode::BAD_REQUEST);
        }

        #[tokio::test]
        async fn fails_when_assignment_not_in_module() {
            let ctx = setup().await;
            let other = ModuleModel::create(&ctx.db, "OTHER", 2025, None, 1).await.unwrap();
            let assignment = AssignmentModel::create(&ctx.db, other.id, "A1", None, AssignmentType::Assignment, Utc::now(), Utc::now()).await.unwrap();
            let path = "/modules/{module_id}/assignments/{assignment_id}";
            let app = create_router(ctx.db.clone(), path);
            let uri = format!("/modules/{}/assignments/{}", ctx.module_id, assignment.id);
            let res = app.oneshot(build_request(&uri, Some(&ctx.admin_token))).await.unwrap();
            assert_eq!(res.status(), StatusCode::NOT_FOUND);
        }

        #[tokio::test]
        async fn succeeds_for_assignment_in_module() {
            let ctx = setup().await;
            let assignment = AssignmentModel::create(&ctx.db, ctx.module_id, "A2", None, AssignmentType::Assignment, Utc::now(), Utc::now()).await.unwrap();
            let path = "/modules/{module_id}/assignments/{assignment_id}";
            let app = create_router(ctx.db.clone(), path);
            let uri = format!("/modules/{}/assignments/{}", ctx.module_id, assignment.id);
            let res = app.oneshot(build_request(&uri, Some(&ctx.admin_token))).await.unwrap();
            assert_eq!(res.status(), StatusCode::OK);
        }

        #[tokio::test]
        async fn succeeds_for_task_hierarchy() {
            let ctx = setup().await;
            let assignment = AssignmentModel::create(&ctx.db, ctx.module_id, "A3", None, AssignmentType::Assignment, Utc::now(), Utc::now()).await.unwrap();
            let task = TaskModel::create(&ctx.db, assignment.id, 1, "T1", "echo").await.unwrap();
            let path = "/modules/{module_id}/assignments/{assignment_id}/tasks/{task_id}";
            let app = create_router(ctx.db.clone(), path);
            let uri = format!("/modules/{}/assignments/{}/tasks/{}", ctx.module_id, assignment.id, task.id);
            let res = app.oneshot(build_request(&uri, Some(&ctx.admin_token))).await.unwrap();
            assert_eq!(res.status(), StatusCode::OK);
        }

        #[tokio::test]
        async fn fails_for_task_not_in_assignment() {
            let ctx = setup().await;
            let assignment1 = AssignmentModel::create(&ctx.db, ctx.module_id, "A4", None, AssignmentType::Assignment, Utc::now(), Utc::now()).await.unwrap();
            let assignment2 = AssignmentModel::create(&ctx.db, ctx.module_id, "A5", None, AssignmentType::Assignment, Utc::now(), Utc::now()).await.unwrap();
            let task = TaskModel::create(&ctx.db, assignment2.id, 1, "T2", "echo").await.unwrap();
            let path = "/modules/{module_id}/assignments/{assignment_id}/tasks/{task_id}";
            let app = create_router(ctx.db.clone(), path);
            let uri = format!("/modules/{}/assignments/{}/tasks/{}", ctx.module_id, assignment1.id, task.id);
            let res = app.oneshot(build_request(&uri, Some(&ctx.admin_token))).await.unwrap();
            assert_eq!(res.status(), StatusCode::NOT_FOUND);
        }

        #[tokio::test]
        async fn succeeds_for_submission_and_file_hierarchy() {
            let ctx = setup().await;
            let assignment = AssignmentModel::create(&ctx.db, ctx.module_id, "A6", None, AssignmentType::Assignment, Utc::now(), Utc::now()).await.unwrap();
            let submission = SubmissionModel::save_file(&ctx.db, assignment.id, 1, 1, "f.txt", b"test").await.unwrap();
            let file = FileModel::save_file(&ctx.db, assignment.id, ctx.module_id, FileType::Spec, "f.txt", b"test").await.unwrap();

            let sub_path = "/modules/{module_id}/assignments/{assignment_id}/submissions/{submission_id}";
            let sub_app = create_router(ctx.db.clone(), sub_path);
            let sub_uri = format!("/modules/{}/assignments/{}/submissions/{}", ctx.module_id, assignment.id, submission.id);
            assert_eq!(sub_app.oneshot(build_request(&sub_uri, Some(&ctx.admin_token))).await.unwrap().status(), StatusCode::OK);

            let file_path = "/modules/{module_id}/assignments/{assignment_id}/files/{file_id}";
            let file_app = create_router(ctx.db.clone(), file_path);
            let file_uri = format!("/modules/{}/assignments/{}/files/{}", ctx.module_id, assignment.id, file.id);
            assert_eq!(file_app.oneshot(build_request(&file_uri, Some(&ctx.admin_token))).await.unwrap().status(), StatusCode::OK);
        }

        #[tokio::test]
        async fn fails_for_submission_not_in_assignment() {
            let ctx = setup().await;
            let other_assignment = AssignmentModel::create(&ctx.db, ctx.module_id, "A7", None, AssignmentType::Assignment, Utc::now(), Utc::now()).await.unwrap();
            let submission = SubmissionModel::save_file(&ctx.db, other_assignment.id, 1, 1, "f.txt", b"test").await.unwrap();
            let path = "/modules/{module_id}/assignments/{assignment_id}/submissions/{submission_id}";
            let app = create_router(ctx.db.clone(), path);
            let uri = format!("/modules/{}/assignments/{}/submissions/{}", ctx.module_id, 0, submission.id);
            let res = app.oneshot(build_request(&uri, Some(&ctx.admin_token))).await.unwrap();
            assert_eq!(res.status(), StatusCode::NOT_FOUND);
        }
    }
}