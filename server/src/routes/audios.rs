use actix_web::{web, HttpRequest, HttpResponse};
use serde::Deserialize;
use crate::db::{lock_db, DbPool};
use crate::db_try;
use crate::middleware::{AuthState, extract_claims, unauthorized, forbidden};
use crate::video_processor::{is_supported_video_url, ProcessingJob, ProcessingQueue};

/// Manual flags hide an audio from every player, so they are rate limited per account.
const MANUAL_FLAGS_PER_HOUR: i64 = 10;

/// How many submissions one account may have awaiting processing at a time.
const MAX_PENDING_PER_USER: i64 = 10;

fn queue_full() -> HttpResponse {
    HttpResponse::ServiceUnavailable().json("Processing queue is full, try again later")
}

fn bad_video_url() -> HttpResponse {
    HttpResponse::BadRequest().json("videoUrl must be an http(s) link")
}

/// Submissions sit in a bounded queue behind a single worker, so one account cannot
/// monopolise it.
fn pending_for_user(conn: &rusqlite::Connection, user_id: &str) -> i64 {
    let audios: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM audios WHERE submitted_by = ?1 AND processing_status = 'processing'",
            rusqlite::params![user_id],
            |row| row.get(0),
        )
        .unwrap_or(0);
    let suggestions: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM suggestions WHERE submitted_by = ?1 AND processing_status = 'processing'",
            rusqlite::params![user_id],
            |row| row.get(0),
        )
        .unwrap_or(0);
    audios + suggestions
}
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
    /// Raised by the client itself after a playback error rather than by a person.
    /// Automatic flags are recorded for contributors to review but never remove the
    /// audio from rotation — a bad afternoon on the server used to be able to empty
    /// the pool for everyone.
    #[serde(default)]
    pub auto: bool,
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
pub struct AudioIdQuery {
    #[serde(rename = "audioId")]
    pub audio_id: String,
}

#[derive(Deserialize)]
pub struct TestAnswerBody {
    #[serde(rename = "audioId")]
    pub audio_id: String,
    #[serde(rename = "userAnswer")]
    pub user_answer: String,
}

