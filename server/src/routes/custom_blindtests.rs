use actix_web::{web, HttpRequest, HttpResponse};
use serde::Deserialize;
use crate::db::{lock_db, DbPool};
use crate::db_try;
use crate::middleware::{AuthState, extract_claims, unauthorized};

#[derive(Deserialize)]
pub struct CreateBody {
    pub name: String,
}

#[derive(Deserialize)]
pub struct UpdateBody {
    pub name: Option<String>,
    pub public: Option<bool>,
    #[serde(rename = "blindtestList")]
    pub blindtest_list: Option<Vec<String>>,
}

pub async fn create(
    req: HttpRequest,
    body: web::Json<CreateBody>,
    db: web::Data<DbPool>,
    auth: web::Data<AuthState>,
) -> HttpResponse {
    let _claims = match extract_claims(&req, &auth) {
        Some(c) => c,
        None => return unauthorized(),
    };

    let id = uuid::Uuid::new_v4().to_string();
    let now = chrono::Utc::now().to_rfc3339();
    let db = lock_db(&db);

    let _ = db.execute(
        "INSERT INTO custom_blindtests (id, name, public, owner_id, added_date, blindtest_list) VALUES (?1, ?2, 0, ?3, ?4, '[]')",
        rusqlite::params![id, body.name, _claims.sub, now],
    );

    HttpResponse::Ok().json(serde_json::json!({"_id": id}))
}

pub async fn get_user_blindtests(
    req: HttpRequest,
    db: web::Data<DbPool>,
    auth: web::Data<AuthState>,
) -> HttpResponse {
    let claims = match extract_claims(&req, &auth) {
        Some(c) => c,
        None => return unauthorized(),
    };

    let db = lock_db(&db);
    let mut stmt = db_try!(db.prepare(
        "SELECT cb.id, cb.name, cb.public, cb.owner_id, cb.added_date, cb.blindtest_list, u.name
         FROM custom_blindtests cb LEFT JOIN users u ON cb.owner_id = u.id
         WHERE cb.owner_id = ?1 ORDER BY cb.added_date DESC"
    ));

    let items: Vec<serde_json::Value> = db_try!(stmt.query_map([&claims.sub], |row| {
        let list_str: String = row.get(5)?;
        let list: Vec<String> = serde_json::from_str(&list_str).unwrap_or_default();
        Ok(serde_json::json!({
            "_id": row.get::<_, String>(0)?,
            "name": row.get::<_, String>(1)?,
            "public": row.get::<_, bool>(2)?,
            "ownerId": row.get::<_, String>(3)?,
            "addedDate": row.get::<_, String>(4)?,
            "blindtestList": list,
            "username": row.get::<_, String>(6).ok(),
        }))
    })).filter_map(|r| r.ok()).collect();

    HttpResponse::Ok().json(items)
}

pub async fn get_public_blindtests(
    db: web::Data<DbPool>,
) -> HttpResponse {
    let db = lock_db(&db);
    let mut stmt = db_try!(db.prepare(
        "SELECT cb.id, cb.name, cb.public, cb.owner_id, cb.added_date, cb.blindtest_list, u.name
         FROM custom_blindtests cb LEFT JOIN users u ON cb.owner_id = u.id
         WHERE cb.public = 1 ORDER BY cb.added_date DESC"
    ));

    let items: Vec<serde_json::Value> = db_try!(stmt.query_map([], |row| {
        let list_str: String = row.get(5)?;
        let list: Vec<String> = serde_json::from_str(&list_str).unwrap_or_default();
        Ok(serde_json::json!({
            "_id": row.get::<_, String>(0)?,
            "name": row.get::<_, String>(1)?,
            "public": row.get::<_, bool>(2)?,
            "ownerId": row.get::<_, String>(3)?,
            "addedDate": row.get::<_, String>(4)?,
            "blindtestList": list,
            "username": row.get::<_, String>(6).ok(),
        }))
    })).filter_map(|r| r.ok()).collect();

    HttpResponse::Ok().json(items)
}

