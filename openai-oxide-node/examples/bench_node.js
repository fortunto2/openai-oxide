const OpenAI = require('openai');
const { Client } = require('openai-oxide');

const MODEL = process.env.OPENAI_MODEL || 'gpt-5.4';
const ITERATIONS = Number.parseInt(process.env.BENCH_ITERATIONS || '5', 10);

if (!process.env.OPENAI_API_KEY) {
  console.error('OPENAI_API_KEY is required');
  process.exit(1);
}

const official = new OpenAI({ apiKey: process.env.OPENAI_API_KEY });
const oxide = new Client();

const weatherTools = [
  {
    type: 'function',
    name: 'get_weather',
    description: 'Get weather',
    parameters: {
      type: 'object',
      properties: {
        city: { type: 'string' },
        unit: { type: 'string', enum: ['celsius', 'fahrenheit'] },
      },
      required: ['city', 'unit'],
      additionalProperties: false,
    },
  },
];

const languageSchema = {
  type: 'object',
  properties: {
    languages: {
      type: 'array',
      items: {
        type: 'object',
        properties: {
          name: { type: 'string' },
          year: { type: 'integer' },
        },
        required: ['name', 'year'],
        additionalProperties: false,
      },
    },
  },
  required: ['languages'],
  additionalProperties: false,
};

function median(values) {
  if (values.length === 0) return 0;
  const sorted = [...values].sort((a, b) => a - b);
  return sorted[Math.floor(sorted.length / 2)];
}

function formatMs(value) {
  return value === 'N/A' ? 'N/A' : `${value}ms`;
}

function winnerLabel(oxideMs, officialMs) {
  if (oxideMs === 'N/A') return 'official';
  if (officialMs === 'N/A') return 'oxide';
  return oxideMs <= officialMs ? 'oxide' : 'official';
}

function printRow(name, oxideMs, officialMs) {
  const winner = winnerLabel(oxideMs, officialMs);
  console.log(
    `${name.padEnd(24)} ${formatMs(oxideMs).padStart(10)} ${formatMs(officialMs).padStart(10)} ${winner.padStart(10)}`
  );
}

async function sample(iterations, fn) {
  const times = [];
  for (let i = 0; i < iterations; i += 1) {
    const start = performance.now();
    await fn();
    times.push(Math.round(performance.now() - start));
  }
  return median(times);
}

async function oxideStreamTtft() {
  return new Promise((resolve, reject) => {
    let done = false;
    const startedAt = performance.now();

    oxide
      .createStream(
        {
          model: MODEL,
          input: 'Explain quicksort in 3 sentences.',
          max_output_tokens: 200,
        },
        (err, event) => {
          if (done) return;
          if (err) {
            done = true;
            reject(err);
            return;
          }
          if (event?.type === 'response.output_text.delta') {
            done = true;
            resolve(Math.round(performance.now() - startedAt));
            return;
          }
          if (event?.type === 'done') {
            done = true;
            reject(new Error('oxide stream ended before first text delta'));
          }
        }
      )
      .catch((error) => {
        if (!done) {
          done = true;
          reject(error);
        }
      });
  });
}

async function officialStreamTtft() {
  const startedAt = performance.now();
  const stream = await official.responses.create({
    model: MODEL,
    input: 'Explain quicksort in 3 sentences.',
    max_output_tokens: 200,
    stream: true,
  });

  for await (const event of stream) {
    if (event.type === 'response.output_text.delta') {
      return Math.round(performance.now() - startedAt);
    }
  }

  throw new Error('official stream ended before first text delta');
}

