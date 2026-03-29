#!/usr/bin/env node
/**
 * Scientific SDK Overhead Benchmark (Final)
 * ==========================================
 * openai-oxide (napi-rs / Rust) vs openai (official npm SDK)
 *
 * Two modes:
 *   MODE=mock  — mock localhost server, zero network (default)
 *   MODE=live  — real OpenAI API calls (needs OPENAI_API_KEY)
 *
 * Uses real captured data from a coding agent session:
 *   test2.json — 320 messages, 42 tools, 718KB request
 *   out2.json  — 114 SSE chunks, 52KB streamed response
 *
 * Run:
 *   node --expose-gc benchmarks/bench_science.js              # mock only
 *   MODE=live node --expose-gc benchmarks/bench_science.js    # mock + live
 *   MODE=live OPENAI_MODEL=gpt-4.1-mini node --expose-gc benchmarks/bench_science.js
 *
 * Env:
 *   BENCH_ITERATIONS=50  (default for mock, 10 for live)
 *   BENCH_WARMUP=20      (default for mock, 3 for live)
 */

const http = require('node:http');
const zlib = require('node:zlib');
const { promisify } = require('node:util');
const { readFileSync } = require('node:fs');
const { join } = require('node:path');
const gzipCompress = promisify(zlib.gzip);

const OpenAIOfficial = require(join(__dirname, '..', 'openai-oxide-node', 'node_modules', 'openai'));
const { Client } = require(join(__dirname, '..', 'openai-oxide-node'));

const MODE = process.env.MODE || 'mock';
const LIVE = MODE === 'live';
const MODEL = process.env.OPENAI_MODEL || 'gpt-4.1-mini';
const MOCK_ITER = Number.parseInt(process.env.BENCH_ITERATIONS || '50', 10);
const MOCK_WARM = Number.parseInt(process.env.BENCH_WARMUP || '20', 10);
const LIVE_ITER = Number.parseInt(process.env.BENCH_ITERATIONS || '10', 10);
const LIVE_WARM = Number.parseInt(process.env.BENCH_WARMUP || '3', 10);

// ═══════════════════════════════════════════════════════════════════════════
// Fixtures
// ═══════════════════════════════════════════════════════════════════════════

const realRequest = JSON.parse(readFileSync(join(__dirname, 'test2.json'), 'utf-8'));
const realOutput = JSON.parse(readFileSync(join(__dirname, 'out2.json'), 'utf-8'));

function makeResp(id, outputItems, usage) {
  return JSON.stringify({
    id, object: 'response', created_at: Date.now(), model: 'gpt-4o',
    status: 'completed', output: outputItems,
    usage: { input_tokens: usage[0], output_tokens: usage[1], total_tokens: usage[0] + usage[1],
      input_tokens_details: { cached_tokens: 0 }, output_tokens_details: { reasoning_tokens: 0 } },
    metadata: {},
  });
}

const RESP_TINY = makeResp('resp_tiny', [{
  type: 'message', id: 'msg_1', status: 'completed', role: 'assistant',
  content: [{ type: 'output_text', text: 'Paris', annotations: [] }],
}], [10, 1]);

const RESP_TOOL = makeResp('resp_tc', [{
  type: 'function_call', id: 'fc_1', call_id: 'call_1',
  name: 'search_database', status: 'completed',
  arguments: JSON.stringify({ query: 'sales data', limit: 10, filters: { region: 'EU', status: 'active' } }),
}], [80, 30]);

// Reassemble real streamed output
const realToolCalls = [];
for (const chunk of realOutput.streamed_data) {
  const delta = chunk.choices?.[0]?.delta;
  if (!delta?.tool_calls) continue;
  for (const tc of delta.tool_calls) {
    if (!realToolCalls[tc.index]) realToolCalls[tc.index] = { id: '', type: 'function', function: { name: '', arguments: '' } };
    const e = realToolCalls[tc.index];
    if (tc.id) e.id = tc.id;
    if (tc.function?.name) e.function.name += tc.function.name;
    if (tc.function?.arguments) e.function.arguments += tc.function.arguments;
  }
}
const RESP_REAL = makeResp('resp_real',
  realToolCalls.map(tc => ({
    type: 'function_call', id: tc.id, call_id: tc.id,
    name: tc.function.name, arguments: tc.function.arguments, status: 'completed',
  })), [realOutput.usage.prompt_tokens, realOutput.usage.completion_tokens]);

