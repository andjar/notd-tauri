# Outliner Architecture Overview

## Phase 1: Core Foundation Architecture

### High-Level System Design

The outliner application follows a **block-first architecture** where every piece of content is a block, and pages serve as organizational containers for these blocks.

### Core Components

#### 1. Database Layer (`src-tauri/src/db/`)
**Purpose**: Manage all data persistence and retrieval operations

**Core Modules:**
- `schema.rs` - Database schema definitions and migrations
- `blocks.rs` - Block CRUD operations and hierarchy management  
- `pages.rs` - Page CRUD operations and navigation
- `search.rs` - Full-text search implementation

**Key Functions to Implement:**
```rust
// Schema management
pub fn initialize_database(db_path: &str) -> Result<Connection, Error>
pub fn run_migrations(conn: &Connection) -> Result<(), Error>

// Block operations
pub fn create_block(page_id: i32, parent_id: Option<i32>, content: &str) -> Result<Block, Error>
pub fn get_block_hierarchy(page_id: i32) -> Result<Vec<Block>, Error>
pub fn update_block_content(block_id: i32, content: &str) -> Result<(), Error>
pub fn reorder_block(block_id: i32, new_position: i32, new_parent_id: Option<i32>) -> Result<(), Error>

// Page operations
pub fn create_page(title: &str, path: &str) -> Result<Page, Error>
pub fn get_page_by_path(path: &str) -> Result<Option<Page>, Error>
pub fn get_daily_page(date: &str) -> Result<Page, Error> // Auto-create if not exists
```

#### 2. Content Processing (`src-tauri/src/parser/`)
**Purpose**: Parse and process block content, extract metadata

**Core Modules:**
- `markdown.rs` - Basic markdown parsing and rendering
- `properties.rs` - Block property extraction and parsing
- `links.rs` - Page link detection and parsing

**Key Functions to Implement:**
```rust
// Markdown processing
pub fn parse_markdown_basic(content: &str) -> String
pub fn extract_task_status(content: &str) -> Option<TaskStatus>

// Property extraction
pub fn extract_properties(content: &str) -> Vec<Property>
pub fn is_property_line(line: &str) -> bool

// Link processing
pub fn extract_page_links(content: &str) -> Vec<String>
pub fn suggest_page_completions(partial: &str) -> Vec<String>
```

#### 3. Core Data Structures

```rust
#[derive(Debug, Clone)]
pub struct Block {
    pub id: i32,
    pub uuid: String,
    pub page_id: i32,
    pub parent_id: Option<i32>,
    pub order: i32,
    pub content: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct Page {
    pub id: i32,
    pub title: String,
    pub path: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct Property {
    pub key: String,
    pub value: String,
    pub value_type: PropertyType,
}

#[derive(Debug, Clone)]
pub enum PropertyType {
    Text,
    List,
}
```

#### 4. Frontend Architecture (`src/js/`)

**Core Modules:**
- `main.js` - Application initialization and state management
- `editor.js` - Block editing functionality
- `outliner.js` - Block hierarchy manipulation
- `components/` - Reusable UI components

**Key Components to Implement:**
```javascript
// Block editing
class BlockEditor {
    constructor(blockElement, blockData) { }
    enterEditMode() { }
    exitEditMode() { }
    saveContent() { }
    handleKeyboardShortcuts(event) { }
}

// Outline management
class OutlineManager {
    constructor(pageContainer) { }
    renderBlockHierarchy(blocks) { }
    handleBlockDrag(sourceBlock, targetPosition) { }
    createNewBlock(parentId, position) { }
    deleteBlock(blockId) { }
}

// Page navigation
class PageManager {
    constructor() { }
    navigateToPage(path) { }
    createDailyPage(date) { }
    searchPages(query) { }
}
```

### Block Ordering System Design

**Gap-Based Ordering Algorithm:**
- Use integer gaps (1000, 2000, 3000) for initial spacing
- Insert between existing blocks by averaging their order values
- Reorder entire sibling group when gaps are exhausted
- Scope ordering by (page_id, parent_id) combination

```rust
const ORDER_GAP: i32 = 1000;

pub fn calculate_insert_order(parent_id: Option<i32>, position: usize) -> i32 {
    // Algorithm:
    // 1. Get all sibling blocks ordered by 'order' field
    // 2. If inserting at start: first_order - ORDER_GAP
    // 3. If inserting at end: last_order + ORDER_GAP  
    // 4. If inserting between: (prev_order + next_order) / 2
    // 5. If no gap available: reorder_all_siblings_with_gaps()
}
```

### UI State Management

**Edit Mode Management:**
- Only one block in edit mode at a time
- Click anywhere on block to enter edit mode
- Auto-save on focus loss or after 2-second delay
- ESC key exits edit mode

**Block Selection and Focus:**
- Track currently selected block for property display
- Keyboard navigation between blocks (Up/Down arrows)
- Visual indication of selected vs. edited blocks

### Data Flow Architecture

```
User Input → Frontend Component → Tauri Command → Database Operation → UI Update
     ↑                                                                      ↓
     └── UI State Management ←── Event Response ←── Result Processing ←────┘
```

**Example: Creating a New Block**
1. User presses Enter at end of block
2. `BlockEditor` captures keypress, calls `OutlineManager.createNewBlock()`
3. `OutlineManager` calculates position, calls Tauri command `create_block`
4. Rust backend processes request, updates database
5. Backend returns new block data
6. Frontend updates UI with new block, enters edit mode

### Performance Considerations for Phase 1

**Database Optimization:**
- Index on (page_id, parent_id, order) for hierarchy queries
- Index on (page_id) for page block loading
- Use prepared statements for common queries

**UI Optimization:**
- Render only visible blocks (implement virtual scrolling later)
- Debounce auto-save operations
- Cache rendered markdown content

**Memory Management:**
- Limit loaded pages in memory (implement page cache later)
- Clean up event listeners on component destruction

This architecture provides a solid foundation for Phase 1 implementation while setting up for future phases.
