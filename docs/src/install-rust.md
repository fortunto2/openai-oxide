# Rust Installation

## Add to Cargo.toml

```bash
cargo add openai-oxide tokio --features tokio/full
```

Or manually:

```toml
[dependencies]
openai-oxide = "0.9"
tokio = { version = "1", features = ["full"] }
```

## Feature Flags

Every API endpoint is behind a feature flag. All enabled by default.

```toml
# Minimal: only Responses API
openai-oxide = { version = "0.9", default-features = false, features = ["responses"] }
```

Available features: `chat`, `responses`, `embeddings`, `images`, `audio`, `files`, `fine-tuning`, `models`, `moderations`, `batches`, `uploads`, `beta`

Ecosystem features: `websocket`, `websocket-wasm`, `simd`, `macros`

## Configuration

```rust
use openai_oxide::OpenAI;

// From environment variable (recommended)
let client = OpenAI::from_env()?; // Uses OPENAI_API_KEY

// Explicit key
let client = OpenAI::new("sk-...");

// Custom config
use openai_oxide::config::ClientConfig;
let client = OpenAI::with_config(
    ClientConfig::new("sk-...").base_url("https://...").timeout_secs(30)
);

// Azure
use openai_oxide::azure::AzureConfig;
let client = OpenAI::azure(
    AzureConfig::new().azure_endpoint("https://my.openai.azure.com").api_key("...")
)?;
```

## API Reference

Full API docs: [docs.rs/openai-oxide](https://docs.rs/openai-oxide)
