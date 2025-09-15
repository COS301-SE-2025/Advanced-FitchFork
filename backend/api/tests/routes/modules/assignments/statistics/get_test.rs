#[cfg(test)]
mod tests {
    use axum::{
        body::Body,
        http::{Request, StatusCode},
    };
    use chrono::{DateTime, Duration, TimeZone, Utc};
    use db::models::{
        assignment::{AssignmentType, Model as AssignmentModel},
        assignment_submission::{Model as AssignmentSubmissionModel},
        module::Model as ModuleModel,
        user::Model as UserModel,
        user_module_role::{Model as UserModuleRoleModel, Role},
    };
    use serde_json::{json, Value};
    use tower::ServiceExt;

    use api::auth::generate_jwt;
    use util::{execution_config::{execution_config::GradingPolicy, ExecutionConfig}, paths::submission_report_path};
    use crate::helpers::app::make_test_app_with_storage;

    use sea_orm::{ActiveModelTrait, Set};
    use serial_test::serial;
    use std::fs;
    
    struct TestData {
        admin_user: UserModel,
        lecturer_user: UserModel,
        student1: UserModel,
        student2: UserModel,
        student3: UserModel,
        outsider_user: UserModel,
        module: ModuleModel,
        assignment_best: AssignmentModel,
        assignment_last: AssignmentModel,
    }

    // ---------- Helpers ----------

    /// Create and persist a default config.json for a given assignment.
    /// Applies overrides for grading policy + pass mark.
    fn write_config_json(
        module_id: i64,
        assignment_id: i64,
        grading_policy: GradingPolicy,
        pass_mark: u32,
    ) {
        // Start with defaults
        let mut cfg = ExecutionConfig::default_config();

        // Apply overrides
        cfg.marking.grading_policy = grading_policy;
        cfg.marking.pass_mark = pass_mark;

        // Ensure config directory exists and save
        if let Err(e) = cfg.save(module_id, assignment_id) {
            panic!(
                "Failed to save config for module {} assignment {}: {}",
                module_id, assignment_id, e
            );
        }
    }

    async fn update_submission_time(
        db: &sea_orm::DatabaseConnection,
        submission_id: i64,
        new_time: chrono::DateTime<Utc>,
    ) {
        use db::models::assignment_submission::{ActiveModel, Entity};
        use sea_orm::EntityTrait;
        if let Some(model) = Entity::find_by_id(submission_id).one(db).await.unwrap() {
            let mut active: ActiveModel = model.into();
            active.created_at = Set(new_time);
            active.updated_at = Set(new_time);
            let _ = active.update(db).await.unwrap();
        }
    }

    async fn mark_submission_ignored(
        db: &sea_orm::DatabaseConnection,
        submission_id: i64,
        ignored: bool,
    ) {
        use db::models::assignment_submission::{ActiveModel, Entity};
        use sea_orm::EntityTrait;
        if let Some(model) = Entity::find_by_id(submission_id).one(db).await.unwrap() {
            let mut active: ActiveModel = model.into();
            active.ignored = Set(ignored);
            let _ = active.update(db).await.unwrap();
        }
    }

    /// Write a `submission_report.json` file for a given attempt.
    /// Uses the path utilities to resolve the correct location.
    fn write_submission_report(
        module_id: i64,
        assignment_id: i64,
        user_id: i64,
        attempt: i64,
        created_at: DateTime<Utc>,
        earned: i64,
        total: i64,
        is_practice: bool,
        is_late: bool,
    ) {
        // Use the shared utility instead of manual path building
        let path = submission_report_path(module_id, assignment_id, user_id, attempt);

        // Ensure parent directories exist
        fs::create_dir_all(path.parent().unwrap())
            .expect("Failed to create directories for submission report");

        // Construct the JSON payload
        let report = json!({
            "attempt": attempt,
            "filename": format!("u{}_a{}.zip", user_id, attempt),
            "created_at": created_at.to_rfc3339(),
            "updated_at": created_at.to_rfc3339(),
            "is_practice": is_practice,
            "is_late": is_late,
            "mark": { "earned": earned, "total": total }
        });

        // Write pretty-printed JSON
        fs::write(&path, serde_json::to_string_pretty(&report).unwrap())
            .expect("Failed to write submission_report.json");
    }

