// Azure OpenAI client configuration.
//
// Provides `AzureConfig` builder for constructing an `OpenAI` client that
// targets Azure OpenAI deployments. Matches the Python SDK's `AzureOpenAI`
// constructor pattern.

use std::env;

use crate::client::OpenAI;
use crate::config::ClientConfig;
use crate::error::OpenAIError;

/// Default Azure API version.
const DEFAULT_API_VERSION: &str = "2024-10-21";

/// Configuration builder for Azure OpenAI deployments.
///
/// Azure OpenAI uses different URL construction and authentication compared
/// to the standard OpenAI API:
/// - URL: `{endpoint}/openai/deployments/{deployment}` or `{endpoint}/openai`
/// - Auth: `api-key` header (not `Authorization: Bearer`)
/// - Query: `api-version` parameter on every request
///
/// # Examples
///
/// ```ignore
/// use openai_oxide::{OpenAI, AzureConfig};
///
/// let client = OpenAI::azure(
///     AzureConfig::new()
///         .azure_endpoint("https://my-resource.openai.azure.com")
///         .azure_deployment("gpt-4")
///         .api_version("2024-10-21")
///         .api_key("my-azure-api-key")
/// )?;
///
/// // All resources work the same as with the standard client
/// let response = client.chat().completions().create(request).await?;
/// ```
#[derive(Debug, Clone, Default)]
pub struct AzureConfig {
    /// Azure endpoint URL, e.g. `https://my-resource.openai.azure.com`.
    pub azure_endpoint: Option<String>,

    /// Azure deployment name, e.g. `gpt-4`.
    pub azure_deployment: Option<String>,

    /// Azure API version, e.g. `2024-10-21`.
    pub api_version: Option<String>,

    /// Azure API key (mutually exclusive with `azure_ad_token`).
    pub api_key: Option<String>,

    /// Azure AD token for authentication (mutually exclusive with `api_key`).
    pub azure_ad_token: Option<String>,
}

impl AzureConfig {
    /// Create a new empty Azure configuration.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the Azure endpoint URL.
    ///
    /// Example: `https://my-resource.openai.azure.com`
    #[must_use]
    pub fn azure_endpoint(mut self, endpoint: impl Into<String>) -> Self {
        self.azure_endpoint = Some(endpoint.into());
        self
    }

    /// Set the Azure deployment name.
    ///
    /// When set, the base URL becomes `{endpoint}/openai/deployments/{deployment}`.
    /// When not set, the base URL is `{endpoint}/openai`.
    #[must_use]
    pub fn azure_deployment(mut self, deployment: impl Into<String>) -> Self {
        self.azure_deployment = Some(deployment.into());
        self
    }

    /// Set the Azure API version.
    ///
    /// Defaults to `2024-10-21` if not set and not in environment.
    #[must_use]
    pub fn api_version(mut self, version: impl Into<String>) -> Self {
        self.api_version = Some(version.into());
        self
    }

    /// Set the Azure API key.
    ///
    /// Mutually exclusive with `azure_ad_token`.
    #[must_use]
    pub fn api_key(mut self, key: impl Into<String>) -> Self {
        self.api_key = Some(key.into());
        self
    }

    /// Set the Azure AD token for authentication.
    ///
    /// Mutually exclusive with `api_key`. When using AD token auth,
    /// requests use `Authorization: Bearer {token}` instead of `api-key` header.
    #[must_use]
    pub fn azure_ad_token(mut self, token: impl Into<String>) -> Self {
        self.azure_ad_token = Some(token.into());
        self
    }

    /// Build an `OpenAI` client from this Azure configuration.
    ///
    /// # Errors
    ///
    /// Returns `OpenAIError::InvalidArgument` if:
    /// - No endpoint is provided (and `AZURE_OPENAI_ENDPOINT` is not set)
    /// - No credentials are provided (neither API key nor AD token)
    /// - Both `api_key` and `azure_ad_token` are set (mutually exclusive)
    pub fn build(self) -> Result<OpenAI, OpenAIError> {
        let endpoint = self.azure_endpoint.ok_or_else(|| {
            OpenAIError::InvalidArgument(
                "Azure endpoint is required. Set azure_endpoint() or AZURE_OPENAI_ENDPOINT env var"
                    .to_string(),
            )
        })?;

        let api_version = self
            .api_version
            .unwrap_or_else(|| DEFAULT_API_VERSION.to_string());

        // Validate mutual exclusivity
        if self.api_key.is_some() && self.azure_ad_token.is_some() {
            return Err(OpenAIError::InvalidArgument(
                "api_key and azure_ad_token are mutually exclusive; only one can be set"
                    .to_string(),
            ));
        }

        // Determine auth mode
        let (auth_key, use_azure_api_key_header) = match (&self.api_key, &self.azure_ad_token) {
            (Some(key), None) => (key.clone(), true),
            (None, Some(token)) => (token.clone(), false),
            (None, None) => {
                return Err(OpenAIError::InvalidArgument(
                    "Azure credentials required. Set api_key() or azure_ad_token()".to_string(),
                ));
            }
            _ => unreachable!(), // already checked above
        };

        // Build base URL
        let base_url = {
            let trimmed = endpoint.trim_end_matches('/');
            match &self.azure_deployment {
                Some(deployment) => format!("{trimmed}/openai/deployments/{deployment}"),
                None => format!("{trimmed}/openai"),
            }
        };

        // Build config with api-version as default query
        let config = ClientConfig::new(auth_key)
            .base_url(base_url)
            .default_query(vec![("api-version".to_string(), api_version)])
            .use_azure_api_key_header(use_azure_api_key_header);

        Ok(OpenAI::with_config(config))
    }

    /// Build an `OpenAI` client from environment variables.
    ///
    /// Reads:
    /// - `AZURE_OPENAI_API_KEY` — API key
    /// - `AZURE_OPENAI_ENDPOINT` — Azure endpoint URL
    /// - `OPENAI_API_VERSION` — API version (defaults to `2024-10-21`)
    /// - `AZURE_OPENAI_AD_TOKEN` — Azure AD token (alternative to API key)
    pub fn from_env() -> Result<OpenAI, OpenAIError> {
        let mut config = Self::new();

        if let Ok(endpoint) = env::var("AZURE_OPENAI_ENDPOINT") {
            config = config.azure_endpoint(endpoint);
        }

        if let Ok(key) = env::var("AZURE_OPENAI_API_KEY") {
            config = config.api_key(key);
        }

        if let Ok(token) = env::var("AZURE_OPENAI_AD_TOKEN") {
            config = config.azure_ad_token(token);
        }

        if let Ok(version) = env::var("OPENAI_API_VERSION") {
            config = config.api_version(version);
        }

        config.build()
    }
}
