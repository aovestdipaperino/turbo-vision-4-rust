# Chapter 8: Views and Groups

**Previous:** [Chapter 7 — Architecture Overview](Chapter-7-Architecture-Overview.md)

---

## Introduction

One of the keys to Turbo Vision is the system used to present information on the screen, using views. Views are objects that represent rectangular regions on the screen, and they are the only way Turbo Vision applications display information to users.

In this chapter, you'll learn the following:

- What is a view?
- What is a group?
- How to use views
- How to use groups

Because views share common behavior, they all implement the same `View` trait defined in `src/views/view.rs:34-166`. Turbo Vision also defines specialized view implementations such as windows, dialog boxes, applications, desktops, menus, and so on. Other chapters in this part of the manual describe how to use these specific views, but this chapter focuses on the principles common to all views.

---

## What is a View?

Unlike traditional terminal programs that use direct print statements to display information, Turbo Vision applications use views, which are objects that know how to represent themselves on the screen.

### Definition of a View

The basic building block of a Turbo Vision application is the view. A view manages a rectangular area of the screen. For example, the menu bar at the top of the screen is a view. Any program action in that area of the screen (for example, clicking the mouse on the menu bar) will be dealt with by the view that controls that area.

In general, anything that shows up on the screen of a Turbo Vision program must be a view, which means it implements the `View` trait. There are three things that all views must do:

- Manage a rectangular region
- Draw itself on demand
- Handle events in its boundaries

The standard views provided with Turbo Vision handle these things automatically, and the views you create for your applications will either inherit these abilities or you'll have to add them to your types. Let's look at each of these properties in more detail.

### Defining a Region

When you construct a view, you assign it boundaries in the form of a `Rect` (defined in `src/core/geometry.rs:18-109`). Boundary rectangles and the Turbo Vision coordinate system are explained in detail in Chapter 7, but it's important when you think about the other two properties of a view that you remember that a view is limited to the area defined by its boundaries.

### Drawing on Demand

The most important visual property of a view is that it knows how to represent itself on the screen. For example, when you want to put a menu bar across the top of the application screen, you construct a menu bar view, giving it the boundaries of the top line of the screen and defining for it a list of menu items. The menu bar view knows how to represent those items in the designated space.

You don't have to concern yourself with when the view appears. You define a `draw()` method for the view that fills in the entire area within its bounding rectangle. Turbo Vision calls `draw()` when it knows that the view needs to show itself, such as when a window is uncovered because the window in front of it closes.

The two important things to remember about `draw()` methods are these:

- The view must fill its entire rectangle
- The view must be able to draw itself at any time

### Handling Events

The third property of any view is that it must handle events that occur inside its boundaries, such as mouse clicks and keystrokes. Event handling is explained in detail in Chapter 2 "Responding to Commands", but for now just remember that a view is responsible for any events within its boundaries, just as it must draw everything within its boundaries.

---

## What is a Group?

Sometimes the easiest way for a view to manage its area is to delegate certain parts of the job to other views, known as subviews or children. A view that has children is called a group. The `Group` struct (defined in `src/views/group.rs:10-16`) implements the `View` trait and manages a collection of child views. A group with children is said to own the children, because it manages those child views. Each child is said to have an owner view, which is the group that owns it.

The most visible example of a group view, but one you might not ordinarily think of as a view, is the application itself. It controls the entire screen, but you don't notice that because the program sets up three other child views—the menu bar, the status line, and the desktop—to handle its interactions with the user. As you will see, what appears to the user as a single object (like a window) is often a group of related views.

### Delegating to Subviews

Since a group is a view, all the normal rules of views still apply. A group covers a rectangle, draws itself on demand, and handles events within its boundaries. The main difference with groups is that they handle most of their tasks by delegating them to children.

For example, the `draw()` method of a group (see `src/views/group.rs:349-381`) generally doesn't draw anything itself except an optional background, but instead calls on each of the group's children in turn to draw itself. The result of the `draw()` methods of all the children, therefore, must result in covering the group's entire rectangle.

---

## Using View Objects

All Turbo Vision views implement the `View` trait defined in `src/views/view.rs`. This trait serves as a common interface for all views, ensuring that all views can operate uniformly within the system. This section describes the following tasks you'll need to perform on views:

- Understanding view boundaries
- Managing view boundaries
- Drawing the view
- Handling the cursor
- Setting state flags
- Validating the view

### Understanding View Boundaries

The `View` trait defines two key methods for working with boundaries:

```rust
fn bounds(&self) -> Rect;
fn set_bounds(&mut self, bounds: Rect);
```

