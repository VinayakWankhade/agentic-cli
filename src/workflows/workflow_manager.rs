use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowArgument {
    pub name: String,
    pub description: String,
    pub default_value: Option<String>,
    #[serde(default)]
    pub required: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Workflow {
    pub name: String,
    pub command: String,
    pub description: String,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub arguments: Vec<WorkflowArgument>,
    #[serde(default)]
    pub author: Option<String>,
    #[serde(default)]
    pub author_url: Option<String>,
    #[serde(default)]
    pub source_url: Option<String>,
    #[serde(default)]
    pub shells: Vec<String>,
}

pub struct WorkflowManager {
    workflows: HashMap<String, Workflow>,
    workflow_directories: Vec<PathBuf>,
    favorites: Vec<String>,
}

impl WorkflowManager {
    pub fn new() -> Self {
        Self {
            workflows: HashMap::new(),
            workflow_directories: vec![
                PathBuf::from("workflows"),
                PathBuf::from("~/.agentic/workflows"),
            ],
            favorites: Vec::new(),
        }
    }

    pub fn add_workflow_directory<P: AsRef<Path>>(&mut self, path: P) {
        self.workflow_directories.push(path.as_ref().to_path_buf());
    }

    pub fn load_workflows(&mut self) -> Result<()> {
        for workflow_dir in &self.workflow_directories.clone() {
            if workflow_dir.exists() {
                self.load_workflows_from_directory(workflow_dir)?;
            }
        }
        Ok(())
    }

    fn load_workflows_from_directory(&mut self, dir: &Path) -> Result<()> {
        let entries = fs::read_dir(dir)
            .with_context(|| format!("Failed to read workflow directory: {:?}", dir))?;

        for entry in entries {
            let entry = entry?;
            let path = entry.path();
            
            if path.is_dir() {
                // Recursively load workflows from subdirectories
                self.load_workflows_from_directory(&path)?;
            } else if path.extension().and_then(|s| s.to_str()) == Some("yaml") 
                   || path.extension().and_then(|s| s.to_str()) == Some("yml") {
                if let Ok(workflow) = self.load_workflow_from_file(&path) {
                    // Use relative path as ID (e.g., "git/clone_with_ssh")
                    let id = self.generate_workflow_id(&path, dir);
                    self.workflows.insert(id, workflow);
                }
            }
        }
        Ok(())
    }

    fn generate_workflow_id(&self, file_path: &Path, base_dir: &Path) -> String {
        if let Ok(relative_path) = file_path.strip_prefix(base_dir) {
            let mut id = relative_path.to_string_lossy().to_string();
            // Remove file extension
            if let Some(pos) = id.rfind('.') {
                id.truncate(pos);
            }
            // Replace backslashes with forward slashes for consistency
            id.replace('\\', "/")
        } else {
            // Fallback to just filename without extension
            file_path
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("unknown")
                .to_string()
        }
    }

    fn load_workflow_from_file(&self, path: &Path) -> Result<Workflow> {
        let content = fs::read_to_string(path)
            .with_context(|| format!("Failed to read workflow file: {:?}", path))?;
        
        let workflow: Workflow = serde_yaml::from_str(&content)
            .with_context(|| format!("Failed to parse workflow YAML: {:?}", path))?;
        
        Ok(workflow)
    }

    pub fn get_workflow(&self, id: &str) -> Option<&Workflow> {
        self.workflows.get(id)
    }

    pub fn list_workflows(&self) -> Vec<(&String, &Workflow)> {
        self.workflows.iter().collect()
    }

    pub fn search_workflows(&self, query: &str) -> Vec<(&String, &Workflow)> {
        let query = query.to_lowercase();
        self.workflows
            .iter()
            .filter(|(id, workflow)| {
                id.to_lowercase().contains(&query)
                    || workflow.name.to_lowercase().contains(&query)
                    || workflow.description.to_lowercase().contains(&query)
                    || workflow.tags.iter().any(|tag| tag.to_lowercase().contains(&query))
            })
            .collect()
    }

    pub fn get_workflows_by_tag(&self, tag: &str) -> Vec<(&String, &Workflow)> {
        self.workflows
            .iter()
            .filter(|(_, workflow)| workflow.tags.contains(&tag.to_string()))
            .collect()
    }

    pub fn get_workflow_categories(&self) -> HashMap<String, Vec<(&String, &Workflow)>> {
        let mut categories = HashMap::new();
        
        for (id, workflow) in &self.workflows {
            for tag in &workflow.tags {
                categories
                    .entry(tag.clone())
                    .or_insert_with(Vec::new)
                    .push((id, workflow));
            }
        }
        
        categories
    }

    pub fn add_favorite(&mut self, workflow_id: &str) {
        if !self.favorites.contains(&workflow_id.to_string()) {
            self.favorites.push(workflow_id.to_string());
        }
    }

    pub fn remove_favorite(&mut self, workflow_id: &str) {
        self.favorites.retain(|id| id != workflow_id);
    }

    pub fn get_favorites(&self) -> Vec<(&String, &Workflow)> {
        self.favorites
            .iter()
            .filter_map(|id| self.workflows.get(id).map(|workflow| (id, workflow)))
            .collect()
    }

    pub fn is_favorite(&self, workflow_id: &str) -> bool {
        self.favorites.contains(&workflow_id.to_string())
    }

    pub fn execute_workflow(&self, workflow_id: &str, args: HashMap<String, String>) -> Result<String> {
        let workflow = self.get_workflow(workflow_id)
            .ok_or_else(|| anyhow::anyhow!("Workflow '{}' not found", workflow_id))?;

        let mut command = workflow.command.clone();
        
        // Replace placeholders with provided arguments
        for (key, value) in args {
            let placeholder = format!("{{{{{}}}}}", key);
            command = command.replace(&placeholder, &value);
        }
        
        // Replace remaining placeholders with default values
        for arg in &workflow.arguments {
            let placeholder = format!("{{{{{}}}}}", arg.name);
            if command.contains(&placeholder) {
                if let Some(default_value) = &arg.default_value {
                    command = command.replace(&placeholder, default_value);
                } else if arg.required {
                    anyhow::bail!("Required argument '{}' not provided for workflow '{}'", arg.name, workflow_id);
                }
            }
        }
        
        Ok(command)
    }

    pub fn validate_workflow_args(&self, workflow_id: &str, args: &HashMap<String, String>) -> Result<()> {
        let workflow = self.get_workflow(workflow_id)
            .ok_or_else(|| anyhow::anyhow!("Workflow '{}' not found", workflow_id))?;

        for arg in &workflow.arguments {
            if arg.required && !args.contains_key(&arg.name) && arg.default_value.is_none() {
                anyhow::bail!("Required argument '{}' missing for workflow '{}'", arg.name, workflow_id);
            }
        }
        
        Ok(())
    }

    pub fn reload_workflows(&mut self) -> Result<()> {
        self.workflows.clear();
        self.load_workflows()
    }

    pub fn get_workflow_suggestions(&self, partial_input: &str) -> Vec<(&String, &Workflow)> {
        let partial = partial_input.to_lowercase();
        self.workflows
            .iter()
            .filter(|(id, workflow)| {
                id.to_lowercase().starts_with(&partial)
                    || workflow.name.to_lowercase().contains(&partial)
            })
            .take(10) // Limit suggestions
            .collect()
    }
}

impl Default for WorkflowManager {
    fn default() -> Self {
        Self::new()
    }
}
