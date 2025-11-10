# Chapter 5: Creating Data-Entry Forms

Up to this point, all the objects you've used have been standard Turbo Vision objects, with the exception of the application object, which you've extended considerably. That gives you an idea of the power of Turbo Vision, but at some point you'll definitely want to create some custom functionality. In this chapter, you'll:

- Create a data-entry dialog
- Send messages between views
- Use control objects
- Validate entered data

Over the next several steps, you'll implement a simple inventory system for a small business. The program isn't meant to be truly useful, but it illustrates a lot of useful principles you will want to use in your Turbo Vision applications.

---

## Step 8: Creating a Data-Entry Dialog

Data entry usually takes place in a dialog box. In this example, the dialog box you'll create will not be modal like the standard message boxes. Rather than executing it with its own event loop, you'll add it to the desktop as a regular window. A Turbo Vision dialog is just a specialized kind of window—`Dialog` wraps a `Window`, which in turn contains a `Group` for managing child views.

Creating your data-entry dialog will happen in three parts:

- Creating a function to build the dialog
- Preventing duplicate dialogs
- Adding controls to the dialog

### Creating a Function to Build the Dialog

Because you're going to make a number of customizations to your data-entry dialog, you'll create a function that constructs and configures it. The application needs to keep track of the order dialog, so you might store a reference to it in your application state.

You'll also add a response to the menu command `CM_ORDER_WIN`, which is bound to the Examine item on the Orders menu. When you choose Orders | Examine, you want the order-entry dialog to appear, so you'll teach the application to handle that command.

First, define the custom command:

```rust
// Custom commands for order management
const CM_ORDER_WIN: u16 = 200;
const CM_ORDER_NEW: u16 = 201;
const CM_ORDER_SAVE: u16 = 202;
const CM_ORDER_CANCEL: u16 = 203;
const CM_ORDER_NEXT: u16 = 204;
const CM_ORDER_PREV: u16 = 205;
const CM_FIND_ORDER_WINDOW: u16 = 206;
```

Then create a function to build the order dialog:

```rust
use turbo_vision::views::dialog::DialogBuilder;
use turbo_vision::core::geometry::Rect;

fn create_order_dialog() -> Dialog {
    // Create dialog using builder pattern
    let mut dialog = DialogBuilder::new()
        .bounds(Rect::new(0, 0, 60, 17))
        .title("Orders")
        .build();

    // Center the dialog on the desktop
    dialog.set_centered(true);

    // Note: Controls will be added in the next section

    dialog
}
```

In your main event loop, handle the `CM_ORDER_WIN` command:

```rust
// In your main loop
if event.what == EventType::Command {
    match event.command {
        CM_ORDER_WIN => {
            open_order_window(&mut app);
            event.clear();
        }
        _ => {}
    }
}
```

And implement the `open_order_window` function:

```rust
fn open_order_window(app: &mut Application) {
    // Create and insert the dialog
    let order_dialog = create_order_dialog();
    app.desktop.add(Box::new(order_dialog));
}
```

If you run the program now, you'll notice several changes. First, if you choose Orders | Examine, a dialog box appears in the middle of the desktop, with the title "Orders". The `set_centered(true)` method ensures the dialog centers itself on the desktop.

### Limiting Open Dialogs

What happens if you choose Orders | Examine while there's already an order dialog open? The `open_order_window` function creates a new order dialog and inserts it into the desktop. Now you have two order dialogs, which is no problem for the desktop to manage, but this could cause confusion. You need to make sure you don't open a new order dialog if there's already one open. Instead, bring the open dialog to the front.

### Sending Messages

A reliable way to find out if there's an order dialog open is to use **broadcast messages**. Turbo Vision gives you the ability to send messages to views. Messages are special events, much like commands, which carry information to a specific view object and allow the receiving view to send information back.

A broadcast message is a message that gets sent to all subviews in a group. By defining a special message that only order dialogs know how to handle, you'll be able to determine whether an order dialog is open.

However, the current Rust implementation takes a simpler approach: you can track whether a dialog is open by checking the desktop's children directly. Here's a practical implementation:

