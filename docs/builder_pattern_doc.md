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
| `Dialog` | `src/views/dialog.rs` | `DialogBuilder` |
| `ScrollBar` | `src/views/scrollbar.rs` | `ScrollBarBuilder` |
| `StaticText` | `src/views/static_text.rs` | `StaticTextBuilder` |
| `MenuItem` | `src/core/menu_data.rs` | `MenuItemBuilder` |
| `PictureValidator` | `src/views/picture_validator.rs` | `PictureValidatorBuilder` |
| `LookupValidator` | `src/views/lookup_validator.rs` | `LookupValidatorBuilder` |
| `InputLine` | `src/views/input_line.rs` | `InputLineBuilder` |
| `RadioButton` | `src/views/radiobutton.rs` | `RadioButtonBuilder` |
| `CheckBox` | `src/views/checkbox.rs` | `CheckBoxBuilder` |
| `Scroller` | `src/views/scroller.rs` | `ScrollerBuilder` |
| `Frame` | `src/views/frame.rs` | `FrameBuilder` |
| `FileDialog` | `src/views/file_dialog.rs` | `FileDialogBuilder` |
| `ColorDialog` | `src/views/color_dialog.rs` | `ColorDialogBuilder` |
| `ChdirDialog` | `src/views/chdir_dialog.rs` | `ChDirDialogBuilder` |
| `StatusItem` | `src/core/status_data.rs` | `StatusItemBuilder` |
| `StatusDef` | `src/core/status_data.rs` | `StatusDefBuilder` |
| `Label` | `src/views/label.rs` | `LabelBuilder` |
| `Background` | `src/views/background.rs` | `BackgroundBuilder` |
| `FilterValidator` | `src/views/validator.rs` | `FilterValidatorBuilder` |
| `RangeValidator` | `src/views/validator.rs` | `RangeValidatorBuilder` |
| `ListBox` | `src/views/listbox.rs` | `ListBoxBuilder` |
| `SortedListBox` | `src/views/sorted_listbox.rs` | `SortedListBoxBuilder` |
| `Group` | `src/views/group.rs` | `GroupBuilder` |
| `Desktop` | `src/views/desktop.rs` | `DesktopBuilder` |
| `ColorSelector` | `src/views/color_selector.rs` | `ColorSelectorBuilder` |
| `Indicator` | `src/views/indicator.rs` | `IndicatorBuilder` |
| `HelpViewer` | `src/views/help_viewer.rs` | `HelpViewerBuilder` |
| `TextViewer` | `src/views/text_viewer.rs` | `TextViewerBuilder` |
| `FileEditor` | `src/views/file_editor.rs` | `FileEditorBuilder` |
| `Memo` | `src/views/memo.rs` | `MemoBuilder` |
| `TerminalWidget` | `src/views/terminal_widget.rs` | `TerminalWidgetBuilder` |
| `FileList` | `src/views/file_list.rs` | `FileListBuilder` |
| `DirListBox` | `src/views/dir_listbox.rs` | `DirListBoxBuilder` |
| `EditWindow` | `src/views/edit_window.rs` | `EditWindowBuilder` |
| `ParamText` | `src/views/paramtext.rs` | `ParamTextBuilder` |
| `HelpTOC` | `src/views/help_toc.rs` | `HelpTocBuilder` |
| `HelpWindow` | `src/views/help_window.rs` | `HelpWindowBuilder` |
| `HelpIndex` | `src/views/help_index.rs` | `HelpIndexBuilder` |
| `HelpFile` | `src/views/help_file.rs` | `HelpFileBuilder` |
| `History` | `src/views/history.rs` | `HistoryBuilder` |
| `HistoryViewer` | `src/views/history_viewer.rs` | `HistoryViewerBuilder` |
| `HistoryWindow` | `src/views/history_window.rs` | `HistoryWindowBuilder` |

---

## 🔴 High Priority - Multiple Constructors

~~These types have multiple `new_*()` variants, making them excellent candidates for builder pattern:~~

**All high-priority types now have builders implemented!**

### ✅ Dialog
**Location:** `src/views/dialog.rs`
- ~~`pub fn new(bounds: Rect, title: &str) -> Self`~~
- ~~`pub fn new_modal(bounds: Rect, title: &str) -> Box<Self>`~~

