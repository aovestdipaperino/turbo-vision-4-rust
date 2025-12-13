# SSH TUI Bridge Implementation Plan

## Current State Analysis

### Architecture Summary
- **No Backend trait exists** - Terminal is a concrete 721-line struct in `src/terminal/mod.rs`
- **49 view files** reference `Terminal` directly via `draw(&mut Terminal)` methods
- **crossterm deeply integrated** - event polling, cursor control, colors, terminal modes
- **Event system** is already well-designed and generic (not crossterm-specific)
- **Double-buffered rendering** already abstracts physical terminal output

### Key Dependencies to Add
```toml
russh = "0.50"
russh-keys = "0.50"
async-trait = "0.1"
parking_lot = "0.12"
```

---

## Implementation Strategy

### Approach: Keep Terminal as Facade with Internal Backend Trait

Rather than making `Terminal` generic (which would require updating all 49 view files), we'll:
1. Extract a `Backend` trait for low-level I/O
2. Keep `Terminal` as the public API (unchanged interface for views)
3. `Terminal` internally delegates to a `Box<dyn Backend>`
4. Create `CrosstermBackend` (default) and `SshBackend` implementations

This minimizes changes to existing code while enabling SSH support.

---

## Phase 1: Backend Trait Extraction

### Step 1.1: Create Backend Trait Module

**File:** `src/terminal/backend.rs` (new)

```rust
pub trait Backend: Send {
    /// Initialize the backend (enter raw mode, alternate screen, etc.)
    fn init(&mut self) -> io::Result<()>;

    /// Restore backend to original state
    fn cleanup(&mut self) -> io::Result<()>;

    /// Get terminal dimensions (width, height)
    fn size(&self) -> io::Result<(u16, u16)>;

    /// Poll for an event with timeout
    fn poll_event(&mut self, timeout: Duration) -> io::Result<Option<Event>>;

    /// Write raw output to terminal
    fn write_raw(&mut self, data: &[u8]) -> io::Result<()>;

    /// Flush buffered output
    fn flush(&mut self) -> io::Result<()>;

    /// Show cursor at position
    fn show_cursor(&mut self, x: u16, y: u16) -> io::Result<()>;

    /// Hide cursor
    fn hide_cursor(&mut self) -> io::Result<()>;

    /// Query terminal capabilities
    fn capabilities(&self) -> Capabilities;

    /// Suspend terminal (Ctrl+Z) - optional
    fn suspend(&mut self) -> io::Result<()> { Ok(()) }

    /// Resume from suspend - optional
    fn resume(&mut self) -> io::Result<()> { Ok(()) }
}
```

### Step 1.2: Extract CrosstermBackend

**File:** `src/terminal/crossterm_backend.rs` (new)

Move crossterm-specific code from `Terminal` into `CrosstermBackend`:
- Raw mode initialization
- Event polling with `crossterm::event::poll()` and `read()`
- Cursor control with `crossterm::cursor`
- Terminal output via `crossterm::execute!` and `queue!`

### Step 1.3: Refactor Terminal

**File:** `src/terminal/mod.rs` (modify)

Changes:
- Add `backend: Box<dyn Backend>` field
- Constructor takes optional backend (default to CrosstermBackend)
- Delegate `poll_event()`, `init()`, `shutdown()` to backend
- Keep all buffer management, clipping, cell writing in Terminal

```rust
impl Terminal {
    pub fn new() -> io::Result<Self> {
        Self::with_backend(Box::new(CrosstermBackend::new()?))
    }

    pub fn with_backend(backend: Box<dyn Backend>) -> io::Result<Self> {
        // ... initialize with provided backend
    }
}
```

### Files to Modify in Phase 1:
- `src/terminal/mod.rs` - Refactor to use Backend trait
- `src/terminal/backend.rs` - New: Backend trait definition
- `src/terminal/crossterm_backend.rs` - New: CrosstermBackend implementation

---

## Phase 2: Input Parser

### Step 2.1: Create Input Parser Module

**File:** `src/terminal/input_parser.rs` (new)

The input parser converts raw terminal bytes into turbo-vision `Event` structures:
- Handle ANSI escape sequences (CSI, SS3)
- Parse keyboard input including function keys and modifiers
- Parse mouse events (X10 and SGR formats)
- Handle UTF-8 multi-byte characters
- Buffer incomplete sequences

Key conversion mapping from specification:
```rust
// Map to turbo-vision KeyCode constants
fn crossterm_to_tv_keycode(code: crossterm::event::KeyCode, mods: KeyModifiers) -> u16 {
    // Convert to KB_* constants (KB_UP, KB_ENTER, KB_ALT_X, etc.)
}
```

### Files to Create in Phase 2:
- `src/terminal/input_parser.rs` - Raw byte to Event parser

---

## Phase 3: SSH Backend

### Step 3.1: Create SSH Backend

**File:** `src/terminal/ssh_backend.rs` (new)

Implements `Backend` trait for SSH channels:
- Uses `mpsc` channels for async I/O with russh
- Maintains terminal size from PTY negotiation
- Uses `InputParser` to convert raw bytes to events
- Buffers output before flushing to SSH channel

```rust
pub struct SshBackend {
    output_buffer: Vec<u8>,
    output_tx: mpsc::UnboundedSender<Vec<u8>>,
    event_rx: mpsc::UnboundedReceiver<Event>,
    event_queue: Vec<Event>,
    size: Arc<Mutex<(u16, u16)>>,
    capabilities: Capabilities,
}
```

### Step 3.2: Create SSH Handler

