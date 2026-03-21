use futures_util::StreamExt;
use openai_oxide::OpenAI;
use openai_oxide::types::chat::*;
use std::time::Instant;

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
    let client = OpenAI::from_env()?;

    // Warmup
    client
        .chat()
        .completions()
        .create(
            ChatCompletionRequest::new(
                MODEL,
                vec![ChatCompletionMessageParam::System {
                    content: "ping".into(),
                    name: None,
                }],
            )
            .max_completion_tokens(16),
        )
        .await?;

    println!("=== openai-oxide CHAT API ===");

    // 1. Plain text
    let mut times = Vec::new();
    for _ in 0..ITERATIONS {
        let req = ChatCompletionRequest::new(
            MODEL,
            vec![ChatCompletionMessageParam::User {
                content: UserContent::Text("What is the capital of France? One word.".into()),
                name: None,
            }],
        )
        .max_completion_tokens(16);

        let t0 = Instant::now();
        let _ = client.chat().completions().create(req).await?;
        times.push(t0.elapsed().as_millis());
    }
    let (med, _, _, _) = stats(&mut times);
    println!("{:<25} {:>6}ms", "Plain text", med);

    // 10. Streaming TTFT
    let mut times = Vec::new();
    for _ in 0..ITERATIONS {
        let req = ChatCompletionRequest::new(
            MODEL,
            vec![ChatCompletionMessageParam::User {
                content: UserContent::Text("Explain quicksort in 3 sentences.".into()),
                name: None,
            }],
        );
        let t0 = Instant::now();
        let mut stream = client.chat().completions().create_stream(req).await?;
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
