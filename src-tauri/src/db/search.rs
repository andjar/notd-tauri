// Search and Query Operations
// Phase 1: Full-text search and basic query functionality

use rusqlite::params;
use serde::{Serialize, Deserialize};

use std::pin::Pin;
use std::future::Future;

use crate::AppError;
use super::{Database, Block, Page, DbResult};
//use super::search_schema::{SearchDocument, SearchResult, GlobalSearchResults, BacklinkResult, ContextualBacklink};

impl Database {
    // ================================
    // Full-Text Search
    // ================================
    
    /// Search blocks using full-text search
    pub async fn search_blocks(&self, query: &str, limit: Option<usize>) -> DbResult<Vec<SearchResult>> {
        if query.trim().is_empty() {
            return Ok(Vec::new());
        }
        
        let limit_clause = limit.map(|l| format!("LIMIT {}", l)).unwrap_or_default();
        let sql = format!(
            r#"
            SELECT 
                b.id, b.uuid, b.page_id, b.parent_id, b."order", b.content, 
                b.is_encrypted, b.created_at, b.updated_at,
                p.title as page_title, p.path as page_path,
                bm25(blocks_fts) as rank
            FROM blocks_fts 
            JOIN blocks b ON blocks_fts.rowid = b.id
            JOIN pages p ON b.page_id = p.id
            WHERE blocks_fts MATCH ?1
            ORDER BY rank
            {}
            "#,
            limit_clause
        );
        
        let conn = self.conn().await;
        let mut stmt = conn.prepare(&sql)?;
        let rows = stmt.query_map(params![query], |row| {
            Ok(SearchResult {
                block: Block {
                    id: row.get(0)?,
                    uuid: row.get(1)?,
                    page_id: row.get(2)?,
                    parent_id: row.get(3)?,
                    order: row.get(4)?,
                    content: row.get(5)?,
                    is_encrypted: row.get(6)?,
                    created_at: row.get(7)?,
                    updated_at: row.get(8)?,
                },
                page_title: row.get(9)?,
                page_path: row.get(10)?,
                rank: row.get(11)?,
                snippet: None, // Will be populated later
            })
        })?;
        
        let mut results = Vec::new();
        for result in rows {
            let mut search_result = result?;
            
            // Generate snippet with highlighted terms
            search_result.snippet = Some(self.generate_snippet(&search_result.block.content, query));
            
            results.push(search_result);
        }
        
        Ok(results)
    }
    
