use actix_web::{web, HttpRequest, HttpResponse};
use serde::Deserialize;
use crate::db::DbPool;
use crate::middleware::{AuthState, extract_claims, unauthorized};

#[derive(Deserialize)]
pub struct RateBody {
    pub rating: f64,
}

pub async fn rate_audio(
    req: HttpRequest,
    path: web::Path<String>,
    body: web::Json<RateBody>,
    db: web::Data<DbPool>,
    auth: web::Data<AuthState>,
) -> HttpResponse {
    let claims = match extract_claims(&req, &auth) {
        Some(c) => c,
        None => return unauthorized(),
    };

    let audio_id = path.into_inner();
    let id = uuid::Uuid::new_v4().to_string();
    let now = chrono::Utc::now().to_rfc3339();
    let db = db.lock().unwrap();

    // Upsert rating
    let existing = db.query_row(
        "SELECT id FROM ratings WHERE audio_id = ?1 AND user_id = ?2",
        rusqlite::params![audio_id, claims.sub],
        |row| row.get::<_, String>(0),
    );

    match existing {
        Ok(existing_id) => {
            let _ = db.execute(
                "UPDATE ratings SET rating = ?1, added_date = ?2 WHERE id = ?3",
                rusqlite::params![body.rating, now, existing_id],
            );
        }
        Err(_) => {
            let _ = db.execute(
                "INSERT INTO ratings (id, audio_id, user_id, rating, added_date) VALUES (?1, ?2, ?3, ?4, ?5)",
                rusqlite::params![id, audio_id, claims.sub, body.rating, now],
            );
        }
    }

    // Recalculate average
    let avg: f64 = db.query_row(
        "SELECT COALESCE(AVG(rating), 0) FROM ratings WHERE audio_id = ?1",
        [&audio_id],
        |row| row.get(0),
    ).unwrap_or(0.0);

    let count: i64 = db.query_row(
        "SELECT COUNT(*) FROM ratings WHERE audio_id = ?1",
        [&audio_id],
        |row| row.get(0),
    ).unwrap_or(0);

    let _ = db.execute(
        "UPDATE audios SET rating = ?1, rating_count = ?2 WHERE id = ?3",
        rusqlite::params![avg, count, audio_id],
    );

    HttpResponse::Ok().json(serde_json::json!({"rating": body.rating, "average": avg}))
}
