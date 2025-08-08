//utils/compression.rs
use std::fs::{self, File};
use std::io::{Cursor, Read};
use std::path::Path;

use flate2::read::GzDecoder;
use tar::Archive;
use zip::read::ZipArchive;

/// Extracts supported archives (`.zip`, `.tar`, `.gz`, `.tgz`) into the destination directory.
pub fn extract_archive_contents(
    file_path: &Path,
    archive_bytes: &[u8],
    max_uncompressed_size: u64,
    destination_dir: &Path,
) -> Result<(), Box<dyn std::error::Error + Send + Sync + 'static>> {
    match file_path.extension().and_then(|e| e.to_str()) {
        Some("zip") => extract_zip(archive_bytes, max_uncompressed_size, destination_dir),
        Some("tar") => extract_tar(archive_bytes, max_uncompressed_size, destination_dir),
        Some("gz") => {
            // Check if it's a .tar.gz or plain .gz
            if let Some(file_name) = file_path.file_stem().and_then(|f| f.to_str()) {
                if file_name.ends_with(".tar") {
                    extract_tgz(archive_bytes, max_uncompressed_size, destination_dir)
                } else {
                    extract_gz(archive_bytes, max_uncompressed_size, destination_dir)
                }
            } else {
                extract_gz(archive_bytes, max_uncompressed_size, destination_dir)
            }
        }
        Some("tgz") => extract_tgz(archive_bytes, max_uncompressed_size, destination_dir),
        Some(ext) => Err(format!("Unsupported archive type: .{}", ext).into()),
        None => Err("File has no extension".into()),
    }
}

/// Checks if the file has a supported archive extension
pub fn is_supported_archive(path: &Path) -> bool {
    match path.extension().and_then(|ext| ext.to_str()) {
        Some("zip") | Some("tar") | Some("gz") | Some("tgz") => true,
        _ => false,
    }
}

/// ZIP extraction
fn extract_zip(
    archive_bytes: &[u8],
    max_uncompressed_size: u64,
    destination_dir: &Path,
) -> Result<(), Box<dyn std::error::Error + Send + Sync + 'static>> {
    let reader = Cursor::new(archive_bytes);
    let mut archive = ZipArchive::new(reader)?;
    let mut total_uncompressed_size = 0;

    for i in 0..archive.len() {
        let mut file = archive.by_index(i)?;
        let outpath = destination_dir.join(file.name());

        if !outpath.starts_with(destination_dir) {
            return Err("Zip archive contains invalid path (zip slip attack?)".into());
        }

        total_uncompressed_size += file.size();
        if total_uncompressed_size > max_uncompressed_size {
            return Err("Uncompressed zip size exceeds allowed maximum".into());
        }

        if file.name().ends_with('/') {
            fs::create_dir_all(&outpath)?;
        } else {
            if let Some(p) = outpath.parent() {
                fs::create_dir_all(p)?;
            }
            let mut outfile = File::create(&outpath)?;
            std::io::copy(&mut file, &mut outfile)?;
        }
    }

    Ok(())
}

/// TAR extraction
fn extract_tar(
    archive_bytes: &[u8],
    max_uncompressed_size: u64,
    destination_dir: &Path,
) -> Result<(), Box<dyn std::error::Error + Send + Sync + 'static>> {
    let cursor = Cursor::new(archive_bytes);
    let mut archive = Archive::new(cursor);
    validate_tar_size(&mut archive, max_uncompressed_size)?;
    archive.unpack(destination_dir)?;
    Ok(())
}

/// .tar.gz or .tgz extraction
fn extract_tgz(
    archive_bytes: &[u8],
    max_uncompressed_size: u64,
    destination_dir: &Path,
) -> Result<(), Box<dyn std::error::Error + Send + Sync + 'static>> {
    // Validate size (first pass)
    let decoder_for_validation = GzDecoder::new(Cursor::new(archive_bytes));
    let mut archive_for_validation = Archive::new(decoder_for_validation);
    validate_tar_size(&mut archive_for_validation, max_uncompressed_size)?;

    // Extraction (second pass)
    let decoder = GzDecoder::new(Cursor::new(archive_bytes));
    let mut archive = Archive::new(decoder);
    archive.unpack(destination_dir)?;
    Ok(())
}

/// Single GZ file extraction
fn extract_gz(
    archive_bytes: &[u8],
    max_uncompressed_size: u64,
    destination_dir: &Path,
) -> Result<(), Box<dyn std::error::Error + Send + Sync + 'static>> {
    let mut decoder = GzDecoder::new(Cursor::new(archive_bytes));
    let outpath = destination_dir.join("extracted_file");
    let mut outfile = File::create(&outpath)?;

    let uncompressed_size = std::io::copy(&mut decoder, &mut outfile)?;
    if uncompressed_size > max_uncompressed_size {
        return Err("Uncompressed gz size exceeds allowed maximum".into());
    }

    Ok(())
}

fn validate_tar_size<R: Read>(
    archive: &mut Archive<R>,
    max_uncompressed_size: u64,
) -> Result<(), Box<dyn std::error::Error + Send + Sync + 'static>> {
    let mut total_size = 0;

    for entry in archive.entries()? {
        let entry = entry?;
        total_size += entry.header().size()? as u64;
        if total_size > max_uncompressed_size {
            return Err("Uncompressed tar size exceeds allowed maximum".into());
        }
    }

    Ok(())
}
