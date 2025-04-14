use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize)]
pub struct SendMessageRequest {
    #[serde(rename = "To")]
    pub to: String,
    #[serde(rename = "From")]
    pub from: String,
    #[serde(rename = "Body")]
    pub body: String,
}

#[derive(Debug, Deserialize)]
pub struct WebhookRequest {
    #[serde(rename = "From")]
    pub from: String,
    #[serde(rename = "Body")]
    pub body: String,
    #[serde(rename = "MessageSid")]
    pub message_sid: String,
}

#[derive(Debug, Serialize)]
pub struct TwiMLResponse {
    pub message: Option<String>,
}