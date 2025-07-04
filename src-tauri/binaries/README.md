# External Binaries

This directory contains external binaries required for the Snapper to function.

## Required Binaries

### For macOS (Intel):
- `yt-dlp-x86_64-apple-darwin` 
- `ffmpeg-x86_64-apple-darwin`

### For macOS (ARM/M1/M2):
- `yt-dlp-aarch64-apple-darwin`
- `ffmpeg-aarch64-apple-darwin`

### For Windows (64-bit):
- `yt-dlp-x86_64-pc-windows-msvc.exe`
- `ffmpeg-x86_64-pc-windows-msvc.exe`

### For Linux (64-bit):
- `yt-dlp-x86_64-unknown-linux-gnu`
- `ffmpeg-x86_64-unknown-linux-gnu`

## Download Instructions

### yt-dlp
Download from: https://github.com/yt-dlp/yt-dlp/releases/latest

### ffmpeg
Download from: https://ffmpeg.org/download.html

For Windows, use builds from: https://www.gyan.dev/ffmpeg/builds/
For macOS, you can also use Homebrew builds
For Linux, use static builds from: https://johnvansickle.com/ffmpeg/

## File Naming Convention

The binaries must be named with the target triple suffix:
- `{binary-name}-{target-triple}[.exe]`

Example:
- `yt-dlp-x86_64-apple-darwin` (macOS Intel)
- `ffmpeg-aarch64-apple-darwin` (macOS ARM)
- `yt-dlp-x86_64-pc-windows-msvc.exe` (Windows)

## Current Platform

To find your current platform's target triple, run:
```bash
rustc -Vv | grep host | cut -f2 -d' '
```

## Alternative Approach

Instead of platform-specific names, you can also place binaries with simple names:
- `yt-dlp` or `yt-dlp.exe`
- `ffmpeg` or `ffmpeg.exe`

The application will automatically detect and use these. 