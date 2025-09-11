// Page Database Operations
// Phase 1: CRUD operations for page management and navigation

use rusqlite::{params, OptionalExtension, Result as SqlResult};
use chrono::{DateTime, Utc};
use anyhow::Result;

use super::{Database, Page, DbResult, ORDER_GAP};
use crate::AppError;

impl Database {
    // ================================
    // Page CRUD Operations
    // ================================
    
    /// Create a new page with the given title and path
    pub async fn create_page(&self, title: &str, path: &str) -> DbResult<Page> {
        let now = Utc::now();
        
        self.conn().await.execute(
            r#"
            INSERT INTO pages (title, path, created_at, updated_at)
            VALUES (?1, ?2, ?3, ?4)
            "#,
            params![title, path, now, now],
        ).map_err(|e| match e {
            rusqlite::Error::SqliteFailure(err, _) if err.code == rusqlite::ErrorCode::ConstraintViolation => {
                AppError::InvalidInput(format!("Page with path '{}' already exists", path))
            }
            _ => AppError::Database(e)
        })?;
        
        let page_id = self.conn().await.last_insert_rowid() as i32;
        
        // Create initial empty block for the page
        self.create_initial_block_for_page(page_id).await?;
        
        Ok(Page {
            id: page_id,
            title: title.to_string(),
            path: path.to_string(),
            created_at: now,
            updated_at: now,
        })
    }
    
    /// Get page by ID
    pub async fn get_page(&self, page_id: i32) -> DbResult<Option<Page>> {
        let result = self.conn().await
            .prepare("SELECT id, title, path, created_at, updated_at FROM pages WHERE id = ?1")?
            .query_row(params![page_id], |row| {
                Ok(Page {
                    id: row.get(0)?,
                    title: row.get(1)?,
                    path: row.get(2)?,
                    created_at: row.get(3)?,
                    updated_at: row.get(4)?,
                })
            })
            .optional()?;
        
        Ok(result)
    }
    
    /// Get page by path (unique identifier for pages)
    pub async fn get_page_by_path(&self, path: &str) -> DbResult<Option<Page>> {
        let result = self.conn().await
            .prepare("SELECT id, title, path, created_at, updated_at FROM pages WHERE path = ?1")?
            .query_row(params![path], |row| {
                Ok(Page {
                    id: row.get(0)?,
                    title: row.get(1)?,
                    path: row.get(2)?,
                    created_at: row.get(3)?,
                    updated_at: row.get(4)?,
                })
            })
            .optional()?;
        
        Ok(result)
    }
    
    /// Update page title and path
    pub async fn update_page(&self, page_id: i32, title: Option<&str>, path: Option<&str>) -> DbResult<()> {
        let now = Utc::now();
        
        match (title, path) {
            (Some(title), Some(path)) => {
                self.conn().await.execute(
                    "UPDATE pages SET title = ?1, path = ?2, updated_at = ?3 WHERE id = ?4",
                    params![title, path, now, page_id],
                )?;
            }
            (Some(title), None) => {
                self.conn().await.execute(
                    "UPDATE pages SET title = ?1, updated_at = ?2 WHERE id = ?3",
                    params![title, now, page_id],
                )?;
            }
            (None, Some(path)) => {
                self.conn().await.execute(
                    "UPDATE pages SET path = ?1, updated_at = ?2 WHERE id = ?3",
                    params![path, now, page_id],
                )?;
            }
            (None, None) => {
                return Err(AppError::InvalidInput("No fields to update".to_string()));
            }
        }
        
        Ok(())
    }
    
    /// Delete page and all its blocks
    pub async fn delete_page(&self, page_id: i32) -> DbResult<()> {
        let deleted_rows = self.conn().await.execute(
            "DELETE FROM pages WHERE id = ?1",
            params![page_id],
        )?;
        
        if deleted_rows == 0 {
            return Err(AppError::NotFound(format!("Page with ID {} not found", page_id)));
        }
        
        Ok(())
    }
    
    /// List all pages with optional filtering
    pub async fn list_pages(&self, filter: Option<PageFilter>) -> DbResult<Vec<Page>> {
        let (query, params) = match filter {
            Some(PageFilter::PathPrefix(prefix)) => {
                ("SELECT id, title, path, created_at, updated_at FROM pages WHERE path LIKE ?1 || '%' ORDER BY path", 
                 vec![prefix])
            }
            Some(PageFilter::SystemPages) => {
                ("SELECT id, title, path, created_at, updated_at FROM pages WHERE path LIKE '_%' ORDER BY path", 
                 vec![])
            }
            Some(PageFilter::UserPages) => {
                ("SELECT id, title, path, created_at, updated_at FROM pages WHERE path NOT LIKE '_%' ORDER BY path", 
                 vec![])
            }
            Some(PageFilter::DailyPages) => {
                ("SELECT id, title, path, created_at, updated_at FROM pages WHERE path REGEXP '^[0-9]{4}-[0-9]{2}-[0-9]{2}$' ORDER BY path DESC", 
                 vec![])
            }
            None => {
                ("SELECT id, title, path, created_at, updated_at FROM pages ORDER BY updated_at DESC", 
                 vec![])
            }
        };
        
        let conn = self.conn().await;
        let mut stmt = conn.prepare(query)?;
        let rows = stmt.query_map(rusqlite::params_from_iter(params), |row| {
            Ok(Page {
                id: row.get(0)?,
                title: row.get(1)?,
                path: row.get(2)?,
                created_at: row.get(3)?,
                updated_at: row.get(4)?,
            })
        })?;
        
        let mut pages = Vec::new();
        for page in rows {
            pages.push(page?);
        }
        
        Ok(pages)
    }
    
