use poem::{FromRequest, Request, RequestBody, Result, Error, http::StatusCode};
use serde::{Deserialize, Serialize};
use poem_openapi::Object;

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
        let jar = req.cookie();
        let token = jar.get("token")
            .ok_or_else(|| Error::from_status(StatusCode::UNAUTHORIZED))?;

        let claims = super::token::validate_token(token.value().map_err(Error::from)?)
            .map_err(|_| Error::from_status(StatusCode::UNAUTHORIZED))?;

        Ok(AuthToken { claims })
    }
}