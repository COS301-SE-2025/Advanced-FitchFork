use std::fs;
use std::path::{Path, PathBuf};
use serde_json::Value;
use crate::error::MarkerError;

#[derive(Debug)]
pub struct LoadedFiles {
    pub submission_id: String,
    pub memo_paths: Vec<PathBuf>,
    pub student_paths: Vec<PathBuf>,
    pub allocator_raw: Value,
    pub coverage_raw: Value,
    pub complexity_raw: Value,
}

const MAX_JSON_SIZE: u64 = 2 * 1024 * 1024; // 2MB

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
    
    Ok(LoadedFiles {
        submission_id: submission_id.to_string(),
        memo_paths,
        student_paths,
        allocator_raw,
        coverage_raw,
        complexity_raw,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

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
        assert_eq!(loaded.memo_paths, memo_paths);
        assert_eq!(loaded.student_paths, student_paths);
        assert!(loaded.allocator_raw.is_object(), "allocator_raw should be a JSON object");
        assert!(loaded.coverage_raw.is_object(), "coverage_raw should be a JSON object");
        assert!(loaded.complexity_raw.is_object(), "complexity_raw should be a JSON object");
    }

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