use poem::{listener::TcpListener, EndpointExt, Server};
use std::sync::Arc;

mod auth;
mod routes;
mod state;

use state::AppState;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let state = Arc::new(AppState::new());
    let app = routes::create_routes().data(state); // <-- SEM ": Route"

    Server::new(TcpListener::bind("127.0.0.1:3000"))
        .run(app)
        .await?;

    Ok(())
}
