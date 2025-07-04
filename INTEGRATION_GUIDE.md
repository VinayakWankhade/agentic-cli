# Warp Integration Guide

This guide explains how to integrate the new Warp-inspired features into your existing Agentic CLI project.

## What We've Built

Based on the comprehensive analysis of Warp's architecture, we've created:

### 1. Complete Directory Structure
```
agentic-cli/
├── themes/                 # Theme configurations (like Warp)
│   ├── base16/            # Base16 color schemes
│   ├── standard/          # Popular themes (Dracula, Nord)
│   ├── warp_bundled/      # Custom Agentic themes
│   └── special_edition/   # Seasonal themes
├── workflows/             # Command workflow definitions
│   ├── git/              # Git operations
│   ├── docker/           # Container management
│   ├── npm/              # Package management
│   └── rust/             # Rust development
├── keysets/              # Keyboard binding configurations
├── config/               # Central configuration
└── src/                  # Enhanced Rust implementation
    ├── themes/           # Theme management system
    ├── workflows/        # Workflow execution engine
    └── keybindings/      # Keybinding management
```

### 2. Rust Modules Created

#### Theme System (`src/themes/`)
- `theme_manager.rs`: Complete theme loading, switching, and management
- Support for hot-reloading themes
- Tag-based categorization and search
- YAML-based theme definitions

#### Workflow System (`src/workflows/`)
- `workflow_manager.rs`: Template-based command execution
- Parameter substitution with validation
- Favorites and search functionality
- Directory-based organization

#### Keybinding System (`src/keybindings/`)
- `keybinding_manager.rs`: Comprehensive key mapping
- Multi-keyset support (default, emacs, vim styles)
- Conflict detection and resolution
- Export/import functionality

### 3. Configuration Files

#### Main Config (`config/settings.yaml`)
- Central configuration management
- AI agent settings (configured for Gemma3)
- Theme, workflow, and keybinding preferences
- UI and performance settings

#### Sample Content Created
- **3 Themes**: Dracula, Nord, and custom Agentic Dark
- **4 Workflows**: Git clone with SSH, Rust project creation, Docker build/run, NPM install
- **1 Keyset**: Default Agentic CLI keybindings with AI-specific shortcuts

## Integration Steps

### Step 1: Update Cargo.toml Dependencies

Add these dependencies to your `Cargo.toml`:

```toml
[dependencies]
# Existing dependencies...
serde_yaml = "0.9"      # For YAML parsing
anyhow = "1.0"          # Already present
crossterm = "0.27"      # Already present  
```

### Step 2: Integrate Modules into src/main.rs

Add the new modules to your main.rs:

```rust
// Add these module declarations
mod themes;
mod workflows; 
mod keybindings;

// Import the managers
use themes::ThemeManager;
use workflows::WorkflowManager;
use keybindings::KeyBindingManager;
```

### Step 3: Update the App State

Modify your main application structure to include the new managers:

```rust
pub struct App {
    // Existing fields...
    pub theme_manager: ThemeManager,
    pub workflow_manager: WorkflowManager,
    pub keybinding_manager: KeyBindingManager,
}

impl App {
    pub fn new() -> Result<Self> {
        let mut theme_manager = ThemeManager::new();
        theme_manager.load_themes()?;
        
        let mut workflow_manager = WorkflowManager::new();
        workflow_manager.load_workflows()?;
        
        let mut keybinding_manager = KeyBindingManager::new();
        keybinding_manager.load_keyset("default_agentic")?;

        Ok(Self {
            // Existing initialization...
            theme_manager,
            workflow_manager,
            keybinding_manager,
        })
    }
}
```

### Step 4: Integrate with Existing UI System

#### Theme Integration (`src/ui/styles.rs`)
```rust
use crate::themes::ThemeManager;

pub fn get_current_theme_colors(theme_manager: &ThemeManager) -> HashMap<String, Color> {
    if let Some(theme) = theme_manager.get_current_theme() {
        // Convert theme colors to ratatui colors
        // Update your existing color scheme
    }
    // Fallback to default colors
}
```

