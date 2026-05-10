use std::path::PathBuf;

use chrono::Utc;

use video2document::models::{ConceptItem, CourseDocument};
use video2document::writer::{render_course_document, validate_section_order};

#[test]
fn rendered_markdown_preserves_required_section_order() {
    let mut document = CourseDocument {
        relative_path: PathBuf::from("CourseA/01_绪论.md"),
        document_path: PathBuf::from("Document/CourseA/01_绪论.md"),
        title: "01_绪论".to_string(),
        document_markdown: String::new(),
        directory_summary: "摘要".to_string(),
        keywords: vec!["语法".to_string()],
        review_flags: Vec::new(),
        generated_at: Utc::now(),
        objectives: vec!["理解课程目标".to_string()],
        knowledge_mainline: vec!["框架 -> 细节".to_string()],
        core_concepts: vec![ConceptItem {
            term: "句子成分".to_string(),
            note: "句子的基础结构".to_string(),
        }],
        formulas: Vec::new(),
        examples: Vec::new(),
        teacher_emphasis: vec!["先学框架".to_string()],
        glossary: vec![ConceptItem {
            term: "时态".to_string(),
            note: "表示时间关系".to_string(),
        }],
        questions_and_quiz: vec!["什么是句子成分？".to_string()],
    };

    document.document_markdown = render_course_document(&document);
    assert!(validate_section_order(&document.document_markdown));
    assert!(document.document_markdown.contains("## 问答与自测"));
    assert!(
        document
            .document_markdown
            .ends_with("1. 什么是句子成分？\n\n")
    );
}

#[test]
fn output_mapping_replaces_video_extension_only() {
    let input = PathBuf::from("Video/CourseA/01_绪论.mp4");
    let output = PathBuf::from("Document")
        .join(input.strip_prefix("Video").expect("missing prefix"))
        .with_extension("md");
    assert_eq!(output, PathBuf::from("Document/CourseA/01_绪论.md"));
}
