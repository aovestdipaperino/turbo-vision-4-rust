# Chapter 4 — Persistence and Configuration (Rust Edition)

**Version:** 0.2.11
**Updated:** 2025-11-04

The original Borland Turbo Vision used **streams** to save and restore objects, and **resources** to load menus, status lines, and dialog boxes from external files. This chapter explains those concepts and provides modern Rust alternatives for achieving similar goals.

**Note:** The current Rust implementation does not include a built-in streams or resources system. This chapter explains the Pascal approach for educational purposes and shows idiomatic Rust alternatives.

In this chapter, you'll learn about:

- The Pascal streams and resources concepts
- Why Rust uses different approaches
- Modern Rust serialization with `serde`
- Configuration file approaches
- Application state management patterns

---

## Understanding Pascal's Streams and Resources

Before diving into Rust solutions, it's valuable to understand what the original Turbo Vision provided and why it was designed that way.

### What Were Streams?

In Borland's Pascal implementation, **streams** were a mechanism for serializing objects to disk or memory. The stream system:

- Wrote objects to a sequential byte stream
- Stored type information with each object
- Could save and restore complex object hierarchies
- Worked with desktop windows, menus, dialogs, and custom objects

Example Pascal code:

```pascal
// Saving
DesktopFile.Init('DESKTOP.TUT', stCreate, 1024);
DesktopFile.Put(Desktop);  // Save entire desktop with all windows
DesktopFile.Done;

// Loading
DesktopFile.Init('DESKTOP.TUT', stOpenRead, 1024);
TempDesktop := PDesktop(DesktopFile.Get);
DesktopFile.Done;
```

### What Were Resources?

**Resources** were named objects stored in a resource file. They allowed:

- External definition of UI elements (menus, status lines, dialogs)
- Loading UI components by name at runtime
- Changing the UI without recompiling the application

Example Pascal code:

```pascal
// Storing
ResFile.Put(MyMenu, 'MAINMENU');  // Save with name

// Loading
MenuBar := PMenuBar(ResFile.Get('MAINMENU'));  // Load by name
```

### Why This Approach Made Sense in Pascal

In the late 1980s/early 1990s:

1. **Limited memory** - Saving desktop state freed up RAM
2. **No configuration standards** - Custom binary formats were common
3. **Single-platform** - DOS-only, no cross-platform concerns
4. **Compiled UI** - Changing menus required recompilation without resources

---

## Why Rust Does Things Differently

The Rust Turbo Vision implementation takes a different approach for several reasons:

### 1. Modern Memory Management

Today's systems have abundant RAM. Saving desktop state to free memory is rarely necessary. Instead:

```rust
// Keep application state in memory
struct AppState {
    open_files: Vec<PathBuf>,
    window_positions: HashMap<String, Rect>,
    user_preferences: UserPrefs,
}
```

### 2. Type Safety and Ownership

Rust's ownership system makes dynamic object loading complex:

```rust
// Pascal: Load any object type at runtime
let obj = stream.Get();  // Returns PObject (any type)
let menu = PMenuBar(obj);  // Typecast

// Rust: Types must be known at compile time
let menu: MenuBar = load_menu()?;  // Type is explicit
```

### 3. Serialization Standards

Modern applications use standard formats:

- **JSON** - Human-readable, widely supported
- **TOML** - Configuration-focused, Rust-native
- **YAML** - Complex configurations
- **Binary formats** - When needed (MessagePack, bincode)

### 4. Build-Time Resources

Modern build systems can embed resources at compile time:

```rust
// Embed resource at compile time
const DEFAULT_MENU: &str = include_str!("menu.toml");
```

---

## Rust Solution 1: Serde Serialization

For saving and loading application state, use `serde` - Rust's standard serialization framework.

### Adding Serde to Your Project

```toml
# Cargo.toml
[dependencies]
turbo-vision = "0.2"
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
```

### Defining Serializable State

```rust
use serde::{Serialize, Deserialize};
use std::path::PathBuf;

#[derive(Serialize, Deserialize, Debug)]
struct AppState {
    open_files: Vec<PathBuf>,
    window_positions: Vec<WindowState>,
    last_search: Option<String>,
    user_preferences: UserPreferences,
}

#[derive(Serialize, Deserialize, Debug)]
struct WindowState {
    title: String,
    x: i16,
    y: i16,
    width: i16,
    height: i16,
}

#[derive(Serialize, Deserialize, Debug)]
struct UserPreferences {
    auto_indent: bool,
    tab_size: usize,
    syntax_highlighting: bool,
}
```

### Saving State to JSON

```rust
use std::fs::File;
use std::io::Write;

fn save_app_state(state: &AppState, path: &str) -> std::io::Result<()> {
    let json = serde_json::to_string_pretty(state)?;
    let mut file = File::create(path)?;
    file.write_all(json.as_bytes())?;
    Ok(())
}
```

