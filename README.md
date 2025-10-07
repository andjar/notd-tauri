# Outliner - A Blazing-Fast Cross-Platform Outliner

A powerful, keyboard-first outliner application with infinite nesting, tags, task management, linking/backlinking, and file attachments. Built with Rust for speed and reliability.

## Features

- **Infinite nesting outliner** with expand/collapse
- **Tags system** with filtering and organization
- **Task management** with checkboxes, due dates, and priorities
- **Bidirectional linking** (wiki-style [[links]] and automatic backlinks)
- **Transclusion** (embed content from other notes)
- **File attachments** with deduplication
- **Journal/daily notes** interface
- **Full-text search** with SQLite FTS5
- **Page-based organization**
- **Keyboard-first workflow** with mouse support

## Project Status

✅ **Phase 1 Complete**: Core data model and storage layer
- ✅ Cargo workspace setup
- ✅ SQLite schema with FTS5 support
- ✅ Complete data models (Note, OutlineNode, Tag, Link, Attachment, etc.)
- ✅ Database initialization and migration logic
- ✅ Repository layer with full CRUD operations
- ✅ Comprehensive unit tests

✅ **Phase 2 Complete**: Basic TUI and read-only outliner
- ✅ Working TUI application with Ratatui
- ✅ Hierarchical outline rendering with proper indentation
- ✅ Tree data structure for nested content
- ✅ Event handling and keyboard input
- ✅ Visual distinction for tasks, priorities, and node types
- ✅ Sample data initialization

🚧 **Phase 3 (Next)**: Interactive outliner with full CRUD operations

## Architecture

```
outliner/
├── core/           # Business logic library (UI-agnostic)
│   ├── models/     # Data structures
│   ├── storage/    # SQLite repositories
│   └── schema.sql  # Database schema
├── tui/            # Ratatui interface (Phase 2+)
├── cli/            # Command-line binary
└── tests/          # Integration tests
```

## Technology Stack

- **Language**: Rust 2021 Edition
- **Database**: SQLite with FTS5 for full-text search
- **TUI Framework**: Ratatui (coming in Phase 2)
- **Terminal Backend**: Crossterm

## Quick Start

### Prerequisites
- Rust 1.70+ (stable)
- A terminal that supports Unicode and colors

### Running the Application

```bash
# Run the TUI outliner
cargo run --bin outliner --release

# Or for development
cargo run --bin outliner
```

On first run, the application will:
1. Create `outliner.db` with the schema
2. Initialize with sample data (welcome note)
3. Display the hierarchical outline

**Controls:**
- `q` or `Ctrl+C` - Quit
- More controls coming in Phase 3!

### Building and Testing

```bash
# Build the project
cargo build --release

# Run all tests
cargo test --workspace

# Run with backtrace for debugging
RUST_BACKTRACE=1 cargo test

# Build for specific platform
cargo build --release --target x86_64-unknown-linux-gnu
```

## Testing

All repositories include comprehensive unit tests:

```bash
# Run all tests
cargo test

# Run tests with output
cargo test -- --nocapture

# Run specific test
cargo test test_create_note
```

## Database Schema

The application uses SQLite as the source of truth with the following tables:
- `notes` - Core notes/pages
- `outline_nodes` - Hierarchical outline structure
- `tags` - Tag definitions
- `node_tags` - Tag associations
- `links` - Bidirectional links between notes
- `attachments` - File attachments
- `daily_notes` - Daily note index
- `favorites` - Favorited notes
- `task_status_log` - Task history
- `nodes_fts` - Full-text search index (FTS5)

## Development

### Using the Core Library

The `outliner-core` library provides all data operations:

```rust
use outliner_core::models::*;
use outliner_core::storage::*;

// Create a database
let db = Database::new("outliner.db");
let conn = db.create()?;

// Create a note
let note = Note::new("My First Note".to_string());
NoteRepository::create(&conn, &note)?;

// Add outline nodes
let node = OutlineNode::new(note.id.clone(), None, "Hello, world!".to_string(), 0);
NodeRepository::create(&conn, &node)?;

// Create nested structure
let child = OutlineNode::new(
    note.id.clone(), 
    Some(node.id.clone()), 
    "Child node".to_string(), 
    0
);
NodeRepository::create(&conn, &child)?;

// Search with FTS5
let results = NodeRepository::search(&conn, "world")?;

// Create tasks
let task = OutlineNode::new_task(
    note.id.clone(),
    None,
    "Complete Phase 3".to_string(),
    1,
    Some(TaskPriority::High),
    None,
);
NodeRepository::create(&conn, &task)?;
```

### Running the Example

```bash
cargo run --example basic_usage
```

This demonstrates all core features with detailed output.

## License

MIT

## Next Steps

See `implementation_plan.md` for the complete phased development plan.

