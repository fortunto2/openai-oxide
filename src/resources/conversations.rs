// Conversations resource — server-side multi-turn conversation state.
//
// OpenAI guide: <https://platform.openai.com/docs/guides/conversational-agents/conversations-api>
// API reference: <https://platform.openai.com/docs/api-reference/conversations>

use crate::client::OpenAI;
use crate::error::OpenAIError;

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
    pub async fn create(
        &self,
        body: &impl serde::Serialize,
    ) -> Result<serde_json::Value, OpenAIError> {
        self.client.post("/conversations", body).await
    }

    /// Retrieve a conversation.
    ///
    /// `GET /conversations/{conversation_id}`
    pub async fn retrieve(&self, conversation_id: &str) -> Result<serde_json::Value, OpenAIError> {
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
    ) -> Result<serde_json::Value, OpenAIError> {
        self.client
            .post(&format!("/conversations/{conversation_id}"), body)
            .await
    }

    /// Delete a conversation.
    ///
    /// `DELETE /conversations/{conversation_id}`
    pub async fn delete(&self, conversation_id: &str) -> Result<serde_json::Value, OpenAIError> {
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
    ) -> Result<serde_json::Value, OpenAIError> {
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
    ) -> Result<serde_json::Value, OpenAIError> {
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
    ) -> Result<serde_json::Value, OpenAIError> {
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
    ) -> Result<serde_json::Value, OpenAIError> {
        self.client
            .delete(&format!("/conversations/{conversation_id}/items/{item_id}"))
            .await
    }
}
