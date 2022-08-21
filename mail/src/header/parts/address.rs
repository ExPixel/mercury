// SPDX-License-Identifier: GPL-3.0-or-later

use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
#[serde(tag = "type")]
pub enum Address {
    Mailbox(Mailbox),
    Group(Group),
}

#[derive(Deserialize, Serialize)]
pub struct Mailbox {
    display_name: String,
    address: String,
}

impl Mailbox {
    pub(crate) fn new_raw(display_name: Option<&[u8]>, address: &[u8]) -> Self {
        let display_name = std::str::from_utf8(display_name.unwrap_or(&[]))
            .expect("display_name not valid UTF8")
            .trim_matches(|ch: char| ch.is_ascii() && ch.is_whitespace())
            .replace("\r\n", "");
        let address = std::str::from_utf8(address)
            .expect("address not valid UTF8")
            .trim_matches(|ch: char| ch.is_ascii() && ch.is_whitespace())
            .replace("\r\n", "");

        Mailbox {
            display_name,
            address,
        }
    }
}

#[derive(Deserialize, Serialize)]
pub struct Group {
    display_name: String,
    mailboxes: Vec<Mailbox>,
}

impl Group {
    pub(crate) fn new_raw(display_name: Option<&[u8]>, mailboxes: Vec<Mailbox>) -> Self {
        let display_name = std::str::from_utf8(display_name.unwrap_or(&[]))
            .expect("display_name not valid UTF8")
            .trim_matches(|ch: char| ch.is_ascii() && ch.is_whitespace())
            .replace("\r\n", "");

        Group {
            display_name,
            mailboxes,
        }
    }
}
