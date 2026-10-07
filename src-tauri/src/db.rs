use rusqlite::{params, Connection, Result};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryRecord {
    pub id: i64,
    pub query: String,
    pub executed_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppUsageRecord {
    pub item_id: String,
    pub title: String,
    pub path: String,
    pub item_type: String,
    pub launch_count: i64,
}

pub struct Database {
    db_path: PathBuf,
}

impl Database {
    pub fn new(app_dir: PathBuf) -> Result<Self> {
        std::fs::create_dir_all(&app_dir).ok();
        let db_path = app_dir.join("history.db");
        let conn = Connection::open(&db_path)?;
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS search_history (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                query TEXT NOT NULL,
                executed_at DATETIME DEFAULT CURRENT_TIMESTAMP
            );
            CREATE TABLE IF NOT EXISTS app_usage (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                item_id TEXT UNIQUE NOT NULL,
                title TEXT NOT NULL,
                path TEXT NOT NULL,
                item_type TEXT NOT NULL,
                launch_count INTEGER DEFAULT 1,
                last_launched DATETIME DEFAULT CURRENT_TIMESTAMP
            );
            CREATE INDEX IF NOT EXISTS idx_search_query ON search_history(query);
            CREATE INDEX IF NOT EXISTS idx_app_launch_count ON app_usage(launch_count DESC);"
        )?;
        Ok(Self { db_path })
    }

    fn get_conn(&self) -> Result<Connection> {
        Connection::open(&self.db_path)
    }

    pub fn record_search(&self, query: &str) -> Result<()> {
        let conn = self.get_conn()?;
        conn.execute(
            "INSERT INTO search_history (query) VALUES (?1)",
            params![query],
        )?;
        Ok(())
    }

    pub fn record_launch(&self, item_id: &str, title: &str, path: &str, item_type: &str) -> Result<()> {
        let conn = self.get_conn()?;
        conn.execute(
            "INSERT INTO app_usage (item_id, title, path, item_type, launch_count, last_launched)
             VALUES (?1, ?2, ?3, ?4, 1, CURRENT_TIMESTAMP)
             ON CONFLICT(item_id) DO UPDATE SET
                launch_count = launch_count + 1,
                last_launched = CURRENT_TIMESTAMP,
                title = excluded.title,
                path = excluded.path",
            params![item_id, title, path, item_type],
        )?;
        Ok(())
    }

    pub fn get_recent_history(&self, limit: usize) -> Result<Vec<HistoryRecord>> {
        let conn = self.get_conn()?;
        let mut stmt = conn.prepare(
            "SELECT id, query, executed_at FROM search_history ORDER BY id DESC LIMIT ?1"
        )?;
        let rows = stmt.query_map(params![limit as i64], |row| {
            Ok(HistoryRecord {
                id: row.get(0)?,
                query: row.get(1)?,
                executed_at: row.get(2)?,
            })
        })?;
        let mut list = Vec::new();
        for r in rows {
            if let Ok(item) = r {
                list.push(item);
            }
        }
        Ok(list)
    }

    pub fn get_top_apps(&self, limit: usize) -> Result<Vec<AppUsageRecord>> {
        let conn = self.get_conn()?;
        let mut stmt = conn.prepare(
            "SELECT item_id, title, path, item_type, launch_count 
             FROM app_usage 
             ORDER BY launch_count DESC, last_launched DESC 
             LIMIT ?1"
        )?;
        let rows = stmt.query_map(params![limit as i64], |row| {
            Ok(AppUsageRecord {
                item_id: row.get(0)?,
                title: row.get(1)?,
                path: row.get(2)?,
                item_type: row.get(3)?,
                launch_count: row.get(4)?,
            })
        })?;
        let mut list = Vec::new();
        for r in rows {
            if let Ok(item) = r {
                list.push(item);
            }
        }
        Ok(list)
    }

    pub fn clear_all_history(&self) -> Result<()> {
        let conn = self.get_conn()?;
        conn.execute("DELETE FROM search_history", [])?;
        conn.execute("DELETE FROM app_usage", [])?;
        Ok(())
    }
}
