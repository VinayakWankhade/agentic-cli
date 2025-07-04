use anyhow::{Context, Result};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct KeyBinding {
    pub key: KeyCode,
    pub modifiers: KeyModifiers,
}

impl KeyBinding {
    pub fn new(key: KeyCode, modifiers: KeyModifiers) -> Self {
        Self { key, modifiers }
    }

    pub fn from_string(key_str: &str) -> Result<Self> {
        let mut modifiers = KeyModifiers::empty();
        let parts: Vec<&str> = key_str.split('-').collect();
        
        if parts.is_empty() {
            anyhow::bail!("Invalid key binding format: {}", key_str);
        }

        let key_part = parts.last().unwrap();
        
        // Parse modifiers
        for part in &parts[..parts.len() - 1] {
            match part.to_lowercase().as_str() {
                "ctrl" => modifiers |= KeyModifiers::CONTROL,
                "alt" => modifiers |= KeyModifiers::ALT,
                "shift" => modifiers |= KeyModifiers::SHIFT,
                "cmd" => modifiers |= KeyModifiers::CONTROL, // Map cmd to ctrl on non-mac
                "meta" => modifiers |= KeyModifiers::ALT,    // Map meta to alt
                _ => anyhow::bail!("Unknown modifier: {}", part),
            }
        }

        // Parse key
        let key = match key_part.to_lowercase().as_str() {
            "enter" => KeyCode::Enter,
            "escape" | "esc" => KeyCode::Esc,
            "space" => KeyCode::Char(' '),
            "tab" => KeyCode::Tab,
            "backspace" => KeyCode::Backspace,
            "delete" => KeyCode::Delete,
            "up" => KeyCode::Up,
            "down" => KeyCode::Down,
            "left" => KeyCode::Left,
            "right" => KeyCode::Right,
            "home" => KeyCode::Home,
            "end" => KeyCode::End,
            "pageup" => KeyCode::PageUp,
            "pagedown" => KeyCode::PageDown,
            "f1" => KeyCode::F(1),
            "f2" => KeyCode::F(2),
            "f3" => KeyCode::F(3),
            "f4" => KeyCode::F(4),
            "f5" => KeyCode::F(5),
            "f6" => KeyCode::F(6),
            "f7" => KeyCode::F(7),
            "f8" => KeyCode::F(8),
            "f9" => KeyCode::F(9),
            "f10" => KeyCode::F(10),
            "f11" => KeyCode::F(11),
            "f12" => KeyCode::F(12),
            "grave" => KeyCode::Char('`'),
            "slash" => KeyCode::Char('/'),
            "comma" => KeyCode::Char(','),
            "period" => KeyCode::Char('.'),
            key if key.len() == 1 => {
                let ch = key.chars().next().unwrap();
                KeyCode::Char(ch)
            }
            _ => anyhow::bail!("Unknown key: {}", key_part),
        };

        Ok(KeyBinding { key, modifiers })
    }

    pub fn matches(&self, event: &KeyEvent) -> bool {
        self.key == event.code && self.modifiers == event.modifiers
    }
}

pub struct KeyBindingManager {
    bindings: HashMap<String, KeyBinding>,
    reverse_bindings: HashMap<KeyBinding, String>,
    keyset_directories: Vec<PathBuf>,
    current_keyset: Option<String>,
}

impl KeyBindingManager {
    pub fn new() -> Self {
        Self {
            bindings: HashMap::new(),
            reverse_bindings: HashMap::new(),
            keyset_directories: vec![
                PathBuf::from("keysets"),
                PathBuf::from("~/.agentic/keysets"),
            ],
            current_keyset: None,
        }
    }

    pub fn add_keyset_directory<P: AsRef<Path>>(&mut self, path: P) {
        self.keyset_directories.push(path.as_ref().to_path_buf());
    }

    pub fn load_keyset(&mut self, keyset_name: &str) -> Result<()> {
        let mut keyset_path = None;
        
        // Find the keyset file
        for dir in &self.keyset_directories {
            let path = dir.join(format!("{}.yaml", keyset_name));
            if path.exists() {
                keyset_path = Some(path);
                break;
            }
        }

        let path = keyset_path
            .ok_or_else(|| anyhow::anyhow!("Keyset '{}' not found", keyset_name))?;

        let content = fs::read_to_string(&path)
            .with_context(|| format!("Failed to read keyset file: {:?}", path))?;

        let keyset_data: HashMap<String, String> = serde_yaml::from_str(&content)
            .with_context(|| format!("Failed to parse keyset YAML: {:?}", path))?;

        // Clear existing bindings
        self.bindings.clear();
        self.reverse_bindings.clear();

        // Load new bindings
        for (command, key_str) in keyset_data {
            if let Ok(key_binding) = KeyBinding::from_string(&key_str) {
                self.bindings.insert(command.clone(), key_binding.clone());
                self.reverse_bindings.insert(key_binding, command);
            }
        }

        self.current_keyset = Some(keyset_name.to_string());
        Ok(())
    }

