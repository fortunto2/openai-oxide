/**
 * Typed wrapper for openai-oxide Node.js client.
 *
 * Provides full TypeScript autocompletion using official OpenAI SDK types.
 * Import from 'openai-oxide' — the wrapper.js auto-applies these types.
 */

import type {
  ChatCompletion,
  ChatCompletionCreateParams,
} from 'openai/resources/chat/completions';

import type {
  Response,
  ResponseCreateParams,
  ResponseCreateParamsNonStreaming,
  ResponseStreamEvent,
  ResponseTool,
} from 'openai/resources/responses/responses';

export interface ClientOptions {
  baseUrl?: string;
  apiKey?: string;
}

export declare class Client {
  constructor(options?: ClientOptions);

  /** Chat Completions API — same params as official SDK. */
  createChatCompletion(request: ChatCompletionCreateParams): Promise<ChatCompletion>;

  /** Chat completion with structured output (Zod schema). */
  createChatParsed(
    request: ChatCompletionCreateParams,
    schemaName: string,
    schema: Record<string, any>,
  ): Promise<{ completion: ChatCompletion; parsed: any }>;

  /** Responses API — same params as official SDK. */
  createResponse(request: ResponseCreateParamsNonStreaming): Promise<Response>;

  /** Responses API with structured output. */
  createResponseParsed(
    request: ResponseCreateParamsNonStreaming,
    schemaName: string,
    schema: Record<string, any>,
  ): Promise<{ response: Response; parsed: any }>;

  /** Fast path: pre-serialized JSON, avoids napi copy overhead on large payloads. */
  createResponseFast(jsonBody: string): Promise<Response>;

  /** Short-hand: returns just the text content. */
  createText(model: string, input: string, maxOutputTokens?: number): Promise<string>;

  /** Short-hand: returns stored response ID for multi-turn. */
  createStoredResponseId(model: string, input: string, maxOutputTokens?: number): Promise<string>;

  /** Multi-turn follow-up using previous_response_id. */
  createTextFollowup(
    model: string,
    input: string,
    previousResponseId: string,
    maxOutputTokens?: number,
  ): Promise<string>;

  /** Streaming — callback receives typed SSE events. */
  createStream(
    request: ResponseCreateParamsNonStreaming,
    callback: (err: Error | null, event: ResponseStreamEvent | null) => void,
  ): Promise<void>;

  /** Streaming chat completions. */
  createChatStream(
    request: ChatCompletionCreateParams,
    callback: (err: Error | null, event: any) => void,
  ): Promise<void>;

  /** Fast streaming: pre-serialized JSON. */
  createStreamFast(
    jsonBody: string,
    callback: (err: Error | null, event: ResponseStreamEvent | null) => void,
  ): Promise<void>;

  /** Open persistent WebSocket session for lower latency. */
  wsSession(): Promise<NodeWsSession>;
}

export declare class NodeWsSession {
  send(model: string, input: string): Promise<Response>;
  close(): Promise<void>;
}
