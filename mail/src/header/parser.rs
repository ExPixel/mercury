// SPDX-License-Identifier: GPL-3.0-or-later

#![allow(dead_code)]

pub(crate) mod address;
mod optional;

use std::{
    borrow::Cow,
    cell::{Cell, RefCell},
    ops::RangeFrom,
};

use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::{char, crlf},
    combinator::{map, opt, recognize},
    error::ParseError,
    multi::{fold_many0, fold_many1, many0_count, many1_count},
    sequence::{delimited, pair, preceded, terminated, tuple},
    IResult, InputIter, Slice,
};

use self::optional::optional_field;

use super::{HeaderMap, InvalidHeaderMap};

pub fn headers(i: &[u8]) -> Result<(&[u8], HeaderMap), InvalidHeaderMap> {
    terminated(
        fold_many0(
            optional_field,
            HeaderMap::default,
            |mut map, (name, value)| {
                let name = std::str::from_utf8(name)
                    .expect("field name not valid UTF8")
                    .to_owned();
                let value = String::from_utf8(value)
                    .expect("value not valid UTF8")
                    .trim_matches(|ch: char| ch.is_ascii() && ch.is_whitespace())
                    .replace("\r\n", "");
                map.insert(name, value);
                map
            },
        ),
        crlf,
    )(i)
    .map_err(|_| InvalidHeaderMap::default())
}

/// FWS = ([*WSP CRLF] 1*WSP) /  obs-FWS
fn fws(i: &[u8]) -> IResult<&[u8], &[u8]> {
    let current = recognize(pair(
        opt(pair(many0_count(wsp), tag("\r\n"))),
        many1_count(wsp),
    ));
    alt((current, obs_fws))(i)
}

/// obs-FWS = 1*WSP *(CRLF 1*WSP)
fn obs_fws(i: &[u8]) -> IResult<&[u8], &[u8]> {
    recognize(pair(
        many1_count(wsp),
        many0_count(pair(crlf, many1_count(wsp))),
    ))(i)
}

/// ctext =   %d33-39 /          ; Printable US-ASCII
///           %d42-91 /          ;  characters not including
///           %d93-126 /         ;  "(", ")", or "\"
///           obs-ctext
fn ctext(i: &[u8]) -> IResult<&[u8], u8> {
    satisfy_u8(is_ctext)(i)
}

/// ccontent = ctext / quoted-pair / comment
fn ccontent(i: &[u8]) -> IResult<&[u8], &[u8]> {
    alt((recognize(ctext), recognize(quoted_pair), comment))(i)
}

/// comment = "(" *([FWS] ccontent) [FWS] ")"
fn comment(i: &[u8]) -> IResult<&[u8], &[u8]> {
    delimited(
        char('('),
        terminated(
            recognize(many0_count(preceded(opt(fws), ccontent))),
            opt(fws),
        ),
        char(')'),
    )(i)
}

/// CFWS = (1*([FWS] comment) [FWS]) / FWS
fn cfws(i: &[u8]) -> IResult<&[u8], &[u8]> {
    alt((
        terminated(
            recognize(many1_count(preceded(opt(fws), comment))),
            opt(fws),
        ),
        fws,
    ))(i)
}

/// atext = ALPHA / DIGIT /    ; Printable US-ASCII
///            "!" / "#" /        ;  characters not including
///            "$" / "%" /        ;  specials.  Used for atoms.
///            "&" / "'" /
///            "*" / "+" /
///            "-" / "/" /
///            "=" / "?" /
///            "^" / "_" /
///            "`" / "{" /
///            "|" / "}" /
///            "~"
fn atext(i: &[u8]) -> IResult<&[u8], u8> {
    satisfy_u8(is_atext)(i)
}

/// atom = [CFWS] 1*atext [CFWS]
fn atom(i: &[u8]) -> IResult<&[u8], Vec<u8>> {
    delimited(opt(cfws), many1_string(map(atext, |ch| [ch])), opt(cfws))(i)
}

/// dot-atom-text = 1*atext *("." 1*atext)
fn dot_atom_text(i: &[u8]) -> IResult<&[u8], &[u8]> {
    recognize(pair(
        many1_count(atext),
        many0_count(preceded(char('.'), many1_count(atext))),
    ))(i)
}

/// dot-atom = [CFWS] dot-atom-text [CFWS]
fn dot_atom(i: &[u8]) -> IResult<&[u8], &[u8]> {
    delimited(opt(cfws), dot_atom_text, opt(cfws))(i)
}

/// specials = "(" / ")" /      ; Special characters that do
///            "<" / ">" /      ;  not appear in atext
///            "[" / "]" /
///            ":" / ";" /
///            "@" / "\" /
///            "," / "." /
///            DQUOTE
fn specials(i: &[u8]) -> IResult<&[u8], u8> {
    satisfy_u8(is_specials)(i)
}

