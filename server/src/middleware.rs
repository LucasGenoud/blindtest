use actix_web::HttpResponse;
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
