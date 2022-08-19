// SPDX-License-Identifier: GPL-3.0-or-later

use std::{
    fmt::Display,
    path::{Path, PathBuf},
};

use mail::HeaderMap;
use serde::{Deserialize, Serialize};
use time::{serde::rfc3339, OffsetDateTime};

use crate::{
    error::{Error, Result},
    sqlite::SqliteStorage,
    MailStorageConfig,
};
use rusqlite::Result as SqliteResult;

#[derive(Clone)]
pub struct MailStorage {
    sql: SqliteStorage,
    config: MailStorageConfig,
}

impl MailStorage {
    pub fn new(sql: SqliteStorage, config: MailStorageConfig) -> Self {
        MailStorage { sql, config }
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

    pub async fn get_mail(&self, max: usize, after: Option<MailId>) -> Result<Vec<StoredMail>> {
        let after = after.map(|mid| mid.0).unwrap_or(-1);
        self.sql
            .with::<SqliteResult<Vec<StoredMail>>, _>(move |conn| {
                let sql =
                    "SELECT id, headers, created_at FROM mail WHERE id > ? ORDER BY id LIMIT ?;";
                let mut statement = conn.prepare_cached(sql)?;
                let rows = statement.query_map((after, max as i64), |row| {
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

#[derive(Serialize)]
pub struct StoredMail {
    pub id: MailId,
    pub headers: HeaderMap,
    #[serde(serialize_with = "rfc3339::serialize")]
    pub created_at: OffsetDateTime,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct MailId(i64);

impl Display for MailId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        <i64 as Display>::fmt(&self.0, f)
    }
}
