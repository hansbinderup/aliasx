use chrono::{DateTime, Local, Utc};
use rusqlite::types::{FromSql, FromSqlError, FromSqlResult, ValueRef};
use serde::{Deserialize, Serialize};

use anyhow::Context;
use rusqlite::{params, Connection};

use crate::task_filter::TaskFilter;

#[derive(Debug, Serialize, Deserialize)]
pub struct HistoryEntry {
    pub id: i64,
    pub task_name: String,
    pub task_command: String,
    pub started_at: DateTime<Utc>,
    pub exit_code: i32,
    pub scope: TaskFilter,
}

impl HistoryEntry {
    pub fn new(task_name: &str, task_command: &str, exit_code: i32, scope: TaskFilter) -> Self {
        Self {
            id: 0,
            task_name: task_name.to_string(),
            task_command: task_command.to_string(),
            started_at: Utc::now(),
            exit_code,
            scope: scope,
        }
    }
}

impl FromSql for TaskFilter {
    fn column_result(value: ValueRef<'_>) -> FromSqlResult<Self> {
        let s = value.as_str()?;

        s.parse().map_err(|e: String| FromSqlError::Other(e.into()))
    }
}

pub struct History;

impl History {
    fn connect() -> anyhow::Result<Connection> {
        let path = dirs::data_local_dir()
            .context("could not find local data directory")?
            .join("aliasx")
            .join("history.db");

        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let conn = Connection::open(path)?;
        conn.execute_batch(
            "
            PRAGMA journal_mode=WAL;
            CREATE TABLE IF NOT EXISTS task_history (
                id           INTEGER PRIMARY KEY AUTOINCREMENT,
                task_name    TEXT    NOT NULL,
                task_command TEXT    NOT NULL,
                started_at   TEXT    NOT NULL,
                exit_code    INTEGER NOT NULL,
                scope        TEXT    NOT NULL
            );
        ",
        )?;
        Ok(conn)
    }

    pub fn load() -> anyhow::Result<Vec<HistoryEntry>> {
        let conn = Self::connect()?;
        let mut stmt = conn.prepare(
            "SELECT id, task_name, task_command, started_at, exit_code, scope
         FROM task_history
         ORDER BY started_at DESC LIMIT 100",
        )?;
        let rows = stmt
            .query_map([], |row| {
                Ok(HistoryEntry {
                    id: row.get(0)?,
                    task_name: row.get(1)?,
                    task_command: row.get(2)?,
                    started_at: row.get(3)?,
                    exit_code: row.get(4)?,
                    scope: row.get(5)?,
                })
            })?
            .collect::<rusqlite::Result<Vec<_>>>()?;
        Ok(rows)
    }

    pub fn append(entry: &HistoryEntry) -> anyhow::Result<()> {
        let mut conn = Self::connect()?;
        let tx = conn.transaction()?;
        tx.execute(
            "INSERT INTO task_history (task_name, task_command, started_at, exit_code, scope)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            params![
                &entry.task_name,
                &entry.task_command,
                &entry.started_at,
                &entry.exit_code,
                entry.scope.to_string(),
            ],
        )?;

        // ensure we never exceed 100 history entries
        tx.execute(
            "DELETE FROM task_history WHERE id NOT IN (
                SELECT id FROM task_history ORDER BY started_at DESC LIMIT 100
            )",
            [],
        )?;
        tx.commit()?;
        Ok(())
    }

    pub fn format_timestamp(dt: &DateTime<Utc>) -> String {
        dt.with_timezone(&Local)
            .format("%Y-%m-%d %H:%M:%S")
            .to_string()
    }
}
