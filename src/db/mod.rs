use anyhow::Result;
use chrono::{DateTime, Utc};
use rusqlite::{Connection, params};
use serde::{Deserialize, Serialize};
use std::path::Path;
use tokio::task;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct Database {
    db_path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandExecution {
    pub id: String,
    pub command: String,
    pub output: String,
    pub status: ExecutionStatus,
    pub timestamp: DateTime<Utc>,
    pub duration_ms: u64,
    pub agent_query: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ExecutionStatus {
    Running,
    Success,
    Error,
    Cancelled,
}

impl Database {
    pub async fn new(db_path: &Path) -> Result<Self> {
        let path_str = db_path.to_string_lossy().to_string();
        
        // Create parent directory if it doesn't exist
        if let Some(parent) = db_path.parent() {
            tokio::fs::create_dir_all(parent).await?;
        }
        
        let db = Database {
            db_path: path_str.clone(),
        };
        
        // Initialize database schema
        db.init_schema().await?;
        
        Ok(db)
    }
    
    async fn init_schema(&self) -> Result<()> {
        let db_path = self.db_path.clone();
        
        task::spawn_blocking(move || -> Result<()> {
            let conn = Connection::open(&db_path)?;
            
            conn.execute(
                "CREATE TABLE IF NOT EXISTS command_executions (
                    id TEXT PRIMARY KEY,
                    command TEXT NOT NULL,
                    output TEXT NOT NULL,
                    status TEXT NOT NULL,
                    timestamp TEXT NOT NULL,
                    duration_ms INTEGER NOT NULL,
                    agent_query TEXT
                )",
                [],
            )?;
            
            conn.execute(
                "CREATE TABLE IF NOT EXISTS tasks (
                    id TEXT PRIMARY KEY,
                    title TEXT NOT NULL,
                    description TEXT,
                    priority TEXT NOT NULL,
                    status TEXT NOT NULL,
                    created_at TEXT NOT NULL,
                    updated_at TEXT NOT NULL
                )",
                [],
            )?;
            
            conn.execute(
                "CREATE TABLE IF NOT EXISTS prep_sessions (
                    id TEXT PRIMARY KEY,
                    exam_type TEXT NOT NULL,
                    session_name TEXT NOT NULL,
                    status TEXT NOT NULL,
                    created_at TEXT NOT NULL,
                    updated_at TEXT NOT NULL
                )",
                [],
            )?;
            
            Ok(())
        }).await??;
        
        Ok(())
    }
    
    pub async fn save_command_execution(&self, execution: &CommandExecution) -> Result<()> {
        let db_path = self.db_path.clone();
        let execution = execution.clone();
        
        task::spawn_blocking(move || -> Result<()> {
            let conn = Connection::open(&db_path)?;
            
            conn.execute(
                "INSERT INTO command_executions 
                (id, command, output, status, timestamp, duration_ms, agent_query) 
                VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
                params![
                    execution.id,
                    execution.command,
                    execution.output,
                    serde_json::to_string(&execution.status)?,
                    execution.timestamp.to_rfc3339(),
                    execution.duration_ms as i64,
                    execution.agent_query,
                ],
            )?;
            
            Ok(())
        }).await??;
        
        Ok(())
    }
    
    pub async fn get_command_history(&self, limit: usize) -> Result<Vec<CommandExecution>> {
        let db_path = self.db_path.clone();
        
        let executions = task::spawn_blocking(move || -> Result<Vec<CommandExecution>> {
            let conn = Connection::open(&db_path)?;
            
            let mut stmt = conn.prepare(
                "SELECT id, command, output, status, timestamp, duration_ms, agent_query 
                FROM command_executions 
                ORDER BY timestamp DESC 
                LIMIT ?1"
            )?;
            
            let rows = stmt.query_map(params![limit], |row| {
                let status_str: String = row.get(3)?;
                let timestamp_str: String = row.get(4)?;
                
                Ok(CommandExecution {
                    id: row.get(0)?,
                    command: row.get(1)?,
                    output: row.get(2)?,
                    status: serde_json::from_str(&status_str).unwrap_or(ExecutionStatus::Error),
                    timestamp: DateTime::parse_from_rfc3339(&timestamp_str)
                        .unwrap_or_else(|_| Utc::now().into())
                        .with_timezone(&Utc),
                    duration_ms: row.get::<_, i64>(5)? as u64,
                    agent_query: row.get(6)?,
                })
            })?;
            
            let mut executions = Vec::new();
            for row in rows {
                executions.push(row?);
            }
            
            Ok(executions)
        }).await??;
        
        Ok(executions)
    }
    
    pub async fn update_execution_status(
        &self, 
        execution_id: &str, 
        status: ExecutionStatus,
        output: &str,
        duration_ms: u64,
    ) -> Result<()> {
        let db_path = self.db_path.clone();
        let execution_id = execution_id.to_string();
        let status_json = serde_json::to_string(&status)?;
        let output = output.to_string();
        
        task::spawn_blocking(move || -> Result<()> {
            let conn = Connection::open(&db_path)?;
            
            conn.execute(
                "UPDATE command_executions 
                SET status = ?1, output = ?2, duration_ms = ?3 
                WHERE id = ?4",
                params![status_json, output, duration_ms as i64, execution_id],
            )?;
            
            Ok(())
        }).await??;
        
        Ok(())
    }
}

impl CommandExecution {
    pub fn new(command: String, agent_query: Option<String>) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            command,
            output: String::new(),
            status: ExecutionStatus::Running,
            timestamp: Utc::now(),
            duration_ms: 0,
            agent_query,
        }
    }
}
