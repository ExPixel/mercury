// SPDX-License-Identifier: GPL-3.0-or-later

use std::{collections::HashMap, fmt::Debug};

mod parser;

#[derive(Default)]
pub struct HeaderMap {
    inner: HashMap<String, String>,
}

impl HeaderMap {
    pub fn parse(bytes: &[u8]) -> Result<(&[u8], Self), InvalidHeaderMap> {
        parser::headers(bytes)
    }

    fn insert(&mut self, header: String, value: String) {
        self.inner.insert(header, value);
    }
}

impl Debug for HeaderMap {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Debug::fmt(&self.inner, f)
    }
}

pub struct InvalidHeaderMap {}
