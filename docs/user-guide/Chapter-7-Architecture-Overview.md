# Chapter 7: Architecture Overview

## Introduction

This chapter assumes that you have a good working knowledge of Rust, especially traits, trait objects, and composition patterns. It also assumes that you have read Part 1 of this book to get an overview of Turbo Vision's philosophy, capabilities, and terminology.

The remainder of this part describes the methods and fields of each standard object type in depth, but until you acquire an overall feel for how the architecture is structured, you can easily become overwhelmed by the mass of detail. This chapter presents an informal browse through the architecture before you tackle the detail. The remainder of this part will give more detailed explanations of the components of Turbo Vision and how to use them.

The view architecture is built around the `View` trait and composition. Understanding that `Dialog`, for example, composes a `Window`, which contains a `Group` for its interior, which in turn owns a collection of child views, reduces the learning curve considerably. Each new view type you encounter shares the common `View` trait interface and may compose other view types to build its functionality.

As you develop your own Turbo Vision applications, you'll find that a general familiarity with the standard view types and their mutual relationships is an enormous help. Mastering the minute details will come later, but as with all well-designed architectures, the initial overall planning of your new views is the key to success.

Each group is described in a separate section of this chapter. Within each of these groups there are also different sorts of views. Some are useful views—you can create instances of them and use them. Others are abstract traits or types that serve as the basis for deriving related, useful views. Before looking at the views in the Turbo Vision architecture, it will help to understand a little about how the architecture is organized.

---

## Working with the View Architecture

This section describes some of the basic properties of the Turbo Vision architecture in Rust, specifically applied to the view system. The topics covered are:

- Basic view operations
- Traits and composition
- Types of methods

### Basic View Operations

Given any view type there are two basic things you can do:

- Compose it within a larger view type
- Create an instance of that type ("instantiate" it)

If you compose a view type within a larger type, you have a new view type on which the previous two operations again apply. The next sections examine both of these operations, then explore the use of abstract types.

#### Composition

Unlike traditional object-oriented programming with inheritance, Rust favors **composition over inheritance**. When you want to extend or change an existing view type, you typically create a new struct that contains the existing type:

```rust
pub struct Dialog {
    window: Window,     // Compose a Window
    result: CommandId,  // Add new fields
}

impl View for Dialog {
    fn draw(&mut self, terminal: &mut Terminal) {
        self.window.draw(terminal);  // Delegate to composed type
    }

    fn handle_event(&mut self, event: &mut Event) {
        // Add or override behavior
        if matches!(event.what, EventType::Keyboard) {
            if event.key_code == KEY_ENTER {
                // Custom behavior for Enter key
            }
        }
        self.window.handle_event(event);  // Delegate
    }
}
```

In defining your new view, you can do three things:

- Add new fields
- Implement trait methods
- Delegate to composed types or add custom behavior

The new methods and fields you define add functionality to the composed type. New view types nearly always need to implement the `View` trait to determine their default values and properties.

#### Instantiation

Creating an instance of a view is accomplished by calling its constructor (typically `new()`) and optionally wrapping it in a `Box` for dynamic dispatch:

```rust
// Create using the builder pattern
let button = ButtonBuilder::new()
    .bounds(Rect::new(2, 2, 12, 4))
    .title("OK")
    .command(CM_OK)
    .default(true)
    .build();

// Create a boxed instance (heap-allocated for polymorphism)
let button: Box<dyn View> = ButtonBuilder::new()
    .bounds(Rect::new(2, 2, 12, 4))
    .title("OK")
    .command(CM_OK)
    .default(true)
    .build_boxed();
```

The button would be initialized with certain default field values set by the builder. Since `Button` implements the `View` trait, it provides all the methods defined in that trait.

To make use of the button, you need to know what its methods do. If the required functionality is not defined in `Button`, you need to create a new type that composes `Button` and adds the needed behavior.

Whether you can create a useful instance of a view type depends on what kind of trait methods it implements. Some of Turbo Vision's standard types provide minimal default implementations that may need to be customized.

#### Abstract Types

Many types exist as "abstract" bases from which you can compose more specialized, useful view types. The reason for having abstract types is partly conceptual but serves the practical aim of reducing coding effort.

