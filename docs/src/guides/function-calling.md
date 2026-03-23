# Function Calling

Let the model invoke your functions by defining tools. openai-oxide supports early-parsing of function call arguments during streaming, allowing you to execute tools ~400ms before the response finishes.

See the official [Function Calling guide](https://platform.openai.com/docs/guides/function-calling) for tool schema definitions.

## Rust

```rust
{{#include ../../../examples/tool_calling.rs}}
```

Run: `OPENAI_API_KEY=sk-... cargo run --example tool_calling`

## Next Steps

- [Streaming](./streaming.md) — Stream function call arguments as they arrive
- [Structured Output](./structured-output.md) — Combine tools with structured responses
