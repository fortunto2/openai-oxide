/**
 * Drop-in replacement for official openai SDK structured output example.
 * Change: `const OpenAI = require('openai')` → `const { OpenAI } = require('openai-oxide/compat')`
 *
 * For Zod support: npm install zod zod-to-json-schema
 */

// ── One-line change from official SDK ──
// const OpenAI = require('openai');
const { OpenAI } = require('../compat');

async function main() {
    const client = new OpenAI();

    // JSON Schema (works without Zod)
    const MathResponseSchema = {
        type: "object",
        properties: {
            steps: {
                type: "array",
                items: {
                    type: "object",
                    properties: {
                        explanation: { type: "string" },
                        output: { type: "string" },
                    },
                    required: ["explanation", "output"],
                    additionalProperties: false,
                },
            },
            final_answer: { type: "string" },
        },
        required: ["steps", "final_answer"],
        additionalProperties: false,
    };

    const result = await client.chat.completions.parse({
        model: "gpt-5.4-mini",
        messages: [
            { role: "system", content: "You are a helpful math tutor." },
            { role: "user", content: "solve 8x + 31 = 2" },
        ],
        response_format: {
            type: "json_schema",
            json_schema: {
                name: "MathResponse",
                schema: MathResponseSchema,
                strict: true,
            },
        },
    });

    const message = result.choices[0].message;
    const parsed = JSON.parse(message.content);
    for (const step of parsed.steps) {
        console.log(`  ${step.explanation} → ${step.output}`);
    }
    console.log("answer:", parsed.final_answer);
}

main();
