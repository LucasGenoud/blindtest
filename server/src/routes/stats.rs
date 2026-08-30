use actix_web::{web, HttpResponse};
use crate::db::{lock_db, DbPool};

type StatsResult = Result<serde_json::Value, Box<dyn std::error::Error + Send + Sync>>;

fn grouped(
    conn: &rusqlite::Connection,
    sql: &str,
    key: &str,
) -> Result<Vec<serde_json::Value>, rusqlite::Error> {
    let mut stmt = conn.prepare(sql)?;
    let rows: Vec<serde_json::Value> = stmt
        .query_map([], |row| {
            Ok(serde_json::json!({
                key: row.get::<_, String>(0)?,
                "count": row.get::<_, i64>(1)?,
            }))
        })?
        .filter_map(|r| r.ok())
        .collect();
    Ok(rows)
}

fn respond(built: Result<StatsResult, actix_web::error::BlockingError>) -> HttpResponse {
    match built {
        Ok(Ok(stats)) => HttpResponse::Ok().json(stats),
        Ok(Err(e)) => {
            log::error!("Failed to build stats: {}", e);
            HttpResponse::InternalServerError().json("Failed to build stats")
        }
        Err(e) => {
            log::error!("Stats worker failed: {}", e);
            HttpResponse::InternalServerError().json("Failed to build stats")
        }
    }
}

// These aggregate hundreds of thousands of `stats` rows without a usable index, so
// they run on a blocking thread: on an actix worker they froze every other request
// for the duration of the scan.
pub async fn get_blindtest_stats(db: web::Data<DbPool>) -> HttpResponse {
    let pool = db.get_ref().clone();
    respond(web::block(move || -> StatsResult {
        let conn = lock_db(&pool);

        // Audio play stats grouped by category
        let play_by_cat = grouped(
            &conn,
            "SELECT json_extract(metadata, '$.audioCat') as cat, COUNT(*) as cnt
             FROM stats WHERE category = 'audioPlay' AND json_extract(metadata, '$.audioCat') IS NOT NULL
             GROUP BY cat ORDER BY cnt DESC",
            "category",
        )?;

        // Audio add stats grouped by month
        let adds_by_month = grouped(
            &conn,
            "SELECT strftime('%Y-%m', date) as month, COUNT(*) as cnt
             FROM stats WHERE category = 'audioAdd'
             GROUP BY month ORDER BY month",
            "month",
        )?;

        // Total counts
        let total_audios: i64 = conn.query_row("SELECT COUNT(*) FROM audios", [], |r| r.get(0)).unwrap_or(0);
        let total_users: i64 = conn.query_row("SELECT COUNT(*) FROM users WHERE deleted = 0", [], |r| r.get(0)).unwrap_or(0);
        let total_plays: i64 = conn.query_row("SELECT COUNT(*) FROM stats WHERE category = 'audioPlay'", [], |r| r.get(0)).unwrap_or(0);

        Ok(serde_json::json!({
            "playsByCategory": play_by_cat,
            "addsByMonth": adds_by_month,
            "totalAudios": total_audios,
            "totalUsers": total_users,
            "totalPlays": total_plays,
        }))
    })
    .await)
}

pub async fn get_canvas_stats(db: web::Data<DbPool>) -> HttpResponse {
    let pool = db.get_ref().clone();
    respond(web::block(move || -> StatsResult {
        let conn = lock_db(&pool);

        // Pixels placed by user
        let by_user = grouped(
            &conn,
            "SELECT u.name, COUNT(*) as cnt FROM stats s
             JOIN users u ON s.user_id = u.id
             WHERE s.category = 'pixel'
             GROUP BY s.user_id ORDER BY cnt DESC LIMIT 20",
            "username",
        )?;

        // Pixels placed by day
        let by_day = grouped(
            &conn,
            "SELECT strftime('%Y-%m-%d', date) as day, COUNT(*) as cnt
             FROM stats WHERE category = 'pixel'
             GROUP BY day ORDER BY day DESC LIMIT 30",
            "day",
        )?;

        let total_pixels: i64 = conn
            .query_row("SELECT COUNT(*) FROM stats WHERE category = 'pixel'", [], |r| r.get(0))
            .unwrap_or(0);

        Ok(serde_json::json!({
            "byUser": by_user,
            "byDay": by_day,
            "totalPixels": total_pixels,
        }))
    })
    .await)
}
