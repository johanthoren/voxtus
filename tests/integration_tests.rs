// Voxtus - Transcribe YouTube videos and local media files to text
// Copyright (C) 2024 Johan Thorén <johan@thoren.xyz>
// SPDX-License-Identifier: AGPL-3.0-or-later

use assert_cmd::cargo::cargo_bin;
use predicates::prelude::*;
use std::fs;
use std::process::Command;

#[test]
fn test_cli_help() {
    let mut cmd = Command::new(cargo_bin("voxtus"));
    let output = cmd.arg("--help").output().unwrap();
    assert!(output.status.success());
    assert!(String::from_utf8_lossy(&output.stdout).contains("Usage: voxtus"));
}

#[test]
fn test_cli_list_models() {
    let mut cmd = Command::new(cargo_bin("voxtus"));
    let output = cmd.arg("--list-models").output().unwrap();
    assert!(output.status.success());
    assert!(String::from_utf8_lossy(&output.stdout).contains("Available Whisper Models"));
}

#[test]
fn test_transcribe_local_file() {
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
        .arg("txt")
        .output()
        .unwrap();

    assert!(output.status.success());
    // Should be quiet by default (no whisper_init messages)
    assert!(!String::from_utf8_lossy(&output.stderr).contains("whisper_init"));

    let output_file = output_dir.join("sample.txt");
    assert!(output_file.exists(), "Output file should exist");

    let content = fs::read_to_string(output_file).unwrap();
    assert!(!content.is_empty(), "Output content should not be empty");
}

#[test]
fn test_transcribe_multiple_formats() {
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
        .arg("json,srt,vtt")
        .output()
        .unwrap();

    assert!(output.status.success());
    assert!(output_dir.join("sample.json").exists());
    assert!(output_dir.join("sample.srt").exists());
    assert!(output_dir.join("sample.vtt").exists());
}

#[test]
fn test_transcribe_verbose() {
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
        .arg("txt")
        .arg("-v")
        .output()
        .unwrap();

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        output.status.success(),
        "Command failed with stderr: {}",
        stderr
    );
    // Verbose shows info logs
    assert!(
        stderr.contains("Converting:"),
        "Expected 'Converting:' in stderr, got: {}",
        stderr
    );
}
