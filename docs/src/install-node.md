# Node.js Installation

## Install

```bash
npm install openai-oxide
# or
pnpm add openai-oxide
# or
yarn add openai-oxide
```

Prebuilt native binaries for: macOS (x64, arm64), Linux (x64, arm64, glibc & musl), Windows (x64).

## Setup

```javascript
const { Client } = require("openai-oxide");

// Uses OPENAI_API_KEY from environment
const client = new Client();

// Explicit key
const client = new Client("sk-...");
```

## Available Methods

| Method | Description |
|--------|-------------|
| `createResponse(params)` | Full Responses API call |
| `createText(model, input)` | Fast path — returns text only |
| `createStoredResponseId(model, input)` | Fast path — returns response ID |
| `createTextFollowup(model, input, prevId)` | Multi-turn fast path |
| `createStream(model, input)` | Streaming responses |
| `wsSession()` | WebSocket persistent connection |

## npm Package

[npmjs.com/package/openai-oxide](https://www.npmjs.com/package/openai-oxide)
