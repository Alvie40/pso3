use poem::{FromRequest, Request, RequestBody, Result, Error, http::{header, StatusCode}};
use std::env;
use std::fmt;
use jsonwebtoken::{decode, DecodingKey, Validation, Algorithm};
use tracing::{debug, error, warn};
use crate::auth::dto::Claims;

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

impl std::error::Error for JwtError {}

impl From<JwtError> for Error {
    fn from(err: JwtError) -> Self {
        Error::from_string(err.to_string(), StatusCode::UNAUTHORIZED)
    }
}

#[derive(Debug)]
pub struct JwtToken {
    pub claims: Claims,
}

#[poem::async_trait]
impl<'a> FromRequest<'a> for JwtToken {
    async fn from_request(req: &'a Request, _body: &mut RequestBody) -> Result<Self> {
        debug!(target: "auth", path = %req.uri().path(), "Validating JWT token");
            
        let header_value = req
            .headers()
            .get(header::AUTHORIZATION)
            .ok_or_else(|| {
                warn!(target: "auth", path = %req.uri().path(), "No authorization header found");
                Error::from_status(StatusCode::UNAUTHORIZED)
            })?;

        let header_str = header_value
            .to_str()
            .map_err(|_| {
                error!(target: "auth", path = %req.uri().path(), "Invalid authorization header format");
                Error::from_status(StatusCode::UNAUTHORIZED)
            })?;

        let token = header_str
            .strip_prefix("Bearer ")
            .ok_or_else(|| {
                warn!(target: "auth", path = %req.uri().path(), "Authorization header missing Bearer prefix");
                Error::from_status(StatusCode::UNAUTHORIZED)
            })?;

        let claims = verify_token(token)
            .map_err(|e| {
                error!(target: "auth", path = %req.uri().path(), error = %e, "Token validation failed");
                Error::from(e)
            })?;

        debug!(target: "auth", path = %req.uri().path(), user = %claims.sub, "Token validated successfully");
        Ok(JwtToken { claims })
    }
}

pub fn verify_token(token: &str) -> Result<Claims, JwtError> {
    let jwt_secret = env::var("JWT_SECRET")
        .map_err(|e| JwtError::MissingEnv(e.to_string()))?;

    let key = DecodingKey::from_secret(jwt_secret.as_bytes());
    let validation = Validation::new(Algorithm::HS256);
    
    decode::<Claims>(token, &key, &validation)
        .map(|data| data.claims)
        .map_err(|e| JwtError::InvalidToken(e.to_string()))
}
