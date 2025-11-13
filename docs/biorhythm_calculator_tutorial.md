# Building a Biorhythm Calculator with Real-Time Validation in Turbo Vision

Have you ever wanted to build a terminal UI application that feels polished and responsive, with validation that updates as you type? Today, we're diving deep into the Biorhythm Calculator example—a complete Turbo Vision application that showcases some fascinating patterns you won't find in typical TUI frameworks.

What makes this interesting isn't just the biorhythm calculations (those theoretical cycles of physical, emotional, and intellectual states). It's how we handle *real-time validation*—watching every keystroke, enabling and disabling buttons dynamically, and making it all feel natural. Along the way, we'll discover how automatic centering simplifies layout, when to abandon standard dialog loops, and how Turbo Vision's command system makes everything click together.

Let's build something that feels right.

---

## The Challenge: Making Validation Feel Natural

Picture this: You open the app, and a dialog asks for your birthdate. You start typing in the month field. You type "1", "5"—wait, month 15? The OK button should be grayed out. You backspace, type "1", "2" instead. December. The button lights up again.

That's the goal. Validation that *responds* to you, not validation that scolds you after you've already clicked OK.

But here's the problem: Turbo Vision's standard `dialog.execute()` doesn't give us that kind of control. It shows the dialog, waits for OK or Cancel, and returns. No hooks for "validate after every keystroke." No way to reach in and toggle button states on the fly.

So we're going to build our own event loop. And we're going to learn some things along the way.

---

## First Impression: What It Feels Like

When you run the biorhythm calculator, here's what happens:

1. The app starts, and immediately a centered dialog appears asking for your birthdate
2. All three fields (day, month, year) are empty
3. The OK button is grayed out—disabled
4. You start typing. The instant your date becomes valid, the OK button lights up
5. If you make it invalid (February 31st, anyone?), it grays out again
6. Press Enter or click OK, and the chart window appears, perfectly centered
7. If you press Cancel instead, the app just... exits. Clean. No orphaned windows.

It *feels* polished. Like someone thought about the details.

Let's see how that polish happens.

---

## The Startup Dance: Dialog First, Windows Later

Here's a subtle but important decision: When does the chart window get created?

**The wrong way**: Create the window at startup, then show the dialog on top of it.

Why wrong? Because if the user cancels, you've got an empty chart window sitting there. Or worse—you show an empty window briefly before the dialog appears, a visual glitch that screams "unfinished."

**The right way**: Show the dialog *first*. Only create the chart window after the user clicks OK.

```rust
fn main() -> std::io::Result<()> {
    let mut app = Application::new()?;

    // Create menu bar and status line
    // ...

    // Calculate where the window WILL go (but don't create it yet)
    // Note: Main window uses manual positioning to account for menu bar
    let window_x = (width as i16 - (window_width + 2)) / 2;
    let window_y = 1 + ((height as i16 - 2) - (window_height + 1)) / 2;

    // Show the birthdate dialog FIRST (uses OF_CENTERED for auto-centering)
    let result = show_birthdate_dialog(...);

    // User canceled? Just exit.
    if result == CM_CANCEL {
        return Ok(());
    }

    // User clicked OK? NOW create the window.
    let mut main_window = Window::new(...);
    app.desktop.add(Box::new(main_window));

    // Enter main event loop
    app.running = true;
    while app.running { ... }
}
```

See that early return? If the user cancels, we never create the window. The app just exits. Clean.

This is one of those details that makes an app feel *intentional*.

---

## Automatic Centering: Let Turbo Vision Handle the Details

Here's something nice: Turbo Vision has an `OF_CENTERED` option that automatically centers windows and dialogs for you.

```rust
use turbo_vision::core::state::OF_CENTERED;

// Create dialog with any position - OF_CENTERED will reposition it
let mut dialog = Dialog::new(
    Rect::new(0, 0, dialog_width, dialog_height),
    "Enter Birth Date"
);

// Enable automatic centering (matches Borland's ofCentered option)
dialog.set_options(OF_CENTERED);
```

That's it. When the dialog is added to the desktop, it'll be perfectly centered.

### Why This Matters: The Shadow Problem

