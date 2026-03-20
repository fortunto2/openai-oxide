// File types — mirrors openai-python types/file_object.py

use serde::{Deserialize, Serialize};

use super::common::SortOrder;

/// The intended purpose of an uploaded file.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub enum FilePurpose {
    #[serde(rename = "assistants")]
    Assistants,
    #[serde(rename = "assistants_output")]
    AssistantsOutput,
    #[serde(rename = "batch")]
    Batch,
    #[serde(rename = "batch_output")]
    BatchOutput,
    #[serde(rename = "fine-tune")]
    FineTune,
    #[serde(rename = "fine-tune-results")]
    FineTuneResults,
    #[serde(rename = "vision")]
    Vision,
    #[serde(rename = "user_data")]
    UserData,
}

/// Processing status of an uploaded file.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum FileStatus {
    Uploaded,
    Processed,
    Error,
}

/// A file object from the API.
#[derive(Debug, Clone, Deserialize)]
pub struct FileObject {
    pub id: String,
    pub bytes: i64,
    pub created_at: i64,
    pub filename: String,
    pub object: String,
    pub purpose: FilePurpose,
    pub status: FileStatus,
    #[serde(default)]
    pub status_details: Option<String>,
    #[serde(default)]
    pub expires_at: Option<i64>,
}

/// Response from listing files.
#[derive(Debug, Clone, Deserialize)]
pub struct FileList {
    pub object: String,
    pub data: Vec<FileObject>,
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

/// Response from deleting a file.
#[derive(Debug, Clone, Deserialize)]
pub struct FileDeleted {
    pub id: String,
    pub deleted: bool,
    pub object: String,
}

/// Parameters for file upload (multipart).
#[derive(Debug)]
pub struct FileUploadParams {
    pub file: Vec<u8>,
    pub filename: String,
    pub purpose: FilePurpose,
}

impl FileUploadParams {
    pub fn new(file: Vec<u8>, filename: impl Into<String>, purpose: FilePurpose) -> Self {
        Self {
            file,
            filename: filename.into(),
            purpose,
        }
    }
}

/// Parameters for listing files with pagination.
#[derive(Debug, Clone, Default)]
pub struct FileListParams {
    /// Cursor for pagination — fetch results after this file ID.
    pub after: Option<String>,
    /// Maximum number of results per page (1–10000).
    pub limit: Option<i64>,
    /// Sort order by `created_at`.
    pub order: Option<SortOrder>,
    /// Filter by file purpose.
    pub purpose: Option<FilePurpose>,
}

impl FileListParams {
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

    pub fn order(mut self, order: SortOrder) -> Self {
        self.order = Some(order);
        self
    }

    pub fn purpose(mut self, purpose: FilePurpose) -> Self {
        self.purpose = Some(purpose);
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
        if let Some(ref order) = self.order {
            q.push((
                "order".into(),
                serde_json::to_value(order)
                    .unwrap()
                    .as_str()
                    .unwrap()
                    .to_string(),
            ));
        }
        if let Some(ref purpose) = self.purpose {
            q.push((
                "purpose".into(),
                serde_json::to_value(purpose)
                    .unwrap()
                    .as_str()
                    .unwrap()
                    .to_string(),
            ));
        }
        q
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deserialize_file_object() {
        let json = r#"{
            "id": "file-abc123",
            "object": "file",
            "bytes": 120000,
            "created_at": 1677610602,
            "filename": "data.jsonl",
            "purpose": "fine-tune",
            "status": "processed"
        }"#;
        let file: FileObject = serde_json::from_str(json).unwrap();
        assert_eq!(file.id, "file-abc123");
        assert_eq!(file.bytes, 120000);
        assert_eq!(file.purpose, FilePurpose::FineTune);
        assert_eq!(file.status, FileStatus::Processed);
    }

    #[test]
    fn test_deserialize_file_list_with_pagination() {
        let json = r#"{
            "object": "list",
            "data": [{
                "id": "file-abc123",
                "object": "file",
                "bytes": 120000,
                "created_at": 1677610602,
                "filename": "data.jsonl",
                "purpose": "fine-tune",
                "status": "processed"
            }],
            "has_more": true,
            "first_id": "file-abc123",
            "last_id": "file-abc123"
        }"#;
        let list: FileList = serde_json::from_str(json).unwrap();
        assert_eq!(list.data.len(), 1);
        assert_eq!(list.has_more, Some(true));
        assert_eq!(list.first_id.as_deref(), Some("file-abc123"));
        assert_eq!(list.last_id.as_deref(), Some("file-abc123"));
    }

    #[test]
    fn test_file_list_params_to_query() {
        let params = FileListParams::new()
            .after("file-cursor")
            .limit(10)
            .order(crate::types::common::SortOrder::Desc)
            .purpose(FilePurpose::FineTune);
        let query = params.to_query();
        assert!(query.contains(&("after".into(), "file-cursor".into())));
        assert!(query.contains(&("limit".into(), "10".into())));
        assert!(query.contains(&("order".into(), "desc".into())));
        assert!(query.contains(&("purpose".into(), "fine-tune".into())));
    }

    #[test]
    fn test_deserialize_file_deleted() {
        let json = r#"{"id": "file-abc123", "object": "file", "deleted": true}"#;
        let deleted: FileDeleted = serde_json::from_str(json).unwrap();
        assert!(deleted.deleted);
    }
}
