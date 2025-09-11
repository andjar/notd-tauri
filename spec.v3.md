# Outliner Application Technical Specification v3.0

*Complete Implementation-Ready Specification*

## 1. Project Overview

### 1.1 Purpose
Build a simple, durable, **block-based** outliner application that combines the simplicity of `nb` with powerful note-taking and journaling features. The application must be deployable as a single executable with maximum longevity.

### 1.2 Core Philosophy
- **Block-First**: The block is the fundamental unit of information
- **Simplicity First**: Minimal UI, essential features only
- **Longevity**: Built to last decades without dependency rot
- **Offline-First**: Complete functionality without internet
- **Data Ownership**: User controls their data completely
- **Plain Text Foundation**: Content is stored in human-readable format

### 1.3 Target Users
- Knowledge workers seeking a simple, reliable outliner
- Researchers and writers who need structured thinking tools
- Users migrating from applications like Logseq or Roam Research
- Privacy-conscious individuals wanting local data storage

## 2. Technical Architecture

### 2.1 Technology Stack
- **Framework**: Tauri 2.0 (Rust backend + Web frontend)
- **Backend Language**: Rust 1.70+
- **Database**: SQLite 3.40+ (embedded)
- **Frontend**: HTML5/CSS3/JavaScript (Vanilla JS or lightweight framework)
- **Build System**: Cargo + Tauri CLI
- **Packaging**: Single executable with embedded assets

### 2.2 Application Structure
```
outliner/
├── src-tauri/           # Rust backend
│   ├── src/
│   │   ├── main.rs      # Application entry point
│   │   ├── db/          # Database operations
│   │   │   ├── mod.rs   # Database module
│   │   │   ├── schema.rs # Schema definitions
│   │   │   ├── blocks.rs # Block operations
│   │   │   ├── pages.rs # Page operations  
│   │   │   └── search.rs # Search operations
│   │   ├── api/         # Local HTTP API implementation
│   │   ├── parser/      # Content parsing
│   │   │   ├── markdown.rs # Markdown parser
│   │   │   ├── properties.rs # Property parser
│   │   │   └── links.rs # Link parser
│   │   ├── commands/    # Internal Tauri commands
│   │   └── utils/       # Utility functions
├── src/                 # Frontend assets
│   ├── index.html
│   ├── css/
│   │   ├── main.css     # Core styles
│   │   └── themes/      # Theme variations
│   ├── js/
│   │   ├── main.js      # Application logic
│   │   ├── editor.js    # Block editor
│   │   ├── outliner.js  # Outline manipulation
│   │   └── components/  # UI components
│   └── assets/
└── tauri.conf.json     # Tauri configuration
```

## 3. Data Model

### 3.1 Core Concepts
- **Pages** are organizational containers that group related blocks
- **Blocks** are the fundamental content units that belong to pages
- **Root blocks** have no parent but belong to a page
- **Child blocks** inherit their page from their parent at creation time
- Pages act like special blocks (containers for parentless blocks) but are stored separately for performance

### 3.2 SQLite Schema

