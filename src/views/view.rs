// (C) 2025 - Enzo Lombardi

//! View trait - base interface for all UI components with event handling and drawing.

use crate::core::draw::DrawBuffer;
use crate::core::event::Event;
use crate::core::geometry::Rect;
use crate::core::state::{SF_FOCUSED, SF_SHADOW, SHADOW_ATTR, StateFlags, shadow_size};
use crate::terminal::Terminal;
use std::io;
use std::sync::atomic::{AtomicUsize, Ordering};

/// Unique identifier for a view within a Group
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ViewId(usize);

impl ViewId {
    /// Generate a new unique ViewId
    pub(crate) fn new() -> Self {
        static NEXT_ID: AtomicUsize = AtomicUsize::new(1);
        ViewId(NEXT_ID.fetch_add(1, Ordering::Relaxed))
    }

    /// Get the ViewId as a u16 for embedding in event fields.
    /// ViewId values are small sequential numbers that fit in u16.
    #[allow(clippy::cast_possible_truncation)]
    pub fn as_u16(self) -> u16 {
        self.0 as u16
    }

    /// Reconstruct a ViewId from a u16 value.
    pub fn from_u16(val: u16) -> Self {
        ViewId(val as usize)
    }
}

/// The fields Borland declares once in `TView` and every subclass inherits.
/// Each view owns exactly one of these and hands it back from `View::core()`.
#[derive(Clone, Default)]
pub struct ViewCore {
    pub bounds: Rect,
    pub state: StateFlags,
    pub options: u16,
    pub grow_mode: crate::core::state::GrowFlags,
    pub palette_chain: Option<crate::core::palette_chain::PaletteChainNode>,
}

impl std::fmt::Debug for ViewCore {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ViewCore")
            .field("bounds", &self.bounds)
            .field("state", &self.state)
            .field("options", &self.options)
            .field("grow_mode", &self.grow_mode)
            .field("palette_chain", &self.palette_chain.is_some())
            .finish()
    }
}

impl ViewCore {
    pub fn new(bounds: Rect) -> Self {
        Self {
            bounds,
            ..Self::default()
        }
    }

    pub fn with_options(bounds: Rect, options: u16) -> Self {
        Self {
            bounds,
            options,
            ..Self::default()
        }
    }
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
/// ```ignore
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
    /// Base fields shared by every view (Borland: the `TView` data members).
    fn core(&self) -> &ViewCore;
    fn core_mut(&mut self) -> &mut ViewCore;

    fn bounds(&self) -> Rect {
        self.core().bounds
    }
    fn set_bounds(&mut self, bounds: Rect) {
        self.core_mut().bounds = bounds;
    }
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

    /// Window number for Alt+1..9 selection (Borland: TWindow::number).
    ///
    /// `None` for views that aren't numbered windows (Borland wnNoNumber).
    fn window_number(&self) -> Option<u8> {
        None
    }

    /// Check if view is focused - reads SF_FOCUSED flag
    fn is_focused(&self) -> bool {
        self.get_state_flag(SF_FOCUSED)
    }

    /// Get view option flags (OF_SELECTABLE, OF_PRE_PROCESS, OF_POST_PROCESS, etc.)
    fn options(&self) -> u16 {
        self.core().options
    }

    /// Set view option flags
    fn set_options(&mut self, options: u16) {
        self.core_mut().options = options;
    }

    /// Get view state flags
    fn state(&self) -> StateFlags {
        self.core().state
    }

    /// Set view state flags
    fn set_state(&mut self, state: StateFlags) {
        self.core_mut().state = state;
    }

    /// Get this view's grow mode flags (Borland: TView::growMode).
    ///
    /// Controls how the view's edges move when its parent Group is resized.
    /// See `GF_GROW_LO_X`, `GF_GROW_LO_Y`, `GF_GROW_HI_X`, `GF_GROW_HI_Y`
    /// and `GF_GROW_ALL` in `core::state`. The default is `0` (fixed size
    /// and position relative to the parent's origin), matching Borland's
    /// default `growMode = 0`.
    fn grow_mode(&self) -> crate::core::state::GrowFlags {
        self.core().grow_mode
    }

