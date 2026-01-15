# Implementation Plan: SQLite + Study Sessions + Drag-Drop Reordering

**Created:** 2026-01-15  
**Status:** Complete

## Overview

This plan covers migrating from JSON file storage to SQLite, adding automatic study session generation for tests/quizzes, implementing drag-and-drop reordering between days, allowing new entries to be created from the web UI, and persisting completion state in the database.

## Design Decisions

| Item | Decision |
|------|----------|
| Study session entry_type | `"studio"` |
| Delete button | Small trash icon, visible on hover |
| Orphaned study sessions | Keep as-is with visual indicator (dashed orange border) |
| Position on drag | Simple "Top" / "Bottom" dialog |
| Study session text | `"Study for: {first 100 chars of task}..."` |
| Migrations | Separate `db/migrations/` directory |
| Gitignore | Add `data/*.db` for SQLite files |

## Database Schema

### `db/migrations/001_initial_schema.sql`

```sql
-- Initial schema for homework entries

CREATE TABLE IF NOT EXISTS entries (
    id TEXT PRIMARY KEY,
    source_id TEXT,
    entry_type TEXT NOT NULL,
    date TEXT NOT NULL,
    subject TEXT NOT NULL DEFAULT '',
    task TEXT NOT NULL,
    completed INTEGER NOT NULL DEFAULT 0,
    position INTEGER NOT NULL DEFAULT 0,
    parent_id TEXT,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    FOREIGN KEY (parent_id) REFERENCES entries(id) ON DELETE SET NULL
);

CREATE INDEX IF NOT EXISTS idx_entries_date ON entries(date);
CREATE INDEX IF NOT EXISTS idx_entries_parent ON entries(parent_id);
CREATE INDEX IF NOT EXISTS idx_entries_date_position ON entries(date, position);
CREATE INDEX IF NOT EXISTS idx_entries_source_id ON entries(source_id);

-- Track applied migrations
CREATE TABLE IF NOT EXISTS schema_migrations (
    version TEXT PRIMARY KEY,
    applied_at TEXT NOT NULL
);
```

### ID vs source_id

- **`id`**: Unique identifier for each entry, generated fresh each time (not content-based)
- **`source_id`**: Content-based hash of `(original_date, subject, task)` used for deduplication during import

This design allows entries to be moved to different dates while still being recognized as duplicates when re-importing from export files. When an entry is imported, its `source_id` is computed from the original export data. If an entry with that `source_id` already exists in the database, the import is skipped - even if the existing entry has been moved to a different date.

## File Changes

### Files to Create

| File | Purpose |
|------|---------|
| `crates/compitutto/db/migrations/001_initial_schema.sql` | Initial DB schema |
| `crates/compitutto/src/db.rs` | Database operations module |

### Files to Modify

| File | Changes |
|------|---------|
| `Cargo.toml` | Add `rusqlite = { version = "0.32", features = ["bundled"] }` |
| `src/lib.rs` | Add `pub mod db;` |
| `src/types.rs` | Add `id`, `source_id`, `completed`, `position`, `parent_id`, `created_at`, `updated_at` fields |
| `src/data.rs` | Add `is_test_or_quiz()`, `generate_study_sessions()`, integrate with DB |
| `src/server.rs` | Add POST/PUT/DELETE endpoints, change `AppState` to use DB connection |
| `src/html.rs` | Add drag-drop JS, delete buttons, add-entry form, position dialog, CSS |
| `src/main.rs` | Initialize DB, run migrations, handle JSON->DB migration |
| `.gitignore` | Add `data/*.db` |

## Types Changes

### Updated `HomeworkEntry` struct

