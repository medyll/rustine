use anyhow::{anyhow, Result};
use crossbeam_channel::{unbounded, Receiver, Sender};
use rusqlite::{params, Connection};
use std::thread;
use once_cell::sync::OnceCell;

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct SiteMeta {
    pub id: i64,
    pub origin: String,
    pub site_name: Option<String>,
    pub description: Option<String>,
    pub manifest_url: Option<String>,
    pub metadata_fetched_at: Option<i64>,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct Icon {
    pub id: i64,
    pub site_id: i64,
    pub src_url: String,
    pub width: Option<i64>,
    pub height: Option<i64>,
    pub mime: Option<String>,
    pub data: Vec<u8>,
    pub fetched_at: Option<i64>,
}

#[derive(Debug, Clone)]
pub struct UrlRecord {
    pub id: i64,
    pub label: String,
    pub url: String,
    pub _timestamp: i64,
    pub site_name: Option<String>,
    pub icon_mime: Option<String>,
    pub icon_data: Option<Vec<u8>>,
}

#[allow(dead_code)]
enum DbRequest {
    Insert {
        label: String,
        url: String,
        _timestamp: i64,
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
    GetById {
        id: i64,
        resp: Sender<anyhow::Result<Option<UrlRecord>>> ,
    },
    UpsertSiteMeta {
        origin: String,
        site_name: Option<String>,
        description: Option<String>,
        manifest_url: Option<String>,
        metadata_fetched_at: Option<i64>,
        resp: Sender<anyhow::Result<()>>,
    },
    GetSiteMetaByOrigin {
        origin: String,
        resp: Sender<anyhow::Result<Option<SiteMeta>>> ,
    },
    InsertIcon {
        site_id: i64,
        src_url: String,
        width: Option<i64>,
        height: Option<i64>,
        mime: Option<String>,
        data: Vec<u8>,
        fetched_at: Option<i64>,
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
            _timestamp: timestamp,
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

    pub fn get_by_id(&self, id: i64) -> Result<Option<UrlRecord>> {
        let (tx, rx) = unbounded();
        let req = DbRequest::GetById { id, resp: tx };
        self.tx
            .send(req)
            .map_err(|e| anyhow!("Failed to send get_by_id request: {}", e))?;
        Ok(rx.recv().map_err(|e| anyhow!("DB response recv failed: {}", e))??)
    }

    pub fn upsert_site_meta(&self, origin: &str, site_name: Option<&str>, description: Option<&str>, manifest_url: Option<&str>, metadata_fetched_at: Option<i64>) -> Result<()> {
        let (tx, rx) = unbounded();
        let req = DbRequest::UpsertSiteMeta {
            origin: origin.to_string(),
            site_name: site_name.map(|s| s.to_string()),
            description: description.map(|s| s.to_string()),
            manifest_url: manifest_url.map(|s| s.to_string()),
            metadata_fetched_at,
            resp: tx,
        };
        self.tx
            .send(req)
            .map_err(|e| anyhow!("Failed to send upsert_site_meta request: {}", e))?;
        rx.recv().map_err(|e| anyhow!("DB response recv failed: {}", e))?
    }

    pub fn get_site_meta_by_origin(&self, origin: &str) -> Result<Option<SiteMeta>> {
        let (tx, rx) = unbounded();
        let req = DbRequest::GetSiteMetaByOrigin { origin: origin.to_string(), resp: tx };
        self.tx
            .send(req)
            .map_err(|e| anyhow!("Failed to send get_site_meta_by_origin request: {}", e))?;
        Ok(rx.recv().map_err(|e| anyhow!("DB response recv failed: {}", e))??)
    }

    pub fn insert_icon(&self, site_id: i64, src_url: &str, width: Option<i64>, height: Option<i64>, mime: Option<&str>, data: Vec<u8>, fetched_at: Option<i64>) -> Result<()> {
        let (tx, rx) = unbounded();
        let req = DbRequest::InsertIcon {
            site_id,
            src_url: src_url.to_string(),
            width,
            height,
            mime: mime.map(|s| s.to_string()),
            data,
            fetched_at,
            resp: tx,
        };
        self.tx
            .send(req)
            .map_err(|e| anyhow!("Failed to send insert_icon request: {}", e))?;
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
    // New tables for site metadata and icons
    conn.execute(
        "CREATE TABLE IF NOT EXISTS site_meta (
            id INTEGER PRIMARY KEY,
            origin TEXT UNIQUE NOT NULL,
            site_name TEXT,
            description TEXT,
            manifest_url TEXT,
            metadata_fetched_at INTEGER
        )",
        params![],
    )?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS icons (
            id INTEGER PRIMARY KEY,
            site_id INTEGER NOT NULL,
            src_url TEXT NOT NULL,
            width INTEGER,
            height INTEGER,
            mime TEXT,
            data BLOB NOT NULL,
            fetched_at INTEGER,
            FOREIGN KEY(site_id) REFERENCES site_meta(id)
        )",
        params![],
    )?;

    while let Ok(req) = rx.recv() {
        match req {
            DbRequest::Insert { label, url, _timestamp, resp } => {
                let res = (|| -> Result<()> {
                    conn.execute(
                        "INSERT INTO urls (label, url, timestamp) VALUES (?1, ?2, ?3)",
                        params![label, url, _timestamp],
                    )?;
                    Ok(())
                })();
                let _ = resp.send(res.map_err(|e| anyhow!(e.to_string())));
            }
            DbRequest::ListRecent { limit, resp } => {
                let res = (|| -> Result<Vec<UrlRecord>> {
                    let mut stmt = conn.prepare(
                        "SELECT u.id, u.label, u.url, u.timestamp, m.site_name, ic.mime, ic.data 
                         FROM urls u 
                         LEFT JOIN site_meta m ON m.origin = (SELECT substr(u.url, instr(u.url, '://') + 3, instr(substr(u.url, instr(u.url, '://') + 3), '/') - 1))
                         LEFT JOIN icons ic ON ic.site_id = m.id AND ic.fetched_at = (
                             SELECT MAX(fetched_at) FROM icons WHERE site_id = m.id
                         )
                         ORDER BY u.timestamp DESC LIMIT ?1",
                    )?;
                    let rows = stmt
                        .query_map(params![limit], |row| {
                            Ok(UrlRecord {
                                id: row.get(0)?,
                                label: row.get(1)?,
                                url: row.get(2)?,
                                _timestamp: row.get(3)?,
                                site_name: row.get(4)?,
                                icon_mime: row.get(5)?,
                                icon_data: row.get(6)?,
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
            DbRequest::GetById { id, resp } => {
                let res = (|| -> Result<Option<UrlRecord>> {
                    let mut stmt = conn.prepare(
                        "SELECT u.id, u.label, u.url, u.timestamp, m.site_name, ic.mime, ic.data 
                         FROM urls u 
                         LEFT JOIN site_meta m ON m.origin = (SELECT substr(u.url, instr(u.url, '://') + 3, instr(substr(u.url, instr(u.url, '://') + 3), '/') - 1))
                         LEFT JOIN icons ic ON ic.site_id = m.id AND ic.fetched_at = (
                             SELECT MAX(fetched_at) FROM icons WHERE site_id = m.id
                         )
                         WHERE u.id = ?1 LIMIT 1",
                    )?;
                    let mut rows = stmt.query_map(params![id], |row| {
                        Ok(UrlRecord {
                            id: row.get(0)?,
                            label: row.get(1)?,
                            url: row.get(2)?,
                            _timestamp: row.get(3)?,
                            site_name: row.get(4)?,
                            icon_mime: row.get(5)?,
                            icon_data: row.get(6)?,
                        })
                    })?;
                    if let Some(r) = rows.next() {
                        Ok(Some(r?))
                    } else {
                        Ok(None)
                    }
                })();
                let _ = resp.send(res.map_err(|e| anyhow!(e.to_string())));
            }
            DbRequest::UpsertSiteMeta { origin, site_name, description, manifest_url, metadata_fetched_at, resp } => {
                let res = (|| -> Result<()> {
                    conn.execute(
                        "INSERT INTO site_meta (origin, site_name, description, manifest_url, metadata_fetched_at) 
                         VALUES (?1, ?2, ?3, ?4, ?5)
                         ON CONFLICT(origin) DO UPDATE SET site_name=excluded.site_name, description=excluded.description, manifest_url=excluded.manifest_url, metadata_fetched_at=excluded.metadata_fetched_at",
                        params![origin, site_name, description, manifest_url, metadata_fetched_at],
                    )?;
                    Ok(())
                })();
                let _ = resp.send(res.map_err(|e| anyhow!(e.to_string())));
            }
            DbRequest::GetSiteMetaByOrigin { origin, resp } => {
                let res = (|| -> Result<Option<SiteMeta>> {
                    let mut stmt = conn.prepare(
                        "SELECT id, origin, site_name, description, manifest_url, metadata_fetched_at FROM site_meta WHERE origin = ?1 LIMIT 1",
                    )?;
                    let mut rows = stmt.query_map(params![origin], |row| {
                        Ok(SiteMeta {
                            id: row.get(0)?,
                            origin: row.get(1)?,
                            site_name: row.get(2)?,
                            description: row.get(3)?,
                            manifest_url: row.get(4)?,
                            metadata_fetched_at: row.get(5)?,
                        })
                    })?;
                    if let Some(r) = rows.next() {
                        Ok(Some(r?))
                    } else {
                        Ok(None)
                    }
                })();
                let _ = resp.send(res.map_err(|e| anyhow!(e.to_string())));
            }
            DbRequest::InsertIcon { site_id, src_url, width, height, mime, data, fetched_at, resp } => {
                let res = (|| -> Result<()> {
                    conn.execute(
                        "INSERT INTO icons (site_id, src_url, width, height, mime, data, fetched_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
                        params![site_id, src_url, width, height, mime, data, fetched_at],
                    )?;
                    Ok(())
                })();
                let _ = resp.send(res.map_err(|e| anyhow!(e.to_string())));
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use rusqlite::Connection;

    #[test]
    fn test_site_meta_upsert_and_icon_insert() -> Result<()> {
        let conn = Connection::open_in_memory()?;
        conn.execute(
            "CREATE TABLE site_meta (
                id INTEGER PRIMARY KEY,
                origin TEXT UNIQUE NOT NULL,
                site_name TEXT,
                description TEXT,
                manifest_url TEXT,
                metadata_fetched_at INTEGER
            )",
            params![],
        )?;
        conn.execute(
            "CREATE TABLE icons (
                id INTEGER PRIMARY KEY,
                site_id INTEGER NOT NULL,
                src_url TEXT NOT NULL,
                width INTEGER,
                height INTEGER,
                mime TEXT,
                data BLOB NOT NULL,
                fetched_at INTEGER,
                FOREIGN KEY(site_id) REFERENCES site_meta(id)
            )",
            params![],
        )?;

        // Insert site_meta
        conn.execute(
            "INSERT INTO site_meta (origin, site_name) VALUES (?1, ?2)",
            params!["example.com", "Example Site"],
        )?;

        // Read back
        let name: String = conn.query_row("SELECT site_name FROM site_meta WHERE origin = ?1", params!["example.com"], |r| r.get(0))?;
        assert_eq!(name, "Example Site");

        // Insert icon
        conn.execute(
            "INSERT INTO icons (site_id, src_url, mime, data) VALUES (?1, ?2, ?3, ?4)",
            params![1i64, "https://example.com/icon.png", "image/png", vec![1u8,2u8,3u8]],
        )?;

        // Read icon back
        let mime: String = conn.query_row("SELECT mime FROM icons WHERE site_id = ?1", params![1i64], |r| r.get(0))?;
        assert_eq!(mime, "image/png");

        Ok(())
    }
}