For example, the `RadioButtons` and `CheckBoxes` types could each implement the `View` trait independently without difficulty. However, they share a great deal in common. They both represent sets of controls with similar responses. A set of radio buttons is a lot like a set of check boxes in which only one box can be checked, although there are other differences. This commonality warrants creating an abstract trait called `Cluster`. `RadioButtons` and `CheckBoxes` both implement `Cluster` and share the `ClusterState` struct, with the addition of a few specialized methods to provide their individual functionalities.

It's never useful, and often not possible, to create an instance of an abstract trait. For example, you cannot instantiate the `Cluster` trait directly—you must use `RadioButtons` or `CheckBoxes`.

If you want a fancy cluster of controls with properties different from radio buttons or check boxes, you might try creating a new type that implements the `Cluster` trait, or it might be easier to compose `RadioButtons` or `CheckBoxes`, depending on which is closer to your needs. In all cases, you add fields and implement or delegate trait methods with the least possible effort. If your plans include a whole family of fancy clusters, you might find it convenient to create an intermediate trait.

### Composing Views

If you take an important trio of types: `View` (trait), `Group`, and `Window`, a glance at their fields reveals composition at work, and also tells you quite a bit about the growing functionality.

**Figure 7.2: Window Composition**

| View (trait) | Group | Window |
|---|---|---|
| bounds() | bounds: Rect | bounds: Rect |
| state() | children: Vec<Box<dyn View>> | frame: Frame |
| options() | focused: usize | interior: Group |
| draw() | background: Option<Attr> | state: StateFlags |
| handle_event() | end_state: CommandId | drag_offset: Option<Point> |
| | | number: usize |
| | | title: String |

Notice that `Group` implements the `View` trait and adds several fields that are pertinent to group operation, such as a vector of child views and a focused index. `Window` in turn composes both `Frame` and `Group` (in its `interior` field) and adds yet more fields which are needed for window operation, such as the title and number of the window.

In order to fully understand `Window`, you need to keep in mind that a window composes a group (in its `interior` field) and implements the view trait.

### Types of Methods

Turbo Vision methods can be characterized in several ways:

- Trait methods with default implementations
- Trait methods that must be implemented
- Inherent methods (specific to a type)
- Generic methods

#### Inherent Methods

An inherent method is specific to a type and cannot be called through a trait object. These methods are defined directly on the struct:

```rust
impl Button {
    pub fn new(bounds: Rect, title: &str, command: CommandId, is_default: bool) -> Self {
        // Specific to Button, not part of View trait
        Self { bounds, title: title.to_string(), command, is_default, state: 0 }
    }
}
```

#### Trait Methods with Default Implementations

Many trait methods provide default implementations that can be overridden:

```rust
pub trait View {
    fn update_cursor(&self, _terminal: &mut Terminal) {
        // Default: do nothing
    }

    fn can_focus(&self) -> bool {
        // Default: cannot focus
        false
    }
}
```

These methods need not be implemented, but the usual intention is that they will be overridden when appropriate.

#### Required Trait Methods

Some trait methods have no default implementation and must be provided by implementors:

```rust
pub trait View {
    fn bounds(&self) -> Rect;  // Must implement
    fn set_bounds(&mut self, bounds: Rect);  // Must implement
    fn draw(&mut self, terminal: &mut Terminal);  // Must implement
    fn handle_event(&mut self, event: &mut Event);  // Must implement
}
```

Views with minimal implementations of these methods might be considered truly abstract—you must create a more specialized type before you can create a useful instance.

#### Pseudo-Abstract Methods

Unlike truly abstract methods that must be implemented, pseudo-abstract methods offer minimal default actions or no actions at all. They serve as placeholders where you can insert code in your view types.

For example, the `View` trait introduces a method called `update_cursor`. It contains no code by default:

```rust
pub trait View {
    fn update_cursor(&self, _terminal: &mut Terminal) {
        // Default: do nothing
    }
}
```

By default, `update_cursor` therefore does nothing. Views that manage a cursor (like `InputLine`) can override this method to position and show the cursor.

---

## View Typology

Not all view types are created equal in Turbo Vision. You can separate their functions into four distinct groups:

- Primitive types
- Views
- Group views
- Engines

### Primitive Types

Turbo Vision provides simple types that exist primarily to be used by other views:

