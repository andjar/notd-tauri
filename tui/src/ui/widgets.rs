use crate::app::{App, TreeNode};
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Clear, List, ListItem, ListState, Paragraph, Wrap},
    Frame,
};
use outliner_core::storage::{TagRepository, LinkRepository, NoteRepository};
use chrono::{Datelike, NaiveDate, Weekday};

/// Render the header with title and key hints
pub fn render_header(frame: &mut Frame, app: &App, area: Rect) {
    let title = if let Some(note) = &app.current_note {
        format!(" 📝 {} ", note.title)
    } else {
        " Outliner ".to_string()
    };

    let key_hints = if app.is_editing {
        " [Enter:Save] [Esc:Cancel] [Typing...] "
    } else if app.page_switcher_open {
        " [Esc:Close] [↑/↓:Select] [Enter:Open] [Type to filter] "
    } else if app.search_open {
        " [Esc:Close] [Type to search] [Backspace:Delete] "
    } else {
        " [q:Quit] [↑/↓:Move] [←/→:Collapse/Expand] [Enter:Edit] [n:New] [d:Del] [x:Toggle Task] [Tab/Shift+Tab:Indent] [Alt+↑/↓:Reorder] [/:Search] [Ctrl+P:Pages] [Ctrl+N:New Page] [Ctrl+D:Del Page] [PgUp/PgDn + Alt+Enter:Open] [Shift+Arrows:Calendar] [Shift+Enter:Open Daily] "
    };

    let header_spans = vec![
        Span::styled(
            title,
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw(" | "),
        Span::styled(key_hints, Style::default().fg(Color::DarkGray)),
    ];

    let header = Paragraph::new(Line::from(header_spans))
        .block(Block::default().borders(Borders::ALL))
        .alignment(Alignment::Left);

    frame.render_widget(header, area);
}

/// Render the outline view
pub fn render_outline(frame: &mut Frame, app: &App, area: Rect) {
    let visible_nodes = app.get_visible_nodes();

    if visible_nodes.is_empty() {
        let empty_message = Paragraph::new("This page is empty. Press 'n' to add a node or Ctrl+N to create a new page.")
            .block(Block::default().borders(Borders::ALL).title(" Outline "))
            .alignment(Alignment::Center)
            .style(Style::default().fg(Color::DarkGray));
        frame.render_widget(empty_message, area);
        return;
    }

    // Build lines for each visible node
    let mut lines: Vec<Line> = Vec::new();

    for (i, tree_node) in visible_nodes.iter().enumerate().skip(app.scroll_offset) {
        let mut line = render_node_line(tree_node);
        // Highlight selected line
        if i == app.cursor_position {
            line = line.style(Style::default().bg(Color::Blue).fg(Color::Black));
        }
        lines.push(line);

        // Limit to visible area
        if lines.len() >= (area.height as usize).saturating_sub(2) {
            break;
        }
    }

    let outline = Paragraph::new(lines)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Outline ")
                .title_alignment(Alignment::Left),
        )
        .wrap(Wrap { trim: false });

    frame.render_widget(outline, area);
}

/// Render a single node line with proper indentation
fn render_node_line(tree_node: &TreeNode) -> Line<'_> {
    let indent = "  ".repeat(tree_node.depth);
    let node = &tree_node.node;

    // Determine bullet point
    let bullet = if node.is_task {
        if node.task_completed {
            "☑ "
        } else {
            "☐ "
        }
    } else if !tree_node.children.is_empty() {
        if tree_node.is_expanded {
            "▼ "
        } else {
            "▶ "
        }
    } else {
        "• "
    };

    // Style based on node type
    let content_style = if node.is_task {
        if node.task_completed {
            Style::default()
                .fg(Color::DarkGray)
                .add_modifier(Modifier::CROSSED_OUT)
        } else {
            Style::default().fg(Color::White)
        }
    } else if !tree_node.children.is_empty() {
        Style::default()
            .fg(Color::Yellow)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Color::White)
    };

    // Priority indicator
    let priority_indicator = if node.is_task {
        match &node.task_priority {
            Some(p) => match p {
                outliner_core::models::TaskPriority::High => " 🔴",
                outliner_core::models::TaskPriority::Medium => " 🟡",
                outliner_core::models::TaskPriority::Low => " 🟢",
            },
            None => "",
        }
    } else {
        ""
    };

    let spans = vec![
        Span::raw(indent),
        Span::styled(bullet, Style::default().fg(Color::Cyan)),
        Span::styled(node.content.clone(), content_style),
        Span::raw(priority_indicator),
    ];

    Line::from(spans)
}

