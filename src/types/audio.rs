// Audio types — mirrors openai-python types/audio/

use crate::openai_enum;
use serde::{Deserialize, Serialize};

openai_enum! {
    /// Response format for audio transcription/translation.
    pub enum AudioResponseFormat {
        Json = "json",
        Text = "text",
        Srt = "srt",
        VerboseJson = "verbose_json",
        Vtt = "vtt",
        DiarizedJson = "diarized_json",
    }
}

openai_enum! {
    /// Audio format for speech output.
    pub enum SpeechResponseFormat {
        Mp3 = "mp3",
        Opus = "opus",
        Aac = "aac",
        Flac = "flac",
        Wav = "wav",
        Pcm = "pcm",
    }
}

openai_enum! {
    /// Voice for text-to-speech.
    pub enum AudioVoice {
        Alloy = "alloy",
        Ash = "ash",
        Ballad = "ballad",
        Coral = "coral",
        Echo = "echo",
        Sage = "sage",
        Shimmer = "shimmer",
        Verse = "verse",
        Marin = "marin",
        Cedar = "cedar",
    }
}

openai_enum! {
    /// Input audio format in content parts.
    pub enum InputAudioFormat {
        Wav = "wav",
        Mp3 = "mp3",
    }
}

// ── Transcription request (multipart) ──

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

// ── Transcription response ──

/// Response from `POST /audio/transcriptions` (json format).
#[derive(Debug, Clone, Deserialize)]
pub struct Transcription {
    pub text: String,
}

// ── Translation request (multipart) ──

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

/// Response from `POST /audio/translations` (json format).
#[derive(Debug, Clone, Deserialize)]
pub struct Translation {
    pub text: String,
}

// ── Speech request (JSON body, binary response) ──

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_serialize_speech_request() {
        let req = SpeechRequest::new("Hello world", "tts-1", AudioVoice::Alloy);
        let json = serde_json::to_value(&req).unwrap();
        assert_eq!(json["input"], "Hello world");
        assert_eq!(json["model"], "tts-1");
        assert_eq!(json["voice"], "alloy");
    }

    #[test]
    fn test_deserialize_transcription() {
        let json = r#"{"text": "Hello world"}"#;
        let t: Transcription = serde_json::from_str(json).unwrap();
        assert_eq!(t.text, "Hello world");
    }

    #[test]
    fn test_deserialize_translation() {
        let json = r#"{"text": "Hello world in English"}"#;
        let t: Translation = serde_json::from_str(json).unwrap();
        assert_eq!(t.text, "Hello world in English");
    }
}
