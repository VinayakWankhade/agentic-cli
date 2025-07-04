use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TerminalColors {
    pub normal: HashMap<String, String>,
    pub bright: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Theme {
    pub name: String,
    pub accent: String,
    pub background: String,
    pub details: String,
    pub foreground: String,
    pub terminal_colors: TerminalColors,
    #[serde(default)]
    pub author: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub tags: Vec<String>,
}

pub struct ThemeManager {
    themes: HashMap<String, Theme>,
    theme_directories: Vec<PathBuf>,
    current_theme: Option<String>,
}

impl ThemeManager {
    pub fn new() -> Self {
        Self {
            themes: HashMap::new(),
            theme_directories: vec![
                PathBuf::from("themes"),
                PathBuf::from("~/.agentic/themes"),
            ],
            current_theme: None,
        }
    }

    pub fn add_theme_directory<P: AsRef<Path>>(&mut self, path: P) {
        self.theme_directories.push(path.as_ref().to_path_buf());
    }

    pub fn load_themes(&mut self) -> Result<()> {
        for theme_dir in &self.theme_directories.clone() {
            if theme_dir.exists() {
                self.load_themes_from_directory(theme_dir)?;
            }
        }
        Ok(())
    }

    fn load_themes_from_directory(&mut self, dir: &Path) -> Result<()> {
        let entries = fs::read_dir(dir)
            .with_context(|| format!("Failed to read theme directory: {:?}", dir))?;

        for entry in entries {
            let entry = entry?;
            let path = entry.path();
            
            if path.is_dir() {
                // Recursively load themes from subdirectories
                self.load_themes_from_directory(&path)?;
            } else if path.extension().and_then(|s| s.to_str()) == Some("yaml") 
                   || path.extension().and_then(|s| s.to_str()) == Some("yml") {
                if let Ok(theme) = self.load_theme_from_file(&path) {
                    self.themes.insert(theme.name.clone(), theme);
                }
            }
        }
        Ok(())
    }

    fn load_theme_from_file(&self, path: &Path) -> Result<Theme> {
        let content = fs::read_to_string(path)
            .with_context(|| format!("Failed to read theme file: {:?}", path))?;
        
        let mut theme: Theme = serde_yaml::from_str(&content)
            .with_context(|| format!("Failed to parse theme YAML: {:?}", path))?;
        
        // If theme doesn't have a name, use filename
        if theme.name.is_empty() {
            if let Some(file_stem) = path.file_stem().and_then(|s| s.to_str()) {
                theme.name = file_stem.to_string();
            }
        }
        
        Ok(theme)
    }

    pub fn get_theme(&self, name: &str) -> Option<&Theme> {
        self.themes.get(name)
    }

    pub fn list_themes(&self) -> Vec<&Theme> {
        self.themes.values().collect()
    }

    pub fn list_themes_by_tag(&self, tag: &str) -> Vec<&Theme> {
        self.themes
            .values()
            .filter(|theme| theme.tags.contains(&tag.to_string()))
            .collect()
    }

    pub fn set_current_theme(&mut self, name: &str) -> Result<()> {
        if self.themes.contains_key(name) {
            self.current_theme = Some(name.to_string());
            Ok(())
        } else {
            anyhow::bail!("Theme '{}' not found", name)
        }
    }

    pub fn get_current_theme(&self) -> Option<&Theme> {
        self.current_theme
            .as_ref()
            .and_then(|name| self.themes.get(name))
    }

    pub fn reload_themes(&mut self) -> Result<()> {
        self.themes.clear();
        self.load_themes()
    }

    pub fn search_themes(&self, query: &str) -> Vec<&Theme> {
        let query = query.to_lowercase();
        self.themes
            .values()
            .filter(|theme| {
                theme.name.to_lowercase().contains(&query)
                    || theme.description
                        .as_ref()
                        .map(|d| d.to_lowercase().contains(&query))
                        .unwrap_or(false)
                    || theme.tags.iter().any(|tag| tag.to_lowercase().contains(&query))
            })
            .collect()
    }

    pub fn get_theme_categories(&self) -> HashMap<String, Vec<&Theme>> {
        let mut categories = HashMap::new();
        
        for theme in self.themes.values() {
            if theme.tags.contains(&"dark".to_string()) {
                categories.entry("Dark".to_string()).or_insert_with(Vec::new).push(theme);
            }
            if theme.tags.contains(&"light".to_string()) {
                categories.entry("Light".to_string()).or_insert_with(Vec::new).push(theme);
            }
            if theme.tags.contains(&"popular".to_string()) {
                categories.entry("Popular".to_string()).or_insert_with(Vec::new).push(theme);
            }
            if theme.tags.contains(&"minimal".to_string()) {
                categories.entry("Minimal".to_string()).or_insert_with(Vec::new).push(theme);
            }
        }
        
        categories
    }
}

impl Default for ThemeManager {
    fn default() -> Self {
        Self::new()
    }
}