```sql
-- Pages act as named entry points and containers for blocks
CREATE TABLE pages (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    title TEXT NOT NULL UNIQUE,
    path TEXT NOT NULL UNIQUE,      -- forward-slash hierarchy for organization
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

-- Blocks are the fundamental unit of content
CREATE TABLE blocks (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    uuid TEXT NOT NULL UNIQUE,      -- Stable identifier for transclusion/linking
    page_id INTEGER NOT NULL REFERENCES pages(id) ON DELETE CASCADE,
    parent_id INTEGER REFERENCES blocks(id) ON DELETE CASCADE, -- NULL for root blocks
    "order" INTEGER NOT NULL,       -- Position within parent (or page for root blocks)
    content TEXT NOT NULL,
    content_encrypted TEXT,         -- Encrypted version of content
    is_encrypted BOOLEAN DEFAULT FALSE,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

-- Flexible key-value properties for blocks
CREATE TABLE block_properties (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    block_id INTEGER NOT NULL REFERENCES blocks(id) ON DELETE CASCADE,
    key TEXT NOT NULL,
    value TEXT NOT NULL,            -- Plain text only, multi-line supported
    value_type TEXT DEFAULT 'text', -- 'text', 'list' for future distinction
    UNIQUE(block_id, key)
);

-- Links connect blocks to pages
CREATE TABLE links (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    source_block_id INTEGER NOT NULL REFERENCES blocks(id) ON DELETE CASCADE,
    target_page_id INTEGER NOT NULL REFERENCES pages(id) ON DELETE CASCADE,
    link_text TEXT,                 -- The text within [[]] brackets
    UNIQUE(source_block_id, target_page_id, link_text)
);

-- Tasks are an indexed view of blocks marked as tasks
CREATE TABLE tasks (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    block_id INTEGER NOT NULL UNIQUE REFERENCES blocks(id) ON DELETE CASCADE,
    status TEXT NOT NULL DEFAULT 'TODO',  -- TODO, DOING, DONE, WAITING, CANCELLED
    priority TEXT,                  -- high, medium, low
    due_date DATETIME,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    completed_at DATETIME
);

-- Task state changes for time tracking
CREATE TABLE task_state_changes (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    task_id INTEGER NOT NULL REFERENCES tasks(id) ON DELETE CASCADE,
    old_status TEXT,
    new_status TEXT NOT NULL,
    changed_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

-- Attachments (linked to pages)
CREATE TABLE attachments (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    page_id INTEGER NOT NULL REFERENCES pages(id) ON DELETE CASCADE,
    filename TEXT NOT NULL,
    file_path TEXT NOT NULL,        -- Relative to attachments directory
    file_size INTEGER,
    mime_type TEXT,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

-- Virtual table for full-text search on blocks
CREATE VIRTUAL TABLE blocks_fts USING fts5(
    content,
    content='blocks',
    content_rowid='id'
);

-- Indexes for performance
CREATE INDEX idx_blocks_parent ON blocks(parent_id);
CREATE INDEX idx_blocks_page ON blocks(page_id);
CREATE INDEX idx_blocks_order ON blocks(page_id, parent_id, "order");
CREATE INDEX idx_tasks_status ON tasks(status);
CREATE INDEX idx_pages_path ON pages(path);
CREATE INDEX idx_links_source ON links(source_block_id);
CREATE INDEX idx_links_target ON links(target_page_id);
```

### 3.3 Block Ordering System
- Blocks use integer ordering within their parent/page scope
- Use gap-based ordering (10, 20, 30, ...) to allow efficient insertions
- When gaps are exhausted, reorder all siblings with new gaps
- Order is scoped to: (page_id, parent_id) combination

### 3.4 Special Pages System
Special pages use reserved namespaces starting with underscore:

#### 3.4.1 Templates (`_templates/`)
- `_templates/daily-note` - Template for daily pages
- `_templates/meeting` - Meeting template
- `_templates/project` - Project template
- `_templates/blocks` - Special page containing block templates:
  - Each root block represents a block template
  - Root block content = template name
  - Child blocks = actual template content to insert

#### 3.4.2 Settings (`_settings/`)
- `_settings/config` - Application configuration
- `_settings/theme` - Theme preferences
- `_settings/shortcuts` - Keyboard shortcuts
- Settings stored as block properties on these pages

#### 3.4.3 Template Variables
Replaced at insertion time:
- `<today>` - Current date (YYYY-MM-DD)
- `<time>` - Current time (HH:MM)
- `<title>` - Page title
- `<path>` - Page path

### 3.5 File System Structure
```
user_data/
├── database.db         # Main SQLite database
├── attachments/        # File attachments organized by UUID
│   ├── [uuid]/        
│   └── [uuid]/        
├── exports/           # Export destination
├── backups/           # Database backups
└── logs/              # Application logs
```

## 4. Feature Specifications

### 4.1 Journal-Based Organization

