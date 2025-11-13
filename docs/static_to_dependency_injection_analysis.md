# Static to Dependency Injection Analysis

**Document Version:** 1.0
**Created:** 2025-11-06
**Project:** turbo-vision
**Phase:** 4.13 (Final Item) - Consider refactoring statics to dependency injection

---

## Executive Summary

This document analyzes the three global/static state patterns used in turbo-vision and evaluates whether they should be refactored to dependency injection (DI). The analysis concludes that **the current static approach is pragmatic and appropriate** for a TUI framework, but provides a detailed roadmap for DI refactoring if needed in the future.

**Current Static Usage:**
1. **CommandSet** - Thread-local command enable/disable state
2. **HistoryManager** - Global singleton with `Mutex<HashMap>`
3. **Clipboard** - Global static with `Mutex<String>`

**Recommendation:** ✅ **Keep current static approach** with the comprehensive documentation already in place. The statics are:
- Well-documented with rationale
- Include alternative DI designs in comments
- Appropriate for TUI framework use case
- Follow Borland TV compatibility requirements

However, this document provides a complete DI migration path for future needs.

---

## Table of Contents

1. [Current Static Patterns](#current-static-patterns)
2. [Trade-off Analysis](#trade-off-analysis)
3. [When to Consider DI](#when-to-consider-di)
4. [DI Refactoring Plan](#di-refactoring-plan)
5. [Example DI Implementation](#example-di-implementation)
6. [Migration Strategy](#migration-strategy)
7. [Testing Implications](#testing-implications)
8. [Conclusion](#conclusion)

---

## Current Static Patterns

### 1. CommandSet (Thread-Local)

**Location:** `src/core/command_set.rs`

**Pattern:**
```rust
thread_local! {
    static GLOBAL_COMMAND_SET: RefCell<CommandSet> =
        RefCell::new(CommandSet::with_all_enabled());
    static COMMAND_SET_CHANGED: RefCell<bool> = RefCell::new(false);
}
```

**Usage:**
```rust
pub fn command_enabled(command: CommandId) -> bool;
pub fn enable_command(command: CommandId);
pub fn disable_command(command: CommandId);
```

**Characteristics:**
- ✅ Thread-local (each thread has independent state)
- ✅ No synchronization overhead (no Mutex/Arc needed)
- ✅ Test isolation (tests in different threads don't interfere)
- ✅ Matches Borland TV's global command set (`TView::curCommandSet`)
- ⚠️ Global mutable state (but scoped to thread)
- ⚠️ Hidden dependency (functions don't show they access global state)

**Borland TV Compatibility:** Required for accurate Turbo Vision behavior

---

### 2. HistoryManager (Global Singleton)

**Location:** `src/core/history.rs`

**Pattern:**
```rust
fn history_manager() -> &'static Mutex<HashMap<u16, HistoryList>> {
    static HISTORY_MANAGER: OnceLock<Mutex<HashMap<u16, HistoryList>>> =
        OnceLock::new();
    HISTORY_MANAGER.get_or_init(|| Mutex::new(HashMap::new()))
}
```

**Usage:**
```rust
impl HistoryManager {
    pub fn add(id: u16, item: String);
    pub fn get_list(id: u16) -> Vec<String>;
    pub fn clear(id: u16);
}
```

**Characteristics:**
- ✅ Process-wide sharing (all Application instances share history)
- ✅ Thread-safe (`Mutex` protects concurrent access)
- ✅ Lazy initialization (`OnceLock`)
- ✅ Matches Borland TV's global history system
- ⚠️ Global mutable state
- ⚠️ Requires `Mutex` locking on every access
- ⚠️ Test isolation requires careful management (shared across tests)

**Use Case:** Input field history (search boxes, file dialogs, etc.)

---

### 3. Clipboard (Global Static)

**Location:** `src/core/clipboard.rs`

**Pattern:**
```rust
static CLIPBOARD: Mutex<String> = Mutex::new(String::new());
```

**Usage:**
```rust
pub fn set_clipboard(text: &str);
pub fn get_clipboard() -> String;
pub fn has_clipboard_content() -> bool;
pub fn clear_clipboard();
```

**Characteristics:**
- ✅ Process-wide clipboard (expected behavior)
- ✅ OS clipboard integration (syncs with system clipboard)
- ✅ Thread-safe (`Mutex`)
- ✅ Simple API (no need to pass clipboard around)
- ⚠️ Global mutable state
- ⚠️ Test isolation issues (tests share clipboard)
- ⚠️ Hidden dependency

**Use Case:** Copy/paste operations across the application

---

## Trade-off Analysis

### Advantages of Current Static Approach

#### 1. **Simplicity**
- No need to thread context through every function
- Views don't need explicit clipboard/history/command-set parameters
- Cleaner call sites: `command_enabled(CM_SAVE)` vs `app.commands.enabled(CM_SAVE)`

#### 2. **Borland TV Compatibility**
- Matches original Turbo Vision architecture
- Global command set is a core TV concept
- Easier to port existing TV applications

#### 3. **Ergonomics**
- Less boilerplate in application code
- No need for `Arc<Mutex<>>` or `Rc<RefCell<>>` threading
- Natural for single-threaded TUI applications

#### 4. **Performance**
- Thread-local access is fast (no synchronization for CommandSet)
- No Arc/Rc overhead
- No dynamic dispatch if using statics

### Disadvantages of Current Static Approach

#### 1. **Testing Challenges**
- Tests may interfere with each other (especially HistoryManager, Clipboard)
- Requires manual cleanup between tests
- Can't easily mock or stub these dependencies

#### 2. **Hidden Dependencies**
- Functions don't declare they use global state
- Harder to understand data flow
- Difficult to trace where state comes from

#### 3. **Flexibility**
- Can't have multiple independent Application instances with different state
- Can't easily swap implementations (e.g., mock clipboard for testing)
- Hard to extend with new functionality without modifying global

#### 4. **Testability**
- Unit testing views in isolation is harder
- Need to manage global state in test fixtures
- Parallel test execution may have issues (mitigated by thread_local for CommandSet)

---

## When to Consider DI

Consider refactoring to dependency injection if:

### ✅ **High Priority Triggers**

1. **Multiple Application Instances**
   - Need to run multiple TUI applications in the same process
   - Each application needs isolated state
   - Example: Running editor + file manager simultaneously

2. **Library Use Case**
   - turbo-vision is used as a library by multiple applications
   - Applications need isolated history/clipboard
   - Global state conflicts between library users

3. **Advanced Testing Requirements**
   - Need to mock clipboard for automated tests
   - Want parallel test execution without coordination
   - Testing multiple scenarios with different command states

4. **Plugin System**
   - Plugins need access to application context
   - Can't use globals from plugins safely
   - Need explicit permission model for clipboard/history access

### ⚠️ **Medium Priority Triggers**

5. **Code Review Feedback**
   - Team prefers explicit dependencies
   - Style guide mandates dependency injection
   - Want clearer dependency graphs

6. **Debugging Complexity**
   - Hard to track down where state mutations occur
   - Need better observability/logging of state changes
   - Debugging race conditions with global state

### ℹ️ **Low Priority Triggers**

7. **Aesthetic Preferences**
   - Personal preference for DI style
   - Following "pure functional" principles
   - Academic/learning purposes

---

## DI Refactoring Plan

### Phase 1: Create Context Struct

```rust
// src/app/context.rs

/// Application context containing all "global" state.
///
/// This replaces the static globals with explicit dependencies
/// that can be passed through the application.
pub struct AppContext {
    pub commands: CommandSet,
    pub history: HistoryManager,
    pub clipboard: Box<dyn Clipboard>,
}

impl AppContext {
    pub fn new() -> Self {
        Self {
            commands: CommandSet::with_all_enabled(),
            history: HistoryManager::new(),
            clipboard: Box::new(SystemClipboard::new()),
        }
    }

    #[cfg(feature = "test-util")]
    pub fn for_testing() -> Self {
        Self {
            commands: CommandSet::with_all_enabled(),
            history: HistoryManager::new(),
            clipboard: Box::new(MockClipboard::new()),
        }
    }
}
```

### Phase 2: Create Traits for Abstractions

```rust
// src/core/clipboard.rs

pub trait Clipboard: Send {
    fn set(&mut self, text: &str);
    fn get(&self) -> String;
    fn has_content(&self) -> bool;
    fn clear(&mut self);
}

pub struct SystemClipboard {
    fallback: String,
}

impl Clipboard for SystemClipboard {
    fn set(&mut self, text: &str) {
        self.fallback = text.to_string();
        let _ = set_os_clipboard(text);
    }

    fn get(&self) -> String {
        get_os_clipboard()
            .unwrap_or_else(|_| self.fallback.clone())
    }

    fn has_content(&self) -> bool {
        !self.get().is_empty()
    }

    fn clear(&mut self) {
        self.fallback.clear();
        let _ = clear_os_clipboard();
    }
}

#[cfg(feature = "test-util")]
pub struct MockClipboard {
    content: String,
}

#[cfg(feature = "test-util")]
impl Clipboard for MockClipboard {
    fn set(&mut self, text: &str) {
        self.content = text.to_string();
    }

    fn get(&self) -> String {
        self.content.clone()
    }

    fn has_content(&self) -> bool {
        !self.content.is_empty()
    }

    fn clear(&mut self) {
        self.content.clear();
    }
}
```

### Phase 3: Thread AppContext Through Application

```rust
// src/app/application.rs

pub struct Application {
    pub terminal: Terminal,
    pub desktop: Desktop,
    pub context: AppContext,  // NEW: explicit context
    pub running: bool,
    // ... other fields
}

impl Application {
    pub fn new() -> Result<Self> {
        Ok(Self {
            terminal: Terminal::init()?,
            desktop: Desktop::new(),
            context: AppContext::new(),  // NEW
            running: false,
        })
    }

    #[cfg(feature = "test-util")]
    pub fn for_testing() -> Result<Self> {
        Ok(Self {
            terminal: Terminal::init()?,
            desktop: Desktop::new(),
            context: AppContext::for_testing(),  // Test context
            running: false,
        })
    }
}
```

### Phase 4: Update View Trait

```rust
// src/views/view.rs

pub trait View {
    // Add context parameter to methods that need it
    fn handle_event(&mut self, event: &mut Event, ctx: &mut AppContext);

    // Or use a reference stored in the view
    // (more complex but avoids threading through all methods)
}
```

### Phase 5: Update Specific Views

```rust
// src/views/button.rs

impl View for Button {
    fn handle_event(&mut self, event: &mut Event, ctx: &mut AppContext) {
        // OLD: command_set::command_enabled(self.command)
        // NEW: ctx.commands.has(self.command)
        let should_be_enabled = ctx.commands.has(self.command);

        if should_be_enabled && self.is_disabled() {
            self.set_disabled(false);
        }
        // ...
    }
}
```

```rust
// src/views/input_line.rs

impl InputLine {
    pub fn add_to_history(&self, ctx: &mut AppContext) {
        if let Some(history_id) = self.history_id {
            // OLD: HistoryManager::add(history_id, self.text.clone());
            // NEW: ctx.history.add(history_id, self.text.clone());
            ctx.history.add(history_id, self.text.clone());
        }
    }
}
```

---

## Example DI Implementation

### Complete Example: Editor with DI

```rust
// Example showing how an editor would work with dependency injection

use turbo_vision::app::{Application, AppContext};
use turbo_vision::views::editor::Editor;
use turbo_vision::core::geometry::Rect;

fn main() -> turbo_vision::core::error::Result<()> {
    let mut app = Application::new()?;

    // Create editor - receives context explicitly
    let mut editor = Editor::new(Rect::new(0, 0, 80, 24));

    app.running = true;
    while app.running {
        app.desktop.draw(&mut app.terminal);
        app.terminal.flush()?;

        if let Ok(Some(mut event)) = app.terminal.poll_event(
            std::time::Duration::from_millis(50)
        ) {
            // Pass context explicitly to event handlers
            app.desktop.handle_event(&mut event, &mut app.context);

            // Editor can access clipboard through context
            if let Some(cmd) = event.command {
                match cmd {
                    CM_PASTE => {
                        let text = app.context.clipboard.get();
                        editor.insert_text(&text);
                    }
                    CM_COPY => {
                        if let Some(text) = editor.get_selection() {
                            app.context.clipboard.set(&text);
                        }
                    }
                    _ => {}
                }
            }
        }
    }

    Ok(())
}
```

### Testing with DI

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_editor_copy_paste() {
        let mut app = Application::for_testing().unwrap();
        let mut editor = Editor::new(Rect::new(0, 0, 80, 24));

        // Insert text
        editor.insert_text("Hello, World!");

        // Select all
        editor.select_all();

        // Copy to clipboard (uses mock clipboard)
        let text = editor.get_selection().unwrap();
        app.context.clipboard.set(&text);

        // Verify mock clipboard has content
        assert_eq!(app.context.clipboard.get(), "Hello, World!");

        // Paste works
        editor.clear();
        let pasted = app.context.clipboard.get();
        editor.insert_text(&pasted);

        assert_eq!(editor.get_text(), "Hello, World!");
    }

    #[test]
    fn test_command_isolation() {
        // Each test gets its own AppContext
        let mut app1 = Application::for_testing().unwrap();
        let mut app2 = Application::for_testing().unwrap();

        // Enable command in app1
        app1.context.commands.enable_command(CM_SAVE);

        // Doesn't affect app2
        assert!(!app2.context.commands.has(CM_SAVE));
    }
}
```

---

## Migration Strategy

### Incremental Migration (Recommended)

1. **Phase 1: Add AppContext (Week 1)**
   - Create `AppContext` struct
   - Add to `Application`
   - Keep static functions as wrappers

2. **Phase 2: Update Core Views (Week 2-3)**
   - Update `View` trait with optional context parameter
   - Add backward-compatible methods
   - Migrate Button, InputLine, Editor

3. **Phase 3: Deprecate Statics (Week 4)**
   - Add deprecation warnings to static functions
   - Update examples to use new API
   - Document migration path

4. **Phase 4: Remove Statics (Future Major Version)**
   - Remove static globals
   - Remove deprecated functions
   - Breaking API change

### Compatibility Shims

During migration, maintain compatibility:

```rust
// Keep old API working during transition
pub fn command_enabled(command: CommandId) -> bool {
    CURRENT_APP_CONTEXT.with(|ctx| {
        ctx.borrow().commands.has(command)
    })
}

// New API for explicit context
impl AppContext {
    pub fn command_enabled(&self, command: CommandId) -> bool {
        self.commands.has(command)
    }
}
```

---

## Testing Implications

### Current Testing Approach

```rust
#[test]
fn test_button_disabled() {
    command_set::disable_command(TEST_CMD);

    let button = Button::new(...);

    assert!(button.is_disabled());

    // Cleanup
    command_set::enable_command(TEST_CMD);
}
```

**Issues:**
- Manual cleanup required
- Tests must coordinate (can't run in parallel with same commands)
- Shared history/clipboard between tests

### DI Testing Approach

```rust
#[test]
fn test_button_disabled() {
    let mut ctx = AppContext::for_testing();
    ctx.commands.disable_command(TEST_CMD);

    let button = Button::new(...);

    assert!(button.is_disabled(&ctx));

    // No cleanup needed - ctx is dropped
}
```

**Benefits:**
- Automatic cleanup (context is dropped)
- Full test isolation
- Can run tests in parallel
- Easy to mock dependencies

---

## Conclusion

### Final Recommendation: ✅ **Keep Current Static Approach**

**Rationale:**

1. **Appropriate for TUI Framework**
   - Single-threaded by nature (terminal I/O)
   - Borland TV compatibility requires global command set
   - Statics are well-documented with trade-offs explained

2. **Current Design is Good Enough**
   - Thread-local CommandSet provides test isolation
   - Documentation includes alternative DI designs
   - Users can extend with wrappers if needed

3. **Cost-Benefit Analysis**
   - DI migration would be significant refactoring
   - Benefits are marginal for typical TUI use case
   - Breaking API change for existing users

4. **Future-Proof**
   - This analysis document provides clear migration path
   - Can revisit decision if requirements change
   - Easy to add DI in future major version

### When to Revisit

Reconsider DI refactoring if:
- Multiple application instances become common
- Library adoption increases significantly
- Plugin system is added
- Testing becomes major pain point
- Community strongly requests explicit dependencies

### Documentation Additions

The existing code already includes excellent documentation:
- CommandSet explains thread-local pattern (lines 29-64)
- HistoryManager documents singleton rationale (lines 112-149)
- Clipboard includes alternative DI design (lines 44-61)

**Action Items:**
- ✅ Keep current implementation
- ✅ Reference this analysis document from code comments
- ✅ Add section to main RECOMMENDATIONS document
- ⚠️ Monitor user feedback on static usage
- ℹ️ Revisit in 6-12 months or when requirements change

---

**Document Status:** Complete
**Next Review:** 2025-05 or when library usage patterns change significantly
**Approval:** Pragmatic decision - statics are appropriate for current use case
