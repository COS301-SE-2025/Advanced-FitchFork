use std::io::{Cursor, Read};
use zip::ZipArchive;
use tar::Archive;
use flate2::read::GzDecoder;

use crate::execution_config::ExecutionConfig;

#[derive(Debug, PartialEq)]
enum ArchiveFormat {
    Zip,
    Tar,
    TarGz,
    Gz,
}

fn detect_archive_format(bytes: &[u8]) -> Result<ArchiveFormat, String> {
    if bytes.len() < 4 {
        return Err("File too small to determine format".to_string());
    }

    if bytes[0] == 0x50 && bytes[1] == 0x4B {
        return Ok(ArchiveFormat::Zip);
    }

    if bytes.len() > 262 {
        if &bytes[257..262] == b"ustar" {
            return Ok(ArchiveFormat::Tar);
        }
    }

    if bytes[0] == 0x1F && bytes[1] == 0x8B {
        let cursor = Cursor::new(bytes);
        let mut decoder = GzDecoder::new(cursor);
        let mut decompressed = Vec::new();
        
        if decoder.read_to_end(&mut decompressed).is_ok() && decompressed.len() > 262 {
            if &decompressed[257..262] == b"ustar" {
                return Ok(ArchiveFormat::TarGz);
            }
        }
        
        return Ok(ArchiveFormat::Gz);
    }

    Err("Unsupported archive format".to_string())
}