#### 4.1.1 Daily Pages
**Functional Requirements:**
- Auto-create daily pages with format: `YYYY-MM-DD`
- Create on first access or app startup for today
- Apply daily template from `_templates/daily-note` if exists
- Quick navigation: Today, Yesterday, Tomorrow shortcuts

**Implementation:**
- Check for today's page on startup, create if missing
- Template application uses variable substitution
- Calendar widget highlights days with content

#### 4.1.2 Page Hierarchy
**Path Structure:**
- Forward-slash notation: `projects/website/design-notes`
- Auto-create parent pages when child is created
- Breadcrumb navigation in UI
- Tree view in sidebar showing hierarchy

#### 4.1.3 First-Run Experience
- On first launch, navigate directly to today's page
- Create today's page with empty block ready for input
- No tutorials, samples, or setup wizards

### 4.2 Block-Based Content System

#### 4.2.1 Block Structure
- Every page contains one or more blocks
- Empty pages automatically get one empty block
- Root blocks belong to page but have no parent
- Child blocks inherit page from parent at creation
- Each block has stable UUID for referencing

#### 4.2.2 Block Creation & Navigation
**Keyboard Shortcuts:**
- `Enter` at end of block: Create sibling block at same level
- `Cmd/Ctrl + Enter`: Create child block (indent one level)
- `Backspace` at start of empty block: Delete block or promote to parent level

**UI Elements:**
- Small `(+)` button at bottom of page to create new root block
- Click anywhere in empty space below blocks to create new root block

#### 4.2.3 Block Editing
**Edit Mode Activation:**
- Click on any block to enter edit mode for that specific block only
- Visual indication: cursor appears, block gets focus border
- Other blocks remain in render mode

**Edit Mode Behavior:**
- Raw markdown content is editable
- Properties, links, and special syntax visible
- Auto-save on focus loss or after 2 seconds of inactivity
- `Escape` key exits edit mode

### 4.3 Content Processing & Parsing

#### 4.3.1 Supported Markdown Features
**Basic Formatting:**
- `**bold**` and `*italic*` and `***bold italic***`
- `~~strikethrough~~`
- `` `inline code` ``
- `[link text](URL)` - external links marked with icon
- `![alt text](image URL)` - embedded images

**Code Blocks:**
```markdown
```language
code content
```
```

**Lists:**
- Unordered lists with `- ` or `* `
- Ordered lists with `1. `
- Nested lists supported

**Not Supported (for simplicity):**
- Tables, footnotes, math expressions
- Complex HTML (only basic tags rendered)

#### 4.3.2 Page Linking
**Syntax:** `[[Page Name]]`
**Behavior:**
- Auto-suggestion dropdown appears when typing `[[`
- Shows matching page titles as user types
- `Tab` to accept suggestion, `Escape` to cancel
- Auto-closes with `]]` when page selected
- Creates link entry in links table
- Non-existent pages shown in different color

#### 4.3.3 Block Properties
**Syntax:**
```markdown
- This is a block with properties
  - priority:: high
  - due:: 2024-12-31
  - tags:: work, urgent
  - notes:: This is a multi-line
    property that continues
    on the next line
```

**Parsing Rules:**
- Property lines start with `key::`
- Multi-line values supported (continue on indented lines)
- Only plain text values (no markdown processing in properties)
- Properties hidden in render mode, editable in right panel
- List-type properties: comma-separated values

#### 4.3.4 Task Management
**Task Identification:**
- Blocks starting with `TODO`, `DOING`, `DONE`, `WAITING`, or `CANCELLED`
- Parser creates/updates corresponding tasks table entry
- Checkbox UI element syncs with text status

**Task Properties:**
- Use standard property system: `due::`, `priority::`
- Inline tags parsed from content: `#urgent #work`
- Tags stored in `_tags` property for querying

### 4.4 Advanced Features (Priority Order)

#### 4.4.1 Attachments (Highest Priority)
**Implementation:**
- Drag & drop files onto page to attach
- Files copied to `attachments/[uuid]/` directory
- Reference stored in attachments table
- Display as file link with icon and size

