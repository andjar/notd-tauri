# Phased Implementation Plan: Cross-Platform Outliner with TUI

## Project Overview

A blazing-fast, cross-platform outliner application with infinite nesting, tags, task management, linking/backlinking, and file attachments. Primary interface: Terminal User Interface (TUI) using Ratatui. Storage: SQLite database as the source of truth, with Markdown export for portability.

## Core Features

### Must-Have Features
- **Infinite nesting outliner** with expand/collapse
- **Tags system** with filtering and organization
- **Simple task management** (checkboxes, due dates, priorities)
- **Task state change logging** (for log book/history)
- **Bidirectional linking** (wiki-style [[links]] and automatic backlinks)
- **Transclusion** (embed content from other notes)
- **File attachments** (images, PDFs, documents)
- **Journal/daily notes** interface with calendar
- **Full-text search** with SQLite FTS5
- **Page-based organization** (named documents/notes)
- **Keyboard-first workflow** with mouse support

### Nice-to-Have Features
- Export to markdown, HTML, JSON
- Quick capture/inbox
- Note templates
- Saved searches/filters
- Vim-style keybindings option

## Application Layout

```
┌─────────────────────────────────────────────────────────────────┐
│ [Outliner] [q:Quit] [/:Search] [n:New] [Ctrl+S:Save]            │
├──────────────┬──────────────────────────────────────────────────┤
│              │  # 2024-10-07 Daily Note                         │
│  CALENDAR    │                                                  │
│  ─────────   │  ▼ Project Planning                             │
│  Oct 2024    │    ▶ Q4 Goals                                   │
│  Su Mo Tu We │    ▼ Team Meeting Notes                         │
│      1  2  3 │      • [x] Review budget                        │
│   4  5  6 [7]│      • [ ] Schedule follow-up                   │
│   8  9 10 11 │      • Discussed roadmap #planning              │
│              │                                                  │
│  FAVORITES   │    Attachments:                                 │
│  ────────    │    [📎] budget.xlsx (18 KB)                     │
│  ⭐ Inbox    │    [📎] mockup.png (245 KB)                     │
│  ⭐ Projects │                                                  │
│  ⭐ Ideas    │  ▶ Research Notes                               │
│              │    Links: [[Product Strategy]], [[Q3 Review]]  │
│  PAGES       │                                                  │
│  ─────       │  ▼ Quick Thoughts                               │
│  • Home      │    • Ideas for next sprint                      │
│  • Work      │    • Need to refactor auth module               │
│  • Personal  │                                                  │
│              │  Backlinks (2):                                 │
│  TAGS        │  • [[Weekly Planning]] mentions this note       │
│  ────        │  • [[Team Retrospective]] links here            │
│  #work (45)  │                                                  │
│  #ideas (23) │                                                  │
│  #planning   │                                                  │
│              │                                                  │
└──────────────┴──────────────────────────────────────────────────┘
│ Status: Saved | 234 notes | 45 links | Ctrl+P: Pages          │
└─────────────────────────────────────────────────────────────────┘
```

### Layout Components

**Left Sidebar (collapsible):**
- **Calendar widget**: Month view, highlight today, click/navigate to daily notes
- **Favorites**: Quick access to important notes/pages
- **Pages list**: All named documents
- **Tags panel**: Tag cloud with counts, click to filter

**Main Area:**
- **Outliner view**: Hierarchical bullet-point structure
- **Task indicators**: Checkboxes for tasks `[ ]` / `[x]`
- **Inline tags**: `#tag` visible in content
- **Links**: `[[Page Name]]` highlighted and clickable
- **Attachments section**: List of files with icons and sizes
- **Backlinks panel**: Shows which notes link to current note
- **Breadcrumbs**: Show current location in hierarchy

**Bottom Status Bar:**
- Save status, note count, search mode, keybinding hints

## Technology Stack

### Core Technologies
- **Language**: Rust (for performance, safety, and longevity)
- **TUI Framework**: Ratatui (terminal UI with mouse support)
- **Terminal Backend**: Crossterm (cross-platform terminal control)
- **Database**: SQLite (source of truth for all data)
- **Storage Format**: SQLite database file.

