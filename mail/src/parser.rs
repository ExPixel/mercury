// SPDX-License-Identifier: GPL-3.0-or-later

use nom::{
    branch::alt,
    character::complete::{char, satisfy},
    sequence::preceded,
    IResult,
};

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
