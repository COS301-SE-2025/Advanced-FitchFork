#[cfg(test)]
mod tests {
    use axum::{
        body::Body,
        http::{Request, StatusCode},
    };
    use chrono::{Duration, TimeZone, Utc};
    use db::models::{
        assignment::{AssignmentType, Model as AssignmentModel},
        assignment_submission::Model as AssignmentSubmissionModel,
        assignment_task::Model as AssignmentTaskModel,
        module::Model as ModuleModel,
        user::Model as UserModel,
        user_module_role::{Model as UserModuleRoleModel, Role},
    };
    use serde_json::{Value, json};
    use std::{collections::HashMap, fs};
    use tower::ServiceExt;
    use util::paths::{config_dir, submission_report_path};

    use crate::helpers::app::make_test_app_with_storage;
    use api::auth::generate_jwt;

    use sea_orm::{ActiveModelTrait, Set};
    use serial_test::serial;

    // ─────────────────────────────────────────────────────────────────────────────
    // Test data bag
    // ─────────────────────────────────────────────────────────────────────────────
    struct TestData {
        lecturer_user: UserModel,
        assistant_user: UserModel,
        tutor_user: UserModel,
        student1: UserModel,
        student2: UserModel,
        module: ModuleModel,
        assignment_last: AssignmentModel,
        // assignment tasks created for assignment_last
        t_ids_by_num: HashMap<i64, i64>, // task_number -> task_id
    }

    fn write_config_json(
        module_id: i64,
        assignment_id: i64,
        grading_policy_lower: &str, // "best" | "last"
        pass_mark: u32,
    ) {
        let config_dir = config_dir(module_id, assignment_id);
        fs::create_dir_all(&config_dir).unwrap();

        let cfg = json!({
            "execution": {},
            "marking": {
                "marking_scheme": "exact",
                "feedback_scheme": "auto",
                "deliminator": "&-=-&",
                "grading_policy": grading_policy_lower,
                "max_attempts": 10,
                "limit_attempts": false,
                "pass_mark": pass_mark
            },
            "project": {"language": "cpp", "submission_mode": "manual"},
            "output": {"stdout": true, "stderr": false, "retcode": false},
            "gatlam": {}
        });

        fs::write(
            config_dir.join("config.json"),
            serde_json::to_string_pretty(&cfg).unwrap(),
        )
        .unwrap();
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

    /// Writes a submission_report.json matching the handler's shape.
    /// "tasks[].name" is set to the **task id as a string** so the route must resolve it.
    fn write_submission_report(
        module_id: i64,
        assignment_id: i64,
        user_id: i64,
        attempt: i64,
        id: i64,
        filename: &str,
        hash: &str,
        created_at: chrono::DateTime<Utc>,
        mark_earned: i64,
        mark_total: i64,
        is_practice: bool,
        tasks_payload: Vec<(
            i64, /* task_id */
            i64, /* task_number */
            i64, /* earned */
            i64, /* total */
        )>,
    ) {
        let path = submission_report_path(module_id, assignment_id, user_id, attempt);

        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).unwrap();
        }

        let tasks_json: Vec<Value> = tasks_payload
            .into_iter()
            .enumerate()
            .map(|(_i, (task_id, task_number, earned, total))| {
                json!({
                  "name": task_id.to_string(),            // ← intentionally task ID string
                  "score": { "earned": earned, "total": total },
                  "subsections": [],
                  "task_number": task_number
                })
            })
            .collect();

        let report = json!({
          "id": id,
          "attempt": attempt,
          "filename": filename,
          "hash": hash,
          "created_at": created_at.to_rfc3339(),
          "updated_at": created_at.to_rfc3339(),
          "mark": { "earned": mark_earned, "total": mark_total },
          "is_practice": is_practice,
          "is_late": false,
          "tasks": tasks_json
        });

