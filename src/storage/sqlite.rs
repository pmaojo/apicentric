use std::path::Path;

use rusqlite::{params, Connection, ToSql};
use std::sync::Mutex;

use crate::errors::{PulseError, PulseResult};
use crate::simulator::config::ServiceDefinition;
use crate::simulator::log::RequestLogEntry;
use crate::storage::Storage;

pub struct SqliteStorage {
    conn: Mutex<Connection>,
}

impl SqliteStorage {
    pub fn init_db<P: AsRef<Path>>(path: P) -> PulseResult<Self> {
        let path_ref = path.as_ref();
        if let Some(parent) = path_ref.parent() {
            std::fs::create_dir_all(parent).map_err(|e| {
                PulseError::runtime_error(
                    format!("Failed to create db directory: {}", e),
                    Some("Check the --db-path provided"),
                )
            })?;
        }

        let conn = Connection::open(path_ref).map_err(|e| {
            PulseError::runtime_error(
                format!("Failed to open database: {}", e),
                Some("Check the --db-path provided"),
            )
        })?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS services (name TEXT PRIMARY KEY, definition TEXT NOT NULL)",
            [],
        )
        .map_err(|e| PulseError::runtime_error(format!("Failed to create services table: {}", e), None::<String>))?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS logs (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                timestamp TEXT NOT NULL,
                service TEXT NOT NULL,
                endpoint INTEGER,
                method TEXT NOT NULL,
                path TEXT NOT NULL,
                status INTEGER NOT NULL
            )",
            [],
        )
        .map_err(|e| PulseError::runtime_error(format!("Failed to create logs table: {}", e), None::<String>))?;

        Ok(Self { conn: Mutex::new(conn) })
    }
}

impl Storage for SqliteStorage {
    fn save_service(&self, service: &ServiceDefinition) -> PulseResult<()> {
        let json = serde_json::to_string(service).map_err(|e| {
            PulseError::runtime_error(format!("Failed to serialize service: {}", e), None::<String>)
        })?;
        let conn = self
            .conn
            .lock()
            .map_err(|_| PulseError::runtime_error("DB locked".to_string(), None::<String>))?;
        conn.execute(
                "INSERT OR REPLACE INTO services (name, definition) VALUES (?1, ?2)",
                params![service.name, json],
            )
            .map_err(|e| PulseError::runtime_error(format!("Failed to save service: {}", e), None::<String>))?;
        Ok(())
    }

    fn load_service(&self, name: &str) -> PulseResult<Option<ServiceDefinition>> {
        let conn = self
            .conn
            .lock()
            .map_err(|_| PulseError::runtime_error("DB locked".to_string(), None::<String>))?;
        let mut stmt = conn
            .prepare("SELECT definition FROM services WHERE name = ?1")
            .map_err(|e| PulseError::runtime_error(format!("Failed to prepare query: {}", e), None::<String>))?;
        let mut rows = stmt
            .query(params![name])
            .map_err(|e| PulseError::runtime_error(format!("Failed to query service: {}", e), None::<String>))?;
        if let Some(row) = rows.next().map_err(|e| {
            PulseError::runtime_error(format!("Failed to read row: {}", e), None::<String>)
        })? {
            let json: String = row.get(0).map_err(|e| {
                PulseError::runtime_error(format!("Failed to get column: {}", e), None::<String>)
            })?;
            let service = serde_json::from_str(&json).map_err(|e| {
                PulseError::runtime_error(format!("Failed to deserialize service: {}", e), None::<String>)
            })?;
            Ok(Some(service))
        } else {
            Ok(None)
        }
    }

    fn append_log(&self, entry: &RequestLogEntry) -> PulseResult<()> {
        let conn = self
            .conn
            .lock()
            .map_err(|_| PulseError::runtime_error("DB locked".to_string(), None::<String>))?;
        conn.execute(
                "INSERT INTO logs (timestamp, service, endpoint, method, path, status) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                params![
                    entry.timestamp.to_rfc3339(),
                    entry.service,
                    entry.endpoint.map(|v| v as i64),
                    entry.method,
                    entry.path,
                    entry.status as i64
                ],
            )
            .map_err(|e| PulseError::runtime_error(format!("Failed to insert log: {}", e), None::<String>))?;
        Ok(())
    }

    fn query_logs(
        &self,
        service: Option<&str>,
        route: Option<&str>,
        method: Option<&str>,
        status: Option<u16>,
        limit: usize,
    ) -> PulseResult<Vec<RequestLogEntry>> {
        let mut sql = String::from(
            "SELECT timestamp, service, endpoint, method, path, status FROM logs",
        );
        let mut conditions: Vec<String> = Vec::new();
        let mut params: Vec<Box<dyn ToSql>> = Vec::new();

        if let Some(s) = service {
            conditions.push("service = ?".to_string());
            params.push(Box::new(s.to_string()));
        }
        if let Some(r) = route {
            conditions.push("path LIKE ?".to_string());
            params.push(Box::new(format!("%{}%", r)));
        }
        if let Some(m) = method {
            conditions.push("LOWER(method) = LOWER(?)".to_string());
            params.push(Box::new(m.to_string()));
        }
        if let Some(st) = status {
            conditions.push("status = ?".to_string());
            params.push(Box::new(st as i64));
        }
        if !conditions.is_empty() {
            sql.push_str(" WHERE ");
            sql.push_str(&conditions.join(" AND "));
        }
        sql.push_str(" ORDER BY id DESC LIMIT ?");
        params.push(Box::new(limit as i64));

        let conn = self
            .conn
            .lock()
            .map_err(|_| PulseError::runtime_error("DB locked".to_string(), None::<String>))?;
        let mut stmt = conn
            .prepare(&sql)
            .map_err(|e| {
                PulseError::runtime_error(format!("Failed to prepare log query: {}", e), None::<String>)
            })?;
        let mapped = stmt
            .query_map(rusqlite::params_from_iter(params.iter().map(|p| &**p)), |row| {
                let ts: String = row.get(0)?;
                let timestamp = chrono::DateTime::parse_from_rfc3339(&ts)
                    .map_err(|e| rusqlite::Error::FromSqlConversionFailure(0, rusqlite::types::Type::Text, Box::new(e)))?
                    .with_timezone(&chrono::Utc);
                Ok(RequestLogEntry {
                    timestamp,
                    service: row.get(1)?,
                    endpoint: row.get::<_, Option<i64>>(2)?.map(|v| v as usize),
                    method: row.get(3)?,
                    path: row.get(4)?,
                    status: row.get::<_, i64>(5)? as u16,
                })
            })
            .map_err(|e| PulseError::runtime_error(format!("Failed to fetch logs: {}", e), None::<String>))?;
        let mut entries: Vec<RequestLogEntry> = Vec::new();
        for item in mapped {
            if let Ok(entry) = item {
                entries.push(entry);
            }
        }
        entries.reverse();
        Ok(entries)
    }
}
