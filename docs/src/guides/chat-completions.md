# Chat Completions

Send messages to GPT models and receive completions. This is the most common API for conversational AI.

See the official [Chat Completions guide](https://platform.openai.com/docs/guides/chat-completions) for full parameter documentation.

## Rust

```rust
use openai_oxide::{OpenAI, types::chat::*};

let client = OpenAI::from_env()?;

let response = client.chat().completions().create(
    ChatCompletionRequest::new("gpt-4o")
        .messages(vec![
            ChatMessage::user("What is the capital of France?"),
        ])
        .temperature(0.7)
).await?;

println!("{}", response.choices[0].message.content.as_deref().unwrap_or(""));
```

## Next Steps

- [Streaming](./streaming.md) — Stream chat completion tokens as they arrive
- [Function Calling](./function-calling.md) — Let the model call your functions
- [Structured Output](./structured-output.md) — Get JSON responses matching a schema
