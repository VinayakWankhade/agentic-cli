use anyhow::Result;
use dirs::home_dir;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tokio::fs;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub database_path: PathBuf,
    pub openai_api_key: Option<String>,
    pub theme: Theme,
    pub agent: AgentConfig,
    pub aliases: std::collections::HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Theme {
    pub dark_mode: bool,
    pub primary_color: String,
    pub secondary_color: String,
    pub accent_color: String,
    pub background_color: String,
    pub text_color: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentConfig {
    pub model: String,
    pub temperature: f32,
    pub max_tokens: u32,
    pub timeout_seconds: u64,
}

impl Default for Config {
    fn default() -> Self {
        let home = home_dir().unwrap_or_else(|| PathBuf::from("."));
        let config_dir = home.join(".agentic");
        
        Self {
            database_path: config_dir.join("history.db"),
            openai_api_key: None,
            theme: Theme::default(),
            agent: AgentConfig::default(),
            aliases: std::collections::HashMap::new(),
        }
    }
}

impl Default for Theme {
    fn default() -> Self {
        Self {
            dark_mode: true,
            primary_color: "#61dafb".to_string(),
            secondary_color: "#282c34".to_string(),
            accent_color: "#98c379".to_string(),
            background_color: "#1e1e1e".to_string(),
            text_color: "#ffffff".to_string(),
        }
    }
}

impl Default for AgentConfig {
    fn default() -> Self {
        Self {
            model: "gpt-3.5-turbo".to_string(),
            temperature: 0.7,
            max_tokens: 1000,
            timeout_seconds: 30,
        }
    }
}

impl Config {
    pub async fn load() -> Result<Self> {
        let config_path = Self::config_path();
        
        if config_path.exists() {
            let content = fs::read_to_string(&config_path).await?;
            let config: Config = toml::from_str(&content)?;
            Ok(config)
        } else {
            let config = Config::default();
            config.save().await?;
            Ok(config)
        }
    }
    
    pub async fn save(&self) -> Result<()> {
        let config_path = Self::config_path();
        
        if let Some(parent) = config_path.parent() {
            fs::create_dir_all(parent).await?;
        }
        
        let content = toml::to_string_pretty(self)?;
        fs::write(&config_path, content).await?;
        
        Ok(())
    }
    
    fn config_path() -> PathBuf {
        let home = home_dir().unwrap_or_else(|| PathBuf::from("."));
        home.join(".agentic").join("config.toml")
    }
    
    pub fn get_openai_api_key(&self) -> Option<String> {
        self.openai_api_key.clone()
            .or_else(|| std::env::var("OPENAI_API_KEY").ok())
    }
}
