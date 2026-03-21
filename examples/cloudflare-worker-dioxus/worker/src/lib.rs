use futures_util::StreamExt;
use serde::{Deserialize, Serialize};
use worker::*;
use openai_oxide::types::common::Role;
use openai_oxide::types::responses::{ResponseCreateRequest, ResponseStreamEvent, ResponseInputItem, ResponseInput};

#[derive(Clone, Serialize, Deserialize, Debug)]
struct ChatMessage {
    role: String,
    content: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct WsMessage {
    action: String,
    content: Option<String>,
    messages: Option<Vec<ChatMessage>>,
    model: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    base_url: Option<String>,
}

#[event(start)]
pub fn start() {
    console_error_panic_hook::set_once();
}

#[event(fetch)]
async fn main(req: Request, env: Env, _ctx: Context) -> Result<Response> {
    let router = Router::new();

    router
        .get_async("/api/ws", |req, ctx| async move {
            let namespace = ctx.env.durable_object("CHAT_DO")?;
            let id = namespace.id_from_name("global")?;
            let stub = id.get_stub()?;
            stub.fetch_with_request(req).await
        })
        .get_async("/*path", |_req, _ctx| async move {
            Response::ok("Not Found")
        })
        .run(req, env)
        .await
}

#[durable_object]
pub struct ChatDurableObject {
    state: State,
    env: Env,
}

impl DurableObject for ChatDurableObject {
    fn new(state: State, env: Env) -> Self {
        Self { state, env }
    }