pub async fn get_next_audio(
    req: HttpRequest,
    query: web::Query<GetNextAudioQuery>,
    db: web::Data<DbPool>,
    auth: web::Data<AuthState>,
) -> HttpResponse {
    // Play stats are attributed to the caller's token, not to a `userId` they picked.
    let user_id = extract_claims(&req, &auth).map(|c| c.sub);
    let db = lock_db(&db);

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
            "SELECT a.id, a.category, a.video_url, a.start_time, a.superflus, a.count, a.submitted_by, a.added_date, u.name
             FROM audios a LEFT JOIN users u ON a.submitted_by = u.id WHERE a.id = ?1",
            [audio_id],
            |row| {
                Ok(serde_json::json!({
                    "videoData": {
                        "_id": row.get::<_, String>(0)?,
                        "category": row.get::<_, String>(1)?,
                        "videoUrl": row.get::<_, String>(2)?,
                        "startTime": row.get::<_, i64>(3)?,
                        "superflus": row.get::<_, bool>(4)?,
                        "count": row.get::<_, i64>(5)?,
                        "submittedBy": row.get::<_, String>(6)?,
                        "addedDate": row.get::<_, String>(7)?,
                        "submittedByUsername": row.get::<_, String>(8).ok(),
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
    conditions.push("a.id NOT IN (SELECT DISTINCT audio_id FROM flagged_audios WHERE auto = 0)".to_string());

    let order = if prioritize { "a.count ASC, RANDOM()" } else { "RANDOM()" };
    let sql = format!(
        "SELECT a.id, a.category, a.video_url, a.start_time, a.superflus, a.count, a.submitted_by, a.added_date, u.name
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
                "videoUrl": row.get::<_, String>(2)?,
                "startTime": row.get::<_, i64>(3)?,
                "superflus": row.get::<_, bool>(4)?,
                "count": row.get::<_, i64>(5)?,
                "submittedBy": row.get::<_, String>(6)?,
                "addedDate": row.get::<_, String>(7)?,
                "submittedByUsername": row.get::<_, String>(8).ok(),
            }
        }))
    });

    match result {
        Ok(mut data) => {
            let audio_id = data["videoData"]["_id"].as_str().unwrap_or("").to_string();
            let _ = db.execute("UPDATE audios SET count = count + 1 WHERE id = ?1", [&audio_id]);

            // Log user stat if available
            if let Some(ref uid) = user_id {
                if !uid.is_empty() {
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

/// The answer is deliberately not part of `/getnextaudio`, otherwise every player
/// could read it out of the network tab before guessing. It is fetched at reveal.
pub async fn get_audio_answer(
    query: web::Query<AudioIdQuery>,
    db: web::Data<DbPool>,
) -> HttpResponse {
    let db = lock_db(&db);
    let answer: Result<String, _> = db.query_row(
        "SELECT answer FROM audios WHERE id = ?1",
        rusqlite::params![query.audio_id],
        |row| row.get(0),
    );

    match answer {
        Ok(answer) => HttpResponse::Ok().json(serde_json::json!({ "answer": answer })),
        Err(_) => HttpResponse::NotFound().json("Audio not found"),
    }
}

pub async fn new_audio(
    req: HttpRequest,
    body: web::Json<NewAudioBody>,
    db: web::Data<DbPool>,
    auth: web::Data<AuthState>,
    queue: web::Data<ProcessingQueue>,
) -> HttpResponse {
    let claims = match extract_claims(&req, &auth) {
        Some(c) if c.role == "contributor" || c.role == "administrator" => c,
        _ => return forbidden(),
    };

    if !is_supported_video_url(&body.video_url) {
        return bad_video_url();
    }

    let id = uuid::Uuid::new_v4().to_string();
    let now = chrono::Utc::now().to_rfc3339();
    
    // Rename to avoid shadowing so `db` can be cloned later
    let db_locked = lock_db(&db); 

    if pending_for_user(&db_locked, &claims.sub) >= MAX_PENDING_PER_USER {
        return queue_full();
    }

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
            if queue.try_send(ProcessingJob {
                db: db.clone(),
                audio_id: id.clone(),
                video_url: body.video_url.clone(),
                start_time: body.start_time.unwrap_or(0),
                table: "audios",
            }).is_err() {
                let _ = db_locked.execute(
                    "UPDATE audios SET processing_status = 'error' WHERE id = ?1",
                    rusqlite::params![id],
                );
                return queue_full();
            }

            HttpResponse::Ok().json(serde_json::json!({"_id": id}))
        }
        Err(e) => HttpResponse::InternalServerError().json(e.to_string()),
    }
}

pub async fn suggest_audio(
    req: HttpRequest,
    body: web::Json<NewAudioBody>,
    db: web::Data<DbPool>,
    auth: web::Data<AuthState>,
    queue: web::Data<ProcessingQueue>,
) -> HttpResponse {
    let claims = match extract_claims(&req, &auth) {
        Some(c) => c,
        None => return unauthorized(),
    };

    if !is_supported_video_url(&body.video_url) {
        return bad_video_url();
    }

    let id = uuid::Uuid::new_v4().to_string();
    let now = chrono::Utc::now().to_rfc3339();
    
    let db_conn = lock_db(&db);

    if pending_for_user(&db_conn, &claims.sub) >= MAX_PENDING_PER_USER {
        return queue_full();
    }

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
            if queue.try_send(ProcessingJob {
                db: db.clone(),
                audio_id: id.clone(),
                video_url: body.video_url.clone(),
                start_time: body.start_time.unwrap_or(0),
                table: "suggestions",
            }).is_err() {
                let _ = db_conn.execute(
                    "UPDATE suggestions SET processing_status = 'error' WHERE id = ?1",
                    rusqlite::params![id],
                );
                return queue_full();
            }
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

    let db = lock_db(&db);
    let mut stmt = db_try!(db.prepare(
        "SELECT a.id, a.category, a.answer, a.video_url, a.start_time, a.superflus, a.count, a.submitted_by, a.added_date, u.name, a.processing_status, a.s3_object_key
         FROM audios a LEFT JOIN users u ON a.submitted_by = u.id ORDER BY a.added_date DESC"
    ));

    let audios: Vec<serde_json::Value> = db_try!(stmt.query_map([], |row| {
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
            "submittedByUsername": row.get::<_, String>(9).ok(),
            "processingStatus": row.get::<_, String>(10).unwrap_or_else(|_| "ready".to_string()),
            "s3ObjectKey": row.get::<_, String>(11).ok(),
        }))
    })).filter_map(|r| r.ok()).collect();

    // Flags for every audio in one pass. This used to prepare and run a statement per
    // audio — over two thousand round trips per page load, all under the connection lock.
    let mut flag_stmt = db_try!(db.prepare(
        "SELECT audio_id, id, report_message, user_id, date, auto FROM flagged_audios"
    ));
    let mut flags_by_audio: std::collections::HashMap<String, Vec<serde_json::Value>> =
        std::collections::HashMap::new();
    let flag_rows = db_try!(flag_stmt.query_map([], |row| {
        Ok((
            row.get::<_, String>(0)?,
            serde_json::json!({
                "id": row.get::<_, String>(1)?,
                "reportMessage": row.get::<_, String>(2)?,
                "userId": row.get::<_, String>(3)?,
                "date": row.get::<_, String>(4)?,
                "auto": row.get::<_, bool>(5).unwrap_or(false),
            }),
        ))
    }));
    for (audio_id, flag) in flag_rows.filter_map(|r| r.ok()) {
        flags_by_audio.entry(audio_id).or_default().push(flag);
    }

    let result: Vec<serde_json::Value> = audios
        .into_iter()
        .map(|mut audio| {
            let aid = audio["_id"].as_str().unwrap_or("").to_string();
            let flags = flags_by_audio.remove(&aid).unwrap_or_default();
            if let Some(obj) = audio.as_object_mut() {
                obj.insert("flagged".to_string(), serde_json::json!(flags));
            }
            audio
        })
        .collect();

    HttpResponse::Ok().json(result)
}

