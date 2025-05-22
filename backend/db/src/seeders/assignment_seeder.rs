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
        assignment_factory::make_assignment(
            1,
            "Assignment 1",
            Some("Initial practical assignment."),
            "Assignment",
            "2025-05-20 08:00:00",
            "2025-06-01 23:59:59",
        ),
        assignment_factory::make_assignment(
            1,
            "Assignment 2",
            Some("Follow-up assignment."),
            "Assignment",
            "2025-06-01 08:00:00",
            "2025-06-15 23:59:59",
        ),
        assignment_factory::make_assignment(
            2,
            "Design Spec",
            Some("Submit design document."),
            "Practical",
            "2025-06-15 08:00:00",
            "2025-07-01 12:00:00",
        ),
    ];

    for a in assignments {
        sqlx::query(
            "INSERT INTO assignments (
                module_id, name, description, assignment_type,
                available_from, due_date, created_at, updated_at
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(a.module_id)
        .bind(a.name)
        .bind(a.description)
        .bind(a.assignment_type)
        .bind(a.available_from)
        .bind(a.due_date)
        .bind(a.created_at)
        .bind(a.updated_at)
        .execute(pool)
        .await
        .unwrap();
    }

    log::info!("Assignments seeded.");
}
