// Voxtus - Transcribe YouTube videos and local media files to text
// Copyright (C) 2024 Johan Thorén <johan@thoren.xyz>
// SPDX-License-Identifier: AGPL-3.0-or-later

use assert_cmd::cargo::cargo_bin;
use std::fs;
use std::process::Command;

#[test]
fn test_regression_output() {
    let temp_dir = tempfile::tempdir().unwrap();
    let output_dir = temp_dir.path();

    let mut cmd = Command::new(cargo_bin("voxtus"));
    let output = cmd
        .arg("tests/data/sample.mp3")
        .arg("--model")
        .arg("tiny")
        .arg("--output")
        .arg(output_dir.to_str().unwrap())
        .arg("--format")
        .arg("json,srt,txt,vtt")
        .output()
        .unwrap();

    assert!(output.status.success());

    let formats = ["json", "srt", "txt", "vtt"];
    for fmt in formats {
        let output_file = output_dir.join(format!("sample.{}", fmt));
        assert!(output_file.exists(), "Output file {} missing", fmt);

        let content = fs::read_to_string(&output_file).unwrap();

        // Basic content check - should contain recognizable words from the sample audio
        let keywords = ["Voxtus", "VoxDus", "command line tool", "transcribing"];
        let found = keywords
            .iter()
            .any(|k| content.to_lowercase().contains(&k.to_lowercase()));

        assert!(
            found,
            "Output {} did not contain expected keywords. Content: {}",
            fmt, content
        );
    }
}