- `Point` (`src/core/geometry.rs`)
- `Rect` (`src/core/geometry.rs`)

These types are used by all the visible views in Turbo Vision. Views of these types are not displayable.

#### Point

This struct represents a point. Its fields, `x` and `y`, define the Cartesian (x,y) coordinates of a screen position. The point (0,0) is the top left corner of the screen. X increases horizontally to the right; Y increases vertically downwards. `Point` has no methods beyond constructors.

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Point {
    pub x: i16,
    pub y: i16,
}
```

#### Rect

This struct represents a rectangle. Its fields, `a` and `b`, are `Point` values defining the rectangle's upper left and lower right points. `Rect` has methods `width()`, `height()`, `contains()`, `intersects()`, `union()`, `grow()`, `move_by()`, and others. `Rect` values are not visible views and can't draw themselves. However, all views are rectangular: Their constructors all take a `Rect` parameter to determine the region they will cover.

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Rect {
    pub a: Point,  // top-left (inclusive)
    pub b: Point,  // bottom-right (exclusive)
}

impl Rect {
    pub fn width(&self) -> i16 { self.b.x - self.a.x }
    pub fn height(&self) -> i16 { self.b.y - self.a.y }
    pub fn contains(&self, p: Point) -> bool { /* ... */ }
    // ... more methods
}
```

### Views

The displayable types implement the `View` trait, defined in `src/views/view.rs`. You should distinguish "visible" from "displayable," since there may be times when a view is wholly or partly hidden by other views. A view is any type that can be displayed in a rectangular portion of the screen.

The `View` trait itself provides an interface for an empty rectangular screen area. Implementing the `View` trait ensures that each view has at least a rectangular portion of the screen and a `draw()` method that can render it.

Turbo Vision includes the following standard views:

- Frames
- Buttons
- Clusters
- Menus
- Histories
- Input lines
- List viewers
- Scrollers
- Scroll bars
- Text devices
- Static text
- Labels
- Status lines

#### Frames

`Frame` (`src/views/frame.rs`) provides the displayable frame (border) for a `Window` together with icons for moving and closing the window. `Frame` instances are never used on their own, but always composed within a `Window`.

#### Buttons

A `Button` (`src/views/button.rs`) is a titled box used to generate a specific command event when "pushed." They are usually placed inside (owned by) dialog boxes, offering such choices as "OK" or "Cancel." The dialog box is usually the modal view when it appears, so it traps and handles all events, including its button events. The event handler offers several ways of pushing a button: mouse-clicking in the button's rectangle, typing the shortcut letter, or selecting the default button with the Enter key.

#### Clusters

`Cluster` is a trait (`src/views/cluster.rs`) used to implement check boxes and radio buttons. A cluster is a group of controls that all respond in the same way. Cluster controls are often associated with `Label` instances, letting you select the control by selecting the adjacent text label.

Radio buttons are special clusters in which only one control can be selected. Each subsequent selection deselects the current one (as with a car radio station selector). Check boxes are clusters in which any number of controls can be marked (selected).

#### Menus

`MenuBar` and related types (`src/views/menu_bar.rs`) provide the basic types for creating pull-down menus and submenus nested to any level. You supply text strings for the menu selections (with optional highlighted shortcut letters) together with the commands associated with each selection.

By default, Turbo Vision applications reserve the top line of the screen for a menu bar, from which menu boxes drop down. You can also create menu boxes that pop up in response to mouse clicks. Menus are explained in Chapter 10, "Application objects."

#### Histories

History support (`src/views/history_viewer.rs`) implements a generic pick-list mechanism. Histories work with input lines to provide a dropdown list of previously entered values.

#### Input Lines

`InputLine` (`src/views/input_line.rs`) provides a basic input line string editor. It handles all the usual keyboard entries and cursor movements. It offers deletes and inserts, selectable insert and overwrite modes, and automatic cursor shape control. Input lines support data validation with validator objects.

#### List Viewers

The `ListViewer` type (`src/views/list_viewer.rs`) serves as a base for list viewing functionality. List viewers let you display collections of items with control over scrolling. The commonly used list viewer displays lists of strings. List viewers are explained in Chapter 12, "Control objects."

#### Scrolling Views

A `Scroller` is a scrollable view that serves as a portal onto another larger "background" view. Scrolling occurs in response to keyboard input or actions in associated scroll bar objects.

