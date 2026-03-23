# Quick Start

Set your API key:

```bash
export OPENAI_API_KEY="sk-..."
```

## Rust

```rust
use openai_oxide::{OpenAI, types::responses::*};

#[tokio::main]
async fn main() -> Result<(), openai_oxide::OpenAIError> {
    let client = OpenAI::from_env()?;

    let response = client.responses().create(
        ResponseCreateRequest::new("gpt-4o-mini")
            .input("Explain quantum computing in one sentence.")
            .max_output_tokens(100)
    ).await?;

    println!("{}", response.output_text());
    Ok(())
}
```

## Node.js

```javascript
const { Client } = require("openai-oxide");

const client = new Client();
const text = await client.createText("gpt-4o-mini", "Hello from Node!");
console.log(text);
```

## Python

```python
import asyncio
from openai_oxide import Client

async def main():
    client = Client()
    res = await client.create("gpt-4o-mini", "Hello from Python!")
    print(res["text"])

asyncio.run(main())
```
