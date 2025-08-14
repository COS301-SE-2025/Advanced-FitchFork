use std::path::{Path, PathBuf};
use tokio::io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader};
use tokio::net::TcpStream;
use tracing::info;

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
    /// * `base_files` - A list of files to be used as the base for comparison.
    /// * `submission_files` - A list of submission files to be compared against each other and the base files.
    /// * `language` - The programming language of the files.
    ///
    /// # Returns
    ///
    /// A `Result` containing the MOSS report URL on success, or an error message on failure.
    pub async fn run(
        &self,
        base_files: Vec<PathBuf>,
        submission_files: Vec<PathBuf>,
        language: &str,
    ) -> Result<String, String> {
        let mut stream = TcpStream::connect((MOSS_SERVER, MOSS_PORT))
            .await
            .map_err(|e| format!("Failed to connect to MOSS server: {}", e))?;

        self.send_command(&mut stream, format!("moss {}\n", self.user_id)).await?;
        
        self.send_command(&mut stream, "directory 0\n".to_string()).await?;
        self.send_command(&mut stream, "X 0\n".to_string()).await?;
        self.send_command(&mut stream, "maxmatches 10\n".to_string()).await?;
        self.send_command(&mut stream, "show 250\n".to_string()).await?;

        self.send_command(&mut stream, format!("language {}\n", language)).await?;
        let mut lang_response = [0; 1024];
        let n = stream.read(&mut lang_response).await.map_err(|e| e.to_string())?;
        let response_str = String::from_utf8_lossy(&lang_response[..n]);
        info!("MOSS server response for language check: '{}'", response_str.trim());
        if response_str.trim() == "no" {
            return Err(format!("Language {} not supported by MOSS.", language));
        }

        for path in &base_files {
            self.upload_file(&mut stream, path, 0, language).await?;
        }

        for (i, path) in submission_files.iter().enumerate() {
            self.upload_file(&mut stream, path, (i + 1) as u32, language).await?;
        }

        self.send_command(&mut stream, "query 0 \n".to_string()).await?;

        let mut response = String::new();
        {
            let mut reader = BufReader::new(&mut stream);
            reader.read_line(&mut response).await.map_err(|e| e.to_string())?;
        }
        info!("MOSS server response URL: '{}'", response.trim());


        self.send_command(&mut stream, "end\n".to_string()).await?;

        Ok(response.trim().to_string())
    }

    /// Sends a command to the MOSS server.
    async fn send_command(&self, stream: &mut TcpStream, command: String) -> Result<(), String> {
        info!("Sending command to MOSS server: {}", command.trim());
        stream.write_all(command.as_bytes()).await.map_err(|e| e.to_string())?;
        Ok(())
    }

    /// Uploads a file to the MOSS server.
    async fn upload_file(
        &self,
        stream: &mut TcpStream,
        path: &Path,
        file_id: u32,
        language: &str,
    ) -> Result<(), String> {
        let content = tokio::fs::read(path).await.map_err(|e| e.to_string())?;
        let filename = path.file_name().unwrap().to_str().unwrap().replace(" ", "_");
        let command = format!(
            "file {} {} {} {}\n",
            file_id,
            language,
            content.len(),
            filename
        );

        println!("Uploading {}...", path.display());
        self.send_command(stream, command).await?;
        stream.write_all(&content).await.map_err(|e| e.to_string())?;
        println!("...done.");

        Ok(())
    }
}