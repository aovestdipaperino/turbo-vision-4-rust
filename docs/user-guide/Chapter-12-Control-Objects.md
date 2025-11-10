# Chapter 12: Control Objects

## Scroll Bar Objects

Scroll bar objects provide several methods you can call to directly manipulate the settings of a scroll bar: `set_range`, `set_value`, and `set_params`.

- `set_range` assigns the minimum and maximum values for the scroll bar's range
- `set_value` sets the current value of the scroll bar indicator
- `set_params` is a general method that sets all parameters at once (value, min, max, page step, arrow step)

The `set_params` method takes care of setting all the parameters for the scroll bar and ensures the value is clamped within the valid range.

### Creating Scroll Bars

The Rust implementation provides two constructors for scroll bars:

```rust
use turbo_vision::views::scrollbar::ScrollBarBuilder;
use turbo_vision::core::geometry::Rect;

// Create a vertical scroll bar using the builder pattern
let vscroll = ScrollBarBuilder::new()
    .bounds(Rect::new(39, 1, 40, 21))
    .vertical()
    .build();

// Create a horizontal scroll bar
let hscroll = ScrollBarBuilder::new()
    .bounds(Rect::new(1, 21, 39, 22))
    .horizontal()
    .build();
```

### Setting Scroll Bar Parameters

The `set_params` method configures all scroll bar parameters at once:

```rust
// Set value to 5, range from 0 to 100, page step 10, arrow step 1
vscroll.set_params(5, 0, 100, 10, 1);
```

You can also set individual parameters:

```rust
// Set just the range
vscroll.set_range(0, 100);

// Set just the value
vscroll.set_value(50);
```

### Responding to Scroll Bars

When a scroll bar's value changes through user interaction, other views need to respond to that change. In Turbo Vision, scroll bars can communicate their state changes to associated views.

Views that need to respond to scroll bar changes should check for scroll bar state during their draw operations. For example, a scroller view (see `src/views/scroller.rs`) uses both horizontal and vertical scroll bars to control its viewport position.

## Using Cluster Objects

Turbo Vision provides controls that appear in clusters, or groups of related controls: check boxes and radio buttons. Although you can have a cluster that holds only one check box, in most cases you have more than one. Radio buttons are specifically designed to be used in groups where only one button can be selected at a time.

There are several tasks that apply to all cluster objects:
- Constructing a cluster control
- Checking if a button is selected
- Setting the selection state

### Working with Cluster Objects

All cluster-based controls in Turbo Vision implement the `Cluster` trait (see `src/views/cluster.rs`), which provides common functionality for:

- Managing selection state through the `ClusterState` helper struct
- Drawing controls with markers and labels
- Handling keyboard events (Space key to toggle/select)
- Providing hotkey support in labels

The `ClusterState` struct maintains the current selection value and provides methods like:

```rust
// Check if selected
if cluster_state.is_selected(item_value) {
    // Item is selected
}

// Set selection
cluster_state.set_value(1);

// Toggle selection (for checkboxes)
cluster_state.toggle();
```

### Using Check Boxes

Check boxes allow the user to toggle individual boolean options. Each check box maintains a simple on/off state (0 = unchecked, 1 = checked).

#### Creating a CheckBox

```rust
use turbo_vision::views::checkbox::CheckBoxBuilder;
use turbo_vision::core::geometry::Rect;

// Create a checkbox at position (3, 5) using the builder pattern
let checkbox = CheckBoxBuilder::new()
    .bounds(Rect::new(3, 5, 20, 6))
    .label("Enable feature")
    .build();
```

Check boxes display as:
```
[ ] Unchecked option
[X] Checked option
```

#### Working with CheckBox State

```rust
// Check if the checkbox is checked
if checkbox.is_checked() {
    // Process checked state
}

// Set the checked state
checkbox.set_checked(true);

// Toggle the state
checkbox.toggle();
```

The `CheckBox` implementation (see `src/views/checkbox.rs`) uses the `Cluster` trait to provide standard button group behavior, including:
- Space key to toggle
- Hotkey support in labels (e.g., "~E~nable feature" shows "E" as hotkey)
- Focus-based color changes

### Using Radio Buttons

Radio buttons provide mutually exclusive selection within a group. Only one radio button in a group can be selected at a time. Radio buttons with the same `group_id` form a mutually exclusive group.

#### Creating Radio Buttons

