use poem::{FromRequest, Request, RequestBody, Result, Error, http::{header, StatusCode}};
use serde::{Deserialize, Serialize};
use poem_openapi::Object;
use tracing::{debug, warn};

#[derive(Debug, Deserialize, Object)]
pub struct LoginPayload {
    pub email: String,
    pub password: String,
}

#[derive(Debug, Deserialize, Object)]
pub struct RegisterPayload {
    pub nome: String,
    pub cpf: String,
    pub telefone: String,
    pub email: String,
    pub password: String,
    pub categoria: String,
}

#[derive(Debug, Serialize, Deserialize, Object)]
pub struct Claims {
    pub sub: String,
    pub exp: usize,
}

pub struct AuthToken {
    pub claims: Claims,
}

#[derive(Debug, Serialize, Object)]
pub struct TokenResponse {
    pub token: String,
}

#[poem::async_trait]
impl<'a> FromRequest<'a> for AuthToken {
    async fn from_request(req: &'a Request, _body: &mut RequestBody) -> Result<Self> {
        // Try Bearer token first
        if let Some(auth_header) = req.headers().get(header::AUTHORIZATION) {
            if let Ok(auth_str) = auth_header.to_str() {
                if let Some(token) = auth_str.strip_prefix("Bearer ") {
                    debug!(target: "auth", "🔑 Token encontrado no header Authorization");
                    let claims = super::token::validate_token(token)
                        .map_err(|e| {
                            warn!(target: "auth", error = %e, "❌ Falha na validação do Bearer token");
                            Error::from_status(StatusCode::UNAUTHORIZED)
                        })?;
                    return Ok(AuthToken { claims });
                }
            }
        }

        // Fall back to cookie
        let jar = req.cookie();
        let token = jar.get("token")
            .ok_or_else(|| {
                warn!(target: "auth", "❌ Nenhum token encontrado (cookie ou Bearer)");
                Error::from_status(StatusCode::UNAUTHORIZED)
            })?;

        debug!(target: "auth", "🍪 Token encontrado no cookie");
        let claims = super::token::validate_token(token.value().map_err(Error::from)?)
            .map_err(|e| {
                warn!(target: "auth", error = %e, "❌ Falha na validação do token do cookie");
                Error::from_status(StatusCode::UNAUTHORIZED)
            })?;

        Ok(AuthToken { claims })
    }
}