    // ================================
    // Daily Page Management
    // ================================
    
    /// Get or create daily page for the given date (YYYY-MM-DD format)
    pub async fn get_daily_page(&self, date: &str) -> DbResult<Page> {
        // Validate date format
        if !is_valid_date_format(date) {
            return Err(AppError::InvalidInput(format!("Invalid date format: {}. Expected YYYY-MM-DD", date)));
        }
        
        // Try to get existing page
        if let Some(page) = self.get_page_by_path(date).await? {
            return Ok(page);
        }
        
        // Create new daily page
        self.create_daily_page(date).await
    }
    
    /// Create daily page with template if available
    async fn create_daily_page(&self, date: &str) -> DbResult<Page> {
        // Create the page
        let page = self.create_page(date, date).await?;
        
        // Apply daily template if exists
        if let Err(e) = self.apply_daily_template(&page).await {
            // Log warning but don't fail page creation
            tracing::warn!("Failed to apply daily template to page {}: {}", date, e);
        }
        
        Ok(page)
    }
    
    /// Apply daily page template to a newly created page
    async fn apply_daily_template(&self, page: &Page) -> Result<()> {
        // Get daily template page
        let template_page = match self.get_page_by_path("_templates/daily-note").await? {
            Some(page) => page,
            None => return Ok(()), // No template, skip
        };
        
        // Get template blocks
        let template_blocks = self.get_page_blocks_flat(template_page.id).await?;
        
        // Remove the initial empty block from the target page
        self.remove_initial_empty_blocks(page.id).await?;
        
        // Copy template blocks with variable substitution
        for template_block in template_blocks {
            let content = substitute_template_variables(&template_block.content, page);
            
            self.conn().await.execute(
                r#"
                INSERT INTO blocks (uuid, page_id, parent_id, "order", content)
                VALUES (?1, ?2, ?3, ?4, ?5)
                "#,
                params![
                    uuid::Uuid::new_v4().to_string(),
                    page.id,
                    template_block.parent_id, // This will be problematic for cross-page references
                    template_block.order,
                    content
                ],
            )?;
        }
        
        Ok(())
    }
    
    // ================================
    // Page Navigation and Hierarchy
    // ================================
    
    /// Get all parent pages for a given path (for breadcrumbs)
    pub async fn get_page_ancestors(&self, path: &str) -> DbResult<Vec<Page>> {
        let mut ancestors = Vec::new();
        let mut current_path = path.to_string();
        
        while let Some(parent_path) = get_parent_path(&current_path) {
            if let Some(parent_page) = self.get_page_by_path(&parent_path).await? {
                ancestors.push(parent_page);
                current_path = parent_path;
            } else {
                break;
            }
        }
        
        ancestors.reverse(); // Return in root-to-leaf order
        Ok(ancestors)
    }
    
    /// Get all child pages for a given path
    pub async fn get_page_children(&self, path: &str) -> DbResult<Vec<Page>> {
        let pattern = format!("{}/", path);
        let conn = self.conn().await;
        let mut stmt = conn.prepare(
            "SELECT id, title, path, created_at, updated_at FROM pages WHERE path LIKE ?1 AND path NOT LIKE ?2 ORDER BY path"
        )?;
        
        let rows = stmt.query_map(params![format!("{}%", pattern), format!("{}%/%", pattern)], |row| {
            Ok(Page {
                id: row.get(0)?,
                title: row.get(1)?,
                path: row.get(2)?,
                created_at: row.get(3)?,
                updated_at: row.get(4)?,
            })
        })?;
        
        let mut children = Vec::new();
        for child in rows {
            children.push(child?);
        }
        
        Ok(children)
    }
    
    // ================================
    // Internal Helper Methods
    // ================================
    
    /// Create initial empty block for a new page
    async fn create_initial_block_for_page(&self, page_id: i32) -> DbResult<()> {
        self.conn().await.execute(
            r#"
            INSERT INTO blocks (uuid, page_id, parent_id, "order", content)
            VALUES (?1, ?2, NULL, ?3, '')
            "#,
            params![
                uuid::Uuid::new_v4().to_string(),
                page_id,
                ORDER_GAP
            ],
        )?;
        
        Ok(())
    }
    
