# Streaming

Stream tokens and events as they are generated, reducing time-to-first-token (TTFT) and enabling real-time UI updates.

See the official [Streaming documentation](https://platform.openai.com/docs/api-reference/streaming) for event types and behavior.

## Rust

```rust
use openai_oxide::{OpenAI, types::responses::*};
use futures::StreamExt;

let client = OpenAI::from_env()?;

let mut stream = client.responses().create_stream(
    ResponseCreateRequest::new("gpt-4o-mini")
        .input("Write a haiku about Rust.")
).await?;

while let Some(event) = stream.next().await {
    let event = event?;
    // Handle each SSE event as it arrives
    println!("{:?}", event);
}
```

## Next Steps

- [Function Calling](./function-calling.md) — Stream with early tool-call parsing
- [WebSocket Sessions](./websockets.md) — Even lower latency with persistent connections