### Key Dependencies
- `ratatui` - Terminal UI framework
- `crossterm` - Terminal abstraction layer
- `rusqlite` - SQLite bindings
- `pulldown-cmark` - Markdown parsing (for import/rendering)
- `tantivy` or `rusqlite` FTS5 - Full-text search
- `serde` + `serde_json` - Serialization
- `chrono` - Date/time handling
- `walkdir` - File system traversal (for attachments)
- `viuer` or `ratatui-image` - Optional inline images

### Optional GUI Layer (Future)
- **Tauri v2** + React/Svelte for desktop GUI
- **Tauri Mobile** or PWA for mobile access

## Architecture

```
┌─────────────────────────────────────────┐
│  Interface Layer (Swappable)            │
│  ├─ Ratatui TUI (primary)               │
│  └─ Tauri GUI (future)                  │
└─────────────────────────────────────────┘
                 ↓
┌─────────────────────────────────────────┐
│  Core Business Logic (Rust Library)     │
│  ├─ Outliner operations                 │
│  ├─ Link/backlink resolution            │
│  ├─ Tag management                      │
│  ├─ Task tracking                       │
│  ├─ Search indexing                     │
│  └─ Transclusion engine                 │
└─────────────────────────────────────────┘
                 ↓
┌─────────────────────────────────────────┐
│  Storage Layer                          │
│  ├─ SQLite database (source of truth)     │
│  │   └─ All notes, tags, links, etc.    │
│  ├─ Markdown files (export/import)      │
│  │   └─ Recreatable from database        │
│  └─ Attachments folder                  │
│      └─ Content-addressed storage       │
└─────────────────────────────────────────┘
```

### File Structure
```
outliner-workspace/
├── attachments/
│   └── [hash-based-filenames]
│       ├── abc123def456.png
│       └── 789ghi012jkl.pdf
└── .outliner/
    ├── index.db (SQLite source of truth)
    └── config.toml
```

### Exported Markdown Format Example

*This shows an example of the format the application will export to. The primary data is stored in SQLite.*

```markdown
---
id: unique-id-123
title: Project Planning
created: 2024-10-07T10:30:00Z
modified: 2024-10-07T14:20:00Z
tags: [work, planning, q4]
---

# Project Planning

- Q4 Goals
  - [ ] Launch new feature @high @2024-10-15
  - [x] Complete design review
  - Research phase
    - See [[Market Analysis]] for details
    - ![[Competitor Research#Summary]] <!-- transclusion -->

- Meeting Notes #meeting
  - Discussed roadmap with team
  - Attachments:
    - [budget.xlsx](../attachments/abc123.xlsx)
    - ![mockup](../attachments/def456.png)

Links: [[Product Strategy]], [[Q3 Review]]
```

### SQLite Schema (Index/Cache)
```sql
-- Core notes table (each note is a page/document)
CREATE TABLE notes (
    id TEXT PRIMARY KEY,
    title TEXT NOT NULL,
    created_at INTEGER NOT NULL,
    modified_at INTEGER NOT NULL
);

-- Full-text search for outline nodes
CREATE VIRTUAL TABLE nodes_fts USING fts5(
    content,
    content='outline_nodes',
    content_rowid='id'
);

-- Tags
CREATE TABLE tags (
    id INTEGER PRIMARY KEY,
    name TEXT UNIQUE NOT NULL,
    color TEXT
);

CREATE TABLE note_tags (
    note_id TEXT,
    tag_id INTEGER,
    FOREIGN KEY(note_id) REFERENCES notes(id),
    FOREIGN KEY(tag_id) REFERENCES tags(id)
);

-- Links (bidirectional)
CREATE TABLE links (
    id INTEGER PRIMARY KEY,
    source_note_id TEXT NOT NULL,
    target_note_id TEXT NOT NULL,
    link_text TEXT,
    link_type TEXT, -- 'wiki', 'transclusion', 'attachment'
    FOREIGN KEY(source_note_id) REFERENCES notes(id)
    -- target_note_id may not exist yet, so no FK
);

-- Outliner structure (nodes)
CREATE TABLE outline_nodes (
    id TEXT PRIMARY KEY,
    note_id TEXT NOT NULL,
    parent_node_id TEXT,
    content TEXT,
    position INTEGER, -- for ordering siblings
    is_task BOOLEAN DEFAULT 0,
    task_completed BOOLEAN DEFAULT 0,
    task_priority TEXT, -- 'low', 'medium', 'high'
    task_due_date INTEGER,
    FOREIGN KEY(note_id) REFERENCES notes(id)
);

-- Attachments
CREATE TABLE attachments (
    id TEXT PRIMARY KEY,
    note_id TEXT NOT NULL,
    filename TEXT NOT NULL,
    filepath TEXT NOT NULL,
    mime_type TEXT,
    size_bytes INTEGER,
    hash TEXT, -- for deduplication
    created_at INTEGER,
    FOREIGN KEY(note_id) REFERENCES notes(id)
);

-- Daily notes index
CREATE TABLE daily_notes (
    date TEXT PRIMARY KEY, -- YYYY-MM-DD
    note_id TEXT UNIQUE,
    FOREIGN KEY(note_id) REFERENCES notes(id)
);

-- Favorites
CREATE TABLE favorites (
    note_id TEXT PRIMARY KEY,
    position INTEGER,
    FOREIGN KEY(note_id) REFERENCES notes(id)
);

-- Log of task status changes
CREATE TABLE task_status_log (
    id INTEGER PRIMARY KEY,
    node_id TEXT NOT NULL,
    status TEXT NOT NULL, -- e.g., 'created', 'completed', 'in_progress'
    timestamp INTEGER NOT NULL,
    FOREIGN KEY(node_id) REFERENCES outline_nodes(id)
);
```

