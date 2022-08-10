use async_compression::tokio::bufread::GzipDecoder;
use axum::{
    body::StreamBody,
    extract::{Path, Query},
    http::StatusCode,
    response::{AppendHeaders, IntoResponse},
    routing::get,
    Extension, Json, Router,
};
use http::header;
use serde::Deserialize;
use storage::{
    mail::{MailId, StoredMail},
    Storage,
};
use tokio_util::io::ReaderStream;
use tracing::error;

pub fn routes() -> Router {
    Router::new()
        .route("/mail", get(mail_list))
        .route("/mail/:id/raw", get(raw_mail))
}

async fn raw_mail(Path(mail_id): Path<MailId>, storage: Extension<Storage>) -> impl IntoResponse {
    let mail_path = storage.mail.mail_file_path(mail_id);
    let file = match tokio::fs::File::open(mail_path).await {
        Ok(file) => file,
        Err(_) => return Err((StatusCode::NOT_FOUND, "mail file not found")),
    };
    let stream = ReaderStream::new(GzipDecoder::new(tokio::io::BufReader::new(file)));
    let body = StreamBody::new(stream);

    let headers = AppendHeaders([
        (header::CONTENT_TYPE, "text/plain; charset=utf-8"),
        (
            header::CONTENT_DISPOSITION,
            "attachment; filename=\"mail.txt\"",
        ),
    ]);

    Ok((headers, body))
}

#[derive(Deserialize)]
struct MailListQuery {
    max: Option<usize>,
    after: Option<MailId>,
}

async fn mail_list(
    Query(params): Query<MailListQuery>,
    storage: Extension<Storage>,
) -> Result<Json<Vec<StoredMail>>, (StatusCode, &'static str)> {
    let max = params.max.unwrap_or(32);
    let list = storage
        .mail
        .get_mail(max, params.after)
        .await
        .map_err(|err| {
            let err = anyhow::Error::from(err);
            error!("error while fetching mail list: {err:?}");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "error occurred while fetching list",
            )
        })?;
    Ok(Json(list))
}