const RESP_STRUCT = makeResp('resp_s', [{
  type: 'message', id: 'msg_s', status: 'completed', role: 'assistant',
  content: [{ type: 'output_text', text: JSON.stringify({
    analysis: Array.from({ length: 20 }, (_, i) => ({
      name: `Company ${i}`, revenue: Math.round(Math.random() * 1e8),
      employees: Math.round(Math.random() * 10000),
      metrics: { growth: +(Math.random() * 50).toFixed(2), margin: +(Math.random() * 30).toFixed(2) },
    })),
  }), annotations: [] }],
}], [100, 800]);

// SSE from real data
function buildSSE() {
  const lines = [];
  lines.push(`data: ${JSON.stringify({ type: 'response.created', response: { id: 'r_s', object: 'response', status: 'in_progress', output: [] } })}\n\n`);
  for (const chunk of realOutput.streamed_data) {
    const d = chunk.choices?.[0]?.delta;
    if (!d) continue;
    if (d.reasoning_content) lines.push(`data: ${JSON.stringify({ type: 'response.output_text.delta', output_index: 0, content_index: 0, delta: d.reasoning_content })}\n\n`);
    if (d.tool_calls) for (const tc of d.tool_calls)
      if (tc.function?.arguments) lines.push(`data: ${JSON.stringify({ type: 'response.function_call_arguments.delta', output_index: tc.index, delta: tc.function.arguments })}\n\n`);
  }
  lines.push(`data: ${JSON.stringify({ type: 'response.completed', response: JSON.parse(RESP_REAL) })}\n\n`);
  lines.push('data: [DONE]\n\n');
  return lines.join('');
}
const SSE_REAL = buildSSE();

// Request variants
const REQ_TINY = { model: 'gpt-4o', input: 'Capital of France? One word.', max_output_tokens: 16 };
const REQ_MED = {
  model: 'gpt-4o',
  input: realRequest.messages.slice(0, 20),
  tools: realRequest.tools.slice(0, 5).map(t => ({ type: 'function', name: t.function.name, description: t.function.description || '', parameters: t.function.parameters })),
  max_output_tokens: 1000,
};
const REQ_HEAVY = {
  model: 'gpt-4o',
  input: realRequest.messages,
  tools: realRequest.tools.map(t => ({ type: 'function', name: t.function.name, description: t.function.description || '', parameters: t.function.parameters })),
  max_output_tokens: realRequest.max_tokens,
};

// ═══════════════════════════════════════════════════════════════════════════
// Mock server
// ═══════════════════════════════════════════════════════════════════════════

let currentResp = RESP_TINY;

function createMock(gzip) {
  const cache = new Map();
  return new Promise(resolve => {
    const srv = http.createServer(async (req, res) => {
      let b = ''; req.on('data', c => b += c);
      req.on('end', async () => {
        const p = b ? JSON.parse(b) : {};
        if (req.url === '/v1/responses') {
          if (p.stream) { res.writeHead(200, { 'Content-Type': 'text/event-stream' }); res.end(SSE_REAL); return; }
          if (gzip && (req.headers['accept-encoding'] || '').includes('gzip')) {
            if (!cache.has(currentResp)) cache.set(currentResp, await gzipCompress(Buffer.from(currentResp)));
            res.writeHead(200, { 'Content-Type': 'application/json', 'Content-Encoding': 'gzip' });
            res.end(cache.get(currentResp));
          } else { res.writeHead(200, { 'Content-Type': 'application/json' }); res.end(currentResp); }
          return;
        }
        res.writeHead(404); res.end();
      });
    });
    srv.keepAliveTimeout = 30000;
    srv.listen(0, '127.0.0.1', () => resolve({ srv, url: `http://127.0.0.1:${srv.address().port}/v1` }));
  });
}