Under the hood, `OF_CENTERED` handles a subtle detail that's easy to get wrong: **shadows**.

Windows have shadows—2 columns to the right and 1 row down. A 50-character dialog actually takes up 52 characters of visual space. If you manually center based on the dialog width alone, it'll look off-center to the right.

```rust
// ❌ WRONG: Ignores shadow, looks off-center
let x = (screen_width - window_width) / 2;

// ✅ RIGHT: Accounts for shadow
let x = (screen_width - (window_width + 2)) / 2;
//                                     ↑
//                              shadow width
```

`OF_CENTERED` handles this automatically. You just specify the dialog dimensions, and the framework calculates the perfect position, shadow included.

### When To Use Manual Positioning

You'd still manually calculate positions when:
- Positioning relative to other windows
- Cascading multiple windows
- Specific layout requirements (toolbars, side panels, etc.)

But for "just center this dialog," `OF_CENTERED` is the way.

---

## Building the Custom Event Loop: Why Standard `execute()` Isn't Enough

Let's talk about why we can't just use `dialog.execute()`.

The standard pattern is simple:

```rust
let result = dialog.execute(&mut app);
if result == CM_OK {
    // Do something with the data
}
```

Easy. Clean. But here's what it *doesn't* give us:

1. **Validation after every keystroke** - We can't check if the date is valid until after OK is clicked
2. **Dynamic button states** - We can't enable/disable the OK button based on current input
3. **Immediate feedback** - The user types "15" in the month field and has to wait until they click OK to learn it's invalid

We need more control. We need our *own* event loop.

Here's the skeleton:

```rust
// Set up the dialog as modal
let old_state = dialog.state();
dialog.set_state(old_state | SF_MODAL);

let mut result = CM_CANCEL;

loop {
    // Draw everything
    app.desktop.draw(&mut app.terminal);
    dialog.draw(&mut app.terminal);
    dialog.update_cursor(&mut app.terminal);
    app.terminal.flush();

    // Wait for an event
    if let Some(mut event) = app.terminal.poll_event(Duration::from_millis(50)) {
        // Let the dialog handle it
        dialog.handle_event(&mut event);

        // Here's where the magic happens: validate after EVERY event
        let is_valid = validate_birth_date(&day, &month, &year);
        update_button_state(is_valid);
    }

    // Check if user clicked OK or Cancel
    let end_state = dialog.get_end_state();
    if end_state != 0 {
        result = end_state;
        break;
    }
}

// Clean up
dialog.set_state(old_state);
```

See that `validate_birth_date()` call inside the loop? That's the key. Every keystroke triggers an event. Every event triggers validation. Every validation updates the button state.

That's how we get real-time feedback.

---

## The Three Layers of Validation

Now let's talk about how we actually *validate* dates. There are three layers, and they all matter:

### Layer 1: RangeValidator (Character Filtering)

This runs *as you type*, filtering out invalid characters:

```rust
// Only allow digits 1-31 for day field
let day_validator = Rc::new(RefCell::new(RangeValidator::new(1, 31)));
day_input.set_validator(day_validator);
```

This stops you from typing "abc" in the day field. But it doesn't know about February having only 28 days.

### Layer 2: Complete Date Validation

This is the real validation—the one that checks everything:

```rust
fn validate_birth_date(day_str: &str, month_str: &str, year_str: &str) -> bool {
    // Empty fields? Invalid.
    if day_str.trim().is_empty() || month_str.trim().is_empty()
       || year_str.trim().is_empty() {
        return false;
    }

    // Can we parse them?
    let (Ok(day), Ok(month), Ok(year)) = (
        day_str.parse::<u32>(),
        month_str.parse::<u32>(),
        year_str.parse::<i32>(),
    ) else {
        return false;
    };

    // In range?
    if day < 1 || day > 31 || month < 1 || month > 12
       || year < 1900 || year > 2100 {
        return false;
    }

    // Valid for that month? (Feb 31 = invalid)
    if day > days_in_month(month, year) {
        return false;
    }

    // Not in the future?
    calculate_days_alive(year, month, day).is_some()
}
```

