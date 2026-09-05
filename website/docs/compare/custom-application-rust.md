# Theoretical Customizations in Rust: Application Variants

This document shows how the theoretical customizations from `TProgram` derivatives would be implemented in the Rust Turbo Vision port, demonstrating **composition over inheritance**.

## Key Difference: Rust's Approach

Instead of inheritance-based selective initialization, Rust uses:
- **Compile-time feature flags** for optional subsystems
- **Builder pattern** for application configuration
- **Composition** instead of class hierarchies

---

## Example 1: Minimal Application (No History Lists)

**Use Case:** A simple utility that doesn't need input field history.

```rust
// Rust implementation using feature flags
use turbo_vision::app::Application;
use turbo_vision::views::{MenuBar, StatusLine, Desktop};
use turbo_vision::core::{Rect, command::*, event::*};

struct MinimalAppConfig {
    enable_history: bool,
}

impl Default for MinimalAppConfig {
    fn default() -> Self {
        Self {
            enable_history: false, // Disable history!
        }
    }
}

fn main() -> std::io::Result<()> {
    // Application with custom configuration
    let mut app = Application::new()?;

    // Add minimal UI components
    let menu = create_minimal_menu(&app);
    app.set_menu_bar(menu);

    let status = create_minimal_status(&app);
    app.set_status_line(status);

    // Note: In Rust, history is typically opt-in per control
    // Rather than a global subsystem, so this is more granular

    app.run();
    Ok(())
}

fn create_minimal_menu(app: &Application) -> MenuBar {
    use turbo_vision::core::menu_data::*;

    let menu = MenuBuilder::new()
        .item(MenuItem::submenu(
            "~F~ile",
            KB_ALT_F,
            Menu::new(vec![
                MenuItemBuilder::new().text("E~x~it").command(CM_QUIT).key("Alt+X").build(),
            ]),
            0,
        ))
        .build();

    MenuBar::new(menu)
}

fn create_minimal_status(app: &Application) -> StatusLine {
    use turbo_vision::core::status_data::StatusItemBuilder;
    use turbo_vision::views::status_line::*;

    let (width, height) = app.terminal.size();

    StatusLine::new(
        Rect::new(0, height as i16 - 1, width as i16, height as i16),
        vec![
            StatusItemBuilder::new().text("~F10~ Menu").key("F10").build(),
            StatusItemBuilder::new().text("~Alt-X~ Exit").key("Alt+X").command(CM_QUIT).build(),
        ]
    )
}
```

---

## Example 2: Read-Only Display Application

**Use Case:** A log viewer that only displays information.

```rust
use turbo_vision::app::Application;
use turbo_vision::views::{Window, StatusLine, TextView};
use turbo_vision::core::{Rect, command::*, event::*, state::*};
use std::time::Duration;

struct DisplayApp {
    app: Application,
    display_only: bool,
}

impl DisplayApp {
    fn new() -> std::io::Result<Self> {
        let mut app = Application::new()?;

        // No menu bar - simpler interface
        let (width, height) = app.terminal.size();

        let status_line = StatusLine::new(
            Rect::new(0, height as i16 - 1, width as i16, height as i16),
            vec![
                StatusItemBuilder::new().text("~Esc~ Exit").key("Esc").command(CM_QUIT).build(),
            ]
        );
        app.set_status_line(status_line);

        Ok(Self {
            app,
            display_only: true,
        })
    }

    fn add_display_window(&mut self, title: &str, content: &str) {
        let mut window = Window::new(
            Rect::new(10, 5, 70, 20),
            title,
            1
        );

        // Add read-only text view
        let text_view = TextView::new(
            Rect::new(1, 1, 58, 13),
            content.to_string()
        );

        window.add(text_view);

        // Make window non-interactive
        window.set_state(SF_DRAGGABLE, false);
        window.set_state(SF_CLOSEABLE, false);

        self.app.desktop.add(window);
    }

    fn run(&mut self) {
        self.app.run();
    }

    fn shutdown(&mut self) -> std::io::Result<()> {
        self.app.terminal.shutdown()
    }
}

fn main() -> std::io::Result<()> {
    let mut app = DisplayApp::new()?;

    app.add_display_window(
        "System Status",
        "Monitoring system...\n\nCPU: 45%\nMemory: 2.1GB\nDisk: 120GB free"
    );

    app.run();
    app.shutdown()
}
```

