// Voxtus - Transcribe YouTube videos and local media files to text
// Copyright (C) 2024 Johan Thorén <johan@thoren.xyz>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Logging configuration.

use chrono::Utc;
use log::LevelFilter;

pub fn setup_logger(verbosity: u8) -> Result<(), fern::InitError> {
    let dispatch = fern::Dispatch::new()
        .format(move |out, message, record| {
            match verbosity {
                0 => {
                    // Just the message
                    out.finish(format_args!("{}", message))
                }
                1 => {
                    // Timestamp, target, level
                    out.finish(format_args!(
                        "{} [{}] [{}] {}",
                        Utc::now().format("%Y-%m-%d %H:%M:%S%.3f"),
                        record.target(),
                        record.level(),
                        message
                    ))
                }
                _ => {
                    // Full format with file:line
                    out.finish(format_args!(
                        "{} [{}] [{}] [{}:{}] {}",
                        Utc::now().format("%Y-%m-%d %H:%M:%S%.3f"),
                        record.target(),
                        record.level(),
                        record.file().unwrap_or("unknown"),
                        record.line().unwrap_or(0),
                        message
                    ))
                }
            }
        })
        .level(if verbosity >= 2 {
            LevelFilter::Debug
        } else {
            LevelFilter::Info
        })
        .chain(std::io::stderr());

    dispatch.apply()?;
    Ok(())
}
