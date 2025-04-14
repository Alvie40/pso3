use poem::{Middleware, Request, Response, Result, Error, Endpoint, http::{StatusCode, header}};
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

fn get_token_from_cookie(req: &Request) -> Option<String> {
    req.header(header::COOKIE)
        .and_then(|value| value.as_bytes().get(0..).map(|s| String::from_utf8_lossy(s).into_owned()))
        .and_then(|cookie_str| {
            debug!(target: "auth", cookie = %cookie_str, "🍪 Cookie recebido");
            cookie_str.split(';')
                .find(|s| s.trim().starts_with("token="))
                .map(|s| {
                    let token = s.trim()[6..].to_string();
                    debug!(target: "auth", token_length = token.len(), "🎫 Token extraído do cookie");
                    token
                })
        })
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

    async fn call(&self, req: Request) -> Result<Self::Output> {
        debug!(target: "auth", path = %req.uri().path(), "🔍 Validando acesso ao chat");

        if let Some(token) = get_token_from_cookie(&req) {
            match token::validate_token(&token) {
                Ok(claims) => {
                    debug!(target: "auth", user = %claims.sub, "✅ Acesso ao chat autorizado");
                    self.0.call(req).await
                }
                Err(e) => {
                    warn!(target: "auth", error = %e, "❌ Token inválido ou expirado");
                    Ok(Response::builder()
                        .status(StatusCode::FOUND)
                        .header("Location", "/login?error=session_expired")
                        .header("Set-Cookie", "token=; Path=/; Max-Age=0; HttpOnly; SameSite=Lax; Secure")
                        .header("Cache-Control", "no-store, no-cache, must-revalidate")
                        .header("Pragma", "no-cache")
                        .finish())
                }
            }
        } else {
            warn!(target: "auth", "⚠️ Nenhum token encontrado");
            Ok(Response::builder()
                .status(StatusCode::FOUND)
                .header("Location", "/login")
                .header("Cache-Control", "no-store, no-cache, must-revalidate")
                .header("Pragma", "no-cache")
                .finish())
        }
    }
}