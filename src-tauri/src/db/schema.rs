// Database Schema Definition and Migration System
// Phase 1: Complete SQLite schema with proper indexes and constraints

use rusqlite::{Connection, Result as SqlResult};
use anyhow::Result;

/// Current schema version for migration tracking
pub const CURRENT_SCHEMA_VERSION: i32 = 1;

/// Initialize database schema and run migrations
pub fn initialize_schema(conn: &Connection) -> Result<()> {
    // Create or update schema
    let current_version = get_schema_version(conn)?;
    
    if current_version == 0 {
        create_initial_schema(conn)?;
        set_schema_version(conn, CURRENT_SCHEMA_VERSION)?;
    } else if current_version < CURRENT_SCHEMA_VERSION {
        run_migrations(conn, current_version)?;
        set_schema_version(conn, CURRENT_SCHEMA_VERSION)?;
    }
    
    Ok(())
}

/// Create the initial database schema (Version 1)
fn create_initial_schema(conn: &Connection) -> SqlResult<()> {
    // Create pages table - organizational containers for blocks
    conn.execute(
        r#"
        CREATE TABLE IF NOT EXISTS pages (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            title TEXT NOT NULL UNIQUE,
            path TEXT NOT NULL UNIQUE,
            created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
            updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
        )
        "#,
        [],
    )?;
    
    // Create blocks table - fundamental content units
    conn.execute(
        r#"
        CREATE TABLE IF NOT EXISTS blocks (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            uuid TEXT NOT NULL UNIQUE,
            page_id INTEGER NOT NULL REFERENCES pages(id) ON DELETE CASCADE,
            parent_id INTEGER REFERENCES blocks(id) ON DELETE CASCADE,
            "order" INTEGER NOT NULL,
            content TEXT NOT NULL,
            content_encrypted TEXT,
            is_encrypted BOOLEAN DEFAULT FALSE,
            created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
            updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
        )
        "#,
        [],
    )?;
    
    // Create block properties table - flexible key-value metadata
    conn.execute(
        r#"
        CREATE TABLE IF NOT EXISTS block_properties (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            block_id INTEGER NOT NULL REFERENCES blocks(id) ON DELETE CASCADE,
            key TEXT NOT NULL,
            value TEXT NOT NULL,
            value_type TEXT DEFAULT 'text',
            UNIQUE(block_id, key)
        )
        "#,
        [],
    )?;
    
    // Create links table - connects blocks to pages
    conn.execute(
        r#"
        CREATE TABLE IF NOT EXISTS links (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            source_block_id INTEGER NOT NULL REFERENCES blocks(id) ON DELETE CASCADE,
            target_page_id INTEGER NOT NULL REFERENCES pages(id) ON DELETE CASCADE,
            link_text TEXT,
            UNIQUE(source_block_id, target_page_id, link_text)
        )
        "#,
        [],
    )?;
    
    // Create tasks table - indexed view of task blocks
    conn.execute(
        r#"
        CREATE TABLE IF NOT EXISTS tasks (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            block_id INTEGER NOT NULL UNIQUE REFERENCES blocks(id) ON DELETE CASCADE,
            status TEXT NOT NULL DEFAULT 'TODO',
            priority TEXT,
            due_date DATETIME,
            created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
            updated_at DATETIME DEFAULT CURRENT_TIMESTAMP,
            completed_at DATETIME
        )
        "#,
        [],
    )?;
    
    // Create task state changes table - time tracking for tasks
    conn.execute(
        r#"
        CREATE TABLE IF NOT EXISTS task_state_changes (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            task_id INTEGER NOT NULL REFERENCES tasks(id) ON DELETE CASCADE,
            old_status TEXT,
            new_status TEXT NOT NULL,
            changed_at DATETIME DEFAULT CURRENT_TIMESTAMP
        )
        "#,
        [],
    )?;
    
    // Create attachments table - file attachments linked to pages
    conn.execute(
        r#"
        CREATE TABLE IF NOT EXISTS attachments (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            page_id INTEGER NOT NULL REFERENCES pages(id) ON DELETE CASCADE,
            filename TEXT NOT NULL,
            file_path TEXT NOT NULL,
            file_size INTEGER,
            mime_type TEXT,
            created_at DATETIME DEFAULT CURRENT_TIMESTAMP
        )
        "#,
        [],
    )?;
    
    // Create full-text search virtual table
    conn.execute(
        r#"
        CREATE VIRTUAL TABLE IF NOT EXISTS blocks_fts USING fts5(
            content,
            content='blocks',
            content_rowid='id'
        )
        "#,
        [],
    )?;
    
    // Create performance indexes
    create_indexes(conn)?;
    
    // Create triggers for maintaining FTS index
    create_triggers(conn)?;
    
    // Insert initial system pages and templates
    create_initial_data(conn)?;
    
    Ok(())
}

