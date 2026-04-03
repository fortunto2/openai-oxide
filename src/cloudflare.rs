// Cloudflare Workers AI client configuration.
//
// Provides `CloudflareConfig` builder for constructing an `OpenAI` client that
// targets Cloudflare Workers AI. Uses the OpenAI-compatible endpoint at
// `https://api.cloudflare.com/client/v4/accounts/{account_id}/ai/v1`.
//
// Key features:
// - `x-session-affinity` header for prefix caching (routes to same instance)
// - Standard Bearer token auth with Cloudflare API token
// - OpenAI guide: <https://developers.cloudflare.com/workers-ai/configuration/open-ai-compatibility/>

use reqwest::header::{HeaderMap, HeaderValue};
use std::env;

use crate::client::OpenAI;
use crate::config::ClientConfig;
use crate::error::OpenAIError;

const SESSION_AFFINITY_HEADER: &str = "x-session-affinity";

/// Configuration builder for Cloudflare Workers AI.
///
/// Cloudflare Workers AI provides an OpenAI-compatible API at
/// `/client/v4/accounts/{account_id}/ai/v1`. It supports prefix caching
/// via the `x-session-affinity` header, which routes requests to the same
/// model instance for better cache hit rates.
///
/// # Examples
///
/// ```ignore
/// use openai_oxide::{OpenAI, CloudflareConfig};
///
/// let client = OpenAI::cloudflare(
///     CloudflareConfig::new("account-id", "cf-api-token")
/// )?;
///
/// // Use any Workers AI model with the standard OpenAI API
/// let response = client.chat().completions().create(
///     ChatCompletionRequest::new(
///         "@cf/meta/llama-3.3-70b-instruct-fp8-fast",
///         vec![/* messages */],
///     )
/// ).await?;
/// ```
///
/// With session affinity for prefix caching:
///
/// ```ignore
/// let client = OpenAI::cloudflare(
///     CloudflareConfig::new("account-id", "cf-api-token")
///         .session_affinity("my-agent-session-123")
/// )?;
/// // All requests routed to the same instance — cached prefixes reused
/// ```
#[derive(Debug, Clone)]
pub struct CloudflareConfig {
    /// Cloudflare account ID.
    pub account_id: String,

    /// Cloudflare API token (Bearer auth).
    pub api_token: String,

    /// Session affinity key for prefix caching.
    /// Routes requests to the same model instance.
    pub session_affinity: Option<String>,

    /// Custom gateway ID for AI Gateway (optional).
    /// Changes base URL to use AI Gateway endpoint.
    pub gateway_id: Option<String>,
}

impl CloudflareConfig {
    /// Create a new Cloudflare Workers AI configuration.
    ///
    /// # Arguments
    /// - `account_id` — your Cloudflare account ID
    /// - `api_token` — Cloudflare API token with Workers AI permissions
    #[must_use]
    pub fn new(account_id: impl Into<String>, api_token: impl Into<String>) -> Self {
        Self {
            account_id: account_id.into(),
            api_token: api_token.into(),
            session_affinity: None,
            gateway_id: None,
        }
    }

    /// Set session affinity key for prefix caching.
    ///
    /// Routes all requests to the same model instance, improving cache hit
    /// rates for prefix caching. Use a unique string per agent session or
    /// conversation. This can reduce TTFT and cost for multi-turn interactions.
    ///
    /// See: <https://blog.cloudflare.com/workers-ai-large-models/>
    #[must_use]
    pub fn session_affinity(mut self, key: impl Into<String>) -> Self {
        self.session_affinity = Some(key.into());
        self
    }

    /// Set AI Gateway ID for request logging and caching.
    ///
    /// When set, the base URL uses the AI Gateway endpoint instead of the
    /// direct Workers AI endpoint.
    #[must_use]
    pub fn gateway_id(mut self, id: impl Into<String>) -> Self {
        self.gateway_id = Some(id.into());
        self
    }

    /// Build an `OpenAI` client from this Cloudflare configuration.
    pub fn build(self) -> Result<OpenAI, OpenAIError> {
        let base_url = match &self.gateway_id {
            Some(gw) => format!(
                "https://gateway.ai.cloudflare.com/v1/{}/{}/openai",
                self.account_id, gw
            ),
            None => format!(
                "https://api.cloudflare.com/client/v4/accounts/{}/ai/v1",
                self.account_id
            ),
        };

        let mut config = ClientConfig::new(&self.api_token).base_url(base_url);

        if let Some(ref session_key) = self.session_affinity {
            let mut headers = HeaderMap::with_capacity(1);
            headers.insert(
                SESSION_AFFINITY_HEADER,
                HeaderValue::from_str(session_key).map_err(|e| {
                    OpenAIError::InvalidArgument(format!("invalid session affinity key: {e}"))
                })?,
            );
            config = config.default_headers(headers);
        }

        Ok(OpenAI::with_config(config))
    }

