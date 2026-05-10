use std::fs;
use std::path::Path;

use anyhow::{Context, Result};

use crate::models::{CourseDocument, DirectoryIndex, REQUIRED_SECTION_TITLES};

pub fn render_course_document(document: &CourseDocument) -> String {
    let mut markdown = String::new();
    markdown.push_str(&format!("# {}\n\n", document.title));
    markdown.push_str(&render_section(
        "本讲目标",
        &render_list(&document.objectives, "暂无明确提炼。"),
    ));
    markdown.push_str(&render_section(
        "知识主线",
        &render_list(&document.knowledge_mainline, "暂无明确提炼。"),
    ));
    markdown.push_str(&render_section(
        "核心概念",
        &render_concepts(&document.core_concepts, "暂无明确提炼。"),
    ));
    markdown.push_str(&render_section(
        "公式与结论",
        &render_formulas(&document.formulas, "暂无明确提炼。"),
    ));
    markdown.push_str(&render_section(
        "例题与案例",
        &render_examples(&document.examples, "暂无明确提炼。"),
    ));
    markdown.push_str(&render_section(
        "老师强调内容",
        &render_list(&document.teacher_emphasis, "暂无明确提炼。"),
    ));
    markdown.push_str(&render_section(
        "术语与概念简记",
        &render_concepts(&document.glossary, "暂无明确提炼。"),
    ));
    markdown.push_str(&render_section(
        "问答与自测",
        &render_numbered_list(&document.questions_and_quiz, "暂无明确提炼。"),
    ));
    markdown
}

pub fn validate_section_order(markdown: &str) -> bool {
    let mut cursor = 0usize;
    for title in REQUIRED_SECTION_TITLES {
        let needle = format!("## {title}");
        let Some(next) = markdown[cursor..].find(&needle) else {
            return false;
        };
        cursor += next + needle.len();
    }
    true
}

pub fn write_markdown(path: &Path, markdown: &str) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("failed to create markdown dir {}", parent.display()))?;
    }
    fs::write(path, markdown).with_context(|| format!("failed to write {}", path.display()))?;
    Ok(())
}

pub fn render_index(index: &DirectoryIndex) -> String {
    let mut markdown = String::new();
    markdown.push_str("# 目录索引\n\n");
    markdown.push_str(&format!("- 文档总数: {}\n", index.document_count));
    if let Some(latest) = index.latest_generated_at {
        markdown.push_str(&format!("- 最近生成: {}\n", latest.to_rfc3339()));
    }
    if !index.summary.is_empty() {
        markdown.push_str(&format!("- 摘要: {}\n", index.summary));
    }
    markdown.push('\n');

    markdown.push_str("## 子目录\n\n");
    if index.child_directories.is_empty() {
        markdown.push_str("- 当前目录下无子目录。\n\n");
    } else {
        for entry in &index.child_directories {
            markdown.push_str(&format!(
                "- [{}]({}) | 文档数: {}{}{}\n",
                entry.name,
                entry.link,
                entry.document_count,
                entry
                    .latest_generated_at
                    .map(|value| format!(" | 最近生成: {}", value.to_rfc3339()))
                    .unwrap_or_default(),
                if entry.summary.is_empty() {
                    String::new()
                } else {
                    format!(" | 摘要: {}", entry.summary)
                }
            ));
        }
        markdown.push('\n');
    }

    markdown.push_str("## 课程文档\n\n");
    if index.child_documents.is_empty() {
        markdown.push_str("- 当前目录下无课程文档。\n");
    } else {
        for document in &index.child_documents {
            markdown.push_str(&format!(
                "- [{}]({}) | 最近生成: {}{}\n",
                document.title,
                document.link,
                document.generated_at.to_rfc3339(),
                if document.summary.is_empty() {
                    String::new()
                } else {
                    format!(" | 摘要: {}", document.summary)
                }
            ));
        }
    }

    markdown
}

fn render_section(title: &str, body: &str) -> String {
    format!("## {title}\n\n{body}\n\n")
}

fn render_list(items: &[String], empty_text: &str) -> String {
    if items.is_empty() {
        return format!("- {empty_text}");
    }
    items
        .iter()
        .map(|item| format!("- {item}"))
        .collect::<Vec<_>>()
        .join("\n")
}

fn render_numbered_list(items: &[String], empty_text: &str) -> String {
    if items.is_empty() {
        return format!("1. {empty_text}");
    }
    items
        .iter()
        .enumerate()
        .map(|(index, item)| format!("{}. {}", index + 1, item))
        .collect::<Vec<_>>()
        .join("\n")
}

fn render_concepts(items: &[crate::models::ConceptItem], empty_text: &str) -> String {
    if items.is_empty() {
        return format!("- {empty_text}");
    }
    items
        .iter()
        .map(|item| format!("- {}: {}", item.term, item.note))
        .collect::<Vec<_>>()
        .join("\n")
}

fn render_formulas(items: &[crate::models::FormulaItem], empty_text: &str) -> String {
    if items.is_empty() {
        return format!("- {empty_text}");
    }
    items
        .iter()
        .map(|item| format!("- {}: {}", item.expression, item.description))
        .collect::<Vec<_>>()
        .join("\n")
}

fn render_examples(items: &[crate::models::ExampleItem], empty_text: &str) -> String {
    if items.is_empty() {
        return format!("- {empty_text}");
    }
    items
        .iter()
        .map(|item| format!("- {}: {}", item.title, item.summary))
        .collect::<Vec<_>>()
        .join("\n")
}
