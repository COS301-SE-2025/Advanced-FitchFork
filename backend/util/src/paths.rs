use crate::config;
use std::{fs, io, path::{Path, PathBuf}};

/// Create a directory (and all parents) if it doesn't exist, and return the path.
pub fn ensure_dir<P: AsRef<Path>>(path: P) -> io::Result<PathBuf> {
    let p = path.as_ref();
    fs::create_dir_all(p)?;
    Ok(p.to_path_buf())
}

/// Ensure the parent directory of a *file path* exists (no-op if none).
pub fn ensure_parent_dir<P: AsRef<Path>>(file_path: P) -> io::Result<()> {
    if let Some(parent) = file_path.as_ref().parent() {
        fs::create_dir_all(parent)?;
    }
    Ok(())
}

/// Global storage root (absolute), from `config::storage_root()`.
/// If relative in env, resolve against current_dir().
pub fn storage_root() -> PathBuf {
    let root = config::storage_root();
    let p = PathBuf::from(root);
    if p.is_absolute() {
        p
    } else {
        std::env::current_dir()
            .unwrap_or_else(|_| PathBuf::from("."))
            .join(p)
    }
}

/// A single module folder: {STORAGE_ROOT}/module_{module_id}
pub fn module_dir(module_id: i64) -> PathBuf {
    storage_root().join(format!("module_{module_id}"))
}

/// Path to a user's folder:  {STORAGE_ROOT}/users/user_{user_id}
pub fn user_dir(user_id: i64) -> PathBuf {
    storage_root().join("users").join(format!("user_{user_id}"))
}

/// A "profile" subfolder for the user:  .../users/user_{id}/profile
pub fn user_profile_dir(user_id: i64) -> PathBuf {
    user_dir(user_id).join("profile")
}

/// Build a path under the user's profile directory (does not create).
/// Example: user_profile_path(42, "avatar.jpg") → .../users/user_42/profile/avatar.jpg
pub fn user_profile_path(user_id: i64, filename: &str) -> PathBuf {
    user_profile_dir(user_id).join(filename)
}

// ─── Directory helpers for assignments ──────────────────────────────

// Top-level:  {STORAGE_ROOT}/module_{module_id}/assignment_{assignment_id}
pub fn assignment_dir(module_id: i64, assignment_id: i64) -> PathBuf {
    module_dir(module_id).join(format!("assignment_{assignment_id}"))
}

// Config directory + files
pub fn config_dir(module_id: i64, assignment_id: i64) -> PathBuf {
    assignment_dir(module_id, assignment_id).join("config")
}
pub fn config_path(module_id: i64, assignment_id: i64, file_id: i64) -> PathBuf {
    config_dir(module_id, assignment_id).join(format!("{file_id}.json"))
}

// Spec
pub fn spec_dir(module_id: i64, assignment_id: i64) -> PathBuf {
    assignment_dir(module_id, assignment_id).join("spec")
}
pub fn spec_path(module_id: i64, assignment_id: i64, file_id: i64) -> PathBuf {
    spec_dir(module_id, assignment_id).join(format!("{file_id}.zip"))
}

// Memo
pub fn memo_dir(module_id: i64, assignment_id: i64) -> PathBuf {
    assignment_dir(module_id, assignment_id).join("memo")
}
pub fn memo_path(module_id: i64, assignment_id: i64, file_id: i64) -> PathBuf {
    memo_dir(module_id, assignment_id).join(format!("{file_id}.zip"))
}

// Main
pub fn main_dir(module_id: i64, assignment_id: i64) -> PathBuf {
    assignment_dir(module_id, assignment_id).join("main")
}
pub fn main_path(module_id: i64, assignment_id: i64, file_id: i64) -> PathBuf {
    main_dir(module_id, assignment_id).join(format!("{file_id}.zip"))
}

// Makefile
pub fn makefile_dir(module_id: i64, assignment_id: i64) -> PathBuf {
    assignment_dir(module_id, assignment_id).join("makefile")
}
pub fn makefile_path(module_id: i64, assignment_id: i64, file_id: i64) -> PathBuf {
    makefile_dir(module_id, assignment_id).join(format!("{file_id}.zip"))
}

// Mark allocator
pub fn mark_allocator_dir(module_id: i64, assignment_id: i64) -> PathBuf {
    assignment_dir(module_id, assignment_id).join("mark_allocator")
}
pub fn mark_allocator_path(module_id: i64, assignment_id: i64) -> PathBuf {
    mark_allocator_dir(module_id, assignment_id).join("allocator.json")
}

