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

use std::num::{NonZeroU16, NonZeroUsize};

#[derive(Default)]
pub struct Reply {
    code: Option<Code>,
    data: Vec<u8>,
    dash: Option<NonZeroUsize>,
}

impl Reply {
    pub fn data(&self) -> &[u8] {
        &self.data[..]
    }

    pub fn finish(&mut self) {
        assert!(!self.is_empty(), "reply must not be empty in finish");
        if !self.data.is_empty() {
            return;
        }
        self.line(self.code.and_then(|code| code.text()).unwrap_or(""))
    }

    pub fn line<D: AsRef<[u8]>>(&mut self, data: D) {
        use std::io::Write as _;

        let code = self.code.expect("must provide a code before reply text");
        write!(self.data, "{} ", u16::from(code)).expect("failed to write code to reply");

        if let Some(dash) = self.dash.take().map(NonZeroUsize::get) {
            self.data[dash] = b'-';
        }
        self.dash = NonZeroUsize::new(self.data.len() - 1);
        self.data.extend(data.as_ref());
        self.data.extend(b"\r\n");
    }

    pub fn code(&mut self, code: Code) {
        self.code = Some(code);
    }

    pub fn clear(&mut self) {
        self.code = None;
        self.dash = None;
        self.data.clear();
    }

    pub fn is_empty(&self) -> bool {
        self.code.is_none() && self.data.is_empty()
    }
}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub struct Code(NonZeroU16);

impl Code {
    pub fn text(&self) -> Option<&'static str> {
        let internal = self.text_internal();
        (!internal.is_empty()).then_some(internal)
    }
}

impl From<Code> for u16 {
    fn from(code: Code) -> u16 {
        code.0.get()
    }
}

macro_rules! define_codes {
    (
        $(
            $(#[$meta:meta])*
            ($code:expr, $name:ident, $text:expr)
        )+
    ) => {
        impl Code {
            $(
                pub const $name: Code = Code(unsafe { NonZeroU16::new_unchecked($code) });
            )+

            fn text_internal(&self) -> &'static str {
                match self.0.get() {
                    $( $code => $text ),+,
                    _ => "",
                }
            }
        }

        impl TryFrom<u16> for Code {
            type Error = InvalidCode;

            fn try_from(code: u16) -> Result<Code, Self::Error> {
                match code {
                    $( $code => Ok(Self::$name) ),+,
                    _ => Err(InvalidCode { _priv: () }),
                }
            }
        }
    };
}

pub struct InvalidCode {
    _priv: (),
}

define_codes! {
    /// 211 System status or system help reply.
    (211, SYSTEM_STATUS, "")

    /// 214 Help message (Information on how to use the receiver or the
    /// meaning of a particular non-standard command; this reply is useful
    /// only to the human user)
    (214, HELP_MESSAGE, "")

    /// 220 Service Ready
    (220, SERVICE_READY, "service ready")

    /// 221 Service closing transmissiong channel
    (221, SERVICE_CLOSING, "service closing transmission channel")

    /// 250 Requested mail action okay, completed
    (250, MAIL_ACTION_OKAY, "requested mail action okay")

    /// 251 User not local; will forward to `<forward-path>`
    (251, USER_NOT_LOCAL_FORWARD, "user not local; will forward")

    /// 252 Cannot VRFY user, but will accept message and attempt delivery
    (252, CANNOT_VRFY_ACCEPT, "cannot VRFY user, will attempt delivery")

    /// 354 Start mail input; end with `<CRLF>.<CRLF>`
    (354, START_MAIL_INPUT, "start mail input")

    /// 421 Service not available, closing transmission channel.
    ///
    /// This may be a reply to any command if the service knows it must
    /// shut down.
    (421, SERVICE_NOT_AVAILABLE, "service not available, closing transmission channel")

    /// 450 Requested action not taken: mailbox unavailable
    ///
    /// e.g. Mailbody busy or temporarily blocked for policy reasons.
    (450, MAILBOX_BUSY, "mail action not taken: mailbox unavailable")

    /// 451 Requested action aborted: local error in processing.
    (451, LOCAL_ERROR_IN_PROCESSING, "action aborted: local error in processing")

    /// 452 Requested action not taken: insufficient system storage
    (452, INSUFFICIENT_SYSTEM_STORAGE, "action not taken: insufficient system storage")

    /// 455 Server unable to accomodate parameters
    (455, UNABLE_TO_ACCOMODATE_PARAMETERS, "server unable to accomodate parameters")

    /// 500 Syntax error, command unrecognized
    ///
    /// This may include errors such as a command line too long.
    (500, UNRECOGNIZED_COMMAND, "syntax error, command unrecognized")

    /// 501 Syntax error in parameters or arguments
    (501, BAD_PARAMETER, "syntax error in parameters or arguments")

    /// 502 Command not implemented
    (502, COMMAND_NOT_IMPLEMENTED, "command not implemented")

    /// 503 Bad sequence of commands
    (503, BAD_SEQUENCE_OF_COMMANDS, "bad sequence of commands")

    /// 504 Command parameter not implemented
    (504, PARAMETER_NOT_IMPLEMENTED, "command parameter not implemented")

    /// 550 Requested action not taken: mailbox unavailable
    ///
    /// e.g. mailbox not found, no access, or command rejected for policy reasons.
    (550, MAILBOX_UNAVAILABLE, "action not taken: mailbox unavailable")

    /// 551 User not local
    (551, USER_NOT_LOCAL, "user not local")

    /// 552 Requested mail action aborted: exceeded storage allocation
    (552, EXCEEDED_STORAGE_ALLOCATION, "mail action aborted: exceeded storage allocation")

    /// 553 Requested action not taken: mailbox name not allowed
    ///
    /// e.g. mailbox syntax incorrect
    (553, MAILBOX_NAME_NOT_ALLOWED, "action not taken: mailbox name not allowed")

    /// 554 Transaction failed
    ///
    /// Or, in the case of a connection-opening response, "No SMTP service here"
    (554, TRANSACTION_FAILED, "transaction failed")

    /// 555 MAIL FROM/RCPT TO parameters not recognized or not implemented
    (555, MAIL_FROM_RCPT_TO_NOT_IMPLEMENTED, "MAIL FROM/RCPT TO parameters not recognized or not implemented")
}
