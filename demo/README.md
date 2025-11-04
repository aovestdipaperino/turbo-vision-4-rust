# Turbo Vision Demo Applications

This directory contains demonstration applications showcasing the capabilities of the Turbo Vision for Rust library.

## Rust Text Editor (`rust_editor.rs`)

A comprehensive text editor application with advanced features specifically designed for Rust development.

### Features

#### 1. **Rust Syntax Highlighting**
- Full syntax highlighting for Rust code
- Keywords, types, strings, comments, and more
- Real-time highlighting as you type

#### 2. **File Operations**
- **New**: Create a new file (`File → New` or `Ctrl+N`)
- **Open**: Open existing files (`File → Open` or `Ctrl+O`)
- **Save**: Save current file (`File → Save` or `Ctrl+S`)
- **Save As**: Save with a new name (`File → Save As`)

#### 3. **Dirty Flag Tracking**
- Editor tracks modifications to the document
- Prompts to save changes before closing or opening new files
- Visual indicator (*) in title bar when file has unsaved changes

#### 4. **Search and Replace**
- **Search**: Find text in the current document (`Edit → Search` or `Ctrl+F`)
- **Replace**: Find and replace text (`Edit → Replace` or `Ctrl+H`)
- **Go to Line**: Jump to a specific line number (`Edit → Goto Line` or `Ctrl+G`)

#### 5. **Rust Analyzer Integration** (Coming Soon)
- **Analyze**: Run rust-analyzer on the current file (`Tools → Analyze` or `F5`)
- **Show Errors**: Display analysis results and errors (`Tools → Show Errors` or `F6`)
- *Note: This feature currently shows a placeholder. Full integration is planned.*

#### 6. **Professional UI**
- Menu bar with File, Edit, and Tools menus
- Status line showing keyboard shortcuts
- Scrollbars for navigation
- Line/column indicator
- Resizable window

### How to Run

```bash
# From the project root
cargo run --bin rust_editor

# Or build and run directly
cargo build --bin rust_editor
./target/debug/rust_editor
```

### Keyboard Shortcuts

#### File Operations
- `Ctrl+N` - New file
- `Ctrl+O` - Open file
- `Ctrl+S` - Save file
- `Ctrl+W` - Close current window
- `Alt+X` - Exit editor

#### Editing
- `Ctrl+F` - Search
- `Ctrl+H` - Replace
- `Ctrl+G` - Go to line
- Arrow keys - Navigate
- `Shift+Arrows` - Text selection
- `Ctrl+C/V/X` - Copy/Paste/Cut (clipboard operations)

#### Editor Navigation
- `Home` - Start of line
- `End` - End of line
- `Page Up/Down` - Scroll page
- `Ctrl+Home` - Start of document
- `Ctrl+End` - End of document

#### Tools
- `F5` - Analyze with rust-analyzer
- `F6` - Show errors
- `F10` - Access menu bar

### Menu Structure

#### File Menu
- New
- Open...
- ─────
- Save
- Save As...
- ─────
- Close
- ─────
- Exit

#### Edit Menu
- Search...
- Replace...
- ─────
- Goto Line...

#### Tools Menu
- Analyze with rust-analyzer
- Show Errors

### Status Line

The status line at the bottom displays:
- `F10 Menu` - Press F10 to access the menu
- `Ctrl+S Save` - Quick save reminder
- `Ctrl+F Find` - Quick find reminder

### Implementation Details

#### Architecture
- Uses Turbo Vision `Editor` component with scrollbars
- `EditorState` struct tracks filename, dirty flag, and content
- Rust `RustHighlighter` provides syntax highlighting
- `FileDialog` for professional file browsing (Open/Save operations)
- Modal dialogs for search/replace operations
- Menu bar with cascading submenus

#### File Handling
- Files are loaded into memory as String
- `FileDialog` provides directory navigation and file filtering (*.rs)
- Dirty flag is set when content changes
- Save prompts appear before destructive operations
- Supports absolute and relative file paths
- Double-click files to open, double-click folders to navigate

#### Future Enhancements
1. **Full Rust Analyzer Integration**
   - Real LSP (Language Server Protocol) communication
   - Live error/warning display
   - Code completion
   - Go to definition
   - Find references

2. **Advanced Editor Features**
   - Multiple file tabs
   - Split view
   - Bookmarks
   - Undo/Redo history
   - Line numbers toggle

3. **Project Support**
   - Cargo project detection
   - Build/run from editor
   - Test runner integration
   - Dependency browser

4. **Configuration**
   - Custom color schemes
   - Font size adjustment
   - Key binding customization
   - Editor preferences

### Code Examples

The editor is particularly useful for editing Rust files. Try opening and editing these example files:
- `src/lib.rs` - Main library file
- `examples/*.rs` - Example programs
- `demo/rust_editor.rs` - The editor itself!

### Technical Notes

- Built with Turbo Vision for Rust v0.2.9+
- Uses crossterm for terminal handling
- Syntax highlighting via built-in `RustHighlighter`
- File I/O via standard Rust `std::fs` module
- Clipboard integration via arboard (on supported platforms)

### Troubleshooting

**Editor doesn't show syntax highlighting:**
- Ensure the file is saved with a `.rs` extension
- Syntax highlighting is automatic for Rust files

**File operations fail:**
- Check file permissions
- Verify the file path is correct
- Ensure sufficient disk space for save operations

**Keyboard shortcuts don't work:**
- Some terminals may intercept certain key combinations
- Try accessing features via the menu bar (F10)

### Contributing

This is a demonstration application showing the capabilities of Turbo Vision for Rust. For contributing to the main library, see the project root README.

### License

Same as the main Turbo Vision for Rust project - MIT License.
