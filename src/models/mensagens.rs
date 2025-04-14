use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};
use sqlx::PgPool;
use anyhow::Result;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Mensagem {
    pub id: Uuid,
    pub user_id: Uuid,
    pub texto: String,
    pub recebido: bool,
    pub criado_em: DateTime<Utc>,
}

pub fn mock_mensagens() -> Vec<Mensagem> {
    let now = Utc::now();
    let users = crate::models::users::mock_users();
    vec![
        Mensagem {
            id: Uuid::new_v4(),
            user_id: users[0].id,
            texto: "Olá, esta é uma mensagem de teste!".to_string(),
            recebido: false,
            criado_em: now,
        },
        Mensagem {
            id: Uuid::new_v4(),
            user_id: users[1].id,
            texto: "Outra mensagem aqui.".to_string(),
            recebido: true,
            criado_em: now,
        },
    ]
}

pub async fn listar_mensagens(pool: &PgPool) -> Result<Vec<Mensagem>> {
    let mensagens = sqlx::query_as!(
        Mensagem,
        r#"
        SELECT id, user_id, texto, recebido, criado_em
        FROM mensagens
        ORDER BY criado_em DESC
        "#
    )
    .fetch_all(pool)
    .await?;

    Ok(mensagens)
}

pub async fn inserir_mensagem(
    pool: &PgPool,
    user_id: Uuid,
    texto: &str,
    recebido: bool,
) -> Result<Mensagem> {
    let mensagem = sqlx::query_as!(
        Mensagem,
        r#"
        INSERT INTO mensagens (id, user_id, texto, recebido, criado_em)
        VALUES ($1, $2, $3, $4, $5)
        RETURNING id, user_id, texto, recebido, criado_em
        "#,
        Uuid::new_v4(),
        user_id,
        texto,
        recebido,
        Utc::now()
    )
    .fetch_one(pool)
    .await?;

    Ok(mensagem)
}

