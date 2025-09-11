// Block Management Commands  
// Phase 1: Tauri commands for block operations

use tauri::{AppHandle, Manager};
use serde::{Serialize, Deserialize};

use crate::{AppState, db::{Block, BlockHierarchy, BlockProperty, PropertyType}};
use super::{CommandResponse, command_wrapper};

// ================================
// Request/Response Types
// ================================

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateBlockRequest {
    pub page_id: i32,
    pub parent_id: Option<i32>,
    pub content: String,
    pub position: Option<usize>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateBlockRequest {
    pub content: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MoveBlockRequest {
    pub new_parent_id: Option<i32>,
    pub new_position: Option<usize>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SetPropertyRequest {
    pub key: String,
    pub value: String,
    pub value_type: String, // "text" or "list"
}

// ================================
// Block CRUD Commands
// ================================

/// Create a new block
#[tauri::command]
pub async fn create_block(
    app: AppHandle,
    request: CreateBlockRequest,
) -> Result<CommandResponse<Block>, ()> {
    let state = app.state::<AppState>();
    Ok(command_wrapper!(
        state.database.lock().await.create_block(
            request.page_id,
            request.parent_id,
            &request.content,
            request.position
        )
    ))
}

/// Get block by ID
#[tauri::command]
pub async fn get_block(
    app: AppHandle,
    block_id: i32,
) -> Result<CommandResponse<Option<Block>>, ()> {
    let state = app.state::<AppState>();
    Ok(command_wrapper!(
        state.database.lock().await.get_block(block_id)
    ))
}

/// Get block by UUID
#[tauri::command]
pub async fn get_block_by_uuid(
    app: AppHandle,
    uuid: String,
) -> Result<CommandResponse<Option<Block>>, ()> {
    let state = app.state::<AppState>();
    Ok(command_wrapper!(
        state.database.lock().await.get_block_by_uuid(&uuid)
    ))
}

/// Update block content
#[tauri::command]
pub async fn update_block(
    app: AppHandle,
    block_id: i32,
    request: UpdateBlockRequest,
) -> Result<CommandResponse<()>, ()> {
    let state = app.state::<AppState>();
    Ok(command_wrapper!(
        state.database.lock().await.update_block_content(block_id, &request.content)
    ))
}

/// Update block content by UUID
#[tauri::command]
pub async fn update_block_by_uuid(
    app: AppHandle,
    uuid: String,
    request: UpdateBlockRequest,
) -> Result<CommandResponse<()>, ()> {
    let state = app.state::<AppState>();
    Ok(command_wrapper!(
        state.database.lock().await.update_block_content_by_uuid(&uuid, &request.content)
    ))
}

/// Delete block and all its children
#[tauri::command]
pub async fn delete_block(
    app: AppHandle,
    block_id: i32,
) -> Result<CommandResponse<()>, ()> {
    let state = app.state::<AppState>();
    Ok(command_wrapper!(
        state.database.lock().await.delete_block(block_id)
    ))
}

/// Delete block by UUID
#[tauri::command]
pub async fn delete_block_by_uuid(
    app: AppHandle,
    uuid: String,
) -> Result<CommandResponse<()>, ()> {
    let state = app.state::<AppState>();
    Ok(command_wrapper!(
        state.database.lock().await.delete_block_by_uuid(&uuid)
    ))
}

// ================================
// Block Hierarchy Commands
// ================================

/// Get complete block hierarchy for a page
#[tauri::command]
pub async fn get_page_blocks(
    app: AppHandle,
    page_id: i32,
) -> Result<CommandResponse<Vec<BlockHierarchy>>, ()> {
    let state = app.state::<AppState>();
    Ok(command_wrapper!(
        state.database.lock().await.get_page_blocks(page_id)
    ))
}

/// Get children of a specific block
#[tauri::command]
pub async fn get_block_children(
    app: AppHandle,
    block_id: i32,
) -> Result<CommandResponse<Vec<Block>>, ()> {
    let state = app.state::<AppState>();
    Ok(command_wrapper!(
        state.database.lock().await.get_block_children(block_id)
    ))
}

/// Get all descendants of a block (recursive children)
#[tauri::command]
pub async fn get_block_descendants(
    app: AppHandle,
    block_id: i32,
) -> Result<CommandResponse<Vec<Block>>, ()> {
    let state = app.state::<AppState>();
    Ok(command_wrapper!(
        state.database.lock().await.get_block_descendants(block_id)
    ))
}

// ================================
// Block Movement Commands
// ================================

/// Move block to new parent and/or position
#[tauri::command]
pub async fn move_block(
    app: AppHandle,
    block_id: i32,
    request: MoveBlockRequest,
) -> Result<CommandResponse<()>, ()> {
    let state = app.state::<AppState>();
    Ok(command_wrapper!(
        state.database.lock().await.move_block(
            block_id,
            request.new_parent_id,
            request.new_position
        )
    ))
}

/// Reorder block within current parent
#[tauri::command]
pub async fn reorder_block(
    app: AppHandle,
    block_id: i32,
    new_position: usize,
) -> Result<CommandResponse<()>, ()> {
    let state = app.state::<AppState>();
    Ok(command_wrapper!(
        state.database.lock().await.reorder_block(block_id, new_position)
    ))
}

/// Indent block (make it child of previous sibling)
#[tauri::command]
pub async fn indent_block(
    app: AppHandle,
    block_id: i32,
) -> Result<CommandResponse<()>, ()> {
    let state = app.state::<AppState>();
    Ok(command_wrapper!(
        state.database.lock().await.indent_block(block_id)
    ))
}

/// Outdent block (move to parent's level)
#[tauri::command]
pub async fn outdent_block(
    app: AppHandle,
    block_id: i32,
) -> Result<CommandResponse<()>, ()> {
    let state = app.state::<AppState>();
    Ok(command_wrapper!(
        state.database.lock().await.outdent_block(block_id)
    ))
}

// ================================
// Block Property Commands
// ================================

/// Get all properties for a block
#[tauri::command]
pub async fn get_block_properties(
    app: AppHandle,
    block_id: i32,
) -> Result<CommandResponse<Vec<BlockProperty>>, ()> {
    let state = app.state::<AppState>();
    Ok(command_wrapper!(
        state.database.lock().await.get_block_properties(block_id)
    ))
}

/// Set block property
#[tauri::command]
pub async fn set_block_property(
    app: AppHandle,
    block_id: i32,
    request: SetPropertyRequest,
) -> Result<CommandResponse<()>, ()> {
    let state = app.state::<AppState>();
    let property_type = match request.value_type.as_str() {
        "list" => PropertyType::List,
        _ => PropertyType::Text,
    };
    
    Ok(command_wrapper!(
        state.database.lock().await.set_block_property(
            block_id,
            &request.key,
            &request.value,
            property_type
        )
    ))
}

/// Delete block property
#[tauri::command]
pub async fn delete_block_property(
    app: AppHandle,
    block_id: i32,
    key: String,
) -> Result<CommandResponse<()>, ()> {
    let state = app.state::<AppState>();
    Ok(command_wrapper!(
        state.database.lock().await.delete_block_property(block_id, &key)
    ))
}

// ================================
// Utility Commands
// ================================

/// Create sibling block (after current block)
#[tauri::command]
pub async fn create_sibling_block(
    app: AppHandle,
    current_block_id: i32,
    content: String,
) -> Result<CommandResponse<Block>, ()> {
    let state = app.state::<AppState>();
    // Get current block to determine parent and position
    let db = state.database.lock().await;
    let current_block = match db.get_block(current_block_id).await {
        Ok(Some(block)) => block,
        Ok(None) => {
            return Ok(CommandResponse::error("Current block not found".to_string()));
        }
        Err(e) => {
            return Ok(CommandResponse::error(e.to_string()));
        }
    };
    
    // Get siblings to determine correct position
    let siblings = match db.get_sibling_blocks(current_block.page_id, current_block.parent_id).await {
        Ok(siblings) => siblings,
        Err(e) => {
            return Ok(CommandResponse::error(e.to_string()));
        }
    };
    
    // Find position of current block
    let current_position = siblings.iter().position(|b| b.id == current_block_id);
    let new_position = current_position.map(|pos| pos + 1);
    
    Ok(command_wrapper!(
        db.create_block(
            current_block.page_id,
            current_block.parent_id,
            &content,
            new_position
        )
    ))
}

/// Create child block (indented under current block)
#[tauri::command]
pub async fn create_child_block(
    app: AppHandle,
    parent_block_id: i32,
    content: String,
) -> Result<CommandResponse<Block>, ()> {
    let state = app.state::<AppState>();
    // Get parent block to determine page
    let db = state.database.lock().await;
    let parent_block = match db.get_block(parent_block_id).await {
        Ok(Some(block)) => block,
        Ok(None) => {
            return Ok(CommandResponse::error("Parent block not found".to_string()));
        }
        Err(e) => {
            return Ok(CommandResponse::error(e.to_string()));
        }
    };
    
    Ok(command_wrapper!(
        db.create_block(
            parent_block.page_id,
            Some(parent_block_id),
            &content,
            None // Add at end of children
        )
    ))
}

/// Get block with full context (properties and immediate children)
#[tauri::command]
pub async fn get_block_context(
    app: AppHandle,
    block_id: i32,
) -> Result<CommandResponse<BlockContext>, ()> {
    let state = app.state::<AppState>();
    let db = state.database.lock().await;
    let block_result = db.get_block(block_id).await;
    let properties_result = db.get_block_properties(block_id).await;
    let children_result = db.get_block_children(block_id).await;
    
    match (block_result, properties_result, children_result) {
        (Ok(Some(block)), Ok(properties), Ok(children)) => {
            Ok(CommandResponse::success(BlockContext {
                block,
                properties,
                children,
            }))
        }
        (Ok(None), _, _) => {
            Ok(CommandResponse::error("Block not found".to_string()))
        }
        (Err(e), _, _) | (_, Err(e), _) | (_, _, Err(e)) => {
            Ok(CommandResponse::error(e.to_string()))
        }
    }
}

/// Duplicate block with all its children
#[tauri::command]
pub async fn duplicate_block(
    app: AppHandle,
    block_id: i32,
) -> Result<CommandResponse<Block>, ()> {
    let state = app.state::<AppState>();
    let db = state.database.lock().await;
    // Get original block
    let original_block = match db.get_block(block_id).await {
        Ok(Some(block)) => block,
        Ok(None) => {
            return Ok(CommandResponse::error("Block not found".to_string()));
        }
        Err(e) => {
            return Ok(CommandResponse::error(e.to_string()));
        }
    };
    
    // Create duplicate
    let duplicate = match db.create_block(
        original_block.page_id,
        original_block.parent_id,
        &original_block.content,
        None,
    ).await {
        Ok(block) => block,
        Err(e) => {
            return Ok(CommandResponse::error(e.to_string()));
        }
    };
    
    // Copy properties
    if let Ok(properties) = db.get_block_properties(block_id).await {
        for property in properties {
            let _ = db.set_block_property(
                duplicate.id,
                &property.key,
                &property.value,
                property.value_type,
            ).await;
        }
    }
    
    Ok(CommandResponse::success(duplicate))
}

// ================================
// Response Types
// ================================

#[derive(Debug, Serialize, Deserialize)]
pub struct BlockContext {
    pub block: Block,
    pub properties: Vec<BlockProperty>,
    pub children: Vec<Block>,
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
    async fn test_block_commands() {
        let app = test::mock_builder().build();
        let app_handle = app.handle().clone();
        let state = app.manage(create_test_app_state().await);
        
        // Create a test page first
        let page = state.database.lock().await.create_page("Test Page", "test").await.unwrap();
        
        // Test create block
        let create_request = CreateBlockRequest {
            page_id: page.id,
            parent_id: None,
            content: "Test block content".to_string(),
            position: None,
        };
        
        let response = create_block(app_handle.clone(), create_request).await.unwrap();
        assert!(response.success);
        
        let block = response.data.unwrap();
        assert_eq!(block.content, "Test block content");
        
        // Test update block
        let update_request = UpdateBlockRequest {
            content: "Updated content".to_string(),
        };
        
        let update_response = update_block(app_handle.clone(), block.id, update_request).await.unwrap();
        assert!(update_response.success);
        
        // Test get block
        let get_response = get_block(app_handle.clone(), block.id).await.unwrap();
        assert!(get_response.success);
        let retrieved_block = get_response.data.unwrap().unwrap();
        assert_eq!(retrieved_block.content, "Updated content");
    }

    #[tokio::test]
    async fn test_block_hierarchy_commands() {
        let app = test::mock_builder().build();
        let app_handle = app.handle().clone();
        let state = app.manage(create_test_app_state().await);

        let page = state.database.lock().await.create_page("Test Page", "test").await.unwrap();
        
        // Create parent block
        let parent_request = CreateBlockRequest {
            page_id: page.id,
            parent_id: None,
            content: "Parent block".to_string(),
            position: None,
        };
        let parent_response = create_block(app_handle.clone(), parent_request).await.unwrap();
        let parent_block = parent_response.data.unwrap();
        
        // Create child block
        let child_response = create_child_block(
            app_handle.clone(),
            parent_block.id,
            "Child block".to_string(),
        ).await.unwrap();
        assert!(child_response.success);
        let child_block = child_response.data.unwrap();
        
        // Test get children
        let children_response = get_block_children(app_handle.clone(), parent_block.id).await.unwrap();
        assert!(children_response.success);
        let children = children_response.data.unwrap();
        assert_eq!(children.len(), 1);
        assert_eq!(children[0].id, child_block.id);
        
        // Test get page blocks
        let page_blocks_response = get_page_blocks(app_handle.clone(), page.id).await.unwrap();
        assert!(page_blocks_response.success);
        let hierarchy = page_blocks_response.data.unwrap();
        
        // Find our parent block in the hierarchy
        let parent_hierarchy = hierarchy.iter().find(|h| h.block.id == parent_block.id);
        assert!(parent_hierarchy.is_some());
        assert_eq!(parent_hierarchy.unwrap().children.len(), 1);
    }
}
