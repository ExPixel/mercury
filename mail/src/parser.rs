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

use nom::{
    branch::alt,
    character::complete::{char, satisfy},
    sequence::preceded,
    IResult,
};

// fn fws(i: &[u8]) -> IResult<&[u8], &[u8]> {
//     alt()
// }

fn quoted_pair(i: &[u8]) -> IResult<&[u8], char> {
    alt((
        preceded(char('\\'), alt((satisfy(is_vchar), satisfy(is_wsp)))),
        obs_qp,
    ))(i)
}

fn obs_qp(i: &[u8]) -> IResult<&[u8], char> {
    preceded(
        char('\\'),
        alt((char('\0'), obs_no_ws_ctl, char('\n'), char('\r'))),
    )(i)
}

fn obs_no_ws_ctl(i: &[u8]) -> IResult<&[u8], char> {
    satisfy(|ch: char| matches!(ch as u32, 1..=8 | 11..=12 | 14..=31 | 127))(i)
}

fn is_vchar(ch: char) -> bool {
    matches!(ch as u32, 0x21..=0x7E)
}

fn is_wsp(ch: char) -> bool {
    ch == ' ' || ch == '\t'
}

fn as_byte_fn<F>(f: F) -> impl Fn(u8) -> bool
where
    F: Fn(char) -> bool,
{
    move |ch: u8| (f)(ch as char)
}
