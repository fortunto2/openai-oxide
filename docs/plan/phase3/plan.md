# openai-rust Phase 3 — Responses API, Fine-tuning, Assistants, README

**Status:** [ ] Not Started
**Track:** phase3

## Context Handoff

**Intent:** Add advanced endpoints (Responses API, fine-tuning, assistants/threads/runs) and polish for publishing.

**Reference:** Fetch Python sources from GitHub for each resource.

**Depends on:** Phase 2 complete

---

- [ ] Task 1.1: Study + implement Responses API (new endpoint). Fetch Python `types/responses/`. Create `src/types/responses.rs` and `src/resources/responses.rs` — `create()` with tools, instructions, previous_response_id. Support streaming. TDD.
- [ ] Task 1.2: Study + implement Fine-tuning Jobs. `fine_tuning.jobs.create()`, `list()`, `retrieve(id)`, `cancel(id)`, `list_events(id)`. TDD.
- [ ] Task 1.3: Study + implement Assistants API (beta). `beta.assistants.create()`, `list()`, `retrieve()`, `update()`, `delete()`. TDD.
- [ ] Task 1.4: Study + implement Threads + Messages. `beta.threads.create()`, `threads.messages.create()`, `list()`. TDD.
- [ ] Task 1.5: Study + implement Runs. `beta.threads.runs.create()`, `retrieve()`, `cancel()`, `submit_tool_outputs()`. Streaming runs. TDD.
- [ ] Task 1.6: Study + implement Vector Stores (for file search). `beta.vector_stores.create()`, `list()`, `file_batches.create()`. TDD.
- [ ] Task 1.7: Add comprehensive error handling — rate limit info parsing (RateLimitInfo from headers: x-ratelimit-remaining, x-ratelimit-reset). Typed error variants for each HTTP status. Test error deserialization.
- [ ] Task 1.8: Write README.md — installation, quickstart (chat, streaming, embeddings, images, audio), all endpoints table, error handling, configuration, examples. Add to Cargo.toml: description, repository, keywords, categories, documentation.
- [ ] Task 1.9: Run full verification. `cargo test`, clippy, fmt. `cargo doc --no-deps` for docs. Tag v0.1.0.
