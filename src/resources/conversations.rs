// Conversations resource — server-side multi-turn conversation state.
//
// OpenAI guide: <https://platform.openai.com/docs/guides/conversational-agents/conversations-api>
// API reference: <https://platform.openai.com/docs/api-reference/conversations>

use crate::client::OpenAI;
use crate::error::OpenAIError;
use crate::types::conversations::{
    Conversation, ConversationDeletedResource, ConversationItem, ConversationItemList,
};

/// Access conversation endpoints for persistent multi-turn state.
///
/// API reference: <https://platform.openai.com/docs/api-reference/conversations>
pub struct Conversations<'a> {
    client: &'a OpenAI,
}

impl<'a> Conversations<'a> {
    pub(crate) fn new(client: &'a OpenAI) -> Self {
        Self { client }
    }

    /// Create a conversation.
    ///
    /// `POST /conversations`
    pub async fn create(&self, body: &impl serde::Serialize) -> Result<Conversation, OpenAIError> {
        self.client.post("/conversations", body).await
    }

    /// Retrieve a conversation.
    ///
    /// `GET /conversations/{conversation_id}`
    pub async fn retrieve(&self, conversation_id: &str) -> Result<Conversation, OpenAIError> {
        self.client
            .get(&format!("/conversations/{conversation_id}"))
            .await
    }

    /// Update a conversation (e.g. metadata).
    ///
    /// `POST /conversations/{conversation_id}`
    pub async fn update(
        &self,
        conversation_id: &str,
        body: &impl serde::Serialize,
    ) -> Result<Conversation, OpenAIError> {
        self.client
            .post(&format!("/conversations/{conversation_id}"), body)
            .await
    }

    /// Delete a conversation.
    ///
    /// `DELETE /conversations/{conversation_id}`
    pub async fn delete(
        &self,
        conversation_id: &str,
    ) -> Result<ConversationDeletedResource, OpenAIError> {
        self.client
            .delete(&format!("/conversations/{conversation_id}"))
            .await
    }

    /// List items in a conversation.
    ///
    /// `GET /conversations/{conversation_id}/items`
    pub async fn list_items(
        &self,
        conversation_id: &str,
    ) -> Result<ConversationItemList, OpenAIError> {
        self.client
            .get(&format!("/conversations/{conversation_id}/items"))
            .await
    }

    /// Create items in a conversation (append messages, tool calls, etc.).
    ///
    /// `POST /conversations/{conversation_id}/items`
    pub async fn create_items(
        &self,
        conversation_id: &str,
        body: &impl serde::Serialize,
    ) -> Result<ConversationItemList, OpenAIError> {
        self.client
            .post(&format!("/conversations/{conversation_id}/items"), body)
            .await
    }

    /// Retrieve a single item from a conversation.
    ///
    /// `GET /conversations/{conversation_id}/items/{item_id}`
    pub async fn retrieve_item(
        &self,
        conversation_id: &str,
        item_id: &str,
    ) -> Result<ConversationItem, OpenAIError> {
        self.client
            .get(&format!("/conversations/{conversation_id}/items/{item_id}"))
            .await
    }

    /// Delete an item from a conversation.
    ///
    /// `DELETE /conversations/{conversation_id}/items/{item_id}`
    pub async fn delete_item(
        &self,
        conversation_id: &str,
        item_id: &str,
    ) -> Result<Conversation, OpenAIError> {
        self.client
            .delete(&format!("/conversations/{conversation_id}/items/{item_id}"))
            .await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn conversation_deserializes() {
        let json = r#"{
            "id": "conv_123",
            "object": "conversation",
            "created_at": 1741900000,
            "metadata": {"topic": "demo"}
        }"#;
        let conversation: Conversation = serde_json::from_str(json).unwrap();
        assert_eq!(conversation.id, "conv_123");
        assert_eq!(conversation.metadata["topic"], "demo");
    }

    #[test]
    fn deleted_resource_deserializes() {
        let json = r#"{"id": "conv_123", "object": "conversation.deleted", "deleted": true}"#;
        let deleted: ConversationDeletedResource = serde_json::from_str(json).unwrap();
        assert!(deleted.deleted);
    }

    #[test]
    fn item_list_deserializes_and_keeps_items_open() {
        // Items are a large open union upstream, so they stay `serde_json::Value`
        // — the envelope is what callers were hand-parsing.
        let json = r#"{
            "object": "list",
            "data": [
                {"type": "message", "id": "msg_1", "role": "user",
                 "content": [{"type": "input_text", "text": "hi"}]}
            ],
            "first_id": "msg_1",
            "last_id": "msg_1",
            "has_more": false
        }"#;
        let list: ConversationItemList = serde_json::from_str(json).unwrap();
        assert!(!list.has_more);
        assert_eq!(list.data.len(), 1);
        assert_eq!(list.data[0]["id"], "msg_1");
    }

    #[test]
    fn unknown_fields_do_not_break_deserialization() {
        let json = r#"{
            "id": "conv_123",
            "object": "conversation",
            "created_at": 1741900000,
            "metadata": {},
            "some_future_field": 42
        }"#;
        assert!(serde_json::from_str::<Conversation>(json).is_ok());
    }
}
