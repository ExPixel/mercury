// SPDX-License-Identifier: GPL-3.0-or-later

use std::{
    fmt::Display,
    path::{Path, PathBuf},
};

use async_compression::tokio::write::GzipEncoder;
use mail::HeaderMap;
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use tokio::{io::AsyncWriteExt, sync::broadcast};
use tracing::debug;

use crate::{
    error::{Error, Result},
    sqlite::SqliteStorage,
    MailStorageConfig, StorageEvent,
};
use rusqlite::Result as SqliteResult;

#[derive(Clone)]
pub struct MailStorage {
    sql: SqliteStorage,
    event_tx: broadcast::Sender<StorageEvent>,
    config: MailStorageConfig,
}

impl MailStorage {
    pub fn new(
        sql: SqliteStorage,
        event_tx: broadcast::Sender<StorageEvent>,
        config: MailStorageConfig,
    ) -> Self {
        MailStorage {
            sql,
            event_tx,
            config,
        }
    }

    pub async fn store_mail(&self, headers: &HeaderMap, data: &[u8]) -> Result<MailId> {
        let mail_id = self.store_mail_headers(headers).await?;
        debug!(id = debug(mail_id), "mail metadata stored");
        let mail_file_path = self.mail_file_path(mail_id);
        write_mail_file(&mail_file_path, data).await?;
        debug!(path = debug(&mail_file_path), "mail data stored");
        let _ = self.event_tx.send(StorageEvent::NewMail(mail_id));
        Ok(mail_id)
    }

    pub async fn store_mail_headers(&self, headers: &HeaderMap) -> Result<MailId> {
        let headers_json = serde_json::to_string(headers)
            .map_err(|e| Error::Json(e, "serializing mail headers"))?;
        self.sql
            .with::<SqliteResult<MailId>, _>(move |conn| {
                let sql = "INSERT INTO mail (headers, created_at) VALUES (?, ?) RETURNING id;";
                let mut statement = conn.prepare_cached(sql)?;
                statement
                    .query_row((headers_json, OffsetDateTime::now_utc()), |r| r.get(0usize))
                    .map(MailId)
            })
            .await
            .map_err(|e| Error::Sqlite(e, "storing mail"))
    }

    pub async fn get_mail(&self, max: usize, before: Option<MailId>) -> Result<Vec<StoredMail>> {
        let before = before.map(|mid| mid.0).unwrap_or(i64::MAX);
        self.sql
            .with::<SqliteResult<Vec<StoredMail>>, _>(move |conn| {
                let sql =
                    "SELECT id, headers, created_at FROM mail WHERE id < ? ORDER BY id DESC LIMIT ?;";
                let mut statement = conn.prepare_cached(sql)?;
                let rows = statement.query_map((before, max as i64), |row| {
                    let headers_json = row.get::<_, String>(1usize)?;
                    let headers = serde_json::from_str(&headers_json).map_err(|e| {
                        rusqlite::Error::FromSqlConversionFailure(
                            1,
                            rusqlite::types::Type::Text,
                            Box::new(e),
                        )
                    })?;

                    Ok(StoredMail {
                        id: MailId(row.get(0usize)?),
                        headers,
                        created_at: row.get(2usize)?,
                    })
                })?;
                rows.collect()
            })
            .await
            .map_err(|e| Error::Sqlite(e, "fetching mail headers"))
    }

    pub fn mail_file_path(&self, id: MailId) -> PathBuf {
        self.config
            .directory
            .join(Path::new(&format!("{}.mail.gz", id.0)))
    }
}

async fn write_mail_file(path: &Path, data: &[u8]) -> Result<()> {
    let file = tokio::fs::File::create(&path)
        .await
        .map_err(|err| Error::CreateFile(err, path.into()))?;

    let mut encoder = GzipEncoder::new(file);
    encoder
        .write_all(data)
        .await
        .map_err(Error::CompressionError)?;
    encoder.shutdown().await.map_err(Error::CompressionError)?;
    Ok(())
}

pub struct StoredMail {
    pub id: MailId,
    pub headers: HeaderMap,
    pub created_at: OffsetDateTime,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct MailId(i64);

impl From<MailId> for i64 {
    fn from(id: MailId) -> Self {
        id.0
    }
}

impl Display for MailId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        <i64 as Display>::fmt(&self.0, f)
    }
}
