# Upgrading to 3.0.0

3.0.0 changes the `View` trait, so every crate that implements a view of its own
has to be adapted. This guide is the ordered list of what to change. The
[changelog](CHANGELOG.md) records the same ground as release notes; the
reasoning is in [the inheritance analysis](docs/MISSING-INHERITANCE.md) and
[the coordinate model](docs/OWNER-COORDINATES.md).

## Do you need this?

If your program only *uses* the library, builds dialogs from the supplied
controls and never writes `impl View`, most of this does not apply. Read steps
4 through 7, which are the changes that reach ordinary application code, and
skip the rest.

If you implement `View`, `Window` or a control of your own, start at step 1 and
work down. A mechanical port of a small view takes a few minutes.

## The short version

| Step | What changes | Who it affects |
|---|---|---|
| 1 | Base fields move into one `ViewCore` | Anyone implementing `View` |
| 2 | Window-shaped types use `WindowLike` and a macro | Anyone subclassing a window |
| 3 | Coordinates are owner-relative | Any custom `draw` or `handle_event` |
| 4 | Flags are typed newtypes | Everyone |
| 5 | Commands have reserved ranges; `CloseOn` replaces the 1000 rule | Everyone |
| 6 | `AppHandler` replaces hand-written event loops | Everyone |
| 7 | An input line owns its text; children are reached by `Handle<T>` | Anyone reading a form |
| 8 | Menus and status items take key chords | Everyone |

## 1. One `ViewCore` instead of scattered fields

A view used to carry `bounds`, `state`, `options`, `grow_mode` and
`palette_chain` itself and implement ten accessors. Those fields now live in a
`ViewCore`, the accessors are trait defaults that read it, and a view cannot
report a state it does not have.

```rust
// 2.x
pub struct MyView {
    bounds: Rect,
    state: StateFlags,
    options: u16,
    palette_chain: Option<PaletteChainNode>,
    // ... plus your own fields
}

impl View for MyView {
    fn bounds(&self) -> Rect { self.bounds }
    fn set_bounds(&mut self, b: Rect) { self.bounds = b; }
    // ... eight more accessors
    fn draw(&mut self, terminal: &mut Terminal) { /* ... */ }
    fn handle_event(&mut self, event: &mut Event) { /* ... */ }
    fn get_palette(&self) -> Option<Palette> { /* ... */ }
}
```

```rust
// 3.0.0
pub struct MyView {
    core: ViewCore,
    // ... plus your own fields
}

impl MyView {
    pub fn new(bounds: Rect) -> Self {
        let mut core = ViewCore::new(bounds);
        core.options |= Options::SELECTABLE;
        core.grow_mode = Grow::HI_X;
        Self { core }
    }
}

impl View for MyView {
    fn core(&self) -> &ViewCore { &self.core }
    fn core_mut(&mut self) -> &mut ViewCore { &mut self.core }
    fn as_any(&self) -> &dyn std::any::Any { self }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any { self }

    fn draw(&mut self, terminal: &mut Terminal) { /* ... */ }
    fn handle_event(&mut self, event: &mut Event) { /* ... */ }
    fn get_palette(&self) -> Option<Palette> { /* ... */ }
}
```

`as_any` and `as_any_mut` are now required. They used to have a panicking
default, and the library relies on them where C++ used `dynamic_cast`.

Six hooks left the trait entirely: `is_default_button`, `button_command`,
`set_list_selection`, `get_list_selection`, `get_end_state` and `set_end_state`.
Downcast to `Button` or `ListBox` for the first four. The modal end state lives
on `GroupLike::end_state` and `end_modal`, reachable from a `&dyn View` through
`View::as_group()`.

## 2. Window-shaped types

`Group`'s and `Window`'s behaviour now live in trait defaults named `group_*`
and `window_*`, so a container type can make a base call the way
`TWindow::handleEvent` called `TGroup::handleEvent`. A window-shaped type
implements two accessor pairs and gets its `View` implementation from a macro,
writing only its overrides inline.

```rust
// 3.0.0
impl GroupLike for MyWindow {
    fn group(&self) -> &Group { self.window.group() }
    fn group_mut(&mut self) -> &mut Group { self.window.group_mut() }
}

impl WindowLike for MyWindow {
    fn window(&self) -> &Window { &self.window }
    fn window_mut(&mut self) -> &mut Window { &mut self.window }
}

impl_view_for_window!(MyWindow {
    fn handle_event(&mut self, event: &mut Event) {
        self.window_handle_event(event);   // the base call
        // your handling
    }
});
```

