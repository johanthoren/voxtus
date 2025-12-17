// Voxtus - Transcribe YouTube videos and local media files to text
// Copyright (C) 2024 Johan Thorén <johan@thoren.xyz>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Configuration handling.

use std::path::PathBuf;
use std::str::FromStr;

use crate::cli::Args;
use crate::error::{Error, Result};

/// Supported output formats.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutputFormat {
    Txt,
    Json,
    Srt,
    Vtt,
}

impl std::str::FromStr for OutputFormat {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        match s.to_lowercase().as_str() {
            "txt" => Ok(Self::Txt),
            "json" => Ok(Self::Json),
            "srt" => Ok(Self::Srt),
            "vtt" => Ok(Self::Vtt),
            _ => Err(Error::InvalidFormat(s.to_string())),
        }
    }
}

impl OutputFormat {
    /// Get the file extension for this format.
    pub fn extension(&self) -> &'static str {
        match self {
            Self::Txt => "txt",
            Self::Json => "json",
            Self::Srt => "srt",
            Self::Vtt => "vtt",
        }
    }
}

/// Available Whisper models.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WhisperModel {
    pub name: &'static str,
    pub description: &'static str,
    pub params: &'static str,
    pub vram: &'static str,
    pub languages: &'static str,
}

/// All available Whisper models.
pub const AVAILABLE_MODELS: &[WhisperModel] = &[
    WhisperModel {
        name: "tiny",
        description: "Fastest model, 39M parameters",
        params: "39M",
        vram: "~1GB",
        languages: "multilingual",
    },
    WhisperModel {
        name: "tiny.en",
        description: "English-only tiny model",
        params: "39M",
        vram: "~1GB",
        languages: "English only",
    },
    WhisperModel {
        name: "base",
        description: "Smaller balanced model, 74M parameters",
        params: "74M",
        vram: "~1GB",
        languages: "multilingual",
    },
    WhisperModel {
        name: "base.en",
        description: "English-only base model",
        params: "74M",
        vram: "~1GB",
        languages: "English only",
    },
    WhisperModel {
        name: "small",
        description: "Default balanced model, 244M parameters",
        params: "244M",
        vram: "~2GB",
        languages: "multilingual",
    },
    WhisperModel {
        name: "small.en",
        description: "English-only small model",
        params: "244M",
        vram: "~2GB",
        languages: "English only",
    },
    WhisperModel {
        name: "medium",
        description: "Good accuracy model, 769M parameters",
        params: "769M",
        vram: "~5GB",
        languages: "multilingual",
    },
    WhisperModel {
        name: "medium.en",
        description: "English-only medium model",
        params: "769M",
        vram: "~5GB",
        languages: "English only",
    },
    WhisperModel {
        name: "large",
        description: "Highest accuracy model, 1550M parameters",
        params: "1550M",
        vram: "~10GB",
        languages: "multilingual",
    },
    WhisperModel {
        name: "large-v2",
        description: "Improved large model, 1550M parameters",
        params: "1550M",
        vram: "~10GB",
        languages: "multilingual",
    },
    WhisperModel {
        name: "large-v3",
        description: "Latest large model, 1550M parameters",
        params: "1550M",
        vram: "~10GB",
        languages: "multilingual",
    },
];

/// Validated configuration for the transcription process.
#[derive(Debug, Clone)]
pub struct Config {
    pub input_path: String,
    pub formats: Vec<OutputFormat>,
    pub custom_name: Option<String>,
    pub output_dir: PathBuf,
    pub verbose_level: u8,
    pub keep_audio: bool,
    pub model: String,
    pub overwrite_files: bool,
    pub stdout_mode: bool,
}

impl Config {
    /// Create a Config from parsed CLI arguments.
    pub fn from_args(args: &Args) -> Result<Self> {
        let formats = parse_formats(&args.format, args.stdout)?;
        let model = validate_model(&args.model)?;
        let output_dir = resolve_output_dir(args.output.as_deref())?;
        let custom_name = args.name.as_ref().map(|n| strip_txt_extension(n));

        Ok(Self {
            input_path: args.input.clone().unwrap_or_default(),
            formats,
            custom_name,
            output_dir,
            verbose_level: args.verbose,
            keep_audio: args.keep,
            model,
            overwrite_files: args.overwrite,
            stdout_mode: args.stdout,
        })
    }
}

