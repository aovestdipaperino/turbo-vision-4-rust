# Turbo Vision Rust API Catalog

Comprehensive catalog of all public structs, traits, and their public methods in the Turbo Vision Rust codebase.

Generated: 2025-11-06

---

## Table of Contents

1. [Core Module](#core-module)
2. [Terminal Module](#terminal-module)
3. [Views Module](#views-module)
4. [Application Module](#application-module)

---

## CORE MODULE

### Geometry Primitives (`src/core/geometry.rs`)

#### Point Struct
**Public Methods:**
- `new(x: i16, y: i16) -> Self` - Create a point
- `zero() -> Self` - Create a point at origin

#### Rect Struct
**Public Methods:**
- `new(x1: i16, y1: i16, x2: i16, y2: i16) -> Self` - Create rectangle
- `from_points(a: Point, b: Point) -> Self` - Create from two points
- `from_coords(x: i16, y: i16, width: i16, height: i16) -> Self` - Create from coords and dimensions
- `move_by(&mut self, dx: i16, dy: i16)` - Move rectangle
- `grow(&mut self, dx: i16, dy: i16)` - Grow/shrink rectangle
- `contains(&self, p: Point) -> bool` - Check if point inside
- `is_empty(&self) -> bool` - Check if empty
- `width(&self) -> i16` - Get width
- `height(&self) -> i16` - Get height
- `size(&self) -> Point` - Get size as point
- `intersect(&self, other: &Rect) -> Rect` - Intersect with another rect
- `intersects(&self, other: &Rect) -> bool` - Check if overlapping
- `union(&self, other: &Rect) -> Rect` - Union with another rect

---

### Color Palette (`src/core/palette.rs`)

#### TvColor Enum
**Variants:** Black, Blue, Green, Cyan, Red, Magenta, Brown, LightGray, DarkGray, LightBlue, LightGreen, LightCyan, LightRed, LightMagenta, Yellow, White

**Public Methods:**
- `to_crossterm(self) -> Color` - Convert to crossterm color
- `from_u8(n: u8) -> Self` - Create from byte

#### Attr Struct
**Fields:**
- `pub fg: TvColor` - Foreground color
- `pub bg: TvColor` - Background color

**Public Methods:**
- `new(fg: TvColor, bg: TvColor) -> Self` - Create attribute
- `from_u8(byte: u8) -> Self` - Create from byte representation
- `to_u8(self) -> u8` - Convert to byte representation

#### Color Constants (colors module)
- NORMAL, HIGHLIGHTED, SELECTED, DISABLED
- MENU_NORMAL, MENU_SELECTED, MENU_DISABLED, MENU_SHORTCUT
- DIALOG_NORMAL, DIALOG_FRAME, DIALOG_FRAME_ACTIVE, DIALOG_TITLE, DIALOG_SHORTCUT
- BUTTON_NORMAL, BUTTON_DEFAULT, BUTTON_SELECTED, BUTTON_DISABLED, BUTTON_SHORTCUT, BUTTON_SHADOW
- STATUS_NORMAL, STATUS_SHORTCUT, STATUS_SELECTED, STATUS_SELECTED_SHORTCUT
- INPUT_NORMAL, INPUT_FOCUSED
- EDITOR_NORMAL, EDITOR_SELECTED
- LISTBOX_NORMAL, LISTBOX_FOCUSED, LISTBOX_SELECTED, LISTBOX_SELECTED_FOCUSED
- SCROLLBAR_PAGE, SCROLLBAR_INDICATOR, SCROLLBAR_ARROW
- SCROLLER_NORMAL, SCROLLER_SELECTED
- DESKTOP
- HELP_NORMAL, HELP_FOCUSED

---

### Drawing Primitives (`src/core/draw.rs`)

#### Cell Struct
**Fields:**
- `pub ch: char` - Character
- `pub attr: Attr` - Attributes

**Public Methods:**
- `new(ch: char, attr: Attr) -> Self` - Create cell

#### DrawBuffer Struct
**Fields:**
- `pub data: Vec<Cell>` - Buffer data

**Public Methods:**
- `new(width: usize) -> Self` - Create new buffer
- `move_char(&mut self, pos: usize, ch: char, attr: Attr, count: usize)` - Fill range with character
- `move_str(&mut self, pos: usize, s: &str, attr: Attr)` - Write string
- `move_buf(&mut self, pos: usize, src: &[Cell], count: usize)` - Copy cells
- `put_char(&mut self, pos: usize, ch: char, attr: Attr)` - Put single character
- `len(&self) -> usize` - Get length
- `is_empty(&self) -> bool` - Check if empty
- `move_str_with_shortcut(&mut self, pos: usize, s: &str, normal_attr: Attr, shortcut_attr: Attr) -> usize` - Write string with shortcut highlighting (format: "~X~" marks X for shortcut)

---

### Event System (`src/core/event.rs`)

#### KeyCode Type
**Definition:** `pub type KeyCode = u16` - Keyboard code (scan code + character)

#### Key Code Constants
- KB_ESC, KB_ENTER, KB_BACKSPACE, KB_TAB, KB_SHIFT_TAB
- KB_F1...KB_F12, KB_SHIFT_F12
- KB_UP, KB_DOWN, KB_LEFT, KB_RIGHT
- KB_HOME, KB_END, KB_PGUP, KB_PGDN, KB_INS, KB_DEL
- KB_ALT_X, KB_ALT_F, KB_ALT_H, KB_ALT_O, KB_ALT_A, KB_ALT_F3
- KB_ESC_F, KB_ESC_H, KB_ESC_X, KB_ESC_A, KB_ESC_O, KB_ESC_E, KB_ESC_S, KB_ESC_V, KB_ESC_ESC

#### EventType Enum
**Variants:** Nothing, Keyboard, MouseDown, MouseUp, MouseMove, MouseAuto, MouseWheelUp, MouseWheelDown, Command, Broadcast

#### Event Masks
- EV_NOTHING, EV_MOUSE_DOWN, EV_MOUSE_UP, EV_MOUSE_MOVE, EV_MOUSE_AUTO, EV_MOUSE_WHEEL_UP, EV_MOUSE_WHEEL_DOWN
- EV_MOUSE (all mouse events), EV_KEYBOARD, EV_COMMAND, EV_BROADCAST, EV_MESSAGE

#### MouseEvent Struct
**Fields:**
- `pub pos: Point` - Position
- `pub buttons: u8` - Button state (bit flags)
- `pub double_click: bool` - Double click flag

#### MouseButton Masks
- MB_LEFT_BUTTON, MB_MIDDLE_BUTTON, MB_RIGHT_BUTTON

#### Event Struct
**Fields:**
- `pub what: EventType` - Event type
- `pub key_code: KeyCode` - Keyboard code
- `pub key_modifiers: KeyModifiers` - Key modifiers
- `pub mouse: MouseEvent` - Mouse data
- `pub command: CommandId` - Command ID

**Public Methods:**
- `nothing() -> Self` - Create nothing event
- `keyboard(key_code: KeyCode) -> Self` - Create keyboard event
- `command(cmd: CommandId) -> Self` - Create command event
- `broadcast(cmd: CommandId) -> Self` - Create broadcast event
- `mouse(event_type: EventType, pos: Point, buttons: u8, double_click: bool) -> Self` - Create mouse event
- `from_crossterm_key(key_event: KeyEvent) -> Self` - Create from crossterm key
- `clear(&mut self)` - Mark event as handled

#### EscSequenceTracker Struct
**Public Methods:**
- `new() -> Self` - Create tracker
- `process_key(&mut self, key: KeyEvent) -> KeyCode` - Process key event for ESC sequences (macOS Alt emulation)

---

### Validators (`src/views/validator.rs`)

#### ValidatorStatus Enum
**Variants:** Ok, Syntax

#### Validator Trait
**Associated Methods:**
- `is_valid(&self, input: &str) -> bool` - Check if complete input is valid
- `is_valid_input(&self, input: &str, append: bool) -> bool` - Check during typing (default calls is_valid)
- `error(&self)` - Display error message
- `options(&self) -> u16` - Get validator options (default 0)
- `valid(&self, input: &str) -> bool` - Validate and show error if invalid

#### ValidatorStatus Flags
- VO_FILL: Fill with default on empty
- VO_TRANSFER: Enable data transfer
- VO_ON_APPEND: Validate on each character append

#### FilterValidator Struct
**Public Methods:**
- `new(valid_chars: &str) -> Self` - Create filter for allowed characters
- `with_options(valid_chars: &str, options: u16) -> Self` - Create with options

#### RangeValidator Struct
**Public Methods:**
- `new(min: i64, max: i64) -> Self` - Create for numeric range
- `with_options(min: i64, max: i64, options: u16) -> Self` - Create with options

#### ValidatorRef Type
**Definition:** `pub type ValidatorRef = Rc<RefCell<dyn Validator>>` - Shared validator reference

---

### History Management (`src/core/history.rs`)

#### HistoryList Struct
**Public Methods:**
- `new() -> Self` - Create empty history list
- `with_max_items(max_items: usize) -> Self` - Create with custom max items
- `add(&mut self, item: String)` - Add item to history (most recent first)
- `items(&self) -> &[String]` - Get all items
- `len(&self) -> usize` - Get number of items
- `is_empty(&self) -> bool` - Check if empty
- `clear(&mut self)` - Clear all items
- `get(&self, index: usize) -> Option<&String>` - Get item by index (0 = most recent)

#### HistoryManager Struct
**Static Methods:**
- Manages global history lists by ID

---

### Menu Data Structures (`src/core/menu_data.rs`)

#### MenuItem Enum
**Variants:**
- `Regular { text, command, key_code, help_ctx, enabled, shortcut }`
- `SubMenu { text, key_code, help_ctx, menu }`
- `Separator`

**Public Methods:**
- `new(text: &str, command: CommandId, key_code: KeyCode, help_ctx: u16) -> Self` - Create regular item
- `with_shortcut(text: &str, command: CommandId, key_code: KeyCode, shortcut: &str, help_ctx: u16) -> Self` - Create with display shortcut
- `new_disabled(text: &str, command: CommandId, key_code: KeyCode, help_ctx: u16) -> Self` - Create disabled item
- `submenu(text: &str, key_code: KeyCode, menu: Menu, help_ctx: u16) -> Self` - Create submenu item
- `separator() -> Self` - Create separator
- `is_selectable(&self) -> bool` - Check if selectable (not separator/disabled)
- `get_accelerator(&self) -> Option<char>` - Extract accelerator key from text

#### Menu Struct
**Public Methods:**
- `new() -> Self` - Create empty menu
- `with_items(items: Vec<MenuItem>) -> Self` - Create with items
- `add(&mut self, item: MenuItem)` - Add item to menu
- `item_count(&self) -> usize` - Get item count
- `get_item(&self, index: usize) -> Option<&MenuItem>` - Get item by index

#### MenuBuilder Struct
**Public Methods:**
- `new() -> Self` - Create builder
- `item(mut self, item: MenuItem) -> Self` - Add item (builder pattern)
- `build(self) -> Menu` - Build menu

---

### Status Line Data (`src/core/status_data.rs`)

#### StatusItem Struct
**Fields:**
- `pub text: String` - Display text
- `pub key_code: KeyCode` - Keyboard shortcut
- `pub command: CommandId` - Command to execute

**Public Methods:**
- `new(text: &str, key_code: KeyCode, command: CommandId) -> Self` - Create item

#### StatusDef Struct
**Public Methods:**
- `new() -> Self` - Create empty definition
- `add(&mut self, item: StatusItem)` - Add item
- `item_count(&self) -> usize` - Get count
- `get_item(&self, index: usize) -> Option<&StatusItem>` - Get by index

#### StatusLine Struct
**Public Methods:**
- `new(items: Vec<StatusItem>) -> Self` - Create status line
- `builder() -> StatusLineBuilder` - Create builder

#### StatusLineBuilder Struct
**Public Methods:**
- `new() -> Self` - Create builder
- `item(mut self, item: StatusItem) -> Self` - Add item
- `build(self) -> StatusLine` - Build status line

---

### Command System (`src/core/command.rs`)

#### CommandId Type
**Definition:** `pub type CommandId = u16`

#### Standard Commands
- CM_QUIT (24), CM_CLOSE (25), CM_OK (10), CM_CANCEL (11), CM_YES (12), CM_NO (13), CM_DEFAULT (14)

#### Broadcast Commands
- CM_COMMAND_SET_CHANGED (52), CM_RECEIVED_FOCUS (50), CM_RELEASED_FOCUS (51)
- CM_GRAB_DEFAULT (62), CM_RELEASE_DEFAULT (63)
- CM_FILE_FOCUSED (64), CM_FILE_DOUBLE_CLICKED (65)

#### File Menu Commands
- CM_NEW (102), CM_OPEN (103), CM_SAVE (104), CM_SAVE_AS (105), CM_SAVE_ALL (106), CM_CLOSE_FILE (107)

#### Edit Menu Commands
- CM_UNDO (110), CM_REDO (111), CM_CUT (112), CM_COPY (113), CM_PASTE (114)
- CM_SELECT_ALL (115), CM_FIND (116), CM_REPLACE (117), CM_SEARCH_AGAIN (118)

#### Search Menu Commands
- CM_FIND_IN_FILES (120), CM_GOTO_LINE (121)

#### View Menu Commands
- CM_ZOOM_IN (130), CM_ZOOM_OUT (131), CM_TOGGLE_SIDEBAR (132), CM_TOGGLE_STATUSBAR (133)

#### Help Menu Commands
- CM_HELP_INDEX (140), CM_KEYBOARD_REF (141)

#### Custom/Demo Commands
- CM_ABOUT (100), CM_BIRTHDATE (101), CM_TEXT_VIEWER (108), CM_CONTROLS_DEMO (109)
- CM_LISTBOX_DEMO (150), CM_LISTBOX_SELECT (151), CM_MEMO_DEMO (152)

---

## TERMINAL MODULE

### Terminal Struct (`src/terminal/mod.rs`)

**Public Methods:**

**Initialization & Shutdown:**
- `init() -> Result<Self>` - Initialize terminal in raw mode with alternate screen
- `shutdown(&mut self) -> Result<()>` - Restore terminal to normal mode

**Terminal Information:**
- `size(&self) -> (u16, u16)` - Get terminal size (width, height)

**Active View Management (for screen dumps):**
- `set_active_view_bounds(&mut self, bounds: Rect)` - Set bounds of active view
- `clear_active_view_bounds(&mut self)` - Clear active view bounds

**Clipping Region:**
- `push_clip(&mut self, rect: Rect)` - Push clipping region onto stack
- `pop_clip(&mut self)` - Pop clipping region

**Rendering:**
- `write_cell(&mut self, x: u16, y: u16, cell: Cell)` - Write single cell
- `write_line(&mut self, x: u16, y: u16, cells: &[Cell])` - Write line from draw buffer
- `clear(&mut self)` - Clear entire screen
- `flush(&mut self) -> io::Result<()>` - Flush changes to terminal (double-buffered)

**Cursor Control:**
- `show_cursor(&mut self, x: u16, y: u16) -> io::Result<()>` - Show cursor at position
- `hide_cursor(&mut self) -> io::Result<()>` - Hide cursor

**Event Handling:**
- `put_event(&mut self, event: Event)` - Queue event for next iteration
- `poll_event(&mut self, timeout: Duration) -> io::Result<Option<Event>>` - Poll for event with timeout
- `read_event(&mut self) -> io::Result<Event>` - Read event (blocking)

**Debugging (Screen Dumps):**
- `dump_screen(&mut self, path: &str) -> Result<()>` - Dump entire screen to ANSI file
- `dump_region(&mut self, x: u16, y: u16, width: u16, height: u16, path: &str) -> Result<()>` - Dump region to ANSI file
- `flash(&mut self) -> Result<()>` - Flash screen (visual feedback)

---

## VIEWS MODULE

### View Trait (`src/views/view.rs`)

**Core Methods (Required):**
- `bounds(&self) -> Rect` - Get view bounds
- `set_bounds(&mut self, bounds: Rect)` - Set view bounds
- `draw(&mut self, terminal: &mut Terminal)` - Draw view
- `handle_event(&mut self, event: &mut Event)` - Handle event

**Optional Methods (with defaults):**
- `can_focus(&self) -> bool` - Can receive focus (default: false)
- `set_focus(&mut self, focused: bool)` - Set focus state
- `is_focused(&self) -> bool` - Check if focused
- `options(&self) -> u16` - Get view option flags (default: 0)
- `set_options(&mut self, options: u16)` - Set view option flags
- `state(&self) -> StateFlags` - Get view state flags (default: 0)
- `set_state(&mut self, state: StateFlags)` - Set view state flags
- `set_state_flag(&mut self, flag: StateFlags, enable: bool)` - Set/clear specific flag(s)
- `get_state_flag(&self, flag: StateFlags) -> bool` - Check if flag(s) set
- `has_shadow(&self) -> bool` - Check if shadow enabled
- `shadow_bounds(&self) -> Rect` - Get bounds including shadow
- `update_cursor(&self, terminal: &mut Terminal)` - Update cursor state (default: do nothing)
- `dump_to_file(&self, terminal: &Terminal, path: &str) -> io::Result<()>` - Dump view region to ANSI file
- `is_default_button(&self) -> bool` - Check if default button (default: false)
- `button_command(&self) -> Option<u16>` - Get button command ID (default: None)
- `set_list_selection(&mut self, index: usize)` - Set listbox selection (default: do nothing)
- `get_list_selection(&self) -> usize` - Get listbox selection (default: 0)
- `get_redraw_union(&self) -> Option<Rect>` - Get union rect for movement tracking (default: None)
- `clear_move_tracking(&mut self)` - Clear movement tracking (default: do nothing)
- `get_end_state(&self) -> CommandId` - Get end state for modal views (default: 0)
- `set_end_state(&mut self, command: CommandId)` - Set end state for modal views

**Helper Functions:**
- `write_line_to_terminal(terminal: &mut Terminal, x: i16, y: i16, buf: &DrawBuffer)` - Draw line to terminal
- `draw_shadow(terminal: &mut Terminal, bounds: Rect, shadow_attr: u8)` - Draw shadow for view

---

### ListViewer Trait & State (`src/views/list_viewer.rs`)

#### ListViewerState Struct
**Fields:**
- `pub top_item: usize` - First visible item
- `pub focused: Option<usize>` - Currently focused item
- `pub range: usize` - Total number of items
- `pub num_cols: u16` - Number of columns for multi-column lists
- `pub handle_space: bool` - Whether space bar selects items

**Public Methods:**
- `new() -> Self` - Create new state
- `with_range(range: usize) -> Self` - Create with item count
- `set_range(&mut self, range: usize)` - Set total items (adjusts focused/top_item)
- `focus_item(&mut self, item: usize, visible_rows: usize)` - Focus specific item
- `focus_item_centered(&mut self, item: usize, visible_rows: usize)` - Focus and center item
- `focus_next(&mut self, visible_rows: usize)` - Focus next item
- `focus_prev(&mut self, visible_rows: usize)` - Focus previous item
- `focus_page_down(&mut self, visible_rows: usize)` - Focus one page down
- `focus_page_up(&mut self, visible_rows: usize)` - Focus one page up
- `focus_first(&mut self, visible_rows: usize)` - Focus first item
- `focus_last(&mut self, visible_rows: usize)` - Focus last item

#### ListViewer Trait
**Methods:**
- Part of View trait implementations for list-based views

---

### Syntax Highlighting (`src/views/syntax.rs`)

#### TokenType Enum
**Variants:** Normal, Keyword, String, Comment, Number, Operator, Identifier, Type, Preprocessor, Function, Special

**Public Methods:**
- `default_color(&self) -> Attr` - Get default color for token type

#### Token Struct
**Fields:**
- `pub start: usize` - Start column
- `pub end: usize` - End column (exclusive)
- `pub token_type: TokenType` - Token type

**Public Methods:**
- `new(start: usize, end: usize, token_type: TokenType) -> Self` - Create token

#### SyntaxHighlighter Trait
**Methods:**
- `language(&self) -> &str` - Get language name
- `highlight_line(&self, line: &str, line_number: usize) -> Vec<Token>` - Highlight single line
- `is_multiline_context(&self, line_number: usize) -> bool` - Check if in multiline context (default: false)
- `update_multiline_state(&mut self, line: &str, line_number: usize)` - Update multiline state (default: do nothing)

#### PlainTextHighlighter Struct
**Public Methods:**
- `new() -> Self` - Create plain text highlighter

#### RustHighlighter Struct
**Public Methods:**
- `new() -> Self` - Create Rust syntax highlighter

---

### Button (`src/views/button.rs`)

#### Button Struct
**Fields:**
- `pub bounds: Rect`
- `pub title: String`
- `pub command: CommandId`
- `pub is_default: bool`
- `pub disabled: bool`
- `pub state: StateFlags`

**Public Methods:**
- `new(bounds: Rect, title: &str, command: CommandId, is_default: bool) -> Self` - Create button
- `set_disabled(&mut self, disabled: bool)` - Set disabled state
- `is_disabled(&self) -> bool` - Check if disabled
- Implements View trait

#### ButtonBuilder Struct
**Public Methods:**
- `new() -> Self` - Create builder
- `bounds(mut self, bounds: Rect) -> Self` - Set bounds
- `title(mut self, title: impl Into<String>) -> Self` - Set title
- `command(mut self, command: CommandId) -> Self` - Set command
- `default(mut self, is_default: bool) -> Self` - Set as default button
- `build(self) -> Button` - Build button

---

### Label (`src/views/label.rs`)

#### Label Struct
**Public Methods:**
- `new(bounds: Rect, text: &str) -> Self` - Create label
- Implements View trait

---

### StaticText (`src/views/static_text.rs`)

#### StaticText Struct
**Public Methods:**
- `new(bounds: Rect, text: &str) -> Self` - Create static text
- `new_centered(bounds: Rect, text: &str) -> Self` - Create centered static text
- Implements View trait

---

### CheckBox (`src/views/checkbox.rs`)

#### CheckBox Struct
**Public Methods:**
- `new(bounds: Rect, label: &str) -> Self` - Create checkbox
- `set_checked(&mut self, checked: bool)` - Set checked state
- `is_checked(&self) -> bool` - Check if checked
- `toggle(&mut self)` - Toggle checked state
- Implements View trait

---

### RadioButton (`src/views/radiobutton.rs`)

#### RadioButton Struct
**Public Methods:**
- `new(bounds: Rect, label: &str, group_id: u16) -> Self` - Create radio button
- `set_selected(&mut self, selected: bool)` - Set selected state
- `is_selected(&self) -> bool` - Check if selected
- `select(&mut self)` - Select button
- `deselect(&mut self)` - Deselect button
- Implements View trait

---

### Frame (`src/views/frame.rs`)

#### FramePaletteType Enum
**Variants:** Standard, etc.

#### Frame Struct
**Public Methods:**
- `new(bounds: Rect, title: &str) -> Self` - Create frame with standard palette
- `with_palette(bounds: Rect, title: &str, palette_type: FramePaletteType) -> Self` - Create with palette
- Implements View trait

---

### Group (`src/views/group.rs`)

#### Group Struct
**Public Methods:**
- `new(bounds: Rect) -> Self` - Create empty group
- `with_background(bounds: Rect, background: Attr) -> Self` - Create with background
- `add(&mut self, view: Box<dyn View>)` - Add child view
- `set_initial_focus(&mut self)` - Set focus to first focusable child
- `clear_all_focus(&mut self)` - Clear focus from all children
- `len(&self) -> usize` - Get child count
- `is_empty(&self) -> bool` - Check if empty
- `child_at(&self, index: usize) -> &dyn View` - Get child by index
- `child_at_mut(&mut self, index: usize) -> &mut dyn View` - Get mutable child by index
- `set_focus_to(&mut self, index: usize)` - Set focus to child by index
- `bring_to_front(&mut self, index: usize) -> usize` - Move child to front
- `remove(&mut self, index: usize)` - Remove child by index
- `execute(&mut self, app: &mut Application) -> CommandId` - Execute as modal
- `end_modal(&mut self, command: CommandId)` - End modal execution
- `get_end_state(&self) -> CommandId` - Get end state
- `set_end_state(&mut self, command: CommandId)` - Set end state
- `broadcast(&mut self, event: &mut Event, owner_index: Option<usize>)` - Broadcast event
- `draw_sub_views(&mut self, terminal: &mut Terminal, start_index: usize, clip: Rect)` - Draw children
- `focused_child(&self) -> Option<&dyn View>` - Get focused child
- `select_next(&mut self)` - Move focus to next child
- `select_previous(&mut self)` - Move focus to previous child
- Implements View trait

---

### Cluster (Radio Button Group) (`src/views/cluster.rs`)

#### ClusterState Struct
**Public Methods:**
- `new() -> Self` - Create state
- `with_group(group_id: u16) -> Self` - Create with group ID
- `is_selected(&self, item_value: u32) -> bool` - Check if value selected
- `set_value(&mut self, value: u32)` - Set selected value
- `toggle(&mut self)` - Toggle selection

#### Cluster Trait
**Associated with View trait for radio button groups**

---

### ListBox (`src/views/listbox.rs`)

#### ListBox Struct
**Public Methods:**
- `new(bounds: Rect, on_select_command: CommandId) -> Self` - Create listbox
- `set_items(&mut self, items: Vec<String>)` - Set items
- `add_item(&mut self, item: String)` - Add item
- `clear(&mut self)` - Clear all items
- `get_selection(&self) -> Option<usize>` - Get selected index
- `get_selected_item(&self) -> Option<&str>` - Get selected item text
- `set_selection(&mut self, index: usize)` - Set selection
- `item_count(&self) -> usize` - Get item count
- `select_prev(&mut self)` - Select previous
- `select_next(&mut self)` - Select next
- `select_first(&mut self)` - Select first
- `select_last(&mut self)` - Select last
- `page_up(&mut self)` - Page up
- `page_down(&mut self)` - Page down
- Implements View & ListViewer traits

---

### SortedListBox (`src/views/sorted_listbox.rs`)

#### SortedListBox Struct
**Public Methods:**
- Similar to ListBox but maintains sorted order
- `new(bounds: Rect, on_select_command: CommandId) -> Self`
- Implements View & ListViewer traits

---

### ScrollBar (`src/views/scrollbar.rs`)

#### ScrollBar Struct
**Public Methods:**
- `new_vertical(bounds: Rect) -> Self` - Create vertical scrollbar
- `new_horizontal(bounds: Rect) -> Self` - Create horizontal scrollbar
- `set_params(&mut self, value: i32, min_val: i32, max_val: i32, pg_step: i32, ar_step: i32)` - Set all parameters
- `set_value(&mut self, value: i32)` - Set current value
- `set_range(&mut self, min_val: i32, max_val: i32)` - Set min/max range
- `get_value(&self) -> i32` - Get current value
- Implements View trait

---

### Scroller (`src/views/scroller.rs`)

#### Scroller Struct
**Public Methods:**
- `new(bounds: Rect, h_scrollbar: Option<Box<ScrollBar>>, v_scrollbar: Option<Box<ScrollBar>>) -> Self` - Create scroller
- `scroll_to(&mut self, x: i16, y: i16)` - Scroll to position
- `set_limit(&mut self, x: i16, y: i16)` - Set scroll limits
- `get_delta(&self) -> Point` - Get scroll delta
- `get_limit(&self) -> Point` - Get scroll limits
- `draw_scrollbars(&mut self, terminal: &mut Terminal)` - Draw scrollbars
- `handle_scrollbar_events(&mut self, event: &mut Event)` - Handle scrollbar events

---

### Indicator (`src/views/indicator.rs`)

#### Indicator Struct
**Public Methods:**
- `new(bounds: Rect) -> Self` - Create indicator
- `set_value(&mut self, location: Point, modified: bool)` - Set indicator position and modified flag
- Implements View trait

---

### ParamText (`src/views/paramtext.rs`)

#### ParamText Struct
**Public Methods:**
- `new(bounds: Rect, template: &str) -> Self` - Create with template
- `set_template(&mut self, template: &str)` - Set template
- `set_param_str(&mut self, value: &str)` - Set string parameter
- `set_params_str(&mut self, values: &[&str])` - Set multiple string parameters
- `set_param_num(&mut self, value: i64)` - Set numeric parameter
- `set_params(&mut self, str_params: &[&str], num_params: &[i64])` - Set mixed parameters
- `get_text(&self) -> &str` - Get rendered text
- `get_template(&self) -> &str` - Get template
- Implements View trait

---

### InputLine (`src/views/input_line.rs`)

#### InputLine Struct
**Public Methods:**
- `new(bounds: Rect, max_length: usize) -> Self` - Create input line
- Input field with text editing capabilities
- Implements View trait

---

### Memo (`src/views/memo.rs`)

#### Memo Struct
**Public Methods:**
- `new(bounds: Rect) -> Self` - Create memo
- `with_scrollbars(mut self, add_scrollbars: bool) -> Self` - Builder: add scrollbars
- `set_read_only(&mut self, read_only: bool)` - Set read-only
- `set_max_length(&mut self, max_length: Option<usize>)` - Set max length
- `set_tab_size(&mut self, tab_size: usize)` - Set tab size
- `get_text(&self) -> String` - Get all text
- `set_text(&mut self, text: &str)` - Set all text
- `is_modified(&self) -> bool` - Check if modified
- `clear_modified(&mut self)` - Clear modified flag
- `line_count(&self) -> usize` - Get number of lines
- `has_selection(&self) -> bool` - Check if text selected
- `get_selection(&self) -> Option<String>` - Get selected text
- `select_all(&mut self)` - Select all text
- Implements View trait

---

### Editor (`src/views/editor.rs`)

#### SearchOptions Struct
**Fields:** Various search configuration options

**Public Methods:**
- `new() -> Self` - Create options

#### Editor Struct
**Public Methods:**
- `new(bounds: Rect) -> Self` - Create editor
- `with_scrollbars_and_indicator(mut self) -> Self` - Builder: add scrollbars and indicator
- `set_read_only(&mut self, read_only: bool)` - Set read-only
- `set_tab_size(&mut self, tab_size: usize)` - Set tab size
- `set_auto_indent(&mut self, auto_indent: bool)` - Set auto-indent
- `set_highlighter(&mut self, highlighter: Box<dyn SyntaxHighlighter>)` - Set syntax highlighter
- `clear_highlighter(&mut self)` - Remove highlighter
- `has_highlighter(&self) -> bool` - Check if highlighter present
- `toggle_insert_mode(&mut self)` - Toggle insert/overwrite mode
- `get_text(&self) -> String` - Get all text
- `set_text(&mut self, text: &str)` - Set all text
- `is_modified(&self) -> bool` - Check if modified
- `clear_modified(&mut self)` - Clear modified flag
- `line_count(&self) -> usize` - Get line count
- `load_file(&mut self, path: impl AsRef<Path>) -> io::Result<()>` - Load from file
- `save_file(&mut self) -> io::Result<()>` - Save to original file
- `save_as(&mut self, path: impl AsRef<Path>) -> io::Result<()>` - Save as new file
- `get_filename(&self) -> Option<&str>` - Get loaded filename
- `undo(&mut self)` - Undo last change
- `redo(&mut self)` - Redo last undo
- `find(&mut self, text: &str, options: SearchOptions) -> Option<Point>` - Find text
- `find_next(&mut self) -> Option<Point>` - Find next occurrence
- `replace_selection(&mut self, replace_text: &str) -> bool` - Replace selected text
- `replace_next(&mut self, find_text: &str, replace_text: &str, options: SearchOptions) -> bool` - Replace next
- `replace_all(&mut self, find_text: &str, replace_text: &str, options: SearchOptions) -> usize` - Replace all
- Implements View trait

---

### FileList (`src/views/file_list.rs`)

#### FileEntry Struct
**Fields:** File information (name, size, is_dir, etc.)

**Public Methods:**
- `from_dir_entry(entry: &fs::DirEntry) -> io::Result<Self>` - Create from directory entry
- `display_name(&self) -> String` - Get display name
- `size_string(&self) -> String` - Get size formatted as string

#### FileList Struct
**Public Methods:**
- `new(bounds: Rect, path: &Path) -> Self` - Create file list
- `set_wildcard(&mut self, wildcard: &str)` - Set file filter
- `set_show_hidden(&mut self, show: bool)` - Show/hide hidden files
- `current_path(&self) -> &Path` - Get current directory
- `change_dir(&mut self, path: &Path) -> io::Result<()>` - Change directory
- `refresh(&mut self)` - Refresh file list
- `get_focused_entry(&self) -> Option<&FileEntry>` - Get focused file entry
- `get_selected_file(&self) -> Option<PathBuf>` - Get selected file path
- `enter_focused_dir(&mut self) -> io::Result<bool>` - Enter focused directory
- `file_count(&self) -> usize` - Get file count
- Implements View & ListViewer traits

---

### Window (`src/views/window.rs`)

#### Window Struct
**Public Methods:**
- `new(bounds: Rect, title: &str) -> Self` - Create window
- `add(&mut self, view: Box<dyn View>)` - Add child view
- `set_initial_focus(&mut self)` - Set initial focus
- `set_focus_to_child(&mut self, index: usize)` - Focus child by index
- `child_count(&self) -> usize` - Get child count
- `child_at(&self, index: usize) -> &dyn View` - Get child by index
- `child_at_mut(&mut self, index: usize) -> &mut dyn View` - Get mutable child
- `get_redraw_union(&self) -> Option<Rect>` - Get redraw union for moved windows
- `clear_move_tracking(&mut self)` - Clear movement tracking
- `execute(&mut self, app: &mut Application) -> CommandId` - Execute as modal
- `end_modal(&mut self, command: CommandId)` - End modal
- `get_end_state(&self) -> CommandId` - Get end state
- `set_end_state(&mut self, command: CommandId)` - Set end state
- Implements View & Group-like traits

#### WindowBuilder Struct
**Public Methods:**
- `new() -> Self` - Create builder
- `bounds(mut self, bounds: Rect) -> Self` - Set bounds
- `title(mut self, title: impl Into<String>) -> Self` - Set title
- `modal(mut self, is_modal: bool) -> Self` - Set modal flag
- `build(self) -> Window` - Build window

---

### Dialog (`src/views/dialog.rs`)

#### Dialog Struct
**Public Methods:**
- Dialog container that extends Window with common dialog features
- Implements View trait

---

### Desktop (`src/views/desktop.rs`)

#### Desktop Struct
**Public Methods:**
- `new(bounds: Rect) -> Self` - Create desktop
- `add(&mut self, view: Box<dyn View>)` - Add window
- `child_count(&self) -> usize` - Get window count
- `child_at(&self, index: usize) -> &dyn View` - Get window by index
- `remove_child(&mut self, index: usize)` - Remove window
- `draw_under_rect(&mut self, terminal: &mut Terminal, rect: Rect, start_from_window: usize)` - Draw background under rect
- `handle_moved_windows(&mut self, terminal: &mut Terminal) -> bool` - Handle moved windows
- `window_at_mut(&mut self, index: usize) -> Option<&mut dyn View>` - Get mutable window
- `remove_closed_windows(&mut self) -> bool` - Remove closed windows
- Implements View trait

---

### FileDialog (`src/views/file_dialog.rs`)

#### FileDialog Struct
**Public Methods:**
- `new(bounds: Rect, title: &str, wildcard: &str, initial_dir: Option<PathBuf>) -> Self` - Create dialog
- `build(mut self) -> Self` - Build dialog
- `execute(&mut self, app: &mut Application) -> Option<PathBuf>` - Execute and get selected file
- `get_selected_file(&self) -> Option<PathBuf>` - Get selected file
- Implements View trait

---

### FileEditor (`src/views/file_editor.rs`)

#### FileEditor Struct
**Public Methods:**
- `new(bounds: Rect) -> Self` - Create file editor
- `load_file(&mut self, path: PathBuf) -> io::Result<()>` - Load file
- `save(&mut self) -> io::Result<bool>` - Save file
- `save_as(&mut self, path: PathBuf) -> io::Result<()>` - Save as new file
- `filename(&self) -> Option<&PathBuf>` - Get filename
- `get_title(&self) -> String` - Get window title
- `is_modified(&self) -> bool` - Check if modified
- `set_text(&mut self, text: &str)` - Set text
- `valid(&mut self, app: &mut Application, command: CommandId) -> bool` - Validate on command
- `editor_mut(&mut self) -> &mut Editor` - Get mutable editor
- `editor(&self) -> &Editor` - Get editor
- Implements View trait

---

### EditWindow (`src/views/edit_window.rs`)

#### EditWindow Struct
**Public Methods:**
- `new(bounds: Rect, title: &str) -> Self` - Create edit window
- `load_file(&mut self, path: impl AsRef<Path>) -> io::Result<()>` - Load file
- `save_file(&mut self) -> io::Result<()>` - Save file
- `save_as(&mut self, path: impl AsRef<Path>) -> io::Result<()>` - Save as
- `get_filename(&self) -> Option<&str>` - Get filename
- `is_modified(&self) -> bool` - Check if modified
- `editor_mut(&mut self) -> &mut Editor` - Get mutable editor
- `editor(&self) -> &Editor` - Get editor
- Implements View trait

---

### MenuBar (`src/views/menu_bar.rs`)

#### SubMenu Struct
**Public Methods:**
- `new(name: &str, menu: Menu) -> Self` - Create submenu item

#### MenuBar Struct
**Public Methods:**
- `new(bounds: Rect) -> Self` - Create menu bar
- `add_submenu(&mut self, submenu: SubMenu)` - Add submenu
- `check_cascading_submenu(&mut self, terminal: &mut Terminal) -> Option<u16>` - Check cascading menus
- Implements View trait

---

### MenuBox (`src/views/menu_box.rs`)

#### MenuBox Struct
**Public Methods:**
- `new(position: Point, menu: Menu) -> Self` - Create menu box
- `get_selected_command(&self) -> Option<CommandId>` - Get selected command
- `execute(&mut self, terminal: &mut Terminal) -> CommandId` - Execute menu
- Implements View trait

---

### MenuViewer (`src/views/menu_viewer.rs`)

#### MenuViewerState Struct
**Public Methods:**
- State management for menu viewing

#### MenuViewer Trait
**Methods:**
- `new() -> Self` - Create menu viewer state
- `with_menu(menu: Menu) -> Self` - Create with menu
- `set_menu(&mut self, menu: Menu)` - Set menu
- `get_menu(&self) -> Option<&Menu>` - Get menu
- `get_menu_mut(&mut self) -> Option<&mut Menu>` - Get mutable menu
- `get_current_item(&self) -> Option<&MenuItem>` - Get current item
- `select_next(&mut self)` - Select next item
- `select_prev(&mut self)` - Select previous item
- `find_item_by_char(&self, ch: char) -> Option<usize>` - Find item by character
- `find_item_by_hotkey(&self, key_code: KeyCode) -> Option<usize>` - Find item by hotkey
- `item_count(&self) -> usize` - Get item count

---

### StatusLine (`src/views/status_line.rs`)

#### StatusItem Struct (View-specific)
**Public Methods:**
- `new(text: &str, key_code: KeyCode, command: CommandId) -> Self` - Create item

#### StatusLine Struct
**Public Methods:**
- `new(bounds: Rect, items: Vec<StatusItem>) -> Self` - Create status line
- `set_hint(&mut self, hint: Option<String>)` - Set hint text
- Implements View trait

---

### TextViewer (`src/views/text_viewer.rs`)

#### TextViewer Struct
**Public Methods:**
- `new(bounds: Rect) -> Self` - Create text viewer
- Implements View trait

---

### ListViewer Implementations (`src/views/list_viewer.rs`)

#### ListViewer Trait (public)
**Methods:**
- Default implementations for list viewing behavior
- Used by ListBox, SortedListBox, etc.

---

### LookupValidator (`src/views/lookup_validator.rs`)

#### LookupValidator Struct
**Public Methods:**
- `new(valid_values: Vec<String>) -> Self` - Create case-sensitive validator
- `new_case_insensitive(valid_values: Vec<String>) -> Self` - Create case-insensitive
- `set_case_sensitive(&mut self, case_sensitive: bool)` - Set case sensitivity
- `valid_values(&self) -> &[String]` - Get valid values
- `add_value(&mut self, value: String)` - Add value
- `remove_value(&mut self, value: &str) -> bool` - Remove value
- `contains(&self, value: &str) -> bool` - Check if value exists
- Implements Validator trait

---

### PictureValidator (`src/views/picture_validator.rs`)

#### PictureValidator Struct
**Public Methods:**
- `new(mask: &str) -> Self` - Create with format mask
- `new_no_format(mask: &str) -> Self` - Create without auto-formatting
- `mask(&self) -> &str` - Get mask
- `set_auto_format(&mut self, auto_format: bool)` - Set auto-format
- `format(&self, input: &str) -> String` - Format input according to mask
- Implements Validator trait

---

### HelpContext (`src/views/help_context.rs`)

#### HelpContext Struct
**Public Methods:**
- `new(id: String, name: String) -> Self` - Create context
- Help system context structure

---

### HelpTopic & HelpFile (`src/views/help_file.rs`)

#### HelpTopic Struct
**Public Methods:**
- `new(id: String, title: String) -> Self` - Create topic
- `add_line(&mut self, line: String)` - Add text line
- `add_link(&mut self, topic_id: String)` - Add link to other topic
- `get_formatted_content(&self) -> Vec<String>` - Get formatted display lines

#### HelpFile Struct
**Public Methods:**
- `new(path: impl AsRef<Path>) -> io::Result<Self>` - Load help file
- `get_topic(&self, id: &str) -> Option<&HelpTopic>` - Get topic by ID
- `get_default_topic(&self) -> Option<&HelpTopic>` - Get default topic
- `get_topic_ids(&self) -> Vec<String>` - Get all topic IDs
- `has_topic(&self, id: &str) -> bool` - Check if topic exists
- `path(&self) -> &str` - Get file path
- `reload(&mut self) -> io::Result<()>` - Reload from disk

---

### HelpViewer (`src/views/help_viewer.rs`)

#### HelpViewer Struct
**Public Methods:**
- `new(bounds: Rect) -> Self` - Create help viewer
- Implements View trait

---

### HelpWindow (`src/views/help_window.rs`)

#### HelpWindow Struct
**Public Methods:**
- `new(pos: Point, help_file: HelpFile, initial_topic: &str) -> Self` - Create help window
- Implements View trait

---

### History (`src/views/history.rs`)

#### History Struct
**Public Methods:**
- `new(pos: Point, history_id: u16) -> Self` - Create history control
- `has_items(&self) -> bool` - Check if history has items

---

### HistoryViewer (`src/views/history_viewer.rs`)

#### HistoryViewer Struct
**Public Methods:**
- `new(bounds: Rect, history_id: u16) -> Self` - Create history viewer
- `refresh(&mut self)` - Refresh from history manager
- `get_selected_item(&self) -> Option<&str>` - Get selected item
- `item_count(&self) -> usize` - Get item count
- Implements View & ListViewer traits

---

### HistoryWindow (`src/views/history_window.rs`)

#### HistoryWindow Struct
**Public Methods:**
- `new(pos: Point, history_id: u16, width: i16) -> Self` - Create history popup window
- `execute(&mut self, terminal: &mut Terminal) -> Option<String>` - Execute and get selection
- Implements View trait

---

### DirListBox (`src/views/dir_listbox.rs`)

#### DirListBox Struct
**Public Methods:**
- Directory list selection box
- `new(bounds: Rect, title: &str, path: &Path) -> Self`
- Implements View & ListViewer traits

---

### Background (`src/views/background.rs`)

#### Background Struct
**Public Methods:**
- `new(bounds: Rect) -> Self` - Create background
- Implements View trait

---

### Cluster (Radio Button Group) (`src/views/cluster.rs`)

#### ClusterState Struct
**Public Methods:**
- Already documented above

#### Cluster Trait
**Associated Methods:**
- Trait for managing groups of radio buttons

---

## APPLICATION MODULE

### Application Struct (`src/app/application.rs`)

**Public Methods:**

**Initialization:**
- `new() -> Result<Self>` - Create application with initialized terminal

**Menu & Status:**
- `set_menu_bar(&mut self, menu_bar: MenuBar)` - Set menu bar
- `set_status_line(&mut self, status_line: StatusLine)` - Set status line

**Desktop Management:**
- `add_window(&mut self, window: Box<dyn View>)` - Add window to desktop
- `execute_window(&mut self, window: &mut dyn View) -> CommandId` - Execute window as modal

**Event Loop:**
- `run(&mut self) -> Result<()>` - Main event loop
- `quit(&mut self)` - Signal application to quit

**Shutdown:**
- `shutdown(mut self) -> Result<()>` - Clean shutdown and restore terminal

**Fields:**
- `pub terminal: Terminal` - Terminal instance
- `pub menu_bar: Option<MenuBar>` - Optional menu bar
- `pub status_line: Option<StatusLine>` - Optional status line
- `pub desktop: Desktop` - Main desktop
- `pub running: bool` - Running flag

---

## TRAIT INHERITANCE PATTERNS

### View Trait Hierarchy

All UI components implement the `View` trait, which provides:

1. **Core Interface:**
   - `bounds()` / `set_bounds()` - Position and size management
   - `draw()` - Rendering
   - `handle_event()` - Event handling

2. **Focus Management:**
   - `can_focus()` - Ability to receive focus
   - `set_focus()` / `is_focused()` - Focus state

3. **State & Options:**
   - `state()` / `set_state()` - State flag management
   - `options()` / `set_options()` - Option flag management
   - `set_state_flag()` / `get_state_flag()` - Individual flag control

4. **Shadow Support:**
   - `has_shadow()` - Shadow presence
   - `shadow_bounds()` - Bounds including shadow

5. **Cursor Management:**
   - `update_cursor()` - Cursor positioning for input fields

6. **Debugging:**
   - `dump_to_file()` - ANSI dump for debugging

7. **Special Behaviors:**
   - `is_default_button()` - For dialog enter key handling
   - `button_command()` - For button-specific logic
   - `set_list_selection()` / `get_list_selection()` - For listbox integration
   - `get_redraw_union()` / `clear_move_tracking()` - For window movement
   - `get_end_state()` / `set_end_state()` - For modal execution

### Container Patterns

**Group-like Containers:**
- `Group` - General purpose child view container
- `Window` - Top-level window (extends Group)
- `Desktop` - Manages windows
- `Dialog` - Dialog container (extends Window)

**List-based Views:**
- Implement both `View` and `ListViewer` traits
- Use `ListViewerState` for shared state management
- Examples: `ListBox`, `SortedListBox`, `FileList`, `HistoryViewer`

### Validator Trait Hierarchy

All input validators implement the `Validator` trait:
- `FilterValidator` - Character set validation
- `RangeValidator` - Numeric range validation
- `LookupValidator` - Value in list validation
- `PictureValidator` - Format mask validation

### Syntax Highlighting

`SyntaxHighlighter` trait implementations:
- `PlainTextHighlighter` - No highlighting
- `RustHighlighter` - Rust language support

---

## DESIGN PATTERNS

### 1. Builder Pattern

**ButtonBuilder:**
```
ButtonBuilder::new()
    .bounds(rect)
    .title("Click Me")
    .command(CM_OK)
    .default(true)
    .build()
```

**WindowBuilder:**
```
WindowBuilder::new()
    .bounds(rect)
    .title("Window")
    .modal(true)
    .build()
```

**MenuBuilder:**
```
MenuBuilder::new()
    .item(MenuItem::new(...))
    .item(MenuItem::separator())
    .build()
```

### 2. State Management Pattern

**ListViewerState** - Embedded state struct for list management
**ClusterState** - Embedded state for radio button groups

Components embed these state structs and expose them via trait methods.

### 3. Composition over Inheritance

- `Group` contains multiple `View` children
- `Window` is a `Group` with title and frame
- `Desktop` manages multiple windows
- `FileList` contains `ListViewerState` and implements `ListViewer`

### 4. Trait-based Polymorphism

- `View` trait for all UI components
- `ListViewer` trait for list-based views
- `Validator` trait for input validation
- `SyntaxHighlighter` trait for code highlighting

### 5. Event Propagation

Events flow up through the component hierarchy:
- Child handles event and sets `event.what = EventType::Command` to bubble up
- Parent Group receives the command event
- Application processes the final command

### 6. Reference Counting for Shared State

- `ValidatorRef = Rc<RefCell<dyn Validator>>`
- Allows InputLine to hold reference to validator
- Mutable access through RefCell

---

## PUBLIC ENUM TYPES

### EventType
Variants: Nothing, Keyboard, MouseDown, MouseUp, MouseMove, MouseAuto, MouseWheelUp, MouseWheelDown, Command, Broadcast

### ValidatorStatus
Variants: Ok, Syntax

### MenuItem
Variants: Regular, SubMenu, Separator

### TokenType
Variants: Normal, Keyword, String, Comment, Number, Operator, Identifier, Type, Preprocessor, Function, Special

### FramePaletteType
Various frame styles

### TvColor
16 standard colors (Black, Blue, Green, etc.)

---

## COMMONLY USED TYPE ALIASES

- `CommandId = u16` - Command identifier
- `KeyCode = u16` - Keyboard code
- `ValidatorRef = Rc<RefCell<dyn Validator>>` - Shared validator
- `StateFlags = u16` - State flag bits

---

## CONSTANT GROUPS

### Key Codes (16-bit: high byte = scan, low byte = char)
### Event Masks (16-bit: event type filters)
### Mouse Button Masks
### Validator Options
### Frame Constants

---

## API SUMMARY STATISTICS

- **Total Rust Files:** 68 files
- **Public Structs:** 90+
- **Public Traits:** 5 major (View, ListViewer, Validator, SyntaxHighlighter, Cluster)
- **Public Methods:** 500+
- **Key Modules:** core, terminal, views, app, test_util

---