#### 4.4.2 SQL Queries (Second Priority)
**Syntax:** `SQL{SELECT title FROM pages WHERE path LIKE 'projects/%'}`
**Security:**
- Read-only queries only
- Whitelist allowed table/column access
- Query results cached with TTL
- Render as table or list in UI

#### 4.4.3 Transclusion
**Syntax:** `{{transclude: block_uuid}}`
**Features:**
- Embed rendered content of block and its children
- Show as raw text in edit mode, rendered in read mode
- Visual border indicates transcluded content
- Prevent infinite recursion loops

#### 4.4.4 Encryption (Third Priority)
**Scope:**
- Page-level encryption (all blocks on page encrypted together)
- Block content encrypted, properties remain plain text
- Master password for session-based decryption

#### 4.4.5 Graph View (Lowest Priority)
- Visual representation of page links
- Interactive node-link diagram
- Filter by date range, tags, or content type

## 5. User Interface Specifications

### 5.1 Layout Structure
```
┌─────────────────────────────────────────────────────────────┐
│ Title Bar                            [Minimize] [Close]     │
├─────────────┬───────────────────────────────┬───────────────┤
│             │                               │               │
│  Sidebar    │        Main Editor           │  Right Panel  │
│  (250px)    │                               │   (300px)     │
│             │  ┌─────────────────────────┐  │               │
│ • Today     │  │ • Block 1               │  │ Backlinks     │
│ • Calendar  │  │   • Child block         │  │ ─────────     │
│ • Pages     │  │ • Block 2 [TODO]        │  │ • [[Link]]    │
│ • Tasks     │  │ • Block 3               │  │               │
│ • Search    │  │   • Sub-block           │  │ Properties    │
│             │  │   • Another sub         │  │ ─────────     │
│             │  │ [+] New block           │  │ priority: high│
│             │  └─────────────────────────┘  │ due: tomorrow │
├─────────────┼───────────────────────────────┼───────────────┤
│             │ Current Page: projects/notes  │               │
└─────────────┴───────────────────────────────┴───────────────┘
```

### 5.2 Block Visual Design
```
• Block content goes here and can wrap to multiple lines
  as needed for longer content
  │
  ├─ • Child block indented with connecting lines
  │    • Multi-line child block content also
  │      wraps nicely within the indentation
  │      │
  │      └─ • Nested child with deeper indentation
  │
  └─ • Another child at the same level
```

**Visual Elements:**
- Bullet points (•) for all blocks
- Connecting lines show hierarchy (`├─`, `│`, `└─`)
- Hover state: bullet becomes draggable handle
- Edit state: block gets subtle border, cursor appears
- No visual boundaries between blocks in render mode

### 5.3 Drag & Drop Behavior
**Dragging:**
- Drag from bullet point only
- Block becomes semi-transparent "ghost"
- Shows insertion line (horizontal bar) at drop zones
- Drop zones appear between blocks at valid levels

**Drop Zones:**
- Above any block (same level)
- Below any block (same level)  
- First child position (indented)
- Cannot drag between pages (use cut/paste)

**Visual Feedback:**
- Horizontal insertion line appears at drop position
- Insertion line indentation shows target level
- Invalid drop zones don't show insertion line

### 5.4 Right Panel Content

#### 5.4.1 Backlinks Section
- Lists pages that link to current page
- Each backlink shows the containing block context
- Clicking backlink navigates to source block
- Live update as links are created/removed

#### 5.4.2 Properties Section
- Shows properties of currently selected/edited block
- Key-value editing interface
- Add new properties with `+` button
- Delete properties with trash icon
- Multi-line text editing for values

#### 5.4.3 Page Info Section
- Created date, modified date
- Block count, task count
- Encryption status if applicable

### 5.5 Keyboard Shortcuts

