# Turbo Vision Demo Applications

This directory contains demonstration applications showcasing the capabilities of the Turbo Vision for Rust library.

## Pascal IDE Editor (`pascal_ide.rs`)

A text editor application based on the [Bruto-Pascal](https://github.com/aovestdipaperino/bruto-pascal) IDE, adapted as a standalone demo for Turbo Vision.

### Features

#### 1. **Pascal Syntax Highlighting**
- Full syntax highlighting for Pascal code
- Keywords, types, strings, comments (line, brace, paren-star), and numbers
- Real-time highlighting as you type

#### 2. **File Operations**
- **New**: Create a new file with sample Pascal program (`File > New`)
- **Open**: Open existing files (`File > Open` or `F3`)
- **Save**: Save current file (`File > Save` or `F2`)
- **Save As**: Save with a new name (`File > Save As`)

#### 3. **Dirty Flag Tracking**
- Editor tracks modifications to the document
- Prompts to save changes before closing or exiting
- Visual indicator (*) in title bar when file has unsaved changes

#### 4. **Search and Replace**
- **Find**: Find text in the current document (`Search > Find`)
- **Replace**: Find and replace text (`Search > Replace`)
- **Search Again**: Repeat last search (`Search > Search Again`)
- **Go to Line**: Jump to a specific line number (`Edit > Goto Line`)

#### 5. **Professional UI**
- Menu bar with File, Edit, Search, and Windows menus
- Status line showing keyboard shortcuts
- Scrollbars for navigation
- Line/column indicator
- Resizable and movable windows

### How to Run

```bash
# From the project root
cargo run --bin pascal_ide

# Or build and run directly
cargo build --bin pascal_ide
./target/debug/pascal_ide
```

### Keyboard Shortcuts

#### File Operations
- `F2` - Save file
- `F3` - Open file
- `Alt+F3` - Close current window
- `Alt+X` - Exit editor

#### Editor Navigation
- Arrow keys - Navigate
- `Home` / `End` - Start/end of line
- `Page Up` / `Page Down` - Scroll page
- `Ctrl+Home` / `Ctrl+End` - Start/end of document
- `Shift+Arrows` - Text selection
- `Ctrl+C` / `Ctrl+V` / `Ctrl+X` - Copy/Paste/Cut
- `Ctrl+G` - Go to line

#### Window Management
- `F5` - Zoom window
- `F6` - Next window
- `Shift+F6` - Previous window
- `F10` - Access menu bar

### Implementation Details

#### Architecture
- Based on the [bruto-ide](https://github.com/aovestdipaperino/bruto-ide) editor from the Bruto-Pascal project
- Uses Turbo Vision `FileEditor` component for file management
- Pascal `PascalHighlighter` provides syntax highlighting (from [bruto-pascal-lang](https://github.com/aovestdipaperino/bruto-pascal-lang))
- `FileDialog` for file browsing (Open/Save operations)
- Menu bar with cascading submenus matching Borland's TEditorApp layout

### License

Same as the main Turbo Vision for Rust project - MIT License.
