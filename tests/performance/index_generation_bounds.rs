use std::path::PathBuf;
use std::time::{Duration, Instant};

use chrono::Utc;

use video2document::indexer::build_indexes;
use video2document::models::CourseDocument;

#[test]
fn index_generation_scales_linearly_for_fixture_sized_inputs() {
    let mut documents = Vec::new();
    for course in 0..10 {
        for lecture in 0..30 {
            documents.push(CourseDocument {
                relative_path: PathBuf::from(format!("Course{course}/Lesson{lecture}.md")),
                document_path: PathBuf::from(format!("Document/Course{course}/Lesson{lecture}.md")),
                title: format!("Lesson{lecture}"),
                document_markdown: String::new(),
                directory_summary: format!("summary {course}-{lecture}"),
                keywords: Vec::new(),
                review_flags: Vec::new(),
                generated_at: Utc::now(),
                objectives: Vec::new(),
                knowledge_mainline: Vec::new(),
                core_concepts: Vec::new(),
                formulas: Vec::new(),
                examples: Vec::new(),
                teacher_emphasis: Vec::new(),
                glossary: Vec::new(),
                questions_and_quiz: Vec::new(),
            });
        }
    }

    let start = Instant::now();
    let indexes = build_indexes(PathBuf::from("Document").as_path(), &documents);
    assert!(start.elapsed() < Duration::from_secs(1));

    let root = indexes
        .iter()
        .find(|index| index.directory_path.as_os_str().is_empty())
        .expect("missing root index");
    assert_eq!(root.child_directories.len(), 10);
    assert_eq!(root.document_count, 300);
}
