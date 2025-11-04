# Chapter 5 — Managing Data Collections (Rust Edition)

**Version:** 0.2.11
**Updated:** 2025-11-04

Now that you understand how to create windows, dialogs, and data entry forms, it's time to connect them with real data. This chapter shows how to manage collections of data records and create custom views to display information.

In this chapter, you'll learn:

- Managing collections of data in Rust
- Loading and saving data with serde
- Navigating through records (next/previous)
- Adding and canceling record edits
- Creating custom views from scratch
- Implementing the View trait

---

## Understanding Pascal's Collections

Before diving into Rust solutions, let's understand what Pascal's `TCollection` provided.

### Pascal's TCollection

In Borland Pascal, `TCollection` was a dynamic array of object pointers:

```pascal
type
  PCollection = ^TCollection;
  TCollection = object (TObject)
    Items: PItemList;      { array of pointers }
    Count: Integer;         { number of items }
    Limit: Integer;         { array capacity }

    function At(Index: Integer): Pointer;
    procedure Insert(Item: Pointer);
    procedure Delete(Item: Pointer);
  end;
```

Key features:
- **Dynamic growth** - Automatically expanded when full
- **Pointer storage** - Held pointers to any object type
- **Stream support** - Could save/load entire collections
- **Type-unsafe** - Required typecasting (`POrderObj(coll.At(i))`)

### Example Pascal Usage

```pascal
// Create collection
OrderColl := New(TCollection, Init(10, 10));

// Add items
OrderColl^.Insert(New(POrderObj, Init));

// Access items (with typecast)
Order := POrderObj(OrderColl^.At(0))^;

// Save to stream
OrderFile.Init('ORDERS.DAT', stCreate, 1024);
OrderFile.Put(OrderColl);
OrderFile.Done;
```

---

## Rust Approach: Vec<T> and Type Safety

Rust provides built-in collection types that are safer and more ergonomic than Pascal's TCollection.

### Using Vec<T>

```rust
// Type-safe vector of orders
let mut orders: Vec<Order> = Vec::new();

// Add items
orders.push(Order::new());

// Access items (no typecast needed!)
let order = &orders[0];

// Iterator support
for order in &orders {
    println!("{}", order.customer);
}
```

### Why Vec<T> is Better

| Feature | Pascal TCollection | Rust Vec<T> |
|---------|-------------------|-------------|
| Type safety | Typecasts required | Compile-time type checking |
| Ownership | Manual memory management | Automatic with Drop |
| Bounds checking | Runtime errors | Panic in debug, undefined in release |
| Iterator support | Manual loops | Rich iterator API |
| Serialization | Custom stream code | Serde derives |

---

## Example: Building a Data Management System

Let's build a complete example that manages customer orders, similar to the Pascal tutorial but with modern Rust patterns.

### Step 1: Define the Data Model

```rust
use serde::{Serialize, Deserialize};
use std::path::PathBuf;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Order {
    pub order_id: u32,
    pub customer: String,
    pub product: String,
    pub quantity: u32,
    pub price: f64,
    pub date: String,
}

impl Order {
    pub fn new() -> Self {
        Self {
            order_id: 0,
            customer: String::new(),
            product: String::new(),
            quantity: 0,
            price: 0.0,
            date: String::new(),
        }
    }

    pub fn total(&self) -> f64 {
        self.quantity as f64 * self.price
    }
}

impl Default for Order {
    fn default() -> Self {
        Self::new()
    }
}
```

### Step 2: Create a Data Manager

