use poem::{
    handler,
    http::StatusCode,
    web::{Data, Form},
    Result, Error, Response, Request,
};
use serde::{Deserialize, Serialize};
use crate::state::AppState;
use uuid::Uuid;
use chrono::Utc;
use crate::auth::token::validate_token;
use tracing::error;

#[derive(Debug, Serialize, Deserialize)]
pub struct EmpresaPayload {
    pub nome: String,
    pub cnpj: String,
}

#[derive(Debug, Serialize)]
pub struct EmpresaResponse {
    pub id: Uuid,
    pub nome: String,
    pub cnpj: String,
}

pub async fn validate_admin_token(token: &str) -> Result<(), Error> {
    validate_token(token)
        .map(|_| ())
        .map_err(|e| {
            error!(target: "auth", error = ?e, "Admin token validation failed");
            Error::from_status(StatusCode::UNAUTHORIZED)
        })
}

async fn check_admin_token(req: &Request) -> Result<(), Error> {
    let jar = req.cookie();
    let token = jar.get("token")
        .ok_or_else(|| Error::from_status(StatusCode::UNAUTHORIZED))?;

    validate_admin_token(token.value().map_err(Error::from)?).await
}

#[derive(Debug, serde::Deserialize)]
pub struct CreateEmpresaPayload {
    pub nome: String,
    pub cnpj: String,
}

#[handler]
pub async fn list_empresas(state: Data<&AppState>) -> Result<Response> {
    let empresas = sqlx::query_as!(
        EmpresaResponse,
        r#"
        SELECT id, nome, cnpj
        FROM empresas
        ORDER BY nome
        "#
    )
    .fetch_all(&state.db)
    .await
    .map_err(|e| Error::from_string(e.to_string(), StatusCode::INTERNAL_SERVER_ERROR))?;

    Ok(Response::builder()
        .content_type("application/json")
        .body(serde_json::to_string(&empresas).unwrap()))
}

#[poem::handler]
pub async fn create_empresa(
    state: Data<&AppState>,
    Form(payload): Form<CreateEmpresaPayload>,
    req: &Request,
) -> Result<Response> {
    check_admin_token(req).await?;

    // Check if CNPJ already exists
    let existing = sqlx::query!(
        "SELECT id FROM empresas WHERE cnpj = $1",
        payload.cnpj
    )
    .fetch_optional(&state.db)
    .await
    .map_err(|e| Error::from_string(e.to_string(), StatusCode::INTERNAL_SERVER_ERROR))?;

    if existing.is_some() {
        return Ok(Response::builder()
            .status(StatusCode::CONFLICT)
            .content_type("text/html")
            .body("CNPJ já cadastrado."));
    }

    let empresa_id = Uuid::new_v4();
    let now = Utc::now();

    sqlx::query!(
        r#"
        INSERT INTO empresas (id, nome, cnpj, created_at, updated_at)
        VALUES ($1, $2, $3, $4, $5)
        "#,
        empresa_id,
        payload.nome,
        payload.cnpj,
        now,
        now
    )
    .execute(&state.db)
    .await
    .map_err(|e| Error::from_string(e.to_string(), StatusCode::INTERNAL_SERVER_ERROR))?;

    Ok(Response::builder()
        .status(StatusCode::FOUND)
        .header("Location", "/admin")
        .body("Empresa cadastrada com sucesso."))
}

#[handler]
pub async fn delete_empresa(
    state: Data<&AppState>,
    id: poem::web::Path<Uuid>,
) -> Result<Response> {
    // Check if there are any users associated with this company
    let has_users = sqlx::query!(
        "SELECT COUNT(*) as count FROM users WHERE empresa_id = $1",
        id.0
    )
    .fetch_one(&state.db)
    .await
    .map_err(|e| Error::from_string(e.to_string(), StatusCode::INTERNAL_SERVER_ERROR))?
    .count
    .unwrap_or(0)
        > 0;

    if has_users {
        return Ok(Response::builder()
            .status(StatusCode::BAD_REQUEST)
            .body("Não é possível excluir uma empresa que possui usuários"));
    }

    sqlx::query!(
        "DELETE FROM empresas WHERE id = $1",
        id.0
    )
    .execute(&state.db)
    .await
    .map_err(|e| Error::from_string(e.to_string(), StatusCode::INTERNAL_SERVER_ERROR))?;

    Ok(Response::builder()
        .status(StatusCode::NO_CONTENT)
        .finish())
}

#[handler]
pub async fn list_usuarios(state: Data<&AppState>) -> Result<Response> {
    let users = sqlx::query!(
        r#"
        SELECT u.id, u.nome, u.email, u.categoria, e.nome as empresa_nome
        FROM users u
        LEFT JOIN empresas e ON u.empresa_id = e.id
        ORDER BY u.nome
        "#
    )
    .fetch_all(&state.db)
    .await
    .map_err(|e| Error::from_string(e.to_string(), StatusCode::INTERNAL_SERVER_ERROR))?;

    let response = users.into_iter().map(|u| serde_json::json!({
        "id": u.id,
        "nome": u.nome,
        "email": u.email,
        "categoria": u.categoria,
        "empresa": u.empresa_nome
    })).collect::<Vec<_>>();

    Ok(Response::builder()
        .content_type("application/json")
        .body(serde_json::to_string(&response).unwrap()))
}

#[handler]
pub async fn delete_usuario(
    state: Data<&AppState>,
    id: poem::web::Path<Uuid>,
    req: &Request,
) -> Result<Response> {
    let jar = req.cookie();
    let token = jar.get("token")
        .ok_or_else(|| Error::from_status(StatusCode::UNAUTHORIZED))?;

    let token_str = token.value()
        .map_err(|_| Error::from_status(StatusCode::UNAUTHORIZED))?;

    let claims = crate::auth::token::validate_token(token_str)
        .map_err(|_| Error::from_status(StatusCode::UNAUTHORIZED))?;

    // Get the current user's ID from their email
    let current_user = sqlx::query!(
        "SELECT id FROM users WHERE email = $1",
        claims.sub
    )
    .fetch_one(&state.db)
    .await
    .map_err(|e| Error::from_string(e.to_string(), StatusCode::INTERNAL_SERVER_ERROR))?;

    // Prevent self-deletion
    if current_user.id == id.0 {
        return Ok(Response::builder()
            .status(StatusCode::BAD_REQUEST)
            .body("Não é possível excluir o próprio usuário"));
    }

    sqlx::query!(
        "DELETE FROM users WHERE id = $1",
        id.0
    )
    .execute(&state.db)
    .await
    .map_err(|e| Error::from_string(e.to_string(), StatusCode::INTERNAL_SERVER_ERROR))?;

    Ok(Response::builder()
        .status(StatusCode::NO_CONTENT)
        .finish())
}