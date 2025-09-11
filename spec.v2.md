Of course. Here is the complete, revised technical specification, incorporating all the feedback and refinements we've discussed. It's now built on a block-centric architecture, includes the advanced backlinking/query system, and specifies a local API for extensibility.

***

# Outliner Application Technical Specification (Revised)

## 1. Project Overview

### 1.1 Purpose
Build a simple, durable, **block-based** outliner application that combines the simplicity of `nb` with powerful note-taking and journaling features. The application must be deployable as a single executable with maximum longevity.

### 1.2 Core Philosophy
- **Block-First**: The block is the fundamental unit of information.
- **Simplicity First**: Minimal UI, essential features only.
- **Longevity**: Built to last decades without dependency rot.
- **Offline-First**: Complete functionality without internet.
- **Data Ownership**: User controls their data completely.
- **Plain Text Foundation**: Content is stored in a human-readable format within the database.

### 1.3 Target Users
- Knowledge workers seeking a simple, reliable outliner.
- Researchers and writers who need structured thinking tools.
- Users migrating from applications like Logseq or Roam Research.
- Privacy-conscious individuals wanting local data storage.

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
│   │   ├── api/         # Local HTTP API implementation
│   │   ├── parser/      # Markdown parsing
│   │   └── commands/    # Internal Tauri commands
├── src/                 # Frontend assets
│   ├── index.html
│   ├── css/
│   ├── js/
│   └── assets/
└── tauri.conf.json     # Tauri configuration
```

### 2.3 Data Storage Architecture

#### 2.3.1 SQLite Schema (Block-Centric)
The data model is centered around blocks. Pages are organizational containers that point to a root block.

```sql
-- Pages act as named entry points to a tree of blocks.
CREATE TABLE pages (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    title TEXT NOT NULL UNIQUE,
    path TEXT NOT NULL UNIQUE,      -- forward-slash hierarchy for organization
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

-- Blocks are the fundamental unit of content.
CREATE TABLE blocks (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    uuid TEXT NOT NULL UNIQUE,      -- A stable, user-facing identifier for transclusion/linking
    page_id INTEGER NOT NULL REFERENCES pages(id) ON DELETE CASCADE,
    parent_id INTEGER REFERENCES blocks(id) ON DELETE CASCADE, -- For hierarchy
    "order" INTEGER NOT NULL,       -- To maintain block order under a parent
    content TEXT NOT NULL,
    content_encrypted TEXT,
    is_encrypted BOOLEAN DEFAULT FALSE,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

-- Flexible key-value properties for any block.
CREATE TABLE block_properties (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    block_id INTEGER NOT NULL REFERENCES blocks(id) ON DELETE CASCADE,
    key TEXT NOT NULL,
    value TEXT NOT NULL,
    UNIQUE(block_id, key)
);

-- Links connect blocks to pages.
CREATE TABLE links (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    source_block_id INTEGER NOT NULL REFERENCES blocks(id) ON DELETE CASCADE,
    target_page_id INTEGER NOT NULL REFERENCES pages(id) ON DELETE CASCADE,
    UNIQUE(source_block_id, target_page_id)
);

-- Tasks are an indexed view of blocks marked as tasks for fast querying.
CREATE TABLE tasks (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    block_id INTEGER NOT NULL UNIQUE REFERENCES blocks(id) ON DELETE CASCADE,
    status TEXT NOT NULL DEFAULT 'TODO',  -- TODO, DOING, DONE, WAITING, CANCELLED
    priority TEXT,
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

-- Attachments
CREATE TABLE attachments (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    page_id INTEGER NOT NULL REFERENCES pages(id) ON DELETE CASCADE,
    filename TEXT NOT NULL,
    file_path TEXT NOT NULL,
    file_size INTEGER,
    mime_type TEXT,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

-- Virtual table for full-text search on blocks.
CREATE VIRTUAL TABLE blocks_fts USING fts5(
    content,
    content='blocks',
    content_rowid='id'
);

-- Indexes for performance
CREATE INDEX idx_blocks_parent ON blocks(parent_id);
CREATE INDEX idx_blocks_page ON blocks(page_id);
CREATE INDEX idx_tasks_status ON tasks(status);
```

#### 2.3.2 File System Structure
```
user_data/
├── database.db         # Main SQLite database
├── attachments/        # File attachments
│   └── [uuid]/        # Organized by UUID
└── exports/           # Export destination
```

## 3. Feature Specifications

### 3.1 Journal-Based Organization

#### 3.1.1 Daily Pages
**Functional Requirements:**
- Auto-create daily pages with format: `YYYY-MM-DD`
- Generate pages on first access or at midnight
- Support custom daily page templates
- Quick navigation to today, yesterday, tomorrow

#### 3.1.2 Calendar Navigation
**UI Components:**
- Month/year picker with activity indicators
- Date cells showing:
  - Has content (filled circle)
  - Has tasks (square indicator)  
  - Task completion status (colored indicators)
- Quick jump to specific dates
- Mini-calendar in sidebar

#### 3.1.3 Hierarchical Organization
**Path Structure:**
- Pages organized with forward-slash notation
- Example: `projects/website/design-notes`
- Automatic parent page creation
- Breadcrumb navigation
- Tree view in sidebar

#### 3.1.3 Template System
**Features:**
- Pre-defined templates (daily, meeting, project, etc.)
- Custom template creation through UI
- Template variables with substitution
- Template inheritance

**Template Variables:**
- `{{date}}` - Current date
- `{{time}}` - Current time
- `{{title}}` - Page title
- `{{path}}` - Page path

### 3.2 Rich Note-Taking & Structuring

#### 3.2.1 Block-Based Outlining
- **Hierarchy:** Blocks are organized hierarchically via `parent_id` and sorted by an `order` field.
- **Manipulation:** Users can indent/outdent blocks and move them within or between pages.
- **Folding:** Collapse and expand blocks to manage complex outlines.

#### 3.2.2 Markdown Support
*(Applies to block content)*
- Headers, lists, links, images, code blocks, tables, blockquotes, basic styling.

#### 3.2.3 Page & Block Linking
**Syntax:** `[[Page Name]]`
**Features:**
- **Auto-suggestion:** When typing `[[`, a dropdown appears with matching page titles.
- **Auto-closing:** Typing `[[` automatically inserts `]]`.
- **Contextual Backlinks (Live Queries):** A page's "Mentions" section will display not just the block linking to it, but that block and its entire hierarchy of children. This content is a live, editable embed from its original location. This allows `[[Page Name]]` to be used as an implicit header for organizing content from a central place like a daily page.

#### 3.2.4 Transclusion
**Syntax:** `{{transclude: block_uuid}}`
**Features:**
- Embed the rendered content of a single block or a block and its children.
- Rendered in "Render Mode," shown as raw text in "Edit Mode".
- A visual border or icon indicates transcluded content.
- Prevent infinite recursion

#### 3.2.5 SQL Queries
**Syntax:** `SQL{SELECT title FROM pages WHERE path LIKE 'projects/%'}`

**Features:**
- Execute queries against the database
- Render results as tables or lists
- Cache query results with TTL
- Security restrictions (read-only queries)

### 3.3 Properties System

#### 3.3.1 Syntax
Properties are key-value pairs indented under a parent block.
```markdown
- This block has properties
  - priority:: high
  - due:: 2024-12-31
```
#### 3.3.2 Implementation
- The parser stores these in the `block_properties` table.
- Users can define any property key. System-managed properties will use a `_` prefix (e.g., `_tags`) as a convention.
- Properties are displayed and editable in the right panel and are hidden in "Render Mode".

### 3.4 Task Management

#### 3.4.1 Task Identification
- Any block starting with `TODO`, `DOING`, `DONE`, `WAITING`, or `CANCELLED` is identified as a task.
- The parser creates/updates a corresponding entry in the `tasks` table for fast querying.
- The UI will display a checkbox that syncs with the block's starting keyword.

#### 3.4.2 Task Properties
- Metadata like `due` and `priority` are managed using the standard block properties system.
- Inline tags (`#urgent`) are parsed and can be stored in a `_tags` property for querying.

#### 3.4.3 Time Logging
**Automatic Tracking:**
- Task creation timestamp
- State change timestamps  
- Time spent in each state
- Total time from creation to completion

**Reports:**
- Task completion time averages
- Time spent by state/priority
- Productivity metrics by day/week

### 3.5 Security & Privacy
*(Encryption applies at the page level)*
- AES-256-GCM client-side encryption for individual pages.
- Master password for session-based decryption.
- AES-256-GCM encryption
- PBKDF2 key derivation (100,000 iterations)
- Per-page encryption with master password
- Salt stored with encrypted content

## 4. User Interface Specifications

### 4.1 Block-Based UI Layout
```
┌─────────────────────────────────────────────────────────────┐
│ Menu Bar                                  Edit/Render Toggle │
├─────────────┬───────────────────────────────┬───────────────┤
│             │                               │               │
│   Sidebar   │        Main Outliner         │   Right Panel │
│             │                               │               │
│ - Calendar  │  ┌─────────────────────────┐  │ - Backlinks   │
│ - Pages     │  │   • Block 1 (TODO)     │  │ - Mentions    │
│ - Tasks     │  │     • Child block      │  │ - Properties  │
│ - Tags      │  │   • Block 2 (content)  │  │ - Graph View  │
│ - Search    │  │   • Block 3 (heading)  │  │               │
│             │  │     • Sub-block 1      │  │               │
│             │  │     • Sub-block 2      │  │               │
│             │  │       • Nested TODO    │  │               │
│             │  └─────────────────────────┘  │               │
├─────────────┼───────────────────────────────┼───────────────┤
│             │     Status Bar + Block Path   │               │
└─────────────┴───────────────────────────────┴───────────────┘
```

### 4.2 Editor Modes (Edit vs. Render)
For the Main Outliner: block are rendered as default, but switches to edit mode whne one clicks on it

- **Edit Mode:** Displays the raw Markdown content. Syntax for links, transclusions, properties, and tasks is visible and directly editable. This is the mode for structuring and power-editing.
- **Render Mode:** Displays a clean, formatted view. Markdown is rendered, transclusions are replaced with embedded content, SQL queries are executed, and property blocks are hidden. This is the mode for reading and presentation.

### 4.3 Editor Experience
- **Auto-suggestion:** A popup suggests page titles when `[[` is typed.
- **Auto-closing Brackets:** Automatically closes `(`, `[`, `{`, and `"` characters.
- **Command Palette:** A global shortcut (`Ctrl/Cmd+P`) opens a unified interface for finding pages/blocks and executing commands.
- **Drag-and-drop:** Drag and drop blocks to move them and their children
- Blocks are rendered with a bullet to the left and threading to clearly show hierarchy. The bullet can be right-clicked to open context-menu (e.g., cut, delete, copy as transclusion) or dragged to move it in the hierarchy.

### 4.4 Keyboard Shortcuts
*(As in original spec, but oriented towards block manipulation)*

```
Global:
Ctrl/Cmd + N     : New page
Ctrl/Cmd + O     : Quick open
Ctrl/Cmd + S     : Save current page
Ctrl/Cmd + F     : Search
Ctrl/Cmd + Shift + F : Global search
Ctrl/Cmd + D     : Today's journal
Ctrl/Cmd + T     : Create task
Ctrl/Cmd + L     : Insert link

Editor:
Ctrl/Cmd + B     : Bold
Ctrl/Cmd + I     : Italic
Ctrl/Cmd + K     : Insert link
Ctrl/Cmd + E     : Toggle edit/preview
Tab              : Indent/outdent lists
Enter            : Smart list continuation
```


## 5. Performance Requirements
- **Block Render Time**: < 100ms for a typical page view.
- **Search Response**: < 200ms for full-text search.
- **Startup Time**: < 2 seconds cold start.
- **Memory Usage**: < 150MB for typical workload.

## 6. Development Phases
*(Phases remain the same, but implementation will follow the block-centric model)*


### Phase 1: Core Foundation (4-6 weeks)
- [ ] Project setup and basic Tauri application
- [ ] SQLite database integration
- [ ] Basic page creation/editing
- [ ] Markdown rendering
- [ ] Simple navigation

### Phase 2: Essential Features (4-6 weeks)
- [ ] Page linking and backlinks
- [ ] Daily pages and calendar
- [ ] Basic task management
- [ ] Search functionality
- [ ] Template system

### Phase 3: Advanced Features (6-8 weeks)
- [ ] Encryption implementation
- [ ] Transclusion
- [ ] SQL queries
- [ ] Advanced task properties
- [ ] Time tracking

### Phase 4: Polish & Optimization (3-4 weeks)
- [ ] UI refinements
- [ ] Performance optimization
- [ ] Testing and bug fixes
- [ ] Documentation
- [ ] Packaging and distribution

## 7. Technical Considerations
- **Data Integrity:** Use transactions for all operations that modify the block tree. Implement automatic, periodic backups of the database file.
- **Data Portability:** Implement a "Full Export to Markdown" feature that recreates the page hierarchy as folders and block hierarchy as nested Markdown lists.
- **Cross-Platform Compatibility:** Ensure consistent behavior and native look-and-feel across Windows, macOS, and Linux.

## 8. Success Criteria
- **Functional:** All specified features implemented with a block-first philosophy.
- **Quality:** Zero data loss in normal operation; intuitive UI; >80% test coverage.
- **Longevity:** Data remains portable via robust export; minimal external dependencies.

## 9. Extensibility & Interoperability

To facilitate interaction with third-party applications and scripts, the application will provide a secure, local API.

### 9.1 Approach: Local HTTP API
- **Mechanism:** The single executable will, on startup, launch an integrated HTTP server listening only on a configurable localhost port (e.g., `127.0.0.1:41824`).
- **Benefits:**
    - **Single Executable:** Preserves the core project goal of simplicity.
    - **Language-Agnostic:** Any script or application that can make HTTP requests can interact with the outliner.
    - **Secure:** Not exposed to the network by default. An optional API key can be configured for added security.
    - **Robust:** The API enforces the application's business logic, preventing direct database access from corrupting data.

### 9.2 Example API Endpoints
- `POST /api/blocks`: Create a new block on a specific page or as a child of another block.
- `GET /api/blocks/{uuid}`: Retrieve the content and properties of a specific block.
- `GET /api/pages/{title}/mentions`: Get all blocks (with their children) that link to a given page.
- `POST /api/query`: Execute a read-only SQL query against a sanitized view of the database.

---
*This revised specification provides a stronger foundation for a modern outliner application while retaining the journal-oriented features. The block-centric model offers greater flexibility and aligns better with powerful knowledge management workflows.*