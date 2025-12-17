// Voxtus - Transcribe YouTube videos and local media files to text
// Copyright (C) 2024 Johan Thorén <johan@thoren.xyz>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! JSON format output.
//!
//! Structured JSON format with transcript segments and metadata.

use serde::Serialize;

use super::{Metadata, Segment};

/// A segment in JSON output format.
#[derive(Debug, Serialize)]
struct JsonSegment {
    id: usize,
    start: f64,
    end: f64,
    text: String,
}

/// JSON output structure.
#[derive(Debug, Serialize)]
struct JsonOutput {
    transcript: Vec<JsonSegment>,
    metadata: JsonMetadata,
}

/// Metadata in JSON output format.
#[derive(Debug, Serialize)]
struct JsonMetadata {
    title: String,
    source: String,
    duration: Option<f64>,
    model: String,
    language: String,
}

/// Convert segments to JSON segment format.
fn to_json_segments(segments: &[Segment]) -> Vec<JsonSegment> {
    segments
        .iter()
        .enumerate()
        .map(|(i, s)| JsonSegment {
            id: i + 1,
            start: s.start,
            end: s.end,
            text: s.text.clone(),
        })
        .collect()
}

/// Convert metadata to JSON metadata format.
fn to_json_metadata(metadata: &Metadata) -> JsonMetadata {
    JsonMetadata {
        title: metadata.title.clone(),
        source: metadata.source.clone(),
        duration: metadata.duration,
        model: metadata.model.clone(),
        language: metadata
            .language
            .clone()
            .unwrap_or_else(|| "en".to_string()),
    }
}

/// Format segments and metadata as JSON string.
///
/// # Example
/// ```
/// use voxtus::formats::{Segment, Metadata, json::format_transcript};
///
/// let segments = vec![Segment::new(0.0, 5.0, "Hello")];
/// let metadata = Metadata::new("Test", "test.mp3", Some(5.0), "tiny", Some("en".to_string()));
/// let json = format_transcript(&segments, &metadata);
/// assert!(json.contains("\"transcript\""));
/// ```
pub fn format_transcript(segments: &[Segment], metadata: &Metadata) -> String {
    let output = JsonOutput {
        transcript: to_json_segments(segments),
        metadata: to_json_metadata(metadata),
    };

    serde_json::to_string_pretty(&output).unwrap_or_else(|_| "{}".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_segments() -> Vec<Segment> {
        vec![
            Segment::new(0.0, 2.0, "Segment 1 text"),
            Segment::new(2.0, 4.0, "Segment 2 text"),
            Segment::new(4.0, 6.0, "Segment 3 text"),
        ]
    }

    fn sample_metadata() -> Metadata {
        Metadata::new(
            "Test Title",
            "test.mp3",
            Some(6.0),
            "base",
            Some("en".to_string()),
        )
    }

    #[test]
    fn test_json_structure() {
        let json = format_transcript(&sample_segments(), &sample_metadata());
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();

        assert!(parsed.get("transcript").is_some());
        assert!(parsed.get("metadata").is_some());
    }

    #[test]
    fn test_json_transcript_array() {
        let json = format_transcript(&sample_segments(), &sample_metadata());
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();

        let transcript = parsed.get("transcript").unwrap().as_array().unwrap();
        assert_eq!(transcript.len(), 3);

        // Check first segment
        let first = &transcript[0];
        assert_eq!(first.get("id").unwrap().as_u64().unwrap(), 1);
        assert_eq!(first.get("start").unwrap().as_f64().unwrap(), 0.0);
        assert_eq!(first.get("end").unwrap().as_f64().unwrap(), 2.0);
        assert_eq!(
            first.get("text").unwrap().as_str().unwrap(),
            "Segment 1 text"
        );
    }

    #[test]
    fn test_json_metadata() {
        let json = format_transcript(&sample_segments(), &sample_metadata());
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();

        let metadata = parsed.get("metadata").unwrap();
        assert_eq!(
            metadata.get("title").unwrap().as_str().unwrap(),
            "Test Title"
        );
        assert_eq!(
            metadata.get("source").unwrap().as_str().unwrap(),
            "test.mp3"
        );
        assert_eq!(metadata.get("duration").unwrap().as_f64().unwrap(), 6.0);
        assert_eq!(metadata.get("model").unwrap().as_str().unwrap(), "base");
        assert_eq!(metadata.get("language").unwrap().as_str().unwrap(), "en");
    }

    #[test]
    fn test_json_segment_ids_sequential() {
        let json = format_transcript(&sample_segments(), &sample_metadata());
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();

        let transcript = parsed.get("transcript").unwrap().as_array().unwrap();
        for (i, segment) in transcript.iter().enumerate() {
            assert_eq!(segment.get("id").unwrap().as_u64().unwrap(), (i + 1) as u64);
        }
    }

    #[test]
    fn test_json_with_unicode() {
        let segments = vec![Segment::new(0.0, 3.0, "Café résumé 中文 🎵")];
        let metadata = Metadata::new(
            "Unicode Test",
            "test.mp3",
            Some(3.0),
            "tiny",
            Some("zh".to_string()),
        );

        let json = format_transcript(&segments, &metadata);
        assert!(json.contains("Café résumé 中文 🎵"));
        assert!(json.contains("\"language\": \"zh\""));
    }

    #[test]
    fn test_json_with_null_duration() {
        let segments = vec![Segment::new(0.0, 5.0, "Test")];
        let metadata = Metadata::new("Test", "test.mp3", None, "tiny", Some("en".to_string()));

        let json = format_transcript(&segments, &metadata);
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();

        assert!(
            parsed
                .get("metadata")
                .unwrap()
                .get("duration")
                .unwrap()
                .is_null()
        );
    }

    #[test]
    fn test_json_default_language() {
        let segments = vec![Segment::new(0.0, 5.0, "Test")];
        let metadata = Metadata::new("Test", "test.mp3", Some(5.0), "tiny", None);

        let json = format_transcript(&segments, &metadata);
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();

        assert_eq!(
            parsed
                .get("metadata")
                .unwrap()
                .get("language")
                .unwrap()
                .as_str()
                .unwrap(),
            "en"
        );
    }

    #[test]
    fn test_json_empty_segments() {
        let segments: Vec<Segment> = vec![];
        let metadata = Metadata::new(
            "Empty",
            "test.mp3",
            Some(0.0),
            "tiny",
            Some("en".to_string()),
        );

        let json = format_transcript(&segments, &metadata);
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();

        let transcript = parsed.get("transcript").unwrap().as_array().unwrap();
        assert!(transcript.is_empty());
    }
}