**Status:** ✅ DialogBuilder implemented with `.modal()` method

### ✅ ScrollBar
**Location:** `src/views/scrollbar.rs`
- ~~`pub fn new_vertical(bounds: Rect) -> Self`~~
- ~~`pub fn new_horizontal(bounds: Rect) -> Self`~~

**Status:** ✅ ScrollBarBuilder implemented with `.vertical()` and `.horizontal()` methods

### ✅ StaticText
**Location:** `src/views/static_text.rs`
- ~~`pub fn new(bounds: Rect, text: &str) -> Self`~~
- ~~`pub fn new_centered(bounds: Rect, text: &str) -> Self`~~

**Status:** ✅ StaticTextBuilder implemented with `.centered()` method

### ✅ MenuItem
**Location:** `src/core/menu_data.rs`
- ~~`pub fn new(text: &str, command: CommandId, key_code: KeyCode, help_ctx: u16) -> Self`~~
- ~~`pub fn new_disabled(text: &str, command: CommandId, key_code: KeyCode, help_ctx: u16) -> Self`~~
- ~~`pub fn with_shortcut(text: &str, command: CommandId, key_code: KeyCode, shortcut: &str, help_ctx: u16) -> Self`~~

**Status:** ✅ MenuItemBuilder implemented with `.enabled()` and `.shortcut()` methods

### ✅ PictureValidator
**Location:** `src/views/picture_validator.rs`
- ~~`pub fn new(mask: &str) -> Self`~~
- ~~`pub fn new_no_format(mask: &str) -> Self`~~

**Status:** ✅ PictureValidatorBuilder implemented with `.auto_format()` method

### ✅ LookupValidator
**Location:** `src/views/lookup_validator.rs`
- ~~`pub fn new(valid_values: Vec<String>) -> Self`~~
- ~~`pub fn new_case_insensitive(valid_values: Vec<String>) -> Self`~~

**Status:** ✅ LookupValidatorBuilder implemented with `.case_sensitive()` method

---

## 🟡 Medium Priority - Complex Constructors

These types have complex constructors with 3+ parameters:

### ✅ InputLine (COMPLETED)
**Location:** `src/views/input_line.rs`
- ~~`pub fn new(bounds: Rect, max_length: usize, data: Rc<RefCell<String>>) -> Self`~~
- ~~`pub fn with_validator(...)` - builder-style method~~

**Status:** ✅ InputLineBuilder implemented with `.max_length()` and `.validator()` methods

### ✅ RadioButton (COMPLETED)
**Location:** `src/views/radiobutton.rs`
- ~~`pub fn new(bounds: Rect, label: &str, group_id: u16) -> Self`~~

**Status:** ✅ RadioButtonBuilder implemented with `.group_id()` and `.selected()` methods

### ✅ CheckBox (COMPLETED)
**Location:** `src/views/checkbox.rs`
- ~~`pub fn new(bounds: Rect, label: &str) -> Self`~~

**Status:** ✅ CheckBoxBuilder implemented with `.checked()` method

### ✅ Scroller (COMPLETED)
**Location:** `src/views/scroller.rs`
- ~~`pub fn new(bounds: Rect, h_scrollbar: Option<Box<ScrollBar>>, v_scrollbar: Option<Box<ScrollBar>>) -> Self`~~

**Status:** ✅ ScrollerBuilder implemented with `.h_scrollbar()` and `.v_scrollbar()` methods

### ✅ Frame (COMPLETED)
**Location:** `src/views/frame.rs`
- ~~`pub fn new(bounds: Rect, title: &str, resizable: bool) -> Self`~~
- ~~`pub fn with_palette(bounds: Rect, title: &str, palette_type: FramePaletteType, resizable: bool) -> Self`~~

**Status:** ✅ FrameBuilder implemented with `.palette_type()` and `.resizable()` methods

---

### Remaining Medium Priority Types

### Button
**Location:** `src/views/button.rs`
- `pub fn new(bounds: Rect, title: &str, command: CommandId, is_default: bool) -> Self`

