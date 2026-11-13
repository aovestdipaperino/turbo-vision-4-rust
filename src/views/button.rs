// (C) 2025 - Enzo Lombardi

//! Button view - clickable button with keyboard shortcuts and command dispatch.

use super::view::{View, ViewCore, write_line_to_terminal};
use crate::core::command::CommandId;
use crate::core::draw::DrawBuffer;
use crate::core::event::{Event, EventType, KB_ENTER, MB_LEFT_BUTTON};
use crate::core::geometry::Rect;
use crate::core::palette::{
    BUTTON_DEFAULT, BUTTON_DISABLED, BUTTON_NORMAL, BUTTON_SELECTED, BUTTON_SHADOW, BUTTON_SHORTCUT,
};
use crate::core::state::Options;
use crate::core::state::{SHADOW_BOTTOM, SHADOW_SOLID, SHADOW_TOP, State};
use crate::terminal::Terminal;
use std::cell::Cell;
use std::time::{Duration, Instant};

/// How long a button pressed from the keyboard is shown pushed in before its
/// command is sent. Matches magiblot's `TButton` (`animationDurationMs`).
pub const DEFAULT_PRESS_ANIMATION: Duration = Duration::from_millis(100);

thread_local! {
    static PRESS_ANIMATION: Cell<Duration> = const { Cell::new(DEFAULT_PRESS_ANIMATION) };
}

/// Set how long a button pressed with Enter, Space or its hotkey stays pushed
/// in before it sends its command. `Duration::ZERO` turns the animation off:
/// the command is then sent at once, from the key event itself.
///
/// Applies to every button on the calling thread.
pub fn set_press_animation(duration: Duration) {
    PRESS_ANIMATION.with(|d| d.set(duration));
}

/// How long a key-pressed button stays pushed in (see [`set_press_animation`]).
pub fn press_animation() -> Duration {
    PRESS_ANIMATION.with(Cell::get)
}

/// The width a button needs to show `title` (its `~` markers and surrounding
/// spaces left out) with a column of space on each side, plus its shadow;
/// never less than `min`. Use it to size buttons whose text is translated.
#[must_use]
pub fn width_for_title(title: &str, min: i16) -> i16 {
    let text: String = title.trim().chars().filter(|&c| c != '~').collect();
    let width = unicode_width::UnicodeWidthStr::width(text.as_str());
    min.max(i16::try_from(width).unwrap_or(i16::MAX - 4) + 4)
}

/// The bounds of a row of buttons on rows `y..y + 2` that ends at column
/// `right` (exclusive), each at least `min` wide and as wide as its title
/// needs, `gap` columns apart, in the order of `titles`.
pub(crate) fn row_from_right(titles: &[&str], right: i16, y: i16, min: i16, gap: i16) -> Vec<Rect> {
    let mut x = right;
    let mut rects: Vec<Rect> = titles
        .iter()
        .rev()
        .map(|title| {
            let width = width_for_title(title, min);
            let rect = Rect::new(x - width, y, x, y + 2);
            x -= width + gap;
            rect
        })
        .collect();
    rects.reverse();
    rects
}

pub struct Button {
    core: ViewCore,
    title: String,
    command: CommandId,
    is_default: bool,
    /// Whether this button is currently the *acting* default.
    ///
    /// Matches Borland's `amDefault` (tbutton.cc): a focused button grabs the
    /// default role (cmGrabDefault); when it loses focus the role reverts to
    /// the statically flagged default button (cmReleaseDefault).
    am_default: bool,
    /// Whether a `MouseDown` was armed inside this button (fires on `MouseUp`).
    pressed: bool,
    /// Whether the button is drawn pushed in: armed and the pointer is still
    /// over it. Dragging off pops it back out (Borland: `drawState(down)`
    /// while tracking the mouse in `TButton::handleEvent`).
    down: bool,
    /// Until when a key press shows the button pushed in. Its command is
    /// queued to arrive at that same instant, so the loop that delivers it
    /// redraws the button popped back out.
    key_down_until: Option<Instant>,
    is_broadcast: bool,
}

impl Button {
    pub fn new(bounds: Rect, title: &str, command: CommandId, is_default: bool) -> Self {
        use crate::core::command_set;

        // Check if command is initially enabled
        // Matches Borland: TButton constructor checks commandEnabled() (tbutton.cc:55-56)
        let mut state = State::empty();
        if !command_set::command_enabled(command) {
            state |= State::DISABLED;
        }

        Self {
            core: ViewCore {
                bounds,
                state,
                options: Options::POST_PROCESS, // Buttons process in post-process phase
                palette_chain: None,
                ..ViewCore::default()
            },
            title: title.to_string(),
            command,
            is_default,
            am_default: is_default,
            pressed: false,
            down: false,
            key_down_until: None,
            is_broadcast: false,
        }
    }

    /// Whether this button was created as the dialog's default button
    /// (Borland: `TButton::amDefault`).
    pub fn is_default(&self) -> bool {
        self.is_default
    }

    /// The command this button emits when pressed.
    pub fn command(&self) -> CommandId {
        self.command
    }

