# Chapter 18: Resources

A resource file, in the original Turbo Vision, was a special object that saved other objects to disk and retrieved them by name. Applications could load objects (menus, status lines, dialog boxes) from resources rather than initializing them in code. This allowed for customization without recompiling, easier internationalization, and separation of UI definitions from program logic.

This chapter explores the resource concept and how the modern Rust implementation addresses the same goals through different mechanisms.

The topics in this chapter include:

- Understanding the original resource file system
- Why resources were valuable in Pascal
- The Rust implementation's approach to resource definition
- Programmatic resource patterns
- When and how to add external resource support

## Understanding the Original Resource System

### What Was a Resource File?

In the original Turbo Vision, a resource file worked like a random-access stream where objects were accessed by keys—unique string identifiers for each resource. The mechanism used two complementary objects:

**`TResourceFile`**: The main resource object that:
- Wrapped a stream (usually `TBufStream` for buffered file I/O)
- Maintained a sorted string collection mapping keys to stream positions
- Provided `Put` method to store objects with keys
- Provided `Get` method to retrieve objects by key
- Returned generic `PObject` pointers requiring type casting

**`TResourceItem`**: Internal structure holding:
- The resource key (string identifier)
- Position in the stream where the object starts
- Size of the serialized object

### How Resource Files Worked

The original system followed a straightforward pattern:

```pascal
// Original Pascal: Creating a resource
var
  MyRez: TResourceFile;
  MyStrm: PBufStream;
  StatusLine: PStatusLine;
begin
  // 1. Open a buffered stream
  MyStrm := New(PBufStream, Init('MY.TVR', stCreate, 1024));

  // 2. Initialize resource file on that stream
  MyRez.Init(MyStrm);

  // 3. Register object types for stream serialization
  RegisterType(RStatusLine);

  // 4. Create and store objects with keys
  StatusLine := New(PStatusLine, Init(...));
  MyRez.Put(StatusLine, 'MainStatusLine');
  Dispose(StatusLine, Done);

  // 5. Close the resource
  MyRez.Done;
end;
```

Loading from the resource was equally simple:

```pascal
// Original Pascal: Reading a resource
constructor TMyApp.Init;
begin
  MyRez.Init(New(PBufStream, Init('MY.TVR', stOpen, 1024)));
  if MyRez.Stream^.Status <> 0 then Halt(1);
  RegisterType(RStatusLine);
  TApplication.Init;
end;

procedure TMyApp.InitStatusLine;
begin
  StatusLine := PStatusLine(MyRez.Get('MainStatusLine'));
end;
```

The stream registration system (`RegisterType`) told the stream infrastructure how to deserialize each object type when reading it back from disk.

## Why Use Resources?

The original documentation highlighted several compelling advantages:

### 1. Customization Without Recompilation

Text in dialog boxes, menu item labels, and view colors could all be altered within a resource file. This allowed the appearance of applications to change without modifying or recompiling source code.

### 2. Code Size Savings

Object initialization code is often complex, containing calculations and operations that make compiled programs larger. By moving initialization to a separate resource-building program, applications only needed simple `Load` calls instead of elaborate `Init` sequences. This typically saved 8-10% of code size.

### 3. Language-Specific Versions

Applications loaded objects by name, but the language they displayed was up to them. Maintaining French, German, and English versions meant maintaining three resource files, not three codebases.

### 4. Feature Variants

Different versions of an application (basic vs. professional) could be deployed with different resource files containing different menus, without any code changes. Upgrading from basic to professional could be accomplished by providing only a new resource file.

### 5. Separation of Concerns

Resources isolated the representation of objects from program logic. Designers could modify UI layouts and text without programmer involvement.

### 6. Performance Optimizations

Resource files were buffered and supported switching to EMS streams for faster access on systems with expanded memory.

## String Lists in the Original System

The original Turbo Vision provided specialized objects for managing string lists—collections of strings accessed by numeric ID rather than string keys.

**`TStringList`**: A read-only resource access object for retrieving strings by number. The Turbo Pascal IDE used string lists for all error messages, allowing different language versions to provide different strings for the same error numbers.

**`TStrListMaker`**: A write-only constructor object used to create string lists on resource files. It sequentially wrote strings and then stored the resulting indexed list.

This design was fast and convenient for scenarios where strings were best identified by integer constants rather than string keys.

