// SPDX-License-Identifier: GPL-3.0-or-later

use std::{convert::Infallible, fmt::Display};

use serde::Serialize;

use crate::header::SUBJECT;

use super::TypedHeader;

#[derive(Serialize)]
pub struct Subject(String);

impl TypedHeader for Subject {
    type Error = InvalidSubject;
    const NAME: crate::header::HeaderName<'static> = SUBJECT;

    fn decode(encoded: &str) -> Result<Self, Self::Error> {
        // NOTE: encoded is already unstructured, so no need to do anything here
        Ok(Subject(encoded.to_owned()))
    }
}

#[derive(Debug)]
pub struct InvalidSubject {
    _inner: Infallible,
}

impl Display for InvalidSubject {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("invalid from header")
    }
}

impl std::error::Error for InvalidSubject {}
