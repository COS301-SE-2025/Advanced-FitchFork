use std::path::PathBuf;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::TcpStream;
use zip::ZipArchive;
use std::io::Cursor;
use std::path::Path;

const MOSS_SERVER: &str = "moss.stanford.edu";
const MOSS_PORT: u16 = 7690;

/// A service for interacting with the MOSS (Measure of Software Similarity) server.
pub struct MossService {
    user_id: String,
}

impl MossService {
    /// Creates a new `MossService` with the given user ID.
    pub fn new(user_id: &str) -> Self {
        Self {
            user_id: user_id.to_string(),
        }
    }

    /// Runs a MOSS check by uploading base files and submission files to the MOSS server.
    ///
    /// # Arguments
    ///
    /// * `base_files` - A list of files to be used as the base for comparison (template/starter code).
    /// * `submission_files` - A list of tuples containing (file_path, optional_username) for submissions.
    /// * `language` - The programming language of the files.
    ///
    /// # Returns
    ///
    /// A `Result` containing the MOSS report URL on success, or an error message on failure.
    pub async fn run(
        &self,
        base_files: Vec<PathBuf>,
        submission_files: Vec<(PathBuf, Option<String>, Option<i64>)>,
        language: &str,
    ) -> Result<String, String> {
        if submission_files.len() < 2 {
            return Err("MOSS requires at least 2 submission files to compare".to_string());
        }
        
        let mut stream = TcpStream::connect((MOSS_SERVER, MOSS_PORT))
            .await
            .map_err(|e| format!("Failed to connect to MOSS server: {}", e))?;

        self.send_command(&mut stream, &format!("moss {}", self.user_id)).await?;
        self.send_command(&mut stream, "directory 0").await?;
        self.send_command(&mut stream, "X 0").await?;
        self.send_command(&mut stream, "maxmatches 10").await?;
        self.send_command(&mut stream, "show 250").await?;
        self.send_command(&mut stream, &format!("language {}", language)).await?;
        
        let mut response = String::new();
        {
            let mut reader = BufReader::new(&mut stream);
            reader.read_line(&mut response).await.map_err(|e| {
                format!("Failed to read language response: {}", e)
            })?;
        }
        
        if response.trim() == "no" {
            return Err(format!("Language '{}' not supported by MOSS", language));
        }

        for path in &base_files {
            self.upload_file(&mut stream, path, 0, language, None, None).await?;
        }

        let mut file_id = 1u32;
        for (path, username, submission_id) in &submission_files {
            if path.extension().and_then(|s| s.to_str()) == Some("zip") {
                self.send_command(&mut stream, "directory 1").await?;
                file_id = self.upload_zip(
                    &mut stream, 
                    path, 
                    file_id, 
                    language, 
                    username.as_deref(), 
                    *submission_id
                ).await?;
                self.send_command(&mut stream, "directory 0").await?;
            } else {
                file_id = self.upload_file(&mut stream, path, file_id, language, username.as_deref(), *submission_id).await?;
            }
        }

        self.send_command(&mut stream, "query 0 ").await?;

        let mut response = String::new();
        {
            let mut reader = BufReader::new(&mut stream);
            reader.read_line(&mut response).await.map_err(|e| {
                format!("Failed to read query response: {}", e)
            })?;
        }
        
        let report_url = response.trim().to_string();

        self.send_command(&mut stream, "end").await?;
        
        if !report_url.starts_with("http") {
            return Err(format!("Invalid response from MOSS server: '{}'", report_url));
        }

        Ok(report_url)
    }

    async fn send_command(&self, stream: &mut TcpStream, command: &str) -> Result<(), String> {
        let command_with_newline = format!("{}\n", command);
        stream.write_all(command_with_newline.as_bytes()).await.map_err(|e| {
            format!("Failed to send command '{}': {}", command, e)
        })?;
        Ok(())
    }

