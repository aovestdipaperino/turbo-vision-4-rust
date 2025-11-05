# Chapter 17: Streams and Persistence

This chapter covers how applications handle data persistence—saving and loading program state. The original Turbo Vision used a sophisticated stream-based serialization system that could save entire object hierarchies to disk and restore them later. The current Rust implementation takes a different approach that's more aligned with modern Rust idioms.

The topics in this chapter include:

- Understanding the original stream architecture
- The Rust implementation's approach to persistence
- File I/O patterns in the current implementation
- Implementing custom persistence if needed
- Alternative serialization strategies

## The Original Stream Architecture

### Stream-Based Persistence in Pascal

The original Turbo Vision provided a powerful stream system (`TStream`) that enabled applications to serialize complex object hierarchies to binary files and restore them later. This system was the foundation for features like saving desktop layouts, storing collections of objects, and creating resource files.

**Key Concepts from the Original System:**

1. **Stream Registration**: Each object type registered itself with the stream system, providing `Store` (serialization) and `Load` (deserialization) methods:

   ```pascal
   // Original Pascal approach
   const
     RGraphPoint: TStreamRec = (
       ObjType: 150;
       VmtLink: Ofs(TypeOf(TGraphPoint)^);
       Load: @TGraphPoint.Load;
       Store: @TGraphPoint.Store
     );
   ```

2. **Store/Load Methods**: Objects implemented virtual methods to serialize their state:

   ```pascal
   // Original Pascal
   procedure TGraphObject.Store(var S: TStream);
   begin
     S.Write(X, SizeOf(X));
     S.Write(Y, SizeOf(Y));
   end;

   constructor TGraphObject.Load(var S: TStream);
   begin
     inherited Load(S);
     S.Read(X, SizeOf(X));
     S.Read(Y, SizeOf(Y));
   end;
   ```

3. **Polymorphic Streaming**: A `TCollection` could contain different object types, and the stream system would automatically handle the type information, storing and loading each object correctly using its VMT (Virtual Method Table).

4. **Random Access**: Streams supported seeking to arbitrary positions, enabling resource file systems with indexed access.

5. **Error Handling**: Streams maintained status codes (`stOK`, `stGetError`, `stPutError`, etc.) and provided error info fields.

### Stream Versioning

The original system even supported version migration—allowing version 2.0 applications to read streams created by version 1.0 programs. Objects used option flags like `ofVersion20` to mark their version, and `Load` constructors would adapt their reading logic based on the version bits they encountered.

### Why Streams Were Powerful

This architecture enabled several important features:

- **Desktop State Persistence**: The entire application desktop (windows, editors, dialogs) could be saved and restored
- **Resource Files**: UI definitions, help text, and data could be stored in external `*.TVR` files
- **Clipboard Operations**: Cut/copy/paste used streams internally
- **Undo/Redo**: Some implementations used streams to snapshot state
- **Type Safety**: The system knew which constructor to call when loading objects

## The Rust Implementation's Approach

### No Stream Infrastructure

**The current Rust implementation does not include a stream-based serialization system.** This is a deliberate architectural choice that reflects modern Rust patterns:

**What's NOT in the Rust Implementation:**
- ❌ No `serde` or serialization framework in `Cargo.toml`
- ❌ No binary stream types (no equivalent to `TBufStream`, `TDosStream`, `TEmsStream`)
- ❌ No registration system for types
- ❌ No `Store`/`Load` traits or methods
- ❌ No desktop state persistence
- ❌ No resource files

**What IS in the Rust Implementation:**
- ✅ Plain text file I/O using `std::fs` (for editor content)
- ✅ Programmatic resource definition (menus, status lines)
- ✅ In-memory state management
- ✅ History tracking (per-session only)

### Why This Approach?

The Rust implementation prioritizes:

1. **Simplicity**: No complex serialization infrastructure to maintain
2. **Type Safety**: Compile-time checking of all resources
3. **Modern Rust Idioms**: Uses standard library patterns (`std::fs::read_to_string`, `std::fs::write`)
4. **Zero Dependencies**: No external serialization crates needed
5. **Clarity**: Resources defined directly in code are easier to understand and modify

