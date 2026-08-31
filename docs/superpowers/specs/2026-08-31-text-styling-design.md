# Text Styling (bold / italic / underline / reverse / dim / strikethrough)

Date: 2026-08-31
Status: Approved design

## Goal

Add real terminal text-styling support (bold, italic, underline, reverse,
dim, strikethrough) to the rendering pipeline. Today `Attr` carries only
foreground/background colors, and "bold"/"italic" are *simulated* by
choosing different palette colors (help viewer) or brightening the color
(ANSI parser). This feature makes styling a first-class attribute that is
emitted as real SGR codes on the live terminal (and SSH) and in ANSI text
dumps.

## Scope

In scope:

- A `Style` bitset (bold, italic, underline, reverse, dim, strikethrough).
- A `style` field on `Attr`, with builder methods, preserving backward
  compatibility for all existing call sites.
- Emitting style SGR codes from the live terminal render (`flush`) — covers
  both the real terminal and SSH backends.
- Emitting style SGR codes from the ANSI text-dump path (`ansi_dump`).
- Converting the existing "faked" styling to real styles:
  - the ANSI parser (`ansi.rs`) sets real style bits on parsed cells;
  - the help viewer (`help_viewer.rs`) applies real bold/italic to `Bold`/
    `Italic` segments.

Out of scope (explicit user decisions):

- PNG screenshot styling (`screenshot.rs` stays color-only; its 8x16
  bitmap-font rasterizer is untouched).
- Blink (inconsistent terminal support).

## Non-goals / constraints

- No new crate dependencies. `Style` is a hand-rolled `u8` newtype (6 bits).
- Existing unstyled ANSI-dump golden output must remain **byte-identical**.
- All existing `Attr::new(fg, bg)` call sites (~111) must compile unchanged.

## Design

### 1. `Style` bitset (`src/core/palette.rs`)

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct Style(u8);

impl Style {
    pub const BOLD: Style          = Style(1 << 0);
    pub const ITALIC: Style        = Style(1 << 1);
    pub const UNDERLINE: Style      = Style(1 << 2);
    pub const REVERSE: Style        = Style(1 << 3);
    pub const DIM: Style            = Style(1 << 4);
    pub const STRIKETHROUGH: Style  = Style(1 << 5);

    pub const fn empty() -> Style { Style(0) }
    pub const fn bits(self) -> u8 { self.0 }
    pub const fn is_empty(self) -> bool { self.0 == 0 }
    pub const fn contains(self, other: Style) -> bool {
        (self.0 & other.0) == other.0
    }
    pub fn insert(&mut self, other: Style) { self.0 |= other.0; }
    pub fn remove(&mut self, other: Style) { self.0 &= !other.0; }
}

impl core::ops::BitOr for Style { /* union */ }
impl core::ops::BitOrAssign for Style { /* union-assign */ }
```

### 2. `Attr` gains `style` (`src/core/palette.rs`)

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Attr {
    pub fg: TvColor,
    pub bg: TvColor,
    pub style: Style,
}
```

Backward-compatibility rules:

- `Attr::new(fg, bg)` keeps its 2-arg signature; sets `style = Style::empty()`.
  This keeps all ~111 call sites compiling unchanged.
- Builder methods, each returning a new `Attr` with the bit set:
  `.bold()`, `.italic()`, `.underline()`, `.reverse()`, `.dim()`,
  `.strikethrough()`, and `.with_style(Style)`.
  Enables `Attr::new(fg, bg).bold().italic()`.
- `to_u8()` / `from_u8()` remain **color-only** (the classic Turbo Vision
  `fg | (bg << 4)` byte cannot hold style). `from_u8` sets
  `style = Style::empty()`. Documented: palette storage is color-only.
- `swap()` and `darken()` preserve `style`.

Because `Attr` remains `Copy`/`Eq` and `style` participates in `PartialEq`,
the render diff and the run-grouping loop in `flush()` automatically split a
run when the style changes — no additional plumbing needed.

### 3. Live terminal render (`src/terminal/mod.rs::flush`)

