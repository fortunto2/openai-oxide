//! Smoke tests for openai-types crate — deserialization of realistic API responses.

use openai_types::chat::*;
use openai_types::responses;
use openai_types::shared::ReasoningEffort;

#[test]
fn chat_completion_deserialize() {
    let json = r#"{
        "id": "chatcmpl-abc123",
        "object": "chat.completion",
        "created": 1700000000,
        "model": "gpt-4o",
        "choices": [{
            "index": 0,
            "finish_reason": "stop",
            "message": {
                "role": "assistant",
                "content": "Hello!"
            }
        }],
        "usage": {
            "prompt_tokens": 10,
            "completion_tokens": 5,
            "total_tokens": 15
        }
    }"#;
    let resp: ChatCompletion = serde_json::from_str(json).unwrap();
    assert_eq!(resp.id, "chatcmpl-abc123");
    assert_eq!(resp.model, "gpt-4o");
    assert_eq!(resp.choices.len(), 1);
}

#[test]
fn response_create_request_serialize() {
    let mut req = responses::ResponseCreateRequest::default();
    req.model = "gpt-4o".to_string();
    req.instructions = Some("Be helpful".to_string());

    let json = serde_json::to_string(&req).unwrap();
    assert!(json.contains("\"model\":\"gpt-4o\""));
    assert!(json.contains("\"instructions\":\"Be helpful\""));
}

#[test]
fn reasoning_effort_enum() {
    let json = r#""high""#;
    let effort: ReasoningEffort = serde_json::from_str(json).unwrap();
    assert_eq!(effort, ReasoningEffort::High);

    let serialized = serde_json::to_string(&effort).unwrap();
    assert_eq!(serialized, r#""high""#);
}

#[test]
fn role_enum() {
    let json = r#""user""#;
    let role: responses::Role = serde_json::from_str(json).unwrap();
    assert_eq!(role, responses::Role::User);
}

#[test]
fn batch_status_roundtrip() {
    let json = r#""in_progress""#;
    let status: openai_types::batch::BatchStatus = serde_json::from_str(json).unwrap();
    let back = serde_json::to_string(&status).unwrap();
    assert_eq!(json, &back);
}

#[test]
fn no_default_features_compiles() {
    // This test just verifies the crate links successfully
    assert!(std::mem::size_of::<responses::Role>() > 0);
}