/// Parse comma-separated format string into validated formats.
///
/// # Examples
///
/// ```
/// use voxtus::config::parse_formats;
///
/// let formats = parse_formats("txt,json", false).unwrap();
/// assert_eq!(formats.len(), 2);
///
/// // Case insensitive
/// let formats = parse_formats("TXT,JSON", false).unwrap();
/// assert_eq!(formats.len(), 2);
///
/// // Multiple formats not allowed with stdout mode
/// assert!(parse_formats("txt,json", true).is_err());
/// ```
pub fn parse_formats(format_str: &str, stdout_mode: bool) -> Result<Vec<OutputFormat>> {
    let formats: Result<Vec<OutputFormat>> = format_str
        .split(',')
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .map(OutputFormat::from_str)
        .collect();

    let formats = formats?;

    if stdout_mode && formats.len() > 1 {
        return Err(Error::MultipleFormatsWithStdout);
    }

    Ok(formats)
}

/// Validate that the model name is valid.
///
/// # Examples
///
/// ```
/// use voxtus::config::validate_model;
///
/// assert!(validate_model("tiny").is_ok());
/// assert!(validate_model("small.en").is_ok());
///
/// // "large" normalizes to "large-v3"
/// assert_eq!(validate_model("large").unwrap(), "large-v3");
///
/// // Invalid models return error
/// assert!(validate_model("invalid").is_err());
/// ```
pub fn validate_model(model: &str) -> Result<String> {
    // Normalize "large" to "large-v3"
    let normalized = if model == "large" { "large-v3" } else { model };

    if AVAILABLE_MODELS.iter().any(|m| m.name == normalized) {
        Ok(normalized.to_string())
    } else {
        Err(Error::InvalidModel(model.to_string()))
    }
}

/// Resolve output directory, expanding ~ and creating if needed.
pub fn resolve_output_dir(output: Option<&str>) -> Result<PathBuf> {
    let path = match output {
        Some(p) => expand_tilde(p),
        None => std::env::current_dir()?,
    };

    // Create directory if it doesn't exist
    if !path.exists() {
        std::fs::create_dir_all(&path)?;
    }

    Ok(path)
}

/// Expand ~ to home directory.
pub fn expand_tilde(path: &str) -> PathBuf {
    if let Some(stripped) = path.strip_prefix("~/")
        && let Some(home) = dirs::home_dir()
    {
        return home.join(stripped);
    }
    PathBuf::from(path)
}

/// Strip .txt extension if present.
pub fn strip_txt_extension(name: &str) -> String {
    name.strip_suffix(".txt").unwrap_or(name).to_string()
}

/// Custom name takes precedence over title.
pub fn get_final_name(title: &str, custom_name: Option<&str>) -> String {
    match custom_name {
        Some(name) if !name.is_empty() => name.to_string(),
        _ => title.to_string(),
    }
}

pub fn is_url(input: &str) -> bool {
    input.starts_with("http://") || input.starts_with("https://")
}

#[cfg(test)]
mod tests {
    use super::*;

    use proptest::prelude::*;

    proptest! {
        #[test]
        fn prop_valid_models_always_validate(model in prop_oneof![
            Just("tiny"),
            Just("tiny.en"),
            Just("base"),
            Just("base.en"),
            Just("small"),
            Just("small.en"),
            Just("medium"),
            Just("medium.en"),
            Just("large"),
            Just("large-v2"),
            Just("large-v3"),
        ]) {
            prop_assert!(validate_model(model).is_ok());
        }

        #[test]
        fn prop_invalid_models_always_fail(model in "[a-z]{1,10}") {
            // Skip if it happens to be a valid model name
            let valid_names = ["tiny", "base", "small", "medium", "large"];
            if !valid_names.contains(&model.as_str())
                && !model.ends_with(".en")
                && !model.starts_with("large-v")
            {
                prop_assert!(validate_model(&model).is_err());
            }
        }

        #[test]
        fn prop_valid_formats_always_parse(format in prop_oneof![
            Just("txt"),
            Just("json"),
            Just("srt"),
            Just("vtt"),
            Just("TXT"),
            Just("JSON"),
            Just("SRT"),
            Just("VTT"),
        ]) {
            prop_assert!(parse_formats(&format, false).is_ok());
        }

        #[test]
        fn prop_format_list_parses_correctly(
            f1 in prop_oneof![Just("txt"), Just("json"), Just("srt"), Just("vtt")],
            f2 in prop_oneof![Just("txt"), Just("json"), Just("srt"), Just("vtt")],
        ) {
            let format_str = format!("{},{}", f1, f2);
            let result = parse_formats(&format_str, false);
            prop_assert!(result.is_ok());
            let formats = result.unwrap();
            prop_assert!(formats.len() <= 2);
        }

        #[test]
        fn prop_multiple_formats_fail_with_stdout(
            f1 in prop_oneof![Just("txt"), Just("json"), Just("srt"), Just("vtt")],
            f2 in prop_oneof![Just("txt"), Just("json"), Just("srt"), Just("vtt")],
        ) {
            let format_str = format!("{},{}", f1, f2);
            let result = parse_formats(&format_str, true); // stdout mode
            prop_assert!(result.is_err());
        }

        #[test]
        fn prop_strip_txt_extension_idempotent(name in "[a-zA-Z0-9_-]{1,20}") {
            let stripped = strip_txt_extension(&name);
            let double_stripped = strip_txt_extension(&stripped);
            prop_assert_eq!(stripped, double_stripped);
        }

        #[test]
        fn prop_strip_txt_removes_extension(name in "[a-zA-Z0-9_-]{1,20}") {
            let with_ext = format!("{}.txt", name);
            let stripped = strip_txt_extension(&with_ext);
            prop_assert_eq!(stripped, name);
        }
    }

