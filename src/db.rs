use std::sync::Arc;
use std::sync::Mutex;
use rusqlite::{Connection, Result as SqlResult, params};
use chrono::Utc;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Monitor {
    pub id: String,
    pub url: String,
    pub name: Option<String>,
    pub interval_seconds: u64,
    pub enabled: bool,
    pub last_check: Option<String>,
    pub last_status: Option<String>,
    pub created_at: String,
}

impl Monitor {
    pub fn new(url: String, interval_seconds: u64) -> Self {
        let now = Utc::now().to_rfc3339();
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            url,
            name: None,
            interval_seconds,
            enabled: true,
            last_check: None,
            last_status: None,
            created_at: now,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Change {
    pub id: String,
    pub monitor_id: String,
    pub change_type: String,
    pub old_value: Option<String>,
    pub new_value: Option<String>,
    pub details: Option<String>,
    pub detected_at: String,
}

#[derive(Clone)]
pub struct Database {
    conn: Arc<Mutex<Connection>>,
}

impl Database {
    pub fn new(path: &str) -> SqlResult<Self> {
        let conn = Connection::open(path)?;
        let db = Self { conn: Arc::new(Mutex::new(conn)) };
        db.init_tables()?;
        Ok(db)
    }

    fn init_tables(&self) -> SqlResult<()> {
        let conn = self.conn.lock().unwrap();
        
        conn.execute(
            "CREATE TABLE IF NOT EXISTS monitors (
                id TEXT PRIMARY KEY,
                url TEXT NOT NULL,
                name TEXT,
                interval_seconds INTEGER NOT NULL DEFAULT 3600,
                enabled INTEGER NOT NULL DEFAULT 1,
                last_check TEXT,
                last_status TEXT,
                created_at TEXT NOT NULL
            )",
            [],
        )?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS changes (
                id TEXT PRIMARY KEY,
                monitor_id TEXT NOT NULL,
                change_type TEXT NOT NULL,
                old_value TEXT,
                new_value TEXT,
                details TEXT,
                detected_at TEXT NOT NULL,
                FOREIGN KEY (monitor_id) REFERENCES monitors(id)
            )",
            [],
        )?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS snapshots (
                id TEXT PRIMARY KEY,
                monitor_id TEXT NOT NULL,
                content_hash TEXT NOT NULL,
                screenshot_path TEXT,
                created_at TEXT NOT NULL,
                FOREIGN KEY (monitor_id) REFERENCES monitors(id)
            )",
            [],
        )?;

