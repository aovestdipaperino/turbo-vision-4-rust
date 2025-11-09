// (C) 2025 - Enzo Lombardi

//! Button view - clickable button with keyboard shortcuts and command dispatch.

use super::view::{write_line_to_terminal, View};
use crate::core::command::CommandId;
use crate::core::draw::DrawBuffer;
use crate::core::event::{Event, EventType, KB_ENTER, MB_LEFT_BUTTON};
use crate::core::geometry::Rect;
use crate::core::palette::{
    BUTTON_DEFAULT, BUTTON_DISABLED, BUTTON_NORMAL, BUTTON_SELECTED, BUTTON_SHADOW, BUTTON_SHORTCUT,
};
use crate::core::state::{StateFlags, SF_DISABLED, SHADOW_BOTTOM, SHADOW_SOLID, SHADOW_TOP};
use crate::terminal::Terminal;

pub struct Button {
    bounds: Rect,
    title: String,
    command: CommandId,
    is_default: bool,
    is_broadcast: bool,
    state: StateFlags,
    options: u16,
    owner: Option<*const dyn View>,
}

impl Button {
    pub fn new(bounds: Rect, title: &str, command: CommandId, is_default: bool) -> Self {
        use crate::core::command_set;
        use crate::core::state::OF_POST_PROCESS;

        // Check if command is initially enabled
        // Matches Borland: TButton constructor checks commandEnabled() (tbutton.cc:55-56)
        let mut state = 0;
        if !command_set::command_enabled(command) {
            state |= SF_DISABLED;
        }

        Self {
            bounds,
            title: title.to_string(),
            command,
            is_default,
            is_broadcast: false,
            state,
            options: OF_POST_PROCESS, // Buttons process in post-process phase
            owner: None,
        }
    }

    pub fn set_disabled(&mut self, disabled: bool) {
        self.set_state_flag(SF_DISABLED, disabled);
    }

    pub fn is_disabled(&self) -> bool {
        self.get_state_flag(SF_DISABLED)
    }

    /// Set whether this button broadcasts its command instead of sending it as a command event
    /// Matches Borland: bfBroadcast flag
    pub fn set_broadcast(&mut self, broadcast: bool) {
        self.is_broadcast = broadcast;
    }

    /// Set whether this button is selectable (can receive focus)
    /// Matches Borland: ofSelectable flag
    pub fn set_selectable(&mut self, selectable: bool) {
        use crate::core::state::OF_SELECTABLE;
        if selectable {
            self.options |= OF_SELECTABLE;
        } else {
            self.options &= !OF_SELECTABLE;
        }
    }
}

impl View for Button {
    fn bounds(&self) -> Rect {
        self.bounds
    }

    fn set_bounds(&mut self, bounds: Rect) {
        self.bounds = bounds;
    }

    fn draw(&mut self, terminal: &mut Terminal) {
        let width = self.bounds.width() as usize;
        let height = self.bounds.height() as usize;

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
        } else if self.is_default {
            self.map_color(BUTTON_DEFAULT) // Default but not focused
        } else {
            self.map_color(BUTTON_NORMAL) // Normal
        };

        let shadow_attr = self.map_color(BUTTON_SHADOW); // Shadow

        // Shortcut attributes
        let shortcut_attr = if is_disabled {
            self.map_color(BUTTON_DISABLED) // Disabled shortcut same as disabled text
        } else {
            self.map_color(BUTTON_SHORTCUT) // Shortcut color
        };

        // Draw all lines except the last (which is the bottom shadow)
        for y in 0..(height - 1) {
            let mut buf = DrawBuffer::new(width);

            // Fill entire line with button color
            buf.move_char(0, ' ', button_attr, width);

            // Right edge gets shadow character and attribute (last column)
            let shadow_char = if y == 0 { SHADOW_TOP } else { SHADOW_SOLID };
            buf.put_char(width - 1, shadow_char, shadow_attr);

            // Draw the label on the middle line
            if y == (height - 1) / 2 {
                // Calculate display length without tildes
                let display_len = self.title.chars().filter(|&c| c != '~').count();
                let content_width = width - 1; // Exclude right shadow column
                let start = (content_width.saturating_sub(display_len)) / 2;
                buf.move_str_with_shortcut(start, &self.title, button_attr, shortcut_attr);
            }

            write_line_to_terminal(terminal, self.bounds.a.x, self.bounds.a.y + y as i16, &buf);
        }

        // Draw bottom shadow line (1 char shorter, offset 1 to the right)
        let mut bottom_buf = DrawBuffer::new(width - 1);
        // Bottom shadow character across width-1
        bottom_buf.move_char(0, SHADOW_BOTTOM, shadow_attr, width - 1);
        write_line_to_terminal(
            terminal,
            self.bounds.a.x + 1,
            self.bounds.a.y + (height - 1) as i16,
            &bottom_buf,
        );
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
            use crate::core::command::CM_COMMAND_SET_CHANGED;
            use crate::core::command_set;

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
                // Only handle keyboard events if focused
                if !self.is_focused() {
                    return;
                }
                if event.key_code == KB_ENTER || event.key_code == ' ' as u16 {
                    if self.is_broadcast {
                        *event = Event::broadcast(self.command);
                    } else {
                        *event = Event::command(self.command);
                    }
                }
            }
            EventType::MouseDown => {
                // Check if click is within button bounds
                let mouse_pos = event.mouse.pos;
                if event.mouse.buttons & MB_LEFT_BUTTON != 0
                    && mouse_pos.x >= self.bounds.a.x
                    && mouse_pos.x < self.bounds.b.x
                    && mouse_pos.y >= self.bounds.a.y
                    && mouse_pos.y < self.bounds.b.y - 1
                // Exclude shadow line
                {
                    // Button clicked - generate command or broadcast
                    if self.is_broadcast {
                        *event = Event::broadcast(self.command);
                    } else {
                        *event = Event::command(self.command);
                    }
                }
            }
            _ => {}
        }
    }

    fn can_focus(&self) -> bool {
        !self.is_disabled()
    }

    // set_focus() now uses default implementation from View trait
    // which sets/clears SF_FOCUSED flag

    fn state(&self) -> StateFlags {
        self.state
    }

    fn set_state(&mut self, state: StateFlags) {
        self.state = state;
    }

    fn options(&self) -> u16 {
        self.options
    }

    fn set_options(&mut self, options: u16) {
        self.options = options;
    }

    fn is_default_button(&self) -> bool {
        self.is_default
    }

    fn button_command(&self) -> Option<u16> {
        Some(self.command)
    }

    fn set_owner(&mut self, owner: *const dyn View) {
        self.owner = Some(owner);
    }

    fn get_owner(&self) -> Option<*const dyn View> {
        self.owner
    }

    fn get_palette(&self) -> Option<crate::core::palette::Palette> {
        use crate::core::palette::{palettes, Palette};
        Some(Palette::from_slice(palettes::CP_BUTTON))
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
    /// Creates a new ButtonBuilder with default values.
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
        assert_eq!(button.is_default_button(), true);
        assert_eq!(button.button_command(), Some(TEST_CMD));
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

        assert_eq!(button.is_default_button(), false);
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
}