Each changed run currently emits an absolute truecolor SGR with no reset:
`\x1b[38;2;r;g;b;48;2;r;g;bm`. Make it self-contained by prefixing a reset
and appending style codes:

```
\x1b[0;38;2;{fr};{fg};{fb};48;2;{br};{bg};{bb}{;1}{;2}{;3}{;4}{;7}{;9}m
```

SGR code per style: bold `1`, dim `2`, italic `3`, underline `4`,
reverse `7`, strikethrough `9`. The leading `0;` (reset) clears any stale
style carried from the previous run. Both the crossterm and SSH backends
receive these bytes via `write_raw`, so both are covered by this one change.

Verified: no test asserts the exact byte output of `flush()`.

### 4. ANSI text dumps (`src/core/ansi_dump.rs`)

Keep the existing incremental fg/bg diffing so unstyled output stays
byte-identical. Add a `last_style` tracker (init: `Style::empty()`); when the
current cell's style differs from `last_style`, emit only the minimal
transition codes before the character:

- turn **on** newly-set bits: bold `1`, dim `2`, italic `3`, underline `4`,
  reverse `7`, strikethrough `9`;
- turn **off** newly-cleared bits: `22` (not bold/dim), `23` (not italic),
  `24` (not underline), `27` (not reverse), `29` (not strikethrough).

For a dump containing no styled cells, `last_style` never leaves empty, so no
style bytes are ever emitted and existing goldens are unchanged. Applies to
all three dump functions (`dump_buffer`, `dump_buffer_region`, and the file
wrapper as appropriate).

### 5. Rewire the current fakes

- **ANSI parser (`src/core/ansi.rs`)**: today only SGR `1` (bold → brighten
  color) and `22` are handled. Extend the parser to track a `Style` value and
  set it on each emitted cell's `Attr`. Handle set codes `1/2/3/4/7/9` and
  reset codes `0/22/23/24/27/29`. (The existing brighten-on-bold behavior may
  be kept or dropped; real bold is now expressed via `Style::BOLD`. Decision:
  set `Style::BOLD` and stop brightening, so styling is not double-applied —
  captured as a plan step with a test.)
- **Help viewer (`src/views/help_viewer.rs`)**: map `TextSegment::Bold` to its
  color `.bold()` and `TextSegment::Italic` to its color `.italic()`.
  `TextSegment::Code` and `TextSegment::Link` keep their existing color-only
  mapping.

## Testing

- Unit: `Style` bit ops (`contains`, `insert`, `remove`, `BitOr`).
- Unit: `Attr` builders set the right bit; chaining composes bits.
- Unit: `to_u8`/`from_u8` round-trip preserves colors and drops style.
- Unit: `swap()` and `darken()` preserve `style`.
- Draw layer: via `MockTerminal::get_cell().attr.style` — cells are stored
  directly, so styles round-trip through the draw layer.
- `ansi_dump` golden: a buffer with styled cells emits the expected on/off
  codes; a buffer with no styled cells produces byte-identical output to
  before.
- `flush()` byte capture via a mock backend: a bold cell's run contains `;1`;
  a styled run followed by an unstyled run resets (`\x1b[0;`).
- ANSI parser: `\x1b[1m` sets `BOLD`, `\x1b[3m` sets `ITALIC`, `\x1b[4m` sets
  `UNDERLINE`; `\x1b[0m`/`22`/`23`/`24` clear the corresponding bits.
- Help viewer: a `Bold` segment resolves to an `Attr` with `Style::BOLD`; an
  `Italic` segment to `Style::ITALIC`.

## Risks

- **SGR bleed**: styles persist until reset. Mitigated by the leading `0;`
  reset in `flush()` and explicit off-codes in `ansi_dump`.
- **Golden snapshot drift**: mitigated by keeping `ansi_dump`'s incremental
  color diffing and only emitting style bytes when style is non-empty/changes.
- **Terminal compatibility**: dim and strikethrough are less universally
  supported than bold/underline; acceptable — they degrade gracefully (ignored
  by terminals that don't support them).
