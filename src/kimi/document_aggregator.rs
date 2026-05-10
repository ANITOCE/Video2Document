use std::collections::HashSet;

use chrono::Utc;

use crate::models::{ConceptItem, CourseDocument, FormulaItem, SegmentAnalysis, SourceVideo};

pub fn aggregate_document(video: &SourceVideo, analyses: &[SegmentAnalysis]) -> CourseDocument {
    let title = video
        .stem_path
        .file_name()
        .and_then(|value| value.to_str())
        .unwrap_or("课程文档")
        .to_string();

    let objectives = unique_strings(
        analyses
            .iter()
            .map(|analysis| format!("理解 {}", analysis.topic.trim()))
            .collect(),
    );
    let knowledge_mainline = unique_strings(
        analyses
            .iter()
            .map(|analysis| format!("{}: {}", analysis.time_range, analysis.summary.trim()))
            .collect(),
    );
    let core_concepts = unique_concepts(
        analyses
            .iter()
            .flat_map(|analysis| analysis.key_concepts.clone())
            .collect(),
    );
    let formulas = unique_formulas(
        analyses
            .iter()
            .flat_map(|analysis| analysis.formulas.clone())
            .collect(),
    );
    let examples = unique_examples(
        analyses
            .iter()
            .flat_map(|analysis| analysis.examples.clone())
            .collect(),
    );
    let teacher_emphasis = unique_strings(
        analyses
            .iter()
            .flat_map(|analysis| analysis.teacher_emphasis.clone())
            .collect(),
    );
    let glossary = core_concepts.iter().take(8).cloned().collect::<Vec<_>>();
    let questions_and_quiz = unique_strings(
        analyses
            .iter()
            .flat_map(|analysis| {
                let mut values = analysis.questions.clone();
                values.extend(
                    analysis
                        .confusions
                        .iter()
                        .map(|item| format!("待核对：{}", item))
                        .collect::<Vec<_>>(),
                );
                values
            })
            .collect(),
    );
    let keywords = unique_strings(
        analyses
            .iter()
            .flat_map(|analysis| analysis.tags.clone())
            .chain(core_concepts.iter().map(|item| item.term.clone()))
            .take(12)
            .collect(),
    );
    let review_flags = unique_strings(
        analyses
            .iter()
            .flat_map(|analysis| analysis.low_confidence.clone())
            .collect(),
    );
    let directory_summary = knowledge_mainline
        .first()
        .cloned()
        .unwrap_or_else(|| "暂无摘要".to_string());

    CourseDocument {
        relative_path: video.relative_path.with_extension("md"),
        document_path: video.output_markdown_path.clone(),
        title,
        document_markdown: String::new(),
        directory_summary,
        keywords,
        review_flags,
        generated_at: Utc::now(),
        objectives,
        knowledge_mainline,
        core_concepts,
        formulas,
        examples,
        teacher_emphasis,
        glossary,
        questions_and_quiz,
    }
}

fn unique_strings(items: Vec<String>) -> Vec<String> {
    let mut seen = HashSet::new();
    let mut result = Vec::new();
    for item in items {
        let normalized = item.trim().to_string();
        if normalized.is_empty() {
            continue;
        }
        if seen.insert(normalized.clone()) {
            result.push(normalized);
        }
    }
    result
}

fn unique_concepts(items: Vec<ConceptItem>) -> Vec<ConceptItem> {
    let mut seen = HashSet::new();
    let mut result = Vec::new();
    for item in items {
        let key = format!("{}::{}", item.term.trim(), item.note.trim());
        if item.term.trim().is_empty() || !seen.insert(key) {
            continue;
        }
        result.push(ConceptItem {
            term: item.term.trim().to_string(),
            note: item.note.trim().to_string(),
        });
    }
    result
}

fn unique_formulas(items: Vec<FormulaItem>) -> Vec<FormulaItem> {
    let mut seen = HashSet::new();
    let mut result = Vec::new();
    for item in items {
        let key = format!("{}::{}", item.expression.trim(), item.description.trim());
        if item.expression.trim().is_empty() || !seen.insert(key) {
            continue;
        }
        result.push(FormulaItem {
            expression: item.expression.trim().to_string(),
            description: item.description.trim().to_string(),
        });
    }
    result
}

fn unique_examples(items: Vec<crate::models::ExampleItem>) -> Vec<crate::models::ExampleItem> {
    let mut seen = HashSet::new();
    let mut result = Vec::new();
    for item in items {
        let key = format!("{}::{}", item.title.trim(), item.summary.trim());
        if item.title.trim().is_empty() || !seen.insert(key) {
            continue;
        }
        result.push(crate::models::ExampleItem {
            title: item.title.trim().to_string(),
            summary: item.summary.trim().to_string(),
        });
    }
    result
}