#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::postgres::PgPoolOptions;
    use dotenvy::dotenv;

    #[test]
    fn test_create_mensagem() {
        let id = Uuid::new_v4();
        let user_id = Uuid::new_v4();
        let now = Utc::now();

        let mensagem = Mensagem {
            id,
            user_id,
            texto: "Test message".to_string(),
            recebido: false,
            criado_em: now,
        };

        assert_eq!(mensagem.id, id);
        assert_eq!(mensagem.user_id, user_id);
        assert_eq!(mensagem.texto, "Test message");
        assert!(!mensagem.recebido);
        assert_eq!(mensagem.criado_em, now);
    }

    #[test]
    fn test_mock_mensagens() {
        let mensagens = mock_mensagens();
        
        assert_eq!(mensagens.len(), 2);
        assert!(!mensagens[0].recebido);
        assert!(mensagens[1].recebido);
        assert_eq!(mensagens[0].texto, "Olá, esta é uma mensagem de teste!");
        assert_eq!(mensagens[1].texto, "Outra mensagem aqui.");
    }

    #[tokio::test]
    async fn test_listar_mensagens() {
        dotenv().ok();
        
        let database_url = std::env::var("DATABASE_URL")
            .expect("DATABASE_URL must be set in .env");

        let pool = PgPoolOptions::new()
            .max_connections(1)
            .connect(&database_url)
            .await
            .expect("Failed to connect to Postgres");

        // Start a transaction
        let mut tx = pool.begin().await.expect("Failed to start transaction");

        // Create test data
        let empresa_id = Uuid::new_v4();
        let user_id = Uuid::new_v4();
        let mensagem_id = Uuid::new_v4();
        let now = Utc::now();

        // First create the empresas table and insert a test empresa
        sqlx::query!(
            r#"
            CREATE TABLE IF NOT EXISTS empresas (
                id UUID PRIMARY KEY,
                nome TEXT NOT NULL,
                cnpj TEXT NOT NULL UNIQUE,
                criado_em TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP
            )
            "#
        )
        .execute(&mut *tx)
        .await
        .expect("Failed to create empresas table");

        sqlx::query!(
            r#"
            INSERT INTO empresas (id, nome, cnpj)
            VALUES ($1, $2, $3)
            "#,
            empresa_id,
            "Test Company",
            "12345678901234"
        )
        .execute(&mut *tx)
        .await
        .expect("Failed to insert test empresa");

        // Create users table and insert test user
        sqlx::query!(
            r#"
            CREATE TABLE IF NOT EXISTS users (
                id UUID PRIMARY KEY,
                empresa_id UUID NOT NULL REFERENCES empresas(id),
                nome TEXT NOT NULL,
                cpf TEXT NOT NULL UNIQUE,
                telefone TEXT NOT NULL,
                email TEXT NOT NULL UNIQUE,
                senha TEXT NOT NULL,
                categoria TEXT NOT NULL,
                criado_em TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP
            )
            "#
        )
        .execute(&mut *tx)
        .await
        .expect("Failed to create users table");

        sqlx::query!(
            r#"
            INSERT INTO users (id, empresa_id, nome, cpf, telefone, email, senha, categoria)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            "#,
            user_id,
            empresa_id,
            "Test User",
            "12345678901",
            "11999999999",
            "test@example.com",
            "password123",
            "USER"
        )
        .execute(&mut *tx)
        .await
        .expect("Failed to insert test user");

        // Create mensagens table with foreign key constraint
        sqlx::query!(
            r#"
            CREATE TABLE IF NOT EXISTS mensagens (
                id UUID PRIMARY KEY,
                user_id UUID NOT NULL REFERENCES users(id),
                texto TEXT NOT NULL,
                recebido BOOLEAN NOT NULL DEFAULT FALSE,
                criado_em TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP
            )
            "#
        )
        .execute(&mut *tx)
        .await
        .expect("Failed to create mensagens table");

        // Insert test message
        sqlx::query!(
            r#"
            INSERT INTO mensagens (id, user_id, texto, recebido, criado_em)
            VALUES ($1, $2, $3, $4, $5)
            "#,
            mensagem_id,
            user_id,
            "Test message",
            false,
            now
        )
        .execute(&mut *tx)
        .await
        .expect("Failed to insert test message");

        // Test listar_mensagens using the transaction
        let mensagens = listar_mensagens(&pool).await.expect("Failed to list messages");
        
        assert!(!mensagens.is_empty());
        assert_eq!(mensagens.len(), 1);
        
        let mensagem = &mensagens[0];
        assert_eq!(mensagem.id, mensagem_id);
        assert_eq!(mensagem.user_id, user_id);
        assert_eq!(mensagem.texto, "Test message");
        assert!(!mensagem.recebido);
        
        let diff = (mensagem.criado_em - now).num_seconds().abs();
        assert!(diff <= 1);

        // Rollback the transaction
        tx.rollback().await.expect("Failed to rollback transaction");
    }

    #[tokio::test]
    async fn test_empty_mensagens() {
        dotenv().ok();
        
        let database_url = std::env::var("DATABASE_URL")
            .expect("DATABASE_URL must be set in .env");

        let pool = PgPoolOptions::new()
            .max_connections(1)
            .connect(&database_url)
            .await
            .expect("Failed to connect to Postgres");

        let mut tx = pool.begin().await.expect("Failed to start transaction");

        // Create empty mensagens table
        sqlx::query!(
            r#"
            CREATE TABLE IF NOT EXISTS mensagens (
                id UUID PRIMARY KEY,
                user_id UUID NOT NULL,
                texto TEXT NOT NULL,
                recebido BOOLEAN NOT NULL DEFAULT FALSE,
                criado_em TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP
            )
            "#
        )
        .execute(&mut *tx)
        .await
        .expect("Failed to create mensagens table");

        let mensagens = listar_mensagens(&pool).await.expect("Failed to list messages");
        assert!(mensagens.is_empty());

        tx.rollback().await.expect("Failed to rollback transaction");
    }

    #[tokio::test]
    async fn test_inserir_mensagem() {
        dotenv().ok();
        
        let database_url = std::env::var("DATABASE_URL")
            .expect("DATABASE_URL must be set in .env");

        let pool = PgPoolOptions::new()
            .max_connections(1)
            .connect(&database_url)
            .await
            .expect("Failed to connect to Postgres");

        let mut tx = pool.begin().await.expect("Failed to start transaction");

        // Setup test data
        let empresa_id = Uuid::new_v4();
        let user_id = Uuid::new_v4();
        let mensagem_id = Uuid::new_v4();
        let test_texto = "Mensagem de teste para inserção";
        
        // Create empresas table and insert test empresa
        sqlx::query!(
            r#"
            CREATE TABLE IF NOT EXISTS empresas (
                id UUID PRIMARY KEY,
                nome TEXT NOT NULL,
                cnpj TEXT NOT NULL UNIQUE,
                criado_em TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP
            )
            "#
        )
        .execute(&mut *tx)
        .await
        .expect("Failed to create empresas table");

        sqlx::query!(
            r#"
            INSERT INTO empresas (id, nome, cnpj)
            VALUES ($1, $2, $3)
            "#,
            empresa_id,
            "Test Company",
            "12345678901234"
        )
        .execute(&mut *tx)
        .await
        .expect("Failed to insert test empresa");

        // Create users table and insert test user
        sqlx::query!(
            r#"
            CREATE TABLE IF NOT EXISTS users (
                id UUID PRIMARY KEY,
                empresa_id UUID NOT NULL REFERENCES empresas(id),
                nome TEXT NOT NULL,
                cpf TEXT NOT NULL UNIQUE,
                telefone TEXT NOT NULL,
                email TEXT NOT NULL UNIQUE,
                senha TEXT NOT NULL,
                categoria TEXT NOT NULL,
                criado_em TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP
            )
            "#
        )
        .execute(&mut *tx)
        .await
        .expect("Failed to create users table");

        sqlx::query!(
            r#"
            INSERT INTO users (id, empresa_id, nome, cpf, telefone, email, senha, categoria)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            "#,
            user_id,
            empresa_id,
            "Test User",
            "12345678901",
            "11999999999",
            "test@example.com",
            "password123",
            "USER"
        )
        .execute(&mut *tx)
        .await
        .expect("Failed to insert test user");
        
        // Create mensagens table
        sqlx::query!(
            r#"
            CREATE TABLE IF NOT EXISTS mensagens (
                id UUID PRIMARY KEY,
                user_id UUID NOT NULL REFERENCES users(id),
                texto TEXT NOT NULL,
                recebido BOOLEAN NOT NULL DEFAULT FALSE,
                criado_em TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP
            )
            "#
        )
        .execute(&mut *tx)
        .await
        .expect("Failed to create mensagens table");

        // Insert test message
        sqlx::query!(
            r#"
            INSERT INTO mensagens (id, user_id, texto, recebido)
            VALUES ($1, $2, $3, $4)
            "#,
            mensagem_id,
            user_id,
            test_texto,
            false
        )
        .execute(&mut *tx)
        .await
        .expect("Failed to insert test message");

        let mensagens = listar_mensagens(&pool).await.expect("Failed to list messages");
        assert_eq!(mensagens.len(), 1);
        assert_eq!(mensagens[0].texto, test_texto);
        assert_eq!(mensagens[0].id, mensagem_id);
        assert_eq!(mensagens[0].user_id, user_id);
        assert!(!mensagens[0].recebido);

        tx.rollback().await.expect("Failed to rollback transaction");
    }

    #[tokio::test]
    async fn test_atualizar_mensagem() {
        dotenv().ok();
        
        let database_url = std::env::var("DATABASE_URL")
            .expect("DATABASE_URL must be set in .env");

        let pool = PgPoolOptions::new()
            .max_connections(1)
            .connect(&database_url)
            .await
            .expect("Failed to connect to Postgres");

        let mut tx = pool.begin().await.expect("Failed to start transaction");

        // Setup test data
        let empresa_id = Uuid::new_v4();
        let user_id = Uuid::new_v4();
        let mensagem_id = Uuid::new_v4();
        let texto_original = "Mensagem original";
        let texto_atualizado = "Mensagem atualizada";

        // Create empresas table and insert test empresa
        sqlx::query!(
            r#"
            CREATE TABLE IF NOT EXISTS empresas (
                id UUID PRIMARY KEY,
                nome TEXT NOT NULL,
                cnpj TEXT NOT NULL UNIQUE,
                criado_em TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP
            )
            "#
        )
        .execute(&mut *tx)
        .await
        .expect("Failed to create empresas table");

        sqlx::query!(
            r#"
            INSERT INTO empresas (id, nome, cnpj)
            VALUES ($1, $2, $3)
            "#,
            empresa_id,
            "Test Company",
            "12345678901234"
        )
        .execute(&mut *tx)
        .await
        .expect("Failed to insert test empresa");

        // Create users table and insert test user
        sqlx::query!(
            r#"
            CREATE TABLE IF NOT EXISTS users (
                id UUID PRIMARY KEY,
                empresa_id UUID NOT NULL REFERENCES empresas(id),
                nome TEXT NOT NULL,
                cpf TEXT NOT NULL UNIQUE,
                telefone TEXT NOT NULL,
                email TEXT NOT NULL UNIQUE,
                senha TEXT NOT NULL,
                categoria TEXT NOT NULL,
                criado_em TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP
            )
            "#
        )
        .execute(&mut *tx)
        .await
        .expect("Failed to create users table");

        sqlx::query!(
            r#"
            INSERT INTO users (id, empresa_id, nome, cpf, telefone, email, senha, categoria)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            "#,
            user_id,
            empresa_id,
            "Test User",
            "12345678901",
            "11999999999",
            "test@example.com",
            "password123",
            "USER"
        )
        .execute(&mut *tx)
        .await
        .expect("Failed to insert test user");

        // Create mensagens table
        sqlx::query!(
            r#"
            CREATE TABLE IF NOT EXISTS mensagens (
                id UUID PRIMARY KEY,
                user_id UUID NOT NULL REFERENCES users(id),
                texto TEXT NOT NULL,
                recebido BOOLEAN NOT NULL DEFAULT FALSE,
                criado_em TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP
            )
            "#
        )
        .execute(&mut *tx)
        .await
        .expect("Failed to create mensagens table");

        // Insert initial message
        sqlx::query!(
            r#"
            INSERT INTO mensagens (id, user_id, texto, recebido)
            VALUES ($1, $2, $3, $4)
            "#,
            mensagem_id,
            user_id,
            texto_original,
            false
        )
        .execute(&mut *tx)
        .await
        .expect("Failed to insert test message");

        // Update the message
        sqlx::query!(
            r#"
            UPDATE mensagens
            SET texto = $1, recebido = true
            WHERE id = $2
            "#,
            texto_atualizado,
            mensagem_id
        )
        .execute(&mut *tx)
        .await
        .expect("Failed to update test message");

        let mensagens = listar_mensagens(&pool).await.expect("Failed to list messages");
        assert_eq!(mensagens.len(), 1);
        assert_eq!(mensagens[0].texto, texto_atualizado);
        assert_eq!(mensagens[0].id, mensagem_id);
        assert!(mensagens[0].recebido);

        tx.rollback().await.expect("Failed to rollback transaction");
    }
}