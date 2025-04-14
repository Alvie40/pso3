use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Empresa {
    pub id: Uuid,
    pub nome: String,
    pub cnpj: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

pub fn mock_empresas() -> Vec<Empresa> {
    let now = Utc::now();
    vec![
        Empresa {
            id: Uuid::new_v4(),
            nome: "Demo Empresa 1".to_string(),
            cnpj: "12.345.678/0001-90".to_string(),
            created_at: now,
            updated_at: now,
        },
        Empresa {
            id: Uuid::new_v4(),
            nome: "Demo Empresa 2".to_string(),
            cnpj: "98.765.432/0001-09".to_string(),
            created_at: now,
            updated_at: now,
        },
    ]
}