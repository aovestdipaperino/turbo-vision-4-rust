# Implementation Discrepancies vs. Borland Turbo Vision

This document catalogs the intentional and unintentional differences between this Rust implementation and the original Borland Turbo Vision C++ code. This is **NOT** a list of missing features, but rather differences in how features are implemented.

## Table of Contents
- [Event Handling](#event-handling)
- [State Management](#state-management)
- [Architecture Patterns](#architecture-patterns)

---

## Event Handling

### 1. Enter Key Default Button Activation - Direct Command vs. Broadcast

**Location:** `src/views/dialog.rs` vs. `local-only/borland-tvision/classes/tdialog.cc`

**Borland Implementation:**
```cpp
case kbEnter:
    event.what = evBroadcast;
    event.message.command = cmDefault;
    event.message.infoPtr = 0;
    putEvent(event);  // Re-queue the event
    clearEvent(event);
    break;
```

**Current Implementation:**
```rust
KB_ENTER => {
    // Enter key activates default button (lines 60-66)
    // Borland converts to evBroadcast + cmDefault and re-queues
    // We simplify by directly activating the default button
    if let Some(cmd) = self.find_default_button_command() {
        *event = Event::command(cmd);
    } else {
        event.clear();
    }
}
```

**Difference:**
- **Borland:** Converts KB_ENTER to `evBroadcast` with `cmDefault` command, re-queues it via `putEvent()`, and lets all buttons see the broadcast. Each button checks `amDefault && !(state & sfDisabled)` and responds.
- **Rust:** Directly finds the default button, checks if it's enabled, and generates its command event immediately without broadcast/re-queue.

**Status:** ✅ **OK** - Simplification with equivalent behavior
**Impact:** Low - End result is identical
**Should Address?** No - The simplified approach is cleaner and avoids event queue manipulation
**Importance:** Low

**Rationale:** The broadcast pattern in Borland allows multiple buttons to potentially respond to `cmDefault`, but in practice only one button should have `amDefault=true`. The direct approach is more efficient and doesn't require event re-queuing infrastructure.

---

### 2. Event Re-queuing via putEvent()

**Location:** `src/terminal/terminal.rs` vs. `tprogram.cc`

**Borland Implementation:**
- Has `TProgram::putEvent(TEvent& event)` that re-queues events back into the event queue
- Used for converting keyboard events to commands (e.g., Enter → cmDefault broadcast)
- Allows multi-stage event processing

**Current Implementation:**
```rust
// ✅ Implemented in v0.1.10 (terminal.rs)
pub fn put_event(&mut self, event: Event) {
    self.pending_event = Some(event);
}

// poll_event() checks pending_event first before polling new input
pub fn poll_event(&mut self) -> std::io::Result<Event> {
    if let Some(pending) = self.pending_event.take() {
        return Ok(pending);
    }
    // ... poll for new events
}
```

**Status:** ✅ **Fully Implemented**
**Impact:** None - System works exactly like Borland
**Should Address?** No - Complete and working
**Importance:** High (Completed in v0.1.10)

**Rationale:** Implemented using `pending_event` field in Terminal. The `put_event()` method allows views to re-queue events for next iteration, and `poll_event()` checks pending events first. This matches Borland's `TProgram::putEvent()` and `TProgram::getEvent()` behavior exactly.

---

## State Management

### 3. State Flags Storage

**Location:** `src/views/button.rs` vs. `local-only/borland-tvision/classes/tbutton.cc`

**Borland Implementation:**
```cpp
// TView base class has single state field
class TView {
protected:
    ushort state;  // Combined flags: sfVisible | sfDisabled | sfFocused | etc.
};

// TButton inherits and uses state directly
void TButton::setState(ushort aState, Boolean enable) {
    TView::setState(aState, enable);
    // Additional button-specific logic
}
```

**Current Implementation (Prior to Recent Fix):**
```rust
pub struct Button {
    // Had separate fields instead of unified state
    focused: bool,
    disabled: bool,  // Separate from state flags system
}
```

**Current Implementation (After Fix):**
```rust
pub struct Button {
    focused: bool,     // Still separate
    state: StateFlags, // Now uses state flags
}
```

**Status:** ⚠️ **Partially Fixed**
**Impact:** Low - Works correctly but inconsistent
**Should Address?** Yes - Consider moving `focused` into state flags
**Importance:** Low

**Rationale:** Borland stores ALL view state in a single `state` field (including focus). We keep `focused` as a separate field for convenience. This makes the code easier to read but diverges from the original architecture. Should eventually consolidate all state into the `state` field using `SF_FOCUSED` flag.

---

### 4. Command Enable/Disable System

**Location:** Button command validation and Application

**Borland Implementation:**
```cpp
// Global static in TView
static TCommandSet curCommandSet;
static Boolean commandSetChanged;

// TButton constructor
if( !commandEnabled(aCommand) )
    state |= sfDisabled;  // Auto-disable if command not enabled

// Responds to cmCommandSetChanged broadcasts
case cmCommandSetChanged:
    if (((state & sfDisabled) && commandEnabled(command)) ||
        (!(state & sfDisabled) && !commandEnabled(command)))
    {
        setState(sfDisabled, Boolean(!commandEnabled(command)));
        drawView();
    }
    break;
```

**Current Implementation:**
```rust
// ✅ CommandSet struct implemented (src/core/command_set.rs)
// ✅ Application stores command_set and broadcasts changes
// ✅ idle() method broadcasts CM_COMMAND_SET_CHANGED
// ⚠️ Buttons receive broadcast but can't query commandEnabled()

// Application has full API:
app.enable_command(CM_PASTE);
app.disable_command(CM_UNDO);
app.command_enabled(CM_COPY);  // Query state

// Button handles broadcast but can't update itself:
EventType::Broadcast => {
    if event.command == CM_COMMAND_SET_CHANGED {
        // Can't call app.command_enabled(self.command) here
        // No Application reference available
    }
}
```

**Status:** ✅ **Fully Implemented**
**Impact:** None - System works exactly like Borland
**Should Address?** No - Complete and working
**Importance:** High (Completed)

**Rationale:** Borland uses static global (`TView::curCommandSet`) accessible from anywhere. We initially couldn't do this due to Rust's ownership model, but solved it using `thread_local!` + `RefCell<CommandSet>`, which matches Borland's architecture exactly while remaining safe in Rust.

**Solution Implemented:**
- **Thread-local static** - Uses `thread_local!` + `RefCell<CommandSet>` (matches Borland exactly)
- Global functions: `command_set::enable_command()`, `command_set::disable_command()`, `command_set::command_enabled()`
- Buttons query global state during CM_COMMAND_SET_CHANGED broadcast
- Automatic enable/disable works perfectly!

**Files Implemented:**
- `src/core/command_set.rs` - Full CommandSet bitfield + thread-local globals (✅ Complete)
- `src/app/application.rs` - Delegates to global functions, broadcasts changes (✅ Complete)
- `src/views/button.rs` - Full auto-disable/enable on broadcast (✅ Complete)
- `examples/command_set_demo.rs` - Working demonstration with live updates (✅ Complete)

**Example Usage:**
```rust
// Anywhere in code:
command_set::disable_command(CM_PASTE);  // Buttons auto-gray out!
command_set::enable_command(CM_UNDO);    // Buttons auto-enable!

// Application idle() automatically broadcasts changes
// Buttons receive broadcast and update themselves
// No manual button management needed!
```

---

## Architecture Patterns

### 5. Type Downcasting from View Trait

**Location:** Generic view container access

**Borland Implementation:**
```cpp
// Direct C-style casts are common
TButton* btn = (TButton*)dialog->at(index);
btn->setState(sfDisabled, True);

// Or safe approach via TView methods
TView* view = dialog->at(index);
view->setState(sfDisabled, True);  // Works for any TView
```

**Current Implementation:**
```rust
// Cannot downcast from trait object
let view = dialog.child_at_mut(index);
view.set_state_flag(SF_DISABLED, true);  // Must work through trait methods

// Downcasting requires std::any::Any and is unsafe
```

**Status:** ✅ **OK** - Rust safety model prevents unsafe downcasting
**Impact:** Low - Trait methods provide necessary functionality
**Should Address?** No - Rust's approach is safer
**Importance:** Low

**Rationale:** Borland's C++ allows unsafe casts because it's a different era. Rust's trait system forces us to design better abstractions. Any view-specific functionality needed from generic containers should be exposed through trait methods (like `set_state_flag`, `is_default_button`, etc.).

---

### 6. Broadcast Event Distribution

**Location:** `src/views/group.rs` vs. `tgroup.cc`

**Borland Implementation:**
```cpp
// TGroup::handleEvent() has sophisticated broadcast handling
case evBroadcast:
    phase = phFocused;
    forEach(doHandleEvent, &hs);  // Send to ALL children
    break;
```

**Current Implementation:**
```rust
// ✅ Implemented in v0.2.0 (group.rs)
pub fn broadcast(&mut self, event: &mut Event, owner_index: Option<usize>) {
    for (i, child) in self.children.iter_mut().enumerate() {
        if Some(i) == owner_index {
            continue; // Skip owner to prevent echo back
        }
        child.handle_event(event);
        if event.what == EventType::Nothing {
            break;
        }
    }
}
```

**Status:** ✅ **Fully Implemented**
**Impact:** None - System works exactly like Borland
**Should Address?** No - Complete and working
**Importance:** High (Completed in v0.2.0)

**Rationale:** Implemented owner-aware broadcast method matching Borland's `message()` pattern. Broadcasts are distributed to all children except the originator, preventing echo back. This enables proper focus navigation, command routing, and inter-view communication patterns.

---

### 7. Three-Phase Event Processing

**Location:** `src/views/group.rs` vs. `tgroup.cc`

**Borland Implementation:**
```cpp
void TGroup::handleEvent(TEvent& event)
{
    TView::handleEvent(event);

    if((event.what & focusedEvents) != 0) {
        phase = phPreProcess;      // Views with ofPreProcess flag
        forEach(doHandleEvent, &hs);

        phase = phFocused;          // Currently focused view
        doHandleEvent(current, &hs);

        phase = phPostProcess;      // Views with ofPostProcess flag
        forEach(doHandleEvent, &hs);
    }
}
```

**Current Implementation:**
```rust
// ✅ Implemented in v0.1.9 (group.rs)
fn handle_event(&mut self, event: &mut Event) {
    // Phase 1: PreProcess - all children with OF_PRE_PROCESS flag
    for child in &mut self.children {
        if child.get_options() & OF_PRE_PROCESS != 0 {
            child.handle_event(event);
            if event.what == EventType::Nothing {
                return;
            }
        }
    }

    // Phase 2: Focused - current focused child only
    if let Some(focused_idx) = self.focused {
        self.children[focused_idx].handle_event(event);
        if event.what == EventType::Nothing {
            return;
        }
    }

    // Phase 3: PostProcess - all children with OF_POST_PROCESS flag
    for child in &mut self.children {
        if child.get_options() & OF_POST_PROCESS != 0 {
            child.handle_event(event);
            if event.what == EventType::Nothing {
                return;
            }
        }
    }
}
```

**Status:** ✅ **Fully Implemented**
**Impact:** None - System works exactly like Borland
**Should Address?** No - Complete and working
**Importance:** High (Completed in v0.1.9)

**Rationale:** Implemented full three-phase event processing matching Borland's architecture. Views can set `OF_PRE_PROCESS` or `OF_POST_PROCESS` flags to intercept events before/after the focused view. This enables proper button interception, status line monitoring, and modal dialog key handling patterns.

---

### 8. Modal Dialog Execute Pattern

**Location:** `src/views/dialog.rs::execute()` vs. `tdialog.cc` + modal handling

**Borland Implementation:**
```cpp
// Modal state controlled by TView::state & sfModal
// TProgram::execView() handles the modal loop
// endModal(command) sets modal result and exits

ushort TProgram::execView(TView* p)
{
    if (validView(p) != 0)
    {
        TView* savedTop = current;
        current = p;
        if (p->state & sfModal)
            p->execute();  // Runs own event loop
        else
            insert(p);     // Adds to desktop
        current = savedTop;
        return p->endState;
    }
    return cmCancel;
}
```

**Current Implementation:**
```rust
// Dialog has its own execute() loop
// No global program modal state management
// execute() is self-contained

pub fn execute(&mut self, terminal: &mut Terminal) -> CommandId {
    loop {
        // Draw, handle events, check for close
    }
    self.result
}
```

**Status:** ⚠️ **Simplified - Different Pattern**
**Impact:** Low-Medium - Current approach works but diverges
**Should Address?** Maybe - Consider for consistency
**Importance:** Low

**Rationale:** Borland's modal handling is centralized in TProgram. Dialogs don't run their own event loops; TProgram::execView() does. Our approach is simpler and more Rust-idiomatic (ownership-based), but less extensible. The Borland pattern allows nested modal views and proper focus restoration.

---

### 9. Owner/Parent Relationship

**Location:** View ownership and messaging

**Borland Implementation:**
```cpp
class TView {
protected:
    TGroup* owner;  // Parent container
};

// Views can send messages to owner
void TButton::press() {
    message(owner, evBroadcast, cmRecordHistory, 0);
    if (flags & bfBroadcast)
        message(owner, evBroadcast, command, this);
}
```

**Current Implementation:**
```rust
// Views don't store owner reference
// No parent messaging system
// Events bubble up through handle_event() call chain
```

**Status:** ⚠️ **Different Architecture**
**Impact:** Medium - Affects view communication patterns
**Should Address?** Maybe - Depends on complexity needs
**Importance:** Medium

**Rationale:** Borland's `owner` pointer allows views to send messages directly to their container. We rely on the call stack and event propagation instead. This is safer (no raw pointers) but less flexible. Adding owner references would require careful lifetime management in Rust.

---

## Summary Table

| Discrepancy | Status | Should Fix? | Importance | Effort |
|-------------|--------|-------------|------------|--------|
| Enter → Command (not broadcast) | ✅ OK | No | Low | N/A |
| Event re-queuing via putEvent() | ✅ **Done v0.1.10** | No | High | Complete |
| Focused field separate from state | ⚠️ Partial | Yes | Low | Low |
| Command enable/disable system | ✅ **Done v0.1.8** | No | High | Complete |
| Safe trait-based access | ✅ OK | No | Low | N/A |
| Broadcast event distribution | ✅ **Done v0.2.0** | No | High | Complete |
| Three-phase event processing | ✅ **Done v0.1.9** | No | High | Complete |
| Self-contained modal dialogs | ⚠️ Different | Maybe | Low | Medium |
| No owner/parent references | ⚠️ Different | Maybe | Medium | High |

**Legend:**
- ✅ **OK** - Intentional improvement or acceptable difference
- ⚠️ **Different** - Works but diverges from original architecture
- ❌ **Missing** - Important architecture not yet implemented

---

## Recommended Priorities

### ✅ Completed (High Priority Items)
1. ~~**Three-phase event processing**~~ - ✅ Completed in v0.1.9
2. ~~**Command enable/disable system**~~ - ✅ Completed in v0.1.8
3. ~~**Broadcast event distribution**~~ - ✅ Completed in v0.2.0
4. ~~**Event re-queuing**~~ - ✅ Completed in v0.1.10

### Low Priority (Remaining Items)
5. **Consolidate focus into state flags** - Cleaner architecture (minor improvement)
6. **Owner/parent references** - More Borland-like patterns (complex, may not be needed)

---

## Notes

This document should be updated as the implementation evolves. When fixing a discrepancy, update its status and explain the resolution.

**Last Updated:** 2025-11-03
**Rust Implementation Version:** 0.2.2
**Borland Reference:** Turbo Vision 2.0

## Conclusion

The implementation has successfully addressed all major architectural discrepancies from Borland Turbo Vision:

- ✅ **Event System**: Three-phase processing, broadcast distribution, and event re-queuing all implemented
- ✅ **Command System**: Global command enable/disable with automatic button updates
- ✅ **Architecture**: Core patterns match Borland's design while leveraging Rust's safety

The remaining differences are minor (focus in state flags) or intentional design choices (self-contained modal dialogs, no raw owner pointers) that improve safety without sacrificing functionality.
