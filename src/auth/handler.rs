use poem::{
    handler,
    web::{Form, Data, Json},
    Response,
    http::StatusCode,
};
use sqlx::error::Error as SqlxError;
use tracing::{debug, warn, error, info};
use uuid::Uuid;
use chrono::Utc;

use crate::state::AppState;
use super::dto::{Claims, LoginPayload, RegisterPayload, AuthToken};
use super::token::{generate_token, validate_token};

#[handler]
pub async fn logout() -> Response {
    Response::builder()
        .status(StatusCode::FOUND)
        .header("Location", "/login")
        .header("Set-Cookie", "token=; Path=/; HttpOnly; SameSite=Lax; Secure; Max-Age=0")
        .body(String::new())
}

#[handler]
pub async fn login(
    state: Data<&AppState>,
    Form(payload): Form<LoginPayload>,
) -> Response {
    debug!(target: "auth", email = %payload.email, "Login attempt");

    let user = sqlx::query_as::<_, crate::models::users::User>(
        "SELECT * FROM users WHERE email = $1 AND senha = $2"
    )
    .bind(&payload.email)
    .bind(&payload.password)
    .fetch_one(&state.db)
    .await;

    match user {
        Ok(u) => {
            let claims = Claims {
                sub: u.email.clone(),
                exp: (chrono::Utc::now().timestamp() + 3600) as usize, // 1 hour
            };
            
            match generate_token(&claims) {
                Ok(token) => {
                    info!(target: "auth", email = %u.email, "Login successful");
                    Response::builder()
                        .status(StatusCode::FOUND)
                        .header("Location", "/chat")
                        .header("Cache-Control", "no-cache, no-store, must-revalidate")
                        .header("Pragma", "no-cache")
                        .header("Expires", "0")
                        .header("Set-Cookie", format!("token={}; Path=/; HttpOnly; SameSite=Lax; Secure; Max-Age=3600", token))
                        .finish()
                },
                Err(e) => {
                    error!(target: "auth", email = %u.email, error = %e, "Token generation failed");
                    Response::builder()
                        .status(StatusCode::FOUND)
                        .header("Location", "/login?error=token")
                        .header("Cache-Control", "no-store")
                        .finish()
                }
            }
        },
        Err(e) => {
            warn!(target: "auth", email = %payload.email, error = %e, "Login failed");
            Response::builder()
                .status(StatusCode::FOUND)
                .header("Location", "/login?error=credentials")
                .header("Cache-Control", "no-store")
                .finish()
        }
    }
}

#[handler]
pub async fn me(token: AuthToken) -> Response {
    debug!(target: "auth", user = %token.claims.sub, "🔍 Verificando identidade do usuário");
    
    Response::builder()
        .content_type("application/json; charset=utf-8")
        .body(serde_json::to_string(&token.claims).unwrap())
}

#[handler]
pub async fn api_login(
    state: Data<&AppState>,
    Json(payload): Json<LoginPayload>,
) -> Response {
    let user = sqlx::query_as::<_, crate::models::users::User>(
        "SELECT * FROM users WHERE email = $1 AND senha = $2"
    )
    .bind(&payload.email)
    .bind(&payload.password)
    .fetch_one(&state.db)
    .await;

    match user {
        Ok(u) => {
            let claims = Claims {
                sub: u.email.clone(),
                exp: (chrono::Utc::now().timestamp() + 3600) as usize, // 1 hour
            };
            
            match generate_token(&claims) {
                Ok(token) => Response::builder()
                    .status(StatusCode::OK)
                    .header("Authorization", format!("Bearer {}", token))
                    .body(serde_json::to_string(&claims).unwrap()),
                Err(e) => {
                    error!(target: "auth", email = %u.email, error = %e, "Token generation failed");
                    Response::builder()
                        .status(StatusCode::INTERNAL_SERVER_ERROR)
                        .content_type("text/plain; charset=utf-8")
                        .body(e.to_string())
                }
            }
        }
        Err(SqlxError::RowNotFound) => {
            warn!(target: "auth", email = %payload.email, "API login failed - invalid credentials");
            Response::builder()
                .status(StatusCode::UNAUTHORIZED)
                .content_type("text/plain; charset=utf-8")
                .body("Invalid credentials")
        }
        Err(e) => {
            error!(target: "auth", email = %payload.email, error = %e, "API login failed - database error");
            Response::builder()
                .status(StatusCode::INTERNAL_SERVER_ERROR)
                .content_type("text/plain; charset=utf-8")
                .body(e.to_string())
        }
    }
}

#[handler]
pub async fn register(
    state: Data<&AppState>,
    Form(payload): Form<RegisterPayload>,
) -> Response {
    let existing_user = match sqlx::query!(
        r#"
        SELECT id FROM users 
        WHERE email = $1 OR cpf = $2
        "#,
        payload.email,
        payload.cpf
    )
    .fetch_optional(&state.db)
    .await {
        Ok(user) => user,
        Err(e) => return Response::builder()
            .status(StatusCode::INTERNAL_SERVER_ERROR)
            .content_type("text/plain; charset=utf-8")
            .body(e.to_string())
    };

    if existing_user.is_some() {
        return Response::builder()
            .status(StatusCode::CONFLICT)
            .content_type("text/html; charset=utf-8")
            .body(format!(r#"
                <div class='text-red-600 p-4 mb-4 bg-red-50 rounded'>
                    Email ou CPF já cadastrado. Por favor, tente novamente.
                </div>
                {}
            "#, std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/src/templates/cadastro.html")).unwrap_or_default()));
    }

    // Get the first company (temporary solution)
    let empresa = match sqlx::query!(
        "SELECT id FROM empresas LIMIT 1"
    )
    .fetch_optional(&state.db)
    .await {
        Ok(e) => e,
        Err(e) => return Response::builder()
            .status(StatusCode::INTERNAL_SERVER_ERROR)
            .content_type("text/plain; charset=utf-8")
            .body(e.to_string())
    };

    let empresa_id = empresa
        .map(|e| e.id)
        .unwrap_or_else(|| Uuid::new_v4()); // Fallback to new UUID if no company exists

    // Insert new user
    let user_id = Uuid::new_v4();
    let now = Utc::now();

    match sqlx::query!(
        r#"
        INSERT INTO users (id, empresa_id, nome, cpf, telefone, email, senha, categoria, created_at, updated_at)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
        "#,
        user_id,
        empresa_id,
        payload.nome,
        payload.cpf,
        payload.telefone,
        payload.email,
        payload.password,  // Changed from payload.senha
        payload.categoria,
        now,
        now
    )
    .execute(&state.db)
    .await {
        Ok(_) => Response::builder()
            .status(StatusCode::FOUND)
            .header("Location", "/login")
            .content_type("text/plain; charset=utf-8")
            .body("Usuário cadastrado com sucesso. Redirecionando para o login..."),
        Err(e) => Response::builder()
            .status(StatusCode::INTERNAL_SERVER_ERROR)
            .content_type("text/plain; charset=utf-8")
            .body(e.to_string())
    }
}