### Loading State from JSON

```rust
use std::fs;

fn load_app_state(path: &str) -> std::io::Result<AppState> {
    let json = fs::read_to_string(path)?;
    let state = serde_json::from_str(&json)?;
    Ok(state)
}
```

### Example: Saving Editor State

```rust
// tutorial_09.rs - Saving and loading application state
use turbo_vision::app::Application;
use turbo_vision::core::geometry::Rect;
use turbo_vision::core::command::{CM_QUIT, CM_NEW, CM_OPEN};
use turbo_vision::core::event::EventType;
use serde::{Serialize, Deserialize};
use std::path::PathBuf;
use std::time::Duration;

#[derive(Serialize, Deserialize, Debug, Default)]
struct EditorState {
    open_files: Vec<PathBuf>,
    last_directory: Option<PathBuf>,
    preferences: EditorPreferences,
}

#[derive(Serialize, Deserialize, Debug)]
struct EditorPreferences {
    auto_indent: bool,
    tab_size: usize,
    show_line_numbers: bool,
}

impl Default for EditorPreferences {
    fn default() -> Self {
        Self {
            auto_indent: true,
            tab_size: 4,
            show_line_numbers: true,
        }
    }
}

impl EditorState {
    fn load() -> Self {
        match std::fs::read_to_string("editor_state.json") {
            Ok(json) => serde_json::from_str(&json).unwrap_or_default(),
            Err(_) => Self::default(),
        }
    }

    fn save(&self) {
        if let Ok(json) = serde_json::to_string_pretty(self) {
            let _ = std::fs::write("editor_state.json", json);
        }
    }
}

fn main() -> std::io::Result<()> {
    let mut app = Application::new()?;
    let mut state = EditorState::load();

    // ... event loop ...

    // Save state on exit
    state.save();
    Ok(())
}
```

---

## Rust Solution 2: Configuration Files

For menus, preferences, and UI configuration, use configuration files with `serde` and TOML.

### Adding TOML Support

```toml
# Cargo.toml
[dependencies]
turbo-vision = "0.2"
serde = { version = "1.0", features = ["derive"] }
toml = "0.8"
```

### Defining Configuration Structure

```rust
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug)]
struct AppConfig {
    window: WindowConfig,
    editor: EditorConfig,
    menu: MenuConfig,
}

#[derive(Serialize, Deserialize, Debug)]
struct WindowConfig {
    default_width: i16,
    default_height: i16,
    remember_position: bool,
}

#[derive(Serialize, Deserialize, Debug)]
struct EditorConfig {
    auto_indent: bool,
    tab_size: usize,
    syntax_highlighting: bool,
    theme: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct MenuConfig {
    show_icons: bool,
    items: Vec<MenuItem>,
}

#[derive(Serialize, Deserialize, Debug)]
struct MenuItem {
    name: String,
    command: String,
    shortcut: Option<String>,
}
```

### Example Configuration File

```toml
# config.toml
[window]
default_width = 80
default_height = 24
remember_position = true

[editor]
auto_indent = true
tab_size = 4
syntax_highlighting = true
theme = "default"

[menu]
show_icons = false

[[menu.items]]
name = "New"
command = "new"
shortcut = "Ctrl+N"

[[menu.items]]
name = "Open"
command = "open"
shortcut = "Ctrl+O"

[[menu.items]]
name = "Save"
command = "save"
shortcut = "Ctrl+S"
```

### Loading Configuration

```rust
use std::fs;

fn load_config(path: &str) -> Result<AppConfig, Box<dyn std::error::Error>> {
    let contents = fs::read_to_string(path)?;
    let config: AppConfig = toml::from_str(&contents)?;
    Ok(config)
}

fn load_config_or_default(path: &str) -> AppConfig {
    load_config(path).unwrap_or_else(|_| AppConfig::default())
}
```

---

## Rust Solution 3: Build-Time Resources

For static resources that don't change, embed them at compile time using Rust's `include_str!` macro.

### Embedding Menu Definitions

```rust
// menu_data.rs

// Embed menu definition at compile time
const MENU_CONFIG: &str = include_str!("../resources/menu.toml");

pub fn create_menu_from_config() -> Result<MenuBar, Box<dyn std::error::Error>> {
    let config: MenuConfig = toml::from_str(MENU_CONFIG)?;

    // Build menu bar from configuration
    let mut menu_bar = MenuBar::new(Rect::new(0, 0, 80, 1));

    for menu_def in config.menus {
        let submenu = create_submenu(&menu_def)?;
        menu_bar.add_submenu(submenu);
    }

    Ok(menu_bar)
}

fn create_submenu(def: &MenuDef) -> Result<SubMenu, Box<dyn std::error::Error>> {
    let mut items = Vec::new();

    for item in &def.items {
        let menu_item = MenuItem::with_shortcut(
            &item.label,
            item.command_id,
            item.key_code,
            &item.shortcut,
            0
        );
        items.push(menu_item);
    }

    Ok(SubMenu::new(&def.name, Menu::from_items(items)))
}
```

