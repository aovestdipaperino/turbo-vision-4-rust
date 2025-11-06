# Turbo Vision - Rust TUI Library

![Turbo Vision Logo](https://raw.githubusercontent.com/aovestdipaperino/turbo-vision-4-rust/main/logo.png)

A Rust implementation of the classic Borland Turbo Vision text user interface framework.

**Version 0.9.0 - CODE COMPLETE** ✅

This port achieves **100% API parity** with Borland Turbo Vision C++. All features from the original framework have been implemented. While the codebase is complete and production-ready, it may contain bugs. Please report any issues you encounter!

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
- **Color Support**: 16-color palette with attribute system
- **Cross-Platform**: Built on crossterm for wide terminal compatibility
- **Modal Dialogs**: Built-in support for modal dialog execution
- **Focus Management**: Tab navigation and keyboard shortcuts
- **ANSI Dump**: Debug UI by dumping screen/views to ANSI text files (F12 for full screen, F11 for active view, with flash effect)

## Quick Start

```rust
use turbo_vision::prelude::*;
use turbo_vision::app::Application;
use turbo_vision::views::{
    dialog::Dialog,
    button::Button,
    static_text::StaticText,
};

fn main() -> std::io::Result<()> {
    let mut app = Application::new()?;

    // Create a simple dialog
    let mut dialog = Dialog::new(
        Rect::new(20, 8, 60, 16),
        "Hello World"
    );

    let text = StaticText::new(
        Rect::new(22, 10, 58, 12),
        "Welcome to Turbo Vision!"
    );
    dialog.add(Box::new(text));

    let button = Button::new(
        Rect::new(35, 13, 45, 15),
        "  OK  ",
        CM_OK,
        true
    );
    dialog.add(Box::new(button));

    dialog.execute(&mut app.terminal);
    Ok(())
}
```

**Tip**: Press F12 at any time to capture full screen to `screen-dump.txt`, or F11 to capture active window/dialog to `active-view-dump.txt` - both with a visual flash effect for debugging!

## Module Overview

- **core**: Fundamental types (geometry, events, drawing, colors)
- **terminal**: Terminal I/O abstraction layer
- **views**: UI components (dialogs, buttons, menus, etc.)
- **app**: Application framework and event loop

## Documentation

See the [examples](examples) for a complete simple examples.

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
- **Reference Implementation**: Studied original Borland C++ source code in `local-only/borland-tvision/`

See `local-only/ARCHITECTURAL-FINDINGS.md` for detailed analysis of how Borland's C++ architecture maps to Rust.

## Project Statistics

```
===============================================================================
 Language            Files        Lines         Code     Comments       Blanks
===============================================================================
 Rust                   90        21640        16408         2058         3174
 |- Markdown            60         1510            5         1305          200
 (Total)                          23150        16413         3363         3374
===============================================================================
```

Generated with [tokei](https://github.com/XAMPPRocky/tokei) - includes inline documentation

**178 tests** - all passing ✅

## License

MIT License - see [LICENSE](LICENSE) file for details.