Dispatch stays late-bound: the base `window_draw` paints with
`self.get_palette()`, so an override of `get_palette` is used by base code.

> **Do not redeclare `View` method names on `WindowLike`.** Two traits in scope
> with the same method name make every call ambiguous (E0034). That is why the
> overrides go inside the macro rather than in a second `impl` block.

`Window` and `Dialog` no longer have inherent `add`, `child_*`, `execute`,
`end_modal`, `get_end_state` or `set_end_state`. Import `GroupLike`, which is in
the prelude, to keep calling them.

## 3. Owner-relative coordinates

This is the change most likely to make your view draw in the wrong place. A
view's `bounds()` are now relative to its owner, as Borland's `TView::origin`
were, and they stay put when the owner moves. A view draws in its own space,
from `(0, 0)` to `extent()`, and mouse positions arrive already translated into
that space.

```rust
// 2.x: bounds were screen coordinates once the view was added
let x = self.bounds().a.x;
write_line_to_terminal(terminal, x, self.bounds().a.y + row, &buf);
if self.bounds().contains(event.mouse.pos) { /* ... */ }

// 3.0.0: draw at the origin, test against your own extent
write_line_to_terminal(terminal, 0, row, &buf);
if self.extent().contains(event.mouse.pos) { /* ... */ }
```

The rules:

- **Draw at `(0, 0)`.** The terminal keeps an origin stack beside the clip
  stack and translates every write, read, cursor and clip call.
- **Test the mouse against `extent()`**, which is your own `(0, 0, w, h)`.
- **A view holding children by value** uses `views::view::draw_child` and
  `views::view::dispatch_to_child`, which push and pop the child's origin.
- **Code outside the tree** that draws a top-level view calls
  `terminal.draw_view(&mut view)` instead of `view.draw(&mut terminal)`, and
  dispatches with `dispatch_to_child(&mut view, &mut event)`.
- **`set_parent_bounds` is `set_owner_extent`** and receives the owner's extent.
- **`make_global` and `make_local` convert one hop**, between a view and its
  owner, not all the way to the screen.

Positions given when adding a child were always owner-relative and do not
change. Tests that inspected a child's `bounds()` after adding it now see owner
coordinates rather than screen coordinates.

## 4. Typed flags

`State`, `Options`, `Grow`, `MsgBox` and `ValidatorOptions` are newtypes with
associated constants, so a state can no longer be passed where options were
expected.

```rust
// 2.x
if state & SF_MODAL != 0 { /* ... */ }
view.set_options(OF_SELECTABLE | OF_PRE_PROCESS);

// 3.0.0
if state.contains(State::MODAL) { /* ... */ }
view.set_options(Options::SELECTABLE | Options::PRE_PROCESS);
```

Use `contains` or `intersects`; never compare against zero. The old `SF_*`,
`OF_*`, `GF_*` and `MF_*` names remain as deprecated aliases for one release, so
your code keeps compiling while you migrate. `StateFlags` and `GrowFlags` are
type aliases of `State` and `Grow`.

## 5. Commands and dialog closing

Command numbers now have owners:

| Range | Belongs to |
|---|---|
| 0-99 | Borland's standard commands |
| 100-199 | This crate's internal commands and broadcasts |
| 200 and up | Your application. Start at `CM_USER`. |

If your application numbered its commands below 200, renumber them from
`CM_USER`. The demo-application commands (`CM_ABOUT`, `CM_BIRTHDATE`,
`CM_TEXT_VIEWER` and friends) left the library; define them in your own program.

The rule that any command below 1000 closed a modal dialog is gone. A dialog
now ends on the commands its `CloseOn` policy names:

```rust
CloseOn::StandardAndButtons  // the default: CM_OK, CM_CANCEL, CM_YES, CM_NO,
                             // plus the commands of the buttons added to it
CloseOn::Standard            // only the four
CloseOn::Commands(vec![..])  // exactly this list
```

Set it with `DialogBuilder::close_on` or `Dialog::set_close_on`. A command from
any other child, a list box say, is left for the caller whatever its number, so
commands no longer need to be numbered above 1000 to pass through.

## 6. The event loop

Programs used to copy the event loop to handle their own commands. Implement
`AppHandler` and pass it to `run_with` instead.

```rust
struct MyApp;

impl AppHandler for MyApp {
    fn pre_event(&mut self, app: &mut Application, event: &mut Event) {}
    fn handle_command(
        &mut self,
        app: &mut Application,
        command: CommandId,
        _event: &Event,
    ) -> bool {
        match command {
            CM_ABOUT => { /* ... */ true }
            _ => false,
        }
    }
    fn idle(&mut self, app: &mut Application) {}
    fn window_closed(&mut self, app: &mut Application, id: ViewId) {}
}

app.run_with(&mut MyApp)?;
```

