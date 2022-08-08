use anyhow::Context as _;
use axum::{http::StatusCode, routing::get_service, Router};
use axum_extra::routing::SpaRouter;
use std::net::SocketAddr;
use tower::ServiceBuilder;
use tower_http::{compression::CompressionLayer, services::ServeDir, trace::TraceLayer};
use tracing::debug;

pub async fn run(http_config: &HttpConfig) -> anyhow::Result<()> {
    let static_files_service =
        get_service(ServeDir::new("static")).handle_error(|error: std::io::Error| async move {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Unhandled internal error: {}", error),
            )
        });
    let spa = SpaRouter::new("/spa", "build").index_file("../build/main.html");
    let app = Router::new()
        .merge(spa)
        .nest("/static", static_files_service)
        .layer(
            ServiceBuilder::new()
                .layer(TraceLayer::new_for_http())
                .layer(CompressionLayer::new()),
        );
    let addr: SocketAddr = http_config
        .address
        .parse()
        .context("failed to parse http addr")?;
    debug!(addr = display(addr), "starting HTTP server");
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .context("error while running http server")
}

#[derive(serde::Deserialize)]
pub struct HttpConfig {
    pub address: String,
}