#### Scroll Bars

`ScrollBar` (`src/views/scrollbar.rs`) provides either vertical or horizontal scroll controls. Windows containing scrolling interiors use scroll bars to control the scroll position. List viewers also use scroll bars.

#### Text Devices

Text device types provide scrollable TTY-type text viewing/device driver functionality. These types exist as a base for deriving real terminal drivers. They essentially provide text file device drivers that write to a view.

#### Static Text

`StaticText` instances are simple views used to display fixed strings. They ignore any events sent to them. The `Label` type adds the property that the view holding the text, known as a label, can be selected (highlighted) by mouse click, cursor key, or shortcut Alt+letter keys. Labels are associated with another view, usually a control view. Selecting the label selects the linked control and selecting the linked control highlights the label as well.

#### Status Lines

A `StatusLine` (`src/views/status_line.rs`) is intended for various status and hint displays, usually at the bottom line of the screen. A status line is a one-character high strip of any length up to the screen width. The type offers dynamic displays reacting with events in the unfolding application. Status lines are explained in Chapter 10, "Application objects."

### Group Views

The importance of the `View` trait is apparent from the architecture. Everything you can see in a Turbo Vision application implements `View` in some way. But some of those visible types are also important for another reason: They allow views to act in concert.

Turbo Vision includes the following standard group views:

- Groups
- Applications
- Desktops
- Windows
- Dialog boxes

#### Groups

`Group` (`src/views/group.rs`) lets you handle dynamically changing collections of related, interacting child views via a vector of trait objects. Since a group itself implements `View`, there can be children that are in turn groups owning their own children, and so on. The state of the collection is constantly changing as the user clicks and types during an application. New groups can be created and child views can be added to (inserted) and removed from a group. Groups and child views are explained in Chapter 8, "Views."

#### Applications

`Application` (`src/app/application.rs`) provides a program template for your Turbo Vision application. Typically, it owns `MenuBar`, `Desktop`, and `StatusLine` instances. `Application` has methods for creating and managing these components. The key method of `Application` is `run()`, which executes the application's main event loop. Application objects are explained in Chapter 10, "Application objects."

#### Desktops

`Desktop` (`src/views/desktop.rs`) is the normal startup background view, providing the familiar user's desktop, usually surrounded by a menu bar and status line. Other views (such as windows and dialog boxes) are created, displayed, and manipulated in the desktop in response to user actions (mouse and keyboard events). Most of the actual work in an application goes on inside the desktop. Desktop objects are explained in Chapter 10, "Application objects."

#### Windows

`Window` (`src/views/window.rs`), with help from `Frame`, provides the bordered rectangular displays that you can drag, resize, and hide using methods from the `View` trait. A window can also zoom and close itself using its own methods. Numbered windows can be selected with Alt+n hot keys. Window objects are explained in Chapter 11, "Window and dialog box objects."

#### Dialog Boxes

`Dialog` (`src/views/dialog.rs`) composes a `Window` and is used to create dialog boxes that handle a variety of user interactions. Dialog boxes typically contain controls such as buttons and check boxes. The main difference between dialog boxes and windows is that dialog boxes are specialized for modal operation. Dialog boxes are explained in Chapter 11, "Window and dialog box objects."

### Engines

Turbo Vision includes several groups of non-view components:

- Events
- Command sets
- Validators
- Drawing utilities

#### Events

An event (`src/core/event.rs`) is a generalized structure for handling input. The event system provides a unified interface for keyboard, mouse, command, and broadcast events.

```rust
pub enum EventType {
    Nothing, Keyboard, MouseDown, MouseUp, MouseMove,
    MouseWheelUp, MouseWheelDown, Command, Broadcast,
}

pub struct Event {
    pub what: EventType,
    pub key_code: KeyCode,
    pub key_modifiers: KeyModifiers,
    pub mouse: MouseEvent,
    pub command: CommandId,
}
```

Events are explained in Chapter 9, "Event-Driven Programming."

#### Command Sets

`CommandSet` (`src/core/command_set.rs`) implements a set of command IDs using a bitfield. It provides efficient enable/disable tracking for up to 65,536 different commands. The system uses thread-local storage for global command state and automatic update notifications.

#### Validators

