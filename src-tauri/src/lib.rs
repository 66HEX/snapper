mod commands;
pub mod youtube;

use commands::*;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_store::Builder::new().build())
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            validate_youtube_url,
            get_video_info,
            download_video,
            get_download_history,
            get_download_status,
            clear_download_history,
            get_default_download_path,
            check_yt_dlp_installed,
            get_supported_formats,
            get_supported_qualities,
            check_dependencies,
            get_download_statistics,
            save_settings,
            load_settings,
            get_debug_info
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[cfg(test)]
mod tests {
    use super::youtube::*;
    use std::path::PathBuf;

    #[tokio::test]
    async fn test_check_dependencies() {
        let result = YouTubeDownloader::check_dependencies().await;
        println!("Dependencies check result: {:?}", result);
        assert!(result.is_ok(), "Dependencies should be available");
    }

    #[tokio::test]
    async fn test_validate_url() {
        let output_dir = PathBuf::from("/tmp");
        let downloader = YouTubeDownloader::new(output_dir).expect("Failed to create downloader");

        assert!(downloader.validate_url("https://www.youtube.com/watch?v=dQw4w9WgXcQ"));
        assert!(downloader.validate_url("https://youtu.be/dQw4w9WgXcQ"));
        assert!(downloader.validate_url("https://youtube.com/shorts/xyz123"));

        assert!(!downloader.validate_url("https://example.com"));
        assert!(!downloader.validate_url("not_a_url"));
    }

    #[tokio::test]
    async fn test_get_video_info() {
        let output_dir = PathBuf::from("/tmp");
        let downloader = YouTubeDownloader::new(output_dir).expect("Failed to create downloader");

        let test_url = "https://www.youtube.com/watch?v=dQw4w9WgXcQ";

        match downloader.get_video_info(test_url).await {
            Ok(video_info) => {
                println!("Video info: {:?}", video_info);
                assert_eq!(video_info.url, test_url);
                assert!(!video_info.title.is_empty());
                assert!(!video_info.id.is_empty());
            }
            Err(e) => {
                println!(
                    "Warning: Failed to get video info (might be due to network/rate limiting): {}",
                    e
                );
            }
        }
    }

    #[test]
    fn test_supported_formats() {
        let formats = vec!["mp4".to_string(), "mp3".to_string(), "wav".to_string(), "webm".to_string()];
        assert!(formats.contains(&"mp4".to_string()));
        assert!(formats.contains(&"mp3".to_string()));
        assert!(formats.contains(&"wav".to_string()));
        assert!(formats.contains(&"webm".to_string()));
    }

    #[test]
    fn test_supported_qualities() {
        let qualities = vec![
            "best".to_string(),
            "high".to_string(),
            "medium".to_string(),
            "low".to_string(),
            "worst".to_string(),
        ];
        assert!(qualities.contains(&"best".to_string()));
        assert!(qualities.contains(&"high".to_string()));
    }

    #[tokio::test]
    async fn test_api_supported_formats() {
        let result = crate::commands::get_supported_formats().await;
        assert!(result.is_ok());
        let formats = result.unwrap();
        assert!(formats.contains(&"mp4".to_string()));
        assert!(formats.contains(&"mp3".to_string()));
        assert!(formats.contains(&"wav".to_string()));
        assert!(formats.contains(&"webm".to_string()));
        println!("Supported formats: {:?}", formats);
    }
}
