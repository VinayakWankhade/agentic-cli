# Agentic CLI

A Warp-inspired agentic terminal interface built in Rust, providing a rich, modular, and intelligent interface for executing commands, interacting with AI agents, and rendering structured output in blocks.

## ✨ Features

### 🚀 Performance-First
- **Native & Fast**: Built entirely in Rust with no Node.js or Electron dependencies
- **Efficient TUI**: Uses `ratatui` for blazing-fast terminal rendering
- **Minimal Resource Usage**: Optimized for low memory and CPU usage

### 🤖 Intelligent Agent System
- **Natural Language Processing**: Convert natural language into actionable CLI commands
- **OpenAI Integration**: Built-in support for OpenAI API with fallback modes
- **Context-Aware**: Maintains conversation history and learns from interactions
- **Command Planning**: Advanced execution planning with dependency resolution

### 🎯 Rich Terminal Interface
- **Block-Based Rendering**: Command executions displayed as interactive blocks
- **Real-Time Status**: Live status indicators (✅ success, ⏳ running, ❌ error)
- **Multiple Layouts**: Horizontal/vertical splits with customizable panels
- **Scrollable History**: Persistent command history with search capabilities

### 📦 Modular Command System
- **Task Management**: Complete task lifecycle with priorities and statuses
- **Exam Preparation**: Structured study sessions with progress tracking
- **Blog Management**: Content creation and publishing workflows
- **Extensible**: Easy plugin system for custom commands

### ⌨️ Advanced Input System
- **Dual Modes**: Switch between natural language (🤖) and raw CLI ($) modes
- **Smart Autocomplete**: Context-aware command suggestions
- **Command History**: Navigate through previous commands with ↑/↓
- **Keyboard Shortcuts**: Vim-inspired keybindings for power users

### 🎨 Customizable Themes
- **Light/Dark Modes**: Toggle between themes
- **Syntax Highlighting**: Colorful output with semantic highlighting
- **Smooth Animations**: Subtle transitions and visual feedback
- **Responsive Design**: Adapts to terminal size and preferences

## 🛠️ Tech Stack

- **Language**: Rust 2021 Edition
- **TUI Library**: `ratatui` 0.26+ for terminal rendering
- **Terminal Backend**: `crossterm` for cross-platform terminal control
- **CLI Parser**: `clap` 4.x with derive macros for ergonomic CLI design
- **Async Runtime**: `tokio` for high-performance async operations
- **AI Integration**: `reqwest` for OpenAI API communication
- **Configuration**: `serde` + `toml` for user settings
- **Database**: `rusqlite` for local history and state persistence
- **Logging**: `tracing` with structured logging support

## 🚀 Installation