**Note:** Already has builder, but constructor still exists for compatibility.

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

**Status:** Categories 1-4 completed! ✅ Remaining types marked as superfluous.

### ✅ Category 1: Views with Single Bounds Parameter (COMPLETED)
- ✅ `Group::new(bounds: Rect)` - `src/views/group.rs` - **GroupBuilder**
- ✅ `Desktop::new(bounds: Rect)` - `src/views/desktop.rs` - **DesktopBuilder**
- ✅ `ColorSelector::new(bounds: Rect)` - `src/views/color_selector.rs` - **ColorSelectorBuilder**
- ✅ `Indicator::new(bounds: Rect)` - `src/views/indicator.rs` - **IndicatorBuilder**
- ✅ `HelpViewer::new(bounds: Rect)` - `src/views/help_viewer.rs` - **HelpViewerBuilder**
- ✅ `TextViewer::new(bounds: Rect)` - `src/views/text_viewer.rs` - **TextViewerBuilder**
- ✅ `FileEditor::new(bounds: Rect)` - `src/views/file_editor.rs` - **FileEditorBuilder**
- ✅ `Memo::new(bounds: Rect)` - `src/views/memo.rs` - **MemoBuilder**
- ✅ `TerminalWidget::new(bounds: Rect)` - `src/views/terminal_widget.rs` - **TerminalWidgetBuilder**

**Note:** These builders provide consistent API and enable optional features like scrollbars and indicators.

### ✅ Category 2: Views with Bounds + Parameter (COMPLETED)
- ✅ `FileList::new(bounds: Rect, path: &Path)` - `src/views/file_list.rs` - **FileListBuilder**
- ✅ `DirListBox::new(bounds: Rect, path: &Path)` - `src/views/dir_listbox.rs` - **DirListBoxBuilder**
- ✅ `EditWindow::new(bounds: Rect, title: &str)` - `src/views/edit_window.rs` - **EditWindowBuilder**
- ✅ `ParamText::new(bounds: Rect, template: &str)` - `src/views/paramtext.rs` - **ParamTextBuilder**
- ✅ `ListBox::new(bounds: Rect, on_select_command: CommandId)` - `src/views/listbox.rs` - **ListBoxBuilder**
- ✅ `SortedListBox::new(bounds: Rect, on_select_command: CommandId)` - `src/views/sorted_listbox.rs` - **SortedListBoxBuilder**

### ✅ Category 3: Help System (COMPLETED)
- ✅ `HelpTOC::new(bounds: Rect, title: &str, help_file: Rc<RefCell<HelpFile>>)` - `src/views/help_toc.rs` - **HelpTocBuilder**
- ✅ `HelpWindow::new(bounds: Rect, title: &str, help_file: Rc<RefCell<HelpFile>>)` - `src/views/help_window.rs` - **HelpWindowBuilder**
- ✅ `HelpIndex::new(bounds: Rect, title: &str, help_file: Rc<RefCell<HelpFile>>)` - `src/views/help_index.rs` - **HelpIndexBuilder**
- ✅ `HelpFile::new(path: &str)` - `src/views/help_file.rs` - **HelpFileBuilder**

### ✅ Category 4: History System (COMPLETED)
- ✅ `History::new(pos: Point, history_id: u16)` - `src/views/history.rs` - **HistoryBuilder**
- ✅ `HistoryViewer::new(bounds: Rect, history_id: u16)` - `src/views/history_viewer.rs` - **HistoryViewerBuilder**
- ✅ `HistoryWindow::new(pos: Point, history_id: u16, width: i16)` - `src/views/history_window.rs` - **HistoryWindowBuilder**

### 🔵 Category 5: Simple Data Structures (SUPERFLUOUS - No Builder Needed)
These types have trivial constructors and are rarely configured. Builders provide no meaningful benefit:
- `Outline::new(data: T)` - `src/views/outline.rs` - Generic container
- `Node::new(data: T)` - `src/views/outline.rs` - Generic tree node
- `Editor::new()` - `src/views/editor.rs` - No parameters
- `Cluster::new()` - `src/views/cluster.rs` - No parameters
- `ListViewer::new()` - `src/views/list_viewer.rs` - No parameters
- `MenuViewer::new()` - `src/views/menu_viewer.rs` - No parameters
- `HelpContext::new()` - `src/views/help_context.rs` - No parameters
- `CommandSet::new()` - `src/core/command_set.rs` - No parameters
- `EventQueue::new()` - `src/core/event.rs` - No parameters
- `PaletteMap::new()` - `src/core/palette.rs` - No parameters