---

## Example 3: Embedded System Application

**Use Case:** Running on embedded hardware with limited memory.

```rust
use turbo_vision::app::Application;
use turbo_vision::views::{Window, StatusLine, MenuBar};
use turbo_vision::core::{Rect, command::*, event::*};

struct EmbeddedAppConfig {
    available_memory: usize,
    low_memory_mode: bool,
}

impl EmbeddedAppConfig {
    fn new(available_memory: usize) -> Self {
        Self {
            available_memory,
            low_memory_mode: available_memory < 64 * 1024, // < 64KB
        }
    }
}

struct EmbeddedApp {
    app: Application,
    config: EmbeddedAppConfig,
}

impl EmbeddedApp {
    fn new(config: EmbeddedAppConfig) -> std::io::Result<Self> {
        let app = Application::new()?;

        Ok(Self { app, config })
    }

    fn init_ui(&mut self) {
        if self.config.low_memory_mode {
            // Minimal UI for low memory
            self.init_minimal_ui();
        } else {
            // Full UI
            self.init_full_ui();
        }
    }

    fn init_minimal_ui(&mut self) {
        // Minimal status line only
        let (width, height) = self.app.terminal.size();

        let status_line = StatusLine::new(
            Rect::new(0, height as i16 - 1, width as i16, height as i16),
            vec![
                StatusItemBuilder::new().text("Quit").key("Alt+X").command(CM_QUIT).build(),
            ]
        );
        self.app.set_status_line(status_line);

        // No menu bar in low memory mode
    }

    fn init_full_ui(&mut self) {
        use turbo_vision::core::menu_data::*;

        // Full menu bar
        let menu = MenuBuilder::new()
            .item(MenuItem::submenu(
                "~S~ystem",
                KB_ALT_S,
                Menu::new(vec![
                    MenuItemBuilder::new().text("~S~tatus").command(CM_STATUS).key("F1").build(),
                    MenuItemBuilder::new().text("~R~eset").command(CM_RESET).key("Ctrl+R").build(),
                    MenuItem::separator(),
                    MenuItemBuilder::new().text("E~x~it").command(CM_QUIT).key("Alt+X").build(),
                ]),
                0,
            ))
            .build();

        self.app.set_menu_bar(MenuBar::new(menu));

        // Full status line
        let (width, height) = self.app.terminal.size();

        let status_line = StatusLine::new(
            Rect::new(0, height as i16 - 1, width as i16, height as i16),
            vec![
                StatusItemBuilder::new().text("~F1~ Status").key("F1").command(CM_STATUS).build(),
                StatusItemBuilder::new().text("~Ctrl-R~ Reset").key("Ctrl+R").command(CM_RESET).build(),
                StatusItemBuilder::new().text("~Alt-X~ Quit").key("Alt+X").command(CM_QUIT).build(),
            ]
        );
        self.app.set_status_line(status_line);
    }

    fn handle_out_of_memory(&mut self) {
        // Embedded-specific handling
        eprintln!("CRITICAL: Out of memory!");

        // In real embedded system, might trigger watchdog reset
        #[cfg(feature = "embedded")]
        {
            // embedded_hal::reset();
        }

        #[cfg(not(feature = "embedded"))]
        {
            std::process::exit(1);
        }
    }

    fn idle(&mut self) {
        // In embedded systems, yield CPU during idle
        #[cfg(feature = "embedded")]
        {
            // rtos::task_yield();
        }

        #[cfg(not(feature = "embedded"))]
        {
            std::thread::sleep(std::time::Duration::from_millis(10));
        }
    }

    fn run(&mut self) {
        self.app.run();
    }
}

// Command constants
const CM_STATUS: u16 = 1000;
const CM_RESET: u16 = 1001;

fn main() -> std::io::Result<()> {
    // Detect available memory (simplified)
    let available_memory = 128 * 1024; // 128KB

    let config = EmbeddedAppConfig::new(available_memory);
    let mut app = EmbeddedApp::new(config)?;

    app.init_ui();
    app.run();

    Ok(())
}
```

---

## Example 4: Builder Pattern for Application Configuration

**Rust Idiomatic Approach:** Instead of inheritance, use a builder pattern.

