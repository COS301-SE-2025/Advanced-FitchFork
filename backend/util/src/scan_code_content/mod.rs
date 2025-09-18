use std::fs::File;
use std::io::Read;
use std::path::Path;
use zip::ZipArchive;

use crate::execution_config::ExecutionConfig;

/// Scans a ZIP archive for any dissalowed code patterns.
///
/// # Arguments
///
/// * `zip_path` - Path to the `.zip` file containing source files.
/// * `config` - The [`ExecutionConfig`] containing the `dissalowed_code` list to check against.
///
/// # Returns
///
/// * `Ok(true)` if any file in the archive contains one of the `dissalowed_code` strings.
/// * `Ok(false)` if none of the files contain dissalowed code.
/// * `Err(String)` if the zip file could not be opened or read.
///
/// # Behavior
///
/// - Iterates over all entries in the zip archive.
/// - Skips directories, only inspects files.
/// - Reads file contents as UTF-8 text.
/// - Stops scanning and returns `true` immediately on the first match.
///
pub fn contains_dissalowed_code<P: AsRef<Path>>(
    zip_path: P,
    config: &ExecutionConfig,
) -> Result<bool, String> {
    let file = File::open(&zip_path)
        .map_err(|e| format!("Failed to open zip file {:?}: {}", zip_path.as_ref(), e))?;

    let mut archive = ZipArchive::new(file)
        .map_err(|e| format!("Failed to read zip archive {:?}: {}", zip_path.as_ref(), e))?;

    for i in 0..archive.len() {
        let mut file = archive
            .by_index(i)
            .map_err(|e| format!("Failed to read file in archive: {e}"))?;

        if file.is_dir() {
            continue;
        }

        let mut contents = String::new();
        file.read_to_string(&mut contents)
            .map_err(|e| format!("Failed to read file contents: {e}"))?;

        for dis in &config.marking.dissalowed_code {
            if contents.contains(dis) {
                return Ok(true);
            }
        }
    }

    Ok(false)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::Write;
    use tempfile::tempdir;
    use zip::write::FileOptions;

    fn create_test_zip(files: Vec<(&str, &str)>, zip_path: &std::path::Path) {
        let file = File::create(zip_path).unwrap();
        let mut zip = zip::ZipWriter::new(file);
        let options: FileOptions<'_, ()> = FileOptions::default();

        for (name, content) in files {
            zip.start_file(name, options).unwrap();
            zip.write_all(content.as_bytes()).unwrap();
        }

        zip.finish().unwrap();
    }

    #[test]
    fn test_contains_disallowed_code_found() {
        let dir = tempdir().unwrap();
        let zip_path = dir.path().join("test.zip");

        create_test_zip(
            vec![
                ("file1.rs", "fn main() { println!(\"Hello\"); }"),
                ("file2.rs", "import forbidden_code;"),
            ],
            &zip_path,
        );

        let mut config = ExecutionConfig::default_config();
        config.marking.dissalowed_code = vec!["forbidden_code".to_string()];

        let result = contains_dissalowed_code(&zip_path, &config).unwrap();
        assert!(result, "Should detect dissalowed code in the zip");
    }

    #[test]
    fn test_contains_disallowed_code_not_found() {
        let dir = tempdir().unwrap();
        let zip_path = dir.path().join("test2.zip");

        create_test_zip(
            vec![
                ("file1.rs", "fn main() { println!(\"Hello\"); }"),
                ("file2.rs", "let x = 42;"),
            ],
            &zip_path,
        );

        let mut config = ExecutionConfig::default_config();
        config.marking.dissalowed_code = vec!["forbidden_code".to_string()];

        let result = contains_dissalowed_code(&zip_path, &config).unwrap();
        assert!(!result, "Should not detect dissalowed code in the zip");
    }

    #[test]
    fn test_contains_disallowed_code_empty_list() {
        let dir = tempdir().unwrap();
        let zip_path = dir.path().join("test3.zip");

        create_test_zip(vec![("file1.rs", "import forbidden_code;")], &zip_path);

        let config = ExecutionConfig::default_config(); // dissallowed_code is empty

        let result = contains_dissalowed_code(&zip_path, &config).unwrap();
        assert!(
            !result,
            "Should not detect anything when dissalowed_code is empty"
        );
    }
}
