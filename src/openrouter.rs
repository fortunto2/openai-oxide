// OpenRouter helpers.
//
// OpenRouter is OpenAI-compatible at `https://openrouter.ai/api/v1`.
// These helpers set the correct base URL, app attribution headers,
// and provide a typed `ProviderPreferences` for request body injection.
//
// Docs: <https://openrouter.ai/docs/quickstart>

use reqwest::header::{HeaderMap, HeaderName, HeaderValue};
use serde::Serialize;

use crate::config::ClientConfig;
use crate::error::OpenAIError;

/// Build a `ClientConfig` for OpenRouter.
///
/// ```ignore
/// let client = OpenAI::with_config(
///     openrouter::config("sk-or-...", None)?
/// );
/// // model: "anthropic/claude-sonnet-4-6"
/// ```
///
/// With app attribution (shows your app on openrouter.ai rankings):
/// ```ignore
/// let client = OpenAI::with_config(
///     openrouter::config("sk-or-...", Some(openrouter::App {
///         name: "my-agent",
///         url: "https://github.com/me/my-agent",
///     }))?
/// );
/// ```
pub fn config(api_key: &str, app: Option<App<'_>>) -> Result<ClientConfig, OpenAIError> {
    let mut cfg = ClientConfig::new(api_key).base_url("https://openrouter.ai/api/v1");

    if let Some(app) = app {
        let mut headers = HeaderMap::with_capacity(2);
        if !app.url.is_empty() {
            set(&mut headers, "http-referer", app.url)?;
        }
        if !app.name.is_empty() {
            set(&mut headers, "x-openrouter-title", app.name)?;
        }
        if !headers.is_empty() {
            cfg = cfg.default_headers(headers);
        }
    }

    Ok(cfg)
}

/// App attribution for OpenRouter rankings.
pub struct App<'a> {
    pub name: &'a str,
    pub url: &'a str,
}

/// Provider preferences for OpenRouter routing.
///
/// Add this to your request body via `serde_json::Value` merging,
/// or use `inject_provider()` to patch a request.
///
/// For long-running agents, pin a provider to avoid mid-session switches:
/// ```ignore
/// let prefs = openrouter::ProviderPreferences::pinned("anthropic");
/// ```
///
/// Docs: <https://openrouter.ai/docs/guides/routing/provider-selection>
#[derive(Debug, Clone, Serialize, Default)]
pub struct ProviderPreferences {
    /// Provider priority order. First available wins.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub order: Option<Vec<String>>,

    /// Only use these providers. Fails if none available.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub only: Option<Vec<String>>,

    /// Exclude these providers.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ignore: Option<Vec<String>>,

    /// Allow fallback to other providers. Default: true.
    /// Set to false for agent consistency.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub allow_fallbacks: Option<bool>,

    /// Only use providers that support all request parameters.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub require_parameters: Option<bool>,

    /// Quantization filter: "fp8", "int8", "bf16", etc.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub quantizations: Option<Vec<String>>,

    /// Sort by: "throughput", "price", "latency".
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sort: Option<String>,

    /// Zero Data Retention.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub zdr: Option<bool>,

    /// Data collection policy: "allow" or "deny".
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data_collection: Option<String>,
}

impl ProviderPreferences {
    /// Pin to a single provider. No fallbacks, no switching mid-session.
    ///
    /// ```ignore
    /// let prefs = openrouter::ProviderPreferences::pinned("anthropic");
    /// ```
    pub fn pinned(provider: &str) -> Self {
        Self {
            only: Some(vec![provider.to_string()]),
            allow_fallbacks: Some(false),
            require_parameters: Some(true),
            ..Default::default()
        }
    }

    /// Prefer providers in order, but allow fallback.
    pub fn prefer(providers: &[&str]) -> Self {
        Self {
            order: Some(providers.iter().map(|s| s.to_string()).collect()),
            require_parameters: Some(true),
            ..Default::default()
        }
    }

    /// Optimized for throughput (fast inference).
    pub fn fast() -> Self {
        Self {
            sort: Some("throughput".to_string()),
            require_parameters: Some(true),
            ..Default::default()
        }
    }

