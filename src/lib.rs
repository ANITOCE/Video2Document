pub mod cli;
pub mod clipper;
pub mod config;
pub mod indexer;
pub mod kimi;
pub mod logging;
pub mod metadata;
pub mod models;
pub mod scanner;
pub mod state_store;
pub mod writer;

pub use config::{AppSettings, ResolvedConfig};

use anyhow::{Context, Result};
use tracing::{error, info, warn};

use crate::config::load_settings;
use crate::models::{CourseDocument, ExitReport, RunSummary, TaskState};

pub async fn run_cli(args: cli::CliArgs) -> Result<ExitReport> {
    let resolved = load_settings(&args)?;
    logging::init_tracing(&resolved.settings.runtime.log_filter);

    std::fs::create_dir_all(&resolved.settings.paths.output_dir).with_context(|| {
        format!(
            "failed to create output dir {}",
            resolved.settings.paths.output_dir.display()
        )
    })?;
    std::fs::create_dir_all(&resolved.settings.paths.working_dir).with_context(|| {
        format!(
            "failed to create working dir {}",
            resolved.settings.paths.working_dir.display()
        )
    })?;

    let state_store = state_store::StateStore::new(resolved.settings.paths.working_dir.clone());

    if resolved.only_index {
        let documents = load_documents_for_indexing(&resolved.settings, &state_store)?;
        let indexes = indexer::build_indexes(&resolved.settings.paths.output_dir, &documents);
        indexer::write_indexes(&indexes)?;
        for index in &indexes {
            state_store.save_directory_index_cache(index)?;
        }
        return Ok(ExitReport {
            code: 0,
            summary: RunSummary {
                discovered: documents.len(),
                processed: 0,
                reused: documents.len(),
                failed: 0,
            },
        });
    }

    let kimi_client = kimi::client::KimiClient::from_settings(&resolved.settings)?;
    let videos = scanner::scan_videos(&resolved.settings)?;
    let fingerprint = resolved.settings.processing_fingerprint()?;

    let mut summary = RunSummary {
        discovered: videos.len(),
        ..RunSummary::default()
    };

    for video in videos {
        if !resolved.force && state_store.is_video_complete_and_valid(&video, &fingerprint)? {
            summary.reused += 1;
            info!(relative_path = %video.relative_path.display(), "reusing completed video output");
            continue;
        }

        let result = process_video_with_retry(
            &video,
            &resolved.settings,
            &state_store,
            &kimi_client,
            &fingerprint,
            resolved.force,
        )
        .await;

        match result {
            Ok(_) => summary.processed += 1,
            Err(error) => {
                summary.failed += 1;
                state_store.record_failure(&video, &fingerprint, error.to_string())?;
                warn!(relative_path = %video.relative_path.display(), error = %error, "video processing failed");
            }
        }
    }

    let documents = load_documents_for_indexing(&resolved.settings, &state_store)?;
    let indexes = indexer::build_indexes(&resolved.settings.paths.output_dir, &documents);
    indexer::write_indexes(&indexes)?;
    for index in &indexes {
        state_store.save_directory_index_cache(index)?;
    }

    let code = if summary.failed > 0 { 2 } else { 0 };
    if summary.failed > 0 {
        error!(failed = summary.failed, "run completed with failed videos");
    } else {
        info!(
            processed = summary.processed,
            reused = summary.reused,
            "run completed successfully"
        );
    }

    Ok(ExitReport { code, summary })
}

fn load_documents_for_indexing(
    settings: &config::AppSettings,
    state_store: &state_store::StateStore,
) -> Result<Vec<CourseDocument>> {
    let mut documents = state_store.load_all_documents()?;
    if documents.is_empty() {
        documents = indexer::load_documents_from_output(&settings.paths.output_dir)?;
    }
    Ok(documents)
}

async fn process_video_with_retry(
    video: &models::SourceVideo,
    settings: &config::AppSettings,
    state_store: &state_store::StateStore,
    kimi_client: &kimi::client::KimiClient,
    fingerprint: &str,
    force: bool,
) -> Result<CourseDocument> {
    let mut last_error = None;

    for attempt in 0..=settings.kimi.max_retries {
        match process_video(
            video,
            settings,
            state_store,
            kimi_client,
            fingerprint,
            force,
        )
        .await
        {
            Ok(document) => return Ok(document),
            Err(error) => {
                last_error = Some(error);
                if attempt < settings.kimi.max_retries {
                    tokio::time::sleep(std::time::Duration::from_secs(
                        settings.kimi.retry_backoff_seconds,
                    ))
                    .await;
                }
            }
        }
    }

    Err(last_error.unwrap_or_else(|| anyhow::anyhow!("video processing failed")))
}

async fn process_video(
    video: &models::SourceVideo,
    settings: &config::AppSettings,
    state_store: &state_store::StateStore,
    kimi_client: &kimi::client::KimiClient,
    fingerprint: &str,
    force: bool,
) -> Result<CourseDocument> {
    state_store.update_state(video, fingerprint, TaskState::Scanning, 0, Vec::new())?;

    let metadata = if !force {
        state_store
            .load_video_metadata(video, fingerprint)?
            .unwrap_or(metadata::probe_video(&video.source_path).await?)
    } else {
        metadata::probe_video(&video.source_path).await?
    };

    state_store.save_video_metadata(video, fingerprint, &metadata)?;
    state_store.update_state(video, fingerprint, TaskState::MetadataDone, 0, Vec::new())?;

    let mut enriched_video = video.clone();
    enriched_video.duration_seconds = Some(metadata.duration_seconds);
    enriched_video.width = Some(metadata.width);
    enriched_video.height = Some(metadata.height);

    let segments = metadata::build_segments(&enriched_video, &metadata, &settings.video)?;
    state_store.update_state(
        &enriched_video,
        fingerprint,
        TaskState::Clipped,
        segments.len(),
        Vec::new(),
    )?;

    let analyses =
        kimi::segment_analyzer::analyze_segments(kimi::segment_analyzer::AnalyzeSegmentsRequest {
            video: &enriched_video,
            metadata: &metadata,
            segments: &segments,
            settings,
            state_store,
            kimi_client,
            fingerprint,
            force,
        })
        .await?;
    state_store.update_state(
        &enriched_video,
        fingerprint,
        TaskState::KimiSegmentDone,
        analyses.len(),
        Vec::new(),
    )?;

    let mut document = kimi::document_aggregator::aggregate_document(&enriched_video, &analyses);
    document.document_markdown = writer::render_course_document(&document);
    anyhow::ensure!(
        writer::validate_section_order(&document.document_markdown),
        "rendered document does not contain the required section order"
    );

    state_store.save_document_bundle(&enriched_video, fingerprint, &document)?;
    state_store.update_state(
        &enriched_video,
        fingerprint,
        TaskState::KimiDocumentDone,
        analyses.len(),
        Vec::new(),
    )?;

    writer::write_markdown(&document.document_path, &document.document_markdown)?;
    state_store.update_state(
        &enriched_video,
        fingerprint,
        TaskState::DocumentWritten,
        analyses.len(),
        Vec::new(),
    )?;

    Ok(document)
}
