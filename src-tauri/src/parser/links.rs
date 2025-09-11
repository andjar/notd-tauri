// Link Parser
// Phase 1: Extract and validate page links from content

use regex::Regex;

/// Extract all [[Page Name]] style links from content
pub fn extract_page_links(content: &str) -> Vec<String> {
    let link_regex = Regex::new(r"\[\[([^\]]+)\]\]").unwrap();
    
    link_regex
        .captures_iter(content)
        .map(|cap| cap[1].trim().to_string())
        .filter(|link| !link.is_empty())
        .collect()
}

/// Find malformed links (incomplete [[ or ]] patterns)
pub fn find_malformed_links(content: &str) -> Vec<String> {
    let mut malformed = Vec::new();
    
    // Find opening [[ without closing ]]
    let open_regex = Regex::new(r"\[\[[^\]]*$").unwrap();
    for mat in open_regex.find_iter(content) {
        malformed.push(mat.as_str().to_string());
    }
    
    // Find closing ]] without opening [[
    let close_regex = Regex::new(r"^[^\[]*\]\]").unwrap();
    for mat in close_regex.find_iter(content) {
        malformed.push(mat.as_str().to_string());
    }
    
    // Find single brackets that might be typos
    let single_open_regex = Regex::new(r"\[[^\[\]]*\n").unwrap();
    for mat in single_open_regex.find_iter(content) {
        let text = mat.as_str().trim();
        if !text.is_empty() && text.len() > 1 {
            malformed.push(format!("Possible incomplete link: {}", text));
        }
    }
    
    malformed
}

/// Get link context at cursor position (for autocomplete)
pub fn get_link_context(content: &str, cursor_position: usize) -> Option<String> {
    if cursor_position > content.len() {
        return None;
    }
    
    // Look backwards from cursor to find [[
    let before_cursor = &content[..cursor_position];
    if let Some(open_pos) = before_cursor.rfind("[[") {
        // Check if there's a closing ]] between open_pos and cursor
        let between = &content[open_pos..cursor_position];
        if !between.contains("]]") {
            // Extract partial link text
            let partial = &content[open_pos + 2..cursor_position];
            if !partial.contains('\n') && partial.len() <= 100 {
                return Some(partial.to_string());
            }
        }
    }
    
    None
}

/// Validate that a page name is valid
pub fn is_valid_page_name(name: &str) -> bool {
    // Basic validation rules
    !name.is_empty() &&
    name.len() <= 255 &&
    !name.contains('\n') &&
    !name.contains('\r') &&
    !name.starts_with('/') &&
    !name.ends_with('/') &&
    !name.contains("//")
}

/// Extract external links (http/https URLs)
pub fn extract_external_links(content: &str) -> Vec<ExternalLink> {
    let mut links = Vec::new();
    
    // Markdown link pattern [text](url)
    let markdown_link_regex = Regex::new(r"\[([^\]]+)\]\(([^)]+)\)").unwrap();
    for cap in markdown_link_regex.captures_iter(content) {
        if let (Some(text), Some(url)) = (cap.get(1), cap.get(2)) {
            links.push(ExternalLink {
                text: text.as_str().to_string(),
                url: url.as_str().to_string(),
                link_type: if url.as_str().starts_with("http") {
                    ExternalLinkType::Http
                } else if url.as_str().starts_with("mailto:") {
                    ExternalLinkType::Email
                } else {
                    ExternalLinkType::Other
                },
            });
        }
    }
    
    // Plain URL pattern (basic detection)
    let url_regex = Regex::new(r"https?://[^\s<>\[\]]+").unwrap();
    for mat in url_regex.find_iter(content) {
        let url = mat.as_str().to_string();
        // Only add if not already captured as markdown link
        if !links.iter().any(|link| link.url == url) {
            links.push(ExternalLink {
                text: url.clone(),
                url: url.clone(),
                link_type: ExternalLinkType::Http,
            });
        }
    }
    
    links
}

/// Replace page links with rendered HTML
pub fn render_page_links(content: &str, link_renderer: impl Fn(&str) -> String) -> String {
    let link_regex = Regex::new(r"\[\[([^\]]+)\]\]").unwrap();
    
    link_regex.replace_all(content, |caps: &regex::Captures| {
        let page_name = caps[1].trim();
        link_renderer(page_name)
    }).to_string()
}

/// Convert page names to valid paths/slugs
pub fn page_name_to_path(name: &str) -> String {
    name.trim()
        .to_lowercase()
        .replace(' ', "-")
        .chars()
        .filter(|c| c.is_alphanumeric() || *c == '-' || *c == '_' || *c == '/')
        .collect::<String>()
        .trim_matches('-')
        .to_string()
}