This catches everything: empty fields, invalid ranges, impossible dates (February 31st), and even future dates.

### Layer 3: Command Set Integration

This is where we *act* on the validation:

```rust
if is_valid {
    command_set::enable_command(CM_OK);
} else {
    command_set::disable_command(CM_OK);
}

// Broadcast the change to all buttons
if command_set::command_set_changed() {
    let mut event = Event::broadcast(CM_COMMAND_SET_CHANGED);
    dialog.handle_event(&mut event);
    command_set::clear_command_set_changed();
}
```

This is Turbo Vision's way of managing button states *globally*. Instead of reaching into the dialog and finding the OK button to disable it (fragile!), we just say "the CM_OK command is disabled." Every button that uses CM_OK will automatically gray itself out.

Elegant. Scalable. The Turbo Vision way.

---

## The Command Set Pattern: Why Global State Isn't Always Bad

Let's pause here because this is important: In Turbo Vision, buttons don't manage their own enabled/disabled state. There's a *global command set*—a bitfield of which commands are currently enabled.

When you disable a command:

```rust
command_set::disable_command(CM_OK);
```

...you're flipping a bit in a global array. Then you broadcast `CM_COMMAND_SET_CHANGED`, and every button in the entire UI checks: "Hey, is my command still enabled?" If not, it grays itself out.

**Why is this better than directly manipulating buttons?**

1. **No fragile child indices**: You don't need to know that the OK button is child #8 of the dialog
2. **Scales automatically**: If you have 5 buttons that all use CM_SAVE, disabling the command disables all 5
3. **Separation of concerns**: Your validation code doesn't need to know about UI structure
4. **Matches Borland**: This is exactly how Turbo Vision C++ worked

Compare the wrong way:

```rust
// ❌ Fragile! Depends on button being child #8
let button = dialog.child_at_mut(8).downcast_mut::<Button>();
button.set_disabled(true);
```

...to the right way:

```rust
// ✅ Robust! Works for all buttons with this command
command_set::disable_command(CM_OK);
broadcast_changes();
```

The second approach is *declarative*. "This command is disabled." The buttons figure out the rest.

---

## The Enter Key Mystery: Event Reprocessing

Here's a weird one I stumbled into: You're in the dialog, focus is on the year field, you've entered a valid date. You press Enter.

Nothing happens.

Wait, what?

Turns out, there's a subtle bug in how events get processed. Here's what *should* happen:

1. You press Enter
2. Dialog sees: "User pressed Enter in an InputLine, and there's a default button (OK)"
3. Dialog converts the event: KB_ENTER → CM_OK
4. Dialog processes the command, which calls `end_modal(CM_OK)`
5. Dialog closes

But step 4 never happens! Why?

Because `handle_event()` only gets called *once* per iteration:

```rust
// Inside the event loop
dialog.handle_event(&mut event);  // Converts KB_ENTER to CM_OK
// Loop continues, polls for a NEW event
// The CM_OK command never gets processed!
```

The dialog converts the event *in place*, but then the loop goes back to polling for a new event. The converted command is lost.

**The fix**: Process the event twice if it was converted to a command:

```rust
dialog.handle_event(&mut event);

// If the event was converted to a command, process it again
if event.what == EventType::Command {
    dialog.handle_event(&mut event);
}
```

Now the workflow is:
1. First `handle_event()`: Converts KB_ENTER → CM_OK
2. Second `handle_event()`: Processes the CM_OK command
3. Dialog closes properly

This matches how Borland Turbo Vision did it—converted events got re-queued via `putEvent()`.

It's a subtle thing. But without it, the Enter key doesn't work. And a dialog where Enter doesn't work feels *broken*.

---

## The Initial State Problem: Don't Forget to Validate First

Here's a mistake I made: I built the custom event loop, added validation after every event, and ran it.

The OK button was enabled at startup, even though all the fields were empty.

Why? Because I only validated *after* events. I never validated the *initial state*.

The fix:

```rust
// BEFORE entering the event loop, validate the initial state
let is_valid = validate_birth_date(&day, &month, &year);
if is_valid {
    command_set::enable_command(CM_OK);
} else {
    command_set::disable_command(CM_OK);
}
broadcast_changes();

// NOW enter the loop
loop {
    // Draw, poll events, etc.
}
```

