// SPDX-License-Identifier: GPL-3.0-or-later

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
use mail::{header::typed, HeaderMap};
use serde::Deserialize;
use serde_json::{Map, Number, Value};
use storage::{mail::MailId, Storage};
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
) -> Result<Json<Value>, (StatusCode, &'static str)> {
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

    let mut resp = Vec::new();

    for mail in list {
        let mut item = Map::<String, Value>::with_capacity(16);
        item.insert("id".to_owned(), Number::from(i64::from(mail.id)).into());
        if let Err(err) = serialize_mail_item_headers(&mail.headers, &mut item) {
            error!("error while serializing mail item headers: {err}");
            continue;
        }
        resp.push(Value::Object(item));
    }

    Ok(Json(Value::Array(resp)))
}

fn serialize_mail_item_headers(
    headers: &HeaderMap,
    item: &mut Map<String, Value>,
) -> Result<(), &'static str> {
    macro_rules! insert_header {
        ($HeaderType:ty, $header_name:literal) => {
            if let Some(value) = headers
                .get_typed::<$HeaderType>()
                .map_err(|_| concat!("invalid ", $header_name, " header"))?
            {
                let value = serde_json::to_value(value)
                    .map_err(|_| concat!("failed to serialize ", $header_name, "header"))?;
                item.insert($header_name.to_owned(), value);
            }
        };
    }

    insert_header!(typed::From, "from");
    insert_header!(typed::Sender, "sender");
    insert_header!(typed::To, "to");

    Ok(())
}
