# Phase 1 — Acceptance Criteria

- `OpenAI::new("sk-...")` creates client
- `OpenAI::from_env()` reads OPENAI_API_KEY
- Retry on 429 with exponential backoff
- `client.chat().completions().create(req).await` works
- Streaming returns `Stream<Item = Result<ChatCompletionChunk>>`
- Tool/function calling support in request/response types
- All types derive Serialize + Deserialize
- SSE parser handles `data:`, `[DONE]`, multiline
- mockito tests for all endpoints (no real API calls)
- `cargo test && cargo clippy -- -D warnings && cargo fmt -- --check`
