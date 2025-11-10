# Builder Pattern Candidates

This document lists all `new()` and `new_*()` methods in the Turbo Vision library, organized by builder pattern implementation status and priority.

## ✅ Already Has Builders

These types already implement the builder pattern:

| Type | Location | Builder |
|------|----------|---------|
| `Window` | `src/views/window.rs` | `WindowBuilder` |
| `Button` | `src/views/button.rs` | `ButtonBuilder` |
| `StatusLine` | `src/core/status_data.rs` | `StatusLineBuilder` |
| `Menu` | `src/core/menu_data.rs` | `MenuBuilder` |

---

## 🔴 High Priority - Multiple Constructors

These types have multiple `new_*()` variants, making them excellent candidates for builder pattern:

### Dialog
**Location:** `src/views/dialog.rs`
- `pub fn new(bounds: Rect, title: &str) -> Self`
- `pub fn new_modal(bounds: Rect, title: &str) -> Box<Self>`

**Rationale:** Two constructors with different return types. Builder could handle both cases.

### ScrollBar
**Location:** `src/views/scrollbar.rs`
- `pub fn new_vertical(bounds: Rect) -> Self`
- `pub fn new_horizontal(bounds: Rect) -> Self`

**Rationale:** Orientation is a key configuration option. Builder pattern would make this clearer.

### StaticText
**Location:** `src/views/static_text.rs`
- `pub fn new(bounds: Rect, text: &str) -> Self`
- `pub fn new_centered(bounds: Rect, text: &str) -> Self`

**Rationale:** Alignment could be a builder option.

### MenuItem
**Location:** `src/core/menu_data.rs`
- `pub fn new(text: &str, command: CommandId, key_code: KeyCode, help_ctx: u16) -> Self`
- `pub fn new_disabled(text: &str, command: CommandId, key_code: KeyCode, help_ctx: u16) -> Self`
- `pub fn with_shortcut(text: &str, command: CommandId, key_code: KeyCode, shortcut: &str, help_ctx: u16) -> Self`

**Rationale:** Multiple options (disabled, shortcut) suggest builder pattern.

### PictureValidator
**Location:** `src/views/picture_validator.rs`
- `pub fn new(mask: &str) -> Self`
- `pub fn new_no_format(mask: &str) -> Self`

**Rationale:** Format option could be builder configuration.

### LookupValidator
**Location:** `src/views/lookup_validator.rs`
- `pub fn new(valid_values: Vec<String>) -> Self`
- `pub fn new_case_insensitive(valid_values: Vec<String>) -> Self`

**Rationale:** Case sensitivity could be builder option.

---

## 🟡 Medium Priority - Complex Constructors

These types have complex constructors with 3+ parameters:

### Button
**Location:** `src/views/button.rs`
- `pub fn new(bounds: Rect, title: &str, command: CommandId, is_default: bool) -> Self`

**Note:** Already has builder, but constructor still exists for compatibility.

### InputLine
**Location:** `src/views/input_line.rs`
- `pub fn new(bounds: Rect, max_length: usize, data: Rc<RefCell<String>>) -> Self`
- `pub fn with_validator(...)` - builder-style method

**Rationale:** Has complex state (data, validator, max_length). Good builder candidate.

### RadioButton
**Location:** `src/views/radiobutton.rs`
- `pub fn new(bounds: Rect, label: &str, group_id: u16) -> Self`

**Rationale:** Three distinct configuration parameters.

### CheckBox
**Location:** `src/views/checkbox.rs`
- `pub fn new(bounds: Rect, label: &str) -> Self`

**Rationale:** Simple now, but likely to gain options (checked state, etc.).

### FileDialog
**Location:** `src/views/file_dialog.rs`
- `pub fn new(bounds: Rect, title: &str, wildcard: &str, initial_dir: Option<PathBuf>) -> Self`

**Rationale:** Complex dialog with many potential configuration options.

### ColorDialog
**Location:** `src/views/color_dialog.rs`
- `pub fn new(bounds: Rect, title: &str, initial_attr: Attr) -> Self`

**Rationale:** Dialog that could benefit from more configuration options.

### ChdirDialog
**Location:** `src/views/chdir_dialog.rs`
- `pub fn new(bounds: Rect, title: &str, initial_dir: Option<PathBuf>) -> Self`

**Rationale:** Dialog with optional initial state.

### Scroller
**Location:** `src/views/scroller.rs`
- `pub fn new(bounds: Rect, h_scrollbar: Option<Box<ScrollBar>>, v_scrollbar: Option<Box<ScrollBar>>) -> Self`

**Rationale:** Complex optional components suggest builder pattern.

### Frame
**Location:** `src/views/frame.rs`
- `pub fn new(bounds: Rect, title: &str, resizable: bool) -> Self`
- `pub fn with_palette(bounds: Rect, title: &str, palette_type: FramePaletteType, resizable: bool) -> Self`

**Rationale:** Multiple configuration options (palette, resizable).

### StatusItem
**Location:** `src/core/status_data.rs`
- `pub fn new(text: &str, key_code: KeyCode, command: CommandId) -> Self`

