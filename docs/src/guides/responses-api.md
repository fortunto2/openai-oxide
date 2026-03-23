# Responses API

The Responses API is OpenAI's latest endpoint for generating text, replacing Chat Completions for new projects. It supports built-in tools, multi-turn conversations via `previous_response_id`, and structured output.

See the official [Responses API reference](https://platform.openai.com/docs/api-reference/responses) for full parameter documentation.

## Rust

```rust
use openai_oxide::{OpenAI, types::responses::*};

let client = OpenAI::from_env()?;

let response = client.responses().create(
    ResponseCreateRequest::new("gpt-4o")
        .input("Summarize the benefits of Rust.")
        .max_output_tokens(200)
        .store(true)
).await?;

println!("{}", response.output_text());
```

## Next Steps

- [Streaming](./streaming.md) — Stream response events in real time
- [Function Calling](./function-calling.md) — Use tools with the Responses API
- [WebSocket Sessions](./websockets.md) — Persistent connections for agent loops
