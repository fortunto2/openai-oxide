#!/usr/bin/env python3
"""
Drop-in replacement for official openai SDK parsing example.
Change: `from openai import AsyncOpenAI` → `from openai_oxide.compat import AsyncOpenAI`
"""

import asyncio
from typing import List

from pydantic import BaseModel

# ── One-line change from official SDK ──
# from openai import AsyncOpenAI
from openai_oxide.compat import AsyncOpenAI


class Step(BaseModel):
    explanation: str
    output: str


class MathResponse(BaseModel):
    steps: List[Step]
    final_answer: str


async def main():
    client = AsyncOpenAI()

    completion = await client.chat.completions.parse(
        model="gpt-5.4-mini",
        messages=[
            {"role": "system", "content": "You are a helpful math tutor."},
            {"role": "user", "content": "solve 8x + 31 = 2"},
        ],
        response_format=MathResponse,
    )

    message = completion.choices[0].message
    if message.parsed:
        for step in message.parsed.steps:
            print(f"  {step.explanation} → {step.output}")
        print("answer:", message.parsed.final_answer)
    else:
        print("refusal:", message.refusal)


asyncio.run(main())
