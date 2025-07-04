use anyhow::{anyhow, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VideoInfo {
    pub id: String,
    pub title: String,
    pub url: String,
    pub duration: Option<u64>,
    pub thumbnail: Option<String>,
    pub uploader: Option<String>,
    pub upload_date: Option<String>,
    pub view_count: Option<u64>,
    pub available_formats: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DownloadRequest {
    pub url: String,
    pub format: String,
    pub quality: String,
    pub output_path: String,
    pub filename: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DownloadProgress {
    pub id: String,
    pub status: DownloadStatus,
    pub progress: f64,
    pub speed: Option<String>,
    pub eta: Option<String>,
    pub filename: Option<String>,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DownloadStatus {
    Pending,
    Downloading,
    Completed,
    Failed,
    Cancelled,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DownloadHistory {
    pub id: String,
    pub title: String,
    pub url: String,
    pub status: DownloadStatus,
    pub downloaded_at: DateTime<Utc>,
    pub file_path: Option<String>,
    pub format: String,
    pub quality: String,
}

pub struct YouTubeDownloader {
    _output_dir: PathBuf,
}

impl YouTubeDownloader {
    pub fn new(output_dir: PathBuf) -> Result<Self> {
        Self::find_yt_dlp_path()?;
        Self::find_ffmpeg_path()?;

        Ok(Self {
            _output_dir: output_dir,
        })
    }

    fn find_yt_dlp_path() -> Result<PathBuf> {
        let embedded_path = Self::get_embedded_binary_path("yt-dlp")?;
        if embedded_path.exists() {
            println!("Using embedded yt-dlp: {}", embedded_path.display());
            return Ok(embedded_path);
        }

        let command_name = if cfg!(windows) { "yt-dlp.exe" } else { "yt-dlp" };
        
        if let Ok(output) = Self::run_which_command(command_name) {
            if output.status.success() {
                let path_str = String::from_utf8(output.stdout)?;
                let path = PathBuf::from(path_str.trim());
                if path.exists() {
                    println!("Using system yt-dlp: {}", path.display());
                    return Ok(path);
                }
            }
        }

        let common_paths = Self::get_common_paths("yt-dlp");
        for path in &common_paths {
            let path_buf = PathBuf::from(path);
            if path_buf.exists() {
                println!("Using yt-dlp from common path: {}", path_buf.display());
                return Ok(path_buf);
            }
        }

        Err(anyhow!("yt-dlp not found. Please install yt-dlp or place the binary in the application directory."))
    }

    fn find_ffmpeg_path() -> Result<PathBuf> {
        let embedded_path = Self::get_embedded_binary_path("ffmpeg")?;
        if embedded_path.exists() {
            println!("Using embedded ffmpeg: {}", embedded_path.display());
            return Ok(embedded_path);
        }

        let command_name = if cfg!(windows) { "ffmpeg.exe" } else { "ffmpeg" };
        
        if let Ok(output) = Self::run_which_command(command_name) {
            if output.status.success() {
                let path_str = String::from_utf8(output.stdout)?;
                let path = PathBuf::from(path_str.trim());
                if path.exists() {
                    println!("Using system ffmpeg: {}", path.display());
                    return Ok(path);
                }
            }
        }

        let common_paths = Self::get_common_paths("ffmpeg");
        for path in &common_paths {
            let path_buf = PathBuf::from(path);
            if path_buf.exists() {
                println!("Using ffmpeg from common path: {}", path_buf.display());
                return Ok(path_buf);
            }
        }

        Err(anyhow!("ffmpeg not found. Please install ffmpeg or place the binary in the application directory."))
    }

    fn get_embedded_binary_path(binary_name: &str) -> Result<PathBuf> {
        let exe_path = std::env::current_exe()?;
        let app_dir = exe_path.parent().ok_or_else(|| anyhow!("Cannot get app directory"))?;
        
        let target_triple = if cfg!(target_os = "macos") {
            if cfg!(target_arch = "aarch64") {
                "aarch64-apple-darwin"
            } else {
                "x86_64-apple-darwin"
            }
        } else if cfg!(target_os = "windows") {
            "x86_64-pc-windows-msvc"
        } else {
            "x86_64-unknown-linux-gnu"
        };
        
        let binary_filename = if cfg!(windows) {
            format!("{}.exe", binary_name)
        } else {
            binary_name.to_string()
        };
        
        let target_specific_filename = if cfg!(windows) {
            format!("{}-{}.exe", binary_name, target_triple)
        } else {
            format!("{}-{}", binary_name, target_triple)
        };
        
        let possible_paths = [
            app_dir.join("../Resources/binaries").join(&target_specific_filename),
            app_dir.join("../Resources").join(&target_specific_filename),
            app_dir.join("binaries").join(&target_specific_filename),
            app_dir.join(&target_specific_filename),
            app_dir.join("libs").join(&target_specific_filename),
            app_dir.join("resources").join(&target_specific_filename),
            app_dir.join("../Resources/binaries").join(&binary_filename),
            app_dir.join("../Resources").join(&binary_filename),
            app_dir.join(&binary_filename),
            app_dir.join("binaries").join(&binary_filename),
            app_dir.join("libs").join(&binary_filename),
            app_dir.join("resources").join(&binary_filename),
        ];
        
        for path in &possible_paths {
            if path.exists() {
                println!("Found embedded binary: {}", path.display());
                return Ok(path.clone());
            }
        }
        
        Ok(possible_paths[0].clone())
    }

    fn run_which_command(command_name: &str) -> Result<std::process::Output, std::io::Error> {
        if cfg!(windows) {
            std::process::Command::new("where").arg(command_name).output()
        } else {
            let mut cmd = std::process::Command::new("which");
            cmd.arg(command_name);
            
            let current_path = std::env::var("PATH").unwrap_or_default();
            let extended_path = format!(
                "{}:/usr/local/bin:/opt/homebrew/bin:/usr/bin:/bin:/opt/local/bin",
                current_path
            );
            cmd.env("PATH", extended_path);
            
            cmd.output()
        }
    }

    fn get_common_paths(binary_name: &str) -> Vec<String> {
        let binary_filename = if cfg!(windows) {
            format!("{}.exe", binary_name)
        } else {
            binary_name.to_string()
        };

        if cfg!(windows) {
            vec![
                format!(r"C:\Program Files\{}\{}", binary_name, binary_filename),
                format!(r"C:\Program Files (x86)\{}\{}", binary_name, binary_filename),
                format!(r"C:\{}\{}", binary_name, binary_filename),
                format!(r".\libs\{}", binary_filename),
                format!(r".\binaries\{}", binary_filename),
                format!(r"C:\ProgramData\chocolatey\bin\{}", binary_filename),
                format!(r"C:\Users\{}\scoop\apps\{}\current\{}", 
                       std::env::var("USERNAME").unwrap_or_else(|_| "User".to_string()), 
                       binary_name, binary_filename),
            ]
        } else if cfg!(target_os = "macos") {
            vec![
                format!("/usr/local/bin/{}", binary_filename),
                format!("/opt/homebrew/bin/{}", binary_filename),
                format!("/usr/bin/{}", binary_filename),
                format!("/opt/local/bin/{}", binary_filename),
                format!("./libs/{}", binary_filename),
                format!("./binaries/{}", binary_filename),
            ]
        } else {
            vec![
                format!("/usr/local/bin/{}", binary_filename),
                format!("/usr/bin/{}", binary_filename),
                format!("/bin/{}", binary_filename),
                format!("/snap/bin/{}", binary_filename),
                format!("./libs/{}", binary_filename),
                format!("./binaries/{}", binary_filename),
            ]
        }
    }

    pub async fn get_video_info(&self, url: &str) -> Result<VideoInfo> {
        let yt_dlp_path = Self::find_yt_dlp_path()?;
        let cache_dir = self.get_cache_dir()?;
        let output = std::process::Command::new(&yt_dlp_path)
            .args(["--dump-json", "--no-playlist", "--cache-dir", &cache_dir, url])
            .output()?;

        if !output.status.success() {
            let error = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow!("Failed to get video info: {}", error));
        }

        let json_str = String::from_utf8(output.stdout)?;
        let video_data: serde_json::Value = serde_json::from_str(&json_str)?;

        let available_formats = video_data["formats"]
            .as_array()
            .unwrap_or(&vec![])
            .iter()
            .filter_map(|f| f["ext"].as_str().map(|s| s.to_string()))
            .collect::<std::collections::HashSet<_>>()
            .into_iter()
            .collect();

        let video_info = VideoInfo {
            id: video_data["id"].as_str().unwrap_or("unknown").to_string(),
            title: video_data["title"]
                .as_str()
                .unwrap_or("Unknown Title")
                .to_string(),
            url: url.to_string(),
            duration: video_data["duration"].as_u64(),
            thumbnail: video_data["thumbnail"].as_str().map(|s| s.to_string()),
            uploader: video_data["uploader"].as_str().map(|s| s.to_string()),
            upload_date: video_data["upload_date"].as_str().map(|s| s.to_string()),
            view_count: video_data["view_count"].as_u64(),
            available_formats,
        };

        Ok(video_info)
    }

    pub async fn download_video(&self, request: DownloadRequest, download_id: String) -> Result<DownloadHistory> {

        let video_info = self.get_video_info(&request.url).await?;

        let clean_title = video_info
            .title
            .chars()
            .filter(|c| c.is_alphanumeric() || *c == ' ' || *c == '-' || *c == '_' || *c == '.')
            .collect::<String>()
            .replace("  ", " ")
            .trim()
            .to_string();

        let clean_title = if clean_title.len() > 100 {
            clean_title.chars().take(100).collect()
        } else {
            clean_title
        };

        let filename = request
            .filename
            .unwrap_or_else(|| format!("{}.{}", clean_title, request.format));

        println!("Downloading: {} as {}", video_info.title, filename);

        let result = self
            .download_with_cli(&request.url, &filename, &request.format, &request.quality)
            .await;

        let history = match result {
            Ok(file_path) => DownloadHistory {
                id: download_id,
                title: video_info.title,
                url: request.url,
                status: DownloadStatus::Completed,
                downloaded_at: Utc::now(),
                file_path: Some(file_path.to_string_lossy().to_string()),
                format: request.format,
                quality: request.quality,
            },
            Err(_e) => DownloadHistory {
                id: download_id,
                title: video_info.title,
                url: request.url,
                status: DownloadStatus::Failed,
                downloaded_at: Utc::now(),
                file_path: None,
                format: request.format,
                quality: request.quality,
            },
        };

        Ok(history)
    }

    async fn download_with_cli(
        &self,
        url: &str,
        filename: &str,
        format: &str,
        quality: &str,
    ) -> Result<PathBuf> {
        println!("Downloading with CLI: {} as {} ({})", url, filename, format);

        let filename_without_ext = if let Some(stem) = std::path::Path::new(filename).file_stem() {
            stem.to_string_lossy().to_string()
        } else {
            filename.to_string()
        };

        let output_dir = self._output_dir.to_string_lossy();
        let output_template = format!("{}/{}.%(ext)s", output_dir, filename_without_ext);
        
        let cache_dir = self.get_cache_dir()?;

        let yt_dlp_path = Self::find_yt_dlp_path()?;
        let mut cmd = std::process::Command::new(&yt_dlp_path);
        
        if cfg!(target_os = "macos") {
            let current_path = std::env::var("PATH").unwrap_or_default();
            let extended_path = format!(
                "{}:/usr/local/bin:/opt/homebrew/bin:/usr/bin:/bin:/opt/local/bin",
                current_path
            );
            cmd.env("PATH", extended_path);
        }
        
        cmd.args([
            "--cache-dir", &cache_dir,
            "--no-playlist",
            "--user-agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/91.0.4472.124 Safari/537.36",
            "--referer", "https://www.youtube.com/",
            "--extractor-retries", "3",
            "--fragment-retries", "3",
        ]);

        match format {
            "mp3" => {
                cmd.args([
                    "-x",
                    "--audio-format",
                    "mp3",
                    "--audio-quality",
                    &self.get_audio_quality_param(quality),
                    "-o",
                    &output_template,
                    url,
                ]);
            }
            "wav" => {
                cmd.args([
                    "-x",
                    "--audio-format",
                    "wav",
                    "--audio-quality",
                    "0",
                    "-o",
                    &output_template,
                    url,
                ]);
            }
            "mp4" => {
                let format_selector = self.get_mp4_format_selector(quality);
                cmd.args([
                    "-f",
                    &format_selector,
                    "--merge-output-format",
                    "mp4",
                    "--extractor-args",
                    "youtube:player_client=android,web",
                    "--no-check-formats",
                    "--prefer-free-formats",
                    "-o",
                    &output_template,
                    url,
                ]);
            }
            "webm" => {
                let format_selector = self.get_webm_format_selector(quality);
                cmd.args([
                    "-f",
                    &format_selector,
                    "--merge-output-format",
                    "webm",
                    "--extractor-args",
                    "youtube:player_client=android,web",
                    "--no-check-formats",
                    "-o",
                    &output_template,
                    url,
                ]);
            }
            _ => {
                return Err(anyhow!("Unsupported format: {}", format));
            }
        }

        println!("Running command: {:?}", cmd);
        let output = cmd.output()?;

        if !output.status.success() {
            let error = String::from_utf8_lossy(&output.stderr);
            eprintln!("yt-dlp error: {}", error);
            
            if error.contains("Requested format is not available") || error.contains("nsig extraction failed") {
                eprintln!("Primary download failed, trying fallback strategy...");
                
                if let Err(list_error) = self.list_available_formats(url).await {
                    eprintln!("Failed to list formats: {}", list_error);
                }
                
                return self.fallback_download(url, filename, format, quality).await;
            }
            
            return Err(anyhow!("Download failed: {}", error));
        }

        let expected_path = self
            ._output_dir
            .join(&format!("{}.{}", filename_without_ext, format));

        let result = if expected_path.exists() {
            println!("File downloaded successfully: {}", expected_path.display());
            Ok(expected_path)
        } else {
            if let Ok(entries) = std::fs::read_dir(&self._output_dir) {
                for entry in entries {
                    if let Ok(entry) = entry {
                        let path = entry.path();
                        if let Some(file_stem) = path.file_stem() {
                            if file_stem
                                .to_string_lossy()
                                .starts_with(&filename_without_ext)
                            {
                                println!("Found downloaded file: {}", path.display());
                                return Ok(path);
                            }
                        }
                    }
                }
            }
            Err(anyhow!(
                "Downloaded file not found at: {}",
                expected_path.display()
            ))
        };

        if let Err(e) = self.cleanup_cache(&cache_dir) {
            eprintln!("Warning: Failed to cleanup cache: {}", e);
        }

        result
    }

    fn get_audio_quality_param(&self, quality: &str) -> String {
        match quality {
            "best" => "0".to_string(),
            "high" => "192K".to_string(),
            "medium" => "128K".to_string(),
            "low" => "96K".to_string(),
            "worst" => "64K".to_string(),
            _ => "192K".to_string(),
        }
    }

    fn get_video_height_param(&self, quality: &str) -> u32 {
        match quality {
            "best" => 2160,
            "high" => 1080,
            "medium" => 720,
            "low" => 480,
            "worst" => 360,
            _ => 1080,
        }
    }

    fn get_mp4_format_selector(&self, quality: &str) -> String {
        let height = self.get_video_height_param(quality);
        match quality {
            "best" => {
                format!(
                    "bestvideo[height<={}][ext=mp4]+bestaudio[ext=m4a]/bestvideo[height<={}]+bestaudio/best[height<={}]/best",
                    height, height, height
                )
            }
            _ => {
                format!(
                    "bestvideo[height<={}]+bestaudio/best[height<={}]/bestvideo[height<={}]/best",
                    height, height, height
                )
            }
        }
    }

    fn get_webm_format_selector(&self, quality: &str) -> String {
        let height = self.get_video_height_param(quality);
        match quality {
            "best" => {
                format!(
                    "bestvideo[height<={}][ext=webm]+bestaudio[ext=webm]/bestvideo[height<={}]+bestaudio/best[height<={}]/best",
                    height, height, height
                )
            }
            _ => {
                format!(
                    "bestvideo[height<={}]+bestaudio/best[height<={}]/bestvideo[height<={}]/best",
                    height, height, height
                )
            }
        }
    }

    async fn list_available_formats(&self, url: &str) -> Result<()> {
        let yt_dlp_path = Self::find_yt_dlp_path()?;
        let mut cmd = std::process::Command::new(&yt_dlp_path);
        
        if cfg!(target_os = "macos") {
            let current_path = std::env::var("PATH").unwrap_or_default();
            let extended_path = format!(
                "{}:/usr/local/bin:/opt/homebrew/bin:/usr/bin:/bin:/opt/local/bin",
                current_path
            );
            cmd.env("PATH", extended_path);
        }
        
        cmd.args([
            "--list-formats",
            "--no-playlist",
            url,
        ]);

        let output = cmd.output()?;
        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);
        
        eprintln!("Available formats for {}:", url);
        eprintln!("STDOUT:\n{}", stdout);
        if !stderr.is_empty() {
            eprintln!("STDERR:\n{}", stderr);
        }
        
        Ok(())
    }

    async fn fallback_download(
        &self,
        url: &str,
        filename: &str,
        format: &str,
        _quality: &str,
    ) -> Result<PathBuf> {
        eprintln!("Attempting fallback download with simpler parameters");

        let filename_without_ext = if let Some(stem) = std::path::Path::new(filename).file_stem() {
            stem.to_string_lossy().to_string()
        } else {
            filename.to_string()
        };

        let output_dir = self._output_dir.to_string_lossy();
        let output_template = format!("{}/{}.%(ext)s", output_dir, filename_without_ext);
        
        let cache_dir = self.get_cache_dir()?;
        let yt_dlp_path = Self::find_yt_dlp_path()?;
        let mut cmd = std::process::Command::new(&yt_dlp_path);
        
        if cfg!(target_os = "macos") {
            let current_path = std::env::var("PATH").unwrap_or_default();
            let extended_path = format!(
                "{}:/usr/local/bin:/opt/homebrew/bin:/usr/bin:/bin:/opt/local/bin",
                current_path
            );
            cmd.env("PATH", extended_path);
        }
        
        cmd.args([
            "--cache-dir", &cache_dir,
            "--no-playlist",
            "--user-agent", "Mozilla/5.0 (compatible; yt-dlp)",
        ]);

        match format {
            "mp3" => {
                cmd.args([
                    "-x",
                    "--audio-format", "mp3",
                    "--audio-quality", "192K",
                    "-o", &output_template,
                    url,
                ]);
            }
            "wav" => {
                cmd.args([
                    "-x",
                    "--audio-format", "wav",
                    "--audio-quality", "0",
                    "-o", &output_template,
                    url,
                ]);
            }
            "mp4" => {
                cmd.args([
                    "-f", "best/worst",
                    "--recode-video", "mp4",
                    "-o", &output_template,
                    url,
                ]);
            }
            "webm" => {
                cmd.args([
                    "-f", "best/worst",
                    "--recode-video", "webm",
                    "-o", &output_template,
                    url,
                ]);
            }
            _ => {
                return Err(anyhow!("Unsupported format for fallback: {}", format));
            }
        }

        eprintln!("Running fallback command: {:?}", cmd);
        let output = cmd.output()?;

        if !output.status.success() {
            let error = String::from_utf8_lossy(&output.stderr);
            eprintln!("Fallback download also failed: {}", error);
            return Err(anyhow!("Fallback download failed: {}", error));
        }

        let expected_path = self._output_dir.join(&format!("{}.{}", filename_without_ext, format));
        if expected_path.exists() {
            eprintln!("Fallback download successful: {}", expected_path.display());
            return Ok(expected_path);
        }

        if let Ok(entries) = std::fs::read_dir(&self._output_dir) {
            for entry in entries {
                if let Ok(entry) = entry {
                    let path = entry.path();
                    if let Some(file_stem) = path.file_stem() {
                        if file_stem.to_string_lossy().starts_with(&filename_without_ext) {
                            eprintln!("Found fallback download file: {}", path.display());
                            return Ok(path);
                        }
                    }
                }
            }
        }

        Err(anyhow!("Fallback download file not found"))
    }
    
    fn get_cache_dir(&self) -> Result<String> {
        let temp_dir = std::env::temp_dir();
        let cache_dir = temp_dir.join("snapper-cache");
        
        std::fs::create_dir_all(&cache_dir)
            .map_err(|e| anyhow!("Failed to create cache directory: {}", e))?;
            
        Ok(cache_dir.to_string_lossy().to_string())
    }

    fn cleanup_cache(&self, cache_dir: &str) -> Result<()> {
        self.cleanup_single_cache_dir(cache_dir)?;
        
        let output_cache_dir = self._output_dir.join("cache");
        if output_cache_dir.exists() {
            let output_cache_str = output_cache_dir.to_string_lossy().to_string();
            println!("Also cleaning output directory cache: {}", output_cache_str);
            self.cleanup_single_cache_dir(&output_cache_str)?;
        }
        
        Ok(())
    }
    
    fn cleanup_single_cache_dir(&self, cache_dir: &str) -> Result<()> {
        let cache_path = std::path::Path::new(cache_dir);
        
        println!("Cleaning up cache directory: {}", cache_dir);
        
        if cache_path.exists() && cache_path.is_dir() {
            println!("Cache directory exists, cleaning contents...");
            
            if let Ok(entries) = std::fs::read_dir(cache_path) {
                let entries: Vec<_> = entries.collect();
                println!("Found {} entries in cache directory", entries.len());
                
                for entry in entries {
                    if let Ok(entry) = entry {
                        let path = entry.path();
                        println!("Removing cache entry: {:?}", path);
                        
                        if path.is_file() {
                            match std::fs::remove_file(&path) {
                                Ok(_) => println!("Successfully removed cache file: {:?}", path),
                                Err(e) => eprintln!("Failed to remove cache file {:?}: {}", path, e),
                            }
                        } else if path.is_dir() {
                            match std::fs::remove_dir_all(&path) {
                                Ok(_) => println!("Successfully removed cache directory: {:?}", path),
                                Err(e) => eprintln!("Failed to remove cache directory {:?}: {}", path, e),
                            }
                        }
                    }
                }
            } else {
                eprintln!("Failed to read cache directory: {}", cache_dir);
            }
            
            if let Ok(entries) = std::fs::read_dir(cache_path) {
                let remaining: Vec<_> = entries.collect();
                if remaining.is_empty() {
                    println!("Cache directory is empty, removing it...");
                    match std::fs::remove_dir(cache_path) {
                        Ok(_) => println!("Successfully removed empty cache directory"),
                        Err(e) => eprintln!("Failed to remove empty cache directory: {}", e),
                    }
                } else {
                    eprintln!("Warning: {} entries still remain in cache directory", remaining.len());
                }
            }
        } else {
            println!("Cache directory does not exist or is not a directory: {}", cache_dir);
        }
        
        Ok(())
    }

    pub fn validate_url(&self, url: &str) -> bool {
        url.contains("youtube.com/watch")
            || url.contains("youtu.be/")
            || url.contains("youtube.com/playlist")
            || url.contains("youtube.com/shorts/")
    }

    pub async fn check_dependencies() -> Result<()> {
        Self::find_yt_dlp_path()?;

        Self::find_ffmpeg_path()?;

        Ok(())
    }
}