The `bounds()` method returns the current bounding rectangle of the view. A `Rect` is defined as:

```rust
pub struct Rect {
    pub a: Point,  // Top-left corner (inclusive)
    pub b: Point,  // Bottom-right corner (exclusive)
}
```

The location of a view is determined by its bounding rectangle. The `a` field indicates the top-left corner of the view in absolute screen coordinates, and the `b` field represents the bottom-right corner. Note that the bottom-right is **exclusive**, meaning a rectangle from (0, 0) to (10, 5) has a width of 10 and height of 5.

When working with child views in a group, coordinates are handled specially. When you create a child view and add it to a group using `Group::add()` (see `src/views/group.rs:39-51`), the child's bounds are specified **relative to the group's origin**. The `add()` method automatically converts these relative coordinates to absolute screen coordinates:

```rust
pub fn add(&mut self, mut view: Box<dyn View>) {
    let child_bounds = view.bounds();
    let absolute_bounds = Rect::new(
        self.bounds.a.x + child_bounds.a.x,
        self.bounds.a.y + child_bounds.a.y,
        self.bounds.a.x + child_bounds.b.x,
        self.bounds.a.y + child_bounds.b.y,
    );
    view.set_bounds(absolute_bounds);
    self.children.push(view);
}
```

### Managing View Boundaries

Once you've constructed a view, the `Rect` type (defined in `src/core/geometry.rs:18-109`) provides methods for manipulating boundaries. In particular, you can do the following:

- Get the view's size
- Move the view
- Resize the view

#### Getting the View's Size

The `Rect` type provides several methods for working with dimensions:

```rust
pub fn width(&self) -> i16;
pub fn height(&self) -> i16;
pub fn size(&self) -> Point;
```

For example, to get the size of a view:

```rust
let view_bounds = view.bounds();
let width = view_bounds.width();
let height = view_bounds.height();
```

#### Moving a View

To change the position of a view without affecting its size, modify the view's bounds by moving both corners by the same amount. The `Rect` type provides a `move_by()` method:

```rust
pub fn move_by(&mut self, dx: i16, dy: i16);
```

For example, to move a view two spaces to the left and one space down:

```rust
let mut bounds = view.bounds();
bounds.move_by(-2, 1);
view.set_bounds(bounds);
```

#### Resizing a View

To change the size of a view, use the `Rect::grow()` method, which adjusts the size while keeping the top-left corner fixed:

```rust
pub fn grow(&mut self, dx: i16, dy: i16);
```

For example, to make a view wider by 10 units and taller by 5 units:

```rust
let mut bounds = view.bounds();
bounds.grow(10, 5);
view.set_bounds(bounds);
```

Note that negative values shrink the rectangle. This is particularly useful when fitting one view inside another, such as creating the interior of a window by shrinking the window's bounds by 1 unit on all sides.

#### Fitting Views Into Owners

One of the most common manipulations of a view's coordinates involves fitting one view into another. For example, creating the interior of a window involves making sure the interior doesn't cover any part of the window's frame. The `grow()` method with negative parameters makes this easy.

For example, to create a view that fits inside another with a 1-unit border:

```rust
let outer_bounds = Rect::from_coords(0, 0, 40, 20);
let mut inner_bounds = outer_bounds;
inner_bounds.grow(-1, -1);  // Shrinks by 1 on all sides
// inner_bounds is now Rect::from_coords(1, 1, 39, 19)
```

This pattern is used extensively in window construction (see `src/views/window.rs:27-96`).

### Drawing a View

The appearance of a view is determined by its `draw()` method. Nearly every new type of view will need to have its own `draw()` implementation, since it is generally the appearance of a view that distinguishes it from other views.

There are a couple of rules that apply to all views with respect to appearance. A view must:

- Cover the entire area for which it is responsible
- Be able to draw itself at any time

Both of these properties are very important and deserve some discussion.

#### Drawing on Demand

A view must always be able to represent itself on the screen. That's because other views may cover part of it but then be removed, or the view itself might move. In any case, when called upon to do so, a view must always know enough about its present state to show itself properly.

Note that this might mean that the view does nothing at all. It might be entirely covered, or it might not even be on the screen. Most of these situations are handled automatically through the clipping system (managed by `Terminal::push_clip()` and `Terminal::pop_clip()` in `src/terminal/mod.rs`), but it is important to remember that your view must always know how to draw itself.

This is different from other windowing schemes, where the writing on a window, for example, is persistent: You write it there and it stays, even if something covers it up then moves away. In Turbo Vision, you can't assume that a view you uncover is displayed correctly—after all, something may have told it to change while it was covered.

