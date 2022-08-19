// SPDX-License-Identifier: GPL-3.0-or-later

mod name;
mod parser;
mod typed;

pub use name::HeaderName;
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    fmt::{Debug, Display},
};

#[derive(Default, Serialize, Deserialize)]
pub struct HeaderMap {
    #[serde(flatten)]
    inner: HashMap<HeaderName<'static>, String>,
}

impl HeaderMap {
    pub fn parse(bytes: &[u8]) -> Result<(&[u8], Self), InvalidHeaderMap> {
        parser::headers(bytes)
    }

    pub fn insert<H, V>(&mut self, header: H, value: V)
    where
        H: TryInto<HeaderName<'static>>,
        H::Error: Debug,
        V: Into<String>,
    {
        self.inner.insert(
            header.try_into().expect("invalid header name"),
            value.into(),
        );
    }

    pub fn get<'s, K>(&'s self, key: K) -> Option<&'s str>
    where
        K: Into<HeaderName<'s>>,
    {
        self.inner.get(&key.into()).map(|s| s.as_str())
    }

    pub fn iter(&self) -> impl '_ + Iterator<Item = (HeaderName, &String)> {
        self.inner.iter().map(|(k, v)| (HeaderName::from(k), v))
    }

    pub fn iter_mut(&mut self) -> impl '_ + Iterator<Item = (HeaderName, &mut String)> {
        self.inner.iter_mut().map(|(k, v)| (HeaderName::from(k), v))
    }
}

impl Debug for HeaderMap {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Debug::fmt(&self.inner, f)
    }
}

#[derive(Default, Debug)]
pub struct InvalidHeaderMap {
    _inner: (),
}

impl Display for InvalidHeaderMap {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("invalid header map")
    }
}

pub static KNOWN_HEADERS: &[HeaderName<'static>] = &[
    HeaderName::from_static("trace"),
    HeaderName::from_static("resent-date"),
    HeaderName::from_static("resent-from"),
    HeaderName::from_static("resent-sender"),
    HeaderName::from_static("resent-to"),
    HeaderName::from_static("resent-cc"),
    HeaderName::from_static("resent-bcc"),
    HeaderName::from_static("resent-msg-id"),
    HeaderName::from_static("orig-date"),
    HeaderName::from_static("from"),
    HeaderName::from_static("sender"),
    HeaderName::from_static("reply-to"),
    HeaderName::from_static("to"),
    HeaderName::from_static("cc"),
    HeaderName::from_static("bcc"),
    HeaderName::from_static("message-id"),
    HeaderName::from_static("in-reply-to"),
    HeaderName::from_static("references"),
    HeaderName::from_static("subject"),
    HeaderName::from_static("comments"),
    HeaderName::from_static("keywords"),
];
