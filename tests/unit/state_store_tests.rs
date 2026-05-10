use std::path::PathBuf;

use tempfile::TempDir;

use video2document::models::{SourceVideo, TaskState};
use video2document::state_store::StateStore;

#[test]
fn cache_validation_respects_mtime_and_fingerprint() {
    let temp = TempDir::new().expect("failed to create temp dir");
    let store = StateStore::new(temp.path().join("Working"));
    let video = SourceVideo {
        source_path: PathBuf::from("Video/CourseA/01_绪论.mp4"),
        relative_path: PathBuf::from("CourseA/01_绪论.mp4"),
        stem_path: PathBuf::from("CourseA/01_绪论"),
        output_markdown_path: temp.path().join("Document/CourseA/01_绪论.md"),
        working_path: PathBuf::from("Working/CourseA/01_绪论"),
        file_name: "01_绪论.mp4".to_string(),
        extension: "mp4".to_string(),
        size_bytes: 10,
        source_mtime_ms: 42,
        duration_seconds: None,
        width: None,
        height: None,
    };
    std::fs::create_dir_all(video.output_markdown_path.parent().expect("missing parent"))
        .expect("failed to create doc dir");
    std::fs::write(&video.output_markdown_path, "# doc").expect("failed to write doc");

    store
        .update_state(
            &video,
            "fingerprint-a",
            TaskState::DocumentWritten,
            1,
            Vec::new(),
        )
        .expect("failed to save status");

    assert!(
        !store
            .is_video_complete_and_valid(&video, "fingerprint-a")
            .expect("failed to validate cache")
    );
    assert!(
        !store
            .is_video_complete_and_valid(&video, "fingerprint-b")
            .expect("failed to validate mismatched fingerprint")
    );
}
