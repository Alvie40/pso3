use poem::{Middleware, Request, Response, Result, Error, Endpoint, http::StatusCode, web::cookie::{Cookie, CookieJar}};
use poem::http::HeaderMap;

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

fn get_token_from_headers(headers: &HeaderMap) -> Option<String> {
    headers.get("Cookie")
        .and_then(|c| c.to_str().ok())
        .and_then(|cookie_str| {
            cookie_str.split(';')
                .find(|s| s.trim().starts_with("token="))
                .map(|s| s.trim()[6..].to_string())
        })
}

#[poem::async_trait]
impl<E: Endpoint<Output = Response>> Endpoint for AdminMiddlewareImpl<E> {
    type Output = Response;

    async fn call(&self, req: Request) -> Result<Self::Output> {
        let token = get_token_from_headers(req.headers());

        match token {
            Some(token_str) => {
                token::validate_token(&token_str)
                    .map_err(|_| Error::from_status(StatusCode::UNAUTHORIZED))?;
                self.0.call(req).await
            }
            None => Err(Error::from_status(StatusCode::UNAUTHORIZED))
        }
    }
}

#[poem::async_trait]
impl<E: Endpoint<Output = Response>> Endpoint for ChatMiddlewareImpl<E> {
    type Output = Response;

    async fn call(&self, req: Request) -> Result<Self::Output> {
        let token = get_token_from_headers(req.headers());

        match token {
            Some(token_str) => match token::validate_token(&token_str) {
                Ok(_) => self.0.call(req).await,
                Err(_) => Ok(Response::builder()
                    .status(StatusCode::FOUND)
                    .header("Location", "/login?error=session_expired")
                    .header("Cache-Control", "no-store, no-cache, must-revalidate")
                    .header("Pragma", "no-cache")
                    .header("Set-Cookie", "token=; Path=/; HttpOnly; SameSite=Lax; Secure; Max-Age=0")
                    .finish())
            },
            None => Ok(Response::builder()
                .status(StatusCode::FOUND)
                .header("Location", "/login")
                .header("Cache-Control", "no-store, no-cache, must-revalidate")
                .header("Pragma", "no-cache")
                .finish())
        }
    }
}