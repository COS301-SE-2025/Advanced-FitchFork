use serde::{Deserialize, Serialize};
use sqlx::Arguments;
use sqlx::FromRow;
use sqlx::SqlitePool;
//ENUM

/// Represents the type of an assignment, either a normal assignment or a practical.
#[derive(Debug, Serialize, Deserialize, sqlx::Type, PartialEq)]
#[sqlx(type_name = "TEXT")]
pub enum AssignmentType {
    Assignment,
    Practical,
}

/// Represents an assignment entity with metadata.
#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct Assignment {
    pub id: i64,
    pub module_id: i64,
    pub name: String,
    pub description: Option<String>,
    pub assignment_type: AssignmentType,
    pub available_from: String,
    pub due_date: String,
    pub created_at: String,
    pub updated_at: String,
}

impl Assignment {
    /// Create a new assignment in the database.
    ///
    /// # Arguments
    ///
    /// * `pool` - Optional reference to an SQLite connection pool.
    /// * `module_id` - The ID of the module this assignment belongs to.
    /// * `name` - The name/title of the assignment.
    /// * `description` - Optional detailed description of the assignment.
    /// * `assignment_type` - The type of assignment (Assignment or Practical).
    /// * `available_from` - Start datetime string when the assignment is available.
    /// * `due_date` - Due datetime string for the assignment.
    ///
    /// # Returns
    ///
    /// Returns the created `Assignment` record on success.
    pub async fn create(
        pool: Option<&SqlitePool>,
        module_id: i64,
        name: &str,
        description: Option<&str>,
        assignment_type: AssignmentType,
        available_from: &str,
        due_date: &str,
    ) -> sqlx::Result<Self> {
        let pool = pool.unwrap_or_else(|| crate::pool::get());
        let record = sqlx::query_as::<_, Assignment>(
            r#"
            INSERT INTO assignments (
                module_id, name, description, assignment_type, 
                available_from, due_date
            )
            VALUES (?, ?, ?, ?, ?, ?)
            RETURNING id, module_id, name, description, assignment_type, 
                      available_from, due_date, created_at, updated_at
            "#,
        )
        .bind(module_id)
        .bind(name)
        .bind(description)
        .bind(assignment_type) //sqlx automatically converts from enums apparently so thats cool
        .bind(available_from)
        .bind(due_date)
        .fetch_one(pool)
        .await?;

        Ok(record)
    }

    /// Delete an assignment from the database by its ID.
    ///
    /// # Arguments
    ///
    /// * `pool` - Optional reference to an SQLite connection pool.
    /// * `id` - The ID of the assignment to delete.
    /// * `mid` - The ID of the module the assignment is related to
    pub async fn delete_by_id(pool: Option<&SqlitePool>, id: i64, mid: i64) -> sqlx::Result<bool> {
        let pool = pool.unwrap_or_else(|| crate::pool::get());
        let result = sqlx::query("DELETE FROM assignments WHERE id = ? AND module_id = ?")
            .bind(id)
            .bind(mid)
            .execute(pool)
            .await?;

        Ok(result.rows_affected() > 0)
    }

    /// Retrieve all assignments from the database.
    ///
    /// # Arguments
    ///
    /// * `pool` - Optional reference to an SQLite connection pool.
    ///
    /// # Returns
    ///
    /// Returns a vector of all `Assignment` records.
    pub async fn get_all(pool: Option<&SqlitePool>) -> sqlx::Result<Vec<Self>> {
        let pool = pool.unwrap_or_else(|| crate::pool::get());
        sqlx::query_as::<_, Assignment>("SELECT * FROM assignments")
            .fetch_all(pool)
            .await
    }

    /// Retrieve a specific assignment by its ID.
    ///
    /// # Arguments
    ///
    /// * `pool` - Optional reference to an SQLite connection pool.
    /// * `id` - The ID of the assignment to retrieve.
    /// * `mid` - The ID of the module the assignment is related to
    ///
    /// # Returns
    ///
    /// Returns an `Option<Assignment>` which is `Some` if found or `None` if not.
    pub async fn get_by_id(
        pool: Option<&SqlitePool>,
        id: i64,
        mid: i64,
    ) -> sqlx::Result<Option<Self>> {
        let pool = pool.unwrap_or_else(|| crate::pool::get());
        sqlx::query_as::<_, Assignment>("SELECT * FROM assignments WHERE id = ? AND module_id = ?")
            .bind(id)
            .bind(mid)
            .fetch_optional(pool)
            .await
    }

    /// Edit a specific assignment by its ID.
    ///
    /// The `updated_at` field is automatically set to the current timestamp during the update.
    ///
    /// # Arguments
    ///
    /// * `pool` - Optional reference to an SQLite connection pool. If `None`, the default pool is used.
    /// * `id` - The ID of the assignment to be edited.
    /// * `module_id` - The ID of the module the assignment belongs to.
    /// * `name` - The new name of the assignment.
    /// * `description` - An optional new description for the assignment.
    /// * `assignment_type` - The new type of the assignment (Practical/Assignment).
    /// * `available_from` - The new availability date for the assignment (ISO 8601 string).
    /// * `due_date` - The new due date for the assignment (ISO 8601 string).
    ///
    /// # Returns
    ///
    /// Returns a `Result` containing the updated `Assignment` if successful, or a `sqlx::Error` if the operation fails.

