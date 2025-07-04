import { invoke } from '@tauri-apps/api/core';

// Types matching Rust structs
export interface VideoInfo {
  id: string;
  title: string;
  url: string;
  duration?: number;
  thumbnail?: string;
  uploader?: string;
  upload_date?: string;
  view_count?: number;
  available_formats: string[];
}

export interface DownloadRequest {
  url: string;
  format: string;      // mp4, mp3, wav, webm
  quality: string;     // best, high, medium, low, worst
  output_path: string;
  filename?: string;
}

export interface DownloadProgress {
  id: string;
  status: DownloadStatus;
  progress: number;
  speed?: string;
  eta?: string;
  filename?: string;
  error?: string;
}

export enum DownloadStatus {
  Pending = 'Pending',
  Downloading = 'Downloading',
  Completed = 'Completed',
  Failed = 'Failed',
  Cancelled = 'Cancelled',
}

export interface DownloadHistory {
  id: string;
  title: string;
  url: string;
  status: DownloadStatus;
  downloaded_at: string; // ISO date string
  file_path?: string;
  format: string;
  quality: string;
}

export interface DownloadStats {
  total: number;
  completed: number;
  failed: number;
  downloading: number;
  most_used_format?: string;
  formats_breakdown: Record<string, number>;
}

export interface AppSettings {
  download_path: string;
  default_format: string;
  default_quality: string;
}

// API functions
export class TauriYouTubeAPI {
  
  static async validateUrl(url: string): Promise<boolean> {
    return invoke<boolean>('validate_youtube_url', { url });
  }

  static async getVideoInfo(url: string): Promise<VideoInfo> {
    return invoke<VideoInfo>('get_video_info', { url });
  }

  static async downloadVideo(request: DownloadRequest): Promise<string> {
    return invoke<string>('download_video', { request });
  }

  static async getDownloadHistory(): Promise<DownloadHistory[]> {
    return invoke<DownloadHistory[]>('get_download_history');
  }

  static async getDownloadStatus(downloadId: string): Promise<DownloadHistory | null> {
    return invoke<DownloadHistory | null>('get_download_status', { downloadId });
  }

  static async clearDownloadHistory(): Promise<void> {
    return invoke<void>('clear_download_history');
  }

  static async getDefaultDownloadPath(): Promise<string> {
    return invoke<string>('get_default_download_path');
  }

  static async checkYtDlpInstalled(): Promise<boolean> {
    return invoke<boolean>('check_yt_dlp_installed');
  }

  static async getSupportedFormats(): Promise<string[]> {
    return invoke<string[]>('get_supported_formats');
  }

  static async getSupportedQualities(): Promise<string[]> {
    return invoke<string[]>('get_supported_qualities');
  }

  static async checkDependencies(): Promise<boolean> {
    return invoke<boolean>('check_dependencies');
  }

  static async getDownloadStatistics(): Promise<DownloadStats> {
    return invoke<DownloadStats>('get_download_statistics');
  }

  static async saveSettings(settings: AppSettings): Promise<void> {
    return invoke<void>('save_settings', { settings });
  }

  static async loadSettings(): Promise<AppSettings> {
    return invoke<AppSettings>('load_settings');
  }
}

// Utility functions
export class YouTubeUtils {
  
  static extractVideoId(url: string): string | null {
    const patterns = [
      /(?:youtube\.com\/watch\?v=|youtu\.be\/|youtube\.com\/embed\/)([^&\n?#]+)/,
      /youtube\.com\/shorts\/([^&\n?#]+)/
    ];
    
    for (const pattern of patterns) {
      const match = url.match(pattern);
      if (match && match[1]) {
        return match[1];
      }
    }
    
    return null;
  }

  static getThumbnailUrl(videoId: string, quality: 'default' | 'medium' | 'high' | 'standard' | 'maxres' = 'medium'): string {
    const qualityMap = {
      'default': 'default',
      'medium': 'mqdefault',
      'high': 'hqdefault',
      'standard': 'sddefault',
      'maxres': 'maxresdefault'
    };
    
    return `https://img.youtube.com/vi/${videoId}/${qualityMap[quality]}.jpg`;
  }

  static formatDuration(seconds: number): string {
    const hours = Math.floor(seconds / 3600);
    const minutes = Math.floor((seconds % 3600) / 60);
    const remainingSeconds = seconds % 60;

    if (hours > 0) {
      return `${hours}:${minutes.toString().padStart(2, '0')}:${remainingSeconds.toString().padStart(2, '0')}`;
    } else {
      return `${minutes}:${remainingSeconds.toString().padStart(2, '0')}`;
    }
  }

  static formatFileSize(bytes: number): string {
    const units = ['B', 'KB', 'MB', 'GB'];
    let size = bytes;
    let unitIndex = 0;

    while (size >= 1024 && unitIndex < units.length - 1) {
      size /= 1024;
      unitIndex++;
    }

    return `${size.toFixed(1)} ${units[unitIndex]}`;
  }

  static formatDate(dateString: string): string {
    return new Date(dateString).toLocaleDateString('pl-PL', {
      year: 'numeric',
      month: 'long',
      day: 'numeric',
      hour: '2-digit',
      minute: '2-digit'
    });
  }
} 