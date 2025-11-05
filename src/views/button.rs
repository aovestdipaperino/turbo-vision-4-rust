use super::view::{write_line_to_terminal, View};
use crate::core::command::CommandId;
use crate::core::draw::DrawBuffer;
use crate::core::event::{Event, EventType, KB_ENTER, MB_LEFT_BUTTON};
use crate::core::geometry::Rect;
use crate::core::palette::colors;
use crate::core::state::{StateFlags, SF_DISABLED, SHADOW_BOTTOM, SHADOW_SOLID, SHADOW_TOP};
use crate::terminal::Terminal;

pub struct Button {
    bounds: Rect,
    title: String,
    command: CommandId,
    is_default: bool,
    state: StateFlags,
    options: u16,
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
            state,
            options: OF_POST_PROCESS,  // Buttons process in post-process phase
        }
    }

    pub fn set_disabled(&mut self, disabled: bool) {
        self.set_state_flag(SF_DISABLED, disabled);
    }

    pub fn is_disabled(&self) -> bool {
        self.get_state_flag(SF_DISABLED)
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

        let button_attr = if is_disabled {
            colors::BUTTON_DISABLED
        } else if is_focused {
            colors::BUTTON_SELECTED
        } else if self.is_default {
            colors::BUTTON_DEFAULT
        } else {
            colors::BUTTON_NORMAL
        };

        // Shadow uses DarkGray on LightGray (not black background!)
        let shadow_attr = colors::BUTTON_SHADOW;

        // Shortcut attributes - use yellow for button shortcuts
        let shortcut_attr = if is_disabled {
            colors::BUTTON_DISABLED  // DarkGray on Green (disabled)
        } else if is_focused {
            colors::BUTTON_SELECTED  // White on Green (focused)
        } else {
            colors::BUTTON_SHORTCUT  // Yellow on Green (not focused)
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

            write_line_to_terminal(
                terminal,
                self.bounds.a.x,
                self.bounds.a.y + y as i16,
                &buf,
            );
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
                    *event = Event::command(self.command);
                }
            }
            EventType::MouseDown => {
                // Check if click is within button bounds
                let mouse_pos = event.mouse.pos;
                if event.mouse.buttons & MB_LEFT_BUTTON != 0
                    && mouse_pos.x >= self.bounds.a.x
                    && mouse_pos.x < self.bounds.b.x
                    && mouse_pos.y >= self.bounds.a.y
                    && mouse_pos.y < self.bounds.b.y - 1  // Exclude shadow line
                {
                    // Button clicked - generate command
                    *event = Event::command(self.command);
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
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::event::{Event, EventType};
    use crate::core::command::CM_COMMAND_SET_CHANGED;
    use crate::core::command_set;
    use crate::core::geometry::Point;

    #[test]
    fn test_button_creation_with_disabled_command() {
        // Test that button is created disabled when command is disabled
        const TEST_CMD: u16 = 500;
        command_set::disable_command(TEST_CMD);

        let button = Button::new(
            Rect::new(0, 0, 10, 2),
            "Test",
            TEST_CMD,
            false
        );

        assert!(button.is_disabled(), "Button should start disabled when command is disabled");
    }

    #[test]
    fn test_button_creation_with_enabled_command() {
        // Test that button is created enabled when command is enabled
        const TEST_CMD: u16 = 501;
        command_set::enable_command(TEST_CMD);

        let button = Button::new(
            Rect::new(0, 0, 10, 2),
            "Test",
            TEST_CMD,
            false
        );

        assert!(!button.is_disabled(), "Button should start enabled when command is enabled");
    }

    #[test]
    fn test_disabled_button_receives_broadcast_and_becomes_enabled() {
        // REGRESSION TEST: Disabled buttons must receive broadcasts to become enabled
        // This tests the fix for the bug where disabled buttons returned early
        // and never received CM_COMMAND_SET_CHANGED broadcasts

        const TEST_CMD: u16 = 502;

        // Start with command disabled
        command_set::disable_command(TEST_CMD);

        let mut button = Button::new(
            Rect::new(0, 0, 10, 2),
            "Test",
            TEST_CMD,
            false
        );

        // Verify button starts disabled
        assert!(button.is_disabled(), "Button should start disabled");

        // Enable the command in the global command set
        command_set::enable_command(TEST_CMD);

        // Send broadcast to button
        let mut event = Event::broadcast(CM_COMMAND_SET_CHANGED);
        button.handle_event(&mut event);

        // Verify button is now enabled
        assert!(!button.is_disabled(), "Button should be enabled after receiving broadcast");
    }

    #[test]
    fn test_enabled_button_receives_broadcast_and_becomes_disabled() {
        // Test that enabled buttons can be disabled via broadcast

        const TEST_CMD: u16 = 503;

        // Start with command enabled
        command_set::enable_command(TEST_CMD);

        let mut button = Button::new(
            Rect::new(0, 0, 10, 2),
            "Test",
            TEST_CMD,
            false
        );

        // Verify button starts enabled
        assert!(!button.is_disabled(), "Button should start enabled");

        // Disable the command in the global command set
        command_set::disable_command(TEST_CMD);

        // Send broadcast to button
        let mut event = Event::broadcast(CM_COMMAND_SET_CHANGED);
        button.handle_event(&mut event);

        // Verify button is now disabled
        assert!(button.is_disabled(), "Button should be disabled after receiving broadcast");
    }

    #[test]
    fn test_disabled_button_ignores_keyboard_events() {
        // Test that disabled buttons don't respond to keyboard input

        const TEST_CMD: u16 = 504;
        command_set::disable_command(TEST_CMD);

        let mut button = Button::new(
            Rect::new(0, 0, 10, 2),
            "Test",
            TEST_CMD,
            false
        );

        button.set_focus(true);

        // Try to activate with Enter key
        let mut event = Event::keyboard(crate::core::event::KB_ENTER);
        button.handle_event(&mut event);

        // Event should not be converted to command
        assert_ne!(event.what, EventType::Command, "Disabled button should not generate command");
    }

    #[test]
    fn test_disabled_button_ignores_mouse_clicks() {
        // Test that disabled buttons don't respond to mouse clicks

        const TEST_CMD: u16 = 505;
        command_set::disable_command(TEST_CMD);

        let mut button = Button::new(
            Rect::new(0, 0, 10, 2),
            "Test",
            TEST_CMD,
            false
        );

        // Try to click the button
        let mut event = Event::mouse(
            EventType::MouseDown,
            Point::new(5, 1),
            crate::core::event::MB_LEFT_BUTTON,
            false
        );
        button.handle_event(&mut event);

        // Event should not be converted to command
        assert_ne!(event.what, EventType::Command, "Disabled button should not generate command");
    }

    #[test]
    fn test_broadcast_does_not_clear_event() {
        // Test that CM_COMMAND_SET_CHANGED broadcast is not cleared
        // (so it can propagate to other buttons)

        const TEST_CMD: u16 = 506;
        command_set::disable_command(TEST_CMD);

        let mut button = Button::new(
            Rect::new(0, 0, 10, 2),
            "Test",
            TEST_CMD,
            false
        );

        command_set::enable_command(TEST_CMD);

        let mut event = Event::broadcast(CM_COMMAND_SET_CHANGED);
        button.handle_event(&mut event);

        // Event should still be a broadcast (not cleared)
        assert_eq!(event.what, EventType::Broadcast, "Broadcast should not be cleared");
        assert_eq!(event.command, CM_COMMAND_SET_CHANGED, "Broadcast command should remain");
    }
}