pub async fn update_audio(
    req: HttpRequest,
    body: web::Json<UpdateAudioBody>,
    db: web::Data<DbPool>,
    auth: web::Data<AuthState>,
    queue: web::Data<ProcessingQueue>,
) -> HttpResponse {
    let claims = match extract_claims(&req, &auth) {
        Some(c) if c.role == "contributor" || c.role == "administrator" => c,
        _ => return forbidden(),
    };

    // Fetch current video_url and start_time to detect changes
    let (current_url, current_start_time): (String, i64) = {
        let db_locked = lock_db(&db);
        match db_locked.query_row(
            "SELECT video_url, start_time FROM audios WHERE id = ?1",
            rusqlite::params![body.id],
            |row| Ok((row.get::<_, String>(0)?, row.get::<_, i64>(1)?)),
        ) {
            Ok(v) => v,
            Err(_) => return HttpResponse::NotFound().json("Audio not found"),
        }
    };

    let new_url = body.video_url.as_deref().unwrap_or(&current_url);
    let new_start_time = body.start_time.unwrap_or(current_start_time);
    let needs_reprocess = new_url != current_url || new_start_time != current_start_time;

    if needs_reprocess && !is_supported_video_url(new_url) {
        return bad_video_url();
    }

    {
        let db_locked = lock_db(&db);
        if let Some(ref cat) = body.category {
            let _ = db_locked.execute("UPDATE audios SET category = ?1 WHERE id = ?2", rusqlite::params![cat, body.id]);
        }
        if let Some(ref answer) = body.answer {
            let _ = db_locked.execute("UPDATE audios SET answer = ?1 WHERE id = ?2", rusqlite::params![answer, body.id]);
        }
        if let Some(ref url) = body.video_url {
            let _ = db_locked.execute("UPDATE audios SET video_url = ?1 WHERE id = ?2", rusqlite::params![url, body.id]);
        }
        if let Some(st) = body.start_time {
            let _ = db_locked.execute("UPDATE audios SET start_time = ?1 WHERE id = ?2", rusqlite::params![st, body.id]);
        }
        if let Some(sup) = body.superflus {
            let _ = db_locked.execute("UPDATE audios SET superflus = ?1 WHERE id = ?2", rusqlite::params![sup, body.id]);
        }
        let _ = db_locked.execute("UPDATE audios SET last_updated_by = ?1 WHERE id = ?2", rusqlite::params![claims.sub, body.id]);

        if needs_reprocess {
            let _ = db_locked.execute(
                "UPDATE audios SET processing_status = 'processing', s3_object_key = NULL WHERE id = ?1",
                rusqlite::params![body.id],
            );
        }
    }

    if needs_reprocess {
        if queue.try_send(ProcessingJob {
            db: db.clone(),
            audio_id: body.id.clone(),
            video_url: new_url.to_string(),
            start_time: new_start_time,
            table: "audios",
        }).is_err() {
            let db_locked = lock_db(&db);
            let _ = db_locked.execute(
                "UPDATE audios SET processing_status = 'error' WHERE id = ?1",
                rusqlite::params![body.id],
            );
            return queue_full();
        }
    }

    HttpResponse::Ok().json("Audio updated")
}

