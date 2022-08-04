use rusqlite::{Params, Result, Statement};
use std::{path::Path, sync::Arc};
use tokio::sync::{MappedMutexGuard, Mutex, MutexGuard};

pub struct Connection {
    inner: Arc<Mutex<rusqlite::Connection>>,
}

impl Connection {
    pub fn wrap(connection: rusqlite::Connection) -> Self {
        Connection {
            inner: Arc::new(Mutex::new(connection)),
        }
    }

    pub fn open<P: AsRef<Path>>(path: P) -> Result<Connection> {
        rusqlite::Connection::open(path).map(Self::wrap)
    }

    pub async fn execute<P: Params + Send>(&self, sql: &str, params: P) -> Result<usize> {
        let conn = self.inner.lock().await;
        conn.execute(sql, params)
    }

    pub async fn prepare(&self, sql: &str) -> Result<MappedMutexGuard<Statement<'_>>> {
        let conn = self.inner.lock().await;
        MutexGuard::try_map(conn, move |conn| conn.prepare(sql))
    }
}

fn try_mutex_map<T, U, E>(m: Mutex<T>) -> Result<MappedMutexGuard<U>, E>
where
    U: 'static,
{
    todo!();
}