**File:** `src/ssh/handler.rs` (new)

Implements russh `Handler` trait:
- `auth_password` / `auth_publickey` - Authentication
- `channel_open_session` - Create TUI session
- `pty_request` - Handle PTY size negotiation
- `shell_request` - Start shell
- `window_change_request` - Handle resize
- `data` - Route input to InputParser
- `channel_close` - Cleanup

### Step 3.3: Create SSH Server Module

**File:** `src/ssh/server.rs` (new)

SSH server setup:
- Host key management (generate or load from file)
- Server configuration
- Connection handling
- TUI session spawning

### Files to Create in Phase 3:
- `src/terminal/ssh_backend.rs` - SSH Backend implementation
- `src/ssh/mod.rs` - SSH module root
- `src/ssh/handler.rs` - russh Handler implementation
- `src/ssh/server.rs` - SSH server setup

---

## Phase 4: Integration

### Step 4.1: Update Cargo.toml

Add SSH dependencies (behind feature flag):
```toml
[features]
default = []
ssh = ["russh", "russh-keys", "async-trait", "parking_lot"]

[dependencies]
russh = { version = "0.50", optional = true }
russh-keys = { version = "0.50", optional = true }
async-trait = { version = "0.1", optional = true }
parking_lot = { version = "0.12", optional = true }
```

### Step 4.2: Update Module Structure

**File:** `src/terminal/mod.rs` - Add module declarations:
```rust
mod backend;
mod crossterm_backend;
mod input_parser;

#[cfg(feature = "ssh")]
mod ssh_backend;

pub use backend::{Backend, Capabilities};
pub use crossterm_backend::CrosstermBackend;

#[cfg(feature = "ssh")]
pub use ssh_backend::SshBackend;
```

**File:** `src/lib.rs` - Add SSH module:
```rust
#[cfg(feature = "ssh")]
pub mod ssh;
```

### Step 4.3: Create Example Application

**File:** `examples/ssh_server.rs` (new)

Demo SSH server that exposes a turbo-vision application:
```rust
#[tokio::main]
async fn main() {
    let config = russh::server::Config { ... };
    russh::server::run(Arc::new(config), "0.0.0.0:2222", TuiServer).await;
}
```

### Files to Modify/Create in Phase 4:
- `Cargo.toml` - Add dependencies and feature flag
- `src/terminal/mod.rs` - Module structure updates
- `src/lib.rs` - Add SSH module export
- `examples/ssh_server.rs` - Example SSH TUI server

---

## File Summary

### New Files (8)
| File | Description | Est. Lines |
|------|-------------|------------|
| `src/terminal/backend.rs` | Backend trait definition | ~100 |
| `src/terminal/crossterm_backend.rs` | CrosstermBackend impl | ~300 |
| `src/terminal/input_parser.rs` | Raw byte to Event parser | ~400 |
| `src/terminal/ssh_backend.rs` | SshBackend impl | ~250 |
| `src/ssh/mod.rs` | SSH module root | ~20 |
| `src/ssh/handler.rs` | russh Handler impl | ~200 |
| `src/ssh/server.rs` | SSH server setup | ~100 |
| `examples/ssh_server.rs` | Example SSH server | ~80 |

### Modified Files (3)
| File | Changes |
|------|---------|
| `Cargo.toml` | Add SSH dependencies, feature flag |
| `src/terminal/mod.rs` | Refactor to use Backend trait |
| `src/lib.rs` | Add SSH module export |

---

## Implementation Order

1. **Phase 1** - Backend trait extraction (enables testing with existing crossterm)
   - 1.1: Create `backend.rs` with trait definition
   - 1.2: Create `crossterm_backend.rs` by extracting from Terminal
   - 1.3: Refactor `Terminal` to use `Box<dyn Backend>`
   - **Test:** Existing examples should work unchanged

2. **Phase 2** - Input parser
   - 2.1: Create `input_parser.rs`
   - **Test:** Unit tests for escape sequence parsing

3. **Phase 3** - SSH backend
   - 3.1: Create `ssh_backend.rs`
   - 3.2: Create `ssh/handler.rs`
   - 3.3: Create `ssh/server.rs`
   - **Test:** Manual testing with SSH client

4. **Phase 4** - Integration
   - 4.1: Update `Cargo.toml` with feature flag
   - 4.2: Update module structure
   - 4.3: Create example SSH server
   - **Test:** Full end-to-end testing

---

## Risk Assessment

### High Risk
- **Event model mismatch** - turbo-vision uses synchronous event polling; SSH is async
  - Mitigation: Use `mpsc` channels to bridge async/sync boundary

### Medium Risk
- **Performance** - SSH latency may affect responsiveness
  - Mitigation: Batch output updates, use differential rendering (already exists)

### Low Risk
- **Existing code breakage** - Backend refactor could break views
  - Mitigation: Keep Terminal API unchanged, only internal changes

---

## Questions for Clarification

1. **Authentication strategy** - Should we implement password auth, public key, or both?
2. **Multi-session support** - Should one server support multiple concurrent SSH connections?
3. **Shared state** - Should SSH sessions share application state (e.g., for admin consoles)?
4. **Feature flag** - Should SSH support be behind a feature flag (default off)?

---

## Acceptance Criteria

- [ ] `cargo build` works without changes for existing users
- [ ] `cargo build --features ssh` enables SSH support
- [ ] Existing examples run unchanged
- [ ] `examples/ssh_server.rs` demonstrates SSH TUI
- [ ] Unit tests for InputParser
- [ ] Integration test for SshBackend (if feasible)
- [ ] Documentation in README
