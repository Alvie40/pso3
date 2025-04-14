use crate::auth::dto::Claims;
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation, Algorithm};
use std::env;
use std::fmt;
use tracing::{debug, error};

#[derive(Debug)]
pub enum JwtError {
    MissingEnv(String),
    InvalidToken(String),
}

impl fmt::Display for JwtError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            JwtError::MissingEnv(e) => write!(f, "Missing environment variable: {}", e),
            JwtError::InvalidToken(e) => write!(f, "Invalid token: {}", e),
        }
    }
}

pub fn generate_token(claims: &Claims) -> Result<String, JwtError> {
    let jwt_secret = env::var("JWT_SECRET")
        .map_err(|e| JwtError::MissingEnv(e.to_string()))?;

    let key = EncodingKey::from_secret(jwt_secret.as_bytes());
    encode(&Header::default(), claims, &key)
        .map_err(|e| JwtError::InvalidToken(e.to_string()))
}

pub fn validate_token(token: &str) -> Result<Claims, JwtError> {
    debug!(target: "auth", token_length = token.len(), "Validating token");
    
    let jwt_secret = match env::var("JWT_SECRET") {
        Ok(secret) => {
            debug!(target: "auth", "JWT_SECRET found");
            secret
        },
        Err(e) => {
            error!(target: "auth", error = %e, "JWT_SECRET environment variable not found");
            return Err(JwtError::MissingEnv(e.to_string()));
        }
    };

    let key = DecodingKey::from_secret(jwt_secret.as_bytes());
    let mut validation = Validation::new(Algorithm::HS256);
    validation.validate_exp = true;
    validation.leeway = 60; // Add 60 seconds leeway for clock skew
    
    match decode::<Claims>(token, &key, &validation) {
        Ok(token_data) => {
            debug!(
                target: "auth",
                user = %token_data.claims.sub,
                exp = %token_data.claims.exp,
                current_time = %(chrono::Utc::now().timestamp() as usize),
                "Token validated successfully"
            );
            Ok(token_data.claims)
        }
        Err(e) => {
            error!(target: "auth", error = %e, "Token validation failed");
            Err(JwtError::InvalidToken(e.to_string()))
        }
    }
}