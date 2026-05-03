use actix_web::{web, HttpRequest, HttpResponse};
use serde::Deserialize;
use crate::db::DbPool;
use crate::middleware::{AuthState, extract_claims, unauthorized};

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
pub struct ChangePasswordBody {
    pub password: String,
    pub token: String,
}

#[derive(Deserialize)]
pub struct ResetPasswordBody {
    pub email: String,
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
    let db = db.lock().unwrap();
    let result = db.query_row(
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
    );

    match result {
        Ok((id, email, name, hash, role, deleted)) => {
            if deleted {
                return HttpResponse::Unauthorized().json("Account deleted");
            }
            match bcrypt::verify(&body.password, &hash) {
                Ok(true) => {
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

    let hash = match bcrypt::hash(&body.password, 10) {
        Ok(h) => h,
        Err(_) => return HttpResponse::InternalServerError().json("Hash error"),
    };

    let id = uuid::Uuid::new_v4().to_string();
    let now = chrono::Utc::now().to_rfc3339();

    let db = db.lock().unwrap();
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
    let db = db.lock().unwrap();
    let result = db.execute(
        "UPDATE users SET email_confirmed = 1, email_confirmation_token = NULL WHERE email_confirmation_token = ?1",
        [&body.token],
    );
    match result {
        Ok(n) if n > 0 => HttpResponse::Ok().json("Email confirmed"),
        _ => HttpResponse::BadRequest().json("Invalid token"),
    }
}

pub async fn reset_password(
    body: web::Json<ResetPasswordBody>,
    _db: web::Data<DbPool>,
) -> HttpResponse {
    // Email sending is discarded — stub endpoint
    HttpResponse::Ok().json("If this email exists, a reset link would be sent")
}

pub async fn change_password(
    body: web::Json<ChangePasswordBody>,
    db: web::Data<DbPool>,
) -> HttpResponse {
    if body.password.len() < 6 {
        return HttpResponse::BadRequest().json("Password must be at least 6 characters");
    }
    let hash = match bcrypt::hash(&body.password, 10) {
        Ok(h) => h,
        Err(_) => return HttpResponse::InternalServerError().json("Hash error"),
    };

    let db = db.lock().unwrap();
    let result = db.execute(
        "UPDATE users SET password = ?1, reset_password_token = NULL WHERE reset_password_token = ?2",
        rusqlite::params![hash, body.token],
    );
    match result {
        Ok(n) if n > 0 => HttpResponse::Ok().json("Password changed"),
        _ => HttpResponse::BadRequest().json("Invalid or expired token"),
    }
}
