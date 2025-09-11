# Outliner - Block-Based Note-Taking Application

A simple, durable, **block-based** outliner application that combines the simplicity of `nb` with powerful note-taking and journaling features. Built for **simplicity, longevity, and data ownership**.

## Phase 1 - Core Foundation ✅

Phase 1 is **COMPLETE** and includes:

- ✅ **Complete Project Setup**: Tauri 2.0 application with proper structure
- ✅ **Database Integration**: Full SQLite schema with block-centric design
- ✅ **CRUD Operations**: Complete database operations for pages, blocks, and properties
- ✅ **Block System**: Gap-based ordering and hierarchy management
- ✅ **UI Framework**: Three-panel layout with clean, minimal design
- ✅ **Block Editor**: Individual block editing with keyboard shortcuts
- ✅ **Markdown Processing**: Custom syntax for links, properties, and tasks
- ✅ **Daily Pages**: Automatic daily page creation and navigation
- ✅ **Navigation**: Calendar, search, and page management
- ✅ **Block Hierarchy**: Indent/outdent and drag-and-drop manipulation

## Features Implemented

### 🎯 Core Functionality
- **Block-First Architecture**: Everything is a block with hierarchical organization
- **Daily Pages**: Auto-created pages with YYYY-MM-DD format
- **Page Linking**: `[[Page Name]]` syntax with auto-suggestions
- **Properties System**: `key:: value` metadata on any block
- **Task Management**: TODO/DOING/DONE/WAITING/CANCELLED detection
- **Hierarchy Manipulation**: Indent/outdent blocks with Tab/Shift+Tab
- **Drag & Drop**: Move blocks around with visual feedback

### 🖥️ User Interface
- **Three-Panel Layout**: Sidebar, editor, and properties panel
- **Individual Block Editing**: Click any block to edit
- **Keyboard Shortcuts**: Complete set of navigation and editing shortcuts
- **Calendar Navigation**: Visual month view with activity indicators
- **Search**: Full-text search across all content
- **Recent Pages**: Quick access to recently modified pages

### 💾 Data Management
- **SQLite Database**: Embedded database with full-text search
- **Gap-Based Ordering**: Efficient block reordering system
- **Properties**: Flexible key-value metadata system
- **Auto-Save**: Automatic saving of changes
- **Data Integrity**: Transactions and foreign key constraints

## Getting Started

### Prerequisites

To run the Outliner application, you'll need:

1. **Rust** (1.70 or later): https://rustup.rs/
2. **Node.js** (for Tauri CLI): https://nodejs.org/
3. **Tauri CLI**: Install with `cargo install tauri-cli --version "^2.0.0"`

### Development Setup

1. **Clone or navigate to the project directory**
   ```bash
   cd notd
   ```

2. **Install Tauri CLI** (if not already installed)
   ```bash
   cargo install tauri-cli --version "^2.0.0"
   ```

3. **Run the application in development mode**
   ```bash
   cargo tauri dev
   ```

   This will:
   - Compile the Rust backend
   - Start the development server
   - Open the application window

### Building for Production

To create a distributable build:

```bash
cargo tauri build
```

This creates a single executable in `target/release/` that can be distributed.

## Project Structure

```
notd/
├── src-tauri/           # Rust backend
│   ├── src/
│   │   ├── main.rs      # Application entry point
│   │   ├── db/          # Database operations
│   │   ├── commands/    # Tauri commands (API)
│   │   ├── parser/      # Content parsing
│   │   ├── api/         # Local HTTP API (Phase 3)
│   │   └── utils/       # Utility functions
├── src/                 # Frontend (HTML/CSS/JS)
│   ├── index.html       # Main application UI
│   ├── css/main.css     # Complete styling
│   ├── js/
│   │   ├── main.js      # Core application logic
│   │   ├── editor.js    # Block editor component
│   │   ├── outliner.js  # Hierarchy management
│   │   └── components/  # UI components
├── spec.v3.md          # Complete technical specification
├── ARCHITECTURE.md     # System architecture overview
└── README.md          # This file
```

## Key Concepts

### Blocks
- **Fundamental Unit**: Everything is a block of content
- **Hierarchical**: Blocks can have parent-child relationships
- **Editable**: Click any block to edit in-place
- **Movable**: Drag bullets to reorder, use Tab/Shift+Tab to indent/outdent

### Pages
- **Containers**: Pages contain blocks and provide organization
- **Daily Pages**: Auto-created for each day (YYYY-MM-DD format)
- **Hierarchical Paths**: Use `/` for organization (e.g., `projects/website`)
- **Linking**: Reference other pages with `[[Page Name]]` syntax

### Properties
- **Metadata**: Add key-value properties to any block
- **Syntax**: `key:: value` on indented lines under blocks
- **Flexible**: Support for text and list values
- **Queryable**: Search and filter by properties

## Keyboard Shortcuts

### Global Navigation
- `Ctrl/Cmd + N`: New page
- `Ctrl/Cmd + O`: Quick open
- `Ctrl/Cmd + S`: Save page
- `Ctrl/Cmd + D`: Go to today's page
- `Ctrl/Cmd + P`: Command palette
- `Ctrl/Cmd + F`: Search current page

### Block Editing
- `Enter`: Create sibling block
- `Ctrl/Cmd + Enter`: Create child block
- `Tab`: Indent block
- `Shift + Tab`: Outdent block
- `Backspace` (on empty block): Delete or promote block
- `Ctrl/Cmd + ↑/↓`: Move between blocks

## Architecture Highlights

### Block-Centric Design
- All content is stored as blocks with UUIDs for stable references
- Pages are organizational containers pointing to root blocks
- Hierarchy maintained through parent-child relationships with gap-based ordering

### Performance
- **SQLite with FTS5**: Full-text search across all content
- **Efficient Ordering**: Gap-based system allows fast insertions
- **Indexed Queries**: Optimized database schema with proper indexes
- **Mock API**: Development mode works without Rust compilation

### Longevity
- **Single Executable**: No external dependencies in production
- **Plain Text Storage**: Content stored in human-readable format
- **Standard Formats**: Uses Markdown for rich text formatting
- **Export Friendly**: Data designed for easy export and portability

## Development Features

### Mock API Mode
When running without Tauri (in browser), the application uses a mock API that simulates all backend functionality, allowing for frontend development without Rust compilation.

### Component Architecture
- **Modular Design**: Separate components for editor, outliner, calendar, search, and sidebar
- **Event-Driven**: Clean separation between UI and data management
- **Extensible**: Easy to add new features and components

## Next Steps (Future Phases)

### Phase 2: Essential Features (4-6 weeks)
- Advanced search with filters
- Template system enhancement
- Improved task management
- Better mobile support

### Phase 3: Advanced Features (6-8 weeks)
- File attachments with drag & drop
- SQL query blocks for data analysis
- Block transclusion system
- Local HTTP API for integrations

### Phase 4: Polish & Optimization (3-4 weeks)
- Graph view visualization
- Performance optimization
- Cross-platform packaging
- Documentation and tutorials

## Contributing

The codebase is designed for maintainability and extensibility. Key principles:

1. **Block-First**: All features should work with the block model
2. **Simplicity**: Prefer simple solutions over complex ones
3. **Performance**: Target smooth operation with 10K+ blocks
4. **Longevity**: Minimize dependencies and use standard formats

## Technical Specifications

For complete technical details, see:
- `spec.v3.md` - Complete implementation specification
- `ARCHITECTURE.md` - High-level system design
- Database schema in `src-tauri/src/db/schema.rs`

## License

This project is built for longevity and simplicity, focusing on user data ownership and application durability.
