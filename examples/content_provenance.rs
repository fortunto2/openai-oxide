//! Content provenance check: does a file carry OpenAI C2PA / SynthID signals?
//!
//! Run with: OPENAI_API_KEY=sk-... cargo run --example content_provenance -- photo.jpg

use openai_oxide::OpenAI;
use openai_oxide::types::content_provenance::ContentProvenanceResult;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let path = std::env::args()
        .nth(1)
        .ok_or("usage: content_provenance <image-or-audio-file>")?;

    let client = OpenAI::from_env()?;
    let check = client.content_provenance_checks().from_path(&path).await?;

    for result in &check.results {
        match result {
            ContentProvenanceResult::C2pa(c2pa) => {
                println!(
                    "C2PA: {:?} (validation: {:?}, issuer: {:?})",
                    c2pa.outcome, c2pa.validation_state, c2pa.issuer
                );
            }
            ContentProvenanceResult::SynthId(synthid) => {
                println!(
                    "SynthID: {:?} (model: {:?})",
                    synthid.outcome, synthid.model
                );
            }
            ContentProvenanceResult::Other(value) => {
                println!("Unknown result type: {value}");
            }
        }
    }
    Ok(())
}