```rust
#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
pub struct HomeworkEntry {
    pub id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source_id: Option<String>,
    #[serde(rename = "type")]
    pub entry_type: String,
    pub date: String,
    pub subject: String,
    pub task: String,
    #[serde(default)]
    pub completed: bool,
    #[serde(default)]
    pub position: i32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parent_id: Option<String>,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub created_at: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub updated_at: String,
}

impl HomeworkEntry {
    /// Check if this is an auto-generated study session
    pub fn is_generated(&self) -> bool {
        self.parent_id.is_some()
    }
    
    /// Check if this is an orphaned study session (was generated but parent deleted)
    pub fn is_orphaned(&self) -> bool {
        self.entry_type == "studio" && self.parent_id.is_none()
    }
    
    /// Generate a source ID for import deduplication
    pub fn generate_source_id(date: &str, subject: &str, task: &str) -> String {
        // Hash of (date, subject, task) - used to detect duplicates during import
    }
}
```

## Database Module (`db.rs`)

### Key Functions

```rust
// Initialization
pub fn init_db(db_path: &Path, migrations_dir: &Path) -> Result<Connection>
pub fn run_migrations(conn: &Connection, migrations_dir: &Path) -> Result<usize>

// Migration from JSON
pub fn migrate_from_json(conn: &Connection, json_path: &Path) -> Result<usize>

// CRUD Operations
pub fn get_all_entries(conn: &Connection) -> Result<Vec<HomeworkEntry>>
pub fn get_entry(conn: &Connection, id: &str) -> Result<Option<HomeworkEntry>>
pub fn insert_entry(conn: &Connection, entry: &HomeworkEntry) -> Result<()>
pub fn update_entry(conn: &Connection, id: &str, updates: &EntryUpdate) -> Result<()>
pub fn delete_entry(conn: &Connection, id: &str) -> Result<()>

// Study session operations
pub fn get_children(conn: &Connection, parent_id: &str) -> Result<Vec<HomeworkEntry>>
pub fn delete_with_children(conn: &Connection, id: &str) -> Result<usize>

// Position management
pub fn get_max_position_for_date(conn: &Connection, date: &str) -> Result<i32>
pub fn reorder_entries(conn: &Connection, date: &str, entry_ids: &[&str]) -> Result<()>

// Helper struct
pub struct EntryUpdate {
    pub date: Option<String>,
    pub completed: Option<bool>,
    pub position: Option<i32>,
    pub task: Option<String>,
}
```

## API Endpoints

### New/Modified Routes

| Endpoint | Method | Description |
|----------|--------|-------------|
| `/api/entries` | GET | Get all entries (existing) |
| `/api/entries` | POST | Create new entry |
| `/api/entries/:id` | GET | Get single entry |
| `/api/entries/:id` | PUT | Update entry (date, completed, position) |
| `/api/entries/:id` | DELETE | Delete entry (orphans children) |
| `/api/entries/:id/children` | GET | Get study sessions for a test |
| `/api/entries/:id/cascade` | DELETE | Delete entry + all children |
| `/api/refresh` | GET | Re-process exports, update DB (existing) |

### Request/Response Types

```rust
#[derive(Deserialize)]
pub struct CreateEntryRequest {
    pub entry_type: String,
    pub date: String,
    pub subject: String,
    pub task: String,
    pub position: Option<i32>,
}

#[derive(Deserialize)]
pub struct UpdateEntryRequest {
    pub date: Option<String>,
    pub completed: Option<bool>,
    pub position: Option<i32>,
}

#[derive(Serialize)]
pub struct DeleteResponse {
    pub success: bool,
    pub had_children: bool,
    pub children_orphaned: usize,
}

#[derive(Serialize)]
pub struct CascadeDeleteResponse {
    pub success: bool,
    pub deleted_count: usize,
}
```

## Study Session Generation

### Test Detection

Keywords that indicate a test/quiz (case-insensitive):
- `verifica` - written test
- `prova` - test/exam
- `test` - test (English word used in Italian schools)
- `interrogazione` - oral examination