/// Render the status bar at the bottom
pub fn render_status_bar(frame: &mut Frame, app: &App, area: Rect) {
    let visible_count = app.get_visible_nodes().len();
    let status_text = if let Some(tag) = &app.tag_filter {
        format!(" {} nodes | Pages: {} | Tag Filter: #{} | [/:Search] [Ctrl+P: Switch] [Ctrl+N: New Page] [Ctrl+D: Delete Page] ", visible_count, app.notes.len(), tag)
    } else {
        format!(" {} nodes | Pages: {} | [/:Search] [Ctrl+P: Switch] [Ctrl+N: New Page] [Ctrl+D: Delete Page] ", visible_count, app.notes.len())
    };

    let status_bar = Paragraph::new(status_text)
        .style(Style::default().bg(Color::DarkGray).fg(Color::White))
        .alignment(Alignment::Center);

    frame.render_widget(status_bar, area);
}

/// Render the sidebar pages list
pub fn render_sidebar_pages(frame: &mut Frame, app: &App, area: Rect) {
    let items: Vec<ListItem> = app
        .notes
        .iter()
        .enumerate()
        .map(|(i, n)| {
            let mut line = Line::from(n.title.clone());
            if Some(&n.id) == app.current_note.as_ref().map(|cn| &cn.id) {
                line = line.style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD));
            }
            if i == app.sidebar_pages_selected_index {
                line = line.style(Style::default().bg(Color::Blue).fg(Color::Black));
            }
            ListItem::new(line)
        })
        .collect();

    let mut state = ListState::default();
    if !app.notes.is_empty() {
        state.select(Some(app.sidebar_pages_selected_index));
    }

    let list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Pages ")
                .title_alignment(Alignment::Left),
        )
        .highlight_style(Style::default().bg(Color::Blue).fg(Color::Black));

    frame.render_stateful_widget(list, area, &mut state);
}

/// Render sidebar with Tags panel (top) and Pages list (bottom)
pub fn render_sidebar_tags_and_pages(frame: &mut Frame, app: &App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(9), Constraint::Length(10), Constraint::Min(0)])
        .split(area);

    // Calendar at the top
    render_calendar(frame, app, chunks[0]);

    // Tags panel (usage counts)
    let mut tag_lines: Vec<Line> = Vec::new();
    if let Ok(counts) = TagRepository::get_usage_counts(&app.db_connection) {
        for (tag, count) in counts.into_iter().take(8) {
            let mut line = Line::from(format!("#{} ({})", tag.name, count));
            if let Some(active) = &app.tag_filter { if *active == tag.name { line = line.style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)); } }
            tag_lines.push(line);
        }
    }
    if tag_lines.is_empty() { tag_lines.push(Line::from("No tags")); }
    let tags_widget = Paragraph::new(tag_lines)
        .block(Block::default().borders(Borders::ALL).title(" Tags "))
        .wrap(Wrap { trim: true });
    frame.render_widget(tags_widget, chunks[1]);

    // Pages list below
    render_sidebar_pages(frame, app, chunks[2]);
}

/// Render backlinks panel for the current note
pub fn render_backlinks_panel(frame: &mut Frame, app: &App, area: Rect) {
    let mut lines: Vec<Line> = Vec::new();
    if let Some(current) = &app.current_note {
        if let Ok(links) = LinkRepository::get_backlinks(&app.db_connection, &current.id) {
            for link in links.into_iter().take((area.height as usize).saturating_sub(2)) {
                // Resolve source note title if possible
                let title = NoteRepository::get_by_id(&app.db_connection, &link.source_note_id)
                    .map(|n| n.title)
                    .unwrap_or(link.source_note_id);
                let text = if let Some(txt) = link.link_text { format!("{} — {}", title, txt) } else { title };
                lines.push(Line::from(text));
            }
        }
    }
    if lines.is_empty() { lines.push(Line::from("No backlinks")); }
    let widget = Paragraph::new(lines)
        .block(Block::default().borders(Borders::ALL).title(" Backlinks "))
        .wrap(Wrap { trim: true });
    frame.render_widget(widget, area);
}

/// Render the search overlay with live results
pub fn render_search_overlay(frame: &mut Frame, app: &App, area: Rect) {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(25), Constraint::Percentage(50), Constraint::Percentage(25)])
        .split(area);

    let area_mid = popup_layout[1];
    let inner_h = area_mid.height.saturating_sub(2);
    let inner_w = area_mid.width.saturating_sub(2);
    let inner_x = area_mid.x + 1;
    let inner_y = area_mid.y + 1;
    let inner = Rect { x: inner_x, y: inner_y, width: inner_w, height: inner_h };

    // Border and clear
    let block = Block::default().borders(Borders::ALL).title(" Search ");
    frame.render_widget(Clear, area_mid);
    frame.render_widget(block, area_mid);

    // Split into input + results
    let inner_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(1), Constraint::Min(0)])
        .split(inner);

    let input = Paragraph::new(Text::from(format!("/ {}", app.search_query)))
        .style(Style::default().fg(Color::White))
        .block(Block::default());
    frame.render_widget(input, inner_chunks[0]);

    // Results list
    let items: Vec<ListItem> = app
        .search_results
        .iter()
        .map(|n| ListItem::new(Line::from(n.content.clone())))
        .collect();
    let list = List::new(items).block(Block::default());
    frame.render_widget(list, inner_chunks[1]);
}

