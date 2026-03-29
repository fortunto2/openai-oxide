// Manual: audio response types (richer than generated — includes Optional fields).

use serde::Deserialize;

use super::Logprob;

/// Response from `POST /audio/transcriptions` (json format).
#[derive(Debug, Clone, Deserialize)]
pub struct Transcription {
    /// The transcribed text.
    pub text: String,
    /// The log probabilities of the tokens in the transcription.
    #[serde(default)]
    pub logprobs: Option<Vec<Logprob>>,
    /// Token usage statistics for the request.
    #[serde(default)]
    pub usage: Option<serde_json::Value>,
}

/// Response from `POST /audio/translations` (json format).
#[derive(Debug, Clone, Deserialize)]
pub struct Translation {
    pub text: String,
}