```rust
const TEST_KEYWORDS: &[&str] = &["verifica", "prova", "test", "interrogazione"];

pub fn is_test_or_quiz(entry: &HomeworkEntry) -> bool {
    let task_lower = entry.task.to_lowercase();
    TEST_KEYWORDS.iter().any(|kw| task_lower.contains(kw))
}
```

### Generation Logic

```rust
pub fn generate_study_sessions(
    test: &HomeworkEntry,
    today: NaiveDate,
) -> Vec<HomeworkEntry> {
    let test_date = NaiveDate::parse_from_str(&test.date, "%Y-%m-%d")?;
    let days_until = (test_date - today).num_days();
    
    // Generate up to 4 days before, but only for future dates
    let days_to_generate = std::cmp::min(4, days_until - 1).max(0) as usize;
    
    // Truncate task to 100 chars for study session text
    let truncated_task = if test.task.len() > 100 {
        format!("{}...", &test.task[..100])
    } else {
        test.task.clone()
    };
    
    (1..=days_to_generate).map(|days_before| {
        let study_date = test_date - chrono::Duration::days(days_before as i64);
        HomeworkEntry {
            id: compute_study_session_id(test, days_before),
            entry_type: "studio".to_string(),
            date: study_date.format("%Y-%m-%d").to_string(),
            subject: test.subject.clone(),
            task: format!("Study for: {}", truncated_task),
            completed: false,
            position: 0,
            parent_id: Some(test.id.clone()),
            created_at: Utc::now().to_rfc3339(),
            updated_at: Utc::now().to_rfc3339(),
        }
    }).collect()
}
```

## Frontend Features

### 1. Drag and Drop

- Items are draggable between date groups
- When dropped on a different day, a position dialog appears
- User chooses "Add to Top" or "Add to Bottom"
- If moving a test with children, confirm dialog appears first

### 2. Delete Button

- Small trash icon (visible on hover)
- If entry has children (study sessions):
  - Prompt: "Type 'delete all' to delete everything, or 'keep' to delete only this entry"
- Otherwise: simple confirmation

### 3. Add Entry

- Floating "+" button in bottom-right corner
- Opens dialog with form:
  - Date picker
  - Subject input
  - Entry type dropdown (compiti/nota)
  - Task textarea
- Creates entry via POST /api/entries

### 4. Completion State

- Checkboxes now persist to database (not localStorage)
- PUT /api/entries/:id with `{ completed: true/false }`

### 5. Visual Indicators

| State | Styling |
|-------|---------|
| Generated (study session) | Cyan left border + "auto" badge |
| Orphaned (parent deleted) | Dashed orange border + "orphaned" badge |
| Dragging | Reduced opacity |
| Drop target | Pink glow/shadow |

## Migration Strategy

### First Run (JSON exists, DB doesn't)

1. Create `data/homework.db`
2. Run migrations from `db/migrations/`
3. Import all entries from `homework.json`
4. For each test entry, generate study sessions
5. Print migration summary

### Subsequent Runs

- Use database directly
- `homework.json` is no longer written (can be kept as backup)

## Implementation Tasks

