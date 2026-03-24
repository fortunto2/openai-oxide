/**
 * Drop-in replacement for official openai SDK demo.
 * Change: `const OpenAI = require('openai')` → `const { OpenAI } = require('openai-oxide/compat')`
 */

// ── One-line change from official SDK ──
// const OpenAI = require('openai');
const { OpenAI } = require('../compat');

async function main() {
    const client = new OpenAI();

    // Non-streaming:
    console.log("----- standard request -----");
    const completion = await client.chat.completions.create({
        model: "gpt-5.4-mini",
        messages: [{ role: "user", content: "Say this is a test" }],
    });
    console.log(completion.choices[0].message.content);

    // Streaming:
    console.log("----- streaming request -----");
    const stream = await client.chat.completions.create({
        model: "gpt-5.4-mini",
        messages: [{ role: "user", content: "How do I list files in a directory using Node.js?" }],
        stream: true,
    });
    for await (const chunk of stream) {
        const content = chunk.choices?.[0]?.delta?.content;
        if (content) process.stdout.write(content);
    }
    console.log();
}

main();
