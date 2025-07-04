use std::collections::VecDeque;
use std::time::{Duration, Instant};
use crate::db::CommandExecution;

/// Performance optimizations for terminal rendering
/// 
/// This module implements various optimizations to achieve Warp-level performance:
/// - Efficient text rendering with virtual scrolling
/// - Smart re-rendering with dirty regions
/// - Animation frame limiting
/// - Memory-efficient command history management

#[derive(Debug)]
#[allow(dead_code)]
pub struct PerformanceManager {
    #[allow(dead_code)]
    pub max_history_size: usize,
    #[allow(dead_code)]
    pub max_output_lines: usize,
    #[allow(dead_code)]
    pub animation_frame_rate: u64,
    #[allow(dead_code)]
    pub last_frame_time: Instant,
    #[allow(dead_code)]
    pub dirty_regions: Vec<DirtyRegion>,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct DirtyRegion {
    #[allow(dead_code)]
    pub x: u16,
    #[allow(dead_code)]
    pub y: u16,
    #[allow(dead_code)]
    pub width: u16,
    #[allow(dead_code)]
    pub height: u16,
    #[allow(dead_code)]
    pub priority: u8,
}

impl PerformanceManager {
    pub fn new() -> Self {
        Self {
            max_history_size: 1000,
            max_output_lines: 10000,
            animation_frame_rate: 60, // 60 FPS
            last_frame_time: Instant::now(),
            dirty_regions: Vec::new(),
        }
    }

    /// Check if we should render a new frame based on target FPS
    pub fn should_render_frame(&mut self) -> bool {
        let frame_duration = Duration::from_millis(1000 / self.animation_frame_rate);
        let elapsed = self.last_frame_time.elapsed();
        
        if elapsed >= frame_duration {
            self.last_frame_time = Instant::now();
            true
        } else {
            false
        }
    }

    /// Mark a region as dirty for efficient re-rendering
    pub fn mark_dirty(&mut self, x: u16, y: u16, width: u16, height: u16, priority: u8) {
        let region = DirtyRegion {
            x,
            y,
            width,
            height,
            priority,
        };
        
        // Insert in priority order (higher priority first)
        let insert_pos = self.dirty_regions
            .iter()
            .position(|r| r.priority < priority)
            .unwrap_or(self.dirty_regions.len());
            
        self.dirty_regions.insert(insert_pos, region);
    }

    /// Clear all dirty regions after rendering
    pub fn clear_dirty_regions(&mut self) {
        self.dirty_regions.clear();
    }

    /// Optimize command history by removing old entries
    pub fn optimize_command_history(&self, history: &mut Vec<CommandExecution>) {
        if history.len() > self.max_history_size {
            history.truncate(self.max_history_size);
        }
    }

    /// Truncate large command outputs for performance
    pub fn optimize_command_output(&self, output: &str) -> String {
        let lines: Vec<&str> = output.lines().collect();
        
        if lines.len() > self.max_output_lines {
            let truncated_lines = &lines[..self.max_output_lines];
            let mut result = truncated_lines.join("\n");
            result.push_str(&format!("\n\n... {} more lines truncated for performance", 
                lines.len() - self.max_output_lines));
            result
        } else {
            output.to_string()
        }
    }
}

/// Virtual scrolling implementation for large lists
#[derive(Debug)]
pub struct VirtualScroller {
    pub viewport_height: usize,
    pub total_items: usize,
    pub scroll_offset: usize,
    pub item_height: usize,
}

impl VirtualScroller {
    pub fn new(viewport_height: usize, item_height: usize) -> Self {
        Self {
            viewport_height,
            total_items: 0,
            scroll_offset: 0,
            item_height,
        }
    }

    /// Update the total number of items
    pub fn update_total_items(&mut self, total: usize) {
        self.total_items = total;
        self.clamp_scroll_offset();
    }

    /// Get the range of visible items
    pub fn get_visible_range(&self) -> (usize, usize) {
        let visible_items = self.viewport_height / self.item_height;
        let start = self.scroll_offset;
        let end = (start + visible_items).min(self.total_items);
        (start, end)
    }

    /// Scroll up by the specified amount
    pub fn scroll_up(&mut self, amount: usize) {
        self.scroll_offset = self.scroll_offset.saturating_sub(amount);
    }

    /// Scroll down by the specified amount
    pub fn scroll_down(&mut self, amount: usize) {
        self.scroll_offset = (self.scroll_offset + amount).min(self.max_scroll_offset());
        self.clamp_scroll_offset();
    }

    /// Scroll to a specific item
    pub fn scroll_to_item(&mut self, item_index: usize) {
        let visible_items = self.viewport_height / self.item_height;
        
        if item_index < self.scroll_offset {
            self.scroll_offset = item_index;
        } else if item_index >= self.scroll_offset + visible_items {
            self.scroll_offset = item_index.saturating_sub(visible_items - 1);
        }
        
        self.clamp_scroll_offset();
    }

    /// Get the maximum scroll offset
    pub fn max_scroll_offset(&self) -> usize {
        let visible_items = self.viewport_height / self.item_height;
        self.total_items.saturating_sub(visible_items)
    }

