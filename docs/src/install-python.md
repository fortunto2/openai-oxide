# Python Installation

## Install

```bash
pip install openai-oxide
# or
uv pip install openai-oxide
# or
uv add openai-oxide
```

No Rust toolchain required — prebuilt wheels available.

## Setup

```python
from openai_oxide import Client

# Uses OPENAI_API_KEY from environment
client = Client()

# Explicit key
client = Client("sk-...")
```

## Available Methods

| Method | Description |
|--------|-------------|
| `await client.create(model, input)` | Basic request |
| `await client.create_stream(model, input)` | Streaming |
| `await client.create_structured(model, input, name, schema)` | Structured output |
| `await client.create_with_tools(model, input, tools)` | Function calling |

## PyPI Package

[pypi.org/project/openai-oxide](https://pypi.org/project/openai-oxide/)
