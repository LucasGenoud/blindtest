use actix_web::{web, HttpRequest, HttpResponse};
use serde::Deserialize;
use crate::db::DbPool;
use crate::middleware::{AuthState, extract_claims, unauthorized};
use crate::ws::WsBroadcaster;
use std::sync::Arc;

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

pub async fn get_canvas(
    db: web::Data<DbPool>,
) -> HttpResponse {
    let db = db.lock().unwrap();
    let mut stmt = db.prepare(
        "SELECT color FROM canvas_pixels ORDER BY y, x"
    ).unwrap();

    let pixels: Vec<String> = stmt.query_map([], |row| {
        row.get::<_, String>(0)
    }).unwrap().filter_map(|r| r.ok()).collect();

    HttpResponse::Ok().json(pixels)
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

    let db = db.lock().unwrap();
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

pub async fn update_pixel(
    req: HttpRequest,
    body: web::Json<UpdatePixelBody>,
    db: web::Data<DbPool>,
    auth: web::Data<AuthState>,
    broadcaster: web::Data<Arc<WsBroadcaster>>,
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

    let hex = body.pixel.selected_color.hex.clone().unwrap_or_else(|| {
        let r = body.pixel.selected_color.r.unwrap_or(255);
        let g = body.pixel.selected_color.g.unwrap_or(255);
        let b = body.pixel.selected_color.b.unwrap_or(255);
        format!("{:02x}{:02x}{:02x}", r, g, b)
    });

    let now = chrono::Utc::now().to_rfc3339();
    let db = db.lock().unwrap();

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