    /// Uploads a single file to the MOSS server with username and submission ID prefix.
    /// Returns the next file_id.
    async fn upload_file(
        &self,
        stream: &mut TcpStream,
        path: &Path,
        file_id: u32,
        language: &str,
        username: Option<&str>,
        submission_id: Option<i64>,
    ) -> Result<u32, String> {
        if !path.exists() {
            return Err(format!("File does not exist: {}", path.display()));
        }

        let content = tokio::fs::read(path).await.map_err(|e| {
            format!("Failed to read file {}: {}", path.display(), e)
        })?;

        let original_filename = path
            .file_name()
            .ok_or_else(|| format!("Invalid filename: {}", path.display()))?
            .to_str()
            .ok_or_else(|| format!("Non-UTF8 filename: {}", path.display()))?
            .replace(' ', "_");

        let filename = match (username, submission_id) {
            (Some(username), Some(submission_id)) => {
                format!("{}_{}_{}", username, submission_id, original_filename)
            }
            (Some(username), None) => {
                format!("{}_{}", username, original_filename)
            }
            (None, Some(submission_id)) => {
                format!("{}_{}", submission_id, original_filename)
            }
            (None, None) => original_filename,
        };

        let command = format!("file {} {} {} {}", file_id, language, content.len(), filename);
        self.send_command(stream, &command).await?;
        
        stream.write_all(&content).await.map_err(|e| {
            format!("Failed to upload file content: {}", e)
        })?;
        
        Ok(file_id + 1)
    }

    /// Uploads a ZIP file as a single directory submission
    /// Returns the next file_id.
    async fn upload_zip(
        &self,
        stream: &mut TcpStream,
        zip_path: &Path,
        starting_file_id: u32,
        language: &str,
        username: Option<&str>,
        submission_id: Option<i64>,
    ) -> Result<u32, String> {
        let zip_data = tokio::fs::read(zip_path).await.map_err(|e| {
            format!("Failed to read ZIP file {}: {}", zip_path.display(), e)
        })?;
        
        let cursor = Cursor::new(zip_data);
        let mut archive = ZipArchive::new(cursor).map_err(|e| {
            format!("Failed to open ZIP file {}: {}", zip_path.display(), e)
        })?;
        
        let directory_name = match (username, submission_id) {
            (Some(u), Some(sid)) => format!("{}_{}", u, sid),
            (Some(u), None) => format!("{}", u),
            (None, Some(sid)) => format!("{}", sid),
            (None, None) => "".to_string(),
        };
        
        let sanitized_dir_name = directory_name
            .replace('/', "_")
            .replace('\\', "_")
            .replace(' ', "_");

        let mut current_file_id = starting_file_id;
        let mut files_uploaded = 0;

        for i in 0..archive.len() {
            let mut file = archive.by_index(i).map_err(|e| {
                format!("Failed to read file {} from ZIP: {}", i, e)
            })?;
            
            if file.is_dir() {
                continue;
            }

            let mut contents = Vec::new();
            std::io::copy(&mut file, &mut contents).map_err(|e| {
                format!("Failed to read file contents: {}", e)
            })?;

            let internal_path = file.name()
                .replace('\\', "/")
                .trim_start_matches('/')
                .to_string();
            
            if internal_path.is_empty() {
                continue;
            }

            let display_name = format!("{}/{}", sanitized_dir_name, internal_path)
                .replace(' ', "_");

            let command = format!(
                "file {} {} {} {}",
                current_file_id,
                language,
                contents.len(),
                display_name
            );

            self.send_command(stream, &command).await?;
            stream.write_all(&contents).await.map_err(|e| {
                format!("Failed to upload file content: {}", e)
            })?;

            current_file_id += 1;
            files_uploaded += 1;
        }

        if files_uploaded == 0 {
            return Err(format!("ZIP contained no valid files: {}", zip_path.display()));
        }

        Ok(current_file_id)
    }
}