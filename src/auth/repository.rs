//! A repository for storing user data.
//!
//! This module provides an `AuthRepository` that can be used to store and
//! retrieve user data from a SQLite database.

use rusqlite::{Connection, params};
use std::path::Path;
use std::sync::{Arc, Mutex};
use crate::auth::model::User;

/// A repository for storing user data.
#[derive(Clone)]
pub struct AuthRepository {
    conn: Arc<Mutex<Connection>>,
}

impl AuthRepository {
    /// Creates a new `AuthRepository`.
    ///
    /// # Arguments
    ///
    /// * `path` - The path to the SQLite database file.
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
        Ok(Self { conn: Arc::new(Mutex::new(conn)) })
    }

    /// Creates a new user.
    ///
    /// # Arguments
    ///
    /// * `username` - The username of the new user.
    /// * `password_hash` - The password hash of the new user.
    ///
    /// # Returns
    ///
    /// The new user.
    pub async fn create_user(&self, username: String, password_hash: String) -> anyhow::Result<User> {
        let conn = self.conn.clone();
        tokio::task::spawn_blocking(move || {
            let now = chrono::Utc::now().to_rfc3339();
            let c = conn.lock().unwrap();
            c.execute(
                "INSERT INTO users (username, password_hash, created_at) VALUES (?1, ?2, ?3)",
                params![username, password_hash, now],
            )?;
            let id = c.last_insert_rowid();
            Ok(User { id, username, password_hash, created_at: now })
        }).await?
    }

    /// Finds a user by their username.
    ///
    /// # Arguments
    ///
    /// * `username` - The username of the user to find.
    ///
    /// # Returns
    ///
    /// An `Option` containing the user if they were found, or `None` if they
    /// were not.
    pub async fn find_by_username(&self, username: String) -> anyhow::Result<Option<User>> {
        let conn = self.conn.clone();
        tokio::task::spawn_blocking(move || {
            let c = conn.lock().unwrap();
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
        }).await?
    }
}