## The Rust Implementation's Approach

### No Resource Files

**The current Rust implementation does not include resource files or binary persistence.** This is a deliberate architectural choice that reflects modern development practices and Rust's strengths.

**What's NOT in the Rust Implementation:**
- ❌ No `TResourceFile` equivalent
- ❌ No stream-based persistence system
- ❌ No binary `*.TVR` resource files
- ❌ No string list resource system
- ❌ No external resource loading at runtime

**What IS in the Rust Implementation:**
- ✅ Programmatic resource definition using builders
- ✅ Compile-time type safety for all resources
- ✅ Excellent IDE support (autocomplete, refactoring)
- ✅ Zero-cost abstractions (no runtime overhead)
- ✅ Optional external configuration via standard formats

### Why This Approach?

The Rust implementation prioritizes different values:

1. **Type Safety**: All resources are checked at compile time. Typos in menu items, status keys, or dialog layouts are caught by the compiler, not discovered at runtime.

2. **Simplicity**: No complex serialization infrastructure, registration systems, or stream handling. The codebase is simpler and easier to understand.

3. **Modern Tooling**: Rust's language server provides autocomplete, inline documentation, and jump-to-definition for resource definitions. Refactoring tools work seamlessly.

4. **Zero Dependencies**: No need for external serialization frameworks. The core framework remains lean.

5. **Performance**: Resources defined in code are optimized by the compiler. No runtime deserialization overhead.

## Programmatic Resource Patterns

The Rust implementation achieves the same goals as resource files through different mechanisms. Let's examine how to handle each use case.

### Defining Menus (Replacing Menu Resources)

In the original system, menus might be stored in resource files. In Rust, they're defined using builders:

```rust
use turbo_vision::core::menu_data::{Menu, MenuBuilder};
use turbo_vision::core::command::*;
use turbo_vision::core::event::*;

// Define commands
const CM_SAVE_AS: u16 = 105;

// Build a menu
fn create_file_menu() -> Menu {
    MenuBuilder::new()
        .item_with_shortcut("~N~ew", CM_NEW, KB_CTRL_N, "Ctrl+N")
        .item_with_shortcut("~O~pen", CM_OPEN, KB_F3, "F3")
        .item_with_shortcut("~S~ave", CM_SAVE, KB_F2, "F2")
        .item("Save ~a~s...", CM_SAVE_AS, 0)
        .separator()
        .item_with_shortcut("E~x~it", CM_QUIT, KB_ALT_X, "Alt+X")
        .build()
}
```

**Benefits over resource files:**
- Compile-time verification of command IDs and key codes
- IDE autocomplete for all builder methods
- Type-safe: impossible to pass wrong parameter types
- Refactoring support: rename commands automatically updates all references

**Implementation**: See `src/core/menu_data.rs:178-356` for the complete menu builder implementation.

### Defining Status Lines (Replacing Status Resources)

Status lines follow a similar pattern:

```rust
use turbo_vision::core::status_data::{StatusItem, StatusLine, StatusLineBuilder};
use turbo_vision::core::geometry::Rect;

fn create_status_line(bounds: Rect) -> StatusLine {
    StatusLineBuilder::new_with_bounds(bounds)
        .add_default_def(vec![
            StatusItem::new("~F1~ Help", KB_F1, CM_HELP),
            StatusItem::new("~F2~ Save", KB_F2, CM_SAVE),
            StatusItem::new("~F3~ Open", KB_F3, CM_OPEN),
            StatusItem::new("~Alt+X~ Exit", KB_ALT_X, CM_QUIT),
        ])
        .build()
}
```

For context-sensitive status lines (different items for different views):

```rust
fn create_context_status(bounds: Rect) -> StatusLine {
    StatusLineBuilder::new_with_bounds(bounds)
        // Default context (0-65535)
        .add_default_def(vec![
            StatusItem::new("~F1~ Help", KB_F1, CM_HELP),
            StatusItem::new("~Alt+X~ Exit", KB_ALT_X, CM_QUIT),
        ])
        // Editor context (100-199)
        .add_def(100, 199, vec![
            StatusItem::new("~F2~ Save", KB_F2, CM_SAVE),
            StatusItem::new("~F3~ Open", KB_F3, CM_OPEN),
            StatusItem::new("~Ctrl+Y~ Delete line", KB_CTRL_Y, CM_DELETE_LINE),
        ])
        // Dialog context (200-299)
        .add_def(200, 299, vec![
            StatusItem::new("~Tab~ Next", KB_TAB, CM_NEXT),
            StatusItem::new("~Esc~ Cancel", KB_ESC, CM_CANCEL),
        ])
        .build()
}
```

