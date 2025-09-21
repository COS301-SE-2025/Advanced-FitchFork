//!
//! File Loader Utility
//!
//! This module provides utilities for loading and validating the set of files required for a submission evaluation.
//! It ensures that all referenced files exist, are of appropriate size, and that the JSON files are valid and not too large.
//!
//! # Functionality
//!
//! - Checks the existence and type of memo and student files.
//! - Ensures the number of memo and student files match.
//! - Loads and validates the allocator and coverage JSON files, enforcing a maximum size.
//! - Returns a [`LoadedFiles`] struct containing all loaded data and paths.
//!
//! # Error Handling
//!
//! Returns [`MarkerError`] variants for missing files, size violations, invalid JSON, or mismatched input lengths.
//!
//! # Tests
//!
//! This module includes comprehensive tests for valid and invalid file sets, covering edge cases and error reporting.

use crate::error::MarkerError;
use serde_json::Value;
use std::fs;
use std::path::{Path, PathBuf};
use tracing::error;

/// Represents all files loaded for a submission, including their paths and parsed JSON content.
#[derive(Debug)]
pub struct LoadedFiles {
    /// Raw content of memo files.
    pub memo_contents: Vec<String>,
    /// Raw content of student files.
    pub student_contents: Vec<String>,
    /// Raw JSON value for the allocator report.
    pub allocator_raw: Value,
    /// Raw JSON value for the coverage report.
    pub coverage_raw: Option<Value>,
}

/// Maximum allowed size for JSON files.
const MAX_JSON_SIZE: u64 = 2 * 1024 * 1024; // 2MB

/// Checks that a file exists, is a file, and (optionally) does not exceed a maximum size.
///
/// # Errors
///
/// Returns [`MarkerError::IoError`] if the file is missing, not a file, unreadable, or too large.
fn check_file(path: &Path, max_size: Option<u64>) -> Result<(), MarkerError> {
    if !path.exists() {
        let specific_error = format!("File not found: {}", path.display());
        error!("{}", specific_error);
        return Err(MarkerError::IoError("File not found".to_string()));
    }

    if !path.is_file() {
        let specific_error = format!("Not a file: {}", path.display());
        error!("{}", specific_error);
        return Err(MarkerError::IoError("Invalid file type".to_string()));
    }

    let metadata = fs::metadata(path).map_err(|e| {
        let specific_error = format!("File unreadable: {} - {}", path.display(), e);
        error!("{}", specific_error);
        MarkerError::IoError("File unreadable".to_string())
    })?;

    if let Some(max) = max_size {
        if metadata.len() > max {
            let specific_error = format!(
                "File too large: {} ({} bytes, max {} bytes)",
                path.display(),
                metadata.len(),
                max
            );
            error!("{}", specific_error);
            return Err(MarkerError::IoError("File too large".to_string()));
        }
    }

    Ok(())
}

