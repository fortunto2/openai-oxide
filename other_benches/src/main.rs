use std::time::Instant;
use futures_util::StreamExt;

const MODEL: &str = "gpt-5.4";
const ITERATIONS: usize = 5;

fn stats(times: &mut Vec<u128>) -> (u128, u128, u128, u128) {
    if times.is_empty() {
        return (0, 0, 0, 0);
    }
    times.sort();
    let min = times[0];
    let max = *times.last().unwrap();
    let median = times[times.len() / 2];
    let p95 = times[((times.len() as f64 * 0.95) as usize).min(times.len() - 1)];
    (median, p95, min, max)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Testing async-openai 0.33.1 (Responses API) ===");
    let client = async_openai::Client::new();
    
    // Warmup
    let req = async_openai::types::responses::CreateResponseArgs::default()
        .model(MODEL)
        .input("ping")
        .build()?;
    client.responses().create(req).await?;
    
    // 1. Plain text
    let mut times = Vec::new();
    for _ in 0..ITERATIONS {
        let req = async_openai::types::responses::CreateResponseArgs::default()
            .model(MODEL)
            .input("What is the capital of France? One word.")
            .max_output_tokens(16_u32)
            .build()?;
        
        let t0 = Instant::now();
        let _ = client.responses().create(req).await?;
        times.push(t0.elapsed().as_millis());
    }
    let (med, _, _, _) = stats(&mut times);
    println!("{:<25} {:>6}ms", "Plain text", med);

    // 2. Structured output
    println!("{:<25} {:>6}", "Structured output", "N/A");

    // 3. Function calling
    let mut times = Vec::new();
    for _ in 0..ITERATIONS {
        let req = async_openai::types::responses::CreateResponseArgs::default()
            .model(MODEL)
            .input("What's the weather in Tokyo?")
            .tools(vec![async_openai::types::responses::Tool::Function(async_openai::types::responses::FunctionTool {
                name: "get_weather".into(),
                description: Some("Get weather".into()),
                parameters: Some(serde_json::json!({"type": "object", "properties": {"city": {"type": "string"}, "unit": {"type": "string", "enum": ["celsius", "fahrenheit"]}}, "required": ["city", "unit"], "additionalProperties": false})),
                strict: Some(true),
            })])
            .build()?;
        
        let t0 = Instant::now();
        let _ = client.responses().create(req).await?;
        times.push(t0.elapsed().as_millis());
    }

    let (med, _, _, _) = stats(&mut times);
    println!("{:<25} {:>6}ms", "Function calling", med);

    // 10. Streaming TTFT
    let mut times = Vec::new();
    for _ in 0..ITERATIONS {
        let req = async_openai::types::responses::CreateResponseArgs::default()
            .model(MODEL)
            .input("Explain quicksort in 3 sentences.")
            .max_output_tokens(200_u32)
            .build()?;
        
        let t0 = Instant::now();
        let mut stream = client.responses().create_stream(req).await?;
        while let Some(res) = stream.next().await {
            let _ = res?;
            times.push(t0.elapsed().as_millis());
            break;
        }
        while let Some(_) = stream.next().await {}
    }
    let (med, _, _, _) = stats(&mut times);
    println!("{:<25} {:>6}ms", "Streaming TTFT", med);
    Ok(())
}
