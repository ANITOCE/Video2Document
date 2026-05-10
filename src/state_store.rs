use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use walkdir::WalkDir;

use crate::models::{
    CourseDocument, DirectoryIndex, SegmentAnalysis, SourceVideo, TaskState, VideoMetadata,
    VideoTaskState,
};

#[derive(Debug, Clone)]
pub struct StateStore {
    root: PathBuf,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct StoredMetadata {
    source_mtime_ms: u128,
    config_fingerprint: String,
    metadata: VideoMetadata,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct StoredDocument {
    source_mtime_ms: u128,
    config_fingerprint: String,
    document: CourseDocument,
}

impl StateStore {
    pub fn new(root: PathBuf) -> Self {
        Self { root }
    }

    pub fn is_video_complete_and_valid(
        &self,
        video: &SourceVideo,
        fingerprint: &str,
    ) -> Result<bool> {
        let Some(status) = self.load_status(video)? else {
            return Ok(false);
        };

        if status.source_mtime_ms != video.source_mtime_ms
            || status.config_fingerprint != fingerprint
        {
            return Ok(false);
        }

        if status.state < TaskState::DocumentWritten || !video.output_markdown_path.exists() {
            return Ok(false);
        }

        Ok(self.load_document_bundle(video, fingerprint)?.is_some())
    }

    pub fn update_state(
        &self,
        video: &SourceVideo,
        fingerprint: &str,
        state: TaskState,
        segment_count: usize,
        errors: Vec<String>,
    ) -> Result<()> {
        let status = VideoTaskState {
            relative_path: video.relative_path.clone(),
            source_mtime_ms: video.source_mtime_ms,
            config_fingerprint: fingerprint.to_string(),
            state,
            segment_count,
            updated_at: Utc::now(),
            errors,
        };
        self.save_status(video, &status)
    }

    pub fn record_failure(
        &self,
        video: &SourceVideo,
        fingerprint: &str,
        error_message: String,
    ) -> Result<()> {
        self.update_state(
            video,
            fingerprint,
            TaskState::Failed,
            0,
            vec![error_message],
        )
    }

    pub fn load_status(&self, video: &SourceVideo) -> Result<Option<VideoTaskState>> {
        self.read_json_if_exists(&self.status_path(video))
    }

    pub fn save_status(&self, video: &SourceVideo, status: &VideoTaskState) -> Result<()> {
        self.write_json(&self.status_path(video), status)
    }

    pub fn save_status_for_relative(&self, relative: &Path, status: &VideoTaskState) -> Result<()> {
        let path = self.root.join(relative).join("status.json");
        self.write_json(&path, status)
    }

    pub fn load_video_metadata(
        &self,
        video: &SourceVideo,
        fingerprint: &str,
    ) -> Result<Option<VideoMetadata>> {
        let Some(stored) =
            self.read_json_if_exists::<StoredMetadata>(&self.metadata_path(video))?
        else {
            return Ok(None);
        };
        if stored.source_mtime_ms == video.source_mtime_ms
            && stored.config_fingerprint == fingerprint
        {
            return Ok(Some(stored.metadata));
        }
        Ok(None)
    }

    pub fn save_video_metadata(
        &self,
        video: &SourceVideo,
        fingerprint: &str,
        metadata: &VideoMetadata,
    ) -> Result<()> {
        self.write_json(
            &self.metadata_path(video),
            &StoredMetadata {
                source_mtime_ms: video.source_mtime_ms,
                config_fingerprint: fingerprint.to_string(),
                metadata: metadata.clone(),
            },
        )
    }

    pub fn load_segment_analysis(
        &self,
        video: &SourceVideo,
        segment_id: &str,
        fingerprint: &str,
    ) -> Result<Option<SegmentAnalysis>> {
        let status = self.load_status(video)?;
        if status.as_ref().map(|status| {
            status.source_mtime_ms == video.source_mtime_ms
                && status.config_fingerprint == fingerprint
        }) != Some(true)
        {
            return Ok(None);
        }
        self.read_json_if_exists(&self.segment_analysis_path(video, segment_id))
    }

    pub fn save_segment_analysis(
        &self,
        video: &SourceVideo,
        segment_id: &str,
        analysis: &SegmentAnalysis,
    ) -> Result<()> {
        self.write_json(&self.segment_analysis_path(video, segment_id), analysis)
    }

    pub fn load_document_bundle(
        &self,
        video: &SourceVideo,
        fingerprint: &str,
    ) -> Result<Option<CourseDocument>> {
        let Some(stored) =
            self.read_json_if_exists::<StoredDocument>(&self.final_document_path(video))?
        else {
            return Ok(None);
        };
        if stored.source_mtime_ms == video.source_mtime_ms
            && stored.config_fingerprint == fingerprint
        {
            return Ok(Some(stored.document));
        }
        Ok(None)
    }

    pub fn save_document_bundle(
        &self,
        video: &SourceVideo,
        fingerprint: &str,
        document: &CourseDocument,
    ) -> Result<()> {
        self.write_json(
            &self.final_document_path(video),
            &StoredDocument {
                source_mtime_ms: video.source_mtime_ms,
                config_fingerprint: fingerprint.to_string(),
                document: document.clone(),
            },
        )
    }

    pub fn load_all_documents(&self) -> Result<Vec<CourseDocument>> {
        let mut documents = Vec::new();

        for entry in WalkDir::new(&self.root).into_iter().filter_map(Result::ok) {
            if !entry.file_type().is_file() || entry.file_name() != "document.json" {
                continue;
            }

            if let Ok(Some(stored)) = self.read_json_if_exists::<StoredDocument>(entry.path()) {
                documents.push(stored.document);
            }
        }

        documents.sort_by(|left, right| left.relative_path.cmp(&right.relative_path));
        Ok(documents)
    }

    pub fn save_directory_index_cache(&self, index: &DirectoryIndex) -> Result<()> {
        let relative = index
            .directory_path
            .strip_prefix("/")
            .unwrap_or(&index.directory_path);
        let path = if relative.as_os_str().is_empty() {
            self.root.join("index_cache.json")
        } else {
            self.root.join(relative).join("index_cache.json")
        };
        self.write_json(&path, index)
    }

    fn status_path(&self, video: &SourceVideo) -> PathBuf {
        self.root.join(&video.stem_path).join("status.json")
    }

    fn metadata_path(&self, video: &SourceVideo) -> PathBuf {
        self.root.join(&video.stem_path).join("metadata.json")
    }

    fn segment_analysis_path(&self, video: &SourceVideo, segment_id: &str) -> PathBuf {
        self.root
            .join(&video.stem_path)
            .join("kimi_segment_outputs")
            .join(format!("{segment_id}.json"))
    }

    fn final_document_path(&self, video: &SourceVideo) -> PathBuf {
        self.root
            .join(&video.stem_path)
            .join("kimi_final_outputs")
            .join("document.json")
    }

    fn write_json<T: Serialize>(&self, path: &Path, value: &T) -> Result<()> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .with_context(|| format!("failed to create state dir {}", parent.display()))?;
        }
        let serialized = serde_json::to_string_pretty(value)?;
        fs::write(path, serialized)
            .with_context(|| format!("failed to write {}", path.display()))?;
        Ok(())
    }

    fn read_json_if_exists<T: for<'de> Deserialize<'de>>(&self, path: &Path) -> Result<Option<T>> {
        if !path.exists() {
            return Ok(None);
        }
        let content = fs::read_to_string(path)
            .with_context(|| format!("failed to read {}", path.display()))?;
        let value = serde_json::from_str(&content)
            .with_context(|| format!("failed to parse {}", path.display()))?;
        Ok(Some(value))
    }
}
