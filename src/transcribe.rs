// Voxtus - Transcribe YouTube videos and local media files to text
// Copyright (C) 2024 Johan Thorén <johan@thoren.xyz>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Transcription functionality using Whisper.

#[cfg(feature = "whisper")]
use std::ffi::CStr;
#[cfg(feature = "whisper")]
use std::os::raw::{c_char, c_void};
use std::path::{Path, PathBuf};

#[cfg(feature = "whisper")]
use std::fs;
#[cfg(feature = "whisper")]
use std::io::Write;

use crate::error::{Error, Result};
use crate::formats::{Metadata, Segment, Transcript};

#[cfg(feature = "whisper")]
unsafe extern "C" fn log_callback(level: u32, message: *const c_char, _user_data: *mut c_void) {
    if message.is_null() {
        return;
    }
    // SAFETY: We trust whisper.cpp to pass valid C strings
    let msg = unsafe { CStr::from_ptr(message).to_string_lossy() };
    let msg = msg.trim_end();

    // Map ggml_log_level to log::Level
    // 2=ERROR, 3=WARN, 4=INFO, 5=DEBUG
    // Whisper seems to use ERROR (2) for initialization info, so we map it to Debug
    // to hide it by default but show it with -v.
    let log_level = match level {
        2 => log::Level::Debug,
        3 => log::Level::Debug,
        4 => log::Level::Trace,
        _ => log::Level::Trace,
    };

    log::log!(target: "whisper", log_level, "{}", msg);
}

/// Directory where Whisper models are stored.
#[cfg(feature = "whisper")]
fn get_models_dir() -> Result<PathBuf> {
    let dir = dirs::data_local_dir()
        .ok_or_else(|| Error::DownloadFailed("Could not determine local data directory".into()))?
        .join("voxtus")
        .join("models");

    if !dir.exists() {
        std::fs::create_dir_all(&dir)?;
    }

    Ok(dir)
}

/// Get the URL for a specific model.
#[cfg(feature = "whisper")]
fn get_model_url(model: &str) -> String {
    let model_name = if model == "large" { "large-v3" } else { model };
    format!(
        "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-{}.bin",
        model_name
    )
}

/// Download the model if it doesn't exist.
#[cfg(feature = "whisper")]
async fn ensure_model(model: &str) -> Result<PathBuf> {
    let models_dir = get_models_dir()?;
    let model_name = if model == "large" { "large-v3" } else { model };
    let model_path = models_dir.join(format!("ggml-{}.bin", model_name));

    if model_path.exists() {
        return Ok(model_path);
    }

    let url = get_model_url(model);
    log::info!("Downloading model '{}'...", model);

    let response = reqwest::get(&url)
        .await
        .map_err(|e| Error::DownloadFailed(format!("Failed to download model: {}", e)))?;

    if !response.status().is_success() {
        return Err(Error::DownloadFailed(format!(
            "Failed to download model: HTTP {}",
            response.status()
        )));
    }

    let content = response
        .bytes()
        .await
        .map_err(|e| Error::DownloadFailed(format!("Failed to read model bytes: {}", e)))?;

    let mut file = fs::File::create(&model_path)?;
    file.write_all(&content)?;

    log::info!("Model saved: {}", model_path.display());

    Ok(model_path)
}

/// Transcribe audio file using Whisper.
///
/// Downloads the model if not already cached and returns a transcript
/// with segments and metadata.
pub fn transcribe(
    audio_path: &Path,
    temp_dir: &Path,
    title: &str,
    source: &str,
    model: &str,
) -> Result<Transcript> {
    #[cfg(feature = "whisper")]
    {
        // Set log callback
        unsafe {
            whisper_rs::set_log_callback(Some(log_callback), std::ptr::null_mut());
        }

        // 1. Ensure model exists (download if needed)
        let rt = tokio::runtime::Runtime::new()
            .map_err(|e| Error::TranscriptionFailed(format!("Failed to create runtime: {}", e)))?;

        let model_path = rt.block_on(ensure_model(model))?;

        // 2. Run Whisper (converts audio to PCM internally)
        run_whisper(audio_path, temp_dir, &model_path, title, source, model)
    }

    #[cfg(not(feature = "whisper"))]
    {
        // Avoid unused variable warnings
        let _ = (audio_path, temp_dir);

        // Return a placeholder transcript for testing without whisper
        log::warn!("Whisper feature not enabled. Using placeholder transcript.");
        let segments = vec![Segment::new(
            0.0,
            1.0,
            "Whisper transcription requires the 'whisper' feature.",
        )];
        let metadata = Metadata::new(title, source, Some(1.0), model, Some("en".to_string()));
        Ok(Transcript::new(segments, metadata))
    }
}