/// Render the page switcher overlay (center modal with filter input and list)
pub fn render_page_switcher(frame: &mut Frame, app: &App, area: Rect) {
    // Centered box
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(35),
            Constraint::Percentage(30),
            Constraint::Percentage(35),
        ])
        .split(area);

    let area_mid = popup_layout[1];
    let inner_h = area_mid.height.saturating_sub(2);
    let inner_w = area_mid.width.saturating_sub(2);
    let inner_x = area_mid.x + 1;
    let inner_y = area_mid.y + 1;
    let inner = Rect { x: inner_x, y: inner_y, width: inner_w, height: inner_h };

    // Draw border and clear background
    let block = Block::default().borders(Borders::ALL).title(" Page Switcher ");
    frame.render_widget(Clear, area_mid);
    frame.render_widget(block, area_mid);

    // Split inner into filter input + list
    let inner_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(1), Constraint::Min(0)])
        .split(inner);

    // Filter line
    let filter = Paragraph::new(Text::from(format!("> {}", app.page_filter)))
        .style(Style::default().fg(Color::White))
        .block(Block::default());
    frame.render_widget(filter, inner_chunks[0]);

    // List of filtered notes
    let filtered = app.get_filtered_notes();
    let items: Vec<ListItem> = filtered
        .iter()
        .enumerate()
        .map(|(i, n)| {
            let mut line = Line::from(n.title.clone());
            if i == app.page_switcher_selection_index {
                line = line.style(Style::default().bg(Color::Blue).fg(Color::Black));
            }
            ListItem::new(line)
        })
        .collect();

    let mut state = ListState::default();
    if !filtered.is_empty() {
        state.select(Some(app.page_switcher_selection_index));
    }

    let list = List::new(items)
        .block(Block::default())
        .highlight_style(Style::default().bg(Color::Blue).fg(Color::Black));
    frame.render_stateful_widget(list, inner_chunks[1], &mut state);
}

/// Render a simple month calendar with current day and selection highlights
pub fn render_calendar(frame: &mut Frame, app: &App, area: Rect) {
    let mut lines: Vec<Line> = Vec::new();
    let month_start = app.calendar_month_start;
    let title = format!("{} {}", month_start.format("%B"), month_start.year());
    lines.push(Line::from(Span::styled(title, Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))));
    lines.push(Line::from("Mo Tu We Th Fr Sa Su"));

    // Determine grid start (Monday as first column)
    let first_weekday = match month_start.weekday() { Weekday::Mon => 0, Weekday::Tue => 1, Weekday::Wed => 2, Weekday::Thu => 3, Weekday::Fri => 4, Weekday::Sat => 5, Weekday::Sun => 6 };
    let mut day = 1i32;
    let days_in_month = days_in_month(month_start.year(), month_start.month());
    let today = chrono::Utc::now().date_naive();

    // Up to 6 rows
    for row in 0..6 {
        let mut row_spans: Vec<Span> = Vec::new();
        for col in 0..7 {
            let mut text = "  ".to_string();
            let cell_index = row * 7 + col;
            if cell_index >= first_weekday && day <= days_in_month as i32 {
                text = format!("{:>2}", day);
                let date = NaiveDate::from_ymd_opt(month_start.year(), month_start.month(), day as u32)
                    .unwrap_or(month_start);
                let mut style = Style::default().fg(Color::White);
                if date == today {
                    style = style.fg(Color::Cyan).add_modifier(Modifier::BOLD);
                }
                if date == app.calendar_selected {
                    style = style.bg(Color::Blue).fg(Color::Black);
                }
                row_spans.push(Span::styled(text, style));
                day += 1;
            } else {
                row_spans.push(Span::raw(text));
            }
            if col < 6 { row_spans.push(Span::raw(" ")); }
        }
        lines.push(Line::from(row_spans));
        if day > days_in_month as i32 { break; }
    }

    let widget = Paragraph::new(lines)
        .block(Block::default().borders(Borders::ALL).title(" Calendar "))
        .wrap(Wrap { trim: true });
    frame.render_widget(widget, area);
}

fn days_in_month(year: i32, month: u32) -> u32 {
    // Next month first day minus one day
    let (ny, nm) = if month == 12 { (year + 1, 1) } else { (year, month + 1) };
    let first_next = NaiveDate::from_ymd_opt(ny, nm, 1).unwrap();
    let last_this = first_next - chrono::Duration::days(1);
    last_this.day()
}