    /// Whether the button broadcasts its command to its siblings instead of
    /// emitting it as a command (see `set_broadcast`).
    pub fn is_broadcast(&self) -> bool {
        self.is_broadcast
    }

    pub fn set_disabled(&mut self, disabled: bool) {
        self.set_state_flag(State::DISABLED, disabled);
        // A press in progress ends with the button: it no longer handles
        // the MouseUp that would disarm it.
        if disabled {
            self.set_armed(false);
        }
    }

    pub fn is_disabled(&self) -> bool {
        self.get_state_flag(State::DISABLED)
    }

    /// Set whether this button broadcasts its command instead of sending it as a command event
    /// Matches Borland: bfBroadcast flag
    pub fn set_broadcast(&mut self, broadcast: bool) {
        self.is_broadcast = broadcast;
    }

    /// Set whether this button is selectable (can receive focus)
    /// Matches Borland: ofSelectable flag
    pub fn set_selectable(&mut self, selectable: bool) {
        if selectable {
            self.core.options |= Options::SELECTABLE;
        } else {
            self.core.options &= !Options::SELECTABLE;
        }
    }

    /// Extract the hotkey character from the button title
    /// Returns the uppercase character following the first '~', or None if no hotkey
    fn get_hotkey(&self) -> Option<char> {
        let mut chars = self.title.chars();
        while let Some(ch) = chars.next() {
            if ch == '~' {
                // Next character is the hotkey
                if let Some(hotkey) = chars.next() {
                    return Some(hotkey.to_uppercase().next().unwrap_or(hotkey));
                }
            }
        }
        None
    }

    /// Returns true if the mouse position is inside the clickable button area.
    ///
    /// Excludes the shadow row/column at the bottom/right of the bounds.
    fn mouse_in_button(&self, pos: crate::core::geometry::Point) -> bool {
        pos.x >= 0 && pos.x < self.extent().b.x && pos.y >= 0 && pos.y < self.extent().b.y - 1
    }

    /// Whether the button is currently drawn pushed in: held down with the
    /// mouse, or pressed from the keyboard less than
    /// [`press_animation`] ago.
    pub fn is_down(&self) -> bool {
        self.down || self.key_down_until.is_some_and(|t| Instant::now() < t)
    }

    /// Arm or disarm a mouse press. While armed the button is
    /// `State::DRAGGING`, so its `Group` keeps sending it `MouseMove` and
    /// `MouseUp` after the pointer leaves it.
    fn set_armed(&mut self, armed: bool) {
        self.pressed = armed;
        self.down = armed;
        self.set_state_flag(State::DRAGGING, armed);
    }

    /// Emit the button's command (or broadcast) in place of `event`.
    fn press(&self, event: &mut Event) {
        if self.is_broadcast {
            *event = Event::broadcast(self.command);
        } else {
            *event = Event::command(self.command);
        }
    }

    /// Press the button from the keyboard (Enter, Space, its hotkey, or a
    /// dialog's Enter for its default button).
    ///
    /// Shows the button pushed in for [`press_animation`] and queues its
    /// command to arrive afterwards, consuming `event`; with no animation the
    /// command replaces `event` at once. Matches magiblot's `TButton`, which
    /// draws itself down and presses when its animation timer expires.
    /// The queued command enters at the top of the event loop, as Borland's
    /// `putEvent` would; a broadcast button's broadcast therefore reaches the
    /// whole modal view or desktop rather than just its owner.
    pub(crate) fn press_from_key(&mut self, event: &mut Event) {
        let duration = press_animation();
        if duration.is_zero() {
            self.press(event);
            return;
        }
        let mut command = Event::nothing();
        self.press(&mut command);
        let until = Instant::now() + duration;
        crate::core::timed_event::post_at(command, until);
        self.key_down_until = Some(until);
        event.clear();
    }
}

impl View for Button {
    fn core(&self) -> &ViewCore {
        &self.core
    }

    fn core_mut(&mut self) -> &mut ViewCore {
        &mut self.core
    }

