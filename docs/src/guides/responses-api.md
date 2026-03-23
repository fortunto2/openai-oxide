# Responses API

The Responses API is OpenAI's latest endpoint for generating text, replacing Chat Completions for new projects. It supports built-in tools, multi-turn conversations via `previous_response_id`, and structured output.

See the official [Responses API reference](https://platform.openai.com/docs/api-reference/responses).

## Rust

```rust
{{#include ../../../examples/responses_api.rs}}
```

Run: `OPENAI_API_KEY=sk-... cargo run --example responses_api`

## Next Steps

- [Streaming](./streaming.md) — Stream response events in real time
- [Function Calling](./function-calling.md) — Use tools with the Responses API
- [WebSocket Sessions](./websockets.md) — Persistent connections for agent loops
