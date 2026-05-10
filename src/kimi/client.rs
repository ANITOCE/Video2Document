use std::env;
use std::path::Path;
use std::sync::Arc;

use anyhow::{Context, Result, anyhow};
use base64::Engine;
use reqwest::multipart::{Form, Part};
use tokio::sync::Semaphore;

use crate::config::AppSettings;
use crate::metadata::format_time_range;
use crate::models::{SegmentAnalysis, SourceVideo, UploadMode, VideoSegment};

use super::types::{
    ChatCompletionRequest, ChatCompletionResponse, ContentPart, MessageContent, RequestMessage,
    UploadFileResponse, VideoUrlPayload,
};

const AUTO_BASE64_MAX_BYTES: u64 = 24 * 1024 * 1024;

#[derive(Clone)]
pub struct KimiClient {
    http: reqwest::Client,
    base_url: String,
    model: String,
    api_key: String,
}

impl KimiClient {
    pub fn from_settings(settings: &AppSettings) -> Result<Self> {
        let api_key = env::var("MOONSHOT_API_KEY")
            .context("MOONSHOT_API_KEY is required for Kimi requests")?;
        Ok(Self {
            http: reqwest::Client::new(),
            base_url: settings.kimi.base_url.trim_end_matches('/').to_string(),
            model: settings.kimi.model.clone(),
            api_key,
        })
    }

    pub async fn analyze_segment(
        &self,
        source_video: &SourceVideo,
        segment: &VideoSegment,
        request_limit: Arc<Semaphore>,
    ) -> Result<SegmentAnalysis> {
        let clip_reference = match segment.upload_mode {
            UploadMode::Base64 => self.encode_as_data_url(&segment.clip_path)?,
            UploadMode::File => {
                self.upload_file(&segment.clip_path, request_limit.clone())
                    .await?
            }
            UploadMode::Auto => {
                let size = std::fs::metadata(&segment.clip_path)?.len();
                if size <= AUTO_BASE64_MAX_BYTES {
                    self.encode_as_data_url(&segment.clip_path)?
                } else {
                    self.upload_file(&segment.clip_path, request_limit.clone())
                        .await?
                }
            }
        };

        let prompt = build_segment_prompt(source_video, segment);
        let request = ChatCompletionRequest {
            model: self.model.clone(),
            temperature: 1.0,
            messages: vec![
                RequestMessage {
                    role: "system".to_string(),
                    content: MessageContent::Text(
                        "你是一名课程整理助手。只输出 JSON 对象，不要输出 Markdown、解释或代码块。"
                            .to_string(),
                    ),
                },
                RequestMessage {
                    role: "user".to_string(),
                    content: MessageContent::Parts(vec![
                        ContentPart::Text { text: prompt },
                        ContentPart::VideoUrl {
                            video_url: VideoUrlPayload {
                                url: clip_reference,
                            },
                        },
                    ]),
                },
            ],
        };

        let _permit = request_limit.acquire_owned().await?;
        let response = self
            .http
            .post(format!("{}/chat/completions", self.base_url))
            .bearer_auth(&self.api_key)
            .json(&request)
            .send()
            .await
            .context("failed to call Kimi chat completions")?;
        let status = response.status();
        if !status.is_success() {
            let body = response.text().await.unwrap_or_default();
            return Err(anyhow!("Kimi chat completions returned {status}: {body}"));
        }

        let payload = response
            .json::<ChatCompletionResponse>()
            .await
            .context("failed to parse Kimi chat completion response")?;

        let content = payload
            .choices
            .first()
            .ok_or_else(|| anyhow!("Kimi chat response did not include any choices"))?
            .message
            .content
            .clone();
        let text = extract_content_text(content);
        let mut analysis = parse_segment_analysis(&text)?;
        analysis.relative_path = source_video.relative_path.clone();
        analysis.segment_id = segment.segment_id.clone();
        analysis.time_range = format_time_range(segment);
        if analysis.course.trim().is_empty() {
            analysis.course = source_video
                .relative_path
                .parent()
                .and_then(|value| value.file_name())
                .map(|value| value.to_string_lossy().to_string())
                .unwrap_or_else(|| "未命名课程".to_string());
        }
        Ok(analysis)
    }

