# CLAUDE.md — openai-rust

Idiomatic Rust client for the OpenAI API. 1:1 parity with [openai-python](https://github.com/openai/openai-python).

## Goal

Replicate the official Python SDK in Rust:
- Same resource structure: `client.chat.completions.create()`
- Same parameter names and types
- Streaming support (SSE)
- All endpoints: Chat, Completions, Embeddings, Images, Audio, Files, Fine-tuning, Moderations, Models, Assistants, Threads, Messages, Runs, Vector Stores, Responses
- Async-first (tokio + reqwest)
- Builder pattern for requests
- Strongly typed responses (serde)

## Reference

Study the Python SDK structure at https://github.com/openai/openai-python:
- `src/openai/resources/` — one module per API resource
- `src/openai/types/` — Pydantic models for requests/responses
- `src/openai/_client.py` — base client with auth, retries, base URL
- `src/openai/_streaming.py` — SSE stream handling

## Tech Stack

| Component | Technology |
|-----------|-----------|
| HTTP | reqwest (rustls-tls) |
| Async | tokio |
| Serialization | serde + serde_json |
| Streaming | reqwest streaming + SSE parsing |
| Errors | thiserror |
| Builder | typed builder pattern (no derive macro) |
| Testing | cargo test + mockito (HTTP mocking) |

## Architecture

```
openai-rust/
  src/
    lib.rs              — pub mod, re-exports
    client.rs           — OpenAI client (api_key, base_url, org, retries)
    error.rs            — OpenAIError enum
    config.rs           — ClientConfig (timeouts, retries, base_url)
    streaming.rs        — SSE stream parser
    resources/
      chat/             — chat.completions.create()
      embeddings.rs     — embeddings.create()
      images.rs         — images.generate()
      audio/            — audio.transcriptions, audio.speech
      files.rs          — files.create(), list(), retrieve(), delete()
      models.rs         — models.list(), retrieve(), delete()
      moderations.rs    — moderations.create()
      fine_tuning/      — fine_tuning.jobs.*
      responses.rs      — responses.create() (new Responses API)
    types/
      chat.rs           — ChatCompletionRequest, ChatCompletionResponse, ...
      embedding.rs      — EmbeddingRequest, EmbeddingResponse
      image.rs          — ImageGenerateRequest, ...
      audio.rs          — TranscriptionRequest, SpeechRequest, ...
      common.rs         — Usage, Model, shared types
```

## Naming Convention

Follow Python SDK names exactly:
- `client.chat.completions.create()` → `client.chat().completions().create(params).await`
- `ChatCompletionMessage` → `ChatCompletionMessage`
- `stream=True` → `.stream(true)` or `create_stream()`

## Essential Commands

```bash
cargo test                          # all tests
cargo test --features "live-tests"  # tests hitting real API (needs OPENAI_API_KEY)
cargo clippy -- -D warnings
cargo fmt -- --check
```

## Don't

- Invent new API names — match Python SDK exactly
- Use derive macros for builders — manual impl for flexibility
- Skip error handling — every API error type must be covered
- Add async-std support — tokio only

## Do

- TDD: write test with expected request/response JSON before implementing
- Use mockito for HTTP mocking (no real API calls in default tests)
- Use serde(rename) to match OpenAI's snake_case JSON
- Support both `OPENAI_API_KEY` env var and explicit key
- Make all response fields `Option` where the API may omit them