### Changing View Option Flags

All views have access to option flags through the `View` trait methods:

```rust
fn options(&self) -> u16;
fn set_options(&mut self, options: u16);
```

Option flags are defined in `src/core/state.rs:20-31` as bitmapped constants. Each bit in the flags field has a special meaning, setting some option in the view. The values of these option flags get set when you first construct the view, and normally stay set, although you can change the values at any time.

The main option flags include:

- `OF_SELECTABLE` (0x001): Whether the user can select the view
- `OF_TOP_SELECT` (0x002): Whether selecting the view brings it to the front
- `OF_FIRST_CLICK` (0x004): Whether the click that selects the view is also processed
- `OF_FRAMED` (0x008): Whether the view has a visible frame
- `OF_PRE_PROCESS` (0x010): Process events before the focused view
- `OF_POST_PROCESS` (0x020): Process events after the focused view
- `OF_BUFFERED` (0x040): Use buffered drawing (groups only)
- `OF_TILEABLE` (0x080): Can be tiled or cascaded (windows only)
- `OF_CENTER_X` (0x100): Center horizontally in owner
- `OF_CENTER_Y` (0x200): Center vertically in owner
- `OF_CENTERED` (0x300): Center both horizontally and vertically

#### Customizing Selection

Most views have `OF_SELECTABLE` set by default, meaning the user can select the view with the mouse. If the view is in a group, the user can also select it with the Tab key. You might not want the user to select purely informational views, so you can clear their `OF_SELECTABLE` bits. Static text objects and window frames, for example, are not selectable by default.

The `OF_TOP_SELECT` bit, if set, causes the view to move to the top of the owner's children when selected. This option is designed primarily for windows on the desktop.

The `OF_FIRST_CLICK` bit controls whether the mouse click that selects the view is also passed to the view for processing. For example, if the user clicks a button, you want to both select the button and press it with just one click, so buttons have `OF_FIRST_CLICK` set by default. But if the user clicks on an inactive window, you probably only want to select the window and not process the click as an action on the window once it's activated.

#### Special Event Handling

The bits `OF_PRE_PROCESS` and `OF_POST_PROCESS` allow a view to process focused events before or after the focused view sees them. Group event handling (see `src/views/group.rs:440-474`) implements three-phase event processing:

1. **PreProcess phase**: Events are sent to children with `OF_PRE_PROCESS` set
2. **Focused phase**: Events are sent to the currently focused child
3. **PostProcess phase**: Events are sent to children with `OF_POST_PROCESS` set

This allows views like status lines to respond to keyboard shortcuts even when they don't have focus.

### Setting the View's State

Every view maintains state information through the `View` trait methods:

```rust
fn state(&self) -> StateFlags;
fn set_state(&mut self, state: StateFlags);
fn set_state_flag(&mut self, flag: StateFlags, enable: bool);
fn get_state_flag(&self, flag: StateFlags) -> bool;
```

State flags are defined in `src/core/state.rs:1-19`. Unlike option flags which are set when you construct a view and rarely change, state flags often change during the lifetime of a view as the state of the view changes. State information includes whether the view is visible, has a cursor or shadow, is being dragged, or has the input focus.

The main state flags include:

- `SF_VISIBLE` (0x001): View is visible
- `SF_CURSOR_VIS` (0x002): Cursor is visible
- `SF_CURSOR_INS` (0x004): Insert cursor style (block) vs. overwrite (underline)
- `SF_SHADOW` (0x008): View has a shadow
- `SF_ACTIVE` (0x010): View is in an active group
- `SF_SELECTED` (0x020): View is selected in its owner group
- `SF_FOCUSED` (0x040): View has keyboard focus
- `SF_DRAGGING` (0x080): View is being dragged
- `SF_DISABLED` (0x100): View is disabled
- `SF_MODAL` (0x200): View is executing modally
- `SF_DEFAULT` (0x400): View is the default button
- `SF_EXPOSED` (0x800): View is exposed (not covered)
- `SF_CLOSED` (0x1000): View has been closed
- `SF_RESIZING` (0x2000): View is being resized

#### Setting and Clearing State Flags

For the most part, you don't need to change state bits manually, since the most common state changes are handled by other methods. For example, the `SF_CURSOR_VIS` bit controls whether the view has a visible text cursor, but you typically control this through the `update_cursor()` method rather than manipulating the bit directly.

To change a state flag, use the `set_state_flag()` method. For example, to set the `SF_SHADOW` flag:

```rust
view.set_state_flag(SF_SHADOW, true);
```

To clear it:

```rust
view.set_state_flag(SF_SHADOW, false);
```

