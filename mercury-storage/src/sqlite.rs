pub mod migrations;

use crossbeam::channel;
use rusqlite::Connection;
use std::time::Duration;
use tokio::sync::oneshot;
use tracing::trace;

#[derive(Clone)]
pub struct SqliteStorage {
    sender: channel::Sender<Message>,
}

impl SqliteStorage {
    pub fn new(connection: Connection) -> Self {
        let (tx, rx) = channel::unbounded();
        tokio::task::spawn_blocking(move || sqlite_storage_loop(connection, rx));
        SqliteStorage { sender: tx }
    }

    pub async fn with<R, F>(&self, f: F) -> R
    where
        F: FnOnce(&mut Connection) -> R + Send + 'static,
        R: 'static + Send,
    {
        let (tx, rx) = oneshot::channel();
        self.with_void(move |conn| {
            let _ = tx.send((f)(conn));
        });
        rx.await.expect("sqlite task stopped unexpectedly")
    }

    pub fn with_void<F>(&self, f: F)
    where
        F: FnOnce(&mut Connection) + Send + 'static,
    {
        self.sender
            .send(Message {
                callback: Box::new(f),
            })
            .expect("sqlite task stopped unexpectedly")
    }
}

struct Message {
    callback: Box<dyn FnOnce(&mut Connection) + Send + 'static>,
}

fn sqlite_storage_loop(mut connection: Connection, receiver: channel::Receiver<Message>) {
    for message in receiver.into_iter() {
        (message.callback)(&mut connection);
    }
}

pub fn add_callbacks(conn: &mut Connection) {
    conn.profile(Some(|statement: &str, duration: Duration| {
        trace!(
            statement = display(statement),
            duration = debug(duration),
            "executed sqlite statement"
        );
    }));
}
