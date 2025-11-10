# Chapter 16: Collections and Streams

In the original Turbo Vision, collections provided a polymorphic way to hold different but related objects, and streams allowed these collections to be persisted to disk for later retrieval. This chapter examines how the Rust implementation handles these concepts using idiomatic Rust patterns.

## Rust Approach: Collections and Serialization

The Rust implementation of Turbo Vision takes a fundamentally different approach than the original Pascal version:

**Original Pascal Approach:**
- Used `TCollection` with stream-based serialization
- Required `Store` and `Load` methods on all objects
- Used registration records with VMT (Virtual Method Table) links
- Implemented custom binary streaming with `TStream`, `TBufStream`, `TEmsStream`

**Rust Implementation Approach:**
- Uses standard Rust collections (`Vec<T>`, `HashMap<K,V>`, etc.)
- **No built-in serialization framework** (no serde in dependencies)
- Relies on Rust's type system and ownership for safety
- File I/O uses standard `std::fs` operations

From `Cargo.toml`:
```toml
[dependencies]
crossterm = "0.27"
unicode-width = "0.1"
arboard = "3.3"
```

Notice there's no `serde`, `bincode`, or other serialization crate. The framework focuses on runtime UI functionality rather than persistence.

## Collections in Turbo Vision Rust

Throughout the codebase, you'll find standard Rust collections used effectively. Let's examine several examples:

### Menu Collections

Menus maintain collections of menu items using `Vec<MenuItem>` (`src/core/menu_data.rs:178-255`):

```rust
#[derive(Clone, Debug)]
pub struct Menu {
    pub items: Vec<MenuItem>,
    pub default_index: Option<usize>,
}

impl Menu {
    pub fn add(&mut self, item: MenuItem) {
        self.items.push(item);
    }

    pub fn len(&self) -> usize {
        self.items.len()
    }

    pub fn from_items(items: Vec<MenuItem>) -> Self {
        Self {
            items,
            default_index: None,
        }
    }
}
```

The `MenuItem` enum provides type-safe polymorphism without requiring a common base class:

```rust
#[derive(Clone, Debug)]
pub enum MenuItem {
    Regular {
        text: String,
        command: CommandId,
        key_code: KeyCode,
        help_ctx: u16,
        enabled: bool,
        shortcut: Option<String>,
    },
    SubMenu {
        text: String,
        key_code: KeyCode,
        help_ctx: u16,
        menu: Menu,
    },
    Separator,
}
```

### History Collections

The history system maintains per-ID collections of strings (`src/core/history.rs:27-96`):

```rust
pub struct HistoryList {
    items: Vec<String>,
    max_items: usize,
}

impl HistoryList {
    pub fn new(max_items: usize) -> Self {
        Self {
            items: Vec::new(),
            max_items,
        }
    }

    pub fn add(&mut self, item: String) {
        // Remove duplicates
        self.items.retain(|x| x != &item);

        // Insert at front (most recent first)
        self.items.insert(0, item);

        // Enforce size limit
        if self.items.len() > self.max_items {
            self.items.truncate(self.max_items);
        }
    }

    pub fn get(&self, index: usize) -> Option<&String> {
        self.items.get(index)
    }

    pub fn items(&self) -> &Vec<String> {
        &self.items
    }
}
```

The global `HistoryManager` uses a `HashMap` to maintain separate lists for different history IDs:

```rust
use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};

pub struct HistoryManager;

static HISTORIES: OnceLock<Mutex<HashMap<u16, HistoryList>>> = OnceLock::new();

impl HistoryManager {
    fn get_histories() -> &'static Mutex<HashMap<u16, HistoryList>> {
        HISTORIES.get_or_init(|| Mutex::new(HashMap::new()))
    }

    pub fn add(history_id: u16, item: String) {
        let mut histories = Self::get_histories().lock().unwrap();
        let list = histories
            .entry(history_id)
            .or_insert_with(|| HistoryList::new(20));
        list.add(item);
    }

    pub fn get_list(history_id: u16) -> Vec<String> {
        let histories = Self::get_histories().lock().unwrap();
        histories
            .get(&history_id)
            .map(|list| list.items().clone())
            .unwrap_or_default()
    }
}
```

