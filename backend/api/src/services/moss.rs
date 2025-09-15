use std::path::{Path, PathBuf};
use std::io::Cursor;

use globset::{Glob, GlobSet, GlobSetBuilder};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::TcpStream;
use zip::ZipArchive;
use db::models::moss_report::FilterMode;

// keep these
const MOSS_SERVER: &str = "moss.stanford.edu";
const MOSS_PORT: u16 = 7690;


/// Options for a single MOSS run.
#[derive(Clone, Debug)]
pub struct MossRunOptions {
    pub language: String,
    pub max_matches: u32,        // -m
    pub show_limit: u32,         // -n
    pub experimental: bool,      // -x
    pub filter_mode: FilterMode,
    pub filter_patterns: Option<Vec<String>>, // glob patterns
    /// Optional spec ZIPs (skeleton code) to upload as base files (-b).
    pub spec_zips: Vec<PathBuf>,
}

impl Default for MossRunOptions {
    fn default() -> Self {
        Self {
            language: "c".to_string(),
            max_matches: 10,
            show_limit: 250,
            experimental: false,
            filter_mode: FilterMode::All,
            filter_patterns: None,
            spec_zips: vec![],
        }
    }
}

/// A service for interacting with the MOSS (Measure of Software Similarity) server.
pub struct MossService {
    user_id: String,
}

impl MossService {
    pub fn new(user_id: &str) -> Self {
        Self { user_id: user_id.to_string() }
    }

    /// Back-compat helper — behaves like the old API (no filtering, no spec bases).
    pub async fn run(
        &self,
        base_files: Vec<PathBuf>,
        submission_files: Vec<(PathBuf, Option<String>, Option<i64>)>,
        language: &str,
    ) -> Result<String, String> {
        let mut opts = MossRunOptions::default();
        opts.language = language.to_string();
        self.run_with_options(base_files, submission_files, opts).await
    }

    /// Full-featured run with filtering and spec ZIPs as base files.
    pub async fn run_with_options(
        &self,
        base_files: Vec<PathBuf>,
        submission_files: Vec<(PathBuf, Option<String>, Option<i64>)>,
        opts: MossRunOptions,
    ) -> Result<String, String> {
        if submission_files.len() < 2 {
            return Err("MOSS requires at least 2 submission files to compare".to_string());
        }

        // Validate filter semantics clearly
        let patterns_len = opts.filter_patterns.as_ref().map(|v| v.len()).unwrap_or(0);
        match opts.filter_mode {
            FilterMode::All => {
                if patterns_len > 0 {
                    return Err("filter_mode=All does not accept filter_patterns".into());
                }
            }
            FilterMode::Whitelist | FilterMode::Blacklist => {
                if patterns_len == 0 {
                    return Err("filter_patterns must be non-empty for whitelist/blacklist".into());
                }
            }
        }

        // Build globset if needed
        let globset = build_globset(opts.filter_patterns.as_ref()).map_err(|e| e.to_string())?;

        let mut stream = TcpStream::connect((MOSS_SERVER, MOSS_PORT))
            .await
            .map_err(|e| format!("Failed to connect to MOSS server: {}", e))?;

        // Header
        self.send_command(&mut stream, &format!("moss {}", self.user_id)).await?;
        self.send_command(&mut stream, "directory 0").await?;
        self.send_command(&mut stream, &format!("X {}", if opts.experimental { 1 } else { 0 })).await?;
        self.send_command(&mut stream, &format!("maxmatches {}", opts.max_matches)).await?;
        self.send_command(&mut stream, &format!("show {}", opts.show_limit)).await?;
        self.send_command(&mut stream, &format!("language {}", opts.language)).await?;

        // Language ack
        {
            let mut line = String::new();
            let mut reader = BufReader::new(&mut stream);
            reader.read_line(&mut line).await.map_err(|e| format!("Failed to read language response: {e}"))?;
            if line.trim() == "no" {
                return Err(format!("Language '{}' not supported by MOSS", opts.language));
            }
        }

        // ---- Base files ----
        // 1) Explicit base files on disk
        for p in &base_files {
            self.upload_base_from_path(&mut stream, p, &opts.language).await?;
        }
        // 2) Spec ZIPs (skeletons) — expand and upload each entry as base (id=0)
        for zip in &opts.spec_zips {
            self.upload_base_from_spec_zip(&mut stream, zip).await?;
        }

        // ---- Submissions ----
        let mut file_id = 1u32;
        for (path, username, submission_id) in &submission_files {
            if path.extension().and_then(|s| s.to_str()) == Some("zip") {
                // Upload directory with filtering
                self.send_command(&mut stream, "directory 1").await?;
                file_id = self.upload_zip_filtered(
                    &mut stream,
                    path,
                    file_id,
                    &opts.language,
                    username.as_deref(),
                    *submission_id,
                    &opts.filter_mode,
                    globset.as_ref(),
                ).await?;
                self.send_command(&mut stream, "directory 0").await?;
            } else {
                // Single file — filter by filename (best effort)
                let fname = path.file_name().and_then(|s| s.to_str()).unwrap_or_default();
                if !should_include(fname, &opts.filter_mode, globset.as_ref()) {
                    continue;
                }
                file_id = self.upload_file(&mut stream, path, file_id, &opts.language, username.as_deref(), *submission_id).await?;
            }
        }

        // Query
        self.send_command(&mut stream, "query 0 ").await?;
        let mut response = String::new();
        {
            let mut reader = BufReader::new(&mut stream);
            reader.read_line(&mut response).await.map_err(|e| format!("Failed to read query response: {e}"))?;
        }
        self.send_command(&mut stream, "end").await?;

        let report_url = response.trim().to_string();
        if !report_url.starts_with("http") {
            return Err(format!("Invalid response from MOSS server: '{report_url}'"));
        }
        Ok(report_url)
    }

