// Model types — mirrors openai-python types/model.py

use serde::Deserialize;

/// A model object from the API.
#[derive(Debug, Clone, Deserialize)]
pub struct Model {
    /// Model identifier (e.g. "gpt-4o").
    pub id: String,

    /// Unix timestamp (seconds) when created.
    pub created: i64,

    /// Always "model".
    pub object: String,

    /// Organization that owns the model.
    pub owned_by: String,
}

/// Response from listing models.
#[derive(Debug, Clone, Deserialize)]
pub struct ModelList {
    pub object: String,
    pub data: Vec<Model>,
}

/// Response from deleting a model.
#[derive(Debug, Clone, Deserialize)]
pub struct ModelDeleted {
    pub id: String,
    pub deleted: bool,
    pub object: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deserialize_model() {
        let json = r#"{
            "id": "gpt-4o",
            "object": "model",
            "created": 1687882411,
            "owned_by": "openai"
        }"#;
        let model: Model = serde_json::from_str(json).unwrap();
        assert_eq!(model.id, "gpt-4o");
        assert_eq!(model.object, "model");
        assert_eq!(model.owned_by, "openai");
    }

    #[test]
    fn test_deserialize_model_list() {
        let json = r#"{
            "object": "list",
            "data": [
                {"id": "gpt-4o", "object": "model", "created": 1687882411, "owned_by": "openai"},
                {"id": "gpt-3.5-turbo", "object": "model", "created": 1677610602, "owned_by": "openai"}
            ]
        }"#;
        let list: ModelList = serde_json::from_str(json).unwrap();
        assert_eq!(list.data.len(), 2);
        assert_eq!(list.data[0].id, "gpt-4o");
    }

    #[test]
    fn test_deserialize_model_deleted() {
        let json = r#"{"id": "ft:gpt-4o:org:custom:id", "object": "model", "deleted": true}"#;
        let deleted: ModelDeleted = serde_json::from_str(json).unwrap();
        assert!(deleted.deleted);
        assert_eq!(deleted.id, "ft:gpt-4o:org:custom:id");
    }
}
