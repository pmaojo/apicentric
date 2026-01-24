//! A repository for storing user data.
//!
//! This module provides an `AuthRepository` that can be used to store and
//! retrieve user data from a SQLite database.

use crate::auth::model::User;
use crate::errors::{ApicentricError, ApicentricResult};
use rusqlite::{params, Connection};
use std::path::Path;
use std::sync::{Arc, Mutex};

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
    pub fn new<P: AsRef<Path>>(path: P) -> ApicentricResult<Self> {
        let conn = Connection::open(path).map_err(|e| ApicentricError::Database {
            message: e.to_string(),
            suggestion: Some("Check database path permissions".to_string()),
        })?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS users (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                username TEXT UNIQUE NOT NULL,
                password_hash TEXT NOT NULL,
                created_at TEXT NOT NULL
            )",
            [],
        )
        .map_err(|e| ApicentricError::Database {
            message: format!("Failed to initialize database: {}", e),
            suggestion: None,
        })?;

        Ok(Self {
            conn: Arc::new(Mutex::new(conn)),
        })
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
    pub async fn create_user(
        &self,
        username: String,
        password_hash: String,
    ) -> ApicentricResult<User> {
        let conn = self.conn.clone();
        tokio::task::spawn_blocking(move || {
            let now = chrono::Utc::now().to_rfc3339();
            let c = conn.lock().map_err(|_| ApicentricError::Database {
                message: "Failed to acquire database lock".to_string(),
                suggestion: None,
            })?;

            c.execute(
                "INSERT INTO users (username, password_hash, created_at) VALUES (?1, ?2, ?3)",
                params![username, password_hash, now],
            )
            .map_err(|e| ApicentricError::Database {
                message: format!("Failed to create user: {}", e),
                suggestion: Some("Check if username already exists".to_string()),
            })?;

            let id = c.last_insert_rowid();
            Ok(User {
                id,
                username,
                password_hash,
                created_at: now,
            })
        })
        .await
        .map_err(|e| ApicentricError::Runtime {
            message: format!("Database task join error: {}", e),
            suggestion: None,
        })?
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
    pub async fn find_by_username(&self, username: String) -> ApicentricResult<Option<User>> {
        let conn = self.conn.clone();
        tokio::task::spawn_blocking(move || {
            let c = conn.lock().map_err(|_| ApicentricError::Database {
                message: "Failed to acquire database lock".to_string(),
                suggestion: None,
            })?;

            let mut stmt = c
                .prepare(
                    "SELECT id, username, password_hash, created_at FROM users WHERE username = ?1",
                )
                .map_err(|e| ApicentricError::Database {
                    message: format!("Failed to prepare query: {}", e),
                    suggestion: None,
                })?;

            let mut rows =
                stmt.query(params![username])
                    .map_err(|e| ApicentricError::Database {
                        message: format!("Failed to execute query: {}", e),
                        suggestion: None,
                    })?;

            if let Some(row) = rows.next().map_err(|e| ApicentricError::Database {
                message: format!("Failed to fetch row: {}", e),
                suggestion: None,
            })? {
                Ok(Some(User {
                    id: row.get(0).map_err(|e| ApicentricError::Database {
                        message: e.to_string(),
                        suggestion: None,
                    })?,
                    username: row.get(1).map_err(|e| ApicentricError::Database {
                        message: e.to_string(),
                        suggestion: None,
                    })?,
                    password_hash: row.get(2).map_err(|e| ApicentricError::Database {
                        message: e.to_string(),
                        suggestion: None,
                    })?,
                    created_at: row.get(3).map_err(|e| ApicentricError::Database {
                        message: e.to_string(),
                        suggestion: None,
                    })?,
                }))
            } else {
                Ok(None)
            }
        })
        .await
        .map_err(|e| ApicentricError::Runtime {
            message: format!("Database task join error: {}", e),
            suggestion: None,
        })?
    }
}