### Embedding Static Text

```rust
// help_text.rs

pub const ABOUT_TEXT: &str = include_str!("../resources/about.txt");
pub const HELP_TEXT: &str = include_str!("../resources/help.txt");
pub const LICENSE_TEXT: &str = include_str!("../resources/license.txt");

pub fn show_about(app: &mut Application) {
    message_box_ok(app, "About", ABOUT_TEXT);
}
```

---

## Practical Example: Complete Configuration System

Here's a complete example showing how to build a configurable Turbo Vision application.

### Project Structure

```
my_app/
├── Cargo.toml
├── config.toml              # User configuration
├── src/
│   ├── main.rs
│   ├── config.rs            # Configuration loading
│   ├── state.rs             # Application state
│   └── menu_builder.rs      # Menu construction from config
└── resources/
    ├── default_config.toml  # Embedded default config
    └── about.txt            # Embedded about text
```

### config.rs - Configuration Management

```rust
use serde::{Serialize, Deserialize};
use std::path::Path;
use std::fs;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Config {
    pub editor: EditorConfig,
    pub window: WindowConfig,
    pub ui: UiConfig,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct EditorConfig {
    pub auto_indent: bool,
    pub tab_size: usize,
    pub show_line_numbers: bool,
    pub syntax_highlighting: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct WindowConfig {
    pub default_width: i16,
    pub default_height: i16,
    pub remember_positions: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct UiConfig {
    pub show_status_bar: bool,
    pub show_toolbar: bool,
    pub theme: String,
}

impl Config {
    // Load from file, falling back to embedded default
    pub fn load() -> Self {
        const DEFAULT_CONFIG: &str = include_str!("../resources/default_config.toml");

        // Try to load user config
        if Path::new("config.toml").exists() {
            if let Ok(contents) = fs::read_to_string("config.toml") {
                if let Ok(config) = toml::from_str(&contents) {
                    return config;
                }
            }
        }

        // Fall back to default config
        toml::from_str(DEFAULT_CONFIG).unwrap_or_default()
    }

    pub fn save(&self) -> std::io::Result<()> {
        let toml = toml::to_string_pretty(self)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
        fs::write("config.toml", toml)?;
        Ok(())
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            editor: EditorConfig {
                auto_indent: true,
                tab_size: 4,
                show_line_numbers: true,
                syntax_highlighting: true,
            },
            window: WindowConfig {
                default_width: 80,
                default_height: 24,
                remember_positions: true,
            },
            ui: UiConfig {
                show_status_bar: true,
                show_toolbar: false,
                theme: "default".to_string(),
            },
        }
    }
}
```

### state.rs - Application State

```rust
use serde::{Serialize, Deserialize};
use std::path::PathBuf;
use turbo_vision::core::geometry::Rect;

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct AppState {
    pub open_files: Vec<PathBuf>,
    pub window_states: Vec<WindowState>,
    pub last_directory: Option<PathBuf>,
    pub recent_files: Vec<PathBuf>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct WindowState {
    pub title: String,
    pub bounds: SerializableRect,
    pub file_path: Option<PathBuf>,
}

// Helper for serializing Rect
#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
pub struct SerializableRect {
    pub x: i16,
    pub y: i16,
    pub width: i16,
    pub height: i16,
}

impl From<Rect> for SerializableRect {
    fn from(rect: Rect) -> Self {
        Self {
            x: rect.left(),
            y: rect.top(),
            width: rect.width(),
            height: rect.height(),
        }
    }
}

impl From<SerializableRect> for Rect {
    fn from(sr: SerializableRect) -> Self {
        Rect::new(sr.x, sr.y, sr.x + sr.width, sr.y + sr.height)
    }
}

impl AppState {
    const STATE_FILE: &'static str = "app_state.json";

    pub fn load() -> Self {
        match std::fs::read_to_string(Self::STATE_FILE) {
            Ok(json) => serde_json::from_str(&json).unwrap_or_default(),
            Err(_) => Self::default(),
        }
    }

    pub fn save(&self) -> std::io::Result<()> {
        let json = serde_json::to_string_pretty(self)?;
        std::fs::write(Self::STATE_FILE, json)?;
        Ok(())
    }

    pub fn add_recent_file(&mut self, path: PathBuf) {
        // Remove if already exists
        self.recent_files.retain(|p| p != &path);

        // Add to front
        self.recent_files.insert(0, path);

        // Keep only last 10
        self.recent_files.truncate(10);
    }
}
```

### main.rs - Using Configuration and State

