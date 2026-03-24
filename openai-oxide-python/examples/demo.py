#!/usr/bin/env python3
"""
Drop-in replacement for official openai SDK demo.
Change: `from openai import AsyncOpenAI` → `from openai_oxide.compat import AsyncOpenAI`
"""

import asyncio

# ── One-line change from official SDK ──
# from openai import AsyncOpenAI
from openai_oxide.compat import AsyncOpenAI


async def main():
    client = AsyncOpenAI()

    # Non-streaming:
    print("----- standard request -----")
    completion = await client.chat.completions.create(
        model="gpt-5.4-mini",
        messages=[
            {
                "role": "user",
                "content": "Say this is a test",
            },
        ],
    )
    print(completion.choices[0].message.content)

    # Streaming:
    print("----- streaming request -----")
    stream = await client.chat.completions.create(
        model="gpt-5.4-mini",
        messages=[
            {
                "role": "user",
                "content": "How do I output all files in a directory using Python?",
            },
        ],
        stream=True,
    )
    async for event in stream:
        if event.get("type") == "OutputTextDelta":
            print(event.get("delta", ""), end="")
        elif event.get("delta"):
            print(event.get("delta", ""), end="")
    print()


asyncio.run(main())
