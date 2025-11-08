use rusqlite::{Connection, params};
use std::path::Path;
use std::sync::Mutex;
use crate::auth::model::User;

pub struct AuthRepository {
    conn: Mutex<Connection>,
}

impl AuthRepository {
    pub fn new<P: AsRef<Path>>(path: P) -> anyhow::Result<Self> {
        let conn = Connection::open(path)?;
        conn.execute(
            "CREATE TABLE IF NOT EXISTS users (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                username TEXT UNIQUE NOT NULL,
                password_hash TEXT NOT NULL,
                created_at TEXT NOT NULL
            )",
            [],
        )?;
        Ok(Self { conn: Mutex::new(conn) })
    }

    pub fn create_user(&self, username: &str, password_hash: &str) -> anyhow::Result<User> {
        let now = chrono::Utc::now().to_rfc3339();
        let c = self.conn.lock().unwrap();
        c.execute(
            "INSERT INTO users (username, password_hash, created_at) VALUES (?1, ?2, ?3)",
            params![username, password_hash, now],
        )?;
        let id = c.last_insert_rowid();
        Ok(User { id, username: username.to_string(), password_hash: password_hash.to_string(), created_at: now })
    }

    pub fn find_by_username(&self, username: &str) -> anyhow::Result<Option<User>> {
        let c = self.conn.lock().unwrap();
        let mut stmt = c.prepare("SELECT id, username, password_hash, created_at FROM users WHERE username = ?1")?;
        let mut rows = stmt.query(params![username])?;
        if let Some(row) = rows.next()? {
            Ok(Some(User {
                id: row.get(0)?,
                username: row.get(1)?,
                password_hash: row.get(2)?,
                created_at: row.get(3)?,
            }))
        } else {
            Ok(None)
        }
    }
}