/// qtext = %d33 /     ; Printable US-ASCII
///         %d35-91 /  ;  characters not including
///         %d93-126 / ;  "\" or the quote character
///         obs-qtext
fn qtext(i: &[u8]) -> IResult<&[u8], u8> {
    satisfy_u8(is_qtext)(i)
}

/// quoted-pair = ("\" (VCHAR / WSP)) / obs-qp
fn quoted_pair(i: &[u8]) -> IResult<&[u8], u8> {
    alt((preceded(tag("\\"), alt((vchar, wsp))), obs_qp))(i)
}

/// obs-qp = "\" (%d0 / obs-NO-WS-CTL / LF / CR)
fn obs_qp(i: &[u8]) -> IResult<&[u8], u8> {
    preceded(
        byte(b'\\'),
        alt((
            satisfy_u8(|ch: u8| ch == 0 || ch == b'\r' || ch == b'\n'),
            obs_no_ws_ctl,
        )),
    )(i)
}

/// qcontent = qtext / quoted-pair
fn qcontent(i: &[u8]) -> IResult<&[u8], u8> {
    alt((qtext, quoted_pair))(i)
}

/// ```norun
/// quoted-string = [CFWS]
///                 DQUOTE *([FWS] qcontent) [FWS] DQUOTE
///                 [CFWS]
/// ```
fn quoted_string(i: &[u8]) -> IResult<&[u8], Vec<u8>> {
    let s = RefCell::new(Vec::with_capacity(16));
    let (i, _) = delimited(
        opt(cfws),
        delimited(
            char('"'),
            many0_count(preceded(
                map(opt(fws), |w| {
                    if w.is_some() {
                        s.borrow_mut().push(b' ');
                    }
                }),
                map(qcontent, |q| s.borrow_mut().push(q as u8)),
            )),
            char('"'),
        ),
        opt(cfws),
    )(i)?;
    Ok((i, s.into_inner()))
}

/// word = atom / quoted-string
fn word(i: &[u8]) -> IResult<&[u8], Vec<u8>> {
    alt((atom, quoted_string))(i)
}

/// phrase = 1*word / obs-phrase
fn phrase(i: &[u8]) -> IResult<&[u8], Vec<u8>> {
    alt((
        fold_many1(word, Vec::new, |mut s, w| {
            if s.is_empty() {
                w
            } else {
                s.extend(w);
                s
            }
        }),
        map(obs_phrase, |o| o),
    ))(i)
}

/// obs-phrase = word *(word / "." / CFWS)
fn obs_phrase(i: &[u8]) -> IResult<&[u8], Vec<u8>> {
    let (i, mut s) = word(i)?;
    let (i, _) = many0_count(map(
        alt((
            map(word, Cow::Owned),
            map(tag("."), |_| Cow::Borrowed(&b"."[..])),
            map(cfws, |_| Cow::Borrowed(&b""[..])),
        )),
        |w| s.extend(&*w),
    ))(i)?;
    Ok((i, s))
}

/// `unstructured = (*([FWS] VCHAR) *WSP) / obs-unstruct`
pub fn unstructured(i: &[u8]) -> IResult<&[u8], Vec<u8>> {
    // let current = pair(many0_count(pair(opt(fws), vchar)), many0_count(wsp));
    let whitespace = Cell::new(false);
    let current = terminated(
        fold_many0(
            preceded(
                map(opt(fws), |w| {
                    if w.is_some() {
                        whitespace.set(true);
                    }
                }),
                vchar,
            ),
            Vec::new,
            |mut acc, c| {
                if whitespace.replace(false) {
                    acc.push(b' ');
                }
                acc.push(c);
                acc
            },
        ),
        many0_count(wsp),
    );

    let ret = alt((current, obs_unstruct))(i);
    let _extend_lifetime = whitespace; // FIXME ??? this is mostly to shut clippy up, but ret required for compiler
    ret
}

/// obs-unstruct = *((*LF *CR *(obs-utext *LF *CR)) / FWS)
fn obs_unstruct(i: &[u8]) -> IResult<&[u8], Vec<u8>> {
    let mut out = Vec::new();
    let whitespace = Cell::new(false);
    let mark_ws = |c: usize| {
        if c > 0 {
            whitespace.set(true)
        }
    };

    let (i, _) = many0_count(alt((
        map(
            tuple((
                map(many0_count(char('\n')), mark_ws),
                map(many0_count(char('\r')), mark_ws),
                many0_count(tuple((
                    map(obs_utext, |c| {
                        if whitespace.replace(false) {
                            out.push(b' ');
                        }
                        out.push(c);
                    }),
                    map(many0_count(char('\n')), mark_ws),
                    map(many0_count(char('\r')), mark_ws),
                ))),
            )),
            |_| (),
        ),
        map(fws, |_| ()),
    )))(i)?;
    Ok((i, out))
}

