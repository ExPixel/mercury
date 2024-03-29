// SPDX-License-Identifier: GPL-3.0-or-later

use std::{collections::HashMap, sync::Arc};

use tracing::debug;

use crate::{OnNewMail, RawMail};

use self::{
    cmd::Command,
    reply::{Code, Reply},
};

pub mod cmd;
pub mod reply;

pub struct Session {
    mode: Mode,
    line_buffer: Vec<u8>,
    data_buffer: Vec<u8>,
    reverse_path: String,
    forward_path: Vec<String>,
    closed: bool,
    on_new_mail: Arc<OnNewMail>,
}

impl Session {
    pub fn new(on_new_mail: Arc<OnNewMail>) -> Self {
        Session {
            mode: Mode::Open,
            line_buffer: Vec::with_capacity(64),
            data_buffer: Vec::new(),
            reverse_path: String::new(),
            forward_path: Vec::with_capacity(1),
            closed: false,
            on_new_mail,
        }
    }

    pub fn on_recv(&mut self, reply: &mut Reply) {
        match self.mode {
            Mode::Open => self.on_open(reply),
            Mode::Line => self.on_line(reply),
            Mode::Data => self.on_data(reply),
        }
    }

    fn on_open(&mut self, reply: &mut Reply) {
        reply.code(Code::SERVICE_READY);
        self.mode = Mode::Line;
    }

    fn on_line(&mut self, reply: &mut Reply) {
        match Command::parse(&self.line_buffer) {
            Ok(cmd) => self.handle_command(reply, cmd),
            Err(code) => reply.code(code),
        }
        self.line_buffer.clear();
    }

    fn on_data(&mut self, reply: &mut Reply) {
        debug!(size = self.data_buffer.len(), "received data");

        let mut mail = RawMail::new(
            std::mem::take(&mut self.reverse_path),
            std::mem::take(&mut self.forward_path),
            std::mem::take(&mut self.data_buffer),
        );

        if mail.data.ends_with(DATA_TERMINATOR) {
            mail.data.truncate(mail.data.len() - DATA_TERMINATOR.len());
        }

        (self.on_new_mail)(mail);

        self.mode = Mode::Line;
        reply.code(Code::MAIL_ACTION_OKAY);
    }

    fn handle_command(&mut self, reply: &mut Reply, cmd: Command) {
        match cmd {
            Command::EHLO { domain } => self.handle_ehlo(reply, domain),
            Command::HELO { domain } => self.handle_helo(reply, domain),
            Command::MAIL {
                reverse_path,
                mail_parameters,
            } => self.handle_mail(reply, reverse_path, mail_parameters),
            Command::RCPT {
                forward_path,
                rcpt_parameters,
            } => self.handle_rcpt(reply, forward_path, rcpt_parameters),
            Command::DATA => self.handle_data(reply),
            Command::RSET => self.handle_rset(reply),
            Command::NOOP { string } => self.handle_noop(reply, string),
            Command::QUIT => self.handle_quit(reply),

            // TODO implement remaining commands: VRFY, EXPN, HELP
            _ => reply.code(Code::COMMAND_NOT_IMPLEMENTED),
        }
    }

    fn handle_ehlo(&mut self, reply: &mut Reply, domain: String) {
        debug!(domain = debug(domain), "EHLO");
        reply.code(Code::MAIL_ACTION_OKAY);
    }

    fn handle_helo(&mut self, reply: &mut Reply, domain: String) {
        debug!(domain = debug(domain), "HELO");
        reply.code(Code::MAIL_ACTION_OKAY);
    }

    fn handle_mail(&mut self, reply: &mut Reply, path: String, params: HashMap<String, String>) {
        debug!(path = debug(&path), params = debug(&params), "MAIL");
        self.forward_path.clear();
        self.data_buffer.clear();
        self.reverse_path = path;
        reply.code(Code::MAIL_ACTION_OKAY);
    }

    fn handle_rcpt(&mut self, reply: &mut Reply, path: String, params: HashMap<String, String>) {
        debug!(path = debug(&path), params = debug(&params), "RCPT");
        self.forward_path.push(path);
        reply.code(Code::MAIL_ACTION_OKAY);
    }

    fn handle_data(&mut self, reply: &mut Reply) {
        debug!("DATA");
        self.mode = Mode::Data;
        reply.code(Code::START_MAIL_INPUT);
    }

    fn handle_rset(&mut self, reply: &mut Reply) {
        debug!("RSET");
        self.reverse_path.clear();
        self.forward_path.clear();
        self.data_buffer.clear();
        reply.code(Code::MAIL_ACTION_OKAY);
    }

    fn handle_noop(&mut self, reply: &mut Reply, string: String) {
        debug!(string = debug(string), "NOOP");
        reply.code(Code::MAIL_ACTION_OKAY);
    }

    fn handle_quit(&mut self, reply: &mut Reply) {
        debug!("QUIT");
        self.closed = true;
        reply.code(Code::SERVICE_CLOSING);
    }

    pub fn terminator(&self) -> &'static [u8] {
        match self.mode {
            Mode::Open => unreachable!("no terminator while in open mode"),
            Mode::Line => LINE_TERMINATOR,
            Mode::Data => DATA_TERMINATOR,
        }
    }

    pub fn buffer_mut(&mut self) -> &mut Vec<u8> {
        match self.mode {
            Mode::Open => unreachable!("no buffer while in open mode"),
            Mode::Line => &mut self.line_buffer,
            Mode::Data => &mut self.data_buffer,
        }
    }

    pub fn closed(&self) -> bool {
        self.closed
    }
}

#[derive(Default)]
pub enum Mode {
    #[default]
    Open,
    Line,
    Data,
}

const LINE_TERMINATOR: &[u8] = b"\r\n";
const DATA_TERMINATOR: &[u8] = b"\r\n.\r\n";