    /// Seed N attempts for a user with given (earned,total) and minute offsets vs due date.
    async fn seed_for_user(
        db: &sea_orm::DatabaseConnection,
        module_id: i64,
        assignment: &AssignmentModel,
        user: &UserModel,
        marks: &[(i64, i64)],
        offsets_min: &[i64],
    ) -> Vec<AssignmentSubmissionModel> {
        assert_eq!(marks.len(), offsets_min.len());
        let mut out = Vec::new();
        for (idx, ((earned, total), off)) in marks.iter().zip(offsets_min.iter()).enumerate() {
            let attempt = (idx as i64) + 1;
            let created = assignment.due_date + Duration::minutes(*off);
            let sub = AssignmentSubmissionModel::save_file(
                db,
                assignment.id,
                user.id,
                attempt,
                *earned,
                *total,
                false, // not practice
                &format!("u{}_a{}.zip", user.id, attempt),
                "hash",
                b"dummy",
            )
            .await
            .unwrap();
            update_submission_time(db, sub.id, created).await;
            write_submission_report(
                module_id,
                assignment.id,
                user.id,
                attempt,
                created,
                *earned,
                *total,
                false, // not practice
                created > assignment.due_date,
            );
            out.push(sub);
        }
        out
    }

    /// Seed a single attempt for a user where we can control practice/ignored flags explicitly.
    async fn seed_one_with_flags(
        db: &sea_orm::DatabaseConnection,
        module_id: i64,
        assignment: &AssignmentModel,
        user: &UserModel,
        attempt: i64,
        earned: i64,
        total: i64,
        offset_min: i64,
        is_practice: bool,
        ignored: bool,
    ) -> AssignmentSubmissionModel {
        let created = assignment.due_date + Duration::minutes(offset_min);
        let sub = AssignmentSubmissionModel::save_file(
            db,
            assignment.id,
            user.id,
            attempt,
            earned,
            total,
            is_practice,
            &format!("u{}_a{}.zip", user.id, attempt),
            "hash",
            b"dummy",
        )
        .await
        .unwrap();
        update_submission_time(db, sub.id, created).await;
        write_submission_report(
            module_id,
            assignment.id,
            user.id,
            attempt,
            created,
            earned,
            total,
            is_practice,
            created > assignment.due_date,
        );
        if ignored {
            mark_submission_ignored(db, sub.id, true).await;
        }
        sub
    }

    async fn setup_test_data(db: &sea_orm::DatabaseConnection) -> TestData {
        // Users & module
        let admin_user =
            UserModel::create(db, "admin1", "admin1@test.com", "password", true).await.unwrap();
        let lecturer_user =
            UserModel::create(db, "lecturer1", "lecturer1@test.com", "password1", false)
                .await
                .unwrap();

        let student1 =
            UserModel::create(db, "student1", "student1@test.com", "password2", false)
                .await
                .unwrap();
        let student2 =
            UserModel::create(db, "student2", "student2@test.com", "password3", false)
                .await
                .unwrap();
        let student3 =
            UserModel::create(db, "student3", "student3@test.com", "password4", false)
                .await
                .unwrap();

        let outsider_user =
            UserModel::create(db, "outsider", "outsider@test.com", "password5", false)
                .await
                .unwrap();

        let module =
            ModuleModel::create(db, "COS101", 2024, Some("Test Module"), 16).await.unwrap();

        UserModuleRoleModel::assign_user_to_module(db, lecturer_user.id, module.id, Role::Lecturer)
            .await
            .unwrap();
        for s in [&student1, &student2, &student3] {
            UserModuleRoleModel::assign_user_to_module(db, s.id, module.id, Role::Student)
                .await
                .unwrap();
        }

        // Two assignments: one with Best, one with Last
        let a_best = AssignmentModel::create(
            db,
            module.id,
            "A Best",
            Some("Stats Best"),
            AssignmentType::Assignment,
            Utc.with_ymd_and_hms(2024, 1, 1, 0, 0, 0).unwrap(),
            Utc.with_ymd_and_hms(2024, 1, 31, 23, 59, 59).unwrap(),
        )
        .await
        .unwrap();

        let a_last = AssignmentModel::create(
            db,
            module.id,
            "A Last",
            Some("Stats Last"),
            AssignmentType::Practical,
            Utc.with_ymd_and_hms(2024, 2, 1, 0, 0, 0).unwrap(),
            Utc.with_ymd_and_hms(2024, 2, 28, 23, 59, 59).unwrap(),
        )
        .await
        .unwrap();

        // Write configs
        write_config_json( module.id, a_best.id, GradingPolicy::Best, 50);
        write_config_json( module.id, a_last.id, GradingPolicy::Last, 50);

        TestData {
            admin_user,
            lecturer_user,
            student1,
            student2,
            student3,
            outsider_user,
            module,
            assignment_best: a_best,
            assignment_last: a_last,
        }
    }

