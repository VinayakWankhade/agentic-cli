use anyhow::Result;
use std::process::Stdio;
use tokio::process::Command;
use tracing::{debug, info, warn};

use crate::db::Database;

pub mod task;
pub mod prep;
pub mod blog;

pub use task::TaskCommand;
pub use prep::PrepCommand;
pub use blog::BlogCommand;

#[derive(Debug, Clone)]
pub struct CommandRegistry {
    // Add any state needed for command execution
}

impl CommandRegistry {
    pub fn new() -> Self {
        Self {}
    }
    
    pub async fn execute_task(&self, task_cmd: TaskCommand, db: &Database) -> Result<()> {
        info!("Executing task command: {:?}", task_cmd);
        task::execute(task_cmd, db).await
    }
    
    pub async fn execute_prep(&self, prep_cmd: PrepCommand, db: &Database) -> Result<()> {
        info!("Executing prep command: {:?}", prep_cmd);
        prep::execute(prep_cmd, db).await
    }
    
    pub async fn execute_blog(&self, blog_cmd: BlogCommand, db: &Database) -> Result<()> {
        info!("Executing blog command: {:?}", blog_cmd);
        blog::execute(blog_cmd, db).await
    }
    
    pub async fn execute_raw_command(&self, command_str: &str) -> Result<()> {
        info!("Executing raw command: {}", command_str);
        
        // Parse command and arguments
        let parts: Vec<&str> = command_str.split_whitespace().collect();
        if parts.is_empty() {
            return Err(anyhow::anyhow!("Empty command"));
        }
        
        let (cmd, args) = parts.split_at(1);
        let cmd = cmd[0];
        
        debug!("Running command: {} with args: {:?}", cmd, args);
        
        let child = Command::new(cmd)
            .args(args)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()?;
        
        let output = child.wait_with_output().await?;
        
        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            if !stdout.trim().is_empty() {
                println!("{}", stdout);
            }
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            warn!("Command failed with error: {}", stderr);
            return Err(anyhow::anyhow!("Command failed: {}", stderr));
        }
        
        Ok(())
    }
}

impl Default for CommandRegistry {
    fn default() -> Self {
        Self::new()
    }
}
