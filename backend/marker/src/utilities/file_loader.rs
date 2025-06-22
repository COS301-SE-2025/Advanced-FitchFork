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
//! - Loads and validates the allocator, coverage, and complexity JSON files, enforcing a maximum size.
//! - Returns a [`LoadedFiles`] struct containing all loaded data and paths.
//!
//! # Error Handling
//!
//! Returns [`MarkerError`] variants for missing files, size violations, invalid JSON, or mismatched input lengths.
//!
//! # Tests
//!
//! This module includes comprehensive tests for valid and invalid file sets, covering edge cases and error reporting.

use std::fs;
use std::path::{Path, PathBuf};
use serde_json::Value;
use crate::error::MarkerError;

/// Represents all files loaded for a submission, including their paths and parsed JSON content.
#[derive(Debug)]
pub struct LoadedFiles {
    /// Submission identifier.
    pub submission_id: String,
    /// Raw content of memo files.
    pub memo_contents: Vec<String>,
    /// Raw content of student files.
    pub student_contents: Vec<String>,
    /// Raw JSON value for the allocator report.
    pub allocator_raw: Value,
    /// Raw JSON value for the coverage report.
    pub coverage_raw: Value,
    /// Raw JSON value for the complexity report.
    pub complexity_raw: Value,
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
        return Err(MarkerError::IoError(format!("File not found: {}", path.display())));
    }

    if !path.is_file() {
        return Err(MarkerError::IoError(format!("Not a file: {}", path.display())));
    }

    let metadata = fs::metadata(path).map_err(|_| MarkerError::IoError(format!("File unreadable: {}", path.display())))?;
    if let Some(max) = max_size {
        if metadata.len() > max {
            return Err(MarkerError::IoError(format!("File too large: {} ({} bytes)", path.display(), metadata.len())));
        }
    }

    Ok(())
}