`Validator` is a trait (`src/views/validator.rs`) that serves as the basis for a family of types used to validate the contents of input lines. The useful validators `FilterValidator`, `RangeValidator`, `LookupValidator`, and `PictureValidator` all implement `Validator` but provide different forms of validation. All the validator types and their use are explained in Chapter 13, "Data validation objects."

#### Drawing Utilities

The drawing system (`src/core/draw.rs`) provides utilities for efficient text rendering with attributes (colors). The `DrawBuffer` type implements line-based rendering, where you build up a line of characters with their attributes and then write it to the terminal in one operation.

```rust
pub struct Cell {
    pub ch: char,
    pub attr: Attr,
}

pub struct DrawBuffer {
    pub data: Vec<Cell>,
}
```

---

## Turbo Vision Coordinates

Turbo Vision's method of assigning coordinates might be different from what you're used to. Unlike coordinate systems that designate the character spaces on the screen, Turbo Vision coordinates specify the grid between the characters. If this seems odd, you'll soon see that the system works very well for specifying the boundaries of view objects.

### Specifying Points

A point in the coordinate system is designated by its x- and y- coordinates. The `Point` struct encapsulates the coordinates in its fields, `x` and `y`. `Point` has no methods beyond constructors, but it makes it easy to deal with both coordinates in a single item.

### Specifying Boundaries

Every item on a Turbo Vision screen is rectangular, defined by a `Rect`. `Rect` has two fields, `a` and `b`, each of which is a `Point`, with `a` representing the upper left corner and `b` holding the lower right corner. When specifying the boundaries of a view, you pass those boundaries to the view's constructor in a `Rect`.

For example, `Rect::new(0, 0, 0, 0)` designates a rectangle with no size—it is only a point. The smallest rectangle that can actually contain anything would be created with `Rect::new(0, 0, 1, 1)`.

`Rect::new(2, 2, 5, 4)` produces a rectangle that contains six character spaces. This makes it easy to calculate such things as the sizes of rectangles and the coordinates of adjacent rectangles.

### Local and Global Coordinates

In the Rust implementation, all view bounds are stored in **absolute (global) coordinates**. This differs from some systems where views store relative coordinates.

When you add a child view to a group, the group converts the child's bounds from relative (to the group's origin) to absolute (screen coordinates):

```rust
pub fn add(&mut self, mut view: Box<dyn View>) {
    let child_bounds = view.bounds();  // Relative coordinates
    let absolute_bounds = Rect::new(
        self.bounds.a.x + child_bounds.a.x,
        self.bounds.a.y + child_bounds.a.y,
        self.bounds.a.x + child_bounds.b.x,
        self.bounds.a.y + child_bounds.b.y,
    );
    view.set_bounds(absolute_bounds);  // Now absolute
    self.children.push(view);
}
```

This simplifies hit testing for mouse events and eliminates the need for coordinate translation during drawing. When you specify a child view's position, you specify it relative to its parent, but internally it's immediately converted to absolute coordinates.

---

## Using Bitmapped Fields

Turbo Vision's views use several fields which are bitmapped. That is, they use the individual bits of a byte or word to indicate different properties. The individual bits are usually called flags, since by being set (equal to 1) or cleared (equal to 0), they indicate whether the designated property is activated.

For example, each view has a bitmapped `u16` field called state flags. Each of the individual bits in the word has a different meaning to Turbo Vision.

**Figure 7.4: State Flag Bits**

The state flags contain the following (defined in `src/core/state.rs`):

- `SF_VISIBLE`
- `SF_CURSOR_VIS`
- `SF_CURSOR_INS`
- `SF_SHADOW`
- `SF_ACTIVE`
- `SF_SELECTED`
- `SF_FOCUSED`
- `SF_DRAGGING`
- `SF_DISABLED`
- `SF_MODAL`
- `SF_DEFAULT`
- `SF_EXPOSED`
- `SF_CLOSED`
- `SF_RESIZING`

Similarly, option flags are defined:

- `OF_SELECTABLE`
- `OF_TOP_SELECT`
- `OF_FIRST_CLICK`
- `OF_FRAMED`
- `OF_PRE_PROCESS`
- `OF_POST_PROCESS`
- `OF_BUFFERED`
- `OF_TILEABLE`
- `OF_CENTER_X`
- `OF_CENTER_Y`

