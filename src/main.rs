// Voxtus - Transcribe YouTube videos and local media files to text
// Copyright (C) 2024 Johan Thorén <johan@thoren.xyz>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Voxtus: Transcribe YouTube videos and local media files to text.

use std::path::{Path, PathBuf};

use voxtus::audio::{check_ffmpeg, convert_to_mp3};
use voxtus::cli::Args;
use voxtus::config::{AVAILABLE_MODELS, Config, OutputFormat, is_url};
use voxtus::download::download_audio_sync;
use voxtus::formats::Transcript;
use voxtus::logging::setup_logger;
use voxtus::signals::{setup_signal_handlers, shutdown_requested};
use voxtus::transcribe::transcribe;

fn main() {
    // Set up signal handlers
    if let Err(e) = setup_signal_handlers() {
        eprintln!("Warning: Failed to set up signal handlers: {}", e);
    }

    let exit_code = run();
    std::process::exit(exit_code);
}

fn run() -> i32 {
    let args = Args::parse_args();

    // Handle --list-models
    if args.list_models {
        print_available_models();
        return 0;
    }

    // Create config from args
    let config = match Config::from_args(&args) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Error: {}", e);
            return 1;
        }
    };

    // Initialize logger
    if let Err(e) = setup_logger(config.verbose_level) {
        eprintln!("Error initializing logger: {}", e);
        return 1;
    }

    // Check ffmpeg is available
    if let Err(e) = check_ffmpeg() {
        log::error!("{}", e);
        log::error!("  - macOS: brew install ffmpeg");
        log::error!("  - Ubuntu/Debian: sudo apt install ffmpeg");
        log::error!("  - Windows: Download from https://ffmpeg.org/download.html");
        return 1;
    }

    // Run the main workflow
    match process(&config) {
        Ok(()) => 0,
        Err(e) => {
            log::error!("{}", e);
            1
        }
    }
}

/// Main processing workflow.
fn process(config: &Config) -> voxtus::Result<()> {
    // Create temp directory for intermediate files (auto-cleaned on drop)
    let temp_dir = tempfile::tempdir()?;

    // Determine input type and get audio file
    let (audio_path, title) = if is_url(&config.input_path) {
        download_and_convert(config, temp_dir.path())?
    } else {
        convert_local_file(config, temp_dir.path())?
    };

    // Check for shutdown
    if shutdown_requested() {
        log::info!("Interrupted, exiting.");
        return Ok(());
    }

    // Transcribe
    let transcript = transcribe(
        &audio_path,
        temp_dir.path(),
        &title,
        &config.input_path,
        &config.model,
    )?;

    // Check for shutdown
    if shutdown_requested() {
        log::info!("Interrupted, exiting.");
        return Ok(());
    }

    // Output results
    output_transcript(&transcript, config)?;

    // Keep audio if requested
    if config.keep_audio {
        let final_audio = config
            .output_dir
            .join(format!("{}.mp3", get_output_name(&title, config)));
        std::fs::copy(&audio_path, &final_audio)?;
        if !config.stdout_mode {
            log::info!("Audio saved: {}", final_audio.display());
        }
    }

    Ok(())
}

/// Download from URL and convert to MP3.
fn download_and_convert(config: &Config, temp_dir: &Path) -> voxtus::Result<(PathBuf, String)> {
    if !config.stdout_mode {
        log::info!("Downloading: {}", config.input_path);
    }

    // Download audio using yt-dlp (returns m4a format)
    let (downloaded_path, info) = download_audio_sync(&config.input_path, temp_dir)?;

    if !config.stdout_mode {
        log::info!("Downloaded: {}", info.title);
    }

    // Convert to MP3 using our ffmpeg wrapper
    let mp3_path = temp_dir.join("audio.mp3");
    convert_to_mp3(&downloaded_path, &mp3_path)?;

    Ok((mp3_path, info.title))
}

/// Convert a local file to MP3.
fn convert_local_file(config: &Config, temp_dir: &Path) -> voxtus::Result<(PathBuf, String)> {
    let input_path = Path::new(&config.input_path);

    if !input_path.exists() {
        return Err(voxtus::Error::FileNotFound(config.input_path.clone()));
    }

    let title = input_path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("audio")
        .to_string();

    if !config.stdout_mode {
        log::info!("Converting: {}", input_path.display());
    }

    // If already MP3, just copy
    let audio_path = if input_path.extension().is_some_and(|e| e == "mp3") {
        let dest = temp_dir.join("audio.mp3");
        std::fs::copy(input_path, &dest)?;
        dest
    } else {
        let output_path = temp_dir.join("audio.mp3");
        convert_to_mp3(input_path, &output_path)?;
        output_path
    };

    Ok((audio_path, title))
}

/// Output transcript in requested formats.
fn output_transcript(transcript: &Transcript, config: &Config) -> voxtus::Result<()> {
    let output_name = get_output_name(&transcript.metadata.title, config);

    for format in &config.formats {
        let content = match format {
            OutputFormat::Txt => transcript.to_txt(),
            OutputFormat::Json => transcript.to_json(),
            OutputFormat::Srt => transcript.to_srt(),
            OutputFormat::Vtt => transcript.to_vtt(),
        };

        if config.stdout_mode {
            println!("{}", content);
        } else {
            let output_path =
                config
                    .output_dir
                    .join(format!("{}.{}", output_name, format.extension()));

            // Check for overwrite
            if output_path.exists() && !config.overwrite_files {
                eprint!("File '{}' exists. Overwrite? [y/N] ", output_path.display());
                let mut response = String::new();
                if std::io::stdin().read_line(&mut response).is_err()
                    || !response.trim().eq_ignore_ascii_case("y")
                {
                    return Err(voxtus::Error::UserAborted);
                }
            }

            std::fs::write(&output_path, content)?;
            log::info!("Saved: {}", output_path.display());
        }
    }

    Ok(())
}

/// Get the output filename (without extension).
fn get_output_name(title: &str, config: &Config) -> String {
    config
        .custom_name
        .clone()
        .unwrap_or_else(|| title.to_string())
}

fn print_available_models() {
    println!("Available Whisper Models:\n");

    let groups = [
        ("Tiny Models", &["tiny", "tiny.en"][..]),
        ("Base Models", &["base", "base.en"][..]),
        ("Small Models", &["small", "small.en"][..]),
        ("Medium Models", &["medium", "medium.en"][..]),
        ("Large Models", &["large", "large-v2", "large-v3"][..]),
    ];

    for (group_name, model_names) in groups {
        println!("{}:", group_name);
        for name in model_names.iter() {
            if let Some(model) = AVAILABLE_MODELS.iter().find(|m| m.name == *name) {
                println!("   {:<18} - {}", model.name, model.description);
                println!(
                    "                      {} params, {} VRAM, {}",
                    model.params, model.vram, model.languages
                );
            }
        }
        println!();
    }

    println!("Examples:");
    println!("   voxtus --model tiny video.mp4            # Fastest transcription");
    println!("   voxtus --model small video.mp4           # Good balance (default)");
    println!("   voxtus --model large-v3 video.mp4        # Best accuracy");
    println!("   voxtus --model small.en audio.mp3        # English-only, faster");
}
