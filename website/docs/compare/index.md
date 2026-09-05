# Turbo Vision in Rust, next to Turbo Vision in C++

This crate is a port, not a reinterpretation. The class tree, the event model, the palettes and the
command numbers are Borland's. What changed is everything C++ expressed with inheritance and raw
pointers, because Rust has neither implementation inheritance nor an unowned pointer you can hand
around safely.

<div class="grid cards" markdown>

-   **[The application model, side by side](app-model.md)**

    `TApplication`, `TProgram`, `TView` and `TGroup` against `Application`, `View`, `GroupLike` and
    `WindowLike`. Read this first if you are porting a program.

-   **[A custom program in C++](custom-program-cpp.md)**

    The original technique: derive from `TProgram` and initialise only the subsystems you want.

-   **[The same program in Rust](custom-application-rust.md)**

    What that looks like here, where the subsystems are values you compose rather than base-class
    constructors you inherit.

</div>

![A Turbo Vision application in a terminal: menu bar, framed windows with shadows, status line](../assets/shots/showcase.png)

## The short version

| Borland C++ | This crate |
|---|---|
| `class TMyView : public TView` | `struct MyView { core: ViewCore, .. }` and `impl View for MyView` |
| Virtual `draw()`, `handleEvent()`, `getPalette()` | Required trait methods with the same names |
| Base fields `origin`, `size`, `state`, `options` | One `ViewCore` field, exposed through trait defaults |
| `TGroup::insert(TView *)` | `GroupLike::add(impl View)`, or `add_typed` for a handle back |
| `owner->something` back-pointer | No back-pointer at all; children speak upward by posting commands |
| `dynamic_cast<TGroup *>(v)` | `View::as_group()` |
| `dynamic_cast<TButton *>(v)` | `view.as_any().downcast_ref::<Button>()` |
| `TView *link` stored in a control | `Handle<T>` resolved by the owner |
| `cmXxx` integer commands | The same integers, in reserved ranges |
| `TStatusLine`, `TMenuBar` from `init*` overrides | Values you build and hand to `set_status_line` / `set_menu_bar` |
| `ushort` bit masks `sfActive`, `ofSelectable` | `State::ACTIVE`, `Options::SELECTABLE` newtypes |
| `TStreamable` and `operator >>` | `serde`-based serialisation |

## What is genuinely the same

If you knew the framework, these still hold, and you should stop reading the porting guide and just
write code.

An application is a menu bar, a status line and a desktop. A window contains views; a view draws
itself in its own coordinate space and handles its own events. Events flow down the tree in three
phases, pre-process views first, then the focused view, then post-process views, and a handler
clears the event to claim it. A modal loop runs until something ends it with a command, and the
command it ended on is the result. Colours are indices resolved through the chain of owners, not
absolute attributes. Commands can be globally enabled and disabled, and controls follow.

## What is genuinely different

Three things, and they all come from the same root: a Rust child cannot point back at its parent.

**Upward communication is by event.** A control that wants something from its owner posts a command,
and the owner handles it. In C++ a history control reached through `owner` to find its input line.
Here the dialog holds a `Handle<InputLine>` and does the wiring itself.

**Behaviour is inherited through traits, not base classes.** `GroupLike` carries `TGroup`'s
behaviour as default methods named `group_*`; `WindowLike` carries `TWindow`'s as `window_*`. An
override calls the base by that name, which is what `TWindow::handleEvent` calling
`TGroup::handleEvent` did. The dispatch is still late-bound, so base code drawing with
`self.get_palette()` picks up your override.

**There is no `new` that never gets deleted.** Views are owned by value. A group owns its children,
a handle names one, and `Shared<T>` exists for the rare child an owner keeps calling after adding
it, such as an editor's scroll bars.