```rust
use turbo_vision::views::radiobutton::RadioButtonBuilder;
use turbo_vision::core::geometry::Rect;

// Create radio buttons in the same group (group_id = 1) using the builder pattern
let radio1 = RadioButtonBuilder::new()
    .bounds(Rect::new(3, 5, 20, 6))
    .label("Option 1")
    .group_id(1)  // group_id
    .build();

let radio2 = RadioButtonBuilder::new()
    .bounds(Rect::new(3, 6, 20, 7))
    .label("Option 2")
    .group_id(1)  // group_id
    .build();

let radio3 = RadioButtonBuilder::new()
    .bounds(Rect::new(3, 7, 20, 8))
    .label("Option 3")
    .group_id(1)  // group_id
    .build();
```

Radio buttons display as:
```
( ) Unselected option
(•) Selected option
```

#### Working with Radio Button State

```rust
// Check if selected
if radio1.is_selected() {
    // This option is selected
}

// Select a radio button
radio2.select();

// Deselect a radio button
radio1.deselect();

// Check group membership
if radio1.group_id() == radio2.group_id() {
    // They're in the same mutually exclusive group
}
```

**Note:** The current implementation requires the parent container to manage mutual exclusivity by deselecting other radio buttons in the same group when one is selected. This is typically handled at the dialog or group level.

The `RadioButton` implementation (see `src/views/radiobutton.rs`) implements the `Cluster` trait with selection behavior instead of toggle behavior.

## Picking from Lists

Turbo Vision provides objects for managing lists, including views that allow the user to choose items from lists. This section describes the list viewer trait `ListViewer` and the list box control `ListBox`.

### Working with List Viewers

The `ListViewer` trait (see `src/views/list_viewer.rs`) provides the foundational infrastructure for list-based views. It defines everything needed to view and pick from a list, including:

- Scrolling through the list
- Selecting items with keyboard and mouse
- Managing focus and viewport
- Navigation (arrows, page up/down, home/end)

The `ListViewerState` helper struct manages:
- The focused item (currently highlighted)
- The top visible item (for scrolling)
- The total number of items (range)
- Number of columns for multi-column lists

#### List Viewer Navigation

The `ListViewer` trait provides default implementations for standard list navigation:

- **Arrow keys**: Move focus up/down through items
- **Page Up/Down**: Move focus by a page at a time
- **Home/End**: Jump to first/last item
- **Mouse clicks**: Click to select an item
- **Mouse wheel**: Scroll through the list

#### Implementing a List Viewer

To create a custom list viewer, implement the `ListViewer` trait:

```rust
use turbo_vision::views::list_viewer::{ListViewer, ListViewerState};
use turbo_vision::views::view::View;

struct MyListViewer {
    // ... view fields ...
    list_state: ListViewerState,
    items: Vec<MyItemType>,
}

impl ListViewer for MyListViewer {
    fn list_state(&self) -> &ListViewerState {
        &self.list_state
    }

    fn list_state_mut(&mut self) -> &mut ListViewerState {
        &mut self.list_state
    }

    fn get_text(&self, item: usize, max_len: usize) -> String {
        // Return the text to display for the given item
        self.items.get(item)
            .map(|item| item.to_string())
            .unwrap_or_default()
    }
}
```

The `get_text` method is the key customization point - it defines how items are converted to displayable strings.

### Using List Box Controls

The `ListBox` type (see `src/views/listbox.rs`) is a useful implementation of `ListViewer` that stores its list of items as a `Vec<String>`. For many uses, you can use the default list box with no modifications.

#### Creating a ListBox

```rust
use turbo_vision::views::listbox::ListBoxBuilder;
use turbo_vision::core::geometry::Rect;
use turbo_vision::core::command::CM_OK;

// Create a list box that generates CM_OK when an item is selected using the builder pattern
let listbox = ListBoxBuilder::new()
    .bounds(Rect::new(5, 5, 34, 11))
    .on_select_command(CM_OK)
    .build();
```

#### Populating the List

```rust
// Set all items at once
listbox.set_items(vec![
    "Item 1".to_string(),
    "Item 2".to_string(),
    "Item 3".to_string(),
]);

// Add items one at a time
listbox.add_item("Item 4".to_string());

// Clear all items
listbox.clear();
```

#### Working with Selection

```rust
// Get the selected item index
if let Some(index) = listbox.get_selection() {
    println!("Selected item at index: {}", index);
}

// Get the selected item text
if let Some(text) = listbox.get_selected_item() {
    println!("Selected: {}", text);
}

// Set selection programmatically
listbox.set_selection(2);  // Select the third item

// Get total number of items
let count = listbox.item_count();
```

#### Navigation Methods

The list box provides convenience methods for navigation:

