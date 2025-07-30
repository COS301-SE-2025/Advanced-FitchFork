use db::models::assignment_memo_output::Model as MemoOutputModel;
use db::models::assignment_submission_output::Model as SubmissionOutputModel;

use std::fs;
use std::io::{self, ErrorKind};
use std::path::PathBuf;

pub struct Output;

impl Output {
    /// Get all memo output files for the given module and assignment,
    /// returning Vec<(filename, file_contents_as_string)>
    pub fn get_memo_output(
        module_id: i64,
        assignment_id: i64,
    ) -> io::Result<Vec<(String, String)>> {
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

        let mut results = Vec::new();

        for entry in fs::read_dir(dir_path)? {
            let entry = entry?;
            if entry.file_type()?.is_file() {
                let filename = entry.file_name().to_string_lossy().to_string();
                let path = entry.path();

                let content = fs::read_to_string(&path)?;
                results.push((filename, content));
            }
        }

        Ok(results)
    }

    /// Get all submission output files for the given parameters,
    /// returning Vec<(filename, file_contents_as_string)>
    pub fn get_submission_output(
        module_id: i64,
        assignment_id: i64,
        task_number: i64,
        user_id: i64,
        attempt_number: i64,
    ) -> io::Result<Vec<(String, String)>> {
        let dir_path = SubmissionOutputModel::storage_root()
            .join(format!("module_{module_id}"))
            .join(format!("assignment_{assignment_id}"))
            .join("assignment_submissions")
            .join(format!("user_{user_id}"))
            .join(format!("attempt_{attempt_number}"))
            .join("submission_output")
            .join(format!("task_{task_number}"));

        if !dir_path.exists() {
            return Err(io::Error::new(
                ErrorKind::NotFound,
                format!("Submission output directory {:?} does not exist", dir_path),
            ));
        }

        let mut results = Vec::new();

        for entry in fs::read_dir(dir_path)? {
            let entry = entry?;
            if entry.file_type()?.is_file() {
                let filename = entry.file_name().to_string_lossy().to_string();
                let path = entry.path();

                let content = fs::read_to_string(&path)?;
                results.push((filename, content));
            }
        }

        Ok(results)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_print_memo_output() {
        let module_id = 9;
        let assignment_id = 18;

        match Output::get_memo_output(module_id, assignment_id) {
            Ok(files) => {
                println!("Memo output files:");
                for (filename, contents) in files {
                    println!("File: {}\nContents:\n{}\n---", filename, contents);
                }
            }
            Err(e) => println!("Error reading memo output: {}", e),
        }
    }

    #[test]
    fn test_print_submission_output() {
        let module_id = 1;
        let assignment_id = 1;
        let task_number = 1;
        let user_id = 1;
        let attempt_number = 1;

        match Output::get_submission_output(
            module_id,
            assignment_id,
            task_number,
            user_id,
            attempt_number,
        ) {
            Ok(files) => {
                println!("Submission output files:");
                for (filename, contents) in files {
                    println!("File: {}\nContents:\n{}\n---", filename, contents);
                }
            }
            Err(e) => println!("Error reading submission output: {}", e),
        }
    }
}