        Ok(())
    }

    pub fn add_monitor(&self, monitor: &Monitor) -> SqlResult<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO monitors (id, url, name, interval_seconds, enabled, last_check, last_status, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![
                monitor.id,
                monitor.url,
                monitor.name,
                monitor.interval_seconds,
                monitor.enabled as i32,
                monitor.last_check,
                monitor.last_status,
                monitor.created_at
            ],
        )?;
        Ok(())
    }

    pub fn get_monitors(&self) -> SqlResult<Vec<Monitor>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, url, name, interval_seconds, enabled, last_check, last_status, created_at FROM monitors"
        )?;

        let monitors = stmt.query_map([], |row| {
            Ok(Monitor {
                id: row.get(0)?,
                url: row.get(1)?,
                name: row.get(2)?,
                interval_seconds: row.get(3)?,
                enabled: row.get::<_, i32>(4)? != 0,
                last_check: row.get(5)?,
                last_status: row.get(6)?,
                created_at: row.get(7)?,
            })
        })?;

        monitors.collect()
    }

    pub fn get_monitor(&self, id: &str) -> SqlResult<Option<Monitor>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, url, name, interval_seconds, enabled, last_check, last_status, created_at FROM monitors WHERE id = ?1"
        )?;

        let mut rows = stmt.query(params![id])?;
        
        if let Some(row) = rows.next()? {
            Ok(Some(Monitor {
                id: row.get(0)?,
                url: row.get(1)?,
                name: row.get(2)?,
                interval_seconds: row.get(3)?,
                enabled: row.get::<_, i32>(4)? != 0,
                last_check: row.get(5)?,
                last_status: row.get(6)?,
                created_at: row.get(7)?,
            }))
        } else {
            Ok(None)
        }
    }

    pub fn update_monitor_status(&self, id: &str, status: &str) -> SqlResult<()> {
        let conn = self.conn.lock().unwrap();
        let now = Utc::now().to_rfc3339();
        conn.execute(
            "UPDATE monitors SET last_check = ?1, last_status = ?2 WHERE id = ?3",
            params![now, status, id],
        )?;
        Ok(())
    }

    pub fn delete_monitor(&self, id: &str) -> SqlResult<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute("DELETE FROM changes WHERE monitor_id = ?1", params![id])?;
        conn.execute("DELETE FROM snapshots WHERE monitor_id = ?1", params![id])?;
        conn.execute("DELETE FROM monitors WHERE id = ?1", params![id])?;
        Ok(())
    }

    pub fn add_change(&self, change: &Change) -> SqlResult<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO changes (id, monitor_id, change_type, old_value, new_value, details, detected_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![
                change.id,
                change.monitor_id,
                change.change_type,
                change.old_value,
                change.new_value,
                change.details,
                change.detected_at
            ],
        )?;
        Ok(())
    }

    pub fn get_changes(&self, monitor_id: Option<&str>) -> SqlResult<Vec<Change>> {
        let conn = self.conn.lock().unwrap();
        let mut results = Vec::new();
        
        if let Some(mid) = monitor_id {
            let mut stmt = conn.prepare(
                "SELECT id, monitor_id, change_type, old_value, new_value, details, detected_at FROM changes WHERE monitor_id = ?1 ORDER BY detected_at DESC"
            )?;
            
            let mut rows = stmt.query(params![mid])?;
            while let Some(row) = rows.next()? {
                results.push(Change {
                    id: row.get(0)?,
                    monitor_id: row.get(1)?,
                    change_type: row.get(2)?,
                    old_value: row.get(3)?,
                    new_value: row.get(4)?,
                    details: row.get(5)?,
                    detected_at: row.get(6)?,
                });
            }
        } else {
            let mut stmt = conn.prepare(
                "SELECT id, monitor_id, change_type, old_value, new_value, details, detected_at FROM changes ORDER BY detected_at DESC"
            )?;
            
            let mut rows = stmt.query([])?;
            while let Some(row) = rows.next()? {
                results.push(Change {
                    id: row.get(0)?,
                    monitor_id: row.get(1)?,
                    change_type: row.get(2)?,
                    old_value: row.get(3)?,
                    new_value: row.get(4)?,
                    details: row.get(5)?,
                    detected_at: row.get(6)?,
                });
            }
        }
        
        Ok(results)
    }

    pub fn get_latest_snapshot(&self, monitor_id: &str) -> SqlResult<Option<String>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT content_hash FROM snapshots WHERE monitor_id = ?1 ORDER BY created_at DESC LIMIT 1"
        )?;

        let mut rows = stmt.query(params![monitor_id])?;
        
        if let Some(row) = rows.next()? {
            Ok(Some(row.get(0)?))
        } else {
            Ok(None)
        }
    }

    pub fn add_snapshot(&self, monitor_id: &str, content_hash: &str, screenshot_path: Option<&str>) -> SqlResult<()> {
        let conn = self.conn.lock().unwrap();
        let now = Utc::now().to_rfc3339();
        conn.execute(
            "INSERT INTO snapshots (id, monitor_id, content_hash, screenshot_path, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            params![
                uuid::Uuid::new_v4().to_string(),
                monitor_id,
                content_hash,
                screenshot_path,
                now
            ],
        )?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_monitor_creation() {
        let monitor = Monitor::new("https://example.com".to_string(), 3600);
        assert!(!monitor.id.is_empty());
        assert_eq!(monitor.url, "https://example.com");
        assert_eq!(monitor.interval_seconds, 3600);
        assert!(monitor.enabled);
    }

    #[test]
    fn test_database_operations() {
        let db_path = "/tmp/test_driftguard.db";
        let _ = fs::remove_file(db_path);
        
        let db = Database::new(db_path).unwrap();
        
        let monitor = Monitor::new("https://test.com".to_string(), 1800);
        db.add_monitor(&monitor).unwrap();
        
        let monitors = db.get_monitors().unwrap();
        assert_eq!(monitors.len(), 1);
        assert_eq!(monitors[0].url, "https://test.com");
        
        let _ = fs::remove_file(db_path);
    }
}
