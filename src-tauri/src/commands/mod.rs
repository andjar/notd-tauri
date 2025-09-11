// Tauri Command Handlers
// Phase 1: Expose database operations to frontend via Tauri commands

pub mod pages;
pub mod blocks;
pub mod content;
pub mod search;

// Re-export command functions for easy access
pub use pages::*;
pub use blocks::*;
pub use content::*;
pub use search::*;

use serde::{Serialize, Deserialize};
use crate::{AppState, AppResult};

/// Standard response wrapper for Tauri commands
#[derive(Debug, Serialize, Deserialize)]
pub struct CommandResponse<T> {
    pub success: bool,
    pub data: Option<T>,
    pub error: Option<String>,
}

impl<T> CommandResponse<T> {
    pub fn success(data: T) -> Self {
        CommandResponse {
            success: true,
            data: Some(data),
            error: None,
        }
    }
    
    pub fn error(message: String) -> Self {
        CommandResponse {
            success: false,
            data: None,
            error: Some(message),
        }
    }
}

/// Convert AppResult to CommandResponse
impl<T> From<AppResult<T>> for CommandResponse<T> {
    fn from(result: AppResult<T>) -> Self {
        match result {
            Ok(data) => CommandResponse::success(data),
            Err(err) => CommandResponse::error(err.to_string()),
        }
    }
}

/// Utility macro for wrapping command functions with error handling
macro_rules! command_wrapper {
    ($func:expr) => {
        match $func.await {
            Ok(result) => CommandResponse::success(result),
            Err(err) => {
                tracing::error!("Command error: {}", err);
                CommandResponse::error(err.to_string())
            }
        }
    };
}

pub(crate) use command_wrapper;
