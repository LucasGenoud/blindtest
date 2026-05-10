use actix_web::{web, HttpRequest, HttpResponse};
use serde::Deserialize;
use crate::db::DbPool;
use crate::middleware::{AuthState, extract_claims, unauthorized, forbidden};
#[derive(Deserialize)]
pub struct GetNextAudioQuery {
    pub category: Option<String>,
    #[serde(rename = "passedAudiosIds")]
    pub passed_audios_ids: Option<String>,
    #[serde(rename = "useSuperflus")]
    pub use_superflus: Option<String>,
    #[serde(rename = "prioritizeLessUsedAudios")]
    pub prioritize_less_used: Option<String>,
    #[serde(rename = "userId")]
    pub user_id: Option<String>,
    #[serde(rename = "disabledUsers")]
    pub disabled_users: Option<String>,
    #[serde(rename = "audioId")]
    pub audio_id: Option<String>,
}

#[derive(Deserialize)]
pub struct NewAudioBody {
    pub category: String,
    pub answer: String,
    #[serde(rename = "videoUrl")]
    pub video_url: String,
    #[serde(rename = "startTime")]
    pub start_time: Option<i64>,
    pub superflus: Option<bool>,
}

#[derive(Deserialize)]
pub struct UpdateAudioBody {
    #[serde(rename = "_id")]
    pub id: String,
    pub category: Option<String>,
    pub answer: Option<String>,
    #[serde(rename = "videoUrl")]
    pub video_url: Option<String>,
    #[serde(rename = "startTime")]
    pub start_time: Option<i64>,
    pub superflus: Option<bool>,
}

#[derive(Deserialize)]
pub struct FlagAudioBody {
    pub audio: serde_json::Value,
    #[serde(rename = "reportMessage")]
    pub report_message: Option<String>,
}

#[derive(Deserialize)]
pub struct ResetFlagBody {
    #[serde(rename = "audioId")]
    pub audio_id: String,
}

#[derive(Deserialize)]
pub struct DeleteAudioQuery {
    pub id: String,
}

#[derive(Deserialize)]
pub struct TestAnswerBody {
    #[serde(rename = "audioId")]
    pub audio_id: String,
    #[serde(rename = "userAnswer")]
    pub user_answer: String,
}

