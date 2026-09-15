use crate::db::{lock_db, DbPool};
use crate::db_try;
use crate::middleware::{extract_current_claims, AuthState, Authed};
use actix_web::{web, HttpRequest, HttpResponse};
use serde::Deserialize;

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
    user: Authed,
    body: web::Json<CreateBody>,
    db: web::Data<DbPool>,
) -> HttpResponse {
    let owner = user.0;
    let id = uuid::Uuid::new_v4().to_string();
    let now = chrono::Utc::now().to_rfc3339();
    let db = lock_db(&db);

    db_try!(db.execute(
        "INSERT INTO custom_blindtests (id, name, public, owner_id, added_date, blindtest_list) VALUES (?1, ?2, 0, ?3, ?4, '[]')",
        rusqlite::params![id, body.name, owner.sub, now],
    ));

    HttpResponse::Ok().json(serde_json::json!({"_id": id}))
}

pub async fn get_user_blindtests(user: Authed, db: web::Data<DbPool>) -> HttpResponse {
    let claims = user.0;

    let db = lock_db(&db);
    let mut stmt = db_try!(db.prepare(
        "SELECT cb.id, cb.name, cb.public, cb.owner_id, cb.added_date, cb.blindtest_list, u.name
         FROM custom_blindtests cb LEFT JOIN users u ON cb.owner_id = u.id
         WHERE cb.owner_id = ?1 ORDER BY cb.added_date DESC"
    ));

    let rows = db_try!(stmt.query_map([&claims.sub], |row| {
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
    }));
    let items = db_try!(rows.collect::<rusqlite::Result<Vec<_>>>());

    HttpResponse::Ok().json(items)
}

pub async fn get_public_blindtests(db: web::Data<DbPool>) -> HttpResponse {
    let db = lock_db(&db);
    let mut stmt = db_try!(db.prepare(
        "SELECT cb.id, cb.name, cb.public, cb.owner_id, cb.added_date, cb.blindtest_list, u.name
         FROM custom_blindtests cb LEFT JOIN users u ON cb.owner_id = u.id
         WHERE cb.public = 1 ORDER BY cb.added_date DESC"
    ));

    let rows = db_try!(stmt.query_map([], |row| {
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
    }));
    let items = db_try!(rows.collect::<rusqlite::Result<Vec<_>>>());

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
    let viewer = extract_current_claims(&req, &auth, &db)
        .map(|c| c.sub)
        .unwrap_or_default();
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
    user: Authed,
    path: web::Path<String>,
    body: web::Json<UpdateBody>,
    db: web::Data<DbPool>,
) -> HttpResponse {
    let claims = user.0;

    let id = path.into_inner();
    let db = lock_db(&db);

    let list = match body.blindtest_list.as_ref() {
        Some(list) => Some(db_try!(serde_json::to_string(list))),
        None => None,
    };
    let changed = db_try!(db.execute(
        "UPDATE custom_blindtests SET name = COALESCE(?1, name), public = COALESCE(?2, public), blindtest_list = COALESCE(?3, blindtest_list) WHERE id = ?4 AND owner_id = ?5",
        rusqlite::params![body.name, body.public, list, id, claims.sub],
    ));
    if changed == 0 {
        return HttpResponse::NotFound().json("Not found");
    }

    HttpResponse::Ok().json("Updated")
}

pub async fn delete(user: Authed, path: web::Path<String>, db: web::Data<DbPool>) -> HttpResponse {
    let claims = user.0;

    let id = path.into_inner();
    let db = lock_db(&db);
    let changed = db_try!(db.execute(
        "DELETE FROM custom_blindtests WHERE id = ?1 AND owner_id = ?2",
        rusqlite::params![id, claims.sub]
    ));
    if changed == 0 {
        return HttpResponse::NotFound().json("Not found");
    }

    HttpResponse::Ok().json("Deleted")
}

pub async fn get_audio_names(_user: Authed, db: web::Data<DbPool>) -> HttpResponse {
    let db = lock_db(&db);
    let mut stmt = db_try!(db.prepare(
        "SELECT id, answer, category FROM audios WHERE processing_status = 'ready' AND id NOT IN (SELECT audio_id FROM flagged_audios WHERE auto = 0) ORDER BY answer"
    ));

    let rows = db_try!(stmt.query_map([], |row| {
        Ok(serde_json::json!({
            "_id": row.get::<_, String>(0)?,
            "answer": row.get::<_, String>(1)?,
            "category": row.get::<_, String>(2)?,
        }))
    }));
    let items = db_try!(rows.collect::<rusqlite::Result<Vec<_>>>());

    HttpResponse::Ok().json(items)
}