    fn draw(&mut self, terminal: &mut Terminal) {
        let width = self.core.bounds.width_clamped() as usize;
        let height = self.core.bounds.height_clamped() as usize;

        // Don't render buttons that are too small
        // Minimum width: 4 (at least 2 chars for content + 1 for right shadow + 1 for spacing)
        // Minimum height: 2 (at least 1 line for content + 1 for bottom shadow)
        if width < 4 || height < 2 {
            return;
        }

        let is_disabled = self.is_disabled();
        let is_focused = self.is_focused();

        // Button color indices (from CP_BUTTON palette):
        // 1: Normal text
        // 2: Default text
        // 3: Selected (focused) text
        // 4: Disabled text
        // 7: Shortcut text
        // 8: Shadow
        let button_attr = if is_disabled {
            self.map_color(BUTTON_DISABLED) // Disabled
        } else if is_focused {
            self.map_color(BUTTON_SELECTED) // Selected/focused
        } else if self.am_default {
            self.map_color(BUTTON_DEFAULT) // Acting default but not focused
        } else {
            self.map_color(BUTTON_NORMAL) // Normal
        };

        // Shadow attribute - Borland uses spaces where BG is visible, we use blocks where FG is visible
        // So we swap FG/BG: 0x70 (Black on LightGray) becomes 0x07 (LightGray on Black)
        let mut shadow_attr = self.map_color(BUTTON_SHADOW);

        // If shadow mapping failed, use a default shadow color.
        // With the QCell palette chain, this should not normally trigger.
        if shadow_attr.to_u8() == 0xCF {
            // ERROR_ATTR
            use crate::core::palette::Attr;
            // Default: White on Black (standard shadow)
            shadow_attr = Attr::from_u8(0x07);
        }

        // Unswapped, a space in the shadow attribute shows the owner's
        // background: what a pushed-in button leaves where its shadow was.
        let background_attr = shadow_attr;
        let shadow_attr = shadow_attr.swap();

        // Shortcut attributes
        let shortcut_attr = if is_disabled {
            self.map_color(BUTTON_DISABLED) // Disabled shortcut same as disabled text
        } else {
            self.map_color(BUTTON_SHORTCUT) // Shortcut color
        };

        // Pushed in, the face slides one column right onto its own shadow
        // and the shadow disappears, the way Borland's drawState(True) does.
        let down = self.is_down() && !is_disabled;
        let face_x = usize::from(down);

        // Draw all lines except the last (which is the bottom shadow)
        for y in 0..(height - 1) {
            let mut buf = DrawBuffer::new(width);

            // Fill entire line with button color
            buf.move_char(0, ' ', button_attr, width);

            if down {
                // The column the face moved away from shows the background
                buf.put_char(0, ' ', background_attr);
            } else {
                // Right edge gets shadow character and attribute (last column)
                let shadow_char = if y == 0 { SHADOW_TOP } else { SHADOW_SOLID };
                buf.put_char(width - 1, shadow_char, shadow_attr);
            }

            // Draw the label on the middle line
            if y == (height - 1) / 2 {
                // Calculate display length without tildes
                let display_len = self.title.chars().filter(|&c| c != '~').count();
                let content_width = width - 1; // Exclude right shadow column
                let start = (content_width.saturating_sub(display_len)) / 2;
                buf.move_str_with_shortcut(start + face_x, &self.title, button_attr, shortcut_attr);
            }

            write_line_to_terminal(terminal, 0, y as i16, &buf);
        }

        // Draw bottom shadow line (1 char shorter, offset 1 to the right);
        // pushed in, there is no shadow and the row shows the background.
        let mut bottom_buf = DrawBuffer::new(width - 1);
        if down {
            bottom_buf.move_char(0, ' ', background_attr, width - 1);
        } else {
            bottom_buf.move_char(0, SHADOW_BOTTOM, shadow_attr, width - 1);
        }
        write_line_to_terminal(terminal, 1, (height - 1) as i16, &bottom_buf);
    }

    fn handle_event(&mut self, event: &mut Event) {
        // Handle broadcasts FIRST, even if button is disabled
        //
        // IMPORTANT: This matches Borland's TButton::handleEvent() behavior:
        // - tbutton.cc:196 calls TView::handleEvent() first
        // - TView::handleEvent() (tview.cc:486) only checks sfDisabled for evMouseDown, NOT broadcasts
        // - tbutton.cc:235-263 processes evBroadcast in switch statement
        // - tbutton.cc:255-262 handles cmCommandSetChanged regardless of disabled state
        //
        // This is critical: disabled buttons MUST receive CM_COMMAND_SET_CHANGED broadcasts
        // so they can become enabled when their command becomes enabled in the global command set.
        if event.what == EventType::Broadcast {
            use crate::core::command::{
                CM_COMMAND_SET_CHANGED, CM_GRAB_DEFAULT, CM_RELEASE_DEFAULT,
            };
            use crate::core::command_set;

            // Default-role handoff (Borland: tbutton.cc cmGrabDefault/cmReleaseDefault):
            // another button grabbed the default role, or asked us to give it back.
            if event.command == CM_GRAB_DEFAULT {
                // A focused button grabbed the default role; only the focused
                // button keeps it.
                self.am_default = self.is_focused();
            } else if event.command == CM_RELEASE_DEFAULT {
                // Role reverts to the statically flagged default button.
                self.am_default = self.is_default;
            }

            if event.command == CM_COMMAND_SET_CHANGED {
                // Query global command set (thread-local static, like Borland)
                let should_be_enabled = command_set::command_enabled(self.command);
                let is_currently_disabled = self.is_disabled();

                // Update disabled state if it changed
                // Matches Borland: tbutton.cc:256-260
                if should_be_enabled && is_currently_disabled {
                    // Command was disabled, now enabled
                    self.set_disabled(false);
                } else if !should_be_enabled && !is_currently_disabled {
                    // Command was enabled, now disabled
                    self.set_disabled(true);
                }

                // Event is not cleared - other views may need it
                // Matches Borland: broadcasts are not cleared in the button handler
            }
            return; // Broadcasts don't fall through to regular event handling
        }

        // Disabled buttons don't handle any other events (mouse, keyboard)
        // Matches Borland: TView::handleEvent() checks sfDisabled for evMouseDown (tview.cc:486)
        // and TButton's switch cases for evMouseDown/evKeyDown won't execute if disabled
        if self.is_disabled() {
            return;
        }

        match event.what {
            EventType::Keyboard => {
                // Handle hotkey (works even without focus, matches Borland PostProcess):
                // Alt plus the letter, or the letter alone. Matches Borland:
                // TButton::handleEvent tests getAltCode(c) as well as the
                // typed character. Alt matters in a form: there the plain
                // letter is typed into the focused input line first.
                // The typed character, not the key code's low byte: past
                // Latin-1 that byte is unrelated to the character (`ł`,
                // U+0142, would read as `B`).
                if let Some(hotkey) = self.get_hotkey() {
                    let by_alt = crate::core::event::alt_code(hotkey) == Some(event.key_code);
                    let by_char = event
                        .typed_char()
                        .is_some_and(|c| c.to_uppercase().next().unwrap_or(c) == hotkey);
                    if by_alt || by_char {
                        self.press_from_key(event);
                        return;
                    }
                }

                // Handle Enter/Space only if focused
                if !self.is_focused() {
                    return;
                }
                if event.key_code == KB_ENTER || event.key_code == ' ' as u16 {
                    self.press_from_key(event);
                }
            }
            EventType::MouseDown => {
                // Arm the button on press; the command fires on MouseUp inside
                // the button. Matches Borland: TButton tracks the mouse and only
                // presses when the button is released inside (tbutton.cc), which
                // lets the user cancel by dragging off before releasing.
                if event.mouse.buttons & MB_LEFT_BUTTON != 0
                    && self.mouse_in_button(event.mouse.pos)
                {
                    self.set_armed(true);
                    event.clear();
                }
            }
            EventType::MouseMove => {
                // While armed, the button follows the pointer: pushed in over
                // it, popped out when dragged off (Borland tracks the mouse
                // with drawState(mouseInView) until the button is released).
                if self.pressed {
                    self.down = self.mouse_in_button(event.mouse.pos);
                    event.clear();
                }
            }
            EventType::MouseUp => {
                let armed = self.pressed;
                self.set_armed(false);
                if armed {
                    if self.mouse_in_button(event.mouse.pos) {
                        // Released inside while armed - fire command or broadcast
                        self.press(event);
                    } else {
                        // Released outside - cancel the press without firing
                        event.clear();
                    }
                }
            }
            _ => {}
        }
    }

