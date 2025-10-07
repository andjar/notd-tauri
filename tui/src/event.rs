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
    // When search is open, handle search input first
    if app.search_open {
        match key.code {
            KeyCode::Esc => app.close_search(),
            KeyCode::Enter => {
                // If query starts with #, treat as tag filter
                if app.search_query.starts_with('#') {
                    let name = app.search_query.trim_start_matches('#').trim().to_string();
                    if !name.is_empty() { let _ = app.set_tag_filter(name); }
                    app.close_search();
                }
            }
            KeyCode::Backspace => { app.backspace_search_query(); },
            KeyCode::Char(c) => { if !key.modifiers.contains(KeyModifiers::CONTROL) { app.update_search_query(c); } },
            _ => {}
        }
        return;
    }
    // When page switcher is open, handle its own controls first
    if app.page_switcher_open {
        match key.code {
            KeyCode::Esc => app.close_page_switcher(),
            KeyCode::Up => app.page_switcher_up(),
            KeyCode::Down => app.page_switcher_down(),
            KeyCode::Enter => { let _ = app.page_switcher_activate(); },
            KeyCode::Backspace => { app.page_filter.pop(); },
            KeyCode::Char(c) => { if !key.modifiers.contains(KeyModifiers::CONTROL) { app.page_filter.push(c); } },
            _ => {}
        }
        return;
    }

    match key.code {
        // Calendar interactions (Shift-modified first to avoid unreachable patterns)
        KeyCode::Left if key.modifiers.contains(KeyModifiers::SHIFT) => { app.calendar_move_day(-1); }
        KeyCode::Right if key.modifiers.contains(KeyModifiers::SHIFT) => { app.calendar_move_day(1); }
        KeyCode::Up if key.modifiers.contains(KeyModifiers::SHIFT) => { app.calendar_move_week(-1); }
        KeyCode::Down if key.modifiers.contains(KeyModifiers::SHIFT) => { app.calendar_move_week(1); }
        KeyCode::PageUp if key.modifiers.contains(KeyModifiers::SHIFT) => { app.calendar_prev_month(); }
        KeyCode::PageDown if key.modifiers.contains(KeyModifiers::SHIFT) => { app.calendar_next_month(); }
        KeyCode::Enter if key.modifiers.contains(KeyModifiers::SHIFT) => { if !app.is_editing { let _ = app.open_selected_daily_note(); } }
        // Task toggle
        KeyCode::Char('x') => {
            if !app.is_editing { let _ = app.toggle_selected_task(); }
        }
        KeyCode::Char(' ') => {
            if !app.is_editing { let _ = app.toggle_selected_task(); }
        }
        // Search toggle
        KeyCode::Char('/') => {
            if !app.is_editing { app.open_search(); }
        }
        KeyCode::Char('q') | KeyCode::Char('Q') => {
            app.quit();
        }
        KeyCode::Char('c') | KeyCode::Char('C') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            app.quit();
        }
        // Page management shortcuts (Phase 4)
        KeyCode::Char('p') | KeyCode::Char('P') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            let _ = app.open_page_switcher();
        }
        KeyCode::Char('n') | KeyCode::Char('N') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            if !app.is_editing { let _ = app.create_new_page(); }
        }
        KeyCode::Char('d') | KeyCode::Char('D') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            if !app.is_editing { let _ = app.delete_current_page(); }
        }
        // Sidebar navigation (use PageUp/PageDown to move selection; Enter to open)
        KeyCode::PageUp => { app.sidebar_select_up(); }
        KeyCode::PageDown => { app.sidebar_select_down(); }
        KeyCode::Enter if key.modifiers.contains(KeyModifiers::ALT) => { let _ = app.sidebar_activate_selected(); }
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
        // Edit mode controls (generic Enter after Shift+Enter)
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
            else if ch == 't' && key.modifiers.contains(KeyModifiers::CONTROL) {
                // Ctrl+T clears tag filter
                let _ = app.clear_tag_filter();
            }
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

