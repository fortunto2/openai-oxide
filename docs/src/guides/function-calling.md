# Function Calling

Let the model invoke your functions by defining tools. openai-oxide supports early-parsing of function call arguments during streaming, allowing you to execute tools ~400ms before the response finishes.

See the official [Function Calling guide](https://platform.openai.com/docs/guides/function-calling) for tool schema definitions.

## Rust

```rust
use openai_oxide::{OpenAI, types::responses::*};

let client = OpenAI::from_env()?;

let response = client.responses().create(
    ResponseCreateRequest::new("gpt-4o")
        .input("What is the weather in Tokyo?")
        .tools(vec![Tool::function(
            "get_weather",
            "Get current weather for a location",
            serde_json::json!({
                "type": "object",
                "properties": {
                    "location": { "type": "string" }
                },
                "required": ["location"]
            }),
        )])
).await?;
```

## Next Steps

- [Streaming](./streaming.md) — Stream function call arguments as they arrive
- [Structured Output](./structured-output.md) — Combine tools with structured responses
