---
title: "Show HN: openai-oxide – Rust OpenAI client I use across terminal, iOS, browser, Node, and Python"
url: https://github.com/fortunto2/openai-oxide
---

## HN first comment (post this after submitting the link)

Hey HN, I built openai-oxide because I needed the same OpenAI client on too many platforms.

I'm a solo dev building three things: a TUI coding agent (Rust, terminal), an iOS video montage app (Swift + Rust via UniFFI), and a realtime voice assistant (WebSocket, latency-sensitive). All three need structured outputs, streaming, and tool calling. I didn't want three separate OpenAI integrations.

So I wrote one Rust crate and generated bindings for everything else:

```
openai-oxide (Rust core)
├── sgr-agent (my agent framework on top)
│   ├── rust-code    — coding agent (terminal)
│   └── va-agent     — video montage (iOS, UniFFI → Swift)
├── Node.js bindings  (napi-rs)
├── Python bindings   (PyO3)
└── WASM             (browsers, edge runtimes)
```

The iOS app is the interesting case. The Swift side handles Apple Vision (face detection, scene classification) and AVFoundation (video decode/render). The Rust side runs an autonomous agent loop: pick scenes, plan voiceover timing, assemble timeline. It talks to OpenAI/OpenRouter/Gemini/Ollama through openai-oxide and crosses into Swift via UniFFI. Types defined once in Rust, auto-exposed to Swift. Provider switchable from the iOS Settings screen.

Technical decisions that might interest HN:

**WebSocket mode for Responses API.** OpenAI has a wss:// endpoint at /v1/responses (separate from the Realtime API). The server caches context per-connection, so continuations are faster. They report ~40% improvement for 20+ tool calls. We measured 29-44% at n=5. The official Python and Node SDKs don't wrap this endpoint yet. openai-oxide does, with a connection pool (WsPool).

**1100+ types auto-synced from the Python SDK.** I wrote py2rust.py that parses Python Pydantic models and generates Rust serde structs. Two-pass resolver for cross-file references. Machine-generated files (_gen.rs) get overwritten on sync, manual overrides preserved. `make sync-types` when OpenAI changes something.

**Structured outputs via parse::\<T\>().** Derive JsonSchema on a Rust struct, call parse::\<T\>(request), get a typed response. Schema generation, API call, and deserialization in one step. Works on Chat and Responses APIs. On iOS this means the same typed structs from the Rust agent are available in Swift automatically via UniFFI.

**Honest benchmarks.** On single API calls (200ms-2s), SDK overhead doesn't matter — all clients are equivalent. Where it starts to matter: fast inference backends (Cerebras/Groq at 10-50ms per call), agent loops with many sequential requests, and voice apps where every ms counts. Mock benchmarks show 2-3x lower SDK overhead vs the official JS SDK (p<0.001). We spent several iterations removing claims that weren't statistically significant.

What I'd appreciate feedback on:
- Is the multi-platform Rust core approach interesting, or would you just use each language's native SDK?
- Anyone done UniFFI for LLM clients on iOS? Curious about your experience.

GitHub: https://github.com/fortunto2/openai-oxide
Coding agent: https://github.com/fortunto2/rust-code
