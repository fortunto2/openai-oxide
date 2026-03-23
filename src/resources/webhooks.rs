// Webhook signature verification — client.webhooks().verify() / unwrap()
//
// OpenAI signs webhook payloads with HMAC-SHA256. The signature is in the
// `webhook-signature` header, the timestamp in `webhook-timestamp`.

use crate::error::OpenAIError;

/// Webhook verification helper.
///
/// ```ignore
/// use openai_oxide::resources::webhooks::Webhooks;
///
/// let webhooks = Webhooks::new("whsec_YOUR_WEBHOOK_SECRET");
/// let payload = webhooks.unwrap(body, &headers)?;
/// ```
pub struct Webhooks {
    secret: Vec<u8>,
}

/// Tolerance for timestamp validation (5 minutes).
const TIMESTAMP_TOLERANCE_SECS: i64 = 300;

impl Webhooks {
    /// Create a new webhook verifier with the given secret.
    ///
    /// The secret may be prefixed with `whsec_` (will be stripped automatically).
    pub fn new(secret: &str) -> Result<Self, OpenAIError> {
        let raw = secret.strip_prefix("whsec_").unwrap_or(secret);
        let decoded = base64::Engine::decode(&base64::engine::general_purpose::STANDARD, raw)
            .map_err(|e| OpenAIError::InvalidArgument(format!("invalid webhook secret: {e}")))?;
        Ok(Self { secret: decoded })
    }

    /// Verify a webhook signature.
    ///
    /// # Arguments
    /// - `payload` — the raw request body bytes
    /// - `signature_header` — the `webhook-signature` header value (e.g. `v1,base64sig`)
    /// - `timestamp_header` — the `webhook-timestamp` header value (Unix seconds)
    ///
    /// Returns `Ok(())` if valid, or an error describing the failure.
    pub fn verify(
        &self,
        payload: &[u8],
        signature_header: &str,
        timestamp_header: &str,
    ) -> Result<(), OpenAIError> {
        use hmac::{Hmac, Mac};
        use sha2::Sha256;

        // Parse and validate timestamp
        let timestamp: i64 = timestamp_header
            .parse()
            .map_err(|_| OpenAIError::InvalidArgument("invalid webhook-timestamp header".into()))?;

        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs() as i64)
            .unwrap_or(0);

        if (now - timestamp).abs() > TIMESTAMP_TOLERANCE_SECS {
            return Err(OpenAIError::InvalidArgument(format!(
                "webhook timestamp too old or too new (delta={}s, tolerance={}s)",
                (now - timestamp).abs(),
                TIMESTAMP_TOLERANCE_SECS
            )));
        }

        // Compute expected signature: HMAC-SHA256(secret, "{msg_id}.{timestamp}.{body}")
        // OpenAI format: sign over "{timestamp}.{body}"
        let signed_content = format!(
            "{}.{}",
            timestamp_header,
            std::str::from_utf8(payload).unwrap_or("")
        );

        let mut mac = Hmac::<Sha256>::new_from_slice(&self.secret)
            .map_err(|e| OpenAIError::InvalidArgument(format!("HMAC init failed: {e}")))?;
        mac.update(signed_content.as_bytes());
        let expected = mac.finalize().into_bytes();
        let expected_b64 =
            base64::Engine::encode(&base64::engine::general_purpose::STANDARD, &expected);

        // Check each signature version in the header (comma-separated)
        for sig in signature_header.split(' ') {
            let parts: Vec<&str> = sig.splitn(2, ',').collect();
            if parts.len() == 2 && parts[0] == "v1" && parts[1] == expected_b64 {
                return Ok(());
            }
        }

        Err(OpenAIError::InvalidArgument(
            "webhook signature verification failed".into(),
        ))
    }

    /// Verify the signature and deserialize the payload.
    ///
    /// Combines [`verify`] + JSON deserialization in one call.
    pub fn unwrap<T: serde::de::DeserializeOwned>(
        &self,
        payload: &[u8],
        signature_header: &str,
        timestamp_header: &str,
    ) -> Result<T, OpenAIError> {
        self.verify(payload, signature_header, timestamp_header)?;
        serde_json::from_slice(payload).map_err(OpenAIError::from)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_webhook_verify_valid() {
        use base64::Engine;
        use hmac::{Hmac, Mac};
        use sha2::Sha256;

        let secret_raw = b"test-secret-key-bytes!!";
        let secret_b64 = base64::engine::general_purpose::STANDARD.encode(secret_raw);
        let webhook_secret = format!("whsec_{secret_b64}");

        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs()
            .to_string();

        let body = r#"{"type":"test","data":{}}"#;
        let signed_content = format!("{timestamp}.{body}");

        let mut mac = Hmac::<Sha256>::new_from_slice(secret_raw).unwrap();
        mac.update(signed_content.as_bytes());
        let sig = base64::engine::general_purpose::STANDARD.encode(mac.finalize().into_bytes());
        let sig_header = format!("v1,{sig}");

        let wh = Webhooks::new(&webhook_secret).unwrap();
        wh.verify(body.as_bytes(), &sig_header, &timestamp).unwrap();
    }

    #[test]
    fn test_webhook_verify_invalid_signature() {
        use base64::Engine;

        let secret_raw = b"test-secret-key-bytes!!";
        let secret_b64 = base64::engine::general_purpose::STANDARD.encode(secret_raw);

        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs()
            .to_string();

        let wh = Webhooks::new(&format!("whsec_{secret_b64}")).unwrap();
        let result = wh.verify(b"body", "v1,invalidsignature", &timestamp);
        assert!(result.is_err());
    }

    #[test]
    fn test_webhook_verify_expired_timestamp() {
        use base64::Engine;

        let secret_raw = b"test-secret-key-bytes!!";
        let secret_b64 = base64::engine::general_purpose::STANDARD.encode(secret_raw);

        let old_timestamp = "1000000000"; // year 2001

        let wh = Webhooks::new(&format!("whsec_{secret_b64}")).unwrap();
        let result = wh.verify(b"body", "v1,sig", old_timestamp);
        assert!(result.is_err());
        assert!(format!("{result:?}").contains("too old"));
    }

    #[test]
    fn test_webhook_unwrap() {
        use base64::Engine;
        use hmac::{Hmac, Mac};
        use sha2::Sha256;

        let secret_raw = b"unwrap-test-secret!!!!!";
        let secret_b64 = base64::engine::general_purpose::STANDARD.encode(secret_raw);

        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs()
            .to_string();

        let body = r#"{"value":42}"#;
        let signed_content = format!("{timestamp}.{body}");

        let mut mac = Hmac::<Sha256>::new_from_slice(secret_raw).unwrap();
        mac.update(signed_content.as_bytes());
        let sig = base64::engine::general_purpose::STANDARD.encode(mac.finalize().into_bytes());

        let wh = Webhooks::new(&format!("whsec_{secret_b64}")).unwrap();
        let parsed: serde_json::Value = wh
            .unwrap(body.as_bytes(), &format!("v1,{sig}"), &timestamp)
            .unwrap();
        assert_eq!(parsed["value"], 42);
    }
}
