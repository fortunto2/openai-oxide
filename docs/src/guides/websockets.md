# WebSocket Sessions

Persistent WebSocket connections eliminate per-request TLS handshakes and HTTP overhead, achieving 37% faster round-trip times for agent loops and multi-turn conversations.

See the official [Realtime API guide](https://platform.openai.com/docs/guides/realtime) for session configuration.

## Rust

```rust
{{#include ../../../examples/websocket.rs}}
```

Run: `OPENAI_API_KEY=sk-... cargo run --example websocket --features websocket`

## When to Use WebSockets

- Agent loops with 3+ sequential LLM calls
- Real-time conversational UIs
- High-throughput batch processing where latency matters
