use std::time::Duration;

#[derive(Debug)]
pub struct EventHandler {
    #[allow(dead_code)]
    pub tick_rate: Duration,
}

impl EventHandler {
    pub fn new(tick_rate: Duration) -> Self {
        Self { tick_rate }
    }
}

impl Default for EventHandler {
    fn default() -> Self {
        Self::new(Duration::from_millis(250))
    }
}
