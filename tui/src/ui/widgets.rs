use crate::app::{App, TreeNode};
use ratatui::{
    layout::{Alignment, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap},
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
    } else {
        " [q:Quit] [↑/↓:Move] [←/→:Collapse/Expand] [Enter:Edit] [n:New] [d:Del] [Tab/Shift+Tab:Indent] [Alt+↑/↓:Reorder] "
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
        let empty_message = Paragraph::new("No content to display.\n\nPress 'q' to quit.")
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
        " {} nodes | Phase 2: Read-Only View | Press 'q' to quit ",
        visible_count
    );

    let status_bar = Paragraph::new(status_text)
        .style(Style::default().bg(Color::DarkGray).fg(Color::White))
        .alignment(Alignment::Center);

    frame.render_widget(status_bar, area);
}