**Implementation**: See `src/core/status_data.rs:1-268` for status line definitions and builders.

### Defining Dialog Boxes (Replacing Dialog Resources)

Dialog boxes are constructed programmatically with full type safety:

```rust
use turbo_vision::views::dialog::Dialog;
use turbo_vision::views::dialog::DialogBuilder;
use turbo_vision::views::button::ButtonBuilder;
use turbo_vision::views::static_text::StaticTextBuilder;
use turbo_vision::views::input_line::InputLineBuilder;
use turbo_vision::core::geometry::Rect;
use std::rc::Rc;
use std::cell::RefCell;

fn create_login_dialog() -> Dialog {
    let mut dialog = DialogBuilder::new()
        .bounds(Rect::new(0, 0, 50, 14))
        .title("Login")
        .build();
    dialog.set_centered(true);

    // Add static text label
    let label = StaticTextBuilder::new()
        .bounds(Rect::new(3, 2, 13, 3))
        .text("Username:")
        .build_boxed();
    dialog.add(label);

    // Add input field
    let username_data = Rc::new(RefCell::new(String::new()));
    let username_input = InputLineBuilder::new()
        .bounds(Rect::new(14, 2, 45, 3))
        .data(username_data)
        .max_length(30)
        .build_boxed();
    dialog.add(username_input);

    // Add password label
    let pwd_label = StaticTextBuilder::new()
        .bounds(Rect::new(3, 4, 13, 5))
        .text("Password:")
        .build_boxed();
    dialog.add(pwd_label);

    // Add password input
    let password_data = Rc::new(RefCell::new(String::new()));
    let pwd_input = InputLineBuilder::new()
        .bounds(Rect::new(14, 4, 45, 5))
        .data(password_data)
        .max_length(30)
        .build_boxed();
    dialog.add(pwd_input);

    // Add buttons
    let ok_button = ButtonBuilder::new()
        .bounds(Rect::new(15, 10, 25, 12))
        .title("O~k~")
        .command(CM_OK)
        .default(true)
        .build_boxed();
    dialog.add(ok_button);

    let cancel_button = ButtonBuilder::new()
        .bounds(Rect::new(27, 10, 37, 12))
        .title("Cancel")
        .command(CM_CANCEL)
        .build_boxed();
    dialog.add(cancel_button);

    dialog
}
```

**Benefits over resource files:**
- Compile-time checking of bounds and control relationships
- Immediate feedback on typos or incorrect parameters
- Full IDE support for exploring dialog structure
- Easy to refactor and reorganize

**Implementation**: See `src/views/dialog.rs` for dialog construction and `demo/rust_editor.rs` for complete dialog examples.

### String Lists Alternative

For scenarios where the original system used `TStringList` (strings accessed by numeric ID), use Rust's standard collections:

```rust
use std::collections::HashMap;

// Option 1: Static array (best for fixed, compile-time strings)
const ERROR_MESSAGES: &[&str] = &[
    "File not found",                    // 0
    "Permission denied",                 // 1
    "Invalid format",                    // 2
    "Out of memory",                     // 3
];

fn get_error_message(code: usize) -> &'static str {
    ERROR_MESSAGES.get(code).unwrap_or(&"Unknown error")
}

// Option 2: Lazy static for runtime initialization
use std::sync::OnceLock;

static UI_STRINGS: OnceLock<HashMap<u16, String>> = OnceLock::new();

fn get_ui_strings() -> &'static HashMap<u16, String> {
    UI_STRINGS.get_or_init(|| {
        let mut map = HashMap::new();
        map.insert(100, "Welcome to the application".to_string());
        map.insert(101, "Please select an option".to_string());
        map.insert(102, "Processing...".to_string());
        map
    })
}

fn get_ui_string(id: u16) -> Option<&'static String> {
    get_ui_strings().get(&id)
}
```

For internationalization, consider the `fluent` or `gettext` crates, which provide professional translation support with plural rules, gender agreement, and more.

## Centralized Resource Management

For larger applications, centralize resource definitions in a dedicated module:

