use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: Uuid,
    pub email: String,
    pub password: String,
}

pub fn mock_users() -> Vec<User> {
    vec![
        User {
            id: Uuid::new_v4(),
            email: "admin@demo.com".to_string(),
            password: "123456".to_string(),
        },
        User {
            id: Uuid::new_v4(),
            email: "user@demo.com".to_string(),
            password: "abcdef".to_string(),
        },
    ]
}