### Handling the Cursor

Any visible view can have a cursor, although the cursor only shows up when the view has the input focus. The cursor provides a visual indication to the user of where keyboard input will go, but it is up to the programmer to make sure the program actually matches the cursor position to the input location.

Views handle cursor management through the `update_cursor()` method:

```rust
fn update_cursor(&self, terminal: &mut Terminal);
```

By default, this method hides the cursor. Views that need to show a cursor (such as input lines and editors) override this method to set the cursor position and make it visible.

For example, the `InputLine` implementation (in `src/views/input_line.rs`) shows the cursor when focused:

```rust
fn update_cursor(&self, terminal: &mut Terminal) {
    if self.is_focused() {
        let cursor_x = self.bounds.a.x + (self.cursor_pos - self.first_pos) as i16;
        let _ = terminal.set_cursor(cursor_x as u16, self.bounds.a.y as u16);
        let _ = terminal.show_cursor();
    }
}
```

The `SF_CURSOR_VIS` bit in the view's state controls whether the view wants a visible cursor when focused. The `SF_CURSOR_INS` bit controls the cursor style: when set, the cursor is shown as a block (insert mode); when clear, it's shown as an underline (overwrite mode).

### Validating a View

Every view can implement validation logic through custom methods. In general, validation is a way of checking the view, asking "If I performed this action, would it be safe?" Validation is used for three different kinds of checks:

- Checking for proper construction
- Checking for safe closing
- Data validation

#### Checking View Construction

Views should ensure that anything done during construction, such as memory allocation, succeeded. In Rust, this is typically handled through the type system and `Result` types rather than a separate validation method. For example, if a view needs to allocate memory, the constructor would return a `Result<Self, Error>` rather than panicking.

#### Checking for Safe Closing

The most common time to validate is when closing a view. For example, when you call a window's close method, it should check whether it's safe to close. This might involve ensuring that information is saved, buffers are flushed, and so on.

For example, a file editor view should check to make sure that any changes have been saved to the file before allowing itself to close. If there are unsaved changes, the view might put up a dialog box asking the user whether to save the changes, providing options to save and close, abandon changes and close, or cancel the close operation.

#### Data Validation

Input line views can validate their contents by checking with validator objects. Data validation can take place when the user closes a window, but you can use the exact same mechanism to validate at any other time.

For example, input line objects can check the validity of their contents when closing. But you can just as easily check the input as the user types it, validating after each keystroke.

---

## Writing Draw Methods

The appearance of any view is determined by its `draw()` method. When you write `draw()` methods, you need to keep in mind the principles outlined in the "Drawing a view" section. Turbo Vision provides several tools you can use to write a view's information to the screen.

Writing `draw()` methods involves the following tasks:

- Selecting colors
- Writing through buffers

### Selecting Colors

When you write data to the screen in Turbo Vision, you specify colors using attribute values. An attribute is a `u8` value where the high nibble (bits 4-7) represents the background color and the low nibble (bits 0-3) represents the foreground color.

The `Attr` type (defined in `src/core/attr.rs`) provides methods for creating and manipulating color attributes:

```rust
pub type Attr = u8;

pub fn make_attr(fg: u8, bg: u8) -> Attr {
    (bg << 4) | (fg & 0x0F)
}
```

When implementing a `draw()` method, you typically define the colors your view needs based on its state (active, focused, disabled, etc.) and use those attributes when writing to the screen.

### Writing Through Buffers

The standard way to handle drawing views is to write the text to a `DrawBuffer`, then display the buffer all at once. Using buffers improves the speed of drawing and reduces flicker caused by large numbers of individual writes to the screen.

A `DrawBuffer` (defined in `src/core/draw.rs:16-105`) is a line-based buffer that holds character and attribute pairs:

```rust
pub struct DrawBuffer {
    pub data: Vec<Cell>,
}

pub struct Cell {
    pub ch: char,
    pub attr: Attr,
}
```

Drawing with a buffer takes three steps:

1. Create a buffer
2. Fill the buffer with text and attributes
3. Write the buffer to the terminal

#### Creating and Filling Buffers

The `DrawBuffer` provides several methods for filling the buffer:

```rust
// Fill with repeated characters
pub fn move_char(&mut self, pos: usize, ch: char, attr: Attr, count: usize);

// Write a string
pub fn move_str(&mut self, pos: usize, s: &str, attr: Attr);

// Copy cells from another buffer
pub fn move_buf(&mut self, pos: usize, src: &[Cell], count: usize);

// Put a single character
pub fn put_char(&mut self, pos: usize, ch: char, attr: Attr);
```

