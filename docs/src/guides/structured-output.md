# Structured Output

Force the model to return JSON matching a specific schema. Guarantees valid, parseable output without prompt engineering tricks.

See the official [Structured Outputs guide](https://platform.openai.com/docs/guides/structured-outputs) for schema format and limitations.

## Rust

```rust
use openai_oxide::{OpenAI, types::responses::*};

let client = OpenAI::from_env()?;

let response = client.responses().create(
    ResponseCreateRequest::new("gpt-4o")
        .input("Extract: John is 30 years old and lives in NYC")
        .text_format(JsonSchema::new("person", serde_json::json!({
            "type": "object",
            "properties": {
                "name": { "type": "string" },
                "age": { "type": "integer" },
                "city": { "type": "string" }
            },
            "required": ["name", "age", "city"]
        })))
).await?;

// response.output_text() contains valid JSON matching the schema
```

## Next Steps

- [Function Calling](./function-calling.md) — Combine structured output with tool use
- [Responses API](./responses-api.md) — Full parameter reference
