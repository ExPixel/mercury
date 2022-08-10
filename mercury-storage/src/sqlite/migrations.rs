// SPDX-License-Identifier: GPL-3.0-or-later

use crate::error::{Error, Result};
use rusqlite::Connection;
use time::OffsetDateTime;
use tracing::{debug, trace};

struct Migration {
    name: &'static str,
    function: fn(connection: &mut Connection) -> rusqlite::Result<()>,
}

macro_rules! m {
    ($f:ident) => {
        Migration {
            name: stringify!($f),
            function: $f,
        }
    };
}

const MIGRATIONS: &[Migration] = &[
    m!(create_mail_table), // no fmt
];

pub fn migrate(conn: &mut Connection) -> Result<()> {
    debug!("ensuring migrations table exists...");
    create_migrations_table(conn).map_err(|e| Error::Sqlite(e, "creating migrations table"))?;

    for migration in MIGRATIONS {
        let migration_done = is_migration_done(conn, migration)
            .map_err(|e| Error::Sqlite(e, "checking if migration is done"))?;
        if migration_done {
            trace!("migration `{}` is done, skipping", migration.name);
            continue;
        }
        debug!("executing migration `{}`", migration.name);
        (migration.function)(conn).map_err(|e| Error::Sqlite(e, "executing migration"))?;
        record_migration_done(conn, migration)
            .map_err(|e| Error::Sqlite(e, "recording migration"))?;
    }
    Ok(())
}

fn create_migrations_table(conn: &mut Connection) -> rusqlite::Result<()> {
    let sql = "\
    CREATE TABLE IF NOT EXISTS migrations (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        name TEXT NOT NULL,
        migrated_at TEXT NOT NULL
    );";
    let mut statement = conn.prepare(sql)?;
    statement.execute(())?;
    Ok(())
}

fn is_migration_done(conn: &mut Connection, migration: &Migration) -> rusqlite::Result<bool> {
    let mut statement = conn.prepare("SELECT EXISTS (SELECT * FROM migrations WHERE name = ?);")?;
    statement.query_row([migration.name], |row| row.get::<usize, bool>(0))
}

fn record_migration_done(conn: &mut Connection, migration: &Migration) -> rusqlite::Result<()> {
    let mut statement = conn.prepare("INSERT INTO migrations (name, migrated_at) VALUES (?, ?)")?;
    statement.execute((migration.name, OffsetDateTime::now_utc()))?;
    Ok(())
}

fn create_mail_table(conn: &mut Connection) -> rusqlite::Result<()> {
    let sql = "CREATE TABLE mail (id INTEGER PRIMARY KEY, metadata TEXT, created_at TEXT);";
    let mut statement = conn.prepare(sql)?;
    statement.execute(())?;
    Ok(())
}
