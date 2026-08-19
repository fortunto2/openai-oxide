// Manual: typed provenance results + multipart upload params.
// The generated union alias (`Result = serde_json::Value`) is replaced by a
// typed enum, and the response struct references it.

use serde::{Deserialize, Serialize};

use super::_gen::{ResultC2PA, ResultSynthID};

/// A single provenance result: a C2PA content-credentials manifest or a
/// SynthID watermark.
///
/// Untagged: a C2PA payload carries `validation_state`, which SynthID lacks,
/// so the variants never overlap. Unknown future result types land in
/// `Other`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "structured", derive(schemars::JsonSchema))]
#[serde(untagged)]
pub enum ContentProvenanceResult {
    /// C2PA content credentials (`"type": "c2pa"`).
    C2pa(ResultC2PA),
    /// SynthID watermark (`"type": "synthid"`).
    SynthId(ResultSynthID),
    /// Forward-compat: a result type this version does not know yet.
    Other(serde_json::Value),
}

/// Union alias matching the Python SDK's `Result`; prefer the enum name.
/// Shadows `std::result::Result` if you glob-import this module — import
/// types individually there.
pub type Result = ContentProvenanceResult;

/// Response of `POST /content_provenance_checks`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "structured", derive(schemars::JsonSchema))]
pub struct ContentProvenanceCheck {
    /// The Unix timestamp, in seconds, when the provenance check was created.
    pub created_at: i64,
    /// The object type. Always `content_provenance_check` for this endpoint.
    pub object: String,
    /// The provenance results that apply to the uploaded file.
    pub results: Vec<ContentProvenanceResult>,
}

/// Multipart params for `POST /content_provenance_checks`.
#[derive(Debug, Clone)]
#[must_use]
pub struct ContentProvenanceCheckParams {
    /// The image or audio file to check for supported OpenAI provenance signals.
    pub file: Vec<u8>,
    /// Filename sent in the multipart form (its extension hints the media type).
    pub filename: String,
}

impl ContentProvenanceCheckParams {
    /// Check the given file bytes for OpenAI provenance signals.
    pub fn new(file: Vec<u8>, filename: impl Into<String>) -> Self {
        Self {
            file,
            filename: filename.into(),
        }
    }
}