/// obs-NO-WS-CTL =   %d1-8 /            ; US-ASCII control
///                   %d11 /             ;  characters that do not
///                   %d12 /             ;  include the carriage
///                   %d14-31 /          ;  return, line feed, and
///                   %d127              ;  white space characters
fn obs_no_ws_ctl(i: &[u8]) -> IResult<&[u8], u8> {
    satisfy_u8(is_obs_no_ws_ctl)(i)
}

/// obs-utext       =   %d0 / obs-NO-WS-CTL / VCHAR
fn obs_utext(i: &[u8]) -> IResult<&[u8], u8> {
    satisfy_u8(is_obs_utext)(i)
}

/// WSP = SPACE | HTAB
fn wsp(i: &[u8]) -> IResult<&[u8], u8> {
    satisfy_u8(|ch: u8| ch == b' ' || ch == b'\t')(i)
}

/// VCHAR = %x21-7E ; visible (printing) characters
fn vchar(i: &[u8]) -> IResult<&[u8], u8> {
    satisfy_u8(is_vchar)(i)
}

/// VCHAR = %x21-7E ; visible (printing) characters
fn is_vchar(ch: u8) -> bool {
    matches!(ch, 0x21..=0x7E)
}

/// see [`qtext`]
fn is_qtext(ch: u8) -> bool {
    matches!(ch, 33 | 35..=91 | 93..=126) || is_obs_qtext(ch)
}

/// see [`ctext`]
fn is_ctext(ch: u8) -> bool {
    matches!(ch, 33..=39 | 42..=91 | 93..=126) || is_obs_ctext(ch)
}

/// obs-ctext = obs-NO-WS-CTL
fn is_obs_ctext(ch: u8) -> bool {
    is_obs_no_ws_ctl(ch)
}

/// obs-qtext = obs-NO-WS-CTL
fn is_obs_qtext(ch: u8) -> bool {
    is_obs_no_ws_ctl(ch)
}

/// see: [`obs_utext`]
fn is_obs_utext(ch: u8) -> bool {
    ch == 0 || is_obs_no_ws_ctl(ch) || is_vchar(ch)
}

/// see: [`obs_no_ws_ctl`]
fn is_obs_no_ws_ctl(ch: u8) -> bool {
    matches!(ch, 1..=8 | 11 | 12 | 14..=31 | 127)
}

/// see: [`atext`]
fn is_atext(ch: u8) -> bool {
    const ATEXT_SYMBOLS: &[u8] = b"!#$%&'*+-/=?^_`{|}~";
    ch.is_ascii_alphanumeric() || ATEXT_SYMBOLS.contains(&ch)
}

/// see: [`specials`]
fn is_specials(ch: u8) -> bool {
    const SPECIALS: &[u8] = b"()<>[]:;@\\,.\"";
    SPECIALS.contains(&ch)
}

pub(crate) fn byte<I, Error: ParseError<I>>(c: u8) -> impl Fn(I) -> IResult<I, u8, Error>
where
    I: Slice<RangeFrom<usize>> + InputIter,
    <I as InputIter>::Item: Into<u8>,
{
    move |i: I| match (i).iter_elements().next().map(|t| {
        let b = t.into() == c;
        (&c, b)
    }) {
        Some((c, true)) => Ok((i.slice(1..), *c)),
        _ => Err(nom::Err::Error(Error::from_char(i, c as char))),
    }
}

pub(crate) fn satisfy_u8<F, I, Error: ParseError<I>>(cond: F) -> impl Fn(I) -> IResult<I, u8, Error>
where
    I: Slice<RangeFrom<usize>> + InputIter,
    <I as InputIter>::Item: Into<u8>,
    F: Fn(u8) -> bool,
{
    move |i: I| match (i).iter_elements().next().map(|t| {
        let c = t.into();
        let b = cond(c);
        (c, b)
    }) {
        Some((c, true)) => Ok((i.slice(1..), c)),
        _ => Err(nom::Err::Error(Error::from_error_kind(
            i,
            nom::error::ErrorKind::Satisfy,
        ))),
    }
}

pub(crate) fn many1_string<I, O, E, F>(mut f: F) -> impl FnMut(I) -> IResult<I, Vec<u8>, E>
where
    I: Clone + nom::InputLength,
    F: nom::Parser<I, O, E>,
    O: AsRef<[u8]>,
    E: nom::error::ParseError<I>,
{
    move |i| {
        fold_many1(
            |i| f.parse(i),
            Vec::new,
            |mut acc, s| {
                acc.extend(s.as_ref());
                acc
            },
        )(i)
    }
}
