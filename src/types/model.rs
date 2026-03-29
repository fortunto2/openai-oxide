// Model types — re-exported from openai-types

pub use openai_types::model::{Model, ModelDeleted};
pub use openai_types::shared::ListResponse;

/// Response from listing models.
pub type ModelList = ListResponse<Model>;

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