```rust
use turbo_vision::views::View;

fn find_order_window(app: &Application) -> bool {
    // Check if any child of the desktop is an order dialog
    // In practice, you might use a more sophisticated approach,
    // such as storing a weak reference or using the dialog's title
    for view in app.desktop.children() {
        if view.title() == Some("Orders") {
            return true;
        }
    }
    false
}

fn open_order_window(app: &mut Application) {
    if !find_order_window(app) {
        // No order window open, create one
        let order_dialog = create_order_dialog();
        app.desktop.add(Box::new(order_dialog));
    } else {
        // Dialog already exists, bring it to front
        // The desktop automatically manages focus when you select a window
        for i in 0..app.desktop.child_count() {
            if let Some(view) = app.desktop.child_at(i) {
                if view.title() == Some("Orders") {
                    app.desktop.select_view(i);
                    break;
                }
            }
        }
    }
}
```

**Note on Broadcasting**: The original Pascal Turbo Vision had a sophisticated message broadcasting system where views could send messages to the desktop, which would forward them to all children. While the Rust implementation supports event broadcasting through `Group::broadcast()`, the pattern shown above is more idiomatic and straightforward for this use case.

### Adding Controls to the Dialog

In order to use the data-entry dialog you've created, you need to give it data-entry fields. These fields are made up of various kinds of Turbo Vision controls. Controls are the specialized views that enable users to enter or manipulate data in a dialog box, such as buttons, check boxes, and input lines.

Adding a control to a dialog takes these steps:

1. Creating shared data storage (for input fields)
2. Setting the boundaries of the control
3. Creating the control object
4. Adding the control to the dialog

Before you actually create the controls, you need to consider how data flows in and out of them. In the Rust implementation, **data is shared between the dialog and your application using `Rc<RefCell<T>>`**. This is different from the original Pascal implementation which used `SetData` and `GetData` methods.

Here's how to create a complete order dialog with various controls:

