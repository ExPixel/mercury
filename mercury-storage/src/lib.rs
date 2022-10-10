// SPDX-License-Identifier: GPL-3.0-or-later

mod error;
pub mod mail;
mod sqlite;

use self::mail::MailId;
use self::mail::MailStorage;
use error::{Error, Result};
use serde::Deserialize;
use sqlite::SqliteStorage;
use std::{path::PathBuf, sync::Arc};
use tokio::sync::broadcast;

#[derive(Clone)]
pub struct Storage {
    inner: Arc<StorageInner>,
}

impl Storage {
    pub fn new(config: StorageConfig) -> Result<Self> {
        StorageInner::new(config).map(|inner| Storage {
            inner: Arc::new(inner),
        })
    }

    pub fn mail(&self) -> &MailStorage {
        &self.inner.mail
    }

    pub fn subscribe(&self) -> broadcast::Receiver<StorageEvent> {
        self.inner.subscribe()
    }
}

pub struct StorageInner {
    pub mail: MailStorage,
    pub event_tx: broadcast::Sender<StorageEvent>,
}

impl StorageInner {
    pub fn new(config: StorageConfig) -> Result<Self> {
        Self::init_paths(&config)?;
        let mut connection = rusqlite::Connection::open(&config.sqlite.path)
            .map_err(|e| Error::Sqlite(e, "opening database"))?;
        sqlite::add_callbacks(&mut connection);
        sqlite::migrations::migrate(&mut connection)?;
        let sql = SqliteStorage::new(connection);

        let (event_tx, _event_rx) = broadcast::channel(8);

        Ok(StorageInner {
            mail: MailStorage::new(sql, event_tx.clone(), config.mail),
            event_tx,
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

    pub fn subscribe(&self) -> broadcast::Receiver<StorageEvent> {
        self.event_tx.subscribe()
    }
}

#[derive(Clone, Debug)]
pub enum StorageEvent {
    NewMail(MailId),
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
