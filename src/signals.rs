// Voxtus - Transcribe YouTube videos and local media files to text
// Copyright (C) 2024 Johan Thorén <johan@thoren.xyz>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Signal handling for graceful shutdown.

use std::sync::atomic::{AtomicBool, Ordering};

/// Global flag indicating if a shutdown signal was received.
static SHUTDOWN_REQUESTED: AtomicBool = AtomicBool::new(false);

/// Check if shutdown has been requested.
pub fn shutdown_requested() -> bool {
    SHUTDOWN_REQUESTED.load(Ordering::SeqCst)
}

/// Request shutdown (called by signal handler).
pub fn request_shutdown() {
    SHUTDOWN_REQUESTED.store(true, Ordering::SeqCst);
}

/// Reset shutdown flag (useful for testing).
#[cfg(test)]
pub fn reset_shutdown() {
    SHUTDOWN_REQUESTED.store(false, Ordering::SeqCst);
}

/// Set up signal handlers for SIGINT and SIGTERM.
///
/// Returns a Result indicating if handlers were successfully installed.
pub fn setup_signal_handlers() -> Result<(), ctrlc::Error> {
    ctrlc::set_handler(move || {
        eprintln!("\nInterrupted, cleaning up...");
        request_shutdown();
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_shutdown_flag() {
        reset_shutdown();
        assert!(!shutdown_requested());

        request_shutdown();
        assert!(shutdown_requested());

        reset_shutdown();
        assert!(!shutdown_requested());
    }
}
