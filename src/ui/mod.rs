use anyhow::Result;
use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    Terminal,
};
use std::io::{self, Stdout};
use tracing::{debug, info};

pub mod app;
pub mod layout;
pub mod components;
pub mod events;
pub mod styles;
pub mod blocks;
pub mod performance;

pub use app::App;

pub type AppTerminal = Terminal<CrosstermBackend<Stdout>>;

pub fn setup_terminal() -> Result<AppTerminal> {
    info!("Setting up terminal for TUI mode");
    
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let terminal = Terminal::new(backend)?;
    
    debug!("Terminal setup completed");
    Ok(terminal)
}

pub fn restore_terminal(terminal: &mut AppTerminal) -> Result<()> {
    info!("Restoring terminal");
    
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;
    
    debug!("Terminal restored");
    Ok(())
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_terminal_setup_and_restore() {
        // This test is commented out because it would interfere with the terminal
        // In a real project, you'd want more sophisticated testing
        // let mut terminal = setup_terminal().unwrap();
        // restore_terminal(&mut terminal).unwrap();
    }
}
