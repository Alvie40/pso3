use serde::{Serialize, Deserialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct User {
    pub id: Uuid,
    pub empresa_id: Uuid,
    pub nome: String,
    pub cpf: String,
    pub telefone: String,
    pub email: String,
    #[serde(skip_serializing)]
    #[sqlx(rename = "senha")]
    pub password: String,
    pub categoria: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

pub fn mock_users() -> Vec<User> {
    let empresa_id = Uuid::new_v4();
    vec![
        User {
            id: Uuid::new_v4(),
            empresa_id,
            nome: "João Silva".to_string(),
            cpf: "123.456.789-00".to_string(),
            telefone: "(11) 99999-9999".to_string(),
            email: "joao@example.com".to_string(),
            password: "senha123".to_string(),
            categoria: "user".to_string(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        },
        User {
            id: Uuid::new_v4(),
            empresa_id,
            nome: "Maria Oliveira".to_string(),
            cpf: "987.654.321-00".to_string(),
            telefone: "(11) 88888-8888".to_string(),
            email: "maria@example.com".to_string(),
            password: "senha456".to_string(),
            categoria: "admin".to_string(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        },
    ]
}