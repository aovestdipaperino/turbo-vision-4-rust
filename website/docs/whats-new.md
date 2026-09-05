# What's new

The full history is in the [changelog](reference/changelog.md). This page tracks the recent work
in prose, newest first.

## 3.0.0 &mdash; September 2026

A major release. The `View` trait changed, so every other breaking change rode along with it.
Downstream crates that implement `View` for their own types need the migration described below.
The two documents behind it are [the inheritance analysis](reference/inheritance.md) and
[the coordinate model](reference/owner-coordinates.md).

### Owner-relative coordinates

The headline change. A view's bounds are now relative to its owner, the way Borland's
`TView::origin` always was, and they stay put when the owner moves. A view draws in its own space
from `(0, 0)` to `extent()`, and mouse positions arrive already translated into that space. The
terminal keeps an origin stack beside the clip stack, so every write, read, cursor and clip call
is translated for you.

```rust
// 2.x: bounds were screen coordinates once the view was added
let x = self.bounds().a.x;
write_line_to_terminal(terminal, x, self.bounds().a.y + row, &buf);
if self.bounds().contains(event.mouse.pos) { /* ... */ }

// 3.0.0: draw at the origin, test against your own extent
write_line_to_terminal(terminal, 0, row, &buf);
if self.extent().contains(event.mouse.pos) { /* ... */ }
```

A view that holds children by value uses `views::view::draw_child` and
`views::view::dispatch_to_child`, which push and pop the child's origin around the call. Code
outside the tree that draws a top-level view calls `terminal.draw_view(&mut view)`.

### One base, one hook layer

`ViewCore` now holds the fields every view has: bounds, state, options, grow mode and the palette
chain. A view implements `core()` and `core_mut()`, and the ten field accessors become trait
defaults reading that core. A view can no longer forget to report its own state.

Above `View` sit two behaviour traits. `GroupLike` carries `TGroup`'s behaviour as `group_*`
default methods, including the modal loop and typed child access. `WindowLike` does the same for
`TWindow` with `window_*` bodies. A window-shaped type implements the two accessor pairs and gets
its `View` implementation from `impl_view_for_window!`, writing only its overrides inline. The
overrides are late-bound, so a base `window_draw` paints with the derived type's palette, which is
what the C++ virtual call did.

Six hooks left the `View` trait entirely: the two button hooks, the two list selection hooks and
the two end-state hooks. Downcast to `Button` or `ListBox`, or reach the modal end state through
`View::as_group`, the analogue of `dynamic_cast<TGroup*>`.

### Typed flags instead of bare integers

`State`, `Options`, `Grow`, `MsgBox` and `ValidatorOptions` are newtypes with associated constants,
so a state can no longer be passed where options were expected. Write `state.contains(State::MODAL)`
rather than a bitwise test against zero. The old `SF_*`, `OF_*`, `GF_*` and `MF_*` names survive as
deprecated aliases for one release.

### Command numbers have owners

Commands from 0 to 99 are Borland's standard set, 100 to 199 belong to this crate, and applications
start at `CM_USER`, which is 200. The demo-application commands left the library. Alongside this,
the old rule that any command below 1000 closed a dialog is gone, replaced by an explicit `CloseOn`
policy: a modal dialog ends on the standard four commands plus its own buttons by default, and any
other child's command passes through to the caller whatever its number.

### Loops you no longer have to write

`Application::run_with` takes an `AppHandler` with `pre_event`, `handle_command`, `idle` and
`window_closed` hooks, replacing the hand-copied event loops programs used to carry.
`Application::execute_modal` is now the single modal loop behind every dialog, and it dispatches to
the outer type's `handle_event`, so a wrapper reacts to its own children without duplicating the
loop.

### Smaller changes worth knowing

An input line owns its own text, so the `Rc<RefCell<String>>` parameter is gone and you read the
value back through a typed `Handle<InputLine>` after the dialog closes. Menus and status items are
built with key chord strings such as `"Ctrl+O"`, and an unknown chord panics on first run rather
than silently binding nothing. A generic `Shared<T>` replaced five per-type forwarding newtypes.
`add` accepts any view, boxed or not.

### Fixed

A window as large as its desktop was pushed to a negative origin, hiding its top row and left
column. Drag limits now clamp the far edges before the near ones, as `TView::moveGrow` does, so the
top-left corner stays where it belongs.

## 2.4.0 &mdash; September 2026

Ten new controls and three completions: a split pane, multi-select list boxes, scroll bar
auto-repeat, multi-item clusters, tooltips and the frame zoom triangle. See
[More controls](reference/more-controls.md).

## Earlier releases

Version 2.0.0 declared the port production ready at full API parity with the C++ original. The
releases before it built up the widget set, the editor, the palette system and the persistence
layer. All of them are in the [changelog](reference/changelog.md).
