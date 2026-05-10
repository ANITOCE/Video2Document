use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::UNIX_EPOCH;

use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use walkdir::WalkDir;

use crate::models::{CourseDocument, DirectoryChildEntry, DirectoryDocumentEntry, DirectoryIndex};
use crate::writer;

#[derive(Default)]
struct DirectoryAccumulator {
    direct_docs: Vec<CourseDocument>,
    child_dirs: BTreeSet<PathBuf>,
}

pub fn build_indexes(output_root: &Path, documents: &[CourseDocument]) -> Vec<DirectoryIndex> {
    if documents.is_empty() {
        return Vec::new();
    }

    let mut directory_map: BTreeMap<PathBuf, DirectoryAccumulator> = BTreeMap::new();
    directory_map.entry(PathBuf::new()).or_default();

    for document in documents {
        let directory = document
            .relative_path
            .parent()
            .map(Path::to_path_buf)
            .unwrap_or_default();
        directory_map
            .entry(directory.clone())
            .or_default()
            .direct_docs
            .push(document.clone());

        let mut ancestor = PathBuf::new();
        for component in directory.components() {
            let child = ancestor.join(component.as_os_str());
            directory_map
                .entry(ancestor.clone())
                .or_default()
                .child_dirs
                .insert(child.clone());
            directory_map.entry(child.clone()).or_default();
            ancestor = child;
        }
    }

    let mut indexes = Vec::new();
    for (directory, accumulator) in &directory_map {
        let child_directories = accumulator
            .child_dirs
            .iter()
            .map(|child| DirectoryChildEntry {
                name: child
                    .file_name()
                    .map(|name| name.to_string_lossy().to_string())
                    .unwrap_or_else(|| ".".to_string()),
                link: format!(
                    "{}/index.md",
                    child.file_name().unwrap_or_default().to_string_lossy()
                ),
                relative_path: child.clone(),
                document_count: count_documents_in_directory(child, documents),
                latest_generated_at: latest_generated_at(child, documents),
                summary: summarize_directory(child, documents),
            })
            .collect::<Vec<_>>();

        let child_documents = accumulator
            .direct_docs
            .iter()
            .map(|document| DirectoryDocumentEntry {
                title: document.title.clone(),
                link: document
                    .document_path
                    .file_name()
                    .map(|name| name.to_string_lossy().to_string())
                    .unwrap_or_default(),
                relative_path: document.relative_path.clone(),
                generated_at: document.generated_at,
                summary: document.directory_summary.clone(),
                keywords: document.keywords.clone(),
            })
            .collect::<Vec<_>>();

        indexes.push(DirectoryIndex {
            directory_path: directory.clone(),
            index_path: output_root.join(directory).join("index.md"),
            child_directories,
            child_documents,
            document_count: count_documents_in_directory(directory, documents),
            latest_generated_at: latest_generated_at(directory, documents),
            summary: summarize_directory(directory, documents),
        });
    }

    indexes.sort_by(|left, right| left.directory_path.cmp(&right.directory_path));
    indexes
}

pub fn write_indexes(indexes: &[DirectoryIndex]) -> Result<()> {
    for index in indexes {
        let markdown = writer::render_index(index);
        writer::write_markdown(&index.index_path, &markdown)?;
    }
    Ok(())
}

pub fn load_documents_from_output(output_root: &Path) -> Result<Vec<CourseDocument>> {
    let mut documents = Vec::new();

    if !output_root.exists() {
        return Ok(documents);
    }

    for entry in WalkDir::new(output_root).into_iter().filter_map(Result::ok) {
        let path = entry.path();
        if !entry.file_type().is_file() {
            continue;
        }
        if path.extension().and_then(|value| value.to_str()) != Some("md") {
            continue;
        }
        if path.file_name().and_then(|value| value.to_str()) == Some("index.md") {
            continue;
        }

        let relative_path = path.strip_prefix(output_root)?.to_path_buf();
        let content = fs::read_to_string(path)
            .with_context(|| format!("failed to read output markdown {}", path.display()))?;
        let generated_at = fs::metadata(path)
            .and_then(|metadata| metadata.modified())
            .ok()
            .and_then(|value| value.duration_since(UNIX_EPOCH).ok())
            .map(|value| DateTime::<Utc>::from(UNIX_EPOCH + value))
            .unwrap_or_else(Utc::now);
        let title = content
            .lines()
            .find_map(|line| line.strip_prefix("# "))
            .unwrap_or_else(|| {
                relative_path
                    .file_stem()
                    .and_then(|stem| stem.to_str())
                    .unwrap_or("课程文档")
            })
            .to_string();
        let summary = content
            .lines()
            .find(|line| line.starts_with("- "))
            .map(|line| line.trim_start_matches("- ").to_string())
            .unwrap_or_default();

        documents.push(CourseDocument {
            relative_path: relative_path.clone(),
            document_path: path.to_path_buf(),
            title,
            document_markdown: content,
            directory_summary: summary,
            keywords: Vec::new(),
            review_flags: Vec::new(),
            generated_at,
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

    documents.sort_by(|left, right| left.relative_path.cmp(&right.relative_path));
    Ok(documents)
}

fn count_documents_in_directory(directory: &Path, documents: &[CourseDocument]) -> usize {
    documents
        .iter()
        .filter(|document| {
            directory.as_os_str().is_empty() || document.relative_path.starts_with(directory)
        })
        .count()
}

fn latest_generated_at(directory: &Path, documents: &[CourseDocument]) -> Option<DateTime<Utc>> {
    documents
        .iter()
        .filter(|document| {
            directory.as_os_str().is_empty() || document.relative_path.starts_with(directory)
        })
        .map(|document| document.generated_at)
        .max()
}

fn summarize_directory(directory: &Path, documents: &[CourseDocument]) -> String {
    documents
        .iter()
        .filter(|document| {
            directory.as_os_str().is_empty() || document.relative_path.starts_with(directory)
        })
        .filter_map(|document| {
            if document.directory_summary.is_empty() {
                None
            } else {
                Some(document.directory_summary.clone())
            }
        })
        .take(2)
        .collect::<Vec<_>>()
        .join("；")
}
