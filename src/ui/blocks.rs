use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, BorderType, Gauge, List, ListItem, Paragraph, Wrap},
    Frame,
};
use std::time::Instant;
use unicode_width::UnicodeWidthStr;

use crate::db::{CommandExecution, ExecutionStatus};

/// Warp-style command block that mimics the exact visual design
#[derive(Debug, Clone)]
pub struct CommandBlock {
    pub execution: CommandExecution,
    pub is_selected: bool,
    pub animation_progress: f64,
    pub created_at: Instant,
}

impl CommandBlock {
    pub fn new(execution: CommandExecution) -> Self {
        Self {
            execution,
            is_selected: false,
            animation_progress: 0.0,
            created_at: Instant::now(),
        }
    }

    /// Render the command block in Warp's signature style
    pub fn render(&self, frame: &mut Frame, area: Rect) {
        // Create the main block with Warp-style borders
        let block_style = if self.is_selected {
            Style::default()
                .bg(Color::Rgb(45, 45, 45))  // Warp's selection color
                .fg(Color::White)
        } else {
            Style::default()
                .bg(Color::Rgb(30, 30, 30))  // Warp's background
                .fg(Color::White)
        };

        let main_block = Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)  // Warp's rounded corners
            .border_style(self.get_border_style())
            .style(block_style);

        let inner_area = main_block.inner(area);
        frame.render_widget(main_block, area);

        // Split into sections like Warp
        let sections = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(1),  // Command line
                Constraint::Length(1),  // Metadata line
                Constraint::Min(1),     // Output area
            ])
            .split(inner_area);

        // Render command line with prompt
        self.render_command_line(frame, sections[0]);
        
        // Render metadata (timestamp, duration, status)
        self.render_metadata_line(frame, sections[1]);
        
        // Render output area
        if !self.execution.output.is_empty() {
            self.render_output_area(frame, sections[2]);
        }

        // Render status indicator and progress
        self.render_status_indicator(frame, area);
    }

    fn get_border_style(&self) -> Style {
        match self.execution.status {
            ExecutionStatus::Running => Style::default().fg(Color::Yellow),
            ExecutionStatus::Success => Style::default().fg(Color::Green),
            ExecutionStatus::Error => Style::default().fg(Color::Red),
            ExecutionStatus::Cancelled => Style::default().fg(Color::Gray),
        }
    }

    fn render_command_line(&self, frame: &mut Frame, area: Rect) {
        let prompt_style = Style::default()
            .fg(Color::Rgb(98, 209, 248))  // Warp's blue
            .add_modifier(Modifier::BOLD);

        let command_style = Style::default()
            .fg(Color::White)
            .add_modifier(Modifier::BOLD);

        let prompt = if self.execution.agent_query.is_some() {
            "🤖 "
        } else {
            "❯ "
        };

        let line = Line::from(vec![
            Span::styled(prompt, prompt_style),
            Span::styled(&self.execution.command, command_style),
        ]);

        let paragraph = Paragraph::new(line);
        frame.render_widget(paragraph, area);
    }

    fn render_metadata_line(&self, frame: &mut Frame, area: Rect) {
        let metadata_style = Style::default()
            .fg(Color::Rgb(128, 128, 128))  // Warp's gray
            .add_modifier(Modifier::DIM);

        let timestamp = self.execution.timestamp.format("%H:%M:%S").to_string();
        let duration = if self.execution.duration_ms > 0 {
            format!("{}ms", self.execution.duration_ms)
        } else {
            "...".to_string()
        };

        let exit_code = match self.execution.status {
            ExecutionStatus::Success => "0",
            ExecutionStatus::Error => "1",
            ExecutionStatus::Running => "...",
            ExecutionStatus::Cancelled => "130",
        };

        let metadata_text = format!("{} • {} • exit {}", timestamp, duration, exit_code);

        let line = Line::from(vec![
            Span::styled("  ", metadata_style),
            Span::styled(metadata_text, metadata_style),
        ]);

        let paragraph = Paragraph::new(line);
        frame.render_widget(paragraph, area);
    }

    fn render_output_area(&self, frame: &mut Frame, area: Rect) {
        let output_style = Style::default().fg(Color::White);

        // Split output into lines and handle long lines
        let lines: Vec<Line> = self.execution.output
            .lines()
            .take(area.height as usize)  // Limit to visible area
            .map(|line| {
                if line.width() > area.width as usize - 4 {
                    // Truncate long lines
                    let truncated = format!("{}...", &line[..area.width as usize - 7]);
                    Line::from(Span::styled(format!("  {}", truncated), output_style))
                } else {
                    Line::from(Span::styled(format!("  {}", line), output_style))
                }
            })
            .collect();

        let text = Text::from(lines);
        let paragraph = Paragraph::new(text)
            .wrap(Wrap { trim: true });

        frame.render_widget(paragraph, area);
    }

    fn render_status_indicator(&self, frame: &mut Frame, area: Rect) {
        // Render status icon in top-right corner like Warp
        let status_area = Rect {
            x: area.x + area.width - 3,
            y: area.y,
            width: 3,
            height: 1,
        };

        let (icon, color) = match self.execution.status {
            ExecutionStatus::Running => {
                // Animated spinner
                let spinner_chars = ["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];
                let elapsed = self.created_at.elapsed().as_millis();
                let index = (elapsed / 100) % spinner_chars.len() as u128;
                (spinner_chars[index as usize], Color::Yellow)
            },
            ExecutionStatus::Success => ("✓", Color::Green),
            ExecutionStatus::Error => ("✗", Color::Red),
            ExecutionStatus::Cancelled => ("⊘", Color::Gray),
        };

        let status_line = Line::from(vec![
            Span::styled(icon, Style::default().fg(color).add_modifier(Modifier::BOLD)),
        ]);

        let paragraph = Paragraph::new(status_line);
        frame.render_widget(paragraph, status_area);

        // Render progress bar for running commands
        if matches!(self.execution.status, ExecutionStatus::Running) {
            let progress_area = Rect {
                x: area.x + 1,
                y: area.y + area.height - 1,
                width: area.width - 2,
                height: 1,
            };

            // Animated progress bar
            let elapsed = self.created_at.elapsed().as_millis() as f64;
            let progress = ((elapsed / 50.0) % 100.0) / 100.0;

            let gauge = Gauge::default()
                .block(Block::default())
                .gauge_style(Style::default().fg(Color::Yellow).bg(Color::Rgb(40, 40, 40)))
                .ratio(progress);

            frame.render_widget(gauge, progress_area);
        }
    }
}

