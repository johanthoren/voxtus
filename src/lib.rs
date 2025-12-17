// Voxtus - Transcribe YouTube videos and local media files to text
// Copyright (C) 2024 Johan Thorén <johan@thoren.xyz>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Voxtus: Transcribe YouTube videos and local media files to text.
//!
//! This library provides the core functionality for the voxtus CLI tool.
//! It supports downloading media from YouTube, processing local files,
//! and transcribing audio using Whisper.

pub mod audio;
pub mod cli;
pub mod config;
pub mod download;
pub mod error;
pub mod formats;
pub mod logging;
pub mod signals;
pub mod transcribe;

// Re-export commonly used types
pub use cli::Args;
pub use config::Config;
pub use error::{Error, Result};
