// Outliner Application - Main Entry Point
// Phase 1: Core Foundation Implementation

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

// External crate imports
use tauri::Manager;
use std::path::PathBuf;
use anyhow::Result;
use tokio::sync::Mutex;

// Internal module imports
mod db;
mod commands;
mod parser;
mod api;
mod utils;

// Re-export core types for convenience
pub use db::{Database, Block, Page};

/// Application state shared across all components
#[derive(Debug)]
pub struct AppState {
    /// Database connection manager
    pub database: Mutex<Database>,
    /// Application data directory path
    pub data_dir: PathBuf,
    /// Local API server handle (for Phase 3)
    pub api_server: Option<api::ServerHandle>,
}

impl AppState {
    /// Initialize application state with database and data directory
    pub async fn new(data_dir: PathBuf) -> Result<Self> {
        // Ensure data directory exists
        tokio::fs::create_dir_all(&data_dir).await?;
        
        // Initialize database with schema
        let db_path = data_dir.join("database.db");
        let database = Database::new(db_path).await?;
        
        // Create initial state
        let state = AppState {
            database: Mutex::new(database),
            data_dir,
            api_server: None,
        };
        
        // Ensure today's page exists
        state.ensure_daily_page_exists().await?;
        
        Ok(state)
    }
    
    /// Ensure today's daily page exists, create if not
    async fn ensure_daily_page_exists(&self) -> Result<()> {
        let today = chrono::Utc::now().format("%Y-%m-%d").to_string();
        let db = self.database.lock().await;
        
        // Check if page exists, create if not
        if db.get_page_by_path(&today).await?.is_none() {
            db.create_page(&today, &today).await?;
        }
        
        Ok(())
    }
}

/// Main application entry point
fn main() {
    // Initialize logging for development
    #[cfg(debug_assertions)]
    {
        tracing_subscriber::fmt()
            .with_env_filter("outliner=debug,tauri=info")
            .init();
    }
    
    // Build and run Tauri application
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .setup(|app| {
            // Initialize application state asynchronously
            let app_handle = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                match initialize_app_state(&app_handle).await {
                    Ok(state) => {
                        app_handle.manage(state);
                        tracing::info!("Application state initialized successfully");
                    }
                    Err(e) => {
                        tracing::error!("Failed to initialize application state: {}", e);
                        std::process::exit(1);
                    }
                }
            });
            
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            // Page management commands
            commands::pages::create_page,
            commands::pages::get_page,
            commands::pages::get_page_by_path,
            commands::pages::update_page,
            commands::pages::delete_page,
            commands::pages::list_pages,
            commands::pages::get_daily_page,
            commands::pages::get_today_page,
            
            // Block management commands
            commands::blocks::create_block,
            commands::blocks::get_block,
            commands::blocks::update_block,
            commands::blocks::delete_block,
            commands::blocks::get_page_blocks,
            commands::blocks::reorder_block,
            commands::blocks::move_block,
            
            // Content processing commands
            commands::content::parse_markdown,
            commands::content::extract_links,
            commands::content::extract_properties,
            
            // Search commands
            commands::search::search_blocks,
            commands::pages::search_pages,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

/// Initialize application state with proper error handling
async fn initialize_app_state(app_handle: &tauri::AppHandle) -> Result<AppState> {
    // Get application data directory
    let data_dir = get_app_data_dir(app_handle)?;
    
    // Initialize state
    AppState::new(data_dir).await
}

/// Get platform-specific application data directory
fn get_app_data_dir(app_handle: &tauri::AppHandle) -> Result<PathBuf> {
    let app_data_dir = app_handle
        .path()
        .app_data_dir()
        .map_err(|e| anyhow::anyhow!("Failed to get app data directory: {}", e))?;
    
    Ok(app_data_dir)
}

/// Global error type for the application
#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("Database error: {0}")]
    Database(#[from] rusqlite::Error),
    
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
    
    #[error("Parse error: {0}")]
    Parse(String),
    
    #[error("Not found: {0}")]
    NotFound(String),
    
    #[error("Invalid input: {0}")]
    InvalidInput(String),
}

/// Result type alias for convenience
pub type AppResult<T> = Result<T, AppError>;

/// Convert AppError to Tauri command error
impl From<AppError> for tauri::Error {
    fn from(err: AppError) -> Self {
        tauri::Error::Anyhow(anyhow::anyhow!(err))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_app_state_initialization() {
        let temp_dir = tempfile::tempdir().unwrap();
        let state = AppState::new(temp_dir.path().to_path_buf()).await.unwrap();
        
        // Test that database is initialized
        assert!(state.data_dir.exists());
        
        // Test that today's page exists
        let today = chrono::Utc::now().format("%Y-%m-%d").to_string();
        let page = state.database.lock().await.get_page_by_path(&today).await.unwrap();
        assert!(page.is_some());
    }
}
