# PowerShell script to download yt-dlp and ffmpeg binaries for Windows

param(
    [switch]$Force
)

# Get target triple
$rustInfo = rustc -Vv
$targetTriple = ($rustInfo | Select-String "host:").Line.Split(" ")[1]
Write-Host "Target triple: $targetTriple"

# Create binaries directory
$scriptDir = Split-Path -Parent $MyInvocation.MyCommand.Definition
$binariesDir = Join-Path $scriptDir "..\binaries"
if (!(Test-Path $binariesDir)) {
    New-Item -ItemType Directory -Path $binariesDir
}

Write-Host "Binaries directory: $binariesDir"

# Download yt-dlp
$ytDlpPath = Join-Path $binariesDir "yt-dlp.exe"
if ($Force -or !(Test-Path $ytDlpPath)) {
    Write-Host "Downloading yt-dlp..."
    $ytDlpUrl = "https://github.com/yt-dlp/yt-dlp/releases/latest/download/yt-dlp.exe"
    try {
        Invoke-WebRequest -Uri $ytDlpUrl -OutFile $ytDlpPath
        Write-Host "Downloaded yt-dlp to: $ytDlpPath"
    } catch {
        Write-Error "Failed to download yt-dlp: $_"
    }
} else {
    Write-Host "yt-dlp already exists at: $ytDlpPath"
}

# Instructions for ffmpeg
Write-Host ""
Write-Host "For ffmpeg, please download manually:"
Write-Host "1. Go to: https://www.gyan.dev/ffmpeg/builds/"
Write-Host "2. Download 'release builds' -> 'ffmpeg-release-essentials.zip'"
Write-Host "3. Extract the zip file"
Write-Host "4. Copy ffmpeg.exe from the 'bin' folder to: $binariesDir\ffmpeg.exe"
Write-Host ""

# Check if ffmpeg already exists
$ffmpegPath = Join-Path $binariesDir "ffmpeg.exe"
if (Test-Path $ffmpegPath) {
    Write-Host "ffmpeg already exists at: $ffmpegPath" -ForegroundColor Green
} else {
    Write-Host "ffmpeg not found. Please download it manually as instructed above." -ForegroundColor Yellow
}

Write-Host "Done!" 