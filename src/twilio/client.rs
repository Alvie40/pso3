use anyhow::Result;
use base64::{engine::general_purpose::STANDARD, Engine};
use reqwest::Client;
use std::env;

use super::dto::SendMessageRequest;

pub struct TwilioClient {
    account_sid: String,
    auth_token: String,
    client: Client,
    from_number: String,
}

impl TwilioClient {
    pub fn new() -> Result<Self> {
        let account_sid = env::var("TWILIO_ACCOUNT_SID")?;
        let auth_token = env::var("TWILIO_AUTH_TOKEN")?;
        let from_number = env::var("TWILIO_FROM_NUMBER")?;

        Ok(Self {
            account_sid,
            auth_token,
            client: Client::new(),
            from_number,
        })
    }

    pub async fn send_message(&self, to: String, body: String) -> Result<()> {
        let url = format!(
            "https://api.twilio.com/2010-04-01/Accounts/{}/Messages.json",
            self.account_sid
        );

        let auth = format!("{}:{}", self.account_sid, self.auth_token);
        let auth_header = format!("Basic {}", STANDARD.encode(auth));

        let request = SendMessageRequest {
            to,
            from: self.from_number.clone(),
            body,
        };

        let response = self
            .client
            .post(&url)
            .header("Authorization", auth_header)
            .form(&request)
            .send()
            .await?;

        if !response.status().is_success() {
            let error_text = response.text().await?;
            anyhow::bail!("Failed to send message: {}", error_text);
        }

        Ok(())
    }
}