pub async fn get_next_audio(
    query: web::Query<GetNextAudioQuery>,
    db: web::Data<DbPool>,
) -> HttpResponse {
    let db = db.lock().unwrap();

    // Parse excluded audio IDs
    let passed_ids: Vec<String> = query.passed_audios_ids.as_deref()
        .and_then(|s| serde_json::from_str(s).ok())
        .unwrap_or_default();

    // Parse disabled users
    let disabled_users: Vec<String> = query.disabled_users.as_deref()
        .and_then(|s| serde_json::from_str(s).ok())
        .unwrap_or_default();

    let use_superflus = query.use_superflus.as_deref() == Some("true");
    let prioritize = query.prioritize_less_used.as_deref() == Some("true");

    // If a specific audioId is provided (custom blindtest mode)
    if let Some(ref audio_id) = query.audio_id {
        let result = db.query_row(
            "SELECT a.id, a.category, a.answer, a.video_url, a.start_time, a.superflus, a.count, a.submitted_by, a.added_date, a.rating, a.rating_count, u.name
             FROM audios a LEFT JOIN users u ON a.submitted_by = u.id WHERE a.id = ?1",
            [audio_id],
            |row| {
                Ok(serde_json::json!({
                    "videoData": {
                        "_id": row.get::<_, String>(0)?,
                        "category": row.get::<_, String>(1)?,
                        "answer": row.get::<_, String>(2)?,
                        "videoUrl": row.get::<_, String>(3)?,
                        "startTime": row.get::<_, i64>(4)?,
                        "superflus": row.get::<_, bool>(5)?,
                        "count": row.get::<_, i64>(6)?,
                        "submittedBy": row.get::<_, String>(7)?,
                        "addedDate": row.get::<_, String>(8)?,
                        "rating": row.get::<_, f64>(9).ok(),
                        "ratingCount": row.get::<_, i64>(10).ok(),
                        "submittedByUsername": row.get::<_, String>(11).ok(),
                    }
                }))
            },
        );

        return match result {
            Ok(data) => {
                // Increment count
                let _ = db.execute("UPDATE audios SET count = count + 1 WHERE id = ?1", [audio_id]);
                HttpResponse::Ok().json(data)
            }
            Err(_) => HttpResponse::NotFound().json("Audio not found"),
        };
    }

    // Build the random selection query
    let mut conditions = vec!["a.processing_status = 'ready'".to_string()];
    let mut params: Vec<Box<dyn rusqlite::types::ToSql>> = Vec::new();

    // Category filter
    if let Some(ref cat) = query.category {
        if !cat.is_empty() {
            params.push(Box::new(cat.clone()));
            conditions.push(format!("a.category = ?{}", params.len()));
        }
    }

    // Exclude superflus if not enabled
    if !use_superflus {
        conditions.push("a.superflus = 0".to_string());
    }

    // Exclude passed audio IDs
    if !passed_ids.is_empty() {
        let placeholders: Vec<String> = passed_ids.iter().enumerate().map(|(i, _)| {
            params.push(Box::new(passed_ids[i].clone()));
            format!("?{}", params.len())
        }).collect();
        conditions.push(format!("a.id NOT IN ({})", placeholders.join(",")));
    }

    // Exclude disabled users
    if !disabled_users.is_empty() {
        let placeholders: Vec<String> = disabled_users.iter().enumerate().map(|(i, _)| {
            params.push(Box::new(disabled_users[i].clone()));
            format!("?{}", params.len())
        }).collect();
        conditions.push(format!("a.submitted_by NOT IN ({})", placeholders.join(",")));
    }

    // Exclude flagged audios
    conditions.push("a.id NOT IN (SELECT DISTINCT audio_id FROM flagged_audios)".to_string());

    let order = if prioritize { "a.count ASC, RANDOM()" } else { "RANDOM()" };
    let sql = format!(
        "SELECT a.id, a.category, a.answer, a.video_url, a.start_time, a.superflus, a.count, a.submitted_by, a.added_date, a.rating, a.rating_count, u.name
         FROM audios a LEFT JOIN users u ON a.submitted_by = u.id
         WHERE {} ORDER BY {} LIMIT 1",
        conditions.join(" AND "), order
    );

    let param_refs: Vec<&dyn rusqlite::types::ToSql> = params.iter().map(|p| p.as_ref()).collect();

    let result = db.query_row(&sql, param_refs.as_slice(), |row| {
        Ok(serde_json::json!({
            "videoData": {
                "_id": row.get::<_, String>(0)?,
                "category": row.get::<_, String>(1)?,
                "answer": row.get::<_, String>(2)?,
                "videoUrl": row.get::<_, String>(3)?,
                "startTime": row.get::<_, i64>(4)?,
                "superflus": row.get::<_, bool>(5)?,
                "count": row.get::<_, i64>(6)?,
                "submittedBy": row.get::<_, String>(7)?,
                "addedDate": row.get::<_, String>(8)?,
                "rating": row.get::<_, f64>(9).ok(),
                "ratingCount": row.get::<_, i64>(10).ok(),
                "submittedByUsername": row.get::<_, String>(11).ok(),
            }
        }))
    });

    match result {
        Ok(mut data) => {
            let audio_id = data["videoData"]["_id"].as_str().unwrap_or("").to_string();
            let _ = db.execute("UPDATE audios SET count = count + 1 WHERE id = ?1", [&audio_id]);

            // Add rating for current user if available
            if let Some(ref uid) = query.user_id {
                if !uid.is_empty() {
                    if let Ok(rating) = db.query_row(
                        "SELECT rating FROM ratings WHERE audio_id = ?1 AND user_id = ?2",
                        rusqlite::params![audio_id, uid],
                        |row| row.get::<_, f64>(0),
                    ) {
                        data.as_object_mut().unwrap().insert("rating".to_string(), serde_json::json!({"rating": rating}));
                    }

                    // Log stat
                    let stat_id = uuid::Uuid::new_v4().to_string();
                    let now = chrono::Utc::now().to_rfc3339();
                    let category = data["videoData"]["category"].as_str().unwrap_or("");
                    let _ = db.execute(
                        "INSERT INTO stats (id, category, user_id, date, metadata) VALUES (?1, 'audioPlay', ?2, ?3, ?4)",
                        rusqlite::params![stat_id, uid, now, serde_json::json!({"audioId": audio_id, "audioCat": category}).to_string()],
                    );
                }
            }

            HttpResponse::Ok().json(data)
        }
        Err(_) => HttpResponse::NotFound().json("No audio found for these criteria"),
    }
}

