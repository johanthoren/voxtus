// Voxtus - Transcribe YouTube videos and local media files to text
// Copyright (C) 2024 Johan Thorén <johan@thoren.xyz>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! VTT (WebVTT) format output.
//!
//! Web standard subtitle format with metadata support.
//! Uses dot for milliseconds: `HH:MM:SS.mmm`

use super::{Metadata, Segment};

/// Format seconds as VTT timestamp (HH:MM:SS.mmm).
///
/// Note: VTT uses dot (.) for milliseconds, unlike SRT which uses comma (,).
///
/// # Example
/// ```
/// use voxtus::formats::vtt::format_timestamp;
///
/// assert_eq!(format_timestamp(0.0), "00:00:00.000");
/// assert_eq!(format_timestamp(65.5), "00:01:05.500");
/// assert_eq!(format_timestamp(3661.123), "01:01:01.123");
/// ```
pub fn format_timestamp(seconds: f64) -> String {
    let total_seconds = seconds.floor() as u64;
    let hours = total_seconds / 3600;
    let minutes = (total_seconds % 3600) / 60;
    let secs = total_seconds % 60;

    // Calculate milliseconds, clamping to 999 to handle rounding
    let milliseconds = ((seconds.fract() * 1000.0).round() as u64).min(999);

    format!(
        "{:02}:{:02}:{:02}.{:03}",
        hours, minutes, secs, milliseconds
    )
}

/// Format a single segment as a VTT cue.
pub fn format_segment(segment: &Segment) -> String {
    format!(
        "{} --> {}\n{}",
        format_timestamp(segment.start),
        format_timestamp(segment.end),
        segment.text.trim()
    )
}

/// Format metadata as VTT NOTE blocks.
pub fn format_metadata(metadata: &Metadata) -> String {
    let mut notes = Vec::new();

    // Title
    notes.push(format!("NOTE Title\n{}", metadata.title));

    // Source
    notes.push(format!("NOTE Source\n{}", metadata.source));

    // Duration
    let duration_str = metadata
        .duration
        .map(format_timestamp)
        .unwrap_or_else(|| "unknown".to_string());
    notes.push(format!("NOTE Duration\n{}", duration_str));

    // Language
    let language = metadata.language.as_deref().unwrap_or("unknown");
    notes.push(format!("NOTE Language\n{}", language));

    // Model
    notes.push(format!("NOTE Model\n{}", metadata.model));

    notes.join("\n\n")
}

/// Format segments and metadata as VTT output.
pub fn format_transcript(segments: &[Segment], metadata: &Metadata) -> String {
    let mut parts = Vec::new();

    // VTT header
    parts.push("WEBVTT".to_string());

    // Metadata notes
    parts.push(format_metadata(metadata));

    // Segments
    for segment in segments {
        parts.push(format_segment(segment));
    }

    parts.join("\n\n")
}

#[cfg(test)]
mod tests {
    use super::*;

    use proptest::prelude::*;

    proptest! {
        #[test]
        fn prop_format_timestamp_produces_valid_format(seconds in 0.0f64..100000.0) {
            let result = format_timestamp(seconds);
            // Should match HH:MM:SS.mmm pattern (VTT uses dot, not comma)
            let parts: Vec<&str> = result.split('.').collect();
            prop_assert_eq!(parts.len(), 2, "Should have exactly one dot");

            let time_parts: Vec<&str> = parts[0].split(':').collect();
            prop_assert_eq!(time_parts.len(), 3, "Should have HH:MM:SS format");

            // Milliseconds should be 3 digits
            prop_assert_eq!(parts[1].len(), 3, "Milliseconds should be 3 digits");

            // All parts should be numeric
            for part in &time_parts {
                prop_assert!(part.parse::<u64>().is_ok(), "Time parts should be numeric");
            }
            prop_assert!(parts[1].parse::<u64>().is_ok(), "Milliseconds should be numeric");
        }

        #[test]
        fn prop_format_timestamp_uses_dot_not_comma(seconds in 0.0f64..100000.0) {
            let result = format_timestamp(seconds);
            prop_assert!(result.contains('.'), "VTT should use dot for milliseconds");
            prop_assert!(!result.contains(','), "VTT should not use comma");
        }

        #[test]
        fn prop_format_timestamp_minutes_under_60(seconds in 0.0f64..100000.0) {
            let result = format_timestamp(seconds);
            let time_parts: Vec<&str> = result.split('.').next().unwrap().split(':').collect();
            let minutes: u64 = time_parts[1].parse().unwrap();
            prop_assert!(minutes < 60, "Minutes should be < 60");
        }

        #[test]
        fn prop_format_timestamp_seconds_under_60(seconds in 0.0f64..100000.0) {
            let result = format_timestamp(seconds);
            let time_parts: Vec<&str> = result.split('.').next().unwrap().split(':').collect();
            let secs: u64 = time_parts[2].parse().unwrap();
            prop_assert!(secs < 60, "Seconds should be < 60");
        }

        #[test]
        fn prop_format_timestamp_milliseconds_under_1000(seconds in 0.0f64..100000.0) {
            let result = format_timestamp(seconds);
            let ms: u64 = result.split('.').nth(1).unwrap().parse().unwrap();
            prop_assert!(ms < 1000, "Milliseconds should be < 1000");
        }
    }

