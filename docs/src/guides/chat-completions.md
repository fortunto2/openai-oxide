# Chat Completions

Send messages to GPT models and receive completions. This is the most common API for conversational AI.

See the official [Chat Completions guide](https://platform.openai.com/docs/guides/chat-completions) and [API reference](https://platform.openai.com/docs/api-reference/chat).

## Rust

```rust
{{#include ../../../examples/chat.rs}}
```

Run: `OPENAI_API_KEY=sk-... cargo run --example chat`

## Next Steps

- [Streaming](./streaming.md) — Stream chat completion tokens as they arrive
- [Function Calling](./function-calling.md) — Let the model call your functions
- [Structured Output](./structured-output.md) — Get JSON responses matching a schema