pub async fn new_audio(
    req: HttpRequest,
    body: web::Json<NewAudioBody>,
    db: web::Data<DbPool>,
    auth: web::Data<AuthState>,
) -> HttpResponse {
    let claims = match extract_claims(&req, &auth) {
        Some(c) if c.role == "contributor" || c.role == "administrator" => c,
        _ => return forbidden(),
    };

    let id = uuid::Uuid::new_v4().to_string();
    let now = chrono::Utc::now().to_rfc3339();
    
    // Rename to avoid shadowing so `db` can be cloned later
    let db_locked = db.lock().unwrap(); 

    let result = db_locked.execute(
        "INSERT INTO audios (id, category, answer, video_url, start_time, superflus, count, submitted_by, added_date, processing_status)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, 0, ?7, ?8, 'processing')",
        rusqlite::params![
            id, body.category, body.answer, body.video_url,
            body.start_time.unwrap_or(0), body.superflus.unwrap_or(false),
            claims.sub, now
        ],
    );

    match result {
        Ok(_) => {
            let stat_id = uuid::Uuid::new_v4().to_string();
            let _ = db_locked.execute(
                "INSERT INTO stats (id, category, user_id, date, metadata) VALUES (?1, 'audioAdd', ?2, ?3, ?4)",
                rusqlite::params![stat_id, claims.sub, now, serde_json::json!({"audioId": id, "audioCat": body.category}).to_string()],
            );

            // Correctly clone the Actix web::Data pool instead of the undefined `db_pool`
            let db_clone = db.clone(); 
            let id_clone = id.clone();
            let video_url = body.video_url.clone();
            let start_time = body.start_time.unwrap_or(0);
            tokio::spawn(async move {
                crate::video_processor::process_and_upload_video(db_clone, id_clone, video_url, start_time, "audios").await;
            });

            HttpResponse::Ok().json(serde_json::json!({"_id": id}))
        }
        Err(e) => HttpResponse::InternalServerError().json(e.to_string()),
    }
}

pub async fn suggest_audio(
    req: HttpRequest,
    body: web::Json<NewAudioBody>,
    db: web::Data<DbPool>, // This is the original 'db'
    auth: web::Data<AuthState>,
) -> HttpResponse {
    let claims = match extract_claims(&req, &auth) {
        Some(c) => c,
        None => return unauthorized(),
    };

    let id = uuid::Uuid::new_v4().to_string();
    let now = chrono::Utc::now().to_rfc3339();
    
    let db_conn = db.lock().unwrap();

    let result = db_conn.execute(
        "INSERT INTO suggestions (id, category, answer, video_url, start_time, superflus, submitted_by, added_date, processing_status)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, 'processing')",
        rusqlite::params![
            id, body.category, body.answer, body.video_url,
            body.start_time.unwrap_or(0), body.superflus.unwrap_or(false),
            claims.sub, now
        ],
    );

    match result {
        Ok(_) => {
            // Now 'db.clone()' refers back to the 'web::Data' in the function arguments
            let db_clone = db.clone(); 
            let id_clone = id.clone();
            let video_url = body.video_url.clone();
            let start_time = body.start_time.unwrap_or(0);
            
            tokio::spawn(async move {
                crate::video_processor::process_and_upload_video(db_clone, id_clone, video_url, start_time, "suggestions").await;
            });
            HttpResponse::Ok().json(serde_json::json!({"_id": id}))
        }
        Err(e) => HttpResponse::InternalServerError().json(e.to_string()),
    }
}