```rust
use std::fs;
use std::io;

pub struct OrderManager {
    orders: Vec<Order>,
    current_index: usize,
    data_file: PathBuf,
    modified: bool,
}

impl OrderManager {
    pub fn new(data_file: PathBuf) -> Self {
        Self {
            orders: Vec::new(),
            current_index: 0,
            data_file,
            modified: false,
        }
    }

    /// Load orders from file
    pub fn load(&mut self) -> io::Result<()> {
        if self.data_file.exists() {
            let json = fs::read_to_string(&self.data_file)?;
            self.orders = serde_json::from_str(&json)?;
            self.modified = false;
        }
        Ok(())
    }

    /// Save orders to file
    pub fn save(&mut self) -> io::Result<()> {
        let json = serde_json::to_string_pretty(&self.orders)?;
        fs::write(&self.data_file, json)?;
        self.modified = false;
        Ok(())
    }

    /// Get current order
    pub fn current(&self) -> Option<&Order> {
        self.orders.get(self.current_index)
    }

    /// Get mutable reference to current order
    pub fn current_mut(&mut self) -> Option<&mut Order> {
        self.orders.get_mut(self.current_index)
    }

    /// Move to next order
    pub fn next(&mut self) -> bool {
        if self.current_index + 1 < self.orders.len() {
            self.current_index += 1;
            true
        } else {
            false
        }
    }

    /// Move to previous order
    pub fn prev(&mut self) -> bool {
        if self.current_index > 0 {
            self.current_index -= 1;
            true
        } else {
            false
        }
    }

    /// Add new order
    pub fn add(&mut self, order: Order) {
        self.orders.push(order);
        self.current_index = self.orders.len() - 1;
        self.modified = true;
    }

    /// Update current order
    pub fn update_current(&mut self, order: Order) {
        if let Some(current) = self.current_mut() {
            *current = order;
            self.modified = true;
        }
    }

    /// Delete current order
    pub fn delete_current(&mut self) -> Option<Order> {
        if self.current_index < self.orders.len() {
            let order = self.orders.remove(self.current_index);
            if self.current_index >= self.orders.len() && self.current_index > 0 {
                self.current_index -= 1;
            }
            self.modified = true;
            Some(order)
        } else {
            None
        }
    }

    /// Get total count
    pub fn count(&self) -> usize {
        self.orders.len()
    }

    /// Get current index (1-based for display)
    pub fn current_number(&self) -> usize {
        self.current_index + 1
    }

    /// Check if at first record
    pub fn is_first(&self) -> bool {
        self.current_index == 0
    }

    /// Check if at last record
    pub fn is_last(&self) -> bool {
        self.current_index + 1 >= self.orders.len()
    }

    /// Check if modified
    pub fn is_modified(&self) -> bool {
        self.modified
    }
}
```

### Step 3: Integrate with Turbo Vision