    /// Search pages by title and path
    pub async fn search_pages(&self, query: &str, limit: Option<usize>) -> DbResult<Vec<Page>> {
        if query.trim().is_empty() {
            return Ok(Vec::new());
        }
        
        let limit_clause = limit.map(|l| format!("LIMIT {}", l)).unwrap_or_default();
        let sql = format!(
            r#"
            SELECT id, title, path, created_at, updated_at
            FROM pages
            WHERE title LIKE ?1 OR path LIKE ?1
            ORDER BY 
                CASE 
                    WHEN title = ?2 THEN 1
                    WHEN title LIKE ?3 THEN 2
                    WHEN path = ?2 THEN 3
                    WHEN path LIKE ?3 THEN 4
                    ELSE 5
                END,
                length(title),
                updated_at DESC
            {}
            "#,
            limit_clause
        );
        
        let search_pattern = format!("%{}%", query);
        let exact_match = query.to_string();
        let prefix_match = format!("{}%", query);
        
        let conn = self.conn().await;
        let mut stmt = conn.prepare(&sql)?;
        let rows = stmt.query_map(params![search_pattern, exact_match, prefix_match], |row| {
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
    
    /// Global search across both pages and blocks
    pub async fn global_search(&self, query: &str, limit: Option<usize>) -> DbResult<GlobalSearchResults> {
        let page_limit = limit.map(|l| l / 2).unwrap_or(10);
        let block_limit = limit.map(|l| l - page_limit).unwrap_or(20);
        
        let pages = self.search_pages(query, Some(page_limit)).await?;
        let blocks = self.search_blocks(query, Some(block_limit)).await?;
        let total_results = pages.len() + blocks.len();
        
        Ok(GlobalSearchResults {
            pages,
            blocks,
            query: query.to_string(),
            total_results,
        })
    }
    
    // ================================
    // Backlinks and References
    // ================================
    
    /// Get all blocks that link to a specific page
    pub async fn get_page_backlinks(&self, page_id: i32) -> DbResult<Vec<BacklinkResult>> {
        let conn = self.conn().await;
        let mut stmt = conn.prepare(
            r#"
            SELECT 
                b.id, b.uuid, b.page_id, b.parent_id, b."order", b.content, 
                b.is_encrypted, b.created_at, b.updated_at,
                p.title as source_page_title, p.path as source_page_path,
                l.link_text
            FROM links l
            JOIN blocks b ON l.source_block_id = b.id
            JOIN pages p ON b.page_id = p.id
            WHERE l.target_page_id = ?1
            ORDER BY b.updated_at DESC
            "#
        )?;
        
        let rows = stmt.query_map(params![page_id], |row| {
            Ok(BacklinkResult {
                block: Block {
                    id: row.get(0)?,
                    uuid: row.get(1)?,
                    page_id: row.get(2)?,
                    parent_id: row.get(3)?,
                    order: row.get(4)?,
                    content: row.get(5)?,
                    is_encrypted: row.get(6)?,
                    created_at: row.get(7)?,
                    updated_at: row.get(8)?,
                },
                source_page_title: row.get(9)?,
                source_page_path: row.get(10)?,
                link_text: row.get(11)?,
            })
        })?;
        
        let mut backlinks = Vec::new();
        for backlink in rows {
            backlinks.push(backlink?);
        }
        
        Ok(backlinks)
    }
    
    /// Get contextual backlinks (block with its children hierarchy)
    pub async fn get_contextual_backlinks(&self, page_id: i32) -> DbResult<Vec<ContextualBacklink>> {
        let backlinks = self.get_page_backlinks(page_id).await?;
        let mut contextual_backlinks = Vec::new();
        
        for backlink in backlinks {
            // Get the full hierarchy starting from this block
            let hierarchy = self.get_block_with_children(backlink.block.id).await?;
            
            contextual_backlinks.push(ContextualBacklink {
                source_page_title: backlink.source_page_title,
                source_page_path: backlink.source_page_path,
                link_text: backlink.link_text,
                hierarchy,
            });
        }
        
        Ok(contextual_backlinks)
    }
    
    /// Get block with children (recursive)
    fn get_block_with_children(&self, block_id: i32) -> Pin<Box<dyn Future<Output = DbResult<super::BlockHierarchy>> + Send + '_>> {
        Box::pin(async move {
            // Get the block itself
            let block = self.get_block(block_id).await?
                .ok_or_else(|| AppError::NotFound(format!("Block with id {} not found", block_id)))?;
            
            // Get properties
            let properties = self.get_block_properties(block_id).await?;

            // Get direct children
            let children = self.get_block_children(block_id).await?;
            
            // Recursively get hierarchy for each child
            let mut child_hierarchies = Vec::new();
            for child in children {
                let child_hierarchy = self.get_block_with_children(child.id).await?;
                child_hierarchies.push(child_hierarchy);
            }
            
            Ok(super::BlockHierarchy {
                block,
                children: child_hierarchies,
                properties,
            })
        })
    }
    
    // ================================
    // Tag and Property Queries
    // ================================
    
    /// Find blocks with specific property values
    pub async fn find_blocks_by_property(&self, key: &str, value: Option<&str>) -> DbResult<Vec<Block>> {
        let (sql, params): (String, Vec<&str>) = if let Some(val) = value {
            (
                r#"
                SELECT DISTINCT b.id, b.uuid, b.page_id, b.parent_id, b."order", b.content, 
                       b.is_encrypted, b.created_at, b.updated_at
                FROM blocks b
                JOIN block_properties bp ON b.id = bp.block_id
                WHERE bp.key = ?1 AND bp.value = ?2
                ORDER BY b.updated_at DESC
                "#.to_string(),
                vec![key, val]
            )
        } else {
            (
                r#"
                SELECT DISTINCT b.id, b.uuid, b.page_id, b.parent_id, b."order", b.content, 
                       b.is_encrypted, b.created_at, b.updated_at
                FROM blocks b
                JOIN block_properties bp ON b.id = bp.block_id
                WHERE bp.key = ?1
                ORDER BY b.updated_at DESC
                "#.to_string(),
                vec![key]
            )
        };
        
        let conn = self.conn().await;
        let mut stmt = conn.prepare(&sql)?;
        let rows = stmt.query_map(rusqlite::params_from_iter(params), |row| {
            Ok(Block {
                id: row.get(0)?,
                uuid: row.get(1)?,
                page_id: row.get(2)?,
                parent_id: row.get(3)?,
                order: row.get(4)?,
                content: row.get(5)?,
                is_encrypted: row.get(6)?,
                created_at: row.get(7)?,
                updated_at: row.get(8)?,
            })
        })?;
        
        let mut blocks = Vec::new();
        for block in rows {
            blocks.push(block?);
        }
        
        Ok(blocks)
    }
    
    /// Get all unique property keys used across all blocks
    pub async fn get_all_property_keys(&self) -> DbResult<Vec<String>> {
        let conn = self.conn().await;
        let mut stmt = conn.prepare(
            "SELECT DISTINCT key FROM block_properties ORDER BY key"
        )?;
        
        let rows = stmt.query_map([], |row| {
            Ok(row.get::<_, String>(0)?)
        })?;
        
        let mut keys = Vec::new();
        for key in rows {
            keys.push(key?);
        }
        
        Ok(keys)
    }
    
    /// Get recent blocks (for quick access)
    pub async fn get_recent_blocks(&self, limit: Option<usize>) -> DbResult<Vec<Block>> {
        let limit_clause = limit.map(|l| format!("LIMIT {}", l)).unwrap_or_default();
        let sql = format!(
            r#"
            SELECT id, uuid, page_id, parent_id, "order", content, is_encrypted, created_at, updated_at
            FROM blocks
            WHERE content != ''
            ORDER BY updated_at DESC
            {}
            "#,
            limit_clause
        );
        
        let conn = self.conn().await;
        let mut stmt = conn.prepare(&sql)?;
        let rows = stmt.query_map([], |row| {
            Ok(Block {
                id: row.get(0)?,
                uuid: row.get(1)?,
                page_id: row.get(2)?,
                parent_id: row.get(3)?,
                order: row.get(4)?,
                content: row.get(5)?,
                is_encrypted: row.get(6)?,
                created_at: row.get(7)?,
                updated_at: row.get(8)?,
            })
        })?;
        
        let mut blocks = Vec::new();
        for block in rows {
            blocks.push(block?);
        }
        
        Ok(blocks)
    }
    
    // ================================
    // Helper Methods
    // ================================
    
    /// Generate highlighted snippet for search results
    fn generate_snippet(&self, content: &str, query: &str) -> String {
        let query_lower = query.to_lowercase();
        let content_lower = content.to_lowercase();
        
        if let Some(pos) = content_lower.find(&query_lower) {
            let start = pos.saturating_sub(50);
            let end = (pos + query.len() + 50).min(content.len());
            
            let mut snippet = content[start..end].to_string();
            
            // Add ellipsis if we truncated
            if start > 0 {
                snippet = format!("...{}", snippet);
            }
            if end < content.len() {
                snippet = format!("{}...", snippet);
            }
            
            // Simple highlighting (frontend can do more sophisticated highlighting)
            snippet.replace(&query_lower, &format!("**{}**", query))
        } else {
            // Fallback to first 100 characters
            if content.len() > 100 {
                format!("{}...", &content[..100])
            } else {
                content.to_string()
            }
        }
    }
}

// ================================
// Search Result Types
// ================================

/// Result of a full-text search on blocks
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    pub block: Block,
    pub page_title: String,
    pub page_path: String,
    pub rank: f64,
    pub snippet: Option<String>,
}

/// Global search results combining pages and blocks
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GlobalSearchResults {
    pub pages: Vec<Page>,
    pub blocks: Vec<SearchResult>,
    pub query: String,
    pub total_results: usize,
}

/// Backlink result with source context
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BacklinkResult {
    pub block: Block,
    pub source_page_title: String,
    pub source_page_path: String,
    pub link_text: Option<String>,
}

/// Contextual backlink showing hierarchy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextualBacklink {
    pub source_page_title: String,
    pub source_page_path: String,
    pub link_text: Option<String>,
    pub hierarchy: super::BlockHierarchy,
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    
    #[tokio::test]
    async fn test_block_search() {
        let temp_file = NamedTempFile::new().unwrap();
        let db = Database::new(temp_file.path()).await.unwrap();
        
        // Create test data
        let page = db.create_page("Test Page", "test").await.unwrap();
        db.create_block(page.id, None, "This is a searchable block with important content", None).await.unwrap();
        db.create_block(page.id, None, "Another block with different words", None).await.unwrap();
        db.create_block(page.id, None, "Important information here", None).await.unwrap();
        
        // Test search
        let results = db.search_blocks("important", Some(10)).await.unwrap();
        assert_eq!(results.len(), 2);
        
        // Test that snippets are generated
        assert!(results[0].snippet.is_some());
        let snippet = results[0].snippet.as_ref().unwrap();
        assert!(snippet.contains("**important**"));
    }
    
