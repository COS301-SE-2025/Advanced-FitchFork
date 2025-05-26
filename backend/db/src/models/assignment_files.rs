use serde::{Deserialize, Serialize};
use sqlx::{FromRow, SqlitePool};
use std::fs;
use std::io::Write;
use std::path::Path;

/// Represents a stored `.zip` file for an assignment.
#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct AssignmentFiles {
    pub id: i64,
    pub assignment_id: i64,
    pub filename: String,
    pub path: String, //Will be constant across the board.
    pub created_at: String,
    pub updated_at: String,
}

impl AssignmentFiles {
    /// Directory where assignment files are stored.
    ///
    /// Defaults to `"../data/assignment_files"`, relative to the executable.
    /// This can be changed if necessary.

    //This is where I think the files should be stored -> We can change this later
    //TODO - Move to .env
    pub const STORAGE_DIR: &'static str = "../data/assignment_files";

    /// Create and store an assignment file.
    ///
    /// # Arguments
    ///
    /// * `pool` - Optional reference to a SQLite pool.
    /// * `assignment_id` - The ID of the assignment this file belongs to.
    /// * `filename` - The name to save the file as.
    /// * `file_bytes` - The raw `.zip` file contents.
    ///
    /// # Returns
    ///
    /// Returns the saved `AssignmentFiles` record.

    pub async fn create_and_store_file(
        pool: Option<&SqlitePool>,
        assignment_id: i64,
        filename: &str,
        file_bytes: &[u8], //.zip file
    ) -> sqlx::Result<Self> {
        let full_dir = Path::new(Self::STORAGE_DIR);
        fs::create_dir_all(&full_dir).expect("Failed to create storage directory");

        let full_path = full_dir.join(filename);

        let mut file = fs::File::create(&full_path)
            .unwrap_or_else(|_| panic!("Failed to create file: {}", full_path.display()));
        file.write_all(file_bytes)
            .expect("Failed to write zip data");

        //Save metadata to db
        let pool = pool.unwrap_or_else(|| crate::pool::get());
        let record = sqlx::query_as::<_, AssignmentFiles>(
            r#"
            INSERT INTO assignment_files (
                assignment_id, filename, path
            )
            VALUES (?, ?, ?)
            RETURNING id, assignment_id, filename, path, created_at, updated_at
            "#,
        )
        .bind(assignment_id)
        .bind(filename)
        .bind(Self::STORAGE_DIR)
        .fetch_one(pool)
        .await?;

        Ok(record)
    }