    /// Remove initial empty blocks (used when applying templates)
    async fn remove_initial_empty_blocks(&self, page_id: i32) -> DbResult<()> {
        self.conn().await.execute(
            "DELETE FROM blocks WHERE page_id = ?1 AND content = '' AND parent_id IS NULL",
            params![page_id],
        )?;
        
        Ok(())
    }
}

/// Filter options for listing pages
#[derive(Debug, Clone)]
pub enum PageFilter {
    /// Pages under a specific path prefix
    PathPrefix(String),
    /// System pages (starting with _)
    SystemPages,
    /// User pages (not starting with _)
    UserPages,
    /// Daily pages (YYYY-MM-DD format)
    DailyPages,
}

// ================================
// Utility Functions
// ================================

/// Validate date string format (YYYY-MM-DD)
fn is_valid_date_format(date: &str) -> bool {
    if date.len() != 10 {
        return false;
    }
    
    let parts: Vec<&str> = date.split('-').collect();
    if parts.len() != 3 {
        return false;
    }
    
    // Basic validation - more robust validation could use chrono
    parts[0].len() == 4 && parts[0].chars().all(|c| c.is_ascii_digit()) &&
    parts[1].len() == 2 && parts[1].chars().all(|c| c.is_ascii_digit()) &&
    parts[2].len() == 2 && parts[2].chars().all(|c| c.is_ascii_digit())
}

/// Get parent path from hierarchical path
fn get_parent_path(path: &str) -> Option<String> {
    match path.rfind('/') {
        Some(pos) => Some(path[..pos].to_string()),
        None => None,
    }
}

/// Substitute template variables in content
fn substitute_template_variables(content: &str, page: &Page) -> String {
    let today = Utc::now().format("%Y-%m-%d").to_string();
    let time = Utc::now().format("%H:%M").to_string();
    
    content
        .replace("<today>", &today)
        .replace("<time>", &time)
        .replace("<title>", &page.title)
        .replace("<path>", &page.path)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    
    #[tokio::test]
    async fn test_page_crud_operations() {
        let temp_file = NamedTempFile::new().unwrap();
        let db = Database::new(temp_file.path()).await.unwrap();
        
        // Test create
        let page = db.create_page("Test Page", "test/page").await.unwrap();
        assert_eq!(page.title, "Test Page");
        assert_eq!(page.path, "test/page");
        
        // Test get by ID
        let retrieved = db.get_page(page.id).await.unwrap().unwrap();
        assert_eq!(retrieved.title, "Test Page");
        
        // Test get by path
        let retrieved = db.get_page_by_path("test/page").await.unwrap().unwrap();
        assert_eq!(retrieved.id, page.id);
        
        // Test update
        db.update_page(page.id, Some("Updated Title"), None).await.unwrap();
        let updated = db.get_page(page.id).await.unwrap().unwrap();
        assert_eq!(updated.title, "Updated Title");
        assert_eq!(updated.path, "test/page");
        
        // Test delete
        db.delete_page(page.id).await.unwrap();
        let deleted = db.get_page(page.id).await.unwrap();
        assert!(deleted.is_none());
    }
    
    #[tokio::test]
    async fn test_daily_page_creation() {
        let temp_file = NamedTempFile::new().unwrap();
        let db = Database::new(temp_file.path()).await.unwrap();
        
        // Test daily page creation
        let page = db.get_daily_page("2024-01-15").await.unwrap();
        assert_eq!(page.title, "2024-01-15");
        assert_eq!(page.path, "2024-01-15");
        assert!(page.is_daily_page());
        
        // Test that getting the same date returns existing page
        let same_page = db.get_daily_page("2024-01-15").await.unwrap();
        assert_eq!(same_page.id, page.id);
    }
    
    #[tokio::test]
    async fn test_page_hierarchy() {
        let temp_file = NamedTempFile::new().unwrap();
        let db = Database::new(temp_file.path()).await.unwrap();
        
        // Create hierarchical pages
        db.create_page("Root", "projects").await.unwrap();
        db.create_page("Website", "projects/website").await.unwrap();
        db.create_page("Notes", "projects/website/notes").await.unwrap();
        
        // Test ancestors
        let ancestors = db.get_page_ancestors("projects/website/notes").await.unwrap();
        assert_eq!(ancestors.len(), 2);
        assert_eq!(ancestors[0].path, "projects");
        assert_eq!(ancestors[1].path, "projects/website");
        
        // Test children
        let children = db.get_page_children("projects").await.unwrap();
        assert_eq!(children.len(), 1);
        assert_eq!(children[0].path, "projects/website");
    }
    
    #[test]
    fn test_utility_functions() {
        assert!(is_valid_date_format("2024-01-15"));
        assert!(!is_valid_date_format("2024-1-15"));
        assert!(!is_valid_date_format("24-01-15"));
        assert!(!is_valid_date_format("not-a-date"));
        
        assert_eq!(get_parent_path("projects/website/notes"), Some("projects/website".to_string()));
        assert_eq!(get_parent_path("projects"), None);
        assert_eq!(get_parent_path(""), None);
    }
}
