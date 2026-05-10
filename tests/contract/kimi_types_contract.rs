use video2document::kimi::types::{
    ChatCompletionRequest, ContentPart, MessageContent, RequestMessage, UploadFileResponse,
    VideoUrlPayload,
};

#[test]
fn serializes_openai_compatible_video_request() {
    let request = ChatCompletionRequest {
        model: "test-model".to_string(),
        temperature: 0.2,
        messages: vec![RequestMessage {
            role: "user".to_string(),
            content: MessageContent::Parts(vec![
                ContentPart::Text {
                    text: "test".to_string(),
                },
                ContentPart::VideoUrl {
                    video_url: VideoUrlPayload {
                        url: "ms://file_123".to_string(),
                    },
                },
            ]),
        }],
    };

    let json = serde_json::to_value(&request).expect("failed to serialize request");
    assert_eq!(json["messages"][0]["content"][0]["type"], "text");
    assert_eq!(json["messages"][0]["content"][1]["type"], "video_url");
    assert_eq!(
        json["messages"][0]["content"][1]["video_url"]["url"],
        "ms://file_123"
    );
}

#[test]
fn parses_file_upload_response() {
    let response: UploadFileResponse =
        serde_json::from_value(serde_json::json!({ "id": "file_123" }))
            .expect("failed to deserialize upload response");
    assert_eq!(response.id, "file_123");
}