    #[tokio::test]
    async fn test_page_search() {
        let temp_file = NamedTempFile::new().unwrap();
        let db = Database::new(temp_file.path()).await.unwrap();
        
        // Create test pages
        db.create_page("Project Meeting Notes", "projects/meetings").await.unwrap();
        db.create_page("Project Overview", "projects/overview").await.unwrap();
        db.create_page("Daily Notes", "daily/2024-01-15").await.unwrap();
        
        // Test search
        let results = db.search_pages("project", Some(10)).await.unwrap();
        assert_eq!(results.len(), 2);
        
        // Test exact match priority
        let exact_results = db.search_pages("Project Overview", Some(10)).await.unwrap();
        assert_eq!(exact_results[0].title, "Project Overview");
    }
    
    #[tokio::test]
    async fn test_property_queries() {
        let temp_file = NamedTempFile::new().unwrap();
        let db = Database::new(temp_file.path()).await.unwrap();
        
        let page = db.create_page("Test Page", "test").await.unwrap();
        let block1 = db.create_block(page.id, None, "High priority task", None).await.unwrap();
        let block2 = db.create_block(page.id, None, "Low priority task", None).await.unwrap();
        
        // Set properties
        db.set_block_property(block1.id, "priority", "high", super::PropertyType::Text).await.unwrap();
        db.set_block_property(block2.id, "priority", "low", super::PropertyType::Text).await.unwrap();
        db.set_block_property(block1.id, "tags", "work,urgent", super::PropertyType::List).await.unwrap();
        
        // Test property search
        let high_priority = db.find_blocks_by_property("priority", Some("high")).await.unwrap();
        assert_eq!(high_priority.len(), 1);
        assert_eq!(high_priority[0].id, block1.id);
        
        // Test getting all property keys
        let keys = db.get_all_property_keys().await.unwrap();
        assert!(keys.contains(&"priority".to_string()));
        assert!(keys.contains(&"tags".to_string()));
    }
}
