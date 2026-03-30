# Palette System Design: QCell-Based Safe Owner Chain

## Problem

Borland's Turbo Vision stores a raw `TView::owner` pointer on every view so
that `mapColor()` can walk up the parent chain, remapping a logical color index
through each ancestor's palette until it reaches the Application palette.

The original Rust port used `*const dyn View` for this, requiring `unsafe` to
dereference. Three unsafe sites existed:

1. `map_color()` -- walking the owner chain
2. `Window::get_drag_limits()` -- reading parent bounds
3. `Label::handle_event()` -- casting owner to `&mut Group`

The goal is to eliminate all unsafe while preserving Borland's chain-walk
semantics exactly.

## Design: Static QCellOwner + QCell Chain Nodes

### Core idea

Each view stores an `Option<PaletteChainNode>` -- a reference-counted,
QCell-protected node that holds a palette and a link to the parent's node.
During `draw()`, the parent builds its node and clones it onto each child,
forming a chain that mirrors Borland's `owner` pointer chain.

Reading the chain requires a `&QCellOwner` token. A single `QCellOwner` lives
in a `static OnceLock`, initialized on first access. Because `QCellOwner` is
`Sync` and we only ever call `cell.ro(&owner)` (shared read), any number of
views can read the chain concurrently without data races.

### Why static and not per-frame or thread-local

`QCellOwner` is `Sync`, so it can live in a `static OnceLock` safely. A
per-frame token would require threading `&token` through `draw()` and
`map_color()` signatures (viral change across 50+ files). A thread-local adds
TLS overhead on every `map_color()` call. A static `OnceLock` has zero runtime
overhead after initialization and requires no signature changes.

### Thread safety

The static `QCellOwner` only yields `&QCellOwner` (shared reference). With a
shared reference, callers can only invoke `cell.ro(owner)` which returns `&T`
(immutable access). Obtaining `&mut QCellOwner` for `cell.rw()` is impossible
from a static without unsafe, so aliased mutation cannot occur.

Additionally, `PaletteChainNode` contains `Rc` which is `!Send`, preventing
nodes from crossing thread boundaries at compile time. The palette chain is
created and consumed exclusively on the UI thread. A background thread sending
events through a channel never touches views or palettes.

### Architecture

```
                           static OnceLock<QCellOwner>
                                      |
                          (shared &QCellOwner for all reads)
                                      |
   Application::draw()                |
         |                            |
    Desktop.draw()                    |
         |                            |
    Window.draw()                     |
      |-- builds PaletteChainNode { palette: CP_BLUE_WINDOW, parent: None }
      |-- sets on Frame, Interior, frame_children
      |
    Group.draw() (interior)
      |-- builds PaletteChainNode { palette: None, parent: Window's node }
      |-- sets on each child
      |
    Button.draw()
      |-- map_color(1) reads chain via token from static OnceLock
      |   1. remap through CP_BUTTON[1] = 10
      |   2. chain_node.remap_color(10, &token)
      |      -> CP_BLUE_WINDOW[10] = 42
      |   3. app_palette[42-1] = 0x30 (final Attr)
```

### Key types

```rust
// src/core/palette_chain.rs

/// Global QCellOwner -- zero-cost access after init.
pub fn palette_token() -> &'static QCellOwner { ... }

/// A node in the owner chain. Rc<QCell<...>> for safe shared access.
pub struct PaletteChainNode {
    inner: Rc<QCell<PaletteChainData>>,
}

struct PaletteChainData {
    palette: Option<Palette>,
    parent: Option<PaletteChainNode>,
}
```

### View trait changes

```rust
// NO changes to draw() or map_color() signatures.
// Token is obtained from the static inside map_color().

fn draw(&mut self, terminal: &mut Terminal);           // original signature preserved
fn map_color(&self, color_index: u8) -> Attr;          // original signature preserved

// New methods (with defaults):
fn set_palette_chain(&mut self, _: Option<PaletteChainNode>) {}
fn get_palette_chain(&self) -> Option<&PaletteChainNode> { None }
fn set_parent_bounds(&mut self, _: Rect) {}             // for Window drag limits
```

### Chain setup during draw

**Group::draw()** builds a node from its own palette + its parent link, then
clones it onto each child before drawing:

```rust
let node = PaletteChainNode::new(
    palette_token(),
    self.get_palette(),           // None for Group (transparent)
    self.palette_chain.clone(),   // link to parent's node
);
for child in &mut self.children {
    child.set_palette_chain(Some(node.clone()));  // Rc clone, cheap
    child.draw(terminal);
}
```

**Window::draw()** does the same but Window has a real palette
(`CP_BLUE_WINDOW`, `CP_GRAY_DIALOG`, etc.).

### map_color() implementation

```rust
fn map_color(&self, color_index: u8) -> Attr {
    // Step 1: remap through own palette
    // Step 2: walk QCell chain via palette_token()
    if let Some(node) = self.get_palette_chain() {
        color = node.remap_color(color, palette_token());
    }
    // Step 3: resolve through app palette
}
```

No token parameter needed -- `palette_token()` returns `&'static QCellOwner`.

### Non-palette owner uses

**Window drag limits**: `Desktop::add()` calls `view.set_parent_bounds(self.bounds())`
so that `Window::get_drag_limits()` returns the desktop bounds without any pointer.

**Label hotkey focus**: Label emits a `CM_FOCUS_LINK` broadcast event. Group
handles it by calling `focus_by_view_id()`. No unsafe cast to `&mut Group`.

## Cost analysis

| Item | Cost |
|------|------|
| `OnceLock::get()` in `palette_token()` | One atomic load (optimized away after first call in practice) |
| `Rc::clone()` per child per frame | Atomic increment/decrement (~2ns) |
| `QCell::ro()` per chain node per `map_color()` call | One integer comparison (owner ID check) |
| Memory per view | One `Option<PaletteChainNode>` = 8 bytes (pointer-sized `Rc`) |

Typical frame: ~20 visible views, ~3 `map_color()` calls each = ~60 QCell reads +
~20 Rc clones. Negligible compared to terminal I/O.

## Migration from current state

The current branch (`qcell-palette-chain`) already has:
- `PaletteChainNode` and `PaletteToken` types
- `palette_chain` field on all 33 view structs
- Chain setup in Group::draw() and Window::draw()
- All old owner infrastructure removed

Remaining work:
1. Add `static OnceLock<QCellOwner>` with `palette_token()` accessor
2. Remove `token` parameter from `draw()` -- revert to original `fn draw(&mut self, terminal: &mut Terminal)`
3. Remove `token` parameter from `map_color()` -- revert to original `fn map_color(&self, color_index: u8) -> Attr`
4. `PaletteChainNode::new()` and `remap_color()` call `palette_token()` internally
5. Remove all `token` parameter threading from 50+ files
6. Remove per-frame `PaletteToken::new()` from Application, Dialog, etc.
