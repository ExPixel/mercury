// SPDX-License-Identifier: GPL-3.0-or-later

mod name;
mod parser;
pub mod parts;
pub mod typed;

pub use name::HeaderName;
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    fmt::{Debug, Display},
};

use self::typed::TypedHeader;

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

    pub fn get_typed<T: TypedHeader>(&self) -> Result<Option<T>, T::Error> {
        self.get(T::NAME).map(|value| T::decode(value)).transpose()
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

macro_rules! known_headers {
    ($(const $ConstName:ident = $string_name:expr),* $(,)?) => {
        $(pub const $ConstName: HeaderName = HeaderName::from_static($string_name);)*

        pub static KNOWN_HEADERS: &[HeaderName<'static>] = &[
            $($ConstName),*
        ];
    };
}

known_headers! {
    const TRACE = "trace",
    const RESENT_DATE = "resent-date",
    const RESENT_FROM = "resent-from",
    const RESENT_SENDER = "resent-sender",
    const RESENT_TO = "resent-to",
    const RESENT_CC = "resent-cc",
    const RESENT_BCC = "resent-bcc",
    const RESENT_MSG_ID = "resent-msg-id",
    const ORIG_DATE = "orig-date",
    const FROM = "from",
    const SENDER = "sender",
    const REPLY_TO = "reply-to",
    const TO = "to",
    const CC = "cc",
    const BCC = "bcc",
    const MESSAGE_ID = "message-id",
    const IN_REPLY_TO = "in-reply-to",
    const REFERENCES = "references",
    const SUBJECT = "subject",
    const COMMENTS = "comments",
    const KEYWORDS = "keywords",
}
