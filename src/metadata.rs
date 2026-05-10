use std::path::Path;

use anyhow::{Context, Result, anyhow};
use serde::Deserialize;
use tokio::process::Command;

use crate::config::VideoSettings;
use crate::models::{SourceVideo, VideoMetadata, VideoSegment};

#[derive(Debug, Deserialize)]
struct FfprobeResponse {
    #[serde(default)]
    streams: Vec<FfprobeStream>,
    format: FfprobeFormat,
}

#[derive(Debug, Deserialize)]
struct FfprobeStream {
    width: Option<u32>,
    height: Option<u32>,
}

#[derive(Debug, Deserialize)]
struct FfprobeFormat {
    duration: Option<String>,
}

pub async fn probe_video(path: &Path) -> Result<VideoMetadata> {
    let ffprobe_bin =
        std::env::var("VIDEO2DOCUMENT_FFPROBE_BIN").unwrap_or_else(|_| "ffprobe".to_string());
    let output = Command::new(ffprobe_bin)
        .arg("-v")
        .arg("error")
        .arg("-show_entries")
        .arg("format=duration:stream=width,height")
        .arg("-of")
        .arg("json")
        .arg(path)
        .output()
        .await
        .with_context(|| format!("failed to invoke ffprobe for {}", path.display()))?;

    if !output.status.success() {
        return Err(anyhow!(
            "ffprobe failed for {}: {}",
            path.display(),
            String::from_utf8_lossy(&output.stderr)
        ));
    }

    let parsed: FfprobeResponse = serde_json::from_slice(&output.stdout)
        .with_context(|| format!("failed to parse ffprobe output for {}", path.display()))?;
    let width = parsed
        .streams
        .iter()
        .find_map(|stream| stream.width)
        .unwrap_or(0);
    let height = parsed
        .streams
        .iter()
        .find_map(|stream| stream.height)
        .unwrap_or(0);
    let duration_seconds = parsed
        .format
        .duration
        .as_deref()
        .unwrap_or("0")
        .parse::<f64>()
        .unwrap_or(0.0);

    Ok(VideoMetadata {
        duration_seconds,
        width,
        height,
    })
}

pub fn build_segments(
    video: &SourceVideo,
    metadata: &VideoMetadata,
    settings: &VideoSettings,
) -> Result<Vec<VideoSegment>> {
    let duration = metadata.duration_seconds.ceil().max(1.0) as u64;
    let total_segments = duration.div_ceil(settings.segment_seconds).max(1);
    let resized = metadata.width > settings.max_resolution_width
        || metadata.height > settings.max_resolution_height;

    let mut segments = Vec::with_capacity(total_segments as usize);

    for index in 0..total_segments {
        let start_seconds = index * settings.segment_seconds;
        let end_seconds = ((index + 1) * settings.segment_seconds).min(duration);
        segments.push(VideoSegment {
            segment_id: format!("clip_{:03}", index + 1),
            source_relative_path: video.relative_path.clone(),
            clip_path: video
                .working_path
                .join("clips")
                .join(format!("clip_{:03}.mp4", index + 1)),
            start_seconds,
            end_seconds: end_seconds.max(start_seconds + 1),
            resized,
            upload_mode: settings.preferred_upload_mode,
            uploaded_file_id: None,
        });
    }

    Ok(segments)
}

pub fn format_time_range(segment: &VideoSegment) -> String {
    format!(
        "{}-{}",
        format_hms(segment.start_seconds),
        format_hms(segment.end_seconds)
    )
}

fn format_hms(total_seconds: u64) -> String {
    let hours = total_seconds / 3600;
    let minutes = (total_seconds % 3600) / 60;
    let seconds = total_seconds % 60;
    format!("{hours:02}:{minutes:02}:{seconds:02}")
}
