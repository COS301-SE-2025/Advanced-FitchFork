use serde::{Deserialize, Serialize};
use sqlx::{FromRow, SqlitePool};
use std::fs;
use std::io::Write;
use std::path::Path;

/// Represents a stored `.zip` student file for an assignment.
#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct AssignmentStudentFiles {
    pub id: i64,
    pub assignment_id: i64,
    pub filename: String,
    pub path: String, // Will be constant across the board.
    pub created_at: String,
    pub updated_at: String,
}

impl AssignmentStudentFiles {
    /// Directory where assignment student files are stored.
    ///
    /// Defaults to `"../data/assignment_student_files"`, relative to the executable.
    /// This can be changed if necessary.
    ///
    /// TODO - Move to .env
    pub const STORAGE_DIR_STUDENT: &'static str = "../data/assignment_student_files";

    /// Create and store an assignment student file.
    ///
    /// # Arguments
    ///
    /// * `pool` - Optional reference to a SQLite pool.
    /// * `assignment_id` - The ID of the assignment this student belongs to.
    /// * `filename` - The name to save the file as.
    /// * `file_bytes` - The raw `.zip` file contents.
    ///
    /// # Returns
    ///
    /// Returns the saved `AssignmentStudentFiles` record.
    pub async fn create_and_store_file(
        pool: Option<&SqlitePool>,
        assignment_id: i64,
        filename: &str,
        file_bytes: &[u8],
    ) -> sqlx::Result<Self> {
        let full_dir = Path::new(Self::STORAGE_DIR_STUDENT);
        fs::create_dir_all(&full_dir).expect("Failed to create storage directory");

        let full_path = full_dir.join(filename);

        let mut file = fs::File::create(&full_path)
            .unwrap_or_else(|_| panic!("Failed to create file: {}", full_path.display()));
        file.write_all(file_bytes)
            .expect("Failed to write zip data");

        let pool = pool.unwrap_or_else(|| crate::pool::get());
        let record = sqlx::query_as::<_, AssignmentStudentFiles>(
            r#"
            INSERT INTO assignment_student_files (
                assignment_id, filename, path
            )
            VALUES (?, ?, ?)
            RETURNING id, assignment_id, filename, path, created_at, updated_at
            "#,
        )
        .bind(assignment_id)
        .bind(filename)
        .bind(Self::STORAGE_DIR_STUDENT)
        .fetch_one(pool)
        .await?;

        Ok(record)
    }

    /// Delete a student file and its database record by ID.
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

        if let Some(record) = AssignmentStudentFiles::get_by_id(pool.into(), id).await? {
            let full_path = Path::new(&record.path).join(&record.filename);
            let _ = fs::remove_file(full_path);
        }

        sqlx::query("DELETE FROM assignment_student_files WHERE id = ?")
            .bind(id)
            .execute(pool)
            .await?;

        Ok(())
    }

    /// Get the raw bytes of a stored student file by ID.
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
        let record = AssignmentStudentFiles::get_by_id(pool, id).await?;

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

    /// Get all student file records from the database.
    ///
    /// # Arguments
    ///
    /// * `pool` - Optional reference to a SQLite pool.
    ///
    /// # Returns
    ///
    /// Returns a vector of all `AssignmentStudentFiles` records.
    pub async fn get_all(pool: Option<&SqlitePool>) -> sqlx::Result<Vec<Self>> {
        let pool = pool.unwrap_or_else(|| crate::pool::get());
        sqlx::query_as::<_, AssignmentStudentFiles>("SELECT * FROM assignment_student_files")
            .fetch_all(pool)
            .await
    }

    /// Get a student file record by its ID.
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
        sqlx::query_as::<_, AssignmentStudentFiles>(
            "SELECT * FROM assignment_student_files WHERE id = ?",
        )
        .bind(id)
        .fetch_optional(pool)
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
        vec![0x50, 0x4B, 0x03, 0x04, 0x14, 0x00]
    }

    #[tokio::test]
    async fn test_create_and_store_file() {
        let db_name = "test_assignment_student_create.db";
        let pool = create_test_db(Some(db_name)).await;

        // Required: Create fake module
        let module = crate::models::module::Module::create(
            Some(&pool),
            "COS333",
            2025,
            Some("student Test"),
            16,
        )
        .await
        .unwrap();

        // Required: Create fake assignment
        let assignment = crate::models::assignment::Assignment::create(
            Some(&pool),
            module.id,
            "student Assignment",
            Some("student Description"),
            crate::models::assignment::AssignmentType::Assignment,
            "2025-02-01T00:00:00Z",
            "2025-02-28T23:59:59Z",
        )
        .await
        .unwrap();

        let filename = "student_test.zip";
        let zip_data = create_fake_zip_bytes();

        let record = AssignmentStudentFiles::create_and_store_file(
            Some(&pool),
            assignment.id,
            filename,
            &zip_data,
        )
        .await
        .unwrap();

        assert_eq!(record.filename, filename);
        assert_eq!(record.assignment_id, assignment.id);

        let file_path = Path::new(AssignmentStudentFiles::STORAGE_DIR_STUDENT).join(filename);
        assert!(file_path.exists());

        pool.close().await;
        let _ = fs::remove_file(&file_path);
        delete_database(db_name);
    }

    #[tokio::test]
    async fn test_get_file_bytes_and_delete() {
        let db_name = "test_assignment_student_read_delete.db";
        let pool = create_test_db(Some(db_name)).await;

        // Required: Create fake module
        let module = crate::models::module::Module::create(
            Some(&pool),
            "COS334",
            2025,
            Some("student Test 2"),
            16,
        )
        .await
        .unwrap();

        // Required: Create fake assignment
        let assignment = crate::models::assignment::Assignment::create(
            Some(&pool),
            module.id,
            "student Assignment 2",
            Some("Another Description"),
            crate::models::assignment::AssignmentType::Practical,
            "2025-03-01T00:00:00Z",
            "2025-03-31T23:59:59Z",
        )
        .await
        .unwrap();

        let filename = "student_delete_test.zip";
        let zip_data = create_fake_zip_bytes();

        let record = AssignmentStudentFiles::create_and_store_file(
            Some(&pool),
            assignment.id,
            filename,
            &zip_data,
        )
        .await
        .unwrap();

        // Get bytes
        let retrieved = AssignmentStudentFiles::get_file_bytes(Some(&pool), record.id)
            .await
            .unwrap();
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap(), zip_data);

        // Delete
        AssignmentStudentFiles::delete_by_id(Some(&pool), record.id)
            .await
            .unwrap();

        // File should be gone
        let file_path = Path::new(AssignmentStudentFiles::STORAGE_DIR_STUDENT).join(filename);
        assert!(!file_path.exists());

        // DB record should be gone
        let gone = AssignmentStudentFiles::get_by_id(Some(&pool), record.id)
            .await
            .unwrap();
        assert!(gone.is_none());

        pool.close().await;
        delete_database(db_name);
    }
}
