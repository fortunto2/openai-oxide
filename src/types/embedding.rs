// Embedding types — mirrors openai-python types/embedding.py

use serde::{Deserialize, Serialize};

// ── Request types ──

/// Input for embeddings: a single string, array of strings, or array of token arrays.
#[derive(Debug, Clone, Serialize)]
#[serde(untagged)]
#[non_exhaustive]
pub enum EmbeddingInput {
    String(String),
    StringArray(Vec<String>),
    Tokens(Vec<Vec<i64>>),
}

impl From<&str> for EmbeddingInput {
    fn from(s: &str) -> Self {
        EmbeddingInput::String(s.to_string())
    }
}

impl From<String> for EmbeddingInput {
    fn from(s: String) -> Self {
        EmbeddingInput::String(s)
    }
}

impl From<Vec<String>> for EmbeddingInput {
    fn from(v: Vec<String>) -> Self {
        EmbeddingInput::StringArray(v)
    }
}

/// Request body for `POST /embeddings`.
#[derive(Debug, Clone, Serialize)]
pub struct EmbeddingRequest {
    /// Input text to embed.
    pub input: EmbeddingInput,

    /// Model ID, e.g. "text-embedding-3-small".
    pub model: String,

    /// Number of output dimensions.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dimensions: Option<i64>,

    /// Encoding format: "float" or "base64".
    #[serde(skip_serializing_if = "Option::is_none")]
    pub encoding_format: Option<String>,

    /// End user identifier.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user: Option<String>,
}

impl EmbeddingRequest {
    /// Create a new embedding request.
    pub fn new(model: impl Into<String>, input: impl Into<EmbeddingInput>) -> Self {
        Self {
            input: input.into(),
            model: model.into(),
            dimensions: None,
            encoding_format: None,
            user: None,
        }
    }
}

// ── Response types ──

/// A single embedding vector.
#[derive(Debug, Clone, Deserialize)]
pub struct Embedding {
    /// The embedding vector.
    pub embedding: Vec<f64>,

    /// Index of this embedding in the request.
    pub index: i64,

    /// Always "embedding".
    pub object: String,
}

/// Embedding-specific usage (no completion_tokens).
#[derive(Debug, Clone, Deserialize)]
pub struct EmbeddingUsage {
    pub prompt_tokens: i64,
    pub total_tokens: i64,
}

/// Response from `POST /embeddings`.
#[derive(Debug, Clone, Deserialize)]
pub struct CreateEmbeddingResponse {
    /// List of embedding objects.
    pub data: Vec<Embedding>,

    /// Model used.
    pub model: String,

    /// Always "list".
    pub object: String,

    /// Token usage.
    pub usage: EmbeddingUsage,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_serialize_embedding_request_string() {
        let req = EmbeddingRequest::new("text-embedding-3-small", "Hello world");
        let json = serde_json::to_value(&req).unwrap();
        assert_eq!(json["model"], "text-embedding-3-small");
        assert_eq!(json["input"], "Hello world");
        assert!(json.get("dimensions").is_none());
    }

    #[test]
    fn test_serialize_embedding_request_array() {
        let req = EmbeddingRequest::new(
            "text-embedding-3-small",
            EmbeddingInput::StringArray(vec!["Hello".into(), "World".into()]),
        );
        let json = serde_json::to_value(&req).unwrap();
        assert_eq!(json["input"].as_array().unwrap().len(), 2);
    }

    #[test]
    fn test_deserialize_embedding_response() {
        let json = r#"{
            "object": "list",
            "data": [{
                "object": "embedding",
                "embedding": [0.0023, -0.0094, 0.0158],
                "index": 0
            }],
            "model": "text-embedding-3-small",
            "usage": {
                "prompt_tokens": 8,
                "total_tokens": 8
            }
        }"#;

        let resp: CreateEmbeddingResponse = serde_json::from_str(json).unwrap();
        assert_eq!(resp.object, "list");
        assert_eq!(resp.model, "text-embedding-3-small");
        assert_eq!(resp.data.len(), 1);
        assert_eq!(resp.data[0].embedding.len(), 3);
        assert_eq!(resp.data[0].index, 0);
        assert_eq!(resp.usage.prompt_tokens, 8);
        assert_eq!(resp.usage.total_tokens, 8);
    }
}
