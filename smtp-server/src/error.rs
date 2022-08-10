// SPDX-License-Identifier: GPL-3.0-or-later

use thiserror::Error;

use std::io::Error as IoError;

pub(crate) type Result<T, E = Error> = std::result::Result<T, E>;

#[derive(Error, Debug)]
pub enum Error {
    #[error("IO error")]
    Io(#[from] IoError),

    #[error("read timeout")]
    ReadTimeout,

    #[error("write timeout")]
    WriteTimeout,
}
