# OpenAI Oxide Cloudflare Worker Example

A lightweight Cloudflare Worker example demonstrating how to use `openai-oxide` in a WASM environment.

## Features Used
- **WASM Support:** Compiles seamlessly to `wasm32-unknown-unknown` without losing streaming or fast-path execution features.
- **Strongly Typed Responses API:** Uses `ResponseCreateRequest` for an elegant structured request.
- **Streaming Support:** Capable of handling server-sent events (`ResponseStreamEvent`) natively on Cloudflare Workers edge nodes.

## Why this architecture?
Cloudflare Workers run on the edge using V8 isolates, meaning there's no native OS-level threading or standard network sockets. Most Rust SDKs rely on `tokio` and native TLS which fail to compile to WASM. `openai-oxide` avoids these issues through `gloo-net` and specialized WASM targets, allowing high-performance Rust AI logic to run directly on the edge.

## Deploy

[![Deploy to Cloudflare Workers](https://deploy.workers.cloudflare.com/button)](https://deploy.workers.cloudflare.com/?url=https://github.com/fortunto2/openai-rust/tree/main/examples/cloudflare-worker)

## Local Dev

```bash
# Add your API Key
npx wrangler secret put OPENAI_API_KEY

# Run locally
npx wrangler dev
```
