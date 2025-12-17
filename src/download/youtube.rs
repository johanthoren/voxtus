// Voxtus - Transcribe YouTube videos and local media files to text
// Copyright (C) 2024 Johan Thorén <johan@thoren.xyz>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! YouTube download functionality using yt-dlp.

use std::path::{Path, PathBuf};

use crate::error::{Error, Result};

/// Video metadata from yt-dlp.
#[derive(Debug, Clone)]
pub struct VideoInfo {
    pub title: String,
}

/// Directory where yt-dlp and ffmpeg binaries are stored.
#[cfg(feature = "youtube")]
fn get_libs_dir() -> Result<PathBuf> {
    let dir = dirs::data_local_dir()
        .ok_or_else(|| Error::DownloadFailed("Could not determine local data directory".into()))?
        .join("voxtus")
        .join("libs");

    if !dir.exists() {
        std::fs::create_dir_all(&dir)?;
    }

    Ok(dir)
}

/// Download audio from URL. Returns m4a path and video info.
#[cfg(feature = "youtube")]
pub async fn download_audio(url: &str, output_dir: &Path) -> Result<(PathBuf, VideoInfo)> {
    use yt_dlp::Youtube;

    let libs_dir = get_libs_dir()?;
    let yt_dlp_path = libs_dir.join(if cfg!(windows) {
        "yt-dlp.exe"
    } else {
        "yt-dlp"
    });
    let ffmpeg_path = libs_dir.join(if cfg!(windows) {
        "ffmpeg.exe"
    } else {
        "ffmpeg"
    });

    // Initialize YouTube client, downloading binaries if needed
    let youtube: Youtube = if yt_dlp_path.exists() && ffmpeg_path.exists() {
        let libs = yt_dlp::client::deps::Libraries::new(yt_dlp_path, ffmpeg_path);
        Youtube::new(libs, output_dir.to_path_buf())
            .await
            .map_err(|e| Error::DownloadFailed(e.to_string()))?
    } else {
        Youtube::with_new_binaries(libs_dir, output_dir.to_path_buf())
            .await
            .map_err(|e| Error::DownloadFailed(e.to_string()))?
    };

    // Fetch video info
    let video = youtube
        .fetch_video_infos(url.to_string())
        .await
        .map_err(|e| Error::DownloadFailed(format!("Failed to fetch video info: {}", e)))?;

    let info = VideoInfo {
        title: video.title.clone(),
    };

    // Download audio stream as m4a (native YouTube format)
    // We'll convert to mp3 later using our own ffmpeg
    let audio_path = youtube
        .download_audio_stream_from_url(url.to_string(), "audio.m4a")
        .await
        .map_err(|e| Error::DownloadFailed(format!("Failed to download audio: {}", e)))?;

    Ok((audio_path, info))
}

#[cfg(not(feature = "youtube"))]
pub async fn download_audio(_url: &str, _output_dir: &Path) -> Result<(PathBuf, VideoInfo)> {
    Err(Error::DownloadFailed(
        "YouTube download requires the 'youtube' feature".into(),
    ))
}

/// Synchronous wrapper for download_audio.
#[cfg(feature = "youtube")]
pub fn download_audio_sync(url: &str, output_dir: &Path) -> Result<(PathBuf, VideoInfo)> {
    let rt = tokio::runtime::Runtime::new()
        .map_err(|e| Error::DownloadFailed(format!("Failed to create runtime: {}", e)))?;

    rt.block_on(download_audio(url, output_dir))
}

#[cfg(not(feature = "youtube"))]
pub fn download_audio_sync(_url: &str, _output_dir: &Path) -> Result<(PathBuf, VideoInfo)> {
    Err(Error::DownloadFailed(
        "YouTube download requires the 'youtube' feature".into(),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[cfg(feature = "youtube")]
    fn test_get_libs_dir() {
        let dir = get_libs_dir().unwrap();
        assert!(dir.ends_with("voxtus/libs") || dir.ends_with("voxtus\\libs"));
    }
}
