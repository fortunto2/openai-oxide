# openai-types

Standalone Rust types for the OpenAI API. Zero runtime dependencies beyond `serde`.

**1079 types** across 24 domains, auto-generated from the [OpenAI Python SDK](https://github.com/openai/openai-python) with manual overrides preserved on re-sync.

## Usage

```toml
[dependencies]
openai-types = "0.1"
```

```rust
use openai_types::chat::ChatCompletion;
use openai_types::responses::{Response, ResponseCreateRequest};
use openai_types::shared::ReasoningEffort;

let resp: ChatCompletion = serde_json::from_str(&json)?;
```

## Features

Each API domain is behind an optional feature flag. All enabled by default.

```toml
# Only what you need
openai-types = { version = "0.1", default-features = false, features = ["chat", "responses"] }
```

| Feature | Types | Description |
|---------|-------|-------------|
| `chat` | 50 | Chat completions |
| `responses` | 314 | Responses API |
| `realtime` | 188 | Realtime/WebSocket API |
| `beta` | 80 | Assistants, threads, runs |
| `audio` | 36 | Speech, transcription, translation |
| `evals` | 93 | Evaluations |
| `image` | 52 | Image generation/editing |
| `fine-tuning` | 25 | Fine-tuning jobs |
| `vector-stores` | 33 | Vector store search |
| `video` | 20 | Sora video generation |
| `webhooks` | 18 | Webhook events |
| `shared` | 26 | Common types (Role, ReasoningEffort) |
| `structured` | - | `schemars::JsonSchema` derive |
| + 12 more | | batch, completion, containers, conversations, embedding, file, graders, model, moderation, skills, uploads, websocket |

## Sync from Python SDK

```bash
make sync-types
# or manually:
python3 scripts/py2rust.py sync ~/openai-python/src/openai/types/ openai-types/src/
```

### Override mechanism

- `_gen.rs` files are machine-owned (overwritten on sync)
- Other `.rs` files are manual overrides (never touched)
- Types in manual files are automatically skipped during generation

To override a generated type: create it in a manual `.rs` file in the domain directory. Next sync skips it.

## Related

- [`openai-oxide`](https://crates.io/crates/openai-oxide) - Full async client using these types
- [OpenAI Python SDK](https://github.com/openai/openai-python) - Source of type definitions

## License

MIT
