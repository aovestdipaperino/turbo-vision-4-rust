// (C) 2025 - Enzo Lombardi

//! View trait - base interface for all UI components with event handling and drawing.

use crate::core::command::CommandId;
use crate::core::draw::DrawBuffer;
use crate::core::event::Event;
use crate::core::geometry::Rect;
use crate::core::state::{StateFlags, SF_FOCUSED, SF_SHADOW, SHADOW_ATTR, SHADOW_SIZE};
use crate::terminal::Terminal;
use std::io;

/// Owner context for palette remapping
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum OwnerType {
    None,   // Top-level view (Application)
    Window, // Inside a Window
    Dialog, // Inside a Dialog
}

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
    fn can_focus(&self) -> bool {
        false
    }

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
    fn options(&self) -> u16 {
        0
    }

    /// Set view option flags
    fn set_options(&mut self, _options: u16) {}

    /// Get view state flags
    fn state(&self) -> StateFlags {
        0
    }

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

    /// Downcast to concrete type (immutable)
    /// Allows accessing specific view type methods from trait object
    fn as_any(&self) -> &dyn std::any::Any {
        panic!("as_any() not implemented for this view type")
    }

    /// Downcast to concrete type (mutable)
    /// Allows accessing specific view type methods from trait object
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        panic!("as_any_mut() not implemented for this view type")
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

    /// Convert local coordinates to global (screen) coordinates
    /// Matches Borland: TView::makeGlobal(TPoint source, TPoint& dest)
    ///
    /// In Borland, makeGlobal traverses the owner chain and accumulates offsets.
    /// In this Rust implementation, views store absolute bounds (converted in Group::add()),
    /// so we simply add the view's origin to the local coordinates.
    ///
    /// # Arguments
    /// * `local_x` - X coordinate relative to view's interior (0,0 = top-left of view)
    /// * `local_y` - Y coordinate relative to view's interior
    ///
    /// # Returns
    /// Global (screen) coordinates as (x, y) tuple
    fn make_global(&self, local_x: i16, local_y: i16) -> (i16, i16) {
        let bounds = self.bounds();
        (bounds.a.x + local_x, bounds.a.y + local_y)
    }

    /// Convert global (screen) coordinates to local view coordinates
    /// Matches Borland: TView::makeLocal(TPoint source, TPoint& dest)
    ///
    /// In Borland, makeLocal is the inverse of makeGlobal, converting screen
    /// coordinates back to view-relative coordinates.
    ///
    /// # Arguments
    /// * `global_x` - X coordinate in screen space
    /// * `global_y` - Y coordinate in screen space
    ///
    /// # Returns
    /// Local coordinates as (x, y) tuple, where (0,0) is the view's top-left
    fn make_local(&self, global_x: i16, global_y: i16) -> (i16, i16) {
        let bounds = self.bounds();
        (global_x - bounds.a.x, global_y - bounds.a.y)
    }

    /// Draw shadow for this view
    /// Draws a shadow offset by (1, 1) from the view bounds
    /// Shadow is semi-transparent - darkens the underlying content by 50%
    /// This matches the Borland Turbo Vision behavior more closely
    fn draw_shadow(&self, terminal: &mut Terminal) {
        use crate::core::palette::Attr;

        const SHADOW_FACTOR: f32 = 0.5; // Darken to 50% of original brightness

        let bounds = self.bounds();
        let mut buf = DrawBuffer::new(SHADOW_SIZE.0 as usize);

        // Draw right edge shadow (2 columns wide, offset by 1 vertically)
        // Read existing cells and darken them for semi-transparency
        for y in (bounds.a.y + 1)..(bounds.b.y + 1) {
            for i in 0..SHADOW_SIZE.0 {
                let x = bounds.b.x + i;

                // Read the existing cell at this position
                if let Some(existing_cell) = terminal.read_cell(x, y) {
                    // Darken the existing cell's attribute
                    let darkened_attr = existing_cell.attr.darken(SHADOW_FACTOR);
                    buf.put_char(i as usize, existing_cell.ch, darkened_attr);
                } else {
                    // Out of bounds - use default shadow
                    let default_attr = Attr::from_u8(SHADOW_ATTR);
                    buf.put_char(i as usize, ' ', default_attr);
                }
            }
            write_line_to_terminal(terminal, bounds.b.x, y, &buf);
        }

        // Draw bottom edge shadow (offset by 1 horizontally, excludes right shadow area to prevent double-darkening)
        let bottom_width = (bounds.b.x - bounds.a.x - 1) as usize;
        let mut bottom_buf = DrawBuffer::new(bottom_width);

        let shadow_y = bounds.b.y;
        for i in 0..bottom_width {
            let x = bounds.a.x + 1 + i as i16;

            // Read the existing cell at this position
            if let Some(existing_cell) = terminal.read_cell(x, shadow_y) {
                // Darken the existing cell's attribute
                let darkened_attr = existing_cell.attr.darken(SHADOW_FACTOR);
                bottom_buf.put_char(i, existing_cell.ch, darkened_attr);
            } else {
                // Out of bounds - use default shadow
                let default_attr = Attr::from_u8(SHADOW_ATTR);
                bottom_buf.put_char(i, ' ', default_attr);
            }
        }
        write_line_to_terminal(terminal, bounds.a.x + 1, bounds.b.y, &bottom_buf);
    }

    /// Get the linked control index for labels
    /// Matches Borland: TLabel::link field
    /// Returns Some(index) if this is a label with a linked control, None otherwise
    /// Used by Group to implement focus transfer when clicking labels
    fn label_link(&self) -> Option<usize> {
        None // Default: not a label or no link
    }

    /// Initialize internal owner pointers after view is added to parent and won't move
    /// This is called by parent's add() method after the view is in its final position
    /// Views that contain other views by value should override this to set up owner chains
    /// Default implementation does nothing
    fn init_after_add(&mut self) {
        // Default: no action needed
    }

    /// Constrain view bounds to parent/owner bounds
    /// Used after positioning (e.g., centering) to ensure view stays within valid area
    /// Matches Borland: TView::locate() constrains position to owner bounds
    fn constrain_to_parent_bounds(&mut self) {
        // Default: no action needed (only windows need this)
    }

    /// Set the owner (parent) of this view
    /// Matches Borland: TView::owner field
    /// Called by Group when adding a child
    fn set_owner(&mut self, _owner: *const dyn View) {
        // Default: do nothing (views that need owner support will override)
    }

    /// Get the owner (parent) of this view
    /// Matches Borland: TView::owner field
    /// Returns None if this view has no owner or doesn't track it
    fn get_owner(&self) -> Option<*const dyn View> {
        None // Default: no owner
    }

    /// Get the owner type for palette remapping
    /// This allows views to know their context (Window vs Dialog)
    fn get_owner_type(&self) -> OwnerType {
        OwnerType::None // Default: no owner
    }

    /// Set the owner type for palette remapping
    fn set_owner_type(&mut self, _owner_type: OwnerType) {
        // Default: do nothing (views that need context will override)
    }

    /// Get this view's palette for the Borland indirect palette system
    /// Matches Borland: TView::getPalette()
    ///
    /// Returns a Palette that maps this view's logical color indices to the parent's indices.
    /// When resolving colors, the system walks up the owner chain remapping through palettes
    /// until reaching the Application which has actual color attributes.
    ///
    /// # Returns
    /// * `Some(Palette)` - This view has a palette for color remapping
    /// * `None` - This view has no palette (transparent to color mapping)
    fn get_palette(&self) -> Option<crate::core::palette::Palette>;

    /// Map a logical color index to an actual color attribute
    /// Matches Borland: TView::mapColor(uchar index)
    ///
    /// Walks up the owner chain, remapping the color index through each view's palette
    /// until reaching a view with no owner (Application), which provides actual attributes.
    ///
    /// # Arguments
    /// * `color_index` - Logical color index (1-based, 0 = error color)
    ///
    /// # Returns
    /// The final color attribute
    fn map_color(&self, color_index: u8) -> crate::core::palette::Attr {
        use crate::core::palette::{palettes, Attr, Palette};

        const ERROR_ATTR: u8 = 0x0F; // White on Black

        if color_index == 0 {
            return Attr::from_u8(ERROR_ATTR);
        }

        let mut color = color_index;

        // First, remap through this view's palette
        if let Some(palette) = self.get_palette() {
            if !palette.is_empty() {
                color = palette.get(color as usize);
                if color == 0 {
                    return Attr::from_u8(ERROR_ATTR);
                }
            }
        }

        // NOTE: We skip the owner chain traversal to avoid unsafe pointer dereference.
        // Instead, we apply a standard palette chain: View -> Window/Dialog -> Application
        //
        // Borland Turbo Vision palette layout (from program.h):
        //    1      = TBackground
        //    2-7    = TMenuView and TStatusLine (direct to app)
        //    8-15   = TWindow(Blue)
        //    16-23  = TWindow(Cyan)
        //    24-31  = TWindow(Gray)
        //    32-63  = TDialog (remapped through dialog palette)
        //
        // Apply palette remapping based on ranges
        // This is a simplified version of Borland's owner chain traversal

        // For dialog controls (indices 1-31), remap through dialog palette based on OwnerType
        // This includes scrollbars (4-5), static text (6), labels (7-9), buttons (10-14), etc.
        // Note: Indices 32+ are already app palette indices and shouldn't be remapped
        // Only remap if we're explicitly in a Dialog context
        // Views with OwnerType::None (MenuBar, StatusLine, Desktop) use direct app palette
        // Views with OwnerType::Window use direct app palette (Window-specific remapping not implemented)
        let owner_type = self.get_owner_type();
        if color >= 1 && color < 32 && owner_type == OwnerType::Dialog {
            // Remap through dialog palette
            let dialog_palette = Palette::from_slice(palettes::CP_GRAY_DIALOG);
            let remapped = dialog_palette.get(color as usize);
            if remapped > 0 {
                color = remapped;
            }
        }

        // Reached root (Application) - color is now an index into app palette
        // Use the application color palette to get the final attribute
        let app_palette_data = palettes::get_app_palette();
        let app_palette = Palette::from_slice(&app_palette_data);
        let final_color = app_palette.get(color as usize);
        if final_color == 0 {
            return Attr::from_u8(ERROR_ATTR);
        }
        Attr::from_u8(final_color)
    }
}

