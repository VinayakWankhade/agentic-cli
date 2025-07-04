# Warp-Inspired Architecture

This document describes how Agentic CLI implements the same architectural patterns and functionality as Warp terminal, creating a powerful and extensible terminal experience.

## Overview

Agentic CLI is built with the same modular architecture as Warp, featuring:
- **Theme System**: Dynamic color schemes and visual customization
- **Workflow System**: Pre-defined command patterns and automation
- **Keybinding System**: Customizable keyboard shortcuts and command mapping
- **AI Integration**: Enhanced with Gemma3 for intelligent assistance
- **Modern UI**: Block-based command execution with real-time feedback

## Core Components

### 1. Theme Management (`src/themes/`)

The theme system provides dynamic visual customization similar to Warp's theming engine.

#### Features:
- **Multi-category Support**: Base16, standard, custom, and special edition themes
- **Hot Reloading**: Themes can be switched without restarting
- **YAML Configuration**: Easy-to-read theme definitions
- **Tag-based Organization**: Themes grouped by color scheme, popularity, etc.

#### Structure:
```
themes/
├── base16/           # Classic Base16 color schemes
├── standard/         # Popular terminal themes (Dracula, Nord, etc.)
├── warp_bundled/     # Custom Agentic CLI themes
└── special_edition/  # Seasonal and limited themes
```

#### Example Theme (Dracula):
```yaml
name: Dracula
accent: "#bd93f9"
background: "#282a36"
foreground: "#f8f8f2"
terminal_colors:
  normal:
    black: "#000000"
    red: "#ff5555"
    green: "#50fa7b"
    # ... more colors
```

### 2. Workflow System (`src/workflows/`)

The workflow system enables users to define and execute complex command patterns with templating and parameterization.

#### Features:
- **Template Variables**: Use `{{variable}}` syntax for dynamic command generation
- **Argument Validation**: Required and optional parameters with defaults
- **Category Organization**: Workflows grouped by technology (git, docker, npm, etc.)
- **Search and Discovery**: Find workflows by name, description, or tags
- **Favorites System**: Bookmark frequently used workflows

#### Structure:
```
workflows/
├── git/              # Git-related workflows
├── docker/           # Docker and containerization
├── npm/              # Node.js package management
├── rust/             # Rust development workflows
└── ...               # Other technology categories
```

#### Example Workflow (Git Clone with SSH):
```yaml
name: Clone git repository with specific SSH Key and User
command: |-
  git -c core.sshCommand='ssh -i {{sshKeyPath}} -o IdentitiesOnly=yes' clone {{repositoryUrl}} {{targetFolder}}
  cd {{targetFolder}}
  git config core.sshCommand 'ssh -i {{sshKeyPath}}'
  git config user.name "{{userName}}"
  git config user.email {{userEmail}}
arguments:
  - name: sshKeyPath
    description: The path of the SSH Key to be used
    default_value: ~/.ssh/id_rsa
  - name: repositoryUrl
    description: The SSH URL of the git repository
    required: true
tags: [git, ssh, clone]
```

### 3. Keybinding System (`src/keybindings/`)

Provides comprehensive keyboard shortcut management with support for multiple keyset presets.

#### Features:
- **Multi-modal Bindings**: Support for different operating systems and preferences
- **Command Categories**: Organized by functionality (editor, terminal, workspace, etc.)
- **Conflict Detection**: Prevents duplicate key assignments
- **Export/Import**: Save and share custom keybinding sets

#### Structure:
```
keysets/
├── default_agentic_keybindings.yaml  # Default Agentic CLI bindings
├── emacs.yaml                        # Emacs-style keybindings
└── vim.yaml                          # Vim-style keybindings
```

#### Example Keybindings:
```yaml
# AI Agent Commands
"agent:toggle_mode": ctrl-shift-a
"agent:explain_command": ctrl-shift-e
"agent:suggest_command": ctrl-space

# Terminal Commands
"terminal:copy": cmd-c
"terminal:paste": cmd-v
"terminal:find": cmd-f

# Workspace Commands
"workspace:new_tab": cmd-t
"workspace:close_tab": cmd-w
```

### 4. Configuration System (`config/settings.yaml`)

Central configuration management that ties all systems together.

#### Key Sections:
- **Theme Configuration**: Current theme, font settings, colors
- **AI Agent Settings**: Model selection (Gemma3), API configuration
- **Workflow Preferences**: Enabled workflows, search paths, favorites
- **Keybinding Presets**: Active keyset and custom overrides
- **UI Preferences**: Layout, animations, performance settings

