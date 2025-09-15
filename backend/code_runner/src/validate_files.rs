use std::fs;
use util::execution_config::ExecutionConfig;
use util::paths::{attempt_dir, main_dir, makefile_dir, memo_dir};

/// Validate that an assignment has a readable execution config and that each of
/// the required directories (`memo`, `makefile`, `main`) contains at least one
/// `.zip` file. Paths are resolved via `util::paths::*`
///
/// # Errors
/// Returns an error if:
/// - the execution config cannot be loaded/parsed, or
/// - any required directory is missing or lacks a `.zip` file.
pub fn validate_memo_files(module_id: i64, assignment_id: i64) -> Result<(), String> {
    // Validate config first
    ExecutionConfig::get_execution_config(module_id, assignment_id)
        .map_err(|e| format!("Config validation failed: {}", e))?;

    // Each of these dirs must contain at least one .zip
    for dir in [
        memo_dir(module_id, assignment_id),
        makefile_dir(module_id, assignment_id),
        main_dir(module_id, assignment_id),
    ] {
        let entries = fs::read_dir(&dir)
            .map_err(|_| format!("Failed to read directory {:?}", dir))?;

        let has_zip = entries
            .filter_map(Result::ok)
            .any(|e| e.path().extension().and_then(|s| s.to_str()) == Some("zip"));

        if !has_zip {
            // Figure out which name to show (memo/makefile/main)
            let name = dir.file_name().and_then(|s| s.to_str()).unwrap_or("<unknown>");
            return Err(format!(
                "Required directory '{}' does not contain any .zip file at {:?}",
                name, dir
            ));
        }
    }

    Ok(())
}

/// Validate that a specific student's submission attempt is present and the
/// assignment has the required `.zip` inputs in `makefile` and `main`.
/// Paths are resolved via `util::paths::*`
///
/// # Errors
/// Returns an error if:
/// - the execution config cannot be loaded/parsed, or
/// - `makefile`/`main` lack a `.zip`, or
/// - the submission attempt directory has no files.
pub fn validate_submission_files(
    module_id: i64,
    assignment_id: i64,
    user_id: i64,
    attempt_number: i64,
) -> Result<(), String> {
    // Validate config
    ExecutionConfig::get_execution_config(module_id, assignment_id)
        .map_err(|e| format!("Config validation failed: {}", e))?;

    // makefile & main must each have a .zip
    for dir in [
        makefile_dir(module_id, assignment_id),
        main_dir(module_id, assignment_id),
    ] {
        let entries = fs::read_dir(&dir)
            .map_err(|_| format!("Failed to read directory {:?}", dir))?;

        let has_zip = entries
            .filter_map(Result::ok)
            .any(|e| e.path().extension().and_then(|s| s.to_str()) == Some("zip"));

        if !has_zip {
            let name = dir.file_name().and_then(|s| s.to_str()).unwrap_or("<unknown>");
            return Err(format!(
                "Required directory '{}' does not contain any .zip file at {:?}",
                name, dir
            ));
        }
    }

    // submission directory must contain at least one file
    let submission_dir = attempt_dir(module_id, assignment_id, user_id, attempt_number);
    let entries = fs::read_dir(&submission_dir)
        .map_err(|_| format!("Failed to read submission directory {:?}", submission_dir))?;

    let has_any_file = entries.filter_map(Result::ok).any(|e| e.path().is_file());
    if !has_any_file {
        return Err(format!("No submission file found at {:?}", submission_dir));
    }

    Ok(())
}

