use rusqlite::{params, Connection, OpenFlags};
use std::path::Path;

use crate::config::{IngestConfig, IngestScope};

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct SessionInfo {
    pub id: String,
    pub title: String,
    pub directory: String,
    pub created_at: i64,
}

#[derive(Debug, Clone)]
pub struct MessageInfo {
    pub id: String,
    pub _session_id: String,
    pub role: String,
    pub _created_at: i64,
    pub parts: Vec<PartInfo>,
}

#[derive(Debug, Clone)]
pub struct PartInfo {
    pub _id: String,
    pub part_type: String,
    pub text: Option<String>,
    pub tool_name: Option<String>,
    pub tool_input: Option<String>,
    pub tool_output: Option<String>,
    pub tool_error: Option<String>,
    pub is_compacted: bool,
}

pub fn open_database(path: &Path) -> rememnemosyne_core::Result<Connection> {
    Connection::open_with_flags(path, OpenFlags::SQLITE_OPEN_READ_ONLY).map_err(|e| {
        rememnemosyne_core::MemoryError::Storage(format!("Failed to open opencode database: {e}"))
    })
}

pub fn list_sessions(
    conn: &Connection,
    config: &IngestConfig,
) -> rememnemosyne_core::Result<Vec<String>> {
    let mut sql = String::from("SELECT id, title, directory, time_created FROM session WHERE 1=1");

    match &config.scope {
        IngestScope::Project { directory } => {
            sql.push_str(" AND directory = ?");
        }
        IngestScope::Session { id } => {
            sql.push_str(" AND id = ?");
        }
        IngestScope::Global => {}
    }

    if config.since.is_some() {
        sql.push_str(" AND time_created >= ?");
    }
    if config.until.is_some() {
        sql.push_str(" AND time_created <= ?");
    }

    sql.push_str(" ORDER BY time_created DESC");

    let mut stmt = conn.prepare(&sql).map_err(|e| {
        rememnemosyne_core::MemoryError::Storage(format!("Failed to prepare session query: {e}"))
    })?;

    let session_ids: Vec<String> = match &config.scope {
        IngestScope::Global => {
            let rows = stmt.query_map([], |row| row.get(0)).map_err(|e| {
                rememnemosyne_core::MemoryError::Storage(format!("Session query failed: {e}"))
            })?;
            rows.filter_map(|r| r.ok()).collect()
        }
        IngestScope::Project { directory } => {
            let dir_str = directory.to_string_lossy().to_string();
            let rows = stmt
                .query_map(params![dir_str], |row| row.get(0))
                .map_err(|e| {
                    rememnemosyne_core::MemoryError::Storage(format!("Session query failed: {e}"))
                })?;
            rows.filter_map(|r| r.ok()).collect()
        }
        IngestScope::Session { id } => {
            let rows = stmt.query_map(params![id], |row| row.get(0)).map_err(|e| {
                rememnemosyne_core::MemoryError::Storage(format!("Session query failed: {e}"))
            })?;
            rows.filter_map(|r| r.ok()).collect()
        }
    };

    Ok(session_ids)
}

pub fn read_session_messages(
    conn: &Connection,
    session_id: &str,
) -> rememnemosyne_core::Result<Vec<MessageInfo>> {
    let mut msg_stmt = conn
        .prepare(
            "SELECT id, session_id, role, time_created FROM message WHERE session_id = ? ORDER BY time_created",
        )
        .map_err(|e| rememnemosyne_core::MemoryError::Storage(format!("Failed to prepare message query: {e}")))?;

    let messages: Vec<MessageInfo> = msg_stmt
        .query_map(params![session_id], |row| {
            Ok(MessageInfo {
                id: row.get(0)?,
                _session_id: row.get(1)?,
                role: row.get(2)?,
                _created_at: row.get(3)?,
                parts: Vec::new(),
            })
        })
        .map_err(|e| {
            rememnemosyne_core::MemoryError::Storage(format!("Message query failed: {e}"))
        })?
        .filter_map(|r| r.ok())
        .collect();

    let mut result = Vec::with_capacity(messages.len());
    for mut msg in messages {
        let parts = read_parts(conn, &msg.id)?;
        msg.parts = parts;
        result.push(msg);
    }

    Ok(result)
}

fn read_parts(conn: &Connection, message_id: &str) -> rememnemosyne_core::Result<Vec<PartInfo>> {
    let table_exists = conn.prepare("SELECT 1 FROM part WHERE 1=0 LIMIT 1").is_ok();

    if !table_exists {
        return Ok(Vec::new());
    }

    let columns = get_table_columns(conn, "part")?;
    let has_compacted = columns.contains(&"compacted".to_string());

    let sql = if has_compacted {
        "SELECT id, type, text, tool, output, input, error, compacted FROM part WHERE message_id = ? ORDER BY id"
    } else {
        "SELECT id, type, text, tool, output, input, error FROM part WHERE message_id = ? ORDER BY id"
    };

    let mut stmt = conn.prepare(sql).map_err(|e| {
        rememnemosyne_core::MemoryError::Storage(format!("Failed to prepare part query: {e}"))
    })?;

    let parts: Vec<PartInfo> = stmt
        .query_map(params![message_id], |row| {
            let compacted = if has_compacted {
                row.get::<_, Option<i64>>(7)?.unwrap_or(0) != 0
            } else {
                false
            };
            Ok(PartInfo {
                _id: row.get::<_, String>(0)?,
                part_type: row.get::<_, String>(1)?,
                text: row.get::<_, Option<String>>(2)?,
                tool_name: row.get::<_, Option<String>>(3)?,
                tool_output: row.get::<_, Option<String>>(4)?,
                tool_input: row.get::<_, Option<String>>(5)?,
                tool_error: row.get::<_, Option<String>>(6)?,
                is_compacted: compacted,
            })
        })
        .map_err(|e| rememnemosyne_core::MemoryError::Storage(format!("Part query failed: {e}")))?
        .filter_map(|r| r.ok())
        .collect();

    Ok(parts)
}

fn get_table_columns(conn: &Connection, table: &str) -> rememnemosyne_core::Result<Vec<String>> {
    let sql = format!("PRAGMA table_info({table})");
    let mut stmt = conn
        .prepare(&sql)
        .map_err(|e| rememnemosyne_core::MemoryError::Storage(format!("PRAGMA failed: {e}")))?;

    let columns: Vec<String> = stmt
        .query_map([], |row| row.get::<_, String>(1))
        .map_err(|e| rememnemosyne_core::MemoryError::Storage(format!("Column enum failed: {e}")))?
        .filter_map(|r| r.ok())
        .collect();

    Ok(columns)
}

pub fn find_default_database() -> Option<std::path::PathBuf> {
    let candidates = [".opencode/data.db", ".opencode/db.sqlite"];

    for candidate in &candidates {
        let path = Path::new(candidate);
        if path.exists() {
            return Some(path.to_path_buf());
        }
    }

    if let Ok(home) = std::env::var("HOME") {
        let home_candidates = [
            format!("{home}/.opencode/data.db"),
            format!("{home}/.local/share/opencode/data.db"),
        ];
        for candidate in &home_candidates {
            let path = Path::new(candidate);
            if path.exists() {
                return Some(path.to_path_buf());
            }
        }
    }

    if let Ok(xdg) = std::env::var("XDG_DATA_HOME") {
        let path = Path::new(&xdg).join("opencode/data.db");
        if path.exists() {
            return Some(path);
        }
    }

    None
}
