/// WebSocket stress test — 10 sequential requests on one persistent session.
///
/// Verifies connection stability, latency consistency, and session reuse.
///
/// Run: OPENAI_API_KEY=sk-... cargo run --example ws_stress --features websocket
use openai_oxide::{OpenAI, types::responses::*};
use std::time::Instant;

#[tokio::main]
async fn main() {
    let client = OpenAI::from_env().expect("OPENAI_API_KEY");
    let n = 10;

    println!("=== WebSocket stress test: {n} requests on one session ===\n");
    let t0 = Instant::now();
    let mut session = client.ws_session().await.expect("ws connect");
    println!("Connected in {}ms", t0.elapsed().as_millis());

    let mut times = Vec::new();
    let mut reconnects = 0;
    for i in 1..=n {
        let t = Instant::now();
        let req = ResponseCreateRequest::new("gpt-4o-mini")
            .input(format!("Reply with just the number {i}"))
            .max_output_tokens(50);
        match session.send(req).await {
            Ok(resp) => {
                let ms = t.elapsed().as_millis();
                times.push(ms);
                println!("  #{i}: {} ({ms}ms)", resp.output_text().trim());
            }
            Err(e) => {
                eprintln!("  #{i}: ERROR: {e}");
                println!("  Reconnecting...");
                session = client.ws_session().await.expect("reconnect");
                reconnects += 1;
            }
        }
    }

    session.close().await.ok();

    let total = t0.elapsed().as_millis();
    if !times.is_empty() {
        let avg = times.iter().sum::<u128>() / times.len() as u128;
        let min = times.iter().min().unwrap();
        let max = times.iter().max().unwrap();
        println!("\n=== Results ===");
        println!("  OK: {}/{n} | Reconnects: {reconnects}", times.len());
        println!("  Total: {total}ms");
        println!("  Avg: {avg}ms | Min: {min}ms | Max: {max}ms");
    }
}
