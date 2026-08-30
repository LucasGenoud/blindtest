use actix_web::{web, HttpRequest, HttpResponse};
use serde::Deserialize;
use crate::db::{lock_db, DbPool};
use crate::middleware::{AuthState, extract_claims, unauthorized};
use crate::ws::WsBroadcaster;
use std::sync::{Arc, Mutex};

/// The canvas body is a million pixels: roughly half a second of SQLite scan plus
/// ~9 MB of JSON. Rebuilding that on every page load stalled every other request,
/// because the work ran on an actix worker thread while holding the connection
/// mutex. It is now built at most once per pixel change, off the async threads.
pub struct CanvasCache {
    encoded: Mutex<Option<web::Bytes>>,
    /// Held while the body is being rebuilt, so a burst of misses costs one scan
    /// rather than one scan per request all queueing on the connection.
    rebuild: tokio::sync::Mutex<()>,
}

impl CanvasCache {
    pub fn new() -> Arc<Self> {
        Arc::new(CanvasCache {
            encoded: Mutex::new(None),
            rebuild: tokio::sync::Mutex::new(()),
        })
    }

    fn get(&self) -> Option<web::Bytes> {
        self.encoded.lock().unwrap_or_else(|p| p.into_inner()).clone()
    }

    fn store(&self, body: web::Bytes) {
        *self.encoded.lock().unwrap_or_else(|p| p.into_inner()) = Some(body);
    }

    fn invalidate(&self) {
        *self.encoded.lock().unwrap_or_else(|p| p.into_inner()) = None;
    }
}

#[derive(Deserialize)]
pub struct PixelQuery {
    pub pixel: Option<String>,
}

#[derive(Deserialize)]
pub struct UpdatePixelBody {
    pub pixel: PixelPayload,
}

#[derive(Deserialize)]
pub struct PixelPayload {
    #[serde(rename = "selectedPixel")]
    pub selected_pixel: PixelCoord,
    #[serde(rename = "selectedColor")]
    pub selected_color: PixelColor,
}

#[derive(Deserialize, Clone, serde::Serialize)]
pub struct PixelCoord {
    pub x: i64,
    pub y: i64,
}

#[derive(Deserialize, Clone, serde::Serialize)]
pub struct PixelColor {
    pub r: Option<u8>,
    pub g: Option<u8>,
    pub b: Option<u8>,
    pub hex: Option<String>,
    pub index: Option<i64>,
    pub name: Option<String>,
}

fn encode_canvas(
    conn: &rusqlite::Connection,
) -> Result<Vec<u8>, Box<dyn std::error::Error + Send + Sync>> {
    let mut stmt = conn.prepare("SELECT color FROM canvas_pixels ORDER BY y, x")?;
    let pixels: Vec<String> = stmt
        .query_map([], |row| row.get::<_, String>(0))?
        .filter_map(|r| r.ok())
        .collect();
    Ok(serde_json::to_vec(&pixels)?)
}

pub async fn get_canvas(
    db: web::Data<DbPool>,
    cache: web::Data<Arc<CanvasCache>>,
) -> HttpResponse {
    if let Some(body) = cache.get() {
        return HttpResponse::Ok().content_type("application/json").body(body);
    }

    // Only one request rebuilds; the rest wait here and take the result.
    let _rebuilding = cache.rebuild.lock().await;
    if let Some(body) = cache.get() {
        return HttpResponse::Ok().content_type("application/json").body(body);
    }

    let pool = db.get_ref().clone();
    let built = web::block(move || -> Result<Vec<u8>, Box<dyn std::error::Error + Send + Sync>> {
        // Prefer a private read-only connection so the scan does not hold the shared
        // one; fall back to it only if the side connection cannot be opened.
        match crate::db::open_read_only() {
            Ok(conn) => encode_canvas(&conn),
            Err(e) => {
                log::warn!("Canvas falling back to the shared connection: {}", e);
                let conn = lock_db(&pool);
                encode_canvas(&conn)
            }
        }
    })
    .await;

    let bytes = match built {
        Ok(Ok(bytes)) => web::Bytes::from(bytes),
        Ok(Err(e)) => {
            log::error!("Failed to build canvas: {}", e);
            return HttpResponse::InternalServerError().json("Failed to load canvas");
        }
        Err(e) => {
            log::error!("Canvas worker failed: {}", e);
            return HttpResponse::InternalServerError().json("Failed to load canvas");
        }
    };

    cache.store(bytes.clone());
    HttpResponse::Ok().content_type("application/json").body(bytes)
}

