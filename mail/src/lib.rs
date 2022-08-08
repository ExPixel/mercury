/*
    Mercury Mail Parser
    Copyright (C) 2022 Adolph Celestin

    This program is free software: you can redistribute it and/or modify
    it under the terms of the GNU General Public License as published by
    the Free Software Foundation, either version 3 of the License, or
    (at your option) any later version.

    This program is distributed in the hope that it will be useful,
    but WITHOUT ANY WARRANTY; without even the implied warranty of
    MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
    GNU General Public License for more details.

    You should have received a copy of the GNU General Public License
    along with this program.  If not, see <https://www.gnu.org/licenses/>.
*/

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
