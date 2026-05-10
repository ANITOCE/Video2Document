use std::path::PathBuf;

use chrono::Utc;
use tempfile::TempDir;

use video2document::models::{ConceptItem, CourseDocument, SourceVideo, TaskState, VideoMetadata};
use video2document::state_store::StateStore;
use video2document::writer::render_course_document;

#[test]
fn state_store_round_trips_metadata_status_and_document() {
    let temp = TempDir::new().expect("failed to create temp dir");
    let store = StateStore::new(temp.path().join("Working"));
    let video = SourceVideo {
        source_path: PathBuf::from("Video/CourseA/01_绪论.mp4"),
        relative_path: PathBuf::from("CourseA/01_绪论.mp4"),
        stem_path: PathBuf::from("CourseA/01_绪论"),
        output_markdown_path: PathBuf::from("Document/CourseA/01_绪论.md"),
        working_path: PathBuf::from("Working/CourseA/01_绪论"),
        file_name: "01_绪论.mp4".to_string(),
        extension: "mp4".to_string(),
        size_bytes: 10,
        source_mtime_ms: 1,
        duration_seconds: Some(12.0),
        width: Some(1280),
        height: Some(720),
    };

    store
        .update_state(
            &video,
            "fingerprint",
            TaskState::MetadataDone,
            1,
            Vec::new(),
        )
        .expect("failed to save status");
    store
        .save_video_metadata(
            &video,
            "fingerprint",
            &VideoMetadata {
                duration_seconds: 12.0,
                width: 1280,
                height: 720,
            },
        )
        .expect("failed to save metadata");

    let mut document = CourseDocument {
        relative_path: PathBuf::from("CourseA/01_绪论.md"),
        document_path: PathBuf::from("Document/CourseA/01_绪论.md"),
        title: "01_绪论".to_string(),
        document_markdown: String::new(),
        directory_summary: "摘要".to_string(),
        keywords: vec!["语法".to_string()],
        review_flags: Vec::new(),
        generated_at: Utc::now(),
        objectives: vec!["理解目标".to_string()],
        knowledge_mainline: vec!["知识主线".to_string()],
        core_concepts: vec![ConceptItem {
            term: "概念".to_string(),
            note: "说明".to_string(),
        }],
        formulas: Vec::new(),
        examples: Vec::new(),
        teacher_emphasis: Vec::new(),
        glossary: Vec::new(),
        questions_and_quiz: vec!["问题".to_string()],
    };
    document.document_markdown = render_course_document(&document);

    store
        .save_document_bundle(&video, "fingerprint", &document)
        .expect("failed to save document");

    assert!(
        store
            .load_status(&video)
            .expect("failed to load status")
            .is_some()
    );
    assert!(
        store
            .load_video_metadata(&video, "fingerprint")
            .expect("failed to load metadata")
            .is_some()
    );
    let loaded = store
        .load_document_bundle(&video, "fingerprint")
        .expect("failed to load document")
        .expect("missing stored document");
    assert!(loaded.has_required_sections());
}