pub async fn get_one(
    req: HttpRequest,
    path: web::Path<String>,
    db: web::Data<DbPool>,
    auth: web::Data<AuthState>,
) -> HttpResponse {
    let id = path.into_inner();
    // A private blindtest is readable by its owner only; it used to be readable by
    // anyone who knew the id.
    let viewer = extract_claims(&req, &auth).map(|c| c.sub).unwrap_or_default();
    let db = lock_db(&db);

    let result = db.query_row(
        "SELECT cb.id, cb.name, cb.public, cb.owner_id, cb.added_date, cb.blindtest_list, u.name
         FROM custom_blindtests cb LEFT JOIN users u ON cb.owner_id = u.id
         WHERE cb.id = ?1 AND (cb.public = 1 OR cb.owner_id = ?2)",
        rusqlite::params![id, viewer],
        |row| {
            let list_str: String = row.get(5)?;
            let list: Vec<String> = serde_json::from_str(&list_str).unwrap_or_default();
            Ok(serde_json::json!({
                "_id": row.get::<_, String>(0)?,
                "name": row.get::<_, String>(1)?,
                "public": row.get::<_, bool>(2)?,
                "ownerId": row.get::<_, String>(3)?,
                "addedDate": row.get::<_, String>(4)?,
                "blindtestList": list,
                "username": row.get::<_, String>(6).ok(),
            }))
        },
    );

    match result {
        Ok(item) => HttpResponse::Ok().json(item),
        Err(_) => HttpResponse::NotFound().json("Not found"),
    }
}

pub async fn update(
    req: HttpRequest,
    path: web::Path<String>,
    body: web::Json<UpdateBody>,
    db: web::Data<DbPool>,
    auth: web::Data<AuthState>,
) -> HttpResponse {
    let claims = match extract_claims(&req, &auth) {
        Some(c) => c,
        None => return unauthorized(),
    };

    let id = path.into_inner();
    let db = lock_db(&db);

    if let Some(ref name) = body.name {
        let _ = db.execute("UPDATE custom_blindtests SET name = ?1 WHERE id = ?2 AND owner_id = ?3",
            rusqlite::params![name, id, claims.sub]);
    }
    if let Some(public) = body.public {
        let _ = db.execute("UPDATE custom_blindtests SET public = ?1 WHERE id = ?2 AND owner_id = ?3",
            rusqlite::params![public, id, claims.sub]);
    }
    if let Some(ref list) = body.blindtest_list {
        let json = serde_json::to_string(list).unwrap_or_default();
        let _ = db.execute("UPDATE custom_blindtests SET blindtest_list = ?1 WHERE id = ?2 AND owner_id = ?3",
            rusqlite::params![json, id, claims.sub]);
    }

    HttpResponse::Ok().json("Updated")
}

pub async fn delete(
    req: HttpRequest,
    path: web::Path<String>,
    db: web::Data<DbPool>,
    auth: web::Data<AuthState>,
) -> HttpResponse {
    let claims = match extract_claims(&req, &auth) {
        Some(c) => c,
        None => return unauthorized(),
    };

    let id = path.into_inner();
    let db = lock_db(&db);
    let _ = db.execute("DELETE FROM custom_blindtests WHERE id = ?1 AND owner_id = ?2",
        rusqlite::params![id, claims.sub]);

    HttpResponse::Ok().json("Deleted")
}

pub async fn get_audio_names(
    req: HttpRequest,
    db: web::Data<DbPool>,
    auth: web::Data<AuthState>,
) -> HttpResponse {
    let _claims = match extract_claims(&req, &auth) {
        Some(c) => c,
        None => return unauthorized(),
    };

    let db = lock_db(&db);
    let mut stmt = db_try!(db.prepare("SELECT id, answer, category FROM audios ORDER BY answer"));

    let items: Vec<serde_json::Value> = db_try!(stmt.query_map([], |row| {
        Ok(serde_json::json!({
            "_id": row.get::<_, String>(0)?,
            "answer": row.get::<_, String>(1)?,
            "category": row.get::<_, String>(2)?,
        }))
    })).filter_map(|r| r.ok()).collect();

    HttpResponse::Ok().json(items)
}
