use crate::app::App;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    Frame,
};

use super::{render_header, render_outline, render_status_bar, render_page_switcher, render_search_overlay, render_sidebar_tags_and_pages, render_backlinks_panel};

/// Render the complete UI
pub fn render(frame: &mut Frame, app: &App) {
    let size = frame.size();

    // Create main layout: header, content, status bar
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),  // Header
            Constraint::Min(0),     // Content
            Constraint::Length(1),  // Status bar
        ])
        .split(size);

    // Render components
    render_header(frame, app, chunks[0]);
    render_content(frame, app, chunks[1]);
    render_status_bar(frame, app, chunks[2]);

    // Overlays (drawn last)
    if app.page_switcher_open {
        render_page_switcher(frame, app, size);
    }
    if app.search_open {
        render_search_overlay(frame, app, size);
    }
}

/// Render the main content area (will have sidebar + outliner in future)
fn render_content(frame: &mut Frame, app: &App, area: Rect) {
    // Phase 4: Split content into sidebar and outline
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Length(30), // Sidebar width
            Constraint::Min(0),     // Main outliner
            Constraint::Length(30), // Backlinks panel
        ])
        .split(area);

    render_sidebar_tags_and_pages(frame, app, chunks[0]);
    render_outline(frame, app, chunks[1]);
    render_backlinks_panel(frame, app, chunks[2]);
}

