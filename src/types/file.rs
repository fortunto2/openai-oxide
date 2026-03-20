// File types — mirrors openai-python types/file_object.py

use serde::Deserialize;

/// A file object from the API.
#[derive(Debug, Clone, Deserialize)]
pub struct FileObject {
    pub id: String,
    pub bytes: i64,
    pub created_at: i64,
    pub filename: String,
    pub object: String,
    pub purpose: String,
    pub status: String,
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
    pub purpose: String,
}

impl FileUploadParams {
    pub fn new(file: Vec<u8>, filename: impl Into<String>, purpose: impl Into<String>) -> Self {
        Self {
            file,
            filename: filename.into(),
            purpose: purpose.into(),
        }
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
        assert_eq!(file.purpose, "fine-tune");
        assert_eq!(file.status, "processed");
    }

    #[test]
    fn test_deserialize_file_deleted() {
        let json = r#"{"id": "file-abc123", "object": "file", "deleted": true}"#;
        let deleted: FileDeleted = serde_json::from_str(json).unwrap();
        assert!(deleted.deleted);
    }
}
