const { Client } = require('./index.js');

async function main() {
    console.log("Testing Node.js bindings powered by openai-oxide (Rust)...");
    const client = new Client();
    
    console.time("Request");
    const response = await client.createResponse("gpt-5.4", "Say hello to Node.js from Rust via NAPI!");
    console.timeEnd("Request");
    
    console.log("Response:", response);
}

main();
