use poem::{
    Request,
    web::Json,
    IntoEndpoint,
    EndpointExt,
    Response,
    endpoint::BoxEndpoint,
};
use tracing::error;
use crate::{
    state::AppState,
    auth::dto::TokenResponse,
};
use serde::{Serialize, Deserialize};
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
enum AuthResponse {
    #[oai(status = 200)]
    Ok,
    #[oai(status = 401)]
    Unauthorized,
    #[oai(status = 500)]
    InternalError,
}

#[derive(ApiResponse)]
enum MessagesResponse {
    #[oai(status = 200)]
    Ok(OpenApiJson<Vec<MessageResponse>>),
    #[oai(status = 401)]
    Unauthorized,
    #[oai(status = 500)]
    InternalError,
}

#[derive(ApiResponse)]
enum MessageCreatedResponse {
    #[oai(status = 200)]
    Ok(OpenApiJson<MessageResponse>),
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

#[derive(Object, Debug, Serialize)]
struct MessageResponse {
    id: String,
    user_id: String,
    texto: String, 
    recebido: bool,
    criado_em: String
}

#[derive(Object, Debug, Serialize, Deserialize)]
struct SendMessageRequest {
    texto: String
}

pub struct Api {
    state: AppState,
}

#[OpenApi]
impl Api {
    pub fn new(state: AppState) -> Self {
        Self { state }
    }

    #[oai(path = "/whatsapp", method = "post", tag = "ApiTags::Messages")]
    async fn send_whatsapp(&self, auth: BearerAuth, message: OpenApiJson<WhatsAppMessage>) -> AuthResponse {
        match crate::auth::jwt::verify_token(&auth.0.token) {
            Ok(_) => AuthResponse::Ok,
            Err(e) => {
                error!(target: "auth", error = ?e, "API authentication error");
                AuthResponse::Unauthorized
            },
        }
    }

    #[oai(path = "/me", method = "get", tag = "ApiTags::Auth")]
    async fn me(&self, auth: BearerAuth) -> AuthResponse {
        match crate::auth::jwt::verify_token(&auth.0.token) {
            Ok(_) => AuthResponse::Ok,
            Err(e) => {
                error!(target: "auth", error = ?e, "API authentication error");
                AuthResponse::Unauthorized
            },
        }
    }

    #[oai(path = "/messages", method = "get", tag = "ApiTags::Messages")]
    async fn list_messages(&self, req: &Request) -> MessagesResponse {
        let token = match crate::auth::get_token_from_cookie(req) {
            Some(token) => token,
            None => return MessagesResponse::Unauthorized,
        };

        match crate::auth::token::validate_token(&token) {
            Ok(_) => {
                match crate::models::mensagens::listar_mensagens(&self.state.db).await {
                    Ok(messages) => {
                        let responses: Vec<MessageResponse> = messages.into_iter()
                            .map(|m| MessageResponse {
                                id: m.id.to_string(),
                                user_id: m.user_id.to_string(),
                                texto: m.texto,
                                recebido: m.recebido,
                                criado_em: m.criado_em.to_rfc3339()
                            })
                            .collect();
                        MessagesResponse::Ok(OpenApiJson(responses))
                    },
                    Err(e) => {
                        error!(target: "messages", error = ?e, "Failed to list messages");
                        MessagesResponse::InternalError
                    }
                }
            },
            Err(e) => {
                error!(target: "auth", error = ?e, "API authentication error");
                MessagesResponse::Unauthorized
            }
        }
    }

    #[oai(path = "/messages", method = "post", tag = "ApiTags::Messages")]
    async fn send_message(&self, req: &Request, payload: OpenApiJson<SendMessageRequest>) -> MessageCreatedResponse {
        let token = match crate::auth::get_token_from_cookie(req) {
            Some(token) => token,
            None => return MessageCreatedResponse::Unauthorized,
        };

        match crate::auth::token::validate_token(&token) {
            Ok(claims) => {
                let user_id = match Uuid::parse_str(&claims.sub) {
                    Ok(id) => id,
                    Err(_) => return MessageCreatedResponse::InternalError,
                };

                match crate::models::mensagens::inserir_mensagem(
                    &self.state.db,
                    user_id,
                    &payload.texto,
                    false
                ).await {
                    Ok(message) => MessageCreatedResponse::Ok(OpenApiJson(MessageResponse {
                        id: message.id.to_string(),
                        user_id: message.user_id.to_string(),
                        texto: message.texto,
                        recebido: message.recebido,
                        criado_em: message.criado_em.to_rfc3339()
                    })),
                    Err(e) => {
                        error!(target: "messages", error = ?e, "Failed to create message");
                        MessageCreatedResponse::InternalError
                    }
                }
            },
            Err(e) => {
                error!(target: "auth", error = ?e, "API authentication error");
                MessageCreatedResponse::Unauthorized
            }
        }
    }
}

impl IntoEndpoint for Api {
    type Endpoint = BoxEndpoint<'static>;

    fn into_endpoint(self) -> Self::Endpoint {
        poem_openapi::OpenApiService::new(self, "PSO3 API", "1.0.0")
            .server("/api")
            .into_endpoint()
            .boxed()
    }
}