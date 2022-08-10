// SPDX-License-Identifier: GPL-3.0-or-later

mod api;

use anyhow::Context as _;
use axum::{http::StatusCode, routing::get_service, Extension, Router};
use axum_extra::routing::SpaRouter;
use std::net::SocketAddr;
use storage::Storage;
use tower::ServiceBuilder;
use tower_http::{compression::CompressionLayer, services::ServeDir, trace::TraceLayer};
use tracing::debug;

pub async fn run(http_config: &HttpConfig, storage: Storage) -> anyhow::Result<()> {
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
        .nest("/api", api::routes())
        .layer(
            ServiceBuilder::new()
                .layer(TraceLayer::new_for_http())
                .layer(CompressionLayer::new())
                .layer(Extension(storage)),
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
