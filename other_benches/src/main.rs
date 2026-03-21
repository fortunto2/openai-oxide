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
            match res? {
                async_openai::types::responses::ResponseStreamEvent::ResponseOutputTextDelta(_) => {
                    times.push(t0.elapsed().as_millis());
                    break;
                },
                _ => {}
            }
        }
        while let Some(_) = stream.next().await {}
    }
    let (med, _, _, _) = stats(&mut times);
    println!("{:<25} {:>6}ms", "Streaming TTFT", med);
    Ok(())
}
