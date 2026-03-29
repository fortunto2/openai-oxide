// Smart wrapper: auto-routes through fast path (pre-serialized JSON)
// to avoid napi object→Value copy overhead on large payloads.
//
// Benchmark: 657KB request → 2293µs (napi copy) vs 475µs (fast path) = 4.8x faster

const native = require('./index.js');

const FAST_PATH_THRESHOLD = 8192; // bytes — use fast path for requests > 8KB

class Client extends native.Client {
  /**
   * Create a response — auto-uses fast path for large requests.
   * Same API as native createResponse, but avoids napi copy overhead.
   */
  async createResponse(request) {
    // Estimate size: fast path for large payloads
    const json = JSON.stringify(request);
    if (json.length > FAST_PATH_THRESHOLD) {
      return super.createResponseFast(json);
    }
    return super.createResponse(request);
  }

  /**
   * Create chat completion — auto-uses fast path for large requests.
   */
  async createChatCompletion(request) {
    const json = JSON.stringify(request);
    if (json.length > FAST_PATH_THRESHOLD) {
      // For chat completions, fast path sends raw JSON bytes
      return super.createResponseFast(json);
    }
    return super.createChatCompletion(request);
  }

  /**
   * Create stream — auto-uses fast path for large requests.
   */
  async createStream(request, callback) {
    const json = JSON.stringify(request);
    if (json.length > FAST_PATH_THRESHOLD) {
      return super.createStreamFast(json, callback);
    }
    return super.createStream(request, callback);
  }
}

module.exports = { ...native, Client };
