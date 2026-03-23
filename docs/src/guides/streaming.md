# Streaming

Stream tokens and events as they are generated, reducing time-to-first-token (TTFT) and enabling real-time UI updates.

See the official [Streaming documentation](https://platform.openai.com/docs/api-reference/streaming) for event types and behavior.

## Rust (Chat Completions)

```rust
{{#include ../../../examples/chat_stream.rs}}
```

Run: `OPENAI_API_KEY=sk-... cargo run --example chat_stream`

## Next Steps

- [Function Calling](./function-calling.md) — Stream with early tool-call parsing
- [WebSocket Sessions](./websockets.md) — Even lower latency with persistent connections
