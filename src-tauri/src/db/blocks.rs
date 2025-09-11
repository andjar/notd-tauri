// Block Database Operations
// Phase 1: CRUD operations for block management and hierarchy

use rusqlite::{params, OptionalExtension, Result as SqlResult};
use chrono::{DateTime, Utc};
use anyhow::Result;
use std::collections::HashMap;

use super::{Database, Block, BlockHierarchy, BlockProperty, PropertyType, DbResult, ORDER_GAP, ordering};
use crate::AppError;

impl Database {
    // ================================
    // Block CRUD Operations
    // ================================
    
    /// Create a new block with content at specified position
    pub async fn create_block(
        &self, 
        page_id: i32, 
        parent_id: Option<i32>, 
        content: &str, 
        position: Option<usize>
    ) -> DbResult<Block> {
        // Calculate order value for the new block
        let order = self.calculate_block_order(page_id, parent_id, position).await?;
        
        let now = Utc::now();
        let uuid = uuid::Uuid::new_v4().to_string();
        
        self.conn().await.execute(
            r#"
            INSERT INTO blocks (uuid, page_id, parent_id, "order", content, created_at, updated_at)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
            "#,
            params![uuid, page_id, parent_id, order, content, now, now],
        )?;
        
        let block_id = self.conn().await.last_insert_rowid() as i32;
        
        Ok(Block {
            id: block_id,
            uuid,
            page_id,
            parent_id,
            order,
            content: content.to_string(),
            is_encrypted: false,
            created_at: now,
            updated_at: now,
        })
    }
    
    /// Get block by ID
    pub async fn get_block(&self, block_id: i32) -> DbResult<Option<Block>> {
        let result = self.conn().await
            .prepare(
                r#"
                SELECT id, uuid, page_id, parent_id, "order", content, is_encrypted, created_at, updated_at
                FROM blocks WHERE id = ?1
                "#
            )?
            .query_row(params![block_id], |row| {
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
            })
            .optional()?;
        
        Ok(result)
    }
    
    /// Get block by UUID (stable identifier)
    pub async fn get_block_by_uuid(&self, uuid: &str) -> DbResult<Option<Block>> {
        let result = self.conn().await
            .prepare(
                r#"
                SELECT id, uuid, page_id, parent_id, "order", content, is_encrypted, created_at, updated_at
                FROM blocks WHERE uuid = ?1
                "#
            )?
            .query_row(params![uuid], |row| {
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
            })
            .optional()?;
        
        Ok(result)
    }
    
    /// Update block content
    pub async fn update_block_content(&self, block_id: i32, content: &str) -> DbResult<()> {
        let now = Utc::now();
        
        let updated_rows = self.conn().await.execute(
            r#"
            UPDATE blocks 
            SET content = ?1, updated_at = ?2
            WHERE id = ?3
            "#,
            params![content, now, block_id],
        )?;
        
        if updated_rows == 0 {
            return Err(AppError::NotFound(format!("Block with ID {} not found", block_id)));
        }
        
        Ok(())
    }
    
    /// Update block content by UUID
    pub async fn update_block_content_by_uuid(&self, uuid: &str, content: &str) -> DbResult<()> {
        let now = Utc::now();
        
        let updated_rows = self.conn().await.execute(
            r#"
            UPDATE blocks 
            SET content = ?1, updated_at = ?2
            WHERE uuid = ?3
            "#,
            params![content, now, uuid],
        )?;
        
        if updated_rows == 0 {
            return Err(AppError::NotFound(format!("Block with UUID {} not found", uuid)));
        }
        
        Ok(())
    }
    
    /// Delete block and all its descendants
    pub async fn delete_block(&self, block_id: i32) -> DbResult<()> {
        // Get all descendant block IDs first (for proper cleanup)
        let descendants = self.get_block_descendants(block_id).await?;
        
        // Delete the block (CASCADE will handle descendants and related data)
        let deleted_rows = self.conn().await.execute(
            "DELETE FROM blocks WHERE id = ?1",
            params![block_id],
        )?;
        
        if deleted_rows == 0 {
            return Err(AppError::NotFound(format!("Block with ID {} not found", block_id)));
        }
        
        tracing::debug!("Deleted block {} and {} descendants", block_id, descendants.len());
        Ok(())
    }
    
