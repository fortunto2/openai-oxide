use std::time::Instant;
use futures_util::StreamExt;

const MODEL: &str = "gpt-5.4";
const ITERATIONS: usize = 5;

fn stats(times: &mut Vec<u128>) -> (u128, u128, u128, u128) {
    times.sort();
    let min = times[0];
    let max = *times.last().unwrap();
    let median = times[times.len() / 2];
    let p95 = times[((times.len() as f64 * 0.95) as usize).min(times.len() - 1)];
    (median, p95, min, max)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Testing async-openai 0.33.1 ===");
    let client = async_openai::Client::new();
    
    // Warmup
    let req = async_openai::types::CreateChatCompletionRequestArgs::default()
        .model(MODEL)
        .messages([async_openai::types::ChatCompletionRequestSystemMessageArgs::default()
            .content("ping")
            .build()?
            .into()])
        .build()?;
    client.chat().create(req).await?;
    
    // 1. Plain text
    let mut times = Vec::new();
    for _ in 0..ITERATIONS {
        let req = async_openai::types::CreateChatCompletionRequestArgs::default()
            .model(MODEL)
            .messages([async_openai::types::ChatCompletionRequestUserMessageArgs::default()
                .content("What is the capital of France? One word.")
                .build()?
                .into()])
            .build()?;
        
        let t0 = Instant::now();
        let _ = client.chat().create(req).await?;
        times.push(t0.elapsed().as_millis());
    }
    let (med, _, _, _) = stats(&mut times);
    println!("{:<25} {:>6}ms", "Plain text", med);

    // 2. Structured output
    let mut times = Vec::new();
    let schema = serde_json::json!({
        "type": "object",
        "properties": {"languages": {"type": "array", "items": {"type": "object", "properties": {"name": {"type": "string"}, "year": {"type": "integer"}}, "required": ["name", "year"], "additionalProperties": false}}},
        "required": ["languages"], "additionalProperties": false
    });
    for _ in 0..ITERATIONS {
        let req = async_openai::types::CreateChatCompletionRequestArgs::default()
            .model(MODEL)
            .messages([async_openai::types::ChatCompletionRequestUserMessageArgs::default()
                .content("List 3 programming languages with year created")
                .build()?
                .into()])
            .response_format(async_openai::types::ResponseFormat::JsonSchema {
                json_schema: async_openai::types::ResponseFormatJsonSchema {
                    name: "languages".into(),
                    description: None,
                    schema: Some(schema.clone()),
                    strict: Some(true)
                }
            })
            .build()?;
        
        let t0 = Instant::now();
        let _ = client.chat().create(req).await?;
        times.push(t0.elapsed().as_millis());
    }
    let (med, _, _, _) = stats(&mut times);
    println!("{:<25} {:>6}ms", "Structured output", med);

    // 3. Function calling
    let mut times = Vec::new();
    for _ in 0..ITERATIONS {
        let req = async_openai::types::CreateChatCompletionRequestArgs::default()
            .model(MODEL)
            .messages([async_openai::types::ChatCompletionRequestUserMessageArgs::default()
                .content("What's the weather in Tokyo?")
                .build()?
                .into()])
            .tools(vec![async_openai::types::ChatCompletionToolArgs::default()
                .r#type(async_openai::types::ChatCompletionToolType::Function)
                .function(async_openai::types::FunctionObjectArgs::default()
                    .name("get_weather")
                    .description("Get weather")
                    .parameters(serde_json::json!({"type": "object", "properties": {"city": {"type": "string"}, "unit": {"type": "string", "enum": ["celsius", "fahrenheit"]}}, "required": ["city", "unit"], "additionalProperties": false}))
                    .strict(true)
                    .build()?)
                .build()?])
            .build()?;
        
        let t0 = Instant::now();
        let _ = client.chat().create(req).await?;
        times.push(t0.elapsed().as_millis());
    }
    let (med, _, _, _) = stats(&mut times);
    println!("{:<25} {:>6}ms", "Function calling", med);

    // 4. Multi-turn
    let mut times = Vec::new();
    for _ in 0..ITERATIONS {
        let t0 = Instant::now();
        let req1 = async_openai::types::CreateChatCompletionRequestArgs::default()
            .model(MODEL)
            .messages([async_openai::types::ChatCompletionRequestUserMessageArgs::default()
                .content("Remember: the answer is 42.")
                .build()?
                .into()])
            .build()?;
        let res1 = client.chat().create(req1).await?;
        let ast = res1.choices[0].message.content.clone().unwrap_or_default();
        
        let req2 = async_openai::types::CreateChatCompletionRequestArgs::default()
            .model(MODEL)
            .messages([
                async_openai::types::ChatCompletionRequestUserMessageArgs::default().content("Remember: the answer is 42.").build()?.into(),
                async_openai::types::ChatCompletionRequestAssistantMessageArgs::default().content(ast).build()?.into(),
                async_openai::types::ChatCompletionRequestUserMessageArgs::default().content("What is the answer?").build()?.into()
            ])
            .build()?;
        let _ = client.chat().create(req2).await?;
        times.push(t0.elapsed().as_millis());
    }
    let (med, _, _, _) = stats(&mut times);
    println!("{:<25} {:>6}ms", "Multi-turn (2 reqs)", med);

    // 10. Streaming TTFT
    let mut times = Vec::new();
    for _ in 0..ITERATIONS {
        let req = async_openai::types::CreateChatCompletionRequestArgs::default()
            .model(MODEL)
            .messages([async_openai::types::ChatCompletionRequestUserMessageArgs::default()
                .content("Explain quicksort in 3 sentences.")
                .build()?
                .into()])
            .build()?;
        let t0 = Instant::now();
        let mut stream = client.chat().create_stream(req).await?;
        while let Some(res) = stream.next().await {
            let _ = res?;
            times.push(t0.elapsed().as_millis());
            break;
        }
        while let Some(_) = stream.next().await {}
    }
    let (med, _, _, _) = stats(&mut times);
    println!("{:<25} {:>6}ms", "Streaming TTFT", med);

    println!("\n=== Testing genai 0.5.3 ===");
    let client = genai::Client::default();
    
    // Warmup
    client.exec_chat(MODEL, genai::chat::ChatRequest::new(vec![genai::chat::ChatMessage::system("ping")]), None).await?;
    
    // 1. Plain text
    let mut times = Vec::new();
    for _ in 0..ITERATIONS {
        let t0 = Instant::now();
        let _ = client.exec_chat(
            MODEL, 
            genai::chat::ChatRequest::new(vec![genai::chat::ChatMessage::user("What is the capital of France? One word.")]), 
            None
        ).await?;
        times.push(t0.elapsed().as_millis());
    }
    let (med, _, _, _) = stats(&mut times);
    println!("{:<25} {:>6}ms", "Plain text", med);

    Ok(())
}