1. [x] Add rusqlite dependency to Cargo.toml
2. [x] Create db/migrations/ directory structure
3. [x] Create 001_initial_schema.sql migration
4. [x] Create src/db.rs module (migrations runner, CRUD ops)
5. [x] Update types.rs with new fields (id, completed, position, parent_id, timestamps)
6. [x] Update data.rs with test detection and study session generation
7. [x] Update server.rs with new API endpoints (GET/POST/PUT/DELETE)
8. [x] Update html.rs with drag-drop JS, delete buttons, dialogs, CSS
9. [x] Update main.rs to add mod db
10. [x] Add data/*.db to .gitignore
11. [x] Write tests for db.rs
12. [x] Write tests for study session generation
13. [x] Write tests for new server endpoints
14. [x] Write tests for new HTML features
15. [x] Run tests to verify everything works (179 tests passing)

## Completed Changes Summary

### Files Created
- `crates/compitutto/db/migrations/001_initial_schema.sql` - Database schema
- `crates/compitutto/src/db.rs` - Database module with CRUD operations

### Files Modified
- `Cargo.toml` - Added rusqlite dependency
- `src/types.rs` - Added id, completed, position, parent_id, created_at, updated_at fields
- `src/data.rs` - Added is_test_or_quiz() and generate_study_sessions()
- `src/server.rs` - Rewrote to use SQLite, added new API endpoints
- `src/main.rs` - Added mod db
- `.gitignore` - Added data/*.db patterns

### New API Endpoints
- `GET /api/entries/:id` - Get single entry
- `POST /api/entries` - Create new entry (auto-generates study sessions for tests)
- `PUT /api/entries/:id` - Update entry (date, completed, position)
- `DELETE /api/entries/:id` - Delete entry (orphans children)
- `GET /api/entries/:id/children` - Get study sessions for a test
- `DELETE /api/entries/:id/cascade` - Delete entry + all children

### html.rs Updates (Completed)
- Visual indicators (cyan border + "auto" badge for generated, orange dashed + "orphaned" badge for orphaned)
- Delete buttons (trash icon visible on hover) with confirmation dialogs
- Drag-and-drop between days with position selection dialog
- Floating "+" button and add entry form dialog
- Completion checkbox now persists via PUT /api/entries/:id instead of localStorage

## CSS Additions

```css
/* Delete button */
.delete-btn {
    position: absolute;
    top: 8px;
    right: 8px;
    background: transparent;
    border: none;
    cursor: pointer;
    opacity: 0;
    transition: opacity 0.2s;
    font-size: 14px;
    padding: 4px;
}

.homework-item:hover .delete-btn {
    opacity: 0.6;
}

.delete-btn:hover {
    opacity: 1 !important;
}

/* Study session (generated) styling */
.homework-item[data-generated="true"] {
    border-left: 3px solid #00ffff;
    background: rgba(0, 255, 255, 0.03);
}

/* Orphaned study session */
.homework-item[data-orphaned="true"] {
    border-left: 3px dashed #ff9900;
    background: rgba(255, 153, 0, 0.03);
}

.auto-badge, .orphan-badge {
    font-size: 0.55em;
    padding: 2px 6px;
    border-radius: 3px;
    margin-left: 8px;
    text-transform: uppercase;
    letter-spacing: 0.05em;
}

.auto-badge {
    background: rgba(0, 255, 255, 0.2);
    color: #00ffff;
}

.orphan-badge {
    background: rgba(255, 153, 0, 0.2);
    color: #ff9900;
}

/* Drag states */
.homework-item.dragging {
    opacity: 0.4;
}

.date-group.drag-over {
    background: rgba(255, 0, 150, 0.05);
    box-shadow: inset 0 0 20px rgba(255, 0, 150, 0.2);
}

/* Add button */
.add-entry-btn {
    position: fixed;
    bottom: 30px;
    right: 30px;
    width: 56px;
    height: 56px;
    border-radius: 50%;
    background: linear-gradient(135deg, #ff0096, #00ffff);
    border: none;
    color: #000;
    font-size: 28px;
    font-weight: bold;
    cursor: pointer;
    box-shadow: 0 4px 20px rgba(255, 0, 150, 0.4);
    z-index: 100;
}

/* Dialogs */
dialog {
    background: #1a1a1a;
    border: 1px solid rgba(255, 255, 255, 0.2);
    border-radius: 8px;
    color: #fff;
    padding: 24px;
    max-width: 400px;
}

dialog::backdrop {
    background: rgba(0, 0, 0, 0.7);
}
```

## JavaScript Additions

See implementation for full JavaScript code including:
- Drag and drop handlers
- Delete confirmation flow
- Position dialog
- Add entry form submission
- Checkbox completion (API-backed)
