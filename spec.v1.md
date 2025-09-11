# Outliner Application Technical Specification

## 1. Project Overview

### 1.1 Purpose
Build a simple, durable outliner application that combines the simplicity of nb with powerful note-taking and journaling features. The application must be deployable as a single executable with maximum longevity.

### 1.2 Core Philosophy
- **Simplicity First**: Minimal UI, essential features only
- **Longevity**: Built to last decades without dependency rot
- **Offline-First**: Complete functionality without internet
- **Data Ownership**: User controls their data completely
- **Plain Text Foundation**: Human-readable storage format

### 1.3 Target Users
- Knowledge workers seeking simple, reliable note-taking
- Researchers and writers who need structured thinking tools
- Users migrating from bloated applications like Logseq
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
│   │   ├── crypto/      # Encryption utilities
│   │   ├── parser/      # Markdown parsing
│   │   └── commands/    # Tauri commands
├── src/                 # Frontend assets
│   ├── index.html
│   ├── css/
│   ├── js/
│   └── assets/
└── tauri.conf.json     # Tauri configuration
```

### 2.3 Data Storage Architecture

#### 2.3.1 SQLite Schema
```sql
-- Core pages table
CREATE TABLE pages (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    title TEXT NOT NULL UNIQUE,
    path TEXT NOT NULL UNIQUE,  -- forward-slash hierarchy
    content TEXT NOT NULL,      -- markdown content
    content_encrypted TEXT,     -- encrypted content if applicable
    is_encrypted BOOLEAN DEFAULT FALSE,
    is_daily_page BOOLEAN DEFAULT FALSE,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    template_id INTEGER REFERENCES templates(id)
);

-- Page links and backlinks
CREATE TABLE page_links (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    source_page_id INTEGER NOT NULL REFERENCES pages(id) ON DELETE CASCADE,
    target_page_id INTEGER NOT NULL REFERENCES pages(id) ON DELETE CASCADE,
    link_text TEXT NOT NULL,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(source_page_id, target_page_id, link_text)
);

