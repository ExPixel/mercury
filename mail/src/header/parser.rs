// SPDX-License-Identifier: GPL-3.0-or-later

use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::{char, crlf, one_of, satisfy},
    combinator::{opt, recognize},
    multi::{many0_count, many1_count},
    sequence::{delimited, pair, preceded, terminated},
    IResult,
};

use super::HeaderMap;

fn headers(i: &[u8]) -> Result<HeaderMap, ()> {
    todo!();
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
fn ctext(i: &[u8]) -> IResult<&[u8], char> {
    satisfy(|ch: char| ch.is_ascii() && is_ctext(ch as u8))(i)
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
fn atext(i: &[u8]) -> IResult<&[u8], char> {
    satisfy(|ch: char| ch.is_ascii() && is_atext(ch as u8))(i)
}

/// atom = [CFWS] 1*atext [CFWS]
fn atom(i: &[u8]) -> IResult<&[u8], &[u8]> {
    delimited(opt(cfws), recognize(many1_count(atext)), opt(cfws))(i)
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
fn specials(i: &[u8]) -> IResult<&[u8], char> {
    satisfy(|ch: char| ch.is_ascii() && is_specials(ch as u8))(i)
}

/// qtext = %d33 /     ; Printable US-ASCII
///         %d35-91 /  ;  characters not including
///         %d93-126 / ;  "\" or the quote character
///         obs-qtext
fn qtext(i: &[u8]) -> IResult<&[u8], char> {
    satisfy(|ch: char| ch.is_ascii() && is_qtext(ch as u8))(i)
}

/// quoted-pair = ("\" (VCHAR / WSP)) / obs-qp
fn quoted_pair(i: &[u8]) -> IResult<&[u8], char> {
    alt((preceded(char('\\'), alt((vchar, wsp))), obs_qp))(i)
}

/// qcontent = qtext / quoted-pair
fn qcontent(i: &[u8]) -> IResult<&[u8], char> {
    alt((qtext, quoted_pair))(i)
}

/// quoted-string = [CFWS]
///                 DQUOTE *([FWS] qcontent) [FWS] DQUOTE
///                 [CFWS]
fn quoted_string(i: &[u8]) -> IResult<&[u8], &[u8]> {
    delimited(
        opt(cfws),
        delimited(
            char('"'),
            recognize(many0_count(preceded(opt(fws), qcontent))),
            char('"'),
        ),
        opt(cfws),
    )(i)
}

/// obs-qp = "\" (%d0 / obs-NO-WS-CTL / LF / CR)
fn obs_qp(i: &[u8]) -> IResult<&[u8], char> {
    preceded(char('\\'), alt((one_of("\0\r\n"), obs_no_ws_ctl)))(i)
}

/// obs-NO-WS-CTL =   %d1-8 /            ; US-ASCII control
///                   %d11 /             ;  characters that do not
///                   %d12 /             ;  include the carriage
///                   %d14-31 /          ;  return, line feed, and
///                   %d127              ;  white space characters
fn obs_no_ws_ctl(i: &[u8]) -> IResult<&[u8], char> {
    satisfy(|ch: char| ch.is_ascii() && is_obs_no_ws_ctl(ch as u8))(i)
}

/// WSP = SPACE | HTAB
fn wsp(i: &[u8]) -> IResult<&[u8], char> {
    one_of(" \t")(i)
}

/// VCHAR = %x21-7E ; visible (printing) characters
fn vchar(i: &[u8]) -> IResult<&[u8], char> {
    satisfy(|ch: char| ch.is_ascii() && is_vchar(ch as u8))(i)
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