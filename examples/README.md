# Turbo Vision Examples

This directory contains 16 examples demonstrating various features of the Turbo Vision framework.

## Core Examples (Essential)

### Application & Layout
- **`menu.rs`** - Complete menu bar with dropdowns and keyboard navigation
- **`dialogs.rs`** - Modal dialogs with buttons and controls

### Editor
- **`editor.rs`** ⭐ **NEW v0.2.6** - Comprehensive editor demonstrations:
  - Basic editing (undo/redo/clipboard)
  - Search and replace
  - Syntax highlighting (Rust)
  - File I/O operations (load/save)

### Validation
- **`validator.rs`** ⭐ **NEW v0.2.6** - All validator types:
  - FilterValidator (character filtering)
  - RangeValidator (numeric ranges)
  - PictureValidator (format masks: phone, dates, product codes)

### Lists & History
- **`sorted_listbox.rs`** - Sorted list with binary search and type-ahead
- **`list_components.rs`** - ListViewer demonstrations
<!-- - **`history.rs`** - Input history dropdowns -->

### File System
- **`file_browser.rs`** - File and directory tree navigation
- **`file_dialog.rs`** - File open/save dialogs

### Windows
- **`window_resize.rs`** - Window dragging and resizing
- **`test_window_modal_overlap.rs`** - Modal window blocking and Z-order

### Help System
- **`help_system.rs`** - Markdown-based context-sensitive help

### Status & Menu
- **`menu_status.rs`** - Status line with hot spots and hints
<!-- - **`menu_status_data.rs`** - Menu and status line data structures -->

## Advanced Examples

- **`broadcast.rs`** - Owner-aware event broadcasting
- **`command_set.rs`** - Command routing patterns

## Examples by Feature (v0.2.6)

| Feature | Example | Lines | Description |
|---------|---------|-------|-------------|
| **All Editor Features** | editor.rs | 290 | Editing, undo/redo, search, syntax highlighting, file I/O |
| **All Validators** | validator.rs | 320 | Filter, Range, Picture validators |
| **File Browser** | file_browser.rs | 120 | Directory tree + file list |
| **Help System** | help_system.rs | 120 | Markdown help with topics |
<!-- | **History** | history.rs | 95 | Input field history dropdowns | -->
| **Sorted Lists** | sorted_listbox.rs | 85 | Binary search sorted lists |
| **Menus** | menu_status.rs | 150 | Menu bar with dropdowns + status line with hot spots |
<!-- | **Status Line** | status_line_demo.rs | 100 | Status bar with hot spots | -->
| **Window Management** | window_resize.rs | 95 | Drag/resize windows |
| **Modal Dialogs** | dialogs.rs | 70 | Basic modal dialog |

## Running Examples

```bash
# Run any example
cargo run --example editor
cargo run --example validator
cargo run --example file_browser

# Build all examples
cargo build --examples

# List all examples
cargo run --example
```

## Quick Start

Best examples to start with:
1. **menu.rs** - See the full application structure
2. **dialogs.rs** - Learn basic dialogs
3. **editor.rs** - See advanced editor features (syntax highlighting, search, file I/O)
4. **validator.rs** - Learn all input validation patterns

## Notes

- **Consolidated v0.2.6 examples** (editor_demo.rs, validator_demo.rs) combine multiple features into menu-driven demonstrations
- All examples use `Application::new()` for setup
- Most examples show modal dialogs with `Dialog::execute()`
- Editor examples demonstrate the most complex widget
- File system examples show cross-platform file operations
- Validator examples show all input validation patterns (Filter, Range, Picture)
