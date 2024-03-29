// SPDX-License-Identifier: GPL-3.0-or-later

use thiserror::Error;

pub type Result<T, E = Error> = std::result::Result<T, E>;

#[derive(Error, Debug)]
pub enum Error {
    #[error("sqlite error: {1}")]
    Sqlite(#[source] rusqlite::Error, &'static str),

    #[error("error while opening file: {1}")]
    OpenFile(#[source] std::io::Error, std::path::PathBuf),

    #[error("error while opening file: {1}")]
    CreateFile(#[source] std::io::Error, std::path::PathBuf),

    #[error("error while creating directory: {1}")]
    CreateDir(#[source] std::io::Error, std::path::PathBuf),

    #[error("json error: {1}")]
    Json(#[source] serde_json::Error, &'static str),

    #[error("compression error")]
    CompressionError(#[source] std::io::Error),
}