/// Loads and validates all files required for a submission.
///
/// - Checks that all memo and student files exist.
/// - Ensures the number of memo and student files match.
/// - Loads and validates the allocator and coverage JSON files, enforcing a maximum size.
///
/// # Errors
///
/// Returns [`MarkerError`] for missing files, size violations, invalid JSON, or mismatched input lengths.
///
/// # Returns
///
/// A [`LoadedFiles`] struct containing all loaded data and paths.
pub fn load_files(
    memo_paths: Vec<PathBuf>,
    student_paths: Vec<PathBuf>,
    allocator_path: PathBuf,
    coverage_path: Option<PathBuf>,
) -> Result<LoadedFiles, MarkerError> {
    for p in &memo_paths {
        check_file(p, None)?;
    }
    for p in &student_paths {
        check_file(p, None)?;
    }
    if memo_paths.len() != student_paths.len() {
        return Err(MarkerError::InputMismatch(format!(
            "memo_paths.len() != student_paths.len(): {} != {}",
            memo_paths.len(),
            student_paths.len()
        )));
    }
    check_file(&allocator_path, Some(MAX_JSON_SIZE))?;

    let allocator_bytes = fs::read(&allocator_path).map_err(|e| {
        let specific_error = format!(
            "Failed to read allocator file {}: {}",
            allocator_path.display(),
            e
        );
        error!("{}", specific_error);
        MarkerError::IoError("Failed to load mark allocator".to_string())
    })?;

    let allocator_raw = serde_json::from_slice(&allocator_bytes).map_err(|e| {
        let specific_error = format!(
            "Invalid JSON in allocator file {}: {}",
            allocator_path.display(),
            e
        );
        error!("{}", specific_error);
        MarkerError::InvalidJson("Failed to parse mark allocator".to_string())
    })?;

    let coverage_raw = if let Some(path) = coverage_path {
        check_file(&path, Some(MAX_JSON_SIZE))?;
        let bytes = fs::read(&path).map_err(|e| {
            let specific_error = format!("Failed to read coverage file {}: {}", path.display(), e);
            error!("{}", specific_error);
            MarkerError::IoError("Failed to load coverage report".to_string())
        })?;
        let coverage_json = serde_json::from_slice(&bytes).map_err(|e| {
            let specific_error = format!("Invalid JSON in coverage file {}: {}", path.display(), e);
            error!("{}", specific_error);
            MarkerError::InvalidJson("Failed to parse coverage report".to_string())
        })?;
        Some(coverage_json)
    } else {
        None
    };

    let mut memo_contents = Vec::new();
    for path in &memo_paths {
        let content = fs::read_to_string(path).map_err(|e| {
            let specific_error = format!("Failed to read memo file {}: {}", path.display(), e);
            error!("{}", specific_error);
            MarkerError::IoError("Failed to read memo file".to_string())
        })?;
        memo_contents.push(content);
    }

    let mut student_contents = Vec::new();
    for path in &student_paths {
        let content = fs::read_to_string(path).map_err(|e| {
            let specific_error = format!("Failed to read student file {}: {}", path.display(), e);
            error!("{}", specific_error);
            MarkerError::IoError("Failed to read student file".to_string())
        })?;
        student_contents.push(content);
    }

    Ok(LoadedFiles {
        memo_contents,
        student_contents,
        allocator_raw,
        coverage_raw,
    })
}

#[cfg(test)]
mod tests {
    //! Unit tests for the file loader utility.
    //! These tests cover valid and invalid file sets, including edge cases and error reporting.
    use super::*;
    use std::path::PathBuf;

    /// Test loading a valid set of files (happy path).
    #[test]
    fn test_happy_path_case1() {
        let dir = "src/test_files/file_loader/case1";
        let memo_paths = vec![PathBuf::from(format!("{}/memo1.txt", dir))];
        let student_paths = vec![PathBuf::from(format!("{}/student1.txt", dir))];
        let allocator_path = PathBuf::from(format!("{}/allocator.json", dir));
        let coverage_path = PathBuf::from(format!("{}/coverage.json", dir));
        let result = load_files(
            memo_paths.clone(),
            student_paths.clone(),
            allocator_path.clone(),
            Some(coverage_path.clone()),
        );
        assert!(
            result.is_ok(),
            "Expected Ok for happy path, got: {:?}",
            result
        );
        let loaded = result.unwrap();
        assert_eq!(loaded.memo_contents.len(), memo_paths.len());
        assert_eq!(loaded.student_contents.len(), student_paths.len());
        assert!(
            loaded.allocator_raw.is_object(),
            "allocator_raw should be a JSON object"
        );
        assert!(loaded.coverage_raw.is_some(), "coverage_raw should be Some");
    }

