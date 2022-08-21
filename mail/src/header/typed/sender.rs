use std::fmt::Display;

// SPDX-License-Identifier: GPL-3.0-or-later

use nom::{
    character::complete::space0,
    combinator::eof,
    sequence::{pair, terminated},
};
use serde::Serialize;

use crate::header::{parser::address::mailbox, parts::Mailbox, SENDER};

use super::TypedHeader;

#[derive(Serialize)]
pub struct Sender(Mailbox);

impl TypedHeader for Sender {
    type Error = InvalidSender;
    const NAME: crate::header::HeaderName<'static> = SENDER;

    fn decode(encoded: &str) -> Result<Self, Self::Error> {
        terminated(mailbox, pair(space0, eof))(encoded.as_bytes())
            .map(|(_, mbox)| Sender(mbox))
            .map_err(|_| InvalidSender::new())
    }
}

#[derive(Debug)]
pub struct InvalidSender {
    _inner: (),
}

impl InvalidSender {
    fn new() -> Self {
        Self { _inner: () }
    }
}

impl Display for InvalidSender {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("invalid sender header")
    }
}

impl std::error::Error for InvalidSender {}