### Prerequisites
- Rust 1.70+ (install via [rustup](https://rustup.rs/))
- Git

### From Source
```bash
# Clone the repository
git clone https://github.com/VinayakWankhade/agentic-cli.git
cd agentic-cli

# Build and install
cargo build --release
cargo install --path .
```

### Quick Start
```bash
# Initialize configuration
agentic

# Start interactive TUI mode
agentic tui

# Or use CLI mode directly
agentic task add --title "Build awesome CLI" --priority high
agentic prep start --exam CET --duration 60
agentic agent "help me organize my study schedule"
```

## 🎮 Usage

### Interactive TUI Mode
Launch the rich terminal interface:
```bash
agentic tui
# or simply
agentic
```

**Key Bindings:**
- `Ctrl+Q` - Quit application
- `Ctrl+A` - Toggle agent mode (🤖 ↔ $)
- `Enter` - Start/execute command
- `Esc` - Exit input mode
- `?` - Show help overlay
- `Ctrl+,` - Open settings
- `↑/↓` - Navigate command history
- `Tab` - Autocomplete

### Command Line Interface

#### Task Management
```bash
# Add tasks with priorities
agentic task add --title "Study calculus" --priority high --description "Chapter 1-3"

# List tasks with filters
agentic task list --status todo --priority high

# Mark tasks complete
agentic task complete task_123

# Update task priorities
agentic task priority task_123 high

# View task details
agentic task show task_123
```

#### Exam Preparation
```bash
# Start study sessions
agentic prep start --exam CET --duration 60 --schedule daily

# Review topics
agentic prep review --exam CET --count 5

# View preparation statistics
agentic prep stats --exam CET --period week

# Add study materials
agentic prep add --topic "Quadratic Equations" --exam CET --priority 4
```

#### Blog Management
```bash
# Create new blog posts
agentic blog new --title "Rust Memory Management" --tags rust,programming

# Edit existing posts
agentic blog edit --post-id blog_001

# Publish posts
agentic blog publish --post-id blog_001

# List posts with filters
agentic blog list --tag rust --drafts
```

#### AI Agent Integration
```bash
# Natural language queries
agentic agent "create a study plan for next week"
agentic agent "add a high priority task for building dashboard"
agentic agent "show me my progress on CET preparation"

# Command interpretation
agentic agent "I need to write a blog post about async Rust"
```

#### Raw Command Execution
```bash
# Execute any terminal command
agentic run "ls -la"
agentic run "git status"
agentic run "cargo test"
```

## ⚙️ Configuration

### Config File Location
- **Linux/macOS**: `~/.agentic/config.toml`
- **Windows**: `%USERPROFILE%\.agentic\config.toml`

### Example Configuration
```toml
[agent]
model = "gpt-3.5-turbo"
temperature = 0.7
max_tokens = 1000
timeout_seconds = 30

[theme]
dark_mode = true
primary_color = "#61dafb"
secondary_color = "#282c34"
accent_color = "#98c379"
background_color = "#1e1e1e"
text_color = "#ffffff"

[aliases]
t = "task"
p = "prep"
b = "blog"
a = "agent"
```

### Environment Variables
```bash
# OpenAI API key for agent functionality
export OPENAI_API_KEY="your-api-key-here"

# Optional: Override config file location
export AGENTIC_CONFIG_PATH="/path/to/config.toml"

# Enable debug logging
export RUST_LOG=debug
```

## 🗄️ Data Storage

The CLI maintains local state in:
- **Database**: `~/.agentic/history.db` (SQLite)
- **Logs**: `~/.agentic/logs/` (if configured)

### Schema Overview
```sql
-- Command execution history
CREATE TABLE command_executions (
    id TEXT PRIMARY KEY,
    command TEXT NOT NULL,
    output TEXT NOT NULL,
    status TEXT NOT NULL,
    timestamp TEXT NOT NULL,
    duration_ms INTEGER NOT NULL,
    agent_query TEXT
);

-- Task management
CREATE TABLE tasks (
    id TEXT PRIMARY KEY,
    title TEXT NOT NULL,
    description TEXT,
    priority TEXT NOT NULL,
    status TEXT NOT NULL,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

-- Preparation sessions
CREATE TABLE prep_sessions (
    id TEXT PRIMARY KEY,
    exam_type TEXT NOT NULL,
    session_name TEXT NOT NULL,
    status TEXT NOT NULL,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);
```

## 🎯 User Scenarios

### Student Workflow
```bash
# Morning routine
agentic prep start --exam JEE --duration 120
agentic task add --title "Practice integration" --priority high
agentic agent "what should I focus on today for JEE maths?"

# During study
agentic prep add --topic "Limits and Derivatives" --priority 5
agentic task complete math_practice_001

# Evening review
agentic prep stats --period today
agentic agent "summarize my study progress today"
```

### Developer Workflow
```bash
# Project planning
agentic task add --title "Implement user authentication" --priority high
agentic blog new --title "Building Secure Web APIs" --tags security,api

# Development session
agentic run "cargo test"
agentic agent "help me debug this Rust compilation error"
agentic task update task_123 --status in-progress

# Content creation
agentic blog edit --post-id blog_002
agentic agent "suggest improvements for my blog post about Rust"
```

## 🔧 Development

### Project Structure
```
src/
├── main.rs              # Application entry point
├── agent/               # AI agent system
│   ├── mod.rs          # Core agent logic
│   └── planner.rs      # Execution planning
├── commands/            # Command implementations
│   ├── mod.rs          # Command registry
│   ├── task.rs         # Task management
│   ├── prep.rs         # Exam preparation
│   └── blog.rs         # Blog management
├── config/              # Configuration management
│   └── mod.rs          # Config loading/saving
├── db/                  # Database layer
│   └── mod.rs          # SQLite operations
└── ui/                  # Terminal user interface
    ├── mod.rs          # TUI setup
    ├── app.rs          # Main application logic
    ├── components.rs   # UI components
    ├── layout.rs       # Layout management
    ├── events.rs       # Event handling
    └── styles.rs       # Theming and styles
```

### Building from Source
```bash
# Development build
cargo build

# Release build with optimizations
cargo build --release

# Run tests
cargo test

# Check code quality
cargo clippy
cargo fmt

# Run with debug logging
RUST_LOG=debug cargo run
```

### Contributing
1. Fork the repository
2. Create a feature branch: `git checkout -b feature/amazing-feature`
3. Make your changes and add tests
4. Ensure all tests pass: `cargo test`
5. Format code: `cargo fmt`
6. Check for issues: `cargo clippy`
7. Commit changes: `git commit -m 'Add amazing feature'`
8. Push to branch: `git push origin feature/amazing-feature`
9. Open a Pull Request

## 🛣️ Roadmap

### Phase 1: Core Features ✅
- [x] Basic TUI with ratatui
- [x] Command execution blocks
- [x] Agent integration with OpenAI
- [x] Task management system
- [x] Configuration management
- [x] Local database storage

### Phase 2: Enhanced UX 🚧
- [ ] Plugin system with WASM
- [ ] Advanced theming engine
- [ ] Real-time collaboration
- [ ] Cloud sync capabilities
- [ ] Mobile companion app

### Phase 3: Advanced Features 🔮
- [ ] Local LLM support (Ollama, GPT4All)
- [ ] Voice commands and TTS
- [ ] Integration with external tools (Notion, Todoist)
- [ ] AI-powered code generation
- [ ] Learning analytics and insights

## 📄 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## 🙏 Acknowledgments

- [ratatui](https://github.com/ratatui-org/ratatui) - Amazing TUI library
- [Warp](https://www.warp.dev/) - Inspiration for the interface design
- [Fig](https://fig.io/) - Command completion and suggestion ideas
- [OpenAI](https://openai.com/) - AI capabilities

## 📞 Support

- **Issues**: [GitHub Issues](https://github.com/VinayakWankhade/agentic-cli/issues)
- **Discussions**: [GitHub Discussions](https://github.com/VinayakWankhade/agentic-cli/discussions)
- **Email**: vinayakwankhade@example.com

---

Built with ❤️ in Rust by [Vinayak Wankhade](https://github.com/VinayakWankhade)
