// Audio types — re-exported from openai-types.
// Manual overrides (enums, builders) live in openai-types/src/audio/*.rs
// Generated types (_gen.rs) supplement with verbose transcriptions, streaming events, etc.

pub use openai_types::audio::*;

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