### View Collections

Groups maintain collections of child views (`src/views/group.rs:10-88`):

```rust
pub struct Group {
    bounds: Rect,
    children: Vec<Box<dyn View>>,
    focused: usize,
}

impl Group {
    pub fn add(&mut self, view: Box<dyn View>) {
        self.children.push(view);
    }

    pub fn child_at(&self, index: usize) -> Option<&dyn View> {
        self.children.get(index).map(|b| &**b)
    }

    pub fn child_at_mut(&mut self, index: usize) -> Option<&mut dyn View> {
        self.children.get_mut(index).map(|b| &mut **b)
    }

    pub fn child_count(&self) -> usize {
        self.children.len()
    }
}
```

This demonstrates Rust's approach to polymorphic collections: using trait objects (`Box<dyn View>`) rather than inheritance-based polymorphism.

## Understanding Ownership in Collections

The original Pascal version used pointers extensively, requiring manual memory management. The owner of an object was responsible for disposing it. Similarly, the owner of a collection was responsible for streaming it.

Rust's ownership system makes these concepts explicit and enforced at compile time:

### Ownership Principle

When a view is added to a group, ownership transfers:

```rust
let button = ButtonBuilder::new()
    .bounds(bounds)
    .title("OK")
    .command(CM_OK)
    .build_boxed();
group.add(button);  // Ownership transfers to group
// button is no longer accessible here
```

The group now owns the button and will drop it when the group itself is dropped.

### Subview References

In the original Pascal version, a dialog might store pointers to its controls in both instance fields and the subview list. The solution involved `GetSubViewPtr` and `PutSubViewPtr` methods.

In Rust, this pattern is handled differently. If you need to reference a child view:

**Option 1: Store the index**
```rust
pub struct MyDialog {
    ok_button_index: usize,
}

impl MyDialog {
    fn get_ok_button(&self) -> &Button {
        self.child_at(self.ok_button_index)
            .unwrap()
            .downcast_ref::<Button>()
            .unwrap()
    }
}
```

**Option 2: Use IDs for lookups**
```rust
const OK_BUTTON_ID: u16 = 1;

impl MyDialog {
    fn find_ok_button(&self) -> Option<&Button> {
        self.children.iter()
            .find(|child| child.get_id() == OK_BUTTON_ID)
            .and_then(|child| child.downcast_ref::<Button>())
    }
}
```

The borrow checker ensures you cannot have mutable and immutable references simultaneously, preventing the class of bugs that could occur with Pascal pointers.

## File I/O Without Streams

While the original Turbo Vision had elaborate streaming mechanisms, the Rust implementation provides straightforward file I/O for text content.

### Editor File Operations

The `Editor` view provides basic file loading and saving (`src/views/editor.rs`):

```rust
impl Editor {
    pub fn load_file(&mut self, path: &str) -> std::io::Result<()> {
        let content = std::fs::read_to_string(path)?;
        self.set_text(&content);
        self.filename = Some(path.to_string());
        self.modified = false;
        self.undo_stack.clear();
        self.redo_stack.clear();
        self.update_indicator();
        Ok(())
    }

    pub fn save_as(&mut self, path: &str) -> std::io::Result<()> {
        let content = self.get_text();
        std::fs::write(path, content)?;
        self.filename = Some(path.to_string());
        self.modified = false;
        self.update_indicator();
        Ok(())
    }

    pub fn save_file(&mut self) -> std::io::Result<()> {
        if let Some(path) = &self.filename {
            let path = path.clone();
            self.save_as(&path)
        } else {
            Err(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "No filename set",
            ))
        }
    }
}
```

This uses Rust's standard library:
- `std::fs::read_to_string()` - Reads entire file into a String
- `std::fs::write()` - Writes string to file
- `Result<T>` - Rust's error handling mechanism
- UTF-8 encoding by default

### Error Handling Pattern