```rust
// src/resources/mod.rs
pub mod menus;
pub mod status;
pub mod dialogs;
pub mod strings;

// src/resources/menus.rs
use turbo_vision::core::menu_data::{Menu, MenuBuilder};
use turbo_vision::core::command::*;
use turbo_vision::core::event::*;

pub fn create_main_menu() -> Vec<SubMenu> {
    vec![
        SubMenu::new("~F~ile", create_file_menu()),
        SubMenu::new("~E~dit", create_edit_menu()),
        SubMenu::new("~W~indow", create_window_menu()),
        SubMenu::new("~H~elp", create_help_menu()),
    ]
}

fn create_file_menu() -> Menu {
    MenuBuilder::new()
        .item_with_shortcut("~N~ew", CM_NEW, KB_CTRL_N, "Ctrl+N")
        .item_with_shortcut("~O~pen", CM_OPEN, KB_F3, "F3")
        .separator()
        .item_with_shortcut("E~x~it", CM_QUIT, KB_ALT_X, "Alt+X")
        .build()
}

fn create_edit_menu() -> Menu {
    MenuBuilder::new()
        .item_with_shortcut("~U~ndo", CM_UNDO, KB_CTRL_Z, "Ctrl+Z")
        .separator()
        .item_with_shortcut("Cu~t~", CM_CUT, KB_SHIFT_DEL, "Shift+Del")
        .item_with_shortcut("~C~opy", CM_COPY, KB_CTRL_INS, "Ctrl+Ins")
        .item_with_shortcut("~P~aste", CM_PASTE, KB_SHIFT_INS, "Shift+Ins")
        .build()
}

// ... more menu constructors

// src/resources/dialogs.rs
use turbo_vision::views::dialog::Dialog;

pub fn create_about_dialog() -> Dialog {
    // ... dialog construction
}

pub fn create_preferences_dialog() -> Dialog {
    // ... dialog construction
}

// Application usage:
use resources::menus;
use resources::dialogs;

let menu_bar = MenuBar::new(
    Rect::new(0, 0, width as i16, 1),
    menus::create_main_menu(),
);
```

This provides:
- **Single Source of Truth**: All UI definitions in one place
- **Easy Maintenance**: Change menus or dialogs in one location
- **Reusability**: Share dialogs between different parts of the application
- **Organization**: Clear structure for large applications

## Adding External Resource Support

While programmatic resources are the default, you can add external resource loading if your application needs runtime customization. Here are several approaches:

### 1. Configuration Files with Serde

For user preferences, window layouts, or application settings:

```toml
# Add to Cargo.toml
[dependencies]
serde = { version = "1.0", features = ["derive"] }
toml = "0.8"  # or json = "1.0" or ron = "0.8"
```

```rust
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
struct AppConfig {
    theme: String,
    window_positions: Vec<WindowPosition>,
    recent_files: Vec<String>,
    language: String,
}

#[derive(Serialize, Deserialize)]
struct WindowPosition {
    title: String,
    x: i16,
    y: i16,
    width: u16,
    height: u16,
}

// Load configuration
fn load_config(path: &str) -> Result<AppConfig, Box<dyn std::error::Error>> {
    let content = std::fs::read_to_string(path)?;
    let config: AppConfig = toml::from_str(&content)?;
    Ok(config)
}

// Save configuration
fn save_config(config: &AppConfig, path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let content = toml::to_string_pretty(config)?;
    std::fs::write(path, content)?;
    Ok(())
}
```

**Example TOML file:**
```toml
theme = "dark"
language = "en"
recent_files = [
    "/home/user/project/main.rs",
    "/home/user/docs/notes.txt",
]

[[window_positions]]
title = "main.rs"
x = 10
y = 5
width = 80
height = 25
```

### 2. Internationalization with External Files

For multi-language support, use resource files for strings:

