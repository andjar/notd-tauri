use crate::app::{App, TreeNode};
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Clear, List, ListItem, ListState, Paragraph, Wrap},
    Frame,
};

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
    } else {
        " [q:Quit] [↑/↓:Move] [←/→:Collapse/Expand] [Enter:Edit] [n:New] [d:Del] [Tab/Shift+Tab:Indent] [Alt+↑/↓:Reorder] [Ctrl+P:Pages] [Ctrl+N:New Page] [Ctrl+D:Del Page] [PgUp/PgDn + Alt+Enter:Open] "
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
    let status_text = format!(
        " {} nodes | Pages: {} | [Ctrl+P: Switch] [Ctrl+N: New Page] [Ctrl+D: Delete Page] ",
        visible_count,
        app.notes.len()
    );

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