/// Create performance-critical database indexes
fn create_indexes(conn: &Connection) -> SqlResult<()> {
    // Primary lookup indexes
    conn.execute("CREATE INDEX IF NOT EXISTS idx_blocks_parent ON blocks(parent_id)", [])?;
    conn.execute("CREATE INDEX IF NOT EXISTS idx_blocks_page ON blocks(page_id)", [])?;
    conn.execute("CREATE INDEX IF NOT EXISTS idx_blocks_uuid ON blocks(uuid)", [])?;
    
    // Ordering and hierarchy indexes
    conn.execute("CREATE INDEX IF NOT EXISTS idx_blocks_page_order ON blocks(page_id, parent_id, \"order\")", [])?;
    conn.execute("CREATE INDEX IF NOT EXISTS idx_blocks_parent_order ON blocks(parent_id, \"order\") WHERE parent_id IS NOT NULL", [])?;
    
    // Page navigation indexes
    conn.execute("CREATE INDEX IF NOT EXISTS idx_pages_path ON pages(path)", [])?;
    conn.execute("CREATE INDEX IF NOT EXISTS idx_pages_title ON pages(title)", [])?;
    
    // Task management indexes
    conn.execute("CREATE INDEX IF NOT EXISTS idx_tasks_status ON tasks(status)", [])?;
    conn.execute("CREATE INDEX IF NOT EXISTS idx_tasks_due_date ON tasks(due_date) WHERE due_date IS NOT NULL", [])?;
    conn.execute("CREATE INDEX IF NOT EXISTS idx_tasks_priority ON tasks(priority) WHERE priority IS NOT NULL", [])?;
    
    // Link and backlink indexes
    conn.execute("CREATE INDEX IF NOT EXISTS idx_links_source ON links(source_block_id)", [])?;
    conn.execute("CREATE INDEX IF NOT EXISTS idx_links_target ON links(target_page_id)", [])?;
    
    // Property lookup indexes
    conn.execute("CREATE INDEX IF NOT EXISTS idx_block_properties_key ON block_properties(key)", [])?;
    conn.execute("CREATE INDEX IF NOT EXISTS idx_block_properties_block ON block_properties(block_id)", [])?;
    
    // Timestamp indexes for performance queries
    conn.execute("CREATE INDEX IF NOT EXISTS idx_blocks_updated ON blocks(updated_at)", [])?;
    conn.execute("CREATE INDEX IF NOT EXISTS idx_pages_updated ON pages(updated_at)", [])?;
    
    Ok(())
}