pub async fn get_all_audios(
    req: HttpRequest,
    db: web::Data<DbPool>,
    auth: web::Data<AuthState>,
) -> HttpResponse {
    let claims = match extract_claims(&req, &auth) {
        Some(c) if c.role == "contributor" || c.role == "administrator" => c,
        _ => return forbidden(),
    };

    let db = db.lock().unwrap();
    let mut stmt = db.prepare(
        "SELECT a.id, a.category, a.answer, a.video_url, a.start_time, a.superflus, a.count, a.submitted_by, a.added_date, a.rating, a.rating_count, u.name, a.processing_status, a.s3_object_key
         FROM audios a LEFT JOIN users u ON a.submitted_by = u.id ORDER BY a.added_date DESC"
    ).unwrap();

    let audios: Vec<serde_json::Value> = stmt.query_map([], |row| {
        let audio_id: String = row.get(0)?;
        Ok(serde_json::json!({
            "_id": audio_id,
            "category": row.get::<_, String>(1)?,
            "answer": row.get::<_, String>(2)?,
            "videoUrl": row.get::<_, String>(3)?,
            "startTime": row.get::<_, i64>(4)?,
            "superflus": row.get::<_, bool>(5)?,
            "count": row.get::<_, i64>(6)?,
            "submittedBy": row.get::<_, String>(7)?,
            "addedDate": row.get::<_, String>(8)?,
            "rating": row.get::<_, f64>(9).ok(),
            "ratingCount": row.get::<_, i64>(10).ok(),
            "submittedByUsername": row.get::<_, String>(11).ok(),
            "processingStatus": row.get::<_, String>(12).unwrap_or_else(|_| "ready".to_string()),
            "s3ObjectKey": row.get::<_, String>(13).ok(),
        }))
    }).unwrap().filter_map(|r| r.ok()).collect();

    // Attach flags to each audio
    let mut result: Vec<serde_json::Value> = Vec::new();
    for mut audio in audios {
        let aid = audio["_id"].as_str().unwrap_or("").to_string();
        let mut flag_stmt = db.prepare(
            "SELECT id, report_message, user_id, date FROM flagged_audios WHERE audio_id = ?1"
        ).unwrap();
        let flags: Vec<serde_json::Value> = flag_stmt.query_map([&aid], |row| {
            Ok(serde_json::json!({
                "id": row.get::<_, String>(0)?,
                "reportMessage": row.get::<_, String>(1)?,
                "userId": row.get::<_, String>(2)?,
                "date": row.get::<_, String>(3)?,
            }))
        }).unwrap().filter_map(|r| r.ok()).collect();

        audio.as_object_mut().unwrap().insert("flagged".to_string(), serde_json::json!(flags));
        result.push(audio);
    }

    HttpResponse::Ok().json(result)
}

pub async fn update_audio(
    req: HttpRequest,
    body: web::Json<UpdateAudioBody>,
    db: web::Data<DbPool>,
    auth: web::Data<AuthState>,
) -> HttpResponse {
    let claims = match extract_claims(&req, &auth) {
        Some(c) if c.role == "contributor" || c.role == "administrator" => c,
        _ => return forbidden(),
    };

    let db = db.lock().unwrap();
    if let Some(ref cat) = body.category {
        let _ = db.execute("UPDATE audios SET category = ?1 WHERE id = ?2", rusqlite::params![cat, body.id]);
    }
    if let Some(ref answer) = body.answer {
        let _ = db.execute("UPDATE audios SET answer = ?1 WHERE id = ?2", rusqlite::params![answer, body.id]);
    }
    if let Some(ref url) = body.video_url {
        let _ = db.execute("UPDATE audios SET video_url = ?1 WHERE id = ?2", rusqlite::params![url, body.id]);
    }
    if let Some(st) = body.start_time {
        let _ = db.execute("UPDATE audios SET start_time = ?1 WHERE id = ?2", rusqlite::params![st, body.id]);
    }
    if let Some(sup) = body.superflus {
        let _ = db.execute("UPDATE audios SET superflus = ?1 WHERE id = ?2", rusqlite::params![sup, body.id]);
    }
    let _ = db.execute("UPDATE audios SET last_updated_by = ?1 WHERE id = ?2", rusqlite::params![claims.sub, body.id]);

    HttpResponse::Ok().json("Audio updated")
}