/// Helper to draw a line to the terminal
pub fn write_line_to_terminal(terminal: &mut Terminal, x: i16, y: i16, buf: &DrawBuffer) {
    if y < 0 || y >= terminal.size().1 as i16 {
        return;
    }
    terminal.write_line(x.max(0) as u16, y as u16, &buf.data);
}

/// Draw shadow for arbitrary bounds (for non-view elements like temporary dropdowns)
///
/// Note: Views should use the `draw_shadow()` trait method instead, which gets bounds
/// from `self.bounds()` following the principle "bounds should not be passed down".
/// This standalone function is only for special cases where you're drawing shadows
/// for elements that aren't views (e.g., temporary dropdowns).
pub fn draw_shadow_bounds(terminal: &mut Terminal, bounds: Rect) {
    use crate::core::palette::Attr;

    const SHADOW_FACTOR: f32 = 0.5; // Darken to 50% of original brightness

    let mut buf = DrawBuffer::new(SHADOW_SIZE.0 as usize);

    // Draw right edge shadow (2 columns wide, offset by 1 vertically)
    // Read existing cells and darken them for semi-transparency
    for y in (bounds.a.y + 1)..(bounds.b.y + 1) {
        for i in 0..SHADOW_SIZE.0 {
            let x = bounds.b.x + i;

            // Read the existing cell at this position
            if let Some(existing_cell) = terminal.read_cell(x, y) {
                // Darken the existing cell's attribute
                let darkened_attr = existing_cell.attr.darken(SHADOW_FACTOR);
                buf.put_char(i as usize, existing_cell.ch, darkened_attr);
            } else {
                // Out of bounds - use default shadow
                let default_attr = Attr::from_u8(SHADOW_ATTR);
                buf.put_char(i as usize, ' ', default_attr);
            }
        }
        write_line_to_terminal(terminal, bounds.b.x, y, &buf);
    }

    // Draw bottom edge shadow (offset by 1 horizontally, excludes right shadow area to prevent double-darkening)
    let bottom_width = (bounds.b.x - bounds.a.x - 1) as usize;
    let mut bottom_buf = DrawBuffer::new(bottom_width);

    let shadow_y = bounds.b.y;
    for i in 0..bottom_width {
        let x = bounds.a.x + 1 + i as i16;

        // Read the existing cell at this position
        if let Some(existing_cell) = terminal.read_cell(x, shadow_y) {
            // Darken the existing cell's attribute
            let darkened_attr = existing_cell.attr.darken(SHADOW_FACTOR);
            bottom_buf.put_char(i, existing_cell.ch, darkened_attr);
        } else {
            // Out of bounds - use default shadow
            let default_attr = Attr::from_u8(SHADOW_ATTR);
            bottom_buf.put_char(i, ' ', default_attr);
        }
    }
    write_line_to_terminal(terminal, bounds.a.x + 1, bounds.b.y, &bottom_buf);
}