/// Create database triggers for data consistency and FTS maintenance
fn create_triggers(conn: &Connection) -> SqlResult<()> {
    // Trigger to update page updated_at when blocks are modified
    conn.execute(
        r#"
        CREATE TRIGGER IF NOT EXISTS update_page_timestamp
        AFTER UPDATE ON blocks
        BEGIN
            UPDATE pages 
            SET updated_at = CURRENT_TIMESTAMP 
            WHERE id = NEW.page_id;
        END
        "#,
        [],
    )?;
    
    // Trigger to maintain FTS index when blocks are inserted
    conn.execute(
        r#"
        CREATE TRIGGER IF NOT EXISTS blocks_fts_insert
        AFTER INSERT ON blocks
        BEGIN
            INSERT INTO blocks_fts(rowid, content) VALUES (NEW.id, NEW.content);
        END
        "#,
        [],
    )?;
    
    // Trigger to maintain FTS index when blocks are updated
    conn.execute(
        r#"
        CREATE TRIGGER IF NOT EXISTS blocks_fts_update
        AFTER UPDATE ON blocks
        BEGIN
            UPDATE blocks_fts SET content = NEW.content WHERE rowid = NEW.id;
        END
        "#,
        [],
    )?;
    
    // Trigger to maintain FTS index when blocks are deleted
    conn.execute(
        r#"
        CREATE TRIGGER IF NOT EXISTS blocks_fts_delete
        AFTER DELETE ON blocks
        BEGIN
            DELETE FROM blocks_fts WHERE rowid = OLD.id;
        END
        "#,
        [],
    )?;
    
    // Trigger to automatically create/update task entries for task blocks
    conn.execute(
        r#"
        CREATE TRIGGER IF NOT EXISTS maintain_task_entries
        AFTER INSERT ON blocks
        WHEN (
            LOWER(TRIM(NEW.content)) LIKE 'todo %' OR
            LOWER(TRIM(NEW.content)) LIKE 'doing %' OR
            LOWER(TRIM(NEW.content)) LIKE 'done %' OR
            LOWER(TRIM(NEW.content)) LIKE 'waiting %' OR
            LOWER(TRIM(NEW.content)) LIKE 'cancelled %'
        )
        BEGIN
            INSERT OR REPLACE INTO tasks (block_id, status, created_at, updated_at)
            VALUES (
                NEW.id,
                CASE 
                    WHEN LOWER(TRIM(NEW.content)) LIKE 'todo %' THEN 'TODO'
                    WHEN LOWER(TRIM(NEW.content)) LIKE 'doing %' THEN 'DOING'
                    WHEN LOWER(TRIM(NEW.content)) LIKE 'done %' THEN 'DONE'
                    WHEN LOWER(TRIM(NEW.content)) LIKE 'waiting %' THEN 'WAITING'
                    WHEN LOWER(TRIM(NEW.content)) LIKE 'cancelled %' THEN 'CANCELLED'
                END,
                CURRENT_TIMESTAMP,
                CURRENT_TIMESTAMP
            );
        END
        "#,
        [],
    )?;
    
    // Trigger to update task entries when blocks are updated
    conn.execute(
        r#"
        CREATE TRIGGER IF NOT EXISTS update_task_entries
        AFTER UPDATE ON blocks
        WHEN (
            LOWER(TRIM(NEW.content)) LIKE 'todo %' OR
            LOWER(TRIM(NEW.content)) LIKE 'doing %' OR
            LOWER(TRIM(NEW.content)) LIKE 'done %' OR
            LOWER(TRIM(NEW.content)) LIKE 'waiting %' OR
            LOWER(TRIM(NEW.content)) LIKE 'cancelled %'
        )
        BEGIN
            -- Record state change if status changed
            INSERT INTO task_state_changes (task_id, old_status, new_status)
            SELECT t.id, t.status, 
                CASE 
                    WHEN LOWER(TRIM(NEW.content)) LIKE 'todo %' THEN 'TODO'
                    WHEN LOWER(TRIM(NEW.content)) LIKE 'doing %' THEN 'DOING'
                    WHEN LOWER(TRIM(NEW.content)) LIKE 'done %' THEN 'DONE'
                    WHEN LOWER(TRIM(NEW.content)) LIKE 'waiting %' THEN 'WAITING'
                    WHEN LOWER(TRIM(NEW.content)) LIKE 'cancelled %' THEN 'CANCELLED'
                END as new_status
            FROM tasks t 
            WHERE t.block_id = NEW.id 
            AND t.status != 
                CASE 
                    WHEN LOWER(TRIM(NEW.content)) LIKE 'todo %' THEN 'TODO'
                    WHEN LOWER(TRIM(NEW.content)) LIKE 'doing %' THEN 'DOING'
                    WHEN LOWER(TRIM(NEW.content)) LIKE 'done %' THEN 'DONE'
                    WHEN LOWER(TRIM(NEW.content)) LIKE 'waiting %' THEN 'WAITING'
                    WHEN LOWER(TRIM(NEW.content)) LIKE 'cancelled %' THEN 'CANCELLED'
                END;
                
            -- Update task entry
            UPDATE tasks 
            SET status = 
                CASE 
                    WHEN LOWER(TRIM(NEW.content)) LIKE 'todo %' THEN 'TODO'
                    WHEN LOWER(TRIM(NEW.content)) LIKE 'doing %' THEN 'DOING'
                    WHEN LOWER(TRIM(NEW.content)) LIKE 'done %' THEN 'DONE'
                    WHEN LOWER(TRIM(NEW.content)) LIKE 'waiting %' THEN 'WAITING'
                    WHEN LOWER(TRIM(NEW.content)) LIKE 'cancelled %' THEN 'CANCELLED'
                END,
                updated_at = CURRENT_TIMESTAMP,
                completed_at = CASE 
                    WHEN LOWER(TRIM(NEW.content)) LIKE 'done %' THEN CURRENT_TIMESTAMP
                    ELSE completed_at
                END
            WHERE block_id = NEW.id;
        END
        "#,
        [],
    )?;
    
    Ok(())
}

