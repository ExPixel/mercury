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

mod conn;
mod error;
mod session;

pub use error::Error;
pub use session::Session;

use error::Result;
use std::{
    net::{SocketAddr, ToSocketAddrs},
    sync::Arc,
};
use tokio::net::{TcpListener, TcpStream};
use tracing::trace;
use tracing_futures::Instrument as _;

use crate::conn::Connection;

type OnConnErr = dyn Fn(Error) + Send + Sync;
type OnNewMail = dyn Fn(RawMail) + Send + Sync;

pub struct Server {
    socket_addr: Vec<SocketAddr>,
    on_conn_err: Arc<OnConnErr>,
    on_new_mail: Arc<OnNewMail>,

    handle_tx: tokio::sync::mpsc::Sender<bool>,
    handle_rx: tokio::sync::mpsc::Receiver<bool>,
}

impl Server {
    pub fn builder() -> ServerBuilder {
        ServerBuilder::default()
    }

    pub async fn run(mut self) -> Result<()> {
        let listener = TcpListener::bind(&self.socket_addr[..]).await?;
        let local_addr = listener.local_addr().expect("no TCP listener local addr");
        tracing::info!(addr = display(local_addr), "starting SMTP server");
        'main_loop: loop {
            tokio::select! {
                accepted = listener.accept() => {
                    let (stream, addr) = accepted?;
                    self.accept(stream, addr);
                },
                Some(true) = self.handle_rx.recv() => break 'main_loop,
            };
        }
        tracing::info!(local_addr = display(local_addr), "stopping SMTP server");
        Ok(())
    }

    fn accept(&mut self, stream: TcpStream, addr: SocketAddr) {
        trace!("accepted connection from {}", addr);

        let sess = Session::new(self.on_new_mail.clone());
        let conn = Connection::new(stream, sess);
        let span = tracing::trace_span!("connection", addr = display(addr));
        let on_conn_err = self.on_conn_err.clone();
        let task = async move {
            if let Err(err) = conn.run().await {
                on_conn_err(err);
            }
            trace!("connection closed");
        }
        .instrument(span);
        tokio::task::spawn(task);
    }

    pub fn handle(&self) -> ServerHandle {
        ServerHandle {
            tx: self.handle_tx.clone(),
        }
    }
}

#[derive(Clone)]
pub struct ServerHandle {
    tx: tokio::sync::mpsc::Sender<bool>,
}

impl ServerHandle {
    pub fn done(&self) -> bool {
        self.tx.is_closed()
    }

    pub fn stop(&self) {
        let _ = self.tx.try_send(true);
    }
}

pub struct ServerBuilder {
    socket_addr: Result<Vec<SocketAddr>>,
    on_conn_err: Option<Arc<OnConnErr>>,
    on_new_mail: Option<Arc<OnNewMail>>,
}

impl ServerBuilder {
    pub fn on_conn_err<F>(mut self, f: F) -> Self
    where
        F: 'static + Fn(Error) + Clone + Send + Sync,
    {
        self.on_conn_err = Some(Arc::new(f));
        self
    }

    pub fn on_new_mail<F>(mut self, f: F) -> Self
    where
        F: 'static + Fn(RawMail) + Clone + Send + Sync,
    {
        self.on_new_mail = Some(Arc::new(f));
        self
    }

    pub fn bind<S>(mut self, addr: S) -> Self
    where
        S: 'static + ToSocketAddrs,
    {
        self.socket_addr = addr
            .to_socket_addrs()
            .map(|addrs| addrs.collect())
            .map_err(Error::from);
        self
    }

    pub fn build(self) -> Result<Server> {
        let on_conn_err = self.on_conn_err.unwrap_or_else(|| Arc::new(|_| {}));
        let on_new_mail = self.on_new_mail.unwrap_or_else(|| Arc::new(|_| {}));
        let socket_addr = self.socket_addr?;

        let (handle_tx, handle_rx) = tokio::sync::mpsc::channel(1);

        Ok(Server {
            socket_addr,
            on_conn_err,
            on_new_mail,

            handle_tx,
            handle_rx,
        })
    }
}

impl Default for ServerBuilder {
    fn default() -> Self {
        ServerBuilder {
            socket_addr: Ok(Vec::new()),
            on_conn_err: None,
            on_new_mail: None,
        }
    }
}

pub struct RawMail {
    pub reverse_path: String,
    pub forward_path: Vec<String>,
    pub data: Vec<u8>,
}

impl RawMail {
    pub fn new(reverse_path: String, forward_path: Vec<String>, data: Vec<u8>) -> Self {
        Self {
            reverse_path,
            forward_path,
            data,
        }
    }
}
