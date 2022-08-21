// SPDX-License-Identifier: GPL-3.0-or-later

use std::fmt::Display;

use nom::{
    character::complete::space0,
    combinator::eof,
    sequence::{pair, terminated},
};
use serde::Serialize;

use crate::header::{parser::address::address_list, parts::Address, TO};

use super::TypedHeader;

#[derive(Serialize)]
pub struct To(Vec<Address>);

impl TypedHeader for To {
    type Error = InvalidTo;
    const NAME: crate::header::HeaderName<'static> = TO;

    fn decode(encoded: &str) -> Result<Self, Self::Error> {
        terminated(address_list, pair(space0, eof))(encoded.as_bytes())
            .map(|(_, list)| To(list))
            .map_err(|_| InvalidTo::new())
    }
}

#[derive(Debug)]
pub struct InvalidTo {
    _inner: (),
}

impl InvalidTo {
    fn new() -> Self {
        Self { _inner: () }
    }
}

impl Display for InvalidTo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("invalid to header")
    }
}

impl std::error::Error for InvalidTo {}
