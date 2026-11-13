# Turbo Vision Rust API Catalog

> Checked against the source on 2026-10-03. Every module under `src/views/`
> and `src/core/` has a section; `tests/docs_index.rs` fails when one is
> missing. For how to use the components together, start with
> [`AGENTS.md`](../AGENTS.md).

Comprehensive catalog of all public structs, traits, and their public methods in the Turbo Vision Rust codebase.


---

## Table of Contents

1. [Core Module](#core-module)
2. [I18n Module](#i18n-module)
3. [Terminal Module](#terminal-module)
4. [Views Module](#views-module)
5. [Application Module](#application-module)

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
- KB_F1...KB_F12, KB_CTRL_F12
- KB_UP, KB_DOWN, KB_LEFT, KB_RIGHT
- KB_HOME, KB_END, KB_PGUP, KB_PGDN, KB_INS, KB_DEL
- KB_ALT_X, KB_ALT_F, KB_ALT_H, KB_ALT_O, KB_ALT_A, KB_ALT_F3
- KB_ESC_F, KB_ESC_H, KB_ESC_X, KB_ESC_A, KB_ESC_O, KB_ESC_E, KB_ESC_S, KB_ESC_V, KB_ESC_ESC
- KB_TEXT - Key code of a typed character past U+00FF; the character is in `Event::ch`
- `alt_code(letter: char) -> Option<KeyCode>` - Alt plus an ASCII letter, either case (Borland `getAltCode`); `None` otherwise

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
- `pub ch: Option<char>` - Character a keyboard event types, as the terminal reported it (`None` for keys that type nothing)

**Public Methods:**
- `nothing() -> Self` - Create nothing event
- `keyboard(key_code: KeyCode) -> Self` - Create keyboard event
- `text(ch: char) -> Self` - Create a key that types `ch`, with the key code a terminal gives it
- `typed_char(&self) -> Option<char>` - The character this event types into a text field: any character one cell wide (read this rather than `key_code` for text)
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

### Timed Events (`src/core/timed_event.rs`)

Per-thread queue of events delivered by `Terminal::poll_event` once due.
- `post_after(event: Event, delay: Duration)` - Deliver `event` after `delay`
- `post_at(event: Event, due: Instant)` - Deliver `event` once `due` has come
- `take_due(now: Instant) -> Option<Event>` - Remove the earliest event due at `now`
- `next_due() -> Option<Instant>` - When the earliest pending event is due
- `clear()` - Drop every pending event

---

### Menu Data Structures (`src/core/menu_data.rs`)

Per-thread queue of events delivered by `Terminal::poll_event` once due.
- `post_after(event: Event, delay: Duration)` - Deliver `event` after `delay`
- `post_at(event: Event, due: Instant)` - Deliver `event` once `due` has come
- `take_due(now: Instant) -> Option<Event>` - Remove the earliest event due at `now`
- `next_due() -> Option<Instant>` - When the earliest pending event is due
- `clear()` - Drop every pending event

---

### Menu Data Structures (`src/core/menu_data.rs`)

Menu data structures - declarative menu building with Borland-compatible API.

- `enum MenuItem` - Menu item - can be a regular command, a submenu, or a separator

**MenuItem**
- `flag(text: &str, command: CommandId, key_code: KeyCode, help_ctx: u16, checked: fn() -> bool) -> Self` - Create a flag (checkable) menu item
- `submenu(text: &str, key_code: KeyCode, menu: Menu, help_ctx: u16) -> Self` - Create a submenu item
- `separator() -> Self` - Create a separator
- `is_selectable(&self) -> bool` - Check if this item is selectable (not a separator and not disabled)
- `get_accelerator(&self) -> Option<char>` - Extract the accelerator key from the text (character between ~ marks)
- `text(&self) -> &str` - Get the display text (with ~ markers)
- `command(&self) -> Option<CommandId>` - Get the command (for Regular items only)
- `shortcut(&self) -> Option<&str>` - Get the shortcut display text (for Regular items only)
- `struct Menu` - Menu - a collection of menu items

**Menu**
- `new() -> Self` - Create an empty menu
- `from_items(items: Vec<MenuItem>) -> Self` - Create a menu from items
- `with_default(items: Vec<MenuItem>, default_index: usize) -> Self` - Create a menu with a default item
- `add(&mut self, item: MenuItem)` - Add an item to the menu
- `find_hotkey(&self, key_code: KeyCode) -> Option<CommandId>` - Find the command bound to a keyboard shortcut, searching submenus.
- `set_default(&mut self, index: usize)` - Set the default item by index
- `len(&self) -> usize` - Get the number of items
- `is_empty(&self) -> bool` - Check if menu is empty
- `struct MenuBuilder` - Builder for constructing menus fluently

**MenuBuilder**
- `new() -> Self` - Create a new menu builder
- `help_context(mut self, help_ctx: u16) -> Self` - Set the default help context for subsequent items
- `item(mut self, text: &str, command: CommandId) -> Self` - Add an item with no key binding.
- `item_key(mut self, text: &str, command: CommandId, chord: &str) -> Self` - Add an item bound to a key chord such as `"Ctrl+O"`, `"F3"` or `"Alt+X"`; the chord is both bound and shown next to the text.
- `item_disabled(mut self, text: &str, command: CommandId) -> Self` - Add a disabled item.
- `add(mut self, item: MenuItem) -> Self` - Add an item built elsewhere, typically with [`MenuItemBuilder`].
- `submenu(mut self, text: &str, key_code: KeyCode, menu: Menu) -> Self` - Add a submenu
- `separator(mut self) -> Self` - Add a separator
- `build(self) -> Menu` - Build the menu
- `struct MenuItemBuilder` - Builder for creating regular menu items with a fluent API.

**MenuItemBuilder**
- `new() -> Self` - Creates a new MenuItemBuilder with default values.
- `text(mut self, text: impl Into<String>) -> Self` - Sets the menu item text (required).
- `command(mut self, command: CommandId) -> Self` - Sets the command to execute (required).
- `key_code(mut self, key_code: KeyCode) -> Self` - Sets the keyboard shortcut key code.
- `help_ctx(mut self, help_ctx: u16) -> Self` - Sets the help context ID.
- `enabled(mut self, enabled: bool) -> Self` - Sets whether the menu item is enabled (default: true).
- `shortcut(mut self, shortcut: impl Into<String>) -> Self` - Sets the shortcut display text (e.g., "F3", "Ctrl+O") without binding a key; see [`key`](Self::key) to do both from one chord.
- `key(mut self, chord: &str) -> Self` - Binds the item to a key chord such as `"Ctrl+O"` and shows it next to the text.
- `checked(mut self, checked: fn() -> bool) -> Self` - Makes this a flag (checkable) item; `checked` is queried on every draw.
- `build(self) -> MenuItem` - Builds the MenuItem::Regular variant.

---

### Status Line Data (`src/core/status_data.rs`)

Status line data structures - declarative status bar building with command-based visibility.

- `struct StatusItem` - Status line item - displays text and responds to keyboard shortcuts

**StatusItem**
- `get_accelerator(&self) -> Option<char>`
- `struct StatusItemBuilder` - Builder for creating status items with a fluent API.

**StatusItemBuilder**
- `new() -> Self` - Creates a new StatusItemBuilder with default values.
- `text(mut self, text: impl Into<String>) -> Self` - Sets the status item text (required).
- `key_code(mut self, key_code: KeyCode) -> Self` - Sets the keyboard shortcut key code.
- `key(mut self, chord: &str) -> Self` - Binds the item to a key chord such as `"Alt+X"` or `"F10"`.
- `command(mut self, command: CommandId) -> Self` - Sets the command to execute.
- `build(self) -> StatusItem` - Builds the StatusItem.
- `struct StatusDef` - Status line definition - defines which items are visible for a command set range

**StatusDef**
- `new(min: u16, max: u16, items: Vec<StatusItem>) -> Self` - Create a new status definition
- `default_range(items: Vec<StatusItem>) -> Self` - Create a status definition for all command ranges (default)
- `applies_to(&self, command_set: u16) -> bool` - Check if this definition applies to the given command set
- `add(&mut self, item: StatusItem)` - Add an item to this definition
- `len(&self) -> usize` - Get the number of items
- `is_empty(&self) -> bool` - Check if definition has no items
- `struct StatusDefBuilder` - Builder for creating status definitions with a fluent API.

**StatusDefBuilder**
- `new() -> Self` - Creates a new StatusDefBuilder with default values (full range: 0-0xFFFF).
- `range(mut self, min: u16, max: u16) -> Self` - Sets the command range (default: 0-0xFFFF).
- `min(mut self, min: u16) -> Self` - Sets the minimum command ID.
- `max(mut self, max: u16) -> Self` - Sets the maximum command ID.
- `add_item(mut self, item: StatusItem) -> Self` - Adds a status item to the definition.
- `items(mut self, items: Vec<StatusItem>) -> Self` - Sets all items at once.
- `build(self) -> StatusDef` - Builds the StatusDef.
- `struct StatusLine` - Status line configuration - collection of status definitions

**StatusLine**
- `new(defs: Vec<StatusDef>) -> Self` - Create a new status line configuration
- `single(items: Vec<StatusItem>) -> Self` - Create a status line with a single default definition
- `get_def_for(&self, command_set: u16) -> Option<&StatusDef>` - Get the status definition that applies to the given command set
- `add_def(&mut self, def: StatusDef)` - Add a status definition
- `struct StatusLineBuilder` - Builder for constructing status line configurations fluently

**StatusLineBuilder**
- `new() -> Self` - Create a new status line builder
- `add_def(mut self, min: u16, max: u16, items: Vec<StatusItem>) -> Self` - Add a status definition with command range
- `add_default_def(mut self, items: Vec<StatusItem>) -> Self` - Add a default status definition (applies to all command sets)
- `build(self) -> StatusLine` - Build the status line configuration

---

### Flag Types (`src/core/state.rs`)
- `State` (`VISIBLE`, `CURSOR_VIS`, `CURSOR_INS`, `SHADOW`, `ACTIVE`, `SELECTED`, `FOCUSED`, `DRAGGING`, `DISABLED`, `MODAL`, `DEFAULT`, `EXPOSED`, `CLOSED`, `RESIZING`)
- `Options` (`SELECTABLE`, `TOP_SELECT`, `FIRST_CLICK`, `FRAMED`, `PRE_PROCESS`, `POST_PROCESS`, `BUFFERED`, `TILEABLE`, `CENTER_X`, `CENTER_Y`, `CENTERED`, `VALIDATE`)
- `Grow` (`LO_X`, `LO_Y`, `HI_X`, `HI_Y`, `ALL`)
- Each has `empty()`, `bits()`, `from_bits()`, `contains()`, `intersects()`, `is_empty()`, `insert()`, `remove()`, `set()` and the bit operators. `MsgBox` (`views::msgbox`) and `ValidatorOptions` (`views::validator`) are built the same way. The 2.x `SF_*`, `OF_*`, `GF_GROW_*`, `MF_*`, `VO_*` constants are deprecated aliases.

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

#### Command Ownership
- `0..=99` Borland's standard commands (including `CM_NEW`..`CM_CLOSE_FILE` at 30..35)
- `100..=199` this crate's internal commands and broadcasts
- `CM_USER` (200) and up: free for applications

### Command Set (`src/core/command_set.rs`)

Command Set System

- `command_enabled(command: CommandId) -> bool` - Check if a command is currently enabled (global query) Matches Borland: TView::commandEnabled(ushort command) (tview.cc:142-147)
- `enable_command(command: CommandId)` - Enable a command in the global command set Matches Borland: TView::enableCommand(ushort command) (tview.cc:384-389)
- `disable_command(command: CommandId)` - Disable a command in the global command set Matches Borland: TView::disableCommand(ushort command) (tview.cc:161-166)
- `get_commands() -> CommandSet` - A copy of the global command set, to put back later with [`set_commands`].
- `set_commands(commands: CommandSet)` - Replace the global command set, flagging a change if it differs.
- `command_set_changed() -> bool` - Check if command set has changed (needs broadcast) Matches Borland: TView::commandSetChanged (tview.cc:51)
- `clear_command_set_changed()` - Clear the command set changed flag Called after broadcasting CM_COMMAND_SET_CHANGED
- `init_command_set()` - Initialize the global command set with specific disabled commands Matches Borland: initCommands() (tview.cc:58-68)
- `struct CommandSet` - Command set bitfield for tracking enabled/disabled commands

**CommandSet**
- `new() -> Self` - Create a new command set with all commands disabled
- `with_all_enabled() -> Self` - Create a command set with all commands enabled
- `has(&self, command: CommandId) -> bool` - Check if a command is enabled
- `enable_command(&mut self, command: CommandId)` - Enable a single command
- `disable_command(&mut self, command: CommandId)` - Disable a single command
- `enable_range(&mut self, cmd_start: CommandId, cmd_end: CommandId)` - Enable a range of commands (inclusive)
- `disable_range(&mut self, cmd_start: CommandId, cmd_end: CommandId)` - Disable a range of commands (inclusive)
- `enable_set(&mut self, other: &CommandSet)` - Enable all commands in another command set
- `disable_set(&mut self, other: &CommandSet)` - Disable all commands in another command set
- `enable_all(&mut self)` - Enable all commands
- `is_empty(&self) -> bool` - Check if command set is empty (all commands disabled)
- `intersect(&mut self, other: &CommandSet)` - Perform bitwise AND with another command set
- `union(&mut self, other: &CommandSet)` - Perform bitwise OR with another command set

---

### Clipboard (`src/core/clipboard.rs`)

Clipboard support - global clipboard management with OS integration.

- `set_clipboard(text: &str)` - Set the clipboard content (both in-memory and OS clipboard)
- `get_clipboard() -> String` - Get the clipboard content (prefers OS clipboard, falls back to in-memory)
- `has_clipboard_content() -> bool` - Check if the clipboard has content
- `clear_clipboard()` - Clear the clipboard (both in-memory and OS)

---

### Keys (`src/core/keys.rs`)

Key types, from one place.

---

### Errors (`src/core/error.rs`)

Error types for Turbo Vision operations.

- `struct TurboVisionError` - Error type for Turbo Vision operations.

**TurboVisionError**
- `is_io(&self) -> bool` - Returns `true` if this error is an I/O error.
- `is_terminal_init(&self) -> bool` - Returns `true` if this error is a terminal initialization error.
- `is_invalid_input(&self) -> bool` - Returns `true` if this error is an invalid input error.
- `is_parse(&self) -> bool` - Returns `true` if this error is a parse error.
- `is_file_operation(&self) -> bool` - Returns `true` if this error is a file operation error.
- `file_path(&self) -> Option<&std::path::Path>` - Returns the file path if this is a file operation error.
- `type Result` - Result type for Turbo Vision operations.

---

### Palette Chain (`src/core/palette_chain.rs`)

QCell-based safe palette chain for Borland-compatible owner traversal.

- `palette_token() -> &'static QCellOwner` - Get the global palette token.
- `struct PaletteChainNode` - A node in the palette owner chain.

**PaletteChainNode**
- `new(palette: Option<Palette>, parent: Option<PaletteChainNode>) -> Self` - Create a new palette chain node.
- `nearest_palette_len(&self) -> Option<usize>` - Length of the nearest non-empty palette on the way up the chain, or `None` when no ancestor below the application carries one.
- `remap_color(&self, mut color: u8) -> u8` - Walk up the owner chain, remapping a color index through each ancestor's palette.

---

### ANSI Dumps (`src/core/ansi_dump.rs`)

ANSI dump utilities for debugging terminal output

- `dump_buffer_to_file(buffer: &[Vec<Cell>], width: usize, height: usize, path: &str) -> io::Result<()>` - Dump a buffer to an ANSI text file.
- `dump_buffer<W: Write>(writer: &mut W, buffer: &[Vec<Cell>], width: usize, height: usize) -> io::Result<()>` - Dump a buffer to any writer with ANSI color codes.
- `dump_buffer_region<W: Write>(writer: &mut W, buffer: &[Vec<Cell>], x: usize, y: usize, width: usize, height: usize) -> io::Result<()>` - Dump a rectangular region of a buffer.

---

### Screenshots (`src/core/screenshot/`, feature `screenshot`)

Screenshot rendering: turn the terminal cell buffer into a PNG image.

- `render_to_png(buffer: &[Vec<Cell>], cols: usize, rows: usize, scale: usize, path: &Path) -> io::Result<()>` - Render the screen buffer to a PNG file at `path`.

---

## I18N MODULE

### Translations (`src/i18n/`, feature `i18n`)

The text Turbo Vision and the application show, in the user's language, with
Project Fluent. The crate's messages live in `locales/<locale>/turbo-vision.ftl`
(`en`, `es`, `ca`), embedded in the binary. The catalogue is per thread and
starts in English. With the feature off, messages are read as plain text with
`{ $name }` placeholders, and Fluent selectors are not available.

- `tr(id: &str) -> String` - The message in the current locale, falling back along the chain to `en`, then to `id` itself.
- `tr_args(id: &str, args: &[(&str, Arg)]) -> String` - The same, with placeholder values.
- `tr!(id)`, `tr!(id, name = value, ...)` - Macro form of `tr` / `tr_args` (exported at the crate root).
- `enum Arg<'a>` - `Text(Cow<str>)`, `Int(i64)`, `Float(f64)`; `From` text and the number types.
- `struct LazyMessage` (`src/i18n/lazy.rs`) - An id and its arguments, translated when displayed, in the displaying thread's locale; `Send`, `Clone`, `Display`. For text made on a worker thread, before the locale is chosen, or kept across a change of language.
  - `const fn new(id: &'static str) -> Self`, `from_id(id: impl Into<String>) -> Self`
  - `arg(self, name: &'static str, value: impl Into<LazyArg>) -> Self` - Text, a number or another `LazyMessage`; a repeated name replaces its value
  - `id(&self) -> &str`, `resolve(&self) -> String` (also `to_string()`, `String::from(&message)`)
- `enum LazyArg` - `Value(Arg<'static>)`, `Message(LazyMessage)`; `From` everything `Arg` takes, and `LazyMessage`.
- `tr_lazy!(id)`, `tr_lazy!(id, name = value, ...)` - Like `tr!`, but makes a `LazyMessage` (exported at the crate root).
- `has_message(id: &str) -> bool`
- `set_locale(tag: &str) -> String` - Choose the locale (`"ca"`, `"es-MX"`, `"ca_ES.UTF-8"`); returns the locale used first.
- `set_locale_from_env() -> String` - Choose the user's locale (`LC_ALL`, `LC_MESSAGES`, `LANG`), or English.
- `system_locale() -> Option<String>` - The user's locale, normalised (`ca-ES`); `None` for unset, `C` or `POSIX`.
- `requested_locale() -> String`, `locale_chain() -> Vec<String>`, `available_locales() -> Vec<String>`
- `add_translations(locale: &str, source: &str) -> Result<(), TranslationError>` - Add Fluent messages; a later message with the same id replaces the earlier one, including a built-in message.
- `add_translations_file(locale: &str, path) -> Result<(), Box<dyn Error>>`
- `localized_path(path) -> PathBuf` - `manual.ca.md` for `manual.md` when that file exists for a locale of the chain.
- `struct TranslationError { locale, problems }`
- `const DEFAULT_LOCALE: &str = "en"`

---

## TERMINAL MODULE

### Terminal Struct (`src/terminal/mod.rs`)

**Public Methods:**

**Initialization & Shutdown:**
- `init() -> Result<Self>` - Initialize terminal in raw mode with alternate screen
- `shutdown(&mut self) -> Result<()>` - Restore terminal to normal mode

**Terminal Information:**
- `size(&self) -> (u16, u16)` - Get terminal size (width, height)

**Screen Capture:**
- `dump_screen(&self, path: &str)` - Write an ASCII (ANSI) dump of the whole screen
- `dump_region(&self, x, y, w, h, path: &str)` - Dump a rectangular region
- `save_screenshot_png(&self, path: &str)` - Render the screen to a PNG

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

**Extension Hooks:**
- `event_injector(&mut self) -> std::sync::mpsc::Sender<Event>` - A sender that queues events for `poll_event` as if typed; safe to use from another thread. Injected Ctrl+F12/F12 key events are served as captures and not returned by `poll_event`
- `set_capture_hook(&mut self, hook: CaptureHook)` - Handle Ctrl+F12 and F12 with `hook` instead of the built-in capture; replaces any earlier hook
- `clear_capture_hook(&mut self) -> Option<CaptureHook>` - Remove the capture hook, returning it
- `run_capture_hook(&mut self, kind: CaptureKind) -> bool` - Run the capture hook for `kind`; returns `false` when none is installed
- `write_raw(&mut self, data: &[u8]) -> io::Result<()>` - Send bytes straight to the terminal, bypassing the cell buffer, and flush (for protocols drawn outside the cells, such as tv-extensions' Kitty graphics)
- `CaptureKind` enum: `Png` (Ctrl+F12, an image of the screen), `Ansi` (F12, a text dump with ANSI colours)
- `CaptureHook` type alias: `Box<dyn FnMut(CaptureKind, &Terminal) + Send>`

Note: `terminal::InputParser` is exported unconditionally, with no `ssh` cargo
feature in core any more — it is public so that tv-extensions' SSH and
remote-input backends (or any other custom `Backend`) can convert raw bytes
into events.

---

## VIEWS MODULE

### View Trait (`src/views/view.rs`)

**Required:**
- `core(&self) -> &ViewCore` / `core_mut(&mut self) -> &mut ViewCore` - The base fields (Borland: `TView` data members)
- `draw(&mut self, terminal: &mut Terminal)` - Draw view
- `handle_event(&mut self, event: &mut Event)` - Handle event
- `get_palette(&self) -> Option<Palette>` - This view's palette
- `as_any(&self) -> &dyn Any` / `as_any_mut(&mut self) -> &mut dyn Any` - Downcasting

**Defaults reading the core:**
- `bounds()` / `set_bounds(Rect)`, `state() -> State` / `set_state(State)`, `options() -> Options` / `set_options(Options)`, `grow_mode() -> Grow` / `set_grow_mode(Grow)`, `set_palette_chain` / `get_palette_chain`
- `set_state_flag(State, bool)`, `get_state_flag(State) -> bool`, `is_focused()`, `has_shadow()`, `shadow_bounds()`

**Other defaults:**
- `can_focus() -> bool` (false), `set_focus(bool)`, `update_cursor(&Terminal)`, `zoom(Rect)`, `valid(CommandId) -> bool` (true)
- `idle(&mut self)` - Called on every idle tick for overlay widgets (no-op by default)
- `as_group() -> Option<&dyn GroupLike>` / `as_group_mut()` - The container interface, if the view is one (Borland: `dynamic_cast<TGroup*>`)
- `window_number()`, `label_link()`, `watches_pointer() -> bool` (false; a view that only watches the pointer, like `Tooltip`, sees mouse events without taking them from the views under it), `init_after_add()`, `constrain_to_parent_bounds()`, `set_owner_extent(Rect)`, `extent() -> Rect` (the view's own `(0,0,w,h)`; bounds are owner-relative), `get_redraw_union()`, `clear_move_tracking()`, `dump_to_file(..)`

A `Box<T: View>` is itself a `View`, so `add(Box::new(v))` and `add(v)` are both accepted.

#### ViewCore Struct
- `bounds: Rect`, `state: State`, `options: Options`, `grow_mode: Grow`, `palette_chain: Option<PaletteChainNode>`
- `ViewCore::new(bounds)`, `ViewCore::with_options(bounds, options)`

**Helper Functions:**
- `write_line_to_terminal(terminal: &mut Terminal, x: i16, y: i16, buf: &DrawBuffer)` - Draw line to terminal

---

### GroupLike Trait (`src/views/group.rs`)

`GroupLike: View` - Borland's `TGroup` behaviour as default methods over a `Group`.
- Required: `group(&self) -> &Group`, `group_mut(&mut self) -> &mut Group`
- Base implementations (callable as base calls): `group_draw`, `group_handle_event`, `group_set_bounds`, `group_update_cursor`, `group_valid`
- Modal loop: `execute(&mut self, app) -> CommandId`, `end_modal(CommandId)`, `end_state() -> CommandId`
- Children: `add(impl View) -> ViewId`, `add_boxed(Box<dyn View>)`, `add_typed(T) -> Handle<T>`, `get(Handle<T>) -> Option<&T>`, `get_mut(Handle<T>)`, `child_count()`, `child_at(i)`, `child_at_mut(i)`, `child_by_id(id)`, `child_by_id_mut(id)`, `remove_by_id(id)`, `set_initial_focus()`, `set_focus_to(i)`, `broadcast(&mut Event, Option<usize>)`

#### Handle Struct (`src/views/handle.rs`)
- `Handle<T: View>`: `Copy`; `from_id(ViewId)`, `id() -> ViewId`. A wrong `T` makes `get` return `None`.

---

### WindowLike Trait and `impl_view_for_window!` (`src/views/window.rs`)

`WindowLike: GroupLike` - Borland's `TWindow` behaviour as `window_*` default methods over a `Window`.
- Required: `window(&self) -> &Window`, `window_mut(&mut self) -> &mut Window`
- Base implementations: `window_set_bounds`, `window_draw`, `window_update_cursor`, `window_handle_event`, `window_set_focus`, `window_zoom`, `window_valid`, `window_get_palette`, `window_init_after_add`, `window_constrain_to_parent_bounds`, `window_set_owner_extent`
- `impl_view_for_window!(MyWindow)` generates `impl View for MyWindow` forwarding every method to the `window_*` body; `impl_view_for_window!(MyWindow { fn handle_event(..) { self.window_handle_event(event); .. } })` writes overrides inline, and the base stays reachable by its `window_*` name.

#### Shared Struct (`src/views/shared.rs`)
- `Shared<T: View>(Rc<RefCell<T>>)`: the one forwarding wrapper for a child the owner keeps calling; `new(rc)`, `inner() -> &Rc<RefCell<T>>`. Forwards `idle` too.

---

### Dialog Close Policy (`src/views/dialog.rs`)
- `CloseOn::Standard` (`CM_OK`, `CM_CANCEL`, `CM_YES`, `CM_NO`), `CloseOn::StandardAndButtons` (default: plus the dialog's own buttons), `CloseOn::Commands(Vec<CommandId>)`
- `DialogBuilder::close_on(CloseOn)`, `Dialog::set_close_on(CloseOn)`, `Dialog::close_on()`

---

### GroupBox (`src/views/group_box.rs`)

A titled single-line box drawn around related controls; it only draws (never focused). The controls are its siblings, added after it.
- `GroupBox::new(bounds: Rect, title: &str) -> Self` - `~` in the title is dropped
- `title(&self) -> &str`

---

### Application Hooks (`src/app/application.rs`)
- `AppHandler` trait: `pre_event(&mut self, app, event)`, `handle_command(&mut self, app, command, event) -> bool`, `idle(&mut self, app)`, `window_closed(&mut self, app, id)`; `Application::run_with(&mut handler)`; `run()` is `run_with(&mut ())`
- `Application::execute_modal(&mut view: impl WindowLike, tick: FnMut(&mut Application, &mut V) -> ModalTick) -> CommandId` - The single modal loop; `ModalTick::Continue` / `ModalTick::End(CommandId)`
- `Dialog::prepare_modal(&mut self, app: &mut Application)` - What `execute` does before its loop (modal state, centring, drag limits, first child focused); call it before `Application::execute_modal` when running the loop yourself
- `Application::add_overlay_widget(impl View)`, `Application::exec_view(impl View) -> CommandId`
- `Application::set_locale(&mut self, tag: &str) -> String`, `set_locale_from_env(&mut self) -> String`, `add_translations(&mut self, locale, source) -> Result<(), TranslationError>` - The `i18n` functions, with a redraw; `set_help_file` loads the locale's version of the file when it exists

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

Button view - clickable button with keyboard shortcuts and command dispatch.

- `set_press_animation(duration: Duration)` - Set how long a button pressed with Enter, Space or its hotkey stays pushed in before it sends its command.
- `press_animation() -> Duration` - How long a key-pressed button stays pushed in (see [`set_press_animation`]).
- `width_for_title(title: &str, min: i16) -> i16` - The width a button needs for `title` (without its `~` markers and surrounding spaces), at least `min`. Use it for translated titles.
- `struct Button`

**Button**
- `new(bounds: Rect, title: &str, command: CommandId, is_default: bool) -> Self`
- `is_default(&self) -> bool` - Whether this button was created as the dialog's default button (Borland: `TButton::amDefault`).
- `command(&self) -> CommandId` - The command this button emits when pressed.
- `is_broadcast(&self) -> bool` - Whether the button broadcasts its command to its siblings instead of emitting it as a command (see `set_broadcast`).
- `set_disabled(&mut self, disabled: bool)`
- `is_disabled(&self) -> bool`
- `set_broadcast(&mut self, broadcast: bool)` - Set whether this button broadcasts its command instead of sending it as a command event Matches Borland: bfBroadcast flag
- `set_selectable(&mut self, selectable: bool)` - Set whether this button is selectable (can receive focus) Matches Borland: ofSelectable flag
- `is_down(&self) -> bool` - Whether the button is currently drawn pushed in: held down with the mouse, or pressed from the keyboard less than [`press_animation`] ago.
- `struct ButtonBuilder` - Builder for creating buttons with a fluent API.

**ButtonBuilder**
- `new() -> Self` - Creates a new ButtonBuilder with default values.
- `bounds(mut self, bounds: Rect) -> Self` - Sets the button bounds (required).
- `title(mut self, title: impl Into<String>) -> Self` - Sets the button title text (required).
- `command(mut self, command: CommandId) -> Self` - Sets the command ID to dispatch when clicked (required).
- `default(mut self, is_default: bool) -> Self` - Sets whether this is the default button (optional, defaults to false).
- `build(self) -> Button` - Builds the Button.
**Public Methods:**
- `new(bounds: Rect, title: &str, command: CommandId, is_default: bool) -> Self` - Create button
- `set_disabled(&mut self, disabled: bool)` - Set disabled state
- `is_disabled(&self) -> bool` - Check if disabled
- `is_down(&self) -> bool` - Whether the button is drawn pushed in (held with the mouse, or during a key press's animation)
- Implements View trait

#### Press Animation
- `DEFAULT_PRESS_ANIMATION: Duration` - 100 ms
- `set_press_animation(duration: Duration)` - How long a key-pressed button shows pushed in before sending its command; `Duration::ZERO` sends it at once (per thread)
- `press_animation() -> Duration` - Current setting

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
- `set_error(&mut self, error: bool)` / `is_error(&self) -> bool` - Mark the label as naming an invalid field; it is drawn in red until cleared
- Implements View trait

---

### StaticText (`src/views/static_text.rs`)

#### StaticText Struct
**Public Methods:**
- `new(bounds: Rect, text: &str) -> Self` - Create static text
- `new_centered(bounds: Rect, text: &str) -> Self` - Create centered static text
- Implements View trait

---

### GroupBox (`src/views/group_box.rs`)

A titled single-line box drawn around related controls; it only draws (never focused). The controls are its siblings, added after it.
- `GroupBox::new(bounds: Rect, title: &str) -> Self` - `~` in the title is dropped
- `title(&self) -> &str`

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
- `focused_view_id(&self) -> Option<ViewId>` - The `ViewId` of the focused child, if any
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
- `end_state(&self) -> CommandId` - The command that ended the modal loop (0 while running)
- `end_modal(&mut self, command: CommandId)` - End the modal loop with `command`
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
- `get_selected_item(&self) -> Option<String>` - Get selected item text (owned; items can come from a `ListProvider`)
- `set_selection(&mut self, index: usize)` - Set selection
- `item_count(&self) -> usize` - Get item count (as of the last refresh)
- `set_provider(&mut self, provider: Box<dyn ListProvider>)` - Read items from a provider instead of an in-memory list
- `refresh_items(&mut self)` - Re-read the item count after a provider's source changed length
- `is_multi_select(&self) -> bool` / `set_multi_select(&mut self, multi: bool)` - Turn marking on or off
- `marked_items(&self) -> Vec<usize>` / `marked_text(&self) -> Vec<String>` - Marked indices, or their text (owned)
- `select_prev(&mut self)` - Select previous
- `select_next(&mut self)` - Select next
- `select_first(&mut self)` - Select first
- `select_last(&mut self)` - Select last
- `page_up(&mut self)` - Page up
- `page_down(&mut self)` - Page down
- Implements View & ListViewer traits

#### ListProvider Trait (`src/views/listbox.rs`)
**Public Methods:**
- `len(&self) -> usize` - Number of items
- `item(&self, index: usize) -> String` - Text of one item; only asked for indices below `len()`
- `is_empty(&self) -> bool` - Whether there are no items (default: `len() == 0`)

---

### SortedListBox (`src/views/sorted_listbox.rs`)

#### SortedListBox Struct
**Public Methods:**
- Similar to ListBox but maintains sorted order
- `new(bounds: Rect, on_select_command: CommandId) -> Self`
- Implements View & ListViewer traits

---

### Table (`src/views/table.rs`)

Not part of the Borland widget set. A scrollable grid with a header row and
sized columns; focus is a cell, not a row.

#### Table Struct
**Public Methods:**
- `new(bounds: Rect, on_select: CommandId) -> Self` - Create an empty table
- `set_columns(&mut self, columns: Vec<Column>)` / `columns(&self) -> &[Column]` - Replace or read the columns
- `set_rows(&mut self, rows: Vec<Vec<String>>)` - Replace the rows with an in-memory list
- `add_row(&mut self, row: Vec<String>)` - Append one row
- `clear_rows(&mut self)` - Drop every row, keeping the columns
- `set_provider(&mut self, provider: Box<dyn RowProvider>)` - Read rows from a provider instead of an in-memory list
- `refresh_rows(&mut self)` - Re-read the row count after a provider's source changed length
- `row_count(&self) -> usize` - Number of rows (as of the last refresh)
- `selected_row(&self) -> Option<usize>` / `selected_col(&self) -> usize` - Focused row and column
- `selected_cell(&self) -> Option<String>` - Text of the focused cell (owned; rows can come from a `RowProvider`)
- `set_selected_row(&mut self, row: usize)` / `set_selected_col(&mut self, col: usize)` - Move focus, clamped to range
- `set_show_header(&mut self, show: bool)` - Show or hide the header row (on by default)
- `set_separators(&mut self, on: bool)` / `separators(&self) -> bool` - Draw `SEPARATOR` between visible columns (off by default)
- `set_frozen_cols(&mut self, count: usize)` / `frozen_cols(&self) -> usize` - Keep the first `count` columns at the left while the rest scroll sideways, followed by `FROZEN_SEPARATOR` (none by default)
- `set_frozen_rows(&mut self, count: usize)` / `frozen_rows(&self) -> usize` - Keep the first `count` rows under the header while the rest scroll, the last one underlined (none by default)
- `set_on_select(&mut self, command: CommandId)` - Command emitted by Enter or a double-click
- `set_row_format(&mut self, row: usize, format: CellFormat)` / `set_column_format(&mut self, column: usize, format: CellFormat)` / `set_cell_format(&mut self, row: usize, column: usize, format: CellFormat)` - Colour and style a row, a column (header excluded) or one cell. Formats stack column, row, cell; the selected row keeps its colours and takes only the style
- `set_row_colors(&mut self, colors: Vec<Option<TvColor>>)` - Text colour of single rows, by index
- `clear_formats(&mut self)` - Remove every row, column and cell format
- Implements View trait

#### CellFormat Struct (`src/views/table.rs`)
Optional text colour, background colour and `Style` flags (bold, italic, underline, reverse, dim, strikethrough) for a row, column or cell. Unset fields keep the table's own colours, so a format follows the theme.

**Public Methods:**
- `new() -> Self` - A format that changes nothing
- `fg(self, TvColor) -> Self`, `bg(self, TvColor) -> Self`, `style(self, Style) -> Self` - Fluent setters

#### RowProvider Trait (`src/views/table.rs`)
**Public Methods:**
- `rows(&self) -> usize` - Number of rows
- `cell(&self, row: usize, col: usize) -> String` - Text of one cell; only asked for rows below `rows()` and columns the table has

#### TableBuilder Struct
**Public Methods:**
- `new() -> Self`, fluent `bounds`, `columns`, `rows`, `show_header`, `separators`, `frozen_cols`, `frozen_rows`, `on_select`
- `build(self) -> Table`

#### Constants
- `SEPARATOR: char` - The `│` drawn between visible columns when separators are on
- `FROZEN_SEPARATOR: char` - The `║` drawn after the last frozen column, separators on or not

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
- `is_valid(&self) -> bool` - Whether the text passes the validator, without the error message box `validate` shows
- `set_max_length(&mut self, max_length: usize)` - Change the limit; longer text is cut
- Input field with text editing capabilities
- Implements View trait

---

### Slider (`src/views/slider.rs`)

Not part of the Borland widget set; moved in from turbo-vision-extras. A
horizontal track with a thumb that picks one integer over `min..=max` — the
dragging counterpart of `Spinner`.

#### Slider Struct
**Public Methods:**
- `new(bounds: Rect, min: i64, max: i64) -> Self` - Create a slider over `min..=max`, starting at `min` (reversed bounds are swapped)
- `value(&self) -> i64` - The current value
- `set_value(&mut self, value: i64) -> bool` - Set the value, clamped to the range; returns whether it changed
- `range(&self) -> (i64, i64)` - The bounds, smallest first
- `set_step(&mut self, step: i64)` - How far Left and Right move (at least 1)
- `set_on_change(&mut self, command: CommandId)` - Broadcast a command whenever the user changes the value (0 turns it off)
- Implements View trait

#### Constants
- `TRACK: char` - The track character
- `THUMB: char` - The thumb character

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

EditorWindow view - advanced multi-line text editor with syntax highlighting support.

- `struct SearchOptions` - Search options flags (matching Borland's efXXX constants)

**SearchOptions**
- `new() -> Self`

**EditAction**
- `enum SelectionMode` - EditorWindow - Advanced multi-line text editor with undo/redo and find/replace
- `struct EditorWindow`

**EditorWindow**
- `new(bounds: Rect) -> Self` - Create a new editor control
- `with_scrollbars(bounds: Rect, h_scrollbar: Option<Rc<RefCell<ScrollBar>>>, v_scrollbar: Option<Rc<RefCell<ScrollBar>>>, indicator: Option<Rc<RefCell<Indicator>>>) -> Self` - Create with scrollbars and indicator (Borland style) Matches Borland: TEditor receives pointers to scrollbars/indicator created by parent
- `set_read_only(&mut self, read_only: bool)` - Set read-only mode
- `set_tab_size(&mut self, tab_size: usize)` - Set tab size
- `set_auto_indent(&mut self, auto_indent: bool)` - Set auto-indent mode
- `set_highlighter(&mut self, highlighter: Box<dyn SyntaxHighlighter>)` - Set syntax highlighter
- `clear_highlighter(&mut self)` - Clear syntax highlighter (use plain text)
- `has_highlighter(&self) -> bool` - Check if syntax highlighting is enabled
- `toggle_insert_mode(&mut self)` - Toggle insert/overwrite mode
- `get_text(&self) -> String` - Get the text content
- `set_text(&mut self, text: &str)` - Set the text content
- `is_modified(&self) -> bool` - Check if text has been modified
- `clear_modified(&mut self)` - Clear the modified flag
- `line_count(&self) -> usize` - Get current line count
- `get_delta(&self) -> Point` - Get the current scroll offset (top-left visible position).
- `cursor(&self) -> Point` - Cursor position in document coordinates (0-based line, 0-based character column).
- `scroll_to_line(&mut self, line: usize)` - Scroll the editor so that the given 0-based line is visible, moving the cursor to the beginning of that line.
- `max_line_width(&self) -> usize` - Get the maximum line width (length of the longest line)
- `needs_vertical_scrollbar(&self) -> bool` - Check if vertical scrollbar is needed
- `needs_horizontal_scrollbar(&self) -> bool` - Check if horizontal scrollbar is needed
- `load_file(&mut self, path: impl AsRef<std::path::Path>) -> std::io::Result<()>` - Load file contents into the editor Matches Borland's TFileEditor::load()
- `save_file(&mut self) -> std::io::Result<()>` - Save editor contents to the associated filename Matches Borland's TFileEditor::save()
- `save_as(&mut self, path: impl AsRef<std::path::Path>) -> std::io::Result<()>` - Save editor contents to a specific filename Matches Borland's TFileEditor::saveAs()
- `set_backup_files(&mut self, backup: bool)` - Enable or disable Borland-style .bak backups on save.
- `get_filename(&self) -> Option<&str>` - Get the current filename, if any
- `undo(&mut self)` - Undo the last action
- `redo(&mut self)` - Redo the last undone action
- `can_undo(&self) -> bool` - True when the undo stack has at least one entry.
- `can_redo(&self) -> bool` - True when the redo stack has at least one entry.
- `find(&mut self, text: &str, options: SearchOptions) -> Option<Point>` - Find text in the editor with options Matches Borland's TEditor::search() (teditor.cc:917-949)
- `find_next(&mut self) -> Option<Point>` - Find next occurrence of last search Matches Borland's cmSearchAgain command
- `replace_selection(&mut self, replace_text: &str) -> bool` - Replace current selection with new text Returns true if replacement was made
- `replace_next(&mut self, find_text: &str, replace_text: &str, options: SearchOptions) -> bool` - Replace next occurrence of find_text with replace_text Matches Borland's TEditor::doSearchReplace() with efDoReplace
- `replace_all(&mut self, find_text: &str, replace_text: &str, options: SearchOptions) -> usize` - Replace all occurrences of find_text with replace_text Matches Borland's TEditor::doSearchReplace() with efReplaceAll
- `sync_from_scrollbars(&mut self)` - Sync editor cursor from scrollbar values and ensure it's visible.
- `has_selection(&self) -> bool`
- `select_all(&mut self)`
- `delete_selection(&mut self)`
- `clip_copy(&mut self) -> bool` - Copy selection to clipboard Matches Borland: TEditor::clipCopy()
- `clip_cut(&mut self) -> bool` - Cut selection to clipboard (copy + delete) Matches Borland: TEditor::clipCut()
- `clip_paste(&mut self) -> bool` - Paste from clipboard Matches Borland: TEditor::clipPaste()

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
- `end_state(&self) -> CommandId` - The command that ended the modal loop (0 while running)
- `end_modal(&mut self, command: CommandId)` - End the modal loop with `command`
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
- `execute(&mut self, app: &mut Application) -> CommandId` - Run the dialog modally; returns the command that closed it
- `prepare_modal(&mut self, app: &mut Application)` - The setup `execute` does first; for callers running `Application::execute_modal` themselves
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

`FileEditorWindow` — `EditWindow` plus file binding (load/save) and

- `struct FileEditorWindow`

**FileEditorWindow**
- `new(bounds: Rect, title: &str) -> Self`
- `refresh_title(&mut self)` - Update the window's title bar from the current filename.
- `set_text(&mut self, text: &str)`
- `edit_window(&self) -> &EditWindow`
- `edit_window_mut(&mut self) -> &mut EditWindow`
- `struct FileEditorBuilder` - Builder for creating file editors with a fluent API.

**FileEditorBuilder**
- `new() -> Self`
- `bounds(mut self, bounds: Rect) -> Self`
- `title(mut self, title: &str) -> Self`
- `build(self) -> FileEditorWindow`
- `build_boxed(self) -> Box<FileEditorWindow>`

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

PictureValidator - validates and formats input using picture mask patterns.

- `enum PicResult` - Result codes of the picture state machine.

**PicMachine**
- `struct PictureValidator` - Picture mask validator for formatted input.

**PictureValidator**
- `new(mask: &str) -> Self` - Create a new picture validator with the given mask (auto-fill on).
- `new_no_format(mask: &str) -> Self` - Create a new picture validator without auto-fill.
- `mask(&self) -> &str` - Get the mask string
- `set_auto_format(&mut self, auto_format: bool)` - Set whether to auto-fill literals while typing
- `picture(&self, input: &str, auto_fill: bool) -> (PicResult, String)` - Runs the picture machine over `input`.
- `picture_validator(mask: &str) -> ValidatorRef` - Helper function to create a ValidatorRef for a PictureValidator
- `struct PictureValidatorBuilder` - Builder for creating picture validators with a fluent API.

**PictureValidatorBuilder**
- `new() -> Self` - Creates a new PictureValidatorBuilder with default values.
- `mask(mut self, mask: impl Into<String>) -> Self` - Sets the picture mask pattern (required).
- `auto_format(mut self, auto_format: bool) -> Self` - Sets whether to auto-fill literal characters (default: true).
- `build(self) -> PictureValidator` - Builds the PictureValidator.
- `build_ref(self) -> ValidatorRef` - Builds the PictureValidator as a ValidatorRef.

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

### CheckBoxes and RadioButtons (`src/views/cluster_group.rs`)

CheckBoxes and RadioButtons - one focusable control holding several items.

Both types (made by one macro) share:
- `new(bounds: Rect, labels: Vec<String>) -> Self` - A cluster of the given items.
- `set_labels(&mut self, labels: Vec<String>)`, `item_count(&self) -> usize`
- `value(&self) -> u32`, `set_value(&mut self, value: u32)` - The items as bits (CheckBoxes) or the selected index (RadioButtons).
- `focused_item(&self) -> usize`, `set_focused_item(&mut self, index: usize)`
- `set_enabled(&mut self, index: usize, enabled: bool)`, `is_enabled(&self, index: usize) -> bool`
- `set_on_change(&mut self, command: CommandId)` - Broadcast `command` to the sibling views when the value changes.

**CheckBoxes**
- `is_checked(&self, index: usize) -> bool` - Whether one box is ticked.
- `set_checked(&mut self, index: usize, checked: bool)` - Tick or untick one box.
- `checked_items(&self) -> Vec<usize>` - The ticked boxes, in order.

**RadioButtons**
- `selected(&self) -> Option<usize>` - Index of the selected button, or `None` when the cluster is empty.
- `set_selected(&mut self, index: usize)` - Select one button.
- `selected_label(&self) -> Option<&str>` - Text of the selected button.

---

### ComboBox (`src/views/combo_box.rs`)

ComboBox view - a text field showing one choice, with a drop-down list.

- `struct ComboState` - The items and current choice of one combo box.

**ComboState**
- `selected_text(&self) -> Option<&str>` - Text of the current choice, if any.
- `lookup(id: u16) -> Option<Rc<RefCell<ComboState>>>` - Look up a registered combo state by id.
- `struct ComboBox` - A field showing one choice, with a drop-down list of the alternatives.

**ComboBox**
- `new(bounds: Rect, id: u16) -> Self` - Create an empty combo box registered under `id`.
- `with_items(bounds: Rect, id: u16, items: Vec<String>) -> Self` - Create a combo box already holding `items`, with the first selected.
- `id(&self) -> u16` - Registration id, as passed to [`ComboBox::new`].
- `state(&self) -> Rc<RefCell<ComboState>>` - Shared state, for callers that want to read the choice later without holding on to the control.
- `set_items(&mut self, items: Vec<String>)` - Replace the item list.
- `add_item(&mut self, item: impl Into<String>)` - Append one item.
- `item_count(&self) -> usize` - Number of items in the list.
- `selected(&self) -> Option<usize>` - Index of the current choice.
- `set_selected(&mut self, index: Option<usize>)` - Set the current choice.
- `selected_text(&self) -> Option<String>` - Text of the current choice.
- `set_on_change(&mut self, command: CommandId)` - Command broadcast when the choice changes.
- `struct DropdownWindow` - The modal list a combo box drops down.

**DropdownWindow**
- `new(state: Rc<RefCell<ComboState>>, screen: Rect) -> Self` - Build the popup for `state`, placed just under its field.
- `execute(&mut self, terminal: &mut Terminal) -> Option<usize>` - Run the popup modally.
- `struct ComboBoxBuilder` - Builder for creating combo boxes with a fluent API.

**ComboBoxBuilder**
- `new() -> Self`
- `bounds(mut self, bounds: Rect) -> Self`
- `id(mut self, id: u16) -> Self`
- `items<I: Into<String>>(mut self, items: impl IntoIterator<Item = I>) -> Self`
- `selected(mut self, index: usize) -> Self`
- `on_change(mut self, command: CommandId) -> Self`
- `build(self) -> ComboBox`
- `build_boxed(self) -> Box<ComboBox>`

---

### Spinner (`src/views/spinner.rs`)

Spinner view - a numeric field with up and down steppers.

- `struct Spinner` - A numeric field with up and down steppers.

**Spinner**
- `new(bounds: Rect, min: i64, max: i64) -> Self` - Create a spinner over the inclusive range `min..=max`, starting at `min`.
- `value(&self) -> i64` - Current value, always within the range.
- `set_value(&mut self, value: i64) -> bool` - Set the value, clamped to the range.
- `range(&self) -> (i64, i64)` - The inclusive range.
- `set_range(&mut self, min: i64, max: i64)` - Set the range, swapping a reversed one, and re-clamp the value.
- `set_step(&mut self, step: i64)` - Amount one Up or Down press moves the value.
- `set_suffix(&mut self, suffix: impl Into<String>)` - Text shown after the number, such as `"%"` or `" ms"`.
- `set_wrap(&mut self, wrap: bool)` - Whether stepping past an end continues from the other end.
- `set_on_change(&mut self, command: CommandId)` - Command broadcast when the value changes.
- `step_up(&mut self) -> bool` - Step up by one step.
- `step_down(&mut self) -> bool` - Step down by one step.
- `struct SpinnerBuilder` - Builder for creating spinners with a fluent API.

**SpinnerBuilder**
- `new() -> Self`
- `bounds(mut self, bounds: Rect) -> Self`
- `range(mut self, min: i64, max: i64) -> Self`
- `value(mut self, value: i64) -> Self`
- `step(mut self, step: i64) -> Self`
- `suffix(mut self, suffix: impl Into<String>) -> Self`
- `wrap(mut self, wrap: bool) -> Self`
- `on_change(mut self, command: CommandId) -> Self`
- `build(self) -> Spinner`
- `build_boxed(self) -> Box<Spinner>`

---

### ProgressBar (`src/views/progress_bar.rs`)

ProgressBar view - determinate and indeterminate progress indicator.

- `enum ProgressMode` - Fill mode of a [`ProgressBar`].
- `enum ProgressStyle` - Character set used to draw the track.

**ProgressStyle**
- `struct ProgressBar` - A horizontal progress indicator.

**ProgressBar**
- `new(bounds: Rect, max: u64) -> Self` - Create a determinate bar with the given upper bound.
- `value(&self) -> u64` - Current value, always within `0..=max`.
- `set_value(&mut self, value: u64)` - Set the current value.
- `advance(&mut self, delta: u64)` - Add to the current value, clamping at `max` and saturating on overflow.
- `max(&self) -> u64` - Upper bound of the bar.
- `set_max(&mut self, max: u64)` - Set the upper bound.
- `reset(&mut self)` - Reset the value to zero and rewind the marquee.
- `fraction(&self) -> f64` - Completion as a fraction in `0.0..=1.0`.
- `percent(&self) -> u32` - Completion as a whole percentage in `0..=100`, truncated.
- `mode(&self) -> ProgressMode` - Current mode.
- `set_mode(&mut self, mode: ProgressMode)` - Switch between determinate and marquee display.
- `set_style(&mut self, style: ProgressStyle)` - Set the glyphs used to draw the track.
- `show_percent(&mut self)` - Overlay the truncated percentage on the track.
- `set_show_percent(&mut self, show: bool)` - Turn the percentage overlay on or off.
- `is_percent_shown(&self) -> bool` - Whether the percentage overlay is currently shown.
- `set_caption(&mut self, text: impl Into<String>)` - Overlay fixed text on the track instead of a percentage.
- `hide_caption(&mut self)` - Draw the track with nothing overlaid.
- `tick(&mut self)` - Advance the marquee by one cell, reversing at either end.
- `set_tick_interval(&mut self, interval: Duration)` - Set how often the idle animation steps the marquee.
- `struct ProgressBarBuilder` - Builder for creating progress bars with a fluent API.

**ProgressBarBuilder**
- `new() -> Self`
- `bounds(mut self, bounds: Rect) -> Self`
- `max(mut self, max: u64) -> Self`
- `marquee(mut self) -> Self`
- `style(mut self, style: ProgressStyle) -> Self`
- `caption(mut self, text: impl Into<String>) -> Self`
- `show_percent(mut self, show: bool) -> Self` - Show or hide the percentage overlay.
- `no_caption(mut self) -> Self`
- `build(self) -> ProgressBar`
- `build_boxed(self) -> Box<ProgressBar>`

---

### TabbedPane (`src/views/tabbed_pane.rs`)

TabbedPane view - a strip of tabs over a stack of pages.

**Tab**
- `struct TabbedPane` - A strip of tabs over a stack of pages.

**TabbedPane**
- `new(bounds: Rect) -> Self` - Create an empty pane.
- `page_area(&self) -> Rect` - The rect a page occupies: everything under the tab strip.
- `add_page(&mut self, title: &str, page: Group) -> usize` - Add a page under `title`.
- `page_count(&self) -> usize` - Number of tabs.
- `active(&self) -> usize` - Index of the active tab.
- `active_title(&self) -> Option<&str>` - Title of the active tab, tildes included, or `None` when there are no tabs.
- `set_active(&mut self, index: usize) -> bool` - Show a page.
- `page_mut(&mut self, index: usize) -> Option<&mut Group>` - The active page, for adding controls after construction.
- `active_page_mut(&mut self) -> Option<&mut Group>` - The active page.
- `set_initial_focus(&mut self)` - Give the active page's first focusable control the focus.
- `struct TabbedPaneBuilder` - Builder for creating tabbed panes with a fluent API.

**TabbedPaneBuilder**
- `new() -> Self`
- `bounds(mut self, bounds: Rect) -> Self`
- `page(mut self, title: impl Into<String>) -> Self` - Add an empty page under `title`.
- `active(mut self, index: usize) -> Self`
- `build(self) -> TabbedPane`
- `build_boxed(self) -> Box<TabbedPane>`

---

### SplitPane (`src/views/split_pane.rs`)

SplitPane view - two panes divided by a draggable splitter.

- `enum Orientation` - Which way the two panes sit relative to each other.
- `struct SplitPane` - Two panes divided by a draggable splitter.

**SplitPane**
- `new(bounds: Rect, orientation: Orientation, position: i16) -> Self` - Create a split pane with empty halves and the divider `position` cells in.
- `set_panes(&mut self, first: Group, second: Group)` - Install both halves, moving each into place.
- `first_mut(&mut self) -> &mut Group` - The first half: the left pane, or the top one.
- `second_mut(&mut self) -> &mut Group` - The second half: the right pane, or the bottom one.
- `orientation(&self) -> Orientation` - Which way the panes sit.
- `position(&self) -> i16` - Cells currently given to the first pane.
- `set_position(&mut self, position: i16) -> bool` - Move the divider, clamped so neither half falls below its minimum.
- `set_minimums(&mut self, first: i16, second: i16)` - Set the smallest size each half may be squeezed to, then re-clamp the divider.
- `grow_first(&mut self) -> bool` - Give the first pane one more cell.
- `shrink_first(&mut self) -> bool` - Give the first pane one fewer cell.
- `second_focused(&self) -> bool` - Whether the second half currently holds the focus.
- `focus_other(&mut self)` - Move the focus to the other half.
- `set_initial_focus(&mut self)` - Give the focused half's first control the focus.
- `first_area(&self) -> Rect` - The rect the first half occupies.
- `second_area(&self) -> Rect` - The rect the second half occupies.
- `divider_area(&self) -> Rect` - The divider's own rect: one cell thick, spanning the pane.
- `struct SplitPaneBuilder` - Builder for creating split panes with a fluent API.

**SplitPaneBuilder**
- `new() -> Self`
- `bounds(mut self, bounds: Rect) -> Self`
- `orientation(mut self, orientation: Orientation) -> Self`
- `position(mut self, position: i16) -> Self` - Cells given to the first pane.
- `minimums(mut self, first: i16, second: i16) -> Self`
- `build(self) -> SplitPane`
- `build_boxed(self) -> Box<SplitPane>`

---

### Tooltip (`src/views/tooltip.rs`)

Tooltip view - hover hints for the controls of a dialog.

- `struct Tooltip` - Hover hints for the controls of a dialog.

**Tooltip**
- `new(bounds: Rect) -> Self` - Create an empty tooltip that may draw anywhere within `bounds`.
- `add_hint(&mut self, target: Rect, text: impl Into<String>)` - Register a hint for the control occupying `target`.
- `hint_count(&self) -> usize` - Number of registered hints.
- `clear_hints(&mut self)` - Drop every hint and hide anything showing.
- `set_delay(&mut self, delay: Duration)` - How long the pointer must rest before a hint appears.
- `is_showing(&self) -> bool` - Whether a hint is on screen.
- `shown_text(&self) -> Option<&str>` - The text currently on screen, if any.
- `hide(&mut self)` - Hide whatever is showing and forget the hover.

Added last, it covers the controls, yet clicks and moves still reach them (`watches_pointer`).
- `struct TooltipBuilder` - Builder for creating tooltips with a fluent API.

**TooltipBuilder**
- `new() -> Self`
- `bounds(mut self, bounds: Rect) -> Self`
- `hint(mut self, target: Rect, text: impl Into<String>) -> Self`
- `delay(mut self, delay: Duration) -> Self`
- `build(self) -> Tooltip`
- `build_boxed(self) -> Box<Tooltip>`

---

### Outline (`src/views/outline.rs`)

Outline view - generic hierarchical tree control for displaying expandable/collapsible data.

- `struct Node` - A node in the tree Matches Borland: TNode

**Node**
- `new(data: T) -> Self` - Create a new leaf node (no children)
- `with_children(data: T, children: Vec<Rc<RefCell<Node<T>>>>) -> Self` - Create a new node with children
- `add_child(&mut self, child: Rc<RefCell<Node<T>>>)` - Add a child node
- `has_children(&self) -> bool` - Check if this node has children
- `toggle(&mut self)` - Toggle expanded state

**DisplayNode**
- `struct OutlineViewer` - OutlineViewer - displays a hierarchical tree of nodes Matches Borland: TOutlineViewer

**OutlineViewer**
- `new<F>(bounds: Rect, format_fn: F) -> Self where F: Fn(&T) -> String + 'static,` - Create a new outline viewer format_fn converts node data to display string
- `set_roots(&mut self, roots: Vec<Rc<RefCell<Node<T>>>>)` - Set the root nodes of the tree
- `add_root(&mut self, root: Rc<RefCell<Node<T>>>)` - Add a root node
- `selected_node(&self) -> Option<Rc<RefCell<Node<T>>>>` - Get the currently selected node

---

### Message boxes (`src/views/msgbox.rs`)

MsgBox - message box utilities for displaying alerts and confirmations.

- `message_box(app: &mut Application, message: &str, options: MsgBox) -> CommandId` - Display a message box with the given message and options.
- `message_box_rect(app: &mut Application, bounds: Rect, message: &str, options: MsgBox) -> CommandId` - Display a message box at a specific location
- `message_box_ok(app: &mut Application, message: &str) -> CommandId` - Display a simple message box with OK button
- `message_box_error(app: &mut Application, message: &str) -> CommandId` - Display an error message box with OK button
- `message_box_warning(app: &mut Application, message: &str) -> CommandId` - Display a warning message box with OK button
- `confirmation_box(app: &mut Application, message: &str) -> CommandId` - Display a confirmation dialog with Yes/No/Cancel buttons
- `confirmation_box_yes_no(app: &mut Application, message: &str) -> CommandId` - Display a confirmation dialog with Yes/No buttons
- `confirmation_box_ok_cancel(app: &mut Application, message: &str) -> CommandId` - Display a confirmation dialog with OK/Cancel buttons
- `input_box(app: &mut Application, title: &str, label: &str, initial: &str, max_length: usize) -> Option<String>` - Display an input box that prompts the user for a string
- `input_box_rect(app: &mut Application, bounds: Rect, title: &str, label: &str, initial: &str, max_length: usize) -> Option<String>` - Display an input box at a specific location
- `search_box(app: &mut Application, title: &str) -> Option<String>` - Display a search dialog that prompts the user for search text
- `search_replace_box(app: &mut Application, title: &str) -> Option<(String, String)>` - Display a search and replace dialog that prompts for find and replace text
- `goto_line_box(app: &mut Application, title: &str) -> Option<usize>` - Display a goto line dialog that prompts for a line number

---

### ColorDialog (`src/views/color_dialog.rs`)

Color Dialog - dialog for selecting foreground and background colors

- `struct ColorDialog` - Color Dialog Matches Borland: TColorDialog (simplified implementation)

**ColorDialog**
- `new(bounds: Rect, title: &str, initial_attr: Attr) -> Self` - Create a new color dialog
- `execute(&mut self, app: &mut crate::app::Application) -> Option<Attr>` - Execute the dialog modally
- `get_selected_attr(&self) -> Option<Attr>` - Get the selected color attribute
- `struct ColorDialogBuilder` - Builder for creating color dialogs with a fluent API.

**ColorDialogBuilder**
- `new() -> Self` - Creates a new ColorDialogBuilder with default values.
- `bounds(mut self, bounds: Rect) -> Self` - Sets the color dialog bounds (required).
- `title(mut self, title: impl Into<String>) -> Self` - Sets the dialog title (required).
- `initial_attr(mut self, attr: Attr) -> Self` - Sets the initial color attribute (default: White on Black).
- `build(self) -> ColorDialog` - Builds the ColorDialog.
- `build_boxed(self) -> Box<ColorDialog>` - Builds the ColorDialog as a Box.

---

### ColorSelector (`src/views/color_selector.rs`)

Color Selector - interactive color picker control

- `struct ColorSelector` - Color Selector - interactive color picker Matches Borland: TColorSelector

**ColorSelector**
- `new(bounds: Rect) -> Self` - Create a new color selector
- `with_shared(bounds: Rect, selected: Rc<RefCell<u8>>) -> Self` - Create a color selector whose selection is shared with the caller
- `get_selected_color(&self) -> u8` - Get the selected color
- `set_selected_color(&mut self, color: u8)` - Set the selected color
- `struct ColorSelectorBuilder` - Builder for creating color selectors with a fluent API.

**ColorSelectorBuilder**
- `new() -> Self`
- `bounds(mut self, bounds: Rect) -> Self`
- `selected_color(mut self, color: u8) -> Self`
- `build(self) -> ColorSelector`
- `build_boxed(self) -> Box<ColorSelector>`

---

### ChDirDialog (`src/views/chdir_dialog.rs`)

Change Directory Dialog - specialized dialog for directory selection

**SharedDirListBox**
- `struct ChDirDialog` - Change Directory Dialog

**ChDirDialog**
- `new(history_id: Option<u16>) -> Self` - Create a new change directory dialog
- `execute(&mut self, app: &mut Application) -> Option<PathBuf>` - History is automatically updated on success
- `get_directory(&self) -> Option<PathBuf>` - Get the selected directory
- `end_state(&self) -> CommandId` - The command that closed the dialog (0 while it runs)
- `struct ChDirDialogBuilder` - Builder for creating change directory dialogs with a fluent API.

**ChDirDialogBuilder**
- `new() -> Self` - Creates a new ChDirDialogBuilder
- `history_id(mut self, history_id: u16) -> Self` - Sets a custom history ID for directory history
- `build(self) -> ChDirDialog` - Builds the ChDirDialog with Borland standard layout
- `build_boxed(self) -> Box<ChDirDialog>` - Builds the ChDirDialog as a Box

---

### HelpIndex (`src/views/help_index.rs`)

Help Index - searchable index of help topics

- `struct HelpIndex` - Help Index - searchable topic list Matches Borland: THelpIndex

**HelpIndex**
- `new(bounds: Rect, title: &str, help_file: Rc<RefCell<HelpFile>>) -> Self` - Create a new help index dialog
- `execute(&mut self, app: &mut crate::app::Application) -> Option<String>` - Execute the dialog modally Returns the selected topic ID if View was pressed, None if closed
- `get_selected_topic(&self) -> Option<String>` - Get the selected topic ID
- `struct HelpIndexBuilder` - Builder for creating help index dialogs with a fluent API.

**HelpIndexBuilder**
- `new() -> Self`
- `bounds(mut self, bounds: Rect) -> Self`
- `title(mut self, title: impl Into<String>) -> Self`
- `help_file(mut self, help_file: Rc<RefCell<HelpFile>>) -> Self`
- `build(self) -> HelpIndex`
- `build_boxed(self) -> Box<HelpIndex>`

---

### HelpToc (`src/views/help_toc.rs`)

Help Table of Contents - hierarchical topic browser

- `struct HelpToc` - Help Table of Contents Matches Borland: THelpToc

**HelpToc**
- `new(bounds: Rect, title: &str, help_file: Rc<RefCell<HelpFile>>) -> Self` - Create a new help table of contents dialog
- `execute(&mut self, app: &mut crate::app::Application) -> Option<String>` - Execute the dialog modally Returns the selected topic title if View was pressed, None if closed
- `get_selected_topic(&self) -> Option<String>` - Get the selected topic
- `struct HelpTocBuilder` - Builder for creating help TOC dialogs with a fluent API.

**HelpTocBuilder**
- `new() -> Self`
- `bounds(mut self, bounds: Rect) -> Self`
- `title(mut self, title: impl Into<String>) -> Self`
- `help_file(mut self, help_file: Rc<RefCell<HelpFile>>) -> Self`
- `build(self) -> HelpToc`
- `build_boxed(self) -> Box<HelpToc>`

---

### Editor and FileEditor traits (`src/views/editor_traits.rs`)

Window-level editor contracts that mirror Borland's TEditor / TFileEditor

- `enum ExternalState` - Result of probing the on-disk file behind a [`FileEditor`].

**trait Editor** - Window-level editor contract.
- `valid_close(&mut self, _app: &mut Application, _command: CommandId) -> bool` - Called by the event loop when this editor's frame requests a close.
- `undo(&mut self)` - Pop the most recent edit from the undo stack and revert it.
- `redo(&mut self)` - Re-apply the most recent undone edit.
- `can_undo(&self) -> bool` - True when the undo stack has at least one entry.
- `can_redo(&self) -> bool` - True when the redo stack has at least one entry.
- `cut(&mut self) -> bool` - Cut the current selection to the clipboard.
- `copy(&mut self) -> bool` - Copy the current selection to the clipboard.
- `paste(&mut self) -> bool` - Insert the clipboard contents at the cursor (replacing any active selection).
- `select_all(&mut self)` - Select the entire buffer.
- `clear_selection(&mut self)` - Delete the current selection without copying it to the clipboard.
- `has_selection(&self) -> bool` - True when there's a non-empty selection.

**trait FileEditor** - Editor that is backed by an on-disk file.
- `file_path(&self) -> Option<PathBuf>` - Current file path.
- `set_file_path(&mut self, path: Option<PathBuf>)`
- `is_dirty(&self) -> bool` - True when in-memory contents differ from the on-disk file (or no file yet).
- `save(&mut self) -> std::io::Result<()>`
- `save_as(&mut self, path: PathBuf) -> std::io::Result<()>`
- `load(&mut self, path: PathBuf) -> std::io::Result<()>`
- `new_buffer(&mut self)` - Reset to an empty Untitled buffer (no file path, no breakpoints, clean).
- `last_known_mtime(&self) -> Option<SystemTime>` - mtime recorded the last time the editor read or wrote the file.
- `poll_external_changes(&self) -> ExternalState` - Probe the on-disk file and report whether it has changed since the last load/save.
- `reload(&mut self) -> std::io::Result<()>` - Re-read the file from disk into the buffer and refresh the last-known mtime.
- `display_name(&self) -> String` - Friendly name used in dialogs and titles.
- `prompt_save_as(&mut self, app: &mut Application) -> bool` - Show a Save As dialog and persist the buffer to the chosen path.
- `confirm_save_on_close<E: FileEditor + ?Sized>(editor: &mut E, app: &mut Application, command: CommandId) -> bool` - Standard "do you want to save?" dialog used by [`FileEditor`] implementors from their [`Editor::valid_close`] override.

---

## APPLICATION MODULE

### Application (`src/app/application.rs`)

Application structure and event loop implementation.

- `struct Application` - public fields: `terminal: Terminal`, `menu_bar: Option<MenuBar>`, `status_line: Option<StatusLine>`, `desktop: Desktop`, `running: bool`

**trait AppHandler** - Application-level hooks for [`Application::run_with`].
- `pre_event(&mut self, _app: &mut Application, _event: &mut Event)` - Called before the menu bar, status line and desktop see the event.
- `handle_command(&mut self, _app: &mut Application, _command: CommandId, _event: &Event) -> bool` - Called after the desktop has seen the event and only if it is still a `Command`.
- `idle(&mut self, _app: &mut Application)` - Called on each idle tick, after [`Application::idle`].
- `window_closed(&mut self, _app: &mut Application, _id: ViewId)` - Called once for every window the desktop removed after `State::CLOSED`.
- `enum ModalTick` - What a modal loop's per-tick hook wants to happen next; see [`Application::execute_modal`].

**Application**
- `new() -> Result<Self>` - Creates a new application instance and initializes the terminal.
- `with_terminal(terminal: Terminal) -> Self` - Creates an application on an already-built terminal, e.g.
- `is_host_driven(&self) -> bool` - Whether an embedder steps this application with [`step`](Self::step) instead of it owning a terminal and an event loop, as its backend's [`is_host_driven`](crate::terminal::Backend::is_host_driven) said when the application was built.
- `menu_is_open(&self) -> bool` - Whether the menu bar has a submenu dropped down.
- `set_menu_bar(&mut self, menu_bar: MenuBar)`
- `set_status_line(&mut self, status_line: StatusLine)`
- `add_overlay_widget<V: View + 'static>(&mut self, widget: V)` - Add an overlay widget that needs idle processing and is drawn on top of everything These widgets continue to animate even during modal dialogs Matches Borland: TProgram::idle() continues running during execView()
- `needs_redraw(&mut self)` - Request a full redraw on the next frame Call this after changing the palette or other global settings
- `handle_redraw(&mut self)` - Handle a full screen redraw (terminal resize, palette change, etc.).
- `set_palette(&mut self, palette: Option<Vec<u8>>)` - Set a custom application palette and automatically trigger redraw if changed Pass None to reset to the default Borland palette
- `put_event(&mut self, event: Event)` - Queue an event to be returned before the next terminal poll.
- `poll_event_or_quit(&mut self) -> Option<Event>` - The next event: the one queued by [`put_event`](Self::put_event) if there is one, otherwise whatever the terminal's backend returns within 20 ms.
- `get_event(&mut self) -> Option<Event>` - Get an event (with drawing) Matches Borland/Magiblot: TProgram::getEvent() (tprogram.cc:105-174) This is called by modal views' execute() methods.
- `exec_view<V: View + 'static>(&mut self, view: V) -> CommandId` - Execute a view (modal or modeless) Matches Borland: TProgram::execView() (tprogram.cc:177-197)
- `execute_modal<V, F>(&mut self, view: &mut V, tick: F) -> CommandId where V: WindowLike + ?Sized, F: FnMut(&mut Application, &mut V) -> ModalTick,` - Run `view` modally: the single modal loop behind `Dialog::execute`, `FileDialog::execute` and `HelpWindow::execute` (Borland: `TGroup::execute`, with `TProgram::getEvent` drawing the screen).
- `run(&mut self)` - Run the event loop with no application hooks; see [`run_with`](Self::run_with).
- `run_with<H: AppHandler>(&mut self, handler: &mut H)` - Run the event loop, giving `handler` a chance before and after each event, on idle, and when a window closes.
- `step<H: AppHandler>(&mut self, handler: &mut H, event: Option<Event>)` - One pass of the event loop around an event the caller already has, for an embedder that steps the application itself instead of calling [`run_with`](Self::run_with).
- `draw(&mut self)`
- `handle_event(&mut self, event: &mut Event)`
- `take_screenshot(&mut self)` - Save a PNG screenshot of the current screen (bound to Ctrl+F12).
- `dump_screen_ansi(&mut self)` - Save an ASCII (ANSI-colored) dump of the whole screen (bound to F12).
- `set_help_file(&mut self, path: &str) -> std::io::Result<()>` - Set the help file for F1 context-sensitive help Matches Borland: TApplication::helpFile initialization
- `set_help(&mut self, help_file: HelpFile)` - Set a pre-built help file for F1 context-sensitive help
- `register_help_context(&mut self, context_id: u16, topic_id: &str)` - Register a help context mapping (context ID to topic ID) This allows views to have help_context set, and F1 will open the corresponding topic
- `show_help_topic(&mut self, topic_id: &str)` - Show help for a specific topic Opens the help window and displays the given topic
- `show_help(&mut self)` - Show context-sensitive help Looks up the focused view's help context and opens the appropriate topic Matches Borland: TProgram::getEvent() F1 handling
- `tile(&mut self)` - Tile all tileable windows in a grid pattern Matches Borland: TApplication::tile() (tapplica.cpp:123-127)
- `cascade(&mut self)` - Cascade all tileable windows in a staircase pattern Matches Borland: TApplication::cascade() (tapplica.cpp:75-79)
- `get_tile_rect(&self) -> Rect` - Get the rectangle to use for tiling/cascading operations Matches Borland: TApplication::getTileRect() (tapplica.cpp:94-97) Default implementation returns the full desktop extent Can be overridden to customize the tile area
- `command_enabled(&self, command: CommandId) -> bool` - Check if a command is currently enabled Matches Borland: TView::commandEnabled(ushort command) (tview.cc:142-147)
- `enable_command(&mut self, command: CommandId)` - Enable a single command Matches Borland: TView::enableCommand(ushort command) (tview.cc:384-389)
- `disable_command(&mut self, command: CommandId)` - Disable a single command Matches Borland: TView::disableCommand(ushort command) (tview.cc:161-166)
- `block_edit_mode(&self) -> bool` - Is block-edit mode on? Editors start rectangular selections while it is.
- `set_block_edit_mode(&mut self, on: bool)` - Turn block-edit mode on or off.
- `toggle_block_edit_mode(&mut self) -> bool` - Flip block-edit mode and return the new value.
- `beep(&mut self)` - Emit a beep sound Matches Borland: TScreen::makeBeep() - provides audio feedback for errors/alerts Commonly used in dialog validation failures and error messages
- `set_esc_timeout(&mut self, timeout_ms: u64) -> Result<()>` - Set the ESC timeout in milliseconds
- `set_help_context(&mut self, help_ctx: u16)` - Idle processing - broadcasts command set changes and updates command states Matches Borland: TProgram::idle() (tprogram.cc:248-257) Set the current help context.
- `idle(&mut self)`
- `suspend(&mut self) -> crate::core::error::Result<()>` - Suspend the application (for Ctrl+Z handling) Matches Borland: TProgram::suspend() - temporarily exits TUI mode Restores terminal to normal mode, allowing user to return to shell Call resume() to return to TUI mode
- `resume(&mut self) -> crate::core::error::Result<()>` - Resume the application after suspension (for Ctrl+Z handling) Matches Borland: TProgram::resume() - returns to TUI mode and redraws Re-enters raw mode and forces a complete screen redraw

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
   - `label_link()` - The view a label focuses
   - `watches_pointer()` - Sees mouse events without taking them (`Tooltip`)
   - `get_redraw_union()` / `clear_move_tracking()` - For window movement
   - `end_state()` / `end_modal()` - For modal execution (`GroupLike`)
   - `idle()` - Periodic work for overlay widgets
   - `valid(command)` - Veto a close (Borland `valid`)

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

