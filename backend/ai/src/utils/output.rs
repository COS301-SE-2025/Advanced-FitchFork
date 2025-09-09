use db::models::assignment_memo_output::Model as MemoOutputModel;
use db::models::assignment_submission_output::Entity as SubmissionOutputEntity;
use db::models::assignment_submission_output::Model as SubmissionOutputModel;
use db::models::assignment_task::Entity as AssignmentTaskEntity;

use sea_orm::EntityTrait;
use std::fs;
use std::io::{self, ErrorKind};

#[allow(dead_code)]
pub struct Output;

impl Output {
    /// Get all memo output files for the given module and assignment,
    /// returning Vec<(task_number, file_contents_as_string)>
    #[allow(dead_code)]
    pub fn get_memo_output(module_id: i64, assignment_id: i64) -> io::Result<Vec<(i64, String)>> {
        let dir_path = MemoOutputModel::storage_root()
            .join(format!("module_{module_id}"))
            .join(format!("assignment_{assignment_id}"))
            .join("memo_output");

        if !dir_path.exists() {
            return Err(io::Error::new(
                ErrorKind::NotFound,
                format!("Memo output directory {:?} does not exist", dir_path),
            ));
        }

        let mut entries: Vec<_> = fs::read_dir(dir_path)?
            .filter_map(Result::ok)
            .filter(|e| e.file_type().map(|ft| ft.is_file()).unwrap_or(false))
            .collect();

        // Sort files alphabetically for deterministic order
        entries.sort_by_key(|e| e.file_name());

        let mut results = Vec::new();
        for (i, entry) in entries.into_iter().enumerate() {
            let path = entry.path();
            let content = fs::read_to_string(&path)?;
            let task_number = (i + 1) as i64;
            results.push((task_number, content));
        }

        Ok(results)
    }

    /// Get all submission output files for the given parameters,
    /// returning Vec<(task_number, file_contents_as_string)>
    #[allow(dead_code)]
    pub async fn get_submission_output_no_coverage(
        db: &sea_orm::DatabaseConnection,
        module_id: i64,
        assignment_id: i64,
        user_id: i64,
        attempt_number: i64,
    ) -> io::Result<Vec<(i64, String)>> {
        Self::get_submission_output_filtered(
            db,
            module_id,
            assignment_id,
            user_id,
            attempt_number,
            false,
        )
        .await
    }

    #[allow(dead_code)]
    pub async fn get_submission_output_code_coverage(
        db: &sea_orm::DatabaseConnection,
        module_id: i64,
        assignment_id: i64,
        user_id: i64,
        attempt_number: i64,
    ) -> io::Result<Vec<(i64, String)>> {
        Self::get_submission_output_filtered(
            db,
            module_id,
            assignment_id,
            user_id,
            attempt_number,
            true,
        )
        .await
    }

    async fn get_submission_output_filtered(
        db: &sea_orm::DatabaseConnection,
        module_id: i64,
        assignment_id: i64,
        user_id: i64,
        attempt_number: i64,
        code_coverage: bool,
    ) -> io::Result<Vec<(i64, String)>> {
        let dir_path = SubmissionOutputModel::storage_root()
            .join(format!("module_{module_id}"))
            .join(format!("assignment_{assignment_id}"))
            .join("assignment_submissions")
            .join(format!("user_{user_id}"))
            .join(format!("attempt_{attempt_number}"))
            .join("submission_output");

        if !dir_path.exists() {
            return Err(io::Error::new(
                ErrorKind::NotFound,
                format!("Submission output directory {:?} does not exist", dir_path),
            ));
        }

        let mut entries: Vec<_> = fs::read_dir(&dir_path)?
            .filter_map(Result::ok)
            .filter(|e| e.file_type().map(|ft| ft.is_file()).unwrap_or(false))
            .collect();

        entries.sort_by_key(|e| e.file_name());

        let mut results = Vec::new();
        for entry in entries {
            let path = entry.path();

            // Extract submission_output id from file name (assumes file stem is the id)
            if let Some(file_stem) = path.file_stem().and_then(|s| s.to_str()) {
                if let Ok(output_id) = file_stem.parse::<i64>() {
                    // Look up the output in the DB
                    if let Ok(Some(output)) =
                        SubmissionOutputEntity::find_by_id(output_id).one(db).await
                    {
                        // Look up the task to check code_coverage
                        if let Ok(Some(task)) = AssignmentTaskEntity::find_by_id(output.task_id)
                            .one(db)
                            .await
                        {
                            if task.code_coverage == code_coverage {
                                let content = fs::read_to_string(&path)?;
                                results.push((output.task_id, content));
                            }
                        }
                    }
                }
            }
        }

        Ok(results)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[ignore]
    #[test]
    fn test_print_memo_output() {
        let module_id = 9;
        let assignment_id = 18;

        match Output::get_memo_output(module_id, assignment_id) {
            Ok(files) => {
                println!("Memo output files:");
                for (task_number, contents) in files {
                    println!("Task {}:\n{}\n---", task_number, contents);
                }
            }
            Err(e) => println!("Error reading memo output: {}", e),
        }
    }
}