### Flag Values

In the bit field diagram, msb indicates the "most significant bit," also called the "high-order bit" because in constructing a binary number, that bit has the highest value (2^15 for a 16-bit word). The bit at the lowest end of the binary number is marked lsb, for "least significant bit," also called the "low-order bit."

So, for example, the fourth bit is `SF_SHADOW`. If the `SF_SHADOW` bit is set to 1, it means the view has a visible shadow around it. If the bit is a 0, the view has no shadow.

You generally don't have to worry about what the values of the flag bits are unless you plan to define your own, and even in that case, you only need to make sure that your definitions are unique. The highest-order bits in the state and option words may be available for custom use.

### Bit Masks

A mask is a convenient way of dealing with a group of bit flags together. For example, Turbo Vision defines masks for different kinds of events. The event mouse mask simply contains all the bits that designate different kinds of mouse events, so if a view needs to check for mouse events, it can compare the event type to see if it's in the mask, rather than having to check for each of the individual kinds of mouse events.

### Bitwise Operations

Rust provides bitwise operations to manipulate individual bits. Rather than giving a detailed explanation of how each one works, this section will simply tell you what to do to get the job done.

#### Setting Bits

To set a bit, use the `|` (bitwise OR) operator. For instance, to set the `SF_FOCUSED` bit in the state flags of a view, you use this code:

```rust
let state = self.state();
self.set_state(state | SF_FOCUSED);
```

Or more concisely using the `|=` operator:

```rust
self.set_state_flag(SF_FOCUSED, true);
```

The `View` trait provides a convenience method:

```rust
fn set_state_flag(&mut self, flag: StateFlags, enable: bool) {
    let current = self.state();
    if enable {
        self.set_state(current | flag);
    } else {
        self.set_state(current & !flag);
    }
}
```

You should not use addition to set bits unless you are absolutely sure what you are doing. Adding bits can have unwanted side effects if the bit is already set. Use the `|` operation to set bits instead.

Before leaving the topic of setting bits, note that you can set several bits in one operation by ORing the field with several bits at once:

```rust
let state = self.state();
self.set_state(state | SF_VISIBLE | SF_ACTIVE);
```

#### Clearing Bits

Clearing a bit is just as easy as setting it. You use a combination of two bitwise operations, `&` (AND) and `!` (NOT). For instance, to clear the `SF_FOCUSED` bit in the state flags, you use:

```rust
let state = self.state();
self.set_state(state & !SF_FOCUSED);
```

Or use the convenience method:

```rust
self.set_state_flag(SF_FOCUSED, false);
```

As with setting bits, multiple bits can be cleared in a single operation:

```rust
let state = self.state();
self.set_state(state & !(SF_FOCUSED | SF_SELECTED));
```

#### Toggling Bits

Sometimes you'll want to toggle a bit, meaning set it if it's clear and clear it if it's set. To do this, use the `^` (XOR) operator. For example, to toggle the horizontal centering option flag:

```rust
let options = self.options();
self.set_options(options ^ OF_CENTER_X);
```

#### Checking Bits

Quite often, a view will want to check to see if a certain flag bit is set. This uses the `&` operation. For example, to see if a window may be tiled by the desktop, you need to check the `OF_TILEABLE` option flag like this:

```rust
if self.options() & OF_TILEABLE != 0 {
    // View can be tiled
}
```

Or use the convenience method:

```rust
if self.get_state_flag(SF_FOCUSED) {
    // View is focused
}
```

#### Using Masks

Much like checking individual bits, you can use `&` to check to see if one or more masked bits are set. For example, to see if an event is some sort of mouse event, check:

```rust
if matches!(event.what, EventType::MouseDown | EventType::MouseUp | EventType::MouseMove) {
    // Handle mouse event
}
```

### Summary

**Table 7.2: Manipulating Bitmapped Fields**

| To do this | Use this code |
|---|---|
| Set a bit | `field = field \| flag;` or `field \|= flag;` |
| Clear a bit | `field = field & !flag;` |
| Toggle a bit | `field = field ^ flag;` or `field ^= flag;` |
| Check if a flag is set | `if field & flag != 0 { ... }` |
| Check for multiple flags | `if field & mask != 0 { ... }` |

---

## End of Chapter 7