    pub fn get_command_for_key(&self, event: &KeyEvent) -> Option<&String> {
        let key_binding = KeyBinding {
            key: event.code,
            modifiers: event.modifiers,
        };
        self.reverse_bindings.get(&key_binding)
    }

    pub fn get_key_for_command(&self, command: &str) -> Option<&KeyBinding> {
        self.bindings.get(command)
    }

    pub fn add_binding(&mut self, command: String, key_binding: KeyBinding) {
        // Remove any existing binding for this key
        if let Some(old_command) = self.reverse_bindings.remove(&key_binding) {
            self.bindings.remove(&old_command);
        }
        
        // Remove any existing binding for this command
        if let Some(old_key) = self.bindings.get(&command).cloned() {
            self.reverse_bindings.remove(&old_key);
        }

        self.bindings.insert(command.clone(), key_binding.clone());
        self.reverse_bindings.insert(key_binding, command);
    }

    pub fn remove_binding(&mut self, command: &str) {
        if let Some(key_binding) = self.bindings.remove(command) {
            self.reverse_bindings.remove(&key_binding);
        }
    }

    pub fn list_bindings(&self) -> Vec<(&String, &KeyBinding)> {
        self.bindings.iter().collect()
    }

    pub fn get_bindings_by_category(&self) -> HashMap<String, Vec<(&String, &KeyBinding)>> {
        let mut categories = HashMap::new();
        
        for (command, key_binding) in &self.bindings {
            let category = if let Some(colon_pos) = command.find(':') {
                command[..colon_pos].to_string()
            } else {
                "general".to_string()
            };
            
            categories
                .entry(category)
                .or_insert_with(Vec::new)
                .push((command, key_binding));
        }
        
        categories
    }

    pub fn search_bindings(&self, query: &str) -> Vec<(&String, &KeyBinding)> {
        let query = query.to_lowercase();
        self.bindings
            .iter()
            .filter(|(command, _)| command.to_lowercase().contains(&query))
            .collect()
    }

    pub fn export_keyset(&self, path: &Path) -> Result<()> {
        let mut keyset_data = HashMap::new();
        
        for (command, key_binding) in &self.bindings {
            let key_str = self.key_binding_to_string(key_binding);
            keyset_data.insert(command.clone(), key_str);
        }

        let yaml = serde_yaml::to_string(&keyset_data)
            .context("Failed to serialize keyset to YAML")?;

        fs::write(path, yaml)
            .with_context(|| format!("Failed to write keyset to: {:?}", path))?;

        Ok(())
    }

    fn key_binding_to_string(&self, key_binding: &KeyBinding) -> String {
        let mut parts = Vec::new();

        if key_binding.modifiers.contains(KeyModifiers::CONTROL) {
            parts.push("ctrl");
        }
        if key_binding.modifiers.contains(KeyModifiers::ALT) {
            parts.push("alt");
        }
        if key_binding.modifiers.contains(KeyModifiers::SHIFT) {
            parts.push("shift");
        }

        let key_str = match key_binding.key {
            KeyCode::Enter => "enter".to_string(),
            KeyCode::Esc => "escape".to_string(),
            KeyCode::Char(' ') => "space".to_string(),
            KeyCode::Char('`') => "grave".to_string(),
            KeyCode::Char('/') => "slash".to_string(),
            KeyCode::Char(',') => "comma".to_string(),
            KeyCode::Char('.') => "period".to_string(),
            KeyCode::Char(c) => c.to_string(),
            KeyCode::Tab => "tab".to_string(),
            KeyCode::Backspace => "backspace".to_string(),
            KeyCode::Delete => "delete".to_string(),
            KeyCode::Up => "up".to_string(),
            KeyCode::Down => "down".to_string(),
            KeyCode::Left => "left".to_string(),
            KeyCode::Right => "right".to_string(),
            KeyCode::Home => "home".to_string(),
            KeyCode::End => "end".to_string(),
            KeyCode::PageUp => "pageup".to_string(),
            KeyCode::PageDown => "pagedown".to_string(),
            KeyCode::F(n) => format!("f{}", n),
            _ => "unknown".to_string(),
        };

        parts.push(&key_str);
        parts.join("-")
    }

    pub fn get_current_keyset(&self) -> Option<&String> {
        self.current_keyset.as_ref()
    }

    pub fn has_binding(&self, command: &str) -> bool {
        self.bindings.contains_key(command)
    }

    pub fn validate_key_string(key_str: &str) -> bool {
        KeyBinding::from_string(key_str).is_ok()
    }
}

impl Default for KeyBindingManager {
    fn default() -> Self {
        Self::new()
    }
}