pub async fn get_pixel_data(
    query: web::Query<PixelQuery>,
    db: web::Data<DbPool>,
) -> HttpResponse {
    // Parse pixel coordinates from JSON query param
    let pixel: PixelCoord = match &query.pixel {
        Some(s) => match serde_json::from_str(s) {
            Ok(p) => p,
            Err(_) => return HttpResponse::BadRequest().json("Invalid pixel format"),
        },
        None => return HttpResponse::BadRequest().json("pixel parameter required"),
    };

    let db = lock_db(&db);
    let result = db.query_row(
        "SELECT cp.color, cp.updated_at, u.name FROM canvas_pixels cp LEFT JOIN users u ON cp.user_id = u.id WHERE cp.x = ?1 AND cp.y = ?2",
        rusqlite::params![pixel.x, pixel.y],
        |row| {
            Ok(serde_json::json!({
                "c": row.get::<_, String>(0)?,
                "d": row.get::<_, Option<String>>(1)?,
                "username": row.get::<_, Option<String>>(2)?,
            }))
        },
    );

    match result {
        Ok(data) => HttpResponse::Ok().json(data),
        Err(_) => HttpResponse::Ok().json(serde_json::json!({"c": "ffffff"})),
    }
}

/// Colours are stored for a million rows and fed straight back to the client's
/// canvas, so only a plain 6-digit hex triple is accepted.
fn normalize_hex(hex: &str) -> Option<String> {
    let hex = hex.strip_prefix('#').unwrap_or(hex);
    if hex.len() == 6 && hex.chars().all(|c| c.is_ascii_hexdigit()) {
        Some(hex.to_ascii_lowercase())
    } else {
        None
    }
}

pub async fn update_pixel(
    req: HttpRequest,
    body: web::Json<UpdatePixelBody>,
    db: web::Data<DbPool>,
    auth: web::Data<AuthState>,
    broadcaster: web::Data<Arc<WsBroadcaster>>,
    cache: web::Data<Arc<CanvasCache>>,
) -> HttpResponse {
    let claims = match extract_claims(&req, &auth) {
        Some(c) => c,
        None => return unauthorized(),
    };

    let x = body.pixel.selected_pixel.x;
    let y = body.pixel.selected_pixel.y;

    if x < 0 || x >= 1000 || y < 0 || y >= 1000 {
        return HttpResponse::BadRequest().json("Pixel out of bounds");
    }

    let raw_hex = body.pixel.selected_color.hex.clone().unwrap_or_else(|| {
        let r = body.pixel.selected_color.r.unwrap_or(255);
        let g = body.pixel.selected_color.g.unwrap_or(255);
        let b = body.pixel.selected_color.b.unwrap_or(255);
        format!("{:02x}{:02x}{:02x}", r, g, b)
    });

    let hex = match normalize_hex(&raw_hex) {
        Some(hex) => hex,
        None => return HttpResponse::BadRequest().json("Colour must be a 6-digit hex value"),
    };

    let now = chrono::Utc::now().to_rfc3339();
    let db = lock_db(&db);

    let _ = db.execute(
        "UPDATE canvas_pixels SET color = ?1, user_id = ?2, updated_at = ?3 WHERE x = ?4 AND y = ?5",
        rusqlite::params![hex, claims.sub, now, x, y],
    );

    // Log stat
    let stat_id = uuid::Uuid::new_v4().to_string();
    let _ = db.execute(
        "INSERT INTO stats (id, category, user_id, date, metadata) VALUES (?1, 'pixel', ?2, ?3, ?4)",
        rusqlite::params![stat_id, claims.sub, now,
            serde_json::json!({"x": x, "y": y, "color": hex}).to_string()],
    );

    drop(db);
    cache.invalidate();

    // Broadcast via WebSocket
    let msg = serde_json::json!({
        "type": "updatePixel",
        "data": {
            "selectedPixel": {"x": x, "y": y},
            "selectedColor": body.pixel.selected_color,
        }
    });
    broadcaster.broadcast(&msg.to_string());

    HttpResponse::Ok().json("Pixel updated")
}

#[cfg(test)]
mod tests {
    use super::normalize_hex;

    #[test]
    fn accepts_six_digit_hex_with_or_without_hash() {
        assert_eq!(normalize_hex("FF00aa").as_deref(), Some("ff00aa"));
        assert_eq!(normalize_hex("#ff00aa").as_deref(), Some("ff00aa"));
    }

    #[test]
    fn rejects_anything_else() {
        assert!(normalize_hex("").is_none());
        assert!(normalize_hex("fff").is_none());
        assert!(normalize_hex("ff00aaff").is_none());
        assert!(normalize_hex("zzzzzz").is_none());
        assert!(normalize_hex("red; background:url(x)").is_none());
    }
}
