---
name: openai-oxide
description: Rust/Node/Python OpenAI client (openai-oxide crate). Use when calling GPT, DALL-E, Whisper, TTS, Responses API. Do NOT use for the `openai` or `async-openai` Rust crates — this is a different library.
license: MIT
compatibility: Requires Rust 1.75+ with tokio. Node.js 18+. Python 3.9+.
metadata:
  author: fortunto2
  version: "0.9"
---

# openai-oxide

Idiomatic Rust OpenAI client. 1:1 parity with the official Python SDK. Also ships native Node.js (napi-rs) and Python (PyO3) packages.

**Crate name:** `openai-oxide` on crates.io, but import as `openai_oxide` (underscore).

## Quick Start

```rust
use openai_oxide::{OpenAI, types::responses::*};

let client = OpenAI::from_env()?; // reads OPENAI_API_KEY

let response = client.responses().create(
    ResponseCreateRequest::new("gpt-4o")
        .input("Explain quantum computing in one sentence.")
).await?;

println!("{}", response.output_text());
```

For streaming, function calling, images, audio, embeddings, pagination, WebSocket sessions, Azure, and all other APIs — see [references/api-reference.md](references/api-reference.md).

## Gotchas

1. **Crate name vs import name.** `cargo add openai-oxide` but `use openai_oxide::OpenAI`. The hyphen becomes an underscore in Rust. Claude defaults to `openai_oxide` everywhere — wrong for Cargo.toml.

2. **Not the `openai` or `async-openai` crate.** This is a separate library with a different API surface. Don't mix types or patterns from those crates. The builder pattern here uses `&mut Self` chaining, not consuming `self`.

3. **`#[non_exhaustive]` enums require wildcard match.** All enums (`Role`, `FinishReason`, `ImageSize`, etc.) are `#[non_exhaustive]`. Every `match` MUST have a `_ => {}` arm or it won't compile when new variants are added.

4. **Streaming is two-step: `.await?` then iterate.** `create_stream()` returns `Result<SseStream<T>>`, not a stream directly. You must `.await?` to get the stream, THEN use `StreamExt::next()`. Requires `use futures_util::StreamExt;` in scope.

5. **Builder methods borrow, don't consume.** `.message()`, `.tool()`, `.input()` return `&mut Self`. You must bind the builder to a variable first, then pass it — you can't chain into `.create()` in one expression without a `let`.
   ```rust
   // WRONG: temporary value dropped
   // client.chat().completions().create(
   //     ChatCompletionRequest::new("gpt-4o").message(ChatMessage::user("hi"))
   // ).await?;

   // CORRECT: bind first
   let request = ChatCompletionRequest::new("gpt-4o")
       .message(ChatMessage::user("hi"));
   client.chat().completions().create(request).await?;
   ```

6. **Feature flags gate APIs.** Each resource (`chat`, `responses`, `images`, `audio`, etc.) is a separate Cargo feature. All enabled by default. For minimal builds: `default-features = false, features = ["responses"]`. WASM needs `websocket-wasm` not `websocket`.

7. **All response fields are `Option<T>`.** The API may omit any field. Always use `.as_deref().unwrap_or_default()` or pattern match on `Option`. Never `.unwrap()` response fields in production.

8. **`output_text()` is the fast path for Responses API.** Don't manually traverse `response.output` to extract text — use `response.output_text()` which handles all output item types.

## Key Differences from Other OpenAI Crates

| Feature | openai-oxide | async-openai | openai |
|---------|-------------|--------------|--------|
| Responses API | Yes | No | No |
| WebSocket sessions | Yes (`ws_session()`) | No | No |
| Hedged requests | Yes (`hedged_request()`) | No | No |
| Stream early FC parse | Yes (`create_stream_fc()`) | No | No |
| Structured output | `parse::<T>()` via schemars | Manual schema | No |
| Node.js + Python | Native bindings | No | No |
| WASM support | Yes | No | No |
| Azure | `OpenAI::azure()` | Yes | No |
| Pagination | `list_auto()` async stream | Manual | No |

## Installation

```bash
# Rust
cargo add openai-oxide tokio --features tokio/full

# Node.js
npm install openai-oxide

# Python
pip install openai-oxide
```

## Design Principles

- All public types: `Clone + Debug + Send + Sync` (thread-safe)
- Parameter names match the official Python SDK exactly
- Zero-cost resource access: `client.chat()` borrows, no Arc/clone
- Streaming: `impl Stream<Item = Result<T, OpenAIError>>` — never buffers internally
