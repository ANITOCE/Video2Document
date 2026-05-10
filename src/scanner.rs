use std::collections::HashMap;
use std::fs;
use std::path::{Component, Path, PathBuf};
use std::time::UNIX_EPOCH;

use anyhow::{Context, Result, bail};
use walkdir::WalkDir;

use crate::config::{AppSettings, PathSettings};
use crate::models::{CourseDocument, SourceVideo};

const SUPPORTED_EXTENSIONS: &[&str] = &["mp4", "mov", "mkv", "avi", "webm", "m4v"];

pub fn scan_videos(settings: &AppSettings) -> Result<Vec<SourceVideo>> {
    let input_root = settings.paths.input_dir.canonicalize().with_context(|| {
        format!(
            "failed to canonicalize input dir {}",
            settings.paths.input_dir.display()
        )
    })?;

    let mut videos = Vec::new();
    let mut collision_map: HashMap<String, PathBuf> = HashMap::new();

    for entry in WalkDir::new(&input_root).into_iter().filter_map(Result::ok) {
        if !entry.file_type().is_file() {
            continue;
        }

        let path = entry.path();
        if is_hidden_or_temporary(path, &input_root) || !is_supported_video(path) {
            continue;
        }

        let metadata = fs::metadata(path)
            .with_context(|| format!("failed to read metadata for {}", path.display()))?;
        let relative_path = path.strip_prefix(&input_root)?.to_path_buf();
        let stem_path = relative_path.with_extension("");
        let output_markdown_path = settings
            .paths
            .output_dir
            .join(&relative_path)
            .with_extension("md");
        let collision_key = output_markdown_path.to_string_lossy().to_lowercase();

        if let Some(previous) = collision_map.insert(collision_key, relative_path.clone()) {
            bail!(
                "output collision detected between {} and {}",
                previous.display(),
                relative_path.display()
            );
        }

        videos.push(SourceVideo {
            source_path: path.to_path_buf(),
            relative_path: relative_path.clone(),
            stem_path: stem_path.clone(),
            output_markdown_path,
            working_path: settings.paths.working_dir.join(&stem_path),
            file_name: path
                .file_name()
                .map(|name| name.to_string_lossy().to_string())
                .unwrap_or_default(),
            extension: path
                .extension()
                .map(|ext| ext.to_string_lossy().to_ascii_lowercase())
                .unwrap_or_default(),
            size_bytes: metadata.len(),
            source_mtime_ms: metadata
                .modified()
                .ok()
                .and_then(|value| value.duration_since(UNIX_EPOCH).ok())
                .map(|value| value.as_millis())
                .unwrap_or_default(),
            duration_seconds: None,
            width: None,
            height: None,
        });
    }

    videos.sort_by(|left, right| left.relative_path.cmp(&right.relative_path));
    Ok(videos)
}

pub fn map_document_to_video_roots(
    document: &CourseDocument,
    paths: &PathSettings,
) -> Option<SourceVideo> {
    let stem_path = document.relative_path.with_extension("");
    Some(SourceVideo {
        source_path: paths.input_dir.join(&document.relative_path),
        relative_path: document.relative_path.clone(),
        stem_path: stem_path.clone(),
        output_markdown_path: paths.output_dir.join(&document.relative_path),
        working_path: paths.working_dir.join(&stem_path),
        file_name: document
            .relative_path
            .file_name()
            .map(|name| name.to_string_lossy().to_string())
            .unwrap_or_default(),
        extension: String::new(),
        size_bytes: 0,
        source_mtime_ms: 0,
        duration_seconds: None,
        width: None,
        height: None,
    })
}

pub fn is_supported_video(path: &Path) -> bool {
    path.extension()
        .map(|ext| ext.to_string_lossy().to_ascii_lowercase())
        .map(|ext| SUPPORTED_EXTENSIONS.contains(&ext.as_str()))
        .unwrap_or(false)
}

pub fn is_hidden_or_temporary(path: &Path, input_root: &Path) -> bool {
    let relative = path.strip_prefix(input_root).unwrap_or(path);
    relative.components().any(|component| match component {
        Component::Normal(name) => {
            let text = name.to_string_lossy();
            text.starts_with('.')
                || text.starts_with("~$")
                || text.ends_with(".tmp")
                || text.ends_with(".part")
        }
        _ => false,
    })
}
