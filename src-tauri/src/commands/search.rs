// Search Commands
// Phase 1: Tauri commands for search and query operations

use tauri::{AppHandle, Manager};
use serde::{Serialize, Deserialize};

use crate::{AppState, db::{Block, search::{SearchResult, GlobalSearchResults, BacklinkResult, ContextualBacklink}}};
use super::{CommandResponse};

// ================================
// Request Types
// ================================

#[derive(Debug, Serialize, Deserialize)]
pub struct SearchRequest {
    pub query: String,
    pub limit: Option<usize>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PropertySearchRequest {
    pub key: String,
    pub value: Option<String>,
}

// ================================
// Search Commands
// ================================

/// Search blocks using full-text search
#[tauri::command]
pub async fn search_blocks(
    app: AppHandle,
    request: SearchRequest,
) -> Result<CommandResponse<Vec<SearchResult>>, ()> {
    let state = app.state::<AppState>();
    let result = state.database.lock().await.search_blocks(&request.query, request.limit).await;
    Ok(result.into())
}

/// Global search across pages and blocks
#[tauri::command]
pub async fn global_search(
    app: AppHandle,
    request: SearchRequest,
) -> Result<CommandResponse<GlobalSearchResults>, ()> {
    let state = app.state::<AppState>();
    let result = state.database.lock().await.global_search(&request.query, request.limit).await;
    Ok(result.into())
}

/// Quick search with limited results (for autocomplete/suggestions)
#[tauri::command]
pub async fn quick_search(
    app: AppHandle,
    query: String,
) -> Result<CommandResponse<QuickSearchResults>, ()> {
    let state = app.state::<AppState>();
    let db = state.database.lock().await;
    // Limit quick search to small number of results for performance
    let pages_result = db.search_pages(&query, Some(5)).await;
    let blocks_result = db.search_blocks(&query, Some(10)).await;
    
    match (pages_result, blocks_result) {
        (Ok(pages), Ok(blocks)) => {
            Ok(CommandResponse::success(QuickSearchResults {
                pages,
                blocks,
            }))
        }
        (Err(e), _) | (_, Err(e)) => {
            Ok(CommandResponse::error(e.to_string()))
        }
    }
}

// ================================
// Backlink Commands
// ================================

/// Get all blocks that link to a specific page
#[tauri::command]
pub async fn get_page_backlinks(
    app: AppHandle,
    page_id: i32,
) -> Result<CommandResponse<Vec<BacklinkResult>>, ()> {
    let state = app.state::<AppState>();
    let result = state.database.lock().await.get_page_backlinks(page_id).await;
    Ok(result.into())
}

/// Get contextual backlinks (with full hierarchy)
#[tauri::command]
pub async fn get_contextual_backlinks(
    app: AppHandle,
    page_id: i32,
) -> Result<CommandResponse<Vec<ContextualBacklink>>, ()> {
    let state = app.state::<AppState>();
    let result = state.database.lock().await.get_contextual_backlinks(page_id).await;
    Ok(result.into())
}

// ================================
// Property and Tag Search Commands
// ================================

/// Find blocks by property key/value
#[tauri::command]
pub async fn search_blocks_by_property(
    app: AppHandle,
    request: PropertySearchRequest,
) -> Result<CommandResponse<Vec<Block>>, ()> {
    let state = app.state::<AppState>();
    let result = state.database.lock().await.find_blocks_by_property(&request.key, request.value.as_deref()).await;
    Ok(result.into())
}

/// Get all unique property keys
#[tauri::command]
pub async fn get_property_keys(
    app: AppHandle,
) -> Result<CommandResponse<Vec<String>>, ()> {
    let state = app.state::<AppState>();
    let result = state.database.lock().await.get_all_property_keys().await;
    Ok(result.into())
}

/// Find blocks with specific tag (convenience function for tag property)
#[tauri::command]
pub async fn search_blocks_by_tag(
    app: AppHandle,
    tag: String,
) -> Result<CommandResponse<Vec<Block>>, ()> {
    let state = app.state::<AppState>();
    let result = state.database.lock().await.find_blocks_by_property("_tags", Some(&tag)).await;
    Ok(result.into())
}

// ================================
// Recent and Trending Commands
// ================================

/// Get recently updated blocks
#[tauri::command]
pub async fn get_recent_blocks(
    app: AppHandle,
    limit: Option<usize>,
) -> Result<CommandResponse<Vec<Block>>, ()> {
    let state = app.state::<AppState>();
    let result = state.database.lock().await.get_recent_blocks(limit).await;
    Ok(result.into())
}

/// Get blocks by task status
#[tauri::command]
pub async fn search_tasks(
    app: AppHandle,
    status: Option<String>,
    limit: Option<usize>,
) -> Result<CommandResponse<Vec<Block>>, ()> {
    let state = app.state::<AppState>();
    // Search for blocks by task status using property search
    let key = "_task_status";
    let result = state.database.lock().await.find_blocks_by_property(key, status.as_deref()).await;
    let mapped_result = result.map(|mut blocks| {
        // Filter to only actual task blocks
        blocks.retain(|b| b.is_task());
        // Apply limit
        if let Some(limit) = limit {
            blocks.truncate(limit);
        }
        blocks
    });
    Ok(mapped_result.into())
}

/// Get blocks by due date (tasks with due dates)
#[tauri::command]
pub async fn search_tasks_by_due_date(
    app: AppHandle,
    date_filter: String, // "today", "overdue", "this_week", or specific date
) -> Result<CommandResponse<Vec<Block>>, ()> {
    let state = app.state::<AppState>();
    let today = chrono::Utc::now().date_naive();
    
    let date_condition = match date_filter.as_str() {
        "today" => today.format("%Y-%m-%d").to_string(),
        "overdue" => {
            // This would need custom SQL query for proper date comparison
            return Ok(CommandResponse::error("Overdue search not yet implemented".to_string()));
        },
        "this_week" => {
            // This would need custom SQL query for date range
            return Ok(CommandResponse::error("Week search not yet implemented".to_string()));
        },
        date => date.to_string(),
    };
    
    let result = state.database.lock().await.find_blocks_by_property("due", Some(&date_condition)).await;
    let mapped_result = result.map(|mut blocks| {
        // Filter to only task blocks
        blocks.retain(|b| b.is_task());
        blocks
    });
    Ok(mapped_result.into())
}

// ================================
// Advanced Search Commands
// ================================

/// Search with multiple criteria
#[tauri::command]
pub async fn advanced_search(
    app: AppHandle,
    criteria: AdvancedSearchCriteria,
) -> Result<CommandResponse<AdvancedSearchResults>, ()> {
    let state = app.state::<AppState>();
    let db = state.database.lock().await;
    let mut results = AdvancedSearchResults {
        blocks: Vec::new(),
        total_results: 0,
        filters_applied: Vec::new(),
    };
    
    // Start with text search if provided
    let mut candidate_blocks = if let Some(text_query) = criteria.text_query {
        results.filters_applied.push(format!("text: '{}'", text_query));
        match db.search_blocks(&text_query, None).await {
            Ok(search_results) => search_results.into_iter().map(|sr| sr.block).collect(),
            Err(e) => return Ok(CommandResponse::error(e.to_string())),
        }
    } else {
        // Get all blocks if no text search
        match db.get_recent_blocks(None).await {
            Ok(blocks) => blocks,
            Err(e) => return Ok(CommandResponse::error(e.to_string())),
        }
    };
    
    // Apply property filters
    for property_filter in criteria.property_filters {
        results.filters_applied.push(format!("{}:{}", property_filter.key, 
            property_filter.value.clone().unwrap_or_else(|| "*".to_string())));
            
        let matching_blocks = match db.find_blocks_by_property(
            &property_filter.key, 
            property_filter.value.as_deref()
        ).await {
            Ok(blocks) => blocks.into_iter().map(|b| b.id).collect::<std::collections::HashSet<_>>(),
            Err(e) => return Ok(CommandResponse::error(e.to_string())),
        };
        
        // Keep only blocks that match this property filter
        candidate_blocks.retain(|block| matching_blocks.contains(&block.id));
    }
    
    // Apply date range filter
    if let Some(date_range) = criteria.date_range {
        results.filters_applied.push(format!("date: {} to {}", 
            date_range.start_date, date_range.end_date));
            
        candidate_blocks.retain(|block| {
            let block_date = block.created_at.date_naive();
            block_date >= date_range.start_date && block_date <= date_range.end_date
        });
    }
    
    // Apply task filter
    if let Some(task_only) = criteria.tasks_only {
        if task_only {
            results.filters_applied.push("tasks only".to_string());
            candidate_blocks.retain(|block| block.is_task());
        }
    }
    
    // Apply limit and sort
    candidate_blocks.sort_by(|a, b| b.updated_at.cmp(&a.updated_at));
    if let Some(limit) = criteria.limit {
        candidate_blocks.truncate(limit);
    }
    
    results.total_results = candidate_blocks.len();
    results.blocks = candidate_blocks;
    
    Ok(CommandResponse::success(results))
}

// ================================
// Response Types
// ================================

#[derive(Debug, Serialize, Deserialize)]
pub struct QuickSearchResults {
    pub pages: Vec<crate::db::Page>,
    pub blocks: Vec<SearchResult>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AdvancedSearchCriteria {
    pub text_query: Option<String>,
    pub property_filters: Vec<PropertyFilter>,
    pub date_range: Option<DateRange>,
    pub tasks_only: Option<bool>,
    pub limit: Option<usize>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PropertyFilter {
    pub key: String,
    pub value: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DateRange {
    pub start_date: chrono::NaiveDate,
    pub end_date: chrono::NaiveDate,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AdvancedSearchResults {
    pub blocks: Vec<Block>,
    pub total_results: usize,
    pub filters_applied: Vec<String>,
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
    async fn test_search_commands() {
        let app = test::mock_builder().build();
        let app_handle = app.handle().clone();
        app.manage(create_test_app_state().await);
        
        // Create test data
        let state = app.state::<AppState>();
        let db = state.database.lock().await;
        let page = db.create_page("Test Page", "test").await.unwrap();
        let block1 = db.create_block(page.id, None, "Important information here", None).await.unwrap();
        let block2 = db.create_block(page.id, None, "Another block with data", None).await.unwrap();
        
        // Add properties
        db.set_block_property(block1.id, "priority", "high", crate::db::PropertyType::Text).await.unwrap();
        
        // Test block search
        let search_request = SearchRequest {
            query: "important".to_string(),
            limit: Some(10),
        };
        
        let search_response = search_blocks(app_handle.clone(), search_request).await.unwrap();
        assert!(search_response.success);
        let results = search_response.data.unwrap();
        assert!(!results.is_empty());
        assert!(results[0].block.content.to_lowercase().contains("important"));
        
        // Test property search
        let property_request = PropertySearchRequest {
            key: "priority".to_string(),
            value: Some("high".to_string()),
        };
        
        let property_response = search_blocks_by_property(app_handle.clone(), property_request).await.unwrap();
        assert!(property_response.success);
        let property_results = property_response.data.unwrap();
        assert_eq!(property_results.len(), 1);
        assert_eq!(property_results[0].id, block1.id);
    }

    #[tokio::test]
    async fn test_quick_search() {
        let app = test::mock_builder().build();
        let app_handle = app.handle().clone();
        app.manage(create_test_app_state().await);
        
        // Create test data
        let state = app.state::<AppState>();
        let db = state.database.lock().await;
        db.create_page("Search Test Page", "search/test").await.unwrap();
        let page2 = db.create_page("Another Page", "other").await.unwrap();
        db.create_block(page2.id, None, "Block with search term", None).await.unwrap();
        
        // Test quick search
        let response = quick_search(app_handle.clone(), "search".to_string()).await.unwrap();
        assert!(response.success);
        
        let results = response.data.unwrap();
        assert!(!results.pages.is_empty() || !results.blocks.is_empty());
    }
}
