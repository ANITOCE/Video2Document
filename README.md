# Video2Document

[English](README.md) | [简体中文](README.zh-CN.md)

Video2Document is a Rust CLI that scans a local video library, sends video segments to Kimi for analysis, and generates one Markdown study note per video plus directory indexes for navigation.

## What It Does

- Recursively scans the input directory for supported video files
- Skips hidden files, temporary files, and non-video files
- Generates exactly one Markdown document for each video using the original video name
- Keeps a stable section order in every generated document
- Writes resumable state and cache data to a separate working directory
- Builds index.md files in the output tree for easier browsing

Supported video extensions:

- .mp4
- .mov
- .mkv
- .avi
- .webm
- .m4v

Every generated course document uses the same section order:

1. Objectives
2. Knowledge Flow
3. Core Concepts
4. Formulas and Conclusions
5. Examples and Cases
6. Teacher Emphasis
7. Glossary Notes
8. Q&A and Self-Test

## Requirements

Make sure the following are available before running the project:

- Rust stable toolchain
- ffmpeg on PATH
- ffprobe on PATH
- A valid MOONSHOT_API_KEY in the current shell

PowerShell example:

```powershell
$env:MOONSHOT_API_KEY = "your-kimi-api-key"
```

Do not commit the API key or place it in any tracked file.

## Run

### Run with the default folders

If the repository already contains a Video directory, run:

```powershell
cargo run --release --
```

Default folders:

- Input: ./Video
- Output: ./Document
- Working cache: ./Working

### Run with explicit paths

```powershell
cargo run --release -- --input-dir "./Video" --output-dir "./Document" --working-dir "./Working"
```

### Run a small smoke test

For a small sample set or a single short video:

```powershell
cargo run --release -- --input-dir "./VideoSmoke" --output-dir "./DocumentSmoke" --working-dir "./WorkingSmoke" --segment-seconds 1800 --upload-mode base64
```

### Rebuild indexes only

If Markdown files already exist and you only want to rebuild indexes:

```powershell
cargo run --release -- --output-dir "./Document" --working-dir "./Working" --only-index
```

## Useful CLI Options

| Option | Description |
| --- | --- |
| --input-dir | Override the input video directory |
| --output-dir | Override the Markdown output directory |
| --working-dir | Override the working cache directory |
| --segment-seconds | Override video segment length |
| --retry | Override retry count for recoverable failures |
| --upload-mode | Choose auto, base64, or file |
| --only-index | Rebuild indexes without reprocessing videos |
| --force | Ignore reusable cache and process again |

## Output Layout

Example output structure:

```text
Document/
  index.md
  CourseA/
    index.md
    01_Intro.md

Working/
  CourseA/
    01_Intro/
      status.json
      metadata.json
      clips/
      kimi_segment_outputs/
      kimi_final_outputs/
```

## Resume Behavior

If a run is interrupted, run the same command again.

- Completed videos are reused when the source and processing options have not changed
- Failed videos keep their state and error information in the working directory
- Use --force if you want to reprocess targets from scratch

## License

See [LICENSE](LICENSE).
