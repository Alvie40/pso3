use poem::{Middleware, Request, Response, Result, Error, Endpoint, http::StatusCode};
use tracing::{debug, warn};

pub mod dto;
pub mod handler;
pub mod jwt;
pub mod token;

pub struct AdminMiddleware;
pub struct ChatMiddleware;

impl<E: Endpoint<Output = Response>> Middleware<E> for AdminMiddleware {
    type Output = AdminMiddlewareImpl<E>;

    fn transform(&self, ep: E) -> Self::Output {
        AdminMiddlewareImpl(ep)
    }
}

impl<E: Endpoint<Output = Response>> Middleware<E> for ChatMiddleware {
    type Output = ChatMiddlewareImpl<E>;

    fn transform(&self, ep: E) -> Self::Output {
        ChatMiddlewareImpl(ep)
    }
}

pub struct AdminMiddlewareImpl<E>(E);
pub struct ChatMiddlewareImpl<E>(E);

pub fn get_token_from_cookie(req: &Request) -> Option<String> {
    req.cookie()
        .get("token")
        .and_then(|c| c.value().ok().map(|v: &str| v.to_string()))
}

#[poem::async_trait]
impl<E: Endpoint<Output = Response>> Endpoint for AdminMiddlewareImpl<E> {
    type Output = Response;

    async fn call(&self, req: Request) -> Result<Self::Output> {
        if let Some(token) = get_token_from_cookie(&req) {
            match token::validate_token(&token) {
                Ok(_) => self.0.call(req).await,
                Err(e) => {
                    warn!(target: "auth", error = %e, "❌ Falha na validação do token admin");
                    Err(Error::from_status(StatusCode::UNAUTHORIZED))
                }
            }
        } else {
            warn!(target: "auth", "⚠️ Token admin não encontrado");
            Err(Error::from_status(StatusCode::UNAUTHORIZED))
        }
    }
}

#[poem::async_trait]
impl<E: Endpoint<Output = Response>> Endpoint for ChatMiddlewareImpl<E> {
    type Output = Response;

    async fn call(&self, req: Request) -> Result<Self::Output, poem::Error> {
        debug!(target: "auth", path = %req.uri().path(), "🔍 Validando acesso ao chat");
        
        match get_token_from_cookie(&req) {
            Some(token) => {
                match token::validate_token(&token) {
                    Ok(_) => self.0.call(req).await,
                    Err(e) => {
                        warn!(target: "auth", error = %e, "❌ Token inválido");
                        Ok(Response::builder()
                            .status(StatusCode::FOUND)
                            .header("Location", "/login")
                            .header(
                                "Set-Cookie",
                                "token=; Path=/; HttpOnly; SameSite=Strict; Secure; Max-Age=0"
                            )
                            .finish())
                    }
                }
            }
            None => {
                warn!(target: "auth", "⚠️ Nenhum token encontrado");
                Ok(Response::builder()
                    .status(StatusCode::FOUND)
                    .header("Location", "/login")
                    .header(
                        "Set-Cookie",
                        "token=; Path=/; HttpOnly; SameSite=Strict; Secure; Max-Age=0"
                    )
                    .finish())
            }
        }
    }
}