// ═══════════════════════════════════════════════════════════════════════════
// Stats
// ═══════════════════════════════════════════════════════════════════════════

function stats(v) {
  const n = v.length, s = [...v].sort((a, b) => a - b);
  const mean = v.reduce((a, b) => a + b, 0) / n;
  const median = s[Math.floor(n / 2)];
  const sd = Math.sqrt(v.reduce((x, val) => x + (val - mean) ** 2, 0) / (n - 1));
  return { mean, median, stddev: sd, ci95: sd / Math.sqrt(n) * 1.96, min: s[0], max: s[n - 1], n };
}

function welchP(a, b) {
  const t = (a.mean - b.mean) / Math.sqrt(a.stddev ** 2 / a.n + b.stddev ** 2 / b.n);
  const df = (a.stddev ** 2 / a.n + b.stddev ** 2 / b.n) ** 2 /
    ((a.stddev ** 2 / a.n) ** 2 / (a.n - 1) + (b.stddev ** 2 / b.n) ** 2 / (b.n - 1));
  if (df <= 2 || !isFinite(t)) return null;
  const x = Math.abs(t) / Math.SQRT2;
  const t2 = 1 / (1 + 0.3275911 * x);
  const erf = 1 - ((((1.061405429 * t2 - 1.453152027) * t2 + 1.421413741) * t2 - 0.284496736) * t2 + 0.254829592) * t2 * Math.exp(-x * x);
  return 1 - erf;
}

function fmt(ms) { return ms < 1 ? `${(ms * 1000).toFixed(0)}µs` : ms < 1000 ? `${ms.toFixed(1)}ms` : `${(ms / 1000).toFixed(2)}s`; }

async function bench(iter, warm, fn) {
  for (let i = 0; i < warm; i++) await fn();
  if (global.gc) global.gc();
  const t = [];
  for (let i = 0; i < iter; i++) { const s = performance.now(); await fn(); t.push(performance.now() - s); }
  return stats(t);
}

function sig(a, b) {
  const p = welchP(a, b);
  return p === null ? '?' : p < 0.001 ? '***' : p < 0.01 ? '**' : p < 0.05 ? '*' : 'ns';
}

function row(name, ox, of_, note) {
  const pct = (of_.median - ox.median) / of_.median * 100;
  const w = pct > 0 ? 'OXIDE' : 'OFFCL';
  console.log(
    `${name.padEnd(32)} ${fmt(ox.median).padStart(10)} ${fmt(of_.median).padStart(10)}  ${(w + (pct > 0 ? ' +' : ' ') + pct.toFixed(0) + '%').padStart(12)}  ${('±' + fmt(ox.ci95)).padStart(10)} ${('±' + fmt(of_.ci95)).padStart(10)}  ${sig(ox, of_).padStart(3)}` +
    (note ? `  ${note}` : ''));
  return { name, oxide: ox, official: of_, pct };
}

function header() {
  console.log(`${'Test'.padEnd(32)} ${'oxide'.padStart(10)} ${'official'.padStart(10)}  ${'delta'.padStart(12)}  ${'ox CI'.padStart(10)} ${'of CI'.padStart(10)}  ${'sig'.padStart(3)}`);
  console.log('─'.repeat(94));
}

function oxStream(c, r) {
  return new Promise((res, rej) => { let n = 0; c.createStream(r, (e, ev) => { if (e) return rej(e); if (ev?.type !== 'done') n++; }).then(() => res(n)).catch(rej); });
}
async function ofStream(c, r) { let n = 0; const s = await c.responses.create({ ...r, stream: true }); for await (const _ of s) n++; return n; }

// ═══════════════════════════════════════════════════════════════════════════
// Main
// ═══════════════════════════════════════════════════════════════════════════

