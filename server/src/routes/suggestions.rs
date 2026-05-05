use actix_web::{web, HttpResponse};
use crate::db::DbPool;

pub async fn get_suggestions(
    db: web::Data<DbPool>,
) -> HttpResponse {
    let db = db.lock().unwrap();
    let mut stmt = db.prepare(
        "SELECT s.id, s.category, s.answer, s.video_url, s.start_time, s.superflus, s.submitted_by, s.added_date, u.name, s.processing_status
         FROM suggestions s LEFT JOIN users u ON s.submitted_by = u.id ORDER BY s.added_date DESC"
    ).unwrap();

    let items: Vec<serde_json::Value> = stmt.query_map([], |row| {
        Ok(serde_json::json!({
            "_id": row.get::<_, String>(0)?,
            "category": row.get::<_, String>(1)?,
            "answer": row.get::<_, String>(2)?,
            "videoUrl": row.get::<_, String>(3)?,
            "startTime": row.get::<_, i64>(4)?,
            "superflus": row.get::<_, bool>(5)?,
            "submittedBy": row.get::<_, String>(6)?,
            "addedDate": row.get::<_, String>(7)?,
            "submittedByUsername": row.get::<_, String>(8).ok(),
            "processingStatus": row.get::<_, String>(9).unwrap_or_else(|_| "ready".to_string()),
        }))
    }).unwrap().filter_map(|r| r.ok()).collect();

    HttpResponse::Ok().json(items)
}
