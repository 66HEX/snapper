use crate::youtube::{
    DownloadHistory, DownloadRequest, DownloadStatus, VideoInfo, YouTubeDownloader,
};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use tauri::AppHandle;
use tauri_plugin_store::StoreExt;

const STORE_FILE: &str = "download_history.json";
const HISTORY_KEY: &str = "downloads";
const SETTINGS_KEY: &str = "settings";

#[tauri::command]
pub async fn validate_youtube_url(url: String) -> Result<bool, String> {
    let output_dir = get_default_download_path().await?;
    let downloader = YouTubeDownloader::new(PathBuf::from(output_dir))
        .map_err(|e| format!("Failed to initialize downloader: {}", e))?;

    Ok(downloader.validate_url(&url))
}

#[tauri::command]
pub async fn get_video_info(url: String) -> Result<VideoInfo, String> {
    let output_dir = get_default_download_path().await?;
    let downloader = YouTubeDownloader::new(PathBuf::from(output_dir))
        .map_err(|e| format!("Failed to initialize downloader: {}", e))?;

    downloader
        .get_video_info(&url)
        .await
        .map_err(|e| format!("Failed to get video info: {}", e))
}

#[tauri::command]
pub async fn download_video(request: DownloadRequest, app: AppHandle) -> Result<String, String> {
    let output_dir = PathBuf::from(&request.output_path);
    let downloader = YouTubeDownloader::new(output_dir)
        .map_err(|e| format!("Failed to initialize downloader: {}", e))?;

    let download_id = uuid::Uuid::new_v4().to_string();

    let initial_history = DownloadHistory {
        id: download_id.clone(),
        title: "Pobieranie...".to_string(),
        url: request.url.clone(),
        status: DownloadStatus::Downloading,
        downloaded_at: Utc::now(),
        file_path: None,
        format: request.format.clone(),
        quality: request.quality.clone(),
    };

    if let Err(e) = save_download_to_store(&app, &initial_history).await {
        eprintln!("Failed to save initial download status: {}", e);
    }

    let app_clone = app.clone();
    let request_clone = request.clone();
    let download_id_clone = download_id.clone();
    let initial_history_clone = initial_history.clone();

    tokio::spawn(async move {
        println!("Starting download for ID: {}", download_id_clone);
        let result = downloader.download_video(request_clone.clone(), download_id_clone.clone()).await;

        match result {
            Ok(history) => {
                println!("Download successful for ID: {}, updating store", history.id);
                if let Err(e) = save_download_to_store(&app_clone, &history).await {
                    eprintln!("Failed to save download result: {}", e);
                } else {
                    println!("Successfully saved completed download to store");
                }
            }
            Err(e) => {
                eprintln!("Download failed for ID {}: {}", download_id_clone, e);

                let failed_history = DownloadHistory {
                    id: download_id_clone.clone(),
                    title: "Błąd pobierania".to_string(),
                    url: initial_history_clone.url,
                    status: DownloadStatus::Failed,
                    downloaded_at: Utc::now(),
                    file_path: None,
                    format: initial_history_clone.format,
                    quality: initial_history_clone.quality,
                };

                if let Err(e) = save_download_to_store(&app_clone, &failed_history).await {
                    eprintln!("Failed to save failed download status: {}", e);
                } else {
                    println!("Successfully saved failed download to store");
                }
            }
        }
    });

    Ok(download_id)
}

#[tauri::command]
pub async fn get_download_history(app: AppHandle) -> Result<Vec<DownloadHistory>, String> {
    load_history_from_store(&app).await
}

#[tauri::command]
pub async fn get_download_status(
    download_id: String,
    app: AppHandle,
) -> Result<Option<DownloadHistory>, String> {
    let history = load_history_from_store(&app).await?;
    Ok(history.into_iter().find(|d| d.id == download_id))
}

#[tauri::command]
pub async fn clear_download_history(app: AppHandle) -> Result<(), String> {
    let store = app
        .store(STORE_FILE)
        .map_err(|e| format!("Failed to get store: {}", e))?;

    store.delete(HISTORY_KEY);
    store
        .save()
        .map_err(|e| format!("Failed to save store: {}", e))?;

    Ok(())
}

#[tauri::command]
pub async fn get_default_download_path() -> Result<String, String> {
    let home_dir = dirs::home_dir().ok_or("Failed to get home directory")?;
    let downloads_dir = home_dir.join("Downloads");
    Ok(downloads_dir.to_string_lossy().to_string())
}



#[tauri::command]
pub async fn check_yt_dlp_installed() -> Result<bool, String> {
    match std::process::Command::new("yt-dlp")
        .arg("--version")
        .output()
    {
        Ok(output) => Ok(output.status.success()),
        Err(_) => Ok(false),
    }
}

#[tauri::command]
pub async fn get_supported_formats() -> Result<Vec<String>, String> {
    Ok(vec![
        "mp4".to_string(),
        "mp3".to_string(),
        "wav".to_string(),
        "webm".to_string(),
    ])
}

#[tauri::command]
pub async fn get_supported_qualities() -> Result<Vec<String>, String> {
    Ok(vec![
        "best".to_string(),
        "high".to_string(),
        "medium".to_string(),
        "low".to_string(),
        "worst".to_string(),
    ])
}

#[tauri::command]
pub async fn check_dependencies() -> Result<bool, String> {
    match YouTubeDownloader::check_dependencies().await {
        Ok(_) => Ok(true),
        Err(e) => {
            eprintln!("Dependencies check failed: {}", e);
            Ok(false)
        }
    }
}