    // ---------------- internal helpers ----------------

    async fn send_command(&self, stream: &mut TcpStream, command: &str) -> Result<(), String> {
        let cmd = format!("{command}\n");
        stream.write_all(cmd.as_bytes())
            .await
            .map_err(|e| format!("Failed to send '{command}': {e}"))
    }

    /// Upload a local file (submission or base depending on `file_id`).
    async fn upload_file(
        &self,
        stream: &mut TcpStream,
        path: &Path,
        file_id: u32,                   // 0 => base file, >0 => submission
        language: &str,
        username: Option<&str>,
        submission_id: Option<i64>,
    ) -> Result<u32, String> {
        if !path.exists() {
            return Err(format!("File does not exist: {}", path.display()));
        }
        let content = tokio::fs::read(path).await.map_err(|e| format!("Failed to read {}: {e}", path.display()))?;

        let original_filename = path.file_name()
            .and_then(|s| s.to_str())
            .ok_or_else(|| format!("Invalid filename: {}", path.display()))?
            .replace(' ', "_");

        let filename = match (username, submission_id) {
            (Some(u), Some(id)) => format!("{}_{}_{}", u, id, original_filename),
            (Some(u), None)     => format!("{}_{}", u, original_filename),
            (None, Some(id))    => format!("{}_{}", id, original_filename),
            (None, None)        => original_filename,
        };

        self.upload_bytes(stream, file_id, language, &filename, &content).await?;
        Ok(file_id + if file_id == 0 { 0 } else { 1 })
    }

    /// Low-level "file" packet sender for in-memory content.
    async fn upload_bytes(
        &self,
        stream: &mut TcpStream,
        file_id: u32,
        language: &str,
        display_name: &str,
        content: &[u8],
    ) -> Result<(), String> {
        let display_name = display_name.replace(' ', "_");
        let header = format!("file {} {} {} {}", file_id, language, content.len(), display_name);
        self.send_command(stream, &header).await?;
        stream.write_all(content)
            .await
            .map_err(|e| format!("Failed to upload file content: {e}"))
    }