    fn sample_metadata() -> Metadata {
        Metadata::new(
            "Test Video",
            "test.mp4",
            Some(123.45),
            "base",
            Some("en".to_string()),
        )
    }

    #[test]
    fn test_format_timestamp_zero() {
        assert_eq!(format_timestamp(0.0), "00:00:00.000");
    }

    #[test]
    fn test_format_timestamp_seconds() {
        assert_eq!(format_timestamp(5.5), "00:00:05.500");
        assert_eq!(format_timestamp(59.999), "00:00:59.999");
    }

    #[test]
    fn test_format_timestamp_minutes() {
        assert_eq!(format_timestamp(65.5), "00:01:05.500");
        assert_eq!(format_timestamp(125.0), "00:02:05.000");
    }

    #[test]
    fn test_format_timestamp_hours() {
        assert_eq!(format_timestamp(3661.123), "01:01:01.123");
        assert_eq!(format_timestamp(7200.0), "02:00:00.000");
    }

    #[test]
    fn test_format_timestamp_edge_cases() {
        assert_eq!(format_timestamp(0.999), "00:00:00.999");
        assert_eq!(format_timestamp(3599.999), "00:59:59.999");
    }

    #[test]
    fn test_format_segment_basic() {
        let segment = Segment::new(0.0, 5.2, "Hello world");
        let result = format_segment(&segment);
        let expected = "00:00:00.000 --> 00:00:05.200\nHello world";
        assert_eq!(result, expected);
    }

    #[test]
    fn test_format_segment_strips_whitespace() {
        let segment = Segment::new(10.5, 15.75, "  Text with spaces  ");
        let result = format_segment(&segment);
        let expected = "00:00:10.500 --> 00:00:15.750\nText with spaces";
        assert_eq!(result, expected);
    }

    #[test]
    fn test_format_metadata_complete() {
        let metadata = sample_metadata();
        let result = format_metadata(&metadata);

        assert!(result.contains("NOTE Title\nTest Video"));
        assert!(result.contains("NOTE Source\ntest.mp4"));
        assert!(result.contains("NOTE Duration\n00:02:03.450"));
        assert!(result.contains("NOTE Language\nen"));
        assert!(result.contains("NOTE Model\nbase"));
    }

    #[test]
    fn test_format_metadata_minimal() {
        let metadata = Metadata::new("unknown", "unknown", None, "base", None);
        let result = format_metadata(&metadata);

        assert!(result.contains("NOTE Title\nunknown"));
        assert!(result.contains("NOTE Source\nunknown"));
        assert!(result.contains("NOTE Duration\nunknown"));
        assert!(result.contains("NOTE Language\nunknown"));
        assert!(result.contains("NOTE Model\nbase"));
    }

    #[test]
    fn test_format_transcript_structure() {
        let segments = vec![
            Segment::new(0.0, 2.0, "Subtitle 1"),
            Segment::new(2.0, 4.0, "Subtitle 2"),
        ];
        let metadata = sample_metadata();

        let result = format_transcript(&segments, &metadata);

        // Check header
        assert!(result.starts_with("WEBVTT"));

        // Check metadata
        assert!(result.contains("NOTE Title\nTest Video"));

        // Check segments
        assert!(result.contains("00:00:00.000 --> 00:00:02.000\nSubtitle 1"));
        assert!(result.contains("00:00:02.000 --> 00:00:04.000\nSubtitle 2"));
    }

    #[test]
    fn test_format_transcript_with_long_duration() {
        let segments = vec![Segment::new(3661.5, 3665.0, "Long duration subtitle")];
        let metadata = Metadata::new(
            "Test",
            "test.mp4",
            Some(3665.0),
            "base",
            Some("en".to_string()),
        );

        let result = format_transcript(&segments, &metadata);

        assert!(result.contains("01:01:01.500 --> 01:01:05.000"));
        assert!(result.contains("NOTE Duration\n01:01:05.000"));
    }

    #[test]
    fn test_format_with_unicode() {
        let segments = vec![Segment::new(0.0, 3.0, "Café résumé naïve 中文 🎵")];
        let metadata = Metadata::new(
            "Unicode Test",
            "test.mp4",
            Some(3.0),
            "base",
            Some("zh".to_string()),
        );

        let result = format_transcript(&segments, &metadata);

        assert!(result.contains("Café résumé naïve 中文 🎵"));
        assert!(result.contains("NOTE Language\nzh"));
    }

    #[test]
    fn test_format_transcript_empty_segments() {
        let segments: Vec<Segment> = vec![];
        let metadata = sample_metadata();

        let result = format_transcript(&segments, &metadata);

        assert!(result.starts_with("WEBVTT"));
        assert!(result.contains("NOTE Title"));
        // Should not have any cue blocks
        assert!(!result.contains(" --> "));
    }

    #[test]
    fn test_vtt_uses_dot_not_comma() {
        // VTT uses dot for milliseconds, SRT uses comma
        let timestamp = format_timestamp(5.5);
        assert!(timestamp.contains("."));
        assert!(!timestamp.contains(","));
    }
}