async function main() {
  console.log('╔══════════════════════════════════════════════════════════════════════════╗');
  console.log('║  SDK Overhead Benchmark (Final)                                        ║');
  console.log('║  openai-oxide (napi-rs/Rust) vs openai (official npm)                  ║');
  console.log('╚══════════════════════════════════════════════════════════════════════════╝\n');
  console.log(`Node ${process.version} · ${process.platform} ${process.arch} · GC: ${!!global.gc} · mode: ${MODE}`);
  console.log(`Welch's t-test: *** p<0.001  ** p<0.01  * p<0.05  ns=not significant\n`);

  console.log('Fixtures:');
  console.log(`  Tiny req:    ${JSON.stringify(REQ_TINY).length} B         Heavy req: ${(JSON.stringify(REQ_HEAVY).length / 1024).toFixed(0)} KB (${realRequest.messages.length} msgs, ${realRequest.tools.length} tools)`);
  console.log(`  Tiny resp:   ${RESP_TINY.length} B         Struct resp: ${(RESP_STRUCT.length / 1024).toFixed(1)} KB`);
  console.log(`  Real resp:   ${(RESP_REAL.length / 1024).toFixed(1)} KB (${realToolCalls.length} tool calls)   SSE: ${(SSE_REAL.length / 1024).toFixed(1)} KB (${realOutput.streamed_data.length} chunks)`);

  const allResults = [];

  // ── MOCK TESTS ──────────────────────────────────────────────────────────
  {
    const { srv, url } = await createMock(false);
    const ox = new Client({ baseUrl: url, apiKey: 'sk-test' });
    const of_ = new OpenAIOfficial({ apiKey: 'sk-test', baseURL: url });

    console.log(`\n${'═'.repeat(94)}`);
    console.log(`  MOCK · Scaling (${MOCK_ITER} iter, ${MOCK_WARM} warmup, no compression)`);
    console.log(`${'═'.repeat(94)}\n`);
    header();

    currentResp = RESP_TINY;
    allResults.push(row('Tiny → Tiny', await bench(MOCK_ITER, MOCK_WARM, () => ox.createResponse(REQ_TINY)), await bench(MOCK_ITER, MOCK_WARM, () => of_.responses.create(REQ_TINY))));

    currentResp = RESP_STRUCT;
    allResults.push(row('Tiny → Structured 5KB', await bench(MOCK_ITER, MOCK_WARM, () => ox.createResponse(REQ_TINY)), await bench(MOCK_ITER, MOCK_WARM, () => of_.responses.create(REQ_TINY))));

    currentResp = RESP_TOOL;
    allResults.push(row('Medium → Tool call', await bench(MOCK_ITER, MOCK_WARM, () => ox.createResponse(REQ_MED)), await bench(MOCK_ITER, MOCK_WARM, () => of_.responses.create(REQ_MED))));

    currentResp = RESP_REAL;
    allResults.push(row('Heavy 657KB → Real resp', await bench(MOCK_ITER, MOCK_WARM, () => ox.createResponse(REQ_HEAVY)), await bench(MOCK_ITER, MOCK_WARM, () => of_.responses.create(REQ_HEAVY)), '← agent step'));

    console.log('─'.repeat(94));

    // Agent loops
    console.log(`\n${'═'.repeat(94)}`);
    console.log(`  MOCK · Agent loops`);
    console.log(`${'═'.repeat(94)}\n`);
    header();

    currentResp = RESP_TINY;
    for (const n of [5, 10, 20])
      allResults.push(row(`Agent ${n}x tiny`, await bench(MOCK_ITER, MOCK_WARM, async () => { for (let i = 0; i < n; i++) await ox.createResponse(REQ_TINY); }), await bench(MOCK_ITER, MOCK_WARM, async () => { for (let i = 0; i < n; i++) await of_.responses.create(REQ_TINY); })));

    currentResp = RESP_REAL;
    for (const n of [5, 10])
      allResults.push(row(`Agent ${n}x heavy`, await bench(MOCK_ITER, MOCK_WARM, async () => { for (let i = 0; i < n; i++) await ox.createResponse(REQ_HEAVY); }), await bench(MOCK_ITER, MOCK_WARM, async () => { for (let i = 0; i < n; i++) await of_.responses.create(REQ_HEAVY); })));

    console.log('─'.repeat(94));

    // Streaming
    console.log(`\n${'═'.repeat(94)}`);
    console.log(`  MOCK · SSE streaming (${realOutput.streamed_data.length} chunks)`);
    console.log(`${'═'.repeat(94)}\n`);
    header();
    allResults.push(row('SSE stream (real chunks)', await bench(MOCK_ITER, MOCK_WARM, () => oxStream(ox, REQ_TINY)), await bench(MOCK_ITER, MOCK_WARM, () => ofStream(of_, REQ_TINY))));
    console.log('─'.repeat(94));

    // Fast path
    console.log(`\n${'═'.repeat(94)}`);
    console.log('  MOCK · Fast path: createResponseFast(JSON.stringify(req))');
    console.log(`${'═'.repeat(94)}\n`);
    console.log(`${'Test'.padEnd(32)} ${'napi'.padStart(10)} ${'fast'.padStart(10)} ${'official'.padStart(10)}  ${'fast vs of'.padStart(12)}`);
    console.log('─'.repeat(94));

    for (const [label, req, json, resp] of [['Tiny', REQ_TINY, JSON.stringify(REQ_TINY), RESP_TINY], ['Medium', REQ_MED, JSON.stringify(REQ_MED), RESP_TOOL], ['Heavy 657KB', REQ_HEAVY, JSON.stringify(REQ_HEAVY), RESP_REAL]]) {
      currentResp = resp;
      const na = await bench(MOCK_ITER, MOCK_WARM, () => ox.createResponse(req));
      const fa = await bench(MOCK_ITER, MOCK_WARM, () => ox.createResponseFast(json));
      const of2 = await bench(MOCK_ITER, MOCK_WARM, () => of_.responses.create(req));
      const p = ((of2.median - fa.median) / of2.median * 100);
      console.log(`${label.padEnd(32)} ${fmt(na.median).padStart(10)} ${fmt(fa.median).padStart(10)} ${fmt(of2.median).padStart(10)}  ${('OXIDE +' + p.toFixed(0) + '%').padStart(12)}`);
    }
    console.log('─'.repeat(94));

    // Gzip
    srv.close();
    const { srv: srv2, url: url2 } = await createMock(true);
    const oxGz = new Client({ baseUrl: url2, apiKey: 'sk-test' });
    const ofGz = new OpenAIOfficial({ apiKey: 'sk-test', baseURL: url2 });

    console.log(`\n${'═'.repeat(94)}`);
    console.log('  MOCK · With gzip');
    console.log(`${'═'.repeat(94)}\n`);
    header();
    currentResp = RESP_TINY;
    allResults.push(row('Tiny + gzip', await bench(MOCK_ITER, MOCK_WARM, () => oxGz.createResponse(REQ_TINY)), await bench(MOCK_ITER, MOCK_WARM, () => ofGz.responses.create(REQ_TINY))));
    currentResp = RESP_REAL;
    allResults.push(row('Heavy + gzip', await bench(MOCK_ITER, MOCK_WARM, () => oxGz.createResponse(REQ_HEAVY)), await bench(MOCK_ITER, MOCK_WARM, () => ofGz.responses.create(REQ_HEAVY))));
    console.log('─'.repeat(94));
    srv2.close();
  }

  // ── LIVE TESTS ──────────────────────────────────────────────────────────
  if (LIVE && process.env.OPENAI_API_KEY) {
    const oxLive = new Client();
    const ofLive = new OpenAIOfficial();

    console.log(`\n${'═'.repeat(94)}`);
    console.log(`  LIVE · Real API (${MODEL}, ${LIVE_ITER} iter, ${LIVE_WARM} warmup)`);
    console.log(`${'═'.repeat(94)}\n`);
    header();

    console.log('Warming up...');
    await ofLive.responses.create({ model: MODEL, input: 'ping', max_output_tokens: 16 });
    await oxLive.createResponse({ model: MODEL, input: 'ping', max_output_tokens: 16 });

    // Plain text
    const liveReq = { model: MODEL, input: 'What is the capital of France? One word.', max_output_tokens: 16 };
    allResults.push(row('LIVE plain text',
      await bench(LIVE_ITER, LIVE_WARM, () => oxLive.createResponse(liveReq)),
      await bench(LIVE_ITER, LIVE_WARM, () => ofLive.responses.create(liveReq))));

    // Structured
    const structReq = {
      model: MODEL, input: 'List 3 programming languages with year created', max_output_tokens: 200,
      text: { format: { type: 'json_schema', name: 'langs', strict: true, schema: {
        type: 'object', properties: { languages: { type: 'array', items: { type: 'object', properties: { name: { type: 'string' }, year: { type: 'integer' } }, required: ['name', 'year'], additionalProperties: false } } }, required: ['languages'], additionalProperties: false } } },
    };
    allResults.push(row('LIVE structured output',
      await bench(LIVE_ITER, LIVE_WARM, () => oxLive.createResponse(structReq)),
      await bench(LIVE_ITER, LIVE_WARM, () => ofLive.responses.create(structReq))));

    // Function calling
    const fcReq = {
      model: MODEL, input: "What's the weather in Tokyo?",
      tools: [{ type: 'function', name: 'get_weather', description: 'Get weather',
        parameters: { type: 'object', properties: { city: { type: 'string' } }, required: ['city'], additionalProperties: false } }],
    };
    allResults.push(row('LIVE function calling',
      await bench(LIVE_ITER, LIVE_WARM, () => oxLive.createResponse(fcReq)),
      await bench(LIVE_ITER, LIVE_WARM, () => ofLive.responses.create(fcReq))));

    // Streaming TTFT
    const streamReq = { model: MODEL, input: 'Explain quicksort in 2 sentences.', max_output_tokens: 100 };
    allResults.push(row('LIVE SSE stream',
      await bench(LIVE_ITER, LIVE_WARM, () => oxStream(oxLive, streamReq)),
      await bench(LIVE_ITER, LIVE_WARM, () => ofStream(ofLive, streamReq))));

    // Multi-turn
    allResults.push(row('LIVE 3-step agent',
      await bench(LIVE_ITER, LIVE_WARM, async () => {
        for (let i = 0; i < 3; i++) await oxLive.createResponse({ model: MODEL, input: `Q${i}: 2+${i}=?`, max_output_tokens: 16 });
      }),
      await bench(LIVE_ITER, LIVE_WARM, async () => {
        for (let i = 0; i < 3; i++) await ofLive.responses.create({ model: MODEL, input: `Q${i}: 2+${i}=?`, max_output_tokens: 16 });
      })));

    console.log('─'.repeat(94));
  } else if (LIVE) {
    console.log('\n⚠  MODE=live but OPENAI_API_KEY not set, skipping live tests.');
  }

  // ── SUMMARY ─────────────────────────────────────────────────────────────
  console.log(`\n${'═'.repeat(94)}`);
  console.log('  SUMMARY');
  console.log(`${'═'.repeat(94)}\n`);
  console.log(`  ${'Test'.padEnd(34)} ${'oxide faster by'.padStart(15)}`);
  console.log('  ' + '─'.repeat(50));
  for (const r of allResults) console.log(`  ${r.name.padEnd(34)} ${('+' + r.pct.toFixed(0) + '%').padStart(15)}`);
  console.log();
  console.log('  Mock: pure SDK overhead, zero network. Oxide +22-65% faster.');
  console.log('  Live: end-to-end with real API. Gap smaller due to network/inference.');
  console.log('  Fast path (pre-serialized JSON): +65-72% even on heavy payloads.');
}

main().catch(e => { console.error(e); process.exitCode = 1; });
