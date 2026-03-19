# openai-rust

Idiomatic Rust client for the OpenAI API — 1:1 parity with the [official Python SDK](https://github.com/openai/openai-python).

## Features

- Async-first (tokio + reqwest)
- Strongly typed requests and responses (serde)
- SSE streaming support
- Automatic retries with exponential backoff
- Builder pattern for requests
- Same resource structure as Python SDK: `client.chat().completions().create()`

## Quick Start

Add to `Cargo.toml`:

```toml
[dependencies]
openai-rust = { git = "https://github.com/fortunto2/openai-rust" }
tokio = { version = "1", features = ["full"] }
```

```rust
use openai_rust::{OpenAI, types::chat::*};

#[tokio::main]
async fn main() -> Result<(), openai_rust::OpenAIError> {
    let client = OpenAI::from_env()?;

    let request = ChatCompletionRequest::new(
        "gpt-4o-mini",
        vec![
            ChatCompletionMessageParam::System {
                content: "You are a helpful assistant.".into(),
                name: None,
            },
            ChatCompletionMessageParam::User {
                content: UserContent::Text("Hello!".into()),
                name: None,
            },
        ],
    );

    let response = client.chat().completions().create(request).await?;
    println!("{}", response.choices[0].message.content.as_deref().unwrap_or(""));
    Ok(())
}
```

## Streaming

```rust
use futures_util::StreamExt;
use openai_rust::{OpenAI, types::chat::*};

#[tokio::main]
async fn main() -> Result<(), openai_rust::OpenAIError> {
    let client = OpenAI::from_env()?;

    let request = ChatCompletionRequest::new(
        "gpt-4o-mini",
        vec![ChatCompletionMessageParam::User {
            content: UserContent::Text("Tell me a joke".into()),
            name: None,
        }],
    );

    let mut stream = client.chat().completions().create_stream(request).await?;
    while let Some(chunk) = stream.next().await {
        let chunk = chunk?;
        if let Some(delta) = chunk.choices.first().and_then(|c| c.delta.content.as_deref()) {
            print!("{delta}");
        }
    }
    Ok(())
}
```

## Configuration

```rust
use openai_rust::{OpenAI, ClientConfig};

// From environment variable OPENAI_API_KEY
let client = OpenAI::from_env()?;

// Explicit API key
let client = OpenAI::new("sk-...");

// Full configuration
let config = ClientConfig::new()
    .base_url("https://api.openai.com/v1")
    .timeout(30)
    .max_retries(3);
let client = OpenAI::with_config("sk-...", config);
```

## Implemented APIs

| API | Method | Status |
|-----|--------|--------|
| Chat Completions | `client.chat().completions().create()` | Done |
| Chat Completions (streaming) | `client.chat().completions().create_stream()` | Done |

More endpoints coming soon: Embeddings, Images, Audio, Files, Models, Fine-tuning, Moderations, Responses.

## Development

```bash
cargo test                          # all tests
cargo test --features live-tests    # tests hitting real API (needs OPENAI_API_KEY)
cargo clippy -- -D warnings         # lint
cargo fmt -- --check                # format check
```

## License

MIT
