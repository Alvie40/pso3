use poem::{
    handler,
    http::StatusCode,
    web::{Data, Json},
    Result, Error, Response,
};
use tracing::error;
use crate::{
    state::AppState,
    auth::dto::{LoginPayload, TokenResponse},
};
use serde_json::json;
use poem_openapi::{
    payload::Json as OpenApiJson,
    ApiResponse,
    Object,
    OpenApi,
    SecurityScheme,
    Tags,
    auth::Bearer,
};
use uuid::Uuid;
use crate::twilio::handler::TwilioHandler;

#[derive(SecurityScheme)]
#[oai(type = "bearer")]
struct BearerAuth(Bearer);

#[derive(Tags)]
enum ApiTags {
    Auth,
    Messages,
}

#[derive(Object, Debug)]
struct WhatsAppMessage {
    to: String,
    message: String,
}

#[derive(ApiResponse)]
enum MessageResponse {
    #[oai(status = 200)]
    Ok,
    #[oai(status = 401)]
    Unauthorized,
    #[oai(status = 500)]
    InternalError,
}

#[derive(Debug)]
pub enum LoginResponse {
    Ok(Json<TokenResponse>),
    InvalidCredentials,
    Error(String),
}

fn handle_auth_error(e: impl std::fmt::Debug) -> Error {
    error!(target: "auth", error = ?e, "API authentication error");
    Error::from_status(StatusCode::UNAUTHORIZED)
}

#[handler]
pub async fn login(
    state: Data<&AppState>,
    Json(payload): Json<LoginPayload>,
) -> Result<Response> {
    let user = sqlx::query!(
        r#"
        SELECT email, senha FROM users 
        WHERE email = $1 AND senha = $2
        "#,
        payload.email,
        payload.password
    )
    .fetch_optional(&state.db)
    .await
    .map_err(|e| Error::from_string(e.to_string(), StatusCode::INTERNAL_SERVER_ERROR))?;

    match user {
        Some(_) => {
            let claims = crate::auth::dto::Claims {
                sub: payload.email,
                exp: (chrono::Utc::now().timestamp() + 3600) as usize,
            };

            match crate::auth::token::generate_token(&claims) {
                Ok(token) => Ok(Response::builder()
                    .content_type("application/json")
                    .body(serde_json::to_string(&json!({
                        "token": token
                    })).unwrap())),
                Err(e) => Err(Error::from_string(e.to_string(), StatusCode::INTERNAL_SERVER_ERROR))
            }
        }
        None => Ok(Response::builder()
            .status(StatusCode::UNAUTHORIZED)
            .finish()),
    }
}

pub struct Api {
    state: AppState,
}

#[OpenApi]
impl Api {
    #[oai(path = "/whatsapp", method = "post", tag = "ApiTags::Messages")]
    async fn send_whatsapp(&self, auth: BearerAuth, message: OpenApiJson<WhatsAppMessage>) -> MessageResponse {
        let claims = match crate::auth::jwt::verify_token(&auth.0.token) {
            Ok(claims) => claims,
            Err(e) => {
                handle_auth_error(e);
                return MessageResponse::Unauthorized;
            },
        };

        let handler = match TwilioHandler::new(self.state.db.clone()) {
            Ok(handler) => handler,
            Err(_) => return MessageResponse::InternalError,
        };

        let user_id = match Uuid::parse_str(&claims.sub) {
            Ok(id) => id,
            Err(_) => return MessageResponse::InternalError,
        };

        match handler.send_whatsapp(user_id, message.0.to, message.0.message).await {
            Ok(_) => MessageResponse::Ok,
            Err(_) => MessageResponse::InternalError,
        }
    }

    #[oai(path = "/me", method = "get", tag = "ApiTags::Auth")]
    async fn me(&self, auth: BearerAuth) -> MessageResponse {
        match crate::auth::jwt::verify_token(&auth.0.token) {
            Ok(_) => MessageResponse::Ok,
            Err(e) => {
                handle_auth_error(e);
                MessageResponse::Unauthorized
            },
        }
    }
}

impl Api {
    pub fn new(state: AppState) -> Self {
        Self { state }
    }
}