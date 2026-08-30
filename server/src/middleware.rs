use actix_web::dev::Payload;
use actix_web::{FromRequest, HttpRequest, HttpResponse, ResponseError};
use std::fmt;
use std::future::{ready, Ready};
use jsonwebtoken::{decode, Algorithm, DecodingKey, Validation};
use serde::{Deserialize, Serialize};
use std::fs;
use std::sync::Arc;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Claims {
    pub sub: String, // user id
    pub email: String,
    pub name: String,
    pub role: String,
    pub exp: usize,
}

#[derive(Clone)]
pub struct AuthState {
    pub decoding_key: Arc<DecodingKey>,
    pub encoding_key: Arc<jsonwebtoken::EncodingKey>,
}

impl AuthState {
    pub fn new() -> Self {
        let public_key = fs::read("secret/public.pem")
            .expect("Could not read secret/public.pem");
        let private_key = fs::read("secret/private.pem")
            .expect("Could not read secret/private.pem");

        AuthState {
            decoding_key: Arc::new(DecodingKey::from_rsa_pem(&public_key)
                .expect("Invalid RSA public key")),
            encoding_key: Arc::new(jsonwebtoken::EncodingKey::from_rsa_pem(&private_key)
                .expect("Invalid RSA private key")),
        }
    }

    pub fn create_token(&self, user_id: &str, email: &str, name: &str, role: &str) -> Result<String, jsonwebtoken::errors::Error> {
        let claims = Claims {
            sub: user_id.to_string(),
            email: email.to_string(),
            name: name.to_string(),
            role: role.to_string(),
            exp: (chrono::Utc::now() + chrono::Duration::days(30)).timestamp() as usize,
        };
        jsonwebtoken::encode(
            &jsonwebtoken::Header::new(Algorithm::RS256),
            &claims,
            &self.encoding_key,
        )
    }

    pub fn verify_token(&self, token: &str) -> Result<Claims, jsonwebtoken::errors::Error> {
        let mut validation = Validation::new(Algorithm::RS256);
        validation.validate_exp = true;
        let token_data = decode::<Claims>(token, &self.decoding_key, &validation)?;
        Ok(token_data.claims)
    }
}

/// Extract claims from the Authorization header.
/// Returns None if not present or invalid.
pub fn extract_claims(req: &actix_web::HttpRequest, auth: &AuthState) -> Option<Claims> {
    let header = req.headers().get("Authorization")?;
    let token_str = header.to_str().ok()?;
    auth.verify_token(token_str).ok()
}

/// Helper to return 401 response
pub fn unauthorized() -> HttpResponse {
    HttpResponse::Unauthorized().json(serde_json::json!({"error": "Unauthorized"}))
}

/// Helper to return 403 response
pub fn forbidden() -> HttpResponse {
    HttpResponse::Forbidden().json(serde_json::json!({"error": "Forbidden"}))
}


/// Why a request was refused. Implements `ResponseError`, so an extractor can
/// reject before the handler body runs and still answer with the same shape the
/// hand-written guards used.
#[derive(Debug)]
pub enum AuthError {
    Unauthorized,
    Forbidden,
}

impl fmt::Display for AuthError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AuthError::Unauthorized => write!(f, "Unauthorized"),
            AuthError::Forbidden => write!(f, "Forbidden"),
        }
    }
}

impl ResponseError for AuthError {
    fn error_response(&self) -> HttpResponse {
        match self {
            AuthError::Unauthorized => unauthorized(),
            AuthError::Forbidden => forbidden(),
        }
    }
}

fn claims_of(req: &HttpRequest) -> Option<Claims> {
    let auth = req.app_data::<actix_web::web::Data<AuthState>>()?;
    extract_claims(req, auth)
}

/// Declaring one of these in a handler's arguments *is* the access check: every
/// handler used to open with the same six-line `match extract_claims` block, and
/// a role could be forgotten silently.
macro_rules! role_extractor {
    ($name:ident, $doc:literal, $check:expr, $err:expr) => {
        #[doc = $doc]
        pub struct $name(pub Claims);

        impl FromRequest for $name {
            type Error = AuthError;
            type Future = Ready<Result<Self, Self::Error>>;

            fn from_request(req: &HttpRequest, _: &mut Payload) -> Self::Future {
                let check: fn(&Claims) -> bool = $check;
                ready(match claims_of(req) {
                    Some(c) if check(&c) => Ok($name(c)),
                    Some(_) => Err($err),
                    None => Err(AuthError::Unauthorized),
                })
            }
        }
    };
}

role_extractor!(Authed, "Any signed-in user.", |_| true, AuthError::Unauthorized);
role_extractor!(
    Contributor,
    "A contributor or an administrator.",
    |c| c.role == "contributor" || c.role == "administrator",
    AuthError::Forbidden
);
role_extractor!(
    Administrator,
    "An administrator only.",
    |c| c.role == "administrator",
    AuthError::Forbidden
);

/// For endpoints that work signed out but behave differently when signed in.
pub struct MaybeAuthed(pub Option<Claims>);

impl FromRequest for MaybeAuthed {
    type Error = AuthError;
    type Future = Ready<Result<Self, Self::Error>>;

    fn from_request(req: &HttpRequest, _: &mut Payload) -> Self::Future {
        ready(Ok(MaybeAuthed(claims_of(req))))
    }
}