    async fn upload_file(&self, path: &Path, request_limit: Arc<Semaphore>) -> Result<String> {
        let _permit = request_limit.acquire_owned().await?;
        let file_name = path
            .file_name()
            .and_then(|value| value.to_str())
            .unwrap_or("segment.mp4")
            .to_string();
        let file_bytes = tokio::fs::read(path)
            .await
            .with_context(|| format!("failed to read clip {}", path.display()))?;
        let part = Part::bytes(file_bytes)
            .file_name(file_name)
            .mime_str("video/mp4")?;
        let form = Form::new()
            .text("purpose", "file-extract")
            .part("file", part);
        let response = self
            .http
            .post(format!("{}/files", self.base_url))
            .bearer_auth(&self.api_key)
            .multipart(form)
            .send()
            .await
            .context("failed to upload clip to Kimi")?;
        let status = response.status();
        if !status.is_success() {
            let body = response.text().await.unwrap_or_default();
            return Err(anyhow!("Kimi file upload returned {status}: {body}"));
        }

        let payload = response
            .json::<UploadFileResponse>()
            .await
            .context("failed to parse Kimi file upload response")?;
        Ok(format!("ms://{}", payload.id))
    }

    fn encode_as_data_url(&self, path: &Path) -> Result<String> {
        let bytes = std::fs::read(path)
            .with_context(|| format!("failed to read clip {}", path.display()))?;
        let encoded = base64::engine::general_purpose::STANDARD.encode(bytes);
        Ok(format!("data:video/mp4;base64,{encoded}"))
    }
}

fn build_segment_prompt(source_video: &SourceVideo, segment: &VideoSegment) -> String {
    format!(
        "请根据视频片段整理中文课程笔记，返回一个 JSON 对象，字段必须完整：\n{{\n  \"course\": \"\",\n  \"relative_path\": \"{}\",\n  \"segment_id\": \"{}\",\n  \"time_range\": \"{}\",\n  \"topic\": \"\",\n  \"summary\": \"\",\n  \"key_concepts\": [{{\"term\": \"\", \"note\": \"\"}}],\n  \"formulas\": [{{\"expression\": \"\", \"description\": \"\"}}],\n  \"teacher_emphasis\": [\"\"],\n  \"examples\": [{{\"title\": \"\", \"summary\": \"\"}}],\n  \"confusions\": [\"\"],\n  \"questions\": [\"\"],\n  \"tags\": [\"\"],\n  \"low_confidence\": [\"\"]\n}}\n要求：\n1. 仅保留课堂精讲内容。\n2. 术语与概念简记保持简短。\n3. 问答与自测相关内容写入 questions。\n4. 无内容的数组返回空数组。\n5. 不要输出 JSON 以外的任何文本。\n视频文件：{}\n时间范围：{}",
        source_video.relative_path.display(),
        segment.segment_id,
        format_time_range(segment),
        source_video.file_name,
        format_time_range(segment),
    )
}

fn extract_content_text(content: serde_json::Value) -> String {
    match content {
        serde_json::Value::String(text) => text,
        serde_json::Value::Array(values) => values
            .iter()
            .filter_map(|item| item.get("text").and_then(|value| value.as_str()))
            .collect::<Vec<_>>()
            .join("\n"),
        other => other.to_string(),
    }
}

fn parse_segment_analysis(content: &str) -> Result<SegmentAnalysis> {
    let candidate = extract_json_object(content).unwrap_or(content);
    serde_json::from_str(candidate).context("failed to parse segment analysis JSON")
}

fn extract_json_object(content: &str) -> Option<&str> {
    let start = content.find('{')?;
    let end = content.rfind('}')?;
    content.get(start..=end)
}
