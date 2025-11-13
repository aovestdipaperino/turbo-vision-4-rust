# Chapter 6: Managing Data Collections

**Previous:** [Chapter 5 — Creating Data Entry Forms](Chapter-5-Creating-Data-Entry-Forms.md)

---

Now that you have a working data-entry window, it makes sense to connect it with a database. Keep in mind that this example is intended to teach you about Turbo Vision, not about database management or inventory control. Some aspects of the program are necessarily simplified to allow you to focus on Turbo Vision without too much attention to the underlying database.

To connect your data-entry window with the database, you'll do the following:

- Manage a collection of data records
- Display, modify, change and add records
- Enable and disable commands as appropriate
- Create a customized view

## Step 11: Managing Collections of Data

**Progress:** Step 1 → Step 2 → Step 3 → Step 4 → Step 5 → Step 6 → Step 7 → Step 8 → Step 9 → Step 10 → **Step 11: Collections** → Step 12

Rust provides powerful collection types that handle data management efficiently. The standard `Vec<T>` type serves as the primary collection mechanism, offering type-safe, growable arrays with excellent performance characteristics.

In this step, you'll do the following:

- Create data structures
- Load data from files or initialize collections
- Display data records
- Move from record to record
- Add new records
- Cancel edits

### Understanding Collections in Turbo Vision Rust

The original Pascal Turbo Vision used `TCollection`, a specialized object-oriented collection type with stream persistence. The Rust implementation uses standard Rust collections, primarily `Vec<T>`, which provides:

- **Type Safety**: Collections are strongly typed at compile time
- **Memory Safety**: Rust's ownership system prevents common collection bugs
- **Performance**: Zero-cost abstractions with efficient memory layouts
- **Iterator Patterns**: Powerful functional-style iteration and transformation

Various parts of the codebase demonstrate collection usage:

**Menu Items** (`src/core/menu_data.rs:178-255`):
```rust
pub struct Menu {
    pub items: Vec<MenuItem>,
    pub default_index: Option<usize>,
}

impl Menu {
    pub fn add(&mut self, item: MenuItem) {
        self.items.push(item);
    }

    pub fn len(&self) -> usize {
        self.items.len()
    }
}
```

**History Lists** (`src/core/history.rs:27-96`):
```rust
pub struct HistoryList {
    items: Vec<String>,
    max_items: usize,
}

impl HistoryList {
    pub fn add(&mut self, item: String) {
        self.items.insert(0, item);  // Most recent first
        if self.items.len() > self.max_items {
            self.items.truncate(self.max_items);
        }
    }

    pub fn get(&self, index: usize) -> Option<&String> {
        self.items.get(index)
    }
}
```

**View Collections** (`src/views/group.rs:10-88`):
```rust
pub struct Group {
    children: Vec<Box<dyn View>>,
    focused: usize,
}

impl Group {
    pub fn add(&mut self, view: Box<dyn View>) {
        self.children.push(view);
    }

    pub fn child_at(&self, index: usize) -> &dyn View {
        &*self.children[index]
    }
}
```

### Creating Data Structures

For your order entry system, you need structures to hold order data. Unlike Pascal, which required wrapper objects for stream persistence, Rust lets you define your data types directly:

```rust
// Order data structure
#[derive(Clone, Debug)]
pub struct Order {
    pub customer: String,
    pub item_number: String,
    pub item_description: String,
    pub quantity: u32,
    pub unit_price: f64,
    pub total: f64,
}

impl Order {
    pub fn new() -> Self {
        Self {
            customer: String::new(),
            item_number: String::new(),
            item_description: String::new(),
            quantity: 0,
            unit_price: 0.0,
            total: 0.0,
        }
    }

    pub fn calculate_total(&mut self) {
        self.total = self.quantity as f64 * self.unit_price;
    }
}

// Application state to manage orders
pub struct OrderDatabase {
    orders: Vec<Order>,
    current_index: usize,
}

impl OrderDatabase {
    pub fn new() -> Self {
        Self {
            orders: Vec::new(),
            current_index: 0,
        }
    }

    pub fn current_order(&self) -> Option<&Order> {
        self.orders.get(self.current_index)
    }

    pub fn current_order_mut(&mut self) -> Option<&mut Order> {
        self.orders.get_mut(self.current_index)
    }

    pub fn add_order(&mut self, order: Order) {
        self.orders.push(order);
    }

    pub fn count(&self) -> usize {
        self.orders.len()
    }
}
```