```rust
// Move selection
listbox.select_next();
listbox.select_prev();
listbox.select_first();
listbox.select_last();

// Page navigation
listbox.page_up();
listbox.page_down();
```

#### List Box Events

When the user double-clicks an item or presses Enter on the focused item, the list box generates a command event with the command ID specified in the constructor. The parent dialog or window can handle this event to process the selection.

```rust
// In the parent's event handler
if event.what == EventType::Command && event.command == CM_OK {
    if let Some(selected) = listbox.get_selected_item() {
        // Process the selected item
        process_selection(selected);
    }
}
```

The list box also responds to:
- Single-click: Focus and select the clicked item
- Double-click: Generate the selection command
- Mouse wheel: Scroll through the list

## Reading User Input

Input line objects enable the user to type and edit single lines of text. For multi-line input, use the memo field control (described in Chapter 15, "Editor and Text Views").

Input line objects support full user editing with the mouse and keyboard, including:
- Cursor movement with arrow keys, Home, and End
- Text editing with Backspace and Delete
- Text selection
- Cut, copy, and paste (Ctrl+X, Ctrl+C, Ctrl+V)
- Select all (Ctrl+A)
- Data validation of various kinds

### Creating an InputLine

Input lines use shared data through Rust's `Rc<RefCell<String>>` pattern to allow multiple views to access the same data:

```rust
use turbo_vision::views::InputLine;
use turbo_vision::core::geometry::Rect;
use std::rc::Rc;
use std::cell::RefCell;

// Create shared string data
let data = Rc::new(RefCell::new(String::new()));

// Create input line with max length of 30 characters
let input = InputLine::new(
    Rect::new(5, 2, 35, 3),
    30,
    data.clone()
);
```

### Working with InputLine Text

```rust
// Set text
input.set_text("Hello".to_string());

// Get text
let text = input.get_text();

// The shared data can also be accessed directly
let text = data.borrow().clone();
```

### Text Selection

```rust
// Select all text
input.select_all();

// Check if there's a selection
if input.has_selection() {
    // Get selected text
    if let Some(selected) = input.get_selection() {
        println!("Selected: {}", selected);
    }
}
```

### Input Validation

Input lines can have validators attached to restrict what the user can type. See Chapter 13 for details on data validation.

```rust
use turbo_vision::views::validator::ValidatorRef;

// Create input line with validator
let input = InputLine::with_validator(
    Rect::new(5, 2, 35, 3),
    30,
    data.clone(),
    validator
);

// Or set validator after creation
input.set_validator(validator);

// Check if current input is valid
if input.validate() {
    // Input is valid
}
```

### Cursor Position

The input line automatically handles cursor positioning and scrolling. When the input line has focus, it displays the cursor at the current insertion point. If the text is longer than the visible width, the view automatically scrolls horizontally, showing `<` and `>` indicators at the edges.

### Keyboard Shortcuts

The input line supports these keyboard shortcuts:

| Key | Action |
|-----|--------|
| Left/Right arrows | Move cursor |
| Home/End | Jump to start/end |
| Backspace | Delete character before cursor |
| Delete | Delete character at cursor |
| Ctrl+A | Select all |
| Ctrl+C | Copy selection to clipboard |
| Ctrl+X | Cut selection to clipboard |
| Ctrl+V | Paste from clipboard |

### Broadcast Events

Input lines can respond to broadcast events from other controls. For example, in file dialogs, the input line responds to `CM_FILE_FOCUSED` broadcasts when the user selects a file in the file list, automatically updating the displayed filename.

The input line responds to broadcasts even when not focused, but only updates its display when not actively being edited by the user.

## Summary

This chapter covered the basic control objects in Turbo Vision:

- **Scroll bars** provide value selection within a range and communicate changes to associated views
- **Cluster objects** (check boxes and radio buttons) provide single and mutually exclusive selection
- **List viewers and list boxes** provide scrollable item selection with keyboard and mouse support
- **Input lines** provide single-line text editing with validation support

All these controls implement the `View` trait and can be inserted into dialogs, windows, and other group objects. They provide standard keyboard navigation, mouse interaction, and visual feedback for focus states.

The Rust implementations use traits (`Cluster`, `ListViewer`) to share common behavior while allowing customization through trait methods. State management is handled through helper structs (`ClusterState`, `ListViewerState`) that can be embedded in custom implementations.

For more advanced controls and validation, see:
- Chapter 13: Data Validation
- Chapter 14: Palettes and Color Selection
- Chapter 15: Editor and Text Views