The Rust `Result` type replaces the original's `TStream.Status` error checking:

**Original Pascal:**
```pascal
GraphicsStream.Init('GRAPHICS.STM', stCreate, 1024);
if GraphicsStream.Status <> stOK then begin
  { Handle error }
end;
GraphicsStream.Put(GraphicsList);
if GraphicsStream.Status <> stOK then begin
  { Handle error }
end;
GraphicsStream.Done;
```

**Rust Equivalent:**
```rust
use std::fs::File;
use std::io::Write;

fn save_data(path: &str, data: &str) -> std::io::Result<()> {
    let mut file = File::create(path)?;
    file.write_all(data.as_bytes())?;
    Ok(())
}

// Usage with error propagation:
match save_data("graphics.dat", &my_data) {
    Ok(()) => println!("Saved successfully"),
    Err(e) => eprintln!("Error saving: {}", e),
}
```

The `?` operator propagates errors up the call stack automatically, and the compiler ensures all `Result` values are handled.

## Resource Definitions

Instead of streaming resource files, the Rust implementation defines resources as Rust code structures.

### Menu Resources

Menus are constructed programmatically (`src/core/menu_data.rs`):

```rust
use turbo_vision::core::menu_data::{Menu, MenuItem, MenuBuilder};
use turbo_vision::core::command::{CM_OPEN, CM_SAVE, CM_QUIT};
use turbo_vision::core::event::{KB_F3, KB_F2, KB_ALT_X};

// Method 1: Direct construction
let file_menu = Menu::from_items(vec![
    MenuItem::new("~O~pen", CM_OPEN, KB_F3, 0),
    MenuItem::new("~S~ave", CM_SAVE, KB_F2, 0),
    MenuItem::separator(),
    MenuItem::new("E~x~it", CM_QUIT, KB_ALT_X, 0),
]);

// Method 2: Builder pattern
let file_menu = MenuBuilder::new()
    .item("~O~pen", CM_OPEN, KB_F3)
    .item("~S~ave", CM_SAVE, KB_F2)
    .separator()
    .item("E~x~it", CM_QUIT, KB_ALT_X)
    .build();
```

### Status Line Resources

Status lines follow a similar pattern (`src/core/status_data.rs`):

```rust
use turbo_vision::core::status_data::{StatusItem, StatusDef, StatusLine};
use turbo_vision::core::command::{CM_HELP, CM_QUIT};
use turbo_vision::core::event::{KB_F1, KB_ALT_X};

let status = StatusLine::single(vec![
    StatusItem::new("~F1~ Help", KB_F1, CM_HELP),
    StatusItem::new("~Alt+X~ Exit", KB_ALT_X, CM_QUIT),
]);
```

These resource definitions are type-checked at compile time, preventing the runtime errors that could occur with incorrectly formatted resource files.

## When You Need Serialization