**Key Points**:
- No separate "wrapper object" needed—`Order` is the data structure
- `#[derive(Clone, Debug)]` provides useful functionality automatically
- Collections own their data (Rust's ownership system manages memory)
- Methods provide controlled access to collection contents

### Loading and Initializing the Collection

The Rust implementation doesn't use the stream-based persistence system from the original Pascal version (see Chapter 4 for details on persistence approaches). Instead, you have several options for initializing your data:

**Option 1: In-Memory Initialization**
```rust
fn load_sample_orders() -> OrderDatabase {
    let mut db = OrderDatabase::new();

    db.add_order(Order {
        customer: "Acme Corp".to_string(),
        item_number: "WDG-001".to_string(),
        item_description: "Super Widget".to_string(),
        quantity: 10,
        unit_price: 49.99,
        total: 499.90,
    });

    db.add_order(Order {
        customer: "TechStart Inc".to_string(),
        item_number: "GAD-042".to_string(),
        item_description: "Digital Gadget".to_string(),
        quantity: 5,
        unit_price: 125.00,
        total: 625.00,
    });

    db
}
```

**Option 2: Plain Text File I/O**
```rust
use std::fs;
use std::io::{self, BufRead};

impl OrderDatabase {
    pub fn load_from_csv(path: &str) -> io::Result<Self> {
        let mut db = Self::new();
        let file = fs::File::open(path)?;
        let reader = io::BufReader::new(file);

        for line in reader.lines().skip(1) {  // Skip header
            let line = line?;
            let fields: Vec<&str> = line.split(',').collect();

            if fields.len() >= 5 {
                let quantity: u32 = fields[3].parse().unwrap_or(0);
                let unit_price: f64 = fields[4].parse().unwrap_or(0.0);

                let mut order = Order {
                    customer: fields[0].to_string(),
                    item_number: fields[1].to_string(),
                    item_description: fields[2].to_string(),
                    quantity,
                    unit_price,
                    total: 0.0,
                };
                order.calculate_total();
                db.add_order(order);
            }
        }

        Ok(db)
    }

    pub fn save_to_csv(&self, path: &str) -> io::Result<()> {
        let mut content = String::from("Customer,Item Number,Description,Quantity,Unit Price,Total\n");

        for order in &self.orders {
            content.push_str(&format!(
                "{},{},{},{},{:.2},{:.2}\n",
                order.customer,
                order.item_number,
                order.item_description,
                order.quantity,
                order.unit_price,
                order.total
            ));
        }

        fs::write(path, content)?;
        Ok(())
    }
}
```

**Option 3: Using Serde (if you add it to your project)**

If your application needs robust serialization, you could add the `serde` crate:

```rust
// In Cargo.toml:
// serde = { version = "1.0", features = ["derive"] }
// serde_json = "1.0"

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Order {
    // ... fields ...
}

impl OrderDatabase {
    pub fn load_from_json(path: &str) -> io::Result<Self> {
        let content = fs::read_to_string(path)?;
        let orders: Vec<Order> = serde_json::from_str(&content)?;
        Ok(Self {
            orders,
            current_index: 0,
        })
    }

    pub fn save_to_json(&self, path: &str) -> io::Result<()> {
        let content = serde_json::to_string_pretty(&self.orders)?;
        fs::write(path, content)?;
        Ok(())
    }
}
```

### Displaying a Record

Now that you have a collection of order records in memory, you can use them to provide data to the order window. Initialize your application with the order database:

```rust
pub struct TutorApp {
    desktop: Desktop,
    order_database: OrderDatabase,
    order_dialog: Option<Box<Dialog>>,
}

impl TutorApp {
    pub fn new() -> Self {
        // Load orders (using whichever method you chose)
        let order_database = load_sample_orders();
        // Or: OrderDatabase::load_from_csv("orders.csv").unwrap_or_else(|_| OrderDatabase::new());

        Self {
            desktop: Desktop::new(),
            order_database,
            order_dialog: None,
        }
    }
}
```

When opening the order window, populate it with the current order data:

```rust
fn open_order_window(&mut self) {
    if self.order_dialog.is_some() {
        // Dialog already open, just bring to front
        return;
    }

    let mut dialog = create_order_dialog();

    // Set dialog data from current order
    if let Some(order) = self.order_database.current_order() {
        set_order_dialog_data(&mut dialog, order);
    }

    self.order_dialog = Some(Box::new(dialog));
    self.desktop.add(self.order_dialog.as_ref().unwrap().clone());

    // Enable/disable navigation commands appropriately
    self.update_command_state();
}

fn set_order_dialog_data(dialog: &mut Dialog, order: &Order) {
    // Assuming you've stored references to your input controls
    // This example shows the concept - actual implementation
    // depends on how you structured your dialog

    // Find and update each control by ID
    if let Some(customer_input) = dialog.find_view_by_id(ID_CUSTOMER) {
        customer_input.set_text(&order.customer);
    }
    if let Some(item_input) = dialog.find_view_by_id(ID_ITEM_NUMBER) {
        item_input.set_text(&order.item_number);
    }
    // ... etc for other fields
}
```

### Saving the Record

When the user saves data, you need to retrieve values from the dialog and update the collection:

```rust
fn save_order_data(&mut self) {
    if let Some(dialog) = &self.order_dialog {
        // Validate dialog first
        if !dialog.valid(CM_CLOSE) {
            return;
        }

        // Get data from dialog controls
        let order = get_order_from_dialog(dialog);

        // Update the current order in the database
        if let Some(current) = self.order_database.current_order_mut() {
            *current = order;
        }

        // Save to disk if using file persistence
        let _ = self.order_database.save_to_csv("orders.csv");
    }
}

fn get_order_from_dialog(dialog: &Dialog) -> Order {
    let mut order = Order::new();

    if let Some(input) = dialog.find_view_by_id(ID_CUSTOMER) {
        order.customer = input.get_text();
    }
    if let Some(input) = dialog.find_view_by_id(ID_ITEM_NUMBER) {
        order.item_number = input.get_text();
    }
    // ... get other fields

    order.calculate_total();
    order
}
```

### Moving from Record to Record

Now that you can edit records, you need navigation commands to move between them. Define command handlers in your application's event loop:

```rust
const CM_ORDER_NEXT: u16 = 204;
const CM_ORDER_PREV: u16 = 205;

impl TutorApp {
    pub fn handle_event(&mut self, event: &mut Event) {
        match event.what {
            EventType::Command => {
                match event.command {
                    CM_ORDER_NEXT => {
                        self.show_next_order();
                        event.clear();
                    }
                    CM_ORDER_PREV => {
                        self.show_prev_order();
                        event.clear();
                    }
                    _ => {}
                }
            }
            _ => {}
        }
    }

    fn show_next_order(&mut self) {
        if self.order_database.current_index < self.order_database.count() - 1 {
            self.order_database.current_index += 1;
            self.update_order_display();
            self.update_command_state();
        }
    }

    fn show_prev_order(&mut self) {
        if self.order_database.current_index > 0 {
            self.order_database.current_index -= 1;
            self.update_order_display();
            self.update_command_state();
        }
    }

    fn update_order_display(&mut self) {
        if let Some(dialog) = &mut self.order_dialog {
            if let Some(order) = self.order_database.current_order() {
                set_order_dialog_data(dialog, order);
            }
        }
    }

    fn update_command_state(&mut self) {
        let db = &self.order_database;

        // Enable/disable previous command
        if db.current_index > 0 {
            enable_command(CM_ORDER_PREV);
        } else {
            disable_command(CM_ORDER_PREV);
        }

        // Enable/disable next command
        if db.current_index < db.count() - 1 {
            enable_command(CM_ORDER_NEXT);
        } else {
            disable_command(CM_ORDER_NEXT);
        }
    }
}
```

By enabling and disabling commands at the right times, your response methods don't have to check whether it's appropriate to respond. This is good for two reasons:

- **Simpler Code**: If `CM_ORDER_NEXT` is always disabled when you're viewing the last order, your response to `CM_ORDER_NEXT` doesn't have to check if there's a next order.
- **Better UX**: Users know what's happening. It's much better to disable an inappropriate command than to offer it and then either ignore it or display an error message.

### Adding New Records

Adding a new record involves creating an empty `Order`, displaying it in the dialog, and inserting it into the collection when saved:

```rust
const CM_ORDER_NEW: u16 = 201;

impl TutorApp {
    fn enter_new_order(&mut self) {
        // Make sure dialog is open
        if self.order_dialog.is_none() {
            self.open_order_window();
        }

        // Create new empty order
        let new_order = Order::new();

        // Display it in the dialog
        if let Some(dialog) = &mut self.order_dialog {
            set_order_dialog_data(dialog, &new_order);
        }

        // Set the index to point past the last record
        // This signals we're adding a new record
        self.order_database.current_index = self.order_database.count();

        // Disable navigation while entering new record
        disable_command(CM_ORDER_NEXT);
        disable_command(CM_ORDER_PREV);
        disable_command(CM_ORDER_NEW);
    }

    fn save_order_data(&mut self) {
        if let Some(dialog) = &self.order_dialog {
            if !dialog.valid(CM_CLOSE) {
                return;
            }

            let order = get_order_from_dialog(dialog);

            // Check if this is a new order or an update
            if self.order_database.current_index >= self.order_database.count() {
                // New order - add to collection
                self.order_database.add_order(order);
                // Keep current_index pointing to the new order
            } else {
                // Existing order - update it
                if let Some(current) = self.order_database.current_order_mut() {
                    *current = order;
                }
            }

            // Save to disk
            let _ = self.order_database.save_to_csv("orders.csv");

            // Re-enable commands
            self.update_command_state();
            enable_command(CM_ORDER_NEW);
        }
    }
}
```

Notice the pattern: when adding a new record, you set `current_index` to equal the collection count (pointing past the end). This signals that you're in "new record" mode. When saving, you check this condition to decide whether to add or update.

### Canceling Edits

One last feature is the ability to cancel changes, either when modifying an existing record or when adding a new one:

```rust
const CM_ORDER_CANCEL: u16 = 203;

impl TutorApp {
    fn cancel_order(&mut self) {
        let db = &mut self.order_database;

        if db.current_index < db.count() {
            // Existing order - just reload the data
            self.update_order_display();
        } else {
            // New order being added - go back to last record
            if db.count() > 0 {
                db.current_index = db.count() - 1;
                self.update_order_display();
            } else {
                // No orders at all - show empty dialog
                if let Some(dialog) = &mut self.order_dialog {
                    let empty_order = Order::new();
                    set_order_dialog_data(dialog, &empty_order);
                }
            }
        }

        // Re-enable commands
        self.update_command_state();
        enable_command(CM_ORDER_NEW);
    }
}
```

---

## Step 12: Creating a Custom View

**Progress:** Step 1 → Step 2 → Step 3 → Step 4 → Step 5 → Step 6 → Step 7 → Step 8 → Step 9 → Step 10 → Step 11 → **Step 12: Custom View**

One thing you've probably noticed in using this simple database is that you can't tell which record you're looking at (unless it happens to be the first or last record) or how many total records exist. A much nicer way to handle this is to show the user the number of the current record and the total record count. Since Turbo Vision doesn't provide such a view for you, you'll create one yourself.

To create your view, you'll do the following:

- Create the internal counting engine
- Construct the view
- Give the view its appearance
- Add the view to the order window

These steps are universal to all custom views. Every view must be able to:

- Cover its full rectangular area
- Respond to events in that area
- Draw itself on the screen when told to
- Perform any internal functions

### Understanding the View Trait

In Rust Turbo Vision, all views implement the `View` trait (`src/core/view.rs`). This trait defines the essential methods every view must provide:

```rust
pub trait View {
    // Required methods
    fn bounds(&self) -> Rect;
    fn set_bounds(&mut self, bounds: Rect);
    fn draw(&mut self, terminal: &mut Terminal);
    fn handle_event(&mut self, event: &mut Event);

    // Optional methods with default implementations
    fn can_focus(&self) -> bool { false }
    fn focused(&self) -> bool { false }
    fn set_focused(&mut self, _focused: bool) {}
    // ... many more
}
```

### Creating the Counting Engine

The counter view needs to track two pieces of data: the current record number and the total number of records. You'll create a struct with these fields and methods to manipulate them:

```rust
use turbo_vision::core::geometry::Rect;
use turbo_vision::core::view::View;
use turbo_vision::core::event::Event;
use turbo_vision::terminal::Terminal;
use turbo_vision::core::draw_buffer::DrawBuffer;
use std::cell::Cell;

pub struct CountView {
    bounds: Rect,
    current: Cell<usize>,
    count: Cell<usize>,
}

impl CountView {
    pub fn new(bounds: Rect) -> Self {
        Self {
            bounds,
            current: Cell::new(1),
            count: Cell::new(0),
        }
    }

    pub fn set_count(&self, new_count: usize) {
        self.count.set(new_count);
        // In a real implementation, you'd call draw_view() here
        // to trigger a redraw
    }

    pub fn inc_count(&self) {
        self.set_count(self.count.get() + 1);
    }

    pub fn dec_count(&self) {
        if self.count.get() > 0 {
            self.set_count(self.count.get() - 1);
        }
    }

    pub fn set_current(&self, new_current: usize) {
        self.current.set(new_current);
        // Trigger redraw
    }

    pub fn inc_current(&self) {
        self.set_current(self.current.get() + 1);
    }

    pub fn dec_current(&self) {
        if self.current.get() > 0 {
            self.set_current(self.current.get() - 1);
        }
    }
}
```

**Key Points**:
- We use `Cell<usize>` for interior mutability—this allows updating the counts from methods that take `&self`
- The methods provide a clean API for manipulating the counter state
- After changing values, we should trigger a redraw (implementation details depend on your application structure)

### Drawing the View

Every view must implement the `draw` method, which renders the view's current state to the terminal. For the counter view, we'll display something like "- 3 - of 10":

```rust
impl View for CountView {
    fn bounds(&self) -> Rect {
        self.bounds
    }

    fn set_bounds(&mut self, bounds: Rect) {
        self.bounds = bounds;
    }

    fn draw(&mut self, terminal: &mut Terminal) {
        let width = self.bounds.width() as usize;
        let current = self.current.get();
        let count = self.count.get();

        // Create the display string
        let display = format!(" -{}- of {} ", current, count);

        // Choose color based on whether we're out of range
        let color = if current > count {
            // Highlight if current exceeds count (adding new record)
            0x4E  // Red background
        } else {
            0x1F  // Normal colors
        };

        // Create a draw buffer for the line
        let mut buf = DrawBuffer::new(width);

        // Fill with spaces first
        buf.move_char(0, ' ', color, width);

        // Center the display string
        let start_pos = if display.len() < width {
            (width - display.len()) / 2
        } else {
            0
        };

        // Write the display string
        buf.move_str(start_pos, &display, color);

        // Write to terminal
        write_line_to_terminal(
            terminal,
            self.bounds.a.x,
            self.bounds.a.y,
            &buf
        );
    }

    fn handle_event(&mut self, _event: &mut Event) {
        // Counter view is display-only, doesn't handle events
    }

    fn can_focus(&self) -> bool {
        false  // Counter can't receive focus
    }
}
```

### Real-World Example: Broadcast Demo Counter

The codebase includes a practical example in `examples/broadcast_demo.rs:26-116`, which shows a button with counter displays:

```rust
struct BroadcastButton {
    bounds: Rect,
    label: String,
    command: CommandId,
    broadcast_count: Cell<u32>,  // Counter tracking broadcasts
    click_count: Cell<u32>,      // Counter tracking clicks
}

impl View for BroadcastButton {
    fn draw(&mut self, terminal: &mut Terminal) {
        let width = self.bounds.width() as usize;
        let height = self.bounds.height();

        for y in 0..height {
            let mut buf = DrawBuffer::new(width);

            if y == 0 {
                // Draw label
                let text = format!("{:^width$}", self.label);
                buf.move_str(0, &text, 0x1F);
            } else if y == 1 {
                // Draw click counter
                let text = format!("{:^width$}",
                    format!("Clicks: {}", self.click_count.get()));
                buf.move_str(0, &text, 0x1E);
            } else if y == 2 {
                // Draw broadcast counter
                let text = format!("{:^width$}",
                    format!("RX: {}", self.broadcast_count.get()));
                buf.move_str(0, &text, 0x1D);
            }

            write_line_to_terminal(terminal,
                self.bounds.a.x,
                self.bounds.a.y + y as i16,
                &buf);
        }
    }

    fn handle_event(&mut self, event: &mut Event) {
        match event.what {
            EventType::MouseDown => {
                // Increment click counter
                self.click_count.set(self.click_count.get() + 1);
                *event = Event::command(self.command);
            }
            EventType::Broadcast => {
                if event.command == CMD_BROADCAST_TEST {
                    // Increment broadcast counter
                    self.broadcast_count.set(
                        self.broadcast_count.get() + 1
                    );
                }
            }
            _ => {}
        }
    }
}
```

This example demonstrates several important patterns:
- Using `Cell<T>` for mutable counters in an immutable context
- Drawing multiple lines with different content and colors
- Responding to both user events and broadcast events
- Combining display and interactive functionality

### Adding the Counter to Your Dialog

To add the counter view to your order dialog, create it during dialog construction:

```rust
fn create_order_dialog(order_count: usize) -> Dialog {
    // Create dialog using the builder pattern
    let mut dialog = DialogBuilder::new()
        .bounds(Rect::new(0, 0, 60, 17))
        .title("Orders")
        .build();

    // Add input controls here...
    // (customer, item number, etc.)

    // Add counter view at the top of the frame
    let counter_bounds = Rect::new(5, 0, 20, 1);  // Top of dialog
    let counter = CountView::new(counter_bounds);
    counter.set_count(order_count);
    counter.set_current(1);

    dialog.insert(Box::new(counter));

    dialog
}
```

### Manipulating the Counter

Update the counter whenever you navigate records or add new ones:

```rust
impl TutorApp {
    fn update_order_display(&mut self) {
        if let Some(dialog) = &mut self.order_dialog {
            // Update dialog controls with order data
            if let Some(order) = self.order_database.current_order() {
                set_order_dialog_data(dialog, order);
            }

            // Update counter display
            if let Some(counter) = dialog.find_view_by_type::<CountView>() {
                counter.set_current(self.order_database.current_index + 1);
                counter.set_count(self.order_database.count());
            }
        }
    }

    fn save_order_data(&mut self) {
        if let Some(dialog) = &self.order_dialog {
            if !dialog.valid(CM_CLOSE) {
                return;
            }

            let order = get_order_from_dialog(dialog);

            if self.order_database.current_index >= self.order_database.count() {
                // New order - add to collection
                self.order_database.add_order(order);

                // Update counter to reflect new total
                if let Some(counter) = dialog.find_view_by_type::<CountView>() {
                    counter.inc_count();
                }
            } else {
                // Update existing order
                if let Some(current) = self.order_database.current_order_mut() {
                    *current = order;
                }
            }

            self.update_command_state();
        }
    }
}
```

---

## Where to Now?

There are many additions and changes you could make to your tutorial application to make it more useful. Here are some suggestions:

### Additional Features to Consider

**Multiple Dialog Types**
- Implement modal dialogs for supplier and stock item databases
- Use different dialog types for different data entry needs
- Modal dialogs can handle their own command responses

**Lookup Validation**
- Create validators that check against existing data
- Ensure stock numbers match actual inventory items
- Validate supplier IDs against a supplier database
- See Chapter 5 for validator patterns

**Sorted Collections**
The codebase includes `SortedListBox` (`src/views/sorted_listbox.rs:30-232`), which demonstrates maintaining a collection in sorted order:

```rust
pub struct SortedListBox {
    items: Vec<String>,
    // ... other fields
}

impl SortedListBox {
    pub fn add_item(&mut self, item: String) {
        // Binary search to find insertion point
        let insertion_point = self.find_insertion_point(&item);
        self.items.insert(insertion_point, item);
    }

    pub fn find_exact(&self, text: &str) -> Option<usize> {
        // Binary search for exact match
        self.items.binary_search_by(|item|
            item.as_str().cmp(text)
        ).ok()
    }
}
```

This pattern is useful for:
- Maintaining alphabetically sorted item lists
- Enabling fast lookups using binary search
- Providing type-ahead functionality

**Search and Filter**
- Add search functionality to find records quickly
- Filter views to show subsets of data
- Implement incremental search patterns

**Data Validation**
- Add comprehensive validators for all fields
- Implement cross-field validation (e.g., quantity × price = total)
- Provide helpful error messages for validation failures

**Undo/Redo**
The editor implementation (`src/views/editor.rs`) demonstrates undo/redo patterns:

```rust
pub struct Editor {
    undo_stack: Vec<EditorAction>,
    redo_stack: Vec<EditorAction>,
    // ... other fields
}

impl Editor {
    pub fn undo(&mut self) {
        if let Some(action) = self.undo_stack.pop() {
            // Apply reverse of action
            // Push to redo stack
            self.redo_stack.push(action);
        }
    }
}
```

You could adapt this pattern for undoing record edits.

---

## Summary

In this chapter, you learned how to:

- **Manage collections** using Rust's `Vec<T>` type instead of Pascal's `TCollection`
- **Structure data** with Rust's struct types and derive traits
- **Load and save data** using plain text I/O or optionally serde
- **Navigate records** with proper command state management
- **Add and cancel records** with clear state tracking
- **Create custom views** by implementing the `View` trait
- **Display dynamic information** using `Cell<T>` for interior mutability

The Rust implementation differs from the original Pascal in several key ways:

| Aspect | Pascal | Rust |
|--------|--------|------|
| Collections | `TCollection` with type-unsafe pointers | `Vec<T>` with compile-time type safety |
| Persistence | Binary streams with registration | Plain text or serde (opt-in) |
| Memory | Manual `Dispose()` calls | Automatic with ownership system |
| Type Safety | Runtime type checking | Compile-time guarantees |
| Mutability | Mutable by default | Explicit `mut`, `Cell`, `RefCell` |

These changes make the Rust version more robust and safer while maintaining the same conceptual patterns and user experience.

---

**End of Chapter 6**

---

**Next:** [Chapter 7 — Architecture Overview](Chapter-7-Architecture-Overview.md)