    pub async fn edit(
        pool: Option<&SqlitePool>,
        id: i64,
        module_id: i64,
        name: &str,
        description: Option<&str>,
        assignment_type: AssignmentType,
        available_from: &str,
        due_date: &str,
    ) -> sqlx::Result<Self> {
        let pool = pool.unwrap_or_else(|| crate::pool::get());
        println!("Editing assignment {} in module {}", id, module_id);
        let record = sqlx::query_as::<_, Assignment>(
            "
            UPDATE assignments
            SET name = ?, description = ?, assignment_type = ?, 
                available_from = ?, due_date = ?, updated_at = CURRENT_TIMESTAMP
            WHERE id = ? AND module_id = ?
            RETURNING id, module_id, name, description, assignment_type, 
                      available_from, due_date, created_at, updated_at
            ",
        )
        .bind(name)
        .bind(description)
        .bind(assignment_type)
        .bind(available_from)
        .bind(due_date)
        .bind(id)
        .bind(module_id)
        .fetch_one(pool)
        .await?;

        Ok(record)
    }

    /// Filters assignments based on various optional criteria, with pagination and sorting support.
    ///
    /// # Arguments
    ///
    /// * `pool` - Optional reference to an SQLite connection pool. If `None`, the default pool is used.
    /// * `page` - The page number for pagination (1-based).
    /// * `length` - The number of assignments to return per page.
    /// * `sort` - Optional comma-separated string specifying sorting fields and directions.
    ///            Fields can be prefixed with '-' for descending order (e.g., "-due_date").
    ///            Allowed fields: `name`, `due_date`, `available_from`, `assignment_type`, `created_at`, `updated_at`.
    /// * `query` - Optional substring to match against both `name` and `description` (case-insensitive).
    /// * `name` - Optional substring filter for assignment name (case-insensitive).
    /// * `assignment_type` - Optional filter by assignment type (e.g., "Practical" or "Assignment").
    /// * `available_before` - Optional ISO 8601 date string to filter assignments available before this date.
    /// * `available_after` - Optional ISO 8601 date string to filter assignments available on or after this date.
    /// * `due_before` - Optional ISO 8601 date string to filter assignments due before this date.
    /// * `due_after` - Optional ISO 8601 date string to filter assignments due on or after this date.
    ///
    /// # Returns
    ///
    /// Returns a `Result` containing a `Vec<Assignment>` that matches the filter criteria, or a `sqlx::Error` if the query fails.

    pub async fn filter(
        pool: Option<&SqlitePool>,
        page: i32,
        length: i32,
        sort: Option<String>,
        query: Option<String>,
        name: Option<String>,
        assignment_type: Option<String>,
        available_before: Option<String>,
        available_after: Option<String>,
        due_before: Option<String>,
        due_after: Option<String>,
    ) -> sqlx::Result<Vec<Self>> {
        let pool = pool.unwrap_or_else(|| crate::pool::get());
        let offset = (page - 1) * length;

        let mut sql = String::from("SELECT * FROM assignments WHERE 1=1");
        let mut args = sqlx::sqlite::SqliteArguments::default();

        if let Some(ref n) = name {
            sql.push_str(" AND LOWER(name) LIKE ?");
            args.add(format!("%{}%", n.to_lowercase()));
        }

        if let Some(ref a_type) = assignment_type {
            sql.push_str(" AND assignment_type = ?");
            args.add(a_type);
        }

        if let Some(ref before) = available_before {
            sql.push_str(" AND available_from < ?");
            args.add(before);
        }

        if let Some(ref after) = available_after {
            sql.push_str(" AND available_from >= ?");
            args.add(after);
        }

        if let Some(ref before) = due_before {
            sql.push_str(" AND due_date < ?");
            args.add(before);
        }

        if let Some(ref after) = due_after {
            sql.push_str(" AND due_date >= ?");
            args.add(after);
        }

        if let Some(ref q) = query {
            sql.push_str(" AND (LOWER(name) LIKE ? OR LOWER(description) LIKE ?)");
            let pattern = format!("%{}%", q.to_lowercase());
            args.add(pattern.clone());
            args.add(pattern);
        }

        if let Some(sort_str) = sort {
            let mut order_clauses = Vec::new();

            for field in sort_str.split(',') {
                let trimmed = field.trim();
                if trimmed.is_empty() {
                    continue;
                }
                let (field_name, direction) = if trimmed.starts_with('-') {
                    (&trimmed[1..], "DESC")
                } else {
                    (trimmed, "ASC")
                };

                order_clauses.push(format!("{} {}", field_name, direction));
            }

            if !order_clauses.is_empty() {
                sql.push_str(" ORDER BY ");
                sql.push_str(&order_clauses.join(", "));
            }
        }

        sql.push_str(" LIMIT ? OFFSET ?");
        args.add(length);
        args.add(offset);

        sqlx::query_as_with::<_, Self, _>(&sql, args)
            .fetch_all(pool)
            .await
    }
}

