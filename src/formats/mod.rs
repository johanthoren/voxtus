// Voxtus - Transcribe YouTube videos and local media files to text
// Copyright (C) 2024 Johan Thorén <johan@thoren.xyz>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Output format implementations.
//!
//! This module contains pure functions for formatting transcription output
//! in various formats: TXT, JSON, SRT, and VTT.

pub mod json;
pub mod srt;
pub mod txt;
pub mod vtt;

use serde::{Deserialize, Serialize};

/// A transcription segment with timing information.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Segment {
    pub start: f64,
    pub end: f64,
    pub text: String,
}

impl Segment {
    pub fn new(start: f64, end: f64, text: impl Into<String>) -> Self {
        Self {
            start,
            end,
            text: text.into(),
        }
    }
}

/// Metadata about the transcription.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Metadata {
    pub title: String,
    pub source: String,
    pub duration: Option<f64>,
    pub model: String,
    pub language: Option<String>,
}

impl Metadata {
    pub fn new(
        title: impl Into<String>,
        source: impl Into<String>,
        duration: Option<f64>,
        model: impl Into<String>,
        language: Option<String>,
    ) -> Self {
        Self {
            title: title.into(),
            source: source.into(),
            duration,
            model: model.into(),
            language,
        }
    }
}

/// A complete transcript with segments and metadata.
#[derive(Debug, Clone, PartialEq)]
pub struct Transcript {
    pub segments: Vec<Segment>,
    pub metadata: Metadata,
}

impl Transcript {
    pub fn new(segments: Vec<Segment>, metadata: Metadata) -> Self {
        Self { segments, metadata }
    }

    /// Format the transcript as TXT.
    pub fn to_txt(&self) -> String {
        txt::format_transcript(&self.segments)
    }

    /// Format the transcript as JSON.
    pub fn to_json(&self) -> String {
        json::format_transcript(&self.segments, &self.metadata)
    }

    /// Format the transcript as SRT.
    pub fn to_srt(&self) -> String {
        srt::format_transcript(&self.segments)
    }

    /// Format the transcript as VTT.
    pub fn to_vtt(&self) -> String {
        vtt::format_transcript(&self.segments, &self.metadata)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_segments() -> Vec<Segment> {
        vec![
            Segment::new(0.0, 5.2, "Hello world"),
            Segment::new(5.2, 10.5, "This is a test"),
        ]
    }

    fn sample_metadata() -> Metadata {
        Metadata::new(
            "Test Video",
            "test.mp3",
            Some(10.5),
            "tiny",
            Some("en".to_string()),
        )
    }

    #[test]
    fn test_segment_creation() {
        let segment = Segment::new(1.5, 3.0, "Test text");
        assert_eq!(segment.start, 1.5);
        assert_eq!(segment.end, 3.0);
        assert_eq!(segment.text, "Test text");
    }

    #[test]
    fn test_metadata_creation() {
        let metadata = Metadata::new(
            "Title",
            "source.mp3",
            Some(60.0),
            "small",
            Some("en".to_string()),
        );
        assert_eq!(metadata.title, "Title");
        assert_eq!(metadata.source, "source.mp3");
        assert_eq!(metadata.duration, Some(60.0));
        assert_eq!(metadata.model, "small");
        assert_eq!(metadata.language, Some("en".to_string()));
    }

    #[test]
    fn test_transcript_to_txt() {
        let transcript = Transcript::new(sample_segments(), sample_metadata());
        let output = transcript.to_txt();
        assert!(output.contains("[0.00 - 5.20]: Hello world"));
        assert!(output.contains("[5.20 - 10.50]: This is a test"));
    }

    #[test]
    fn test_transcript_to_json() {
        let transcript = Transcript::new(sample_segments(), sample_metadata());
        let output = transcript.to_json();
        assert!(output.contains("\"transcript\""));
        assert!(output.contains("\"metadata\""));
        assert!(output.contains("Hello world"));
    }

    #[test]
    fn test_transcript_to_srt() {
        let transcript = Transcript::new(sample_segments(), sample_metadata());
        let output = transcript.to_srt();
        assert!(output.contains("00:00:00,000 --> 00:00:05,200"));
        assert!(output.contains("Hello world"));
    }

    #[test]
    fn test_transcript_to_vtt() {
        let transcript = Transcript::new(sample_segments(), sample_metadata());
        let output = transcript.to_vtt();
        assert!(output.starts_with("WEBVTT"));
        assert!(output.contains("00:00:00.000 --> 00:00:05.200"));
        assert!(output.contains("Hello world"));
    }
}