```rust
use std::rc::Rc;
use std::cell::RefCell;
use turbo_vision::views::input_line::InputLineBuilder;
use turbo_vision::views::label::LabelBuilder;
use turbo_vision::views::button::ButtonBuilder;
use turbo_vision::views::checkbox::CheckBoxBuilder;
use turbo_vision::views::radiobutton::RadioButtonBuilder;
use turbo_vision::views::static_text::StaticTextBuilder;

// Data storage for the order dialog
struct OrderData {
    order_num: Rc<RefCell<String>>,
    order_date: Rc<RefCell<String>>,
    stock_num: Rc<RefCell<String>>,
    quantity: Rc<RefCell<String>>,
    payment_method: Vec<RadioButton>,
    received: CheckBox,
}

impl OrderData {
    fn new() -> Self {
        OrderData {
            order_num: Rc::new(RefCell::new(String::new())),
            order_date: Rc::new(RefCell::new(String::new())),
            stock_num: Rc::new(RefCell::new(String::new())),
            quantity: Rc::new(RefCell::new(String::new())),
            payment_method: Vec::new(),
            received: CheckBoxBuilder::new()
                .bounds(Rect::new(0, 0, 1, 1))
                .label("Placeholder")
                .build(),
        }
    }
}

fn create_order_dialog() -> (Dialog, OrderData) {
    // Create dialog using builder pattern
    let mut dialog = DialogBuilder::new()
        .bounds(Rect::new(0, 0, 60, 17))
        .title("Orders")
        .build();
    dialog.set_centered(true);

    let mut data = OrderData::new();
    let mut y = 2;

    // Order number field
    let mut r = Rect::new(13, y, 23, y + 1);
    let order_input = InputLineBuilder::new()
        .bounds(r)
        .data(data.order_num.clone())
        .max_length(8)
        .build_boxed();
    dialog.add(order_input);

    r = Rect::new(2, y, 12, y + 1);
    let label = LabelBuilder::new()
        .bounds(r)
        .text("~O~rder #:")
        .build_boxed();
    dialog.add(label);

    // Date field
    r = Rect::new(43, y, 53, y + 1);
    let date_input = InputLineBuilder::new()
        .bounds(r)
        .data(data.order_date.clone())
        .max_length(8)
        .build_boxed();
    dialog.add(date_input);

    r = Rect::new(26, y, 41, y + 1);
    let label = LabelBuilder::new()
        .bounds(r)
        .text("~D~ate of order:")
        .build_boxed();
    dialog.add(label);

    y += 2;

    // Stock number field
    r = Rect::new(13, y, 23, y + 1);
    let stock_input = InputLineBuilder::new()
        .bounds(r)
        .data(data.stock_num.clone())
        .max_length(8)
        .build_boxed();
    dialog.add(stock_input);

    r = Rect::new(2, y, 12, y + 1);
    let label = LabelBuilder::new()
        .bounds(r)
        .text("~S~tock #:")
        .build_boxed();
    dialog.add(label);

    // Quantity field
    r = Rect::new(46, y, 53, y + 1);
    let qty_input = InputLineBuilder::new()
        .bounds(r)
        .data(data.quantity.clone())
        .max_length(5)
        .build_boxed();
    dialog.add(qty_input);

    r = Rect::new(26, y, 44, y + 1);
    let label = LabelBuilder::new()
        .bounds(r)
        .text("~Q~uantity ordered:")
        .build_boxed();
    dialog.add(label);

    y += 2;

    // Payment method label
    r = Rect::new(2, y, 21, y + 1);
    let label = LabelBuilder::new()
        .bounds(r)
        .text("~P~ayment method:")
        .build_boxed();
    dialog.add(label);

    y += 1;

    // Payment method radio buttons
    let payment_group = 1; // Group ID for radio buttons
    let mut x = 3;
    let payment_options = ["Cash", "Check", "P.O.", "Account"];

    for option in &payment_options {
        r = Rect::new(x, y, x + option.len() as i32 + 4, y + 1);
        let radio = RadioButtonBuilder::new()
            .bounds(r)
            .label(option)
            .group_id(payment_group)
            .build_boxed();
        dialog.add(radio);
        x += option.len() as i32 + 4;
    }

    y += 2;

    // Received checkbox
    r = Rect::new(22, y, 37, y + 1);
    let received = CheckBoxBuilder::new()
        .bounds(r)
        .label("~R~eceived")
        .build_boxed();
    dialog.add(received);

    y += 2;

    // Notes label
    r = Rect::new(2, y, 9, y + 1);
    let label = LabelBuilder::new()
        .bounds(r)
        .text("Notes:")
        .build_boxed();
    dialog.add(label);

    y += 1;

    // Notes field (multi-line text area)
    // Note: The current implementation doesn't have a Memo control yet,
    // but you could use a larger InputLine or implement a custom view
    r = Rect::new(3, y, 57, y + 3);
    let notes_data = Rc::new(RefCell::new(String::new()));
    let notes_input = InputLineBuilder::new()
        .bounds(r)
        .data(notes_data)
        .max_length(255)
        .build_boxed();
    dialog.add(notes_input);

    y += 4;

    // Buttons at the bottom
    r = Rect::new(2, y, 12, y + 2);
    let btn_new = ButtonBuilder::new()
        .bounds(r)
        .title("  ~N~ew  ")
        .command(CM_ORDER_NEW)
        .build_boxed();
    dialog.add(btn_new);

    r = Rect::new(13, y, 23, y + 2);
    let btn_save = ButtonBuilder::new()
        .bounds(r)
        .title("  ~S~ave  ")
        .command(CM_ORDER_SAVE)
        .default(true)
        .build_boxed();
    dialog.add(btn_save);

    r = Rect::new(24, y, 34, y + 2);
    let btn_revert = ButtonBuilder::new()
        .bounds(r)
        .title(" Re~v~ert ")
        .command(CM_ORDER_CANCEL)
        .build_boxed();
    dialog.add(btn_revert);

    r = Rect::new(35, y, 45, y + 2);
    let btn_next = ButtonBuilder::new()
        .bounds(r)
        .title("  N~e~xt  ")
        .command(CM_ORDER_NEXT)
        .build_boxed();
    dialog.add(btn_next);

    r = Rect::new(46, y, 56, y + 2);
    let btn_prev = ButtonBuilder::new()
        .bounds(r)
        .title("  ~P~rev  ")
        .command(CM_ORDER_PREV)
        .build_boxed();
    dialog.add(btn_prev);

    // Set initial focus to first input field
    dialog.set_initial_focus();

    (dialog, data)
}
```

