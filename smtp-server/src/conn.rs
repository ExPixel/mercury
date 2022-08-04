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

use std::time::Duration;

use tokio::io::{AsyncBufReadExt, AsyncWriteExt as _, BufStream};
use tokio::net::TcpStream;
use tracing::debug;

use crate::error::Result;
use crate::session::reply::Reply;
use crate::{Error, Session};

pub struct Connection {
    stream: BufStream<TcpStream>,
    session: Session,
    read_timeout: Duration,
    write_timeout: Duration,
}

impl Connection {
    pub fn new(stream: TcpStream, session: Session) -> Self {
        Connection {
            stream: BufStream::new(stream),
            session,
            read_timeout: Duration::from_secs(5),
            write_timeout: Duration::from_secs(5),
        }
    }

    pub async fn run(mut self) -> Result<()> {
        let mut reply = Reply::default();

        loop {
            self.session.on_recv(&mut reply);
            reply.finish();
            self.write_reply(&reply).await?;
            if self.session.closed() {
                break;
            }
            self.read_line().await?;
            reply.clear();
        }

        Ok(())
    }

    async fn write_reply(&mut self, reply: &Reply) -> Result<()> {
        debug!(
            count = display(reply.data().len()),
            bytes = debug(String::from_utf8_lossy(reply.data())),
            "sending",
        );

        tokio::time::timeout(self.write_timeout, async move {
            self.stream.get_mut().writable().await?;
            self.stream.write_all(reply.data()).await?;
            self.stream.flush().await?;
            Result::<()>::Ok(())
        })
        .await
        .map_err(|_| Error::WriteTimeout)??;

        Ok(())
    }

    async fn read_line(&mut self) -> Result<()> {
        let terminator = self.session.terminator();
        let terminator_end = *terminator
            .last()
            .expect("terminator must be at least 1 byte in length");
        let buffer = self.session.buffer_mut();

        tokio::time::timeout(self.read_timeout, self.stream.get_mut().readable())
            .await
            .map_err(|_| Error::ReadTimeout)??;

        loop {
            let count = tokio::time::timeout(self.read_timeout, {
                self.stream.read_until(terminator_end, buffer)
            })
            .await
            .map_err(|_| Error::ReadTimeout)??;

            debug!(
                count = display(count),
                bytes = debug(String::from_utf8_lossy(&buffer[(buffer.len() - count)..])),
                "received"
            );

            if count == 0 || buffer.ends_with(terminator) {
                break;
            }
        }
        Ok(())
    }
}