/// Extract all unique page references from content
pub fn get_referenced_pages(content: &str) -> Vec<PageReference> {
    let mut references = Vec::new();
    let link_regex = Regex::new(r"\[\[([^\]]+)\]\]").unwrap();
    
    for (index, cap) in link_regex.captures_iter(content).enumerate() {
        let page_name = cap[1].trim();
        let start_pos = cap.get(0).unwrap().start();
        let end_pos = cap.get(0).unwrap().end();
        
        references.push(PageReference {
            page_name: page_name.to_string(),
            position: start_pos,
            length: end_pos - start_pos,
            reference_index: index,
        });
    }
    
    references
}

/// Check if content has any links
pub fn has_links(content: &str) -> bool {
    content.contains("[[") && content.contains("]]")
}

// ================================
// Data Structures
// ================================

#[derive(Debug, Clone)]
pub struct ExternalLink {
    pub text: String,
    pub url: String,
    pub link_type: ExternalLinkType,
}

#[derive(Debug, Clone)]
pub enum ExternalLinkType {
    Http,
    Email,
    Other,
}

#[derive(Debug, Clone)]
pub struct PageReference {
    pub page_name: String,
    pub position: usize,
    pub length: usize,
    pub reference_index: usize,
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_extract_page_links() {
        let content = "See [[Page One]] and [[Page Two]] for more info about [[Another Page]].";
        let links = extract_page_links(content);
        
        assert_eq!(links.len(), 3);
        assert!(links.contains(&"Page One".to_string()));
        assert!(links.contains(&"Page Two".to_string()));
        assert!(links.contains(&"Another Page".to_string()));
    }
    
    #[test]
    fn test_extract_page_links_with_whitespace() {
        let content = "[[ Page With Spaces ]] and [[  Trimmed  ]]";
        let links = extract_page_links(content);
        
        assert_eq!(links.len(), 2);
        assert!(links.contains(&"Page With Spaces".to_string()));
        assert!(links.contains(&"Trimmed".to_string()));
    }
    
    #[test]
    fn test_empty_links_filtered() {
        let content = "[[]] and [[   ]] should be filtered out, but [[Real Link]] should remain";
        let links = extract_page_links(content);
        
        assert_eq!(links.len(), 1);
        assert_eq!(links[0], "Real Link");
    }
    
    #[test]
    fn test_find_malformed_links() {
        let content = "This has [[ incomplete link at end";
        let malformed = find_malformed_links(content);
        
        assert!(!malformed.is_empty());
    }
    
    #[test]
    fn test_get_link_context() {
        let content = "Start [[Partial Link] here";
        let context = get_link_context(content, 17); // Position after "Partial"
        
        assert_eq!(context, Some("Partial".to_string()));
    }
    
    #[test]
    fn test_get_link_context_no_context() {
        let content = "No link context here";
        let context = get_link_context(content, 10);
        
        assert_eq!(context, None);
    }
    
    #[test]
    fn test_valid_page_name() {
        assert!(is_valid_page_name("Valid Page Name"));
        assert!(is_valid_page_name("project/sub-page"));
        assert!(!is_valid_page_name(""));
        assert!(!is_valid_page_name("name\nwith\nnewlines"));
        assert!(!is_valid_page_name("/starts-with-slash"));
        assert!(!is_valid_page_name("ends-with-slash/"));
    }
    
    #[test]
    fn test_extract_external_links() {
        let content = "Visit [Google](https://google.com) or check out https://example.com directly.";
        let links = extract_external_links(content);
        
        assert_eq!(links.len(), 2);
        assert_eq!(links[0].text, "Google");
        assert_eq!(links[0].url, "https://google.com");
        assert_eq!(links[1].text, "https://example.com");
        assert_eq!(links[1].url, "https://example.com");
    }
    
    #[test]
    fn test_page_name_to_path() {
        assert_eq!(page_name_to_path("My Project Page"), "my-project-page");
        assert_eq!(page_name_to_path("  Spaces  "), "spaces");
        assert_eq!(page_name_to_path("Special!@#$%Characters"), "specialcharacters");
        assert_eq!(page_name_to_path("project/sub-page"), "project/sub-page");
    }
    
    #[test]
    fn test_get_referenced_pages() {
        let content = "See [[Page A]] and then [[Page B]] for details.";
        let references = get_referenced_pages(content);
        
        assert_eq!(references.len(), 2);
        assert_eq!(references[0].page_name, "Page A");
        assert_eq!(references[0].reference_index, 0);
        assert_eq!(references[1].page_name, "Page B");
        assert_eq!(references[1].reference_index, 1);
    }
    
    #[test]
    fn test_has_links() {
        assert!(has_links("Text with [[a link]] here"));
        assert!(!has_links("Text with no links"));
        assert!(!has_links("Text with only [[ opening"));
        assert!(!has_links("Text with only ]] closing"));
    }
    
    #[test]
    fn test_render_page_links() {
        let content = "See [[Page One]] and [[Page Two]]";
        let rendered = render_page_links(content, |name| {
            format!("<a href=\"{}\">{}</a>", name.to_lowercase(), name)
        });
        
        assert!(rendered.contains("<a href=\"page one\">Page One</a>"));
        assert!(rendered.contains("<a href=\"page two\">Page Two</a>"));
    }
}
