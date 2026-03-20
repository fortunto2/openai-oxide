// Responses API types — mirrors openai-python types/responses/

use serde::{Deserialize, Serialize};

// ── Request types ──

/// Input for the Responses API.
#[derive(Debug, Clone, Serialize)]
#[serde(untagged)]
pub enum ResponseInput {
    Text(String),
    Messages(Vec<ResponseInputItem>),
}

impl From<&str> for ResponseInput {
    fn from(s: &str) -> Self {
        ResponseInput::Text(s.to_string())
    }
}

impl From<String> for ResponseInput {
    fn from(s: String) -> Self {
        ResponseInput::Text(s)
    }
}

/// An input message for the Responses API.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResponseInputItem {
    pub role: String,
    pub content: serde_json::Value,
}

/// Request body for `POST /responses`.
#[derive(Debug, Clone, Serialize)]
pub struct ResponseCreateRequest {
    /// Model to use.
    pub model: String,

    /// Input text or messages.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub input: Option<ResponseInput>,

    /// System instructions.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub instructions: Option<String>,

    /// Tools available.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tools: Option<Vec<serde_json::Value>>,

    /// Previous response ID for multi-turn.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub previous_response_id: Option<String>,

    /// Temperature (0–2).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f64>,

    /// Max output tokens.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_output_tokens: Option<i64>,

    /// Store for evals.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub store: Option<bool>,

    /// Metadata.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<std::collections::HashMap<String, String>>,

    /// Whether to stream.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stream: Option<bool>,
}

impl ResponseCreateRequest {
    pub fn new(model: impl Into<String>) -> Self {
        Self {
            model: model.into(),
            input: None,
            instructions: None,
            tools: None,
            previous_response_id: None,
            temperature: None,
            max_output_tokens: None,
            store: None,
            metadata: None,
            stream: None,
        }
    }
}

// ── Response types ──

/// Output item in a response.
#[derive(Debug, Clone, Deserialize)]
pub struct ResponseOutputItem {
    #[serde(rename = "type")]
    pub type_: String,
    #[serde(default)]
    pub id: Option<String>,
    #[serde(default)]
    pub role: Option<String>,
    #[serde(default)]
    pub content: Option<Vec<ResponseOutputContent>>,
    #[serde(default)]
    pub status: Option<String>,
}

/// Content block within an output item.
#[derive(Debug, Clone, Deserialize)]
pub struct ResponseOutputContent {
    #[serde(rename = "type")]
    pub type_: String,
    #[serde(default)]
    pub text: Option<String>,
    #[serde(default)]
    pub annotations: Option<Vec<serde_json::Value>>,
}

/// Usage for the Responses API.
#[derive(Debug, Clone, Deserialize)]
pub struct ResponseUsage {
    #[serde(default)]
    pub input_tokens: Option<i64>,
    #[serde(default)]
    pub output_tokens: Option<i64>,
    #[serde(default)]
    pub total_tokens: Option<i64>,
}

/// Response from `POST /responses`.
#[derive(Debug, Clone, Deserialize)]
pub struct Response {
    pub id: String,
    pub object: String,
    pub created_at: f64,
    pub model: String,
    pub output: Vec<ResponseOutputItem>,
    #[serde(default)]
    pub status: Option<String>,
    #[serde(default)]
    pub error: Option<serde_json::Value>,
    #[serde(default)]
    pub instructions: Option<String>,
    #[serde(default)]
    pub metadata: Option<std::collections::HashMap<String, String>>,
    #[serde(default)]
    pub temperature: Option<f64>,
    #[serde(default)]
    pub top_p: Option<f64>,
    #[serde(default)]
    pub max_output_tokens: Option<i64>,
    #[serde(default)]
    pub previous_response_id: Option<String>,
    #[serde(default)]
    pub usage: Option<ResponseUsage>,
}

impl Response {
    /// Get the text output, concatenating all text content blocks.
    pub fn output_text(&self) -> String {
        let mut result = String::new();
        for item in &self.output {
            if let Some(content) = &item.content {
                for block in content {
                    if block.type_ == "output_text"
                        && let Some(text) = &block.text
                    {
                        result.push_str(text);
                    }
                }
            }
        }
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_serialize_response_create_request() {
        let mut req = ResponseCreateRequest::new("gpt-4o");
        req.input = Some("Hello".into());
        let json = serde_json::to_value(&req).unwrap();
        assert_eq!(json["model"], "gpt-4o");
        assert_eq!(json["input"], "Hello");
    }

    #[test]
    fn test_deserialize_response() {
        let json = r#"{
            "id": "resp-abc123",
            "object": "response",
            "created_at": 1677610602.0,
            "model": "gpt-4o",
            "output": [{
                "type": "message",
                "id": "msg-abc123",
                "role": "assistant",
                "status": "completed",
                "content": [{
                    "type": "output_text",
                    "text": "Hello! How can I help?",
                    "annotations": []
                }]
            }],
            "status": "completed",
            "usage": {
                "input_tokens": 10,
                "output_tokens": 6,
                "total_tokens": 16
            }
        }"#;

        let resp: Response = serde_json::from_str(json).unwrap();
        assert_eq!(resp.id, "resp-abc123");
        assert_eq!(resp.output.len(), 1);
        assert_eq!(resp.output_text(), "Hello! How can I help?");
        let usage = resp.usage.as_ref().unwrap();
        assert_eq!(usage.total_tokens, Some(16));
    }
}
