// SPDX-License-Identifier: GPL-3.0-or-later

mod header;

use std::ops::Range;

#[allow(dead_code)]
mod parser;

pub enum Entity {
    SinglePart(SinglePart),
    MultiPart(MultiPart),
}

pub struct SinglePart {
    encoding: Encoding,
    body: Range<usize>,
}

pub struct MultiPart {
    header: Header,
    body: Range<usize>,
    parts: Vec<Entity>,
}

pub enum Encoding {}

pub enum Part {
    Single(SinglePart),
    Multi(MultiPart),
}

pub struct Header {}