    async fn fetch(&self, req: Request) -> Result<Response> {
        self.handle(req).await
    }
}

impl ChatDurableObject {
    async fn handle(&self, req: Request) -> Result<Response> {
        if req.headers().get("Upgrade")?.unwrap_or_default().to_lowercase() == "websocket" {
            let pair = WebSocketPair::new()?;
            let client = pair.client;
            let browser_ws = pair.server;
            browser_ws.accept()?;

            let url = req.url()?;
            let api_key = url.query_pairs().find(|(k, _)| k == "key").map(|(_, v)| v.into_owned())
                .filter(|k| !k.is_empty())
                .or_else(|| self.env.var("OPENAI_API_KEY").ok().map(|v| v.to_string()))
                .filter(|k| !k.is_empty());

            let mut base_url = url.query_pairs().find(|(k, _)| k == "base_url").map(|(_, v)| v.into_owned())
                .filter(|k| !k.is_empty())
                .unwrap_or_else(|| "https://api.openai.com/v1".to_string());
                
            // OpenAI Responses API is only available on WSS. If the custom base URL is HTTP/S, we need to adapt it,
            // or just use it if it's WSS. But `fetch` works with https:// for upgrading.
            // But we actually use `fetch` with https://, so base_url should be https://
            if !base_url.starts_with("http") {
                base_url = format!("https://{}", base_url);
            }
            // remove trailing slash
            let base_url = base_url.trim_end_matches('/');

            let api_key = match api_key {
                Some(k) => k,
                None => {
                    console_log!("OPENAI_API_KEY missing in query params and env");
                    browser_ws.close(Some(1011), Some("Missing API Key"))?;
                    return Response::from_websocket(client);
                }
            };

            let is_openai = base_url.contains("api.openai.com");

            // Connect to OpenAI Responses API via WebSocket
            let mut headers = Headers::new();
            headers.set("Upgrade", "websocket")?;
            headers.set("Authorization", &format!("Bearer {}", api_key))?;
            
            let mut init = RequestInit::new();
            init.with_method(Method::Get);
            init.with_headers(headers);

            // OpenRouter doesn't support the /responses endpoint WSS yet.
            // If it's not OpenAI, we cannot establish a native WSS session. We'd have to use standard HTTP /chat/completions.
            // BUT, since this is a pure WebSocket example built on `Responses API`, we'll try to use the WSS endpoint.
            // If the user provided OpenRouter, they likely mean standard HTTP streaming.
            // But Cloudflare worker websocket() requires a 101 Switching Protocols.
            
            let req_url = if is_openai {
                format!("{}/responses", base_url)
            } else {
                // For non-OpenAI, we assume they support WSS on the base URL, or just fail for now.
                // We will attempt to connect to WSS. If it fails, we gracefully close.
                // We'll append /chat/completions just in case some support WS there.
                format!("{}/chat/completions", base_url)
            };

            let openai_req = Request::new_with_init(&req_url, &init)?;
            let openai_res = match Fetch::Request(openai_req).send().await {
                Ok(res) => res,
                Err(e) => {
                    console_log!("Failed to fetch Upstream: {:?}", e);
                    browser_ws.close(Some(1011), Some("Failed to connect to Upstream"))?;
                    return Response::from_websocket(client);
                }
            };

            let openai_status = openai_res.status_code();
            let openai_ws = match openai_res.websocket() {
                Some(ws) => ws,
                None => {
                    console_log!("Upstream did not return a WebSocket. Status: {}. URL: {}", openai_status, req_url);
                    // OpenRouter/etc don't support WSS for this natively in the same way.
                    let error_msg = format!("Upstream does not support WebSocket Responses API (Status: {})", openai_status);
                    browser_ws.close(Some(1011), Some(&error_msg))?;
                    return Response::from_websocket(client);
                }
            };
            openai_ws.accept()?;

            wasm_bindgen_futures::spawn_local(async move {
                let mut browser_events = browser_ws.events().expect("Failed to get browser events");
                let mut openai_events = openai_ws.events().expect("Failed to get OpenAI events");

                loop {
                    tokio::select! {
                        Some(browser_event) = browser_events.next() => {
                            if let Ok(worker::WebsocketEvent::Message(msg)) = browser_event {
                                if let Some(text) = msg.text() {
                                    if let Ok(ws_msg) = serde_json::from_str::<WsMessage>(&text) {
                                        if ws_msg.action == "send" {
                                            if let Some(msgs) = ws_msg.messages {
                                                let model = ws_msg.model.unwrap_or_else(|| "gpt-4o-mini".to_string());
                                                
                                                let input_items = msgs.into_iter().map(|m| {
                                                    let role = match m.role.as_str() {
                                                        "user" => Role::User,
                                                        "assistant" => Role::Assistant,
                                                        "system" => Role::System,
                                                        _ => Role::User,
                                                    };
                                                    ResponseInputItem {
                                                        role,
                                                        content: serde_json::Value::String(m.content),
                                                    }
                                                }).collect::<Vec<_>>();
                                                
                                                // We use prompt caching feature!
                                                let mut o_req = ResponseCreateRequest::new(model)
                                                    .input(ResponseInput::Messages(input_items));
                                                    
                                                // Only add prompt caching for native OpenAI as others might reject it
                                                if is_openai {
                                                    o_req = o_req.prompt_cache_key("oxide-dioxus-chat")
                                                                 .prompt_cache_retention("24h");
                                                }
                                                
                                                let mut value = serde_json::to_value(&o_req).unwrap();
                                                if let serde_json::Value::Object(ref mut map) = value {
                                                    map.insert("type".to_string(), serde_json::Value::String("response.create".to_string()));
                                                }
                                                let request_text = serde_json::to_string(&value).unwrap();
                                                
                                                let _ = openai_ws.send_with_str(request_text);
                                            }
                                        }
                                    }
                                }
                            } else if let Ok(worker::WebsocketEvent::Close(_)) = browser_event {
                                let _ = openai_ws.close(None, Some(""));
                                break;
                            }
                        }
                        Some(openai_event) = openai_events.next() => {
                            if let Ok(worker::WebsocketEvent::Message(msg)) = openai_event {
                                if let Some(text) = msg.text() {
                                    if let Ok(event) = serde_json::from_str::<ResponseStreamEvent>(&text) {
                                        if event.type_ == "response.output_text.delta" {
                                            if let Some(delta) = event.data["delta"].as_str() {
                                                let reply = WsMessage {
                                                    action: "chunk".into(),
                                                    content: Some(delta.to_string()),
                                                    messages: None,
                                                    model: None,
                                                    base_url: None,
                                                };
                                                if let Ok(json) = serde_json::to_string(&reply) {
                                                    let _ = browser_ws.send_with_str(json);
                                                }
                                            }
                                        } else if event.type_ == "response.completed" {
                                            let done_msg = WsMessage {
                                                action: "done".into(),
                                                content: None,
                                                messages: None,
                                                model: None,
                                                base_url: None,
                                            };
                                            if let Ok(json) = serde_json::to_string(&done_msg) {
                                                let _ = browser_ws.send_with_str(json);
                                            }
                                        }
                                    }
                                }
                            } else if let Ok(worker::WebsocketEvent::Close(_)) = openai_event {
                                let _ = browser_ws.close(None, Some(""));
                                break;
                            }
                        }
                        else => break,
                    }
                }
            });

            return Response::from_websocket(client);
        }

        Response::error("Expected WebSocket", 400)
    }
}
