// SPDX-License-Identifier: GPL-3.0-or-later

use std::fmt::Display;

use nom::{
    character::complete::space0,
    combinator::eof,
    sequence::{pair, terminated},
};
use serde::Serialize;

use crate::header::{parser::address::mailbox_list, parts::Mailbox, FROM};

use super::TypedHeader;

#[derive(Serialize)]
pub struct From(Vec<Mailbox>);

impl TypedHeader for From {
    type Error = InvalidFrom;
    const NAME: crate::header::HeaderName<'static> = FROM;

    fn decode(encoded: &str) -> Result<Self, Self::Error> {
        terminated(mailbox_list, pair(space0, eof))(encoded.as_bytes())
            .map(|(_, list)| From(list))
            .map_err(|_| InvalidFrom::new())
    }
}

#[derive(Debug)]
pub struct InvalidFrom {
    _inner: (),
}

impl InvalidFrom {
    fn new() -> Self {
        Self { _inner: () }
    }
}

impl Display for InvalidFrom {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("invalid from header")
    }
}

impl std::error::Error for InvalidFrom {}
