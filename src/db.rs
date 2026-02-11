use anyhow::Result;
use rusqlite::{params, Connection};
use std::sync::{Arc, Mutex};

pub type DbHandle = Arc<Mutex<Connection>>;

/// Initialize or open the database and create `urls` table if needed.
pub fn init_db() -> Result<DbHandle> {
    let conn = Connection::open("rustine.db")?;
    conn.execute(
        "CREATE TABLE IF NOT EXISTS urls (
            id INTEGER PRIMARY KEY,
            label TEXT NOT NULL,
            url TEXT NOT NULL,
            timestamp INTEGER NOT NULL
        )",
        params![],
    )?;
    Ok(Arc::new(Mutex::new(conn)))
}

/// Insert a URL record (simple API for scaffold)
pub fn insert_url(db: &DbHandle, label: &str, url: &str, timestamp: i64) -> Result<()> {
    let conn = db.lock().unwrap();
    conn.execute(
        "INSERT INTO urls (label, url, timestamp) VALUES (?1, ?2, ?3)",
        params![label, url, timestamp],
    )?;
    Ok(())
}

/// List recent URLs (limit)
pub fn list_recent(db: &DbHandle, limit: i64) -> Result<Vec<(i64, String, String, i64)>> {
    let conn = db.lock().unwrap();
    let mut stmt = conn.prepare("SELECT id, label, url, timestamp FROM urls ORDER BY timestamp DESC LIMIT ?1")?;
    let rows = stmt
        .query_map(params![limit], |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?)))?
        .collect::<Result<Vec<_>, rusqlite::Error>>()?;
    Ok(rows)
}

/// Delete by id
pub fn delete(db: &DbHandle, id: i64) -> Result<()> {
    let conn = db.lock().unwrap();
    conn.execute("DELETE FROM urls WHERE id = ?1", params![id])?;
    Ok(())
}
