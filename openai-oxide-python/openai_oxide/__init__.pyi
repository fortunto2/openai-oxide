"""Type stubs for openai-oxide Python bindings."""

from typing import Any, AsyncIterator, Optional

class Client:
    """Fastest OpenAI client — Rust core, Python interface.

    Uses the same request/response format as the official OpenAI Python SDK.
    Accepts OPENAI_API_KEY env var or explicit api_key parameter.
    """

    def __init__(self, *, api_key: Optional[str] = None, base_url: Optional[str] = None) -> None: ...

    async def create(
        self,
        model: str,
        input: str,
        *,
        max_output_tokens: Optional[int] = None,
        temperature: Optional[float] = None,
        instructions: Optional[str] = None,
    ) -> str:
        """Create a response (Responses API). Returns JSON string."""
        ...

    async def create_stream(
        self,
        model: str,
        input: str,
        *,
        max_output_tokens: Optional[int] = None,
        temperature: Optional[float] = None,
    ) -> AsyncIterator[str]:
        """Create streaming response. Yields SSE event strings."""
        ...

    async def create_chat(
        self,
        model: str,
        messages: list[dict[str, Any]],
        *,
        max_tokens: Optional[int] = None,
        temperature: Optional[float] = None,
    ) -> str:
        """Chat Completions API. Returns JSON string."""
        ...
