# Turbo Vision - Rust TUI Library

![Turbo Vision Logo](https://raw.githubusercontent.com/aovestdipaperino/turbo-vision-4-rust/main/logo.png)

A Rust implementation of the classic Borland Turbo Vision text user interface framework.

**Version 0.10.0 - CODE COMPLETE** ✅

Based on
kloczek Borland Turbo Vision C++ port [here](https://github.com/kloczek/tvision)

Other C++ implementations:
- [Magiblot Turbo Vision for C++](https://github.com/magiblot/tvision)
- [Borland Original Turbo Vision 2.0.3 code](http://www.sigala.it/sergio/tvision/source/tv2orig.zip)

This port achieves **100% API parity** with
kloczek port of Borland Turbo Vision C++. All features from the original framework have been implemented. While the codebase is complete and production-ready, it may contain bugs. Please report any issues you encounter!

## Features

- **Complete UI Component Set**: Windows, dialogs, buttons, input fields, menus, status bars, scrollbars
- **Z-Order Management**: Click any non-modal window to bring it to the front
- **Modal Dialog Support**: Modal dialogs block interaction with background windows
- **Borland-Accurate Styling**: Menu borders and shadows match original Borland Turbo Vision
- **Scrollable Views**: Built-in scrollbar support with keyboard navigation
- **Text Viewer**: Ready-to-use scrollable text viewer with line numbers
- **Event-Driven Architecture**:
  - Three-phase event processing (PreProcess → Focused → PostProcess)
  - Event re-queuing for deferred processing
  - Owner-aware broadcast system to prevent echo back to sender
- **Mouse Support**: Full mouse support for buttons, menus, status bar, dialog close buttons, scroll wheel, and double-click detection
- **Window Dragging and Resizing**: Drag windows by title bar, resize by bottom-right corner
- **Flexible Layout System**: Geometry primitives with absolute and relative positioning
- **Color Support**: 16-color palette with Borland-accurate attribute system and context-aware remapping
- **Cross-Platform**: Built on crossterm for wide terminal compatibility
- **Modal Dialogs**: Built-in support for modal dialog execution
- **Focus Management**: Tab navigation and keyboard shortcuts
- **ANSI Dump**: Debug UI by dumping screen/views to ANSI text files (F12 for full screen, Shift+F12 for active view, with flash effect)

## Quick Start

```rust
use turbo_vision::prelude::*;

fn main() -> turbo_vision::core::error::Result<()> {
    // Create application with terminal
    let mut app = Application::new()?;
    // Create a window
    let mut window = turbo_vision::views::window::Window::new(Rect::new(10, 5, 50, 15), "My First Window");
    // Create a button
    let button = turbo_vision::views::button::Button::new(
        Rect::new(15, 5, 25, 7),
        "Click Me",
        turbo_vision::core::command::CM_OK,
        false,
    );
    // Add the button to the window
    window.add(Box::new(button));
    // Add the window to the desktop
    app.desktop.add(Box::new(window));
    // Run event loop
    app.running = true;
    while app.running {
        app.desktop.draw(&mut app.terminal);
        app.terminal.flush()?;
        if let Ok(Some(mut event)) = app
            .terminal
            .poll_event(std::time::Duration::from_millis(50))
        {
            app.desktop.handle_event(&mut event);
            if event.what == EventType::Command {
                match event.command {
                    CM_QUIT => app.running = false,
                    CM_OK => {
                        // Handle button click
                        app.running = false;
                    }
                    _ => {}
                }
            }
        }
    }
    app.terminal.shutdown()?;
    Ok(())
}
```

**Tip**: Press F12 at any time to capture full screen to `screen-dump.txt`, or F11 to capture active window/dialog to `active-view-dump.txt` - both with a visual flash effect for debugging!

## Palette System

The color palette system accurately replicates Borland Turbo Vision's behavior:

- **Context-Aware Remapping**: Views automatically remap colors based on their container (Dialog, Window, or Desktop)
- **Owner Type Support**: Each view tracks its owner type for correct palette inheritance
- **Borland-Accurate Colors**: All UI elements (menus, buttons, labels, dialogs) match original Borland colors
- **Runtime Customization**: Change the entire application palette at runtime with `app.set_palette()` for custom themes
- **Regression Testing**: 9 comprehensive palette tests ensure color stability across changes

The palette system uses a three-level mapping chain:
1. View palette (e.g., Button, Label) → indices 1-31
2. Container palette (Dialog/Window) → remaps to indices 32-63
3. Application palette → final RGB colors

### Custom Palettes and Theming

You can customize the entire application palette at runtime to create custom themes:

```rust
// Create a custom palette (63 bytes, each encoding foreground << 4 | background)
let dark_palette = vec![/* 63 color bytes */];

// Set the palette - redraw happens automatically!
app.set_palette(Some(dark_palette));

// Reset to default Borland palette
app.set_palette(None);
```

See `examples/palette_themes_demo.rs` for a complete example with multiple themes.

## Module Overview

- **core**: Fundamental types (geometry, events, drawing, colors)
- **terminal**: Terminal I/O abstraction layer
- **views**: UI components (dialogs, buttons, menus, etc.)
- **app**: Application framework and event loop

## Documentation

### Examples

Read [examples/README.md](examples/README.md) for a complete set of examples.

Compile all examples at once:

```bash
cargo build --examples           # Checkout `target/debug/examples`
cargo run --example full-demo    # Run the comprehensive demo
```

Build and run the demo `rust_editor` 

```bash
cargo run --bin rust_editor --release 
```

### User Guide
Read [Chapter 1](docs/user-guide/Chapter-1-Stepping-into-Turbo-Vision.md) of the User Guide. The 18 chapters are available in `docs/user-guide/` directory. 

### References
Read the [index](docs/DOCUMENTATION_INDEX.md). Other references are available in the `docs/` directory: 



## Status

Currently implements:
- ✅ Core drawing and event system
- ✅ Dialog boxes with frames and close buttons
- ✅ Buttons with keyboard shortcuts
- ✅ Static text labels (with centered text support)
- ✅ Input fields
- ✅ Menu bar with dropdowns and keyboard shortcut display
- ✅ Status line with hot spots (hover highlighting, context-sensitive hints)
- ✅ Desktop manager
- ✅ Scrollbars (vertical and horizontal)
- ✅ Scroller base class for scrollable views
- ✅ Indicator (position display)
- ✅ Text viewer with scrolling
- ✅ CheckBoxes
- ✅ RadioButtons
- ✅ ListBoxes
- ✅ Memo (multi-line text editor)
- ✅ Mouse support (buttons, menus, status bar, close buttons, hover effects, listbox clicks, scroll wheel, double-click detection)
- ✅ Window dragging and resizing (drag by title bar, resize from bottom-right corner with minimum size constraints)
- ✅ Window closing (non-modal windows close with close button, modal dialogs convert to cancel)
- ✅ File Dialog (fully functional with mouse/keyboard support and directory navigation)
- ✅ ANSI Dump for debugging (dump screen/views to text files with colors)
- ✅ Input Validators (FilterValidator, RangeValidator with hex/octal, LookupValidator)
- ✅ Editor with search/replace and file I/O (load_file, save_file, save_as)
- ✅ EditWindow (ready-to-use editor window wrapper)
- ✅ OS Clipboard integration (cross-platform with arboard)
- ✅ Help System (markdown-based with HelpFile, HelpViewer, HelpWindow, HelpContext)

## Architecture

This implementation closely follows Borland Turbo Vision's architecture, adapted for Rust:

- **Event Loop**: Located in `Group` (matching Borland's `TGroup::execute()`), not in individual views
- **Modal Dialogs**: Use Borland's `endModal()` pattern to exit event loops
- **View Hierarchy**: Composition-based design (`Window` contains `Group`, `Dialog` wraps `Window`)
- **Drawing**: Event-driven redraws with Borland's `drawUnderRect` pattern for efficient updates
- **Event System**:
  - Three-phase processing (PreProcess → Focused → PostProcess) matching Borland's `TGroup::handleEvent()`
  - Event re-queuing via `Terminal::put_event()` matching Borland's `TProgram::putEvent()`
  - Owner-aware broadcasts via `Group::broadcast()` matching Borland's `message(owner, ...)` pattern

## Project Statistics

```
===============================================================================
 Language            Files        Lines         Code     Comments       Blanks
===============================================================================
 Rust                  108        31264        23572         2865         4827
 |- Markdown            82         3504          209         2752          543
 (Total)                          34768        23781         5617         5370
===============================================================================
```

Generated with [tokei](https://github.com/XAMPPRocky/tokei) - includes inline documentation

**198 tests** - all passing ✅
- 189 unit tests
- 9 palette regression tests

## License

MIT License - see [LICENSE](LICENSE) file for details.