## Detailed Phased Implementation Plan

---

### Phase 1: Project Setup & Core Data Model (Week 1)

**Goal**: Establish the project foundation, including the database schema, data models, and basic project structure. The application will not have a UI at this stage, but the core data layer will be functional.

#### Deliverables
- [ ] Complete Rust project setup with Cargo workspace and all initial dependencies.
- [ ] Finalized SQLite schema committed to a `.sql` file.
- [ ] Database initialization and migration logic using `rusqlite`.
- [ ] Core Rust structs (`Note`, `OutlineNode`, `Tag`, etc.) that map directly to the database schema.
- [ ] A `storage` module (repository pattern) with CRUD functions for all data models.
- [ ] Unit tests for all CRUD operations to ensure data integrity.

#### Technical Tasks
1.  **Project Setup**: Initialize Cargo workspace (`core`, `tui`, `cli`). Add `rusqlite`, `serde`, `chrono`, etc.
2.  **Schema Definition**: Write the complete `schema.sql`.
3.  **DB Connection**: Create a database manager that handles connections and runs initial setup/migrations.
4.  **Data Models**: Define `structs` in `core/models` with `serde` support.
5.  **Repository Layer**: Implement `core/storage` with functions like `create_note`, `get_node`, `update_node_content`, etc.
6.  **Unit Testing**: Write tests for the storage layer, ensuring all database interactions work as expected.

#### Success Criteria
- The project compiles successfully.
- A command-line tool or integration test can successfully create and read from the SQLite database.
- Database schema is stable and well-documented.
- Core data logic is covered by unit tests.

---

### Phase 2: Basic TUI & Read-Only Outliner (Weeks 2-3)

**Goal**: Create a visible TUI that can read from the SQLite database and display an outline. No editing capabilities yet. This phase focuses on rendering data.

#### Deliverables
- [ ] Basic TUI application shell using Ratatui and Crossterm.
- [ ] TUI layout with placeholders for sidebar and main content area.
- [ ] Logic to fetch a hardcoded note and its outline nodes from the database.
- [ ] A tree-like data structure to hold the outline for rendering.
- [ ] A TUI component that can render the outline structure with proper nesting and indentation.
- [ ] Basic keyboard input for quitting the application (`q` or `Ctrl+C`).

#### Technical Tasks
1.  **TUI Setup**: Initialize the terminal, create the main loop in `tui/app.rs`.
2.  **Layout**: Use Ratatui layout managers to define the main UI areas.
3.  **Data Fetching**: Connect the TUI to the `storage` module to load a note on startup.
4.  **State Management**: Create a state struct in the TUI to manage the currently displayed outline.
5.  **Rendering Logic**: Write the `ui` function that draws the outliner view.
6.  **Event Handling**: Implement a basic event loop to handle quit signals.