    /// Expand a ZIP and upload each entry into the current directory-block (for submissions).
    async fn upload_zip_filtered(
        &self,
        stream: &mut TcpStream,
        zip_path: &Path,
        starting_file_id: u32,
        language: &str,
        username: Option<&str>,
        submission_id: Option<i64>,
        filter_mode: &FilterMode,    
        globset: Option<&GlobSet>,
    ) -> Result<u32, String> {
        let zip_data = tokio::fs::read(zip_path)
            .await
            .map_err(|e| format!("Failed to read ZIP {}: {e}", zip_path.display()))?;

        let cursor = Cursor::new(zip_data);
        let mut archive = ZipArchive::new(cursor)
            .map_err(|e| format!("Failed to open ZIP {}: {e}", zip_path.display()))?;

        let dir_tag = match (username, submission_id) {
            (Some(u), Some(id)) => format!("{}_{}", u, id),
            (Some(u), None)     => u.to_string(),
            (None, Some(id))    => id.to_string(),
            (None, None)        => "submission".to_string(),
        };
        let dir_tag = sanitize(&dir_tag);

        let mut next_id = starting_file_id;
        let mut uploaded = 0;

        for i in 0..archive.len() {
            let mut file = archive.by_index(i).map_err(|e| format!("Failed to read ZIP entry {i}: {e}"))?;
            if file.is_dir() { continue; }

            let mut contents = Vec::new();
            std::io::copy(&mut file, &mut contents).map_err(|e| format!("Failed to read entry bytes: {e}"))?;

            let internal = normalize_path(file.name());
            if internal.is_empty() { continue; }

            // Filtering by internal path
            if !should_include(&internal, filter_mode, globset) {
                continue;
            }

            let display_name = format!("{}/{}", dir_tag, internal);
            self.upload_bytes(stream, next_id, language, &display_name, &contents).await?;
            next_id += 1;
            uploaded += 1;
        }

        if uploaded == 0 {
            return Err(format!("ZIP contained no files after filtering: {}", zip_path.display()));
        }

        Ok(next_id)
    }

    /// Upload a single local file as a base (-b) file (id=0).
    async fn upload_base_from_path(
        &self,
        stream: &mut TcpStream,
        path: &Path,
        language: &str,
    ) -> Result<(), String> {
        // For base, the "filename" sent to MOSS doesn't have to be on-disk; we just label it.
        // We'll reuse the local name.
        self.upload_file(stream, path, 0, language, None, None).await.map(|_| ())
    }

    /// Expand a spec ZIP and upload every entry as a **base** file (id=0).
    async fn upload_base_from_spec_zip(
        &self,
        stream: &mut TcpStream,
        zip_path: &Path,
    ) -> Result<(), String> {
        let zip_data = tokio::fs::read(zip_path)
            .await
            .map_err(|e| format!("Failed to read spec ZIP {}: {e}", zip_path.display()))?;

        let cursor = Cursor::new(zip_data);
        let mut archive = ZipArchive::new(cursor)
            .map_err(|e| format!("Failed to open spec ZIP {}: {e}", zip_path.display()))?;

        // Base files do not depend on directory blocks; just send id=0 entries.
        for i in 0..archive.len() {
            let mut file = archive.by_index(i).map_err(|e| format!("Failed to read spec entry {i}: {e}"))?;
            if file.is_dir() { continue; }

            let mut contents = Vec::new();
            std::io::copy(&mut file, &mut contents).map_err(|e| format!("Failed to read spec entry bytes: {e}"))?;

            let internal = normalize_path(file.name());
            if internal.is_empty() { continue; }

            // For base, language is ignored by MOSS matching; any supported language string works.
            // We'll use "ascii" which is always allowed.
            self.upload_bytes(stream, 0, "ascii", &format!("base/{}", internal), &contents).await?;
        }
        Ok(())
    }
}

// ---------------- small pure helpers ----------------

fn sanitize(s: &str) -> String {
    s.replace('\\', "_").replace('/', "_").replace(' ', "_")
}

fn normalize_path(p: &str) -> String {
    p.replace('\\', "/").trim_start_matches('/').to_string()
}

fn build_globset(patterns: Option<&Vec<String>>) -> Result<Option<GlobSet>, globset::Error> {
    let Some(list) = patterns else { return Ok(None); };
    let mut b = GlobSetBuilder::new();
    for pat in list {
        // allow "**" etc.
        b.add(Glob::new(pat)?);
    }
    let set = b.build()?;
    Ok(Some(set))
}

fn should_include(path_like: &str, mode: &FilterMode, set: Option<&GlobSet>) -> bool {
    match *mode {
        FilterMode::All => true,
        FilterMode::Whitelist => {
            let Some(gs) = set else { return false; };
            gs.is_match(path_like)
        }
        FilterMode::Blacklist => {
            let Some(gs) = set else { return true; };
            !gs.is_match(path_like)
        }
    }
}