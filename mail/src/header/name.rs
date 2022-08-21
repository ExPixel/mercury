// SPDX-License-Identifier: GPL-3.0-or-later

use std::{
    borrow::{Borrow, Cow},
    fmt::{Debug, Display},
};

use serde::{Deserialize, Serialize};

#[derive(Eq, Clone)]
pub struct HeaderName<'a>(Cow<'a, [u8]>);

impl<'a> Serialize for HeaderName<'a> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        str::serialize(self.as_str(), serializer)
    }
}

impl<'a, 'de> Deserialize<'de> for HeaderName<'a> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        String::deserialize(deserializer)
            .map(|s| HeaderName::try_from(s).expect("invalid header name deserialized"))
    }
}

impl<'a> HeaderName<'a> {
    pub const fn from_static(s: &'static str) -> HeaderName<'static> {
        let bytes = s.as_bytes();
        let mut idx = 0;
        while idx < bytes.len() {
            if !bytes[idx].is_ascii() {
                panic!("header name must be ASCII");
            }
            idx += 1;
        }
        HeaderName(Cow::Borrowed(bytes))
    }

    pub fn into_owned(self) -> HeaderName<'static> {
        HeaderName(Cow::Owned(self.0.into_owned()))
    }
}

impl Debug for HeaderName<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Debug::fmt(&self.0, f)
    }
}

impl Display for HeaderName<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Display::fmt(self.as_str(), f)
    }
}

impl<'a> Ord for HeaderName<'a> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        let a = self
            .as_ref()
            .iter()
            .copied()
            .map(|c| c.to_ascii_lowercase());
        let b = other
            .as_ref()
            .iter()
            .copied()
            .map(|c| c.to_ascii_lowercase());
        a.cmp(b)
    }
}

impl<'a> PartialOrd for HeaderName<'a> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl<'a, 'b> PartialEq<HeaderName<'b>> for HeaderName<'a> {
    fn eq(&self, other: &HeaderName<'b>) -> bool {
        self.as_ref().eq_ignore_ascii_case(other.as_ref())
    }
}

impl<'a> AsRef<[u8]> for HeaderName<'a> {
    fn as_ref(&self) -> &[u8] {
        self.0.as_ref()
    }
}

impl<'a> std::hash::Hash for HeaderName<'a> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        for &ch in self.0.as_ref() {
            state.write_u8(ch.to_ascii_lowercase());
        }
    }
}

impl<'a> HeaderName<'a> {
    pub fn as_bytes(&self) -> &[u8] {
        &self.0
    }

    pub fn as_str(&self) -> &str {
        unsafe { std::str::from_utf8_unchecked(self.as_bytes()) }
    }
}

impl<'a> Borrow<str> for HeaderName<'a> {
    fn borrow(&self) -> &str {
        self.as_str()
    }
}

impl<'a> Borrow<[u8]> for HeaderName<'a> {
    fn borrow(&self) -> &[u8] {
        self.0.borrow()
    }
}

#[derive(Debug)]
pub struct InvalidHeaderName {
    _inner: (),
}

impl Display for InvalidHeaderName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("invalid header name")
    }
}

impl TryFrom<String> for HeaderName<'static> {
    type Error = InvalidHeaderName;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        value
            .is_ascii()
            .then(move || HeaderName(Cow::Owned(value.into_bytes())))
            .ok_or(InvalidHeaderName { _inner: () })
    }
}

impl<'a> TryFrom<&'a str> for HeaderName<'a> {
    type Error = InvalidHeaderName;

    fn try_from(value: &'a str) -> Result<Self, Self::Error> {
        value
            .is_ascii()
            .then(move || HeaderName(Cow::Borrowed(value.as_bytes())))
            .ok_or(InvalidHeaderName { _inner: () })
    }
}

impl<'a> TryFrom<&'a [u8]> for HeaderName<'a> {
    type Error = InvalidHeaderName;

    fn try_from(value: &'a [u8]) -> Result<Self, Self::Error> {
        value
            .is_ascii()
            .then(move || HeaderName(Cow::Borrowed(value)))
            .ok_or(InvalidHeaderName { _inner: () })
    }
}

impl<'a, 'b: 'a> From<&'a HeaderName<'b>> for HeaderName<'a> {
    fn from(h: &'a HeaderName<'b>) -> Self {
        HeaderName(Cow::Borrowed(h.as_bytes()))
    }
}
