use std::path::{Path, PathBuf};

use time::OffsetDateTime;
use uuid::Uuid;

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

    pub async fn store_mail_metadata(&self, id: MailId) -> Result<()> {
        self.sql
            .with::<SqliteResult<()>, _>(move |conn| {
                let sql = "INSERT INTO mail (id, metadata, created_at) VALUES (?, ?, ?);";
                let mut statement = conn.prepare_cached(sql)?;
                statement.execute((id.0, "", OffsetDateTime::now_utc()))?;
                Ok(())
            })
            .await
            .map_err(|e| Error::Sqlite(e, "storing mail"))
    }

    pub fn mail_file_path(&self, id: MailId) -> PathBuf {
        self.config
            .directory
            .join(Path::new(&format!("{}.mail.gz", id.0)))
    }

    pub fn generate_mail_id(&self) -> MailId {
        MailId(Uuid::new_v4())
    }
}

#[derive(Clone, Copy)]
pub struct MailId(pub Uuid);
