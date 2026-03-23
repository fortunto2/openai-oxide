# WebSocket Sessions

Persistent WebSocket connections eliminate per-request TLS handshakes and HTTP overhead, achieving 37% faster round-trip times for agent loops and multi-turn conversations.

See the official [Realtime API guide](https://platform.openai.com/docs/guides/realtime) for session configuration.

## Rust

```rust
use openai_oxide::OpenAI;

let client = OpenAI::from_env()?;

let mut session = client.ws_session().await?;

// Send multiple requests over the same connection
let response = session.create("gpt-4o-mini", "First message").await?;
let followup = session.create("gpt-4o-mini", "Follow-up question").await?;

session.close().await?;
```

## When to Use WebSockets

- Agent loops with 3+ sequential LLM calls
- Real-time conversational UIs
- High-throughput batch processing where latency matters
