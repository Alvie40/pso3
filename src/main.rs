use dotenvy::dotenv;
use sqlx::postgres::PgPoolOptions;
use poem::{
    listener::TcpListener,
    middleware::AddData,
    EndpointExt,
    Route,
    Server,
    Error,
    web::Html,
    endpoint::StaticFilesEndpoint,
};
use poem_openapi::OpenApiService;
use psocial3::{routes, api::Api, state::AppState, logging::{setup_logging, LogMiddleware}, templates::pages};
use tracing::{info, error, debug};
use poem::middleware::CookieJarManager;

async fn not_found_handler(_: Error) -> Html<&'static str> {
    pages::not_found_page()
}

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    dotenv().ok();
    setup_logging();

    info!(target: "server", "🚀 Iniciando PSO3 server...");
    debug!(target: "server", "Carregando configurações do ambiente...");

    let database_url = std::env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set in .env");

    info!(target: "db", "Conectando ao banco de dados...");
    let pool = match PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await {
            Ok(pool) => {
                info!(target: "db", "✅ Conexão com banco de dados estabelecida");
                pool
            },
            Err(e) => {
                error!(target: "db", error = %e, "❌ Falha ao conectar com o banco de dados");
                panic!("Database connection failed: {}", e);
            }
    };

    let state = AppState::new(pool.clone());
    debug!(target: "server", "Configurando serviços da API...");
    let api_service = OpenApiService::new(Api::new(state.clone()), "WhatsApp Integration", "1.0")
        .server("http://localhost:3000/api");
    
    let ui = api_service.swagger_ui();

    debug!(target: "server", "Configurando rotas...");
    let app = Route::new()
        .nest("/api", api_service)
        .nest("/swagger", ui)
        .nest("/static", StaticFilesEndpoint::new("public")) // Changed to /static path
        .nest("/", routes::routes())
        .with(CookieJarManager::new())
        .with(LogMiddleware)
        .with(AddData::new(state));

    info!(target: "server", "✨ Servidor pronto! Escutando em http://localhost:3000");
    
    Server::new(TcpListener::bind("127.0.0.1:3000"))
        .run(app)
        .await
}