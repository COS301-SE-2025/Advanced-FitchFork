use crate::factories::assignment_factory;
use sqlx::SqlitePool;

pub async fn seed(pool: &SqlitePool) {
    let count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM assignments")
        .fetch_one(pool)
        .await
        .unwrap();

    if count.0 > 0 {
        log::info!("Skipping assignment seeder.");
        return;
    }

    log::info!("Seeding assignments...");

    let assignments = vec![
        assignment_factory::make_assignment(1, "Assignment 1", Some("2025-06-01 23:59:59")),
        assignment_factory::make_assignment(1, "Assignment 2", Some("2025-06-15 23:59:59")),
        assignment_factory::make_assignment(2, "Design Spec", Some("2025-07-01 12:00:00")),
    ];

    for assignment in assignments {
        sqlx::query("INSERT INTO assignments (module_id, name, due_date) VALUES (?, ?, ?)")
            .bind(assignment.module_id)
            .bind(assignment.name)
            .bind(assignment.due_date)
            .execute(pool)
            .await
            .unwrap();
    }

    log::info!("Assignments seeded.");
}