async function main() {
  console.log(`Warming up (${MODEL})...`);
  await official.responses.create({ model: MODEL, input: 'ping', max_output_tokens: 16 });
  await oxide.createResponse({ model: MODEL, input: 'ping', max_output_tokens: 16 });
  console.log('Ready.\n');

  console.log(`${'Test'.padEnd(24)} ${'oxide'.padStart(10)} ${'official'.padStart(10)} ${'winner'.padStart(10)}`);
  console.log('-'.repeat(58));

  printRow(
    'Plain text',
    await sample(ITERATIONS, () =>
      oxide.createResponse({
        model: MODEL,
        input: 'What is the capital of France? One word.',
        max_output_tokens: 16,
      })
    ),
    await sample(ITERATIONS, () =>
      official.responses.create({
        model: MODEL,
        input: 'What is the capital of France? One word.',
        max_output_tokens: 16,
      })
    )
  );

  printRow(
    'Structured output',
    await sample(ITERATIONS, () =>
      oxide.createResponse({
        model: MODEL,
        input: 'List 3 programming languages with year created',
        max_output_tokens: 200,
        text: {
          format: {
            type: 'json_schema',
            name: 'langs',
            strict: true,
            schema: languageSchema,
          },
        },
      })
    ),
    await sample(ITERATIONS, () =>
      official.responses.create({
        model: MODEL,
        input: 'List 3 programming languages with year created',
        max_output_tokens: 200,
        text: {
          format: {
            type: 'json_schema',
            name: 'langs',
            strict: true,
            schema: languageSchema,
          },
        },
      })
    )
  );

  printRow(
    'Function calling',
    await sample(ITERATIONS, () =>
      oxide.createResponse({
        model: MODEL,
        input: "What's the weather in Tokyo?",
        tools: weatherTools,
      })
    ),
    await sample(ITERATIONS, () =>
      official.responses.create({
        model: MODEL,
        input: "What's the weather in Tokyo?",
        tools: weatherTools,
      })
    )
  );

  printRow(
    'Multi-turn (2 reqs)',
    await sample(ITERATIONS, async () => {
      const first = await oxide.createResponse({
        model: MODEL,
        input: 'Remember: the answer is 42.',
        store: true,
        max_output_tokens: 32,
      });
      await oxide.createResponse({
        model: MODEL,
        input: 'What is the answer?',
        previous_response_id: first.id,
        max_output_tokens: 16,
      });
    }),
    await sample(ITERATIONS, async () => {
      const first = await official.responses.create({
        model: MODEL,
        input: 'Remember: the answer is 42.',
        store: true,
        max_output_tokens: 32,
      });
      await official.responses.create({
        model: MODEL,
        input: 'What is the answer?',
        previous_response_id: first.id,
        max_output_tokens: 16,
      });
    })
  );

  printRow(
    'Rapid-fire (5 calls)',
    await sample(ITERATIONS, async () => {
      for (let i = 1; i <= 5; i += 1) {
        await oxide.createResponse({
          model: MODEL,
          input: `What is ${i}+${i}? Reply with just the number.`,
          max_output_tokens: 16,
        });
      }
    }),
    await sample(ITERATIONS, async () => {
      for (let i = 1; i <= 5; i += 1) {
        await official.responses.create({
          model: MODEL,
          input: `What is ${i}+${i}? Reply with just the number.`,
          max_output_tokens: 16,
        });
      }
    })
  );

  printRow(
    'Streaming TTFT',
    await sample(ITERATIONS, oxideStreamTtft),
    await sample(ITERATIONS, officialStreamTtft)
  );

  printRow(
    'Parallel 3x',
    await sample(ITERATIONS, () =>
      Promise.all([
        oxide.createResponse({ model: MODEL, input: 'Capital of France? One word.', max_output_tokens: 16 }),
        oxide.createResponse({ model: MODEL, input: 'Capital of Japan? One word.', max_output_tokens: 16 }),
        oxide.createResponse({ model: MODEL, input: 'Capital of Brazil? One word.', max_output_tokens: 16 }),
      ])
    ),
    await sample(ITERATIONS, () =>
      Promise.all([
        official.responses.create({ model: MODEL, input: 'Capital of France? One word.', max_output_tokens: 16 }),
        official.responses.create({ model: MODEL, input: 'Capital of Japan? One word.', max_output_tokens: 16 }),
        official.responses.create({ model: MODEL, input: 'Capital of Brazil? One word.', max_output_tokens: 16 }),
      ])
    )
  );

  printRow(
    'WebSocket hot pair',
    await sample(ITERATIONS, async () => {
      const session = await oxide.wsSession();
      try {
        await session.send(MODEL, 'Say ping');
        await session.send(MODEL, 'Say pong');
      } finally {
        await session.close();
      }
    }),
    'N/A'
  );

  console.log(`\n${ITERATIONS} iterations, median. Model: ${MODEL}`);
  console.log('oxide:    openai-oxide (native napi-rs)');
  console.log('official: openai npm SDK');
}

main().catch((error) => {
  console.error(error);
  process.exitCode = 1;
});