// Memo output
pub fn memo_output_dir(module_id: i64, assignment_id: i64) -> PathBuf {
    assignment_dir(module_id, assignment_id).join("memo_output")
}
pub fn memo_output_path(module_id: i64, assignment_id: i64, file_id: i64) -> PathBuf {
    memo_output_dir(module_id, assignment_id).join(format!("{file_id}.txt"))
}

// Interpreter
pub fn interpreter_dir(module_id: i64, assignment_id: i64) -> PathBuf {
    assignment_dir(module_id, assignment_id).join("interpreter")
}
pub fn interpreter_path(module_id: i64, assignment_id: i64, file_id: i64) -> PathBuf {
    interpreter_dir(module_id, assignment_id).join(format!("{file_id}.zip"))
}

// MOSS archives (versioned)
// {STORAGE_ROOT}/module_{module_id}/assignment_{assignment_id}/moss_archives
pub fn moss_archives_dir(module_id: i64, assignment_id: i64) -> PathBuf {
    assignment_dir(module_id, assignment_id).join("moss_archives")
}

// One specific archive folder: .../moss_archives/{archive_id}
pub fn moss_archive_dir(module_id: i64, assignment_id: i64, archive_id: &str) -> PathBuf {
    moss_archives_dir(module_id, assignment_id).join(archive_id)
}

// Zip path for a specific archive: .../moss_archives/{archive_id}/archive.zip
pub fn moss_archive_zip_path(module_id: i64, assignment_id: i64, archive_id: &str) -> PathBuf {
    moss_archive_dir(module_id, assignment_id, archive_id).join("archive.zip")
}

// Overwrite files
pub fn overwrite_files_dir(module_id: i64, assignment_id: i64) -> PathBuf {
    assignment_dir(module_id, assignment_id).join("overwrite_files")
}
pub fn overwrite_task_dir(module_id: i64, assignment_id: i64, task: i64) -> PathBuf {
    overwrite_files_dir(module_id, assignment_id).join(format!("task_{task}"))
}
pub fn overwrite_file_path(
    module_id: i64,
    assignment_id: i64,
    task: i64,
    filename: &str,
) -> PathBuf {
    overwrite_task_dir(module_id, assignment_id, task).join(filename)
}

// Submissions
pub fn submissions_dir(module_id: i64, assignment_id: i64) -> PathBuf {
    assignment_dir(module_id, assignment_id).join("assignment_submissions")
}
pub fn user_submission_dir(module_id: i64, assignment_id: i64, user_id: i64) -> PathBuf {
    submissions_dir(module_id, assignment_id).join(format!("user_{user_id}"))
}
pub fn attempt_dir(
    module_id: i64,
    assignment_id: i64,
    user_id: i64,
    attempt: i64,
) -> PathBuf {
    user_submission_dir(module_id, assignment_id, user_id).join(format!("attempt_{attempt}"))
}

/// Stored filename used for a submission's primary file: "{submission_id}{.ext?}"
#[inline]
pub fn submission_stored_filename(submission_id: i64, ext: Option<&str>) -> String {
    match ext {
        Some(e) if !e.is_empty() => {
            let e = e.trim_start_matches('.');
            format!("{submission_id}.{e}")
        }
        _ => submission_id.to_string(),
    }
}

/// Full path to the stored submission file (id + optional extension).
pub fn submission_file_path(
    module_id: i64,
    assignment_id: i64,
    user_id: i64,
    attempt: i64,
    submission_id: i64,
    ext: Option<&str>,
) -> PathBuf {
    attempt_dir(module_id, assignment_id, user_id, attempt)
        .join(submission_stored_filename(submission_id, ext))
}

pub fn submission_report_path(
    module_id: i64,
    assignment_id: i64,
    user_id: i64,
    attempt: i64,
) -> PathBuf {
    attempt_dir(module_id, assignment_id, user_id, attempt).join("submission_report.json")
}
pub fn submission_output_dir(
    module_id: i64,
    assignment_id: i64,
    user_id: i64,
    attempt: i64,
) -> PathBuf {
    attempt_dir(module_id, assignment_id, user_id, attempt).join("submission_output")
}
pub fn submission_output_path(
    module_id: i64,
    assignment_id: i64,
    user_id: i64,
    attempt: i64,
    filename: &str,
) -> PathBuf {
    submission_output_dir(module_id, assignment_id, user_id, attempt).join(filename)
}


#[cfg(test)]
mod tests {
    use super::*;
    use serial_test::serial;
    use tempfile::TempDir;

    // Minimal set mutated by these tests; other config fields use defaults.
    const MUT_VARS: &[&str] = &[
        "STORAGE_ROOT",
        "DATABASE_PATH",
        "JWT_SECRET",
    ];

    fn clear_mut_vars() {
        for k in MUT_VARS {
            unsafe { std::env::remove_var(k) };
        }
    }

