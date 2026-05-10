use std::path::PathBuf;

use chrono::Utc;

use video2document::indexer::build_indexes;
use video2document::models::CourseDocument;

#[test]
fn index_lists_only_direct_children_and_metadata() {
    let now = Utc::now();
    let documents = vec![
        CourseDocument {
            relative_path: PathBuf::from("CourseA/01_绪论.md"),
            document_path: PathBuf::from("Document/CourseA/01_绪论.md"),
            title: "01_绪论".to_string(),
            document_markdown: String::new(),
            directory_summary: "课程A摘要".to_string(),
            keywords: vec!["A".to_string()],
            review_flags: Vec::new(),
            generated_at: now,
            objectives: Vec::new(),
            knowledge_mainline: Vec::new(),
            core_concepts: Vec::new(),
            formulas: Vec::new(),
            examples: Vec::new(),
            teacher_emphasis: Vec::new(),
            glossary: Vec::new(),
            questions_and_quiz: Vec::new(),
        },
        CourseDocument {
            relative_path: PathBuf::from("CourseA/Sub/02_进阶.md"),
            document_path: PathBuf::from("Document/CourseA/Sub/02_进阶.md"),
            title: "02_进阶".to_string(),
            document_markdown: String::new(),
            directory_summary: "课程A子目录摘要".to_string(),
            keywords: vec!["Sub".to_string()],
            review_flags: Vec::new(),
            generated_at: now,
            objectives: Vec::new(),
            knowledge_mainline: Vec::new(),
            core_concepts: Vec::new(),
            formulas: Vec::new(),
            examples: Vec::new(),
            teacher_emphasis: Vec::new(),
            glossary: Vec::new(),
            questions_and_quiz: Vec::new(),
        },
    ];

    let indexes = build_indexes(PathBuf::from("Document").as_path(), &documents);
    let root = indexes
        .iter()
        .find(|index| index.directory_path.as_os_str().is_empty())
        .expect("missing root index");
    let course_a = indexes
        .iter()
        .find(|index| index.directory_path == std::path::Path::new("CourseA"))
        .expect("missing CourseA index");

    assert_eq!(root.child_directories.len(), 1);
    assert_eq!(root.child_documents.len(), 0);
    assert_eq!(root.document_count, 2);
    assert_eq!(course_a.child_documents.len(), 1);
    assert_eq!(course_a.child_directories.len(), 1);
    assert_eq!(course_a.child_documents[0].link, "01_绪论.md");
}