#### Keybinding Integration (`src/ui/events.rs`)
```rust
use crate::keybindings::KeyBindingManager;

pub fn handle_key_event(
    key_event: KeyEvent, 
    keybinding_manager: &KeyBindingManager,
    app: &mut App
) -> Result<()> {
    if let Some(command) = keybinding_manager.get_command_for_key(&key_event) {
        match command.as_str() {
            "agent:toggle_mode" => app.toggle_agent_mode(),
            "terminal:copy" => app.copy_selection(),
            "workflow:show_palette" => app.show_workflow_palette(),
            // Add more command handlers...
            _ => {}
        }
    }
    Ok(())
}
```

### Step 5: Add CLI Commands

Extend your existing command system to support the new features:

```rust
// Add to src/commands/mod.rs
pub mod theme;
pub mod workflow;
pub mod keybinding;

// Add to your CLI structure
#[derive(Subcommand)]
pub enum Commands {
    // Existing commands...
    
    /// Theme management
    #[command(subcommand)]
    Theme(theme::ThemeCommands),
    
    /// Workflow execution
    #[command(subcommand)]  
    Workflow(workflow::WorkflowCommands),
    
    /// Keybinding management
    #[command(subcommand)]
    Keys(keybinding::KeybindingCommands),
}
```

### Step 6: Enhance AI Integration

Update your agent system to work with workflows:

```rust
// In src/agent/mod.rs
use crate::workflows::WorkflowManager;

impl Agent {
    pub fn suggest_workflow(&self, query: &str, workflow_manager: &WorkflowManager) -> Vec<String> {
        // AI suggests relevant workflows based on natural language query
        let suggestions = workflow_manager.search_workflows(query);
        // Return workflow IDs that match the intent
    }
    
    pub fn explain_workflow(&self, workflow_id: &str, workflow_manager: &WorkflowManager) -> String {
        if let Some(workflow) = workflow_manager.get_workflow(workflow_id) {
            // AI explains what the workflow does and its parameters
            format!("This workflow {} does: {}", workflow.name, workflow.description)
        } else {
            "Workflow not found".to_string()
        }
    }
}
```

## Usage Examples

Once integrated, users can:

### Theme Management
```bash
# Switch to Dracula theme
agentic theme set dracula

# List all themes  
agentic theme list

# Search for dark themes
agentic theme search dark
```

### Workflow Execution
```bash
# Execute a git clone workflow
agentic workflow exec git/clone_with_ssh \
  --repositoryUrl git@github.com:user/repo.git \
  --targetFolder my-project

# List available workflows
agentic workflow list

# Search for Docker workflows  
agentic workflow search docker
```

### Keybinding Management
```bash
# Show current keybindings
agentic keys list

# Switch to Emacs-style keybindings
agentic keys set emacs
```

### In TUI Mode
- `Ctrl+Shift+T`: Open theme selector
- `Ctrl+Shift+R`: Open workflow palette  
- `Ctrl+Shift+A`: Toggle AI agent mode
- `Ctrl+Space`: AI suggest workflow for current context

## AI Enhancement Opportunities

The new architecture enables enhanced AI features:

1. **Context-Aware Suggestions**: AI can suggest workflows based on current directory, git state, etc.
2. **Natural Language to Workflow**: "create a new rust project" → executes rust/create_new_project workflow
3. **Workflow Learning**: AI learns user preferences and suggests favorite workflows
4. **Theme Recommendations**: AI suggests themes based on time of day, project type, etc.

## Next Steps

1. **Implement the integration steps** above
2. **Test the new modules** with your existing codebase
3. **Add more themes and workflows** based on user needs
4. **Enhance AI integration** with workflow suggestions
5. **Create a workflow recorder** to capture user commands as workflows
6. **Add plugin system** for community contributions

This Warp-inspired architecture transforms Agentic CLI from a simple terminal replacement into a comprehensive development environment that adapts to user workflows and preferences while maintaining the power and flexibility developers need.

<citations>
<document>
<document_type>RULE</document_type>
<document_id>x0aMNjHcZfSHLPmaUY1ARN</document_id>
</document>
</citations>
