// Content Processing Commands
// Phase 1: Basic content parsing and processing operations

use tauri::{AppHandle, Manager};
use serde::{Serialize, Deserialize};

use crate::{AppState, parser};
use super::CommandResponse;

// ================================
// Request/Response Types
// ================================

#[derive(Debug, Serialize, Deserialize)]
pub struct ParseMarkdownRequest {
    pub content: String,
    pub render_mode: Option<bool>, // true for render mode, false for edit mode
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ExtractLinksRequest {
    pub content: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ExtractPropertiesRequest {
    pub content: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MarkdownResult {
    pub html: String,
    pub raw: String,
    pub links: Vec<String>,
    pub has_tasks: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PropertyResult {
    pub key: String,
    pub value: String,
    pub line_number: usize,
}

// ================================
// Content Processing Commands
// ================================

/// Parse markdown content and return HTML
#[tauri::command]
pub async fn parse_markdown(
    request: ParseMarkdownRequest,
) -> Result<CommandResponse<MarkdownResult>, ()> {
    let render_mode = request.render_mode.unwrap_or(true);
    
    // Parse markdown to HTML
    let html = parser::markdown::parse_to_html(&request.content);
    
    // Extract links
    let links = parser::links::extract_page_links(&request.content);
    
    // Check for tasks
    let has_tasks = parser::tasks::has_task_markers(&request.content);
    
    let result = MarkdownResult {
        html: if render_mode { html } else { request.content.clone() },
        raw: request.content,
        links,
        has_tasks,
    };
    
    Ok(CommandResponse::success(result))
}

/// Extract page links from content
#[tauri::command]
pub async fn extract_links(
    request: ExtractLinksRequest,
) -> Result<CommandResponse<Vec<String>>, ()> {
    let links = parser::links::extract_page_links(&request.content);
    Ok(CommandResponse::success(links))
}

/// Extract properties from content
#[tauri::command]
pub async fn extract_properties(
    request: ExtractPropertiesRequest,
) -> Result<CommandResponse<Vec<PropertyResult>>, ()> {
    let properties = parser::properties::extract_properties(&request.content);
    
    let results = properties.into_iter().enumerate().map(|(index, (key, value))| {
        PropertyResult {
            key,
            value,
            line_number: index + 1, // Simplified line numbering
        }
    }).collect();
    
    Ok(CommandResponse::success(results))
}

/// Validate content and return warnings/errors
#[tauri::command]
pub async fn validate_content(
    content: String,
) -> Result<CommandResponse<ContentValidationResult>, ()> {
    let mut warnings = Vec::new();
    let errors = Vec::new();
    
    // Check for malformed links
    let malformed_links = parser::links::find_malformed_links(&content);
    for link in malformed_links {
        warnings.push(format!("Malformed link: {}", link));
    }
    
    // Check for malformed properties
    let malformed_properties = parser::properties::find_malformed_properties(&content);
    for prop in malformed_properties {
        warnings.push(format!("Malformed property: {}", prop));
    }
    
    // Check for potential issues
    if content.len() > 10000 {
        warnings.push("Block content is very long (>10k characters)".to_string());
    }
    
    let result = ContentValidationResult {
        is_valid: errors.is_empty(),
        warnings,
        errors,
    };
    
    Ok(CommandResponse::success(result))
}

/// Get content suggestions (autocomplete for links, properties, etc.)
#[tauri::command]
pub async fn get_content_suggestions(
    app: AppHandle,
    content: String,
    cursor_position: usize,
) -> Result<CommandResponse<ContentSuggestions>, ()> {
    let state = app.state::<AppState>();
    let db = state.database.lock().await;
    let mut suggestions = ContentSuggestions {
        page_suggestions: Vec::new(),
        property_suggestions: Vec::new(),
        template_suggestions: Vec::new(),
    };
    
    // Check if cursor is in a link context [[
    if let Some(link_context) = parser::links::get_link_context(&content, cursor_position) {
        // Get matching pages
        match db.search_pages(&link_context, Some(10)).await {
            Ok(pages) => {
                suggestions.page_suggestions = pages.into_iter().map(|p| p.title).collect();
            }
            Err(_) => {
                // Ignore errors for suggestions
            }
        }
    }
    
    // Check if cursor is in a property context
    if let Some(property_context) = parser::properties::get_property_context(&content, cursor_position) {
        match property_context.context_type.as_str() {
            "key" => {
                // Suggest property keys
                if let Ok(keys) = db.get_all_property_keys().await {
                    suggestions.property_suggestions = keys.into_iter()
                        .filter(|k| k.starts_with(&property_context.partial_text))
                        .collect();
                }
            }
            "value" => {
                // Could suggest values for specific keys in the future
            }
            _ => {}
        }
    }
    
    Ok(CommandResponse::success(suggestions))
}

/// Preview content rendering (without saving)
#[tauri::command]
pub async fn preview_content(
    app: AppHandle,
    content: String,
) -> Result<CommandResponse<ContentPreview>, ()> {
    let state = app.state::<AppState>();
    let db = state.database.lock().await;
    // Parse markdown
    let html = parser::markdown::parse_to_html(&content);
    
    // Extract metadata
    let links = parser::links::extract_page_links(&content);
    let properties = parser::properties::extract_properties(&content);
    let task_status = parser::tasks::extract_task_status(&content);
    
    // Check if links exist
    let mut link_statuses = Vec::new();
    for link in &links {
        let exists = match db.get_page_by_path(link).await {
            Ok(Some(_)) => true,
            _ => false,
        };
        link_statuses.push(LinkStatus {
            link: link.clone(),
            exists,
        });
    }
    
    let preview = ContentPreview {
        html,
        properties: properties.into_iter().map(|(k, v)| PropertyResult {
            key: k,
            value: v,
            line_number: 0,
        }).collect(),
        link_statuses,
        task_status: task_status.map(|ts| ts.to_string()),
        word_count: content.split_whitespace().count(),
        character_count: content.len(),
    };
    
    Ok(CommandResponse::success(preview))
}

// ================================
// Response Types
// ================================

#[derive(Debug, Serialize, Deserialize)]
pub struct ContentValidationResult {
    pub is_valid: bool,
    pub warnings: Vec<String>,
    pub errors: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ContentSuggestions {
    pub page_suggestions: Vec<String>,
    pub property_suggestions: Vec<String>,
    pub template_suggestions: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ContentPreview {
    pub html: String,
    pub properties: Vec<PropertyResult>,
    pub link_statuses: Vec<LinkStatus>,
    pub task_status: Option<String>,
    pub word_count: usize,
    pub character_count: usize,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LinkStatus {
    pub link: String,
    pub exists: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PropertyContext {
    pub context_type: String, // "key" or "value"
    pub partial_text: String,
    pub property_key: Option<String>, // For value contexts
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    use crate::AppState;
    use tauri::test;

    async fn create_test_app_state() -> AppState {
        let temp_file = NamedTempFile::new().unwrap();
        let temp_dir = temp_file.path().parent().unwrap().to_path_buf();
        AppState::new(temp_dir).await.unwrap()
    }

    #[tokio::test]
    async fn test_parse_markdown() {
        let request = ParseMarkdownRequest {
            content: "# Header\n\nThis is **bold** text with a [[link]]".to_string(),
            render_mode: Some(true),
        };
        
        let response = parse_markdown(request).await.unwrap();
        assert!(response.success);
        
        let result = response.data.unwrap();
        assert!(result.html.contains("<h1>"));
        assert!(result.html.contains("<strong>"));
        assert_eq!(result.links.len(), 1);
        assert_eq!(result.links[0], "link");
    }

    #[tokio::test]
    async fn test_extract_links() {
        let request = ExtractLinksRequest {
            content: "Here are links: [[Page 1]] and [[Page 2]]".to_string(),
        };
        
        let response = extract_links(request).await.unwrap();
        assert!(response.success);
        
        let links = response.data.unwrap();
        assert_eq!(links.len(), 2);
        assert!(links.contains(&"Page 1".to_string()));
        assert!(links.contains(&"Page 2".to_string()));
    }

    #[tokio::test]
    async fn test_extract_properties() {
        let request = ExtractPropertiesRequest {
            content: "priority:: high\ndue:: tomorrow\ntags:: work, urgent".to_string(),
        };
        
        let response = extract_properties(request).await.unwrap();
        assert!(response.success);
        
        let properties = response.data.unwrap();
        assert_eq!(properties.len(), 3);
        
        let priority_prop = properties.iter().find(|p| p.key == "priority").unwrap();
        assert_eq!(priority_prop.value, "high");
    }

    #[tokio::test]
    async fn test_content_validation() {
        // Test valid content
        let response = validate_content("Valid content here".to_string()).await.unwrap();
        assert!(response.success);
        let result = response.data.unwrap();
        assert!(result.is_valid);
        assert!(result.errors.is_empty());
    }
}