#### Success Criteria
- The TUI application starts and displays a hardcoded outline from the database.
- The outline shows correct hierarchy and nesting.
- The application can be closed gracefully.

---

### Phase 3: Interactive Outliner (Weeks 4-5)

**Goal**: Implement full CRUD (Create, Read, Update, Delete) functionality within the TUI. Users should be able to navigate and modify the outline.

#### Deliverables
- [ ] A cursor or selection indicator in the outliner view.
- [ ] Keyboard navigation: move up/down the outline.
- [ ] Node editing: Enter an "edit mode" to change the text of a node.
- [ ] Node creation: Add new nodes to the outline.
- [ ] Node deletion: Remove nodes from the outline.
- [ ] Node manipulation: Indent/outdent and move nodes up/down in the hierarchy.
- [ ] Expand/collapse parent nodes.
- [ ] All changes are immediately persisted to the SQLite database.

#### Technical Tasks
1.  **Cursor Logic**: Add cursor position to the TUI state.
2.  **Navigation Events**: Handle arrow keys to update the cursor and scroll the view.
3.  **Edit Mode**: Create a separate input handler and UI state for editing node text.
4.  **Database Integration**: Call the appropriate `storage` functions for create, update, and delete operations on user action.
5.  **Hierarchy Logic**: Implement the business logic for moving nodes and restructuring the tree.
6.  **State Update**: Ensure the TUI state is refreshed from the database or updated locally after every change.

#### Success Criteria
- A user can create a complex, nested outline from scratch.
- All changes are saved correctly and visible after restarting the app.
- Navigation and editing feel responsive.

---

### Phase 4: Page Management & Navigation (Week 6)

**Goal**: Introduce the concept of multiple pages (notes) and allow users to switch between them.

#### Deliverables
- [ ] A "Pages" list widget in the sidebar, populated from the `notes` table.
- [ ] A command or keybinding (e.g., `Ctrl+P`) to open a fuzzy-finder/quick-switcher for pages.
- [ ] Ability to create new pages.
- [ ] Ability to delete pages.
- [ ] The main outliner view updates to show the content of the selected page.

#### Technical Tasks
1.  **Pages Widget**: Create a new Ratatui component for the list of pages.
2.  **Data Fetching**: Query the database for all notes and display them.
3.  **Switcher UI**: Implement a popup or new screen for the page switcher.
4.  **State Management**: Update the application state to track the `current_note_id`.
5.  **CRUD for Notes**: Wire up the UI to the `create_note` and `delete_note` storage functions.

#### Success Criteria
- Can create multiple distinct notes/pages.
- Can switch between pages, and the correct outline is displayed for each.
- Page list updates correctly when new pages are created or deleted.

---

### Phase 5: Search, Tags, and Links (Weeks 7-8)

**Goal**: Make notes discoverable and interconnected.

#### Deliverables
- [ ] A search input mode, triggered by `/`.
- [ ] A search results view that displays matching nodes and their context.
- [ ] SQLite FTS5 integration for fast full-text search across all outline nodes.
- [ ] Parsing of `#tags` from node content and storing them in the database.
- [ ] A "Tags" panel in the sidebar, showing all tags and their counts.
- [ ] Filtering the pages list by clicking or selecting a tag.
- [ ] Parsing of `[[Wiki Links]]` from node content.
- [ ] Storing link relationships in the `links` table.
- [ ] A "Backlinks" panel in the outliner view, showing pages that link to the current page.

#### Technical Tasks
1.  **FTS Setup**: Configure `nodes_fts` virtual table and write search queries.
2.  **Search UI**: Create a UI component for the search bar and results list.
3.  **Parsing Logic**: Implement regex or a simple parser to extract tags and links as text is updated.
4.  **Tag Storage**: Implement logic to update `tags` and `note_tags` tables.
5.  **Link Storage**: Implement logic to update the `links` table.
6.  **Backlink Query**: Write a DB query to find all notes linking to the current `note_id`.
7.  **UI Integration**: Add the Tags and Backlinks panels to the UI.

#### Success Criteria
- Full-text search is fast and accurate.
- Tags are correctly identified and associated with notes.
- Clicking a tag filters the list of notes.
- Links are identified, and backlinks are displayed correctly.