    fn clamp_scroll_offset(&mut self) {
        self.scroll_offset = self.scroll_offset.min(self.max_scroll_offset());
    }
}

/// Animation system for smooth transitions
#[derive(Debug)]
pub struct AnimationSystem {
    pub animations: Vec<Animation>,
}

#[derive(Debug, Clone)]
pub struct Animation {
    pub id: String,
    pub start_time: Instant,
    pub duration: Duration,
    pub easing: EasingFunction,
    pub from_value: f64,
    pub to_value: f64,
    pub current_value: f64,
}

#[derive(Debug, Clone, Copy)]
pub enum EasingFunction {
    Linear,
    EaseIn,
    EaseOut,
    EaseInOut,
}

impl AnimationSystem {
    pub fn new() -> Self {
        Self {
            animations: Vec::new(),
        }
    }

    /// Start a new animation
    pub fn start_animation(
        &mut self,
        id: String,
        duration: Duration,
        from_value: f64,
        to_value: f64,
        easing: EasingFunction,
    ) {
        // Remove existing animation with the same ID
        self.animations.retain(|a| a.id != id);

        let animation = Animation {
            id,
            start_time: Instant::now(),
            duration,
            easing,
            from_value,
            to_value,
            current_value: from_value,
        };

        self.animations.push(animation);
    }

    /// Update all animations and return whether any are still running
    pub fn update(&mut self) -> bool {
        let now = Instant::now();
        let mut any_running = false;

        for animation in &mut self.animations {
            let elapsed = now.duration_since(animation.start_time);
            
            if elapsed >= animation.duration {
                animation.current_value = animation.to_value;
            } else {
                let progress = elapsed.as_secs_f64() / animation.duration.as_secs_f64();
                let eased_progress = apply_easing(progress, animation.easing);
                
                animation.current_value = animation.from_value + 
                    (animation.to_value - animation.from_value) * eased_progress;
                
                any_running = true;
            }
        }

        // Remove completed animations
        self.animations.retain(|a| {
            let elapsed = now.duration_since(a.start_time);
            elapsed < a.duration
        });

        any_running
    }

    /// Get the current value of an animation
    pub fn get_value(&self, id: &str) -> Option<f64> {
        self.animations.iter()
            .find(|a| a.id == id)
            .map(|a| a.current_value)
    }
}

fn apply_easing(progress: f64, easing: EasingFunction) -> f64 {
    match easing {
        EasingFunction::Linear => progress,
        EasingFunction::EaseIn => progress * progress,
        EasingFunction::EaseOut => 1.0 - (1.0 - progress) * (1.0 - progress),
        EasingFunction::EaseInOut => {
            if progress < 0.5 {
                2.0 * progress * progress
            } else {
                1.0 - 2.0 * (1.0 - progress) * (1.0 - progress)
            }
        }
    }
}

/// Text renderer optimized for terminal performance
#[derive(Debug)]
pub struct OptimizedTextRenderer {
    pub line_cache: VecDeque<String>,
    pub max_cache_size: usize,
}

impl OptimizedTextRenderer {
    pub fn new() -> Self {
        Self {
            line_cache: VecDeque::new(),
            max_cache_size: 10000,
        }
    }

    /// Render text with syntax highlighting (simplified)
    pub fn render_with_highlighting(&mut self, text: &str, language: Option<&str>) -> Vec<String> {
        // Simple syntax highlighting based on language
        match language {
            Some("rust") => self.highlight_rust(text),
            Some("bash") | Some("shell") => self.highlight_shell(text),
            Some("json") => self.highlight_json(text),
            _ => text.lines().map(|line| line.to_string()).collect(),
        }
    }

    fn highlight_rust(&self, text: &str) -> Vec<String> {
        // Simplified Rust syntax highlighting
        text.lines()
            .map(|line| {
                let mut highlighted = line.to_string();
                
                // Highlight keywords (this is very simplified)
                for keyword in &["fn", "let", "mut", "pub", "struct", "impl", "use"] {
                    highlighted = highlighted.replace(
                        keyword,
                        &format!("\x1b[94m{}\x1b[0m", keyword) // Blue
                    );
                }
                
                highlighted
            })
            .collect()
    }

    fn highlight_shell(&self, text: &str) -> Vec<String> {
        text.lines()
            .map(|line| {
                let mut highlighted = line.to_string();
                
                // Highlight common shell commands
                for cmd in &["cd", "ls", "git", "cargo", "npm", "docker"] {
                    if line.trim_start().starts_with(cmd) {
                        highlighted = format!("\x1b[92m{}\x1b[0m", line); // Green
                        break;
                    }
                }
                
                highlighted
            })
            .collect()
    }

    fn highlight_json(&self, text: &str) -> Vec<String> {
        text.lines()
            .map(|line| {
                let mut highlighted = line.to_string();
                
                // Highlight JSON keys and strings (very simplified)
                if line.contains(':') {
                    highlighted = highlighted.replace(":", "\x1b[93m:\x1b[0m"); // Yellow
                }
                
                highlighted
            })
            .collect()
    }

    /// Cache frequently used lines for performance
    pub fn cache_line(&mut self, line: String) {
        if self.line_cache.len() >= self.max_cache_size {
            self.line_cache.pop_front();
        }
        self.line_cache.push_back(line);
    }
}

impl Default for PerformanceManager {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for AnimationSystem {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for OptimizedTextRenderer {
    fn default() -> Self {
        Self::new()
    }
}
