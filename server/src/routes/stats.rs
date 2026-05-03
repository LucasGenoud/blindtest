use actix_web::{web, HttpResponse};
use crate::db::DbPool;

pub async fn get_blindtest_stats(db: web::Data<DbPool>) -> HttpResponse {
    let db = db.lock().unwrap();

    // Audio play stats grouped by category
    let mut stmt = db.prepare(
        "SELECT json_extract(metadata, '$.audioCat') as cat, COUNT(*) as cnt
         FROM stats WHERE category = 'audioPlay' AND json_extract(metadata, '$.audioCat') IS NOT NULL
         GROUP BY cat ORDER BY cnt DESC"
    ).unwrap();

    let play_by_cat: Vec<serde_json::Value> = stmt.query_map([], |row| {
        Ok(serde_json::json!({
            "category": row.get::<_, String>(0)?,
            "count": row.get::<_, i64>(1)?,
        }))
    }).unwrap().filter_map(|r| r.ok()).collect();

    // Audio add stats grouped by month
    let mut stmt2 = db.prepare(
        "SELECT strftime('%Y-%m', date) as month, COUNT(*) as cnt
         FROM stats WHERE category = 'audioAdd'
         GROUP BY month ORDER BY month"
    ).unwrap();

    let adds_by_month: Vec<serde_json::Value> = stmt2.query_map([], |row| {
        Ok(serde_json::json!({
            "month": row.get::<_, String>(0)?,
            "count": row.get::<_, i64>(1)?,
        }))
    }).unwrap().filter_map(|r| r.ok()).collect();

    // Total counts
    let total_audios: i64 = db.query_row("SELECT COUNT(*) FROM audios", [], |r| r.get(0)).unwrap_or(0);
    let total_users: i64 = db.query_row("SELECT COUNT(*) FROM users WHERE deleted = 0", [], |r| r.get(0)).unwrap_or(0);
    let total_plays: i64 = db.query_row("SELECT COUNT(*) FROM stats WHERE category = 'audioPlay'", [], |r| r.get(0)).unwrap_or(0);

    HttpResponse::Ok().json(serde_json::json!({
        "playsByCategory": play_by_cat,
        "addsByMonth": adds_by_month,
        "totalAudios": total_audios,
        "totalUsers": total_users,
        "totalPlays": total_plays,
    }))
}

pub async fn get_canvas_stats(db: web::Data<DbPool>) -> HttpResponse {
    let db = db.lock().unwrap();

    // Pixels placed by user
    let mut stmt = db.prepare(
        "SELECT u.name, COUNT(*) as cnt FROM stats s
         JOIN users u ON s.user_id = u.id
         WHERE s.category = 'pixel'
         GROUP BY s.user_id ORDER BY cnt DESC LIMIT 20"
    ).unwrap();

    let by_user: Vec<serde_json::Value> = stmt.query_map([], |row| {
        Ok(serde_json::json!({
            "username": row.get::<_, String>(0)?,
            "count": row.get::<_, i64>(1)?,
        }))
    }).unwrap().filter_map(|r| r.ok()).collect();

    // Pixels placed by day
    let mut stmt2 = db.prepare(
        "SELECT strftime('%Y-%m-%d', date) as day, COUNT(*) as cnt
         FROM stats WHERE category = 'pixel'
         GROUP BY day ORDER BY day DESC LIMIT 30"
    ).unwrap();

    let by_day: Vec<serde_json::Value> = stmt2.query_map([], |row| {
        Ok(serde_json::json!({
            "day": row.get::<_, String>(0)?,
            "count": row.get::<_, i64>(1)?,
        }))
    }).unwrap().filter_map(|r| r.ok()).collect();

    let total_pixels: i64 = db.query_row("SELECT COUNT(*) FROM stats WHERE category = 'pixel'", [], |r| r.get(0)).unwrap_or(0);

    HttpResponse::Ok().json(serde_json::json!({
        "byUser": by_user,
        "byDay": by_day,
        "totalPixels": total_pixels,
    }))
}
