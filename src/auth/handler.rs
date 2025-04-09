use poem::{handler, http::StatusCode, Request, Result};
use crate::auth::jwt::validate_token;

#[handler]
pub async fn login() -> &'static str {
    "Login endpoint funcionando!"
}

#[handler]
pub async fn me(req: &Request) -> Result<String> {
    let token = req
        .headers()
        .get("Authorization")
        .and_then(|v| v.to_str().ok());

    match token {
        Some(token) => match validate_token(token) {
            Ok(claims) => Ok(format!("Usuário autenticado: {}", claims.sub)),
            Err(_) => Err(StatusCode::UNAUTHORIZED.into()),
        },
        None => Err(StatusCode::UNAUTHORIZED.into()),
    }
}
