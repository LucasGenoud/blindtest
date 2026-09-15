use actix_web::{web, HttpResponse};
use serde::Deserialize;
use crate::db::{lock_db, DbPool};
use crate::db_try;
use crate::middleware::{Administrator, Authed, Contributor};

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
    user: Authed,
    db: web::Data<DbPool>,
) -> HttpResponse {
    let claims = user.0;

    let db = lock_db(&db);
    let result = db.query_row(
        "SELECT id, email, name, role, register_date FROM users WHERE id = ?1 AND deleted = 0",
        [&claims.sub],
        |row| {
            Ok(serde_json::json!({
                "_id": row.get::<_, String>(0)?,
                "email": row.get::<_, String>(1)?,
                "name": row.get::<_, String>(2)?,
                "role": row.get::<_, String>(3)?,
                "registerDate": row.get::<_, String>(4)?,
            }))
        },
    );

    match result {
        Ok(user) => HttpResponse::Ok().json(user),
        Err(_) => HttpResponse::NotFound().json("User not found"),
    }
}

pub async fn get_users(
    _user: Contributor,
    db: web::Data<DbPool>,
) -> HttpResponse {
    let db = lock_db(&db);
    let mut stmt = db_try!(db.prepare(
        "SELECT id, email, name, role, register_date, deleted FROM users ORDER BY register_date DESC"
    ));

    let users: Vec<serde_json::Value> = db_try!(stmt.query_map([], |row| {
        Ok(serde_json::json!({
            "_id": row.get::<_, String>(0)?,
            "email": row.get::<_, String>(1)?,
            "name": row.get::<_, String>(2)?,
            "role": row.get::<_, String>(3)?,
            "registerDate": row.get::<_, String>(4)?,
            "deleted": row.get::<_, bool>(5)?,
        }))
    })).filter_map(|r| r.ok()).collect();

    HttpResponse::Ok().json(users)
}

pub async fn get_contributor_users(
    db: web::Data<DbPool>,
) -> HttpResponse {
    let db = lock_db(&db);
    let mut stmt = db_try!(db.prepare(
        "SELECT id, name FROM users WHERE (role = 'contributor' OR role = 'administrator') AND deleted = 0 ORDER BY name"
    ));

    let users: Vec<serde_json::Value> = db_try!(stmt.query_map([], |row| {
        Ok(serde_json::json!({
            "_id": row.get::<_, String>(0)?,
            "name": row.get::<_, String>(1)?,
        }))
    })).filter_map(|r| r.ok()).collect();

    HttpResponse::Ok().json(users)
}

pub async fn update_user(
    _user: Administrator,
    body: web::Json<UpdateUserBody>,
    db: web::Data<DbPool>,
) -> HttpResponse {
    let db = lock_db(&db);
    if let Some(ref role) = body.role {
        // Anything else silently creates a role that matches no permission check.
        if !matches!(role.as_str(), "user" | "contributor" | "administrator") {
            return HttpResponse::BadRequest().json("Unknown role");
        }
        let _ = db.execute("UPDATE users SET role = ?1 WHERE id = ?2", rusqlite::params![role, body.id]);
    }
    if let Some(deleted) = body.deleted {
        let _ = db.execute("UPDATE users SET deleted = ?1 WHERE id = ?2", rusqlite::params![deleted, body.id]);
    }

    HttpResponse::Ok().json("User updated")
}

pub async fn delete_user(
    _user: Administrator,
    query: web::Query<DeleteUserQuery>,
    db: web::Data<DbPool>,
) -> HttpResponse {
    let db = lock_db(&db);
    let _ = db.execute("UPDATE users SET deleted = 1 WHERE id = ?1", [&query.id]);
    HttpResponse::Ok().json("User deleted")
}
