// SPDX-License-Identifier: GPL-3.0-or-later

pub mod header;

use std::ops::Range;

pub use header::HeaderMap;

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
    header: HeaderMap,
    body: Range<usize>,
    parts: Vec<Entity>,
}

pub enum Encoding {}

pub enum Part {
    Single(SinglePart),
    Multi(MultiPart),
}