/// Command palette for Warp-style command suggestions
#[derive(Debug)]
pub struct CommandPalette {
    pub suggestions: Vec<String>,
    pub selected_index: usize,
    pub filter: String,
    pub is_visible: bool,
}

impl CommandPalette {
    pub fn new() -> Self {
        Self {
            suggestions: vec![
                "task add --title 'New task' --priority high".to_string(),
                "prep start --exam CET --duration 60".to_string(),
                "blog new --title 'My Blog Post'".to_string(),
                "agent 'help me with...'".to_string(),
                "git status".to_string(),
                "git add .".to_string(),
                "git commit -m 'message'".to_string(),
                "cargo build".to_string(),
                "cargo test".to_string(),
                "ls -la".to_string(),
                "cd ..".to_string(),
                "pwd".to_string(),
            ],
            selected_index: 0,
            filter: String::new(),
            is_visible: false,
        }
    }

    pub fn toggle(&mut self) {
        self.is_visible = !self.is_visible;
        if !self.is_visible {
            self.filter.clear();
            self.selected_index = 0;
        }
    }

    pub fn update_filter(&mut self, filter: String) {
        self.filter = filter;
        self.selected_index = 0;
    }

    pub fn move_selection(&mut self, direction: i32) {
        let filtered_count = self.get_filtered_suggestions().len();
        if filtered_count == 0 {
            return;
        }

        if direction > 0 {
            self.selected_index = (self.selected_index + 1) % filtered_count;
        } else if direction < 0 {
            self.selected_index = if self.selected_index == 0 {
                filtered_count - 1
            } else {
                self.selected_index - 1
            };
        }
    }

    pub fn get_selected_suggestion(&self) -> Option<String> {
        let filtered = self.get_filtered_suggestions();
        filtered.get(self.selected_index).cloned()
    }

    fn get_filtered_suggestions(&self) -> Vec<String> {
        if self.filter.is_empty() {
            self.suggestions.clone()
        } else {
            self.suggestions
                .iter()
                .filter(|s| s.to_lowercase().contains(&self.filter.to_lowercase()))
                .cloned()
                .collect()
        }
    }

    pub fn render(&self, frame: &mut Frame, area: Rect) {
        if !self.is_visible {
            return;
        }

        // Create centered popup like Warp's command palette
        let popup_area = centered_rect(80, 60, area);

        // Clear background
        frame.render_widget(
            Block::default().style(Style::default().bg(Color::Black)),
            popup_area,
        );

        let main_block = Block::default()
            .title("Command Palette")
            .title_style(Style::default().fg(Color::Rgb(98, 209, 248)).add_modifier(Modifier::BOLD))
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(Color::Rgb(98, 209, 248)))
            .style(Style::default().bg(Color::Rgb(20, 20, 20)));

        let inner_area = main_block.inner(popup_area);
        frame.render_widget(main_block, popup_area);

        // Split into filter input and suggestions
        let sections = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),  // Filter input
                Constraint::Min(1),     // Suggestions list
            ])
            .split(inner_area);

        // Render filter input
        self.render_filter_input(frame, sections[0]);

        // Render suggestions
        self.render_suggestions(frame, sections[1]);
    }

    fn render_filter_input(&self, frame: &mut Frame, area: Rect) {
        let input_block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Yellow))
            .title("Filter")
            .style(Style::default().bg(Color::Rgb(30, 30, 30)));

        let inner_area = input_block.inner(area);
        frame.render_widget(input_block, area);

        let input_text = format!("❯ {}", self.filter);
        let line = Line::from(vec![
            Span::styled(input_text, Style::default().fg(Color::White)),
        ]);

        let paragraph = Paragraph::new(line);
        frame.render_widget(paragraph, inner_area);
    }

    fn render_suggestions(&self, frame: &mut Frame, area: Rect) {
        let filtered_suggestions = self.get_filtered_suggestions();
        
        let items: Vec<ListItem> = filtered_suggestions
            .iter()
            .enumerate()
            .map(|(index, suggestion)| {
                let style = if index == self.selected_index {
                    Style::default()
                        .bg(Color::Rgb(98, 209, 248))
                        .fg(Color::Black)
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(Color::White)
                };

                let line = Line::from(vec![
                    Span::styled(suggestion.clone(), style),
                ]);

                ListItem::new(line)
            })
            .collect();

        let suggestions_list = List::new(items)
            .style(Style::default().bg(Color::Rgb(20, 20, 20)));

        frame.render_widget(suggestions_list, area);
    }
}

// Helper function for centered rectangles
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

impl Default for CommandPalette {
    fn default() -> Self {
        Self::new()
    }
}