    fn can_focus(&self) -> bool {
        !self.is_disabled()
    }

    fn set_focus(&mut self, focused: bool) {
        // Default View behavior: set/clear State::FOCUSED
        self.set_state_flag(State::FOCUSED, focused);

        // Default-role handoff (Borland: TButton::setState() sends
        // cmGrabDefault on focus gain and cmReleaseDefault on focus loss).
        // A focused button becomes the acting default; when it loses focus,
        // the role reverts to the statically flagged default button.
        self.am_default = if focused { true } else { self.is_default };

        // Losing focus also cancels any armed (but unreleased) mouse press.
        if !focused {
            self.set_armed(false);
        }
    }

    fn get_palette(&self) -> Option<crate::core::palette::Palette> {
        use crate::core::palette::{Palette, palettes};
        Some(Palette::from_slice(palettes::CP_BUTTON))
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}

/// Builder for creating buttons with a fluent API.
///
/// # Examples
///
/// ```
/// use turbo_vision::views::button::ButtonBuilder;
/// use turbo_vision::core::geometry::Rect;
/// use turbo_vision::core::command::CM_OK;
///
/// let button = ButtonBuilder::new()
///     .bounds(Rect::new(10, 5, 20, 7))
///     .title("OK")
///     .command(CM_OK)
///     .default(true)
///     .build();
/// ```
pub struct ButtonBuilder {
    bounds: Option<Rect>,
    title: Option<String>,
    command: Option<CommandId>,
    is_default: bool,
}

impl ButtonBuilder {
    /// Creates a new `ButtonBuilder` with default values.
    pub fn new() -> Self {
        Self {
            bounds: None,
            title: None,
            command: None,
            is_default: false,
        }
    }

    /// Sets the button bounds (required).
    #[must_use]
    pub fn bounds(mut self, bounds: Rect) -> Self {
        self.bounds = Some(bounds);
        self
    }

    /// Sets the button title text (required).
    #[must_use]
    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
        self
    }

    /// Sets the command ID to dispatch when clicked (required).
    #[must_use]
    pub fn command(mut self, command: CommandId) -> Self {
        self.command = Some(command);
        self
    }

    /// Sets whether this is the default button (optional, defaults to false).
    ///
    /// The default button is highlighted differently and can be activated
    /// by pressing Enter even when not focused.
    #[must_use]
    pub fn default(mut self, is_default: bool) -> Self {
        self.is_default = is_default;
        self
    }

    /// Builds the Button.
    ///
    /// # Panics
    ///
    /// Panics if required fields (bounds, title, command) are not set.
    pub fn build(self) -> Button {
        let bounds = self.bounds.expect("Button bounds must be set");
        let title = self.title.expect("Button title must be set");
        let command = self.command.expect("Button command must be set");

        Button::new(bounds, &title, command, self.is_default)
    }
}

