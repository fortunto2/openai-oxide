# Structured Output

Force the model to return JSON matching a specific schema. Guarantees valid, parseable output without prompt engineering tricks.

See the official [Structured Outputs guide](https://platform.openai.com/docs/guides/structured-outputs) for schema format and limitations.

## Rust

```rust
{{#include ../../../examples/structured_output.rs}}
```

Run: `OPENAI_API_KEY=sk-... cargo run --example structured_output`

## Next Steps

- [Function Calling](./function-calling.md) — Combine structured output with tool use
- [Responses API](./responses-api.md) — Full parameter reference