For example, to create a buffer for a line and fill it with text:

```rust
let mut buf = DrawBuffer::new(width);
buf.move_char(0, ' ', normal_attr, width);  // Fill with spaces
buf.move_str(2, "Hello, World!", text_attr); // Write text at position 2
```

#### Writing Buffers to the Screen

Once you've filled a buffer, write it to the terminal using the provided helper function (defined in `src/views/view.rs:169-200`):

```rust
pub fn write_line_to_terminal(terminal: &mut Terminal, x: i16, y: i16, buf: &DrawBuffer);
```

This function handles clipping automatically based on the current clip region.

Here's a complete example from the `Button` implementation (`src/views/button.rs:50-126`):

```rust
fn draw(&mut self, terminal: &mut Terminal) {
    let width = self.bounds.width() as usize;
    let mut buf = DrawBuffer::new(width);

    let attr = if self.is_focused() {
        make_attr(COLOR_BLACK, COLOR_CYAN)
    } else if self.get_state_flag(SF_DISABLED) {
        make_attr(COLOR_DARK_GRAY, COLOR_BLACK)
    } else {
        make_attr(COLOR_BLACK, COLOR_LIGHT_GRAY)
    };

    buf.move_char(0, ' ', attr, width);
    buf.move_str(1, &self.title, attr);

    write_line_to_terminal(terminal, self.bounds.a.x, self.bounds.a.y, &buf);

    if self.has_shadow() {
        draw_shadow(terminal, self.bounds, SHADOW_ATTR);
    }
}
```

---

## Using Group Objects

You've already learned something about the `Group` struct (defined in `src/views/group.rs:10-16`). Groups are collectively referred to as container views, as they contain and manage other views.

Basically a group is just an empty box that contains and manages other views. Technically, it is a view, and therefore responsible for all the things that any view must be able to do: manage a rectangular area of the screen, visually represent itself at any time, and handle events in its screen region. The difference is really in how it accomplishes these things. Most of it is handled by child views.

This section covers the following topics regarding group views:

- Groups, children, and owners
- Inserting children
- Understanding children
- Selecting and focusing children
- Drawing groups
- Executing modal groups
- Managing children

Although you need to understand them, you should never need to change the basic behavior of groups, such as inserting, drawing, and executing. Most of that behavior is simple and straightforward.

For example, it might not be apparent, but the processes of adding a menu bar to an application, a window to the desktop, and a control to a dialog box are exactly the same. In each case, you're inserting a child view into a group, using the same `add()` method.

### Groups, Children, and Owners

A group is a holder for other views. You can think of a group as a composite view. Instead of handling all its responsibilities itself, it divides its duties among various child views. A child view is a view that is owned by another view, and the group that owns it is called the owner view.

An excellent example is `Application`. `Application` is a view that controls a region of the screen—the whole screen, in fact. The application owns a desktop view (which is itself a group), which in turn owns the menu bar, status line, and any windows. What appears to the user as a single object (like a window) is often a group of related views.

### Inserting Children

To attach a child view to an owner, you insert the child into the owner using the `add()` method (defined in `src/views/group.rs:39-51`):

```rust
pub fn add(&mut self, view: Box<dyn View>);
```

Any group can have any number of children, and any view can be a child. The group stores its children as a `Vec<Box<dyn View>>`, maintaining them in insertion order.

**Important: Coordinate Conversion**

When you create a child view and pass it to `add()`, you specify the child's bounds **relative to the group's origin**. The `add()` method automatically converts these to absolute screen coordinates:

```rust
pub fn add(&mut self, mut view: Box<dyn View>) {
    let child_bounds = view.bounds();
    let absolute_bounds = Rect::new(
        self.bounds.a.x + child_bounds.a.x,  // Add group's origin
        self.bounds.a.y + child_bounds.a.y,
        self.bounds.a.x + child_bounds.b.x,
        self.bounds.a.y + child_bounds.b.y,
    );
    view.set_bounds(absolute_bounds);
    self.children.push(view);
}
```

At a minimum, the children of a group must cover the entire area of the group's boundaries. There are two ways to handle this:

- Dividing the group
- Providing a background

In general, the first approach is used in cases where children don't overlap, such as the application or a window divided into separate panes. The background method is used in cases where children need to overlap and move, such as the desktop, or cases where the important children are separated, such as the controls in a dialog box.

#### Dividing the Group

Some groups just divide their rectangular region into parts and permanently assign views to each part.

For example, an application might divide its screen into menu bar, desktop, and status line regions, creating three views with non-overlapping bounds and adding them to the application's desktop group.

