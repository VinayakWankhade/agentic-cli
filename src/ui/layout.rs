use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
};

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct AppLayout {
    #[allow(dead_code)]
    pub main_horizontal_split: f32,
    #[allow(dead_code)]
    pub sidebar_width: u16,
    #[allow(dead_code)]
    pub input_height: u16,
    #[allow(dead_code)]
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
