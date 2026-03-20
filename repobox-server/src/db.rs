use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

use rusqlite::{Connection, OptionalExtension, params};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct RepoRecord {
    pub address: String,
    pub name: String,
    pub owner_address: String,
    pub created_at: String,
}

pub(crate) fn init(db_path: &Path) -> std::io::Result<()> {
    let connection = Connection::open(db_path).map_err(to_io_error)?;
    connection
        .execute(
            "CREATE TABLE IF NOT EXISTS repos (
                address TEXT NOT NULL,
                name TEXT NOT NULL,
                owner_address TEXT NOT NULL,
                created_at TEXT NOT NULL,
                PRIMARY KEY(address, name)
            )",
            [],
        )
        .map_err(to_io_error)?;
    Ok(())
}

pub(crate) fn insert_repo_if_missing(
    db_path: &Path,
    address: &str,
    name: &str,
    owner_address: &str,
) -> std::io::Result<()> {
    let connection = Connection::open(db_path).map_err(to_io_error)?;
    connection
        .execute(
            "INSERT OR IGNORE INTO repos(address, name, owner_address, created_at)
             VALUES(?1, ?2, ?3, ?4)",
            params![address, name, owner_address, now_string()],
        )
        .map_err(to_io_error)?;
    Ok(())
}

pub(crate) fn get_repo(
    db_path: &Path,
    address: &str,
    name: &str,
) -> std::io::Result<Option<RepoRecord>> {
    let connection = Connection::open(db_path).map_err(to_io_error)?;
    connection
        .query_row(
            "SELECT address, name, owner_address, created_at
             FROM repos
             WHERE address = ?1 AND name = ?2",
            params![address, name],
            |row| {
                Ok(RepoRecord {
                    address: row.get(0)?,
                    name: row.get(1)?,
                    owner_address: row.get(2)?,
                    created_at: row.get(3)?,
                })
            },
        )
        .optional()
        .map_err(to_io_error)
}

pub(crate) fn find_repo_by_name(
    db_path: &Path,
    name: &str,
) -> std::io::Result<Option<RepoRecord>> {
    let connection = Connection::open(db_path).map_err(to_io_error)?;
    connection
        .query_row(
            "SELECT address, name, owner_address, created_at
             FROM repos
             WHERE name = ?1",
            params![name],
            |row| {
                Ok(RepoRecord {
                    address: row.get(0)?,
                    name: row.get(1)?,
                    owner_address: row.get(2)?,
                    created_at: row.get(3)?,
                })
            },
        )
        .optional()
        .map_err(to_io_error)
}

fn now_string() -> String {
    let seconds = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    seconds.to_string()
}

fn to_io_error(error: rusqlite::Error) -> std::io::Error {
    std::io::Error::other(error)
}