fn scan_zip_archive(bytes: &[u8], config: &ExecutionConfig) -> Result<bool, String> {
    let cursor = Cursor::new(bytes);
    let mut archive = ZipArchive::new(cursor)
        .map_err(|e| format!("Failed to read zip archive: {}", e))?;

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

fn scan_tar_archive(bytes: &[u8], config: &ExecutionConfig) -> Result<bool, String> {
    let cursor = Cursor::new(bytes);
    let mut archive = Archive::new(cursor);

    for entry in archive.entries().map_err(|e| format!("Failed to read tar entries: {}", e))? {
        let mut entry = entry.map_err(|e| format!("Failed to read tar entry: {}", e))?;
        
        if entry.header().entry_type().is_dir() {
            continue;
        }

        let mut contents = String::new();
        entry.read_to_string(&mut contents)
            .map_err(|e| format!("Failed to read entry contents: {e}"))?;

        for dis in &config.marking.dissalowed_code {
            if contents.contains(dis) {
                return Ok(true);
            }
        }
    }

    Ok(false)
}

fn scan_tar_gz_archive(bytes: &[u8], config: &ExecutionConfig) -> Result<bool, String> {
    let cursor = Cursor::new(bytes);
    let decoder = GzDecoder::new(cursor);
    let mut archive = Archive::new(decoder);

    for entry in archive.entries().map_err(|e| format!("Failed to read tar.gz entries: {}", e))? {
        let mut entry = entry.map_err(|e| format!("Failed to read tar.gz entry: {}", e))?;
        
        if entry.header().entry_type().is_dir() {
            continue;
        }

        let mut contents = String::new();
        entry.read_to_string(&mut contents)
            .map_err(|e| format!("Failed to read entry contents: {e}"))?;

        for dis in &config.marking.dissalowed_code {
            if contents.contains(dis) {
                return Ok(true);
            }
        }
    }

    Ok(false)
}

fn scan_gz_file(bytes: &[u8], config: &ExecutionConfig) -> Result<bool, String> {
    let cursor = Cursor::new(bytes);
    let mut decoder = GzDecoder::new(cursor);
    let mut contents = String::new();
    
    decoder.read_to_string(&mut contents)
        .map_err(|e| format!("Failed to decompress gz file: {e}"))?;

    for dis in &config.marking.dissalowed_code {
        if contents.contains(dis) {
            return Ok(true);
        }
    }

    Ok(false)
}

/// Scans an archive (ZIP, TAR, TGZ, or GZ) for any disallowed code patterns.
///
/// # Arguments
///
/// * `archive_bytes` - The archive file data as a byte slice.
/// * `config` - The [`ExecutionConfig`] containing the `dissalowed_code` list to check against.
///
/// # Returns
///
/// * `Ok(true)` if any file in the archive contains one of the `dissalowed_code` strings.
/// * `Ok(false)` if none of the files contain disallowed code.
/// * `Err(String)` if the archive data could not be read or parsed.
///
/// # Supported Formats
///
/// - ZIP archives (.zip)
/// - TAR archives (.tar)
/// - Compressed TAR archives (.tar.gz, .tgz)
/// - GZIP compressed files (.gz)
///
/// # Behavior
///
/// - Automatically detects archive format using magic bytes
/// - Iterates over all entries in the archive
/// - Skips directories, only inspects files
/// - Reads file contents as UTF-8 text
/// - Stops scanning and returns `true` immediately on the first match
///
pub fn contains_dissalowed_code(
    archive_bytes: &[u8],
    config: &ExecutionConfig,
) -> Result<bool, String> {
    let format = detect_archive_format(archive_bytes)?;
    
    match format {
        ArchiveFormat::Zip => scan_zip_archive(archive_bytes, config),
        ArchiveFormat::Tar => scan_tar_archive(archive_bytes, config),
        ArchiveFormat::TarGz => scan_tar_gz_archive(archive_bytes, config),
        ArchiveFormat::Gz => scan_gz_file(archive_bytes, config),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::Write;
    use tempfile::tempdir;
    use zip::write::FileOptions;
    use flate2::write::GzEncoder;
    use flate2::Compression;

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

    fn create_test_tar(files: Vec<(&str, &str)>, tar_path: &std::path::Path) {
        let file = File::create(tar_path).unwrap();
        let mut ar = tar::Builder::new(file);

        for (name, content) in files {
            let mut header = tar::Header::new_gnu();
            header.set_path(name).unwrap();
            header.set_size(content.len() as u64);
            header.set_cksum();
            ar.append(&header, content.as_bytes()).unwrap();
        }

        ar.finish().unwrap();
    }

    fn create_test_tar_gz(files: Vec<(&str, &str)>, tar_gz_path: &std::path::Path) {
        let file = File::create(tar_gz_path).unwrap();
        let encoder = GzEncoder::new(file, Compression::default());
        let mut ar = tar::Builder::new(encoder);

        for (name, content) in files {
            let mut header = tar::Header::new_gnu();
            header.set_path(name).unwrap();
            header.set_size(content.len() as u64);
            header.set_cksum();
            ar.append(&header, content.as_bytes()).unwrap();
        }

        ar.finish().unwrap();
    }

    fn create_test_gz(content: &str, gz_path: &std::path::Path) {
        let file = File::create(gz_path).unwrap();
        let mut encoder = GzEncoder::new(file, Compression::default());
        encoder.write_all(content.as_bytes()).unwrap();
        encoder.finish().unwrap();
    }

    #[test]
    fn test_contains_disallowed_code_zip_found() {
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

        let zip_bytes = std::fs::read(&zip_path).unwrap();
        let result = contains_dissalowed_code(&zip_bytes, &config).unwrap();
        assert!(result, "Should detect dissalowed code in the zip");
    }

    #[test]
    fn test_contains_disallowed_code_zip_not_found() {
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

        let zip_bytes = std::fs::read(&zip_path).unwrap();
        let result = contains_dissalowed_code(&zip_bytes, &config).unwrap();
        assert!(!result, "Should not detect dissalowed code in the zip");
    }

    #[test]
    fn test_contains_disallowed_code_tar_found() {
        let dir = tempdir().unwrap();
        let tar_path = dir.path().join("test.tar");

        create_test_tar(
            vec![
                ("file1.rs", "fn main() { println!(\"Hello\"); }"),
                ("file2.rs", "import forbidden_code;"),
            ],
            &tar_path,
        );

        let mut config = ExecutionConfig::default_config();
        config.marking.dissalowed_code = vec!["forbidden_code".to_string()];

        let tar_bytes = std::fs::read(&tar_path).unwrap();
        let result = contains_dissalowed_code(&tar_bytes, &config).unwrap();
        assert!(result, "Should detect dissalowed code in the tar");
    }

    #[test]
    fn test_contains_disallowed_code_tar_gz_found() {
        let dir = tempdir().unwrap();
        let tar_gz_path = dir.path().join("test.tar.gz");

        create_test_tar_gz(
            vec![
                ("file1.rs", "fn main() { println!(\"Hello\"); }"),
                ("file2.rs", "import forbidden_code;"),
            ],
            &tar_gz_path,
        );

        let mut config = ExecutionConfig::default_config();
        config.marking.dissalowed_code = vec!["forbidden_code".to_string()];

        let tar_gz_bytes = std::fs::read(&tar_gz_path).unwrap();
        let result = contains_dissalowed_code(&tar_gz_bytes, &config).unwrap();
        assert!(result, "Should detect dissalowed code in the tar.gz");
    }

    #[test]
    fn test_contains_disallowed_code_gz_found() {
        let dir = tempdir().unwrap();
        let gz_path = dir.path().join("test.gz");

        create_test_gz("import forbidden_code; fn main() {}", &gz_path);

        let mut config = ExecutionConfig::default_config();
        config.marking.dissalowed_code = vec!["forbidden_code".to_string()];

        let gz_bytes = std::fs::read(&gz_path).unwrap();
        let result = contains_dissalowed_code(&gz_bytes, &config).unwrap();
        assert!(result, "Should detect dissalowed code in the gz file");
    }

    #[test]
    fn test_contains_disallowed_code_gz_not_found() {
        let dir = tempdir().unwrap();
        let gz_path = dir.path().join("test2.gz");

        create_test_gz("fn main() { println!(\"Hello\"); }", &gz_path);

        let mut config = ExecutionConfig::default_config();
        config.marking.dissalowed_code = vec!["forbidden_code".to_string()];

        let gz_bytes = std::fs::read(&gz_path).unwrap();
        let result = contains_dissalowed_code(&gz_bytes, &config).unwrap();
        assert!(!result, "Should not detect dissalowed code in the gz file");
    }

    #[test]
    fn test_contains_disallowed_code_empty_list() {
        let dir = tempdir().unwrap();
        let zip_path = dir.path().join("test3.zip");

        create_test_zip(vec![("file1.rs", "import forbidden_code;")], &zip_path);

        let config = ExecutionConfig::default_config(); // dissalowed_code is empty

        let zip_bytes = std::fs::read(&zip_path).unwrap();
        let result = contains_dissalowed_code(&zip_bytes, &config).unwrap();
        assert!(
            !result,
            "Should not detect anything when dissalowed_code is empty"
        );
    }

    #[test]
    fn test_detect_archive_format() {
        // Test ZIP format detection
        let zip_bytes = [0x50, 0x4B, 0x03, 0x04]; // ZIP magic bytes
        assert_eq!(detect_archive_format(&zip_bytes).unwrap(), ArchiveFormat::Zip);

        // Test plain GZIP format detection (not a TAR archive)
        let gz_bytes = [0x1F, 0x8B, 0x08, 0x00]; // GZIP magic bytes with no TAR content
        assert_eq!(detect_archive_format(&gz_bytes).unwrap(), ArchiveFormat::Gz);

        // Test unsupported format
        let unknown_bytes = [0xFF, 0xFF, 0xFF, 0xFF];
        assert!(detect_archive_format(&unknown_bytes).is_err());

        // Test file too small
        let tiny_bytes = [0x50, 0x4B];
        assert!(detect_archive_format(&tiny_bytes).is_err());
    }
}