### Tradeoffs

**Advantages:**
- Simpler codebase with fewer moving parts
- No runtime type registration errors
- Better IDE support for resource definitions
- No version migration complexity

**Limitations:**
- Can't save/restore window layouts between sessions
- No external resource files (UI compiled into binary)
- Each application session starts fresh
- Manual implementation required for any persistence needs

## File I/O Patterns in the Current Implementation

### Editor File Operations

The primary persistence mechanism in the Rust implementation is plain text file I/O for editor content. This is implemented in `src/views/editor.rs`:

```rust
use std::fs;
use std::io;

// Loading a file
pub fn load_file(&mut self, path: &str) -> io::Result<()> {
    let content = fs::read_to_string(path)?;
    self.set_text(&content);
    self.filename = Some(path.to_string());
    self.modified = false;
    self.undo_stack.clear();
    self.redo_stack.clear();
    Ok(())
}

// Saving a file
pub fn save_as(&mut self, path: &str) -> io::Result<()> {
    let content = self.get_text();
    fs::write(path, content)?;
    self.filename = Some(path.to_string());
    self.modified = false;
    Ok(())
}

// Check if modified
pub fn is_modified(&self) -> bool {
    self.modified
}
```

**Key Points:**
- Uses standard `std::fs::read_to_string` and `std::fs::write`
- UTF-8 text files only (no binary formats)
- Tracks modification state with a boolean flag
- Clears undo history on load to prevent inconsistencies
- Error handling via Rust's `Result` type

### History Management

The Rust implementation includes an in-memory history system for input fields (similar to the original `THistory` system). This is implemented in `src/core/history.rs`:

```rust
use std::sync::{Mutex, OnceLock};
use std::collections::HashMap;

pub struct HistoryList {
    items: Vec<String>,
    max_items: usize,
}

pub struct HistoryManager;

impl HistoryManager {
    // Add an item to a history list
    pub fn add(history_id: u16, item: String) {
        // Thread-safe singleton access
        let mut histories = HISTORIES.get_or_init(|| {
            Mutex::new(HashMap::new())
        }).lock().unwrap();

        let list = histories.entry(history_id)
            .or_insert_with(|| HistoryList::new(20));
        list.add(item);
    }

    // Get all items for a history ID
    pub fn get_list(history_id: u16) -> Vec<String> {
        // Returns copy of history items
    }

    // Check if history exists
    pub fn has_history(history_id: u16) -> bool {
        // ...
    }

    // Clear specific history
    pub fn clear(history_id: u16) {
        // ...
    }
}
```

