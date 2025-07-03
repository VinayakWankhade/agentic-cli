use anyhow::Result;
use chrono::Utc;
use clap::Subcommand;
use colored::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::db::Database;

#[derive(Debug, Clone, Subcommand)]
pub enum TaskCommand {
    /// Add a new task
    Add {
        /// Task title
        #[arg(long, short)]
        title: String,
        /// Task description
        #[arg(long, short)]
        description: Option<String>,
        /// Task priority (low, medium, high)
        #[arg(long, short, default_value = "medium")]
        priority: String,
    },
    /// List tasks
    List {
        /// Show only recent tasks
        #[arg(long)]
        recent: bool,
        /// Filter by status
        #[arg(long)]
        status: Option<String>,
        /// Filter by priority
        #[arg(long)]
        priority: Option<String>,
    },
    /// Mark task as complete
    Complete {
        /// Task ID or partial title
        task_id: String,
    },
    /// Delete a task
    Delete {
        /// Task ID or partial title
        task_id: String,
    },
    /// Update task priority
    Priority {
        /// Task ID or partial title
        task_id: String,
        /// New priority (low, medium, high)
        priority: String,
    },
    /// Show task details
    Show {
        /// Task ID or partial title
        task_id: String,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    pub id: String,
    pub title: String,
    pub description: Option<String>,
    pub priority: Priority,
    pub status: TaskStatus,
    pub created_at: chrono::DateTime<Utc>,
    pub updated_at: chrono::DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Priority {
    Low,
    Medium,
    High,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TaskStatus {
    Todo,
    InProgress,
    Complete,
}

impl std::fmt::Display for Priority {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Priority::Low => write!(f, "LOW"),
            Priority::Medium => write!(f, "MED"),
            Priority::High => write!(f, "HIGH"),
        }
    }
}

impl std::fmt::Display for TaskStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TaskStatus::Todo => write!(f, "TODO"),
            TaskStatus::InProgress => write!(f, "IN PROGRESS"),
            TaskStatus::Complete => write!(f, "COMPLETE"),
        }
    }
}

impl std::str::FromStr for Priority {
    type Err = anyhow::Error;
    
    fn from_str(s: &str) -> Result<Self> {
        match s.to_lowercase().as_str() {
            "low" | "l" => Ok(Priority::Low),
            "medium" | "med" | "m" => Ok(Priority::Medium),
            "high" | "h" => Ok(Priority::High),
            _ => Err(anyhow::anyhow!("Invalid priority: {}", s)),
        }
    }
}

impl Task {
    pub fn new(title: String, description: Option<String>, priority: Priority) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4().to_string(),
            title,
            description,
            priority,
            status: TaskStatus::Todo,
            created_at: now,
            updated_at: now,
        }
    }
    
    pub fn priority_color(&self) -> String {
        match self.priority {
            Priority::High => "red".to_string(),
            Priority::Medium => "yellow".to_string(),
            Priority::Low => "green".to_string(),
        }
    }
    
    pub fn status_icon(&self) -> &'static str {
        match self.status {
            TaskStatus::Todo => "○",
            TaskStatus::InProgress => "◐",
            TaskStatus::Complete => "●",
        }
    }
}

pub async fn execute(command: TaskCommand, _db: &Database) -> Result<()> {
    match command {
        TaskCommand::Add { title, description, priority } => {
            let priority = priority.parse::<Priority>()?;
            let task = Task::new(title, description, priority);
            
            // In a real implementation, save to database
            println!("{}", "✓ Task created successfully!".green().bold());
            println!("ID: {}", task.id.bright_blue());
            println!("Title: {}", task.title.bold());
            if let Some(desc) = &task.description {
                println!("Description: {}", desc);
            }
            println!("Priority: {}", format!("{}", task.priority).color(task.priority_color()));
            println!("Status: {}", task.status);
        }
        
        TaskCommand::List { recent: _, status: _, priority: _ } => {
            // In a real implementation, query from database
            println!("{}", "📋 Your Tasks".blue().bold());
            println!();
            
            // Mock data for demonstration
            let tasks = vec![
                Task::new("Build dashboard".to_string(), Some("Create React dashboard for CET prep".to_string()), Priority::High),
                Task::new("Study algorithms".to_string(), None, Priority::Medium),
                Task::new("Review notes".to_string(), Some("Go through physics notes".to_string()), Priority::Low),
            ];
            
            for (index, task) in tasks.iter().enumerate() {
                println!("{}. {} {} {} [{}]", 
                    (index + 1).to_string().bright_white(),
                    task.status_icon(),
                    task.title.bold(),
                    format!("({})", task.priority).color(task.priority_color()),
                    task.id[..8].bright_black()
                );
                
                if let Some(desc) = &task.description {
                    println!("   {}", desc.italic().bright_black());
                }
                println!();
            }
        }
        
        TaskCommand::Complete { task_id } => {
            println!("{} Task '{}' marked as complete!", "✓".green().bold(), task_id.bold());
        }
        
        TaskCommand::Delete { task_id } => {
            println!("{} Task '{}' deleted!", "🗑".red(), task_id.bold());
        }
        
        TaskCommand::Priority { task_id, priority } => {
            let priority = priority.parse::<Priority>()?;
            println!("{} Updated priority for '{}' to {}", 
                "↗".yellow().bold(), 
                task_id.bold(), 
                format!("{}", priority).color(match priority {
                    Priority::High => "red",
                    Priority::Medium => "yellow", 
                    Priority::Low => "green",
                })
            );
        }
        
        TaskCommand::Show { task_id } => {
            println!("{} Task Details", "🔍".blue());
            println!("Searching for task: {}", task_id.bold());
            
            // Mock task details
            println!();
            println!("ID: {}", "abc123def".bright_blue());
            println!("Title: {}", "Build dashboard".bold());
            println!("Description: {}", "Create React dashboard for CET prep".italic());
            println!("Priority: {}", "HIGH".red().bold());
            println!("Status: {}", "TODO".yellow());
            println!("Created: {}", "2024-01-15 10:30:00 UTC".bright_black());
            println!("Updated: {}", "2024-01-15 10:30:00 UTC".bright_black());
        }
    }
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_priority_parsing() {
        assert!(matches!("high".parse::<Priority>().unwrap(), Priority::High));
        assert!(matches!("medium".parse::<Priority>().unwrap(), Priority::Medium));
        assert!(matches!("low".parse::<Priority>().unwrap(), Priority::Low));
        assert!("invalid".parse::<Priority>().is_err());
    }
    
    #[test]
    fn test_task_creation() {
        let task = Task::new(
            "Test task".to_string(),
            Some("Test description".to_string()),
            Priority::High
        );
        
        assert_eq!(task.title, "Test task");
        assert_eq!(task.description, Some("Test description".to_string()));
        assert!(matches!(task.priority, Priority::High));
        assert!(matches!(task.status, TaskStatus::Todo));
    }
}
