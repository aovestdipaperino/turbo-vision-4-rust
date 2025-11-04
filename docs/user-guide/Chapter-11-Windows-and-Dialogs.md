# Chapter 11 — Windows and Dialog Boxes (Rust Edition)

**Version:** 0.2.11
**Updated:** 2025-11-04

This chapter explores Window and Dialog objects, the primary containers for user interaction in Turbo Vision applications. You'll learn how to create windows, customize frames, work with dialog boxes, and use standard dialogs for common tasks.

**Prerequisites:** Chapters 7-10 (Architecture, Views, Events, Application)

---

## Table of Contents

1. [Understanding Windows](#understanding-windows)
2. [Creating Windows](#creating-windows)
3. [Window Features](#window-features)
4. [Working with Dialog Boxes](#working-with-dialog-boxes)
5. [Modal vs Modeless](#modal-vs-modeless)
6. [Controls in Dialog Boxes](#controls-in-dialog-boxes)
7. [Data Transfer](#data-transfer)
8. [Standard Dialog Boxes](#standard-dialog-boxes)

---

## Understanding Windows

### What is a Window?

A **Window** is a bordered, movable, resizable container for views:

```rust
pub struct Window {
    bounds: Rect,
    frame: Frame,           // Border with title, close box, zoom icon
    interior: Group,        // Contains child views
    title: String,
    number: Option<u8>,     // Optional window number (1-9)
    state: StateFlags,
    zoom_rect: Rect,        // "Unzoomed" size
}
```

### Window Components

```
┌─[ Title ]──────────────────[1]─[■]─[═]─┐
│                                         │
│                                         │
│             Interior                    │
│           (child views)                 │
│                                         │
│                                         │
└─────────────────────────────────────────┘
    Frame                            Resize corner
```

**Frame** - Border with:
- Title bar
- Close box `[■]`
- Zoom icon `[═]`
- Window number `[1]`
- Resize corner

**Interior** - Group that contains child views

### Window vs Dialog

| Window | Dialog |
|--------|--------|
| Standard colors | Gray colors |
| Has window number | No window number |
| Resizable by default | Fixed size by default |
| Can zoom | Cannot zoom |
| For general views | For data entry/forms |

---

## Creating Windows

### Basic Window

```rust
use turbo_vision::prelude::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut app = Application::new()?;

    // Create a window
    let window = Window::new(
        Rect::new(10, 5, 70, 20),  // Position and size
        "My Window"                 // Title
    );

    // Add to desktop
    app.desktop.add(Box::new(window));

    app.run()?;
    Ok(())
}
```

### Window with Content

```rust
let mut window = Window::new(
    Rect::new(10, 5, 70, 20),
    "Text Viewer"
);

// Add static text
let text = StaticText::new(
    Rect::new(2, 2, 56, 10),
    "Welcome to Turbo Vision!\n\n\
     This is a text viewing window.\n\
     Press Alt+X to exit."
);

window.add(Box::new(text));

app.desktop.add(Box::new(window));
```

### Window with Numbered Title

Windows can have numbers 1-9 displayed in the frame:

```rust
pub struct Window {
    // ...
    number: Option<u8>,  // Window number
}

impl Window {
    pub fn new_with_number(
        bounds: Rect,
        title: &str,
        number: u8
    ) -> Self {
        assert!(number >= 1 && number <= 9, "Window number must be 1-9");

        let mut window = Self::new(bounds, title);
        window.number = Some(number);
        window
    }
}

// Usage:
let editor1 = Window::new_with_number(
    Rect::new(5, 3, 75, 22),
    "Document1.txt",
    1  // Shows as [1] in frame
);

let editor2 = Window::new_with_number(
    Rect::new(7, 5, 77, 24),
    "Document2.txt",
    2  // Shows as [2] in frame
);
```

**Keyboard shortcuts:**
- Alt+1 through Alt+9 select windows by number
- Application manages which numbers are in use

### Window Builder Pattern

For complex windows, use a builder:

```rust
pub struct WindowBuilder {
    bounds: Rect,
    title: String,
    number: Option<u8>,
    with_scrollbars: bool,
    resizable: bool,
    zoomable: bool,
}

impl WindowBuilder {
    pub fn new(bounds: Rect, title: &str) -> Self {
        Self {
            bounds,
            title: title.to_string(),
            number: None,
            with_scrollbars: false,
            resizable: true,
            zoomable: true,
        }
    }

    pub fn number(mut self, n: u8) -> Self {
        self.number = Some(n);
        self
    }

    pub fn with_scrollbars(mut self) -> Self {
        self.with_scrollbars = true;
        self
    }

    pub fn fixed_size(mut self) -> Self {
        self.resizable = false;
        self.zoomable = false;
        self
    }

    pub fn build(self) -> Window {
        let mut window = Window::new(self.bounds, &self.title);
        window.number = self.number;

        // Set flags
        if !self.resizable {
            window.flags &= !WF_GROW;
        }
        if !self.zoomable {
            window.flags &= !WF_ZOOM;
        }

        if self.with_scrollbars {
            window.add_standard_scrollbars();
        }

        window
    }
}

// Usage:
let window = WindowBuilder::new(
    Rect::new(5, 3, 75, 22),
    "Editor"
)
.number(1)
.with_scrollbars()
.build();
```

---

## Window Features

### Window Sizing

#### Minimum and Maximum Size

Control window size limits:

```rust
impl Window {
    pub fn size_limits(&self) -> (Rect, Rect) {
        let min = Rect::new(0, 0, MIN_WIN_WIDTH, MIN_WIN_HEIGHT);
        let max = self.owner_bounds(); // Desktop size

        (min, max)
    }

    pub fn set_min_size(&mut self, width: i16, height: i16) {
        self.min_size = Size { width, height };
    }
}

// Constants
pub const MIN_WIN_WIDTH: i16 = 16;   // Show title, icons
pub const MIN_WIN_HEIGHT: i16 = 6;   // Show frame, content
```

### Zooming

Toggle between full desktop and custom size:

```rust
impl Window {
    pub fn zoom(&mut self) {
        if self.is_zoomed() {
            // Restore to previous size
            self.set_bounds(self.zoom_rect);
        } else {
            // Save current size
            self.zoom_rect = self.bounds;

            // Zoom to fill desktop
            let desktop_bounds = self.owner_bounds();
            self.set_bounds(desktop_bounds);
        }
    }

    fn is_zoomed(&self) -> bool {
        self.bounds == self.owner_bounds()
    }

    fn owner_bounds(&self) -> Rect {
        // Get desktop bounds
        // In practice, query from owner
        Rect::new(0, 0, 80, 24)
    }
}

// User can:
// - Click zoom icon [═] in frame
// - Press F5 (bound to CM_ZOOM)
```

### Custom Zoom Behavior

```rust
pub struct MyWindow {
    window: Window,
    preferred_size: Rect,
}

impl MyWindow {
    pub fn zoom(&mut self) {
        if self.window.is_zoomed() {
            // Restore to preferred size (not previous size)
            self.window.set_bounds(self.preferred_size);
        } else {
            self.window.zoom();  // Normal zoom
        }
    }
}
```

### Scrollbars

Add standard scrollbars to a window:

```rust
impl Window {
    pub fn add_standard_scrollbars(&mut self) {
        let bounds = self.interior.bounds();

        // Vertical scrollbar (right side)
        let vscroll_bounds = Rect::new(
            bounds.b.x - 1,
            bounds.a.y,
            bounds.b.x,
            bounds.b.y - 1  // Leave room for corner
        );
        let vscroll = ScrollBar::new(vscroll_bounds);
        self.interior.add(Box::new(vscroll));

        // Horizontal scrollbar (bottom)
        let hscroll_bounds = Rect::new(
            bounds.a.x,
            bounds.b.y - 1,
            bounds.b.x - 1,  // Leave room for corner
            bounds.b.y
        );
        let hscroll = ScrollBar::new(hscroll_bounds);
        self.interior.add(Box::new(hscroll));
    }
}

// With scrollable content:
let mut window = Window::new(bounds, "Scrollable");

// Add scroller interior
let interior_bounds = window.interior.bounds();
let scroller = Scroller::new(interior_bounds)
    .with_vertical_scrollbar()
    .with_horizontal_scrollbar();

window.interior.add(Box::new(scroller));
```

### Window State

Windows track their state with flags:

```rust
pub const WF_MOVE: u16     = 0x01;  // Can move
pub const WF_GROW: u16     = 0x02;  // Can resize
pub const WF_CLOSE: u16    = 0x04;  // Can close
pub const WF_ZOOM: u16     = 0x08;  // Can zoom

impl Window {
    pub fn new(bounds: Rect, title: &str) -> Self {
        Self {
            bounds,
            title: title.to_string(),
            flags: WF_MOVE | WF_GROW | WF_CLOSE | WF_ZOOM,
            // ...
        }
    }

    pub fn make_fixed(&mut self) {
        self.flags &= !(WF_GROW | WF_ZOOM);
    }

    pub fn make_uncloseable(&mut self) {
        self.flags &= !WF_CLOSE;
    }
}
```

---

## Working with Dialog Boxes

### What is a Dialog?

A **Dialog** is a specialized window for data entry and forms:

```rust
pub struct Dialog {
    window: Window,
}

impl Dialog {
    pub fn new(bounds: Rect, title: &str) -> Self {
        let mut window = Window::new(bounds, title);

        // Dialog-specific attributes
        window.flags = WF_MOVE | WF_CLOSE;  // No resize, no zoom
        window.number = None;                // No window number
        window.palette = GRAY_PALETTE;       // Gray colors

        Self { window }
    }
}
```

### Dialog vs Window Differences

| Feature | Window | Dialog |
|---------|--------|--------|
| **Colors** | Normal | Gray |
| **Resizable** | Yes | No |
| **Zoomable** | Yes | No |
| **Window number** | Optional | None |
| **Typical use** | Documents | Data entry |

### Creating a Dialog

```rust
let dialog = Dialog::new(
    Rect::new(20, 8, 60, 18),
    "User Information"
);

// Dialog is just a specialized window
app.desktop.add(Box::new(dialog));
```

### Dialog with Controls

```rust
let mut dialog = Dialog::new(
    Rect::new(20, 8, 60, 18),
    "Login"
);

// Add label
let name_label = Label::new(
    Rect::new(2, 2, 15, 3),
    "~N~ame:"
);
dialog.add(Box::new(name_label));

// Add input
let name_data = Rc::new(RefCell::new(String::new()));
let name_input = InputLine::new(
    Rect::new(15, 2, 36, 3),
    50,
    name_data.clone()
);
dialog.add(Box::new(name_input));

// Add buttons
let ok_button = Button::new(
    Rect::new(10, 6, 20, 8),
    "~O~K",
    CM_OK,
    true  // Default button
);
dialog.add(Box::new(ok_button));

let cancel_button = Button::new(
    Rect::new(22, 6, 32, 8),
    "~C~ancel",
    CM_CANCEL,
    false
);
dialog.add(Box::new(cancel_button));
```

---

## Modal vs Modeless

### Modeless Dialogs

Normal window behavior - user can switch to other windows:

```rust
// Create dialog
let dialog = Dialog::new(bounds, "Modeless Dialog");

// Add to desktop like any window
app.desktop.add(Box::new(dialog));

// User can interact with other windows
```

### Modal Dialogs

Block all other interaction until closed:

```rust
// Create dialog
let mut dialog = Dialog::new(
    Rect::new(20, 8, 60, 16),
    "Confirm Action"
);

// Add controls...

// Execute modally
let result = dialog.execute(&mut app);

// User CANNOT interact with other windows
// Dialog blocks until closed

match result {
    CM_OK => {
        // User clicked OK
        println!("Confirmed!");
    }
    CM_CANCEL => {
        // User clicked Cancel or pressed Esc
        println!("Cancelled");
    }
    _ => {}
}
```

### Modal Execution Loop

Inside `dialog.execute()`:

```rust
impl Dialog {
    pub fn execute(&mut self, app: &mut Application) -> u16 {
        self.window.set_state_flag(SF_MODAL, true);

        let mut running = true;
        let mut result = 0;

        while running {
            // Draw
            self.window.draw(&mut app.terminal);
            app.terminal.flush().ok();

            // Get event
            if let Ok(Some(mut event)) = app.terminal.poll_event(None) {
                // Special key handling
                if event.what == EventType::Keyboard {
                    match event.key_code {
                        KB_ENTER => {
                            // Broadcast cmDefault
                            event = Event::broadcast(CM_DEFAULT);
                        }
                        KB_ESC => {
                            // Generate cmCancel
                            event = Event::command(CM_CANCEL);
                        }
                        _ => {}
                    }
                }

                // Handle event
                self.window.handle_event(&mut event);

                // Check for closing commands
                if event.what == EventType::Command {
                    match event.command {
                        CM_OK | CM_CANCEL | CM_YES | CM_NO => {
                            result = event.command;
                            running = false;
                        }
                        _ => {}
                    }
                }
            }
        }

        self.window.set_state_flag(SF_MODAL, false);
        result
    }
}
```

**Special behavior:**
- `Enter` → broadcasts `CM_DEFAULT` (activates default button)
- `Esc` → generates `CM_CANCEL`
- `CM_OK`, `CM_CANCEL`, `CM_YES`, `CM_NO` → closes dialog

---

## Controls in Dialog Boxes

### Tab Order

Controls receive focus in **insertion order**:

```rust
let mut dialog = Dialog::new(bounds, "Form");

// Tab order: Name → Email → Phone → OK → Cancel

dialog.add(Box::new(name_input));    // Tab 1
dialog.add(Box::new(email_input));   // Tab 2
dialog.add(Box::new(phone_input));   // Tab 3
dialog.add(Box::new(ok_button));     // Tab 4
dialog.add(Box::new(cancel_button)); // Tab 5
```

**Best Practices:**
- Insert controls in logical order (top-to-bottom or left-to-right)
- Group related controls together
- Put buttons last

### Example: Data Entry Form

```rust
pub fn create_customer_form(bounds: Rect) -> Dialog {
    let mut dialog = Dialog::new(bounds, "New Customer");

    let mut y = 2;

    // Name field
    dialog.add(Box::new(Label::new(
        Rect::new(2, y, 15, y + 1),
        "~N~ame:"
    )));
    let name_data = Rc::new(RefCell::new(String::new()));
    dialog.add(Box::new(InputLine::new(
        Rect::new(15, y, 50, y + 1),
        100,
        name_data
    )));
    y += 2;

    // Email field
    dialog.add(Box::new(Label::new(
        Rect::new(2, y, 15, y + 1),
        "~E~mail:"
    )));
    let email_data = Rc::new(RefCell::new(String::new()));
    dialog.add(Box::new(InputLine::new(
        Rect::new(15, y, 50, y + 1),
        100,
        email_data
    )));
    y += 2;

    // Phone field
    dialog.add(Box::new(Label::new(
        Rect::new(2, y, 15, y + 1),
        "~P~hone:"
    )));
    let phone_data = Rc::new(RefCell::new(String::new()));
    dialog.add(Box::new(InputLine::new(
        Rect::new(15, y, 50, y + 1),
        20,
        phone_data
    )));
    y += 2;

    // Buttons
    dialog.add(Box::new(Button::new(
        Rect::new(15, y, 25, y + 2),
        "~O~K",
        CM_OK,
        true
    )));

    dialog.add(Box::new(Button::new(
        Rect::new(27, y, 37, y + 2),
        "~C~ancel",
        CM_CANCEL,
        false
    )));

    dialog
}
```

---

## Data Transfer

### The Problem

How do you get data in/out of dialog controls?

```rust
// Need to:
// 1. Initialize controls with data
// 2. Read control values after dialog closes
```

### Solution: GetData and SetData

Pascal used `GetData`/`SetData` with struct packing. Rust uses shared state:

#### Pascal Approach (for reference)

```pascal
type
  TCustomerData = record
    Name: String[50];
    Email: String[50];
    Phone: String[20];
  end;

var
  Data: TCustomerData;
begin
  // Initialize
  Data.Name := 'John Doe';
  Data.Email := 'john@example.com';
  Data.Phone := '555-1234';

  // Set controls
  Dialog.SetData(Data);

  // Execute
  if ExecuteDialog(Dialog, @Data) = cmOk then
  begin
    // Data now contains values from controls
    WriteLn(Data.Name);
  end;
end;
```

#### Rust Approach: Shared State

```rust
// Define data structure
pub struct CustomerData {
    pub name: Rc<RefCell<String>>,
    pub email: Rc<RefCell<String>>,
    pub phone: Rc<RefCell<String>>,
}

impl CustomerData {
    pub fn new() -> Self {
        Self {
            name: Rc::new(RefCell::new(String::new())),
            email: Rc::new(RefCell::new(String::new())),
            phone: Rc::new(RefCell::new(String::new())),
        }
    }

    pub fn with_values(name: &str, email: &str, phone: &str) -> Self {
        Self {
            name: Rc::new(RefCell::new(name.to_string())),
            email: Rc::new(RefCell::new(email.to_string())),
            phone: Rc::new(RefCell::new(phone.to_string())),
        }
    }
}

// Create dialog with shared data
pub fn create_customer_dialog(data: &CustomerData) -> Dialog {
    let mut dialog = Dialog::new(
        Rect::new(15, 5, 65, 15),
        "Customer"
    );

    // Name input (shares data)
    dialog.add(Box::new(Label::new(
        Rect::new(2, 2, 12, 3),
        "~N~ame:"
    )));
    dialog.add(Box::new(InputLine::new(
        Rect::new(12, 2, 46, 3),
        50,
        data.name.clone()  // Share reference
    )));

    // Email input (shares data)
    dialog.add(Box::new(Label::new(
        Rect::new(2, 4, 12, 5),
        "~E~mail:"
    )));
    dialog.add(Box::new(InputLine::new(
        Rect::new(12, 4, 46, 5),
        50,
        data.email.clone()
    )));

    // Phone input (shares data)
    dialog.add(Box::new(Label::new(
        Rect::new(2, 6, 12, 7),
        "~P~hone:"
    )));
    dialog.add(Box::new(InputLine::new(
        Rect::new(12, 6, 32, 7),
        20,
        data.phone.clone()
    )));

    // Buttons
    dialog.add(Box::new(Button::new(
        Rect::new(15, 8, 25, 10),
        "~O~K",
        CM_OK,
        true
    )));

    dialog.add(Box::new(Button::new(
        Rect::new(27, 8, 37, 10),
        "~C~ancel",
        CM_CANCEL,
        false
    )));

    dialog
}

// Usage:
fn edit_customer() {
    // Create data (initialized)
    let data = CustomerData::with_values(
        "John Doe",
        "john@example.com",
        "555-1234"
    );

    // Create dialog with shared data
    let dialog = create_customer_dialog(&data);

    // Execute
    let result = dialog.execute(&mut app);

    if result == CM_OK {
        // Read values from shared data
        println!("Name:  {}", data.name.borrow());
        println!("Email: {}", data.email.borrow());
        println!("Phone: {}", data.phone.borrow());
    }
}
```

### Why Shared State Works Better

**Pascal:**
- Must pack data into exact byte layout
- Must match control order precisely
- Fragile (add control → update struct)

**Rust:**
- Each control has direct reference to its data
- Order doesn't matter
- Adding controls doesn't break existing code
- Type-safe at compile time

---

## Standard Dialog Boxes

### Message Boxes

Simple dialogs for displaying messages:

```rust
pub struct MessageBox {
    title: String,
    message: String,
    buttons: MessageBoxButtons,
}

pub enum MessageBoxButtons {
    Ok,
    OkCancel,
    YesNo,
    YesNoCancel,
}

impl MessageBox {
    pub fn new(title: &str, message: &str, buttons: MessageBoxButtons) -> Self {
        Self {
            title: title.to_string(),
            message: message.to_string(),
            buttons,
        }
    }

    pub fn show(&mut self, app: &mut Application) -> u16 {
        // Calculate size based on message
        let width = self.message.len().min(60) as i16 + 4;
        let height = 8;

        let bounds = Rect::new(
            (80 - width) / 2,
            (24 - height) / 2,
            (80 - width) / 2 + width,
            (24 - height) / 2 + height
        );

        let mut dialog = Dialog::new(bounds, &self.title);

        // Add message text
        let text = StaticText::new(
            Rect::new(2, 2, width - 2, 5),
            &self.message
        );
        dialog.add(Box::new(text));

        // Add buttons based on type
        let button_y = 5;
        match self.buttons {
            MessageBoxButtons::Ok => {
                dialog.add(Box::new(Button::new(
                    Rect::new((width - 10) / 2, button_y, (width - 10) / 2 + 10, button_y + 2),
                    "~O~K",
                    CM_OK,
                    true
                )));
            }
            MessageBoxButtons::OkCancel => {
                dialog.add(Box::new(Button::new(
                    Rect::new(width / 2 - 12, button_y, width / 2 - 2, button_y + 2),
                    "~O~K",
                    CM_OK,
                    true
                )));
                dialog.add(Box::new(Button::new(
                    Rect::new(width / 2 + 2, button_y, width / 2 + 12, button_y + 2),
                    "~C~ancel",
                    CM_CANCEL,
                    false
                )));
            }
            MessageBoxButtons::YesNo => {
                dialog.add(Box::new(Button::new(
                    Rect::new(width / 2 - 12, button_y, width / 2 - 2, button_y + 2),
                    "~Y~es",
                    CM_YES,
                    true
                )));
                dialog.add(Box::new(Button::new(
                    Rect::new(width / 2 + 2, button_y, width / 2 + 12, button_y + 2),
                    "~N~o",
                    CM_NO,
                    false
                )));
            }
            MessageBoxButtons::YesNoCancel => {
                dialog.add(Box::new(Button::new(
                    Rect::new(width / 2 - 18, button_y, width / 2 - 10, button_y + 2),
                    "~Y~es",
                    CM_YES,
                    true
                )));
                dialog.add(Box::new(Button::new(
                    Rect::new(width / 2 - 5, button_y, width / 2 + 5, button_y + 2),
                    "~N~o",
                    CM_NO,
                    false
                )));
                dialog.add(Box::new(Button::new(
                    Rect::new(width / 2 + 10, button_y, width / 2 + 18, button_y + 2),
                    "~C~ancel",
                    CM_CANCEL,
                    false
                )));
            }
        }

        dialog.execute(app)
    }
}

// Usage:
let result = MessageBox::new(
    "Confirm",
    "Delete this file?",
    MessageBoxButtons::YesNo
).show(&mut app);

if result == CM_YES {
    // Delete file
}
```

### Information, Warning, Error Boxes

```rust
pub enum MessageBoxType {
    Information,
    Warning,
    Error,
}

impl MessageBox {
    pub fn info(message: &str) -> Self {
        Self::new("Information", message, MessageBoxButtons::Ok)
    }

    pub fn warning(message: &str) -> Self {
        Self::new("Warning", message, MessageBoxButtons::OkCancel)
    }

    pub fn error(message: &str) -> Self {
        Self::new("Error", message, MessageBoxButtons::Ok)
    }
}

// Usage:
MessageBox::info("File saved successfully").show(&mut app);

let result = MessageBox::warning("File has unsaved changes").show(&mut app);
if result == CM_OK {
    // Proceed
}

MessageBox::error("Cannot open file: Permission denied").show(&mut app);
```

### File Dialog

Standard file open/save dialog:

```rust
pub struct FileDialog {
    dialog: Dialog,
    file_list: Rc<RefCell<Vec<PathBuf>>>,
    selected: Rc<RefCell<Option<PathBuf>>>,
}

impl FileDialog {
    pub fn new(title: &str, pattern: &str) -> Self {
        let bounds = Rect::new(10, 5, 70, 20);
        let mut dialog = Dialog::new(bounds, title);

        // File list
        let file_list = Rc::new(RefCell::new(Vec::new()));
        let list_box = FileListBox::new(
            Rect::new(2, 2, 56, 13),
            file_list.clone()
        );
        dialog.add(Box::new(list_box));

        // Path input
        let path_data = Rc::new(RefCell::new(pattern.to_string()));
        dialog.add(Box::new(Label::new(
            Rect::new(2, 13, 12, 14),
            "~P~ath:"
        )));
        dialog.add(Box::new(InputLine::new(
            Rect::new(12, 13, 56, 14),
            255,
            path_data
        )));

        // Buttons
        dialog.add(Box::new(Button::new(
            Rect::new(20, 15, 30, 17),
            "~O~K",
            CM_OK,
            true
        )));

        dialog.add(Box::new(Button::new(
            Rect::new(32, 15, 42, 17),
            "~C~ancel",
            CM_CANCEL,
            false
        )));

        Self {
            dialog,
            file_list,
            selected: Rc::new(RefCell::new(None)),
        }
    }

    pub fn execute(&mut self, app: &mut Application) -> Option<PathBuf> {
        // Populate file list
        self.read_directory("*.txt");

        let result = self.dialog.execute(app);

        if result == CM_OK {
            self.selected.borrow().clone()
        } else {
            None
        }
    }

    fn read_directory(&mut self, pattern: &str) {
        use std::fs;

        let mut files = Vec::new();
        if let Ok(entries) = fs::read_dir(".") {
            for entry in entries.flatten() {
                if let Ok(metadata) = entry.metadata() {
                    if metadata.is_file() {
                        files.push(entry.path());
                    }
                }
            }
        }

        *self.file_list.borrow_mut() = files;
    }
}

// Usage:
if let Some(path) = FileDialog::new("Open File", "*.txt").execute(&mut app) {
    println!("Selected: {}", path.display());
    // Open file
}
```

### Directory Change Dialog

```rust
pub struct DirDialog {
    dialog: Dialog,
    selected: Rc<RefCell<Option<PathBuf>>>,
}

impl DirDialog {
    pub fn new() -> Self {
        let bounds = Rect::new(15, 6, 65, 18);
        let mut dialog = Dialog::new(bounds, "Change Directory");

        // Directory tree
        let dir_list = DirTreeView::new(
            Rect::new(2, 2, 46, 10)
        );
        dialog.add(Box::new(dir_list));

        // Buttons
        dialog.add(Box::new(Button::new(
            Rect::new(15, 10, 25, 12),
            "~O~K",
            CM_OK,
            true
        )));

        dialog.add(Box::new(Button::new(
            Rect::new(27, 10, 37, 12),
            "~C~ancel",
            CM_CANCEL,
            false
        )));

        Self {
            dialog,
            selected: Rc::new(RefCell::new(None)),
        }
    }

    pub fn execute(&mut self, app: &mut Application) -> Option<PathBuf> {
        let result = self.dialog.execute(app);

        if result == CM_OK {
            self.selected.borrow().clone()
        } else {
            None
        }
    }
}

// Usage:
if let Some(dir) = DirDialog::new().execute(&mut app) {
    std::env::set_current_dir(dir)?;
}
```

---

## Complete Example: Application with Dialogs

```rust
use turbo_vision::prelude::*;
use std::rc::Rc;
use std::cell::RefCell;

pub struct MyApp {
    app: Application,
    customer_data: CustomerData,
}

pub struct CustomerData {
    name: Rc<RefCell<String>>,
    email: Rc<RefCell<String>>,
    phone: Rc<RefCell<String>>,
}

impl CustomerData {
    pub fn new() -> Self {
        Self {
            name: Rc::new(RefCell::new(String::new())),
            email: Rc::new(RefCell::new(String::new())),
            phone: Rc::new(RefCell::new(String::new())),
        }
    }
}

impl MyApp {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let app = Application::new()?;
        let customer_data = CustomerData::new();

        Ok(Self { app, customer_data })
    }

    pub fn run(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        self.create_main_window();

        self.app.run()?;
        Ok(())
    }

    fn create_main_window(&mut self) {
        let window = WindowBuilder::new(
            Rect::new(10, 5, 70, 20),
            "Customer Manager"
        )
        .number(1)
        .build();

        self.app.desktop.add(Box::new(window));
    }

    fn handle_event(&mut self, event: &mut Event) {
        if event.what == EventType::Command {
            match event.command {
                CM_NEW => {
                    self.new_customer();
                    event.clear();
                }
                CM_EDIT => {
                    self.edit_customer();
                    event.clear();
                }
                CM_DELETE => {
                    if self.confirm_delete() {
                        self.delete_customer();
                    }
                    event.clear();
                }
                _ => {}
            }
        }
    }

    fn new_customer(&mut self) {
        // Clear data
        *self.customer_data.name.borrow_mut() = String::new();
        *self.customer_data.email.borrow_mut() = String::new();
        *self.customer_data.phone.borrow_mut() = String::new();

        // Show dialog
        let dialog = self.create_customer_dialog();
        let result = dialog.execute(&mut self.app);

        if result == CM_OK {
            // Save customer
            println!("New customer: {}", self.customer_data.name.borrow());
        }
    }

    fn edit_customer(&mut self) {
        // Load existing data
        *self.customer_data.name.borrow_mut() = "John Doe".to_string();
        *self.customer_data.email.borrow_mut() = "john@example.com".to_string();
        *self.customer_data.phone.borrow_mut() = "555-1234".to_string();

        // Show dialog
        let dialog = self.create_customer_dialog();
        let result = dialog.execute(&mut self.app);

        if result == CM_OK {
            // Update customer
            println!("Updated: {}", self.customer_data.name.borrow());
        }
    }

    fn delete_customer(&mut self) {
        println!("Deleting customer...");
    }

    fn confirm_delete(&mut self) -> bool {
        let result = MessageBox::new(
            "Confirm Delete",
            "Delete this customer?\nThis cannot be undone.",
            MessageBoxButtons::YesNo
        ).show(&mut self.app);

        result == CM_YES
    }

    fn create_customer_dialog(&self) -> Dialog {
        let mut dialog = Dialog::new(
            Rect::new(15, 6, 65, 16),
            "Customer"
        );

        // Name
        dialog.add(Box::new(Label::new(
            Rect::new(2, 2, 12, 3),
            "~N~ame:"
        )));
        dialog.add(Box::new(InputLine::new(
            Rect::new(12, 2, 46, 3),
            50,
            self.customer_data.name.clone()
        )));

        // Email
        dialog.add(Box::new(Label::new(
            Rect::new(2, 4, 12, 5),
            "~E~mail:"
        )));
        dialog.add(Box::new(InputLine::new(
            Rect::new(12, 4, 46, 5),
            50,
            self.customer_data.email.clone()
        )));

        // Phone
        dialog.add(Box::new(Label::new(
            Rect::new(2, 6, 12, 7),
            "~P~hone:"
        )));
        dialog.add(Box::new(InputLine::new(
            Rect::new(12, 6, 32, 7),
            20,
            self.customer_data.phone.clone()
        )));

        // Buttons
        dialog.add(Box::new(Button::new(
            Rect::new(15, 8, 25, 10),
            "~O~K",
            CM_OK,
            true
        )));

        dialog.add(Box::new(Button::new(
            Rect::new(27, 8, 37, 10),
            "~C~ancel",
            CM_CANCEL,
            false
        )));

        dialog
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut app = MyApp::new()?;
    app.run()?;
    Ok(())
}
```

---

## Best Practices

### 1. Use Dialogs for Data Entry

```rust
// ✓ Good - dialog for form
let dialog = Dialog::new(bounds, "Enter Data");

// ✗ Bad - window for simple form
let window = Window::new(bounds, "Enter Data");
```

### 2. Logical Tab Order

```rust
// ✓ Good - top to bottom, left to right
dialog.add(Box::new(name_input));     // Top
dialog.add(Box::new(email_input));    // Middle
dialog.add(Box::new(phone_input));    // Bottom
dialog.add(Box::new(ok_button));      // Buttons last

// ✗ Bad - random order
dialog.add(Box::new(ok_button));      // Button first?
dialog.add(Box::new(email_input));
dialog.add(Box::new(name_input));
```

### 3. Use Shared State

```rust
// ✓ Good - direct data binding
let data = Rc::new(RefCell::new(String::new()));
let input = InputLine::new(bounds, 50, data.clone());

// After dialog:
println!("{}", data.borrow());  // Read directly

// ✗ Bad - trying to extract data later
// (Complex, error-prone)
```

### 4. Standard Dialog Sizes

```rust
// Common dialog sizes
const SMALL_DIALOG: Rect = Rect::new(20, 8, 60, 16);   // 40×8
const MEDIUM_DIALOG: Rect = Rect::new(15, 6, 65, 18);  // 50×12
const LARGE_DIALOG: Rect = Rect::new(10, 4, 70, 20);   // 60×16

// Message box
const MESSAGE_BOX: Rect = Rect::new(20, 9, 60, 15);    // 40×6
```

### 5. Modal for Important Choices

```rust
// ✓ Good - modal for confirmation
let result = MessageBox::warning("Unsaved changes").show(&mut app);
if result == CM_OK {
    // Proceed
}

// ✗ Bad - modeless for critical confirmation
// (User might not see it)
```

---

## Pascal vs. Rust Summary

| Concept | Pascal | Rust |
|---------|--------|------|
| **Window** | `TWindow = object(TGroup)` | `struct Window` |
| **Dialog** | `TDialog = object(TWindow)` | `struct Dialog` (contains Window) |
| **Constructor** | `Init(Bounds, Title, Number)` | `Window::new(bounds, title)` |
| **Frame** | `InitFrame` virtual method | Constructed in `Window::new()` |
| **Scrollbars** | `StandardScrollBar` method | `add_standard_scrollbars()` |
| **Modal** | `Desktop^.Execute(Dialog)` | `dialog.execute(&mut app)` |
| **Data Transfer** | `GetData`/`SetData` with structs | `Rc<RefCell<T>>` shared state |
| **Tab Order** | Insertion order | Insertion order (same) |
| **Window Number** | `Number: Integer` field | `number: Option<u8>` |
| **Message Box** | `MessageBox(msg, nil, flags)` | `MessageBox::new(...).show()` |

---

## Summary

### Key Concepts

1. **Windows** - Bordered, movable, resizable containers
2. **Dialogs** - Specialized windows for data entry (gray, fixed size)
3. **Modal** - Blocks other interaction until closed
4. **Modeless** - Normal window behavior
5. **Tab Order** - Insertion order determines focus cycling
6. **Data Transfer** - Use `Rc<RefCell<T>>` for shared state
7. **Standard Dialogs** - Message boxes, file dialogs, directory dialogs

### The Dialog Pattern

```rust
// 1. Define data
let data = MyData::new();

// 2. Create dialog with shared data
let dialog = create_dialog(&data);

// 3. Execute modally
let result = dialog.execute(&mut app);

// 4. Check result
if result == CM_OK {
    // Read data
    process(data);
}
```

---

## See Also

- **Chapter 8** - Views and Groups
- **Chapter 9** - Event-Driven Programming
- **Chapter 10** - Application Objects
- **Chapter 12** - Control Objects (upcoming)
- **docs/TURBOVISION-DESIGN.md** - Implementation details
- **examples/dialogs_demo.rs** - Dialog examples
- **examples/window_resize_demo.rs** - Window sizing

---

Windows and dialogs are the primary user interaction containers in Turbo Vision. Master these concepts to build professional, user-friendly terminal applications.
