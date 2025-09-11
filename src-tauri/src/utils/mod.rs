// Utility Functions
// Phase 1: Common utility functions used across the application

use chrono::{DateTime, Utc};
use std::path::Path;

/// Generate a UUID v4 string
pub fn generate_uuid() -> String {
    uuid::Uuid::new_v4().to_string()
}

/// Get current timestamp as UTC DateTime
pub fn now() -> DateTime<Utc> {
    Utc::now()
}

/// Format a date as YYYY-MM-DD string
pub fn format_date_path(date: &DateTime<Utc>) -> String {
    date.format("%Y-%m-%d").to_string()
}

/// Check if a string is a valid date path (YYYY-MM-DD format)
pub fn is_date_path(path: &str) -> bool {
    path.len() == 10 && path.matches('-').count() == 2 && {
        let parts: Vec<&str> = path.split('-').collect();
        parts.len() == 3 &&
        parts[0].len() == 4 && parts[0].chars().all(|c| c.is_ascii_digit()) &&
        parts[1].len() == 2 && parts[1].chars().all(|c| c.is_ascii_digit()) &&
        parts[2].len() == 2 && parts[2].chars().all(|c| c.is_ascii_digit())
    }
}

/// Validate that a page title is acceptable
pub fn is_valid_page_title(title: &str) -> bool {
    !title.is_empty() && 
    title.len() <= 255 && 
    !title.contains('\n') && 
    !title.contains('\r')
}

/// Validate that a page path is acceptable
pub fn is_valid_page_path(path: &str) -> bool {
    !path.is_empty() && 
    path.len() <= 500 && 
    !path.contains('\n') && 
    !path.contains('\r') && 
    !path.starts_with('/') && 
    !path.ends_with('/') && 
    !path.contains("//")
}

/// Sanitize user input for safe storage
pub fn sanitize_string(input: &str) -> String {
    input
        .chars()
        .filter(|&c| c != '\0' && c != '\u{FEFF}') // Remove null and BOM
        .collect::<String>()
        .trim()
        .to_string()
}

/// Ensure a directory exists, creating it if necessary
pub fn ensure_dir_exists<P: AsRef<Path>>(path: P) -> std::io::Result<()> {
    let path = path.as_ref();
    if !path.exists() {
        std::fs::create_dir_all(path)?;
    }
    Ok(())
}

/// Get the parent directory from a file path
pub fn get_parent_dir(path: &str) -> Option<String> {
    match path.rfind('/') {
        Some(pos) => Some(path[..pos].to_string()),
        None => None,
    }
}

/// Calculate a simple hash for content (for change detection)
pub fn simple_hash(content: &str) -> u64 {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    
    let mut hasher = DefaultHasher::new();
    content.hash(&mut hasher);
    hasher.finish()
}

/// Truncate text to a specified length with ellipsis
pub fn truncate_text(text: &str, max_len: usize) -> String {
    if text.len() <= max_len {
        text.to_string()
    } else {
        format!("{}...", &text[..max_len.saturating_sub(3)])
    }
}

/// Convert a relative timestamp to human-readable format
pub fn format_relative_time(timestamp: &DateTime<Utc>) -> String {
    let now = Utc::now();
    let duration = now.signed_duration_since(*timestamp);
    
    if duration.num_seconds() < 60 {
        "just now".to_string()
    } else if duration.num_minutes() < 60 {
        format!("{}m ago", duration.num_minutes())
    } else if duration.num_hours() < 24 {
        format!("{}h ago", duration.num_hours())
    } else if duration.num_days() < 7 {
        format!("{}d ago", duration.num_days())
    } else {
        timestamp.format("%Y-%m-%d").to_string()
    }
}

/// Extract words from text for search indexing
pub fn extract_words(text: &str) -> Vec<String> {
    text
        .split_whitespace()
        .map(|word| word.trim_matches(|c: char| !c.is_alphanumeric()).to_lowercase())
        .filter(|word| !word.is_empty() && word.len() > 2)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_is_date_path() {
        assert!(is_date_path("2024-01-15"));
        assert!(is_date_path("1999-12-31"));
        assert!(!is_date_path("24-01-15"));
        assert!(!is_date_path("2024-1-15"));
        assert!(!is_date_path("2024-01-1"));
        assert!(!is_date_path("not-a-date"));
        assert!(!is_date_path(""));
    }
    
    #[test]
    fn test_is_valid_page_title() {
        assert!(is_valid_page_title("Valid Title"));
        assert!(is_valid_page_title("Title with 123 numbers"));
        assert!(!is_valid_page_title(""));
        assert!(!is_valid_page_title("Title\nwith\nnewlines"));
    }
    
    #[test]
    fn test_is_valid_page_path() {
        assert!(is_valid_page_path("valid/path"));
        assert!(is_valid_page_path("projects/website/notes"));
        assert!(!is_valid_page_path(""));
        assert!(!is_valid_page_path("/starts-with-slash"));
        assert!(!is_valid_page_path("ends-with-slash/"));
        assert!(!is_valid_page_path("has//double/slash"));
    }
    
    #[test]
    fn test_sanitize_string() {
        assert_eq!(sanitize_string("  clean text  "), "clean text");
        assert_eq!(sanitize_string("text\u{0000}with\u{FEFF}marks"), "textwithmarks");
    }
    
    #[test]
    fn test_get_parent_dir() {
        assert_eq!(get_parent_dir("projects/website/notes"), Some("projects/website".to_string()));
        assert_eq!(get_parent_dir("single"), None);
        assert_eq!(get_parent_dir(""), None);
    }
    
    #[test]
    fn test_truncate_text() {
        assert_eq!(truncate_text("short", 10), "short");
        assert_eq!(truncate_text("this is a very long text", 10), "this is...");
        assert_eq!(truncate_text("exactly10!", 10), "exactly10!");
    }
    
    #[test]
    fn test_extract_words() {
        let words = extract_words("Hello, world! This is a test.");
        assert!(words.contains(&"hello".to_string()));
        assert!(words.contains(&"world".to_string()));
        assert!(words.contains(&"test".to_string()));
        assert!(!words.contains(&"is".to_string())); // Too short
        assert!(!words.contains(&"a".to_string())); // Too short
    }
}
