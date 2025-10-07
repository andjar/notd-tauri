use anyhow::Result;
use crossterm::event::{self, KeyCode, KeyEvent, KeyModifiers};
use std::time::Duration;

/// Terminal events
#[derive(Debug, Clone, Copy)]
pub enum Event {
    /// Key press event
    Key(KeyEvent),
    /// Terminal tick event
    Tick,
}

/// Event handler for the terminal
pub struct EventHandler {
    /// Tick rate in milliseconds
    tick_rate: Duration,
}

impl EventHandler {
    /// Create a new event handler
    pub fn new(tick_rate_ms: u64) -> Self {
        Self {
            tick_rate: Duration::from_millis(tick_rate_ms),
        }
    }

    /// Poll for the next event
    pub fn next(&self) -> Result<Event> {
        if event::poll(self.tick_rate)? {
            if let event::Event::Key(key) = event::read()? {
                return Ok(Event::Key(key));
            }
        }
        Ok(Event::Tick)
    }
}

/// Handle key events for the application
pub fn handle_key_event(key: KeyEvent, app: &mut crate::app::App) {
    match key.code {
        KeyCode::Char('q') | KeyCode::Char('Q') => {
            app.quit();
        }
        KeyCode::Char('c') | KeyCode::Char('C') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            app.quit();
        }
        // Future: Add more key handlers in Phase 3
        KeyCode::Up => {
            // Navigate up (Phase 3)
        }
        KeyCode::Down => {
            // Navigate down (Phase 3)
        }
        KeyCode::Left => {
            // Collapse node (Phase 3)
        }
        KeyCode::Right => {
            // Expand node (Phase 3)
        }
        KeyCode::Enter => {
            // Edit mode (Phase 3)
        }
        _ => {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_event_handler_creation() {
        let handler = EventHandler::new(250);
        assert_eq!(handler.tick_rate, Duration::from_millis(250));
    }
}

