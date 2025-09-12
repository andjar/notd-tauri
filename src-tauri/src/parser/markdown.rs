// Markdown Parser
// Phase 1: Basic markdown parsing using pulldown-cmark

use pulldown_cmark::{Parser, Options, html};

/// Parse markdown content to HTML
pub fn parse_to_html(content: &str) -> String {
    // Set up markdown parser options
    let mut options = Options::empty();
    options.insert(Options::ENABLE_STRIKETHROUGH);
    options.insert(Options::ENABLE_TABLES);
    options.insert(Options::ENABLE_FOOTNOTES);
    
    // Parse markdown
    let parser = Parser::new_ext(content, options);
    
    // Convert to HTML
    let mut html_output = String::new();
    html::push_html(&mut html_output, parser);
    
    // Post-process to handle custom syntax
    post_process_html(html_output)
}

/// Post-process HTML to handle custom outliner syntax
fn post_process_html(html: String) -> String {
    let mut processed = html;
    
    // Convert [[page links]] to clickable links
    processed = convert_page_links(processed);
    
    // Convert {{transclusion}} markers (for future use)
    processed = convert_transclusion_markers(processed);
    
    // Handle task checkboxes
    processed = convert_task_markers(processed);
    
    processed
}

/// Convert [[Page Name]] syntax to HTML links
fn convert_page_links(html: String) -> String {
    use regex::Regex;
    
    let link_regex = Regex::new(r"\[\[([^\]]+)\]\]").unwrap();
    
    link_regex.replace_all(&html, |caps: &regex::Captures| {
        let page_name = &caps[1];
        format!(
            r#"<a href='#' class='page-link' data-page='{}'>{}</a>"#,
            page_name, page_name
        )
    }).to_string()
}

/// Convert {{transclusion}} markers to placeholders
fn convert_transclusion_markers(html: String) -> String {
    use regex::Regex;
    
    let transclusion_regex = Regex::new(r"\{\{([^}]+)\}\}").unwrap();
    
    transclusion_regex.replace_all(&html, |caps: &regex::Captures| {
        let content = &caps[1];
        format!(
            r#"<div class="transclusion" data-content="{}">[Transclusion: {}]</div>"#,
            content, content
        )
    }).to_string()
}

/// Convert task markers to HTML checkboxes
fn convert_task_markers(html: String) -> String {
    use regex::Regex;
    
    // Match task patterns at the beginning of lines
    let task_regex = Regex::new(r"(?m)^(TODO|DOING|DONE|WAITING|CANCELLED)\s+(.*)").unwrap();
    
    task_regex.replace_all(&html, |caps: &regex::Captures| {
        let status = &caps[1];
        let content = &caps[2];
        
        let (checked, class) = match status {
            "DONE" => (true, "task-done"),
            "CANCELLED" => (true, "task-cancelled"),
            _ => (false, "task-pending"),
        };
        
        format!(
            r#"<div class="task-item {}">
                <input type="checkbox" {} data-status="{}" />
                <span class="task-status">{}</span>
                <span class="task-content">{}</span>
            </div>"#,
            class,
            if checked { "checked" } else { "" },
            status.to_lowercase(),
            status,
            content
        )
    }).to_string()
}

/// Extract plain text from markdown (for search indexing)
pub fn extract_plain_text(content: &str) -> String {
    let parser = Parser::new(content);
    let mut text_parts = Vec::new();
    
    for event in parser {
        match event {
            pulldown_cmark::Event::Text(text) => {
                text_parts.push(text.to_string());
            }
            pulldown_cmark::Event::Code(code) => {
                text_parts.push(code.to_string());
            }
            _ => {}
        }
    }
    
    text_parts.join(" ")
}

/// Check if content contains any markdown formatting
pub fn has_markdown_formatting(content: &str) -> bool {
    // Simple heuristics for common markdown patterns
    content.contains("**") ||    // Bold
    content.contains("*") ||     // Italic
    content.contains("##") ||    // Headers
    content.contains("[") ||     // Links
    content.contains("`") ||     // Code
    content.contains(">") ||     // Blockquotes
    content.contains("- ") ||    // Lists
    content.contains("1. ")      // Numbered lists
}

/// Escape markdown special characters
pub fn escape_markdown(text: &str) -> String {
    text.replace("*", "\\*")
        .replace("_", "\\_")
        .replace("[", "\\[")
        .replace("]", "\\]")
        .replace("(", "\\(")
        .replace(")", "\\)")
        .replace("#", "\\#")
        .replace("+", "\\+")
        .replace("-", "\\-")
        .replace(".", "\\.")
        .replace("!", "\\!")
        .replace("`", "\\`")
        .replace("|", "\\|")
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_basic_markdown_parsing() {
        let content = "# Header\n\nThis is **bold** and *italic* text.";
        let html = parse_to_html(content);
        
        assert!(html.contains("<h1>Header</h1>"));
        assert!(html.contains("<strong>bold</strong>"));
        assert!(html.contains("<em>italic</em>"));
    }
    
    #[test]
    fn test_page_link_conversion() {
        let content = "See [[Page Name]] for details.";
        let html = parse_to_html(content);
        
        assert!(html.contains(r#"<a href='#' class='page-link'"#));
        assert!(html.contains(r#"data-page='Page Name'"#));
    }
    
    #[test]
    fn test_task_marker_conversion() {
        let content = "TODO Complete this task\nDONE Finished task";
        let html = parse_to_html(content);
        
        assert!(html.contains(r#"class="task-item"#));
        assert!(html.contains(r#"data-status="todo""#));
        assert!(html.contains(r#"data-status="done""#));
        assert!(html.contains("checked"));
    }
    
    #[test]
    fn test_plain_text_extraction() {
        let content = "# Header\n\nThis is **bold** text with a [link](http://example.com).";
        let text = extract_plain_text(content);
        
        assert_eq!(text.trim(), "Header This is bold text with a link .");
    }
    
    #[test]
    fn test_markdown_detection() {
        assert!(has_markdown_formatting("This is **bold** text"));
        assert!(has_markdown_formatting("# Header"));
        assert!(has_markdown_formatting("- List item"));
        assert!(!has_markdown_formatting("Plain text"));
    }
    
    #[test]
    fn test_markdown_escaping() {
        let text = "Text with *asterisks* and [brackets]";
        let escaped = escape_markdown(text);
        
        assert_eq!(escaped, "Text with \\*asterisks\\* and \\[brackets\\]");
    }
}