#### Providing a Background

There is no reason that views can't overlap. Indeed, one of the big advantages of a windowed environment is the ability to have multiple, overlapping windows on the desktop. Groups know how to handle overlapping children.

The basic idea of a background is to assure that something is drawn over the entire area of the group, letting other children cover only the particular area they need to. The desktop (defined in `src/views/desktop.rs:9-31`) provides a good example—it creates a `Background` view as its first child, which fills the entire desktop area with a pattern. Windows can then be added on top, overlapping each other.

The `Group` struct supports an optional background color:

```rust
pub fn with_background(bounds: Rect, background: Attr) -> Self;
```

Any time you're dealing with a background or other overlapping views, you need to understand how Turbo Vision decides which views are "in front of" or "behind" others. The front-to-back positioning of objects is determined by the objects' Z-order, which is the topic of the next section.

### Understanding Children

There are two important aspects to the relationship between an owner view and its children: the actual links between the views, and the order of the views. This section answers two important questions:

- What is a view tree?
- What is Z-order?

#### What is a View Tree?

When you insert children into a group, the views create a kind of view tree, with the owner as the "trunk" and the children as "branches." The ownership linkages of all the views in a complex application can get fairly complex, but if you visualize them as a single branching tree, you can grasp the overall structure.

For example, the application owns a desktop. The desktop owns a background view and any windows. Each window owns a frame and an interior group. The interior group owns controls and other views. The ownership linkages form a tree structure:

```
Application
└── Desktop (Group)
    ├── Background
    ├── Window 1
    │   ├── Frame
    │   └── Interior (Group)
    │       ├── Button 1
    │       ├── Button 2
    │       └── InputLine
    └── Window 2
        ├── Frame
        └── Interior (Group)
            └── StaticText
```

If the user closes Window 1, Turbo Vision removes it from the desktop's children and disposes of it. The window will dispose of all its children (frame and interior), and the interior will dispose of its children (the buttons and input line), then they dispose of themselves.

#### What is Z-order?

Groups keep track of the order in which children are inserted. That order is referred to as Z-order. The term Z-order refers to the fact that children have a three-dimensional spatial relationship.

Every view has a position and size within the plane of the view as you see it (the X and Y dimensions), determined by its bounds. But views and children can overlap, and in order for Turbo Vision to know which view is in front of which others, we have to add a third dimension, the Z-dimension.

Z-order, then, refers to the order in which you encounter views as you start closest to you and move back "into" the screen. Think of X-order as going from left to right, Y-order from top to bottom, and Z-order from front to back. The last view inserted is the "front" view.

##### Visualizing Z-order

Rather than thinking of the screen as a flat plane with things written on it, consider it a pane of glass providing a portal onto a three-dimensional world of views. Indeed, every group may be thought of as a "sandwich" of views.

For example, a window is a group containing a frame and an interior. The frame is inserted first, making it the background. The interior is inserted after, so it appears "above" the frame, but typically has smaller bounds so the frame remains visible around the edges.

On a larger scale, the desktop is a larger group containing a background and multiple windows. The background is "behind" all the others. Windows are stacked in insertion order, with the most recently inserted window appearing on top.

The `Group::bring_to_front()` method (defined in `src/views/group.rs:97-122`) allows you to change Z-order by moving a child to the end of the children vector:

```rust
pub fn bring_to_front(&mut self, index: usize) -> usize;
```

### Selecting and Focusing Children

Within each group of views, one and only one child can be selected and focused. For example, when your application sets up its desktop, one of the windows (if any are open) is the selected window. This is also called the active window (typically the topmost window).

Within the active window, the selected child is called the focused view. You can think of the focused view as being the one you're looking at, or the one where action will take place. In an editor window, the focused view is the interior view with the text in it. In a dialog box, the focused view is the highlighted control. The focused view is the end of the chain of selected views that starts at the application.

Among other things, knowing which view is focused tells you which view gets information from the keyboard.

#### Finding the Focused View

The currently focused view is usually highlighted in some way on the screen. For example, if you have several windows open on the desktop, the active window is the one with the double-lined frame. The others' frames are single-lined. Within a dialog box, the focused control is brighter than the others, indicating that it is the one acted upon if you press Enter.

Groups track the focused child with an index field and provide methods to query and change focus (defined in `src/views/group.rs:53-95`):

```rust
pub fn focused_child(&self) -> Option<&dyn View>;
pub fn set_focus_to(&mut self, index: usize);
pub fn set_initial_focus(&mut self);
pub fn select_next(&mut self);
pub fn select_previous(&mut self);
```

#### How Does a View Get the Focus?

