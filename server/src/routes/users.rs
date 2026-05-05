use actix_web::{web, HttpRequest, HttpResponse};
use serde::Deserialize;
use crate::db::DbPool;
use crate::middleware::{AuthState, extract_claims, unauthorized, forbidden};

#[derive(Deserialize)]
pub struct UpdateProfileBody {
    pub name: Option<String>,
    pub clear_mode: Option<bool>,
    pub hide_carousel: Option<bool>,
}

#[derive(Deserialize)]
pub struct UpdateUserBody {
    pub id: String,
    pub role: Option<String>,
    pub deleted: Option<bool>,
}

#[derive(Deserialize)]
pub struct DeleteUserQuery {
    pub id: String,
}

pub async fn get_user(
    req: HttpRequest,
    db: web::Data<DbPool>,
    auth: web::Data<AuthState>,
) -> HttpResponse {
    let claims = match extract_claims(&req, &auth) {
        Some(c) => c,
        None => return unauthorized(),
    };

    let db = db.lock().unwrap();
    let result = db.query_row(
        "SELECT id, email, name, role, clear_mode, hide_carousel, email_confirmed, register_date FROM users WHERE id = ?1 AND deleted = 0",
        [&claims.sub],
        |row| {
            Ok(serde_json::json!({
                "_id": row.get::<_, String>(0)?,
                "email": row.get::<_, String>(1)?,
                "name": row.get::<_, String>(2)?,
                "role": row.get::<_, String>(3)?,
                "clearMode": row.get::<_, bool>(4)?,
                "hideCarousel": row.get::<_, bool>(5)?,
                "emailConfirmed": row.get::<_, bool>(6)?,
                "registerDate": row.get::<_, String>(7)?,
            }))
        },
    );

    match result {
        Ok(user) => HttpResponse::Ok().json(user),
        Err(_) => HttpResponse::NotFound().json("User not found"),
    }
}

pub async fn get_users(
    req: HttpRequest,
    db: web::Data<DbPool>,
    auth: web::Data<AuthState>,
) -> HttpResponse {
    let claims = match extract_claims(&req, &auth) {
        Some(c) if c.role == "contributor" || c.role == "administrator" => c,
        _ => return unauthorized(),
    };

    let db = db.lock().unwrap();
    let mut stmt = db.prepare(
        "SELECT id, email, name, role, email_confirmed, register_date, deleted FROM users ORDER BY register_date DESC"
    ).unwrap();

    let users: Vec<serde_json::Value> = stmt.query_map([], |row| {
        Ok(serde_json::json!({
            "_id": row.get::<_, String>(0)?,
            "email": row.get::<_, String>(1)?,
            "name": row.get::<_, String>(2)?,
            "role": row.get::<_, String>(3)?,
            "emailConfirmed": row.get::<_, bool>(4)?,
            "registerDate": row.get::<_, String>(5)?,
            "deleted": row.get::<_, bool>(6)?,
        }))
    }).unwrap().filter_map(|r| r.ok()).collect();

    HttpResponse::Ok().json(users)
}

pub async fn get_contributor_users(
    db: web::Data<DbPool>,
) -> HttpResponse {
    let db = db.lock().unwrap();
    let mut stmt = db.prepare(
        "SELECT id, name FROM users WHERE (role = 'contributor' OR role = 'administrator') AND deleted = 0 ORDER BY name"
    ).unwrap();

    let users: Vec<serde_json::Value> = stmt.query_map([], |row| {
        Ok(serde_json::json!({
            "_id": row.get::<_, String>(0)?,
            "name": row.get::<_, String>(1)?,
        }))
    }).unwrap().filter_map(|r| r.ok()).collect();

    HttpResponse::Ok().json(users)
}

pub async fn update_profile(
    req: HttpRequest,
    body: web::Json<UpdateProfileBody>,
    db: web::Data<DbPool>,
    auth: web::Data<AuthState>,
) -> HttpResponse {
    let claims = match extract_claims(&req, &auth) {
        Some(c) => c,
        None => return unauthorized(),
    };

    let db = db.lock().unwrap();
    if let Some(ref name) = body.name {
        let _ = db.execute("UPDATE users SET name = ?1 WHERE id = ?2", rusqlite::params![name, claims.sub]);
    }
    if let Some(clear_mode) = body.clear_mode {
        let _ = db.execute("UPDATE users SET clear_mode = ?1 WHERE id = ?2", rusqlite::params![clear_mode, claims.sub]);
    }
    if let Some(hide_carousel) = body.hide_carousel {
        let _ = db.execute("UPDATE users SET hide_carousel = ?1 WHERE id = ?2", rusqlite::params![hide_carousel, claims.sub]);
    }

    HttpResponse::Ok().json("Profile updated")
}

pub async fn update_user(
    req: HttpRequest,
    body: web::Json<UpdateUserBody>,
    db: web::Data<DbPool>,
    auth: web::Data<AuthState>,
) -> HttpResponse {
    let _claims = match extract_claims(&req, &auth) {
        Some(c) if c.role == "administrator" => c,
        _ => return forbidden(),
    };

    let db = db.lock().unwrap();
    if let Some(ref role) = body.role {
        let _ = db.execute("UPDATE users SET role = ?1 WHERE id = ?2", rusqlite::params![role, body.id]);
    }
    if let Some(deleted) = body.deleted {
        let _ = db.execute("UPDATE users SET deleted = ?1 WHERE id = ?2", rusqlite::params![deleted, body.id]);
    }

    HttpResponse::Ok().json("User updated")
}

pub async fn delete_user(
    req: HttpRequest,
    query: web::Query<DeleteUserQuery>,
    db: web::Data<DbPool>,
    auth: web::Data<AuthState>,
) -> HttpResponse {
    let _claims = match extract_claims(&req, &auth) {
        Some(c) if c.role == "administrator" => c,
        _ => return forbidden(),
    };

    let db = db.lock().unwrap();
    let _ = db.execute("UPDATE users SET deleted = 1 WHERE id = ?1", [&query.id]);
    HttpResponse::Ok().json("User deleted")
}

pub async fn get_user_profile(
    req: HttpRequest,
    db: web::Data<DbPool>,
    auth: web::Data<AuthState>,
) -> HttpResponse {
    let claims = match extract_claims(&req, &auth) {
        Some(c) => c,
        None => return unauthorized(),
    };

    let db = db.lock().unwrap();
    let result = db.query_row(
        "SELECT id, name, clear_mode, hide_carousel FROM users WHERE id = ?1",
        [&claims.sub],
        |row| {
            Ok(serde_json::json!({
                "_id": row.get::<_, String>(0)?,
                "name": row.get::<_, String>(1)?,
                "clearMode": row.get::<_, bool>(2)?,
                "hideCarousel": row.get::<_, bool>(3)?,
            }))
        },
    );

    match result {
        Ok(user) => HttpResponse::Ok().json(user),
        Err(_) => HttpResponse::NotFound().json("User not found"),
    }
}