        fs::write(path, serde_json::to_string_pretty(&report).unwrap()).unwrap();
    }

    /// Simple seeder (non-practice, non-ignored).
    async fn seed_submission(
        db: &sea_orm::DatabaseConnection,
        module_id: i64,
        assignment: &AssignmentModel,
        user: &UserModel,
        attempt: i64,
        earned: i64,
        total: i64,
        created_offset_min: i64,
        task_rows: Vec<(
            i64, /* task_id */
            i64, /* task_number */
            i64, /* earned */
            i64, /* total */
        )>,
    ) -> AssignmentSubmissionModel {
        let created = assignment.due_date + Duration::minutes(created_offset_min);
        let filename = format!("u{}_a{}.zip", user.id, attempt);
        let hash = format!("h{}_{}", user.id, attempt);

        let sub = AssignmentSubmissionModel::save_file(
            db,
            assignment.id,
            user.id,
            attempt,
            earned,
            total,
            false, // is_practice
            &filename,
            &hash,
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
            sub.id,
            &filename,
            &hash,
            created,
            earned,
            total,
            false, // is_practice in report
            task_rows,
        );

        sub
    }

    /// Seeder that lets you set practice/ignored flags explicitly.
    async fn seed_submission_with_flags(
        db: &sea_orm::DatabaseConnection,
        module_id: i64,
        assignment: &AssignmentModel,
        user: &UserModel,
        attempt: i64,
        earned: i64,
        total: i64,
        created_offset_min: i64,
        is_practice: bool,
        ignored: bool,
        task_rows: Vec<(i64, i64, i64, i64)>,
    ) -> AssignmentSubmissionModel {
        let created = assignment.due_date + Duration::minutes(created_offset_min);
        let filename = format!("u{}_a{}.zip", user.id, attempt);
        let hash = format!("h{}_{}", user.id, attempt);

        let sub = AssignmentSubmissionModel::save_file(
            db,
            assignment.id,
            user.id,
            attempt,
            earned,
            total,
            is_practice,
            &filename,
            &hash,
            b"dummy",
        )
        .await
        .unwrap();

        update_submission_time(db, sub.id, created).await;

        if ignored {
            mark_submission_ignored(db, sub.id, true).await;
        }

        write_submission_report(
            module_id,
            assignment.id,
            user.id,
            attempt,
            sub.id,
            &filename,
            &hash,
            created,
            earned,
            total,
            is_practice,
            task_rows,
        );

        sub
    }

    async fn setup_test_data(db: &sea_orm::DatabaseConnection) -> TestData {
        // Users & module
        let lecturer_user =
            UserModel::create(db, "lecturer1", "lecturer1@test.com", "password1", false)
                .await
                .unwrap();

        let assistant_user =
            UserModel::create(db, "assist1", "assist1@test.com", "passwordA", false)
                .await
                .unwrap();

        let tutor_user = UserModel::create(db, "tutor1", "tutor1@test.com", "passwordT", false)
            .await
            .unwrap();

        let student1 = UserModel::create(db, "student1", "student1@test.com", "password2", false)
            .await
            .unwrap();
        let student2 = UserModel::create(db, "student2", "student2@test.com", "password3", false)
            .await
            .unwrap();

        let module = ModuleModel::create(db, "COS101", 2024, Some("Test Module"), 16)
            .await
            .unwrap();

        // Roles
        UserModuleRoleModel::assign_user_to_module(db, lecturer_user.id, module.id, Role::Lecturer)
            .await
            .unwrap();
        UserModuleRoleModel::assign_user_to_module(
            db,
            assistant_user.id,
            module.id,
            Role::AssistantLecturer,
        )
        .await
        .unwrap();
        UserModuleRoleModel::assign_user_to_module(db, tutor_user.id, module.id, Role::Tutor)
            .await
            .unwrap();
        for s in [&student1, &student2] {
            UserModuleRoleModel::assign_user_to_module(db, s.id, module.id, Role::Student)
                .await
                .unwrap();
        }

        // One assignment → grading policy LAST
        let assignment_last = AssignmentModel::create(
            db,
            module.id,
            "A Last",
            Some("Grades Last"),
            AssignmentType::Assignment,
            Utc.with_ymd_and_hms(2024, 3, 1, 0, 0, 0).unwrap(),
            Utc.with_ymd_and_hms(2024, 3, 31, 23, 59, 59).unwrap(),
        )
        .await
        .unwrap();
        write_config_json(module.id, assignment_last.id, "last", 50);

        // assignment_tasks (task_number → name). IDs are auto; we capture them.
        let mut t_ids_by_num = HashMap::new();

        let t1 =
            AssignmentTaskModel::create(db, assignment_last.id, 1, "FizzBuzz", "run fizz", false)
                .await
                .unwrap();
        let t2 =
            AssignmentTaskModel::create(db, assignment_last.id, 2, "Palindrome", "run pal", false)
                .await
                .unwrap();
        let t3 =
            AssignmentTaskModel::create(db, assignment_last.id, 3, "Sorting", "run sort", false)
                .await
                .unwrap();
        t_ids_by_num.insert(1, t1.id);
        t_ids_by_num.insert(2, t2.id);
        t_ids_by_num.insert(3, t3.id);

        TestData {
            lecturer_user,
            assistant_user,
            tutor_user,
            student1,
            student2,
            module,
            assignment_last,
            t_ids_by_num,
        }
    }

    // ─────────────────────────────────────────────────────────────────────────────
    // Tests
    // ─────────────────────────────────────────────────────────────────────────────

    #[tokio::test]
    #[serial]
    async fn access_control_only_lecturer_and_assistant_can_list_grades() {
        let (app, app_state, _tmp) = make_test_app_with_storage().await;
        let db = app_state.db();
        let data = setup_test_data(db).await;

        // Seed a minimal submission so the handler has something to chew on
        let _ = seed_submission(
            db,
            data.module.id,
            &data.assignment_last,
            &data.student1,
            1,
            10,
            20,
            -10,
            vec![(data.t_ids_by_num[&1], 1, 5, 10)],
        )
        .await;

        // Check role-based access for the list endpoint
        let cases = vec![
            (
                data.lecturer_user.id,
                data.lecturer_user.admin,
                StatusCode::OK,
            ),
            (
                data.assistant_user.id,
                data.assistant_user.admin,
                StatusCode::OK,
            ),
            (
                data.tutor_user.id,
                data.tutor_user.admin,
                StatusCode::FORBIDDEN,
            ),
            (data.student1.id, data.student1.admin, StatusCode::FORBIDDEN),
        ];

        for (uid, is_admin, expected) in cases {
            let (token, _) = generate_jwt(uid, is_admin);
            let uri = format!(
                "/api/modules/{}/assignments/{}/grades?sort=username",
                data.module.id, data.assignment_last.id
            );
            let req = Request::builder()
                .uri(&uri)
                .header("Authorization", format!("Bearer {}", token))
                .body(Body::empty())
                .unwrap();
            let resp = app.clone().oneshot(req).await.unwrap();
            assert_eq!(resp.status(), expected, "uid {} wrong status", uid);
        }

        // Unauthenticated should be 401
        let uri = format!(
            "/api/modules/{}/assignments/{}/grades?sort=username",
            data.module.id, data.assignment_last.id
        );
        let req = Request::builder().uri(&uri).body(Body::empty()).unwrap();
        let resp = app.clone().oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    #[serial]
    async fn access_control_only_lecturer_and_assistant_can_export_csv() {
        let (app, app_state, _tmp) = make_test_app_with_storage().await;
        let db = app_state.db();
        let data = setup_test_data(db).await;

        // Minimal seed
        let _ = seed_submission(
            db,
            data.module.id,
            &data.assignment_last,
            &data.student2,
            1,
            18,
            20,
            -5,
            vec![(data.t_ids_by_num[&2], 2, 9, 10)],
        )
        .await;

        let cases = vec![
            (
                data.lecturer_user.id,
                data.lecturer_user.admin,
                StatusCode::OK,
            ),
            (
                data.assistant_user.id,
                data.assistant_user.admin,
                StatusCode::OK,
            ),
            (
                data.tutor_user.id,
                data.tutor_user.admin,
                StatusCode::FORBIDDEN,
            ),
            (data.student1.id, data.student1.admin, StatusCode::FORBIDDEN),
        ];

        for (uid, is_admin, expected) in cases {
            let (token, _) = generate_jwt(uid, is_admin);
            let uri = format!(
                "/api/modules/{}/assignments/{}/grades/export",
                data.module.id, data.assignment_last.id
            );
            let req = Request::builder()
                .uri(&uri)
                .header("Authorization", format!("Bearer {}", token))
                .body(Body::empty())
                .unwrap();
            let resp = app.clone().oneshot(req).await.unwrap();
            assert_eq!(resp.status(), expected, "uid {} wrong status", uid);
        }

        // Unauthenticated should be 401
        let uri = format!(
            "/api/modules/{}/assignments/{}/grades/export",
            data.module.id, data.assignment_last.id
        );
        let req = Request::builder().uri(&uri).body(Body::empty()).unwrap();
        let resp = app.clone().oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    #[serial]
    async fn list_grades_happy_last_policy_with_task_name_mapping() {
        let (app, app_state, _tmp) = make_test_app_with_storage().await;
        let db = app_state.db();
        let data = setup_test_data(db).await;

        // student1 → attempt 1 older, attempt 2 newer → LAST picks attempt 2
        // Report has "name" = task_id string; handler must map to real names.
        let _s1a1 = seed_submission(
            db,
            data.module.id,
            &data.assignment_last,
            &data.student1,
            1,
            10,
            27,
            -200,
            vec![
                (data.t_ids_by_num[&1], 1, 3, 9),
                (data.t_ids_by_num[&2], 2, 4, 9),
            ],
        )
        .await;

        let _s1a2 = seed_submission(
            db,
            data.module.id,
            &data.assignment_last,
            &data.student1,
            2,
            21,
            27,
            -100,
            vec![
                (data.t_ids_by_num[&1], 1, 8, 9),
                (data.t_ids_by_num[&2], 2, 8, 9),
                (data.t_ids_by_num[&3], 3, 5, 9),
            ],
        )
        .await;

        // student2 → one attempt
        let _s2a1 = seed_submission(
            db,
            data.module.id,
            &data.assignment_last,
            &data.student2,
            1,
            24,
            27,
            -50,
            vec![
                (data.t_ids_by_num[&1], 1, 9, 9),
                (data.t_ids_by_num[&2], 2, 7, 9),
            ],
        )
        .await;

        let (token, _) = generate_jwt(data.lecturer_user.id, data.lecturer_user.admin);
        let uri = format!(
            "/api/modules/{}/assignments/{}/grades?sort=username",
            data.module.id, data.assignment_last.id
        );
        let req = Request::builder()
            .uri(&uri)
            .header("Authorization", format!("Bearer {}", token))
            .body(Body::empty())
            .unwrap();

        let resp = app.clone().oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);

        let body = axum::body::to_bytes(resp.into_body(), usize::MAX)
            .await
            .unwrap();
        let json: Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(json["success"], true, "response should be success=true");

        let grades = json["data"]["grades"].as_array().unwrap();
        assert_eq!(grades.len(), 2, "one row per student");

        // rows sorted by username asc → student1, then student2
        let row1 = &grades[0];
        assert_eq!(row1["username"], "student1");
        let s1_score = row1["score"].as_f64().unwrap();
        assert!((s1_score - (21.0 * 100.0 / 27.0)).abs() < 0.2);

        // task names resolved
        let tasks1 = row1["tasks"].as_array().unwrap();
        let names1: Vec<String> = tasks1
            .iter()
            .map(|t| t["name"].as_str().unwrap_or("").to_string())
            .collect();
        for expected in ["FizzBuzz", "Palindrome", "Sorting"] {
            assert!(
                names1.contains(&expected.to_string()),
                "expected task name {} in tasks",
                expected
            );
        }

        // student2
        let row2 = &grades[1];
        assert_eq!(row2["username"], "student2");
        let s2_score = row2["score"].as_f64().unwrap();
        assert!((s2_score - (24.0 * 100.0 / 27.0)).abs() < 0.2);

        let tasks2 = row2["tasks"].as_array().unwrap();
        let names2: Vec<String> = tasks2
            .iter()
            .map(|t| t["name"].as_str().unwrap_or("").to_string())
            .collect();
        for expected in ["FizzBuzz", "Palindrome"] {
            assert!(
                names2.contains(&expected.to_string()),
                "expected task name {} in tasks",
                expected
            );
        }
    }

    #[tokio::test]
    #[serial]
    async fn list_grades_excludes_practice_and_ignored() {
        let (app, app_state, _tmp) = make_test_app_with_storage().await;
        let db = app_state.db();
        let data = setup_test_data(db).await;

        // student1:
        //  a1: valid (kept)
        //  a2: practice (excluded)
        //  a3: ignored (excluded)
        let _s1a1_valid = seed_submission_with_flags(
            db,
            data.module.id,
            &data.assignment_last,
            &data.student1,
            1,
            10,
            20,
            -300,
            false, /* is_practice */
            false, /* ignored */
            vec![(data.t_ids_by_num[&1], 1, 5, 10)],
        )
        .await;

        let _s1a2_practice = seed_submission_with_flags(
            db,
            data.module.id,
            &data.assignment_last,
            &data.student1,
            2,
            18,
            20,
            -200,
            true,  /* is_practice */
            false, /* ignored */
            vec![(data.t_ids_by_num[&2], 2, 9, 10)],
        )
        .await;

        let _s1a3_ignored = seed_submission_with_flags(
            db,
            data.module.id,
            &data.assignment_last,
            &data.student1,
            3,
            19,
            20,
            -100,
            false, /* is_practice */
            true,  /* ignored */
            vec![(data.t_ids_by_num[&3], 3, 9, 10)],
        )
        .await;

        // student2: only ignored → should not appear in final list
        let _s2a1_ignored = seed_submission_with_flags(
            db,
            data.module.id,
            &data.assignment_last,
            &data.student2,
            1,
            20,
            20,
            -50,
            false, /* is_practice */
            true,  /* ignored */
            vec![(data.t_ids_by_num[&1], 1, 10, 10)],
        )
        .await;

        let (token, _) = generate_jwt(data.lecturer_user.id, data.lecturer_user.admin);
        let uri = format!(
            "/api/modules/{}/assignments/{}/grades?sort=username",
            data.module.id, data.assignment_last.id
        );
        let req = Request::builder()
            .uri(&uri)
            .header("Authorization", format!("Bearer {}", token))
            .body(Body::empty())
            .unwrap();

        let resp = app.clone().oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);

        let body = axum::body::to_bytes(resp.into_body(), usize::MAX)
            .await
            .unwrap();
        let json: Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(json["success"], true);

        let grades = json["data"]["grades"].as_array().unwrap();

        // Only student1 appears (student2 has only ignored)
        assert_eq!(grades.len(), 1);
        assert_eq!(grades[0]["username"], "student1");

        // Chosen submission must be the non-practice, non-ignored one (attempt 1)
        let s1_score = grades[0]["score"].as_f64().unwrap();
        assert!(
            (s1_score - 50.0).abs() < 0.2,
            "expected ≈50, got {}",
            s1_score
        );

        // Task-name mapping still OK (FizzBuzz only; the others belonged to excluded attempts)
        let tasks = grades[0]["tasks"].as_array().unwrap();
        let names: std::collections::HashSet<_> =
            tasks.iter().filter_map(|t| t["name"].as_str()).collect();
        assert!(names.contains("FizzBuzz"));
        assert!(!names.contains("Palindrome"));
        assert!(!names.contains("Sorting"));
    }

    #[tokio::test]
    #[serial]
    async fn list_grades_invalid_sort_returns_400() {
        let (app, app_state, _tmp) = make_test_app_with_storage().await;
        let db = app_state.db();
        let data = setup_test_data(db).await;

        let (token, _) = generate_jwt(data.lecturer_user.id, data.lecturer_user.admin);
        let uri = format!(
            "/api/modules/{}/assignments/{}/grades?sort=not_a_field",
            data.module.id, data.assignment_last.id
        );
        let req = Request::builder()
            .uri(&uri)
            .header("Authorization", format!("Bearer {}", token))
            .body(Body::empty())
            .unwrap();

        let resp = app.clone().oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    #[serial]
    async fn list_grades_username_query_filters() {
        let (app, app_state, _tmp) = make_test_app_with_storage().await;
        let db = app_state.db();
        let data = setup_test_data(db).await;

        // minimal: one submission each + reports
        let _ = seed_submission(
            db,
            data.module.id,
            &data.assignment_last,
            &data.student1,
            1,
            10,
            20,
            -10,
            vec![(data.t_ids_by_num[&1], 1, 5, 10)],
        )
        .await;

        let _ = seed_submission(
            db,
            data.module.id,
            &data.assignment_last,
            &data.student2,
            1,
            18,
            20,
            -5,
            vec![(data.t_ids_by_num[&2], 2, 9, 10)],
        )
        .await;

        let (token, _) = generate_jwt(data.lecturer_user.id, data.lecturer_user.admin);
        // filter only student2
        let uri = format!(
            "/api/modules/{}/assignments/{}/grades?query=student2",
            data.module.id, data.assignment_last.id
        );
        let req = Request::builder()
            .uri(&uri)
            .header("Authorization", format!("Bearer {}", token))
            .body(Body::empty())
            .unwrap();

        let resp = app.clone().oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);

        let body = axum::body::to_bytes(resp.into_body(), usize::MAX)
            .await
            .unwrap();
        let json: Value = serde_json::from_slice(&body).unwrap();
        let grades = json["data"]["grades"].as_array().unwrap();
        assert_eq!(grades.len(), 1);
        assert_eq!(grades[0]["username"], "student2");
    }

    #[tokio::test]
    #[serial]
    async fn export_grades_csv_union_of_tasks_and_percentages() {
        let (app, app_state, _tmp) = make_test_app_with_storage().await;
        let db = app_state.db();
        let data = setup_test_data(db).await;

        // student1 (last attempt is #2)
        let _s1a1 = seed_submission(
            db,
            data.module.id,
            &data.assignment_last,
            &data.student1,
            1,
            10,
            27,
            -200,
            vec![(data.t_ids_by_num[&1], 1, 5, 9)],
        )
        .await;
        let _s1a2 = seed_submission(
            db,
            data.module.id,
            &data.assignment_last,
            &data.student1,
            2,
            21,
            27,
            -100,
            vec![
                (data.t_ids_by_num[&1], 1, 8, 9), // 88.888…
                (data.t_ids_by_num[&2], 2, 8, 9), // 88.888…
                (data.t_ids_by_num[&3], 3, 5, 9), // 55.555…
            ],
        )
        .await;

        // student2 (only attempt). Leave out task 3 to test empty cells.
        let _s2a1 = seed_submission(
            db,
            data.module.id,
            &data.assignment_last,
            &data.student2,
            1,
            24,
            27,
            -50,
            vec![
                (data.t_ids_by_num[&1], 1, 9, 9), // 100
                (data.t_ids_by_num[&2], 2, 7, 9), // 77.777…
            ],
        )
        .await;

        let (token, _) = generate_jwt(data.lecturer_user.id, data.lecturer_user.admin);
        let uri = format!(
            "/api/modules/{}/assignments/{}/grades/export",
            data.module.id, data.assignment_last.id
        );
        let req = Request::builder()
            .uri(&uri)
            .header("Authorization", format!("Bearer {}", token))
            .body(Body::empty())
            .unwrap();

        let resp = app.clone().oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        // content-type text/csv
        let ct = resp
            .headers()
            .get("content-type")
            .and_then(|h| h.to_str().ok())
            .unwrap_or("");
        assert!(ct.starts_with("text/csv"));

        let body = axum::body::to_bytes(resp.into_body(), usize::MAX)
            .await
            .unwrap();
        let csv = String::from_utf8(body.to_vec()).unwrap();

        // Parse CSV roughly (no quoted commas expected in our test)
        let mut lines = csv.lines();
        let header = lines.next().unwrap();
        let headers: Vec<&str> = header.split(',').collect();
        assert!(headers.len() >= 2);
        assert_eq!(headers[0], "username");
        assert_eq!(headers[1], "score");

        // The remaining headers should be the union of task names.
        let header_tasks: Vec<&str> = headers[2..].to_vec();
        let set = header_tasks
            .iter()
            .cloned()
            .collect::<std::collections::HashSet<_>>();
        for expected in ["FizzBuzz", "Palindrome", "Sorting"] {
            assert!(set.contains(expected), "missing task column {}", expected);
        }

        // Index lookup
        let idx_of = |name: &str| {
            header_tasks
                .iter()
                .position(|h| *h == name)
                .map(|i| i + 2)
                .unwrap()
        };

        // Read rows into a map username -> Vec<String>
        let mut rows: HashMap<String, Vec<String>> = HashMap::new();
        for line in lines {
            if line.trim().is_empty() {
                continue;
            }
            let cols: Vec<String> = line.split(',').map(|s| s.to_string()).collect();
            rows.insert(cols[0].clone(), cols);
        }

        let r1 = rows.get("student1").expect("student1 row");
        let r2 = rows.get("student2").expect("student2 row");

        // Final scores in column 1
        let s1_final: f64 = r1[1].parse().unwrap();
        let s2_final: f64 = r2[1].parse().unwrap();
        assert!((s1_final - (21.0 * 100.0 / 27.0)).abs() < 0.2);
        assert!((s2_final - (24.0 * 100.0 / 27.0)).abs() < 0.2);

        // Per-task percentages (as raw to_string floats; accept small epsilon)
        let f = |v: &str| v.parse::<f64>().unwrap_or(0.0);

        let i_fb = idx_of("FizzBuzz");
        let i_pal = idx_of("Palindrome");
        let i_sort = idx_of("Sorting");

        // student1 has all three: 88.888.., 88.888.., 55.555..
        assert!((f(&r1[i_fb]) - (8.0 * 100.0 / 9.0)).abs() < 0.2);
        assert!((f(&r1[i_pal]) - (8.0 * 100.0 / 9.0)).abs() < 0.2);
        assert!((f(&r1[i_sort]) - (5.0 * 100.0 / 9.0)).abs() < 0.2);

        // student2 has only first two; Sorting should be empty → ensure the cell is actually empty string.
        assert!(
            r2[i_sort].is_empty(),
            "expected empty cell for missing Sorting"
        );
        assert!((f(&r2[i_fb]) - 100.0).abs() < 0.2);
        assert!((f(&r2[i_pal]) - (7.0 * 100.0 / 9.0)).abs() < 0.2);
    }
}
