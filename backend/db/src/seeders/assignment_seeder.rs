use crate::factories::assignment_factory;
use crate::models::assignment_files::AssignmentFiles;
use sqlx::SqlitePool;

pub async fn seed(pool: &SqlitePool) {
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

    let mut assignment_ids = Vec::new();

    for a in assignments {
        let rec = sqlx::query(
            "INSERT INTO assignments (
                module_id, name, description, assignment_type,
                available_from, due_date, created_at, updated_at
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(a.module_id)
        .bind(a.name.clone())
        .bind(a.description.clone())
        .bind(a.assignment_type)
        .bind(a.available_from.clone())
        .bind(a.due_date.clone())
        .bind(a.created_at.clone())
        .bind(a.updated_at.clone())
        .execute(pool)
        .await
        .unwrap();

        let id = rec.last_insert_rowid();
        assignment_ids.push((id, a.name));
    }

    log::info!("Assignments seeded.");
    log::info!("Seeding assignment files...");

    for (id, name) in assignment_ids {
        let zip_bytes = vec![0x50, 0x4B, 0x03, 0x04, 0x14, 0x00]; // fake ZIP file header
        let filename: String = format!("{}_file.zip", name.replace(" ", "_").to_lowercase());

        AssignmentFiles::create_and_store_file(None, id, &filename, &zip_bytes)
            .await
            .unwrap();
    }

    log::info!("Assignment files seeded.");
}