/// Requeue an audio whose processing failed. Without this the only way to retry was
/// to edit the video URL into something different and back again.
pub async fn reprocess_audio(
    req: HttpRequest,
    query: web::Query<AudioIdQuery>,
    db: web::Data<DbPool>,
    auth: web::Data<AuthState>,
    queue: web::Data<ProcessingQueue>,
) -> HttpResponse {
    match extract_claims(&req, &auth) {
        Some(c) if c.role == "contributor" || c.role == "administrator" => c,
        _ => return forbidden(),
    };

    let (video_url, start_time): (String, i64) = {
        let db_locked = lock_db(&db);
        match db_locked.query_row(
            "SELECT video_url, start_time FROM audios WHERE id = ?1",
            rusqlite::params![query.audio_id],
            |row| Ok((row.get::<_, String>(0)?, row.get::<_, i64>(1)?)),
        ) {
            Ok(v) => v,
            Err(_) => return HttpResponse::NotFound().json("Audio not found"),
        }
    };

    if !is_supported_video_url(&video_url) {
        return bad_video_url();
    }

    {
        let db_locked = lock_db(&db);
        let _ = db_locked.execute(
            "UPDATE audios SET processing_status = 'processing', s3_object_key = NULL WHERE id = ?1",
            rusqlite::params![query.audio_id],
        );
    }

    if queue.try_send(ProcessingJob {
        db: db.clone(),
        audio_id: query.audio_id.clone(),
        video_url,
        start_time,
        table: "audios",
    }).is_err() {
        let db_locked = lock_db(&db);
        let _ = db_locked.execute(
            "UPDATE audios SET processing_status = 'error' WHERE id = ?1",
            rusqlite::params![query.audio_id],
        );
        return queue_full();
    }

    HttpResponse::Ok().json("Audio queued for reprocessing")
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
    if audio_id.is_empty() {
        return HttpResponse::BadRequest().json("Audio id required");
    }
    let id = uuid::Uuid::new_v4().to_string();
    let now = chrono::Utc::now().to_rfc3339();
    let msg = body.report_message.as_deref().unwrap_or("");

    let db = lock_db(&db);

    // A manual flag hides the audio from everyone, so cap how many one account can
    // raise per hour. Without it a single user could empty the pool.
    if !body.auto {
        let cutoff = (chrono::Utc::now() - chrono::Duration::hours(1)).to_rfc3339();
        let recent: i64 = db
            .query_row(
                "SELECT COUNT(*) FROM flagged_audios WHERE user_id = ?1 AND auto = 0 AND date > ?2",
                rusqlite::params![claims.sub, cutoff],
                |row| row.get(0),
            )
            .unwrap_or(0);
        if recent >= MANUAL_FLAGS_PER_HOUR {
            return HttpResponse::TooManyRequests().json("Too many flags, try again later");
        }
    }

    // One flag per user per audio (enforced by a unique index); re-flagging just
    // updates the message rather than piling up rows.
    let result = db.execute(
        "INSERT INTO flagged_audios (id, audio_id, user_id, report_message, date, auto)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6)
         ON CONFLICT (audio_id, user_id) DO UPDATE SET
             report_message = excluded.report_message,
             date = excluded.date,
             auto = MIN(auto, excluded.auto)",
        rusqlite::params![id, audio_id, claims.sub, msg, now, body.auto as i64],
    );

    match result {
        Ok(_) => HttpResponse::Ok().json("Audio flagged"),
        Err(e) => {
            log::error!("Failed to flag audio {}: {}", audio_id, e);
            HttpResponse::InternalServerError().json("Failed to flag audio")
        }
    }
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

    let db = lock_db(&db);
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

    let db = lock_db(&db);
    let _ = db.execute("DELETE FROM audios WHERE id = ?1", [&query.id]);
    HttpResponse::Ok().json("Audio deleted")
}

pub async fn test_answer(
    body: web::Json<TestAnswerBody>,
    db: web::Data<DbPool>,
) -> HttpResponse {
    let db = lock_db(&db);
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

    let db = lock_db(&db);

    // Collect all audios as JSON
    let mut stmt = db_try!(db.prepare("SELECT id, category, answer, video_url, start_time, superflus, count, submitted_by, added_date FROM audios"));
    let audios: Vec<serde_json::Value> = db_try!(stmt.query_map([], |row| {
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
    })).filter_map(|r| r.ok()).collect();

    let json_data = serde_json::to_string_pretty(&audios).unwrap_or_default();

    use std::io::Write;
    let mut buf = Vec::new();
    {
        let mut zip_writer = zip::ZipWriter::new(std::io::Cursor::new(&mut buf));
        let options = zip::write::SimpleFileOptions::default();
        let zipped = zip_writer
            .start_file("audios.json", options)
            .and_then(|_| zip_writer.write_all(json_data.as_bytes()).map_err(Into::into))
            .and_then(|_| zip_writer.finish().map(|_| ()));
        if let Err(e) = zipped {
            log::error!("Failed to build backup archive: {}", e);
            return HttpResponse::InternalServerError().json("Failed to build backup archive");
        }
    }

    HttpResponse::Ok()
        .content_type("application/zip")
        .append_header(("Content-Disposition", "attachment; filename=backup.zip"))
        .body(buf)
}