```rust
// tutorial_10.rs - Data collection management
use turbo_vision::app::Application;
use turbo_vision::core::geometry::Rect;
use turbo_vision::core::command::{CM_QUIT, CM_NEW};
use turbo_vision::core::event::EventType;
use turbo_vision::views::msgbox::message_box_ok;
use std::time::Duration;
use std::path::PathBuf;

// Define custom commands
const CMD_ORDER_NEXT: u16 = 2001;
const CMD_ORDER_PREV: u16 = 2002;
const CMD_ORDER_SAVE: u16 = 2003;
const CMD_ORDER_NEW: u16 = 2004;

mod order_manager;
use order_manager::{Order, OrderManager};

fn main() -> std::io::Result<()> {
    let mut app = Application::new()?;
    let (width, height) = app.terminal.size();

    // Initialize data manager
    let mut manager = OrderManager::new(PathBuf::from("orders.json"));
    if let Err(e) = manager.load() {
        message_box_ok(&mut app, "Info",
            &format!("No existing data file. Starting fresh.\n{}", e));
    }

    // Create menu bar and status line
    let menu_bar = create_menu_bar(width);
    app.set_menu_bar(menu_bar);

    let status_line = create_status_line(height, width, &manager);
    app.set_status_line(status_line);

    // Main event loop
    app.running = true;
    while app.running {
        // Draw
        app.desktop.draw(&mut app.terminal);
        if let Some(ref mut menu_bar) = app.menu_bar {
            menu_bar.draw(&mut app.terminal);
        }
        if let Some(ref mut status_line) = app.status_line {
            status_line.draw(&mut app.terminal);
        }
        let _ = app.terminal.flush();

        // Handle events
        if let Ok(Some(mut event)) = app.terminal.poll_event(Duration::from_millis(50)) {
            // Phase 1: PreProcess
            if let Some(ref mut status_line) = app.status_line {
                status_line.handle_event(&mut event);
            }

            // Menu bar
            if let Some(ref mut menu_bar) = app.menu_bar {
                menu_bar.handle_event(&mut event);
                if event.what == EventType::Keyboard || event.what == EventType::MouseUp {
                    if let Some(command) = menu_bar.check_cascading_submenu(&mut app.terminal) {
                        if command != 0 {
                            event = turbo_vision::core::event::Event::command(command);
                        }
                    }
                }
            }

            // Phase 2: Focused
            app.desktop.handle_event(&mut event);

            // Application commands
            if event.what == EventType::Command {
                match event.command {
                    CM_QUIT => {
                        if manager.is_modified() {
                            message_box_ok(&mut app, "Warning",
                                "Don't forget to save your changes!");
                        }
                        app.running = false;
                    }
                    CMD_ORDER_NEXT => {
                        if manager.next() {
                            show_current_order(&mut app, &manager);
                        }
                    }
                    CMD_ORDER_PREV => {
                        if manager.prev() {
                            show_current_order(&mut app, &manager);
                        }
                    }
                    CMD_ORDER_SAVE => {
                        if let Err(e) = manager.save() {
                            message_box_ok(&mut app, "Error",
                                &format!("Failed to save: {}", e));
                        } else {
                            message_box_ok(&mut app, "Success", "Orders saved!");
                        }
                    }
                    CMD_ORDER_NEW => {
                        let new_order = Order::new();
                        manager.add(new_order);
                        show_current_order(&mut app, &manager);
                    }
                    _ => {}
                }
            }
        }
    }

    Ok(())
}

fn show_current_order(app: &mut Application, manager: &OrderManager) {
    if let Some(order) = manager.current() {
        let msg = format!(
            "Order #{} of {}\n\n\
             Customer: {}\n\
             Product: {}\n\
             Quantity: {}\n\
             Price: ${:.2}\n\
             Total: ${:.2}",
            manager.current_number(),
            manager.count(),
            order.customer,
            order.product,
            order.quantity,
            order.price,
            order.total()
        );
        message_box_ok(app, "Current Order", &msg);
    } else {
        message_box_ok(app, "Info", "No orders in database");
    }
}

fn create_menu_bar(width: u16) -> turbo_vision::views::menu_bar::MenuBar {
    use turbo_vision::core::menu_data::{Menu, MenuItem};
    use turbo_vision::views::menu_bar::{MenuBar, SubMenu};
    use turbo_vision::core::event::KB_ALT_X;

    let mut menu_bar = MenuBar::new(Rect::new(0, 0, width as i16, 1));

    let file_items = vec![
        MenuItem::with_shortcut("~N~ew Order", CMD_ORDER_NEW, 0, "", 0),
        MenuItem::with_shortcut("~S~ave", CMD_ORDER_SAVE, 0, "", 0),
        MenuItem::separator(),
        MenuItem::with_shortcut("E~x~it", CM_QUIT, KB_ALT_X, "Alt+X", 0),
    ];
    menu_bar.add_submenu(SubMenu::new("~F~ile", Menu::from_items(file_items)));

    menu_bar
}

fn create_status_line(
    height: u16,
    width: u16,
    manager: &OrderManager,
) -> turbo_vision::views::status_line::StatusLine {
    use turbo_vision::views::status_line::{StatusLine, StatusItem};
    use turbo_vision::core::event::{KB_ALT_X, KB_F2};

    let mut items = vec![
        StatusItem::new("~Alt+X~ Exit", KB_ALT_X, CM_QUIT),
        StatusItem::new("~F2~ Save", KB_F2, CMD_ORDER_SAVE),
    ];

    if !manager.is_first() {
        items.push(StatusItem::new("~PgUp~ Prev", 0x2149, CMD_ORDER_PREV));
    }
    if !manager.is_last() {
        items.push(StatusItem::new("~PgDn~ Next", 0x2151, CMD_ORDER_NEXT));
    }

    StatusLine::new(
        Rect::new(0, height as i16 - 1, width as i16, height as i16),
        items,
    )
}
```

---

## Creating a Custom View: Counter Display

One of the most powerful features of Turbo Vision is the ability to create custom views. Let's create a counter view that displays "Record X of Y".

### Understanding Custom Views

Every custom view must:
1. Implement the `View` trait
2. Maintain its own state
3. Draw itself when requested
4. Handle events if interactive

### Step 1: Define the Counter View Structure

```rust
// counter_view.rs
use turbo_vision::core::geometry::Rect;
use turbo_vision::core::event::Event;
use turbo_vision::core::draw::DrawBuffer;
use turbo_vision::core::palette::colors;
use turbo_vision::terminal::Terminal;
use turbo_vision::views::view::{View, write_line_to_terminal};

pub struct CounterView {
    bounds: Rect,
    current: usize,
    total: usize,
}

impl CounterView {
    pub fn new(bounds: Rect) -> Self {
        Self {
            bounds,
            current: 1,
            total: 0,
        }
    }

    pub fn set_current(&mut self, current: usize) {
        self.current = current;
    }

    pub fn set_total(&mut self, total: usize) {
        self.total = total;
    }

    pub fn update(&mut self, current: usize, total: usize) {
        self.current = current;
        self.total = total;
    }
}
```

