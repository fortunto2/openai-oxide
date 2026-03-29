// Batch types — mirrors openai-python types/batch.py

use crate::openai_enum;
use serde::{Deserialize, Serialize};

// Re-export new types from openai-types
pub use openai_types::batch::{
    BatchError, BatchErrors, BatchUsage, InputTokensDetails, OutputExpiresAfter,
    OutputTokensDetails,
};

/// Request body for `POST /batches`.
#[derive(Debug, Clone, Serialize)]
pub struct BatchCreateRequest {
    /// File ID of the input JSONL file.
    pub input_file_id: String,

    /// API endpoint (e.g. "/v1/chat/completions").
    pub endpoint: String,

    /// Completion window (currently only "24h").
    pub completion_window: String,

    /// Metadata.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<std::collections::HashMap<String, String>>,
}

impl BatchCreateRequest {
    pub fn new(
        input_file_id: impl Into<String>,
        endpoint: impl Into<String>,
        completion_window: impl Into<String>,
    ) -> Self {
        Self {
            input_file_id: input_file_id.into(),
            endpoint: endpoint.into(),
            completion_window: completion_window.into(),
            metadata: None,
        }
    }
}

openai_enum! {
    /// Status of a batch job.
    pub enum BatchStatus {
        Validating = "validating",
        Failed = "failed",
        InProgress = "in_progress",
        Finalizing = "finalizing",
        Completed = "completed",
        Expired = "expired",
        Cancelling = "cancelling",
        Cancelled = "cancelled",
    }
}

/// A batch object.
#[derive(Debug, Clone, Deserialize)]
pub struct Batch {
    pub id: String,
    pub object: String,
    pub endpoint: String,
    pub input_file_id: String,
    pub completion_window: String,
    pub status: BatchStatus,
    pub created_at: i64,
    #[serde(default)]
    pub output_file_id: Option<String>,
    #[serde(default)]
    pub error_file_id: Option<String>,
    #[serde(default)]
    pub in_progress_at: Option<i64>,
    #[serde(default)]
    pub completed_at: Option<i64>,
    #[serde(default)]
    pub failed_at: Option<i64>,
    #[serde(default)]
    pub cancelled_at: Option<i64>,
    #[serde(default)]
    pub expired_at: Option<i64>,
    #[serde(default)]
    pub request_counts: Option<BatchRequestCounts>,
    #[serde(default)]
    pub metadata: Option<std::collections::HashMap<String, String>>,
}

/// Request counts for a batch.
#[derive(Debug, Clone, Deserialize)]
pub struct BatchRequestCounts {
    pub total: i64,
    pub completed: i64,
    pub failed: i64,
}

/// List of batches.
#[derive(Debug, Clone, Deserialize)]
pub struct BatchList {
    pub object: String,
    pub data: Vec<Batch>,
    /// Whether there are more results available.
    #[serde(default)]
    pub has_more: Option<bool>,
    /// ID of the first object in the list.
    #[serde(default)]
    pub first_id: Option<String>,
    /// ID of the last object in the list.
    #[serde(default)]
    pub last_id: Option<String>,
}

/// Parameters for listing batches with pagination.
#[derive(Debug, Clone, Default)]
pub struct BatchListParams {
    /// Cursor for pagination — fetch results after this batch ID.
    pub after: Option<String>,
    /// Maximum number of results per page (1–100).
    pub limit: Option<i64>,
}

impl BatchListParams {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn after(mut self, after: impl Into<String>) -> Self {
        self.after = Some(after.into());
        self
    }

    pub fn limit(mut self, limit: i64) -> Self {
        self.limit = Some(limit);
        self
    }

    /// Convert to query parameter pairs for the HTTP request.
    pub fn to_query(&self) -> Vec<(String, String)> {
        let mut q = Vec::new();
        if let Some(ref after) = self.after {
            q.push(("after".into(), after.clone()));
        }
        if let Some(limit) = self.limit {
            q.push(("limit".into(), limit.to_string()));
        }
        q
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_serialize_batch_create() {
        let req = BatchCreateRequest::new("file-abc123", "/v1/chat/completions", "24h");
        let json = serde_json::to_value(&req).unwrap();
        assert_eq!(json["input_file_id"], "file-abc123");
        assert_eq!(json["endpoint"], "/v1/chat/completions");
        assert_eq!(json["completion_window"], "24h");
    }

    #[test]
    fn test_deserialize_batch() {
        let json = r#"{
            "id": "batch_abc123",
            "object": "batch",
            "endpoint": "/v1/chat/completions",
            "input_file_id": "file-abc123",
            "completion_window": "24h",
            "status": "completed",
            "created_at": 1699012949,
            "request_counts": {"total": 100, "completed": 95, "failed": 5}
        }"#;
        let batch: Batch = serde_json::from_str(json).unwrap();
        assert_eq!(batch.id, "batch_abc123");
        assert_eq!(batch.status, BatchStatus::Completed);
        let counts = batch.request_counts.unwrap();
        assert_eq!(counts.total, 100);
    }
}
