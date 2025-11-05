# Biorhythm Calculator Tutorial

A comprehensive guide to the Biorhythm Calculator example application, demonstrating advanced Turbo Vision patterns including real-time validation, command set integration, and proper dialog handling.

## Table of Contents

1. [Overview](#overview)
2. [Application Flow](#application-flow)
3. [Key Features](#key-features)
4. [Architecture Patterns](#architecture-patterns)
5. [Real-Time Validation System](#real-time-validation-system)
6. [Command Set Integration](#command-set-integration)
7. [Window Centering with Shadows](#window-centering-with-shadows)
8. [Custom Event Loops](#custom-event-loops)
9. [Code Walkthrough](#code-walkthrough)
10. [Common Patterns and Gotchas](#common-patterns-and-gotchas)

---

## Overview

The Biorhythm Calculator is a complete Turbo Vision application that demonstrates:

- **Real-time input validation** with dynamic button enabling/disabling
- **Command set integration** for managing button states
- **Custom modal dialog event loops** for specialized behavior
- **Proper window centering** accounting for shadows and UI chrome
- **Data persistence** across dialog invocations
- **Date validation** including leap years and month lengths

### What is a Biorhythm?

Biorhythms are theoretical cycles that supposedly influence human behavior:
- **Physical cycle**: 23 days (strength, coordination)
- **Emotional cycle**: 28 days (mood, creativity)
- **Intellectual cycle**: 33 days (alertness, reasoning)

The app calculates your position in each cycle based on days lived since birth.

---

## Application Flow

### Startup Sequence

1. **Application initializes**
   - Creates menu bar and status line
   - Calculates window positions (accounting for shadows)
   - **Does NOT create chart window yet**

2. **Birthdate dialog appears**
   - Centered on screen
   - Fields are empty (not prefilled)
   - OK button is **disabled** (validation requires all fields)

3. **User enters birthdate**
   - Real-time validation on every keystroke
   - OK button enables when date is valid
   - Cancel button always enabled

4. **User choice**
   - **Cancel/Escape**: App exits immediately
   - **OK**: Chart window created and displayed

5. **Main application runs**
   - Chart shows biorhythm cycles
   - Alt+C reopens dialog to recalculate
   - Previous date is prefilled

### Why This Flow?

This pattern ensures:
- No visual artifacts (empty chart window behind dialog)
- Clean exit path if user cancels
- Centered windows appear after validation
- User can't submit invalid data

---

## Key Features

### 1. Real-Time Date Validation

The OK button is **automatically disabled** when:
- Any field is empty
- Day is not 1-31
- Month is not 1-12
- Year is not 1900-2100
- Date is impossible (Feb 31, Apr 31, etc.)
- Date is in the future
- Input contains non-numeric characters

**Example:**
```
Day:   31
Month: 2      ← Invalid! February only has 28/29 days
Year:  2000
[  OK  ]      ← Button is DISABLED (grayed out)
```

### 2. Date Prefill

After entering a valid date once:
- The dialog remembers it
- Alt+C shows the dialog with previous values prefilled
- Useful for recalculating or making small adjustments

### 3. Proper Centering

All windows and dialogs are centered accounting for:
- Menu bar (1 row at top)
- Status line (1 row at bottom)
- **Shadow** (2 columns width, 1 row height)

This ensures perfect visual balance.

### 4. Command Set Integration

Uses Turbo Vision's command enable/disable system:
- `CM_OK` enabled/disabled based on validation
- Buttons automatically update via `CM_COMMAND_SET_CHANGED` broadcasts
- Matches Borland's architecture exactly

---

## Architecture Patterns

### The Custom Event Loop Pattern

**Why needed?**

The standard `dialog.execute()` doesn't support custom validation that runs after every event. We need to:
1. Handle events normally
2. Re-validate after each event
3. Update command state
4. Broadcast changes to buttons

**Implementation:**

```rust
// Set modal flag
dialog.set_state(dialog.state() | SF_MODAL);

// Initial validation
let is_valid = validate_birth_date(&day, &month, &year);
if is_valid {
    command_set::enable_command(CM_OK);
} else {
    command_set::disable_command(CM_OK);
}

// Broadcast initial state
let mut event = Event::broadcast(CM_COMMAND_SET_CHANGED);
dialog.handle_event(&mut event);
command_set::clear_command_set_changed();

// Event loop
loop {
    app.desktop.draw(&mut app.terminal);
    dialog.draw(&mut app.terminal);
    dialog.update_cursor(&mut app.terminal);
    app.terminal.flush();

    if let Some(mut event) = app.terminal.poll_event(...) {
        // Handle the event
        dialog.handle_event(&mut event);

        // Re-process if converted to command (KB_ENTER → CM_OK)
        if event.what == EventType::Command {
            dialog.handle_event(&mut event);
        }

        // Re-validate after EVERY event
        let is_valid = validate_birth_date(&day, &month, &year);
        if is_valid {
            command_set::enable_command(CM_OK);
        } else {
            command_set::disable_command(CM_OK);
        }

        // Broadcast if command set changed
        if command_set::command_set_changed() {
            let mut broadcast = Event::broadcast(CM_COMMAND_SET_CHANGED);
            dialog.handle_event(&mut broadcast);
            command_set::clear_command_set_changed();
        }
    }

    // Check if dialog should close
    if dialog.get_end_state() != 0 {
        break;
    }
}
```

### Key Points

1. **Modal flag must be set manually** - tells Dialog it's in modal mode
2. **Initial validation runs before loop** - sets correct button state
3. **Event reprocessing** - when KB_ENTER converts to CM_OK, process again
4. **Validation after every event** - ensures button state is always correct
5. **Conditional broadcasting** - only broadcast if command set actually changed
6. **End state check** - Dialog sets this when OK/Cancel clicked

---

## Real-Time Validation System

### Three-Layer Validation

#### Layer 1: Input Validators (RangeValidator)

Attached to InputLine fields to filter characters during typing:

```rust
// Day: 1-31
let day_validator = Rc::new(RefCell::new(RangeValidator::new(1, 31)));
let mut day_input = InputLine::new(Rect::new(12, 4, 18, 5), 2, day_data);
day_input.set_validator(day_validator);
```

**What it does:**
- Allows only digits during typing
- Prevents input like "abc" or special characters
- Checks range during `is_valid()` call

**What it DOESN'T do:**
- Doesn't disable buttons automatically
- Doesn't check complete date validity (Feb 31)
- Doesn't check if date is in future

#### Layer 2: Complete Date Validation

The `validate_birth_date()` function checks everything:

```rust
fn validate_birth_date(day_str: &str, month_str: &str, year_str: &str) -> bool {
    // 1. Empty field check
    if day_str.trim().is_empty() || month_str.trim().is_empty()
       || year_str.trim().is_empty() {
        return false;
    }

    // 2. Parse to numbers
    let (Ok(day), Ok(month), Ok(year)) = (...) else {
        return false;
    };

    // 3. Basic range check
    if day < 1 || day > 31 || month < 1 || month > 12
       || year < 1900 || year > 2100 {
        return false;
    }

    // 4. Month length check (handles leap years)
    if day > days_in_month(month, year) {
        return false;
    }

    // 5. Not in future check
    calculate_days_alive(year, month, day).is_some()
}
```

**Why separate from RangeValidator?**
- RangeValidator only validates individual fields
- We need cross-field validation (day valid for that month/year)
- We need semantic validation (not in future)

#### Layer 3: Command Set Updates

After validation, update global command state:

```rust
if is_valid {
    command_set::enable_command(CM_OK);
} else {
    command_set::disable_command(CM_OK);
}

// Broadcast to all buttons
if command_set::command_set_changed() {
    let mut event = Event::broadcast(CM_COMMAND_SET_CHANGED);
    dialog.handle_event(&mut event);
    command_set::clear_command_set_changed();
}
```

---

## Command Set Integration

### The Turbo Vision Command Pattern

In Borland Turbo Vision, commands are globally enabled/disabled:

```
Global Command Set (bitfield)
  ↓
  ├─ CM_OK: enabled/disabled
  ├─ CM_CANCEL: always enabled
  ├─ CM_SAVE: enabled/disabled
  └─ ...

When command state changes:
  1. Flag is set: command_set_changed = true
  2. Broadcast CM_COMMAND_SET_CHANGED
  3. All buttons check their command
  4. Buttons update disabled state
```

### Why Use Command Set Instead of Direct Button Manipulation?

**Wrong approach (brittle):**
```rust
// ❌ BAD: Direct button manipulation
let button = dialog.child_at_mut(8).downcast_mut::<Button>();
button.set_disabled(!is_valid);
```

**Problems:**
- Hard-coded child index (fragile)
- Requires downcasting (complex)
- Doesn't scale to multiple buttons
- Not the Turbo Vision way

**Correct approach (robust):**
```rust
// ✅ GOOD: Command set
if is_valid {
    command_set::enable_command(CM_OK);
} else {
    command_set::disable_command(CM_OK);
}

// Broadcast to ALL views
if command_set::command_set_changed() {
    let mut event = Event::broadcast(CM_COMMAND_SET_CHANGED);
    dialog.handle_event(&mut event);
    command_set::clear_command_set_changed();
}
```

**Benefits:**
- Works for all buttons with same command
- No child indices needed
- Automatic propagation to all views
- Matches Borland architecture
- Easy to extend

### How Buttons Respond to Command Changes

In `Button::handle_event()`:

```rust
if event.what == EventType::Broadcast {
    if event.command == CM_COMMAND_SET_CHANGED {
        // Query global command set
        let should_be_enabled = command_set::command_enabled(self.command);
        let is_currently_disabled = self.is_disabled();

        // Update if changed
        if should_be_enabled && is_currently_disabled {
            self.set_disabled(false);
        } else if !should_be_enabled && !is_currently_disabled {
            self.set_disabled(true);
        }
    }
}
```

This happens automatically for **every button** when broadcast is sent.

---

## Window Centering with Shadows

### The Shadow Problem

Windows in Turbo Vision have shadows that extend beyond their bounds:
- **Shadow width**: 2 columns (right edge)
- **Shadow height**: 1 row (bottom edge)

```
Window bounds: 76×21
Visual size with shadow: 78×22
```

### Naive Centering (WRONG)

```rust
// ❌ This centers the window bounds, not the visual appearance
let x = (screen_width - window_width) / 2;
let y = (screen_height - window_height) / 2;
```

**Result:** Window appears off-center to the right and down.

### Correct Centering

```rust
// ✅ Account for shadow in calculation
let window_width = 76i16;
let window_height = 21i16;

// Horizontal: center the visual width (window + shadow)
let window_x = (screen_width as i16 - (window_width + 2)) / 2;

// Vertical: account for menu bar and shadow
let available_height = screen_height as i16 - 2;  // minus menu + status
let window_y = 1 + (available_height - (window_height + 1)) / 2;
//             ↑                                      ↑
//          menu bar offset                      shadow height
```

### Dialog Centering

Dialogs also have shadows:

```rust
let dialog_width = 50i16;
let dialog_height = 12i16;

// Center including shadow
let dialog_x = (screen_width as i16 - (dialog_width + 2)) / 2;
let dialog_y = (screen_height as i16 - (dialog_height + 1)) / 2;
```

**Note:** Dialogs don't need menu bar offset since they float above everything.

---

## Custom Event Loops

### Why Custom Loops?

Standard `dialog.execute()` is fine for simple dialogs, but our needs:

1. **Validate after every keystroke** - standard execute doesn't do this
2. **Update button states dynamically** - need to call command_set functions
3. **Broadcast changes** - need to send CM_COMMAND_SET_CHANGED
4. **Handle Enter key properly** - need event reprocessing

### The Dialog Event Loop Pattern

```rust
// 1. Setup
dialog.set_state(dialog.state() | SF_MODAL);
let mut result = CM_CANCEL;

// 2. Initial state
validate_and_update_commands();
broadcast_command_changes();

// 3. Event loop
loop {
    // Draw
    app.desktop.draw(&mut app.terminal);
    dialog.draw(&mut app.terminal);
    dialog.update_cursor(&mut app.terminal);
    app.terminal.flush();

    // Poll event
    if let Some(mut event) = app.terminal.poll_event(timeout) {
        // Handle event
        dialog.handle_event(&mut event);

        // Reprocess if converted to command
        if event.what == EventType::Command {
            dialog.handle_event(&mut event);
        }

        // Post-event validation
        validate_and_update_commands();
        broadcast_command_changes();
    }

    // Check for exit
    let end_state = dialog.get_end_state();
    if end_state != 0 {
        result = end_state;
        break;
    }
}

// 4. Cleanup
dialog.set_state(old_state);
command_set::enable_command(CM_OK);  // restore for other uses
```

### Event Reprocessing

**Why needed?**

When user presses Enter in an InputLine:
1. Dialog converts KB_ENTER to CM_OK (default button command)
2. Event is modified in place
3. BUT: Dialog only calls `handle_event()` once per iteration
4. The converted command never gets processed!

**Solution:**

```rust
dialog.handle_event(&mut event);

// If event was converted to command, process again
if event.what == EventType::Command {
    dialog.handle_event(&mut event);
}
```

This matches Borland's behavior where `putEvent()` re-queues converted events.

---

## Code Walkthrough

### Main Function Structure

```rust
fn main() -> std::io::Result<()> {
    // 1. Initialize application
    let mut app = Application::new()?;
    let (width, height) = app.terminal.size();

    // 2. Create shared data (NO initial biorhythm data)
    let biorhythm_data = Arc::new(Mutex::new(None));
    let mut prev_day = String::from("");
    let mut prev_month = String::from("");
    let mut prev_year = String::from("");

    // 3. Setup menu bar and status line
    // ...

    // 4. Calculate window positions (but don't create window yet!)
    let window_width = 76i16;
    let window_height = 21i16;
    let window_x = (width as i16 - (window_width + 2)) / 2;
    let window_y = 1 + ((height as i16 - 2) - (window_height + 1)) / 2;

    // 5. Show birthdate dialog with custom event loop
    let (mut dialog, day_data, month_data, year_data) =
        create_biorhythm_dialog(&prev_day, &prev_month, &prev_year, width, height);

    // ... custom event loop ...

    // 6. Check result
    if result == CM_CANCEL {
        return Ok(());  // Exit if canceled
    }

    // 7. Parse date and create biorhythm data
    if result == CM_OK {
        // ... parse and validate ...
        *biorhythm_data.lock().unwrap() = Some(Biorhythm::new(days_alive));
        prev_day = day_str;  // Save for next time
        prev_month = month_str;
        prev_year = year_str;
    }

    // 8. NOW create and show the chart window
    let mut main_window = Window::new(
        Rect::new(window_x, window_y, window_x + window_width, window_y + window_height),
        "Biorhythm Calculator"
    );
    let chart = BiorhythmChart::new(Rect::new(1, 1, 74, 19), biorhythm_data);
    main_window.add(Box::new(chart));
    app.desktop.add(Box::new(main_window));

    // 9. Main event loop
    app.running = true;
    while app.running {
        // ... handle events ...
        // Alt+C triggers CM_BIORHYTHM which shows dialog again
    }

    Ok(())
}
```

### Date Validation Implementation

```rust
fn validate_birth_date(day_str: &str, month_str: &str, year_str: &str) -> bool {
    // Empty check
    if day_str.trim().is_empty() || month_str.trim().is_empty()
       || year_str.trim().is_empty() {
        return false;
    }

    // Parse check
    let (Ok(day), Ok(month), Ok(year)) = (
        day_str.parse::<u32>(),
        month_str.parse::<u32>(),
        year_str.parse::<i32>(),
    ) else {
        return false;
    };

    // Range check
    if day < 1 || day > 31 || month < 1 || month > 12
       || year < 1900 || year > 2100 {
        return false;
    }

    // Month length check (accounts for leap years)
    if day > days_in_month(month, year) {
        return false;
    }

    // Future date check
    calculate_days_alive(year, month, day).is_some()
}

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

### Chart Drawing

```rust
impl View for BiorhythmChart {
    fn draw(&mut self, terminal: &mut Terminal) {
        // Get biorhythm data (may be None at startup)
        let biorhythm_lock = self.biorhythm.lock().unwrap();
        let biorhythm = match biorhythm_lock.as_ref() {
            Some(b) => b,
            None => {
                // No data yet - draw empty chart or message
                return;
            }
        };

        // Draw axis
        let mid_y = height / 2;
        let mut axis_buf = DrawBuffer::new(width);
        axis_buf.move_str(0, "─".repeat(width), axis_attr);
        write_line_to_terminal(terminal, x, y + mid_y, &axis_buf);

        // Draw each cycle
        let cycles = [
            ('■', Attr::new(Red, LightGray), 0.9, physical),
            ('■', Attr::new(Green, LightGray), 1.0, emotional),
            ('■', Attr::new(Blue, LightGray), 0.8, intellectual),
        ];

        for day_offset in -30..31 {
            let x_pos = (day_offset + 30) as usize;

            for (char, attr, scale, cycle_fn) in &cycles {
                let value = cycle_fn(biorhythm, day_offset);
                let y_offset = (-value * mid_y as f64 * scale) as i16;
                let y_pos = mid_y + y_offset;

                if y_pos >= 0 && y_pos < height {
                    let mut buf = DrawBuffer::new(1);
                    buf.put_char(0, *char, *attr);
                    write_line_to_terminal(terminal, x + x_pos, y + y_pos, &buf);
                }
            }
        }
    }
}
```

---

## Common Patterns and Gotchas

### ✅ DO: Use Command Set for Button States

```rust
// Enable/disable commands, not buttons directly
command_set::enable_command(CM_OK);
command_set::disable_command(CM_SAVE);

// Broadcast changes
if command_set::command_set_changed() {
    let mut event = Event::broadcast(CM_COMMAND_SET_CHANGED);
    dialog.handle_event(&mut event);
    command_set::clear_command_set_changed();
}
```

### ❌ DON'T: Manipulate Buttons Directly

```rust
// Don't do this - fragile and doesn't scale
let button = dialog.child_at_mut(8).downcast_mut::<Button>();
button.set_disabled(true);
```

### ✅ DO: Account for Shadows When Centering

```rust
let x = (screen_width - (window_width + 2)) / 2;  // +2 for shadow
let y = (screen_height - (window_height + 1)) / 2;  // +1 for shadow
```

### ❌ DON'T: Forget Shadow in Calculations

```rust
let x = (screen_width - window_width) / 2;  // Wrong! Off by 1 column
```

### ✅ DO: Reprocess Events After Conversion

```rust
dialog.handle_event(&mut event);

if event.what == EventType::Command {
    dialog.handle_event(&mut event);  // Process the converted command
}
```

### ❌ DON'T: Assume One handle_event() is Enough

```rust
dialog.handle_event(&mut event);
// Command conversion lost! Enter key won't work.
```

### ✅ DO: Validate Empty Strings

```rust
if day_str.trim().is_empty() {
    return false;
}
```

### ❌ DON'T: Assume Parse Will Catch Empty

```rust
// "".parse::<u32>() returns Err, but should fail earlier
let Ok(day) = day_str.parse::<u32>() else { return false };
```

### ✅ DO: Set Initial Button State Before Loop

```rust
// Initial validation
let is_valid = validate_birth_date(...);
if is_valid {
    command_set::enable_command(CM_OK);
} else {
    command_set::disable_command(CM_OK);
}
broadcast_changes();

// Then start loop
loop { ... }
```

### ❌ DON'T: Assume Default Button State is Correct

```rust
// Button might be enabled when it should be disabled initially
loop { ... }  // Wrong! No initial validation
```

### ✅ DO: Restore Command State After Modal Loop

```rust
// Before exiting modal loop
command_set::enable_command(CM_OK);  // Other dialogs might need it
```

### ❌ DON'T: Leave Commands Disabled

```rust
// CM_OK stays disabled, breaking other dialogs
return Ok(());
```

---

## Testing the Application

### Manual Test Cases

1. **Empty fields test**
   - Start app
   - Expected: OK button disabled
   - Action: Leave fields empty, try to click OK
   - Result: Button stays disabled, can't submit

2. **Invalid month test**
   - Enter day: 15
   - Enter month: 15
   - Enter year: 2000
   - Expected: OK button disabled (month > 12)

3. **Invalid day for month test**
   - Enter day: 31
   - Enter month: 2
   - Enter year: 2000
   - Expected: OK button disabled (Feb 31 doesn't exist)

4. **Leap year test**
   - Enter day: 29
   - Enter month: 2
   - Enter year: 2000 (leap year)
   - Expected: OK button **enabled** (Feb 29, 2000 is valid)

   - Change year to 1900 (not a leap year)
   - Expected: OK button **disabled** (Feb 29, 1900 is invalid)

5. **Future date test**
   - Enter day: 1
   - Enter month: 1
   - Enter year: 2100
   - Expected: OK button disabled (date in future)

6. **Valid date test**
   - Enter day: 15
   - Enter month: 6
   - Enter year: 1990
   - Expected: OK button **enabled**
   - Click OK
   - Result: Chart window appears centered

7. **Cancel test**
   - Start app
   - Press Escape or click Cancel
   - Expected: App exits immediately

8. **Recalculate test**
   - Enter valid date, click OK
   - Press Alt+C
   - Expected: Dialog shows with previous date prefilled
   - Change date, click OK
   - Result: Chart updates

9. **Centering test**
   - Resize terminal to different sizes
   - Expected: All windows remain centered
   - Shadows should not cause off-center appearance

10. **Enter key test**
    - Enter valid date
    - Focus on year field
    - Press Enter
    - Expected: Same as clicking OK button (dialog closes, chart appears)

---

## Advanced Topics

### Extending the Validation

Add custom validation rules:

```rust
fn validate_birth_date(day_str: &str, month_str: &str, year_str: &str) -> bool {
    // ... existing validation ...

    // Custom: Don't allow dates before 1900
    if year < 1900 {
        return false;
    }

    // Custom: Don't allow unrealistic ages (>150 years)
    if let Some(days) = calculate_days_alive(year, month, day) {
        if days > 150 * 365 {
            return false;
        }
    }

    true
}
```

### Multiple Validation States

Track why validation failed:

```rust
enum ValidationError {
    EmptyFields,
    InvalidRange,
    InvalidDate,
    FutureDate,
}

fn validate_birth_date_detailed(...) -> Result<(), ValidationError> {
    if day_str.trim().is_empty() || ... {
        return Err(ValidationError::EmptyFields);
    }

    let (Ok(day), Ok(month), Ok(year)) = (...) else {
        return Err(ValidationError::InvalidRange);
    };

    if day > days_in_month(month, year) {
        return Err(ValidationError::InvalidDate);
    }

    // ... etc
}
```

Then show specific error messages to user.

### Async Validation

For validation that requires I/O (database lookups, API calls):

```rust
// Launch validation in background
let validation_result = Arc::new(Mutex::new(None));

// In event loop
if let Ok(mut guard) = validation_result.try_lock() {
    if let Some(result) = guard.take() {
        // Update button based on async result
        if result {
            command_set::enable_command(CM_OK);
        } else {
            command_set::disable_command(CM_OK);
        }
        broadcast_changes();
    }
}
```

---

## Summary

The Biorhythm Calculator demonstrates:

1. **Real-time validation** - Validate on every keystroke, disable buttons dynamically
2. **Command set pattern** - Use global command enable/disable, not direct button manipulation
3. **Custom event loops** - For specialized dialog behavior beyond standard execute()
4. **Proper centering** - Account for shadows and UI chrome (menu/status)
5. **Event reprocessing** - Handle command conversion (KB_ENTER → CM_OK)
6. **Data persistence** - Remember user input across dialog invocations
7. **Clean startup flow** - Dialog first, exit on cancel, create windows only after validation

These patterns apply to any Turbo Vision application requiring:
- Form validation
- Dynamic UI updates
- Proper visual layout
- Professional user experience

Study this example to understand how to build robust, user-friendly TUI applications!
