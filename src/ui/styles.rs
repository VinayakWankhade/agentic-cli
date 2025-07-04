use ratatui::style::{Color, Style};
use crate::config::Config;

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct AppTheme {
    #[allow(dead_code)]
    pub primary_color: Color,
    #[allow(dead_code)]
    pub secondary_color: Color,
    #[allow(dead_code)]
    pub accent_color: Color,
    #[allow(dead_code)]
    pub background_color: Color,
    #[allow(dead_code)]
    pub text_color: Color,
    #[allow(dead_code)]
    pub success_color: Color,
    #[allow(dead_code)]
    pub error_color: Color,
    #[allow(dead_code)]
    pub warning_color: Color,
    #[allow(dead_code)]
    pub info_color: Color,
}

impl AppTheme {
    pub fn from_config(config: &Config) -> Self {
        if config.theme.dark_mode {
            Self::dark_theme()
        } else {
            Self::light_theme()
        }
    }
    
    pub fn dark_theme() -> Self {
        Self {
            primary_color: Color::Blue,
            secondary_color: Color::DarkGray,
            accent_color: Color::Cyan,
            background_color: Color::Black,
            text_color: Color::White,
            success_color: Color::Green,
            error_color: Color::Red,
            warning_color: Color::Yellow,
            info_color: Color::Blue,
        }
    }
    
    pub fn light_theme() -> Self {
        Self {
            primary_color: Color::Blue,
            secondary_color: Color::Gray,
            accent_color: Color::Magenta,
            background_color: Color::White,
            text_color: Color::Black,
            success_color: Color::Green,
            error_color: Color::Red,
            warning_color: Color::Yellow,
            info_color: Color::Blue,
        }
    }
    
    pub fn primary_style(&self) -> Style {
        Style::default().fg(self.primary_color)
    }
    
    pub fn secondary_style(&self) -> Style {
        Style::default().fg(self.secondary_color)
    }
    
    pub fn accent_style(&self) -> Style {
        Style::default().fg(self.accent_color)
    }
    
    pub fn success_style(&self) -> Style {
        Style::default().fg(self.success_color)
    }
    
    pub fn error_style(&self) -> Style {
        Style::default().fg(self.error_color)
    }
    
    pub fn warning_style(&self) -> Style {
        Style::default().fg(self.warning_color)
    }
    
    pub fn info_style(&self) -> Style {
        Style::default().fg(self.info_color)
    }
}