    /// Delete a file and its database record by ID.
    ///
    /// # Arguments
    ///
    /// * `pool` - Optional reference to a SQLite pool.
    /// * `id` - The file record ID.
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` on success.

    pub async fn delete_by_id(pool: Option<&SqlitePool>, id: i64) -> sqlx::Result<()> {
        let pool = pool.unwrap_or_else(|| crate::pool::get());

        if let Some(record) = AssignmentFiles::get_by_id(pool.into(), id).await? {
            let full_path = Path::new(&record.path).join(&record.filename);
            let _ = fs::remove_file(full_path); //ignore if file doesn't exist
        }

        sqlx::query("DELETE FROM assignment_files WHERE id = ?")
            .bind(id)
            .execute(pool)
            .await?;

        Ok(())
    }

    /// Get the raw bytes of a stored file by ID.
    ///
    /// # Arguments
    ///
    /// * `pool` - Optional reference to a SQLite pool.
    /// * `id` - The file record ID.
    ///
    /// # Returns
    ///
    /// Returns `Some(bytes)` if found, `None` if not, or an error.

    pub async fn get_file_bytes(
        pool: Option<&SqlitePool>,
        id: i64,
    ) -> sqlx::Result<Option<Vec<u8>>> {
        let record = AssignmentFiles::get_by_id(pool, id).await?;

        if let Some(file) = record {
            let full_path = Path::new(&file.path).join(&file.filename);
            match fs::read(full_path) {
                Ok(bytes) => Ok(Some(bytes)),
                Err(_) => Ok(None),
            }
        } else {
            Ok(None)
        }
    }

    /// Get all file records from the database.
    ///
    /// # Arguments
    ///
    /// * `pool` - Optional reference to a SQLite pool.
    ///
    /// # Returns
    ///
    /// Returns a vector of all `AssignmentFiles` records.

    pub async fn get_all(pool: Option<&SqlitePool>) -> sqlx::Result<Vec<Self>> {
        let pool = pool.unwrap_or_else(|| crate::pool::get());
        sqlx::query_as::<_, AssignmentFiles>("SELECT * FROM assignment_files")
            .fetch_all(pool)
            .await
    }

    /// Get a file record by its ID.
    ///
    /// # Arguments
    ///
    /// * `pool` - Optional reference to a SQLite pool.
    /// * `id` - The record ID.
    ///
    /// # Returns
    ///
    /// Returns `Some(record)` if found, `None` if not, or an error.

    pub async fn get_by_id(pool: Option<&SqlitePool>, id: i64) -> sqlx::Result<Option<Self>> {
        let pool = pool.unwrap_or_else(|| crate::pool::get());
        sqlx::query_as::<_, AssignmentFiles>("SELECT * FROM assignment_files WHERE id = ?")
            .bind(id)
            .fetch_optional(pool)
            .await
    }

    /// Get a file record by its assignment ID.
    ///
    /// # Arguments
    ///
    /// * `pool` - Optional reference to a SQLite pool.
    /// * `id` - The assignment ID.
    ///
    /// # Returns
    ///
    /// Returns a vector of records for the assignment, or an error.
    pub async fn get_by_assignment_id(
        pool: Option<&SqlitePool>,
        id: i64,
    ) -> sqlx::Result<Vec<Self>> {
        let pool = pool.unwrap_or_else(|| crate::pool::get());
        sqlx::query_as::<_, AssignmentFiles>(
            "SELECT * FROM assignment_files WHERE assignment_id = ?",
        )
        .bind(id)
        .fetch_all(pool)
        .await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{create_test_db, delete_database};
    use std::fs;
    use std::path::Path;

    fn create_fake_zip_bytes() -> Vec<u8> {
        vec![0x50, 0x4B, 0x03, 0x04, 0x14, 0x00] //this is apparently a minimal zip header. Cool
    }

    #[tokio::test]
    async fn test_create_and_store_file() {
        let db_name = "test_assignment_file_create.db";
        let pool = create_test_db(Some(db_name)).await;

        //Required: Create fake module
        let module =
            crate::models::module::Module::create(Some(&pool), "COS300", 2025, Some("SQA"), 16)
                .await
                .unwrap();

        //Required: Create fake assignment
        let assignment = crate::models::assignment::Assignment::create(
            Some(&pool),
            module.id,
            "Dummy Assignment",
            Some("desc"),
            crate::models::assignment::AssignmentType::Assignment,
            "2025-01-01T00:00:00Z",
            "2025-01-31T23:59:59Z",
        )
        .await
        .unwrap();

        let filename = "test_assignment.zip";
        let zip_data = create_fake_zip_bytes();

        let record =
            AssignmentFiles::create_and_store_file(Some(&pool), assignment.id, filename, &zip_data)
                .await
                .unwrap();

        assert_eq!(record.filename, filename);
        assert_eq!(record.assignment_id, assignment.id);

        let file_path = Path::new(AssignmentFiles::STORAGE_DIR).join(filename);
        assert!(file_path.exists());

        //clean up
        pool.close().await;
        let _ = fs::remove_file(&file_path);
        delete_database(db_name);
    }

    #[tokio::test]
    async fn test_get_file_bytes_and_delete() {
        let db_name = "test_assignment_file_read_delete.db";
        let pool = create_test_db(Some(db_name)).await;

        //Required: Create fake module
        let module =
            crate::models::module::Module::create(Some(&pool), "COS301", 2024, Some("SQA"), 16)
                .await
                .unwrap();

        //Required: Create fake assignment
        let assignment = crate::models::assignment::Assignment::create(
            Some(&pool),
            module.id,
            "Dummy Assignment",
            Some("desc"),
            crate::models::assignment::AssignmentType::Assignment,
            "2025-01-01T00:00:00Z",
            "2025-01-31T23:59:59Z",
        )
        .await
        .unwrap();

        let filename = "delete_me.zip";
        let zip_data = create_fake_zip_bytes();

        let record =
            AssignmentFiles::create_and_store_file(Some(&pool), assignment.id, filename, &zip_data)
                .await
                .unwrap();

        //get file bytes
        let retrieved = AssignmentFiles::get_file_bytes(Some(&pool), record.id)
            .await
            .unwrap();
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap(), zip_data);

        //delete file
        AssignmentFiles::delete_by_id(Some(&pool), record.id)
            .await
            .unwrap();

        //make sure file is gone
        let file_path = Path::new(AssignmentFiles::STORAGE_DIR).join(filename);
        assert!(!file_path.exists());

        //ensure db record is gone
        let record = AssignmentFiles::get_by_id(Some(&pool), record.id)
            .await
            .unwrap();
        assert!(record.is_none());

        pool.close().await;
        let _ = fs::remove_file(&file_path);
        delete_database(db_name);
    }
}