pub async fn flag_audio(
    req: HttpRequest,
    body: web::Json<FlagAudioBody>,
    db: web::Data<DbPool>,
    auth: web::Data<AuthState>,
) -> HttpResponse {
    let claims = match extract_claims(&req, &auth) {
        Some(c) => c,
        None => return unauthorized(),
    };

    let audio_id = body.audio.get("_id").and_then(|v| v.as_str()).unwrap_or("");
    let id = uuid::Uuid::new_v4().to_string();
    let now = chrono::Utc::now().to_rfc3339();
    let msg = body.report_message.as_deref().unwrap_or("");

    let db = db.lock().unwrap();
    let _ = db.execute(
        "INSERT INTO flagged_audios (id, audio_id, user_id, report_message, date) VALUES (?1, ?2, ?3, ?4, ?5)",
        rusqlite::params![id, audio_id, claims.sub, msg, now],
    );

    HttpResponse::Ok().json("Audio flagged")
}

pub async fn reset_flag(
    req: HttpRequest,
    body: web::Json<ResetFlagBody>,
    db: web::Data<DbPool>,
    auth: web::Data<AuthState>,
) -> HttpResponse {
    let claims = match extract_claims(&req, &auth) {
        Some(c) if c.role == "contributor" || c.role == "administrator" => c,
        _ => return forbidden(),
    };

    let db = db.lock().unwrap();
    let _ = db.execute("DELETE FROM flagged_audios WHERE audio_id = ?1", [&body.audio_id]);
    HttpResponse::Ok().json("Flags reset")
}

pub async fn delete_audio(
    req: HttpRequest,
    query: web::Query<DeleteAudioQuery>,
    db: web::Data<DbPool>,
    auth: web::Data<AuthState>,
) -> HttpResponse {
    let claims = match extract_claims(&req, &auth) {
        Some(c) if c.role == "administrator" => c,
        _ => return forbidden(),
    };

    let db = db.lock().unwrap();
    let _ = db.execute("DELETE FROM audios WHERE id = ?1", [&query.id]);
    HttpResponse::Ok().json("Audio deleted")
}

pub async fn test_answer(
    body: web::Json<TestAnswerBody>,
    db: web::Data<DbPool>,
) -> HttpResponse {
    let db = db.lock().unwrap();
    let result = db.query_row(
        "SELECT answer FROM audios WHERE id = ?1",
        [&body.audio_id],
        |row| row.get::<_, String>(0),
    );

    match result {
        Ok(correct) => {
            let similarity = strsim::jaro_winkler(
                &correct.to_lowercase(),
                &body.user_answer.to_lowercase(),
            );
            HttpResponse::Ok().json(serde_json::json!({"similarity": similarity}))
        }
        Err(_) => HttpResponse::NotFound().json("Audio not found"),
    }
}

pub async fn backup_audios(
    req: HttpRequest,
    db: web::Data<DbPool>,
    auth: web::Data<AuthState>,
) -> HttpResponse {
    let claims = match extract_claims(&req, &auth) {
        Some(c) if c.role == "administrator" => c,
        _ => return forbidden(),
    };

    let db = db.lock().unwrap();

    // Collect all audios as JSON
    let mut stmt = db.prepare("SELECT id, category, answer, video_url, start_time, superflus, count, submitted_by, added_date FROM audios").unwrap();
    let audios: Vec<serde_json::Value> = stmt.query_map([], |row| {
        Ok(serde_json::json!({
            "_id": row.get::<_, String>(0)?,
            "category": row.get::<_, String>(1)?,
            "answer": row.get::<_, String>(2)?,
            "videoUrl": row.get::<_, String>(3)?,
            "startTime": row.get::<_, i64>(4)?,
            "superflus": row.get::<_, bool>(5)?,
            "count": row.get::<_, i64>(6)?,
            "submittedBy": row.get::<_, String>(7)?,
            "addedDate": row.get::<_, String>(8)?,
        }))
    }).unwrap().filter_map(|r| r.ok()).collect();

    let json_data = serde_json::to_string_pretty(&audios).unwrap_or_default();

    use std::io::Write;
    let mut buf = Vec::new();
    {
        let mut zip_writer = zip::ZipWriter::new(std::io::Cursor::new(&mut buf));
        let options = zip::write::SimpleFileOptions::default();
        zip_writer.start_file("audios.json", options).unwrap();
        zip_writer.write_all(json_data.as_bytes()).unwrap();
        zip_writer.finish().unwrap();
    }

    HttpResponse::Ok()
        .content_type("application/zip")
        .append_header(("Content-Disposition", "attachment; filename=backup.zip"))
        .body(buf)
}
