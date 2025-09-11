// Database Layer - Core Data Operations
// Phase 1: SQLite integration with block-centric schema

use rusqlite::{Connection, Result as SqlResult, params, OptionalExtension};
use chrono::{DateTime, Utc};
use serde::{Serialize, Deserialize};
use std::path::Path;
use uuid::Uuid;
use anyhow::Result;
use tokio::sync::Mutex;

pub mod schema;
pub mod blocks;
pub mod pages;
pub mod search;

// Re-export core operations
pub use blocks::*;
pub use pages::*;
pub use search::*;

/// Core database manager with connection pooling and transaction support
#[derive(Debug)]
pub struct Database {
    /// SQLite connection, protected by an async-aware Mutex
    connection: Mutex<Connection>,
}

impl Database {
    /// Create new database instance and initialize schema
    pub async fn new<P: AsRef<Path>>(db_path: P) -> Result<Self> {
        let connection = Connection::open(db_path)?;
        
        // Enable foreign key constraints
        connection.execute("PRAGMA foreign_keys = ON", [])?;
        connection.execute("PRAGMA journal_mode = WAL", [])?;
        
        // Initialize schema
        schema::initialize_schema(&connection)?;
        
        Ok(Database { connection: Mutex::new(connection) })
    }
    
    /// Get a locked database connection (for internal use)
    pub(crate) async fn conn(&self) -> tokio::sync::MutexGuard<'_, Connection> {
        self.connection.lock().await
    }
    
    /// Execute database operations within a transaction
    pub async fn with_transaction<F, T>(&self, f: F) -> Result<T>
    where
        F: FnOnce(&Connection) -> Result<T>,
    {
        let mut conn = self.connection.lock().await;
        let tx = conn.transaction()?;
        match f(&tx) {
            Ok(result) => {
                tx.commit()?;
                Ok(result)
            }
            Err(e) => {
                tx.rollback()?;
                Err(e)
            }
        }
    }
}

/// Core data structures representing the domain model

/// Represents a page - organizational container for blocks
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Page {
    pub id: i32,
    pub title: String,
    pub path: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Page {
    /// Create new page instance (not yet saved to database)
    pub fn new(title: String, path: String) -> Self {
        let now = Utc::now();
        Page {
            id: 0, // Will be assigned by database
            title,
            path,
            created_at: now,
            updated_at: now,
        }
    }
    
    /// Check if this is a daily page (format: YYYY-MM-DD)
    pub fn is_daily_page(&self) -> bool {
        // Simple regex check for YYYY-MM-DD format
        let parts: Vec<&str> = self.path.split('-').collect();
        if parts.len() == 3 {
            parts[0].len() == 4 && parts[1].len() == 2 && parts[2].len() == 2
        } else {
            false
        }
    }
    
    /// Check if this is a special system page (starts with _)
    pub fn is_system_page(&self) -> bool {
        self.path.starts_with('_')
    }
    
    /// Get parent path for hierarchical organization
    pub fn parent_path(&self) -> Option<String> {
        match self.path.rfind('/') {
            Some(pos) => Some(self.path[..pos].to_string()),
            None => None,
        }
    }
}

/// Represents a block - fundamental unit of content
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Block {
    pub id: i32,
    pub uuid: String,
    pub page_id: i32,
    pub parent_id: Option<i32>,
    pub order: i32,
    pub content: String,
    pub is_encrypted: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Block {
    /// Create new block instance (not yet saved to database)
    pub fn new(page_id: i32, parent_id: Option<i32>, content: String, order: i32) -> Self {
        let now = Utc::now();
        Block {
            id: 0, // Will be assigned by database
            uuid: Uuid::new_v4().to_string(),
            page_id,
            parent_id,
            order,
            content,
            is_encrypted: false,
            created_at: now,
            updated_at: now,
        }
    }
    
    /// Check if block is a task (starts with task keywords)
    pub fn is_task(&self) -> bool {
        let content_lower = self.content.trim().to_lowercase();
        content_lower.starts_with("todo ") ||
        content_lower.starts_with("doing ") ||
        content_lower.starts_with("done ") ||
        content_lower.starts_with("waiting ") ||
        content_lower.starts_with("cancelled ")
    }
    
    /// Extract task status if this block is a task
    pub fn task_status(&self) -> Option<TaskStatus> {
        if !self.is_task() {
            return None;
        }
        
        let content_lower = self.content.trim().to_lowercase();
        if content_lower.starts_with("todo ") {
            Some(TaskStatus::Todo)
        } else if content_lower.starts_with("doing ") {
            Some(TaskStatus::Doing)
        } else if content_lower.starts_with("done ") {
            Some(TaskStatus::Done)
        } else if content_lower.starts_with("waiting ") {
            Some(TaskStatus::Waiting)
        } else if content_lower.starts_with("cancelled ") {
            Some(TaskStatus::Cancelled)
        } else {
            None
        }
    }
    
    /// Check if block is a root block (no parent)
    pub fn is_root(&self) -> bool {
        self.parent_id.is_none()
    }
    
    /// Get display content without task status prefix
    pub fn display_content(&self) -> String {
        if let Some(_) = self.task_status() {
            // Remove task status prefix for display
            let parts: Vec<&str> = self.content.splitn(2, ' ').collect();
            if parts.len() > 1 {
                parts[1].to_string()
            } else {
                self.content.clone()
            }
        } else {
            self.content.clone()
        }
    }
}

/// Represents block property key-value pairs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockProperty {
    pub id: i32,
    pub block_id: i32,
    pub key: String,
    pub value: String,
    pub value_type: PropertyType,
}

/// Property value types for future differentiation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PropertyType {
    Text,
    List,
}