#[cfg(feature = "whisper")]
fn run_whisper(
    audio_path: &Path,
    temp_dir: &Path,
    model_path: &Path,
    title: &str,
    source: &str,
    model_name: &str,
) -> Result<Transcript> {
    use whisper_rs::{FullParams, SamplingStrategy, WhisperContext, WhisperContextParameters};

    // Load model
    let ctx = WhisperContext::new_with_params(
        model_path.to_str().unwrap(),
        WhisperContextParameters::default(),
    )
    .map_err(|e| Error::TranscriptionFailed(format!("Failed to load model: {}", e)))?;

    // Create state
    let mut state = ctx
        .create_state()
        .map_err(|e| Error::TranscriptionFailed(format!("Failed to create state: {}", e)))?;

    // Convert audio directly to raw f32le PCM for Whisper (16kHz mono)
    let pcm_path = temp_dir.join("whisper_input.pcm");
    let output = std::process::Command::new("ffmpeg")
        .args([
            "-i",
            &audio_path.to_string_lossy(),
            "-f",
            "f32le",
            "-acodec",
            "pcm_f32le",
            "-ac",
            "1",
            "-ar",
            "16000",
            "-y",
            &pcm_path.to_string_lossy(),
        ])
        .output()
        .map_err(|e| Error::FfmpegError(e.to_string()))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(Error::FfmpegError(format!(
            "Failed to convert audio to PCM: {}",
            stderr.lines().last().unwrap_or("unknown error")
        )));
    }

    let audio_bytes = fs::read(&pcm_path)?;
    let audio_len = audio_bytes.len() / 4;
    let mut audio_data = Vec::with_capacity(audio_len);

    for chunk in audio_bytes.chunks_exact(4) {
        let val = f32::from_le_bytes(chunk.try_into().unwrap());
        audio_data.push(val);
    }

    // Run transcription
    let mut params = FullParams::new(SamplingStrategy::Greedy { best_of: 1 });
    params.set_print_special(false);
    params.set_print_progress(false);
    params.set_print_realtime(false);
    params.set_print_timestamps(false);

    state
        .full(params, &audio_data[..])
        .map_err(|e| Error::TranscriptionFailed(format!("Failed to run whisper: {}", e)))?;

    // Collect segments
    let num_segments = state.full_n_segments();

    let mut segments = Vec::new();
    for i in 0..num_segments {
        let segment = state
            .get_segment(i)
            .ok_or_else(|| Error::TranscriptionFailed(format!("Failed to get segment {}", i)))?;

        let text = segment.to_str().map_err(|e| {
            Error::TranscriptionFailed(format!("Failed to get segment text: {}", e))
        })?;

        // Whisper returns time in centiseconds (10ms units)
        let start_sec = segment.start_timestamp() as f64 / 100.0;
        let end_sec = segment.end_timestamp() as f64 / 100.0;

        segments.push(Segment::new(start_sec, end_sec, text));
    }

    // Get detected language from whisper
    let lang_id = state.full_lang_id_from_state();
    let language = whisper_rs::get_lang_str(lang_id).map(|s| s.to_string());

    let metadata = Metadata::new(
        title,
        source,
        Some(segments.last().map(|s| s.end).unwrap_or(0.0)),
        model_name,
        language,
    );

    Ok(Transcript::new(segments, metadata))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[cfg(feature = "whisper")]
    fn test_get_model_url() {
        assert_eq!(
            get_model_url("tiny"),
            "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-tiny.bin"
        );
        assert_eq!(
            get_model_url("large"),
            "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-large-v3.bin"
        );
    }

    #[test]
    #[cfg(feature = "whisper")]
    fn test_get_models_dir() {
        let dir = get_models_dir().unwrap();
        assert!(dir.ends_with("voxtus/models") || dir.ends_with("voxtus\\models"));
    }
}
