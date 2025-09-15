use std::env;
use tempfile::TempDir;

/// Creates a unique temporary directory and sets `STORAGE_ROOT`
/// to its absolute path for the duration of the test. The directory is
/// automatically cleaned up when the returned `TempDir` is dropped.
///
/// Keep the returned `TempDir` in scope for as long as you need the files.
pub fn setup_test_storage_root() -> TempDir {
    let tmp = TempDir::new().expect("failed to create tempdir");
    let abs = tmp
        .path()
        .canonicalize()
        .unwrap_or_else(|_| tmp.path().to_path_buf());
    unsafe {
        env::set_var("STORAGE_ROOT", &abs);
    }
    tmp
}
