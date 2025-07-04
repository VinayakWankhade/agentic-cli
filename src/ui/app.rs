use anyhow::Result;
use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use ratatui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph, Wrap},
    Frame, Terminal,
};
use std::time::{Duration, Instant};
use tracing::{debug, info};
use clap::Parser;

use crate::{
    agent::Agent,
    commands::CommandRegistry,
    config::Config,
    db::{CommandExecution, Database, ExecutionStatus},
};

use super::{
    components::{InputBar, StatusBar, Sidebar},
    events::EventHandler,
    layout::AppLayout,
    styles::AppTheme,
    performance::VirtualScroller,
};

#[derive(Debug, Clone, PartialEq)]
pub enum AppMode {
    Normal,
    Agent,
    Help,
    Settings,
}

#[derive(Debug, Clone, PartialEq)]
pub enum InputMode {
    Normal,
    Editing,
}

pub struct App {
    pub config: Config,
    pub db: Database,
    pub agent: Agent,
    pub command_registry: CommandRegistry,
    
    // UI State
    pub mode: AppMode,
    pub input_mode: InputMode,
    pub input: String,
    pub command_history: Vec<CommandExecution>,
    #[allow(dead_code)]
    pub selected_block: usize,
    pub should_quit: bool,
    
    // Theme and Layout
    #[allow(dead_code)]
    pub theme: AppTheme,
    #[allow(dead_code)]
    pub layout: AppLayout,
    
    // Components
    pub input_bar: InputBar,
    pub status_bar: StatusBar,
    pub sidebar: Sidebar,
    
    // Event handling
    #[allow(dead_code)]
    pub event_handler: EventHandler,
    #[allow(dead_code)]
    pub last_render: Instant,
    
    // Scrolling
    pub scroller: VirtualScroller,
}

impl App {
    pub fn new(
        config: Config,
        db: Database,
        agent: Agent,
        command_registry: CommandRegistry,
    ) -> Self {
        let theme = AppTheme::from_config(&config);
        let layout = AppLayout::new();
        
        Self {
            config: config.clone(),
            db,
            agent,
            command_registry,
            
            mode: AppMode::Normal,
            input_mode: InputMode::Normal,
            input: String::new(),
            command_history: Vec::new(),
            selected_block: 0,
            should_quit: false,
            
            theme,
            layout,
            
            input_bar: InputBar::new(),
            status_bar: StatusBar::new(),
            sidebar: Sidebar::new(),
            
            event_handler: EventHandler::new(Duration::from_millis(16)), // 60 FPS
            last_render: Instant::now(),
            
            // Initialize scroller with default values
            scroller: VirtualScroller::new(10, 4), // 4 lines per command execution
        }
    }
    
    pub async fn run<B: Backend>(&mut self, terminal: &mut Terminal<B>) -> Result<()> {
        info!("Starting TUI application");
        
        // Load command history
        self.load_command_history().await?;
        
        loop {
            // Render the UI
            terminal.draw(|f| self.render(f))?;
            
            // Handle events
            if let Ok(event) = event::poll(Duration::from_millis(16)) {
                if event {
                    if let Ok(event) = event::read() {
                        self.handle_event(event).await?;
                    }
                }
            }
            
            // Check if we should quit
            if self.should_quit {
                break;
            }
            
            // Update components
            self.update().await?;
        }
        
        info!("TUI application exited");
        Ok(())
    }
    
