use actix_web::{web, HttpResponse};
use serde::Deserialize;
use crate::db::{lock_db, DbPool};
use crate::middleware::AuthState;

#[derive(Deserialize)]
pub struct SigninBody {
    pub email: String,
    pub password: String,
}

#[derive(Deserialize)]
pub struct SignupBody {
    pub email: String,
    pub password: String,
    pub name: String,
}

#[derive(Deserialize)]
pub struct ConfirmEmailBody {
    pub token: String,
}

pub async fn signin(
    body: web::Json<SigninBody>,
    db: web::Data<DbPool>,
    auth: web::Data<AuthState>,
) -> HttpResponse {
    // Scoped: bcrypt below takes ~100 ms, and holding the connection guard through it
    // blocked every other database route for the duration of each sign-in.
    let result = {
        let db = lock_db(&db);
        db.query_row(
            "SELECT id, email, name, password, role, deleted FROM users WHERE LOWER(email) = LOWER(?1)",
            [&body.email],
            |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                    row.get::<_, String>(3)?,
                    row.get::<_, String>(4)?,
                    row.get::<_, bool>(5)?,
                ))
            },
        )
    };

    match result {
        Ok((id, email, name, hash, role, deleted)) => {
            if deleted {
                return HttpResponse::Unauthorized().json("Account deleted");
            }
            // Cost-10 bcrypt is pure CPU; on an actix worker thread it stalls every
            // other connection that worker is serving.
            let password = body.password.clone();
            let verified = web::block(move || bcrypt::verify(&password, &hash)).await;
            match verified {
                Ok(Ok(true)) => {
                    match auth.create_token(&id, &email, &name, &role) {
                        Ok(token) => HttpResponse::Ok().json(serde_json::json!({
                            "token": token,
                            "user": {
                                "_id": id,
                                "email": email,
                                "name": name,
                                "role": role,
                            }
                        })),
                        Err(_) => HttpResponse::InternalServerError().json("Token creation failed"),
                    }
                }
                _ => HttpResponse::Unauthorized().json("Invalid credentials"),
            }
        }
        Err(_) => HttpResponse::Unauthorized().json("Invalid credentials"),
    }
}

pub async fn signup(
    body: web::Json<SignupBody>,
    db: web::Data<DbPool>,
    auth: web::Data<AuthState>,
) -> HttpResponse {
    if body.password.len() < 6 {
        return HttpResponse::BadRequest().json("Password must be at least 6 characters");
    }
    if body.name.is_empty() {
        return HttpResponse::BadRequest().json("Name is required");
    }

    let password = body.password.clone();
    let hash = match web::block(move || bcrypt::hash(&password, 10)).await {
        Ok(Ok(h)) => h,
        _ => return HttpResponse::InternalServerError().json("Hash error"),
    };

    let id = uuid::Uuid::new_v4().to_string();
    let now = chrono::Utc::now().to_rfc3339();

    let db = lock_db(&db);
    let result = db.execute(
        "INSERT INTO users (id, email, name, password, role, email_confirmed, register_date) VALUES (?1, ?2, ?3, ?4, 'user', 1, ?5)",
        rusqlite::params![id, body.email.to_lowercase(), body.name, hash, now],
    );

    match result {
        Ok(_) => {
            // Email confirmation is discarded; auto-confirm
            HttpResponse::Ok().json(serde_json::json!({"message": "User created"}))
        }
        Err(e) => {
            let msg = e.to_string();
            if msg.contains("UNIQUE") {
                HttpResponse::Conflict().json("Email or username already taken")
            } else {
                HttpResponse::InternalServerError().json(msg)
            }
        }
    }
}

pub async fn confirm_email(
    body: web::Json<ConfirmEmailBody>,
    db: web::Data<DbPool>,
) -> HttpResponse {
    // Email sending is discarded, but keep endpoint for compatibility
    let db = lock_db(&db);
    let result = db.execute(
        "UPDATE users SET email_confirmed = 1, email_confirmation_token = NULL WHERE email_confirmation_token = ?1",
        [&body.token],
    );
    match result {
        Ok(n) if n > 0 => HttpResponse::Ok().json("Email confirmed"),
        _ => HttpResponse::BadRequest().json("Invalid token"),
    }
}
