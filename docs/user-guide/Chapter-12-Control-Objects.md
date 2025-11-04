# Chapter 12 — Control Objects (Rust Edition)

**Version:** 0.2.11
**Updated:** 2025-11-04

This chapter explores Control Objects, the specialized views that handle user interaction in Turbo Vision applications. You'll learn about buttons, input fields, checkboxes, radio buttons, list boxes, and how to work with them in Rust.

**Prerequisites:** Chapters 8-11 (Views, Events, Application, Windows/Dialogs)

---

## Table of Contents

1. [Understanding Control Objects](#understanding-control-objects)
2. [Buttons](#buttons)
3. [Input Lines](#input-lines)
4. [Static Text and Labels](#static-text-and-labels)
5. [Checkboxes and Radio Buttons](#checkboxes-and-radio-buttons)
6. [List Boxes](#list-boxes)
7. [Scroll Bars](#scroll-bars)
8. [Complete Examples](#complete-examples)

---

## Understanding Control Objects

### What is a Control?

A **control** is a specialized view for user interaction:

```rust
pub trait Control: View {
    // Controls typically have associated data
    fn data_type() -> &'static str;

    // Can be disabled
    fn is_enabled(&self) -> bool;
    fn set_enabled(&mut self, enabled: bool);
}
```

### Common Control Characteristics

**All controls:**
- Occupy rectangular screen area (View trait)
- Handle keyboard and mouse events
- Can receive focus (usually selectable)
- Have visual feedback for state (focused, disabled, pressed)
- Work with shared data (`Rc<RefCell<T>>`)

### Control vs View

| View | Control |
|------|---------|
| Generic display | Specific interaction |
| May not be selectable | Usually selectable |
| Any purpose | User input/choice |
| Example: StaticText | Example: Button, InputLine |

---

## Buttons

### What is a Button?

A button generates a command when clicked:

```rust
pub struct Button {
    bounds: Rect,
    text: String,
    command: u16,       // Command to generate
    is_default: bool,   // Default button (Enter activates)
    state: StateFlags,
}
```

### Creating Buttons

```rust
let button = Button::new(
    Rect::new(10, 10, 20, 12),
    "~O~K",             // Text with shortcut
    CM_OK,              // Command
    true                // Is default
);

dialog.add(Box::new(button));
```

**Visual representation:**
```
┌────────┐
│   OK   │  ← Normal
└────────┘

┏━━━━━━━━┓
┃   OK   ┃  ← Focused (double border)
┗━━━━━━━━┛

 [  OK  ]   ← Default (brackets)
```

### Button Properties

```rust
impl Button {
    pub fn new(
        bounds: Rect,
        text: &str,
        command: u16,
        is_default: bool
    ) -> Self {
        Self {
            bounds,
            text: text.to_string(),
            command,
            is_default,
            state: SF_VISIBLE | SF_SELECTABLE,
        }
    }

    pub fn set_default(&mut self, is_default: bool) {
        self.is_default = is_default;
    }
}
```

### Shortcut Keys

Buttons support two types of shortcuts:

**1. Alt+Letter** - Always works (PreProcess):
```rust
let button = Button::new(bounds, "~O~K", CM_OK, false);
// User can press Alt+O from anywhere in dialog
```

**2. Letter alone** - Works when button unfocused (PostProcess):
```rust
// In PostProcess phase, 'O' activates button if no other control handled it
```

### Button Behavior

```rust
impl View for Button {
    fn handle_event(&mut self, event: &mut Event) {
        match event.what {
            EventType::MouseDown => {
                if self.bounds.contains(event.mouse.position) {
                    self.press();
                    *event = Event::command(self.command);
                    event.clear();
                }
            }
            EventType::Keyboard => {
                // Space when focused
                if self.is_focused() && event.key_code == KB_SPACE {
                    self.press();
                    *event = Event::command(self.command);
                    event.clear();
                }
                // Enter when default
                if self.is_default && event.key_code == KB_ENTER {
                    self.press();
                    *event = Event::command(self.command);
                    event.clear();
                }
                // Alt+letter shortcut
                if event.key_code == self.shortcut_key() {
                    self.press();
                    *event = Event::command(self.command);
                    event.clear();
                }
            }
            EventType::Broadcast => {
                // CM_DEFAULT broadcast activates default button
                if event.command == CM_DEFAULT && self.is_default {
                    self.press();
                    *event = Event::command(self.command);
                    event.clear();
                }
            }
            _ => {}
        }
    }

    fn options(&self) -> u16 {
        // PostProcess to see events after focused view
        OF_SELECTABLE | OF_POST_PROCESS
    }
}
```

### Button Groups

Common button arrangements:

```rust
// OK/Cancel buttons
pub fn create_ok_cancel_buttons(
    dialog: &mut Dialog,
    y: i16
) -> (Rc<RefCell<Button>>, Rc<RefCell<Button>>) {
    let ok = Rc::new(RefCell::new(Button::new(
        Rect::new(15, y, 25, y + 2),
        "~O~K",
        CM_OK,
        true  // Default
    )));

    let cancel = Rc::new(RefCell::new(Button::new(
        Rect::new(27, y, 37, y + 2),
        "~C~ancel",
        CM_CANCEL,
        false
    )));

    dialog.add(Box::new(ok.borrow_mut()));
    dialog.add(Box::new(cancel.borrow_mut()));

    (ok, cancel)
}

// Yes/No/Cancel buttons
pub fn create_yes_no_cancel_buttons(
    dialog: &mut Dialog,
    y: i16
) {
    dialog.add(Box::new(Button::new(
        Rect::new(10, y, 20, y + 2),
        "~Y~es",
        CM_YES,
        true
    )));

    dialog.add(Box::new(Button::new(
        Rect::new(22, y, 32, y + 2),
        "~N~o",
        CM_NO,
        false
    )));

    dialog.add(Box::new(Button::new(
        Rect::new(34, y, 44, y + 2),
        "~C~ancel",
        CM_CANCEL,
        false
    )));
}
```

---

## Input Lines

### What is an Input Line?

Single-line text input field:

```rust
pub struct InputLine {
    bounds: Rect,
    data: Rc<RefCell<String>>,  // Shared data
    max_length: usize,
    cursor_pos: usize,
    first_pos: usize,           // Scroll position
    state: StateFlags,
    validator: Option<Box<dyn Validator>>,
}
```

### Creating Input Lines

```rust
// Create shared data
let name_data = Rc::new(RefCell::new(String::new()));

// Create input line
let input = InputLine::new(
    Rect::new(2, 2, 40, 3),
    50,               // Max length
    name_data.clone()
);

dialog.add(Box::new(input));

// Later, read data
println!("Name: {}", name_data.borrow());
```

### Input Line Features

```rust
impl InputLine {
    pub fn new(
        bounds: Rect,
        max_length: usize,
        data: Rc<RefCell<String>>
    ) -> Self {
        Self {
            bounds,
            data,
            max_length,
            cursor_pos: 0,
            first_pos: 0,
            state: SF_VISIBLE | SF_SELECTABLE,
            validator: None,
        }
    }

    pub fn with_validator(mut self, validator: Box<dyn Validator>) -> Self {
        self.validator = Some(validator);
        self
    }

    // Cursor movement
    fn move_cursor_left(&mut self) {
        if self.cursor_pos > 0 {
            self.cursor_pos -= 1;
            self.adjust_scroll();
        }
    }

    fn move_cursor_right(&mut self) {
        let len = self.data.borrow().len();
        if self.cursor_pos < len {
            self.cursor_pos += 1;
            self.adjust_scroll();
        }
    }

    // Text editing
    fn insert_char(&mut self, ch: char) {
        let mut data = self.data.borrow_mut();

        if data.len() < self.max_length {
            data.insert(self.cursor_pos, ch);
            self.cursor_pos += 1;
            self.adjust_scroll();
        }
    }

    fn delete_char(&mut self) {
        let mut data = self.data.borrow_mut();

        if self.cursor_pos > 0 {
            data.remove(self.cursor_pos - 1);
            self.cursor_pos -= 1;
            self.adjust_scroll();
        }
    }

    // Scroll adjustment
    fn adjust_scroll(&mut self) {
        let width = self.bounds.width() as usize;

        // Scroll left if cursor before visible area
        if self.cursor_pos < self.first_pos {
            self.first_pos = self.cursor_pos;
        }

        // Scroll right if cursor after visible area
        if self.cursor_pos >= self.first_pos + width {
            self.first_pos = self.cursor_pos - width + 1;
        }
    }
}
```

### Input Line Event Handling

```rust
impl View for InputLine {
    fn handle_event(&mut self, event: &mut Event) {
        if event.what == EventType::Keyboard {
            match event.key_code {
                KB_LEFT => {
                    self.move_cursor_left();
                    event.clear();
                }
                KB_RIGHT => {
                    self.move_cursor_right();
                    event.clear();
                }
                KB_HOME => {
                    self.cursor_pos = 0;
                    self.first_pos = 0;
                    event.clear();
                }
                KB_END => {
                    self.cursor_pos = self.data.borrow().len();
                    self.adjust_scroll();
                    event.clear();
                }
                KB_BACK => {
                    self.delete_char();
                    event.clear();
                }
                KB_DEL => {
                    if self.cursor_pos < self.data.borrow().len() {
                        self.data.borrow_mut().remove(self.cursor_pos);
                    }
                    event.clear();
                }
                _ => {
                    // Regular character
                    if let Some(ch) = key_to_char(event.key_code) {
                        // Validate if validator present
                        if let Some(ref validator) = self.validator {
                            if validator.valid_char(ch) {
                                self.insert_char(ch);
                            }
                        } else {
                            self.insert_char(ch);
                        }
                        event.clear();
                    }
                }
            }
        }
    }
}
```

### Input Line with History

Support for history lists (like command line):

```rust
pub struct InputLineWithHistory {
    input: InputLine,
    history: Vec<String>,
    history_index: Option<usize>,
}

impl InputLineWithHistory {
    pub fn handle_up_arrow(&mut self) {
        if self.history.is_empty() {
            return;
        }

        self.history_index = match self.history_index {
            None => Some(self.history.len() - 1),
            Some(0) => Some(0),
            Some(i) => Some(i - 1),
        };

        if let Some(idx) = self.history_index {
            *self.input.data.borrow_mut() = self.history[idx].clone();
            self.input.cursor_pos = self.history[idx].len();
        }
    }

    pub fn handle_down_arrow(&mut self) {
        if let Some(idx) = self.history_index {
            if idx < self.history.len() - 1 {
                self.history_index = Some(idx + 1);
                *self.input.data.borrow_mut() = self.history[idx + 1].clone();
            } else {
                self.history_index = None;
                *self.input.data.borrow_mut() = String::new();
            }
            self.input.cursor_pos = self.input.data.borrow().len();
        }
    }

    pub fn add_to_history(&mut self, text: String) {
        if !text.is_empty() && !self.history.contains(&text) {
            self.history.push(text);
        }
    }
}
```

---

## Static Text and Labels

### Static Text

Display read-only text:

```rust
pub struct StaticText {
    bounds: Rect,
    text: String,
    state: StateFlags,
}

impl StaticText {
    pub fn new(bounds: Rect, text: &str) -> Self {
        Self {
            bounds,
            text: text.to_string(),
            state: SF_VISIBLE,  // Not selectable
        }
    }
}
```

**Formatting:**
- `\n` - Line break
- Centered text - Special handling

```rust
let text = StaticText::new(
    Rect::new(2, 2, 40, 6),
    "Welcome to the application!\n\n\
     Please enter your information\n\
     in the fields below."
);
```

### Labels

Labels link to controls and show shortcuts:

```rust
pub struct Label {
    bounds: Rect,
    text: String,
    linked_control: Option<usize>,  // Index in parent
    state: StateFlags,
}

impl Label {
    pub fn new(bounds: Rect, text: &str) -> Self {
        Self {
            bounds,
            text: text.to_string(),
            linked_control: None,
            state: SF_VISIBLE,
        }
    }

    pub fn link_to(&mut self, control_index: usize) {
        self.linked_control = Some(control_index);
    }

    fn shortcut_key(&self) -> Option<u16> {
        // Extract from ~X~ syntax
        extract_shortcut(&self.text)
    }
}

impl View for Label {
    fn handle_event(&mut self, event: &mut Event) {
        if event.what == EventType::Keyboard {
            if let Some(key) = self.shortcut_key() {
                if event.key_code == key {
                    // Focus linked control
                    if let Some(_idx) = self.linked_control {
                        // Broadcast to parent to focus control
                        *event = Event::broadcast(CM_FOCUS_CONTROL);
                    }
                    event.clear();
                }
            }
        }
    }

    fn options(&self) -> u16 {
        OF_PRE_PROCESS  // See shortcuts first
    }
}
```

### Label-Control Pattern

```rust
// Create control
let name_data = Rc::new(RefCell::new(String::new()));
let name_input = InputLine::new(
    Rect::new(12, 2, 40, 3),
    50,
    name_data.clone()
);

// Create label linked to control
let name_label = Label::new(
    Rect::new(2, 2, 12, 3),
    "~N~ame:"
);
// Label links to next control added

dialog.add(Box::new(name_label));
dialog.add(Box::new(name_input));

// Now Alt+N focuses the input field
```

---

## Checkboxes and Radio Buttons

### Checkbox

Boolean toggle control:

```rust
pub struct Checkbox {
    bounds: Rect,
    text: String,
    data: Rc<RefCell<bool>>,  // Shared boolean
    state: StateFlags,
}

impl Checkbox {
    pub fn new(
        bounds: Rect,
        text: &str,
        data: Rc<RefCell<bool>>
    ) -> Self {
        Self {
            bounds,
            text: text.to_string(),
            data,
            state: SF_VISIBLE | SF_SELECTABLE,
        }
    }

    fn toggle(&mut self) {
        let mut data = self.data.borrow_mut();
        *data = !*data;
    }
}

impl View for Checkbox {
    fn draw(&mut self, terminal: &mut Terminal) {
        let checked = *self.data.borrow();
        let color = if self.is_focused() {
            colors::CHECKBOX_FOCUSED
        } else {
            colors::CHECKBOX_NORMAL
        };

        let width = self.bounds.width() as usize;
        let mut buf = DrawBuffer::new(width);

        // Draw checkbox: [X] or [ ]
        let checkbox = if checked { "[X] " } else { "[ ] " };
        buf.move_str(0, checkbox, color);

        // Draw text with shortcut
        buf.move_str_with_shortcut(
            4,
            &self.text,
            color,
            colors::CHECKBOX_SHORTCUT
        );

        write_line_to_terminal(
            terminal,
            self.bounds.a.x,
            self.bounds.a.y,
            &buf
        );
    }

    fn handle_event(&mut self, event: &mut Event) {
        match event.what {
            EventType::MouseDown => {
                if self.bounds.contains(event.mouse.position) {
                    self.toggle();
                    event.clear();
                }
            }
            EventType::Keyboard => {
                if event.key_code == KB_SPACE {
                    self.toggle();
                    event.clear();
                }
            }
            _ => {}
        }
    }
}
```

**Usage:**
```rust
let enabled = Rc::new(RefCell::new(false));

let checkbox = Checkbox::new(
    Rect::new(2, 2, 20, 3),
    "~E~nabled",
    enabled.clone()
);

dialog.add(Box::new(checkbox));

// Later check state
if *enabled.borrow() {
    println!("Feature is enabled");
}
```

### Radio Buttons

Mutually exclusive choices:

```rust
pub struct RadioButton {
    bounds: Rect,
    text: String,
    value: u16,                      // This button's value
    group_data: Rc<RefCell<u16>>,    // Shared group value
    state: StateFlags,
}

impl RadioButton {
    pub fn new(
        bounds: Rect,
        text: &str,
        value: u16,
        group_data: Rc<RefCell<u16>>
    ) -> Self {
        Self {
            bounds,
            text: text.to_string(),
            value,
            group_data,
            state: SF_VISIBLE | SF_SELECTABLE,
        }
    }

    fn is_selected(&self) -> bool {
        *self.group_data.borrow() == self.value
    }

    fn select(&mut self) {
        *self.group_data.borrow_mut() = self.value;
    }
}

impl View for RadioButton {
    fn draw(&mut self, terminal: &mut Terminal) {
        let selected = self.is_selected();
        let color = if self.is_focused() {
            colors::RADIO_FOCUSED
        } else {
            colors::RADIO_NORMAL
        };

        let width = self.bounds.width() as usize;
        let mut buf = DrawBuffer::new(width);

        // Draw radio button: (•) or ( )
        let radio = if selected { "(•) " } else { "( ) " };
        buf.move_str(0, radio, color);

        // Draw text
        buf.move_str_with_shortcut(
            4,
            &self.text,
            color,
            colors::RADIO_SHORTCUT
        );

        write_line_to_terminal(
            terminal,
            self.bounds.a.x,
            self.bounds.a.y,
            &buf
        );
    }

    fn handle_event(&mut self, event: &mut Event) {
        match event.what {
            EventType::MouseDown => {
                if self.bounds.contains(event.mouse.position) {
                    self.select();
                    event.clear();
                }
            }
            EventType::Keyboard => {
                if event.key_code == KB_SPACE {
                    self.select();
                    event.clear();
                }
            }
            _ => {}
        }
    }
}
```

**Usage:**
```rust
// Shared group value
let payment_method = Rc::new(RefCell::new(0u16));

// Create radio buttons with values 0, 1, 2
let cash = RadioButton::new(
    Rect::new(2, 2, 20, 3),
    "~C~ash",
    0,
    payment_method.clone()
);

let check = RadioButton::new(
    Rect::new(2, 3, 20, 4),
    "C~h~eck",
    1,
    payment_method.clone()
);

let card = RadioButton::new(
    Rect::new(2, 4, 20, 5),
    "~C~redit Card",
    2,
    payment_method.clone()
);

dialog.add(Box::new(cash));
dialog.add(Box::new(check));
dialog.add(Box::new(card));

// Later check selection
match *payment_method.borrow() {
    0 => println!("Cash selected"),
    1 => println!("Check selected"),
    2 => println!("Credit card selected"),
    _ => {}
}
```

### Radio Button Group Helper

```rust
pub struct RadioGroup {
    selected: Rc<RefCell<u16>>,
    buttons: Vec<RadioButton>,
}

impl RadioGroup {
    pub fn new() -> Self {
        Self {
            selected: Rc::new(RefCell::new(0)),
            buttons: Vec::new(),
        }
    }

    pub fn add_button(
        &mut self,
        bounds: Rect,
        text: &str,
        value: u16
    ) -> RadioButton {
        let button = RadioButton::new(
            bounds,
            text,
            value,
            self.selected.clone()
        );
        self.buttons.push(button.clone());
        button
    }

    pub fn selection(&self) -> u16 {
        *self.selected.borrow()
    }
}

// Usage:
let mut payment_group = RadioGroup::new();

dialog.add(Box::new(payment_group.add_button(
    Rect::new(2, 2, 20, 3),
    "~C~ash",
    0
)));

dialog.add(Box::new(payment_group.add_button(
    Rect::new(2, 3, 20, 4),
    "C~h~eck",
    1
)));

// Later
println!("Selected: {}", payment_group.selection());
```

---

## List Boxes

### What is a List Box?

Scrollable list of items:

```rust
pub struct ListBox {
    bounds: Rect,
    items: Rc<RefCell<Vec<String>>>,
    selected: Rc<RefCell<Option<usize>>>,
    focused: usize,
    top_item: usize,              // Scroll position
    scrollbar: Option<ScrollBar>,
    state: StateFlags,
}
```

### Creating List Boxes

```rust
let items = Rc::new(RefCell::new(vec![
    "Apple".to_string(),
    "Banana".to_string(),
    "Cherry".to_string(),
    "Date".to_string(),
    "Elderberry".to_string(),
]));

let selected = Rc::new(RefCell::new(None));

let list_box = ListBox::new(
    Rect::new(2, 2, 30, 12),
    items.clone(),
    selected.clone()
);

dialog.add(Box::new(list_box));

// Later check selection
if let Some(idx) = *selected.borrow() {
    let items = items.borrow();
    println!("Selected: {}", items[idx]);
}
```

### List Box with Scrollbar

```rust
let items = Rc::new(RefCell::new(
    (1..=100).map(|i| format!("Item {}", i)).collect()
));

// Create scrollbar
let scrollbar = ScrollBar::new(
    Rect::new(30, 2, 31, 12)  // Vertical bar
);

// Create list box with scrollbar
let list_box = ListBox::new(bounds, items, selected)
    .with_scrollbar(scrollbar);
```

### List Box Implementation

```rust
impl ListBox {
    pub fn new(
        bounds: Rect,
        items: Rc<RefCell<Vec<String>>>,
        selected: Rc<RefCell<Option<usize>>>
    ) -> Self {
        Self {
            bounds,
            items,
            selected,
            focused: 0,
            top_item: 0,
            scrollbar: None,
            state: SF_VISIBLE | SF_SELECTABLE,
        }
    }

    pub fn with_scrollbar(mut self, scrollbar: ScrollBar) -> Self {
        self.scrollbar = Some(scrollbar);
        self.update_scrollbar();
        self
    }

    fn visible_items(&self) -> usize {
        self.bounds.height() as usize
    }

    fn move_focus_up(&mut self) {
        if self.focused > 0 {
            self.focused -= 1;
            if self.focused < self.top_item {
                self.top_item = self.focused;
            }
            self.update_scrollbar();
        }
    }

    fn move_focus_down(&mut self) {
        let item_count = self.items.borrow().len();
        if self.focused < item_count - 1 {
            self.focused += 1;
            let visible = self.visible_items();
            if self.focused >= self.top_item + visible {
                self.top_item = self.focused - visible + 1;
            }
            self.update_scrollbar();
        }
    }

    fn select_current(&mut self) {
        *self.selected.borrow_mut() = Some(self.focused);
        // Broadcast selection
        // event = Event::broadcast(CM_LIST_ITEM_SELECTED);
    }

    fn update_scrollbar(&mut self) {
        if let Some(ref mut scrollbar) = self.scrollbar {
            let item_count = self.items.borrow().len();
            scrollbar.set_range(0, item_count.saturating_sub(1) as i32);
            scrollbar.set_value(self.top_item as i32);
        }
    }
}

impl View for ListBox {
    fn draw(&mut self, terminal: &mut Terminal) {
        let items = self.items.borrow();
        let width = self.bounds.width() as usize;
        let height = self.bounds.height() as usize;

        for row in 0..height {
            let item_idx = self.top_item + row;
            let mut buf = DrawBuffer::new(width);

            if item_idx < items.len() {
                let color = if item_idx == self.focused {
                    colors::LIST_FOCUSED
                } else if Some(item_idx) == *self.selected.borrow() {
                    colors::LIST_SELECTED
                } else {
                    colors::LIST_NORMAL
                };

                let item = &items[item_idx];
                buf.move_str(0, item, color);

                // Pad rest of line
                let len = item.len().min(width);
                buf.move_char(len, ' ', color, width - len);
            } else {
                // Empty line
                buf.move_char(0, ' ', colors::LIST_NORMAL, width);
            }

            write_line_to_terminal(
                terminal,
                self.bounds.a.x,
                self.bounds.a.y + row as i16,
                &buf
            );
        }
    }

    fn handle_event(&mut self, event: &mut Event) {
        match event.what {
            EventType::Keyboard => {
                match event.key_code {
                    KB_UP => {
                        self.move_focus_up();
                        event.clear();
                    }
                    KB_DOWN => {
                        self.move_focus_down();
                        event.clear();
                    }
                    KB_PGUP => {
                        let visible = self.visible_items();
                        for _ in 0..visible {
                            self.move_focus_up();
                        }
                        event.clear();
                    }
                    KB_PGDN => {
                        let visible = self.visible_items();
                        for _ in 0..visible {
                            self.move_focus_down();
                        }
                        event.clear();
                    }
                    KB_HOME => {
                        self.focused = 0;
                        self.top_item = 0;
                        self.update_scrollbar();
                        event.clear();
                    }
                    KB_END => {
                        let item_count = self.items.borrow().len();
                        self.focused = item_count.saturating_sub(1);
                        let visible = self.visible_items();
                        self.top_item = self.focused.saturating_sub(visible - 1);
                        self.update_scrollbar();
                        event.clear();
                    }
                    KB_ENTER | KB_SPACE => {
                        self.select_current();
                        event.clear();
                    }
                    _ => {}
                }
            }
            EventType::MouseDown => {
                let local_y = event.mouse.position.y - self.bounds.a.y;
                let item_idx = self.top_item + local_y as usize;

                if item_idx < self.items.borrow().len() {
                    self.focused = item_idx;

                    if event.mouse.double_click {
                        self.select_current();
                    }
                }
                event.clear();
            }
            _ => {}
        }
    }
}
```

---

## Scroll Bars

### What is a Scroll Bar?

Visual representation of position in range:

```rust
pub struct ScrollBar {
    bounds: Rect,
    orientation: Orientation,
    value: i32,
    min_value: i32,
    max_value: i32,
    page_size: i32,
    state: StateFlags,
}

pub enum Orientation {
    Horizontal,
    Vertical,
}
```

### Creating Scroll Bars

```rust
// Vertical scrollbar
let vscroll = ScrollBar::new(
    Rect::new(39, 1, 40, 20)  // Width=1 → vertical
);

// Horizontal scrollbar
let hscroll = ScrollBar::new(
    Rect::new(1, 20, 39, 21)  // Height=1 → horizontal
);
```

### Scroll Bar Methods

```rust
impl ScrollBar {
    pub fn new(bounds: Rect) -> Self {
        let orientation = if bounds.width() == 1 {
            Orientation::Vertical
        } else {
            Orientation::Horizontal
        };

        Self {
            bounds,
            orientation,
            value: 0,
            min_value: 0,
            max_value: 100,
            page_size: 10,
            state: SF_VISIBLE,
        }
    }

    pub fn set_range(&mut self, min: i32, max: i32) {
        self.min_value = min;
        self.max_value = max;
        self.value = self.value.clamp(min, max);
    }

    pub fn set_value(&mut self, value: i32) {
        let old_value = self.value;
        self.value = value.clamp(self.min_value, self.max_value);

        if old_value != self.value {
            self.on_value_changed();
        }
    }

    pub fn set_page_size(&mut self, size: i32) {
        self.page_size = size;
    }

    fn on_value_changed(&self) {
        // Broadcast CM_SCROLLBAR_CHANGED
    }

    fn thumb_position(&self) -> i16 {
        let range = self.max_value - self.min_value;
        if range == 0 {
            return 0;
        }

        let available = match self.orientation {
            Orientation::Vertical => self.bounds.height() - 2,  // -2 for arrows
            Orientation::Horizontal => self.bounds.width() - 2,
        };

        let ratio = (self.value - self.min_value) as f32 / range as f32;
        (ratio * available as f32) as i16
    }
}

impl View for ScrollBar {
    fn handle_event(&mut self, event: &mut Event) {
        if event.what == EventType::MouseDown {
            let pos = match self.orientation {
                Orientation::Vertical => {
                    event.mouse.position.y - self.bounds.a.y
                }
                Orientation::Horizontal => {
                    event.mouse.position.x - self.bounds.a.x
                }
            };

            let size = match self.orientation {
                Orientation::Vertical => self.bounds.height(),
                Orientation::Horizontal => self.bounds.width(),
            };

            if pos == 0 {
                // Up/left arrow
                self.set_value(self.value - 1);
            } else if pos == size - 1 {
                // Down/right arrow
                self.set_value(self.value + 1);
            } else {
                // Page area or thumb
                let thumb_pos = self.thumb_position();
                if pos < thumb_pos {
                    self.set_value(self.value - self.page_size);
                } else if pos > thumb_pos {
                    self.set_value(self.value + self.page_size);
                }
            }
            event.clear();
        }
    }
}
```

### Responding to Scroll Bar Changes

```rust
impl View for MyView {
    fn handle_event(&mut self, event: &mut Event) {
        if event.what == EventType::Broadcast {
            if event.command == CM_SCROLLBAR_CHANGED {
                // Check if it's our scrollbar
                // (Would need pointer comparison in real implementation)

                // Update view based on scrollbar value
                self.update_display();
                event.clear();
            }
        }
    }
}
```

---

## Complete Examples

### Example 1: User Registration Form

```rust
use turbo_vision::prelude::*;
use std::rc::Rc;
use std::cell::RefCell;

pub struct RegistrationData {
    pub username: Rc<RefCell<String>>,
    pub password: Rc<RefCell<String>>,
    pub email: Rc<RefCell<String>>,
    pub newsletter: Rc<RefCell<bool>>,
    pub account_type: Rc<RefCell<u16>>,
}

impl RegistrationData {
    pub fn new() -> Self {
        Self {
            username: Rc::new(RefCell::new(String::new())),
            password: Rc::new(RefCell::new(String::new())),
            email: Rc::new(RefCell::new(String::new())),
            newsletter: Rc::new(RefCell::new(false)),
            account_type: Rc::new(RefCell::new(0)),
        }
    }
}

pub fn create_registration_dialog(
    data: &RegistrationData
) -> Dialog {
    let mut dialog = Dialog::new(
        Rect::new(15, 5, 65, 20),
        "User Registration"
    );

    let mut y = 2;

    // Username
    dialog.add(Box::new(Label::new(
        Rect::new(2, y, 14, y + 1),
        "~U~sername:"
    )));
    dialog.add(Box::new(InputLine::new(
        Rect::new(14, y, 46, y + 1),
        20,
        data.username.clone()
    )));
    y += 2;

    // Password
    dialog.add(Box::new(Label::new(
        Rect::new(2, y, 14, y + 1),
        "~P~assword:"
    )));
    dialog.add(Box::new(InputLine::new(
        Rect::new(14, y, 46, y + 1),
        20,
        data.password.clone()
    ).with_password_mode()));
    y += 2;

    // Email
    dialog.add(Box::new(Label::new(
        Rect::new(2, y, 14, y + 1),
        "~E~mail:"
    )));
    dialog.add(Box::new(InputLine::new(
        Rect::new(14, y, 46, y + 1),
        50,
        data.email.clone()
    )));
    y += 2;

    // Newsletter checkbox
    dialog.add(Box::new(Checkbox::new(
        Rect::new(2, y, 30, y + 1),
        "~N~ewsletter subscription",
        data.newsletter.clone()
    )));
    y += 2;

    // Account type radio buttons
    dialog.add(Box::new(StaticText::new(
        Rect::new(2, y, 20, y + 1),
        "Account Type:"
    )));
    y += 1;

    dialog.add(Box::new(RadioButton::new(
        Rect::new(4, y, 20, y + 1),
        "~F~ree",
        0,
        data.account_type.clone()
    )));
    y += 1;

    dialog.add(Box::new(RadioButton::new(
        Rect::new(4, y, 20, y + 1),
        "~B~asic ($5/mo)",
        1,
        data.account_type.clone()
    )));
    y += 1;

    dialog.add(Box::new(RadioButton::new(
        Rect::new(4, y, 20, y + 1),
        "~P~ro ($10/mo)",
        2,
        data.account_type.clone()
    )));
    y += 2;

    // Buttons
    dialog.add(Box::new(Button::new(
        Rect::new(15, y, 25, y + 2),
        "~R~egister",
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

// Usage:
fn show_registration_dialog(app: &mut Application) {
    let data = RegistrationData::new();
    let dialog = create_registration_dialog(&data);

    let result = dialog.execute(app);

    if result == CM_OK {
        println!("Username: {}", data.username.borrow());
        println!("Email: {}", data.email.borrow());
        println!("Newsletter: {}", data.newsletter.borrow());

        let account_type = match *data.account_type.borrow() {
            0 => "Free",
            1 => "Basic",
            2 => "Pro",
            _ => "Unknown",
        };
        println!("Account type: {}", account_type);
    }
}
```

### Example 2: File Selection Dialog

```rust
pub struct FileSelector {
    dialog: Dialog,
    file_list: ListBox,
    selected_file: Rc<RefCell<Option<PathBuf>>>,
}

impl FileSelector {
    pub fn new(directory: &Path, pattern: &str) -> Self {
        let files = Self::scan_directory(directory, pattern);
        let files = Rc::new(RefCell::new(files));
        let selected = Rc::new(RefCell::new(None));

        let mut dialog = Dialog::new(
            Rect::new(10, 5, 70, 20),
            "Select File"
        );

        // File list
        let list_box = ListBox::new(
            Rect::new(2, 2, 56, 13),
            files.clone(),
            selected.clone()
        );
        dialog.add(Box::new(list_box));

        // Path display
        dialog.add(Box::new(StaticText::new(
            Rect::new(2, 13, 56, 14),
            &format!("Path: {}", directory.display())
        )));

        // Buttons
        dialog.add(Box::new(Button::new(
            Rect::new(20, 14, 30, 16),
            "~O~pen",
            CM_OK,
            true
        )));

        dialog.add(Box::new(Button::new(
            Rect::new(32, 14, 42, 16),
            "~C~ancel",
            CM_CANCEL,
            false
        )));

        Self {
            dialog,
            file_list: list_box,
            selected_file: selected,
        }
    }

    fn scan_directory(dir: &Path, pattern: &str) -> Vec<String> {
        use std::fs;

        let mut files = Vec::new();

        if let Ok(entries) = fs::read_dir(dir) {
            for entry in entries.flatten() {
                if let Ok(metadata) = entry.metadata() {
                    if metadata.is_file() {
                        if let Some(name) = entry.file_name().to_str() {
                            files.push(name.to_string());
                        }
                    }
                }
            }
        }

        files.sort();
        files
    }

    pub fn execute(&mut self, app: &mut Application) -> Option<PathBuf> {
        let result = self.dialog.execute(app);

        if result == CM_OK {
            if let Some(idx) = *self.selected_file.borrow() {
                let files = self.file_list.items.borrow();
                Some(PathBuf::from(&files[idx]))
            } else {
                None
            }
        } else {
            None
        }
    }
}

// Usage:
if let Some(path) = FileSelector::new(Path::new("."), "*.txt").execute(&mut app) {
    println!("Selected: {}", path.display());
}
```

---

## Best Practices

### 1. Use Shared State

```rust
// ✓ Good - direct data binding
let data = Rc::new(RefCell::new(String::new()));
let input = InputLine::new(bounds, 50, data.clone());

// After interaction
println!("{}", data.borrow());

// ✗ Bad - trying to query control later
// (More complex, error-prone)
```

### 2. Consistent Layout

```rust
// ✓ Good - aligned labels and inputs
let label_width = 12;
let input_start = 14;

for (y, field) in fields.iter().enumerate() {
    let y = (y * 2) as i16 + 2;

    dialog.add(Box::new(Label::new(
        Rect::new(2, y, label_width, y + 1),
        field.label
    )));

    dialog.add(Box::new(InputLine::new(
        Rect::new(input_start, y, 46, y + 1),
        field.max_len,
        field.data.clone()
    )));
}
```

### 3. Logical Control Groups

```rust
// ✓ Good - group related controls
dialog.add(StaticText::new(bounds, "Personal Information:"));
// ... name, email, phone

dialog.add(StaticText::new(bounds2, "Account Settings:"));
// ... username, password
```

### 4. Validate Input

```rust
// ✓ Good - validate on input
let email_input = InputLine::new(bounds, 100, data.clone())
    .with_validator(Box::new(EmailValidator::new()));

// ✓ Good - validate on submit
if result == CM_OK {
    if !is_valid_email(&email.borrow()) {
        MessageBox::error("Invalid email address").show(&mut app);
        return;
    }
}
```

### 5. Disable Unavailable Controls

```rust
// ✓ Good - disable when not applicable
if !advanced_mode {
    advanced_checkbox.set_enabled(false);
}
```

---

## Pascal vs. Rust Summary

| Concept | Pascal | Rust |
|---------|--------|------|
| **Button** | `PButton` pointer | `Button` struct |
| **Input** | `PInputLine` | `InputLine` with `Rc<RefCell<String>>` |
| **Checkbox** | `PCheckBoxes` with bitmask | `Checkbox` with `Rc<RefCell<bool>>` |
| **Radio** | `PRadioButtons` with value | `RadioButton` with `Rc<RefCell<u16>>` |
| **List Box** | `PListBox` with `PCollection` | `ListBox` with `Rc<RefCell<Vec<T>>>` |
| **Data Transfer** | `GetData`/`SetData` with structs | Shared references |
| **Shortcuts** | `~X~` syntax | Same `~X~` syntax |
| **Tab Order** | Insertion order | Insertion order (same) |

---

## Summary

### Key Concepts

1. **Controls** - Specialized views for user interaction
2. **Shared State** - `Rc<RefCell<T>>` for data binding
3. **Shortcuts** - `~X~` syntax for keyboard access
4. **Tab Order** - Insertion order determines focus cycling
5. **Validation** - Validators for input constraints
6. **Events** - Controls generate commands and broadcasts

### The Control Pattern

```rust
// 1. Define data
let data = Rc::new(RefCell::new(T::default()));

// 2. Create control with shared data
let control = Control::new(bounds, data.clone());

// 3. Add to dialog
dialog.add(Box::new(control));

// 4. Execute dialog
let result = dialog.execute(&mut app);

// 5. Read data
if result == CM_OK {
    let value = data.borrow().clone();
    process(value);
}
```

---

## See Also

- **Chapter 11** - Windows and Dialogs
- **Chapter 13** - Data Validation (upcoming)
- **Chapter 15** - Editor and Text Views (upcoming)
- **examples/dialogs_demo.rs** - Control examples
- **examples/form_demo.rs** - Data entry forms

---

Controls are the building blocks of user interaction in Turbo Vision. Master these concepts to build professional, user-friendly forms and dialogs.