This ensures that empty fields start with a disabled button. Only when you fill them in does it enable.

Seems obvious in retrospect. But it's easy to miss.

---

## Date Validation: The Devil's in the Details

Let's talk about validating dates properly. It's trickier than you think.

**The easy part**: Check if day is 1-31, month is 1-12, year is reasonable.

**The tricky part**: February 31st is invalid. But so is February 29th... *sometimes*.

```rust
fn days_in_month(month: u32, year: i32) -> u32 {
    match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 => if is_leap_year(year) { 29 } else { 28 },
        _ => 0,
    }
}

fn is_leap_year(year: i32) -> bool {
    (year % 4 == 0 && year % 100 != 0) || (year % 400 == 0)
}
```

So February 29, 2000 is valid (2000 is a leap year).
But February 29, 1900 is *invalid* (1900 is NOT a leap year, even though it's divisible by 4).

Why? Because years divisible by 100 are NOT leap years... *unless* they're also divisible by 400.

This is why 2000 was a leap year but 1900 wasn't. And why validation can't just be "if year % 4 == 0."

**And one more thing**: We also check if the date is in the future:

```rust
// Future dates are invalid (can't calculate biorhythm for unborn people)
calculate_days_alive(year, month, day).is_some()
```

If `calculate_days_alive()` returns `None`, the date is in the future. Reject it.

---

## Putting It All Together: The Complete Flow

Let's walk through what happens when you run the app:

1. **App starts**
   - Menu bar and status line created
   - Window positions calculated (with shadow offsets)
   - Chart window is *not* created yet

2. **Dialog appears**
   - Centered, accounting for shadow
   - Fields are empty
   - Initial validation runs: all fields empty → OK button disabled

3. **User types in day field: "1"**
   - Event: Keyboard
   - Dialog handles it, updates the InputLine
   - Validation runs: day=1, month=empty, year=empty → still invalid
   - OK button stays disabled

4. **User types in month field: "2"**
   - Event: Keyboard
   - Validation runs: day=1, month=2, year=empty → still invalid
   - OK button stays disabled

5. **User types in year field: "2", "0", "0", "0"**
   - Four keyboard events
   - After "2": day=1, month=2, year=2 → invalid (year < 1900)
   - After "20": day=1, month=2, year=20 → invalid (year < 1900)
   - After "200": day=1, month=2, year=200 → invalid (year < 1900)
   - After "2000": day=1, month=2, year=2000 → **VALID!**
   - Command set enables CM_OK
   - Broadcast sent
   - OK button lights up

6. **User presses Enter**
   - Event: Keyboard (KB_ENTER)
   - First `handle_event()`: Dialog converts KB_ENTER → CM_OK
   - Second `handle_event()`: Dialog processes CM_OK, calls `end_modal(CM_OK)`
   - Dialog closes, loop exits with `result = CM_OK`

7. **Back in main()**
   - Check result: CM_OK, not CM_CANCEL
   - Parse date fields: 1/2/2000
   - Calculate days alive
   - Store in `biorhythm_data`
   - **NOW** create the chart window
   - Add it to desktop
   - Enter main event loop

8. **Chart appears**
   - Centered, accounting for shadow and menu bar
   - Shows three colored sine waves (physical, emotional, intellectual)
   - User can press Alt+C to recalculate

Every detail matters. Every piece builds on the others.

---

## The Patterns We Discovered

Let me summarize the key patterns we've learned:

### ✅ DO: Use Command Set for Button States

```rust
// Enable/disable commands, not buttons
command_set::enable_command(CM_OK);
if command_set::command_set_changed() {
    broadcast(CM_COMMAND_SET_CHANGED);
}
```

**Why**: Scales, matches Turbo Vision architecture, separates concerns.

### ❌ DON'T: Manipulate Buttons Directly

```rust
// Fragile, doesn't scale
let button = dialog.child_at_mut(8).downcast_mut::<Button>();
button.set_disabled(true);
```

### ✅ DO: Use OF_CENTERED for Simple Centering

```rust
use turbo_vision::core::state::OF_CENTERED;

let mut dialog = Dialog::new(Rect::new(0, 0, width, height), "Title");
dialog.set_options(OF_CENTERED);  // Automatically centered on desktop
```

**Why**: Handles shadows automatically. Less error-prone. Cleaner code.

### ✅ DO: Account for Shadows When Manually Positioning

For special cases (relative positioning, menu bar offsets):

```rust
let x = (screen_width - (window_width + 2)) / 2;  // +2 for shadow
let y = (screen_height - (window_height + 1)) / 2; // +1 for shadow
```

**Why**: Shadows are part of the visual. Ignoring them makes windows look off-center.

### ❌ DON'T: Ignore Shadows in Manual Positioning

```rust
let x = (screen_width - window_width) / 2;  // Looks off-center!
```

### ✅ DO: Reprocess Events After Conversion

```rust
dialog.handle_event(&mut event);
if event.what == EventType::Command {
    dialog.handle_event(&mut event);  // Process converted command
}
```

**Why**: KB_ENTER → CM_OK conversion happens in first call, but needs second call to process.

### ❌ DON'T: Assume One Call Is Enough

```rust
dialog.handle_event(&mut event);
// Enter key won't work!
```

### ✅ DO: Validate Initial State

```rust
// Before event loop
let is_valid = validate(...);
update_command_state(is_valid);
broadcast_changes();

loop { ... }
```

**Why**: Ensures correct button state at dialog open.

### ❌ DON'T: Only Validate After Events

```rust
loop {
    if let Some(event) = poll() {
        validate();  // Initial state never validated!
    }
}
```

### ✅ DO: Create Windows After Validation

```rust
// Show dialog first
let result = dialog.execute(...);
if result == CM_CANCEL {
    return Ok(());  // Clean exit
}

// Create window only if user clicked OK
let window = Window::new(...);
```

**Why**: No visual artifacts, clean exit path.

### ❌ DON'T: Create Windows Too Early

```rust
let window = Window::new(...);
app.desktop.add(window);  // Visible behind dialog!

let result = dialog.execute(...);
// If canceled, window is still there
```

---

## Testing It Out: Manual Test Cases

Here are the scenarios I used to verify everything works:

**Test 1: Empty Fields**
- Open app → OK button should be disabled
- Try to click OK → Nothing happens
- ✅ Button stays disabled, can't submit

**Test 2: Invalid Month**
- Enter: day=15, month=15, year=2000
- ✅ OK button disabled (month > 12)

**Test 3: February 31st**
- Enter: day=31, month=2, year=2000
- ✅ OK button disabled (Feb only has 29 days in leap years)

**Test 4: Leap Year**
- Enter: day=29, month=2, year=2000
- ✅ OK button enabled (2000 is a leap year)
- Change year to 1900
- ✅ OK button disabled (1900 is NOT a leap year)

**Test 5: Future Date**
- Enter: day=1, month=1, year=2100
- ✅ OK button disabled (date in future)

**Test 6: Valid Date**
- Enter: day=15, month=6, year=1990
- ✅ OK button enables
- Click OK
- ✅ Chart window appears centered

**Test 7: Cancel at Startup**
- Start app
- Press Escape
- ✅ App exits immediately, no orphaned windows

**Test 8: Enter Key**
- Enter valid date
- Focus on year field
- Press Enter
- ✅ Same as clicking OK (dialog closes, chart appears)

**Test 9: Centering**
- Resize terminal to various sizes
- ✅ All windows remain centered
- ✅ Shadows don't cause off-center appearance

**Test 10: Recalculate**
- Enter date, click OK (chart appears)
- Press Alt+C
- ✅ Dialog shows with previous date prefilled
- Change date, click OK
- ✅ Chart updates

---

## Beyond the Basics: What You Could Add

This example shows the core patterns, but there's room to grow:

### Error Messages

Show *why* validation failed:

```rust
enum ValidationError {
    EmptyFields,
    InvalidRange,
    ImpossibleDate,  // Feb 31
    FutureDate,
}

fn validate_detailed(...) -> Result<(), ValidationError> {
    // Return specific error
}

// Then show a message:
match validate_detailed(...) {
    Err(ValidationError::ImpossibleDate) =>
        show_error("That date doesn't exist (check the month/year)"),
    // etc.
}
```

### Async Validation

For validation requiring I/O (database lookups, API calls):

```rust
// Spawn validation task
let handle = spawn_validation_task(day, month, year);

// In event loop
if let Some(result) = handle.try_get_result() {
    update_command_state(result.is_valid);
}
```

### Progressive Validation

Only validate *completed* fields:

```rust
// If year field isn't 4 digits yet, don't validate it
if year_str.len() < 4 {
    // Don't check year validity yet
}
```

### Visual Feedback

Highlight invalid fields in red:

```rust
if !is_day_valid() {
    day_input.set_color(Attr::new(Red, White));
}
```

---

## What We Learned

Building this biorhythm calculator taught me things I wouldn't have learned from just reading documentation:

1. **Use OF_CENTERED for dialogs** - Turbo Vision's `OF_CENTERED` option handles shadow positioning automatically. For special cases (menu bar offsets, relative positioning), calculate manually and account for shadows.

2. **Command set over direct manipulation** - Global command state is more robust than finding and toggling individual buttons.

3. **Custom event loops unlock features** - Real-time validation requires stepping outside standard `execute()`.

4. **Event reprocessing is necessary** - When events get converted (KB_ENTER → CM_OK), you need to process them twice.

5. **Initial state validation isn't automatic** - Don't forget to validate *before* entering the event loop.

6. **Window creation timing matters** - Create windows after validation, not before. Gives you clean exit paths.

7. **Details make the difference** - Leap years, empty string checks, proper centering—these are what make it feel polished.

These aren't things you'd guess from the API. They're things you learn by *building*.

---

## The Takeaway

The Biorhythm Calculator isn't just a demo of calculating sine waves. It's a demonstration of how to build a *professional-feeling* TUI application—one that responds to you, validates properly, and doesn't feel janky.

The patterns here—command set integration, custom event loops, proper centering, real-time validation—apply to any Turbo Vision application. You might be building a database front-end, a system monitor, a text editor. These patterns still apply.

And the philosophy applies too: **Sweat the details.** The automatic centering. The initial validation. The Enter key. None of these are individually important, but together they're the difference between "it works" and "it feels right."

Build something that feels right.

---

## Quick Reference

For when you need to remember the key patterns:

**Custom event loop template:**
```rust
dialog.set_state(dialog.state() | SF_MODAL);
validate_and_update_initial_state();

loop {
    draw_everything();

    if let Some(mut event) = poll_event() {
        dialog.handle_event(&mut event);
        if event.what == EventType::Command {
            dialog.handle_event(&mut event);  // Reprocess
        }
        validate_and_update_state();
    }

    if dialog.get_end_state() != 0 { break; }
}

restore_state();
```

**Centering (automatic):**
```rust
use turbo_vision::core::state::OF_CENTERED;
dialog.set_options(OF_CENTERED);  // Handles shadows automatically
```

**Centering (manual, with shadows):**
```rust
let x = (screen_width - (width + 2)) / 2;
let y = 1 + ((screen_height - 2) - (height + 1)) / 2;
```

**Command set pattern:**
```rust
if is_valid {
    command_set::enable_command(CM_OK);
} else {
    command_set::disable_command(CM_OK);
}

if command_set::command_set_changed() {
    let mut event = Event::broadcast(CM_COMMAND_SET_CHANGED);
    dialog.handle_event(&mut event);
    command_set::clear_command_set_changed();
}
```

**Date validation:**
```rust
fn validate_birth_date(day: &str, month: &str, year: &str) -> bool {
    // 1. Check empty
    if day.trim().is_empty() { return false; }

    // 2. Parse
    let (Ok(d), Ok(m), Ok(y)) = (...) else { return false; };

    // 3. Range check
    if d < 1 || d > 31 || m < 1 || m > 12 { return false; }

    // 4. Month length
    if d > days_in_month(m, y) { return false; }

    // 5. Not future
    calculate_days_alive(y, m, d).is_some()
}
```

Now go build something.