-- Templates
CREATE TABLE templates (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL UNIQUE,
    content TEXT NOT NULL,
    description TEXT,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

-- Tasks
CREATE TABLE tasks (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    page_id INTEGER NOT NULL REFERENCES pages(id) ON DELETE CASCADE,
    content TEXT NOT NULL,
    status TEXT NOT NULL DEFAULT 'TODO',  -- TODO, DOING, DONE, WAITING, CANCELLED
    priority TEXT,
    assignee TEXT,
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

-- Full-text search
CREATE VIRTUAL TABLE pages_fts USING fts5(
    title, content, 
    content='pages', 
    content_rowid='id'
);

-- Indexes for performance
CREATE INDEX idx_pages_path ON pages(path);
CREATE INDEX idx_pages_daily ON pages(is_daily_page, created_at);
CREATE INDEX idx_tasks_status ON tasks(status);
CREATE INDEX idx_tasks_due_date ON tasks(due_date);
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

**Implementation Details:**
```rust
// Daily page creation logic
fn get_or_create_daily_page(date: NaiveDate) -> Result<Page> {
    let title = date.format("%Y-%m-%d").to_string();
    
    if let Some(page) = db.get_page_by_title(&title)? {
        Ok(page)
    } else {
        let template = db.get_template_by_name("daily")?;
        let content = template.map(|t| t.content).unwrap_or_default();
        
        db.create_page(Page {
            title,
            path: format!("journal/{}", title),
            content,
            is_daily_page: true,
            ..Default::default()
        })
    }
}
```

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

**Implementation:**
```rust
fn create_hierarchical_page(path: &str, title: &str) -> Result<Page> {
    let segments: Vec<&str> = path.split('/').collect();
    
    // Ensure all parent pages exist
    for i in 1..segments.len() {
        let parent_path = segments[0..i].join("/");
        let parent_title = segments[i-1];
        
        if !db.page_exists(&parent_path)? {
            db.create_page(Page {
                title: parent_title.to_string(),
                path: parent_path,
                content: format!("# {}\n\nAuto-created parent page.", parent_title),
                ..Default::default()
            })?;
        }
    }
    
    // Create the target page
    db.create_page(Page {
        title: title.to_string(),
        path: path.to_string(),
        content: format!("# {}\n\n", title),
        ..Default::default()
    })
}
```

#### 3.1.4 Template System
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

### 3.2 Rich Note-Taking

#### 3.2.1 Markdown Support
**Supported Syntax:**
- Headers (`#`, `##`, `###`)
- Lists (ordered, unordered, nested)
- Links (`[text](url)`)
- Images (`![alt](path)`)
- Code blocks with syntax highlighting
- Tables
- Blockquotes
- Bold, italic, strikethrough

**Live Preview:**
- Split-pane editor (edit/preview)
- Synchronized scrolling
- Toggle between edit/preview modes

#### 3.2.2 Page Linking
**Syntax:** `[[Page Name]]` or `[[Page Name|Display Text]]`

**Features:**
- Auto-complete during typing
- Create pages on-the-fly when linking
- Backlink tracking and display
- Visual link indicators (existing vs. new)

**Implementation:**
```rust
fn extract_page_links(content: &str) -> Vec<PageLink> {
    let re = Regex::new(r"\[\[([^|\]]+)(?:\|([^\]]+))?\]\]").unwrap();
    
    re.captures_iter(content)
        .map(|cap| PageLink {
            target: cap[1].to_string(),
            display_text: cap.get(2).map(|m| m.as_str().to_string()),
        })
        .collect()
}
```

#### 3.2.3 Transclusion
**Syntax:** `{{transclude:Page Name}}` or `{{transclude:Page Name#Header}}`

**Features:**
- Embed entire pages or sections
- Live updates when source content changes
- Prevent infinite recursion
- Clear visual indication of transcluded content

#### 3.2.4 SQL Queries
**Syntax:** `SQL{SELECT title FROM pages WHERE path LIKE 'projects/%'}`

**Features:**
- Execute queries against the database
- Render results as tables or lists
- Cache query results with TTL
- Security restrictions (read-only queries)

**Safety Implementation:**
```rust
fn execute_sql_query(query: &str) -> Result<QueryResult> {
    // Parse and validate query
    let parsed = sqlparser::parse_sql(&MySqlDialect {}, query)?;
    
    // Ensure only SELECT statements
    if !parsed.iter().all(|stmt| matches!(stmt, Statement::Query(_))) {
        return Err("Only SELECT queries allowed".into());
    }
    
    // Execute with read-only connection
    let conn = db.get_readonly_connection()?;
    let mut stmt = conn.prepare(query)?;
    
    // Execute and format results
    let results = stmt.query_map([], |row| {
        // Convert row to JSON-serializable format
    })?;
    
    Ok(QueryResult::from_rows(results))
}
```

### 3.3 Security & Privacy

#### 3.3.1 Client-Side Encryption
**Syntax:** `ENC{encrypted content here}`

**Implementation:**
- AES-256-GCM encryption
- PBKDF2 key derivation (100,000 iterations)
- Per-page encryption with master password
- Salt stored with encrypted content

```rust
use aes_gcm::{Aes256Gcm, Key, Nonce};
use argon2::Argon2;

struct Encryption {
    cipher: Aes256Gcm,
}

impl Encryption {
    fn encrypt_content(&self, content: &str, password: &str) -> Result<String> {
        let salt = generate_random_bytes(32);
        let key = self.derive_key(password, &salt)?;
        let nonce = generate_random_bytes(12);
        
        let ciphertext = self.cipher.encrypt(&nonce.into(), content.as_bytes())?;
        
        // Format: salt(32) + nonce(12) + ciphertext
        let encrypted = [salt, nonce, ciphertext].concat();
        Ok(base64::encode(encrypted))
    }
    
    fn decrypt_content(&self, encrypted: &str, password: &str) -> Result<String> {
        let data = base64::decode(encrypted)?;
        let (salt, rest) = data.split_at(32);
        let (nonce, ciphertext) = rest.split_at(12);
        
        let key = self.derive_key(password, salt)?;
        let plaintext = self.cipher.decrypt(&nonce.into(), ciphertext)?;
        
        Ok(String::from_utf8(plaintext)?)
    }
}
```

#### 3.3.2 Password Protection
**Features:**
- Per-page password protection
- Password hint storage (encrypted)
- Session-based password caching
- Auto-lock after inactivity

### 3.4 Task Management

#### 3.4.1 Task States
**Supported States:**
- `TODO` - Not started (⬜)
- `DOING` - In progress (🔄)
- `DONE` - Completed (✅)
- `WAITING` - Blocked/waiting (⏳)
- `CANCELLED` - Cancelled (❌)

**Syntax:**
```markdown
- [ ] TODO item
- [/] DOING item  
- [x] DONE item
- [w] WAITING item
- [c] CANCELLED item
```

#### 3.4.2 Task Properties
**Syntax:**
```markdown
- [ ] Task description
  - priority:: high
  - assignee:: john.doe
  - due:: 2024-12-31
  - tags:: #urgent #client-work
```

**Property Types:**
- `priority`: low, medium, high, critical
- `assignee`: free text
- `due`: ISO date format
- `tags`: space-separated tags with #
- `estimate`: time estimate (1h, 30m, 2d)

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

**Implementation:**
```rust
fn update_task_status(task_id: i64, new_status: TaskStatus) -> Result<()> {
    let task = db.get_task(task_id)?;
    
    // Log state change
    db.insert_state_change(TaskStateChange {
        task_id,
        old_status: task.status.clone(),
        new_status: new_status.clone(),
        changed_at: Utc::now(),
    })?;
    
    // Update task
    let mut updated_task = task;
    updated_task.status = new_status;
    updated_task.updated_at = Utc::now();
    
    if matches!(new_status, TaskStatus::Done) {
        updated_task.completed_at = Some(Utc::now());
    }
    
    db.update_task(updated_task)
}
```

## 4. User Interface Specifications

### 4.1 Application Layout
```
┌─────────────────────────────────────────────────────────────┐
│ Menu Bar                                                    │
├─────────────┬───────────────────────────────┬───────────────┤
│             │                               │               │
│   Sidebar   │        Main Editor           │   Right Panel │
│             │                               │               │
│ - Calendar  │  ┌─────────────────────────┐  │ - Backlinks   │
│ - Pages     │  │                         │  │ - Tasks       │
│ - Tasks     │  │     Content Area        │  │ - Outline     │
│ - Tags      │  │                         │  │ - Properties  │
│             │  └─────────────────────────┘  │               │
│             │                               │               │
├─────────────┼───────────────────────────────┼───────────────┤
│             │        Status Bar             │               │
└─────────────┴───────────────────────────────┴───────────────┘
```

### 4.2 Key UI Components

#### 4.2.1 Sidebar Navigation
- **Calendar Widget**: Mini calendar with activity indicators
- **Page Tree**: Hierarchical page navigation
- **Recent Pages**: Last 10 accessed pages
- **Starred Pages**: User-favorited pages
- **Task Overview**: Quick task status counts

#### 4.2.2 Main Editor
- **Split View**: Edit/Preview panes
- **Toolbar**: Format buttons, insert options
- **Search Bar**: Global search with autocomplete
- **Breadcrumbs**: Current page hierarchy

#### 4.2.3 Right Panel (Collapsible)
- **Backlinks**: Pages linking to current page
- **Outline**: Document structure/headers
- **Page Properties**: Metadata editing
- **Task Panel**: Page-specific tasks

### 4.3 Keyboard Shortcuts
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

### 5.1 Database Performance
- **Page Load Time**: < 100ms for typical pages
- **Search Response**: < 200ms for full-text search
- **Startup Time**: < 2 seconds cold start
- **Memory Usage**: < 100MB for typical workload

### 5.2 Scalability Targets
- **Pages**: Support 10,000+ pages efficiently
- **Database Size**: Handle 1GB+ databases smoothly
- **Concurrent Operations**: Non-blocking UI during saves

### 5.3 Optimization Strategies
- Lazy loading of page content
- Virtual scrolling for large lists
- Background indexing for search
- Database connection pooling
- Asset compression and caching

## 6. Development Phases

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

### 7.1 Security
- Input sanitization for all user content
- SQL injection prevention
- XSS protection in rendered content
- Secure random number generation for encryption
- Regular security dependency updates

### 7.2 Data Integrity
- Database transaction management
- Automatic backups
- Data validation
- Migration system for schema updates
- Corruption detection and recovery

### 7.3 Cross-Platform Compatibility
- Windows 10+ support
- macOS 10.15+ support
- Linux (major distributions)
- Consistent behavior across platforms
- Native look-and-feel per platform

### 7.4 Maintenance & Updates
- Automatic update mechanism
- Version migration system
- Configuration management
- Error reporting and logging
- User feedback collection

## 8. Success Criteria

### 8.1 Functional Requirements
- ✅ All specified features implemented
- ✅ Data remains accessible and portable
- ✅ Single executable deployment works
- ✅ Offline functionality complete
- ✅ Performance targets met

### 8.2 Quality Requirements
- ✅ Zero data loss in normal operation
- ✅ Intuitive user interface
- ✅ Comprehensive documentation
- ✅ Automated testing coverage > 80%
- ✅ Security review completed

### 8.3 Long-term Viability
- ✅ Minimal external dependencies
- ✅ Clear upgrade path
- ✅ Data export capabilities
- ✅ Open source considerations
- ✅ Community contribution guidelines

---

*This specification document should be reviewed and approved by all stakeholders before development begins. Regular reviews should be conducted to ensure alignment with project goals and user needs.*