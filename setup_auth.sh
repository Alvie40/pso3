#!/bin/bash

echo "🔧 Criando estrutura do módulo auth..."

mkdir -p src/auth

# auth/mod.rs
cat <<EOF > src/auth/mod.rs
pub mod handler;
pub mod jwt;
pub mod dto;
EOF

# auth/dto.rs
cat <<EOF > src/auth/dto.rs
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub struct LoginPayload {
    pub email: String,
    pub password: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,
    pub exp: usize,
}
EOF

# auth/jwt.rs
cat <<EOF > src/auth/jwt.rs
use jsonwebtoken::{encode, decode, Header, Validation, EncodingKey, DecodingKey, errors::Result};
use crate::auth::dto::Claims;

const SECRET: &[u8] = b"supersecret";

pub fn generate_token(user_id: &str) -> Result<String> {
    let claims = Claims {
        sub: user_id.to_string(),
        exp: (chrono::Utc::now().timestamp() + 3600) as usize,
    };
    encode(&Header::default(), &claims, &EncodingKey::from_secret(SECRET))
}

pub fn validate_token(token: &str) -> Result<Claims> {
    decode::<Claims>(token, &DecodingKey::from_secret(SECRET), &Validation::default())
        .map(|data| data.claims)
}
EOF

# auth/handler.rs
cat <<EOF > src/auth/handler.rs
use poem::{
    handler,
    web::{Json, Data},
    Route, post,
};
use crate::auth::dto::LoginPayload;
use crate::auth::jwt::generate_token;
use crate::models::user::User;

#[handler]
pub fn login(users: Data<&Vec<User>>, Json(payload): Json<LoginPayload>) -> Json<serde_json::Value> {
    let user = users.iter().find(|u| u.email == payload.email && u.password == payload.password);

    if let Some(u) = user {
        if let Ok(token) = generate_token(&u.id) {
            return Json(serde_json::json!({ "token": token }));
        }
    }

    Json(serde_json::json!({ "error": "Invalid credentials" }))
}

pub fn auth_routes(users: Vec<User>) -> Route {
    Route::new()
        .at("/login", post(login.data(users)))
}
EOF

echo "✅ Módulo auth criado com sucesso!"
