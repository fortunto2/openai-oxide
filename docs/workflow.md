# Development Workflow — openai-rust

## TDD Cycle

1. **Red** — Write a failing test (mockito mock + expected types)
2. **Green** — Implement minimum code to pass
3. **Refactor** — Clean up, run clippy

## Per-Task Flow

```
1. Read Python SDK source for the resource (WebFetch from GitHub)
2. Write Rust types (serde structs) with test deserializing fixture JSON
3. Write resource impl with mockito test
4. cargo test && cargo clippy -- -D warnings && cargo fmt -- --check
5. Commit
```

## Commands

| Command | Purpose |
|---------|---------|
| `make test` | Run all tests |
| `make clippy` | Lint with clippy (warnings = errors) |
| `make fmt` | Check formatting |
| `make check` | All of the above |
| `make live` | Run tests with real API (needs OPENAI_API_KEY) |
| `make doc` | Generate docs |

## Commit Convention

```
feat: add embeddings resource
fix: handle empty stream chunks
test: add tool_calls deserialization test
refactor: extract SSE parser into streaming module
```

One commit = one task from the plan. Each commit must pass `make check`.

## Phase Gates

Before moving to the next phase:
- All tasks in current phase completed
- `make check` passes
- No `todo!()` or `unimplemented!()` in shipped code
- Update phase status in `docs/plan/phaseN/plan.md`

## Reference Workflow

For each new API resource:
1. Fetch Python types: `https://raw.githubusercontent.com/openai/openai-python/main/src/openai/types/{resource}.py`
2. Fetch Python resource: `https://raw.githubusercontent.com/openai/openai-python/main/src/openai/resources/{resource}.py`
3. Map Pydantic models → serde structs
4. Map resource methods → async Rust methods
5. Write mockito tests with fixture JSON from real API docs