#[cfg(test)]
mod tests {
    use crate::create_test_db;
    use crate::delete_database;
    use crate::models::assignment::{Assignment, AssignmentType};
    use crate::models::module::Module;

    #[tokio::test]
    async fn test_asignment_update() {
        let pool = create_test_db(Some("test_assignment_update.db")).await;
        let module = Module::create(
            Some(&pool),
            "COS333",
            2025,
            Some("Software Engineering"),
            16,
        )
        .await
        .unwrap();
        let assignment = Assignment::create(
            Some(&pool),
            module.id,
            "Initial Assignment",
            Some("Basic programming tasks"),
            AssignmentType::Assignment,
            "2025-01-01T00:00:00Z",
            "2025-01-15T23:59:59Z",
        )
        .await
        .unwrap();
        assert_eq!(assignment.name, "Initial Assignment");

        let updated_assignment = Assignment::edit(
            Some(&pool),
            1,
            1,
            "Updated Assignment",
            Some("Updated description"),
            AssignmentType::Practical,
            "2025-01-02T00:00:00Z",
            "2025-01-16T23:59:59Z",
        )
        .await
        .unwrap();
        assert_eq!(updated_assignment.name, "Updated Assignment");
        assert_eq!(
            updated_assignment.description,
            Some("Updated description".to_string())
        );
        assert_eq!(
            updated_assignment.assignment_type,
            AssignmentType::Practical
        );
        assert_eq!(updated_assignment.available_from, "2025-01-02T00:00:00Z");
        assert_eq!(updated_assignment.due_date, "2025-01-16T23:59:59Z");
        pool.close().await;
        delete_database("test_assignment_update.db");
    }

    #[tokio::test]
    async fn test_assignment_create_and_find() {
        let pool = create_test_db(Some("test_assignment_create_and_find.db")).await;

        // Create a module for the assignment to belong to
        let module = Module::create(
            Some(&pool),
            "COS333",
            2025,
            Some("Software Engineering"),
            16,
        )
        .await
        .unwrap();

        // Create an assignment
        let assignment = Assignment::create(
            Some(&pool),
            module.id,
            "Assignment 1",
            Some("Implement a CRUD app"),
            AssignmentType::Assignment,
            "2025-04-01T00:00:00Z",
            "2025-04-15T23:59:59Z",
        )
        .await
        .unwrap();

        assert_eq!(assignment.module_id, module.id);
        assert_eq!(assignment.name, "Assignment 1");
        assert_eq!(assignment.assignment_type, AssignmentType::Assignment);
        assert_eq!(assignment.available_from, "2025-04-01T00:00:00Z");
        assert_eq!(assignment.due_date, "2025-04-15T23:59:59Z");

        // Fetch it again by ID
        let found = Assignment::get_by_id(Some(&pool), assignment.id, module.id)
            .await
            .unwrap();
        assert!(found.is_some());
        let found = found.unwrap();
        assert_eq!(found.name, "Assignment 1");

        pool.close().await;
        delete_database("test_assignment_create_and_find.db");
    }

    #[tokio::test]
    async fn test_assignment_deletion() {
        let pool = create_test_db(Some("test_assignment_deletion.db")).await;

        // Create module and assignment
        let module = Module::create(Some(&pool), "COS221", 2025, Some("Databases"), 16)
            .await
            .unwrap();

        let assignment = Assignment::create(
            Some(&pool),
            module.id,
            "SQL Practice",
            Some("Write SQL queries"),
            AssignmentType::Practical,
            "2025-03-01T00:00:00Z",
            "2025-03-10T23:59:59Z",
        )
        .await
        .unwrap();

        // Delete it
        Assignment::delete_by_id(Some(&pool), assignment.id, module.id)
            .await
            .unwrap();

        // Confirm it's gone
        let found = Assignment::get_by_id(Some(&pool), assignment.id, module.id)
            .await
            .unwrap();
        assert!(found.is_none());

        pool.close().await;
        delete_database("test_assignment_deletion.db");
    }

    #[tokio::test]
    async fn test_assignment_get_all() {
        let pool = create_test_db(Some("test_assignment_get_all.db")).await;

        let module = Module::create(Some(&pool), "COS212", 2025, Some("Data Structures"), 16)
            .await
            .unwrap();

        Assignment::create(
            Some(&pool),
            module.id,
            "List Implementations",
            None,
            AssignmentType::Assignment,
            "2025-02-01T00:00:00Z",
            "2025-02-15T23:59:59Z",
        )
        .await
        .unwrap();

        Assignment::create(
            Some(&pool),
            module.id,
            "Linked List Lab",
            Some("Implement a linked list"),
            AssignmentType::Practical,
            "2025-02-16T00:00:00Z",
            "2025-02-28T23:59:59Z",
        )
        .await
        .unwrap();

        let all = Assignment::get_all(Some(&pool)).await.unwrap();
        assert_eq!(all.len(), 2);

        pool.close().await;
        delete_database("test_assignment_get_all.db");
    }
}
