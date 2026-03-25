// Shared types (Usage, Role, etc.)

use serde::{Deserialize, Serialize};

/// Macro to create an OpenAI API enum with forward-compatible `Other(String)` variant.
///
/// Syntax: `VariantName = "json_value"`
///
/// Example:
/// ```ignore
/// openai_enum! {
///     /// Message role
///     pub enum Role {
///         System = "system",
///         Developer = "developer",
///         InProgress = "in_progress",  // auto-handles snake_case
///         FineTune = "fine-tune",      // handles hyphens
///     }
/// }
/// ```
#[macro_export]
macro_rules! openai_enum {
    (
        $(#[$meta:meta])*
        $vis:vis enum $name:ident {
            $(
                $(#[$var_meta:meta])*
                $variant:ident = $json:literal
            ),*$(,)?
        }
    ) => {
        $(#[$meta])*
        #[derive(Debug, Clone, PartialEq, Eq)]
        #[non_exhaustive]
        $vis enum $name {
            $(
                $(#[$var_meta])*
                $variant,
            )*
            /// Catch-all for unknown variants (forward compatibility).
            Other(String),
        }

        impl serde::Serialize for $name {
            fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
            where
                S: serde::Serializer,
            {
                match self {
                    $(Self::$variant => serializer.serialize_str($json),)*
                    Self::Other(s) => serializer.serialize_str(s),
                }
            }
        }

        impl<'de> serde::Deserialize<'de> for $name {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                let s = String::deserialize(deserializer)?;
                match s.as_str() {
                    $($json => Ok(Self::$variant),)*
                    _ => Ok(Self::Other(s)),
                }
            }
        }
    };
}

/// Helper function to serialize the `Other(String)` variant.
/// Used by enums that don't use the openai_enum! macro (e.g., FilePurpose with custom serde renames).
pub fn serialize_other<S>(value: &str, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    serializer.serialize_str(value)
}

openai_enum! {
    /// Message role in chat/thread conversations.
    pub enum Role {
        System = "system",
        Developer = "developer",
        User = "user",
        Assistant = "assistant",
        Tool = "tool",
        Function = "function",
    }
}

/// Token usage information returned by the API.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Usage {
    pub prompt_tokens: Option<i64>,
    pub completion_tokens: Option<i64>,
    pub total_tokens: Option<i64>,
    /// Detailed breakdown of prompt tokens.
    #[serde(default)]
    pub prompt_tokens_details: Option<PromptTokensDetails>,
    /// Detailed breakdown of completion tokens.
    #[serde(default)]
    pub completion_tokens_details: Option<CompletionTokensDetails>,
}

/// Detailed breakdown of prompt token usage.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromptTokensDetails {
    #[serde(default)]
    pub cached_tokens: Option<i64>,
    #[serde(default)]
    pub audio_tokens: Option<i64>,
}

openai_enum! {
    /// Reason the model stopped generating tokens.
    pub enum FinishReason {
        Stop = "stop",
        Length = "length",
        ToolCalls = "tool_calls",
        ContentFilter = "content_filter",
        FunctionCall = "function_call",
    }
}

impl std::fmt::Display for FinishReason {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Stop => write!(f, "stop"),
            Self::Length => write!(f, "length"),
            Self::ToolCalls => write!(f, "tool_calls"),
            Self::ContentFilter => write!(f, "content_filter"),
            Self::FunctionCall => write!(f, "function_call"),
            Self::Other(s) => write!(f, "{}", s),
        }
    }
}

openai_enum! {
    /// Service tier used for the request.
    pub enum ServiceTier {
        Auto = "auto",
        Default = "default",
        Flex = "flex",
        Scale = "scale",
        Priority = "priority",
    }
}

openai_enum! {
    /// Reasoning effort level for o-series models.
    pub enum ReasoningEffort {
        Low = "low",
        Medium = "medium",
        High = "high",
    }
}

openai_enum! {
    /// Search context size for web search.
    pub enum SearchContextSize {
        Low = "low",
        Medium = "medium",
        High = "high",
    }
}

/// A value that is either "auto" or a fixed number.
///
/// Used for hyperparameters like `n_epochs`, `batch_size`, `learning_rate_multiplier`.
/// Serializes as the string `"auto"` or a bare number.
#[derive(Debug, Clone, PartialEq)]
pub enum AutoOrFixed<T> {
    Auto,
    Fixed(T),
}

impl<T: Serialize> Serialize for AutoOrFixed<T> {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            Self::Auto => serializer.serialize_str("auto"),
            Self::Fixed(v) => v.serialize(serializer),
        }
    }
}

impl<'de, T: Deserialize<'de>> Deserialize<'de> for AutoOrFixed<T> {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let value = serde_json::Value::deserialize(deserializer)?;
        match &value {
            serde_json::Value::String(s) if s == "auto" => Ok(Self::Auto),
            _ => T::deserialize(value)
                .map(Self::Fixed)
                .map_err(serde::de::Error::custom),
        }
    }
}

/// Token limit that is either "inf" (unlimited) or a fixed integer.
///
/// Used for `max_response_output_tokens` in the Realtime API.
#[derive(Debug, Clone, PartialEq)]
pub enum MaxResponseTokens {
    Inf,
    Fixed(i64),
}

impl Serialize for MaxResponseTokens {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            Self::Inf => serializer.serialize_str("inf"),
            Self::Fixed(v) => serializer.serialize_i64(*v),
        }
    }
}

impl<'de> Deserialize<'de> for MaxResponseTokens {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let value = serde_json::Value::deserialize(deserializer)?;
        match &value {
            serde_json::Value::String(s) if s == "inf" => Ok(Self::Inf),
            serde_json::Value::Number(n) => n
                .as_i64()
                .map(Self::Fixed)
                .ok_or_else(|| serde::de::Error::custom("expected integer")),
            _ => Err(serde::de::Error::custom("expected \"inf\" or integer")),
        }
    }
}

openai_enum! {
    /// Sort order for paginated list endpoints.
    pub enum SortOrder {
        Asc = "asc",
        Desc = "desc",
    }
}

/// Detailed breakdown of completion token usage.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompletionTokensDetails {
    #[serde(default)]
    pub reasoning_tokens: Option<i64>,
    #[serde(default)]
    pub audio_tokens: Option<i64>,
    #[serde(default)]
    pub accepted_prediction_tokens: Option<i64>,
    #[serde(default)]
    pub rejected_prediction_tokens: Option<i64>,
}
