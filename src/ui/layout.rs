use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
};

#[derive(Debug, Clone)]
pub struct AppLayout {
    pub main_horizontal_split: f32,
    pub sidebar_width: u16,
    pub input_height: u16,
    pub status_height: u16,
}

impl AppLayout {
    pub fn new() -> Self {
        Self {
            main_horizontal_split: 0.7,
            sidebar_width: 30,
            input_height: 3,
            status_height: 1,
        }
    }
    
    pub fn create_main_layout(&self, area: Rect) -> std::rc::Rc<[Rect]> {
        Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(self.status_height),
                Constraint::Min(0),
                Constraint::Length(self.input_height),
            ])
            .split(area)
    }
    
    pub fn create_content_layout(&self, area: Rect) -> std::rc::Rc<[Rect]> {
        Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Min(60),
                Constraint::Length(self.sidebar_width),
            ])
            .split(area)
    }
    
    pub fn create_sidebar_layout(&self, area: Rect) -> std::rc::Rc<[Rect]> {
        Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(8),  // Agent info
                Constraint::Min(0),     // Suggestions
            ])
            .split(area)
    }
}

impl Default for AppLayout {
    fn default() -> Self {
        Self::new()
    }
}
