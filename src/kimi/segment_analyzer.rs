use std::sync::Arc;

use anyhow::Result;
use futures::stream::{self, StreamExt, TryStreamExt};
use tokio::sync::Semaphore;

use crate::clipper;
use crate::config::AppSettings;
use crate::models::{SegmentAnalysis, SourceVideo, VideoMetadata, VideoSegment};
use crate::state_store::StateStore;

use super::client::KimiClient;

pub struct AnalyzeSegmentsRequest<'a> {
    pub video: &'a SourceVideo,
    pub metadata: &'a VideoMetadata,
    pub segments: &'a [VideoSegment],
    pub settings: &'a AppSettings,
    pub state_store: &'a StateStore,
    pub kimi_client: &'a KimiClient,
    pub fingerprint: &'a str,
    pub force: bool,
}

pub async fn analyze_segments(request: AnalyzeSegmentsRequest<'_>) -> Result<Vec<SegmentAnalysis>> {
    let preprocess_limit = Arc::new(Semaphore::new(
        request.settings.runtime.preprocess_concurrency,
    ));
    let request_limit = Arc::new(Semaphore::new(request.settings.runtime.request_concurrency));

    let mut analyses = stream::iter(request.segments.iter().cloned().enumerate())
        .map(|(index, segment)| {
            let preprocess_limit = preprocess_limit.clone();
            let request_limit = request_limit.clone();
            let state_store = request.state_store.clone();
            let kimi_client = request.kimi_client.clone();
            let video = request.video.clone();
            let metadata = request.metadata.clone();
            let video_settings = request.settings.video.clone();
            let fingerprint = request.fingerprint.to_string();
            let force = request.force;

            async move {
                if !force
                    && let Some(existing) = state_store.load_segment_analysis(
                        &video,
                        &segment.segment_id,
                        &fingerprint,
                    )?
                {
                    return Ok::<_, anyhow::Error>((index, existing));
                }

                {
                    let _permit = preprocess_limit.acquire_owned().await?;
                    clipper::create_clip(&video.source_path, &segment, &metadata, &video_settings)
                        .await?;
                }

                let analysis = kimi_client
                    .analyze_segment(&video, &segment, request_limit.clone())
                    .await?;
                state_store.save_segment_analysis(&video, &segment.segment_id, &analysis)?;
                Ok::<_, anyhow::Error>((index, analysis))
            }
        })
        .buffer_unordered(
            request
                .settings
                .runtime
                .request_concurrency
                .max(request.settings.runtime.preprocess_concurrency),
        )
        .try_collect::<Vec<_>>()
        .await?;

    analyses.sort_by_key(|(index, _)| *index);
    Ok(analyses.into_iter().map(|(_, analysis)| analysis).collect())
}