Every hook has an ancestor. `pre_event` is `TApplication::handleEvent` running
before the desktop sees the event, `handle_command` the `case cmXxx` block after
it, `idle` is `TProgram::idle`, and `window_closed` the cleanup after the
desktop removed a window. `Application::run()` is now `run_with(&mut ())`.

`Application::execute_modal` is the single modal loop behind `Dialog::execute`,
`FileDialog::execute`, `ChDirDialog::execute` and `HelpWindow::execute`. It
dispatches to the outer type's `handle_event`, so a wrapper reacts to its own
children without a copy of the loop.

## 7. Reading a form

An input line owns its text. The `Rc<RefCell<String>>` you used to pass in and
read out sideways is gone. Keep a typed handle and ask the dialog for the child
after it closes.

```rust
// 2.x
let data = Rc::new(RefCell::new(String::new()));
dialog.add(InputLineBuilder::new().data(data.clone()).build());
dialog.execute(&mut app);
let value = data.borrow().clone();

// 3.0.0
let field = dialog.add_typed(InputLineBuilder::new().text("").build());
dialog.execute(&mut app);
let value = dialog.get(field).map(|f| f.text().to_string()).unwrap_or_default();
```

`InputLine::new(bounds, max_length)` and `with_validator(bounds, max_length,
validator)` lost their shared-string parameter, and `InputLineBuilder::text`
replaces `data`.

`GroupLike::add_typed` returns a `Handle<T>`; `get` and `get_mut` give the
concrete child back. `Desktop` has the same three methods for its windows. This
is also how sibling coordination works now that there is no owner back-pointer:
`History::new` takes the `Handle<InputLine>` it is linked to, and the owning
dialog records and fills the input.

`add` accepts any view: `add(v)` rather than `add(Box::new(v))`, though the
boxed form still compiles because a boxed view is itself a `View`. The boxed
primitive is `add_boxed`.

## 8. Menus and status items

Both builders take key chord strings.

```rust
// 3.0.0
MenuBuilder::new().item("New", CM_NEW).item_key("Open...", CM_OPEN, "Ctrl+O")
StatusItemBuilder::new().text("~Alt-X~ Exit").key("Alt+X").command(CM_QUIT)
```

The positional constructors `MenuItem::new`, `MenuItem::with_shortcut`,
`MenuItem::new_disabled` and `StatusItem::new` are gone; `MenuItem::flag`,
`submenu` and `separator` stay. An unknown chord panics the first time the
application runs, so a typo surfaces immediately instead of becoming a dead key.
The `KB_*` constants remain public for `handle_event` match arms.

## Renamed and removed, at a glance

| 2.x | 3.0.0 |
|---|---|
| Own `bounds` / `state` / `options` / `palette_chain` fields | one `core: ViewCore` |
| `set_parent_bounds` | `set_owner_extent` |
| `impl IdleView` | `fn idle` on `View` |
| `editor_rc()` / `viewer_rc()` | `editor()` / `viewer()` (old names deprecated) |
| `SharedScrollBar`, `SharedEditor`, `SharedIndicator`, ... | `Shared<T>` |
| `Desktop::remove_closed_windows -> bool` | returns `Vec<ViewId>` |
| `FileDialog::handle_selection(&mut Terminal, ..)` | no terminal parameter |
| `helpers::msgbox` implementation | `views::msgbox` (the helper re-exports it) |
| `views::status_line::StatusItem` | re-export of `core::status_data::StatusItem` |

## When something goes wrong

**The view draws in the wrong place, or twice as far from the corner as it
should.** You are still adding the owner's origin. Draw at `(0, 0)`; see step 3.

**Clicks land next to the control, not on it.** You are testing the mouse
against `bounds()` instead of `extent()`.

**E0034, "multiple applicable items in scope".** A `View` method name was
redeclared on `WindowLike`. Move the override inside `impl_view_for_window!`.

**The control's colours are wrong.** A view takes its colours from the palette
of whatever owns it, so a `Button` in a plain `Window` picks up the window's
palette rather than a dialog's. The `wrong_owner` example demonstrates this
deliberately.

**A panic at startup about a key chord.** A menu or status item was given a
chord the parser does not know. The message names it.

**A command reaches the application that used to close a dialog.** Set a
`CloseOn` policy naming it; see step 5.