A view can get the focus in two ways, either by default when it is created, or by some action by the user.

When a group of views is created, the group can specify which of its children is to be focused by calling `set_initial_focus()`, which finds the first child that can accept focus (has `OF_SELECTABLE` set and `can_focus()` returns true).

The user usually determines which view currently has the focus by clicking a particular view or pressing Tab to cycle through selectable views. For instance, if the application has several windows open on the desktop, the user can select different ones simply by clicking them. In a dialog box, the user can move the focus among views by pressing Tab (which calls `select_next()`), which cycles through all the selectable views, by clicking a particular view, or by pressing a hot key.

Note that some views are not selectable, including the background of the desktop and frames of windows. When you construct a view, you can control whether it's selectable through the `OF_SELECTABLE` option flag. If you click the frame of a window, for example, the frame does not get the focus, because the frame doesn't have `OF_SELECTABLE` set.

### Drawing Groups

Groups are an exception to the rule that views must know how to draw themselves, because a group does not draw itself per se. Rather, a `Group` tells its children to draw themselves (see the implementation in `src/views/group.rs:349-381`). The cumulative effect of drawing the children must cover the entire area assigned to the group.

A dialog box, for example, is a group, and its children—frame, interior, controls, and static text—must combine to fully "cover" the full area of the dialog box view. Otherwise, "holes" in the dialog box appear, with unpredictable results.

You will rarely, if ever, need to change the way groups draw themselves, but you do need to understand the following aspects of group drawing:

- Drawing in Z-order
- Clipping children

#### Drawing in Z-order

The group calls on its children to draw themselves in insertion order, meaning that the first child inserted is drawn first and the last child inserted is drawn last. If children overlap, the one most recently inserted will appear on top of any others.

Here's the `Group::draw()` implementation:

```rust
fn draw(&mut self, terminal: &mut Terminal) {
    // Draw optional background
    if let Some(bg_attr) = self.background {
        // ... fill with background pattern ...
    }

    // Clip to our bounds
    terminal.push_clip(self.bounds);

    // Draw all children that intersect our bounds
    for child in &mut self.children {
        let child_bounds = child.bounds();
        if self.bounds.intersects(&child_bounds) {
            child.draw(terminal);
        }
    }

    terminal.pop_clip();
}
```

#### Clipping Children

When the children of a group draw themselves, drawing is automatically clipped at the borders of the group. The terminal maintains a clip stack (see `src/terminal/mod.rs`) that groups push and pop. Because children are clipped, when you initialize a view and give it to a group, the view needs to reside at least partially within the group's boundaries.

Only the part of a child that is within the bounds of its owner group will be visible. The clipping is handled automatically by the terminal when writing cells to the screen buffer.

### Executing Modal Groups

Most complex programs have several different modes of operation, where a mode is some distinct way of functioning. Depending on which mode is active, keys on the keyboard might have varying effects (or no effect at all).

Almost any Turbo Vision view can define a mode of operation, in which case it is called a modal view, but modal views are nearly always groups. The classic example of a modal view is a dialog box. Usually, when a dialog box is active, nothing outside it functions. You can't use the menus or other controls not owned by the dialog box. The dialog box has control of your program until the user closes it.

In order to use modal views, you need to understand four things:

- What is modality?
- Executing a view
- Ending a modal state
- Getting the modal result

There is always a modal view when a Turbo Vision application is running. When you start the program, and often for the duration of the program, the modal view is the application itself.

#### What is Modality?

When you make a view modal, only that view and its children can interact with the user. You can think of a modal view as defining the "scope" of a portion of your program. When you create a block in a Rust function, any variables declared within that block are only valid within that block. Similarly, a modal view determines what behaviors are valid within it—events are handled only by the modal view and its children. Any part of the view tree that is not the modal view or owned by the modal view is inactive.

There is one exception to this rule: The status line is always "hot," no matter what view is modal. That way you can have active status line items, even when your program is executing a modal dialog box that does not own the status line.

#### Making a Group Modal

You can make a group the current modal view by executing it; that is, calling its `execute()` method (defined in `src/views/group.rs:142-203`):

```rust
pub fn execute(&mut self, app: &mut Application) -> CommandId;
```

This method implements an event loop, interacting with the user and dispatching events to the proper children until the modal state is ended.

Here's the basic structure:

```rust
pub fn execute(&mut self, app: &mut Application) -> CommandId {
    self.end_state = 0;

    loop {
        // Draw the view
        self.draw(&mut app.terminal);

        // Poll for events
        if let Some(mut event) = app.terminal.poll_event(timeout) {
            // Handle the event
            self.handle_event(&mut event);
        }

        // Check if modal state ended
        if self.end_state != 0 {
            return self.end_state;
        }
    }
}
```