**Rationale:** No configuration options, parameterless or single generic parameter.

### 🔵 Category 6: Core Infrastructure (SUPERFLUOUS - No Builder Needed)
These are low-level infrastructure types with specific instantiation patterns:
- `Application::new() -> Result<Self>` - `src/app/application.rs` - Singleton with error handling
- `DrawBuffer::new(width: usize)` - `src/core/draw.rs` - Single parameter, internal use
- `MenuBox::new(position: Point, menu: Menu)` - `src/views/menu_box.rs` - Internal to menu system
- `SubMenu::new(name: &str, menu: Menu)` - `src/views/menu_bar.rs` - Internal to menu system
- `MenuBar::new(bounds: Rect)` - `src/views/menu_bar.rs` - Simple bounds constructor

**Rationale:** Infrastructure types with fixed initialization patterns, rarely customized by users.

---

## Summary Statistics

- **Total types with constructors:** ~70
- **Types with builders:** 50 (71%) ✅
  - **Core builders (4):** Window, Button, StatusLine, Menu
  - **High-priority (6):** Dialog, ScrollBar, StaticText, MenuItem, PictureValidator, LookupValidator
  - **Medium-priority forms (5):** InputLine, RadioButton, CheckBox, Scroller, Frame
  - **Dialog builders (3):** FileDialog, ColorDialog, ChDirDialog
  - **Status builders (2):** StatusItem, StatusDef
  - **Validator builders (2):** FilterValidator, RangeValidator
  - **List builders (2):** ListBox, SortedListBox
  - **Category 1 - Single bounds (9):** Group, Desktop, ColorSelector, Indicator, HelpViewer, TextViewer, FileEditor, Memo, TerminalWidget
  - **Category 2 - Bounds + param (4):** FileList, DirListBox, EditWindow, ParamText
  - **Category 3 - Help system (4):** HelpTOC, HelpWindow, HelpIndex, HelpFile
  - **Category 4 - History system (3):** History, HistoryViewer, HistoryWindow
  - **Additional builders (2):** Label, Background
- **Superfluous types (no builder needed):** ~20 (29%)
  - Category 5: Simple data structures (10 types) - Parameterless or trivial constructors
  - Category 6: Core infrastructure (5 types) - Fixed initialization patterns
- **High priority candidates:** 0 (All completed! ✅)
- **Medium priority candidates:** 0 (All completed! ✅)
- **Low priority categories 1-4:** 0 (All completed! ✅)

## Recommendations

1. ~~**Immediate action:** Implement builders for high-priority types~~ ✅ **Completed!**
2. ~~**Next phase:** Add builders to complex constructors~~ ✅ **Completed!**
3. ~~**Dialog builders:** Add builders to dialogs~~ ✅ **Completed!**
4. ~~**Validators:** Completed FilterValidator and RangeValidator builders~~ ✅ **Completed!**
5. ~~**Common views:** Completed Label and Background builders~~ ✅ **Completed!**
6. ~~**Categories 1-4:** Completed all low-priority builders for common views, help, and history systems~~ ✅ **Completed!**
7. **Categories 5-6:** Marked as superfluous - these types have trivial constructors or are infrastructure types that don't benefit from the builder pattern

**Result:** Builder pattern implementation is now comprehensive across all user-facing views and components! 🎉

## Implementation Patterns

For the remaining simple types, builders follow these patterns:

### Single-Bounds Pattern
```rust
pub struct TypeBuilder {
    bounds: Option<Rect>,
}
```

### Bounds + Parameter Pattern
```rust
pub struct TypeBuilder {
    bounds: Option<Rect>,
    param: ParamType,
}
```

These patterns can be applied to remaining types as needed based on user demand.

---

*Generated: 2025*
*Based on analysis of src/ directory*
