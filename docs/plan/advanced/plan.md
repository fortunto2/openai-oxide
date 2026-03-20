# openai-oxide — Advanced Features (GPT-5.4 era)

**Status:** [ ] Not Started
**Track:** advanced

## Context Handoff

**Intent:** Add advanced GPT-5.4 features and ensure 95%+ field coverage vs Python SDK.

**What's DONE:** 18 resources, 93 tests, tool calling, streaming, responses API (basic). v0.2.0 on crates.io.

**CRITICAL WORKFLOW — Read Python SDK LOCALLY:**
For EVERY task:
1. Read `~/startups/shared/openai-python/src/openai/types/{file}.py` — the Python Pydantic model
2. Read `~/startups/shared/openai-python/src/openai/resources/{file}.py` — the Python resource methods
3. Compare EVERY field with our Rust struct
4. Add ALL missing fields. Same names via `#[serde(rename = "...")]` if needed
5. Do NOT invent fields — if Python doesn't have it, we don't add it

**OpenAPI spec:** `tests/openapi.yaml` in repo — use for fixture generation and validation tests.

---

## Phase 0: OpenAPI Validation Tests

- [ ] Task 0.1: Parse `tests/openapi.yaml` `components/schemas` section. Create `tests/openapi_coverage.rs` — for each major schema (ChatCompletion, Embedding, etc.) extract all field names, compare with our Rust struct fields. Report coverage %.
- [ ] Task 0.2: Create `tests/fixtures/` with JSON response fixtures from OpenAPI examples. One per endpoint. Test each fixture deserializes into our Rust types without error.

## Phase 1: Chat Completions — Full Field Parity

- [ ] Task 1.1: Read `~/startups/shared/openai-python/src/openai/types/chat/chat_completion_create_params.py`. Compare ALL fields with our `ChatCompletionRequest`. Add missing: `prediction`, `reasoning_effort`, `audio`, `modalities`, `metadata`, `service_tier`, `store`, `user`, `seed`, `logit_bias`, `logprobs`, `top_logprobs`, `n`, `presence_penalty`, `frequency_penalty`, `stop`. TDD.
- [ ] Task 1.2: Read `~/startups/shared/openai-python/src/openai/types/chat/chat_completion.py`. Compare ALL fields with our `ChatCompletionResponse`. Add missing: `service_tier`, `system_fingerprint`, `usage.prompt_tokens_details` (cached_tokens, audio_tokens), `usage.completion_tokens_details` (reasoning_tokens, audio_tokens, accepted/rejected_prediction_tokens). TDD.
- [ ] Task 1.3: Read `~/startups/shared/openai-python/src/openai/types/chat/chat_completion_chunk.py`. Add missing streaming fields: `usage` (stream_options), `service_tier`, `system_fingerprint`. TDD.

## Phase 2: Responses API — Full Power

- [ ] Task 2.1: Read `~/startups/shared/openai-python/src/openai/types/responses/response_create_params.py`. Update `ResponseCreateRequest`: add `instructions`, `tools` array, `tool_choice`, `truncation`, `max_output_tokens`, `metadata`, `reasoning` (effort+summary), `include`, `parallel_tool_calls`, `temperature`, `top_p`. TDD.
- [ ] Task 2.2: Read `~/startups/shared/openai-python/src/openai/types/responses/response.py`. Update `Response` struct: `output` array variants (message, function_call, file_search), `status`, `usage` with cache fields, `metadata`. TDD.
- [ ] Task 2.3: Read `~/startups/shared/openai-python/src/openai/resources/responses/responses.py`. Implement `create_stream()` → `Stream<Item = Result<ResponseStreamEvent>>`. Read stream event types from `types/responses/`. TDD with mock SSE.
- [ ] Task 2.4: Read Python tool type definitions. Create `ResponseTool` enum: `WebSearch`, `FileSearch`, `CodeInterpreter`, `ComputerUse`, `Mcp`, `Function`. Each with config fields from Python source. TDD.

## Phase 3: Structured Outputs + Builders

- [ ] Task 3.1: Read `~/startups/shared/openai-python/src/openai/types/chat/chat_completion_create_params.py` — find `response_format`. Add `strict: bool` to `JsonSchema`. Add `FunctionDef.strict: Option<bool>`. TDD.
- [ ] Task 3.2: Builder pattern for `ChatCompletionRequest` — `.model()`, `.messages()`, `.tools()`, `.temperature()`, `.max_tokens()`, `.response_format()`, `.reasoning_effort()`, `.prediction()`. Chainable. TDD.
- [ ] Task 3.3: Builder pattern for `ResponseCreateRequest` — `.model()`, `.input()`, `.instructions()`, `.tools()`, `.previous_response_id()`, `.reasoning()`. Chainable. TDD.

## Phase 4: Realtime API + Examples

- [ ] Task 4.1: Read `~/startups/shared/openai-python/src/openai/resources/beta/realtime/sessions.py` and `types/beta/realtime/`. Create `src/resources/realtime.rs` + `src/types/realtime.rs`. Session creation + ephemeral token. TDD.
- [ ] Task 4.2: Examples: `tool_calling.rs`, `structured_output.rs`, `responses_api.rs`. Working flows. Behind `live-tests`.
- [ ] Task 4.3: Bump to 0.3.0. README update. `make check`. Final commit.

## Review Criteria

1. Read `~/startups/shared/openai-python/src/openai/types/chat/chat_completion_create_params.py` — list ALL fields, compare with our struct. Report coverage %.
2. Read `~/startups/shared/openai-python/src/openai/types/responses/response_create_params.py` — same.
3. `cargo test` must pass including OpenAPI fixture tests.
4. Coverage < 95% → `<solo:redo/>`. Coverage ≥ 95% → `<solo:done/>`.
