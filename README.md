# Voxtus

[![crates.io](https://img.shields.io/crates/v/voxtus.svg)](https://crates.io/crates/voxtus)
[![Documentation](https://docs.rs/voxtus/badge.svg)](https://docs.rs/voxtus)
[![AGPL-3.0-or-later licensed](https://img.shields.io/crates/l/voxtus.svg)](./LICENSE)

A command-line tool for transcribing YouTube videos and local media files to text using OpenAI's Whisper.

## Features

- Transcribe YouTube videos by URL
- Transcribe local audio/video files
- Multiple output formats: TXT, JSON, SRT, VTT
- Automatic Whisper model downloading
- Signal handling for graceful cleanup

## Installation

### Prerequisites

- [Rust](https://rustup.rs/) (1.85+)
- [FFmpeg](https://ffmpeg.org/) in your PATH
- CMake (for building whisper-rs)

### From crates.io

```bash
cargo install voxtus
```

### From source

```bash
git clone https://github.com/johanthoren/voxtus
cd voxtus
cargo install --path .
```

## Usage

```bash
# Transcribe a YouTube video
voxtus https://www.youtube.com/watch?v=VIDEO_ID

# Transcribe a local file
voxtus recording.mp3

# Specify output format(s)
voxtus -f json,srt video.mp4

# Use a different Whisper model
voxtus --model large-v3 audio.mp3

# Output to stdout (for piping)
voxtus --stdout -f json video.mp4 | jq '.transcript'

# List available models
voxtus --list-models
```

### Options

```
Arguments:
  <INPUT>  YouTube URL or local media file path

Options:
  -f, --format <FORMAT>    Output format(s), comma-separated: txt,json,srt,vtt [default: txt]
  -n, --name <NAME>        Base name for output files (no extension)
  -o, --output <DIR>       Output directory [default: current directory]
  -v, --verbose            Increase verbosity (-v, -vv for debug)
  -k, --keep               Keep the downloaded/converted audio file
      --model <MODEL>      Whisper model to use [default: small]
      --list-models        List available models and exit
      --overwrite          Overwrite existing files without confirmation
      --stdout             Output to stdout only (single format, no files created)
  -h, --help               Show help
  -V, --version            Show version
```

## Output Formats

### TXT
Plain text with timestamps:
```
[0.00 - 5.20]: Welcome to our podcast.
[5.20 - 10.50]: Today we're discussing Rust.
```

### JSON
Structured data with metadata:
```json
{
  "transcript": [
    {"id": 1, "start": 0.0, "end": 5.2, "text": "Welcome to our podcast."}
  ],
  "metadata": {
    "title": "Episode 42",
    "source": "https://youtube.com/watch?v=...",
    "duration": 1523.5,
    "model": "small",
    "language": "en"
  }
}
```

### SRT
SubRip subtitle format:
```
1
00:00:00,000 --> 00:00:05,200
Welcome to our podcast.
```

### VTT
WebVTT format with metadata:
```
WEBVTT

NOTE Title
Episode 42

00:00:00.000 --> 00:00:05.200
Welcome to our podcast.
```

## Whisper Models

| Model | Parameters | VRAM | Speed | Accuracy |
|-------|------------|------|-------|----------|
| tiny | 39M | ~1GB | Fastest | Lower |
| base | 74M | ~1GB | Fast | Basic |
| small | 244M | ~2GB | Moderate | Good |
| medium | 769M | ~5GB | Slow | Better |
| large-v3 | 1550M | ~10GB | Slowest | Best |

English-only variants (`.en` suffix) are faster for English content.

Models are automatically downloaded on first use to `~/.local/share/voxtus/models/`.

## License

This project is licensed under the [GNU Affero General Public License v3.0 or later](LICENSE) (AGPL-3.0-or-later).

## Acknowledgments

- [whisper.cpp](https://github.com/ggerganov/whisper.cpp) - C/C++ port of OpenAI's Whisper
- [whisper-rs](https://github.com/tazz4843/whisper-rs) - Rust bindings for whisper.cpp
- [yt-dlp](https://github.com/yt-dlp/yt-dlp) - YouTube downloader
