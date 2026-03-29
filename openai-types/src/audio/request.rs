// Manual: audio request types with builder patterns.

use serde::Serialize;

use super::enums::{AudioResponseFormat, AudioVoice, SpeechResponseFormat};

/// Parameters for audio transcription (multipart upload).
#[derive(Debug)]
pub struct TranscriptionParams {
    pub file: Vec<u8>,
    pub filename: String,
    pub model: String,
    pub language: Option<String>,
    pub prompt: Option<String>,
    pub response_format: Option<AudioResponseFormat>,
    pub temperature: Option<f64>,
}

impl TranscriptionParams {
    pub fn new(file: Vec<u8>, filename: impl Into<String>, model: impl Into<String>) -> Self {
        Self {
            file,
            filename: filename.into(),
            model: model.into(),
            language: None,
            prompt: None,
            response_format: None,
            temperature: None,
        }
    }
}

/// Parameters for audio translation (multipart upload).
#[derive(Debug)]
pub struct TranslationParams {
    pub file: Vec<u8>,
    pub filename: String,
    pub model: String,
    pub prompt: Option<String>,
    pub response_format: Option<AudioResponseFormat>,
    pub temperature: Option<f64>,
}

impl TranslationParams {
    pub fn new(file: Vec<u8>, filename: impl Into<String>, model: impl Into<String>) -> Self {
        Self {
            file,
            filename: filename.into(),
            model: model.into(),
            prompt: None,
            response_format: None,
            temperature: None,
        }
    }
}

/// Request body for `POST /audio/speech`.
#[derive(Debug, Clone, Serialize)]
pub struct SpeechRequest {
    /// Text to convert to audio.
    pub input: String,
    /// TTS model (e.g. "tts-1", "tts-1-hd").
    pub model: String,
    /// Voice for audio output.
    pub voice: AudioVoice,
    /// Audio format.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub response_format: Option<SpeechResponseFormat>,
    /// Playback speed (0.25 to 4.0).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub speed: Option<f64>,
}

impl SpeechRequest {
    pub fn new(input: impl Into<String>, model: impl Into<String>, voice: AudioVoice) -> Self {
        Self {
            input: input.into(),
            model: model.into(),
            voice,
            response_format: None,
            speed: None,
        }
    }
}
