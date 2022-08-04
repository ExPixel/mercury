/*
    Mercury SMTP Server
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

use std::collections::HashMap;

use nom::{
    branch::alt,
    bytes::complete::{tag, tag_no_case, take_while, take_while1, take_while_m_n},
    character::{
        complete::{alphanumeric1, char, satisfy},
        is_digit, is_hex_digit,
    },
    combinator::{eof, opt, recognize, value},
    multi::{many0_count, many1_count, separated_list1},
    sequence::{delimited, pair, preceded, separated_pair, terminated, tuple},
    IResult,
};

use super::reply::Code;

#[derive(Debug, PartialEq)]
#[allow(clippy::upper_case_acronyms)]
#[rustfmt::skip]
pub enum Command {
    EHLO { domain: String, },
    HELO { domain: String, },
    MAIL { reverse_path: String, mail_parameters: HashMap<String, String>, },
    RCPT { forward_path: String, rcpt_parameters: HashMap<String, String>, },
    DATA,
    RSET,
    VRFY { string: String, },
    EXPN { string: String, },
    HELP { string: String, },
    NOOP { string: String, },
    QUIT,
}

impl Command {
    pub fn parse<I: AsRef<[u8]>>(input: I) -> Result<Self, Code> {
        command(input.as_ref())
    }
}

#[derive(Clone, Copy)]
#[allow(clippy::upper_case_acronyms)]
enum CommandKind {
    EHLO,
    HELO,
    MAIL,
    RCPT,
    DATA,
    RSET,
    VRFY,
    EXPN,
    HELP,
    NOOP,
    QUIT,
}

fn command(i: &[u8]) -> Result<Command, Code> {
    let res = match command_name(i).map_err(|_| Code::UNRECOGNIZED_COMMAND)? {
        (i, CommandKind::EHLO) => ehlo(i),
        (i, CommandKind::HELO) => helo(i),
        (i, CommandKind::MAIL) => mail(i),
        (i, CommandKind::RCPT) => rcpt(i),
        (i, CommandKind::DATA) => data(i),
        (i, CommandKind::RSET) => rset(i),
        (i, CommandKind::VRFY) => vrfy(i),
        (i, CommandKind::EXPN) => expn(i),
        (i, CommandKind::HELP) => help(i),
        (i, CommandKind::NOOP) => noop(i),
        (i, CommandKind::QUIT) => quit(i),
    };
    let (i, cmd) = res.map_err(|_| Code::BAD_PARAMETER)?;
    let (i, _) = take_while::<_, _, nom::error::Error<&[u8]>>(|ch: u8| {
        ch != b'\r' && ch != b'\n' && ch.is_ascii_whitespace()
    })(i)
    .map_err(|_| Code::PARAMETER_NOT_IMPLEMENTED)?;
    let (i, _) =
        tag::<_, _, nom::error::Error<&[u8]>>("\r\n")(i).map_err(|_| Code::UNRECOGNIZED_COMMAND)?;
    eof::<_, nom::error::Error<&[u8]>>(i)
        .map(|_| cmd)
        .map_err(|_| Code::UNRECOGNIZED_COMMAND)
}

fn ehlo(i: &[u8]) -> IResult<&[u8], Command> {
    let (i, domain) = preceded(char(' '), alt((domain, address_literal)))(i)?;
    let domain = std::str::from_utf8(domain)
        .expect("domain not valid UTF-8")
        .to_owned();
    Ok((i, Command::EHLO { domain }))
}

fn helo(i: &[u8]) -> IResult<&[u8], Command> {
    let (i, domain) = preceded(char(' '), domain)(i)?;
    let domain = std::str::from_utf8(domain)
        .expect("domain not valid UTF-8")
        .to_owned();
    Ok((i, Command::HELO { domain }))
}

fn mail(i: &[u8]) -> IResult<&[u8], Command> {
    let (i, (rp, mp)) = preceded(
        tag_no_case(" FROM:"),
        pair(reverse_path, opt(preceded(char(' '), mail_parameters))),
    )(i)?;

    Ok((
        i,
        Command::MAIL {
            reverse_path: std::str::from_utf8(rp)
                .expect("reverse-path not valid UTF-8")
                .to_owned(),
            mail_parameters: mp.unwrap_or_default(),
        },
    ))
}

fn rcpt(i: &[u8]) -> IResult<&[u8], Command> {
    let postmaster_with_domnain = delimited(
        char('<'),
        recognize(pair(tag_no_case("Postmaster@"), domain)),
        char('>'),
    );
    let postmaster_no_domain = delimited(char('<'), tag_no_case("Postmaster"), char('>'));
    let forward_path_ext = alt((postmaster_with_domnain, postmaster_no_domain, forward_path));

    let (i, (fp, rp)) = preceded(
        tag_no_case(" TO:"),
        pair(forward_path_ext, opt(preceded(char(' '), rcpt_parameters))),
    )(i)?;

    Ok((
        i,
        Command::RCPT {
            forward_path: std::str::from_utf8(fp)
                .expect("reverse-path not valid UTF-8")
                .to_owned(),
            rcpt_parameters: rp.unwrap_or_default(),
        },
    ))
}

fn data(i: &[u8]) -> IResult<&[u8], Command> {
    Ok((i, Command::DATA))
}

fn rset(i: &[u8]) -> IResult<&[u8], Command> {
    Ok((i, Command::RSET))
}

fn vrfy(i: &[u8]) -> IResult<&[u8], Command> {
    let (i, o) = preceded(char(' '), string)(i)?;
    Ok((
        i,
        Command::VRFY {
            string: std::str::from_utf8(o)
                .expect("vrfy string not valid UTF-8")
                .to_owned(),
        },
    ))
}

fn expn(i: &[u8]) -> IResult<&[u8], Command> {
    let (i, o) = preceded(char(' '), string)(i)?;
    Ok((
        i,
        Command::EXPN {
            string: std::str::from_utf8(o)
                .expect("expn string not valid UTF-8")
                .to_owned(),
        },
    ))
}

fn help(i: &[u8]) -> IResult<&[u8], Command> {
    let (i, o) = opt(preceded(char(' '), string))(i)?;
    let string = o
        .map(|o| {
            std::str::from_utf8(o)
                .expect("help string not valid UTF-8")
                .to_owned()
        })
        .unwrap_or_else(String::new);
    Ok((i, Command::HELP { string }))
}

fn noop(i: &[u8]) -> IResult<&[u8], Command> {
    let (i, o) = opt(preceded(char(' '), string))(i)?;
    let string = o
        .map(|o| {
            std::str::from_utf8(o)
                .expect("noop string not valid UTF-8")
                .to_owned()
        })
        .unwrap_or_else(String::new);
    Ok((i, Command::NOOP { string }))
}

fn quit(i: &[u8]) -> IResult<&[u8], Command> {
    Ok((i, Command::QUIT))
}

fn command_name(i: &[u8]) -> IResult<&[u8], CommandKind> {
    alt((
        value(CommandKind::EHLO, tag_no_case("EHLO")),
        value(CommandKind::HELO, tag_no_case("HELO")),
        value(CommandKind::MAIL, tag_no_case("MAIL")),
        value(CommandKind::RCPT, tag_no_case("RCPT")),
        value(CommandKind::DATA, tag_no_case("DATA")),
        value(CommandKind::RSET, tag_no_case("RSET")),
        value(CommandKind::VRFY, tag_no_case("VRFY")),
        value(CommandKind::EXPN, tag_no_case("EXPN")),
        value(CommandKind::HELP, tag_no_case("HELP")),
        value(CommandKind::NOOP, tag_no_case("NOOP")),
        value(CommandKind::QUIT, tag_no_case("QUIT")),
    ))(i)
}

fn reverse_path(i: &[u8]) -> IResult<&[u8], &[u8]> {
    if let (i, Some(path)) = opt(path)(i)? {
        Ok((i, path))
    } else {
        let (i, _) = tag("<>")(i)?;
        Ok((i, &[]))
    }
}

fn forward_path(i: &[u8]) -> IResult<&[u8], &[u8]> {
    path(i)
}

fn mail_parameters(i: &[u8]) -> IResult<&[u8], HashMap<String, String>> {
    separated_list1(char(' '), esmtp_param)(i).map(|(i, p)| {
        (
            i,
            p.into_iter()
                .map(|(k, v)| {
                    let k = std::str::from_utf8(k)
                        .expect("esmtp-value invalid utf-8")
                        .to_owned();
                    let v = std::str::from_utf8(v)
                        .expect("esmtp-value invalid utf-8")
                        .to_owned();
                    (k, v)
                })
                .collect(),
        )
    })
}

fn rcpt_parameters(i: &[u8]) -> IResult<&[u8], HashMap<String, String>> {
    mail_parameters(i)
}

fn esmtp_param(i: &[u8]) -> IResult<&[u8], (&[u8], &[u8])> {
    separated_pair(esmtp_keyword, char('='), esmtp_value)(i)
}

fn esmtp_keyword(i: &[u8]) -> IResult<&[u8], &[u8]> {
    let first = satisfy(|ch| ch.is_ascii_alphanumeric());
    let rem = take_while(|ch: u8| ch == b'-' || ch.is_ascii_alphanumeric());
    recognize(pair(first, rem))(i)
}

fn esmtp_value(i: &[u8]) -> IResult<&[u8], &[u8]> {
    take_while1(|ch| matches!(ch, 33..=60 | 62..=126))(i)
}

fn path(i: &[u8]) -> IResult<&[u8], &[u8]> {
    delimited(pair(char('<'), opt(at_domain_list)), mailbox, char('>'))(i)
}

fn mailbox(i: &[u8]) -> IResult<&[u8], &[u8]> {
    recognize(separated_pair(
        local_part,
        char('@'),
        alt((domain, address_literal)),
    ))(i)
}

fn local_part(i: &[u8]) -> IResult<&[u8], &[u8]> {
    alt((dot_string, quoted_string))(i)
}

fn at_domain_list(i: &[u8]) -> IResult<&[u8], &[u8]> {
    recognize(pair(at_domain, many0_count(pair(char(':'), at_domain))))(i)
}

fn at_domain(i: &[u8]) -> IResult<&[u8], &[u8]> {
    preceded(char('@'), domain)(i)
}

fn domain(i: &[u8]) -> IResult<&[u8], &[u8]> {
    recognize(pair(subdomain, many0_count(pair(char('.'), subdomain))))(i)
}

fn subdomain(i: &[u8]) -> IResult<&[u8], &[u8]> {
    recognize(tuple((let_dig, ldh_str)))(i)
}

fn address_literal(i: &[u8]) -> IResult<&[u8], &[u8]> {
    delimited(
        char('['),
        alt((
            ipv4_address_literal,
            ipv6_address_literal,
            general_address_literal,
        )),
        char(']'),
    )(i)
}

fn ipv4_address_literal(i: &[u8]) -> IResult<&[u8], &[u8]> {
    recognize(pair(snum, repeat(3, preceded(char('.'), snum))))(i)
}

fn snum(i: &[u8]) -> IResult<&[u8], &[u8]> {
    take_while_m_n(1, 3, is_digit)(i)
}

fn ipv6_address_literal(i: &[u8]) -> IResult<&[u8], &[u8]> {
    preceded(tag_no_case("IPv6:"), ipv6_addr)(i)
}

fn ipv6_addr(i: &[u8]) -> IResult<&[u8], &[u8]> {
    alt((ipv6_full, ipv6_comp, ipv6v4_full, ipv6v4_comp))(i)
}

fn ipv6_hex(i: &[u8]) -> IResult<&[u8], &[u8]> {
    take_while_m_n(1, 4, is_hex_digit)(i)
}

#[rustfmt::skip]
fn ipv6_full(i: &[u8]) -> IResult<&[u8], &[u8]> {
    recognize(pair(ipv6_hex, repeat(7, preceded(char(':'), ipv6_hex))))(i)
}

fn ipv6_comp(i: &[u8]) -> IResult<&[u8], &[u8]> {
    let lhs = pair(ipv6_hex, repeat_m_n(0, 5, preceded(char(':'), ipv6_hex)));
    let rhs = pair(ipv6_hex, repeat_m_n(0, 5, preceded(char(':'), ipv6_hex)));
    recognize(tuple((opt(lhs), tag("::"), opt(rhs))))(i)
}

fn ipv6v4_full(i: &[u8]) -> IResult<&[u8], &[u8]> {
    let v6 = pair(ipv6_hex, repeat_m_n(0, 5, preceded(char(':'), ipv6_hex)));
    let v4 = ipv4_address_literal;
    recognize(tuple((v6, char(':'), v4)))(i)
}

fn ipv6v4_comp(i: &[u8]) -> IResult<&[u8], &[u8]> {
    let v6_lhs = pair(ipv6_hex, repeat_m_n(0, 3, preceded(char(':'), ipv6_hex)));
    let v6_rhs = pair(
        ipv6_hex,
        terminated(repeat_m_n(0, 3, preceded(char(':'), ipv6_hex)), char(':')),
    );
    recognize(tuple((
        opt(v6_lhs),
        tag("::"),
        opt(v6_rhs),
        ipv4_address_literal,
    )))(i)
}

fn general_address_literal(i: &[u8]) -> IResult<&[u8], &[u8]> {
    recognize(tuple((
        standardized_tag,
        char(':'),
        take_while1(is_dcontent),
    )))(i)
}

fn standardized_tag(i: &[u8]) -> IResult<&[u8], &[u8]> {
    ldh_str(i)
}

fn string(i: &[u8]) -> IResult<&[u8], &[u8]> {
    alt((atom, quoted_string))(i)
}

fn dot_string(i: &[u8]) -> IResult<&[u8], &[u8]> {
    recognize(pair(atom, many0_count(atom)))(i)
}

fn atom(i: &[u8]) -> IResult<&[u8], &[u8]> {
    take_while1(is_atext)(i)
}

fn quoted_string(i: &[u8]) -> IResult<&[u8], &[u8]> {
    delimited(char('"'), recognize(many0_count(qcontent_smtp)), char('"'))(i)
}

fn qcontent_smtp(i: &[u8]) -> IResult<&[u8], char> {
    alt((qtext_smtp, quoted_pair_smtp))(i)
}

fn qtext_smtp(i: &[u8]) -> IResult<&[u8], char> {
    satisfy(|ch| matches!(ch as u32, 32..=33 | 35..=91 | 93..=126))(i)
}

fn quoted_pair_smtp(i: &[u8]) -> IResult<&[u8], char> {
    preceded(
        char('\\'),
        satisfy(|ch| matches!(ch as u32, 32..=33 | 35..=91 | 93..=126)),
    )(i)
}

fn is_dcontent(ch: u8) -> bool {
    matches!(ch, 33..=90)
}

fn let_dig(i: &[u8]) -> IResult<&[u8], char> {
    satisfy(|ch| ch.is_ascii_alphanumeric())(i)
}

fn ldh_str(i: &[u8]) -> IResult<&[u8], &[u8]> {
    recognize(many1_count(alt((
        preceded(char('-'), alphanumeric1),
        alphanumeric1,
    ))))(i)
}

#[rustfmt::skip]
fn is_atext(ch: u8) -> bool {
    ch.is_ascii_alphanumeric()
        || matches!( ch,
            b'!' | b'#' | b'$' | b'%' | b'&' | b'\'' | b'*' | b'+' | b'-' | b'/' |
            b'=' | b'?' | b'^' | b'_' | b'`' | b'{' | b'|' | b'}' | b'~')
}

pub fn repeat<I, O, E, F>(mut count: usize, mut f: F) -> impl FnMut(I) -> IResult<I, O, E>
where
    I: Clone + PartialEq,
    O: Default,
    F: nom::Parser<I, O, E>,
    E: nom::error::ParseError<I>,
{
    move |i: I| {
        let (mut i, mut o) = (i, O::default());

        if count == 0 {
            return Ok((i, o));
        }

        while count > 0 {
            count -= 1;
            (i, o) = f.parse(i)?;
        }

        Ok((i, o))
    }
}

pub fn repeat_m_n<I, O, E, F>(min: usize, max: usize, mut f: F) -> impl FnMut(I) -> IResult<I, O, E>
where
    I: Clone + PartialEq,
    O: Default,
    F: nom::Parser<I, O, E>,
    E: nom::error::ParseError<I>,
{
    assert!(min <= max, "min <= max");

    move |i: I| {
        let (mut i, mut o) = (i, O::default());

        let mut iterations = 0;
        while iterations < max {
            iterations += 1;
            (i, o) = f.parse(i)?;
        }

        if iterations < min {
            return Err(nom::Err::Error(E::from_error_kind(
                i,
                nom::error::ErrorKind::Count,
            )));
        }

        Ok((i, o))
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn parse_ehlo_domain() {
        assert_eq!(
            Command::parse("EHLO rust-lang.org\r\n"),
            Ok(Command::EHLO {
                domain: "rust-lang.org".to_owned()
            })
        );
    }

    #[test]
    fn parse_ehlo_ipv4() {
        assert_eq!(
            Command::parse("EHLO [192.168.1.1]\r\n"),
            Ok(Command::EHLO {
                domain: "192.168.1.1".to_owned()
            })
        );
    }

    #[test]
    fn parse_mail_simple() {
        assert_eq!(
            Command::parse("MAIL FROM:<no-reply@rust-lang.org>\r\n"),
            Ok(Command::MAIL {
                reverse_path: "no-reply@rust-lang.org".to_owned(),
                mail_parameters: HashMap::new()
            })
        );
    }

    #[test]
    fn parse_mail_empty() {
        assert_eq!(
            Command::parse("MAIL FROM:<>\r\n"),
            Ok(Command::MAIL {
                reverse_path: "".to_owned(),
                mail_parameters: HashMap::new()
            })
        );
    }

    #[test]
    fn parse_mail_with_parameters() {
        let params = [
            ("first".to_owned(), "first_value".to_owned()),
            ("second".to_owned(), "second-value".to_owned()),
        ]
        .into_iter()
        .collect::<HashMap<String, String>>();

        assert_eq!(
            Command::parse(
                "MAIL FROM:<no-reply@rust-lang.org> first=first_value second=second-value\r\n"
            ),
            Ok(Command::MAIL {
                reverse_path: "no-reply@rust-lang.org".to_owned(),
                mail_parameters: params,
            })
        );
    }

    #[test]
    fn parse_rcpt_simple() {
        assert_eq!(
            Command::parse("RCPT TO:<no-reply@rust-lang.org>\r\n"),
            Ok(Command::RCPT {
                forward_path: "no-reply@rust-lang.org".to_owned(),
                rcpt_parameters: HashMap::new()
            })
        );
    }

    #[test]
    fn parse_rcpt_postmaster() {
        assert_eq!(
            Command::parse("RCPT TO:<Postmaster>\r\n"),
            Ok(Command::RCPT {
                forward_path: "Postmaster".to_owned(),
                rcpt_parameters: HashMap::new()
            })
        );
    }

    #[test]
    fn parse_rcpt_with_parameters() {
        let params = [
            ("first".to_owned(), "first_value".to_owned()),
            ("second".to_owned(), "second-value".to_owned()),
        ]
        .into_iter()
        .collect::<HashMap<String, String>>();

        assert_eq!(
            Command::parse(
                "RCPT TO:<no-reply@rust-lang.org> first=first_value second=second-value\r\n"
            ),
            Ok(Command::RCPT {
                forward_path: "no-reply@rust-lang.org".to_owned(),
                rcpt_parameters: params,
            })
        );
    }

    #[test]
    fn parse_data() {
        assert_eq!(Command::parse("DATA\r\n"), Ok(Command::DATA));
    }

    #[test]
    fn parse_rset() {
        assert_eq!(Command::parse("RSET\r\n"), Ok(Command::RSET));
    }

    #[test]
    fn parse_vrfy_simple() {
        assert_eq!(
            Command::parse("VRFY test\r\n"),
            Ok(Command::VRFY {
                string: "test".to_owned()
            })
        );
    }

    #[test]
    fn parse_vrfy_quoted() {
        assert_eq!(
            Command::parse("VRFY \"test\"\r\n"),
            Ok(Command::VRFY {
                string: "test".to_owned()
            })
        );
    }

    #[test]
    fn parse_expn_simple() {
        assert_eq!(
            Command::parse("EXPN test\r\n"),
            Ok(Command::EXPN {
                string: "test".to_owned()
            })
        );
    }

    #[test]
    fn parse_expn_quoted() {
        assert_eq!(
            Command::parse("EXPN \"test\"\r\n"),
            Ok(Command::EXPN {
                string: "test".to_owned()
            })
        );
    }

    #[test]
    fn parse_help_simple() {
        assert_eq!(
            Command::parse("HELP test\r\n"),
            Ok(Command::HELP {
                string: "test".to_owned()
            })
        );
    }

    #[test]
    fn parse_help_quoted() {
        assert_eq!(
            Command::parse("HELP \"test\"\r\n"),
            Ok(Command::HELP {
                string: "test".to_owned()
            })
        );
    }

    #[test]
    fn parse_help_empty() {
        assert_eq!(
            Command::parse("HELP\r\n"),
            Ok(Command::HELP {
                string: String::new()
            })
        );
    }

    #[test]
    fn parse_noop_simple() {
        assert_eq!(
            Command::parse("NOOP test\r\n"),
            Ok(Command::NOOP {
                string: "test".to_owned()
            })
        );
    }

    #[test]
    fn parse_noop_quoted() {
        assert_eq!(
            Command::parse("NOOP \"test\"\r\n"),
            Ok(Command::NOOP {
                string: "test".to_owned()
            })
        );
    }

    #[test]
    fn parse_noop_empty() {
        assert_eq!(
            Command::parse("NOOP\r\n"),
            Ok(Command::NOOP {
                string: String::new()
            })
        );
    }

    #[test]
    fn parse_quit() {
        assert_eq!(Command::parse("QUIT\r\n"), Ok(Command::QUIT));
    }
}
