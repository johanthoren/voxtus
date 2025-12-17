// Voxtus - Transcribe YouTube videos and local media files to text
// Copyright (C) 2024 Johan Thorén <johan@thoren.xyz>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! TXT format output.
//!
//! Plain text format with timestamps, designed to be LLM-friendly.
//! Format: `[start - end]: text`

use super::Segment;

/// Format a single segment as a TXT line.
///
/// # Example
/// ```
/// use voxtus::formats::{Segment, txt::format_segment};
///
/// let segment = Segment::new(0.0, 5.5, "Hello world");
/// assert_eq!(format_segment(&segment), "[0.00 - 5.50]: Hello world");
/// ```
pub fn format_segment(segment: &Segment) -> String {
    format!(
        "[{:.2} - {:.2}]: {}",
        segment.start, segment.end, segment.text
    )
}

/// Format multiple segments as TXT output.
///
/// Each segment is on its own line.
pub fn format_transcript(segments: &[Segment]) -> String {
    segments
        .iter()
        .map(format_segment)
        .collect::<Vec<_>>()
        .join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_basic_segment() {
        let segment = Segment::new(0.0, 5.5, "Hello world");
        assert_eq!(format_segment(&segment), "[0.00 - 5.50]: Hello world");
    }

    #[test]
    fn test_format_with_decimal_precision() {
        let segment = Segment::new(1.234, 7.89, "Test message");
        // Should truncate to 2 decimal places
        assert_eq!(format_segment(&segment), "[1.23 - 7.89]: Test message");
    }

    #[test]
    fn test_format_with_special_characters() {
        let segment = Segment::new(10.0, 15.0, "Hello, world! How are you? 🎉");
        assert_eq!(
            format_segment(&segment),
            "[10.00 - 15.00]: Hello, world! How are you? 🎉"
        );
    }

    #[test]
    fn test_format_with_unicode() {
        let segment = Segment::new(0.0, 3.0, "Café résumé naïve 中文");
        assert_eq!(
            format_segment(&segment),
            "[0.00 - 3.00]: Café résumé naïve 中文"
        );
    }

    #[test]
    fn test_format_transcript_single() {
        let segments = vec![Segment::new(0.0, 5.0, "Single line")];
        assert_eq!(format_transcript(&segments), "[0.00 - 5.00]: Single line");
    }

    #[test]
    fn test_format_transcript_multiple() {
        let segments = vec![
            Segment::new(0.0, 3.0, "First line"),
            Segment::new(3.0, 6.0, "Second line"),
        ];
        let expected = "[0.00 - 3.00]: First line\n[3.00 - 6.00]: Second line";
        assert_eq!(format_transcript(&segments), expected);
    }

    #[test]
    fn test_format_transcript_empty() {
        let segments: Vec<Segment> = vec![];
        assert_eq!(format_transcript(&segments), "");
    }

    #[test]
    fn test_format_large_timestamps() {
        let segment = Segment::new(3661.5, 3665.0, "Over an hour");
        assert_eq!(
            format_segment(&segment),
            "[3661.50 - 3665.00]: Over an hour"
        );
    }
}