**Important Notes**:

1. **Builder Pattern**: All controls now use the builder pattern for consistent, type-safe construction. This makes the code more readable and extensible.

2. **Tab Order**: The order in which you add controls is very important, because it determines the tab order for the dialog box. Tab order indicates where the input focus goes when the user presses Tab.

3. **Shared Data**: Input fields use `Rc<RefCell<String>>` to share data between the dialog and your application. This allows you to read and write values even while the dialog is displayed.

4. **Labels and Accelerators**: Labels can include accelerator keys (marked with `~`). When the user presses Alt+key, focus moves to the associated control.

5. **Default Buttons**: Use `.default(true)` in ButtonBuilder to mark a button as the default (activated by pressing Enter).

If you run the application now, you'll find that you have a fully functional data entry dialog. You can type data into the input lines, manipulate the radio buttons and checkboxes, and use Tab to move between fields.

**See Also**: For complete working examples, see:
- `examples/dialog_example.rs` — Basic dialog creation
- `examples/validator_demo.rs` — Dialog with all control types
- `src/views/dialog.rs` — Dialog implementation details

---

## Step 9: Setting and Reading Control Values

Now that you have a data-entry dialog, you need to be able to set initial values for the controls and read the data when the user makes changes. You've created the user interface, so now you need to manage the data flow. This step covers:

- Understanding data sharing with Rc<RefCell<T>>
- Setting initial values
- Reading values from controls
- Responding to button commands

### Understanding Data Sharing: Rc<RefCell<T>> Pattern

**Architectural Difference**: The original Pascal Turbo Vision used `SetData` and `GetData` methods to transfer data between a dialog and a record structure. The Rust implementation uses a different, more idiomatic approach: **shared ownership through `Rc<RefCell<T>>`**.

Here's how it works:

1. **Create shared data** wrapped in `Rc<RefCell<T>>`
2. **Clone the `Rc`** when creating controls (this creates a new reference, not a copy of the data)
3. **Read data** using `data.borrow()`
4. **Write data** using `data.borrow_mut()`
5. **Data is live** — changes are visible immediately to all holders of the `Rc`

This pattern is more powerful than `SetData`/`GetData` because:
- No explicit transfer needed — data is always synchronized
- Multiple views can share the same data
- Compile-time borrow checking prevents data races
- Zero-cost abstraction (no runtime overhead)

### Setting Initial Values

To set initial values for controls, you simply modify the shared data before or after creating the dialog:

```rust
fn open_order_window_with_data(app: &mut Application) {
    let (mut dialog, data) = create_order_dialog();

    // Set initial values by modifying the shared data
    *data.order_num.borrow_mut() = String::from("42");
    *data.stock_num.borrow_mut() = String::from("AAA-9999");
    *data.order_date.borrow_mut() = String::from("01/15/61");
    *data.quantity.borrow_mut() = String::from("1");

    // Set checkbox state (if you stored a reference)
    // data.received.set_checked(false);

    // Add dialog to desktop
    app.desktop.add(Box::new(dialog));
}
```

Because the `InputLine` controls share references to the same `Rc<RefCell<String>>` objects, they'll automatically display these values when the dialog appears.

### Reading Control Values

Reading values is equally straightforward — you can access the shared data at any time:

```rust
fn save_order_data(data: &OrderData) {
    // Read current values from the shared data
    let order_num = data.order_num.borrow().clone();
    let stock_num = data.stock_num.borrow().clone();
    let order_date = data.order_date.borrow().clone();
    let quantity = data.quantity.borrow().clone();

    // Get checkbox state
    let received = data.received.is_checked();

    // Save to database, file, etc.
    println!("Saving order #{}", order_num);
    println!("  Stock: {}", stock_num);
    println!("  Date: {}", order_date);
    println!("  Quantity: {}", quantity);
    println!("  Received: {}", received);
}
```

