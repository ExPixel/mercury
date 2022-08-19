use nom::{
    character::complete::{char, crlf, satisfy},
    combinator::recognize,
    multi::{many0_count, many1_count},
    sequence::{pair, separated_pair, terminated},
    IResult,
};

use super::{unstructured, wsp};

/// optional-field = field-name ":" unstructured CRLF
pub fn optional_field(i: &[u8]) -> IResult<&[u8], (&[u8], &[u8])> {
    terminated(
        separated_pair(field_name, pair(many0_count(wsp), char(':')), unstructured),
        crlf,
    )(i)
}

/// field-name = 1*ftext
pub fn field_name(i: &[u8]) -> IResult<&[u8], &[u8]> {
    recognize(many1_count(ftext))(i)
}

/// ftext =   %d33-57 / ; Printable US-ASCII
///           %d59-126  ;  characters not including
///                     ;  ":".
pub fn ftext(i: &[u8]) -> IResult<&[u8], char> {
    satisfy(|ch: char| ch.is_ascii() && is_ftext(ch as u8))(i)
}

/// see: [`ftext`]
pub fn is_ftext(ch: u8) -> bool {
    matches!(ch, 33..=57 | 59..=126)
}
