use actix_web::{web, HttpRequest, HttpResponse};
use serde::Deserialize;
use crate::db::DbPool;
use crate::middleware::{AuthState, extract_claims, unauthorized};
use crate::ws::WsBroadcaster;
use std::sync::Arc;

#[derive(Deserialize)]
pub struct SendMessageBody {
    #[serde(rename = "messageValue")]
    pub message_value: String,
}

pub async fn get_latest(
    req: HttpRequest,
    db: web::Data<DbPool>,
    auth: web::Data<AuthState>,
) -> HttpResponse {
    // Auth is optional for reading messages but the original requires it
    let _claims = match extract_claims(&req, &auth) {
        Some(c) => c,
        None => return unauthorized(),
    };

    let db = db.lock().unwrap();
    let mut stmt = db.prepare(
        "SELECT id, user_id, username, message_value, date FROM chat_messages ORDER BY date DESC LIMIT 100"
    ).unwrap();

    let mut messages: Vec<serde_json::Value> = stmt.query_map([], |row| {
        Ok(serde_json::json!({
            "_id": row.get::<_, String>(0)?,
            "userId": row.get::<_, String>(1)?,
            "username": row.get::<_, String>(2)?,
            "messageValue": row.get::<_, String>(3)?,
            "date": row.get::<_, String>(4)?,
        }))
    }).unwrap().filter_map(|r| r.ok()).collect();

    messages.reverse(); // Return oldest first
    HttpResponse::Ok().json(messages)
}

pub async fn send_message(
    req: HttpRequest,
    body: web::Json<SendMessageBody>,
    db: web::Data<DbPool>,
    auth: web::Data<AuthState>,
    broadcaster: web::Data<Arc<WsBroadcaster>>,
) -> HttpResponse {
    let claims = match extract_claims(&req, &auth) {
        Some(c) => c,
        None => return unauthorized(),
    };

    if body.message_value.is_empty() {
        return HttpResponse::BadRequest().json("Message cannot be empty");
    }

    let id = uuid::Uuid::new_v4().to_string();
    let now = chrono::Utc::now().to_rfc3339();

    let db = db.lock().unwrap();
    let _ = db.execute(
        "INSERT INTO chat_messages (id, user_id, username, message_value, date) VALUES (?1, ?2, ?3, ?4, ?5)",
        rusqlite::params![id, claims.sub, claims.name, body.message_value, now],
    );

    // Broadcast via WebSocket
    let msg = serde_json::json!({
        "type": "chatMessage",
        "data": {
            "_id": id,
            "userId": claims.sub,
            "username": claims.name,
            "messageValue": body.message_value,
            "date": now,
        }
    });
    broadcaster.broadcast(&msg.to_string());

    HttpResponse::Ok().json("Message sent")
}