**Key Points:**
- Global singleton using `OnceLock` for initialization
- Thread-safe with `Mutex`
- Per-ID history lists (like Pascal's history IDs)
- Configurable maximum items (default 20)
- **Not persisted to disk**—clears when application exits

**Usage Example:**

```rust
const HC_FILE_OPEN: u16 = 1001;

// Add to history when opening a file
HistoryManager::add(HC_FILE_OPEN, filename.clone());

// Retrieve history for a file dialog
let recent_files = HistoryManager::get_list(HC_FILE_OPEN);
```

### Application State

Application state is stored entirely in memory within the `Application` struct (`src/app/application.rs`):

```rust
pub struct Application {
    pub terminal: Terminal,
    pub menu_bar: Option<MenuBar>,
    pub status_line: Option<StatusLine>,
    pub desktop: Desktop,
    pub running: bool,
    needs_redraw: bool,
}
```

**Characteristics:**
- All state fields are plain Rust types
- No serialization attributes or derives
- State exists only during the application session
- No automatic save/restore mechanism

## Implementing Custom Persistence

If your application needs to save and restore state, you have several options:

### Option 1: Using Serde for Configuration

The most straightforward approach is to add `serde` to your `Cargo.toml` and serialize application configuration to JSON, TOML, or RON format.

**1. Add dependencies:**

```toml
[dependencies]
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
```

**2. Define your configuration structure:**

```rust
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct AppConfig {
    pub window_positions: Vec<WindowState>,
    pub recent_files: Vec<String>,
    pub theme: String,
    pub auto_save: bool,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct WindowState {
    pub title: String,
    pub bounds: Rect,
    pub is_maximized: bool,
}
```

**3. Save configuration:**

```rust
use std::fs;

pub fn save_config(config: &AppConfig) -> Result<(), Box<dyn std::error::Error>> {
    let json = serde_json::to_string_pretty(config)?;
    fs::write("config.json", json)?;
    Ok(())
}
```

**4. Load configuration:**

```rust
pub fn load_config() -> Result<AppConfig, Box<dyn std::error::Error>> {
    let json = fs::read_to_string("config.json")?;
    let config = serde_json::from_str(&json)?;
    Ok(config)
}
```

**Example usage:**

```rust
// On application startup
let config = match load_config() {
    Ok(c) => c,
    Err(_) => AppConfig::default(), // Use defaults if no config exists
};

// Restore window positions
for window_state in config.window_positions {
    let window = restore_window(&window_state);
    app.desktop.insert(window);
}

// On application shutdown
let config = collect_current_config(&app);
save_config(&config)?;
```

### Option 2: Desktop State Snapshots

For more complex scenarios like saving the entire desktop layout, you could implement a snapshot system:

```rust
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct DesktopSnapshot {
    windows: Vec<WindowSnapshot>,
    active_window: Option<usize>,
}

#[derive(Serialize, Deserialize)]
pub struct WindowSnapshot {
    window_type: String,  // "Editor", "Dialog", etc.
    title: String,
    bounds: Rect,
    data: WindowData,
}

#[derive(Serialize, Deserialize)]
pub enum WindowData {
    Editor {
        filename: Option<String>,
        cursor_pos: Point,
        scroll_pos: Point,
    },
    Dialog {
        fields: Vec<String>,
    },
    // Add other window types as needed
}

impl Desktop {
    pub fn create_snapshot(&self) -> DesktopSnapshot {
        let mut windows = Vec::new();

        // Iterate through all windows and capture their state
        self.for_each(|view| {
            if let Some(window) = view.downcast_ref::<Window>() {
                windows.push(create_window_snapshot(window));
            }
        });

        DesktopSnapshot {
            windows,
            active_window: self.current, // Track which window was active
        }
    }

    pub fn restore_snapshot(&mut self, snapshot: &DesktopSnapshot) {
        // Clear existing windows
        self.clear();

        // Recreate windows from snapshot
        for window_snap in &snapshot.windows {
            let window = restore_window_from_snapshot(window_snap);
            self.insert(window);
        }

        // Restore active window
        if let Some(index) = snapshot.active_window {
            self.select_nth(index);
        }
    }
}
```

**Usage:**

```rust
// Save desktop on exit
let snapshot = app.desktop.create_snapshot();
let json = serde_json::to_string_pretty(&snapshot)?;
fs::write("desktop.json", json)?;

// Restore desktop on startup
if let Ok(json) = fs::read_to_string("desktop.json") {
    if let Ok(snapshot) = serde_json::from_str(&json) {
        app.desktop.restore_snapshot(&snapshot);
    }
}
```

### Option 3: Binary Formats

For more efficient storage or when dealing with large data sets, consider using binary serialization:

```toml
[dependencies]
bincode = "1.3"
serde = { version = "1.0", features = ["derive"] }
```

```rust
use bincode;

pub fn save_binary(config: &AppConfig) -> Result<(), Box<dyn std::error::Error>> {
    let bytes = bincode::serialize(config)?;
    fs::write("config.bin", bytes)?;
    Ok(())
}

pub fn load_binary() -> Result<AppConfig, Box<dyn std::error::Error>> {
    let bytes = fs::read("config.bin")?;
    let config = bincode::deserialize(&bytes)?;
    Ok(config)
}
```

**Advantages:**
- Smaller file sizes
- Faster serialization/deserialization
- Better for performance-critical scenarios

**Disadvantages:**
- Not human-readable
- Less portable across versions
- Harder to debug

## Error Handling Patterns

### Original Stream Error Handling

The original stream system used status codes and error info fields:

```pascal
// Original Pascal approach
if GraphicsStream.Status <> stOK then
begin
  case GraphicsStream.Status of
    stGetError: WriteLn('Unknown stream type: ', GraphicsStream.ErrorInfo);
    stPutError: WriteLn('Write error at VMT offset: ', GraphicsStream.ErrorInfo);
    stReadError: WriteLn('Read error');
    stWriteError: WriteLn('Write error');
  end;
end;
```

### Rust Error Handling

The Rust implementation uses the standard `Result` type for error handling:

```rust
use std::io;
use std::fmt;

#[derive(Debug)]
pub enum PersistenceError {
    IoError(io::Error),
    SerializationError(String),
    DeserializationError(String),
    InvalidFormat(String),
    VersionMismatch { expected: u32, found: u32 },
}

impl fmt::Display for PersistenceError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            PersistenceError::IoError(e) => write!(f, "I/O error: {}", e),
            PersistenceError::SerializationError(msg) => write!(f, "Serialization error: {}", msg),
            PersistenceError::DeserializationError(msg) => write!(f, "Deserialization error: {}", msg),
            PersistenceError::InvalidFormat(msg) => write!(f, "Invalid format: {}", msg),
            PersistenceError::VersionMismatch { expected, found } => {
                write!(f, "Version mismatch: expected {}, found {}", expected, found)
            }
        }
    }
}

impl From<io::Error> for PersistenceError {
    fn from(err: io::Error) -> Self {
        PersistenceError::IoError(err)
    }
}

impl From<serde_json::Error> for PersistenceError {
    fn from(err: serde_json::Error) -> Self {
        PersistenceError::DeserializationError(err.to_string())
    }
}

// Usage
pub fn save_config(config: &AppConfig) -> Result<(), PersistenceError> {
    let json = serde_json::to_string_pretty(config)
        .map_err(|e| PersistenceError::SerializationError(e.to_string()))?;
    fs::write("config.json", json)?;
    Ok(())
}
```

**Best Practices:**
- Define custom error types for domain-specific errors
- Use `Result<T, E>` for all operations that can fail
- Implement `From` for automatic error conversion
- Provide context with error messages
- Use `?` operator for error propagation

## Version Migration Strategies

### Handling Configuration Changes

Unlike the original stream versioning system, modern Rust applications typically handle version migration through explicit logic:

```rust
#[derive(Serialize, Deserialize, Debug)]
pub struct AppConfigV2 {
    pub version: u32,
    pub window_positions: Vec<WindowState>,
    pub recent_files: Vec<String>,
    pub theme: String,
    pub auto_save: bool,
    // New field in v2
    pub font_size: u16,
}

impl AppConfigV2 {
    pub fn migrate_from_v1(v1: AppConfigV1) -> Self {
        AppConfigV2 {
            version: 2,
            window_positions: v1.window_positions,
            recent_files: v1.recent_files,
            theme: v1.theme,
            auto_save: v1.auto_save,
            font_size: 10, // Default for migrated configs
        }
    }
}

pub fn load_config_with_migration() -> Result<AppConfigV2, PersistenceError> {
    let json = fs::read_to_string("config.json")?;

    // Try to parse as latest version first
    if let Ok(v2) = serde_json::from_str::<AppConfigV2>(&json) {
        return Ok(v2);
    }

    // Fall back to v1 and migrate
    if let Ok(v1) = serde_json::from_str::<AppConfigV1>(&json) {
        let v2 = AppConfigV2::migrate_from_v1(v1);
        // Optionally save the migrated config
        save_config(&v2)?;
        return Ok(v2);
    }

    Err(PersistenceError::InvalidFormat("Unrecognized config version".into()))
}
```

### Using Serde's Flexibility

Serde provides several attributes to handle versioning gracefully:

```rust
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct AppConfig {
    #[serde(default)]  // Use Default::default() if missing
    pub version: u32,

    pub window_positions: Vec<WindowState>,

    #[serde(default = "default_theme")]  // Custom default
    pub theme: String,

    #[serde(skip_serializing_if = "Option::is_none")]  // Omit if None
    pub experimental_feature: Option<bool>,

    #[serde(rename = "fontsize")]  // Handle field name changes
    pub font_size: u16,
}

fn default_theme() -> String {
    "Default".to_string()
}
```

## Comparing Approaches

### Original Pascal Stream System

**Strengths:**
- Powerful and flexible
- Handled arbitrary object graphs
- Built-in versioning support
- Random access for resource files
- Type registration at runtime

**Weaknesses:**
- Complex to implement correctly
- Registration errors were runtime errors
- Binary formats were opaque
- Debugging serialization issues was difficult
- Version migration could be fragile

### Modern Rust Approach

**Strengths:**
- Compile-time type safety
- Human-readable formats (JSON, TOML)
- Standard error handling with `Result`
- Excellent tooling support (serde ecosystem)
- Easy to debug and inspect
- Explicit version migration

**Weaknesses:**
- Requires explicit implementation
- Each field must be marked for serialization
- Need to handle version migration manually
- Can't serialize arbitrary trait objects easily

## Practical Recommendations

### When to Add Persistence

Consider adding persistence when your application needs to:

1. **Remember user preferences** (theme, font size, window positions)
2. **Track recent files** or working sets
3. **Save incomplete work** (autosave, session restore)
4. **Store application data** (databases, documents)
5. **Cache expensive computations**

### When to Skip Persistence

You might not need persistence if your application:

1. **Performs one-time transformations** (converters, formatters)
2. **Is stateless by design** (filters, viewers)
3. **Has trivial state** (easily recreated each session)
4. **Runs in constrained environments** (no filesystem access)

### Choosing a Format

| Format | Best For | Considerations |
|--------|----------|----------------|
| **JSON** | Configuration, settings, human-readable data | Widely supported, easy to debug |
| **TOML** | Application config files | Clean syntax, good for users to edit |
| **RON** | Rust-specific data | Rust-native types, enums, tuples |
| **Bincode** | Performance-critical, large datasets | Fast, compact, not human-readable |
| **MessagePack** | Cross-platform binary, APIs | Compact, widely supported |

## Summary

The evolution from Turbo Vision's stream-based persistence to the modern Rust approach reflects broader shifts in software architecture:

**Original Philosophy**: "The framework provides everything you need for persistence."

**Modern Philosophy**: "The framework provides the primitives; you compose the persistence layer you need."

While the original stream system was sophisticated for its era, modern Rust applications benefit from:
- **Explicit over implicit**: Clear code is easier to maintain
- **Composition over inheritance**: Build what you need from smaller parts
- **Type safety at compile time**: Catch errors before runtime
- **Standard patterns**: Use familiar Result types and error handling

The current Rust Turbo Vision implementation gives you the foundation (file I/O, history management, state structures) and lets you choose the persistence strategy that fits your needs—whether that's simple JSON configuration files, sophisticated desktop state snapshots, or no persistence at all.

## References

**Source Files:**
- `src/views/editor.rs` - File I/O implementation (lines 1-1400)
- `src/core/history.rs` - History management system (279 lines)
- `src/app/application.rs` - Application state structure (310 lines)
- `demo/rust_editor.rs` - Complete editor example with file operations

**Documentation:**
- `docs/SERIALIZATION_AND_PERSISTENCE.md` - Detailed persistence analysis
- `docs/SERIALIZATION_QUICK_REFERENCE.md` - Quick reference guide
- Chapter 4: Persistence and Configuration - Programmatic resource definition
- Chapter 6: Managing Data Collections - In-memory collection patterns

**External Resources:**
- [Serde documentation](https://serde.rs/) - Rust serialization framework
- [JSON in Rust](https://docs.serde.rs/serde_json/) - JSON serialization
- [TOML in Rust](https://docs.rs/toml/) - Configuration file format
- [Bincode](https://docs.rs/bincode/) - Binary serialization