---

### Phase 6: Tasks, Calendar & Daily Notes (Week 9)

**Goal**: Add task management and a journaling interface.

#### Deliverables
- [ ] Parsing of `[ ]` and `[x]` syntax to mark nodes as tasks.
- [ ] A keybinding to toggle the completion status of a task.
- [ ] Logging of task status changes to a history table.
- [ ] A calendar widget in the sidebar.
- [ ] Ability to navigate the calendar and select a date.
- [ ] Selecting a date creates or navigates to a "Daily Note" for that day.

#### Technical Tasks
1.  **Task Parsing**: Add logic to identify tasks and update the `is_task` and `task_completed` fields in the `outline_nodes` table.
2.  **Task Status Logging**: Update task toggling logic to insert a record into `task_status_log` on every change.
3.  **Calendar Widget**: Implement a calendar view component using Ratatui.
4.  **Daily Note Logic**: Implement the `get_or_create_daily_note` function.
5.  **Date Navigation**: Handle input for moving between days/months in the calendar.
6.  **Task Query**: Write a DB query to select all tasks.

#### Success Criteria
- Can create and complete tasks within any outline.
- Every task completion or reversal is recorded in the history log.
- The calendar correctly highlights the current day.
- Selecting a date opens the corresponding daily note.

---

### Phase 7: Attachments and Transclusion (Week 10)

**Goal**: Allow associating external files with notes and embedding content between notes.

#### Deliverables
- [ ] A command to attach a file to the current note.
- [ ] Files are copied into the `attachments` directory with a hash-based name.
- [ ] An "Attachments" section is displayed in the UI for the current note.
- [ ] A command to open an attachment in its default external application.
- [ ] Parsing of transclusion syntax, e.g., `![[Note Title#Node ID]]`.
- [ ] Transcluded content is rendered read-only within the outliner view.

#### Technical Tasks
1.  **File Logic**: Implement file hashing and copying.
2.  **Attachment Storage**: Update the `attachments` table with file metadata.
3.  **Attachment UI**: Create the UI list component.
4.  **External Opener**: Use a crate like `opener` to open files.
5.  **Transclusion Parser**: Extend the link parser to handle transclusion syntax.
6.  **Transclusion Renderer**: Implement logic to fetch and display transcluded content from the database.

#### Success Criteria
- Can attach and open files.
- Transcluded content from another note is displayed correctly.

---

### Phase 8: Polish, Export & Configuration (Weeks 11-12)

**Goal**: Refine the user experience, add configuration options, and implement export functionality.

#### Deliverables
- [ ] Mouse support: clicking on pages, tags, and outline nodes.
- [ ] A `config.toml` file for basic settings (e.g., color schemes).
- [ ] An "Export to Markdown" command that generates a set of `.md` files from the database.
- [ ] A "Favorites" system to pin important notes to the sidebar.
- [ ] A "Log Book" view to see the history of a specific task.
- [ ] Basic Undo/Redo functionality for text editing and node operations.
- [ ] Comprehensive error handling and user-facing messages.

#### Technical Tasks
1.  **Mouse Events**: Add mouse event handling to the TUI loop.
2.  **Config Logic**: Use a crate like `config` to read and parse the TOML file.
3.  **Markdown Exporter**: Write a module that queries the database and constructs Markdown file content.
4.  **Favorites Storage**: Implement the `favorites` table and associated UI.
5.  **Log Book UI**: Build a UI component to display the task history from the `task_status_log` table.
6.  **Undo Stack**: Implement a command pattern or similar to manage an undo/redo stack.
7.  **Error Handling**: Replace all `unwrap()` and `expect()` calls with robust error handling.

#### Success Criteria
- The application is stable and handles errors gracefully.
- Key features are configurable.
- The entire database can be exported to a human-readable Markdown format.
- Core editing actions can be undone.

---

### Phase 9: Future Enhancements (Post-MVP)

**Goal**: Extend capabilities and reach based on user feedback.

#### Possible Additions
- [ ] Tauri GUI layer (desktop)
- [ ] Sync service (multi-device)
- [ ] PWA or Tauri Mobile (mobile access)
- [ ] Plugin system (extensibility)
- [ ] Git integration (version control)
- [ ] Collaborative editing
- [ ] Advanced query language
- [ ] Graph view visualization
- [ ] Note templates
- [ ] Vim keybindings mode

