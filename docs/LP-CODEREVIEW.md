# Code Review

A full-tree review of turbo-vision (51.7k LOC across `app`, `core`, `helpers`,
`ssh`, `terminal`, `views`), carried out with a reliability-focused engineering
lens: strong domain types, fail-closed boundaries, invariants that the compiler
or a test actually enforces, and complexity that some observable failure earns.

Reviewed at commit `0558baf`. Evidence gathered from `cargo clippy --all-targets
--all-features`, `cargo test`, `cargo test --all-features`, and a read of the
terminal and rendering layer, `core::geometry`, group event dispatch, the
application event loop, and the SSH stack.

Nothing in the tree was changed to produce this document.

## Blocking

### The crate does not build with `--all-features`

`examples/ssh_server.rs:98` calls `dialog.get_end_state()`. The method is now
`end_state()`. The example fails to compile, which aborts the whole
`--all-features` build, so **no test runs at all** on the `ssh` feature. Every
finding below about the SSH stack is therefore read from the source rather than
observed at runtime, because the feature currently cannot be exercised.

The fix is the rename. The reason it survived is the absence of CI, covered
under Suggestions.

## Important

### Overlay widgets are painted at the wrong position on idle frames

`src/app/application.rs:646` draws overlay widgets with
`widget.draw(&mut self.terminal)`. Every other overlay draw site — `:511`,
`:685`, and `draw_child` at `:709` — first pushes `view.bounds().a` as the
origin. `Terminal::draw_view`'s own documentation states the consequence:
calling `draw` directly paints at the current origin, which on this path is the
screen corner.

So between events an animated overlay is painted at the top-left of the screen,
and the next event-driven frame snaps it back to its real bounds. The correction
is to use `Self::draw_child` here as well. No test covers the position of an
overlay on the idle path.

### `Terminal::resume` re-introduces the trap `FORCE_REDRAW_CELL` exists to prevent

`src/terminal/mod.rs:285` fills `prev_buffer` with the empty cell (`' '` at
`0x07`) to force a redraw. `with_backend` and `resize` deliberately use
`FORCE_REDRAW_CELL` instead, and the comment at `:118` spells out why: a view
that fills with LightGray-on-Black spaces compares equal to the empty cell and
is silently skipped.

After Ctrl+Z and resume, every plain background cell the application draws is
therefore not resent, and the shell's leftover output shows through. The fix is
one line — `self.force_full_redraw()`, which already does exactly the right
thing.

### `Rect::contains` and `Rect::is_empty` disagree about degenerate rectangles

`src/core/geometry.rs:111` treats `b.x <= a.x` as "the point must equal `a.x`",
so `Rect::new(5, 5, 5, 5)` is both `is_empty() == true` and
`contains(Point(5, 5)) == true`. Two things follow.

A child collapsed to zero size still wins the hit test at
`src/views/group.rs:550`, so it captures a click on its origin and takes focus
away from whatever the user meant to reach. And `intersect` of two disjoint clip
rectangles yields an inverted rectangle whose `a` corner passes `is_clipped`, so
a cell can be written outside the clip region.

The single-row and single-column controls the doc comment cares about are
already handled by the `b > a` test on the other axis. The collapsed case should
require `b == a`, and an empty rectangle should contain nothing.

### The SSH host key is written through a permissions race

`src/ssh/server.rs:339` creates the key file with `File::create` — mode 0644
subject to umask — then calls `set_permissions(0o600)`, then writes the key. A
local process that opens the file during that window keeps a readable descriptor
and reads the private key as soon as it is written. Creating the file with the
mode closes the window:

```rust
OpenOptions::new().write(true).create_new(true).mode(0o600).open(path)
```

On Windows the key gets default ACLs and no warning is issued. Either restrict
it there too or state the gap where the user can see it.

### An unreadable host key file is silently replaced

`src/ssh/server.rs:145`: when loading an existing key fails, the code logs a
warning and then overwrites that file with a freshly generated key. This
destroys a private key that may well have been recoverable, and silently changes
the server's identity — which is precisely the event clients' host-key checking
exists to flag. A load error should fail closed; generation belongs only in the
branch where the file does not exist.

### A missing host key falls back to an ephemeral one

`build_russh_config` (`src/ssh/server.rs:186`) and `run_ssh_server` (`:315`)
generate a throwaway Ed25519 key at each start when none is configured. Every
reconnect trips the client's host-key warning, which trains operators to dismiss
it, and makes a genuine machine-in-the-middle indistinguishable from an ordinary
restart. A missing host key should be an error that asks for an explicit
`generate_key()` or `load_or_generate_key(path)`.

### `max_connections` is configured but never enforced

`SshServerConfig::max_connections` is stored (`:57`, `:176`) and documented, but
neither `build_russh_config` nor `TuiServer` ever reads it. Each connection
spawns a blocking thread, so the bound an operator believes they set does not
exist. Enforce it with a semaphore, or remove the knob rather than leave a
setting that quietly does nothing.

### A second channel on one SSH connection is silently dead

`src/ssh/handler.rs:177` overwrites `self.session` unconditionally, and
`app_factory` is a `FnOnce` already consumed by the first `start_tui`. The
second channel receives `channel_success` but never gets a TUI and never
produces output, while input routing has already moved off the first session.
The client hangs on an open, silent channel. A second `channel_open_session`
should be rejected explicitly.

## Suggestions

**There is no CI that builds or tests.** `.github/workflows/` contains only
`publish.yml`. A job running `cargo clippy --all-targets --all-features` and
`cargo test --all-features` would have caught the blocking finding at the commit
that renamed the method, and would keep the SSH feature honest from here on.

**38 doctests are marked `rust,ignore`**, including every example in the SSH
module. They are never type-checked, and they have drifted from the API they
document. `no_run` compiles them without executing them, which is what these
examples actually need.

**`Terminal::shutdown` documents errors it can no longer return.** `cleanup()`
delegates to `restore_terminal()`, which swallows every failure and returns
`Ok(())` unconditionally. Either drop the `# Errors` section or have
`restore_terminal` report the first failure it hits.

**`exec_request` ignores the requested command.** `src/ssh/handler.rs:246`
starts the TUI whatever the client asked to run. That is a defensible policy for
a single-purpose server, but it should be stated in the doc comment rather than
left to be discovered.

**Per-cell origin and clip work.** `write_cell` folds the whole origin stack and
`is_clipped` re-intersects the whole clip stack, once per cell. This is not a
demonstrated hot path, so it is not worth changing today; caching the folded
origin on push and pop is the cheap move if profiling ever points here.

## What holds up

641 tests pass on default features. Clippy reports no errors: three pedantic
warnings in the library, and the 84 in `examples/showcase.rs` are cosmetic.
There is no `unsafe` anywhere in `src/`. Panic sites are confined to tests and
to builder `expect`s that carry clear messages. UTF-8 handling in
`input_line.rs` and `editor.rs` consistently converts character indices to byte
offsets through a clamping helper, and carries regression tests for the
multibyte cases that used to panic.

## Verification gaps

The SSH findings were not reproduced at runtime, because the feature does not
compile. The rendering and geometry findings are proven by reading the code but
have no failing test today; each of the three deserves one, and each of those
tests would fail before the corresponding fix and pass after it.