If your application needs to save and restore complex state (like the IDE's desktop configuration), you have several options:

### Option 1: Add Serde Support

Add to `Cargo.toml`:
```toml
[dependencies]
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"  # or serde_ron, toml, etc.
```

Then derive serialization on your types:
```rust
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
pub struct AppConfig {
    window_positions: Vec<WindowPosition>,
    recent_files: Vec<String>,
    editor_settings: EditorSettings,
}

#[derive(Serialize, Deserialize)]
pub struct WindowPosition {
    x: i16,
    y: i16,
    width: i16,
    height: i16,
}

// Save configuration
fn save_config(config: &AppConfig) -> std::io::Result<()> {
    let json = serde_json::to_string_pretty(config)?;
    std::fs::write("config.json", json)?;
    Ok(())
}

// Load configuration
fn load_config() -> std::io::Result<AppConfig> {
    let json = std::fs::read_to_string("config.json")?;
    let config = serde_json::from_str(&json)?;
    Ok(config)
}
```

### Option 2: Manual Serialization

For simple cases, implement your own serialization:

```rust
impl AppConfig {
    pub fn save(&self, path: &str) -> std::io::Result<()> {
        let mut output = String::new();
        for pos in &self.window_positions {
            output.push_str(&format!("{}:{},{}+{}x{}\n",
                pos.id, pos.x, pos.y, pos.width, pos.height));
        }
        std::fs::write(path, output)
    }

    pub fn load(path: &str) -> std::io::Result<Self> {
        let content = std::fs::read_to_string(path)?;
        let mut positions = Vec::new();

        for line in content.lines() {
            // Parse each line
            // ...
        }

        Ok(Self { window_positions: positions, ... })
    }
}
```

### Option 3: Binary Serialization

For compact storage, use `bincode`:

```toml
[dependencies]
serde = { version = "1.0", features = ["derive"] }
bincode = "1.3"
```

```rust
use bincode;

fn save_binary(config: &AppConfig) -> std::io::Result<()> {
    let bytes = bincode::serialize(config)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
    std::fs::write("config.bin", bytes)?;
    Ok(())
}

fn load_binary() -> std::io::Result<AppConfig> {
    let bytes = std::fs::read("config.bin")?;
    bincode::deserialize(&bytes)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))
}
```

## Iterator Patterns Replace ForEach

The original Turbo Vision used `ForEach` to iterate over collection items:

**Original Pascal:**
```pascal
procedure PrintItem(Item: Pointer); far;
begin
  WriteLn(PGraphObject(Item)^.X, ' ', PGraphObject(Item)^.Y);
end;

GraphicsList.ForEach(@PrintItem);
```

In Rust, use iterator methods:

```rust
// Simple iteration
for item in &graphics_list {
    println!("{} {}", item.x, item.y);
}

// Functional style
graphics_list.iter()
    .for_each(|item| println!("{} {}", item.x, item.y));

// With filtering
graphics_list.iter()
    .filter(|item| item.x > 0)
    .for_each(|item| println!("{} {}", item.x, item.y));

// With transformation
let x_coords: Vec<i16> = graphics_list.iter()
    .map(|item| item.x)
    .collect();
```

Rust's iterators are:
- **Zero-cost abstractions** - compiled to efficient code
- **Composable** - can chain operations
- **Lazy** - only computed when consumed
- **Type-safe** - prevent common errors at compile time

## Comparison: Pascal vs Rust Collections

| Aspect | Original Pascal | Rust Implementation |
|--------|----------------|---------------------|
| **Collection Type** | `TCollection` | `Vec<T>`, `HashMap<K,V>` |
| **Polymorphism** | Inheritance from `TObject` | Trait objects (`Box<dyn Trait>`) |
| **Memory Management** | Manual (`New`/`Dispose`) | Automatic (ownership/RAII) |
| **Serialization** | Built-in streams | Optional (serde) |
| **Registration** | `RegisterType()` required | Not needed |
| **Iteration** | `ForEach` with callbacks | Iterator methods |
| **Type Safety** | Runtime (casting) | Compile-time |
| **Error Handling** | Status codes | `Result<T, E>` |

## Desktop Persistence Example

Here's how you might implement desktop save/restore functionality:

```rust
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
struct DesktopState {
    windows: Vec<WindowState>,
}

#[derive(Serialize, Deserialize)]
struct WindowState {
    title: String,
    bounds: Rect,
    state_flags: u16,
    // Window-specific data...
}

impl Application {
    pub fn save_desktop(&self, path: &str) -> std::io::Result<()> {
        let state = DesktopState {
            windows: self.desktop.children.iter()
                .filter_map(|child| {
                    // Extract window state
                    child.downcast_ref::<Window>()
                        .map(|w| WindowState {
                            title: w.title.clone(),
                            bounds: w.bounds,
                            state_flags: w.state,
                        })
                })
                .collect(),
        };

        let json = serde_json::to_string_pretty(&state)?;
        std::fs::write(path, json)?;
        Ok(())
    }

    pub fn load_desktop(&mut self, path: &str) -> std::io::Result<()> {
        let json = std::fs::read_to_string(path)?;
        let state: DesktopState = serde_json::from_str(&json)?;

        // Clear current desktop
        self.desktop.remove_all();

        // Recreate windows
        for window_state in state.windows {
            let window = WindowBuilder::new()
                .bounds(window_state.bounds)
                .title(window_state.title)
                .build_boxed();
            // Restore window state...
            self.desktop.add(window);
        }

        Ok(())
    }
}
```

## Practical Example: Managing Application Data

Let's look at a complete example of managing a collection of records with file persistence:

```rust
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ContactRecord {
    pub name: String,
    pub email: String,
    pub phone: String,
}

pub struct ContactManager {
    contacts: Vec<ContactRecord>,
    modified: bool,
    filename: Option<String>,
}

impl ContactManager {
    pub fn new() -> Self {
        Self {
            contacts: Vec::new(),
            modified: false,
            filename: None,
        }
    }

    pub fn add(&mut self, contact: ContactRecord) {
        self.contacts.push(contact);
        self.modified = true;
    }

    pub fn remove(&mut self, index: usize) -> Option<ContactRecord> {
        if index < self.contacts.len() {
            self.modified = true;
            Some(self.contacts.remove(index))
        } else {
            None
        }
    }

    pub fn get(&self, index: usize) -> Option<&ContactRecord> {
        self.contacts.get(index)
    }

    pub fn count(&self) -> usize {
        self.contacts.len()
    }

    pub fn is_modified(&self) -> bool {
        self.modified
    }

    pub fn load(&mut self, path: &str) -> std::io::Result<()> {
        let json = std::fs::read_to_string(path)?;
        self.contacts = serde_json::from_str(&json)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
        self.filename = Some(path.to_string());
        self.modified = false;
        Ok(())
    }

    pub fn save(&mut self) -> std::io::Result<()> {
        if let Some(path) = &self.filename {
            let path = path.clone();
            self.save_as(&path)
        } else {
            Err(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "No filename set",
            ))
        }
    }

    pub fn save_as(&mut self, path: &str) -> std::io::Result<()> {
        let json = serde_json::to_string_pretty(&self.contacts)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
        std::fs::write(path, json)?;
        self.filename = Some(path.to_string());
        self.modified = false;
        Ok(())
    }

    // Iterator support
    pub fn iter(&self) -> impl Iterator<Item = &ContactRecord> {
        self.contacts.iter()
    }
}
```

Usage in an application:

```rust
fn main() -> std::io::Result<()> {
    let mut manager = ContactManager::new();

    // Add some contacts
    manager.add(ContactRecord {
        name: "Alice Smith".to_string(),
        email: "alice@example.com".to_string(),
        phone: "555-1234".to_string(),
    });

    manager.add(ContactRecord {
        name: "Bob Jones".to_string(),
        email: "bob@example.com".to_string(),
        phone: "555-5678".to_string(),
    });

    // Save to file
    manager.save_as("contacts.json")?;

    // Later, load from file
    let mut manager2 = ContactManager::new();
    manager2.load("contacts.json")?;

    // Iterate over contacts
    for (i, contact) in manager2.iter().enumerate() {
        println!("{}. {} - {}", i + 1, contact.name, contact.email);
    }

    Ok(())
}
```

## Summary

The Rust implementation of Turbo Vision replaces the original's elaborate streaming system with simpler, more idiomatic Rust approaches:

1. **Collections** use standard library types (`Vec`, `HashMap`) with compile-time type safety
2. **Ownership** is enforced by the compiler, preventing memory management bugs
3. **Serialization** is optional - add serde when you need it
4. **File I/O** uses standard `std::fs` operations with `Result` error handling
5. **Resources** are defined in Rust code, not external files
6. **Iteration** uses powerful, composable iterator methods

This approach provides:
- **Better performance** - zero-cost abstractions, no runtime overhead
- **Better safety** - ownership prevents use-after-free, data races
- **Better ergonomics** - iterators, pattern matching, error propagation
- **Simpler code** - no registration records, no manual memory management
- **Flexibility** - add serialization only where needed

When you need persistence, Rust's ecosystem provides excellent options like serde that integrate seamlessly with the standard library's file I/O capabilities.