/// Loads and validates all files required for a submission.
///
/// - Checks that all memo and student files exist.
/// - Ensures the number of memo and student files match.
/// - Loads and validates the allocator, coverage, and complexity JSON files, enforcing a maximum size.
///
/// # Errors
///
/// Returns [`MarkerError`] for missing files, size violations, invalid JSON, or mismatched input lengths.
///
/// # Returns
///
/// A [`LoadedFiles`] struct containing all loaded data and paths.
pub fn load_files(
    submission_id: &str,
    memo_paths: Vec<PathBuf>,
    student_paths: Vec<PathBuf>,
    allocator_path: PathBuf,
    coverage_path: PathBuf,
    complexity_path: PathBuf
) -> Result<LoadedFiles, MarkerError> {
    for p in &memo_paths {
        check_file(p, None)?;
    }

    for p in &student_paths {
        check_file(p, None)?;
    }

    if memo_paths.len() != student_paths.len() {
        return Err(MarkerError::InputMismatch(format!("memo_paths.len() != student_paths.len(): {} != {}", memo_paths.len(), student_paths.len())));
    }

    for p in [&allocator_path, &coverage_path, &complexity_path] {
        check_file(p, Some(MAX_JSON_SIZE))?;
    }

    let allocator_bytes = fs::read(&allocator_path).map_err(|e| MarkerError::IoError(format!("{}: {}", allocator_path.display(), e)))?;
    let coverage_bytes  = fs::read(&coverage_path ).map_err(|e| MarkerError::IoError(format!("{}: {}", coverage_path.display(), e)))?;
    let complexity_bytes= fs::read(&complexity_path).map_err(|e| MarkerError::IoError(format!("{}: {}", complexity_path.display(), e)))?;
    let allocator_raw = serde_json::from_slice(&allocator_bytes).map_err(|_| MarkerError::InvalidJson(allocator_path.display().to_string()))?;
    let coverage_raw  = serde_json::from_slice(&coverage_bytes ).map_err(|_| MarkerError::InvalidJson(coverage_path.display().to_string()))?;
    let complexity_raw= serde_json::from_slice(&complexity_bytes).map_err(|_| MarkerError::InvalidJson(complexity_path.display().to_string()))?;
    
    // Read memo and student file contents
    let mut memo_contents = Vec::new();
    for path in &memo_paths {
        let content = fs::read_to_string(path)
            .map_err(|e| MarkerError::IoError(format!("Failed to read memo file {}: {}", path.display(), e)))?;
        memo_contents.push(content);
    }

    let mut student_contents = Vec::new();
    for path in &student_paths {
        let content = fs::read_to_string(path)
            .map_err(|e| MarkerError::IoError(format!("Failed to read student file {}: {}", path.display(), e)))?;
        student_contents.push(content);
    }
    
    Ok(LoadedFiles {
        submission_id: submission_id.to_string(),
        memo_contents,
        student_contents,
        allocator_raw,
        coverage_raw,
        complexity_raw,
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
        let complexity_path = PathBuf::from(format!("{}/complexity.json", dir));
        let result = load_files(
            "sub1",
            memo_paths.clone(),
            student_paths.clone(),
            allocator_path.clone(),
            coverage_path.clone(),
            complexity_path.clone(),
        );
        assert!(result.is_ok(), "Expected Ok for happy path, got: {:?}", result);
        let loaded = result.unwrap();
        assert_eq!(loaded.submission_id, "sub1");
        assert_eq!(loaded.memo_contents.len(), memo_paths.len());
        assert_eq!(loaded.student_contents.len(), student_paths.len());
        assert!(loaded.allocator_raw.is_object(), "allocator_raw should be a JSON object");
        assert!(loaded.coverage_raw.is_object(), "coverage_raw should be a JSON object");
        assert!(loaded.complexity_raw.is_object(), "complexity_raw should be a JSON object");
    }

    /// Test error handling for a missing memo file.
    #[test]
    fn test_missing_memo_file_case2() {
        let dir = "src/test_files/file_loader/case2";
        let memo_paths = vec![PathBuf::from(format!("{}/memo1.txt", dir))]; // file does not exist
        let student_paths = vec![PathBuf::from(format!("{}/student1.txt", dir))];
        let allocator_path = PathBuf::from(format!("{}/allocator.json", dir));
        let coverage_path = PathBuf::from(format!("{}/coverage.json", dir));
        let complexity_path = PathBuf::from(format!("{}/complexity.json", dir));
        let result = load_files(
            "sub2",
            memo_paths,
            student_paths,
            allocator_path,
            coverage_path,
            complexity_path,
        );
        match result {
            Err(MarkerError::IoError(msg)) => {
                assert!(msg.contains("File not found"), "Error message should mention file not found, got: {}", msg);
            },
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
        let complexity_path = PathBuf::from(format!("{}/complexity.json", dir));
        let result = load_files(
            "sub3",
            memo_paths,
            student_paths,
            allocator_path,
            coverage_path,
            complexity_path,
        );
        match result {
            Err(MarkerError::InputMismatch(msg)) => {
                assert!(msg.contains("memo_paths.len() != student_paths.len()"), "Error message should mention length mismatch, got: {}", msg);
            },
            other => panic!("Expected InputMismatch for length mismatch, got: {:?}", other),
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
        let complexity_path = PathBuf::from(format!("{}/complexity.json", dir));
        let result = load_files(
            "sub4",
            memo_paths,
            student_paths,
            allocator_path,
            coverage_path,
            complexity_path,
        );
        match result {
            Err(MarkerError::IoError(msg)) => {
                assert!(msg.contains("File too large"), "Error message should mention file too large, got: {}", msg);
            },
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
        let complexity_path = PathBuf::from(format!("{}/complexity.json", dir));
        let result = load_files(
            "sub5",
            memo_paths,
            student_paths,
            allocator_path,
            coverage_path,
            complexity_path,
        );
        match result {
            Err(MarkerError::InvalidJson(path)) => {
                assert!(path.contains("allocator.json"), "Error path should mention allocator.json, got: {}", path);
            },
            other => panic!("Expected InvalidJson for invalid json content, got: {:?}", other),
        }
    }
}