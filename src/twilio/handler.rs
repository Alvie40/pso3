use poem::{
    handler,
    web::{Form, Data, Json},
    Result,
    Response,
    http::StatusCode,
    Error,
};
use sqlx::PgPool;
use uuid::Uuid;
use serde::Deserialize;
use crate::auth::dto::AuthToken;
use crate::state::AppState;

use super::{
    client::TwilioClient,
    dto::{WebhookRequest, SendMessageRequest},
};

#[derive(Debug, Deserialize)]
pub struct SendWhatsAppRequest {
    pub to: String,
    pub message: String,
}

pub struct TwilioHandler {
    client: TwilioClient,
    pool: PgPool,
}

impl TwilioHandler {
    pub fn new(pool: PgPool) -> anyhow::Result<Self> {
        let client = TwilioClient::new()?;
        Ok(Self { client, pool })
    }

    pub async fn send_whatsapp(&self, user_id: Uuid, to: String, message: String) -> Result<()> {
        // First send the message via Twilio
        self.client
            .send_message(to.clone(), message.clone())
            .await
            .map_err(|e| Error::from_string(e.to_string(), StatusCode::INTERNAL_SERVER_ERROR))?;

        // Then store it in our database
        sqlx::query!(
            r#"
            INSERT INTO mensagens (id, user_id, texto, recebido)
            VALUES ($1, $2, $3, $4)
            "#,
            Uuid::new_v4(),
            user_id,
            message,
            false
        )
        .execute(&self.pool)
        .await
        .map_err(|e| Error::from_string(e.to_string(), StatusCode::INTERNAL_SERVER_ERROR))?;

        Ok(())
    }
}

#[handler]
pub async fn webhook_handler(
    Form(payload): Form<WebhookRequest>,
    Data(pool): Data<&PgPool>,
) -> Result<Response> {
    // Store received message in database
    sqlx::query!(
        r#"
        INSERT INTO mensagens (id, user_id, texto, recebido)
        VALUES ($1, $2, $3, $4)
        "#,
        Uuid::new_v4(),
        Uuid::new_v4(), // Using a new UUID as placeholder for now
        payload.body,
        true
    )
    .execute(pool)
    .await
    .map_err(|e| Error::from_string(e.to_string(), StatusCode::INTERNAL_SERVER_ERROR))?;

    // Return empty TwiML response
    Ok(Response::builder()
        .content_type("text/xml")
        .body("<?xml version=\"1.0\" encoding=\"UTF-8\"?><Response></Response>"))
}

#[handler]
pub async fn send_whatsapp_handler(
    auth: AuthToken,
    state: Data<&AppState>,
    Json(payload): Json<SendWhatsAppRequest>,
) -> Result<Response> {
    let handler = TwilioHandler::new(state.db.clone())
        .map_err(|e| Error::from_string(e.to_string(), StatusCode::INTERNAL_SERVER_ERROR))?;

    // Get user ID from email
    let user = sqlx::query!(
        "SELECT id FROM users WHERE email = $1",
        auth.claims.sub
    )
    .fetch_one(&state.db)
    .await
    .map_err(|e| Error::from_string(e.to_string(), StatusCode::INTERNAL_SERVER_ERROR))?;

    handler.send_whatsapp(user.id, payload.to, payload.message).await?;

    Ok(Response::builder()
        .status(StatusCode::OK)
        .body("Message sent successfully"))
}