/// Create initial system data (templates, settings, etc.)
fn create_initial_data(conn: &Connection) -> SqlResult<()> {
    // If any pages already exist, assume initial data has been created and skip
    let existing_pages: i64 = conn
        .prepare("SELECT COUNT(*) FROM pages")?
        .query_row([], |row| row.get(0))?;
    if existing_pages > 0 {
        return Ok(());
    }
    // Create system template pages
    conn.execute(
        "INSERT INTO pages (title, path) VALUES (?, ?)",
        ["Block Templates", "_templates/blocks"],
    )?;
    let block_template_page_id = conn.last_insert_rowid();

    conn.execute(
        "INSERT INTO pages (title, path) VALUES (?, ?)",
        ["Daily Template", "_templates/daily-note"],
    )?;

    conn.execute(
        "INSERT INTO pages (title, path) VALUES (?, ?)",
        ["Meeting Template", "_templates/meeting"],
    )?;

    // Create system settings pages
    conn.execute(
        "INSERT INTO pages (title, path) VALUES (?, ?)",
        ["Configuration", "_settings/config"],
    )?;

    conn.execute(
        "INSERT INTO pages (title, path) VALUES (?, ?)",
        ["Theme Settings", "_settings/theme"],
    )?;

    // Create default templates with initial blocks
    conn.execute(
        "INSERT INTO blocks (uuid, page_id, parent_id, \"order\", content) VALUES (?, ?, ?, ?, ?)",
        rusqlite::params![
            &uuid::Uuid::new_v4().to_string(),
            block_template_page_id,
            None::<i64>,
            1000,
            "Meeting Notes"
        ],
    )?;

    let meeting_template_block_id = conn.last_insert_rowid();

    // Add child blocks for meeting template
    conn.execute(
        "INSERT INTO blocks (uuid, page_id, parent_id, \"order\", content) VALUES (?, ?, ?, ?, ?)",
        rusqlite::params![
            &uuid::Uuid::new_v4().to_string(),
            block_template_page_id,
            meeting_template_block_id,
            1000,
            "## Attendees"
        ],
    )?;

    conn.execute(
        "INSERT INTO blocks (uuid, page_id, parent_id, \"order\", content) VALUES (?, ?, ?, ?, ?)",
        rusqlite::params![
            &uuid::Uuid::new_v4().to_string(),
            block_template_page_id,
            meeting_template_block_id,
            2000,
            "## Agenda"
        ],
    )?;

    conn.execute(
        "INSERT INTO blocks (uuid, page_id, parent_id, \"order\", content) VALUES (?, ?, ?, ?, ?)",
        rusqlite::params![
            &uuid::Uuid::new_v4().to_string(),
            block_template_page_id,
            meeting_template_block_id,
            3000,
            "## Action Items"
        ],
    )?;

    Ok(())
}

