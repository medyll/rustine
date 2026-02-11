use anyhow::{anyhow, Result};
use crossbeam_channel::{unbounded, Receiver, Sender};
use rusqlite::{params, Connection};
use std::thread;
use once_cell::sync::OnceCell;

#[derive(Debug, Clone)]
pub struct UrlRecord {
    pub id: i64,
    pub label: String,
    pub url: String,
    pub timestamp: i64,
}

enum DbRequest {
    Insert {
        label: String,
        url: String,
        timestamp: i64,
        resp: Sender<anyhow::Result<()>>,
    },
    ListRecent {
        limit: i64,
        resp: Sender<anyhow::Result<Vec<UrlRecord>>> ,
    },
    Delete {
        id: i64,
        resp: Sender<anyhow::Result<()>>,
    },
}

#[derive(Clone)]
pub struct DbHandle {
    tx: Sender<DbRequest>,
}

static GLOBAL_DB: OnceCell<DbHandle> = OnceCell::new();

pub fn set_global(db: DbHandle) -> Result<()> {
    GLOBAL_DB
        .set(db)
        .map_err(|_| anyhow::anyhow!("global DB handle already set"))
}

pub fn get_global() -> Option<DbHandle> {
    GLOBAL_DB.get().cloned()
}

impl DbHandle {
    pub fn insert_url(&self, label: &str, url: &str, timestamp: i64) -> Result<()> {
        let (tx, rx) = unbounded();
        let req = DbRequest::Insert {
            label: label.to_string(),
            url: url.to_string(),
            timestamp,
            resp: tx,
        };
        self.tx
            .send(req)
            .map_err(|e| anyhow!("Failed to send insert request: {}", e))?;
        rx.recv().map_err(|e| anyhow!("DB response recv failed: {}", e))?
    }

    pub fn list_recent(&self, limit: i64) -> Result<Vec<UrlRecord>> {
        let (tx, rx) = unbounded();
        let req = DbRequest::ListRecent { limit, resp: tx };
        self.tx
            .send(req)
            .map_err(|e| anyhow!("Failed to send list request: {}", e))?;
        Ok(rx.recv().map_err(|e| anyhow!("DB response recv failed: {}", e))??)
    }

    pub fn delete(&self, id: i64) -> Result<()> {
        let (tx, rx) = unbounded();
        let req = DbRequest::Delete { id, resp: tx };
        self.tx
            .send(req)
            .map_err(|e| anyhow!("Failed to send delete request: {}", e))?;
        rx.recv().map_err(|e| anyhow!("DB response recv failed: {}", e))?
    }
}

/// Initialize the DB actor: spawn a dedicated thread owning the Connection and return a `DbHandle`.
pub fn init_db() -> Result<DbHandle> {
    let (tx, rx): (Sender<DbRequest>, Receiver<DbRequest>) = unbounded();

    thread::spawn(move || {
        if let Err(e) = db_thread(rx) {
            eprintln!("DB thread error: {}", e);
        }
    });

    Ok(DbHandle { tx })
}

fn db_thread(rx: Receiver<DbRequest>) -> Result<()> {
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

    while let Ok(req) = rx.recv() {
        match req {
            DbRequest::Insert { label, url, timestamp, resp } => {
                let res = (|| -> Result<()> {
                    conn.execute(
                        "INSERT INTO urls (label, url, timestamp) VALUES (?1, ?2, ?3)",
                        params![label, url, timestamp],
                    )?;
                    Ok(())
                })();
                let _ = resp.send(res.map_err(|e| anyhow!(e.to_string())));
            }
            DbRequest::ListRecent { limit, resp } => {
                let res = (|| -> Result<Vec<UrlRecord>> {
                    let mut stmt = conn.prepare(
                        "SELECT id, label, url, timestamp FROM urls ORDER BY timestamp DESC LIMIT ?1",
                    )?;
                    let rows = stmt
                        .query_map(params![limit], |row| {
                            Ok(UrlRecord {
                                id: row.get(0)?,
                                label: row.get(1)?,
                                url: row.get(2)?,
                                timestamp: row.get(3)?,
                            })
                        })?
                        .collect::<Result<Vec<_>, rusqlite::Error>>()?;
                    Ok(rows)
                })();
                let _ = resp.send(res.map_err(|e| anyhow!(e.to_string())));
            }
            DbRequest::Delete { id, resp } => {
                let res = (|| -> Result<()> {
                    conn.execute("DELETE FROM urls WHERE id = ?1", params![id])?;
                    Ok(())
                })();
                let _ = resp.send(res.map_err(|e| anyhow!(e.to_string())));
            }
        }
    }

    Ok(())
}
