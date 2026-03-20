// Embeddings resource — client.embeddings().create()

use crate::client::OpenAI;
use crate::error::OpenAIError;
use crate::types::embedding::{CreateEmbeddingResponse, EmbeddingRequest};

/// Access embedding endpoints.
pub struct Embeddings<'a> {
    client: &'a OpenAI,
}

impl<'a> Embeddings<'a> {
    pub(crate) fn new(client: &'a OpenAI) -> Self {
        Self { client }
    }

    /// Create embeddings with a custom request type, returning raw JSON.
    ///
    /// Use this when you need to send fields not yet in [`EmbeddingRequest`]
    /// or want to work with the raw API response.
    ///
    /// ```ignore
    /// use serde_json::json;
    ///
    /// let raw = client.embeddings().create_raw(&json!({
    ///     "model": "text-embedding-3-small",
    ///     "input": "Hello world",
    ///     "custom_field": true
    /// })).await?;
    /// println!("{:?}", raw["data"][0]["embedding"]);
    /// ```
    pub async fn create_raw(
        &self,
        request: &impl serde::Serialize,
    ) -> Result<serde_json::Value, crate::error::OpenAIError> {
        self.client.post_json("/embeddings", request).await
    }

    /// Create embeddings.
    ///
    /// `POST /embeddings`
    pub async fn create(
        &self,
        request: EmbeddingRequest,
    ) -> Result<CreateEmbeddingResponse, OpenAIError> {
        self.client.post("/embeddings", &request).await
    }
}

#[cfg(test)]
mod tests {
    use crate::OpenAI;
    use crate::config::ClientConfig;
    use crate::types::embedding::EmbeddingRequest;

    #[tokio::test]
    async fn test_embeddings_create_raw() {
        let mut server = mockito::Server::new_async().await;
        let mock = server
            .mock("POST", "/embeddings")
            .match_header("authorization", "Bearer sk-test")
            .match_body(mockito::Matcher::Json(serde_json::json!({
                "model": "text-embedding-3-small",
                "input": "Hello world",
                "custom_dim": 256
            })))
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(r#"{"object":"list","data":[{"embedding":[0.1,0.2],"index":0}],"custom_resp":true}"#)
            .create_async()
            .await;

        let client = OpenAI::with_config(ClientConfig::new("sk-test").base_url(server.url()));

        let raw = client
            .embeddings()
            .create_raw(&serde_json::json!({
                "model": "text-embedding-3-small",
                "input": "Hello world",
                "custom_dim": 256
            }))
            .await
            .unwrap();

        assert_eq!(raw["object"], "list");
        assert_eq!(raw["custom_resp"], true);
        assert_eq!(raw["data"][0]["embedding"][0], 0.1);
        mock.assert_async().await;
    }

    #[tokio::test]
    async fn test_embeddings_create() {
        let mut server = mockito::Server::new_async().await;
        let mock = server
            .mock("POST", "/embeddings")
            .match_header("authorization", "Bearer sk-test")
            .match_header("content-type", "application/json")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(
                r#"{
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
                }"#,
            )
            .create_async()
            .await;

        let client = OpenAI::with_config(ClientConfig::new("sk-test").base_url(server.url()));
        let request = EmbeddingRequest::new("text-embedding-3-small", "Hello world");

        let response = client.embeddings().create(request).await.unwrap();
        assert_eq!(response.object, "list");
        assert_eq!(response.model, "text-embedding-3-small");
        assert_eq!(response.data.len(), 1);
        assert_eq!(response.data[0].embedding.len(), 3);
        mock.assert_async().await;
    }
}