```rust
use serde::{Serialize, Deserialize};
use std::collections::HashMap;

#[derive(Serialize, Deserialize)]
struct LanguageResources {
    strings: HashMap<String, String>,
}

// Load language resource
fn load_language(lang_code: &str) -> Result<LanguageResources, Box<dyn std::error::Error>> {
    let path = format!("resources/lang/{}.toml", lang_code);
    let content = std::fs::read_to_string(path)?;
    let resources: LanguageResources = toml::from_str(&content)?;
    Ok(resources)
}

// Global resource accessor
use std::sync::OnceLock;

static CURRENT_LANG: OnceLock<LanguageResources> = OnceLock::new();

fn tr(key: &str) -> String {
    CURRENT_LANG
        .get()
        .and_then(|lang| lang.strings.get(key))
        .map(|s| s.clone())
        .unwrap_or_else(|| format!("Missing: {}", key))
}

// Usage in menus
MenuBuilder::new()
    .item(&tr("menu.file.new"), CM_NEW, 0)
    .item(&tr("menu.file.open"), CM_OPEN, 0)
    .item(&tr("menu.file.exit"), CM_QUIT, 0)
    .build()
```

**Example language file (resources/lang/en.toml):**
```toml
[strings]
"menu.file.new" = "~N~ew"
"menu.file.open" = "~O~pen"
"menu.file.save" = "~S~ave"
"menu.file.exit" = "E~x~it"
"dialog.about.title" = "About"
"dialog.about.version" = "Version 1.0"
```

**Example language file (resources/lang/es.toml):**
```toml
[strings]
"menu.file.new" = "~N~uevo"
"menu.file.open" = "~A~brir"
"menu.file.save" = "~G~uardar"
"menu.file.exit" = "~S~alir"
"dialog.about.title" = "Acerca de"
"dialog.about.version" = "Versión 1.0"
```

### 3. Custom UI Definition Format

For applications that need runtime UI customization, define a custom format:

```rust
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
struct MenuDefinition {
    items: Vec<MenuItemDef>,
}

#[derive(Serialize, Deserialize)]
#[serde(tag = "type")]
enum MenuItemDef {
    Regular {
        text: String,
        command: u16,
        key_code: u16,
        shortcut: Option<String>,
    },
    Separator,
    SubMenu {
        text: String,
        items: Vec<MenuItemDef>,
    },
}

// Convert definition to runtime menu
fn build_menu_from_def(def: MenuDefinition) -> Menu {
    let mut items = Vec::new();
    for item_def in def.items {
        match item_def {
            MenuItemDef::Regular { text, command, key_code, shortcut } => {
                items.push(if let Some(sc) = shortcut {
                    MenuItem::with_shortcut(&text, command, key_code, &sc, 0)
                } else {
                    MenuItem::new(&text, command, key_code, 0)
                });
            }
            MenuItemDef::Separator => {
                items.push(MenuItem::separator());
            }
            MenuItemDef::SubMenu { text, items: sub_items } => {
                let submenu = build_menu_from_def(MenuDefinition { items: sub_items });
                items.push(MenuItem::submenu(&text, 0, submenu, 0));
            }
        }
    }
    Menu::from_items(items)
}
```

**Example menu definition (menus/file.json):**
```json
{
  "items": [
    {
      "type": "Regular",
      "text": "~N~ew",
      "command": 100,
      "key_code": 19,
      "shortcut": "Ctrl+N"
    },
    {
      "type": "Regular",
      "text": "~O~pen",
      "command": 101,
      "key_code": 61,
      "shortcut": "F3"
    },
    {
      "type": "Separator"
    },
    {
      "type": "Regular",
      "text": "E~x~it",
      "command": 27,
      "key_code": 45,
      "shortcut": "Alt+X"
    }
  ]
}
```

### 4. Hot Reloading for Development

For rapid UI iteration during development:

```rust
use notify::{Watcher, RecursiveMode, watcher};
use std::sync::mpsc::channel;
use std::time::Duration;

struct ResourceManager {
    menu_path: String,
    cached_menu: Option<Menu>,
}

impl ResourceManager {
    fn watch_for_changes(&mut self) -> notify::Result<()> {
        let (tx, rx) = channel();
        let mut watcher = watcher(tx, Duration::from_secs(1))?;

        watcher.watch(&self.menu_path, RecursiveMode::NonRecursive)?;

        loop {
            match rx.recv() {
                Ok(_event) => {
                    println!("Resource file changed, reloading...");
                    self.reload_menu();
                }
                Err(e) => println!("Watch error: {:?}", e),
            }
        }
    }

    fn reload_menu(&mut self) {
        match self.load_menu_from_file() {
            Ok(menu) => {
                self.cached_menu = Some(menu);
                println!("Menu reloaded successfully");
            }
            Err(e) => println!("Failed to reload menu: {}", e),
        }
    }

    fn load_menu_from_file(&self) -> Result<Menu, Box<dyn std::error::Error>> {
        // Load and parse menu definition
        // ...
    }
}
```

