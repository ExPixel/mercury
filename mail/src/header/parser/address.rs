//    address         =   mailbox / group
//
//    mailbox         =   name-addr / addr-spec
//
//    name-addr       =   [display-name] angle-addr
//
//    angle-addr      =   [CFWS] "<" addr-spec ">" [CFWS] /
//                        obs-angle-addr
//
//    group           =   display-name ":" [group-list] ";" [CFWS]
//
//    display-name    =   phrase
//
//    mailbox-list    =   (mailbox *("," mailbox)) / obs-mbox-list
//
//    address-list    =   (address *("," address)) / obs-addr-list
//
//    group-list      =   mailbox-list / CFWS / obs-group-list
//    addr-spec       =   local-part "@" domain
//
//    local-part      =   dot-atom / quoted-string / obs-local-part
//
//    domain          =   dot-atom / domain-literal / obs-domain
//
//    domain-literal  =   [CFWS] "[" *([FWS] dtext) [FWS] "]" [CFWS]
//
//    dtext           =   %d33-90 /          ; Printable US-ASCII
//                        %d94-126 /         ;  characters not including
//                        obs-dtext          ;  "[", "]", or "\"

use nom::{branch::alt, character::complete::satisfy, IResult};

use super::{obs_no_ws_ctl, quoted_pair};

///dtext = %d33-90 /          ; Printable US-ASCII
///        %d94-126 /         ;  characters not including
///        obs-dtext          ;  "[", "]", or "\"
pub fn dtext(i: &[u8]) -> IResult<&[u8], char> {
    let current = satisfy(|ch: char| ch.is_ascii() && is_dtext(ch as u8));
    alt((current, obs_dtext))(i)
}

/// obs-dtext = obs-NO-WS-CTL / quoted-pair
pub fn obs_dtext(i: &[u8]) -> IResult<&[u8], char> {
    alt((obs_no_ws_ctl, quoted_pair))(i)
}

/// see: [`dtext`]
fn is_dtext(ch: u8) -> bool {
    matches!(ch, 33..=90 | 94..=126)
}