    /// Delete block by UUID
    pub async fn delete_block_by_uuid(&self, uuid: &str) -> DbResult<()> {
        let deleted_rows = self.conn().await.execute(
            "DELETE FROM blocks WHERE uuid = ?1",
            params![uuid],
        )?;
        
        if deleted_rows == 0 {
            return Err(AppError::NotFound(format!("Block with UUID {} not found", uuid)));
        }
        
        Ok(())
    }
    
    // ================================
    // Block Hierarchy Operations
    // ================================
    
    /// Get complete block hierarchy for a page
    pub async fn get_page_blocks(&self, page_id: i32) -> DbResult<Vec<BlockHierarchy>> {
        // Get all blocks for the page
        let blocks = self.get_page_blocks_flat(page_id).await?;
        
        // Build hierarchy
        let hierarchy = self.build_block_hierarchy(blocks).await?;
        
        Ok(hierarchy)
    }
    
    /// Get flat list of all blocks for a page, ordered by hierarchy
    async fn get_page_blocks_flat(&self, page_id: i32) -> DbResult<Vec<Block>> {
        let conn = self.conn().await;
        let mut stmt = conn.prepare(
            r#"
            SELECT id, uuid, page_id, parent_id, "order", content, is_encrypted, created_at, updated_at
            FROM blocks 
            WHERE page_id = ?1
            ORDER BY 
                CASE WHEN parent_id IS NULL THEN 0 ELSE 1 END,
                "order"
            "#
        )?;
        
        let rows = stmt.query_map(params![page_id], |row| {
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
    
    /// Build hierarchical structure from flat block list
    async fn build_block_hierarchy(&self, blocks: Vec<Block>) -> DbResult<Vec<BlockHierarchy>> {
        let mut block_map: HashMap<i32, BlockHierarchy> = HashMap::new();
        let mut roots = Vec::new();
        
        // First pass: create hierarchy nodes
        for block in blocks {
            let hierarchy = BlockHierarchy::new(block.clone());
            
            if block.is_root() {
                roots.push(hierarchy);
            } else {
                block_map.insert(block.id, hierarchy);
            }
        }
        
        // Second pass: build parent-child relationships
        let mut blocks_to_process: Vec<_> = block_map.into_iter().collect();
        blocks_to_process.sort_by(|a, b| a.0.cmp(&b.0)); // Process in ID order for consistency
        
        for (block_id, mut hierarchy) in blocks_to_process {
            if let Some(parent_id) = hierarchy.block.parent_id {
                // Find parent and add this as child
                if let Some(parent) = self.find_parent_in_hierarchy(&mut roots, parent_id) {
                    parent.add_child(hierarchy);
                } else {
                    // Parent not found in roots, this might be an orphaned block
                    tracing::warn!("Block {} has parent {} that wasn't found in roots", block_id, parent_id);
                    roots.push(hierarchy); // Add as root for now
                }
            } else {
                roots.push(hierarchy);
            }
        }
        
        // Sort all children by order
        for root in &mut roots {
            root.sort_children();
        }
        
        roots.sort_by_key(|root| root.block.order);
        Ok(roots)
    }
    
    /// Find parent node in block hierarchy recursively
    fn find_parent_in_hierarchy<'a>(&self, hierarchy: &'a mut [BlockHierarchy], parent_id: i32) -> Option<&'a mut BlockHierarchy> {
        for node in hierarchy {
            if node.block.id == parent_id {
                return Some(node);
            }
            if let Some(found) = self.find_parent_in_hierarchy(&mut node.children, parent_id) {
                return Some(found);
            }
        }
        None
    }
    
    /// Get all child blocks of a parent block
    pub async fn get_block_children(&self, parent_id: i32) -> DbResult<Vec<Block>> {
        let conn = self.conn().await;
        let mut stmt = conn.prepare(
            r#"
            SELECT id, uuid, page_id, parent_id, "order", content, is_encrypted, created_at, updated_at
            FROM blocks 
            WHERE parent_id = ?1
            ORDER BY "order"
            "#
        )?;
        
        let rows = stmt.query_map(params![parent_id], |row| {
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
        
        let mut children = Vec::new();
        for child in rows {
            children.push(child?);
        }
        
        Ok(children)
    }
    
    /// Get all descendant blocks (recursive children)
    pub async fn get_block_descendants(&self, block_id: i32) -> DbResult<Vec<Block>> {
        // Use recursive CTE to get all descendants
        let conn = self.conn().await;
        let mut stmt = conn.prepare(
            r#"
            WITH RECURSIVE descendants AS (
                SELECT id, uuid, page_id, parent_id, "order", content, is_encrypted, created_at, updated_at
                FROM blocks 
                WHERE parent_id = ?1
                
                UNION ALL
                
                SELECT b.id, b.uuid, b.page_id, b.parent_id, b."order", b.content, b.is_encrypted, b.created_at, b.updated_at
                FROM blocks b
                INNER JOIN descendants d ON b.parent_id = d.id
            )
            SELECT * FROM descendants ORDER BY "order"
            "#
        )?;
        
        let rows = stmt.query_map(params![block_id], |row| {
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
        
        let mut descendants = Vec::new();
        for descendant in rows {
            descendants.push(descendant?);
        }
        
        Ok(descendants)
    }
    
    // ================================
    // Block Movement and Ordering
    // ================================
    
    /// Move block to new position within same parent or different parent
    pub async fn move_block(
        &self,
        block_id: i32,
        new_parent_id: Option<i32>,
        new_position: Option<usize>
    ) -> DbResult<()> {
        // Get current block
        let block = self.get_block(block_id).await?
            .ok_or_else(|| AppError::NotFound(format!("Block {} not found", block_id)))?;
        
        // If moving to different parent, validate the target parent
        if new_parent_id != block.parent_id {
            if let Some(parent_id) = new_parent_id {
                self.validate_parent_move(block_id, parent_id).await?;
            }
        }
        
        // Calculate new order
        let new_order = self.calculate_block_order(block.page_id, new_parent_id, new_position).await?;
        
        // Update block
        let now = Utc::now();
        self.conn().await.execute(
            r#"
            UPDATE blocks 
            SET parent_id = ?1, "order" = ?2, updated_at = ?3
            WHERE id = ?4
            "#,
            params![new_parent_id, new_order, now, block_id],
        )?;
        
        // Update all descendants to new page if parent changed pages
        if let Some(new_parent_id) = new_parent_id {
            let new_parent = self.get_block(new_parent_id).await?
                .ok_or_else(|| AppError::NotFound(format!("Parent block {} not found", new_parent_id)))?;
            
            if new_parent.page_id != block.page_id {
                self.move_descendants_to_page(block_id, new_parent.page_id).await?;
            }
        }
        
        Ok(())
    }
    
    /// Reorder block within its current parent to new position
    pub async fn reorder_block(&self, block_id: i32, new_position: usize) -> DbResult<()> {
        let block = self.get_block(block_id).await?
            .ok_or_else(|| AppError::NotFound(format!("Block {} not found", block_id)))?;
        
        self.move_block(block_id, block.parent_id, Some(new_position)).await
    }
    
    /// Indent block (make it child of previous sibling)
    pub async fn indent_block(&self, block_id: i32) -> DbResult<()> {
        let block = self.get_block(block_id).await?
            .ok_or_else(|| AppError::NotFound(format!("Block {} not found", block_id)))?;
        
        // Find previous sibling to use as new parent
        let siblings = self.get_sibling_blocks(block.page_id, block.parent_id).await?;
        
        let current_index = siblings.iter().position(|b| b.id == block_id)
            .ok_or_else(|| AppError::NotFound("Block not found in siblings".to_string()))?;
        
        if current_index == 0 {
            return Err(AppError::InvalidInput("Cannot indent first block".to_string()));
        }
        
        let new_parent_id = siblings[current_index - 1].id;
        
        // Move to end of new parent's children
        self.move_block(block_id, Some(new_parent_id), None).await
    }
    
    /// Outdent block (move it to same level as parent)
    pub async fn outdent_block(&self, block_id: i32) -> DbResult<()> {
        let block = self.get_block(block_id).await?
            .ok_or_else(|| AppError::NotFound(format!("Block {} not found", block_id)))?;
        
        let parent_id = block.parent_id
            .ok_or_else(|| AppError::InvalidInput("Cannot outdent root block".to_string()))?;
        
        let parent = self.get_block(parent_id).await?
            .ok_or_else(|| AppError::NotFound(format!("Parent block {} not found", parent_id)))?;
        
        // Move to after parent at grandparent level
        let grandparent_siblings = self.get_sibling_blocks(parent.page_id, parent.parent_id).await?;
        let parent_index = grandparent_siblings.iter().position(|b| b.id == parent_id)
            .ok_or_else(|| AppError::NotFound("Parent not found in siblings".to_string()))?;
        
        self.move_block(block_id, parent.parent_id, Some(parent_index + 1)).await
    }
    
    // ================================
    // Block Properties
    // ================================
    
    /// Get all properties for a block
    pub async fn get_block_properties(&self, block_id: i32) -> DbResult<Vec<BlockProperty>> {
        let conn = self.conn().await;
        let mut stmt = conn.prepare(
            "SELECT id, block_id, key, value, value_type FROM block_properties WHERE block_id = ?1 ORDER BY key"
        )?;
        
        let rows = stmt.query_map(params![block_id], |row| {
            Ok(BlockProperty {
                id: row.get(0)?,
                block_id: row.get(1)?,
                key: row.get(2)?,
                value: row.get(3)?,
                value_type: match row.get::<_, String>(4)?.as_str() {
                    "list" => PropertyType::List,
                    _ => PropertyType::Text,
                },
            })
        })?;
        
        let mut properties = Vec::new();
        for property in rows {
            properties.push(property?);
        }
        
        Ok(properties)
    }
    
    /// Set block property (insert or update)
    pub async fn set_block_property(
        &self,
        block_id: i32,
        key: &str,
        value: &str,
        value_type: PropertyType
    ) -> DbResult<()> {
        self.conn().await.execute(
            r#"
            INSERT INTO block_properties (block_id, key, value, value_type)
            VALUES (?1, ?2, ?3, ?4)
            ON CONFLICT (block_id, key) DO UPDATE SET
                value = excluded.value,
                value_type = excluded.value_type
            "#,
            params![block_id, key, value, value_type.to_string()],
        )?;
        
        Ok(())
    }
    
    /// Delete block property
    pub async fn delete_block_property(&self, block_id: i32, key: &str) -> DbResult<()> {
        self.conn().await.execute(
            "DELETE FROM block_properties WHERE block_id = ?1 AND key = ?2",
            params![block_id, key],
        )?;
        
        Ok(())
    }
    
    // ================================
    // Helper Methods
    // ================================
    
    /// Calculate appropriate order value for new block position
    async fn calculate_block_order(
        &self,
        page_id: i32,
        parent_id: Option<i32>,
        position: Option<usize>
    ) -> DbResult<i32> {
        // Get sibling order values
        let sibling_orders = self.get_sibling_order_values(page_id, parent_id).await?;
        
        let target_position = position.unwrap_or(sibling_orders.len());
        let calculated_order = ordering::calculate_insert_order(&sibling_orders, target_position);
        
        if calculated_order == -1 {
            // Need to reorder siblings to make space
            self.reorder_siblings_with_gaps(page_id, parent_id).await?;
            
            // Recalculate after reordering
            let new_orders = self.get_sibling_order_values(page_id, parent_id).await?;
            Ok(ordering::calculate_insert_order(&new_orders, target_position))
        } else {
            Ok(calculated_order)
        }
    }
    
    /// Get order values of all sibling blocks
    async fn get_sibling_order_values(&self, page_id: i32, parent_id: Option<i32>) -> DbResult<Vec<i32>> {
        let conn = self.conn().await;
        let mut stmt = if parent_id.is_some() {
            conn.prepare(
                r#"SELECT "order" FROM blocks WHERE page_id = ?1 AND parent_id = ?2 ORDER BY "order""#
            )?
        } else {
            conn.prepare(
                r#"SELECT "order" FROM blocks WHERE page_id = ?1 AND parent_id IS NULL ORDER BY "order""#
            )?
        };
        
        let mut rows = if let Some(pid) = parent_id {
            stmt.query_map(params![page_id, pid], |row| row.get(0))
        } else {
            stmt.query_map(params![page_id], |row| row.get(0))
        }?;
        
        let mut orders = Vec::new();
        for order_result in rows {
            orders.push(order_result?);
        }
        
        Ok(orders)
    }
    
    /// Get all sibling blocks (same parent and page)
    async fn get_sibling_blocks(&self, page_id: i32, parent_id: Option<i32>) -> DbResult<Vec<Block>> {
        let conn = self.conn().await;
        let mut stmt = if parent_id.is_some() {
            conn.prepare(
                r#"
                SELECT id, uuid, page_id, parent_id, "order", content, is_encrypted, created_at, updated_at
                FROM blocks 
                WHERE page_id = ?1 AND parent_id = ?2 
                ORDER BY "order"
                "#
            )?
        } else {
            conn.prepare(
                r#"
                SELECT id, uuid, page_id, parent_id, "order", content, is_encrypted, created_at, updated_at
                FROM blocks 
                WHERE page_id = ?1 AND parent_id IS NULL 
                ORDER BY "order"
                "#
            )?
        };
        
        let map_row = |row: &rusqlite::Row<'_>| {
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
        };

        let rows = if let Some(pid) = parent_id {
            stmt.query_map(params![page_id, pid], map_row)
        } else {
            stmt.query_map(params![page_id], map_row)
        }?;
        
        let mut siblings = Vec::new();
        for block in rows {
            siblings.push(block?);
        }
        
        Ok(siblings)
    }
    
    /// Reorder all siblings with proper gaps
    async fn reorder_siblings_with_gaps(&self, page_id: i32, parent_id: Option<i32>) -> DbResult<()> {
        let siblings = self.get_sibling_blocks(page_id, parent_id).await?;
        let new_orders = ordering::generate_reordered_values(siblings.len());
        
        // Update each sibling with new order
        for (block, new_order) in siblings.iter().zip(new_orders.iter()) {
            self.conn().await.execute(
                r#"UPDATE blocks SET "order" = ?1 WHERE id = ?2"#,
                params![new_order, block.id],
            )?;
        }
        
        Ok(())
    }
    
    /// Validate that moving a block to new parent won't create cycles
    async fn validate_parent_move(&self, block_id: i32, new_parent_id: i32) -> DbResult<()> {
        // Check that new_parent_id is not a descendant of block_id
        let descendants = self.get_block_descendants(block_id).await?;
        
        for descendant in descendants {
            if descendant.id == new_parent_id {
                return Err(AppError::InvalidInput("Cannot move block under its own descendant".to_string()));
            }
        }
        
        // Check that new_parent_id exists
        if self.get_block(new_parent_id).await?.is_none() {
            return Err(AppError::NotFound(format!("Target parent block {} not found", new_parent_id)));
        }
        
        Ok(())
    }
    
    /// Move all descendants of a block to a new page (used when block changes pages)
    async fn move_descendants_to_page(&self, block_id: i32, new_page_id: i32) -> DbResult<()> {
        self.conn().await.execute(
            r#"
            WITH RECURSIVE descendants AS (
                SELECT id FROM blocks WHERE parent_id = ?1
                UNION ALL
                SELECT b.id FROM blocks b
                INNER JOIN descendants d ON b.parent_id = d.id
            )
            UPDATE blocks SET page_id = ?2 WHERE id IN (SELECT id FROM descendants)
            "#,
            params![block_id, new_page_id],
        )?;
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    
    #[tokio::test]
    async fn test_block_creation_and_retrieval() {
        let temp_file = NamedTempFile::new().unwrap();
        let db = Database::new(temp_file.path()).await.unwrap();
        
        // Create a test page first
        let page = db.create_page("Test Page", "test").await.unwrap();
        
        // Create a block
        let block = db.create_block(page.id, None, "Test content", None).await.unwrap();
        assert_eq!(block.content, "Test content");
        assert_eq!(block.page_id, page.id);
        assert!(block.parent_id.is_none());
        
        // Retrieve by ID
        let retrieved = db.get_block(block.id).await.unwrap().unwrap();
        assert_eq!(retrieved.content, "Test content");
        
        // Retrieve by UUID
        let retrieved_by_uuid = db.get_block_by_uuid(&block.uuid).await.unwrap().unwrap();
        assert_eq!(retrieved_by_uuid.id, block.id);
    }
    
    #[tokio::test]
    async fn test_block_hierarchy() {
        let temp_file = NamedTempFile::new().unwrap();
        let db = Database::new(temp_file.path()).await.unwrap();
        
        let page = db.create_page("Test Page", "test").await.unwrap();
        
        // Create root block
        let root = db.create_block(page.id, None, "Root block", None).await.unwrap();
        
        // Create child blocks
        let child1 = db.create_block(page.id, Some(root.id), "Child 1", None).await.unwrap();
        let child2 = db.create_block(page.id, Some(root.id), "Child 2", None).await.unwrap();
        
        // Create grandchild
        let grandchild = db.create_block(page.id, Some(child1.id), "Grandchild", None).await.unwrap();
        
        // Test hierarchy retrieval
        let hierarchy = db.get_page_blocks(page.id).await.unwrap();
        
        // Should have one root (plus the initial empty block)
        assert!(hierarchy.len() >= 1);
        
        let root_hierarchy = hierarchy.iter().find(|h| h.block.id == root.id).unwrap();
        assert_eq!(root_hierarchy.children.len(), 2);
        
        let child1_hierarchy = root_hierarchy.children.iter().find(|h| h.block.id == child1.id).unwrap();
        assert_eq!(child1_hierarchy.children.len(), 1);
        assert_eq!(child1_hierarchy.children[0].block.id, grandchild.id);
    }
    
    #[tokio::test]
    async fn test_block_ordering() {
        let temp_file = NamedTempFile::new().unwrap();
        let db = Database::new(temp_file.path()).await.unwrap();
        
        let page = db.create_page("Test Page", "test").await.unwrap();
        
        // Create blocks in specific positions
        let block1 = db.create_block(page.id, None, "First", Some(0)).await.unwrap();
        let block2 = db.create_block(page.id, None, "Second", Some(1)).await.unwrap();
        let block_middle = db.create_block(page.id, None, "Middle", Some(1)).await.unwrap();
        
        // Check that ordering is correct
        let blocks = db.get_page_blocks_flat(page.id).await.unwrap();
        let user_blocks: Vec<_> = blocks.into_iter().filter(|b| !b.content.is_empty()).collect();
        
        assert_eq!(user_blocks[0].content, "First");
        assert_eq!(user_blocks[1].content, "Middle");
        assert_eq!(user_blocks[2].content, "Second");
    }
    
    #[tokio::test]
    async fn test_block_movement() {
        let temp_file = NamedTempFile::new().unwrap();
        let db = Database::new(temp_file.path()).await.unwrap();
        
        let page = db.create_page("Test Page", "test").await.unwrap();
        
        // Create hierarchy
        let parent1 = db.create_block(page.id, None, "Parent 1", None).await.unwrap();
        let parent2 = db.create_block(page.id, None, "Parent 2", None).await.unwrap();
        let child = db.create_block(page.id, Some(parent1.id), "Child", None).await.unwrap();
        
        // Move child from parent1 to parent2
        db.move_block(child.id, Some(parent2.id), Some(0)).await.unwrap();
        
        // Verify movement
        let updated_child = db.get_block(child.id).await.unwrap().unwrap();
        assert_eq!(updated_child.parent_id, Some(parent2.id));
        
        // Verify hierarchy
        let parent2_children = db.get_block_children(parent2.id).await.unwrap();
        assert_eq!(parent2_children.len(), 1);
        assert_eq!(parent2_children[0].id, child.id);
        
        let parent1_children = db.get_block_children(parent1.id).await.unwrap();
        assert_eq!(parent1_children.len(), 0);
    }
    
    #[tokio::test]
    async fn test_block_properties() {
        let temp_file = NamedTempFile::new().unwrap();
        let db = Database::new(temp_file.path()).await.unwrap();
        
        let page = db.create_page("Test Page", "test").await.unwrap();
        let block = db.create_block(page.id, None, "Test block", None).await.unwrap();
        
        // Set properties
        db.set_block_property(block.id, "priority", "high", PropertyType::Text).await.unwrap();
        db.set_block_property(block.id, "tags", "work,urgent", PropertyType::List).await.unwrap();
        
        // Get properties
        let properties = db.get_block_properties(block.id).await.unwrap();
        assert_eq!(properties.len(), 2);
        
        let priority_prop = properties.iter().find(|p| p.key == "priority").unwrap();
        assert_eq!(priority_prop.value, "high");
        
        // Delete property
        db.delete_block_property(block.id, "priority").await.unwrap();
        let updated_properties = db.get_block_properties(block.id).await.unwrap();
        assert_eq!(updated_properties.len(), 1);
    }
}
