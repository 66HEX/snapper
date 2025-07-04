#!/bin/bash

# Script to download yt-dlp and ffmpeg binaries for the current platform

set -e

# Get target triple
TARGET_TRIPLE=$(rustc -Vv | grep host | cut -f2 -d' ')
echo "Target triple: $TARGET_TRIPLE"

# Create binaries directory
BINARIES_DIR="$(dirname "$0")/../binaries"
mkdir -p "$BINARIES_DIR"

# Platform detection
if [[ "$TARGET_TRIPLE" == *"windows"* ]]; then
    PLATFORM="windows"
    EXT=".exe"
elif [[ "$TARGET_TRIPLE" == *"apple"* ]]; then
    PLATFORM="macos"
    EXT=""
else
    PLATFORM="linux"
    EXT=""
fi

echo "Detected platform: $PLATFORM"

# Download yt-dlp
echo "Downloading yt-dlp..."
YT_DLP_URL=""
YT_DLP_FILENAME="yt-dlp${EXT}"

if [[ "$PLATFORM" == "windows" ]]; then
    YT_DLP_URL="https://github.com/yt-dlp/yt-dlp/releases/latest/download/yt-dlp.exe"
elif [[ "$PLATFORM" == "macos" ]]; then
    YT_DLP_URL="https://github.com/yt-dlp/yt-dlp/releases/latest/download/yt-dlp_macos"
else
    YT_DLP_URL="https://github.com/yt-dlp/yt-dlp/releases/latest/download/yt-dlp"
fi

curl -L -o "$BINARIES_DIR/$YT_DLP_FILENAME" "$YT_DLP_URL"
chmod +x "$BINARIES_DIR/$YT_DLP_FILENAME"

echo "Downloaded yt-dlp to: $BINARIES_DIR/$YT_DLP_FILENAME"

# Download ffmpeg (platform-specific instructions)
echo "For ffmpeg, please download manually from:"

if [[ "$PLATFORM" == "windows" ]]; then
    echo "Windows: https://www.gyan.dev/ffmpeg/builds/"
    echo "Download 'release builds' -> 'ffmpeg-release-essentials.zip'"
    echo "Extract ffmpeg.exe to: $BINARIES_DIR/ffmpeg.exe"
elif [[ "$PLATFORM" == "macos" ]]; then
    echo "macOS: Use Homebrew or download from https://evermeet.cx/ffmpeg/"
    echo "With Homebrew: brew install ffmpeg"
    echo "Then copy: $(which ffmpeg) to $BINARIES_DIR/ffmpeg"
else
    echo "Linux: https://johnvansickle.com/ffmpeg/"
    echo "Download 'release builds' -> extract ffmpeg binary"
    echo "Copy ffmpeg binary to: $BINARIES_DIR/ffmpeg"
fi

echo "Done! yt-dlp downloaded. Please download ffmpeg manually as instructed above." 