// (C) 2025 - Enzo Lombardi

//! View trait - base interface for all UI components with event handling and drawing.

use crate::core::geometry::Rect;
use crate::core::event::Event;
use crate::core::draw::DrawBuffer;
use crate::core::state::{StateFlags, SF_SHADOW, SF_FOCUSED, SHADOW_SIZE};
use crate::core::command::CommandId;
use crate::terminal::Terminal;
use std::io;

/// View trait - all UI components implement this
///
/// ## Owner/Parent Communication Pattern
///
/// Unlike Borland's TView which stores an `owner` pointer to the parent TGroup,
/// Rust views communicate with parents through event propagation:
///
/// **Borland Pattern:**
/// ```cpp
/// void TButton::press() {
///     message(owner, evBroadcast, command, this);
/// }
/// ```
///
/// **Rust Pattern:**
/// ```rust
/// fn handle_event(&mut self, event: &mut Event) {
///     // Transform event to send message upward
///     *event = Event::command(self.command);
///     // Event bubbles up through Group::handle_event() call stack
/// }
/// ```
///
/// This achieves the same result (child-to-parent communication) without raw pointers,
/// using Rust's ownership system and the call stack for context.
pub trait View {
    fn bounds(&self) -> Rect;
    fn set_bounds(&mut self, bounds: Rect);
    fn draw(&mut self, terminal: &mut Terminal);
    fn handle_event(&mut self, event: &mut Event);
    fn can_focus(&self) -> bool { false }

    /// Set focus state - default implementation uses SF_FOCUSED flag
    /// Views should override only if they need custom focus behavior
    fn set_focus(&mut self, focused: bool) {
        self.set_state_flag(SF_FOCUSED, focused);
    }

    /// Check if view is focused - reads SF_FOCUSED flag
    fn is_focused(&self) -> bool {
        self.get_state_flag(SF_FOCUSED)
    }

    /// Get view option flags (OF_SELECTABLE, OF_PRE_PROCESS, OF_POST_PROCESS, etc.)
    fn options(&self) -> u16 { 0 }

    /// Set view option flags
    fn set_options(&mut self, _options: u16) {}

    /// Get view state flags
    fn state(&self) -> StateFlags { 0 }

    /// Set view state flags
    fn set_state(&mut self, _state: StateFlags) {}

    /// Set or clear specific state flag(s)
    /// Matches Borland's TView::setState(ushort aState, Boolean enable)
    /// If enable is true, sets the flag(s), otherwise clears them
    fn set_state_flag(&mut self, flag: StateFlags, enable: bool) {
        let current = self.state();
        if enable {
            self.set_state(current | flag);
        } else {
            self.set_state(current & !flag);
        }
    }

    /// Check if specific state flag(s) are set
    /// Matches Borland's TView::getState(ushort aState)
    fn get_state_flag(&self, flag: StateFlags) -> bool {
        (self.state() & flag) == flag
    }

    /// Check if view has shadow enabled
    fn has_shadow(&self) -> bool {
        (self.state() & SF_SHADOW) != 0
    }

    /// Get bounds including shadow area
    fn shadow_bounds(&self) -> Rect {
        let mut bounds = self.bounds();
        if self.has_shadow() {
            bounds.b.x += SHADOW_SIZE.0;
            bounds.b.y += SHADOW_SIZE.1;
        }
        bounds
    }

    /// Update cursor state (called after draw)
    /// Views that need to show a cursor when focused should override this
    fn update_cursor(&self, _terminal: &mut Terminal) {
        // Default: do nothing (cursor stays hidden)
    }

    /// Zoom (maximize/restore) the view with given maximum bounds
    /// Matches Borland: TWindow::zoom() toggles between current and max size
    /// Default implementation does nothing (only windows support zoom)
    fn zoom(&mut self, _max_bounds: Rect) {
        // Default: do nothing (only Window implements zoom)
    }

    /// Validate the view before performing a command (usually closing)
    /// Matches Borland: TView::valid(ushort command) - returns Boolean
    /// Returns true if the view's state is valid for the given command
    /// Used for "Save before closing?" type scenarios and input validation
    ///
    /// # Arguments
    /// * `command` - The command being performed (CM_OK, CM_CANCEL, CM_RELEASED_FOCUS, etc.)
    ///
    /// # Returns
    /// * `true` - View state is valid, command can proceed
    /// * `false` - View state is invalid, command should be blocked
    ///
    /// Default implementation always returns true (no validation)
    fn valid(&mut self, _command: crate::core::command::CommandId) -> bool {
        true
    }

