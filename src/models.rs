use std::path::PathBuf;

use chrono::{DateTime, Utc};
use clap::ValueEnum;
use serde::{Deserialize, Serialize};

pub const REQUIRED_SECTION_TITLES: [&str; 8] = [
    "本讲目标",
    "知识主线",
    "核心概念",
    "公式与结论",
    "例题与案例",
    "老师强调内容",
    "术语与概念简记",
    "问答与自测",
];

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, ValueEnum, Default)]
#[serde(rename_all = "lowercase")]
pub enum UploadMode {
    #[default]
    Auto,
    Base64,
    File,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SourceVideo {
    pub source_path: PathBuf,
    pub relative_path: PathBuf,
    pub stem_path: PathBuf,
    pub output_markdown_path: PathBuf,
    pub working_path: PathBuf,
    pub file_name: String,
    pub extension: String,
    pub size_bytes: u64,
    pub source_mtime_ms: u128,
    pub duration_seconds: Option<f64>,
    pub width: Option<u32>,
    pub height: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct VideoMetadata {
    pub duration_seconds: f64,
    pub width: u32,
    pub height: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct VideoSegment {
    pub segment_id: String,
    pub source_relative_path: PathBuf,
    pub clip_path: PathBuf,
    pub start_seconds: u64,
    pub end_seconds: u64,
    pub resized: bool,
    pub upload_mode: UploadMode,
    pub uploaded_file_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ConceptItem {
    pub term: String,
    pub note: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct FormulaItem {
    pub expression: String,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ExampleItem {
    pub title: String,
    pub summary: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SegmentAnalysis {
    pub course: String,
    pub relative_path: PathBuf,
    pub segment_id: String,
    pub time_range: String,
    pub topic: String,
    pub summary: String,
    pub key_concepts: Vec<ConceptItem>,
    pub formulas: Vec<FormulaItem>,
    pub teacher_emphasis: Vec<String>,
    pub examples: Vec<ExampleItem>,
    pub confusions: Vec<String>,
    pub questions: Vec<String>,
    pub tags: Vec<String>,
    pub low_confidence: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CourseDocument {
    pub relative_path: PathBuf,
    pub document_path: PathBuf,
    pub title: String,
    pub document_markdown: String,
    pub directory_summary: String,
    pub keywords: Vec<String>,
    pub review_flags: Vec<String>,
    pub generated_at: DateTime<Utc>,
    pub objectives: Vec<String>,
    pub knowledge_mainline: Vec<String>,
    pub core_concepts: Vec<ConceptItem>,
    pub formulas: Vec<FormulaItem>,
    pub examples: Vec<ExampleItem>,
    pub teacher_emphasis: Vec<String>,
    pub glossary: Vec<ConceptItem>,
    pub questions_and_quiz: Vec<String>,
}

impl CourseDocument {
    pub fn has_required_sections(&self) -> bool {
        REQUIRED_SECTION_TITLES
            .iter()
            .all(|title| self.document_markdown.contains(&format!("## {title}")))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DirectoryDocumentEntry {
    pub title: String,
    pub link: String,
    pub relative_path: PathBuf,
    pub generated_at: DateTime<Utc>,
    pub summary: String,
    pub keywords: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DirectoryChildEntry {
    pub name: String,
    pub link: String,
    pub relative_path: PathBuf,
    pub document_count: usize,
    pub latest_generated_at: Option<DateTime<Utc>>,
    pub summary: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DirectoryIndex {
    pub directory_path: PathBuf,
    pub index_path: PathBuf,
    pub child_directories: Vec<DirectoryChildEntry>,
    pub child_documents: Vec<DirectoryDocumentEntry>,
    pub document_count: usize,
    pub latest_generated_at: Option<DateTime<Utc>>,
    pub summary: String,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "snake_case")]
pub enum TaskState {
    Pending,
    Scanning,
    MetadataDone,
    Clipped,
    KimiSegmentDone,
    KimiDocumentDone,
    DocumentWritten,
    Indexed,
    Reviewed,
    Failed,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct VideoTaskState {
    pub relative_path: PathBuf,
    pub source_mtime_ms: u128,
    pub config_fingerprint: String,
    pub state: TaskState,
    pub segment_count: usize,
    pub updated_at: DateTime<Utc>,
    pub errors: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct RunSummary {
    pub discovered: usize,
    pub processed: usize,
    pub reused: usize,
    pub failed: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ExitReport {
    pub code: i32,
    pub summary: RunSummary,
}