    // ---------- Tests ----------

    // --- Access control & 404s ---

    #[tokio::test]
    #[serial]
    async fn test_stats_unauthorized() {
        let (app, app_state, _tmp) = make_test_app_with_storage().await;
        let data = setup_test_data(app_state.db()).await;

        let uri = format!(
            "/api/modules/{}/assignments/{}/stats",
            data.module.id, data.assignment_best.id
        );
        let req = Request::builder().uri(&uri).body(Body::empty()).unwrap();
        let response = app.clone().oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    #[serial]
    async fn test_stats_forbidden_for_student_and_outsider() {
        let (app, app_state, _tmp) = make_test_app_with_storage().await;
        let data = setup_test_data(app_state.db()).await;

        for (uid, is_admin) in [
            (data.student1.id, data.student1.admin),
            (data.outsider_user.id, data.outsider_user.admin),
        ] {
            let (token, _) = generate_jwt(uid, is_admin);
            let uri = format!(
                "/api/modules/{}/assignments/{}/stats",
                data.module.id, data.assignment_best.id
            );
            let req = Request::builder()
                .uri(&uri)
                .header("Authorization", format!("Bearer {}", token))
                .body(Body::empty())
                .unwrap();
            let response = app.clone().oneshot(req).await.unwrap();
            assert_eq!(response.status(), StatusCode::FORBIDDEN);
        }
    }

    #[tokio::test]
    #[serial]
    async fn test_stats_lecturer_and_admin_ok() {
        let (app, app_state, _tmp) = make_test_app_with_storage().await;
        let data = setup_test_data(app_state.db()).await;

        for (uid, is_admin) in [
            (data.lecturer_user.id, data.lecturer_user.admin),
            (data.admin_user.id, data.admin_user.admin),
        ] {
            let (token, _) = generate_jwt(uid, is_admin);
            let uri = format!(
                "/api/modules/{}/assignments/{}/stats",
                data.module.id, data.assignment_best.id
            );
            let req = Request::builder()
                .uri(&uri)
                .header("Authorization", format!("Bearer {}", token))
                .body(Body::empty())
                .unwrap();
            let response = app.clone().oneshot(req).await.unwrap();
            // no data yet → should still be 200 OK with zeros
            assert_eq!(response.status(), StatusCode::OK);
        }
    }

    #[tokio::test]
    #[serial]
    async fn test_stats_assignment_not_found_and_wrong_module() {
        let (app, app_state, _tmp) = make_test_app_with_storage().await;
        let data = setup_test_data(app_state.db()).await;
        let (token, _) = generate_jwt(data.admin_user.id, data.admin_user.admin);

        // wrong module id
        let uri = format!("/api/modules/{}/assignments/{}/stats", 9999, data.assignment_best.id);
        let req = Request::builder()
            .uri(&uri)
            .header("Authorization", format!("Bearer {}", token))
            .body(Body::empty())
            .unwrap();
        let response = app.clone().oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::NOT_FOUND);

        // wrong assignment id
        let uri = format!("/api/modules/{}/assignments/{}/stats", data.module.id, 999999);
        let req = Request::builder()
            .uri(&uri)
            .header("Authorization", format!("Bearer {}", token))
            .body(Body::empty())
            .unwrap();
        let response = app.clone().oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    // --- Functional math: BEST policy with 3 users, multi-attempts ---

    #[tokio::test]
    #[serial]
    async fn test_stats_math_best_policy() {
        let (app, app_state, _tmp) = make_test_app_with_storage().await;
        let data = setup_test_data(app_state.db()).await;
        let db = app_state.db();

        // student1: 80 (early), 60 (late), 90 (early) → best = 90
        let _s1 = seed_for_user(
            db,
            data.module.id,
            &data.assignment_best,
            &data.student1,
            &[(80, 100), (60, 100), (90, 100)],
            &[-120, 10, -30],
        )
        .await;

        // student2: 30 (early), 50 (early) → best = 50
        let _s2 = seed_for_user(
            db,
            data.module.id,
            &data.assignment_best,
            &data.student2,
            &[(30, 100), (50, 100)],
            &[-240, -60],
        )
        .await;

        // student3: 100 (early), 100 (late) → best = 100
        let _s3 = seed_for_user(
            db,
            data.module.id,
            &data.assignment_best,
            &data.student3,
            &[(100, 100), (100, 100)],
            &[-10, 120],
        )
        .await;

        // Call as lecturer
        let (token, _) = generate_jwt(data.lecturer_user.id, data.lecturer_user.admin);
        let uri = format!(
            "/api/modules/{}/assignments/{}/stats",
            data.module.id, data.assignment_best.id
        );
        let req = Request::builder()
            .uri(&uri)
            .header("Authorization", format!("Bearer {}", token))
            .body(Body::empty())
            .unwrap();
        let response = app.clone().oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let json: Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(json["success"], true);

        let d = &json["data"];

        // totals: attempts = 3 + 2 + 2 = 7
        assert_eq!(d["total"], 7);
        // graded = 3 students with valid reports
        assert_eq!(d["graded"], 3);
        assert_eq!(d["pending"], 4); // 7 - 3

        // flags: late counts from offsets: s1 has 1 late, s2 0 late, s3 1 late → late=2, on_time=5
        assert_eq!(d["late"], 2);
        assert_eq!(d["on_time"], 5);
        assert_eq!(d["ignored"], 0);

        // effective marks: [90, 50, 100]
        // avg = 80.0, median = 90.0, p75 = 95.0, stddev ≈ 26.5
        assert_eq!(d["avg_mark"], 80.0);
        assert_eq!(d["median"], 90.0);
        assert_eq!(d["p75"], 95.0);
        assert_eq!(d["best"], 100);
        assert_eq!(d["worst"], 50);
        assert_eq!(d["stddev"], 26.5); // ((10^2 + 30^2 + 20^2)/2)^0.5 = 26.5

        // pass mark 50 → student2 with best=50 passes (>=), all 3 pass
        assert_eq!(d["num_passed"], 3);
        assert_eq!(d["num_failed"], 0);
        assert_eq!(d["num_full_marks"], 1); // only student3 has 100

        // total_marks is sum of per-row totals: 7 * 100
        assert_eq!(d["total_marks"], 700);
        assert_eq!(d["num_students_submitted"], 3);

    }

    // --- Functional math: LAST policy with 3 users, multi-attempts ---

    #[tokio::test]
    #[serial]
    async fn test_stats_math_last_policy() {
        let (app, app_state, _tmp) = make_test_app_with_storage().await;
        let data = setup_test_data(app_state.db()).await;
        let db = app_state.db();

        // student1: 40 (early), 65 (early), 70 (late) → last=70
        let _s1 = seed_for_user(
            db,
            data.module.id,
            &data.assignment_last,
            &data.student1,
            &[(40, 100), (65, 100), (70, 100)],
            &[-120, -110, 10],
        )
        .await;

        // student2: 55 (early), 45 (late) → last=45  (to test a fail)
        let _s2 = seed_for_user(
            db,
            data.module.id,
            &data.assignment_last,
            &data.student2,
            &[(55, 100), (45, 100)],
            &[-60, 30],
        )
        .await;

        // student3: 80 (early), 90 (early) → last=90
        let _s3 = seed_for_user(
            db,
            data.module.id,
            &data.assignment_last,
            &data.student3,
            &[(80, 100), (90, 100)],
            &[-5, -1],
        )
        .await;

        // Call as admin
        let (token, _) = generate_jwt(data.admin_user.id, data.admin_user.admin);
        let uri = format!(
            "/api/modules/{}/assignments/{}/stats",
            data.module.id, data.assignment_last.id
        );
        let req = Request::builder()
            .uri(&uri)
            .header("Authorization", format!("Bearer {}", token))
            .body(Body::empty())
            .unwrap();
        let response = app.clone().oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let json: Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(json["success"], true);

        let d = &json["data"];
        // totals: attempts 3 + 2 + 2 = 7
        assert_eq!(d["total"], 7);
        // graded = 3 students with reports; pending = total - graded
        assert_eq!(d["graded"], 3);
        assert_eq!(d["pending"], 4);

        // flags: late = s1(1) + s2(1) + s3(0) = 2, on_time = 5
        assert_eq!(d["late"], 2);
        assert_eq!(d["on_time"], 5);
        assert_eq!(d["ignored"], 0);

        // effective LAST marks: [70, 45, 90]
        assert_eq!(d["avg_mark"], 68.3);
        assert_eq!(d["median"], 70.0);
        assert_eq!(d["p75"], 80.0);
        assert_eq!(d["best"], 90);
        assert_eq!(d["worst"], 45);
        assert_eq!(d["stddev"], 22.5);

        // pass mark 50 → pass: 70 & 90 = 2, fail: 45 = 1
        assert_eq!(d["num_passed"], 2);
        assert_eq!(d["num_failed"], 1);
        assert_eq!(d["num_full_marks"], 0);

        // total_marks = 7 * 100
        assert_eq!(d["total_marks"], 700);
        assert_eq!(d["num_students_submitted"], 3);

    }

    // --- Staff submissions shouldn't affect stats (BEST & LAST policies) ---

    #[tokio::test]
    #[serial]
    async fn test_stats_staff_submissions_are_ignored_best_and_last() {
        let (app, app_state, _tmp) = make_test_app_with_storage().await;
        let data = setup_test_data(app_state.db()).await;
        let db = app_state.db();

        // Add extra staff users: assistant lecturer & tutor
        let assistant =
            UserModel::create(db, "assist1", "assist1@test.com", "pw", false).await.unwrap();
        let tutor =
            UserModel::create(db, "tutor1", "tutor1@test.com", "pw", false).await.unwrap();

        // Assign them to the module with their roles
        UserModuleRoleModel::assign_user_to_module(db, assistant.id, data.module.id, Role::AssistantLecturer)
            .await
            .unwrap();
        UserModuleRoleModel::assign_user_to_module(db, tutor.id, data.module.id, Role::Tutor)
            .await
            .unwrap();

        // ----- Seed student data (3 users, multi-attempts) -----
        // BEST assignment
        let _ = seed_for_user(
            db, data.module.id, &data.assignment_best, &data.student1,
            &[(80, 100), (60, 100), (90, 100)], &[-120, 10, -30],
        ).await;
        let _ = seed_for_user(
            db,  data.module.id, &data.assignment_best, &data.student2,
            &[(30, 100), (50, 100)], &[-240, -60],
        ).await;
        let _ = seed_for_user(
            db,  data.module.id, &data.assignment_best, &data.student3,
            &[(100, 100), (100, 100)], &[-10, 120],
        ).await;

        // LAST assignment
        let _ = seed_for_user(
            db, data.module.id, &data.assignment_last, &data.student1,
            &[(40, 100), (65, 100), (70, 100)], &[-120, -110, 10],
        ).await;
        let _ = seed_for_user(
            db, data.module.id, &data.assignment_last, &data.student2,
            &[(55, 100), (45, 100)], &[-60, 30],
        ).await;
        let _ = seed_for_user(
            db, data.module.id, &data.assignment_last, &data.student3,
            &[(80, 100), (90, 100)], &[-5, -1],
        ).await;

        // ----- Seed STAFF submissions that should be ignored in stats -----
        let staff_marks = &[(1, 100), (100, 100)];
        let staff_offsets = &[-1, 1];

        // admin (already in data.admin_user)
        let _ = seed_for_user(
            db, data.module.id, &data.assignment_best, &data.admin_user,
            staff_marks, staff_offsets,
        ).await;
        let _ = seed_for_user(
            db, data.module.id, &data.assignment_last, &data.admin_user,
            staff_marks, staff_offsets,
        ).await;

        // lecturer (already in data.lecturer_user)
        let _ = seed_for_user(
            db, data.module.id, &data.assignment_best, &data.lecturer_user,
            staff_marks, staff_offsets,
        ).await;
        let _ = seed_for_user(
            db, data.module.id, &data.assignment_last, &data.lecturer_user,
            staff_marks, staff_offsets,
        ).await;

        // assistant lecturer
        let _ = seed_for_user(
            db, data.module.id, &data.assignment_best, &assistant,
            staff_marks, staff_offsets,
        ).await;
        let _ = seed_for_user(
            db, data.module.id, &data.assignment_last, &assistant,
            staff_marks, staff_offsets,
        ).await;

        // tutor
        let _ = seed_for_user(
            db, data.module.id, &data.assignment_best, &tutor,
            staff_marks, staff_offsets,
        ).await;
        let _ = seed_for_user(
            db, data.module.id, &data.assignment_last, &tutor,
            staff_marks, staff_offsets,
        ).await;

        // ----- Assert BEST stats unchanged (i.e., staff ignored) -----
        let (token_lect, _) = generate_jwt(data.lecturer_user.id, data.lecturer_user.admin);
        let uri_best = format!(
            "/api/modules/{}/assignments/{}/stats",
            data.module.id, data.assignment_best.id
        );
        let req_best = Request::builder()
            .uri(&uri_best)
            .header("Authorization", format!("Bearer {}", token_lect))
            .body(Body::empty())
            .unwrap();
        let resp_best = app.clone().oneshot(req_best).await.unwrap();
        assert_eq!(resp_best.status(), StatusCode::OK);
        let body_best = axum::body::to_bytes(resp_best.into_body(), usize::MAX).await.unwrap();
        let json_best: Value = serde_json::from_slice(&body_best).unwrap();
        let d = &json_best["data"];
        assert_eq!(d["total"], 7);
        assert_eq!(d["graded"], 3);
        assert_eq!(d["pending"], 4);
        assert_eq!(d["late"], 2);
        assert_eq!(d["on_time"], 5);
        assert_eq!(d["ignored"], 0);
        assert_eq!(d["avg_mark"], 80.0);
        assert_eq!(d["median"], 90.0);
        assert_eq!(d["p75"], 95.0);
        assert_eq!(d["best"], 100);
        assert_eq!(d["worst"], 50);
        assert_eq!(d["stddev"], 26.5);
        assert_eq!(d["num_passed"], 3);
        assert_eq!(d["num_failed"], 0);
        assert_eq!(d["num_full_marks"], 1);
        assert_eq!(d["total_marks"], 700);
        assert_eq!(d["num_students_submitted"], 3);

        // ----- Assert LAST stats unchanged (i.e., staff ignored) -----
        let (token_admin, _) = generate_jwt(data.admin_user.id, data.admin_user.admin);
        let uri_last = format!(
            "/api/modules/{}/assignments/{}/stats",
            data.module.id, data.assignment_last.id
        );
        let req_last = Request::builder()
            .uri(&uri_last)
            .header("Authorization", format!("Bearer {}", token_admin))
            .body(Body::empty())
            .unwrap();
        let resp_last = app.clone().oneshot(req_last).await.unwrap();
        assert_eq!(resp_last.status(), StatusCode::OK);
        let body_last = axum::body::to_bytes(resp_last.into_body(), usize::MAX).await.unwrap();
        let json_last: Value = serde_json::from_slice(&body_last).unwrap();
        let d = &json_last["data"];
        assert_eq!(d["total"], 7);
        assert_eq!(d["graded"], 3);
        assert_eq!(d["pending"], 4);
        assert_eq!(d["late"], 2);
        assert_eq!(d["on_time"], 5);
        assert_eq!(d["ignored"], 0);
        assert_eq!(d["avg_mark"], 68.3);
        assert_eq!(d["median"], 70.0);
        assert_eq!(d["p75"], 80.0);
        assert_eq!(d["best"], 90);
        assert_eq!(d["worst"], 45);
        assert_eq!(d["stddev"], 22.5);
        assert_eq!(d["num_passed"], 2);
        assert_eq!(d["num_failed"], 1);
        assert_eq!(d["num_full_marks"], 0);
        assert_eq!(d["total_marks"], 700);
        assert_eq!(d["num_students_submitted"], 3);

    }

    // --- NEW: Practice and Ignored submissions are excluded from stats (but 'ignored' is reported) ---

    #[tokio::test]
    #[serial]
    async fn test_stats_practice_and_ignored_are_excluded() {
        let (app, app_state, _tmp) = make_test_app_with_storage().await;
        let data = setup_test_data(app_state.db()).await;
        let db = app_state.db();

        // Baseline (counted) student attempts on BEST assignment:
        // s1: 80, 90; s2: 50; s3: 100, 100  → totals=5 counted attempts
        let _ = seed_for_user(
            db, data.module.id, &data.assignment_best, &data.student1,
            &[(80, 100), (90, 100)], &[-60, -30],
        ).await;
        let _ = seed_for_user(
            db, data.module.id, &data.assignment_best, &data.student2,
            &[(50, 100)], &[-40],
        ).await;
        let _ = seed_for_user(
            db, data.module.id, &data.assignment_best, &data.student3,
            &[(100, 100), (100, 100)], &[-10, 10],
        ).await;

        // Add extra PRACTICE attempts (should NOT affect metrics)
        // s1 practice 5%, s2 practice 100% (tempting!), s3 practice 0%
        let _p1 = seed_one_with_flags(
            db, data.module.id, &data.assignment_best, &data.student1,
            99, 5, 100, -5, true, false,
        ).await;
        let _p2 = seed_one_with_flags(
            db, data.module.id, &data.assignment_best, &data.student2,
            98, 100, 100, -3, true, false,
        ).await;
        let _p3 = seed_one_with_flags(
            db, data.module.id, &data.assignment_best, &data.student3,
            97, 0, 100, -1, true, false,
        ).await;

        // Add extra IGNORED attempts (not practice, but ignored flag) → count only toward 'ignored' metric
        // s1 ignored 0%, s2 ignored 100%, s3 ignored 100%
        let _ig1 = seed_one_with_flags(
            db, data.module.id, &data.assignment_best, &data.student1,
            96, 0, 100, -2, false, true,
        ).await;
        let _ig2 = seed_one_with_flags(
            db, data.module.id, &data.assignment_best, &data.student2,
            95, 100, 100, -2, false, true,
        ).await;
        let _ig3 = seed_one_with_flags(
            db, data.module.id, &data.assignment_best, &data.student3,
            94, 100, 100, -2, false, true,
        ).await;

        // Call as lecturer
        let (token, _) = generate_jwt(data.lecturer_user.id, data.lecturer_user.admin);
        let uri = format!(
            "/api/modules/{}/assignments/{}/stats",
            data.module.id, data.assignment_best.id
        );
        let req = Request::builder()
            .uri(&uri)
            .header("Authorization", format!("Bearer {}", token))
            .body(Body::empty())
            .unwrap();
        let response = app.clone().oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let json: Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(json["success"], true);
        let d = &json["data"];

        // Counted totals must IGNORE practice and ignored → only baseline 5 attempts
        assert_eq!(d["total"], 5);
        // 'ignored' reflects ALL student ignored attempts we added (3)
        assert_eq!(d["ignored"], 3);

        // Effective BEST marks using only counted attempts:
        // s1 best=90, s2 best=50, s3 best=100 → avg=80.0, median=90, p75=95, std≈26.5
        assert_eq!(d["graded"], 3);
        assert_eq!(d["pending"], 2); // attempts(5) - graded(3) = 2
        assert_eq!(d["avg_mark"], 80.0);
        assert_eq!(d["median"], 90.0);
        assert_eq!(d["p75"], 95.0);
        assert_eq!(d["best"], 100);
        assert_eq!(d["worst"], 50);
        assert_eq!(d["stddev"], 26.5);

        // pass mark 50 → all 3 pass (50,90,100) → 3/3
        assert_eq!(d["num_passed"], 3);
        assert_eq!(d["num_failed"], 0);
        assert_eq!(d["num_full_marks"], 1);

        // totals computed only over counted attempts (5 * 100)
        assert_eq!(d["total_marks"], 500);
        assert_eq!(d["num_students_submitted"], 3);

    }
}