### Step 2: Implement the View Trait

```rust
impl View for CounterView {
    fn bounds(&self) -> Rect {
        self.bounds
    }

    fn set_bounds(&mut self, bounds: Rect) {
        self.bounds = bounds;
    }

    fn draw(&mut self, terminal: &mut Terminal) {
        let width = self.bounds.width() as usize;
        let mut buf = DrawBuffer::new(width);

        // Fill with spaces
        buf.move_char(0, ' ', colors::FRAME_PASSIVE, width);

        // Format the display text
        let text = if self.total > 0 {
            format!(" -{}- of {} ", self.current, self.total)
        } else {
            " No records ".to_string()
        };

        // Choose color based on state
        let color = if self.current > self.total {
            colors::ERROR_TEXT  // Highlight if out of range
        } else {
            colors::FRAME_PASSIVE
        };

        // Center the text
        let text_len = text.len();
        let start = if width > text_len {
            (width - text_len) / 2
        } else {
            0
        };

        buf.move_str(start, &text, color);

        // Write to terminal
        write_line_to_terminal(
            terminal,
            self.bounds.a.x,
            self.bounds.a.y,
            &buf
        );
    }

    fn handle_event(&mut self, _event: &mut Event) {
        // Counter doesn't handle events
    }
}
```

### Step 3: Using the Counter View

```rust
use turbo_vision::views::window::Window;

fn create_order_window_with_counter(
    manager: &OrderManager,
) -> Window {
    let mut window = Window::new(
        Rect::new(10, 5, 70, 20),
        "Order Details"
    );

    // Add counter view on the window frame
    // Position it at bottom of frame
    let counter_bounds = Rect::new(20, 14, 40, 15);
    let mut counter = CounterView::new(counter_bounds);
    counter.update(manager.current_number(), manager.count());

    window.add(Box::new(counter));

    // Add other controls...

    window
}
```

### Advanced: Interactive Counter View

Here's a more advanced counter that responds to clicks:

```rust
use turbo_vision::core::event::{EventType, MouseEvent};

pub struct ClickableCounterView {
    bounds: Rect,
    current: usize,
    total: usize,
    on_click: Option<Box<dyn Fn(usize) + Send>>,
}

impl ClickableCounterView {
    pub fn new(bounds: Rect) -> Self {
        Self {
            bounds,
            current: 1,
            total: 0,
            on_click: None,
        }
    }

    pub fn on_click<F>(mut self, callback: F) -> Self
    where
        F: Fn(usize) + Send + 'static,
    {
        self.on_click = Some(Box::new(callback));
        self
    }

    // ... set_current, set_total methods ...
}

impl View for ClickableCounterView {
    fn bounds(&self) -> Rect {
        self.bounds
    }

    fn set_bounds(&mut self, bounds: Rect) {
        self.bounds = bounds;
    }

    fn draw(&mut self, terminal: &mut Terminal) {
        // Same as before...
    }

    fn handle_event(&mut self, event: &mut Event) {
        if event.what == EventType::MouseDown {
            let mouse_pos = event.mouse.position;
            if self.bounds.contains(mouse_pos) {
                if let Some(ref callback) = self.on_click {
                    callback(self.current);
                }
                event.clear();
            }
        }
    }

    fn can_focus(&self) -> bool {
        true  // Allow focus for keyboard interaction
    }
}
```

---

## Complete Example: Order Management Application

Here's a complete, working application that combines everything:

```rust
// order_app.rs - Complete order management application
use turbo_vision::app::Application;
use turbo_vision::core::geometry::Rect;
use turbo_vision::core::command::{CM_QUIT, CM_OK};
use turbo_vision::core::event::{EventType, KB_ALT_X, KB_F2};
use turbo_vision::core::menu_data::{Menu, MenuItem};
use turbo_vision::views::menu_bar::{MenuBar, SubMenu};
use turbo_vision::views::status_line::{StatusLine, StatusItem};
use turbo_vision::views::dialog::Dialog;
use turbo_vision::views::input_line::InputLine;
use turbo_vision::views::label::Label;
use turbo_vision::views::button::Button;
use turbo_vision::views::msgbox::{message_box_ok, confirmation_box};
use std::time::Duration;
use std::path::PathBuf;

mod order_manager;
mod counter_view;

use order_manager::{Order, OrderManager};
use counter_view::CounterView;

// Commands
const CMD_ORDER_NEXT: u16 = 2001;
const CMD_ORDER_PREV: u16 = 2002;
const CMD_ORDER_SAVE: u16 = 2003;
const CMD_ORDER_NEW: u16 = 2004;
const CMD_ORDER_EDIT: u16 = 2005;
const CMD_ORDER_DELETE: u16 = 2006;

fn main() -> std::io::Result<()> {
    let mut app = Application::new()?;
    let (width, height) = app.terminal.size();

    let mut manager = OrderManager::new(PathBuf::from("orders.json"));
    let _ = manager.load();

    // Add sample data if empty
    if manager.count() == 0 {
        add_sample_data(&mut manager);
    }

    let menu_bar = create_menu_bar(width);
    app.set_menu_bar(menu_bar);

    app.running = true;
    while app.running {
        // Update status line based on current state
        let status_line = create_dynamic_status_line(height, width, &manager);
        app.set_status_line(status_line);

        // Draw
        app.desktop.draw(&mut app.terminal);
        if let Some(ref mut menu_bar) = app.menu_bar {
            menu_bar.draw(&mut app.terminal);
        }
        if let Some(ref mut status_line) = app.status_line {
            status_line.draw(&mut app.terminal);
        }
        let _ = app.terminal.flush();

        // Handle events
        if let Ok(Some(mut event)) = app.terminal.poll_event(Duration::from_millis(50)) {
            // PreProcess
            if let Some(ref mut status_line) = app.status_line {
                status_line.handle_event(&mut event);
            }

            // Menu bar
            if let Some(ref mut menu_bar) = app.menu_bar {
                menu_bar.handle_event(&mut event);
                if event.what == EventType::Keyboard || event.what == EventType::MouseUp {
                    if let Some(command) = menu_bar.check_cascading_submenu(&mut app.terminal) {
                        if command != 0 {
                            event = turbo_vision::core::event::Event::command(command);
                        }
                    }
                }
            }

            // Desktop
            app.desktop.handle_event(&mut event);

            // Commands
            if event.what == EventType::Command {
                handle_command(&mut app, &mut manager, event.command);
            }
        }
    }

    // Prompt to save on exit
    if manager.is_modified() {
        let result = confirmation_box(&mut app, "Save changes before exit?");
        if result == turbo_vision::core::command::CM_YES {
            let _ = manager.save();
        }
    }

    Ok(())
}

fn handle_command(app: &mut Application, manager: &mut OrderManager, command: u16) {
    match command {
        CM_QUIT => {
            app.running = false;
        }
        CMD_ORDER_NEXT => {
            manager.next();
        }
        CMD_ORDER_PREV => {
            manager.prev();
        }
        CMD_ORDER_SAVE => {
            if let Err(e) = manager.save() {
                message_box_ok(app, "Error", &format!("Save failed: {}", e));
            } else {
                message_box_ok(app, "Success", "Orders saved successfully!");
            }
        }
        CMD_ORDER_NEW => {
            if let Some(order) = show_order_dialog(app, None, manager) {
                manager.add(order);
                message_box_ok(app, "Success", "New order added!");
            }
        }
        CMD_ORDER_EDIT => {
            if let Some(current) = manager.current().cloned() {
                if let Some(edited) = show_order_dialog(app, Some(current), manager) {
                    manager.update_current(edited);
                    message_box_ok(app, "Success", "Order updated!");
                }
            }
        }
        CMD_ORDER_DELETE => {
            let result = confirmation_box(app, "Delete current order?");
            if result == turbo_vision::core::command::CM_YES {
                manager.delete_current();
                message_box_ok(app, "Success", "Order deleted!");
            }
        }
        _ => {}
    }
}

fn show_order_dialog(
    app: &mut Application,
    order: Option<Order>,
    manager: &OrderManager,
) -> Option<Order> {
    let mut dialog = Dialog::new(Rect::new(15, 5, 65, 18), "Order Details");

    // Add counter at top
    let counter_bounds = Rect::new(15, 0, 35, 1);
    let mut counter = CounterView::new(counter_bounds);
    counter.update(manager.current_number(), manager.count());
    dialog.add(Box::new(counter));

    // Labels and inputs
    dialog.add(Box::new(Label::new(Rect::new(2, 2, 12, 3), "Customer:")));
    let customer_input = InputLine::new(Rect::new(14, 2, 46, 3), 50);
    dialog.add(Box::new(customer_input));

    dialog.add(Box::new(Label::new(Rect::new(2, 4, 12, 5), "Product:")));
    let product_input = InputLine::new(Rect::new(14, 4, 46, 5), 50);
    dialog.add(Box::new(product_input));

    // Buttons
    let ok_btn = Button::new(Rect::new(15, 10, 25, 12), "  ~O~K  ", CM_OK, true);
    dialog.add(Box::new(ok_btn));

    let cancel_btn = Button::new(
        Rect::new(27, 10, 37, 12),
        "~C~ancel",
        turbo_vision::core::command::CM_CANCEL,
        false
    );
    dialog.add(Box::new(cancel_btn));

    // Set initial data
    if let Some(ref o) = order {
        // TODO: Set dialog data from order
    }

    let result = dialog.execute(app);

    if result == CM_OK {
        // TODO: Get data from dialog
        Some(Order::new())
    } else {
        None
    }
}

fn create_menu_bar(width: u16) -> MenuBar {
    let mut menu_bar = MenuBar::new(Rect::new(0, 0, width as i16, 1));

    let file_items = vec![
        MenuItem::with_shortcut("~N~ew", CMD_ORDER_NEW, 0, "", 0),
        MenuItem::with_shortcut("~E~dit", CMD_ORDER_EDIT, 0, "", 0),
        MenuItem::with_shortcut("~D~elete", CMD_ORDER_DELETE, 0, "", 0),
        MenuItem::separator(),
        MenuItem::with_shortcut("~S~ave", CMD_ORDER_SAVE, KB_F2, "F2", 0),
        MenuItem::separator(),
        MenuItem::with_shortcut("E~x~it", CM_QUIT, KB_ALT_X, "Alt+X", 0),
    ];
    menu_bar.add_submenu(SubMenu::new("~F~ile", Menu::from_items(file_items)));

    menu_bar
}

fn create_dynamic_status_line(height: u16, width: u16, manager: &OrderManager) -> StatusLine {
    let mut items = vec![
        StatusItem::new("~Alt+X~ Exit", KB_ALT_X, CM_QUIT),
        StatusItem::new("~F2~ Save", KB_F2, CMD_ORDER_SAVE),
    ];

    if !manager.is_first() {
        items.push(StatusItem::new("~PgUp~ Prev", 0x2149, CMD_ORDER_PREV));
    }
    if !manager.is_last() {
        items.push(StatusItem::new("~PgDn~ Next", 0x2151, CMD_ORDER_NEXT));
    }

    StatusLine::new(
        Rect::new(0, height as i16 - 1, width as i16, height as i16),
        items,
    )
}

fn add_sample_data(manager: &mut OrderManager) {
    manager.add(Order {
        order_id: 1,
        customer: "Acme Corp".to_string(),
        product: "Widget".to_string(),
        quantity: 100,
        price: 9.99,
        date: "2025-01-01".to_string(),
    });

    manager.add(Order {
        order_id: 2,
        customer: "TechCo".to_string(),
        product: "Gadget".to_string(),
        quantity: 50,
        price: 19.99,
        date: "2025-01-02".to_string(),
    });

    manager.add(Order {
        order_id: 3,
        customer: "MegaMart".to_string(),
        product: "Gizmo".to_string(),
        quantity: 200,
        price: 4.99,
        date: "2025-01-03".to_string(),
    });
}
```

---

## Summary

In this chapter, you learned:

### Data Management:
- Pascal's `TCollection` vs. Rust's `Vec<T>`
- Type-safe collection management
- Navigating through records
- Adding, updating, and deleting records
- Persistent storage with serde

### Custom Views:
- Implementing the `View` trait
- Drawing with `DrawBuffer`
- Handling events in custom views
- Creating reusable UI components

### Best Practices:
1. **Use Vec<T>** - Type-safe, no casts needed
2. **Encapsulate logic** - Create manager classes
3. **Separate concerns** - Data, UI, and business logic
4. **Handle errors** - Use Result for I/O operations
5. **Test custom views** - Ensure proper bounds and drawing

---

## See Also

- **Chapter 3** - Adding Windows (window basics)
- **Chapter 4** - Persistence and Configuration (serde patterns)
- **src/views/view.rs** - View trait documentation
- **src/views/static_text.rs** - Simple custom view example
- **examples/dialogs_demo.rs** - Complex dialog examples

---

In future chapters (if developed), you would learn about:
- Validators for data entry fields
- List boxes and selection controls
- Complex dialog layouts
- Database integration patterns
