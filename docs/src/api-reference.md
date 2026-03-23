# API Reference

Full API documentation for each platform:

| Platform | Documentation |
|----------|---------------|
| Rust | [docs.rs/openai-oxide](https://docs.rs/openai-oxide) |
| Node.js | [npmjs.com/package/openai-oxide](https://www.npmjs.com/package/openai-oxide) |
| Python | [pypi.org/project/openai-oxide](https://pypi.org/project/openai-oxide/) |

## Rust API

The Rust crate provides the most complete API surface. All endpoints are accessed through the `OpenAI` client via resource methods:

| Resource | Access | Docs |
|----------|--------|------|
| Chat Completions | `client.chat().completions()` | [docs.rs](https://docs.rs/openai-oxide/latest/openai_oxide/resources/chat/) |
| Responses | `client.responses()` | [docs.rs](https://docs.rs/openai-oxide/latest/openai_oxide/resources/responses/) |
| Embeddings | `client.embeddings()` | [docs.rs](https://docs.rs/openai-oxide/latest/openai_oxide/resources/embeddings/) |
| Images | `client.images()` | [docs.rs](https://docs.rs/openai-oxide/latest/openai_oxide/resources/images/) |
| Audio | `client.audio()` | [docs.rs](https://docs.rs/openai-oxide/latest/openai_oxide/resources/audio/) |
| Files | `client.files()` | [docs.rs](https://docs.rs/openai-oxide/latest/openai_oxide/resources/files/) |
| Fine-tuning | `client.fine_tuning()` | [docs.rs](https://docs.rs/openai-oxide/latest/openai_oxide/resources/fine_tuning/) |
| Models | `client.models()` | [docs.rs](https://docs.rs/openai-oxide/latest/openai_oxide/resources/models/) |
| Moderations | `client.moderations()` | [docs.rs](https://docs.rs/openai-oxide/latest/openai_oxide/resources/moderations/) |
| Batches | `client.batches()` | [docs.rs](https://docs.rs/openai-oxide/latest/openai_oxide/resources/batches/) |
| Uploads | `client.uploads()` | [docs.rs](https://docs.rs/openai-oxide/latest/openai_oxide/resources/uploads/) |
| Assistants (beta) | `client.beta().assistants()` | [docs.rs](https://docs.rs/openai-oxide/latest/openai_oxide/resources/beta/) |
| Threads (beta) | `client.beta().threads()` | [docs.rs](https://docs.rs/openai-oxide/latest/openai_oxide/resources/beta/) |
| Runs (beta) | `client.beta().runs()` | [docs.rs](https://docs.rs/openai-oxide/latest/openai_oxide/resources/beta/) |
| Vector Stores (beta) | `client.beta().vector_stores()` | [docs.rs](https://docs.rs/openai-oxide/latest/openai_oxide/resources/beta/) |