### 5. Rust Implementation

#### Theme Manager (`src/themes/theme_manager.rs`)
```rust
pub struct ThemeManager {
    themes: HashMap<String, Theme>,
    theme_directories: Vec<PathBuf>,
    current_theme: Option<String>,
}
```

Key methods:
- `load_themes()`: Discovers and loads all available themes
- `set_current_theme()`: Switches active theme
- `search_themes()`: Find themes by query
- `get_theme_categories()`: Group themes by tags

#### Workflow Manager (`src/workflows/workflow_manager.rs`)
```rust
pub struct WorkflowManager {
    workflows: HashMap<String, Workflow>,
    workflow_directories: Vec<PathBuf>,
    favorites: Vec<String>,
}
```

Key methods:
- `load_workflows()`: Discovers workflows from directories
- `execute_workflow()`: Processes templates and executes commands
- `search_workflows()`: Find workflows by criteria
- `validate_workflow_args()`: Ensures required parameters are provided

#### Keybinding Manager (`src/keybindings/keybinding_manager.rs`)
```rust
pub struct KeyBindingManager {
    bindings: HashMap<String, KeyBinding>,
    reverse_bindings: HashMap<KeyBinding, String>,
    current_keyset: Option<String>,
}
```

Key methods:
- `load_keyset()`: Loads keybinding configuration
- `get_command_for_key()`: Maps key events to commands
- `add_binding()`: Creates new key bindings
- `export_keyset()`: Saves custom keybinding sets

## Integration with Existing Agentic CLI

### AI Enhancement
The Warp-inspired architecture enhances the existing AI capabilities:
- **Smart Workflow Suggestions**: AI recommends relevant workflows based on context
- **Natural Language to Workflow**: Convert plain English to workflow execution
- **Command Explanation**: AI explains complex workflow commands
- **Theme Recommendations**: Suggest themes based on usage patterns

### TUI Integration
The theme and keybinding systems integrate seamlessly with the existing ratatui-based interface:
- **Dynamic Theme Application**: Colors and styles update in real-time
- **Contextual Keybindings**: Different key mappings for different UI modes
- **Workflow Palette**: Quick access to workflows via command palette

### Command System Enhancement
Workflows extend the existing command system:
- **Task Workflows**: Pre-defined patterns for task management
- **Development Workflows**: Common development operations
- **System Administration**: Server and infrastructure management

## Usage Examples

### Theme Management
```bash
# List available themes
agentic theme list

# Switch to Dracula theme
agentic theme set dracula

# Search for dark themes
agentic theme search dark

# Create custom theme
agentic theme create my-theme --base dracula
```

### Workflow Execution
```bash
# List available workflows
agentic workflow list

# Execute git clone workflow
agentic workflow exec git/clone_with_ssh \
  --repositoryUrl git@github.com:user/repo.git \
  --targetFolder my-project

# Search for Docker workflows
agentic workflow search docker

# Add workflow to favorites
agentic workflow favorite docker/build_and_run_app
```

### Keybinding Management
```bash
# Show current keybindings
agentic keys list

# Switch to Emacs keybindings
agentic keys set emacs

# Add custom binding
agentic keys bind "terminal:new_tab" "ctrl-shift-t"

# Export current keybindings
agentic keys export ~/.agentic/my-keys.yaml
```

## Benefits of Warp-Inspired Architecture

1. **Modularity**: Each system (themes, workflows, keybindings) is independent and extensible
2. **User Customization**: Deep personalization options for power users
3. **Community Contributions**: Easy for users to create and share themes/workflows
4. **Professional Experience**: Modern, polished interface comparable to commercial tools
5. **Productivity**: Pre-defined workflows and intelligent assistance accelerate development

## Future Enhancements

- **Plugin System**: WASM-based plugins for custom functionality
- **Cloud Sync**: Synchronize themes, workflows, and settings across devices
- **Theme Editor**: Visual theme creation and editing interface
- **Workflow Recorder**: Record terminal sessions to create new workflows
- **Advanced AI Integration**: Context-aware suggestions and automation

This architecture makes Agentic CLI not just a terminal replacement, but a comprehensive development environment that learns and adapts to user workflows while maintaining the power and flexibility that developers need.

<citations>
<document>
<document_type>RULE</document_type>
<document_id>x0aMNjHcZfSHLPmaUY1ARN</document_id>
</document>
</citations>