/// Get current schema version from database
fn get_schema_version(conn: &Connection) -> Result<i32> {
    // Check if schema_version table exists
    let table_exists_count: i64 = conn
        .prepare("SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='schema_version'")?
        .query_row([], |row| row.get(0))?;
    let table_exists = table_exists_count > 0;
    
    if !table_exists {
        // Create schema_version table
        conn.execute(
            "CREATE TABLE schema_version (version INTEGER PRIMARY KEY)",
            [],
        )?;
        conn.execute("INSERT INTO schema_version (version) VALUES (0)", [])?;
        return Ok(0);
    }
    
    // Get current version
    let version: i32 = conn
        .prepare("SELECT version FROM schema_version LIMIT 1")?
        .query_row([], |row| row.get(0))
        .unwrap_or(0);
    
    Ok(version)
}

/// Set schema version in database
fn set_schema_version(conn: &Connection, version: i32) -> SqlResult<()> {
    conn.execute(
        "UPDATE schema_version SET version = ? WHERE rowid = 1",
        [version],
    )?;
    Ok(())
}

/// Run database migrations from old version to current
fn run_migrations(_conn: &Connection, from_version: i32) -> Result<()> {
    // Migration logic to be added in Phase 2
    if from_version < 1 {
        // Version 1: Initial schema
        // No migrations needed yet
    }
    Ok(())
}

/// Utility functions for schema management
pub mod utils {
    use super::*;
    
    /// Check database integrity
    pub fn check_database_integrity(conn: &Connection) -> Result<bool> {
        let integrity_check: String = conn
            .prepare("PRAGMA integrity_check")?
            .query_row([], |row| row.get(0))?;
        
        Ok(integrity_check == "ok")
    }
    
    /// Get database statistics
    pub fn get_database_stats(conn: &Connection) -> Result<DatabaseStats> {
        let page_count: i32 = conn
            .prepare("SELECT COUNT(*) FROM pages")?
            .query_row([], |row| row.get(0))?;
        
        let block_count: i32 = conn
            .prepare("SELECT COUNT(*) FROM blocks")?
            .query_row([], |row| row.get(0))?;
        
        let task_count: i32 = conn
            .prepare("SELECT COUNT(*) FROM tasks")?
            .query_row([], |row| row.get(0))?;
        
        let db_size: i64 = conn
            .prepare("SELECT page_count * page_size FROM pragma_page_count(), pragma_page_size()")?
            .query_row([], |row| row.get(0))?;
        
        Ok(DatabaseStats {
            page_count,
            block_count,
            task_count,
            database_size_bytes: db_size,
        })
    }
    
    /// Vacuum database to reclaim space
    pub fn vacuum_database(conn: &Connection) -> SqlResult<()> {
        conn.execute("VACUUM", [])?;
        Ok(())
    }
}

/// Database statistics for monitoring and debugging
#[derive(Debug, Clone)]
pub struct DatabaseStats {
    pub page_count: i32,
    pub block_count: i32,
    pub task_count: i32,
    pub database_size_bytes: i64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    
    #[test]
    fn test_schema_initialization() {
        let temp_file = NamedTempFile::new().unwrap();
        let conn = Connection::open(temp_file.path()).unwrap();
        
        initialize_schema(&conn).unwrap();
        
        // Verify all tables exist
        let tables = get_table_names(&conn).unwrap();
        assert!(tables.contains(&"pages".to_string()));
        assert!(tables.contains(&"blocks".to_string()));
        assert!(tables.contains(&"tasks".to_string()));
        assert!(tables.contains(&"blocks_fts".to_string()));
        
        // Verify initial data
        let page_count: i32 = conn
            .prepare("SELECT COUNT(*) FROM pages")
            .unwrap()
            .query_row([], |row| row.get(0))
            .unwrap();
        assert!(page_count > 0); // Should have system pages
    }
    
    #[test]
    fn test_database_integrity() {
        let temp_file = NamedTempFile::new().unwrap();
        let conn = Connection::open(temp_file.path()).unwrap();
        initialize_schema(&conn).unwrap();
        
        assert!(utils::check_database_integrity(&conn).unwrap());
    }
    
    fn get_table_names(conn: &Connection) -> SqlResult<Vec<String>> {
        let mut stmt = conn.prepare("SELECT name FROM sqlite_master WHERE type='table'")?;
        let rows = stmt.query_map([], |row| {
            Ok(row.get::<_, String>(0)?)
        })?;
        
        let mut names = Vec::new();
        for name in rows {
            names.push(name?);
        }
        Ok(names)
    }
}
