// Embedding types — mirrors openai-python types/embedding.py

use crate::openai_enum;
use serde::{Deserialize, Serialize};

openai_enum! {
    /// Encoding format for embedding output.
    pub enum EncodingFormat {
        Float = "float",
        Base64 = "base64",
    }
}

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

impl From<Vec<Vec<i64>>> for EmbeddingInput {
    fn from(v: Vec<Vec<i64>>) -> Self {
        EmbeddingInput::Tokens(v)
    }
}

/// Request body for `POST /embeddings`.
#[derive(Debug, Clone, Serialize)]
pub struct EmbeddingCreateRequest {
    /// Input text to embed.
    pub input: EmbeddingInput,

    /// Embedding model (e.g. "text-embedding-3-small").
    pub model: String,

    /// Encoding format for the embedding vectors.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub encoding_format: Option<EncodingFormat>,

    /// Number of dimensions to return (for supported models).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dimensions: Option<i64>,

    /// A unique identifier representing your end-user.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user: Option<String>,
}

impl EmbeddingCreateRequest {
    pub fn new(input: impl Into<EmbeddingInput>, model: impl Into<String>) -> Self {
        Self {
            input: input.into(),
            model: model.into(),
            encoding_format: None,
            dimensions: None,
            user: None,
        }
    }
}

/// Backward compatibility alias.
pub type EmbeddingRequest = EmbeddingCreateRequest;

// ── Response types ──

/// A single embedding vector.
#[derive(Debug, Clone, Deserialize)]
pub struct Embedding {
    pub object: String,
    /// The embedding vector (when encoding_format is float).
    #[serde(default)]
    pub embedding: Option<Vec<f64>>,
    /// Base64-encoded embedding (when encoding_format is base64).
    #[serde(default)]
    pub b64_embedding: Option<String>,
    pub index: i64,
}

/// Response from `POST /embeddings`.
#[derive(Debug, Clone, Deserialize)]
pub struct EmbeddingResponse {
    pub object: String,
    pub data: Vec<Embedding>,
    pub model: String,
    pub usage: crate::types::common::Usage,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_serialize_embedding_request() {
        let req = EmbeddingCreateRequest::new("Hello world", "text-embedding-3-small");
        let json = serde_json::to_value(&req).unwrap();
        assert_eq!(json["input"], "Hello world");
        assert_eq!(json["model"], "text-embedding-3-small");
    }

    #[test]
    fn test_serialize_embedding_request_with_array() {
        let req = EmbeddingCreateRequest::new(
            vec!["Hello".to_string(), "World".to_string()],
            "text-embedding-3-small",
        );
        let json = serde_json::to_value(&req).unwrap();
        let arr = json["input"].as_array().unwrap();
        assert_eq!(arr.len(), 2);
    }

    #[test]
    fn test_deserialize_embedding_response() {
        let json = r#"{
            "object": "list",
            "data": [{
                "object": "embedding",
                "embedding": [0.1, 0.2, 0.3],
                "index": 0
            }],
            "model": "text-embedding-3-small",
            "usage": {"prompt_tokens": 10, "total_tokens": 10}
        }"#;
        let resp: EmbeddingResponse = serde_json::from_str(json).unwrap();
        assert_eq!(resp.data.len(), 1);
        assert_eq!(resp.data[0].embedding.as_ref().unwrap().len(), 3);
    }
}
