#![allow(non_snake_case)]

use dioxus::prelude::*;
use futures_util::{SinkExt, StreamExt};
use gloo_net::websocket::{futures::WebSocket, Message};
use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize, PartialEq, Debug)]
struct ChatMessage {
    role: String,
    content: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct WsMessage {
    action: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    content: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    messages: Option<Vec<ChatMessage>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    model: Option<String>,
}

fn main() {
    dioxus_logger::init(tracing::Level::INFO).expect("failed to init logger");
    launch(App);
}

pub fn App() -> Element {
    let mut messages = use_signal(Vec::<ChatMessage>::new);
    let mut input_text = use_signal(String::new);
    let mut model = use_signal(|| "gpt-5.4-mini".to_string());
    let mut custom_model = use_signal(String::new);
    
    let mut base_url = use_signal(|| {
        let storage = web_sys::window().unwrap().local_storage().unwrap().unwrap();
        storage.get_item("openai_base_url").unwrap().unwrap_or_else(|| "https://api.openai.com/v1".to_string())
    });
    
    let mut api_key = use_signal(|| {
        let storage = web_sys::window().unwrap().local_storage().unwrap().unwrap();
        storage.get_item("openai_api_key").unwrap().unwrap_or_default()
    });
    let mut connected = use_signal(|| false);
    
    // Stats
    let mut ttft = use_signal(|| 0.0);
    let mut speed = use_signal(|| 0.0);
    let mut token_count = use_signal(|| 0);
    let mut stream_start_time = use_signal(|| 0.0);
    let mut last_chunk_time = use_signal(|| 0.0);

    let ws_task = use_coroutine(|mut rx: UnboundedReceiver<String>| async move {
        let host = web_sys::window().unwrap().location().host().unwrap();
        let protocol = web_sys::window().unwrap().location().protocol().unwrap();
        let ws_protocol = if protocol == "https:" { "wss:" } else { "ws:" };
        
        let mut key = String::new();
        let mut url_param = String::new();
        
        while let Some(msg) = rx.next().await {
            if msg.starts_with("CONNECT:") {
                let parts: Vec<&str> = msg.split("|||").collect();
                key = parts[0].replace("CONNECT:", "");
                if parts.len() > 1 {
                    url_param = parts[1].to_string();
                }
                break;
            }
        }
        
        let mut ws_url = format!("{}//{}/api/ws", ws_protocol, host);
        let mut has_query = false;
        
        if !key.is_empty() {
            ws_url.push_str(&format!("?key={}", key));
            has_query = true;
        }
        
        if !url_param.is_empty() {
            let prefix = if has_query { "&" } else { "?" };
            // URL encode the base_url
            let encoded_url = js_sys::encode_uri_component(&url_param).as_string().unwrap_or(url_param);
            ws_url.push_str(&format!("{}base_url={}", prefix, encoded_url));
        }
        
        tracing::info!("Connecting to {}", ws_url);
        
        let ws_conn = match WebSocket::open(&ws_url) {
            Ok(conn) => conn,
            Err(e) => {
                tracing::error!("Failed to open WS: {:?}", e);
                return;
            }
        };

        connected.set(true);
        let (mut write, mut read) = ws_conn.split();

        loop {
            tokio::select! {
                Some(msg_to_send) = rx.next() => {
                    // Expecting a serialized JSON string from the channel
                    if let Err(e) = write.send(Message::Text(msg_to_send)).await {
                        tracing::error!("WS send error: {:?}", e);
                        break;
                    }
                    
                    let now = web_sys::window().unwrap().performance().unwrap().now();
                    stream_start_time.set(now);
                    ttft.set(0.0);
                    speed.set(0.0);
                    token_count.set(0);
                }
                Some(ws_msg) = read.next() => {
                    if let Ok(Message::Text(text)) = ws_msg {
                        if let Ok(incoming) = serde_json::from_str::<WsMessage>(&text) {
                            let now = web_sys::window().unwrap().performance().unwrap().now();
                            
                            if incoming.action == "chunk" {
                                if let Some(chunk) = incoming.content {
                                    if *ttft.read() == 0.0 {
                                        ttft.set(now - *stream_start_time.read());
                                    }
                                    
                                    let current_count = *token_count.read();
                                    token_count.set(current_count + 1); // rough estimate
                                    last_chunk_time.set(now);
                                    
                                    let mut msgs = messages.read().clone();
                                    if let Some(last) = msgs.last_mut() {
                                        if last.role == "assistant" {
                                            last.content.push_str(&chunk);
                                        } else {
                                            msgs.push(ChatMessage { role: "assistant".into(), content: chunk });
                                        }
                                    } else {
                                        msgs.push(ChatMessage { role: "assistant".into(), content: chunk });
                                    }
                                    messages.set(msgs);
                                }
                            } else if incoming.action == "done" {
                                let total_time = (*last_chunk_time.read() - *stream_start_time.read()) / 1000.0;
                                if total_time > 0.0 {
                                    speed.set((*token_count.read() as f64) / total_time);
                                }
                                tracing::info!("Stream done");
                            }
                        }
                    }
                }
                else => break,
            }
        }
        
        connected.set(false);
        tracing::info!("WS Disconnected");
    });

    let mut send_message = move || {
        let text = input_text.read().clone();
        if text.is_empty() { return; }

        let mut current_msgs = messages.read().clone();
        current_msgs.push(ChatMessage { role: "user".into(), content: text.clone() });
        messages.set(current_msgs.clone());
        
        let selected_model = model.read().clone();
        let actual_model = if selected_model == "custom" {
            custom_model.read().clone()
        } else {
            selected_model
        };
        
        let payload = WsMessage {
            action: "send".into(),
            content: None,
            messages: Some(current_msgs),
            model: Some(actual_model),
        };
        if let Ok(json) = serde_json::to_string(&payload) {
            ws_task.send(json);
        }
        
        input_text.set(String::new());
    };

    let connect_ws = move || {
        let key = api_key.read().clone();
        let url = base_url.read().clone();
        if let Ok(Some(storage)) = web_sys::window().unwrap().local_storage() {
            let _ = storage.set_item("openai_api_key", &key);
            let _ = storage.set_item("openai_base_url", &url);
        }
        ws_task.send(format!("CONNECT:{}|||{}", key, url));
    };

    let status_color = if connected() { "green" } else { "red" };
    let status_text = if connected() { "Status: Connected" } else { "Status: Disconnected" };

    rsx! {
        div {
            style: "max-width: 800px; margin: 0 auto; padding: 20px; font-family: sans-serif;",
            h1 { "OpenAI Oxide + Rust WASM + Durable Objects" }
            
            div {
                style: "margin-bottom: 20px; padding: 10px; background-color: #f8f9fa; border-radius: 5px; display: flex; flex-direction: column; gap: 10px;",
                div {
                    style: "display: flex; justify-content: space-between; align-items: center;",
                    div {
                        label {
                            style: "margin-right: 10px;",
                            "API Key: "
                        }
                        input {
                            "type": "password",
                            value: "{api_key}",
                            oninput: move |e| api_key.set(e.value()),
                            placeholder: "sk-...",
                            style: "margin-right: 10px; padding: 5px; width: 200px;"
                        }
                        label {
                            style: "margin-right: 10px;",
                            "Base URL: "
                        }
                        input {
                            "type": "text",
                            value: "{base_url}",
                            oninput: move |e| base_url.set(e.value()),
                            placeholder: "https://api.openai.com/v1",
                            style: "margin-right: 10px; padding: 5px; width: 250px;"
                        }
                        button {
                            onclick: move |_| connect_ws(),
                            disabled: connected(),
                            style: "padding: 5px 15px; cursor: pointer;",
                            "Connect"
                        }
                    }
                    div {
                        style: "color: {status_color}; font-weight: bold;",
                        "{status_text}"
                    }
                }
            }

            div {
                style: "margin-bottom: 20px; padding: 10px; background-color: #e9ecef; border-radius: 5px; display: flex; justify-content: space-between; align-items: center;",
                div {
                    style: "font-family: monospace; font-size: 14px;",
                    span { style: "margin-right: 20px;", "TTFT: {ttft():.0}ms" }
                    span { "Speed: {speed():.1} tokens/sec" }
                }
                div {
                    style: "display: flex; gap: 10px; align-items: center;",
                    if *model.read() == "custom" {
                        input {
                            "type": "text",
                            value: "{custom_model}",
                            oninput: move |e| custom_model.set(e.value()),
                            placeholder: "Enter custom model name...",
                            style: "padding: 5px; border-radius: 5px; width: 200px;"
                        }
                    }
                    select {
                        value: "{model}",
                        onchange: move |e| model.set(e.value()),
                        style: "padding: 5px; border-radius: 5px;",
                        option { value: "gpt-5.4-mini", "gpt-5.4-mini" }
                        option { value: "gpt-5", "gpt-5" }
                        option { value: "gpt-4.5-preview", "gpt-4.5-preview" }
                        option { value: "gpt-4o", "gpt-4o" }
                        option { value: "gpt-4o-mini", "gpt-4o-mini" }
                        option { value: "custom", "Custom..." }
                    }
                }
            }
            
            div {
                style: "height: 400px; overflow-y: auto; border: 1px solid #ccc; padding: 10px; margin-bottom: 20px; white-space: pre-wrap;",
                for msg in messages() {
                    div {
                        style: "margin-bottom: 10px; padding: 10px; border-radius: 5px;",
                        background_color: if msg.role == "user" { "#e3f2fd" } else { "#f5f5f5" },
                        strong { "{msg.role}: " }
                        "{msg.content}"
                    }
                }
            }
            
            div {
                style: "display: flex; gap: 10px;",
                input {
                    style: "flex: 1; padding: 10px; font-size: 16px;",
                    value: "{input_text}",
                    oninput: move |e| input_text.set(e.value()),
                    onkeypress: move |e| {
                        if e.key() == dioxus::events::Key::Enter {
                            send_message();
                        }
                    },
                    placeholder: "Type your message...",
                    disabled: !connected(),
                }
                button {
                    style: "padding: 10px 20px; font-size: 16px; background-color: #007bff; color: white; border: none; border-radius: 5px; cursor: pointer;",
                    onclick: move |_| send_message(),
                    disabled: !connected(),
                    "Send"
                }
            }
        }
    }
}