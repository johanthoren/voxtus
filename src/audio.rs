// Voxtus - Transcribe YouTube videos and local media files to text
// Copyright (C) 2024 Johan Thorén <johan@thoren.xyz>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Audio extraction and conversion via ffmpeg.

use std::path::Path;
use std::process::Command;

use crate::error::{Error, Result};

/// Check if ffmpeg is available in PATH.
pub fn check_ffmpeg() -> Result<()> {
    Command::new("ffmpeg")
        .arg("-version")
        .output()
        .map_err(|_| Error::FfmpegNotFound)?;
    Ok(())
}

/// Build ffmpeg arguments for MP3 conversion.
pub fn ffmpeg_convert_args(input: &Path, output: &Path) -> Vec<String> {
    vec![
        "-i".to_string(),
        input.to_string_lossy().to_string(),
        "-vn".to_string(), // No video
        "-acodec".to_string(),
        "mp3".to_string(),
        "-q:a".to_string(),
        "2".to_string(),  // High quality
        "-y".to_string(), // Overwrite output
        output.to_string_lossy().to_string(),
    ]
}

/// Convert a media file to MP3 using ffmpeg.
pub fn convert_to_mp3(input: &Path, output: &Path) -> Result<()> {
    let args = ffmpeg_convert_args(input, output);

    let result = Command::new("ffmpeg")
        .args(&args)
        .output()
        .map_err(|e| Error::FfmpegError(e.to_string()))?;

    if !result.status.success() {
        let stderr = String::from_utf8_lossy(&result.stderr);
        return Err(Error::FfmpegError(format!(
            "ffmpeg exited with status {}: {}",
            result.status,
            stderr.lines().last().unwrap_or("unknown error")
        )));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_ffmpeg_convert_args() {
        let input = PathBuf::from("/tmp/input.mp4");
        let output = PathBuf::from("/tmp/output.mp3");

        let args = ffmpeg_convert_args(&input, &output);

        assert_eq!(args[0], "-i");
        assert_eq!(args[1], "/tmp/input.mp4");
        assert_eq!(args[2], "-vn");
        assert_eq!(args[3], "-acodec");
        assert_eq!(args[4], "mp3");
        assert!(args.contains(&"-y".to_string()));
        assert_eq!(args.last().unwrap(), "/tmp/output.mp3");
    }
}
