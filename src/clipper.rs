use std::path::Path;

use anyhow::{Context, Result, anyhow};
use tokio::process::Command;

use crate::config::VideoSettings;
use crate::models::{VideoMetadata, VideoSegment};

pub async fn create_clip(
    source_path: &Path,
    segment: &VideoSegment,
    metadata: &VideoMetadata,
    settings: &VideoSettings,
) -> Result<()> {
    if segment.clip_path.exists() && std::fs::metadata(&segment.clip_path)?.len() > 0 {
        return Ok(());
    }

    if let Some(parent) = segment.clip_path.parent() {
        std::fs::create_dir_all(parent)
            .with_context(|| format!("failed to create clip dir {}", parent.display()))?;
    }

    let ffmpeg_bin =
        std::env::var("VIDEO2DOCUMENT_FFMPEG_BIN").unwrap_or_else(|_| "ffmpeg".to_string());
    let mut command = Command::new(ffmpeg_bin);
    command
        .arg("-y")
        .arg("-ss")
        .arg(segment.start_seconds.to_string())
        .arg("-i")
        .arg(source_path)
        .arg("-t")
        .arg((segment.end_seconds - segment.start_seconds).to_string());

    if metadata.width > settings.max_resolution_width
        || metadata.height > settings.max_resolution_height
    {
        command.arg("-vf").arg(format!(
            "scale='min({},iw)':'min({},ih)':force_original_aspect_ratio=decrease",
            settings.max_resolution_width, settings.max_resolution_height
        ));
    }

    command
        .arg("-c:v")
        .arg("libx264")
        .arg("-preset")
        .arg("veryfast")
        .arg("-crf")
        .arg("28")
        .arg("-pix_fmt")
        .arg("yuv420p")
        .arg("-c:a")
        .arg("aac")
        .arg("-movflags")
        .arg("+faststart")
        .arg(&segment.clip_path);

    let output = command
        .output()
        .await
        .with_context(|| format!("failed to invoke ffmpeg for {}", source_path.display()))?;

    if !output.status.success() {
        return Err(anyhow!(
            "ffmpeg failed for {}: {}",
            source_path.display(),
            String::from_utf8_lossy(&output.stderr)
        ));
    }

    Ok(())
}
