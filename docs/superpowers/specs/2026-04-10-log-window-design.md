# LogWindow Design

Addresses GitHub issue #87 (internal logging).

## Overview

A `LogWindow` view that wraps a `Window` + `TerminalWidget` and implements `tracing::Subscriber`, so `tracing::info!()`, `debug!()`, etc. automatically route to a scrollable, color-coded log window with black background.

## Construction

```rust
let log_window = LogWindowBuilder::new()
    .bounds(Rect::new(0, 0, 80, 15))
    .title("Log")
    .min_level(tracing::Level::DEBUG)
    .build();
app.desktop.add(Box::new(log_window));
```

After the LogWindow's subscriber is set as the global default, all `tracing` macros route to it.

## Architecture

- `LogWindow` wraps `Window` (blue palette, resizable) + shared `Rc<RefCell<TerminalWidget>>` (scrollbar, auto-scroll)
- Black background via custom window palette override (`set_custom_palette`)
- `LogSubscriber` struct implements `tracing::Subscriber`, holds the same `Rc<RefCell<TerminalWidget>>`
- `LogWindowBuilder::build()` returns `LogWindow` and installs the global subscriber
- Each log line formatted as `HH:MM:SS LEVEL message`

## Log Level Colors (on black background)

| Level | Foreground | Hex |
|-------|-----------|-----|
| ERROR | LightRed | 0x0C |
| WARN | Yellow | 0x0E |
| INFO | White | 0x0F |
| DEBUG | LightGray | 0x07 |
| TRACE | DarkGray | 0x08 |

## Files

- New: `src/views/log_window.rs`
- Modify: `src/views/mod.rs` (add `pub mod log_window`)
- Modify: `Cargo.toml` (add `tracing` dependency)

## Out of Scope

- OutputWindow (not needed)
- Log file persistence
- Log filtering UI
