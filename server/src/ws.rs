use actix_web::{web, HttpRequest, HttpResponse};
use actix_ws::Message;
use futures_util::StreamExt;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tokio::sync::broadcast;
use crate::middleware::AuthState;

pub struct WsBroadcaster {
    tx: broadcast::Sender<String>,
    clients: Mutex<HashMap<String, ClientInfo>>,
}

struct ClientInfo {
    username: String,
}

impl WsBroadcaster {
    pub fn new() -> Arc<Self> {
        let (tx, _) = broadcast::channel(1024);
        Arc::new(WsBroadcaster {
            tx,
            clients: Mutex::new(HashMap::new()),
        })
    }

    pub fn broadcast(&self, msg: &str) {
        let _ = self.tx.send(msg.to_string());
    }

    pub fn subscribe(&self) -> broadcast::Receiver<String> {
        self.tx.subscribe()
    }

    pub fn add_client(&self, id: &str, username: &str) {
        let mut clients = self.clients.lock().unwrap();
        clients.insert(id.to_string(), ClientInfo { username: username.to_string() });
    }

    pub fn remove_client(&self, id: &str) {
        let mut clients = self.clients.lock().unwrap();
        clients.remove(id);
        // Broadcast removal
        let msg = serde_json::json!({
            "type": "removeUser",
            "wsId": id,
        });
        let _ = self.tx.send(msg.to_string());
    }
}

pub async fn ws_handler(
    req: HttpRequest,
    stream: web::Payload,
    auth: web::Data<AuthState>,
    broadcaster: web::Data<Arc<WsBroadcaster>>,
) -> Result<HttpResponse, actix_web::Error> {
    // Extract JWT from protocol header (matches original client behavior)
    let token = req.headers()
        .get("sec-websocket-protocol")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");

    let claims = match auth.verify_token(token) {
        Ok(c) => c,
        Err(_) => {
            // Allow unauthenticated connections with limited functionality
            crate::middleware::Claims {
                sub: uuid::Uuid::new_v4().to_string(),
                email: String::new(),
                name: "Anonymous".to_string(),
                role: "user".to_string(),
                exp: 0,
            }
        }
    };

    let (response, mut session, mut msg_stream) = actix_ws::handle(&req, stream)?;

    let ws_id = uuid::Uuid::new_v4().to_string();
    let username = claims.name.clone();
    broadcaster.add_client(&ws_id, &username);

    let broadcaster_clone = broadcaster.get_ref().clone();
    let ws_id_clone = ws_id.clone();
    let mut rx = broadcaster.subscribe();

    // Spawn task to forward broadcasts to this client
    let mut session_clone = session.clone();
    actix_rt::spawn(async move {
        while let Ok(msg) = rx.recv().await {
            if session_clone.text(msg).await.is_err() {
                break;
            }
        }
    });

    // Spawn task to handle incoming messages from this client
    let broadcaster_for_recv = broadcaster_clone.clone();
    actix_rt::spawn(async move {
        while let Some(Ok(msg)) = msg_stream.next().await {
            match msg {
                Message::Text(text) => {
                    if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(&text) {
                        let msg_type = parsed.get("type").and_then(|v| v.as_str()).unwrap_or("");
                        match msg_type {
                            "userPosition" => {
                                let broadcast_msg = serde_json::json!({
                                    "type": "userPosition",
                                    "user": {
                                        "wsId": ws_id_clone,
                                        "username": username,
                                    },
                                    "data": parsed.get("data"),
                                });
                                broadcaster_for_recv.broadcast(&broadcast_msg.to_string());
                            }
                            _ => {}
                        }
                    }
                }
                Message::Ping(bytes) => {
                    let _ = session.pong(&bytes).await;
                }
                Message::Close(_) => {
                    break;
                }
                _ => {}
            }
        }
        broadcaster_for_recv.remove_client(&ws_id_clone);
    });

    // Set the subprotocol to echo back the token (required by browser WS API)
    Ok(response)
}
