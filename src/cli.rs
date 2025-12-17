// Voxtus - Transcribe YouTube videos and local media files to text
// Copyright (C) 2024 Johan Thorén <johan@thoren.xyz>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Command-line argument parsing.

use clap::Parser;

/// Transcribe YouTube videos and local media files to text.
#[derive(Parser, Debug, Clone)]
#[command(name = "voxtus")]
#[command(author, version, about, long_about = None)]
pub struct Args {
    /// YouTube URL or local media file path
    #[arg(required_unless_present = "list_models")]
    pub input: Option<String>,

    /// Output format(s), comma-separated: txt,json,srt,vtt
    #[arg(short, long, default_value = "txt")]
    pub format: String,

    /// Base name for output files (no extension)
    #[arg(short, long)]
    pub name: Option<String>,

    /// Output directory
    #[arg(short, long)]
    pub output: Option<String>,

    /// Increase verbosity (-v, -vv for debug)
    #[arg(short, long, action = clap::ArgAction::Count)]
    pub verbose: u8,

    /// Keep the downloaded/converted audio file
    #[arg(short, long)]
    pub keep: bool,

    /// Whisper model to use
    #[arg(long, default_value = "small")]
    pub model: String,

    /// List available models and exit
    #[arg(long)]
    pub list_models: bool,

    /// Overwrite existing files without confirmation
    #[arg(long)]
    pub overwrite: bool,

    /// Output to stdout only (single format, no files created)
    #[arg(long)]
    pub stdout: bool,
}

impl Args {
    /// Parse arguments from command line.
    pub fn parse_args() -> Self {
        Self::parse()
    }

    /// Parse arguments from iterator (for testing).
    pub fn parse_from_iter<I, T>(iter: I) -> Self
    where
        I: IntoIterator<Item = T>,
        T: Into<std::ffi::OsString> + Clone,
    {
        Self::parse_from(iter)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_basic_arguments() {
        let args = Args::parse_from_iter(["voxtus", "test.mp3"]);
        assert_eq!(args.input, Some("test.mp3".to_string()));
        assert_eq!(args.format, "txt");
        assert_eq!(args.verbose, 0);
        assert!(!args.keep);
        assert!(!args.overwrite);
        assert!(!args.stdout);
        assert_eq!(args.model, "small");
    }

    #[test]
    fn test_parse_all_flags() {
        let args = Args::parse_from_iter([
            "voxtus",
            "test.mp3",
            "-v",
            "-v",
            "--keep",
            "--overwrite",
            "--format",
            "json",
            "--name",
            "custom_name",
            "--output",
            "/tmp/output",
            "--stdout",
            "--model",
            "tiny",
        ]);

        assert_eq!(args.input, Some("test.mp3".to_string()));
        assert_eq!(args.verbose, 2);
        assert!(args.keep);
        assert!(args.overwrite);
        assert_eq!(args.format, "json");
        assert_eq!(args.name, Some("custom_name".to_string()));
        assert_eq!(args.output, Some("/tmp/output".to_string()));
        assert!(args.stdout);
        assert_eq!(args.model, "tiny");
    }

    #[test]
    fn test_parse_short_flags() {
        let args = Args::parse_from_iter([
            "voxtus",
            "test.mp3",
            "-v",
            "-k",
            "-f",
            "txt,json",
            "-n",
            "short_name",
            "-o",
            "/tmp/short",
        ]);

        assert_eq!(args.input, Some("test.mp3".to_string()));
        assert_eq!(args.verbose, 1);
        assert!(args.keep);
        assert_eq!(args.format, "txt,json");
        assert_eq!(args.name, Some("short_name".to_string()));
        assert_eq!(args.output, Some("/tmp/short".to_string()));
    }

    #[test]
    fn test_list_models_without_input() {
        let args = Args::parse_from_iter(["voxtus", "--list-models"]);
        assert!(args.list_models);
        assert!(args.input.is_none());
    }
}