    /// Test error handling for a missing memo file.
    #[test]
    fn test_missing_memo_file_case2() {
        let dir = "src/test_files/file_loader/case2";
        let memo_paths = vec![PathBuf::from(format!("{}/memo1.txt", dir))]; // file does not exist
        let student_paths = vec![PathBuf::from(format!("{}/student1.txt", dir))];
        let allocator_path = PathBuf::from(format!("{}/allocator.json", dir));
        let coverage_path = PathBuf::from(format!("{}/coverage.json", dir));
        let result = load_files(
            memo_paths,
            student_paths,
            allocator_path,
            Some(coverage_path),
        );
        match result {
            Err(MarkerError::IoError(msg)) => {
                assert_eq!(
                    msg, "File not found",
                    "Error message should be general, got: {}",
                    msg
                );
            }
            other => panic!("Expected IoError for missing memo file, got: {:?}", other),
        }
    }

    /// Test error handling for mismatched memo and student file counts.
    #[test]
    fn test_length_mismatch_case3() {
        let dir = "src/test_files/file_loader/case3";
        let memo_paths = vec![
            PathBuf::from(format!("{}/memo1.txt", dir)),
            PathBuf::from(format!("{}/memo2.txt", dir)),
        ];
        let student_paths = vec![PathBuf::from(format!("{}/student1.txt", dir))];
        let allocator_path = PathBuf::from(format!("{}/allocator.json", dir));
        let coverage_path = PathBuf::from(format!("{}/coverage.json", dir));
        let result = load_files(
            memo_paths,
            student_paths,
            allocator_path,
            Some(coverage_path),
        );
        match result {
            Err(MarkerError::InputMismatch(msg)) => {
                assert!(
                    msg.contains("memo_paths.len() != student_paths.len()"),
                    "Error message should mention length mismatch, got: {}",
                    msg
                );
            }
            other => panic!(
                "Expected InputMismatch for length mismatch, got: {:?}",
                other
            ),
        }
    }

    /// Test error handling for a JSON file that is too large.
    #[test]
    fn test_json_too_large_case4() {
        let dir = "src/test_files/file_loader/case4";
        let memo_paths = vec![PathBuf::from(format!("{}/memo1.txt", dir))];
        let student_paths = vec![PathBuf::from(format!("{}/student1.txt", dir))];
        let allocator_path = PathBuf::from(format!("{}/allocator.json", dir)); // >2MB
        let coverage_path = PathBuf::from(format!("{}/coverage.json", dir));
        let result = load_files(
            memo_paths,
            student_paths,
            allocator_path,
            Some(coverage_path),
        );
        match result {
            Err(MarkerError::IoError(msg)) => {
                assert_eq!(
                    msg, "File too large",
                    "Error message should be general, got: {}",
                    msg
                );
            }
            other => panic!("Expected IoError for file too large, got: {:?}", other),
        }
    }

    /// Test error handling for invalid JSON content in a file.
    #[test]
    fn test_invalid_json_content_case5() {
        let dir = "src/test_files/file_loader/case5";
        let memo_paths = vec![PathBuf::from(format!("{}/memo1.txt", dir))];
        let student_paths = vec![PathBuf::from(format!("{}/student1.txt", dir))];
        let allocator_path = PathBuf::from(format!("{}/allocator.json", dir)); // invalid json
        let coverage_path = PathBuf::from(format!("{}/coverage.json", dir));
        let result = load_files(
            memo_paths,
            student_paths,
            allocator_path,
            Some(coverage_path),
        );
        match result {
            Err(MarkerError::InvalidJson(msg)) => {
                assert_eq!(
                    msg, "Failed to parse mark allocator",
                    "Error message should be general, got: {}",
                    msg
                );
            }
            other => panic!(
                "Expected InvalidJson for invalid json content, got: {:?}",
                other
            ),
        }
    }

    /// Test loading with no optional reports.
    #[test]
    fn test_no_optional_reports() {
        let dir = "src/test_files/file_loader/case1";
        let memo_paths = vec![PathBuf::from(format!("{}/memo1.txt", dir))];
        let student_paths = vec![PathBuf::from(format!("{}/student1.txt", dir))];
        let allocator_path = PathBuf::from(format!("{}/allocator.json", dir));

        let result = load_files(memo_paths, student_paths, allocator_path, None);

        assert!(result.is_ok());
        let loaded = result.unwrap();
        assert!(loaded.coverage_raw.is_none());
    }
}
