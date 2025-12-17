// Voxtus - Transcribe YouTube videos and local media files to text
// Copyright (C) 2024 Johan Thorén <johan@thoren.xyz>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Error types for voxtus.

use thiserror::Error;

/// Result type alias using voxtus Error.
pub type Result<T> = std::result::Result<T, Error>;

/// All possible errors in voxtus.
#[derive(Error, Debug)]
pub enum Error {
    #[error("Invalid format: {0}")]
    InvalidFormat(String),

    #[error("Multiple formats not allowed with --stdout")]
    MultipleFormatsWithStdout,

    #[error("File not found: {0}")]
    FileNotFound(String),

    #[error("Invalid URL: {0}")]
    InvalidUrl(String),

    #[error("Download failed: {0}")]
    DownloadFailed(String),

    #[error("Transcription failed: {0}")]
    TranscriptionFailed(String),

    #[error("FFmpeg error: {0}")]
    FfmpegError(String),

    #[error("FFmpeg not found. Please install ffmpeg.")]
    FfmpegNotFound,

    #[error("Invalid model: {0}")]
    InvalidModel(String),

    #[error("User aborted")]
    UserAborted,

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
}