```rust
use turbo_vision::app::Application;
use turbo_vision::views::{MenuBar, StatusLine};
use turbo_vision::core::{Rect, command::*};

/// Application configuration builder
struct ApplicationBuilder {
    enable_menu: bool,
    enable_status: bool,
    enable_history: bool,
    enable_mouse: bool,
    menu_items: Option<MenuBar>,
    status_items: Option<Vec<StatusItem>>,
}

impl ApplicationBuilder {
    fn new() -> Self {
        Self {
            enable_menu: true,
            enable_status: true,
            enable_history: true,
            enable_mouse: true,
            menu_items: None,
            status_items: None,
        }
    }

    fn with_menu(mut self, enable: bool) -> Self {
        self.enable_menu = enable;
        self
    }

    fn with_status(mut self, enable: bool) -> Self {
        self.enable_status = enable;
        self
    }

    fn with_history(mut self, enable: bool) -> Self {
        self.enable_history = enable;
        self
    }

    fn with_mouse(mut self, enable: bool) -> Self {
        self.enable_mouse = enable;
        self
    }

    fn menu_bar(mut self, menu: MenuBar) -> Self {
        self.menu_items = Some(menu);
        self
    }

    fn status_line(mut self, items: Vec<StatusItem>) -> Self {
        self.status_items = Some(items);
        self
    }

    fn build(self) -> std::io::Result<Application> {
        let mut app = Application::new()?;

        if self.enable_menu {
            if let Some(menu) = self.menu_items {
                app.set_menu_bar(menu);
            }
        }

        if self.enable_status {
            if let Some(items) = self.status_items {
                let (width, height) = app.terminal.size();
                let status_line = StatusLine::new(
                    Rect::new(0, height as i16 - 1, width as i16, height as i16),
                    items
                );
                app.set_status_line(status_line);
            }
        }

        // History would be configured per-input control
        // Mouse handling is part of terminal

        Ok(app)
    }
}

// Usage examples

fn example_minimal() -> std::io::Result<()> {
    // Minimal app: no menu, no history
    let mut app = ApplicationBuilder::new()
        .with_menu(false)
        .with_history(false)
        .status_line(vec![
            StatusItemBuilder::new().text("~Esc~ Exit").key("Esc").command(CM_QUIT).build(),
        ])
        .build()?;

    app.run();
    Ok(())
}

fn example_display_only() -> std::io::Result<()> {
    // Display-only: no menu, no mouse
    let mut app = ApplicationBuilder::new()
        .with_menu(false)
        .with_mouse(false)
        .with_history(false)
        .status_line(vec![
            StatusItemBuilder::new().text("Read-only mode").build(),
        ])
        .build()?;

    app.run();
    Ok(())
}

fn example_full() -> std::io::Result<()> {
    use turbo_vision::core::menu_data::*;

    // Full app: everything enabled
    let menu = MenuBuilder::new()
        .item(MenuItem::submenu(
            "~F~ile",
            KB_ALT_F,
            Menu::new(vec![
                MenuItemBuilder::new().text("~O~pen").command(CM_OPEN).key("F3").build(),
                MenuItemBuilder::new().text("E~x~it").command(CM_QUIT).key("Alt+X").build(),
            ]),
            0,
        ))
        .build();

    let mut app = ApplicationBuilder::new()
        .menu_bar(MenuBar::new(menu))
        .status_line(vec![
            StatusItemBuilder::new().text("~F10~ Menu").key("F10").build(),
            StatusItemBuilder::new().text("~Alt-X~ Exit").key("Alt+X").command(CM_QUIT).build(),
        ])
        .build()?;

    app.run();
    Ok(())
}

fn main() -> std::io::Result<()> {
    example_minimal()
}
```

---

## Example 5: A Custom Window Type

**Use Case:** A window that draws something of its own and reacts to one extra
command, the way `TDialog` specialises `TWindow`.

Borland subclasses `TWindow` and overrides `handleEvent` and `draw`, calling
the base first. The Rust shape is the same: implement `GroupLike` and
`WindowLike` over a `Window`, then let `impl_view_for_window!` generate the
`View` implementation, writing only the overrides inline. The `window_*`
methods are the base calls.

