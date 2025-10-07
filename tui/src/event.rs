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
        // Move/reorder with Alt, plain navigation otherwise (order of patterns matters)
        KeyCode::Up if key.modifiers.contains(KeyModifiers::ALT) => {
            if !app.is_editing { let _ = app.move_selected_up(); }
        }
        KeyCode::Down if key.modifiers.contains(KeyModifiers::ALT) => {
            if !app.is_editing { let _ = app.move_selected_down(); }
        }
        KeyCode::Up => {
            if !app.is_editing { app.move_cursor_up(); }
        }
        KeyCode::Down => {
            if !app.is_editing { app.move_cursor_down(); }
        }
        // Expand/Collapse
        KeyCode::Left => {
            if !app.is_editing { app.toggle_selected_expand_collapse(Some(false)); }
        }
        KeyCode::Right => {
            if !app.is_editing { app.toggle_selected_expand_collapse(Some(true)); }
        }
        // Edit mode controls
        KeyCode::Enter => {
            if app.is_editing {
                let _ = app.commit_edit();
            } else {
                app.start_editing();
            }
        }
        KeyCode::Esc => {
            if app.is_editing { app.cancel_edit(); }
        }
        KeyCode::Backspace => {
            if app.is_editing { app.edit_buffer.pop(); }
        }
        KeyCode::Char(ch) => {
            if app.is_editing { app.edit_buffer.push(ch); }
            else if ch == 'n' { let _ = app.create_sibling_below(); }
            else if ch == 'd' { let _ = app.delete_selected(); }
        }
        // CRUD via non-char
        KeyCode::Insert => {
            if !app.is_editing { let _ = app.create_sibling_below(); }
        }
        KeyCode::Delete => {
            if !app.is_editing { let _ = app.delete_selected(); }
        }
        // Indent / Outdent
        KeyCode::Tab => {
            if !app.is_editing { let _ = app.indent_selected(); }
        }
        KeyCode::BackTab => {
            if !app.is_editing { let _ = app.outdent_selected(); }
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