/// Write a default `config.json` for a module/assignment pair at the canonical
/// location resolved by `util::paths::config_dir`. This uses
/// `ExecutionConfig::default_config()` and `ExecutionConfig::save(...)`.
///
/// Use this in tests or initialization; adjust fields before `save()` if a
/// specific test needs non-default config.
pub fn write_config_json(module_id: i64, assignment_id: i64) {
    let cfg = ExecutionConfig::default_config();
    cfg.save(module_id, assignment_id)
        .expect("Failed to save config.json");
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use util::{paths::{attempt_dir, config_dir, main_dir, makefile_dir, memo_dir}, test_helpers::setup_test_storage_root};

    fn write_required_file_zip(module_id: i64, assignment_id: i64, which: &str, filename: &str) {
        let dir = match which {
            "memo" => memo_dir(module_id, assignment_id),
            "makefile" => makefile_dir(module_id, assignment_id),
            "main" => main_dir(module_id, assignment_id),
            _ => panic!("unknown dir"),
        };
        fs::create_dir_all(&dir).unwrap();
        fs::write(dir.join(filename), b"dummy").unwrap();
    }

    #[test]
    fn test_validate_all_files_exist() {
        let _tmp = setup_test_storage_root();

        let module_id = 5;
        let assignment_id = 99;

        write_config_json(module_id, assignment_id);

        write_required_file_zip(module_id, assignment_id, "memo", "memo.zip");
        write_required_file_zip(module_id, assignment_id, "makefile", "makefile.zip");
        write_required_file_zip(module_id, assignment_id, "main", "main.zip");

        let result = validate_memo_files(module_id, assignment_id);
        assert!(result.is_ok());
    }

    #[test]
    fn test_missing_file_causes_error() {
        let _tmp = setup_test_storage_root();

        let module_id = 1;
        let assignment_id = 101;

        write_config_json(module_id, assignment_id);

        write_required_file_zip(module_id, assignment_id, "memo", "memo.zip");
        write_required_file_zip(module_id, assignment_id, "main", "main.zip");

        // Ensure makefile dir exists but contains no zip to trigger the error
        fs::create_dir_all(makefile_dir(module_id, assignment_id)).unwrap();

        let result = validate_memo_files(module_id, assignment_id);
        println!("{:?}", result);

        assert!(result.is_err());
        assert!(result.unwrap_err().contains("makefile"));
    }

    #[test]
    fn test_invalid_config_causes_error() {
        let _tmp = setup_test_storage_root();

        let module_id = 2;
        let assignment_id = 123;

        // Write invalid config JSON directly
        let cfg_dir = config_dir(module_id, assignment_id);
        fs::create_dir_all(&cfg_dir).unwrap();
        fs::write(cfg_dir.join("config.json"), b"{ invalid json }").unwrap();

        write_required_file_zip(module_id, assignment_id, "memo", "memo.zip");
        write_required_file_zip(module_id, assignment_id, "makefile", "makefile.zip");
        write_required_file_zip(module_id, assignment_id, "main", "main.zip");

        let result = validate_memo_files(module_id, assignment_id);
        println!("{:?}", result);

        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Config validation failed"));
    }

    #[test]
    fn test_validate_submission_success() {
        let _tmp = setup_test_storage_root();

        let module_id = 7;
        let assignment_id = 888;
        let user_id = 42;
        let attempt_number = 2;

        write_config_json(module_id, assignment_id);

        write_required_file_zip(module_id, assignment_id, "makefile", "mf.zip");
        write_required_file_zip(module_id, assignment_id, "main", "main.zip");

        let submission_dir = attempt_dir(module_id, assignment_id, user_id, attempt_number);
        fs::create_dir_all(&submission_dir).unwrap();
        fs::write(submission_dir.join("482.txt"), b"submission").unwrap();

        let result =
            validate_submission_files(module_id, assignment_id, user_id, attempt_number);

        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_submission_missing_file() {
        let _tmp = setup_test_storage_root();

        let module_id = 7;
        let assignment_id = 999;
        let user_id = 1;
        let attempt_number = 1;

        write_config_json(module_id, assignment_id);

        write_required_file_zip(module_id, assignment_id, "makefile", "mf.zip");
        write_required_file_zip(module_id, assignment_id, "main", "main.zip");

        // Missing submission dir (or empty) → should error
        let result =
            validate_submission_files(module_id, assignment_id, user_id, attempt_number);

        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .contains("Failed to read submission directory"));
    }
}