#### 5.5.1 Global Navigation
```
Ctrl/Cmd + N         : New page
Ctrl/Cmd + O         : Quick open (search pages)
Ctrl/Cmd + S         : Save current page (manual trigger)
Ctrl/Cmd + D         : Go to today's journal
Ctrl/Cmd + P         : Command palette
Ctrl/Cmd + F         : Search current page
Ctrl/Cmd + Shift + F : Global search all pages
Ctrl/Cmd + B         : Toggle sidebar
```

#### 5.5.2 Block Editing
```
Enter                : Create sibling block
Ctrl/Cmd + Enter     : Create child block  
Ctrl/Cmd + Shift + Enter : Create parent block (outdent new)
Tab                  : Indent block (make child of previous)
Shift + Tab          : Outdent block (promote to parent level)
Ctrl/Cmd + Up        : Move block up
Ctrl/Cmd + Down      : Move block down
Ctrl/Cmd + Shift + Up   : Move block up and outdent
Ctrl/Cmd + Shift + Down : Move block down and indent
```

#### 5.5.3 Text Formatting (in edit mode)
```
Ctrl/Cmd + B         : Bold selection
Ctrl/Cmd + I         : Italic selection
Ctrl/Cmd + K         : Insert/edit link
Ctrl/Cmd + T         : Convert to task (add TODO)
Escape               : Exit edit mode
```

## 6. Performance Requirements & Targets

### 6.1 Performance Targets
- **Perfect Performance Target**: Up to 10,000 blocks
- **Acceptable Performance**: Up to 100,000 blocks (with expected slowdown)
- **Block Render Time**: < 50ms for typical page (50-100 blocks)
- **Search Response**: < 200ms for full-text search across 10K blocks
- **Startup Time**: < 2 seconds cold start
- **Memory Usage**: < 150MB for typical workload (5K blocks)

### 6.2 Performance Implementation Strategy
- Lazy load blocks outside viewport for large pages
- Index all searchable content in FTS5
- Use efficient SQL queries with proper indexing
- Cache rendered markdown content with invalidation
- Debounce save operations (2 second delay)

## 7. API Specifications

### 7.1 Local HTTP API
**Port**: `127.0.0.1:41824` (configurable)
**Security**: Optional API key in headers
**Content-Type**: `application/json`

### 7.2 Core Endpoints

#### 7.2.1 Pages
```http
GET /api/pages                    # List all pages
POST /api/pages                   # Create new page
GET /api/pages/{id}               # Get page details
PUT /api/pages/{id}               # Update page
DELETE /api/pages/{id}            # Delete page
GET /api/pages/{id}/blocks        # Get page blocks hierarchy
```

#### 7.2.2 Blocks  
```http
POST /api/blocks                  # Create new block
GET /api/blocks/{uuid}            # Get block details
PUT /api/blocks/{uuid}            # Update block content
DELETE /api/blocks/{uuid}         # Delete block
POST /api/blocks/{uuid}/move      # Move block to new position
GET /api/blocks/{uuid}/children   # Get block children
```

#### 7.2.3 Search & Query
```http
GET /api/search?q={query}         # Full-text search
POST /api/query                   # Execute SQL query
GET /api/pages/{id}/mentions      # Get backlinks to page
```

#### 7.2.4 System Operations
```http
POST /api/backup                  # Create database backup
GET /api/stats                    # Get database statistics
POST /api/import                  # Import data
GET /api/export                   # Export data
```

## 8. Development Implementation Plan

### Phase 1: Core Foundation (4-6 weeks)
**Goal**: Basic functional outliner with block editing

**Deliverables:**
- [ ] Tauri 2.0 project setup with Rust backend
- [ ] SQLite database integration with schema
- [ ] Basic page creation and navigation
- [ ] Block creation, editing, and hierarchy
- [ ] Simple markdown rendering
- [ ] Block ordering and repositioning

**Technical Tasks:**
- Database schema implementation
- Basic CRUD operations for pages/blocks
- Simple web UI with block editing
- Markdown parser integration
- Block ordering system implementation

### Phase 2: Essential Features (4-6 weeks)  
**Goal**: Daily workflow functionality

