// Voxtus - Transcribe YouTube videos and local media files to text
// Copyright (C) 2024 Johan Thorén <johan@thoren.xyz>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! SRT (SubRip) format output.
//!
//! Standard subtitle format compatible with video players.
//! Uses comma for milliseconds: `HH:MM:SS,mmm`

use super::Segment;

/// Format seconds as SRT timestamp (HH:MM:SS,mmm).
///
/// # Example
/// ```
/// use voxtus::formats::srt::format_timestamp;
///
/// assert_eq!(format_timestamp(0.0), "00:00:00,000");
/// assert_eq!(format_timestamp(65.5), "00:01:05,500");
/// assert_eq!(format_timestamp(3661.123), "01:01:01,123");
/// ```
pub fn format_timestamp(seconds: f64) -> String {
    let total_seconds = seconds.floor() as u64;
    let hours = total_seconds / 3600;
    let minutes = (total_seconds % 3600) / 60;
    let secs = total_seconds % 60;

    // Calculate milliseconds, clamping to 999 to handle rounding
    let milliseconds = ((seconds.fract() * 1000.0).round() as u64).min(999);

    format!(
        "{:02}:{:02}:{:02},{:03}",
        hours, minutes, secs, milliseconds
    )
}

/// Format a single segment as an SRT block.
///
/// # Example
/// ```
/// use voxtus::formats::{Segment, srt::format_segment};
///
/// let segment = Segment::new(0.0, 5.2, "Hello world");
/// let srt = format_segment(&segment, 1);
/// assert!(srt.contains("00:00:00,000 --> 00:00:05,200"));
/// ```
pub fn format_segment(segment: &Segment, index: usize) -> String {
    format!(
        "{}\n{} --> {}\n{}",
        index,
        format_timestamp(segment.start),
        format_timestamp(segment.end),
        segment.text.trim()
    )
}

/// Format multiple segments as SRT output.
pub fn format_transcript(segments: &[Segment]) -> String {
    segments
        .iter()
        .enumerate()
        .map(|(i, s)| format_segment(s, i + 1))
        .collect::<Vec<_>>()
        .join("\n\n")
}

#[cfg(test)]
mod tests {
    use super::*;

    use proptest::prelude::*;

    proptest! {
        #[test]
        fn prop_format_timestamp_produces_valid_format(seconds in 0.0f64..100000.0) {
            let result = format_timestamp(seconds);
            // Should match HH:MM:SS,mmm pattern
            let parts: Vec<&str> = result.split(',').collect();
            prop_assert_eq!(parts.len(), 2, "Should have exactly one comma");

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
        fn prop_format_timestamp_minutes_under_60(seconds in 0.0f64..100000.0) {
            let result = format_timestamp(seconds);
            let time_parts: Vec<&str> = result.split(',').next().unwrap().split(':').collect();
            let minutes: u64 = time_parts[1].parse().unwrap();
            prop_assert!(minutes < 60, "Minutes should be < 60");
        }

        #[test]
        fn prop_format_timestamp_seconds_under_60(seconds in 0.0f64..100000.0) {
            let result = format_timestamp(seconds);
            let time_parts: Vec<&str> = result.split(',').next().unwrap().split(':').collect();
            let secs: u64 = time_parts[2].parse().unwrap();
            prop_assert!(secs < 60, "Seconds should be < 60");
        }

        #[test]
        fn prop_format_timestamp_milliseconds_under_1000(seconds in 0.0f64..100000.0) {
            let result = format_timestamp(seconds);
            let ms: u64 = result.split(',').nth(1).unwrap().parse().unwrap();
            prop_assert!(ms < 1000, "Milliseconds should be < 1000");
        }
    }

    #[test]
    fn test_format_timestamp_zero() {
        assert_eq!(format_timestamp(0.0), "00:00:00,000");
    }

    #[test]
    fn test_format_timestamp_seconds() {
        assert_eq!(format_timestamp(5.5), "00:00:05,500");
        assert_eq!(format_timestamp(59.999), "00:00:59,999");
    }

    #[test]
    fn test_format_timestamp_minutes() {
        assert_eq!(format_timestamp(65.5), "00:01:05,500");
        assert_eq!(format_timestamp(125.0), "00:02:05,000");
    }

    #[test]
    fn test_format_timestamp_hours() {
        assert_eq!(format_timestamp(3661.123), "01:01:01,123");
        assert_eq!(format_timestamp(7200.0), "02:00:00,000");
    }

    #[test]
    fn test_format_timestamp_edge_cases() {
        assert_eq!(format_timestamp(0.999), "00:00:00,999");
        assert_eq!(format_timestamp(3599.999), "00:59:59,999");
    }

    #[test]
    fn test_format_segment_basic() {
        let segment = Segment::new(0.0, 5.2, "Hello world");
        let result = format_segment(&segment, 1);
        let expected = "1\n00:00:00,000 --> 00:00:05,200\nHello world";
        assert_eq!(result, expected);
    }

    #[test]
    fn test_format_segment_strips_whitespace() {
        let segment = Segment::new(10.5, 15.75, "  Text with spaces  ");
        let result = format_segment(&segment, 42);
        let expected = "42\n00:00:10,500 --> 00:00:15,750\nText with spaces";
        assert_eq!(result, expected);
    }

    #[test]
    fn test_format_segment_long_duration() {
        let segment = Segment::new(3661.5, 3665.0, "Long duration subtitle");
        let result = format_segment(&segment, 1);
        assert!(result.contains("01:01:01,500 --> 01:01:05,000"));
    }

    #[test]
    fn test_format_transcript_structure() {
        let segments = vec![
            Segment::new(0.0, 2.0, "Subtitle 1"),
            Segment::new(2.0, 4.0, "Subtitle 2"),
            Segment::new(4.0, 6.0, "Subtitle 3"),
        ];

        let result = format_transcript(&segments);

        // Check structure
        assert!(result.contains("1\n00:00:00,000 --> 00:00:02,000\nSubtitle 1"));
        assert!(result.contains("2\n00:00:02,000 --> 00:00:04,000\nSubtitle 2"));
        assert!(result.contains("3\n00:00:04,000 --> 00:00:06,000\nSubtitle 3"));

        // Check blank lines between segments
        assert!(result.contains("\n\n"));
    }

    #[test]
    fn test_format_transcript_single() {
        let segments = vec![Segment::new(0.0, 5.0, "Single subtitle")];
        let result = format_transcript(&segments);
        assert_eq!(result, "1\n00:00:00,000 --> 00:00:05,000\nSingle subtitle");
    }

    #[test]
    fn test_format_transcript_empty() {
        let segments: Vec<Segment> = vec![];
        let result = format_transcript(&segments);
        assert_eq!(result, "");
    }

    #[test]
    fn test_format_with_unicode() {
        let segment = Segment::new(0.0, 3.0, "Café résumé naïve 中文 🎵");
        let result = format_segment(&segment, 1);
        assert!(result.contains("Café résumé naïve 中文 🎵"));
    }
}