#### Strategic Decisions
- **When to add GUI**: After core TUI is stable and feature-complete
- **Mobile strategy**: PWA vs Tauri Mobile (decide based on user needs)
- **Sync approach**: File-based (Syncthing/Dropbox) vs custom server

## Development Guidelines

### Code Organization
```
outliner/
├── Cargo.toml
├── core/           # Business logic library (UI-agnostic)
│   ├── models/     # Data structures
│   ├── storage/    # SQLite + file operations
│   ├── parser/     # Markdown parsing
│   └── indexer/    # Search, links, tags
├── tui/            # Ratatui interface
│   ├── app.rs      # Main app state
│   ├── ui/         # UI components
│   └── events.rs   # Input handling
├── cli/            # Command-line binary
└── tests/
    ├── integration/
    └── fixtures/
```

### Testing Strategy
- **Unit tests**: Core logic, parsers, data structures
- **Integration tests**: Database operations, file I/O
- **Manual testing**: TUI interaction, keyboard/mouse
- **Test fixtures**: Sample markdown files and databases

### Performance Targets
- Startup: < 100ms with 1000 notes
- Search: < 50ms for full-text query
- Note switching: < 10ms
- UI rendering: 60 FPS minimum

### Longevity Principles
1. **Data format first**: Markdown + SQLite are primary concerns
2. **No vendor lock-in**: Must be able to export/rebuild everything
3. **Minimal dependencies**: Prefer std library when possible
4. **Documented schema**: SQLite schema fully documented
5. **Version migrations**: Plan for schema evolution
6. **Export capability**: Can export the entire database to Markdown files

## Risk Mitigation

### Potential Risks
1. **Ratatui abandonment**: Mitigated by using stable crossterm foundation
2. **Dependency conflicts**: Use cargo vendor, lock dependencies
3. **Terminal compatibility**: Test on major terminals, fallback to basics
4. **Performance with large datasets**: Profile early, optimize DB queries
5. **Data corruption**: Implement backups, validation, atomic DB transactions

### Backup Strategy
- Automatic SQLite database backups
- Export to Markdown as a human-readable backup
- Validation on startup

## Success Metrics

### MVP Success (End of Phase 8)
- Can manage 1000+ notes smoothly
- All core features working
- Stable on Windows, macOS, Linux
- Positive feedback from 10 test users
- < 5 critical bugs

### Long-term Success
- Active user base (100+ daily users)
- Community contributions (plugins, themes)
- Used reliably for 1+ year without data loss
- Alternative interfaces built on core library

## Timeline Summary

| Phase | Duration | Key Milestone |
|-------|----------|---------------|
| Phase 1 | 1 week   | Project setup and data model complete |
| Phase 2 | 2 weeks  | Read-only TUI displays outline |
| Phase 3 | 2 weeks  | Fully interactive outliner |
| Phase 4 | 1 week   | Page management and navigation |
| Phase 5 | 2 weeks  | Search, tags, and links functional |
| Phase 6 | 1 week   | Task management and daily notes |
| Phase 7 | 1 week   | Attachments and transclusion |
| Phase 8 | 2 weeks  | Polished, configurable MVP with export |
| **Total** | **12 weeks** | **Feature-complete v1.0** |
| Phase 9 | Ongoing  | GUI, mobile, advanced features |

## Getting Started

### Prerequisites
- Rust 1.70+ (stable)
- SQLite 3.35+
- Modern terminal emulator

### Initial Setup
```bash
# Create project
cargo new outliner --lib
cd outliner

# Add dependencies
cargo add ratatui crossterm rusqlite serde chrono

# Create workspace structure
mkdir -p core/{models,storage,parser}
mkdir -p tui/{ui,components}

# Initialize git
git init
echo "target/" > .gitignore
echo "*.db" >> .gitignore
echo "attachments/" >> .gitignore
```

### First Sprint (Week 1)
1. Set up project structure
2. Create basic data models for notes and outline nodes
3. Implement SQLite database creation and schema setup
4. Build minimal TUI (hello world)
5. End-of-sprint demo: Show a hardcoded note from the database in the terminal