**Rationale:** Part of StatusLine system which already has builder.

### StatusDef
**Location:** `src/core/status_data.rs`
- `pub fn new(min: u16, max: u16, items: Vec<StatusItem>) -> Self`

**Rationale:** Part of StatusLine system which already has builder.

---

## 🟢 Low Priority - Simple Constructors

These types have simple constructors and may not benefit from builders:

### Views with Single Bounds Parameter
- `Group::new(bounds: Rect)` - `src/views/group.rs`
- `Desktop::new(bounds: Rect)` - `src/views/desktop.rs`
- `ColorSelector::new(bounds: Rect)` - `src/views/color_selector.rs`
- `Indicator::new(bounds: Rect)` - `src/views/indicator.rs`
- `HelpViewer::new(bounds: Rect)` - `src/views/help_viewer.rs`
- `TextViewer::new(bounds: Rect)` - `src/views/text_viewer.rs`
- `FileEditor::new(bounds: Rect)` - `src/views/file_editor.rs`
- `Memo::new(bounds: Rect)` - `src/views/memo.rs`
- `TerminalView::new(bounds: Rect)` - `src/views/terminal_widget.rs`

**Note:** Some of these have `with_*` methods for optional features (scrollbars, indicators), which suggests partial builder pattern adoption.

### Views with Bounds + Path
- `FileList::new(bounds: Rect, path: &Path)` - `src/views/file_list.rs`
- `DirListBox::new(bounds: Rect, path: &Path)` - `src/views/dir_listbox.rs`

### Views with Bounds + Title
- `EditWindow::new(bounds: Rect, title: &str)` - `src/views/edit_window.rs`
- `Label::new(bounds: Rect, text: &str)` - `src/views/label.rs`
- `ParamText::new(bounds: Rect, template: &str)` - `src/views/paramtext.rs`

### Views with Bounds + Command
- `ListBox::new(bounds: Rect, on_select_command: CommandId)` - `src/views/listbox.rs`
- `SortedListBox::new(bounds: Rect, on_select_command: CommandId)` - `src/views/sorted_listbox.rs`

### Help System
- `HelpTOC::new(bounds: Rect, title: &str, help_file: Rc<RefCell<HelpFile>>)` - `src/views/help_toc.rs`
- `HelpWindow::new(bounds: Rect, title: &str, help_file: Rc<RefCell<HelpFile>>)` - `src/views/help_window.rs`
- `HelpIndex::new(bounds: Rect, title: &str, help_file: Rc<RefCell<HelpFile>>)` - `src/views/help_index.rs`
- `HelpFile::new(id: String, title: String)` - `src/views/help_file.rs`

### History System
- `History::new(pos: Point, history_id: u16)` - `src/views/history.rs`
- `HistoryViewer::new(bounds: Rect, history_id: u16)` - `src/views/history_viewer.rs`
- `HistoryWindow::new(pos: Point, history_id: u16, width: i16)` - `src/views/history_window.rs`

### Simple Data Structures
- `Outline::new(data: T)` - `src/views/outline.rs`
- `Node::new(data: T)` - `src/views/outline.rs`
- `Editor::new()` - `src/views/editor.rs`
- `Cluster::new()` - `src/views/cluster.rs`
- `ListViewer::new()` - `src/views/list_viewer.rs`
- `MenuViewer::new()` - `src/views/menu_viewer.rs`
- `HelpContext::new()` - `src/views/help_context.rs`
- `CommandSet::new()` - `src/core/command_set.rs`
- `EventQueue::new()` - `src/core/event.rs`
- `PaletteMap::new()` - `src/core/palette.rs`

### Validators
- `FilterValidator::new(valid_chars: &str)` - `src/views/validator.rs`
- `RangeValidator::new(min: i64, max: i64)` - `src/views/validator.rs`

### Core Types
- `Application::new() -> Result<Self>` - `src/app/application.rs`
- `DrawBuffer::new(width: usize)` - `src/core/draw.rs`
- `Background::new(bounds: Rect, pattern: char, attr: Attr)` - `src/views/background.rs`
- `MenuBox::new(position: Point, menu: Menu)` - `src/views/menu_box.rs`
- `SubMenu::new(name: &str, menu: Menu)` - `src/views/menu_bar.rs`
- `MenuBar::new(bounds: Rect)` - `src/views/menu_bar.rs`

---

## Summary Statistics

- **Total types with constructors:** ~70
- **Already have builders:** 4 (6%)
- **High priority candidates:** 7 (10%)
- **Medium priority candidates:** 14 (20%)
- **Low priority:** ~45 (64%)

## Recommendations

1. **Immediate action:** Implement builders for high-priority types (Dialog, ScrollBar, MenuItem, StaticText)
2. **Next phase:** Add builders to complex dialogs (FileDialog, ColorDialog, InputLine)
3. **Future consideration:** Evaluate `with_*` pattern vs full builders for simple types with optional features

---

*Generated: 2025*
*Based on analysis of src/ directory*
