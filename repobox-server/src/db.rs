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

    // Create push_log table if it doesn't exist
    connection
        .execute(
            "CREATE TABLE IF NOT EXISTS push_log (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                address TEXT NOT NULL,
                name TEXT NOT NULL,
                pusher_address TEXT,
                commit_hash TEXT,
                commit_message TEXT,
                pushed_at TEXT NOT NULL
            )",
            [],
        )
        .map_err(to_io_error)?;

    // Create aliases table
    connection
        .execute(
            "CREATE TABLE IF NOT EXISTS aliases (
                alias TEXT PRIMARY KEY,
                address TEXT NOT NULL,
                created_at TEXT NOT NULL
            )",
            [],
        )
        .map_err(to_io_error)?;

    // Create indexes for performance
    create_push_log_indexes(&connection).map_err(to_io_error)?;
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

pub(crate) fn insert_push_log(
    db_path: &Path,
    address: &str,
    name: &str,
    pusher_address: Option<&str>,
    commit_hash: Option<&str>,
    commit_message: Option<&str>,
) -> std::io::Result<()> {
    let connection = Connection::open(db_path).map_err(to_io_error)?;
    connection
        .execute(
            "INSERT INTO push_log(address, name, pusher_address, commit_hash, commit_message, pushed_at)
             VALUES(?1, ?2, ?3, ?4, ?5, ?6)",
            params![
                address,
                name,
                pusher_address,
                commit_hash,
                commit_message,
                now_string()
            ],
        )
        .map_err(to_io_error)?;
    Ok(())
}

fn create_push_log_indexes(connection: &Connection) -> Result<(), rusqlite::Error> {
    connection.execute(
        "CREATE INDEX IF NOT EXISTS idx_push_log_timestamp ON push_log(pushed_at DESC)",
        [],
    )?;
    connection.execute(
        "CREATE INDEX IF NOT EXISTS idx_push_log_repo ON push_log(address, name)",
        [],
    )?;
    Ok(())
}

fn to_io_error(error: rusqlite::Error) -> std::io::Error {
    std::io::Error::other(error)
}

/// Generate and assign a random alias for an address
pub(crate) fn assign_random_alias(db_path: &Path, address: &str) -> std::io::Result<String> {
    // Use the new word lists to generate a deterministic alias
    let mut alias = crate::words::generate_alias_from_address(address);

    // If alias already exists, append a number
    let mut counter = 1;
    while resolve_alias(db_path, &alias)?.is_some() {
        alias = format!("{}-{}", crate::words::generate_alias_from_address(address), counter);
        counter += 1;
    }

    // Insert the alias
    let connection = Connection::open(db_path).map_err(to_io_error)?;
    connection
        .execute(
            "INSERT INTO aliases(alias, address, created_at) VALUES(?1, ?2, ?3)",
            params![alias, address, now_string()],
        )
        .map_err(to_io_error)?;

    Ok(alias)
}

/// Resolve an alias to an address
pub(crate) fn resolve_alias(db_path: &Path, alias: &str) -> std::io::Result<Option<String>> {
    let connection = Connection::open(db_path).map_err(to_io_error)?;
    connection
        .query_row(
            "SELECT address FROM aliases WHERE alias = ?1",
            params![alias],
            |row| Ok(row.get::<_, String>(0)?),
        )
        .optional()
        .map_err(to_io_error)
}

/// Get the alias for an address (if any)
pub(crate) fn get_alias_for_address(db_path: &Path, address: &str) -> std::io::Result<Option<String>> {
    let connection = Connection::open(db_path).map_err(to_io_error)?;
    connection
        .query_row(
            "SELECT alias FROM aliases WHERE address = ?1",
            params![address],
            |row| Ok(row.get::<_, String>(0)?),
        )
        .optional()
        .map_err(to_io_error)
}

/// Validate if an alias has the correct format
pub(crate) fn is_valid_alias_format(alias: &str) -> bool {
    crate::words::is_valid_alias(alias)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempdir::TempDir;

    #[test]
    fn test_assign_and_resolve_alias() {
        let temp_dir = TempDir::new("repobox_test").unwrap();
        let db_path = temp_dir.path().join("test.db");

        // Initialize the database
        init(&db_path).unwrap();

        // Test address
        let address = "0x1234567890123456789012345678901234567890";

        // Assign an alias
        let alias = assign_random_alias(&db_path, address).unwrap();
        assert!(!alias.is_empty());
        assert!(alias.contains('-')); // Should be adjective-animal format

        // Resolve the alias back to address
        let resolved = resolve_alias(&db_path, &alias).unwrap();
        assert_eq!(resolved, Some(address.to_string()));

        // Get alias for address
        let found_alias = get_alias_for_address(&db_path, address).unwrap();
        assert_eq!(found_alias, Some(alias));

        // Test non-existent alias
        let no_alias = resolve_alias(&db_path, "non-existent-alias").unwrap();
        assert_eq!(no_alias, None);
    }

    #[test]
    fn test_deterministic_alias_generation() {
        let temp_dir = TempDir::new("repobox_test").unwrap();
        let db_path = temp_dir.path().join("test.db");

        init(&db_path).unwrap();

        let address = "0x1234567890123456789012345678901234567890";

        // Generate alias twice for the same address
        let alias1 = assign_random_alias(&db_path, address).unwrap();

        // Clear the database and regenerate
        std::fs::remove_file(&db_path).unwrap();
        init(&db_path).unwrap();

        let alias2 = assign_random_alias(&db_path, address).unwrap();

        // Should be the same (deterministic based on address)
        assert_eq!(alias1, alias2);
    }
}