```rust
use turbo_vision::core::command::{CM_USER, CommandId};
use turbo_vision::core::event::Event;
use turbo_vision::core::geometry::Rect;
use turbo_vision::impl_view_for_window;
use turbo_vision::terminal::Terminal;
use turbo_vision::views::group::{Group, GroupLike};
use turbo_vision::views::window::{Window, WindowLike};

const CM_TOGGLE_BANNER: CommandId = CM_USER + 1;

pub struct BannerWindow {
    window: Window,
    show_banner: bool,
}

impl BannerWindow {
    pub fn new(bounds: Rect) -> Self {
        Self { window: Window::new(bounds, "Banner"), show_banner: true }
    }
}

impl GroupLike for BannerWindow {
    fn group(&self) -> &Group { self.window.group() }
    fn group_mut(&mut self) -> &mut Group { self.window.group_mut() }
}

impl WindowLike for BannerWindow {
    fn window(&self) -> &Window { &self.window }
    fn window_mut(&mut self) -> &mut Window { &mut self.window }
}

impl_view_for_window!(BannerWindow {
    fn draw(&mut self, terminal: &mut Terminal) {
        self.window_draw(terminal); // TWindow::draw()
        if self.show_banner {
            // paint the banner over the interior here
        }
    }

    fn handle_event(&mut self, event: &mut Event) {
        self.window_handle_event(event); // TWindow::handleEvent(event)
        if event.what == turbo_vision::core::event::EventType::Command
            && event.command == CM_TOGGLE_BANNER
        {
            self.show_banner = !self.show_banner;
            event.clear();
        }
    }
});
```

Every other `View` method (`set_bounds`, `valid`, `get_palette`, `zoom`,
`as_any`, ...) is forwarded to the `Window` base by the macro, so the type
cannot forward selectively and fall back to a trait default by accident.

---

## Feature Flag Based Configuration

For true compile-time optimization, use Cargo features:

```toml
# Cargo.toml
[features]
default = ["menu-bar", "status-line", "history", "mouse"]

# Optional subsystems
menu-bar = []
status-line = []
history = []
mouse = []

# Presets
minimal = []
embedded = ["status-line"]  # Only status line
display-only = ["status-line"]  # No input features
```

```rust
// src/app/mod.rs
pub struct Application {
    pub terminal: Terminal,

    #[cfg(feature = "menu-bar")]
    pub menu_bar: Option<MenuBar>,

    #[cfg(feature = "status-line")]
    pub status_line: Option<StatusLine>,

    pub desktop: Desktop,
    pub running: bool,
}

impl Application {
    pub fn new() -> std::io::Result<Self> {
        let terminal = Terminal::init()?;
        let (width, height) = terminal.size();

        #[cfg(feature = "status-line")]
        let desktop_height = height as i16 - 1;

        #[cfg(not(feature = "status-line"))]
        let desktop_height = height as i16;

        let desktop = Desktop::new(Rect::new(0, 1, width as i16, desktop_height));

        Ok(Self {
            terminal,
            #[cfg(feature = "menu-bar")]
            menu_bar: None,
            #[cfg(feature = "status-line")]
            status_line: None,
            desktop,
            running: false,
        })
    }
}
```

Usage:
```bash
# Minimal build (no menu, status, or history)
cargo build --no-default-features

# Embedded build (only status line)
cargo build --no-default-features --features embedded

# Full build (default)
cargo build
```

---

## Summary: Rust vs C++ Approach

| C++ (Inheritance) | Rust (Layered Traits) |
|-------------------|-------------------|
| Derive from TProgram | Implement `AppHandler` and call `run_with` |
| Derive from TWindow, override `handleEvent` | Implement `WindowLike`, override inside `impl_view_for_window!` |
| Call the base method first | Call the `window_*` / `group_*` base method first |
| Override constructor | Use builder pattern |
| Selective Init* calls | Conditional compilation |
| Runtime overhead | Zero-cost abstraction |

**Rust Advantages:**
1. **Compile-time optimization** - Unused code eliminated at compile time
2. **No runtime overhead** - No virtual dispatch, no unused subsystems
3. **Type safety** - Features checked at compile time
4. **Simpler code** - No inheritance hierarchies
5. **Better for embedded** - Can strip entire subsystems from binary

This demonstrates why the Rust port doesn't need a separate `TProgram` type - the same flexibility is achieved more elegantly through Rust's type system and compilation features.