    /// Optimized for lowest price.
    pub fn cheap() -> Self {
        Self {
            sort: Some("price".to_string()),
            require_parameters: Some(true),
            ..Default::default()
        }
    }

    /// Serialize as JSON Value for merging into request body.
    pub fn to_value(&self) -> Result<serde_json::Value, OpenAIError> {
        serde_json::to_value(self).map_err(|e| {
            OpenAIError::InvalidArgument(format!("failed to serialize provider preferences: {e}"))
        })
    }
}

/// Inject `provider` preferences into a JSON request body.
///
/// ```ignore
/// let mut body = serde_json::to_value(&request)?;
/// openrouter::inject_provider(&mut body, &ProviderPreferences::pinned("anthropic"))?;
/// // body now has "provider": { "only": ["anthropic"], "allow_fallbacks": false, ... }
/// ```
pub fn inject_provider(
    body: &mut serde_json::Value,
    prefs: &ProviderPreferences,
) -> Result<(), OpenAIError> {
    let provider_value = prefs.to_value()?;
    if let serde_json::Value::Object(map) = body {
        map.insert("provider".to_string(), provider_value);
    }
    Ok(())
}

fn set(headers: &mut HeaderMap, name: &'static str, value: &str) -> Result<(), OpenAIError> {
    headers.insert(
        HeaderName::from_static(name),
        HeaderValue::from_str(value)
            .map_err(|e| OpenAIError::InvalidArgument(format!("invalid header {name}: {e}")))?,
    );
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_base_url() {
        let cfg = config("sk-or-test", None).unwrap();
        assert_eq!(cfg.base_url, "https://openrouter.ai/api/v1");
    }

    #[test]
    fn test_bearer_auth() {
        let cfg = config("sk-or-test", None).unwrap();
        assert_eq!(cfg.api_key, "sk-or-test");
    }

    #[test]
    fn test_app_headers() {
        let cfg = config(
            "sk-or-test",
            Some(App {
                name: "my-agent",
                url: "https://example.com",
            }),
        )
        .unwrap();
        let headers = cfg.default_headers.as_ref().unwrap();
        assert_eq!(headers.get("http-referer").unwrap(), "https://example.com");
        assert_eq!(headers.get("x-openrouter-title").unwrap(), "my-agent");
    }

    #[test]
    fn test_no_app_no_headers() {
        let cfg = config("sk-or-test", None).unwrap();
        assert!(cfg.default_headers.is_none());
    }

    #[test]
    fn test_pinned_provider() {
        let prefs = ProviderPreferences::pinned("anthropic");
        let v = prefs.to_value().unwrap();
        assert_eq!(v["only"], serde_json::json!(["anthropic"]));
        assert_eq!(v["allow_fallbacks"], serde_json::json!(false));
        assert_eq!(v["require_parameters"], serde_json::json!(true));
        assert!(v.get("order").is_none());
    }

    #[test]
    fn test_prefer_providers() {
        let prefs = ProviderPreferences::prefer(&["together", "openai"]);
        let v = prefs.to_value().unwrap();
        assert_eq!(v["order"], serde_json::json!(["together", "openai"]));
        assert!(v.get("only").is_none());
    }

    #[test]
    fn test_inject_provider() {
        let mut body = serde_json::json!({
            "model": "anthropic/claude-sonnet-4-6",
            "messages": [{"role": "user", "content": "hi"}]
        });
        inject_provider(&mut body, &ProviderPreferences::pinned("anthropic")).unwrap();
        assert_eq!(body["provider"]["only"], serde_json::json!(["anthropic"]));
        assert_eq!(
            body["provider"]["allow_fallbacks"],
            serde_json::json!(false)
        );
        // original fields preserved
        assert_eq!(body["model"], "anthropic/claude-sonnet-4-6");
    }

    #[test]
    fn test_fast_sort() {
        let prefs = ProviderPreferences::fast();
        let v = prefs.to_value().unwrap();
        assert_eq!(v["sort"], "throughput");
    }
}