**Deliverables:**
- [ ] Daily pages with auto-creation
- [ ] Page linking with `[[]]` syntax and auto-suggestions
- [ ] Basic task management (TODO/DONE detection)
- [ ] Search functionality (full-text)
- [ ] Template system with special pages
- [ ] Properties system for blocks

**Technical Tasks:**
- Link parsing and suggestion UI
- Task detection and management
- FTS5 search implementation
- Template engine with variable substitution
- Property parser and storage

### Phase 3: Advanced Features (6-8 weeks)
**Goal**: Power user features and data management

**Deliverables:**
- [ ] File attachments with drag & drop
- [ ] SQL query execution with security
- [ ] Block transclusion system
- [ ] Basic encryption for sensitive pages
- [ ] Time tracking for tasks
- [ ] Data export/import functionality

**Technical Tasks:**
- File attachment handling
- SQL query sandboxing
- Transclusion rendering engine
- Encryption implementation (AES-256-GCM)
- Export system (Markdown format)

### Phase 4: Polish & Optimization (3-4 weeks)
**Goal**: Production-ready application

**Deliverables:**
- [ ] Graph view visualization
- [ ] Performance optimization
- [ ] Error handling and recovery
- [ ] Cross-platform testing
- [ ] Documentation and packaging
- [ ] Local API implementation

**Technical Tasks:**
- Graph visualization component
- Performance profiling and optimization
- Comprehensive error handling
- Build system and distribution setup
- API documentation

## 9. Technical Implementation Details

### 9.1 Block Ordering Algorithm
```rust
// Efficient block reordering with gap-based system
const ORDER_GAP: i32 = 1000;

fn insert_block_at_position(parent_id: Option<i32>, position: usize) -> i32 {
    let siblings = get_sibling_blocks(parent_id);
    
    if siblings.is_empty() {
        return ORDER_GAP;
    }
    
    if position == 0 {
        // Insert at beginning
        siblings[0].order - ORDER_GAP
    } else if position >= siblings.len() {
        // Insert at end
        siblings.last().unwrap().order + ORDER_GAP
    } else {
        // Insert between siblings
        let prev_order = siblings[position - 1].order;
        let next_order = siblings[position].order;
        
        if next_order - prev_order > 1 {
            (prev_order + next_order) / 2
        } else {
            // Reorder all siblings with new gaps
            reorder_siblings_with_gaps(parent_id);
            position as i32 * ORDER_GAP
        }
    }
}
```

### 9.2 Content Processing Pipeline
1. **Parse** markdown and extract special syntax
2. **Validate** content structure and references
3. **Store** parsed content and update related tables
4. **Index** content in FTS5 for search
5. **Render** processed content for display

### 9.3 Security Considerations
- SQL injection prevention in query API
- Path traversal protection for attachments
- Encryption key management for secure pages
- Local-only API binding (no external network access)

### 9.4 Error Handling Strategy
- Graceful degradation for corrupted blocks
- Warning messages for broken references
- Automatic backup before destructive operations
- Recovery mode for database corruption

## 10. Success Criteria & Quality Gates

### 10.1 Functional Requirements
- [ ] All core features implemented and working
- [ ] Data integrity maintained across operations
- [ ] Cross-platform compatibility (Windows, macOS, Linux)
- [ ] Performance targets met for 10K block database

### 10.2 Quality Requirements
- [ ] Zero data loss in normal operation
- [ ] Intuitive UI requiring minimal learning
- [ ] Comprehensive test coverage (>80%)
- [ ] Robust error handling and recovery

### 10.3 Longevity Requirements
- [ ] Data exportable to standard formats
- [ ] Minimal external dependencies
- [ ] Clear upgrade path for future versions
- [ ] Comprehensive documentation for maintainers

---

*This specification provides a complete, implementation-ready foundation for building a durable, simple, and powerful block-based outliner application. Every major component is detailed with specific implementation guidance while maintaining the core principles of simplicity and longevity.*
