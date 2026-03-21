"""Test openai-oxide Python bindings."""
import asyncio
import json
import time

from openai_oxide_python import Client


async def main():
    client = Client()  # reads OPENAI_API_KEY

    print("=== openai-oxide Python bindings test ===\n")

    # 1. Simple
    t0 = time.perf_counter()
    r = await client.create("gpt-5.4", "What is 2+2? One number.", max_output_tokens=16)
    ms = int((time.perf_counter() - t0) * 1000)
    data = json.loads(r)
    print(f"1. Simple:     {ms}ms — {data['text']}")

    # 2. Structured
    schema = json.dumps({
        "type": "object",
        "properties": {"answer": {"type": "integer"}},
        "required": ["answer"],
        "additionalProperties": False,
    })
    t0 = time.perf_counter()
    r = await client.create_structured("gpt-5.4", "What is 7*8?", "math", schema, max_output_tokens=32)
    ms = int((time.perf_counter() - t0) * 1000)
    data = json.loads(json.loads(r)["text"])
    print(f"2. Structured: {ms}ms — answer={data['answer']}")

    # 3. Function calling
    tools = json.dumps([{
        "name": "get_weather",
        "description": "Get weather",
        "parameters": {
            "type": "object",
            "properties": {"city": {"type": "string"}},
            "required": ["city"],
            "additionalProperties": False,
        },
    }])
    t0 = time.perf_counter()
    r = await client.create_with_tools("gpt-5.4", "Weather in Paris?", tools)
    ms = int((time.perf_counter() - t0) * 1000)
    data = json.loads(r)
    fc = data["function_calls"][0] if data["function_calls"] else {}
    print(f"3. FC:         {ms}ms — {fc.get('name', '?')}({json.dumps(fc.get('arguments', {}))})")

    # 4. Compare with Python openai SDK
    try:
        from openai import OpenAI as PyOpenAI
        py_client = PyOpenAI()
        t0 = time.perf_counter()
        py_r = py_client.responses.create(model="gpt-5.4", input="What is 2+2? One number.", max_output_tokens=16)
        py_ms = int((time.perf_counter() - t0) * 1000)
        print(f"\n4. Python SDK: {py_ms}ms — {py_r.output_text}")
    except ImportError:
        print("\n4. Python SDK: not installed (pip install openai)")

    print("\n=== Done ===")


if __name__ == "__main__":
    asyncio.run(main())
