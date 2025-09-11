// Page Management Commands
// Phase 1: Tauri commands for page operations

use tauri::{AppHandle, Manager};
use serde::{Serialize, Deserialize};

use crate::{AppState, db::{Page, PageFilter}};
use super::{CommandResponse, command_wrapper};

// ================================
// Request/Response Types
// ================================

#[derive(Debug, Serialize, Deserialize)]
pub struct CreatePageRequest {
    pub title: String,
    pub path: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdatePageRequest {
    pub title: Option<String>,
    pub path: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ListPagesRequest {
    pub filter: Option<String>, // "system", "user", "daily", or path prefix
    pub limit: Option<usize>,
}

// ================================
// Page Commands
// ================================

/// Create a new page
#[tauri::command]
pub async fn create_page(
    app: AppHandle,
    request: CreatePageRequest,
) -> Result<CommandResponse<Page>, ()> {
    let state = app.state::<AppState>();
    Ok(command_wrapper!(
        state.database.lock().await.create_page(&request.title, &request.path)
    ))
}

/// Get page by ID
#[tauri::command]
pub async fn get_page(
    app: AppHandle,
    page_id: i32,
) -> Result<CommandResponse<Option<Page>>, ()> {
    let state = app.state::<AppState>();
    Ok(command_wrapper!(
        state.database.lock().await.get_page(page_id)
    ))
}

/// Get page by path
#[tauri::command]
pub async fn get_page_by_path(
    app: AppHandle,
    path: String,
) -> Result<CommandResponse<Option<Page>>, ()> {
    let state = app.state::<AppState>();
    Ok(command_wrapper!(
        state.database.lock().await.get_page_by_path(&path)
    ))
}

/// Update page
#[tauri::command]
pub async fn update_page(
    app: AppHandle,
    page_id: i32,
    request: UpdatePageRequest,
) -> Result<CommandResponse<()>, ()> {
    let state = app.state::<AppState>();
    Ok(command_wrapper!(
        state.database.lock().await.update_page(
            page_id,
            request.title.as_deref(),
            request.path.as_deref()
        )
    ))
}

/// Delete page
#[tauri::command]
pub async fn delete_page(
    app: AppHandle,
    page_id: i32,
) -> Result<CommandResponse<()>, ()> {
    let state = app.state::<AppState>();
    Ok(command_wrapper!(
        state.database.lock().await.delete_page(page_id)
    ))
}

/// List pages with optional filtering
#[tauri::command]
pub async fn list_pages(
    app: AppHandle,
    request: Option<ListPagesRequest>,
) -> Result<CommandResponse<Vec<Page>>, ()> {
    let state = app.state::<AppState>();
    let filter = request.as_ref().and_then(|r| {
        r.filter.as_ref().map(|f| match f.as_str() {
            "system" => PageFilter::SystemPages,
            "user" => PageFilter::UserPages,
            "daily" => PageFilter::DailyPages,
            prefix => PageFilter::PathPrefix(prefix.to_string()),
        })
    });
    
    Ok(command_wrapper!(
        state.database.lock().await.list_pages(filter)
    ))
}

/// Get or create daily page for specified date
#[tauri::command]
pub async fn get_daily_page(
    app: AppHandle,
    date: String,
) -> Result<CommandResponse<Page>, ()> {
    let state = app.state::<AppState>();
    Ok(command_wrapper!(
        state.database.lock().await.get_daily_page(&date)
    ))
}

/// Get today's daily page
#[tauri::command]
pub async fn get_today_page(
    app: AppHandle,
) -> Result<CommandResponse<Page>, ()> {
    let state = app.state::<AppState>();
    let today = chrono::Utc::now().format("%Y-%m-%d").to_string();
    Ok(command_wrapper!(
        state.database.lock().await.get_daily_page(&today)
    ))
}

/// Get page ancestors (for breadcrumb navigation)
#[tauri::command]
pub async fn get_page_ancestors(
    app: AppHandle,
    path: String,
) -> Result<CommandResponse<Vec<Page>>, ()> {
    let state = app.state::<AppState>();
    Ok(command_wrapper!(
        state.database.lock().await.get_page_ancestors(&path)
    ))
}

/// Get page children (for tree navigation)
#[tauri::command]
pub async fn get_page_children(
    app: AppHandle,
    path: String,
) -> Result<CommandResponse<Vec<Page>>, ()> {
    let state = app.state::<AppState>();
    Ok(command_wrapper!(
        state.database.lock().await.get_page_children(&path)
    ))
}

/// Search pages by title or path
#[tauri::command]
pub async fn search_pages(
    app: AppHandle,
    query: String,
    limit: Option<usize>,
) -> Result<CommandResponse<Vec<Page>>, ()> {
    let state = app.state::<AppState>();
    Ok(command_wrapper!(
        state.database.lock().await.search_pages(&query, limit)
    ))
}

// ================================
// Page Navigation Helpers
// ================================

/// Get navigation data for current page (ancestors + children)
#[tauri::command]
pub async fn get_page_navigation(
    app: AppHandle,
    path: String,
) -> Result<CommandResponse<PageNavigationData>, ()> {
    let state = app.state::<AppState>();
    let db = state.database.lock().await;
    let ancestors_result = db.get_page_ancestors(&path).await;
    let children_result = db.get_page_children(&path).await;
    
    match (ancestors_result, children_result) {
        (Ok(ancestors), Ok(children)) => {
            Ok(CommandResponse::success(PageNavigationData {
                ancestors,
                children,
            }))
        }
        (Err(e), _) | (_, Err(e)) => {
            Ok(CommandResponse::error(e.to_string()))
        }
    }
}

/// Get recent pages for quick access
#[tauri::command]
pub async fn get_recent_pages(
    app: AppHandle,
    limit: Option<usize>,
) -> Result<CommandResponse<Vec<Page>>, ()> {
    let state = app.state::<AppState>();
    Ok(command_wrapper!(
        state.database.lock().await.list_pages(None)
            .map(|mut pages| {
                // Sort by updated_at and limit
                pages.sort_by(|a, b| b.updated_at.cmp(&a.updated_at));
                if let Some(limit) = limit {
                    pages.truncate(limit);
                }
                pages
            })
    ))
}

// ================================
// Response Types
// ================================

#[derive(Debug, Serialize, Deserialize)]
pub struct PageNavigationData {
    pub ancestors: Vec<Page>,
    pub children: Vec<Page>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    use crate::AppState;
    use std::path::PathBuf;
    use tauri::test;

    async fn create_test_app_state() -> AppState {
        let temp_file = NamedTempFile::new().unwrap();
        let temp_dir = temp_file.path().parent().unwrap().to_path_buf();
        AppState::new(temp_dir).await.unwrap()
    }

    #[tokio::test]
    async fn test_page_commands() {
        let app = test::mock_builder().build();
        let app_handle = app.handle().clone();
        app.manage(create_test_app_state().await);
        
        // Test create page
        let create_request = CreatePageRequest {
            title: "Test Page".to_string(),
            path: "test/page".to_string(),
        };
        
        let response = create_page(app_handle.clone(), create_request).await.unwrap();
        assert!(response.success);
        
        let page = response.data.unwrap();
        assert_eq!(page.title, "Test Page");
        assert_eq!(page.path, "test/page");
        
        // Test get page
        let get_response = get_page(app_handle.clone(), page.id).await.unwrap();
        assert!(get_response.success);
        assert!(get_response.data.unwrap().is_some());
        
        // Test update page
        let update_request = UpdatePageRequest {
            title: Some("Updated Title".to_string()),
            path: None,
        };
        
        let update_response = update_page(app_handle.clone(), page.id, update_request).await.unwrap();
        assert!(update_response.success);
        
        // Test list pages
        let list_response = list_pages(app_handle.clone(), None).await.unwrap();
        assert!(list_response.success);
        assert!(!list_response.data.unwrap().is_empty());
    }

    #[tokio::test]
    async fn test_daily_page_commands() {
        let app = test::mock_builder().build();
        let app_handle = app.handle().clone();
        app.manage(create_test_app_state().await);
        
        // Test get daily page
        let daily_response = get_daily_page(app_handle.clone(), "2024-01-15".to_string()).await.unwrap();
        assert!(daily_response.success);
        
        let daily_page = daily_response.data.unwrap();
        assert_eq!(daily_page.path, "2024-01-15");
        
        // Test get today page
        let today_response = get_today_page(app_handle.clone()).await.unwrap();
        assert!(today_response.success);
        assert!(today_response.data.is_some());
    }
}
