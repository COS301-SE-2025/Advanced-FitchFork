use serde::{Deserialize, Serialize};
use sqlx::Arguments;
use sqlx::{FromRow, SqlitePool};

/// Represents a university module.
#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct Module {
    pub id: i64,
    pub code: String,
    pub year: i32,
    pub description: Option<String>,
    pub credits: i32,
    pub created_at: String,
    pub updated_at: String,
}

impl Module {
    /// Creates a new module record in the database.
    ///
    /// # Arguments
    ///
    /// * `pool` - Optional reference to a `SqlitePool`. If `None`, the default pool is used.
    /// * `code` - The module code (e.g., "COS301").
    /// * `year` - The year the module is offered.
    /// * `description` - An optional description of the module.
    /// * `credits` - The number of credits for the module.
    ///
    /// # Returns
    ///
    /// Returns the newly created `Module` instance.
    ///
    /// # Errors
    ///
    /// Returns a `sqlx::Error` if the insert fails.
    pub async fn create(
        pool: Option<&SqlitePool>,
        code: &str,
        year: i32,
        description: Option<&str>,
        credits: i32,
    ) -> sqlx::Result<Self> {
        let pool = pool.unwrap_or_else(|| crate::pool::get());
        let record = sqlx::query_as::<_, Module>(
            r#"
            INSERT INTO modules (code, year, description, credits)
            VALUES (?, ?, ?, ?)
            RETURNING id, code, year, description, credits, created_at, updated_at
            "#,
        )
        .bind(code)
        .bind(year)
        .bind(description)
        .bind(credits)
        .fetch_one(pool)
        .await?;

        Ok(record)
    }

