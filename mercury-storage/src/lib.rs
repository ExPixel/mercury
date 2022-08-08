mod error;
mod mail;
mod sqlite;

use error::{Error, Result};
pub use mail::MailStorage;
use serde::Deserialize;
use sqlite::SqliteStorage;
use std::path::PathBuf;

#[derive(Clone)]
pub struct Storage {
    pub mail: MailStorage,
}

impl Storage {
    pub fn new(config: StorageConfig) -> Result<Self> {
        Self::init_paths(&config)?;
        let mut connection = rusqlite::Connection::open(&config.sqlite.path)
            .map_err(|e| Error::Sqlite(e, "opening database"))?;
        sqlite::add_callbacks(&mut connection);
        sqlite::migrations::migrate(&mut connection)?;
        let sql = SqliteStorage::new(connection);
        Ok(Storage {
            mail: MailStorage::new(sql, config.mail),
        })
    }

    fn init_paths(config: &StorageConfig) -> Result<()> {
        if let Some(parent) = config.sqlite.path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| Error::CreateDir(e, parent.into()))?;
        }
        std::fs::create_dir_all(&config.mail.directory)
            .map_err(|e| Error::CreateDir(e, config.mail.directory.clone()))?;
        Ok(())
    }
}

#[derive(Deserialize)]
pub struct StorageConfig {
    sqlite: SqliteStorageConfig,
    mail: MailStorageConfig,
}

#[derive(Deserialize, Clone)]
pub struct SqliteStorageConfig {
    path: PathBuf,
}

#[derive(Deserialize, Clone)]
pub struct MailStorageConfig {
    directory: PathBuf,
}