### Responding to Button Commands

In a non-modal dialog (one that's added to the desktop rather than executed with `execute()`), you need to handle button commands in your main event loop:

```rust
// In your main event loop
if event.what == EventType::Command {
    match event.command {
        CM_ORDER_SAVE => {
            save_order_data(&order_data);
            event.clear();
        }
        CM_ORDER_NEW => {
            clear_order_data(&order_data);
            event.clear();
        }
        CM_ORDER_CANCEL => {
            revert_order_data(&order_data, &original_values);
            event.clear();
        }
        _ => {}
    }
}
```

Where helper functions might look like:

```rust
fn clear_order_data(data: &OrderData) {
    *data.order_num.borrow_mut() = String::new();
    *data.stock_num.borrow_mut() = String::new();
    *data.order_date.borrow_mut() = String::new();
    *data.quantity.borrow_mut() = String::new();
}

fn revert_order_data(data: &OrderData, original: &OrderData) {
    *data.order_num.borrow_mut() = original.order_num.borrow().clone();
    *data.stock_num.borrow_mut() = original.stock_num.borrow().clone();
    *data.order_date.borrow_mut() = original.order_date.borrow().clone();
    *data.quantity.borrow_mut() = original.quantity.borrow().clone();
}
```

### Modal Dialogs vs. Non-Modal Dialogs

If you're using a **modal dialog** (one that runs its own event loop via `dialog.execute()`), the pattern is slightly different:

```rust
fn get_order_info(app: &mut Application) -> Option<OrderData> {
    let (mut dialog, data) = create_order_dialog();

    // Set initial values
    *data.order_num.borrow_mut() = String::from("42");

    // Execute the dialog modally
    let result = dialog.execute(app);

    // Check if user clicked OK/Save
    if result == CM_ORDER_SAVE {
        // Return the data
        Some(data)
    } else {
        // User cancelled
        None
    }
}
```

In this pattern, `execute()` runs until the user clicks a button, then returns the command ID. You can then decide what to do based on which button was clicked.

**Key Differences**:
- **Modal**: Blocks until closed, returns command ID
- **Non-modal**: Runs alongside other windows, commands handled in main loop

**See Also**:
- `examples/dialog_example.rs` — Modal dialog execution
- `examples/validator_demo.rs` — Complete data sharing example (lines 68-267)

---

## Step 10: Validating Data Entry

Now that you have a working data-entry dialog where you can display, enter, and change data, you can address the issue of validating that data. Validating is the process of assuring that a field contains correct data. Turbo Vision gives you the ability to validate individual fields or entire screens of data.

In general, you need to validate only input line controls—they are the only controls that allow free-form input.

Validating a data field takes only two steps:

- Assigning validator objects
- Calling validation methods

### Assigning Validator Objects

The Rust implementation provides validator objects in `src/views/validator.rs` and `src/views/picture_validator.rs`. Every `InputLine` can have a validator that checks its contents against criteria such as a numeric range, a list of allowed characters, or a "picture" format.

There are three main types of validators:

1. **FilterValidator** — Restricts input to allowed characters
2. **RangeValidator** — Validates numeric ranges
3. **PictureValidator** — Validates against a format template

#### Using FilterValidator

The simplest validator restricts input to a specific set of characters:

```rust
use turbo_vision::views::validator::{Validator, FilterValidator};

// Create an input line for numeric input only
let quantity_data = Rc::new(RefCell::new(String::new()));
let mut quantity_input = InputLine::new(
    Rect::new(46, 4, 53, 5),
    5,
    quantity_data.clone()
);

// Only allow digits
let validator = FilterValidator::new("0123456789");
quantity_input.set_validator(Box::new(validator));

dialog.add(Box::new(quantity_input));
```

Now the quantity field will only accept numeric digits.

#### Using RangeValidator

For numeric fields with specific ranges:

```rust
use turbo_vision::views::validator::RangeValidator;

// Order number must be between 1 and 99999
let order_data = Rc::new(RefCell::new(String::new()));
let mut order_input = InputLine::new(
    Rect::new(13, 2, 23, 3),
    8,
    order_data.clone()
);

let validator = RangeValidator::new(1, 99999);
order_input.set_validator(Box::new(validator));

dialog.add(Box::new(order_input));
```

The `RangeValidator` will:
- Allow only numeric input during typing
- Check that the final value is within range
- Support decimal, hexadecimal (0x prefix), and octal (0 prefix) formats

#### Using PictureValidator

For fields that must match a specific format (like dates or product codes):

```rust
use turbo_vision::views::picture_validator::PictureValidator;

// Date field: MM/DD/YY or MM/DD/YYYY
let date_data = Rc::new(RefCell::new(String::new()));
let mut date_input = InputLine::new(
    Rect::new(43, 2, 53, 3),
    10,
    date_data.clone()
);

// Picture: # = digit, {} = optional
let validator = PictureValidator::new("##/##/{##}##");
date_input.set_validator(Box::new(validator));

dialog.add(Box::new(date_input));

// Stock number field: AAA-9999 (3 letters, dash, 4 digits)
let stock_data = Rc::new(RefCell::new(String::new()));
let mut stock_input = InputLine::new(
    Rect::new(13, 4, 23, 5),
    8,
    stock_data.clone()
);

// @ = letter, # = digit
let validator = PictureValidator::new("@@@-####");
stock_input.set_validator(Box::new(validator));

dialog.add(Box::new(stock_input));
```

**Picture Validator Format Characters**:
- `#` — Requires a digit (0-9)
- `@` — Requires a letter (A-Z, a-z)
- `!` — Allows any character
- `{` `}` — Marks optional section
- Any other character — Literal (auto-inserted)

The picture validator will:
- Auto-insert literal characters (like `/` and `-`)
- Only allow valid characters in each position
- Validate the complete format when the user finishes editing

### Complete Example with Validators

Here's a complete order dialog with all validators applied:

```rust
fn create_validated_order_dialog() -> (Dialog, OrderData) {
    let bounds = Rect::new(0, 0, 60, 17);
    let mut dialog = Dialog::new(bounds, "Orders");
    dialog.set_centered(true);

    let mut data = OrderData::new();
    let mut y = 2;

    // Order number: 1-99999
    let mut r = Rect::new(13, y, 23, y + 1);
    let mut order_input = InputLine::new(r, 8, data.order_num.clone());
    order_input.set_validator(Box::new(RangeValidator::new(1, 99999)));
    dialog.add(Box::new(order_input));

    r = Rect::new(2, y, 12, y + 1);
    dialog.add(Box::new(Label::new(r, "~O~rder #:")));

    // Date: MM/DD/YY or MM/DD/YYYY
    r = Rect::new(43, y, 53, y + 1);
    let mut date_input = InputLine::new(r, 10, data.order_date.clone());
    date_input.set_validator(Box::new(PictureValidator::new("##/##/{##}##")));
    dialog.add(Box::new(date_input));

    r = Rect::new(26, y, 41, y + 1);
    dialog.add(Box::new(Label::new(r, "~D~ate:")));

    y += 2;

    // Stock number: AAA-9999
    r = Rect::new(13, y, 23, y + 1);
    let mut stock_input = InputLine::new(r, 8, data.stock_num.clone());
    stock_input.set_validator(Box::new(PictureValidator::new("@@@-####")));
    dialog.add(Box::new(stock_input));

    r = Rect::new(2, y, 12, y + 1);
    dialog.add(Box::new(Label::new(r, "~S~tock #:")));

    // Quantity: 1-99999
    r = Rect::new(46, y, 53, y + 1);
    let mut qty_input = InputLine::new(r, 5, data.quantity.clone());
    qty_input.set_validator(Box::new(RangeValidator::new(1, 99999)));
    dialog.add(Box::new(qty_input));

    r = Rect::new(26, y, 44, y + 1);
    dialog.add(Box::new(Label::new(r, "~Q~uantity:")));

    // ... rest of dialog controls ...

    dialog.set_initial_focus();
    (dialog, data)
}
```

### When Validation Occurs

Validation can occur at different times:

1. **During typing** — `is_valid_input()` checks each keystroke
2. **When focus changes** — Can validate when Tab is pressed
3. **On demand** — Explicitly call validation before saving

#### Validation During Typing

Validators automatically restrict input as the user types. For example:
- `FilterValidator` only allows specified characters
- `RangeValidator` only allows digits (and 0x/0 prefixes)
- `PictureValidator` only allows valid characters for each position

#### Validation When Focus Changes

You can force validation when the user tabs out of a field by setting a flag (this feature may need to be implemented in your custom dialog):

```rust
// Pseudocode - implementation depends on your event handling
if moving_to_next_field {
    if !input.validate() {
        // Stay on this field
        keep_focus();
    }
}
```

**Note**: Use this sparingly, as it can be intrusive. Only force validation on tab when entering invalid data would waste the user's time.

#### Validation on Demand

The most useful approach is to validate before saving:

```rust
fn save_order_data(data: &OrderData) -> Result<(), String> {
    // Validate all fields before saving
    let order_num = data.order_num.borrow();
    if order_num.is_empty() {
        return Err("Order number is required".to_string());
    }

    if let Ok(num) = order_num.parse::<i32>() {
        if num < 1 || num > 99999 {
            return Err("Order number must be between 1 and 99999".to_string());
        }
    } else {
        return Err("Order number must be numeric".to_string());
    }

    // Validate other fields...

    // All valid, proceed with save
    println!("Saving order #{}", order_num);
    Ok(())
}

// In event handler
CM_ORDER_SAVE => {
    match save_order_data(&order_data) {
        Ok(()) => {
            // Show success message
            show_message_box(&mut app, "Order saved successfully");
        }
        Err(msg) => {
            // Show error message
            show_error_box(&mut app, &msg);
        }
    }
    event.clear();
}
```

### Displaying Validation Errors

When validation fails, you should inform the user. The Rust implementation provides message box functions:

```rust
use turbo_vision::views::dialogs::{message_box, MessageBoxKind};

fn show_error_box(app: &mut Application, message: &str) {
    message_box(
        app,
        "Validation Error",
        message,
        MessageBoxKind::Error
    );
}
```

You can also create a custom validation error dialog that highlights the problematic field and provides specific guidance.

### Validator Examples

For complete, working examples of all validator types, see:
- **`examples/validator_demo.rs`** — Comprehensive demonstration showing:
  - FilterValidator for phone numbers and product codes
  - RangeValidator for age and price fields
  - PictureValidator for dates, times, and formatted codes
  - How to handle validation errors
  - Modal dialog data flow with shared state

This example (lines 68-267) is the definitive reference for implementing validated data-entry dialogs in the Rust implementation.

---

## Summary

You've now created a complete data-entry interface with proper validation. The techniques you've learned here form the foundation for building sophisticated user interfaces in Turbo Vision applications:

**Key Concepts Covered**:

1. **Dialog Creation** — Using `Dialog::new()` and adding controls
2. **Data Sharing** — Using `Rc<RefCell<T>>` for bidirectional data flow
3. **Control Types** — InputLine, Button, CheckBox, RadioButton, Label
4. **Validation** — FilterValidator, RangeValidator, PictureValidator
5. **Event Handling** — Responding to button commands and user input

**Architectural Differences from Pascal**:

| Pascal Pattern | Rust Pattern |
|----------------|--------------|
| `TOrderWindow = object(TDialog)` | `Dialog` struct (composition) |
| `SetData(OrderInfo)` | `Rc<RefCell<T>>` shared data |
| `GetData(OrderInfo)` | Direct access via `borrow()` |
| `Message(Desktop, evBroadcast, ...)` | Direct desktop child iteration |
| Type registration for streaming | No built-in persistence |

The Rust implementation emphasizes:
- **Type safety** through compile-time checking
- **Shared ownership** instead of explicit data transfer
- **Composition** over inheritance
- **Explicit state management** rather than implicit streaming

These patterns provide the same functionality as the Pascal original while leveraging Rust's modern language features for safety and performance.