    /// Deletes a module by its ID.
    ///
    /// # Arguments
    ///
    /// * `pool` - Optional reference to a `SqlitePool`. If `None`, the default pool is used.
    /// * `id` - The ID of the module to delete.
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` if the deletion was successful.
    ///
    /// # Errors
    ///
    /// Returns a `sqlx::Error` if the deletion fails.
    pub async fn delete_by_id(pool: Option<&SqlitePool>, id: i64) -> sqlx::Result<()> {
        let pool = pool.unwrap_or_else(|| crate::pool::get());
        sqlx::query("DELETE FROM modules WHERE id = ?")
            .bind(id)
            .execute(pool)
            .await?;
        Ok(())
    }

    /// Retrieves a module by its ID.
    ///
    /// # Arguments
    ///
    /// * `pool` - Optional reference to a `SqlitePool`. If `None`, the default pool is used.
    /// * `id` - The ID of the module to retrieve.
    ///
    /// # Returns
    ///
    /// An `Option<Module>` if found, or `None` if no matching module exists.
    ///
    /// # Errors
    ///
    /// Returns a `sqlx::Error` if the query fails.
    pub async fn get_by_id(pool: Option<&SqlitePool>, id: i64) -> sqlx::Result<Option<Self>> {
        let pool = pool.unwrap_or_else(|| crate::pool::get());
        sqlx::query_as::<_, Module>("SELECT * FROM modules WHERE id = ?")
            .bind(id)
            .fetch_optional(pool)
            .await
    }

    /// Retrieves all modules from the database.
    ///
    /// # Arguments
    ///
    /// * `pool` - Optional reference to a `SqlitePool`. If `None`, the default pool is used.
    ///
    /// # Returns
    ///
    /// A `Vec<Module>` containing all modules in the database.
    ///
    /// # Errors
    ///
    /// Returns a `sqlx::Error` if the query fails.
    pub async fn get_all(pool: Option<&SqlitePool>) -> sqlx::Result<Vec<Self>> {
        let pool = pool.unwrap_or_else(|| crate::pool::get());
        sqlx::query_as::<_, Module>("SELECT * FROM modules")
            .fetch_all(pool)
            .await
    }
    /// Edit a specific module by its ID.
    ///
    /// The `updated_at` field is automatically set to the current timestamp during the update.
    ///
    /// # Arguments
    ///
    /// * `pool` - Optional reference to an SQLite connection pool. If `None`, the default pool is used.
    /// * `id` - The ID of the module to be edited.
    /// * `code` - The new code of the module (e.g., "CS101").
    /// * `year` - The academic year associated with the module.
    /// * `description` - The new description of the module.
    /// * `credits` - The number of credits the module is worth.
    ///
    /// # Returns
    ///
    /// Returns a `Result` containing the updated `Module` if successful, or a `sqlx::Error` if the operation fails.

    pub async fn edit(
        pool: Option<&SqlitePool>,
        id: i64,
        code: &str,
        year: i32,
        description: &str,
        credits: i32,
    ) -> sqlx::Result<Self> {
        let pool = pool.unwrap_or_else(|| crate::pool::get());

        let record = sqlx::query_as::<_, Module>(
            "
        UPDATE modules
        SET code = ?, year = ?, description = ?, credits = ?, updated_at = CURRENT_TIMESTAMP
        WHERE id = ?
        RETURNING *",
        )
        .bind(code)
        .bind(year)
        .bind(description)
        .bind(credits)
        .bind(id)
        .fetch_one(pool)
        .await?;

        Ok(record)
    }

    /// Filters and paginates modules based on optional search criteria.
    ///
    /// This function retrieves a list of modules from the database that match the specified filter criteria.
    ///
    /// # Arguments
    ///
    /// * `pool` - Optional reference to an SQLite connection pool. If `None`, the default pool is used.
    /// * `page` - The page number used for pagination.
    /// * `length` - The number of results per page.
    /// * `query` - Optional search query that matches against `code` or `description` fields (case-insensitive).
    /// * `code` - Optional module code filter (partial match, case-insensitive).
    /// * `year` - Optional academic year filter.
    /// * `sort` - Optional comma-separated list of fields to sort by. Prefix with `-` for descending order.
    ///
    /// # Returns
    ///
    /// Returns a `Result` containing a `Vec<Module>` that matches the filter criteria, or a `sqlx::Error` if the query fails.
    pub async fn filter(
        pool: Option<&SqlitePool>,
        page: i32,
        length: i32,
        query: Option<String>,
        code: Option<String>,
        year: Option<i32>,
        sort: Option<String>,
    ) -> sqlx::Result<Vec<Self>> {
        let pool = pool.unwrap_or_else(|| crate::pool::get());

        let offset = (page - 1) * length;

        let mut sql = String::from("SELECT * FROM modules WHERE 1=1");
        let mut args = sqlx::sqlite::SqliteArguments::default();

        if let Some(ref q) = query {
            sql.push_str(" AND (LOWER(code) LIKE ? OR LOWER(description) LIKE ?)");
            let q_like = format!("%{}%", q.to_lowercase());
            args.add(q_like.clone());
            args.add(q_like);
        }

        if let Some(ref c) = code {
            sql.push_str(" AND LOWER(code) LIKE ?");
            args.add(format!("%{}%", c.to_lowercase()));
        }

        if let Some(y) = year {
            sql.push_str(" AND year = ?");
            args.add(y);
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
    use crate::models::module::Module;
    use crate::{create_test_db, delete_database};

    #[tokio::test]
    async fn test_module_update() {
        let pool = create_test_db(Some("test_module_update.db")).await;

        let code = "COS117";
        let year = 2025;
        let description = Some("Intro to CS");
        let credits = 3;

        let created = Module::create(Some(&pool), code, year, description, credits)
            .await
            .unwrap();

        assert_eq!(created.code, code);
        assert_eq!(created.year, year);
        assert_eq!(created.description.as_deref(), description);
        assert_eq!(created.credits, credits);

        let updated_code = "COS112";
        let updated_year = 2026;
        let updated_description = "Opyy";
        let updated_credits = 4;
        println!("Updating module with ID: {}", created.id);

        let updated_module = Module::edit(
            Some(&pool),
            created.id,
            updated_code,
            updated_year,
            updated_description,
            updated_credits,
        )
        .await
        .unwrap();

        assert_eq!(updated_module.code, updated_code);
        assert_eq!(updated_module.year, updated_year);
        assert_eq!(
            updated_module.description.as_deref(),
            Some(updated_description)
        );
        assert_eq!(updated_module.credits, updated_credits);

        pool.close().await;
        delete_database("test_module_update.db");
    }

    #[tokio::test]
    async fn test_module_create_and_find() {
        let pool = create_test_db(Some("test_module_create_and_find.db")).await;

        let code = "COS301";
        let year = 2025;
        let description = Some("Software Engineering");

        let created = Module::create(Some(&pool), code, year, description, 4)
            .await
            .unwrap();

        assert_eq!(created.code, code);
        assert_eq!(created.year, year);
        assert_eq!(created.description.as_deref(), description);

        let found = Module::get_by_id(Some(&pool), created.id).await.unwrap();
        assert!(found.is_some());
        let found = found.unwrap();
        assert_eq!(found.code, code);
        assert_eq!(found.year, year);

        pool.close().await;
        delete_database("test_module_create_and_find.db");
    }

    #[tokio::test]
    async fn test_module_deletion() {
        let pool = create_test_db(Some("test_module_deletion.db")).await;

        let module = Module::create(Some(&pool), "COS110", 2025, Some("Intro to CS"), 3)
            .await
            .unwrap();
        let id = module.id;

        let found = Module::get_by_id(Some(&pool), id).await.unwrap();
        assert!(found.is_some());

        Module::delete_by_id(Some(&pool), id).await.unwrap();

        let after_delete = Module::get_by_id(Some(&pool), id).await.unwrap();
        assert!(after_delete.is_none());

        pool.close().await;
        delete_database("test_module_deletion.db");
    }

    #[tokio::test]
    async fn test_module_get_all() {
        let pool = create_test_db(Some("test_module_get_all.db")).await;

        let m1 = Module::create(Some(&pool), "COS132", 2024, Some("Programming"), 4)
            .await
            .unwrap();
        let m2 = Module::create(Some(&pool), "COS212", 2024, None, 3)
            .await
            .unwrap();

        let all = Module::get_all(Some(&pool)).await.unwrap();
        let codes: Vec<String> = all.iter().map(|m| m.code.clone()).collect();

        assert!(codes.contains(&m1.code));
        assert!(codes.contains(&m2.code));
        assert!(all.len() >= 2);

        pool.close().await;
        delete_database("test_module_get_all.db");
    }
}
