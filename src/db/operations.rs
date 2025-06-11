use rusqlite::Connection;
use crate::error::{Result, PathBrainError};
use super::models::PathEntry;
use std::path::PathBuf;
use directories::ProjectDirs;
use super::schema::setup_schema;

pub fn get_database_path() -> Result<PathBuf> {
    let proj_dirs = ProjectDirs::from("com", "pathbrain", "pathbrain")
        .ok_or_else(|| PathBrainError::Other("Could not determine project directories".to_string()))?;
    
    let data_dir = proj_dirs.data_dir();
    std::fs::create_dir_all(data_dir)?;
    
    Ok(data_dir.join("pathbrain.db"))
}

pub fn open_db() -> Result<Connection> {
    let db_path = get_database_path()?;
    let conn = Connection::open(db_path)?;
    setup_schema(&conn)?;
    Ok(conn)
}

pub fn create_path_tag_entry(conn: &mut Connection, path: &str, tags: &[String], timestamp: i64) -> Result<()> {
    let tx = conn.transaction()?;
    
    tx.execute(
        "INSERT OR IGNORE INTO paths (path, last_used, freq) VALUES (?1, ?2, 1)",
        [path, &timestamp.to_string()],
    )?;

    tx.execute(
        "UPDATE paths SET last_used = ?1, freq = freq + 1 WHERE path = ?2 AND last_used < ?1",
        [&timestamp.to_string(), path],
    )?;

    let path_id: i64 = tx.query_row(
        "SELECT id FROM paths WHERE path = ?1",
        [path],
        |row| row.get(0),
    )?;

    for tag in tags {
        tx.execute(
            "INSERT OR IGNORE INTO tags (path_id, tag) VALUES (?1, ?2)",
            [&path_id.to_string(), tag],
        )?;
    }

    tx.commit()?;
    Ok(())
}

pub fn get_path(conn: &Connection, path: &str) -> Result<Option<PathEntry>> {
    let result = conn.query_row(
        "SELECT id, path, last_used, freq FROM paths WHERE path = ?1",
        [path],
        |row| {
            Ok(PathEntry {
                id: Some(row.get(0)?),
                path: row.get(1)?,
                last_used: row.get(2)?,
                freq: row.get(3)?,
            })
        },
    );

    match result {
        Ok(entry) => Ok(Some(entry)),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(e) => Err(e.into()),
    }
}

pub fn get_tags_for_path(conn: &Connection, path_id: i64) -> Result<Vec<String>> {
    let mut stmt = conn.prepare("SELECT tag FROM tags WHERE path_id = ?1")?;
    let tags = stmt.query_map([path_id], |row| row.get(0))?
        .collect::<std::result::Result<Vec<String>, _>>()?;
    Ok(tags)
}

pub fn find_paths_by_tag(conn: &Connection, tag: &str) -> Result<Vec<PathEntry>> {
    let mut stmt = conn.prepare(
        "SELECT p.id, p.path, p.last_used, p.freq 
         FROM paths p 
         JOIN tags t ON p.id = t.path_id 
         WHERE t.tag = ?1"
    )?;

    let paths = stmt.query_map([tag], |row| {
        Ok(PathEntry {
            id: Some(row.get(0)?),
            path: row.get(1)?,
            last_used: row.get(2)?,
            freq: row.get(3)?,
        })
    })?
    .collect::<std::result::Result<Vec<_>, _>>()?;

    Ok(paths)
}

pub fn list_paths(conn: &Connection) -> Result<Vec<PathEntry>> {
    let mut stmt = conn.prepare("SELECT id, path, last_used, freq FROM paths ORDER BY freq DESC")?;
    let paths = stmt.query_map([], |row| {
        Ok(PathEntry {
            id: Some(row.get(0)?),
            path: row.get(1)?,
            last_used: row.get(2)?,
            freq: row.get(3)?,
        })
    })?
    .collect::<std::result::Result<Vec<_>, _>>()?;

    Ok(paths)
}

pub fn remove_tags_from_path(conn: &mut Connection, path: &str, tags: &[String]) -> Result<()> {
    let path_id = get_path(conn, path)?
        .ok_or_else(|| PathBrainError::PathNotFound(path.to_string()))?
        .id
        .unwrap();

    let tx = conn.transaction()?;
    for tag in tags {
        tx.execute(
            "DELETE FROM tags WHERE path_id = ?1 AND tag = ?2",
            [&path_id.to_string(), tag],
        )?;
    }
    tx.commit()?;
    Ok(())
}

pub fn remove_all_tags_from_path(conn: &mut Connection, path: &str) -> Result<()> {
    let path_id = get_path(conn, path)?
        .ok_or_else(|| PathBrainError::PathNotFound(path.to_string()))?
        .id
        .unwrap();

    conn.execute("DELETE FROM tags WHERE path_id = ?1", [path_id])?;
    Ok(())
}

pub fn path_exists(conn: &Connection, path: &str) -> Result<bool> {
    let count: i64 = conn.query_row(
        "SELECT COUNT(*) FROM paths WHERE path = ?1",
        [path],
        |row| row.get(0),
    )?;
    Ok(count > 0)
}

pub fn bump_path_usage(conn: &mut Connection, path: &str, timestamp: i64) -> Result<()> {
    conn.execute(
        "UPDATE paths SET last_used = ?1, freq = freq + 1 WHERE path = ?2",
        [&timestamp.to_string(), path],
    )?;
    Ok(())
}

pub fn list_all_tags(conn: &Connection) -> Result<Vec<String>> {
    let mut stmt = conn.prepare("SELECT DISTINCT tag FROM tags")?;
    let tags = stmt.query_map([], |row| row.get(0))?
        .collect::<std::result::Result<Vec<String>, _>>()?;
    Ok(tags)
}

pub fn get_path_id(conn: &Connection, path: &str) -> Result<Option<i64>> {
    let mut stmt = conn.prepare("SELECT id FROM paths WHERE path = ?")?;
    let mut rows = stmt.query([path])?;
    
    if let Some(row) = rows.next()? {
        Ok(Some(row.get(0)?))
    } else {
        Ok(None)
    }
} 