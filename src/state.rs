use sqlx::PgPool;

#[derive(Clone)]
pub struct AppState {
    pub db: PgPool,
}

impl AppState {
    pub fn new(pool: PgPool) -> Self {
        Self { db: pool }
    }
}
