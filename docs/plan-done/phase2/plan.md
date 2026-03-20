# openai-rust Phase 2 — Embeddings, Images, Audio, Models, Moderations

**Status:** [ ] Not Started
**Track:** phase2

## Context Handoff

**Intent:** Add remaining core endpoints. Each one follows the same pattern: study Python types → create Rust types → implement resource → mockito test.

**Reference:** For each resource, fetch the Python types from GitHub: `https://raw.githubusercontent.com/openai/openai-python/main/src/openai/types/{resource}.py` and `resources/{resource}.py`.

**Depends on:** Phase 1 complete (client, error, chat working)

---

- [ ] Task 1.1: Study + implement Embeddings. Fetch Python `types/embedding.py`. Create `src/types/embedding.rs` (EmbeddingRequest, EmbeddingResponse, EmbeddingObject, EmbeddingUsage). `src/resources/embeddings.rs` — `client.embeddings().create(req)`. TDD with mockito.
- [ ] Task 1.2: Study + implement Models. Fetch Python `types/model.py`. Create `src/types/model.rs` (Model, ModelList). `src/resources/models.rs` — `list()`, `retrieve(id)`, `delete(id)`. TDD.
- [ ] Task 1.3: Study + implement Moderations. Fetch Python `types/moderation.py`. Create `src/types/moderation.rs`. `src/resources/moderations.rs` — `create(input)`. TDD.
- [ ] Task 1.4: Study + implement Images. Fetch Python `types/image.py`. Create `src/types/image.rs` (ImageGenerateRequest, ImagesResponse, Image). `src/resources/images.rs` — `generate()`, `edit()`, `create_variation()`. TDD.
- [ ] Task 1.5: Study + implement Audio Transcription. Fetch Python `types/audio/transcription.py`. Create `src/types/audio.rs` (TranscriptionRequest, Transcription). `src/resources/audio.rs` — `transcriptions().create(file, model)`. Multipart upload. TDD.
- [ ] Task 1.6: Study + implement Audio Speech (TTS). Create types for `audio.speech.create(input, voice, model)`. Returns raw bytes (mp3/opus). TDD.
- [ ] Task 1.7: Study + implement Audio Translation. `audio.translations.create(file, model)`. Multipart. TDD.
- [ ] Task 1.8: Study + implement Files API. `files.create(file, purpose)`, `list()`, `retrieve(id)`, `delete(id)`, `content(id)`. Multipart upload. TDD.
- [ ] Task 1.9: Run full verification. Ensure all resources are wired in lib.rs re-exports. `cargo test`, clippy, fmt.
