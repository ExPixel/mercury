// SPDX-License-Identifier: GPL-3.0-or-later

use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::char,
    combinator::{map, opt, recognize},
    multi::{fold_many0, many0_count, many1_count, separated_list1},
    sequence::{delimited, pair, preceded, separated_pair, terminated, tuple},
    IResult,
};

use crate::header::parts::{Address, Group, Mailbox};

use super::{
    atom, byte, cfws, dot_atom, fws, obs_no_ws_ctl, phrase, quoted_pair, quoted_string, satisfy_u8,
    word,
};

pub fn address(i: &[u8]) -> IResult<&[u8], Address> {
    alt((map(mailbox, Address::Mailbox), map(group, Address::Group)))(i)
}

pub(crate) fn mailbox(i: &[u8]) -> IResult<&[u8], Mailbox> {
    alt((
        map(name_addr, |(d, a)| Mailbox::new_raw(d, a)),
        map(addr_spec, |a| Mailbox::new_raw(None, a)),
    ))(i)
}

/// mailbox-list = (mailbox *("," mailbox)) / obs-mbox-list
pub(crate) fn mailbox_list(i: &[u8]) -> IResult<&[u8], Vec<Mailbox>> {
    alt((separated_list1(char(','), mailbox), obs_mbox_list))(i)
}

/// obs-mbox-list = *([CFWS] ",") mailbox *("," [mailbox / CFWS])
fn obs_mbox_list(i: &[u8]) -> IResult<&[u8], Vec<Mailbox>> {
    let (i, _) = many0_count(pair(opt(cfws), char(',')))(i)?;
    let mut list = Vec::new();
    let (i, mbox) = mailbox(i)?;
    list.push(mbox);
    let (i, _) = many0_count(preceded(
        char(','),
        opt(alt((map(mailbox, |m| list.push(m)), map(cfws, |_| ())))),
    ))(i)?;
    Ok((i, list))
}

/// address-list = (address *("," address)) / obs-addr-list
pub fn address_list(i: &[u8]) -> IResult<&[u8], Vec<Address>> {
    alt((separated_list1(char(','), address), obs_addr_list))(i)
}

// obs-addr-list = *([CFWS] ",") address *("," [address / CFWS])
pub fn obs_addr_list(i: &[u8]) -> IResult<&[u8], Vec<Address>> {
    let (i, _) = many0_count(pair(opt(cfws), char(',')))(i)?;
    let mut list = Vec::new();
    let (i, addr) = address(i)?;
    list.push(addr);
    let (i, _) = many0_count(preceded(
        char(','),
        opt(alt((map(address, |a| list.push(a)), map(cfws, |_| ())))),
    ))(i)?;
    Ok((i, list))
}

#[allow(clippy::type_complexity)]
fn name_addr(i: &[u8]) -> IResult<&[u8], (Option<Vec<u8>>, &[u8])> {
    pair(opt(display_name), angle_addr)(i)
}

/// angle-addr = [CFWS] "<" addr-spec ">" [CFWS] /
///              obs-angle-addr
fn angle_addr(i: &[u8]) -> IResult<&[u8], &[u8]> {
    let current = delimited(
        opt(cfws),
        delimited(char('<'), addr_spec, char('>')),
        opt(cfws),
    );
    alt((current, obs_angle_addr))(i)
}

/// obs-angle-addr = [CFWS] "<" obs-route addr-spec ">" [CFWS]
fn obs_angle_addr(i: &[u8]) -> IResult<&[u8], &[u8]> {
    delimited(
        opt(cfws),
        delimited(char('<'), preceded(obs_route, addr_spec), char('>')),
        opt(cfws),
    )(i)
}

/// obs-route = obs-domain-list ":"
fn obs_route(i: &[u8]) -> IResult<&[u8], &[u8]> {
    terminated(obs_domain_list, char(':'))(i)
}

// obs-domain-list = *(CFWS / ",") "@" domain
//                   *("," [CFWS] ["@" domain])
fn obs_domain_list(i: &[u8]) -> IResult<&[u8], &[u8]> {
    recognize(pair(
        separated_pair(many0_count(alt((cfws, tag(",")))), char('@'), domain),
        many0_count(tuple((
            char(','),
            opt(cfws),
            opt(preceded(char('@'), domain)),
        ))),
    ))(i)
}

// group = display-name ":" [group-list] ";" [CFWS]
fn group(i: &[u8]) -> IResult<&[u8], Group> {
    map(
        separated_pair(
            display_name,
            char(':'),
            terminated(opt(group_list), pair(char(';'), opt(cfws))),
        ),
        |(d, g)| Group::new_raw(Some(d), g.unwrap_or_default()),
    )(i)
}

fn group_list(i: &[u8]) -> IResult<&[u8], Vec<Mailbox>> {
    alt((mailbox_list, map(cfws, |_| Vec::new()), obs_group_list))(i)
}

fn obs_group_list(i: &[u8]) -> IResult<&[u8], Vec<Mailbox>> {
    let ws = terminated(many1_count(terminated(opt(cfws), char(','))), opt(cfws));
    map(ws, |_| Vec::new())(i)
}

fn display_name(i: &[u8]) -> IResult<&[u8], Vec<u8>> {
    phrase(i)
}

fn addr_spec(i: &[u8]) -> IResult<&[u8], &[u8]> {
    recognize(separated_pair(local_part, char('@'), domain))(i)
}

// local-part = dot-atom / quoted-string / obs-local-part
fn local_part(i: &[u8]) -> IResult<&[u8], Vec<u8>> {
    alt((map(dot_atom, |s| s.to_vec()), quoted_string, obs_local_part))(i)
}

/// obs-local-part = word *("." word)
fn obs_local_part(i: &[u8]) -> IResult<&[u8], Vec<u8>> {
    let (i, mut out) = word(i)?;
    fold_many0(
        preceded(byte(b'.'), word),
        move || std::mem::take(&mut out),
        |mut acc, s| {
            acc.extend(s.into_iter());
            acc
        },
    )(i)
}

// domain = dot-atom / domain-literal / obs-domain
fn domain(i: &[u8]) -> IResult<&[u8], &[u8]> {
    alt((dot_atom, domain_literal, obs_domain))(i)
}

// obs-domain = atom *("." atom)
fn obs_domain(i: &[u8]) -> IResult<&[u8], &[u8]> {
    recognize(pair(atom, many0_count(preceded(char('.'), atom))))(i)
}

// domain-literal = [CFWS] "[" *([FWS] dtext) [FWS] "]" [CFWS]
fn domain_literal(i: &[u8]) -> IResult<&[u8], &[u8]> {
    let inner = recognize(many0_count(preceded(opt(fws), dtext)));
    delimited(opt(cfws), delimited(char('['), inner, char(']')), opt(cfws))(i)
}

///dtext = %d33-90 /          ; Printable US-ASCII
///        %d94-126 /         ;  characters not including
///        obs-dtext          ;  "[", "]", or "\"
fn dtext(i: &[u8]) -> IResult<&[u8], u8> {
    let current = satisfy_u8(is_dtext);
    alt((current, obs_dtext))(i)
}

/// obs-dtext = obs-NO-WS-CTL / quoted-pair
fn obs_dtext(i: &[u8]) -> IResult<&[u8], u8> {
    alt((obs_no_ws_ctl, quoted_pair))(i)
}

/// see: [`dtext`]
fn is_dtext(ch: u8) -> bool {
    matches!(ch, 33..=90 | 94..=126)
}