    #[test]
    fn test_parse_single_format() {
        let formats = parse_formats("txt", false).unwrap();
        assert_eq!(formats, vec![OutputFormat::Txt]);
    }

    #[test]
    fn test_parse_multiple_formats() {
        let formats = parse_formats("txt,json", false).unwrap();
        assert_eq!(formats, vec![OutputFormat::Txt, OutputFormat::Json]);
    }

    #[test]
    fn test_parse_formats_with_spaces() {
        let formats = parse_formats("txt, json", false).unwrap();
        assert_eq!(formats, vec![OutputFormat::Txt, OutputFormat::Json]);
    }

    #[test]
    fn test_parse_formats_case_insensitive() {
        let formats = parse_formats("TXT,JSON", false).unwrap();
        assert_eq!(formats, vec![OutputFormat::Txt, OutputFormat::Json]);
    }

    #[test]
    fn test_invalid_format_error() {
        let result = parse_formats("invalid", false);
        assert!(matches!(result, Err(Error::InvalidFormat(_))));
    }

    #[test]
    fn test_multiple_formats_with_stdout_error() {
        let result = parse_formats("txt,json", true);
        assert!(matches!(result, Err(Error::MultipleFormatsWithStdout)));
    }

    #[test]
    fn test_single_format_with_stdout_allowed() {
        let formats = parse_formats("json", true).unwrap();
        assert_eq!(formats, vec![OutputFormat::Json]);
    }

    #[test]
    fn test_validate_model_valid() {
        assert_eq!(validate_model("tiny").unwrap(), "tiny");
        assert_eq!(validate_model("small").unwrap(), "small");
        assert_eq!(validate_model("large-v3").unwrap(), "large-v3");
    }

    #[test]
    fn test_validate_model_normalizes_large() {
        assert_eq!(validate_model("large").unwrap(), "large-v3");
    }

    #[test]
    fn test_validate_model_invalid() {
        let result = validate_model("invalid-model");
        assert!(matches!(result, Err(Error::InvalidModel(_))));
    }

    #[test]
    fn test_strip_txt_extension() {
        assert_eq!(strip_txt_extension("my_file.txt"), "my_file");
        assert_eq!(strip_txt_extension("my_file"), "my_file");
        assert_eq!(strip_txt_extension("my_file.json"), "my_file.json");
    }

    #[test]
    fn test_get_final_name_uses_custom_name() {
        assert_eq!(
            get_final_name("original_title", Some("custom_name")),
            "custom_name"
        );
    }

    #[test]
    fn test_get_final_name_uses_title_when_no_custom() {
        assert_eq!(get_final_name("original_title", None), "original_title");
    }

    #[test]
    fn test_get_final_name_empty_custom_uses_title() {
        assert_eq!(get_final_name("original_title", Some("")), "original_title");
    }

    #[test]
    fn test_expand_tilde() {
        let expanded = expand_tilde("~/test");
        assert!(expanded.is_absolute() || !expanded.to_string_lossy().starts_with("~"));
    }

    #[test]
    fn test_expand_tilde_no_tilde() {
        let path = "/absolute/path";
        assert_eq!(expand_tilde(path), PathBuf::from(path));
    }

    #[test]
    fn test_is_url() {
        assert!(is_url("https://example.com/video.mp4"));
        assert!(is_url("http://localhost:8080/file.mp3"));
        assert!(!is_url("/local/path/file.mp3"));
        assert!(!is_url("file.mp3"));
    }

    #[test]
    fn test_output_format_extension() {
        assert_eq!(OutputFormat::Txt.extension(), "txt");
        assert_eq!(OutputFormat::Json.extension(), "json");
        assert_eq!(OutputFormat::Srt.extension(), "srt");
        assert_eq!(OutputFormat::Vtt.extension(), "vtt");
    }
}