impl Default for ButtonBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::command::CM_COMMAND_SET_CHANGED;
    use crate::core::command_set;
    use crate::core::geometry::Point;
    use crate::core::timed_event;

    #[test]
    fn test_button_creation_with_disabled_command() {
        // Test that button is created disabled when command is disabled
        const TEST_CMD: u16 = 500;
        command_set::disable_command(TEST_CMD);

        let button = Button::new(Rect::new(0, 0, 10, 2), "Test", TEST_CMD, false);

        assert!(
            button.is_disabled(),
            "Button should start disabled when command is disabled"
        );
    }

    #[test]
    fn test_button_creation_with_enabled_command() {
        // Test that button is created enabled when command is enabled
        const TEST_CMD: u16 = 501;
        command_set::enable_command(TEST_CMD);

        let button = Button::new(Rect::new(0, 0, 10, 2), "Test", TEST_CMD, false);

        assert!(
            !button.is_disabled(),
            "Button should start enabled when command is enabled"
        );
    }

    #[test]
    fn test_disabled_button_receives_broadcast_and_becomes_enabled() {
        // REGRESSION TEST: Disabled buttons must receive broadcasts to become enabled
        // This tests the fix for the bug where disabled buttons returned early
        // and never received CM_COMMAND_SET_CHANGED broadcasts

        const TEST_CMD: u16 = 502;

        // Start with command disabled
        command_set::disable_command(TEST_CMD);

        let mut button = Button::new(Rect::new(0, 0, 10, 2), "Test", TEST_CMD, false);

        // Verify button starts disabled
        assert!(button.is_disabled(), "Button should start disabled");

        // Enable the command in the global command set
        command_set::enable_command(TEST_CMD);

        // Send broadcast to button
        let mut event = Event::broadcast(CM_COMMAND_SET_CHANGED);
        button.handle_event(&mut event);

        // Verify button is now enabled
        assert!(
            !button.is_disabled(),
            "Button should be enabled after receiving broadcast"
        );
    }

    #[test]
    fn test_enabled_button_receives_broadcast_and_becomes_disabled() {
        // Test that enabled buttons can be disabled via broadcast

        const TEST_CMD: u16 = 503;

        // Start with command enabled
        command_set::enable_command(TEST_CMD);

        let mut button = Button::new(Rect::new(0, 0, 10, 2), "Test", TEST_CMD, false);

        // Verify button starts enabled
        assert!(!button.is_disabled(), "Button should start enabled");

        // Disable the command in the global command set
        command_set::disable_command(TEST_CMD);

        // Send broadcast to button
        let mut event = Event::broadcast(CM_COMMAND_SET_CHANGED);
        button.handle_event(&mut event);

        // Verify button is now disabled
        assert!(
            button.is_disabled(),
            "Button should be disabled after receiving broadcast"
        );
    }

    #[test]
    fn test_disabled_button_ignores_keyboard_events() {
        // Test that disabled buttons don't respond to keyboard input

        const TEST_CMD: u16 = 504;
        command_set::disable_command(TEST_CMD);

        let mut button = Button::new(Rect::new(0, 0, 10, 2), "Test", TEST_CMD, false);

        button.set_focus(true);

        // Try to activate with Enter key
        let mut event = Event::keyboard(crate::core::event::KB_ENTER);
        button.handle_event(&mut event);

        // Event should not be converted to command
        assert_ne!(
            event.what,
            EventType::Command,
            "Disabled button should not generate command"
        );
    }

    #[test]
    fn test_disabled_button_ignores_mouse_clicks() {
        // Test that disabled buttons don't respond to mouse clicks

        const TEST_CMD: u16 = 505;
        command_set::disable_command(TEST_CMD);

        let mut button = Button::new(Rect::new(0, 0, 10, 2), "Test", TEST_CMD, false);

        // Try to click the button
        let mut event = Event::mouse(
            EventType::MouseDown,
            Point::new(5, 1),
            crate::core::event::MB_LEFT_BUTTON,
            false,
        );
        button.handle_event(&mut event);

        // Event should not be converted to command
        assert_ne!(
            event.what,
            EventType::Command,
            "Disabled button should not generate command"
        );
    }

    #[test]
    fn test_broadcast_does_not_clear_event() {
        // Test that CM_COMMAND_SET_CHANGED broadcast is not cleared
        // (so it can propagate to other buttons)

        const TEST_CMD: u16 = 506;
        command_set::disable_command(TEST_CMD);

        let mut button = Button::new(Rect::new(0, 0, 10, 2), "Test", TEST_CMD, false);

        command_set::enable_command(TEST_CMD);

        let mut event = Event::broadcast(CM_COMMAND_SET_CHANGED);
        button.handle_event(&mut event);

        // Event should still be a broadcast (not cleared)
        assert_eq!(
            event.what,
            EventType::Broadcast,
            "Broadcast should not be cleared"
        );
        assert_eq!(
            event.command, CM_COMMAND_SET_CHANGED,
            "Broadcast command should remain"
        );
    }

    #[test]
    fn test_button_builder() {
        const TEST_CMD: u16 = 507;
        command_set::enable_command(TEST_CMD);

        let button = ButtonBuilder::new()
            .bounds(Rect::new(5, 10, 15, 12))
            .title("Test")
            .command(TEST_CMD)
            .default(true)
            .build();

        assert_eq!(button.bounds(), Rect::new(5, 10, 15, 12));
        assert!(button.is_default());
        assert_eq!(button.command(), TEST_CMD);
    }

    #[test]
    fn test_button_builder_default_is_false() {
        const TEST_CMD: u16 = 508;
        command_set::enable_command(TEST_CMD);

        let button = ButtonBuilder::new()
            .bounds(Rect::new(0, 0, 10, 2))
            .title("Test")
            .command(TEST_CMD)
            .build();

        assert!(!button.is_default());
    }

    #[test]
    #[should_panic(expected = "Button bounds must be set")]
    fn test_button_builder_panics_without_bounds() {
        const TEST_CMD: u16 = 509;
        ButtonBuilder::new().title("Test").command(TEST_CMD).build();
    }

    #[test]
    #[should_panic(expected = "Button title must be set")]
    fn test_button_builder_panics_without_title() {
        const TEST_CMD: u16 = 510;
        ButtonBuilder::new()
            .bounds(Rect::new(0, 0, 10, 2))
            .command(TEST_CMD)
            .build();
    }

    #[test]
    #[should_panic(expected = "Button command must be set")]
    fn test_button_builder_panics_without_command() {
        ButtonBuilder::new()
            .bounds(Rect::new(0, 0, 10, 2))
            .title("Test")
            .build();
    }

    #[test]
    fn test_button_fires_on_mouse_up_inside() {
        // Press-on-release: MouseDown only arms the button, MouseUp inside fires
        const TEST_CMD: u16 = 512;
        command_set::enable_command(TEST_CMD);

        let mut button = Button::new(Rect::new(0, 0, 10, 3), "Test", TEST_CMD, false);

        let mut down = Event::mouse(
            EventType::MouseDown,
            Point::new(5, 1),
            MB_LEFT_BUTTON,
            false,
        );
        button.handle_event(&mut down);
        assert_eq!(
            down.what,
            EventType::Nothing,
            "MouseDown must arm the button, not fire the command"
        );

        let mut up = Event::mouse(EventType::MouseUp, Point::new(5, 1), MB_LEFT_BUTTON, false);
        button.handle_event(&mut up);
        assert_eq!(up.what, EventType::Command, "MouseUp inside must fire");
        assert_eq!(up.command, TEST_CMD);
    }

    #[test]
    fn test_button_press_cancelled_by_release_outside() {
        // Dragging off the button before releasing cancels the press
        const TEST_CMD: u16 = 513;
        command_set::enable_command(TEST_CMD);

        let mut button = Button::new(Rect::new(0, 0, 10, 3), "Test", TEST_CMD, false);

        let mut down = Event::mouse(
            EventType::MouseDown,
            Point::new(5, 1),
            MB_LEFT_BUTTON,
            false,
        );
        button.handle_event(&mut down);

        // Release outside the button - cancels, no command
        let mut up = Event::mouse(EventType::MouseUp, Point::new(20, 5), MB_LEFT_BUTTON, false);
        button.handle_event(&mut up);
        assert_ne!(up.what, EventType::Command, "release outside must not fire");

        // A later MouseUp inside without a fresh press must not fire either
        let mut up2 = Event::mouse(EventType::MouseUp, Point::new(5, 1), MB_LEFT_BUTTON, false);
        button.handle_event(&mut up2);
        assert_ne!(
            up2.what,
            EventType::Command,
            "MouseUp without an armed press must not fire"
        );
    }

    fn mouse(what: EventType, x: i16, y: i16) -> Event {
        Event::mouse(what, Point::new(x, y), MB_LEFT_BUTTON, false)
    }

    #[test]
    fn a_held_button_is_down_until_released() {
        const TEST_CMD: u16 = 533;
        command_set::enable_command(TEST_CMD);
        let mut button = Button::new(Rect::new(0, 0, 10, 2), "Test", TEST_CMD, false);
        assert!(!button.is_down());

        button.handle_event(&mut mouse(EventType::MouseDown, 3, 0));
        assert!(button.is_down(), "pushed in while held");
        assert!(button.state().contains(State::DRAGGING), "keeps the mouse");

        let mut up = mouse(EventType::MouseUp, 3, 0);
        button.handle_event(&mut up);
        assert_eq!((up.what, up.command), (EventType::Command, TEST_CMD));
        assert!(!button.is_down(), "pops out on release");
        assert!(!button.state().contains(State::DRAGGING));
    }

    #[test]
    fn dragging_off_pops_the_button_out_and_back_in() {
        const TEST_CMD: u16 = 534;
        command_set::enable_command(TEST_CMD);
        let mut button = Button::new(Rect::new(0, 0, 10, 2), "Test", TEST_CMD, false);

        button.handle_event(&mut mouse(EventType::MouseDown, 3, 0));
        button.handle_event(&mut mouse(EventType::MouseMove, 20, 5));
        assert!(!button.is_down(), "out while the pointer is off it");
        button.handle_event(&mut mouse(EventType::MouseMove, 4, 0));
        assert!(button.is_down(), "back in when the pointer returns");

        button.handle_event(&mut mouse(EventType::MouseMove, 20, 5));
        let mut up = mouse(EventType::MouseUp, 20, 5);
        button.handle_event(&mut up);
        assert_eq!(
            up.what,
            EventType::Nothing,
            "released off: consumed, no command"
        );
        assert!(!button.state().contains(State::DRAGGING));
    }

    #[test]
    fn disabling_or_blurring_a_held_button_releases_it() {
        const TEST_CMD: u16 = 535;
        command_set::enable_command(TEST_CMD);
        let mut button = Button::new(Rect::new(0, 0, 10, 2), "Test", TEST_CMD, false);

        button.handle_event(&mut mouse(EventType::MouseDown, 3, 0));
        button.set_disabled(true);
        assert!(!button.is_down());
        assert!(!button.state().contains(State::DRAGGING));

        button.set_disabled(false);
        button.set_focus(true);
        button.handle_event(&mut mouse(EventType::MouseDown, 3, 0));
        button.set_focus(false);
        assert!(!button.is_down());
        assert!(!button.state().contains(State::DRAGGING));
    }

    #[test]
    fn a_pushed_in_button_shifts_right_and_loses_its_shadow() {
        const TEST_CMD: u16 = 536;
        command_set::enable_command(TEST_CMD);
        // Face columns 0..8, shadow column 9, label "OK" at columns 3..5.
        let mut button = Button::new(Rect::new(0, 0, 10, 2), "OK", TEST_CMD, false);
        let mut terminal = crate::test_util::test_terminal(20, 5);

        button.draw(&mut terminal);
        let ch = |t: &Terminal, x, y| t.read_cell(x, y).unwrap().ch;
        assert_eq!((ch(&terminal, 3, 0), ch(&terminal, 4, 0)), ('O', 'K'));
        assert_eq!(ch(&terminal, 9, 0), SHADOW_TOP);
        assert_eq!(ch(&terminal, 1, 1), SHADOW_BOTTOM);
        let face = terminal.read_cell(0, 0).unwrap().attr;

        button.handle_event(&mut mouse(EventType::MouseDown, 3, 0));
        button.draw(&mut terminal);
        assert_eq!((ch(&terminal, 4, 0), ch(&terminal, 5, 0)), ('O', 'K'));
        assert_eq!(ch(&terminal, 9, 0), ' ', "face covers the shadow column");
        assert_eq!(terminal.read_cell(9, 0).unwrap().attr, face);
        assert_ne!(
            terminal.read_cell(0, 0).unwrap().attr,
            face,
            "vacated column"
        );
        assert_eq!(ch(&terminal, 1, 1), ' ', "no bottom shadow");
    }

    #[test]
    fn a_dialog_keeps_feeding_a_held_button_after_the_pointer_leaves_it() {
        use crate::views::dialog::Dialog;
        use crate::views::group::GroupLike as _;
        const TEST_CMD: u16 = 537;
        command_set::enable_command(TEST_CMD);

        let mut dialog = Dialog::new(Rect::new(0, 0, 40, 10), "Test");
        dialog.add(Button::new(Rect::new(2, 2, 12, 4), "Go", TEST_CMD, false));
        let button = |d: &Dialog| {
            d.child_at(0)
                .as_any()
                .downcast_ref::<Button>()
                .expect("button")
                .is_down()
        };
        // Dialog coordinates; the client area starts inside the frame.
        let inside = |p: Point| Point::new(p.x + 1, p.y + 1);
        let at = |what, p: Point| Event::mouse(what, p, MB_LEFT_BUTTON, false);

        dialog.handle_event(&mut at(EventType::MouseDown, inside(Point::new(5, 2))));
        assert!(button(&dialog));
        dialog.handle_event(&mut at(EventType::MouseMove, inside(Point::new(30, 6))));
        assert!(!button(&dialog), "the move off the button reached it");
        dialog.handle_event(&mut at(EventType::MouseMove, inside(Point::new(6, 2))));
        assert!(button(&dialog));
        let mut up = at(EventType::MouseUp, inside(Point::new(6, 2)));
        dialog.handle_event(&mut up);
        assert!(!button(&dialog));
    }

    #[test]
    fn test_button_grabs_default_on_focus_and_releases_on_blur() {
        // Focused button becomes the acting default; on blur the role reverts
        const TEST_CMD: u16 = 514;
        command_set::enable_command(TEST_CMD);

        let mut plain = Button::new(Rect::new(0, 0, 10, 3), "Plain", TEST_CMD, false);
        assert!(!plain.am_default, "non-default button starts without role");
        plain.set_focus(true);
        assert!(plain.am_default, "focused button grabs the default role");
        plain.set_focus(false);
        assert!(!plain.am_default, "blur releases the grabbed role");

        let mut default = Button::new(Rect::new(0, 0, 10, 3), "Def", TEST_CMD, true);
        assert!(default.am_default, "flagged default starts with the role");
        default.set_focus(true);
        default.set_focus(false);
        assert!(default.am_default, "flagged default keeps role after blur");
    }

    #[test]
    fn test_button_with_small_dimensions_doesnt_panic() {
        // REGRESSION TEST: Buttons with small/negative dimensions should not panic
        // This tests the fix for issue #53 where shrinking windows caused panics
        //
        // We can't actually call draw() in unit tests (no TTY), but we can verify
        // that the dimension clamping logic works correctly.

        const TEST_CMD: u16 = 511;

        // Test various small dimensions - should not panic on creation
        let test_cases = vec![
            Rect::new(0, 0, 0, 0),  // Zero dimensions
            Rect::new(0, 0, 1, 1),  // Too small (min is 4x2)
            Rect::new(0, 0, 2, 1),  // Width too small
            Rect::new(0, 0, 3, 1),  // Width too small
            Rect::new(0, 0, 4, 1),  // Height too small
            Rect::new(0, 0, 1, 2),  // Width too small
            Rect::new(0, 0, 2, 2),  // Width too small
            Rect::new(0, 0, 3, 2),  // Width too small
            Rect::new(10, 5, 5, 2), // Negative width (inverted)
            Rect::new(5, 10, 2, 5), // Negative height (inverted)
        ];

        for rect in test_cases {
            // Should not panic on creation or bounds queries
            let button = Button::new(rect, "Test", TEST_CMD, false);
            let bounds = button.bounds();

            // Verify clamping works
            assert!(bounds.width_clamped() >= 0);
            assert!(bounds.height_clamped() >= 0);
        }
    }

    #[test]
    fn a_non_ascii_hotkey_answers_its_letter() {
        const CMD: u16 = 531;
        command_set::enable_command(CMD);
        let mut button = Button::new(Rect::new(0, 0, 10, 2), "~Ñ~o", CMD, false);
        let mut e = Event::text('ñ');
        button.handle_event(&mut e);
        let e = timed_event::take_next().expect("a queued press");
        assert_eq!((e.what, e.command), (EventType::Command, CMD));
    }

    #[test]
    fn a_key_press_shows_the_button_down_then_sends_its_command() {
        const CMD: u16 = 538;
        command_set::enable_command(CMD);
        let mut button = Button::new(Rect::new(0, 0, 10, 2), "Go", CMD, false);
        button.set_focus(true);

        let before = Instant::now();
        let mut e = Event::keyboard(KB_ENTER);
        button.handle_event(&mut e);
        assert_eq!(e.what, EventType::Nothing, "the key is consumed");
        assert!(button.is_down(), "pushed in during the animation");

        let due = timed_event::next_due().expect("command queued");
        assert!(due >= before + press_animation());
        assert!(timed_event::take_due(Instant::now()).is_none(), "not yet");
        assert_eq!(button.key_down_until, Some(due), "pops out as it arrives");
        let e = timed_event::take_due(due).expect("due");
        assert_eq!((e.what, e.command), (EventType::Command, CMD));
    }

    #[test]
    fn a_broadcast_button_queues_its_broadcast() {
        const CMD: u16 = 539;
        command_set::enable_command(CMD);
        let mut button = Button::new(Rect::new(0, 0, 10, 2), "~G~o", CMD, false);
        button.set_broadcast(true);
        button.handle_event(&mut Event::text('g'));
        let e = timed_event::take_next().expect("a queued press");
        assert_eq!((e.what, e.command), (EventType::Broadcast, CMD));
    }

    #[test]
    fn without_animation_a_key_press_sends_the_command_at_once() {
        const CMD: u16 = 540;
        command_set::enable_command(CMD);
        set_press_animation(Duration::ZERO);
        let mut button = Button::new(Rect::new(0, 0, 10, 2), "Go", CMD, false);
        button.set_focus(true);
        let mut e = Event::keyboard(KB_ENTER);
        button.handle_event(&mut e);
        assert_eq!((e.what, e.command), (EventType::Command, CMD));
        assert!(!button.is_down());
        assert!(timed_event::next_due().is_none());
    }

    #[test]
    fn alt_and_the_hot_key_letter_presses_the_button() {
        const CMD: u16 = 541;
        command_set::enable_command(CMD);
        set_press_animation(Duration::ZERO);
        let mut button = Button::new(Rect::new(0, 0, 10, 2), "~O~K", CMD, false);
        // Not focused: the hot key works from the post-process phase.
        let mut e = Event::keyboard(crate::core::event::KB_ALT_O);
        button.handle_event(&mut e);
        assert_eq!((e.what, e.command), (EventType::Command, CMD));

        let mut other = Event::keyboard(crate::core::event::KB_ALT_X);
        button.handle_event(&mut other);
        assert_eq!(
            other.what,
            EventType::Keyboard,
            "another letter is left alone"
        );
    }

    #[test]
    fn a_character_past_latin1_does_not_press_an_unrelated_button() {
        // `ł` is U+0142: its key code's low byte used to read as `B`.
        const CMD: u16 = 532;
        command_set::enable_command(CMD);
        let mut button = Button::new(Rect::new(0, 0, 10, 2), "~B~ack", CMD, false);
        let mut e = Event::text('ł');
        button.handle_event(&mut e);
        assert_eq!(e.what, EventType::Keyboard, "left alone");
    }
}