The `Dialog` implementation (in `src/views/dialog.rs:81-129`) shows a complete example of modal execution with proper state management.

#### Ending a Modal State

Any view can end its modal state by setting its end state through the `set_end_state()` method (defined in `src/views/view.rs:160-161`):

```rust
fn set_end_state(&mut self, command: CommandId);
```

This stores a command value that will be returned by the `execute()` method. The modal loop checks this value and exits when it becomes non-zero.

For example, the `Dialog` event handler (in `src/views/dialog.rs:149-194`) ends the modal state when it sees certain commands:

```rust
fn handle_event(&mut self, event: &mut Event) {
    match event.what {
        EventType::Command => {
            match event.command {
                CM_OK | CM_CANCEL | CM_YES | CM_NO => {
                    if self.is_modal() {
                        self.window.end_modal(event.command);
                        event.clear();
                    }
                }
                // ... other commands ...
            }
        }
        // ... other event types ...
    }
}
```

The end state is returned by `execute()`, allowing the caller to determine how the modal view was closed:

```rust
let result = dialog.execute(&mut app);
match result {
    CM_OK => { /* handle OK */ }
    CM_CANCEL => { /* handle Cancel */ }
    _ => { /* handle other cases */ }
}
```

### Managing Children

Once you've inserted a child into a group, the group handles nearly all the management of the child for you, making sure it's drawn, moved, and so on. When you dispose of a group, it automatically disposes of all its children, so you don't have to dispose of them individually. In Rust, this happens automatically through the `Drop` trait.

Aside from the automatic child management, you'll sometimes need to perform the following tasks on a group's children:

- Removing children
- Accessing children
- Broadcasting events

#### Removing Children

Although a group automatically disposes of all its children when it's dropped, you sometimes want to remove a child while you're still using the group. An obvious example is closing a window on the desktop: dropping the desktop drops all windows, but you'll often need to remove a window in the course of running an application.

To remove a child from its owner, use the group's `remove()` method (defined in `src/views/group.rs:125-140`):

```rust
pub fn remove(&mut self, index: usize);
```

This removes the child from the owner's list of children. The child is dropped and its memory is freed.

The `Desktop` implementation (in `src/views/desktop.rs:129-146`) uses this to remove closed windows:

```rust
pub fn remove_closed_windows(&mut self) {
    let mut i = 0;
    while i < self.group.len() {
        let child = self.group.child_at(i);
        if child.get_state_flag(SF_CLOSED) {
            self.group.remove(i);
        } else {
            i += 1;
        }
    }
}
```

#### Accessing Children

Groups provide methods to access their children (defined in `src/views/group.rs`):

```rust
pub fn len(&self) -> usize;
pub fn child_at(&self, index: usize) -> &dyn View;
pub fn child_at_mut(&mut self, index: usize) -> &mut dyn View;
```

These allow you to query or modify specific children. For example, you might search for a particular window or check the state of all children.

#### Broadcasting Events

Groups provide a `broadcast()` method (defined in `src/views/group.rs:205-238`) that sends an event to all children:

```rust
pub fn broadcast(&mut self, event: &mut Event, owner_index: Option<usize>);
```

Broadcast events are sent to all children, even if one of them handles (clears) the event. This is different from normal event handling, where an event stops propagating once it's cleared.

For example, when a dialog's default button changes, it broadcasts `CM_COMMAND_SET_CHANGED` to all buttons so they can update their appearance. See the button implementation in `src/views/button.rs:139-161` for an example of handling broadcast events.

---

## End of Chapter 8

In this chapter, you've learned about the fundamental building blocks of Turbo Vision applications:

- **Views** are objects that manage rectangular screen regions, draw themselves, and handle events
- The **View trait** (`src/views/view.rs`) defines the common interface all views implement
- **Groups** (`src/views/group.rs`) are containers that manage child views
- Views work with **Rectangles** (`src/core/geometry.rs`) and **Points** for positioning
- Drawing uses **DrawBuffers** (`src/core/draw.rs`) for efficient rendering
- Events (`src/core/event.rs`) flow through the view tree with three-phase processing
- **State flags** (`src/core/state.rs`) track view state like visibility, focus, and modality
- **Modal execution** allows views like dialogs to take control of the event loop

The next chapters will build on these concepts, showing you how to work with specific view types like windows, dialogs, and controls.

---

**Next:** [Chapter 9 — Event-Driven Programming](Chapter-9-Event-Driven-Programming.md)