    fn set_required_with_root(root: &str) {
        unsafe {
            std::env::set_var("STORAGE_ROOT", root);
            std::env::set_var("DATABASE_PATH", "/tmp/test.db");
            std::env::set_var("JWT_SECRET", "test");
        }
    }

    #[test]
    #[serial]
    fn root_resolves_relative_against_cwd() {
        clear_mut_vars();
        set_required_with_root("storage_rel");

        let expected = std::env::current_dir().unwrap().join("storage_rel");
        assert_eq!(storage_root(), expected);
    }

    #[test]
    #[serial]
    fn root_uses_absolute_as_is() {
        clear_mut_vars();
        let td = TempDir::new().unwrap();
        let abs = td.path().to_path_buf();

        set_required_with_root(abs.to_str().unwrap());

        assert_eq!(storage_root(), abs);
    }

    #[test]
    #[serial]
    fn helpers_construct_expected_paths() {
        clear_mut_vars();
        let td = TempDir::new().unwrap();
        let root = td.path().to_path_buf();

        set_required_with_root(root.to_str().unwrap());

        let m = 7_i64;
        let a = 42_i64;
        let f = 99_i64;
        let u = 5_i64;
        let t = 3_i64;
        let attempt = 2_i64;

        let base = root.join("module_7").join("assignment_42");

        // module root
        assert_eq!(module_dir(m), root.join("module_7"));

        // Base
        assert_eq!(assignment_dir(m, a), base);

        // Config
        assert_eq!(config_dir(m, a), base.join("config"));
        assert_eq!(config_path(m, a, f), base.join("config").join("99.json"));

        // Spec
        assert_eq!(spec_dir(m, a), base.join("spec"));
        assert_eq!(spec_path(m, a, f), base.join("spec").join("99.zip"));

        // Memo
        assert_eq!(memo_dir(m, a), base.join("memo"));
        assert_eq!(memo_path(m, a, f), base.join("memo").join("99.zip"));

        // Main
        assert_eq!(main_dir(m, a), base.join("main"));
        assert_eq!(main_path(m, a, f), base.join("main").join("99.zip"));

        // Makefile
        assert_eq!(makefile_dir(m, a), base.join("makefile"));
        assert_eq!(makefile_path(m, a, f), base.join("makefile").join("99.zip"));

        // Mark allocator
        assert_eq!(mark_allocator_dir(m, a), base.join("mark_allocator"));
        assert_eq!(mark_allocator_path(m, a), base.join("mark_allocator").join("allocator.json"));

        // Memo output
        assert_eq!(memo_output_dir(m, a), base.join("memo_output"));
        assert_eq!(memo_output_path(m, a, f), base.join("memo_output").join("99.txt"));

        // Interpreter
        assert_eq!(interpreter_dir(m, a), base.join("interpreter"));
        assert_eq!(interpreter_path(m, a, f), base.join("interpreter").join("99.zip"));

        // Overwrite files
        assert_eq!(overwrite_files_dir(m, a), base.join("overwrite_files"));
        assert_eq!(overwrite_task_dir(m, a, t), base.join("overwrite_files").join("task_3"));
        assert_eq!(
            overwrite_file_path(m, a, t, "foo.c"),
            base.join("overwrite_files").join("task_3").join("foo.c")
        );

                // Submissions tree
        assert_eq!(submissions_dir(m, a), base.join("assignment_submissions"));
        assert_eq!(user_submission_dir(m, a, u), base.join("assignment_submissions").join("user_5"));
        assert_eq!(
            attempt_dir(m, a, u, attempt),
            base.join("assignment_submissions").join("user_5").join("attempt_2")
        );

        // Use a fake submission id to validate filename construction
        let s = 123_i64;
        assert_eq!(
            submission_file_path(m, a, u, attempt, s, Some("zip")),
            base.join("assignment_submissions").join("user_5").join("attempt_2").join("123.zip")
        );
        assert_eq!(
            submission_file_path(m, a, u, attempt, s, None),
            base.join("assignment_submissions").join("user_5").join("attempt_2").join("123")
        );

        assert_eq!(
            submission_report_path(m, a, u, attempt),
            base.join("assignment_submissions").join("user_5").join("attempt_2").join("submission_report.json")
        );
        assert_eq!(
            submission_output_dir(m, a, u, attempt),
            base.join("assignment_submissions").join("user_5").join("attempt_2").join("submission_output")
        );
        assert_eq!(
            submission_output_path(m, a, u, attempt, "stdout.txt"),
            base.join("assignment_submissions").join("user_5").join("attempt_2").join("submission_output").join("stdout.txt")
        );

    }
}