    fn render(&mut self, frame: &mut Frame) {
        let size = frame.size();
        
        // Create main layout
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(1),     // Status bar
                Constraint::Min(0),        // Main content
                Constraint::Length(3),     // Input bar
            ])
            .split(size);
        
        // Render status bar
        self.render_status_bar(frame, chunks[0]);
        
        // Create horizontal layout for main content
        let main_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Min(60),       // Main content
                Constraint::Length(30),    // Sidebar
            ])
            .split(chunks[1]);
        
        // Render main content area
        self.render_main_content(frame, main_chunks[0]);
        
        // Render sidebar
        self.render_sidebar(frame, main_chunks[1]);
        
        // Render input bar
        self.render_input_bar(frame, chunks[2]);
        
        // Render overlays based on mode
        match self.mode {
            AppMode::Help => self.render_help_overlay(frame, size),
            AppMode::Settings => self.render_settings_overlay(frame, size),
            _ => {}
        }
    }
    
    fn render_status_bar(&self, frame: &mut Frame, area: Rect) {
        let mode_text = match self.mode {
            AppMode::Normal => "NORMAL",
            AppMode::Agent => "AGENT",
            AppMode::Help => "HELP",
            AppMode::Settings => "SETTINGS",
        };
        
        let mode_color = match self.mode {
            AppMode::Normal => Color::Blue,
            AppMode::Agent => Color::Green,
            AppMode::Help => Color::Yellow,
            AppMode::Settings => Color::Magenta,
        };
        
        let status_line = Line::from(vec![
            Span::styled(
                format!(" {} ", mode_text),
                Style::default()
                    .fg(Color::White)
                    .bg(mode_color)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(" | "),
            Span::styled("Ctrl+Q", Style::default().fg(Color::Gray)),
            Span::raw(" quit | "),
            Span::styled("Ctrl+A", Style::default().fg(Color::Gray)),
            Span::raw(" agent | "),
            Span::styled("?", Style::default().fg(Color::Gray)),
            Span::raw(" help"),
        ]);
        
        let status_paragraph = Paragraph::new(status_line)
            .style(Style::default().bg(Color::Black));
        
        frame.render_widget(status_paragraph, area);
    }
    
    fn render_main_content(&mut self, frame: &mut Frame, area: Rect) {
        // Create command execution blocks
        let mut items = Vec::new();
        
        // Update scroller with current viewport height and total items
        self.scroller.viewport_height = area.height as usize;
        self.scroller.update_total_items(self.command_history.len());
        
        // Get visible range based on scroll position
        let (start_idx, end_idx) = self.scroller.get_visible_range();
        
        // Only render visible items
        for (_index, execution) in self.command_history.iter().enumerate().skip(start_idx).take(end_idx - start_idx) {
            let status_icon = match execution.status {
                ExecutionStatus::Running => "⏳",
                ExecutionStatus::Success => "✅",
                ExecutionStatus::Error => "❌",
                ExecutionStatus::Cancelled => "🚫",
            };
            
            let status_color = match execution.status {
                ExecutionStatus::Running => Color::Yellow,
                ExecutionStatus::Success => Color::Green,
                ExecutionStatus::Error => Color::Red,
                ExecutionStatus::Cancelled => Color::Gray,
            };
            
            let item = ListItem::new(vec![
                Line::from(vec![
                    Span::styled(
                        format!("{} ", status_icon),
                        Style::default().fg(status_color),
                    ),
                    Span::styled(
                        execution.command.clone(),
                        Style::default()
                            .fg(Color::White)
                            .add_modifier(Modifier::BOLD),
                    ),
                ]),
                Line::from(vec![
                    Span::raw("  "),
                    Span::styled(
                        execution.timestamp.format("%H:%M:%S").to_string(),
                        Style::default().fg(Color::Gray),
                    ),
                    Span::raw(" | "),
                    Span::styled(
                        format!("{}{}", execution.duration_ms, "ms"),
                        Style::default().fg(Color::Gray),
                    ),
                ]),
                Line::from(vec![
                    Span::raw("  "),
                    Span::styled(
                        if execution.output.len() > 100 {
                            format!("{}{}", &execution.output[..100], "...")
                        } else {
                            execution.output.clone()
                        },
                        Style::default().fg(Color::Cyan),
                    ),
                ]),
                Line::from(vec![Span::raw("")]), // Empty line separator
            ]);
            
            items.push(item);
        }
        
        // Add scroll indicator if needed
        let title = if self.command_history.len() > (end_idx - start_idx) {
            format!("Command History (Scroll: {}/{})", start_idx + 1, self.command_history.len())
        } else {
            "Command History".to_string()
        };
        
        let block = Block::default()
            .title(title)
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Blue));
        
        let list = List::new(items)
            .block(block)
            .highlight_style(
                Style::default()
                    .bg(Color::DarkGray)
                    .add_modifier(Modifier::BOLD),
            );
        
        frame.render_stateful_widget(list, area, &mut self.sidebar.list_state);
    }
    
    fn render_sidebar(&self, frame: &mut Frame, area: Rect) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(8),     // Agent info
                Constraint::Min(0),        // Suggestions
            ])
            .split(area);
        
        // Agent info panel
        let agent_info = vec![
            Line::from(vec![
                Span::styled("🤖 Agent", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
            ]),
            Line::from(vec![
                Span::raw("Model: "),
                Span::styled(self.config.agent.model.clone(), Style::default().fg(Color::Yellow)),
            ]),
            Line::from(vec![
                Span::raw("Status: "),
                Span::styled("Ready", Style::default().fg(Color::Green)),
            ]),
            Line::from(vec![
                Span::raw("API: "),
                Span::styled(
                    if self.config.get_openai_api_key().is_some() {
                        "Connected"
                    } else {
                        "No API Key"
                    },
                    Style::default().fg(if self.config.get_openai_api_key().is_some() {
                        Color::Green
                    } else {
                        Color::Red
                    }),
                ),
            ]),
        ];
        
        let agent_block = Block::default()
            .title("Agent Status")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Green));
        
        let agent_paragraph = Paragraph::new(agent_info)
            .block(agent_block)
            .wrap(Wrap { trim: true });
        
        frame.render_widget(agent_paragraph, chunks[0]);
        
        // Suggestions panel
        let suggestions = vec![
            ListItem::new("task add --title 'New task'"),
            ListItem::new("prep start --exam CET"),
            ListItem::new("blog new --title 'My Post'"),
            ListItem::new("agent 'help me study'"),
        ];
        
        let suggestions_block = Block::default()
            .title("Quick Commands")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Yellow));
        
        let suggestions_list = List::new(suggestions)
            .block(suggestions_block)
            .style(Style::default().fg(Color::White));
        
        frame.render_widget(suggestions_list, chunks[1]);
    }
    
    fn render_input_bar(&self, frame: &mut Frame, area: Rect) {
        let input_style = match self.input_mode {
            InputMode::Normal => Style::default(),
            InputMode::Editing => Style::default().fg(Color::Yellow),
        };
        
        let mode_indicator = match self.mode {
            AppMode::Agent => "🤖 ",
            _ => "$ ",
        };
        
        let input_text = format!("{}{}", mode_indicator, self.input);
        
        let input = Paragraph::new(input_text)
            .style(input_style)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(match self.input_mode {
                        InputMode::Normal => Style::default(),
                        InputMode::Editing => Style::default().fg(Color::Yellow),
                    })
                    .title(match self.mode {
                        AppMode::Agent => "Agent Query",
                        _ => "Command",
                    }),
            );
        
        frame.render_widget(input, area);
        
        if self.input_mode == InputMode::Editing {
            // Calculate cursor position
            let cursor_x = area.x + self.input.len() as u16 + 3; // +3 for prompt and border
            let cursor_y = area.y + 1; // +1 for border
            
            frame.set_cursor(cursor_x, cursor_y);
        }
    }
    
    fn render_help_overlay(&self, frame: &mut Frame, area: Rect) {
        let popup_area = centered_rect(60, 70, area);
        
        let help_text = vec![
            Line::from(vec![
                Span::styled("Agentic CLI Help", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
            ]),
            Line::from(vec![Span::raw("")]),
            Line::from(vec![
                Span::styled("Key Bindings:", Style::default().add_modifier(Modifier::BOLD)),
            ]),
            Line::from(vec![
                Span::styled("  Ctrl+Q", Style::default().fg(Color::Green)),
                Span::raw("  - Quit application"),
            ]),
            Line::from(vec![
                Span::styled("  Ctrl+A", Style::default().fg(Color::Green)),
                Span::raw("  - Toggle agent mode"),
            ]),
            Line::from(vec![
                Span::styled("  Enter", Style::default().fg(Color::Green)),
                Span::raw("   - Execute command"),
            ]),
            Line::from(vec![
                Span::styled("  Esc", Style::default().fg(Color::Green)),
                Span::raw("     - Exit input mode"),
            ]),
            Line::from(vec![
                Span::styled("  ?", Style::default().fg(Color::Green)),
                Span::raw("       - Toggle this help"),
            ]),
            // Add scrolling key bindings
            Line::from(vec![Span::raw("")]),
            Line::from(vec![
                Span::styled("Scrolling:", Style::default().add_modifier(Modifier::BOLD)),
            ]),
            Line::from(vec![
                Span::styled("  ↑/↓", Style::default().fg(Color::Green)),
                Span::raw("     - Scroll up/down"),
            ]),
            Line::from(vec![
                Span::styled("  PgUp/PgDn", Style::default().fg(Color::Green)),
                Span::raw(" - Scroll page up/down"),
            ]),
            Line::from(vec![
                Span::styled("  Home/End", Style::default().fg(Color::Green)),
                Span::raw("  - Scroll to top/bottom"),
            ]),
            Line::from(vec![Span::raw("")]),
            Line::from(vec![
                Span::styled("Commands:", Style::default().add_modifier(Modifier::BOLD)),
            ]),
            Line::from(vec![
                Span::styled("  task", Style::default().fg(Color::Cyan)),
                Span::raw("     - Task management"),
            ]),
            Line::from(vec![
                Span::styled("  prep", Style::default().fg(Color::Cyan)),
                Span::raw("     - Exam preparation"),
            ]),
            Line::from(vec![
                Span::styled("  blog", Style::default().fg(Color::Cyan)),
                Span::raw("     - Blog management"),
            ]),
            Line::from(vec![
                Span::styled("  agent", Style::default().fg(Color::Cyan)),
                Span::raw("    - AI assistance"),
            ]),
        ];
        
        let help_paragraph = Paragraph::new(help_text)
            .block(
                Block::default()
                    .title("Help")
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::Yellow)),
            )
            .wrap(Wrap { trim: true });
        
        frame.render_widget(Clear, popup_area);
        frame.render_widget(help_paragraph, popup_area);
    }
    
    fn render_settings_overlay(&self, frame: &mut Frame, area: Rect) {
        let popup_area = centered_rect(50, 60, area);
        
        let settings_text = vec![
            Line::from(vec![
                Span::styled("Settings", Style::default().fg(Color::Magenta).add_modifier(Modifier::BOLD)),
            ]),
            Line::from(vec![Span::raw("")]),
            Line::from(vec![
                Span::raw("Theme: "),
                Span::styled(
                    if self.config.theme.dark_mode { "Dark" } else { "Light" },
                    Style::default().fg(Color::Yellow),
                ),
            ]),
            Line::from(vec![
                Span::raw("Agent Model: "),
                Span::styled(self.config.agent.model.clone(), Style::default().fg(Color::Yellow)),
            ]),
            Line::from(vec![
                Span::raw("API Key: "),
                Span::styled(
                    if self.config.get_openai_api_key().is_some() { "Set" } else { "Not Set" },
                    Style::default().fg(if self.config.get_openai_api_key().is_some() {
                        Color::Green
                    } else {
                        Color::Red
                    }),
                ),
            ]),
        ];
        
        let settings_paragraph = Paragraph::new(settings_text)
            .block(
                Block::default()
                    .title("Settings")
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::Magenta)),
            )
            .wrap(Wrap { trim: true });
        
        frame.render_widget(Clear, popup_area);
        frame.render_widget(settings_paragraph, popup_area);
    }
    
    async fn handle_event(&mut self, event: Event) -> Result<()> {
        match event {
            Event::Key(key) if key.kind == KeyEventKind::Press => {
                match self.input_mode {
                    InputMode::Normal => self.handle_normal_key(key).await?,
                    InputMode::Editing => self.handle_editing_key(key).await?,
                }
            }
            _ => {}
        }
        Ok(())
    }
    
    async fn handle_normal_key(&mut self, key: crossterm::event::KeyEvent) -> Result<()> {
        match key.code {
            KeyCode::Char('q') if key.modifiers.contains(crossterm::event::KeyModifiers::CONTROL) => {
                self.should_quit = true;
            }
            KeyCode::Char('a') if key.modifiers.contains(crossterm::event::KeyModifiers::CONTROL) => {
                self.mode = if self.mode == AppMode::Agent {
                    AppMode::Normal
                } else {
                    AppMode::Agent
                };
            }
            KeyCode::Char('?') => {
                self.mode = if self.mode == AppMode::Help {
                    AppMode::Normal
                } else {
                    AppMode::Help
                };
            }
            KeyCode::Char(',') if key.modifiers.contains(crossterm::event::KeyModifiers::CONTROL) => {
                self.mode = if self.mode == AppMode::Settings {
                    AppMode::Normal
                } else {
                    AppMode::Settings
                };
            }
            // Add scrolling with arrow keys
            KeyCode::Up => {
                self.scroller.scroll_up(1);
            }
            KeyCode::Down => {
                self.scroller.scroll_down(1);
            }
            KeyCode::PageUp => {
                self.scroller.scroll_up(5);
            }
            KeyCode::PageDown => {
                self.scroller.scroll_down(5);
            }
            KeyCode::Home => {
                self.scroller.scroll_offset = 0;
            }
            KeyCode::End => {
                self.scroller.scroll_offset = self.scroller.max_scroll_offset();
            }
            KeyCode::Enter => {
                self.input_mode = InputMode::Editing;
            }
            _ => {}
        }
        Ok(())
    }
    
    async fn handle_editing_key(&mut self, key: crossterm::event::KeyEvent) -> Result<()> {
        match key.code {
            KeyCode::Enter => {
                if !self.input.trim().is_empty() {
                    self.execute_command().await?;
                }
                self.input.clear();
                self.input_mode = InputMode::Normal;
            }
            KeyCode::Esc => {
                self.input_mode = InputMode::Normal;
            }
            KeyCode::Char(c) => {
                self.input.push(c);
            }
            KeyCode::Backspace => {
                self.input.pop();
            }
            _ => {}
        }
        return Ok(())
    }
    
    async fn execute_command(&mut self) -> Result<()> {
        let command = self.input.trim().to_string();
        info!("Executing command: {}", command);

        let execution = CommandExecution::new(
            command.clone(),
            if self.mode == AppMode::Agent {
                Some(command.clone())
            } else {
                None
            },
        );

        // Add to history immediately
        self.command_history.insert(0, execution.clone());
        self.db.save_command_execution(&execution).await?;

        // Use shell_words for proper splitting
        let args = match shell_words::split(&command) {
            Ok(a) => a,
            Err(e) => {
                self.update_execution_output(0, &format!("Error parsing command: {}", e), ExecutionStatus::Error, 0).await?;
                return Ok(());
            }
        };
        let mut cli_args = vec!["agentic-cli"]; // program name for clap
        cli_args.extend(args.iter().map(|s| s.as_str()));

        match crate::Cli::try_parse_from(cli_args) {
            Ok(cli) => {
                match cli.command {
                    Some(crate::Commands::Task { task_cmd }) => {
                        match self.command_registry.execute_task(task_cmd, &self.db).await {
                            Ok(_) => {
                                self.update_execution_output(0, "Task command executed successfully", ExecutionStatus::Success, 75).await?;
                            }
                            Err(e) => {
                                self.update_execution_output(0, &format!("Error: {}", e), ExecutionStatus::Error, 25).await?;
                            }
                        }
                        return Ok(());
                    }
                    Some(crate::Commands::Prep { prep_cmd }) => {
                        match self.command_registry.execute_prep(prep_cmd, &self.db).await {
                            Ok(_) => {
                                self.update_execution_output(0, "Prep command executed successfully", ExecutionStatus::Success, 75).await?;
                            }
                            Err(e) => {
                                self.update_execution_output(0, &format!("Error: {}", e), ExecutionStatus::Error, 25).await?;
                            }
                        }
                        return Ok(());
                    }
                    Some(crate::Commands::Blog { blog_cmd }) => {
                        match self.command_registry.execute_blog(blog_cmd, &self.db).await {
                            Ok(_) => {
                                self.update_execution_output(0, "Blog command executed successfully", ExecutionStatus::Success, 75).await?;
                            }
                            Err(e) => {
                                self.update_execution_output(0, &format!("Error: {}", e), ExecutionStatus::Error, 25).await?;
                            }
                        }
                        return Ok(());
                    }
                    Some(crate::Commands::Agent { query }) => {
                        match self.agent.process_query(&query).await {
                            Ok(response) => {
                                self.update_execution_output(0, &response, ExecutionStatus::Success, 100).await?;
                            }
                            Err(e) => {
                                self.update_execution_output(0, &format!("Error: {}", e), ExecutionStatus::Error, 50).await?;
                            }
                        }
                        return Ok(());
                    }
                    Some(crate::Commands::Warp { request, dry_run }) => {
                        let pipeline = crate::warp::WarpPipeline::new(&self.config)?;
                        if dry_run {
                            let (_plan, command) = pipeline.dry_run(&request).await?;
                            let output = format!("\n{} Would execute: {}", "📋", command);
                            self.update_execution_output(0, &output, ExecutionStatus::Success, 100).await?;
                        } else {
                            let result = pipeline.execute(&request).await?;
                            let output = if !result.is_success() && !result.cancelled {
                                "Pipeline execution failed".to_string()
                            } else {
                                "Pipeline executed successfully".to_string()
                            };
                            self.update_execution_output(0, &output, ExecutionStatus::Success, 100).await?;
                        }
                        return Ok(());
                    }
                    Some(crate::Commands::Run { command }) => {
                        match self.command_registry.execute_raw_command(&command).await {
                            Ok(_) => {
                                self.update_execution_output(0, "Command executed successfully", ExecutionStatus::Success, 75).await?;
                            }
                            Err(e) => {
                                self.update_execution_output(0, &format!("Error: {}", e), ExecutionStatus::Error, 25).await?;
                            }
                        }
                        return Ok(());
                    }
                    Some(crate::Commands::Tui) | None => {
                        // Already in TUI mode, do nothing
                        return Ok(());
                    }
                }

            }
            Err(e) => {
                // If not a recognized CLI command, try agent mode if enabled
                if self.mode == AppMode::Agent {
                    match self.agent.process_query(&command).await {
                        Ok(response) => {
                            self.update_execution_output(0, &response, ExecutionStatus::Success, 100).await?;
                        }
                        Err(e) => {
                            self.update_execution_output(0, &format!("Error: {}", e), ExecutionStatus::Error, 50).await?;
                        }
                    }
                    return Ok(());
                } else {
                    self.update_execution_output(0, &format!("Error: {}", e), ExecutionStatus::Error, 0).await?;
                    Ok(())
                }
            }
        }
        // Ok(()) <-- REMOVE THIS LINE
    }

    async fn update_execution_output(&mut self, index: usize, output: &str, status: ExecutionStatus, duration_ms: u128) -> Result<()> {
        if let Some(exec) = self.command_history.get_mut(index) {
            exec.output = output.to_string();
            exec.status = status.clone();
            exec.duration_ms = duration_ms as u64;
            self.db.update_execution_status(
                &exec.id,
                status,
                output,
                duration_ms as u64,
            ).await?;
        }
        Ok(())
    }
    
    async fn load_command_history(&mut self) -> Result<()> {
        // Load command history from database
        self.command_history = self.db.get_command_history(100).await?;
        
        // Update scroller with the total number of items
        self.scroller.update_total_items(self.command_history.len());
        
        // Scroll to the bottom to show most recent commands
        if !self.command_history.is_empty() {
            self.scroller.scroll_to_item(self.command_history.len() - 1);
        }
        
        Ok(())
    }
    
    async fn update(&mut self) -> Result<()> {
        // Update components
        self.input_bar.update();
        self.status_bar.update();
        self.sidebar.update();
        
        Ok(())
    }
}

// Helper function to create centered rectangle
fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}