    /// Set this view's grow mode flags (Borland: TView::growMode).
    ///
    /// The default implementation is a no-op; views that participate in
    /// parent-resize layout store the flags in a field and override both
    /// `grow_mode()` and `set_grow_mode()`.
    fn set_grow_mode(&mut self, grow_mode: crate::core::state::GrowFlags) {
        self.core_mut().grow_mode = grow_mode;
    }

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
            let ss = shadow_size();
            bounds.b.x += ss.0;
            bounds.b.y += ss.1;
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
    fn as_any(&self) -> &dyn std::any::Any;

    /// The container interface of this view, if it is one (Borland:
    /// `dynamic_cast<TGroup*>`). Containers built on `Group` return `Some`;
    /// leaf views keep the default `None`. `Application::exec_view` uses it
    /// to read a modal view's end state without knowing its concrete type.
    fn as_group(&self) -> Option<&dyn crate::views::group::GroupLike> {
        None
    }
    fn as_group_mut(&mut self) -> Option<&mut dyn crate::views::group::GroupLike> {
        None
    }

    /// Downcast to concrete type (mutable)
    /// Allows accessing specific view type methods from trait object
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any;

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
    /// Draws a shadow offset dynamically based on terminal cell aspect ratio
    /// Shadow is semi-transparent - darkens the underlying content by 50%
    /// This matches the Borland Turbo Vision behavior more closely
    fn draw_shadow(&self, terminal: &mut Terminal) {
        use crate::core::palette::Attr;

        const SHADOW_FACTOR: f32 = 0.5; // Darken to 50% of original brightness

        let bounds = self.bounds();
        let ss = shadow_size();
        let mut buf = DrawBuffer::new(ss.0 as usize);

        // Draw right edge shadow (ss.0 columns wide, offset by ss.1 vertically)
        // Read existing cells and darken them for semi-transparency
        for y in (bounds.a.y + ss.1)..(bounds.b.y + ss.1) {
            for i in 0..ss.0 {
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

        // Draw bottom edge shadow (offset by ss.0 horizontally, excludes right shadow area to prevent double-darkening)
        let bottom_width = (bounds.b.x - bounds.a.x - ss.0) as usize;
        let mut bottom_buf = DrawBuffer::new(bottom_width);

        let shadow_y = bounds.b.y;
        for i in 0..bottom_width {
            let x = bounds.a.x + ss.0 + i as i16;

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
        write_line_to_terminal(terminal, bounds.a.x + ss.0, bounds.b.y, &bottom_buf);
    }

    /// Get the linked control ViewId for labels
    /// Matches Borland: TLabel::link field
    /// Returns Some(ViewId) if this is a label with a linked control, None otherwise
    /// Used by Group to implement focus transfer when clicking labels
    fn label_link(&self) -> Option<ViewId> {
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

    /// Set the QCell-based palette chain node for this view.
    /// Called by parent (Group/Window) during draw to establish the safe owner chain.
    fn set_palette_chain(&mut self, node: Option<crate::core::palette_chain::PaletteChainNode>) {
        self.core_mut().palette_chain = node;
    }

    /// Get the QCell-based palette chain node for this view.
    /// Used by `map_color()` to safely walk the owner chain.
    fn get_palette_chain(&self) -> Option<&crate::core::palette_chain::PaletteChainNode> {
        self.core().palette_chain.as_ref()
    }

    /// Set the parent's bounds for drag/resize limit resolution.
    /// Called by Desktop when adding windows.
    fn set_parent_bounds(&mut self, _bounds: crate::core::geometry::Rect) {
        // Default: do nothing (only Window needs this)
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
        use crate::core::palette::{Attr, palettes};

        // Borland's errorAttr = 0xCF (Light Red/Magenta background, White foreground)
        const ERROR_ATTR: u8 = 0xCF;

        if color_index == 0 {
            return Attr::from_u8(ERROR_ATTR);
        }

        let mut color = color_index;

        // Step 1: Remap through this view's own palette
        if let Some(palette) = self.get_palette() {
            if !palette.is_empty() {
                if color as usize > palette.len() {
                    return Attr::from_u8(ERROR_ATTR);
                }
                color = palette.get(color as usize);
                if color == 0 {
                    return Attr::from_u8(ERROR_ATTR);
                }
            }
        }

        // Step 2: Walk up the owner chain via QCell-based palette chain.
        // Matches Borland: TView::mapColor() traverses owner->getPalette() up to
        // TApplication. Views without a palette (get_palette returns None) are
        // transparent. The chain stops when there's no parent.
        if let Some(chain_node) = self.get_palette_chain() {
            color = chain_node.remap_color(color);
            if color == 0 {
                return Attr::from_u8(ERROR_ATTR);
            }
        }
        // Views without a palette chain (top-level views like MenuBar, StatusLine)
        // skip the chain walk and go directly to the app palette.

        // Step 3: Resolve through application palette (1-indexed)
        let app_palette_data = palettes::get_app_palette();
        let app_index = (color as usize).wrapping_sub(1);
        if app_index < app_palette_data.len() {
            let final_color = app_palette_data[app_index];
            if final_color == 0 {
                return Attr::from_u8(ERROR_ATTR);
            }
            Attr::from_u8(final_color)
        } else {
            Attr::from_u8(ERROR_ATTR)
        }
    }
}

/// Trait for views that need idle processing (animations, timers, etc.)
/// These views have their idle() method called periodically even during modal dialogs,
/// matching Borland's TProgram::idle() behavior which continues running during execView().
///
/// # Examples
///
/// ```ignore
/// use turbo_vision::views::{View, IdleView};
/// use turbo_vision::terminal::Terminal;
/// use std::time::Instant;
///
/// struct AnimatedWidget {
///     position: usize,
///     last_update: Instant,
///     // ... other View fields
/// }
///
/// impl IdleView for AnimatedWidget {
///     fn idle(&mut self) {
///         if self.last_update.elapsed().as_millis() > 100 {
///             self.position = (self.position + 1) % 10;
///             self.last_update = Instant::now();
///         }
///     }
/// }
/// ```
pub trait IdleView: View {
    /// Called periodically to update animation state, timers, etc.
    /// Matches Borland: TProgram::idle() continues running even during modal dialogs
    fn idle(&mut self);
}

/// Helper to draw a line to the terminal
pub fn write_line_to_terminal(terminal: &mut Terminal, x: i16, y: i16, buf: &DrawBuffer) {
    if y < 0 || y >= terminal.size().1 {
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

    let ss = shadow_size();
    let mut buf = DrawBuffer::new(ss.0 as usize);

    // Draw right edge shadow (ss.0 columns wide, offset by ss.1 vertically)
    // Read existing cells and darken them for semi-transparency
    for y in (bounds.a.y + ss.1)..(bounds.b.y + ss.1) {
        for i in 0..ss.0 {
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

    // Draw bottom edge shadow (offset by ss.0 horizontally, excludes right shadow area to prevent double-darkening)
    let bottom_width = (bounds.b.x - bounds.a.x - ss.0) as usize;
    let mut bottom_buf = DrawBuffer::new(bottom_width);

    let shadow_y = bounds.b.y;
    for i in 0..bottom_width {
        let x = bounds.a.x + ss.0 + i as i16;

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
    write_line_to_terminal(terminal, bounds.a.x + ss.0, bounds.b.y, &bottom_buf);
}

/// A boxed view is itself a view, so `GroupLike::add` can take boxed and
/// unboxed views alike and older `add(Box::new(v))` calls keep compiling.
macro_rules! forward_view_through_box {
    ($t:ty) => {
        impl<T: View + ?Sized> View for $t {
            fn core(&self) -> &ViewCore {
                (**self).core()
            }
            fn core_mut(&mut self) -> &mut ViewCore {
                (**self).core_mut()
            }
            fn bounds(&self) -> Rect {
                (**self).bounds()
            }
            fn set_bounds(&mut self, bounds: Rect) {
                (**self).set_bounds(bounds)
            }
            fn draw(&mut self, terminal: &mut Terminal) {
                (**self).draw(terminal)
            }
            fn handle_event(&mut self, event: &mut Event) {
                (**self).handle_event(event)
            }
            fn can_focus(&self) -> bool {
                (**self).can_focus()
            }
            fn set_focus(&mut self, focused: bool) {
                (**self).set_focus(focused)
            }
            fn window_number(&self) -> Option<u8> {
                (**self).window_number()
            }
            fn options(&self) -> u16 {
                (**self).options()
            }
            fn set_options(&mut self, options: u16) {
                (**self).set_options(options)
            }
            fn state(&self) -> StateFlags {
                (**self).state()
            }
            fn set_state(&mut self, state: StateFlags) {
                (**self).set_state(state)
            }
            fn grow_mode(&self) -> crate::core::state::GrowFlags {
                (**self).grow_mode()
            }
            fn set_grow_mode(&mut self, grow_mode: crate::core::state::GrowFlags) {
                (**self).set_grow_mode(grow_mode)
            }
            fn update_cursor(&self, terminal: &mut Terminal) {
                (**self).update_cursor(terminal)
            }
            fn zoom(&mut self, max_bounds: Rect) {
                (**self).zoom(max_bounds)
            }
            fn valid(&mut self, command: crate::core::command::CommandId) -> bool {
                (**self).valid(command)
            }
            fn as_any(&self) -> &dyn std::any::Any {
                (**self).as_any()
            }
            fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
                (**self).as_any_mut()
            }
            fn as_group(&self) -> Option<&dyn crate::views::group::GroupLike> {
                (**self).as_group()
            }
            fn as_group_mut(&mut self) -> Option<&mut dyn crate::views::group::GroupLike> {
                (**self).as_group_mut()
            }
            fn get_redraw_union(&self) -> Option<Rect> {
                (**self).get_redraw_union()
            }
            fn clear_move_tracking(&mut self) {
                (**self).clear_move_tracking()
            }
            fn label_link(&self) -> Option<ViewId> {
                (**self).label_link()
            }
            fn init_after_add(&mut self) {
                (**self).init_after_add()
            }
            fn constrain_to_parent_bounds(&mut self) {
                (**self).constrain_to_parent_bounds()
            }
            fn set_palette_chain(
                &mut self,
                node: Option<crate::core::palette_chain::PaletteChainNode>,
            ) {
                (**self).set_palette_chain(node)
            }
            fn get_palette_chain(&self) -> Option<&crate::core::palette_chain::PaletteChainNode> {
                (**self).get_palette_chain()
            }
            fn set_parent_bounds(&mut self, bounds: Rect) {
                (**self).set_parent_bounds(bounds)
            }
            fn get_palette(&self) -> Option<crate::core::palette::Palette> {
                (**self).get_palette()
            }
        }
    };
}

forward_view_through_box!(Box<T>);

impl<T: IdleView + ?Sized> IdleView for Box<T> {
    fn idle(&mut self) {
        (**self).idle()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_view_in_a_group_can_be_downcast_without_panicking() {
        use crate::views::button::Button;
        use crate::views::group::{Group, GroupLike};
        use crate::views::static_text::StaticText;
        let mut g = Group::new(Rect::new(0, 0, 40, 10));
        g.add(Button::new(Rect::new(0, 0, 10, 2), "ok", 1, true));
        g.add(StaticText::new(Rect::new(0, 3, 10, 4), "hi"));
        for i in 0..g.len() {
            let _ = g.child_at(i).as_any(); // would have panicked with the old default
        }
        assert!(g.child_at(0).as_any().downcast_ref::<Button>().is_some());
    }

    #[test]
    fn accessors_read_and_write_the_core() {
        struct Probe(ViewCore);
        impl View for Probe {
            fn core(&self) -> &ViewCore {
                &self.0
            }
            fn core_mut(&mut self) -> &mut ViewCore {
                &mut self.0
            }
            fn draw(&mut self, _t: &mut Terminal) {}
            fn handle_event(&mut self, _e: &mut Event) {}
            fn get_palette(&self) -> Option<crate::core::palette::Palette> {
                None
            }

            fn as_any(&self) -> &dyn std::any::Any {
                self
            }

            fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
                self
            }
        }
        let mut p = Probe(ViewCore::new(Rect::new(1, 2, 3, 4)));
        assert_eq!(p.bounds(), Rect::new(1, 2, 3, 4));
        p.set_state(SF_FOCUSED);
        assert!(p.is_focused());
        p.set_options(0x0004);
        assert_eq!(p.options(), 0x0004);
        p.set_bounds(Rect::new(0, 0, 8, 8));
        assert_eq!(p.core().bounds, Rect::new(0, 0, 8, 8));
    }
}
