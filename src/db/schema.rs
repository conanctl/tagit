use rusqlite::Connection;
use crate::error::Result;

const SCHEMA: &str = "
    PRAGMA foreign_keys = ON;

    CREATE TABLE IF NOT EXISTS paths (
        id INTEGER PRIMARY KEY,
        path TEXT UNIQUE,
        last_used INTEGER,
        freq INTEGER DEFAULT 0
    );

    CREATE TABLE IF NOT EXISTS tags (
        id INTEGER PRIMARY KEY,
        path_id INTEGER,
        tag TEXT,
        UNIQUE(path_id, tag),
        FOREIGN KEY(path_id) REFERENCES paths(id) ON DELETE CASCADE
    );
";

pub fn initialize_db() -> Result<Connection> {
    let conn = Connection::open_in_memory()?;
    conn.execute_batch(SCHEMA)?;
    Ok(conn)
}

pub fn setup_schema(conn: &Connection) -> Result<()> {
    conn.execute_batch(SCHEMA)?;
    Ok(())
} 