#[tauri::command]
pub async fn get_debug_info() -> Result<serde_json::Value, String> {
    use serde_json::json;
    use std::env;
    
    let exe_path = env::current_exe()
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_else(|_| "unknown".to_string());
    
    let current_dir = env::current_dir()
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_else(|_| "unknown".to_string());
    
    let path_var = env::var("PATH").unwrap_or_else(|_| "not set".to_string());
    
    let app_dir = std::path::Path::new(&exe_path).parent()
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_else(|| "unknown".to_string());
    
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
    
    let yt_dlp_embedded = format!("{}/binaries/yt-dlp-{}", app_dir, target_triple);
    let ffmpeg_embedded = format!("{}/binaries/ffmpeg-{}", app_dir, target_triple);
    
    let yt_dlp_exists = std::path::Path::new(&yt_dlp_embedded).exists();
    let ffmpeg_exists = std::path::Path::new(&ffmpeg_embedded).exists();
    
    Ok(json!({
        "exe_path": exe_path,
        "current_dir": current_dir,
        "app_dir": app_dir,
        "path_var": path_var,
        "target_triple": target_triple,
        "yt_dlp_embedded_path": yt_dlp_embedded,
        "ffmpeg_embedded_path": ffmpeg_embedded,
        "yt_dlp_embedded_exists": yt_dlp_exists,
        "ffmpeg_embedded_exists": ffmpeg_exists
    }))
}

#[tauri::command]
pub async fn get_download_statistics(app: AppHandle) -> Result<DownloadStats, String> {
    let history = load_history_from_store(&app).await?;

    let total = history.len();
    let completed = history
        .iter()
        .filter(|d| matches!(d.status, DownloadStatus::Completed))
        .count();
    let failed = history
        .iter()
        .filter(|d| matches!(d.status, DownloadStatus::Failed))
        .count();
    let downloading = history
        .iter()
        .filter(|d| matches!(d.status, DownloadStatus::Downloading))
        .count();

    let formats: HashMap<String, usize> = history.iter().fold(HashMap::new(), |mut acc, d| {
        *acc.entry(d.format.clone()).or_insert(0) += 1;
        acc
    });

    Ok(DownloadStats {
        total,
        completed,
        failed,
        downloading,
        most_used_format: formats
            .iter()
            .max_by_key(|(_, &count)| count)
            .map(|(format, _)| format.clone()),
        formats_breakdown: formats,
    })
}

async fn load_history_from_store(app: &AppHandle) -> Result<Vec<DownloadHistory>, String> {
    let store = app
        .store(STORE_FILE)
        .map_err(|e| format!("Failed to get store: {}", e))?;

    match store.get(HISTORY_KEY) {
        Some(value) => serde_json::from_value::<Vec<DownloadHistory>>(value.clone())
            .map_err(|e| format!("Failed to deserialize history: {}", e)),
        None => Ok(Vec::new()),
    }
}

async fn save_download_to_store(app: &AppHandle, download: &DownloadHistory) -> Result<(), String> {
    let store = app
        .store(STORE_FILE)
        .map_err(|e| format!("Failed to get store: {}", e))?;

    let mut history = load_history_from_store(app).await?;

    if let Some(index) = history.iter().position(|d| d.id == download.id) {
        history[index] = download.clone();
    } else {
        history.push(download.clone());
    }

    history.sort_by(|a, b| b.downloaded_at.cmp(&a.downloaded_at));

    if history.len() > 100 {
        history.truncate(100);
    }

    let history_value = serde_json::to_value(&history)
        .map_err(|e| format!("Failed to serialize history: {}", e))?;

    store.set(HISTORY_KEY, history_value);
    store
        .save()
        .map_err(|e| format!("Failed to save store: {}", e))?;

    Ok(())
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct DownloadStats {
    pub total: usize,
    pub completed: usize,
    pub failed: usize,
    pub downloading: usize,
    pub most_used_format: Option<String>,
    pub formats_breakdown: HashMap<String, usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppSettings {
    pub download_path: String,
    pub default_format: String,
    pub default_quality: String,
}

impl Default for AppSettings {
    fn default() -> Self {
        let home_dir = dirs::home_dir().unwrap_or_else(|| PathBuf::from("/"));
        let downloads_dir = home_dir.join("Downloads");
        
        Self {
            download_path: downloads_dir.to_string_lossy().to_string(),
            default_format: "mp4".to_string(),
            default_quality: "high".to_string(),
        }
    }
}

#[tauri::command]
pub async fn save_settings(settings: AppSettings, app: AppHandle) -> Result<(), String> {
    let store = app
        .store(STORE_FILE)
        .map_err(|e| format!("Failed to get store: {}", e))?;

    let settings_value = serde_json::to_value(&settings)
        .map_err(|e| format!("Failed to serialize settings: {}", e))?;
    store.set(SETTINGS_KEY, settings_value);
    store
        .save()
        .map_err(|e| format!("Failed to save settings: {}", e))?;

    println!("Settings saved: {:?}", settings);
    Ok(())
}

#[tauri::command]
pub async fn load_settings(app: AppHandle) -> Result<AppSettings, String> {
    let store = app
        .store(STORE_FILE)
        .map_err(|e| format!("Failed to get store: {}", e))?;

    match store.get(SETTINGS_KEY) {
        Some(value) => {
            let settings: AppSettings = serde_json::from_value(value.clone())
                .map_err(|e| format!("Failed to deserialize settings: {}", e))?;
            println!("Settings loaded: {:?}", settings);
            Ok(settings)
        }
        None => {
            println!("No settings found, using defaults");
            let default_settings = AppSettings::default();
            
            save_settings(default_settings.clone(), app).await?;
            Ok(default_settings)
        }
    }
}
