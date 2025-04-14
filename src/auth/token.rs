use crate::auth::dto::Claims;
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation, Algorithm};
use std::env;
use std::fmt;
use tracing::{debug, error, warn};

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
    debug!(target: "auth", token_length = token.len(), "🔐 Iniciando validação do token");
    
    let jwt_secret = match env::var("JWT_SECRET") {
        Ok(secret) => {
            debug!(target: "auth", "✅ JWT_SECRET encontrada");
            secret
        },
        Err(e) => {
            error!(target: "auth", error = %e, "❌ JWT_SECRET não encontrada");
            return Err(JwtError::MissingEnv(e.to_string()));
        }
    };

    let key = DecodingKey::from_secret(jwt_secret.as_bytes());
    let mut validation = Validation::new(Algorithm::HS256);
    validation.validate_exp = true;
    validation.leeway = 120; // Aumentando para 2 minutos de tolerância

    debug!(target: "auth", leeway = validation.leeway, "⚙️ Configuração de validação");
    
    match decode::<Claims>(token, &key, &validation) {
        Ok(token_data) => {
            let current_time = chrono::Utc::now().timestamp() as usize;
            debug!(
                target: "auth",
                user = %token_data.claims.sub,
                exp = %token_data.claims.exp,
                current_time = %current_time,
                diff = %(token_data.claims.exp as i64 - current_time as i64),
                "✅ Token validado com sucesso"
            );
            Ok(token_data.claims)
        }
        Err(e) => {
            warn!(target: "auth", error = %e, "❌ Falha na validação do token");
            Err(JwtError::InvalidToken(e.to_string()))
        }
    }
}