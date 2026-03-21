# Full-Stack Rust WASM Example

A production-ready architecture demonstrating a 100% Rust stack using **Dioxus** on the frontend and a **Cloudflare Worker** on the backend. 

## Features Used
- **Prompt Caching (`prompt_cache_key`, `prompt_cache_retention`)**: Chat history is sent in full on every request. By using `openai-oxide`'s prompt caching, the OpenAI server caches the chat history prefix natively. This results in **-80% lower latency** and massive cost savings on multi-turn conversations.
- **WebSocket Mode (`websocket` feature)**: Cloudflare Workers are stateless, but this example uses **Durable Objects** to hold a persistent `wss://` connection to the OpenAI Responses API. This eliminates the latency of TLS handshakes per message.
- **Full WASM Support**: `openai-oxide` compiles natively to `wasm32-unknown-unknown`. Unlike other libraries that strip features in WASM, Oxide maintains full functionality.

## Why this architecture?
1. **Zero JS**: Both the UI and the Backend are written in Rust.
2. **Edge Performance**: Deploying to Cloudflare puts the compute physically closer to the user.
3. **Lowest Latency**: Connecting a Browser WebSocket -> Cloudflare Durable Object -> OpenAI `wss://` yields the lowest Time-To-First-Token (TTFT) possible (~350ms).
4. **Chat History via Caching**: The frontend maintains the state and sends it completely on every new message, allowing OpenAI's server-side prompt caching to handle context efficiently without requiring a separate database for history.

## Deploy

[![Deploy to Cloudflare Workers](https://deploy.workers.cloudflare.com/button)](https://deploy.workers.cloudflare.com/?url=https://github.com/fortunto2/openai-rust/tree/main/examples/cloudflare-worker-dioxus)

*(Note: Deploying this requires a Paid Cloudflare Workers plan because it uses Durable Objects).*

## Local Dev

```bash
# Add your API Key
cd worker
npx wrangler secret put OPENAI_API_KEY

# Build the Dioxus App & run the Worker
cd ..
./build.sh
```
