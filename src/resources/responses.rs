// Responses resource — client.responses().create() / retrieve() / delete()

use crate::client::OpenAI;
use crate::error::OpenAIError;
use crate::streaming::SseStream;
use crate::types::responses::{Response, ResponseCreateRequest, ResponseStreamEvent};

/// Access the Responses API endpoints.
pub struct Responses<'a> {
    client: &'a OpenAI,
}

impl<'a> Responses<'a> {
    pub(crate) fn new(client: &'a OpenAI) -> Self {
        Self { client }
    }

    /// Create a response.
    ///
    /// `POST /responses`
    pub async fn create(&self, request: ResponseCreateRequest) -> Result<Response, OpenAIError> {
        self.client.post("/responses", &request).await
    }

    /// Create a streaming response.
    ///
    /// Returns a `Stream<Item = Result<ResponseStreamEvent>>`.
    /// The `stream` field in the request is automatically set to `true`.
    pub async fn create_stream(
        &self,
        mut request: ResponseCreateRequest,
    ) -> Result<SseStream<ResponseStreamEvent>, OpenAIError> {
        request.stream = Some(true);
        let response = self
            .client
            .request(reqwest::Method::POST, "/responses")
            .json(&request)
            .send()
            .await?;

        let status = response.status();
        if !status.is_success() {
            let status_code = status.as_u16();
            let body = response.text().await.unwrap_or_default();
            if let Ok(error_resp) = serde_json::from_str::<crate::error::ErrorResponse>(&body) {
                return Err(OpenAIError::ApiError {
                    status: status_code,
                    message: error_resp.error.message,
                    type_: error_resp.error.type_,
                    code: error_resp.error.code,
                });
            }
            return Err(OpenAIError::ApiError {
                status: status_code,
                message: body,
                type_: None,
                code: None,
            });
        }

        Ok(SseStream::new(response))
    }

    /// Retrieve a response by ID.
    ///
    /// `GET /responses/{response_id}`
    pub async fn retrieve(&self, response_id: &str) -> Result<Response, OpenAIError> {
        self.client.get(&format!("/responses/{response_id}")).await
    }

    /// Delete a response.
    ///
    /// `DELETE /responses/{response_id}`
    pub async fn delete(&self, response_id: &str) -> Result<(), OpenAIError> {
        let response = self
            .client
            .request(
                reqwest::Method::DELETE,
                &format!("/responses/{response_id}"),
            )
            .send()
            .await?;

        let status = response.status();
        if status.is_success() {
            Ok(())
        } else {
            let status_code = status.as_u16();
            let body = response.text().await.unwrap_or_default();
            if let Ok(error_resp) = serde_json::from_str::<crate::error::ErrorResponse>(&body) {
                Err(OpenAIError::ApiError {
                    status: status_code,
                    message: error_resp.error.message,
                    type_: error_resp.error.type_,
                    code: error_resp.error.code,
                })
            } else {
                Err(OpenAIError::ApiError {
                    status: status_code,
                    message: body,
                    type_: None,
                    code: None,
                })
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::OpenAI;
    use crate::config::ClientConfig;
    use crate::types::responses::ResponseCreateRequest;

    const RESPONSE_JSON: &str = r#"{
        "id": "resp-abc123",
        "object": "response",
        "created_at": 1677610602.0,
        "model": "gpt-4o",
        "output": [{
            "type": "message",
            "id": "msg-abc123",
            "role": "assistant",
            "status": "completed",
            "content": [{
                "type": "output_text",
                "text": "Hello!",
                "annotations": []
            }]
        }],
        "status": "completed",
        "usage": {
            "input_tokens": 10,
            "output_tokens": 2,
            "total_tokens": 12
        }
    }"#;

    #[tokio::test]
    async fn test_responses_create() {
        let mut server = mockito::Server::new_async().await;
        let mock = server
            .mock("POST", "/responses")
            .match_header("authorization", "Bearer sk-test")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(RESPONSE_JSON)
            .create_async()
            .await;

        let client = OpenAI::with_config(ClientConfig::new("sk-test").base_url(server.url()));
        let mut request = ResponseCreateRequest::new("gpt-4o");
        request.input = Some("Hello".into());

        let response = client.responses().create(request).await.unwrap();
        assert_eq!(response.id, "resp-abc123");
        assert_eq!(response.output_text(), "Hello!");
        mock.assert_async().await;
    }

    #[tokio::test]
    async fn test_responses_retrieve() {
        let mut server = mockito::Server::new_async().await;
        let mock = server
            .mock("GET", "/responses/resp-abc123")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(RESPONSE_JSON)
            .create_async()
            .await;

        let client = OpenAI::with_config(ClientConfig::new("sk-test").base_url(server.url()));
        let response = client.responses().retrieve("resp-abc123").await.unwrap();
        assert_eq!(response.id, "resp-abc123");
        mock.assert_async().await;
    }

    #[tokio::test]
    async fn test_responses_create_with_tools() {
        use crate::types::responses::{Reasoning, ResponseTool};

        let mut server = mockito::Server::new_async().await;
        let mock = server
            .mock("POST", "/responses")
            .match_header("authorization", "Bearer sk-test")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(RESPONSE_JSON)
            .create_async()
            .await;

        let client = OpenAI::with_config(ClientConfig::new("sk-test").base_url(server.url()));
        let mut request = ResponseCreateRequest::new("gpt-4o");
        request.input = Some("Search for Rust".into());
        request.tools = Some(vec![ResponseTool::WebSearch {
            search_context_size: Some("medium".into()),
            user_location: None,
        }]);
        request.reasoning = Some(Reasoning {
            effort: Some("high".into()),
            summary: None,
        });
        request.truncation = Some("auto".into());

        let response = client.responses().create(request).await.unwrap();
        assert_eq!(response.id, "resp-abc123");
        mock.assert_async().await;
    }

    #[tokio::test]
    async fn test_responses_delete() {
        let mut server = mockito::Server::new_async().await;
        let mock = server
            .mock("DELETE", "/responses/resp-abc123")
            .with_status(200)
            .with_body("")
            .create_async()
            .await;

        let client = OpenAI::with_config(ClientConfig::new("sk-test").base_url(server.url()));
        client.responses().delete("resp-abc123").await.unwrap();
        mock.assert_async().await;
    }
}