    /// Dump this view's region of the terminal buffer to an ANSI file for debugging
    fn dump_to_file(&self, terminal: &Terminal, path: &str) -> io::Result<()> {
        let bounds = self.shadow_bounds();
        terminal.dump_region(
            bounds.a.x as u16,
            bounds.a.y as u16,
            (bounds.b.x - bounds.a.x) as u16,
            (bounds.b.y - bounds.a.y) as u16,
            path,
        )
    }

    /// Check if this view is a default button (for Enter key handling at Dialog level)
    /// Corresponds to Borland's TButton::amDefault flag (tbutton.cc line 239)
    fn is_default_button(&self) -> bool {
        false
    }

    /// Get the command ID for this button (if it's a button)
    /// Returns None if not a button
    /// Used by Dialog to activate default button on Enter key
    fn button_command(&self) -> Option<u16> {
        None
    }

    /// Set the selection index for listbox views
    /// Only implemented by ListBox, other views ignore this
    fn set_list_selection(&mut self, _index: usize) {
        // Default: do nothing (not a listbox)
    }

    /// Get the selection index for listbox views
    /// Only implemented by ListBox, other views return 0
    fn get_list_selection(&self) -> usize {
        0
    }

    /// Get the union rect of previous and current bounds for redrawing
    /// Matches Borland: TView::locate() calculates union of old and new bounds
    /// Returns None if the view hasn't moved since last redraw
    /// Used by Desktop to implement Borland's drawUnderRect pattern
    fn get_redraw_union(&self) -> Option<Rect> {
        None // Default: no movement tracking
    }

    /// Clear movement tracking after redrawing
    /// Matches Borland: Called after drawUnderRect completes
    fn clear_move_tracking(&mut self) {
        // Default: do nothing (no movement tracking)
    }

    /// Get the end state for modal views
    /// Matches Borland: TGroup::endState field
    /// Returns the command ID that ended modal execution (0 if still running)
    fn get_end_state(&self) -> CommandId {
        0 // Default: not ended
    }

    /// Set the end state for modal views
    /// Called by end_modal() to signal the modal loop should exit
    fn set_end_state(&mut self, _command: CommandId) {
        // Default: do nothing (only modal views need this)
    }
}

/// Helper to draw a line to the terminal
pub fn write_line_to_terminal(terminal: &mut Terminal, x: i16, y: i16, buf: &DrawBuffer) {
    if y < 0 || y >= terminal.size().1 as i16 {
        return;
    }
    terminal.write_line(x.max(0) as u16, y as u16, &buf.data);
}

/// Draw shadow for a view
/// Draws a shadow offset by (1, 1) from the view bounds
/// Shadow is the same size as the view, but only the right and bottom edges are visible
pub fn draw_shadow(terminal: &mut Terminal, bounds: Rect, shadow_attr: u8) {
    use crate::core::palette::Attr;

    let attr = Attr::from_u8(shadow_attr);
    let mut buf = DrawBuffer::new(SHADOW_SIZE.0 as usize);

    // Draw right edge shadow (2 columns wide, offset by 1 vertically)
    // Starts at y+1 and extends to y+height+1
    for y in (bounds.a.y + 1)..(bounds.b.y + 1) {
        buf.move_char(0, ' ', attr, SHADOW_SIZE.0 as usize);
        write_line_to_terminal(terminal, bounds.b.x, y, &buf);
    }

    // Draw bottom edge shadow (offset by 1 horizontally, includes right shadow area)
    // Starts at x+1 and extends to match the right shadow end point
    // Width = view_width + (SHADOW_SIZE.0 - 1) because we offset by 1
    let bottom_width = (bounds.b.x - bounds.a.x + SHADOW_SIZE.0 - 1) as usize;
    let mut bottom_buf = DrawBuffer::new(bottom_width);
    bottom_buf.move_char(0, ' ', attr, bottom_width);
    write_line_to_terminal(terminal, bounds.a.x + 1, bounds.b.y, &bottom_buf);
}