This approach is useful during development but should typically be disabled in release builds.

## Comparison: Original vs. Current Implementation

| Feature | Original Pascal | Current Rust |
|---------|----------------|--------------|
| **Menu Definition** | Binary `*.TVR` files or code | Programmatic builders (code only) |
| **Status Lines** | Binary `*.TVR` files or code | Programmatic builders (code only) |
| **Dialog Boxes** | Binary `*.TVR` files or code | Programmatic construction (code only) |
| **String Lists** | `TStringList` with numeric IDs | Standard collections (`Vec`, `HashMap`) |
| **Type Safety** | Runtime (stream registration) | Compile-time (full type checking) |
| **External Resources** | Built-in (`TResourceFile`) | Optional (via `serde` ecosystem) |
| **Customization** | Edit `*.TVR` files | Recompile or add external config |
| **Internationalization** | Swap resource files | Add `serde` with language files |
| **Tool Support** | Resource workshop utilities | Standard IDE features + Rust tooling |
| **Performance** | Runtime deserialization | Zero-cost (compiled in) |
| **Version Control** | Binary files (merge conflicts) | Source code (clean diffs) |

## Design Philosophy

The Rust implementation makes a clear tradeoff:

**Original Approach (Resource Files):**
- ✅ Runtime customization without recompilation
- ✅ Easy language switching
- ✅ Designer-friendly (no programming required)
- ❌ Binary format (hard to version control)
- ❌ Runtime errors possible (missing resources, wrong types)
- ❌ Need special tools to create/edit resources
- ❌ Harder to debug (can't step through initialization)

**Rust Approach (Programmatic Definition):**
- ✅ Compile-time verification (catch errors early)
- ✅ Excellent IDE support (autocomplete, refactoring)
- ✅ Version control friendly (text-based, clean diffs)
- ✅ Easy to debug (step through construction)
- ✅ No special tools required
- ❌ Must recompile for UI changes
- ❌ Internationalization requires more setup
- ❌ Non-programmers can't customize

For most modern applications, the Rust approach provides better safety, maintainability, and developer experience. When runtime customization is needed, targeted use of `serde` and configuration files provides a clean, idiomatic solution.

## Summary

This chapter explored resources in Turbo Vision:

1. **Original Resource System**: The Pascal version used `TResourceFile` to store and retrieve objects from binary `*.TVR` files, enabling runtime customization and internationalization without recompilation.

2. **Why Resources Were Valuable**: They provided code size savings, language-specific versions, feature variants, and separation of concerns between designers and programmers.

3. **String Lists**: The original system included `TStringList` for accessing strings by numeric ID, useful for error messages and internationalization.

4. **Rust's Approach**: The current implementation uses programmatic resource definition with builder patterns, providing compile-time type safety and excellent IDE support at the cost of runtime flexibility.

5. **Programmatic Patterns**: Menus, status lines, and dialog boxes are defined directly in Rust code using builders and data structures from `src/core/menu_data.rs`, `src/core/status_data.rs`, and `src/views/dialog.rs`.

6. **Adding External Resources**: When runtime customization is needed, the `serde` ecosystem provides modern solutions for configuration files, internationalization, and custom UI definitions using JSON, TOML, or RON formats.

7. **Centralized Management**: Larger applications should centralize resource definitions in dedicated modules for maintainability and reusability.

The Rust implementation's programmatic approach reflects modern development practices where compile-time safety, version control integration, and IDE support are prioritized over runtime flexibility. When external resources are needed, Rust's ecosystem provides powerful, type-safe alternatives to binary resource files.

**Key Files to Explore**:
- `src/core/menu_data.rs` - Menu definitions and builders (356 lines)
- `src/core/status_data.rs` - Status line definitions and builders (268 lines)
- `src/views/dialog.rs` - Dialog box construction
- `demo/rust_editor.rs` - Complete application showing all resource patterns
- `docs/user-guide/Chapter-4-Persistence-and-Configuration.md` - Related persistence concepts
- `docs/user-guide/Chapter-16-Collections-and-Streams.md` - Collection patterns
- `docs/user-guide/Chapter-17-Streams-and-Persistence.md` - Serialization approaches
