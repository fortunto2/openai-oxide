/**
 * TypeScript types for openai-oxide.
 *
 * Re-exports types from the official OpenAI SDK for full compatibility.
 * Our native Client accepts the same request/response shapes.
 *
 * Usage:
 *   import type { ChatCompletionCreateParams } from 'openai-oxide/types';
 *   import type { ResponseCreateParams, Response } from 'openai-oxide/types';
 */

// Re-export all types from official OpenAI SDK
export type {
  ChatCompletion,
  ChatCompletionAssistantMessageParam,
  ChatCompletionChunk,
  ChatCompletionCreateParams,
  ChatCompletionCreateParamsNonStreaming,
  ChatCompletionCreateParamsStreaming,
  ChatCompletionMessage,
  ChatCompletionMessageParam,
  ChatCompletionSystemMessageParam,
  ChatCompletionTool,
  ChatCompletionToolMessageParam,
  ChatCompletionUserMessageParam,
} from 'openai/resources/chat/completions';

export type {
  Response,
  ResponseCreateParams,
  ResponseCreateParamsNonStreaming,
  ResponseCreateParamsStreaming,
  ResponseFunctionToolCall,
  ResponseInputItem,
  ResponseOutputItem,
  ResponseOutputMessage,
  ResponseOutputText,
  ResponseStreamEvent,
  ResponseTool,
  Tool,
} from 'openai/resources/responses/responses';

export type {
  Embedding,
  EmbeddingCreateParams,
} from 'openai/resources/embeddings';

export type {
  Moderation,
  ModerationCreateParams,
} from 'openai/resources/moderations';

export type {
  FileObject,
} from 'openai/resources/files';

export type {
  Model,
} from 'openai/resources/models';

export type {
  ImagesResponse,
  ImageGenerateParams,
  ImageEditParams,
} from 'openai/resources/images';