    /// Build an `OpenAI` client from environment variables.
    ///
    /// Reads:
    /// - `CLOUDFLARE_ACCOUNT_ID` — Cloudflare account ID
    /// - `CLOUDFLARE_API_TOKEN` — Cloudflare API token
    /// - `CLOUDFLARE_SESSION_AFFINITY` — optional session affinity key
    /// - `CLOUDFLARE_GATEWAY_ID` — optional AI Gateway ID
    pub fn from_env() -> Result<OpenAI, OpenAIError> {
        let account_id = env::var("CLOUDFLARE_ACCOUNT_ID").map_err(|_| {
            OpenAIError::InvalidArgument(
                "CLOUDFLARE_ACCOUNT_ID environment variable not set".to_string(),
            )
        })?;

        let api_token = env::var("CLOUDFLARE_API_TOKEN").map_err(|_| {
            OpenAIError::InvalidArgument(
                "CLOUDFLARE_API_TOKEN environment variable not set".to_string(),
            )
        })?;

        let mut config = Self::new(account_id, api_token);

        if let Ok(session) = env::var("CLOUDFLARE_SESSION_AFFINITY") {
            config = config.session_affinity(session);
        }

        if let Ok(gw) = env::var("CLOUDFLARE_GATEWAY_ID") {
            config = config.gateway_id(gw);
        }

        config.build()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cloudflare_base_url() {
        let client = CloudflareConfig::new("abc123", "cf-token").build().unwrap();

        assert_eq!(
            client.config.base_url(),
            "https://api.cloudflare.com/client/v4/accounts/abc123/ai/v1"
        );
    }

    #[test]
    fn test_cloudflare_gateway_url() {
        let client = CloudflareConfig::new("abc123", "cf-token")
            .gateway_id("my-gateway")
            .build()
            .unwrap();

        assert_eq!(
            client.config.base_url(),
            "https://gateway.ai.cloudflare.com/v1/abc123/my-gateway/openai"
        );
    }

    #[test]
    fn test_cloudflare_session_affinity_header() {
        let client = CloudflareConfig::new("abc123", "cf-token")
            .session_affinity("ses_12345")
            .build()
            .unwrap();

        let headers = client.config.default_headers().unwrap();
        assert_eq!(headers.get(SESSION_AFFINITY_HEADER).unwrap(), "ses_12345");
    }

    #[test]
    fn test_cloudflare_no_session_affinity() {
        let client = CloudflareConfig::new("abc123", "cf-token").build().unwrap();

        assert!(client.config.default_headers().is_none());
    }

    #[test]
    fn test_cloudflare_bearer_auth() {
        let client = CloudflareConfig::new("abc123", "cf-token").build().unwrap();

        assert_eq!(client.config.api_key(), "cf-token");
    }

    /// E2E test: build through CloudflareConfig, verify headers reach the server.
    ///
    /// We build via CloudflareConfig to get the correct default_headers, then
    /// reconstruct with the mock server URL (CloudflareConfig hardcodes the
    /// Cloudflare domain, so we extract the headers it produced).
    #[tokio::test]
    async fn test_cloudflare_e2e_session_affinity() {
        // 1. Build through CloudflareConfig to get the real headers
        let cf_client = CloudflareConfig::new("test-account", "cf-token")
            .session_affinity("agent-42")
            .build()
            .unwrap();
        let built_headers = cf_client.config.default_headers().unwrap().clone();

        // 2. Set up mock server
        let mut server = mockito::Server::new_async().await;
        let mock = server
            .mock("POST", "/ai/v1/chat/completions")
            .match_header(SESSION_AFFINITY_HEADER, "agent-42")
            .match_header("authorization", "Bearer cf-token")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(
                r#"{
                "id": "chatcmpl-cf-123",
                "object": "chat.completion",
                "created": 1700000000,
                "model": "@cf/meta/llama-3.3-70b-instruct-fp8-fast",
                "choices": [{
                    "index": 0,
                    "message": {
                        "role": "assistant",
                        "content": "Hello from Cloudflare!"
                    },
                    "finish_reason": "stop"
                }],
                "usage": {
                    "prompt_tokens": 10,
                    "completion_tokens": 5,
                    "total_tokens": 15
                }
            }"#,
            )
            .create_async()
            .await;

        // 3. Rebuild with mock URL but same headers from CloudflareConfig
        let config = ClientConfig::new("cf-token")
            .base_url(format!("{}/ai/v1", server.url()))
            .default_headers(built_headers);
        let client = OpenAI::with_config(config);

        use crate::types::chat::{ChatCompletionMessageParam, ChatCompletionRequest, UserContent};

        let request = ChatCompletionRequest::new(
            "@cf/meta/llama-3.3-70b-instruct-fp8-fast",
            vec![ChatCompletionMessageParam::User {
                content: UserContent::Text("Hello!".into()),
                name: None,
            }],
        );

        let response = client.chat().completions().create(request).await.unwrap();
        assert_eq!(
            response.choices[0].message.content.as_deref().unwrap(),
            "Hello from Cloudflare!"
        );
        mock.assert_async().await;
    }
}
