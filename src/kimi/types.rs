use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize)]
pub struct ChatCompletionRequest {
    pub model: String,
    pub messages: Vec<RequestMessage>,
    pub temperature: f32,
}

#[derive(Debug, Clone, Serialize)]
pub struct RequestMessage {
    pub role: String,
    pub content: MessageContent,
}

#[derive(Debug, Clone, Serialize)]
#[serde(untagged)]
pub enum MessageContent {
    Text(String),
    Parts(Vec<ContentPart>),
}

#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ContentPart {
    Text { text: String },
    VideoUrl { video_url: VideoUrlPayload },
}

#[derive(Debug, Clone, Serialize)]
pub struct VideoUrlPayload {
    pub url: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ChatCompletionResponse {
    pub choices: Vec<ChatChoice>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ChatChoice {
    pub message: ChatMessage,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ChatMessage {
    pub content: serde_json::Value,
}

#[derive(Debug, Clone, Deserialize)]
pub struct UploadFileResponse {
    pub id: String,
}
