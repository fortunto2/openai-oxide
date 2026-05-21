// Realtime resource (GA) — ephemeral client secret issuance.
//
// OpenAI guide: <https://platform.openai.com/docs/guides/realtime>
// API reference: <https://platform.openai.com/docs/api-reference/realtime-client-secrets>
//
// Note: the legacy beta endpoints (`/v1/realtime/sessions`,
// `/v1/realtime/transcription_sessions`) were retired by OpenAI on
// 2026-05-21 with the response `beta_api_shape_disabled`. New code should
// use `client.realtime().client_secrets().create()` here; the beta
// resource under `client.beta().realtime()` is kept for legacy callers
// (and will return an error from the server).

use crate::client::OpenAI;
use crate::error::OpenAIError;
use crate::types::realtime::{ClientSecretCreateParams, ClientSecretCreateResponse};

/// Access realtime GA endpoints.
///
/// API reference: <https://platform.openai.com/docs/api-reference/realtime-client-secrets>
pub struct Realtime<'a> {
    client: &'a OpenAI,
}

impl<'a> Realtime<'a> {
    pub(crate) fn new(client: &'a OpenAI) -> Self {
        Self { client }
    }

    /// Issue ephemeral client secrets for realtime / transcription sessions.
    pub fn client_secrets(&self) -> ClientSecrets<'_> {
        ClientSecrets {
            client: self.client,
        }
    }
}

/// Realtime client secrets endpoint.
///
/// Mints short-lived ephemeral tokens (`ek_…`) so end-user clients can
/// open a Realtime session without ever holding the master API key.
pub struct ClientSecrets<'a> {
    client: &'a OpenAI,
}

impl<'a> ClientSecrets<'a> {
    /// Create a client secret for a realtime or transcription session.
    ///
    /// `POST /realtime/client_secrets`
    ///
    /// Returns an ephemeral token (`value`, prefixed with `ek_`) plus the
    /// echoed session config the token was bound to. The token expires at
    /// `expires_at` (unix seconds).
    pub async fn create(
        &self,
        params: &ClientSecretCreateParams,
    ) -> Result<ClientSecretCreateResponse, OpenAIError> {
        self.client.post("/realtime/client_secrets", params).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::OpenAI;
    use crate::config::ClientConfig;
    use serde_json::json;

    #[tokio::test]
    async fn client_secret_create_transcription() {
        let mut server = mockito::Server::new_async().await;
        let mock = server
            .mock("POST", "/realtime/client_secrets")
            .match_header("authorization", "Bearer sk-test")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(
                r#"{
                    "value": "ek_abc123",
                    "expires_at": 1779000000,
                    "session": {
                        "type": "transcription",
                        "object": "realtime.transcription_session",
                        "id": "sess_xyz"
                    }
                }"#,
            )
            .create_async()
            .await;

        let client = OpenAI::with_config(ClientConfig::new("sk-test").base_url(server.url()));

        // We send the body as untyped JSON via the Session = serde_json::Value alias.
        let session = json!({
            "type": "transcription",
            "audio": {
                "input": {
                    "format": { "type": "audio/pcm", "rate": 24000 },
                    "transcription": { "model": "gpt-4o-transcribe", "language": "en" }
                }
            }
        });
        let params = ClientSecretCreateParams {
            session: Some(session),
            expires_after: None,
        };
        let response = client
            .realtime()
            .client_secrets()
            .create(&params)
            .await
            .unwrap();

        assert_eq!(response.value, "ek_abc123");
        assert_eq!(response.expires_at, 1779000000);
        mock.assert_async().await;
    }
}
