use anyhow::Result;
use dirs::home_dir;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tokio::fs;

/// Configuration for the Warp pipeline loaded from .agentic.toml
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgenticConfig {
    pub warp: WarpConfig,
}

/// Warp-specific configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WarpConfig {
    pub models: ModelConfig,
    pub execution: ExecutionConfig,
    pub safety: SafetyConfig,
}

/// Model configuration for different agents
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelConfig {
    pub planner: String,
    pub coder: String,
    pub fallback: String,
    pub ollama_host: String,
    pub timeout_seconds: u64,
}

/// Execution configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionConfig {
    pub streaming: bool,
    pub auto_confirm: bool,
    pub max_execution_time: u64,
    pub working_directory: Option<String>,
}

/// Safety configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SafetyConfig {
    pub enable_safety_checks: bool,
    pub dangerous_commands: Vec<String>,
    pub require_confirmation: bool,
    pub allowed_directories: Vec<String>,
}

impl Default for AgenticConfig {
    fn default() -> Self {
        Self {
            warp: WarpConfig::default(),
        }
    }
}

impl Default for WarpConfig {
    fn default() -> Self {
        Self {
            models: ModelConfig::default(),
            execution: ExecutionConfig::default(),
            safety: SafetyConfig::default(),
        }
    }
}

impl Default for ModelConfig {
    fn default() -> Self {
        Self {
            planner: "phi4".to_string(),
            coder: "codellama".to_string(),
            fallback: "gemma3".to_string(),
            ollama_host: "http://localhost:11434".to_string(),
            timeout_seconds: 30,
        }
    }
}

impl Default for ExecutionConfig {
    fn default() -> Self {
        Self {
            streaming: true,
            auto_confirm: false,
            max_execution_time: 300, // 5 minutes
            working_directory: None,
        }
    }
}

impl Default for SafetyConfig {
    fn default() -> Self {
        Self {
            enable_safety_checks: true,
            dangerous_commands: vec![
                "rm -rf /".to_string(),
                "del /s /q c:".to_string(),
                "format c:".to_string(),
                "shutdown".to_string(),
                "reboot".to_string(),
                "dd if=".to_string(),
                "mkfs.".to_string(),
                "> /dev/".to_string(),
                "chmod 777 /".to_string(),
                "chown root /".to_string(),
            ],
            require_confirmation: true,
            allowed_directories: vec![
                "~/".to_string(),
                "./".to_string(),
                "/tmp/".to_string(),
                "C:\\temp\\".to_string(),
            ],
        }
    }
}

impl AgenticConfig {
    /// Load configuration from .agentic.toml file
    pub async fn load() -> Result<Self> {
        let config_path = Self::config_path();
        
        if config_path.exists() {
            let content = fs::read_to_string(&config_path).await?;
            let config: AgenticConfig = toml::from_str(&content)?;
            Ok(config)
        } else {
            let config = Self::default();
            config.save().await?;
            Ok(config)
        }
    }

    /// Save configuration to .agentic.toml file
    pub async fn save(&self) -> Result<()> {
        let config_path = Self::config_path();
        
        if let Some(parent) = config_path.parent() {
            fs::create_dir_all(parent).await?;
        }
        
        let content = toml::to_string_pretty(self)?;
        fs::write(&config_path, content).await?;
        
        Ok(())
    }

    /// Get the path to the .agentic.toml config file
    fn config_path() -> PathBuf {
        // Look for .agentic.toml in current directory first, then home directory
        let current_dir_config = PathBuf::from(".agentic.toml");
        if current_dir_config.exists() {
            return current_dir_config;
        }
        
        let home = home_dir().unwrap_or_else(|| PathBuf::from("."));
        home.join(".agentic").join("agentic.toml")
    }

    /// Check if a command is dangerous based on configuration
    pub fn is_dangerous_command(&self, command: &str) -> bool {
        if !self.warp.safety.enable_safety_checks {
            return false;
        }

        let command_lower = command.to_lowercase();
        self.warp.safety.dangerous_commands
            .iter()
            .any(|pattern| command_lower.contains(&pattern.to_lowercase()))
    }

    /// Check if execution in a directory is allowed
    pub fn is_directory_allowed(&self, directory: &str) -> bool {
        if self.warp.safety.allowed_directories.is_empty() {
            return true; // No restrictions
        }

        let dir_path = PathBuf::from(directory);
        
        self.warp.safety.allowed_directories
            .iter()
            .any(|allowed| {
                let allowed_path = PathBuf::from(allowed);
                dir_path.starts_with(&allowed_path) || 
                allowed.contains("*") || // Wildcard support
                directory.starts_with(allowed)
            })
    }

    /// Get the working directory for command execution
    pub fn get_working_directory(&self) -> Option<PathBuf> {
        self.warp.execution.working_directory
            .as_ref()
            .map(|dir| PathBuf::from(dir))
    }

    /// Check if auto-confirmation is enabled
    pub fn auto_confirm_enabled(&self) -> bool {
        self.warp.execution.auto_confirm
    }
}

/// Create a sample .agentic.toml configuration file
pub async fn create_sample_config() -> Result<()> {
    let _config = AgenticConfig::default();
    
    let sample_path = PathBuf::from(".agentic.toml.sample");
    let content = format!(
        r#"# Agentic CLI Configuration
# This file configures the Warp-mode pipeline for natural language to shell commands

[warp.models]
# Primary planning model (converts natural language to structured plans)
planner = "phi4"

# Primary coding model (converts plans to shell commands)  
coder = "codellama"

# Fallback model (used when primary models fail)
fallback = "gemma3"

# Ollama host configuration
ollama_host = "http://localhost:11434"
timeout_seconds = 30

[warp.execution]
# Enable streaming output (shows command output in real-time)
streaming = true

# Auto-confirm command execution (dangerous! only for trusted environments)
auto_confirm = false

# Maximum execution time in seconds
max_execution_time = 300

# Working directory for command execution (optional)
# working_directory = "/path/to/project"

[warp.safety]
# Enable safety checks for dangerous commands
enable_safety_checks = true

# Require confirmation before executing commands
require_confirmation = true

# Additional dangerous command patterns to block
dangerous_commands = [
    "rm -rf /",
    "del /s /q c:",
    "format c:",
    "shutdown",
    "reboot"
]

# Allowed directories for command execution (empty = no restrictions)
allowed_directories = [
    "~/",
    "./",
    "/tmp/",
    "C:\\temp\\"
]
"#
    );
    
    fs::write(&sample_path, content).await?;
    println!("Sample configuration created at: {}", sample_path.display());
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = AgenticConfig::default();
        
        assert_eq!(config.warp.models.planner, "phi4");
        assert_eq!(config.warp.models.coder, "codellama");
        assert_eq!(config.warp.models.fallback, "gemma3");
        assert!(config.warp.safety.enable_safety_checks);
        assert!(config.warp.execution.streaming);
        assert!(!config.warp.execution.auto_confirm);
    }

    #[test]
    fn test_dangerous_command_detection() {
        let config = AgenticConfig::default();
        
        assert!(config.is_dangerous_command("rm -rf /"));
        assert!(config.is_dangerous_command("shutdown now"));
        assert!(!config.is_dangerous_command("ls -la"));
        assert!(!config.is_dangerous_command("npm install"));
    }

    #[test]
    fn test_directory_allowlist() {
        let config = AgenticConfig::default();
        
        assert!(config.is_directory_allowed("~/projects"));
        assert!(config.is_directory_allowed("./src"));
        assert!(config.is_directory_allowed("/tmp/test"));
    }
}
