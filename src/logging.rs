use poem::{Middleware, Request, Endpoint, Result};
use std::time::Instant;
use tracing::{info, debug, error};
use tracing_appender::rolling::{RollingFileAppender, Rotation};
use tracing_subscriber::{
    fmt::format::FmtSpan,
    EnvFilter,
    prelude::*,
};

pub fn setup_logging() {
    let file_appender = RollingFileAppender::new(
        Rotation::DAILY,
        "logs",
        "pso3.log",
    );

    let (non_blocking, _guard) = tracing_appender::non_blocking(file_appender);

    let env_filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("info"));

    tracing_subscriber::registry()
        .with(env_filter)
        .with(tracing_subscriber::fmt::layer()
            .with_target(true)
            .with_thread_ids(true)
            .with_thread_names(true)
            .with_file(true)
            .with_line_number(true)
            .with_span_events(FmtSpan::CLOSE)
            .with_writer(std::io::stdout))
        .with(tracing_subscriber::fmt::layer()
            .with_writer(non_blocking)
            .with_ansi(false)
            .with_target(true)
            .with_thread_ids(true)
            .with_thread_names(true)
            .with_file(true)
            .with_line_number(true))
        .init();

    info!(target: "server", "Logging system initialized");
}

pub struct LogMiddleware;

impl<E: Endpoint> Middleware<E> for LogMiddleware 
where
    E::Output: Send + Sync,
{
    type Output = LoggedEndpoint<E>;

    fn transform(&self, ep: E) -> Self::Output {
        LoggedEndpoint(ep)
    }
}

pub struct LoggedEndpoint<E>(E);

#[poem::async_trait]
impl<E: Endpoint> Endpoint for LoggedEndpoint<E>
where
    E::Output: Send + Sync,
{
    type Output = E::Output;

    async fn call(&self, req: Request) -> Result<Self::Output> {
        let start = Instant::now();
        let method = req.method().to_string();
        let uri = req.uri().to_string();
        let remote_addr = req.remote_addr().to_string();

        debug!(
            target: "http",
            method = %method,
            path = %uri,
            client = %remote_addr,
            "Request received"
        );

        let res = self.0.call(req).await;

        match &res {
            Ok(_) => {
                let duration = start.elapsed();
                info!(
                    target: "http",
                    method = %method,
                    path = %uri,
                    duration = ?duration,
                    "Request completed"
                );
            }
            Err(e) => {
                error!(
                    target: "http",
                    method = %method,
                    path = %uri,
                    error = %e,
                    "Request failed"
                );
            }
        }

        res
    }
}