```rust
use turbo_vision::app::Application;
use turbo_vision::core::geometry::Rect;
use turbo_vision::core::command::{CM_QUIT, CM_NEW, CM_OPEN};
use turbo_vision::core::event::EventType;
use turbo_vision::views::window::Window;
use turbo_vision::views::editor::Editor;
use std::time::Duration;

mod config;
mod state;

use config::Config;
use state::AppState;

fn main() -> std::io::Result<()> {
    // Load configuration and state
    let config = Config::load();
    let mut state = AppState::load();

    let mut app = Application::new()?;
    let (width, height) = app.terminal.size();

    // Create menu bar (hardcoded for now, could be from config)
    let menu_bar = create_menu_bar(width);
    app.set_menu_bar(menu_bar);

    // Create status line if configured
    if config.ui.show_status_bar {
        let status_line = create_status_line(height, width);
        app.set_status_line(status_line);
    }

    // Restore open windows from state
    for window_state in &state.window_states {
        if let Some(ref path) = window_state.file_path {
            if let Ok(content) = std::fs::read_to_string(path) {
                let mut window = Window::new(
                    window_state.bounds.into(),
                    &window_state.title
                );

                // Create editor with configured settings
                let editor_bounds = Rect::new(1, 1,
                    window_state.bounds.width - 2,
                    window_state.bounds.height - 2);
                let mut editor = Editor::new(editor_bounds)
                    .with_scrollbars_and_indicator();
                editor.set_text(&content);
                editor.set_auto_indent(config.editor.auto_indent);
                editor.set_tab_size(config.editor.tab_size);

                window.add(Box::new(editor));
                app.desktop.add(Box::new(window));
            }
        }
    }

    // Main event loop
    app.running = true;
    while app.running {
        // Draw
        app.desktop.draw(&mut app.terminal);
        if let Some(ref mut menu_bar) = app.menu_bar {
            menu_bar.draw(&mut app.terminal);
        }
        if let Some(ref mut status_line) = app.status_line {
            status_line.draw(&mut app.terminal);
        }
        let _ = app.terminal.flush();

        // Handle events (abbreviated)
        if let Ok(Some(mut event)) = app.terminal.poll_event(Duration::from_millis(50)) {
            // ... event handling as in previous chapters ...

            if event.what == EventType::Command {
                match event.command {
                    CM_QUIT => {
                        // Save state before exit
                        capture_window_states(&app, &mut state);
                        state.save()?;
                        config.save()?;
                        app.running = false;
                    }
                    _ => {}
                }
            }
        }
    }

    Ok(())
}

fn capture_window_states(app: &Application, state: &mut AppState) {
    state.window_states.clear();

    // Iterate through desktop windows and capture their state
    // (This would require additional API support in Window)
    // For now, this is a placeholder showing the concept
}

// ... create_menu_bar, create_status_line as in previous chapters ...
```

---

## Comparison: Pascal vs. Rust

| Feature | Pascal (1990s) | Rust (2025) |
|---------|---------------|-------------|
| **Object Serialization** | TStream with Put/Get | serde with JSON/TOML/bincode |
| **Type Information** | Runtime type IDs | Compile-time types with serde |
| **Configuration** | Binary resource files | Text files (JSON, TOML, YAML) |
| **Menu Loading** | ResFile.Get('MAINMENU') | Hardcoded or config-based builder |
| **Dialog Loading** | ResFile.Get('DIALOG') | Constructed in code or from config |
| **State Persistence** | Stream to DESKTOP.TUT | JSON state file |
| **Resource Embedding** | External .RES files | include_str! / include_bytes! |

---

## Summary

While the original Borland Turbo Vision used streams and resources for persistence and configuration, modern Rust applications use different approaches:

### For Application State:
- Use `serde` with JSON for human-readable state
- Use `bincode` for efficient binary serialization
- Save on exit, load on startup

### For Configuration:
- Use TOML for user-editable configuration files
- Provide sensible defaults
- Embed default configs at compile time

### For UI Resources:
- Build menus and dialogs in code
- Use configuration files for customization
- Embed static resources with `include_str!`

### Best Practices:

1. **Separate concerns** - Config, state, and code
2. **Fail gracefully** - Handle missing/corrupt config files
3. **Provide defaults** - Always have embedded defaults
4. **Use standard formats** - JSON/TOML are widely understood
5. **Version your formats** - Include version numbers in serialized data
6. **Test serialization** - Ensure forward/backward compatibility

---

## See Also

- **Chapter 3** - Adding Windows (window management basics)
- **Chapter 5** - Data Entry and Controls (complex UI construction)
- **serde documentation** - https://serde.rs/
- **TOML specification** - https://toml.io/
- **examples/file_dialog.rs** - File I/O patterns

---

In the next chapter, you'll learn about **data entry** and **controls**, creating forms with input fields, checkboxes, radio buttons, and other interactive UI elements.