impl std::fmt::Display for PropertyType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PropertyType::Text => write!(f, "text"),
            PropertyType::List => write!(f, "list"),
        }
    }
}

/// Task status enumeration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TaskStatus {
    Todo,
    Doing,
    Done,
    Waiting,
    Cancelled,
}

impl std::fmt::Display for TaskStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TaskStatus::Todo => write!(f, "TODO"),
            TaskStatus::Doing => write!(f, "DOING"),
            TaskStatus::Done => write!(f, "DONE"),
            TaskStatus::Waiting => write!(f, "WAITING"),
            TaskStatus::Cancelled => write!(f, "CANCELLED"),
        }
    }
}

/// Link between blocks and pages
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Link {
    pub id: i32,
    pub source_block_id: i32,
    pub target_page_id: i32,
    pub link_text: Option<String>,
}

/// Block hierarchy representation for efficient frontend rendering
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockHierarchy {
    pub block: Block,
    pub children: Vec<BlockHierarchy>,
    pub properties: Vec<BlockProperty>,
}

impl BlockHierarchy {
    /// Create new hierarchy node
    pub fn new(block: Block) -> Self {
        BlockHierarchy {
            block,
            children: Vec::new(),
            properties: Vec::new(),
        }
    }
    
    /// Add child to hierarchy
    pub fn add_child(&mut self, child: BlockHierarchy) {
        self.children.push(child);
    }
    
    /// Sort children by order field
    pub fn sort_children(&mut self) {
        self.children.sort_by_key(|child| child.block.order);
        // Recursively sort all children
        for child in &mut self.children {
            child.sort_children();
        }
    }
    
    /// Get total block count in this hierarchy
    pub fn block_count(&self) -> usize {
        1 + self.children.iter().map(|child| child.block_count()).sum::<usize>()
    }
}

/// Database operation result with rich error information
pub type DbResult<T> = Result<T, crate::AppError>;

/// Constants for block ordering system
pub const ORDER_GAP: i32 = 1000;
pub const ORDER_INITIAL: i32 = ORDER_GAP;

/// Utility functions for ordering calculations
pub mod ordering {
    use super::ORDER_GAP;
    
    /// Calculate order for inserting block at specific position among siblings
    pub fn calculate_insert_order(sibling_orders: &[i32], position: usize) -> i32 {
        if sibling_orders.is_empty() {
            return ORDER_GAP;
        }
        
        if position == 0 {
            // Insert at beginning
            sibling_orders[0] - ORDER_GAP
        } else if position >= sibling_orders.len() {
            // Insert at end
            sibling_orders[sibling_orders.len() - 1] + ORDER_GAP
        } else {
            // Insert between siblings
            let prev_order = sibling_orders[position - 1];
            let next_order = sibling_orders[position];
            
            if next_order - prev_order > 1 {
                (prev_order + next_order) / 2
            } else {
                // No space available, need reordering
                -1 // Signal that reordering is needed
            }
        }
    }
    
    /// Generate new order values with gaps for reordering operation
    pub fn generate_reordered_values(count: usize) -> Vec<i32> {
        (0..count).map(|i| (i as i32 + 1) * ORDER_GAP).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    
    #[tokio::test]
    async fn test_database_initialization() {
        let temp_file = NamedTempFile::new().unwrap();
        let db = Database::new(temp_file.path()).await.unwrap();
        
        // Test that schema tables exist
        let mut stmt = db.conn().await.prepare("SELECT name FROM sqlite_master WHERE type='table'").unwrap();
        let table_names: Vec<String> = stmt.query_map([], |row| {
            Ok(row.get::<_, String>(0)?)
        }).unwrap().collect::<Result<Vec<_>, _>>().unwrap();
        
        assert!(table_names.contains(&"pages".to_string()));
        assert!(table_names.contains(&"blocks".to_string()));
        assert!(table_names.contains(&"block_properties".to_string()));
    }
    
    #[test]
    fn test_block_task_detection() {
        let block = Block::new(1, None, "TODO Write tests".to_string(), 1000);
        assert!(block.is_task());
        assert_eq!(block.task_status(), Some(TaskStatus::Todo));
        assert_eq!(block.display_content(), "Write tests");
        
        let non_task = Block::new(1, None, "Just a regular note".to_string(), 1000);
        assert!(!non_task.is_task());
        assert_eq!(non_task.task_status(), None);
    }
    
    #[test]
    fn test_page_classification() {
        let daily = Page::new("2024-01-15".to_string(), "2024-01-15".to_string());
        assert!(daily.is_daily_page());
        assert!(!daily.is_system_page());
        
        let system = Page::new("Settings".to_string(), "_settings/config".to_string());
        assert!(!system.is_daily_page());
        assert!(system.is_system_page());
        
        let regular = Page::new("Project Notes".to_string(), "projects/website".to_string());
        assert!(!regular.is_daily_page());
        assert!(!regular.is_system_page());
        assert_eq!(regular.parent_path(), Some("projects".to_string()));
    }
}
