// (C) 2025 - Enzo Lombardi

//! Window view - draggable, resizable window with frame and shadow.

use super::frame::Frame;
use super::group::{Group, GroupLike};
use super::view::{View, ViewCore};
use crate::core::command::{CM_CANCEL, CM_CLOSE};
use crate::core::event::{Event, EventType};
use crate::core::geometry::{Point, Rect};
use crate::core::state::{SF_DRAGGING, SF_MODAL, SF_RESIZING, SF_SHADOW, shadow_size};
use crate::terminal::Terminal;

pub struct Window {
    core: ViewCore,
    frame: Frame,
    interior: Group,
    /// Direct children of window (positioned relative to window frame, not interior)
    /// Used for scrollbars and other frame-relative elements
    frame_children: Vec<Box<dyn View>>,
    /// Drag start position (relative to mouse when drag started)
    drag_offset: Option<Point>,
    /// Resize start size (size when resize drag started)
    resize_start_size: Option<Point>,
    /// Minimum window size (matches Borland's minWinSize)
    min_size: Point,
    /// Window number for Alt+1..9 selection (Borland: TWindow::number;
    /// None = wnNoNumber)
    number: Option<u8>,
    /// Saved bounds while keyboard move/resize mode is active (Borland:
    /// cmResize -> dragView; Esc restores these bounds)
    keyboard_resize_saved: Option<Rect>,
    /// Saved bounds for zoom/restore (matches Borland's zoomRect)
    zoom_rect: Rect,
    /// Previous bounds (for calculating union rect for redrawing)
    /// Matches Borland: TView::locate() calculates union of old and new bounds
    prev_bounds: Option<Rect>,
    /// Palette type (Dialog vs EditorWindow window)
    palette_type: WindowPaletteType,
    /// Custom palette override — applied to both Window and Frame.
    custom_palette: Option<Vec<u8>>,
    /// Explicit drag limits (for modal dialogs not added to desktop)
    /// Used when owner is None but we still want to constrain dragging
    explicit_drag_limits: Option<Rect>,
    /// When true (default), a non-modal `CM_CLOSE` makes the window mark
    /// itself `SF_CLOSED` + clear the event, so the next
    /// `Desktop::remove_closed_windows()` sweep removes it. Set to `false`
    /// for windows whose owner needs to intercept the close (e.g. an editor
    /// that wants to prompt "save changes?" first); in that case `CM_CLOSE`
    /// bubbles up uncleared and the owner is responsible for both the
    /// validation and the eventual `set_state(SF_CLOSED)`.
    auto_close: bool,
}

#[derive(Clone, Copy)]
pub enum WindowPaletteType {
    Blue,   // Uses CP_BLUE_WINDOW
    Cyan,   // Uses CP_CYAN_WINDOW
    Gray,   // Uses CP_GRAY_WINDOW
    Dialog, // Uses CP_GRAY_DIALOG
}

impl Window {
    /// The window's own `ViewCore`; `impl_view_for_window!` routes
    /// `View::core` here for every window-shaped type.
    pub fn view_core(&self) -> &ViewCore {
        &self.core
    }

    pub fn view_core_mut(&mut self) -> &mut ViewCore {
        &mut self.core
    }

    /// Create a new TWindow with blue palette (default Borland TWindow behavior)
    /// Matches Borland: TWindow constructor sets palette(wpBlueWindow)
    /// For TDialog (gray palette), use new_for_dialog() instead
    pub fn new(bounds: Rect, title: &str) -> Self {
        Self::new_with_palette(
            bounds,
            title,
            super::frame::FramePaletteType::EditorWindow,
            WindowPaletteType::Blue,
            true, // resizable
        )
    }

    /// Create a window for TDialog with gray palette
    /// Matches Borland: TDialog overrides TWindow palette to use cpGrayDialog
    pub(crate) fn new_for_dialog(bounds: Rect, title: &str) -> Self {
        Self::new_with_palette(
            bounds,
            title,
            super::frame::FramePaletteType::Dialog,
            WindowPaletteType::Dialog,
            false, // not resizable (TDialog doesn't have wfGrow)
        )
    }

    /// Create a window for THelpWindow with cyan palette
    /// Matches Borland: THelpWindow uses cyan help window palette (cHelpWindow)
    pub fn new_for_help(bounds: Rect, title: &str) -> Self {
        Self::new_with_palette(
            bounds,
            title,
            super::frame::FramePaletteType::HelpWindow,
            WindowPaletteType::Cyan,
            true, // help windows are resizable
        )
    }

    /// Create a window with a specific palette type.
    /// This allows users to create Gray, Cyan, or Blue windows without
    /// being constrained to the preset constructors.
    pub fn new_with_type(bounds: Rect, title: &str, palette_type: WindowPaletteType) -> Self {
        let (frame_palette, resizable) = match palette_type {
            WindowPaletteType::Blue => (super::frame::FramePaletteType::EditorWindow, true),
            WindowPaletteType::Cyan => (super::frame::FramePaletteType::HelpWindow, true),
            WindowPaletteType::Gray => (super::frame::FramePaletteType::Dialog, true),
            WindowPaletteType::Dialog => (super::frame::FramePaletteType::Dialog, false),
        };
        Self::new_with_palette(bounds, title, frame_palette, palette_type, resizable)
    }

    fn new_with_palette(
        bounds: Rect,
        title: &str,
        frame_palette: super::frame::FramePaletteType,
        window_palette: WindowPaletteType,
        resizable: bool,
    ) -> Self {
        use crate::core::state::{OF_SELECTABLE, OF_TILEABLE, OF_TOP_SELECT};

        let frame = Frame::with_palette(bounds, title, frame_palette, resizable);

        // Interior bounds are ABSOLUTE (inset by 1 from window bounds for frame)
        let mut interior_bounds = bounds;
        interior_bounds.grow(-1, -1);
        // Don't use background - the Frame fills the interior space (matching Borland)
        let interior = Group::new(interior_bounds);

        let window = Self {
            core: ViewCore {
                bounds,
                state: SF_SHADOW, // Windows have shadows by default
                options: OF_SELECTABLE | OF_TOP_SELECT | OF_TILEABLE, // Matches Borland: TWindow/TEditWindow flags
                palette_chain: None,
                // Grow mode flags (Borland: `TWindow::growMode`), controlling how this
                // window's bounds move when its owner (the Desktop) is resized.
                //
                // Borland's `TWindow` constructor sets `growMode = gfGrowAll`, but this
                // crate's resize cascade (`Group::set_bounds`) gives `gfGrowAll`
                // (all four `GF_GROW_*` bits) a literal "translate by the full size
                // delta, keep the same size" meaning — see the `gfGrowAll` case in
                // `Group`'s own `test_grow_modes_on_resize` — which is right for a
                // widget pinned to the far corner (e.g. a resize handle) but does
                // nothing to fix a full-size window being clipped at the new screen
                // edge; it would just slide the window away from the corner it was
                // already filling. The default here is deliberately
                // `GF_GROW_HI_X | GF_GROW_HI_Y` instead: the window's top-left corner
                // stays put and its bottom-right edge follows the desktop's growth,
                // i.e. the window actually stretches to fill the new space, which is
                // the resizing behaviour the bug report asked for. Use
                // `set_grow_mode()` to opt out (e.g. `0` for a fixed window, or
                // `GF_GROW_ALL` for corner-tracking).
                grow_mode: crate::core::state::GF_GROW_HI_X | crate::core::state::GF_GROW_HI_Y,
            },
            frame,
            interior,
            frame_children: Vec::new(),
            drag_offset: None,
            resize_start_size: None,
            min_size: Point::new(16, 6),
            number: None,
            keyboard_resize_saved: None, // Minimum size: 16 wide, 6 tall (matches Borland's minWinSize)
            zoom_rect: bounds,           // Initialize to current bounds
            prev_bounds: None,
            palette_type: window_palette,
            custom_palette: None,
            explicit_drag_limits: None,
            auto_close: true,
        };

        window
    }

    /// Set a custom palette override for this window.
    /// The palette maps logical color indices (1-8 for windows, 1-32 for dialogs)
    /// to app palette positions. The Frame and all children inherit this palette
    /// through the owner chain — no separate Frame palette needed.
    pub fn set_custom_palette(&mut self, palette: Vec<u8>) {
        self.custom_palette = Some(palette);
    }

    /// Add a child positioned relative to the window frame (not interior)
    /// Used for scrollbars and other frame-edge elements
    /// Matches Borland: TWindow is a TGroup, all children use window-relative coords
    pub fn add_frame_child(&mut self, mut view: Box<dyn View>) -> usize {
        // Convert from relative to absolute coordinates (relative to window frame)
        // Palette chain is set up during draw
        let child_bounds = view.bounds();
        let absolute_bounds = Rect::new(
            self.core.bounds.a.x + child_bounds.a.x,
            self.core.bounds.a.y + child_bounds.a.y,
            self.core.bounds.a.x + child_bounds.b.x,
            self.core.bounds.a.y + child_bounds.b.y,
        );
        view.set_bounds(absolute_bounds);

        self.frame_children.push(view);
        self.frame_children.len() - 1
    }

    /// Update a frame child's bounds (for use by subclasses during resize)
    pub fn update_frame_child(&mut self, index: usize, bounds: Rect) {
        if let Some(child) = self.frame_children.get_mut(index) {
            child.set_bounds(bounds);
        }
    }

    /// Get mutable access to a frame child by index (for conditional drawing)
    pub fn get_frame_child_mut(&mut self, index: usize) -> Option<&mut Box<dyn View>> {
        self.frame_children.get_mut(index)
    }

    /// Get access to the frame (for subclasses to draw manually)
    pub(crate) fn frame_mut(&mut self) -> &mut Frame {
        &mut self.frame
    }

    /// Get access to the interior (for subclasses to draw manually)
    pub(crate) fn interior_mut(&mut self) -> &mut Group {
        &mut self.interior
    }

    /// Set the window title
    /// Matches Borland: TWindow allows title mutation via setTitle()
    /// The frame will be redrawn on the next draw() call
    pub fn set_title(&mut self, title: &str) {
        self.frame.set_title(title);
    }

    /// Set whether the window is resizable.
    /// Resizable windows show single-line bottom corners and a resize handle.
    /// Show or hide the frame's zoom icon (Borland: wfZoom).
    pub fn set_zoomable(&mut self, zoomable: bool) {
        self.frame.set_zoomable(zoomable);
    }

    pub fn set_resizable(&mut self, resizable: bool) {
        self.frame.set_resizable(resizable);
    }

    /// Control whether the window self-closes on a non-modal `CM_CLOSE`.
    ///
    /// Default is `true`: clicking the frame's close button marks the window
    /// `SF_CLOSED` and clears the event, so the next
    /// [`Desktop::remove_closed_windows`] sweep removes it. Mirrors Borland's
    /// `TWindow::close()` flow with a trivial `valid()` (auto-accept).
    ///
    /// Set to `false` for windows whose owner needs to intercept the close —
    /// e.g. an editor that prompts "save changes?" before destroying the
    /// buffer. With auto-close off, `CM_CLOSE` bubbles up uncleared and the
    /// owner is responsible for both validation and the eventual
    /// `set_state(SF_CLOSED)`. Modal windows ignore this flag (they always
    /// `end_modal(CM_CANCEL)` on `CM_CLOSE`).
    pub fn set_auto_close(&mut self, auto_close: bool) {
        self.auto_close = auto_close;
    }

    /// Set minimum window size (matches Borland: minWinSize)
    /// Prevents window from being resized smaller than these dimensions
    pub fn set_min_size(&mut self, min_size: Point) {
        self.min_size = min_size;
    }

    /// Get size limits for this window
    /// Matches Borland: TWindow::sizeLimits(TPoint &min, TPoint &max)
    /// Returns (min, max) where max is typically the desktop size
    pub fn size_limits(&self) -> (Point, Point) {
        // Max size would typically be the desktop/owner size
        // For now, return a large max (similar to Borland's INT_MAX approach)
        let max = Point::new(999, 999);
        (self.min_size, max)
    }

    /// Get drag limits from parent bounds or explicit limits
    /// Matches Borland: TFrame::dragWindow() gets limits = owner->owner->getExtent()
    /// Returns parent bounds if set, otherwise unrestricted
    fn get_drag_limits(&self) -> Rect {
        if let Some(limits) = self.explicit_drag_limits {
            limits
        } else {
            // No parent bounds set - unrestricted movement
            Rect::new(-999, -999, 9999, 9999)
        }
    }

    /// Set explicit drag limits (for modal dialogs not added to desktop)
    /// This is used when a dialog runs its own event loop without being added to desktop
    pub fn set_drag_limits(&mut self, limits: Rect) {
        self.explicit_drag_limits = Some(limits);
    }

    /// Constrain window bounds to drag limits
    /// Ensures window is positioned within parent bounds (including shadow)
    /// Matches Borland: TView position is constrained during locate()
    pub fn constrain_to_limits(&mut self) {
        let limits = self.get_drag_limits();
        let width = self.core.bounds.width();
        let height = self.core.bounds.height();

        // Account for shadow when constraining edges
        let (shadow_x, shadow_y) = if (self.core.state & SF_SHADOW) != 0 {
            shadow_size()
        } else {
            (0, 0)
        };

        let mut new_x = self.core.bounds.a.x;
        let mut new_y = self.core.bounds.a.y;

        // Apply all drag mode constraints
        // dmLimitLoX: keep left edge within bounds
        new_x = new_x.max(limits.a.x);

        // dmLimitLoY: keep top edge within bounds
        new_y = new_y.max(limits.a.y);

        // dmLimitHiX: keep right edge (including shadow) within bounds
        new_x = new_x.min(limits.b.x - width - shadow_x);

        // dmLimitHiY: keep bottom edge (including shadow) within bounds
        new_y = new_y.min(limits.b.y - height - shadow_y);

        // Update bounds if position changed
        if new_x != self.core.bounds.a.x || new_y != self.core.bounds.a.y {
            self.core.bounds = Rect::new(new_x, new_y, new_x + width, new_y + height);

            // Update frame and interior bounds
            self.frame.set_bounds(self.core.bounds);
            let mut interior_bounds = self.core.bounds;
            interior_bounds.grow(-1, -1);
            self.interior.set_bounds(interior_bounds);
        }
    }

    /// Set the maximum size for zoom operations
    /// Typically set to desktop size when added to desktop
    pub fn set_max_size(&mut self, _max_size: Point) {
        // Store max size as zoom_rect if we want to zoom to it
        // For now, we'll calculate it dynamically in zoom()
    }

    /// Set focus to a specific child by index
    /// Matches Borland: owner->setCurrent(this, normalSelect)
    pub fn set_focus_to_child(&mut self, index: usize) {
        // Clear focus from all children first
        self.interior.clear_all_focus();
        // Set focus to the specified child (updates both focused index and focus state)
        self.interior.set_focus_to(index);
    }

    /// Get the union rect of current and previous bounds (for redrawing)
    /// Matches Borland: TView::locate() calculates union rect
    /// Returns None if window hasn't moved yet
    pub fn get_redraw_union(&self) -> Option<Rect> {
        self.prev_bounds.map(|prev| {
            // Union of old and new bounds, including shadows
            let mut union = prev.union(&self.core.bounds);

            // Expand by shadow_size on right and bottom for shadow
            // Matches Borland: TView::shadowSize
            let ss = shadow_size();
            union.b.x += ss.0;
            union.b.y += ss.1;

            union
        })
    }

    /// Clear the movement tracking (call after redraw)
    pub fn clear_move_tracking(&mut self) {
        self.prev_bounds = None;
    }

    /// End the modal event loop
    /// Delegates to the interior Group's end_modal() method
    /// Set the window number shown in the frame and used by Alt+1..9
    /// selection (Borland: TWindow::number).
    pub fn set_number(&mut self, number: u8) {
        self.number = Some(number);
        self.frame.set_number(Some(number));
    }

    /// Get the window number, if assigned.
    pub fn number(&self) -> Option<u8> {
        self.number
    }

    /// Initialize the interior's owner pointer after Window is in its final memory location.
    /// Must be called after any operation that moves the Window (adding to parent, etc.)
    /// This ensures the interior Group has a valid pointer to this Window.
    pub fn init_interior_owner(&mut self) {
        // NOTE: We don't set interior's owner pointer to avoid unsafe casting
        // Color palette resolution is handled without needing parent pointers
    }
}

/// The behaviour of Borland's `TWindow`, expressed as default methods over a
/// `Window` core. A window-shaped type (`Dialog`, `EditWindow`, a downstream
/// custom window) implements the two accessors here plus `GroupLike`, and
/// gets its `View` implementation from [`impl_view_for_window!`], which
/// forwards every `View` method to the `window_*` body below unless the type
/// supplies its own override inline.
///
/// The `window_*` names are the inherited implementations, callable as base
/// calls from an override: `TDialog::handleEvent` starts with
/// `TWindow::handleEvent(event)`, and a `Dialog` override starts with
/// `self.window_handle_event(event)`. Inside these bodies, `self.get_palette()`,
/// `self.valid(..)` and the other `View` hooks dispatch to the outer type, so
/// an override is seen by the base drawing and event code.
pub trait WindowLike: GroupLike {
    fn window(&self) -> &Window;
    fn window_mut(&mut self) -> &mut Window;

    fn window_set_bounds(&mut self, bounds: Rect) {
        self.core_mut().bounds = bounds;
        self.window_mut().frame.set_bounds(bounds);

        // Update interior bounds (absolute, inset by 1 for frame)
        let mut interior_bounds = bounds;
        interior_bounds.grow(-1, -1);
        self.window_mut().interior.set_bounds(interior_bounds);

        // NOTE: We do NOT automatically update frame_children here
        // Subclasses like EditWindow handle frame_children positioning manually
        // because scrollbars need to be repositioned based on new window SIZE, not just offset
    }

    fn window_draw(&mut self, terminal: &mut Terminal) {
        // Build Window's palette chain node for safe palette traversal.
        // Window is a palette-bearing node (CP_BLUE_WINDOW, CP_GRAY_DIALOG, etc.)
        let my_chain_node = crate::core::palette_chain::PaletteChainNode::new(
            self.get_palette(),
            self.get_palette_chain().cloned(),
        );

        self.window_mut()
            .frame
            .set_palette_chain(Some(my_chain_node.clone()));
        self.window_mut().frame.draw(terminal);

        self.window_mut()
            .interior
            .set_palette_chain(Some(my_chain_node.clone()));
        self.window_mut().interior.draw(terminal);

        // Draw frame children (scrollbars, etc.) after interior so they appear on top
        for child in &mut self.window_mut().frame_children {
            child.set_palette_chain(Some(my_chain_node.clone()));
            child.draw(terminal);
        }

        // Draw shadow if enabled
        if self.has_shadow() {
            self.draw_shadow(terminal);
        }
    }

    fn window_update_cursor(&self, terminal: &mut Terminal) {
        // Propagate cursor update to interior group
        self.window().interior.update_cursor(terminal);
    }

    fn window_handle_event(&mut self, event: &mut Event) {
        // Keyboard move/resize mode (Borland: cmResize enters dragView with
        // dmDragMove|dmDragGrow; arrows move, Shift+arrows resize, Enter
        // confirms, Esc restores the saved bounds)
        if event.what == EventType::Command
            && event.command == crate::core::command::CM_RESIZE
            && (self.state() & crate::core::state::SF_ACTIVE) != 0
        {
            self.window_mut().keyboard_resize_saved = Some(self.bounds());
            event.clear();
            return;
        }
        if let Some(saved) = self.window_mut().keyboard_resize_saved {
            if event.what == EventType::Keyboard {
                use crate::core::event::{
                    KB_DOWN, KB_ENTER, KB_ESC, KB_ESC_ESC, KB_LEFT, KB_RIGHT, KB_UP,
                };
                let shift = event
                    .key_modifiers
                    .contains(crossterm::event::KeyModifiers::SHIFT);
                let (mut dx, mut dy) = (0i16, 0i16);
                match event.key_code {
                    KB_LEFT => dx = -1,
                    KB_RIGHT => dx = 1,
                    KB_UP => dy = -1,
                    KB_DOWN => dy = 1,
                    KB_ENTER => {
                        self.window_mut().keyboard_resize_saved = None;
                        event.clear();
                        return;
                    }
                    KB_ESC | KB_ESC_ESC => {
                        self.set_bounds(saved);
                        self.window_mut().keyboard_resize_saved = None;
                        event.clear();
                        return;
                    }
                    _ => return, // swallow nothing else; stay in mode
                }
                let mut b = self.bounds();
                if shift {
                    // Resize the bottom-right corner, respecting min size
                    b.b.x = (b.b.x + dx).max(b.a.x + self.window_mut().min_size.x);
                    b.b.y = (b.b.y + dy).max(b.a.y + self.window_mut().min_size.y);
                } else {
                    b.a.x += dx;
                    b.a.y += dy;
                    b.b.x += dx;
                    b.b.y += dy;
                }
                self.set_bounds(b);
                event.clear();
                return;
            }
        }

        // First, let the frame handle the event (for close button clicks, drag start, etc.)
        self.window_mut().frame.handle_event(event);

        // Check if frame started dragging or resizing
        let frame_dragging = (self.window_mut().frame.state() & SF_DRAGGING) != 0;
        let frame_resizing = (self.window_mut().frame.state() & SF_RESIZING) != 0;

        if frame_dragging && self.window_mut().drag_offset.is_none() {
            // Frame just started dragging - record offset
            if event.what == EventType::MouseDown || event.what == EventType::MouseMove {
                let mouse_pos = event.mouse.pos;
                self.window_mut().drag_offset = Some(Point::new(
                    mouse_pos.x - self.bounds().a.x,
                    mouse_pos.y - self.bounds().a.y,
                ));
                self.set_state_flag(SF_DRAGGING, true);
                event.clear(); // Mark event as handled
                return;
            }
        }

        if frame_resizing && self.window_mut().resize_start_size.is_none() {
            // Frame just started resizing - record initial size
            if event.what == EventType::MouseDown || event.what == EventType::MouseMove {
                let mouse_pos = event.mouse.pos;
                // Calculate offset from bottom-right corner
                // Borland: p = size - event.mouse.where (tview.cc:235)
                self.window_mut().resize_start_size = Some(Point::new(
                    self.bounds().b.x - mouse_pos.x,
                    self.bounds().b.y - mouse_pos.y,
                ));
                self.set_state_flag(SF_RESIZING, true);
                event.clear(); // Mark event as handled
                return;
            }
        }

        // Handle mouse move during drag
        if frame_dragging && self.window_mut().drag_offset.is_some() {
            if event.what == EventType::MouseMove {
                let mouse_pos = event.mouse.pos;
                let offset = self.window_mut().drag_offset.unwrap();

                // Calculate new position
                let mut new_x = mouse_pos.x - offset.x;
                let mut new_y = mouse_pos.y - offset.y;

                // Get drag limits from owner (parent bounds)
                // Matches Borland: TView::moveGrow() constrains position to limits
                let limits = self.window_mut().get_drag_limits();
                let width = self.bounds().width();
                let height = self.bounds().height();

                // Account for shadow when constraining edges
                let (shadow_x, shadow_y) = if (self.state() & SF_SHADOW) != 0 {
                    shadow_size()
                } else {
                    (0, 0)
                };

                // Apply drag constraints to keep window fully within parent bounds
                // Matches Borland: dmLimitLoX | dmLimitLoY | dmLimitHiX | dmLimitHiY (full containment)

                // dmLimitLoX: keep left edge within bounds (prevent negative x)
                new_x = new_x.max(limits.a.x);

                // dmLimitLoY: keep top edge within bounds (prevent negative y)
                new_y = new_y.max(limits.a.y);

                // dmLimitHiX: keep right edge (including shadow) within bounds
                new_x = new_x.min(limits.b.x - width - shadow_x);

                // dmLimitHiY: keep bottom edge (including shadow) within bounds
                new_y = new_y.min(limits.b.y - height - shadow_y);

                // Save previous bounds for union rect calculation (Borland's locate pattern)
                self.window_mut().prev_bounds = Some(self.bounds());

                // Update bounds (maintaining size)
                self.core_mut().bounds = Rect::new(new_x, new_y, new_x + width, new_y + height);

                // Update frame and interior bounds
                let bounds = self.bounds();
                self.window_mut().frame.set_bounds(bounds);
                let mut interior_bounds = self.bounds();
                interior_bounds.grow(-1, -1);
                self.window_mut().interior.set_bounds(interior_bounds);

                event.clear(); // Mark event as handled
                return;
            }
        }

        // Handle mouse move during resize
        if frame_resizing && self.window_mut().resize_start_size.is_some() {
            if event.what == EventType::MouseMove {
                let mouse_pos = event.mouse.pos;
                let offset = self.window_mut().resize_start_size.unwrap();

                // Calculate new size (Borland: event.mouse.where += p, then use as size)
                // Ensure positive before casting to u16 to avoid wraparound
                let new_width = (mouse_pos.x + offset.x - self.bounds().a.x).max(0) as u16;
                let new_height = (mouse_pos.y + offset.y - self.bounds().a.y).max(0) as u16;

                // Apply size constraints (Borland: sizeLimits)
                let (min, max) = self.window_mut().size_limits();
                let mut final_width = new_width.max(min.x as u16).min(max.x as u16);
                let mut final_height = new_height.max(min.y as u16).min(max.y as u16);

                // Constrain size to not exceed parent bounds
                // Borland: TView::moveGrow() constrains both position and size to limits
                let limits = self.window_mut().get_drag_limits();
                let max_width = (limits.b.x - self.bounds().a.x).max(0) as u16;
                let max_height = (limits.b.y - self.bounds().a.y).max(0) as u16;
                final_width = final_width.min(max_width);
                final_height = final_height.min(max_height);

                // Save previous bounds for union rect calculation
                self.window_mut().prev_bounds = Some(self.bounds());

                // Update bounds (maintaining position, changing size)
                self.bounds().b.x = self.bounds().a.x + final_width as i16;
                self.bounds().b.y = self.bounds().a.y + final_height as i16;

                // Update frame and interior bounds
                let bounds = self.bounds();
                self.window_mut().frame.set_bounds(bounds);
                let mut interior_bounds = self.bounds();
                interior_bounds.grow(-1, -1);
                self.window_mut().interior.set_bounds(interior_bounds);

                event.clear(); // Mark event as handled
                return;
            }
        }

        // Check if frame ended dragging
        if !frame_dragging && self.window_mut().drag_offset.is_some() {
            self.window_mut().drag_offset = None;
            self.set_state_flag(SF_DRAGGING, false);
        }

        // Check if frame ended resizing
        if !frame_resizing && self.window_mut().resize_start_size.is_some() {
            self.window_mut().resize_start_size = None;
            self.set_state_flag(SF_RESIZING, false);
        }

        // Handle ESC key for modal windows
        // Modal windows should close when ESC or ESC ESC is pressed
        if event.what == EventType::Keyboard {
            let is_esc = event.key_code == crate::core::event::KB_ESC;
            let is_esc_esc = event.key_code == crate::core::event::KB_ESC_ESC;

            if (is_esc || is_esc_esc) && (self.state() & SF_MODAL) != 0 {
                // Modal window: ESC ends the modal loop with CM_CANCEL
                self.end_modal(CM_CANCEL);
                event.clear();
                return;
            }
        }

        // Handle CM_CLOSE command (Borland: twindow.cc TWindow::handleEvent ~118-132)
        // Frame generates CM_CLOSE on close-button MouseUp.
        if event.what == EventType::Command && event.command == CM_CLOSE {
            if (self.state() & SF_MODAL) != 0 {
                // Modal: end_modal with CM_CANCEL (Borland converts cmClose → cmCancel)
                self.end_modal(CM_CANCEL);
                event.clear();
            } else if self.window_mut().auto_close {
                // Non-modal default: self-close. Mirrors Borland's
                // TWindow::close(): `if (valid(cmClose)) destroy(this)` — the
                // valid() hook gives children (editors, dialogs) a chance to
                // veto the close ("save changes?"). The event is cleared
                // either way (Borland clears it before calling close()); only
                // SF_CLOSED is gated on validation.
                use crate::core::state::SF_CLOSED;
                if self.valid(CM_CLOSE) {
                    self.set_state_flag(SF_CLOSED, true);
                }
                event.clear();
            } else {
                // Owner opted out of auto-close (set_auto_close(false)) — used
                // by editors that need to prompt "save changes?" before
                // destruction. Leave event uncleared so it bubbles up; owner
                // handles validation and eventual SF_CLOSED.
            }
            return; // Don't pass CM_CLOSE to interior
        }

        // Then let the interior handle it (if not already handled)
        self.window_mut().interior.handle_event(event);
    }

    fn window_set_focus(&mut self, focused: bool) {
        // Mirror Borland: TWindow::setState(sfSelected) forwards sfActive to
        // the window and its frame, so inactive windows draw with the
        // inactive frame palette (see Frame::get_frame_colors).
        use crate::core::state::SF_ACTIVE;
        self.set_state_flag(SF_ACTIVE, focused);
        self.window_mut().frame.set_state_flag(SF_ACTIVE, focused);

        // Propagate focus to the interior group
        // When the window gets focus, set focus on its first focusable child
        if focused {
            self.window_mut().interior.set_initial_focus();
        } else {
            self.window_mut().interior.clear_all_focus();
        }
    }

    /// Zoom (maximize) or restore window
    /// Matches Borland: TWindow::zoom() toggles between current size and maximum size
    /// In Borland, this is called by owner in response to cmZoom command
    fn window_zoom(&mut self, max_bounds: Rect) {
        let (_min, _max_size) = self.window_mut().size_limits();
        let current_size = Point::new(self.bounds().width(), self.bounds().height());

        // If not at max size, zoom to max
        if current_size.x != max_bounds.width() || current_size.y != max_bounds.height() {
            // Save current bounds for restore
            self.window_mut().zoom_rect = self.bounds();

            // Save previous bounds for redraw union
            self.window_mut().prev_bounds = Some(self.bounds());

            // Zoom to max size (typically desktop bounds)
            self.core_mut().bounds = max_bounds;
        } else {
            // Restore to saved bounds
            self.window_mut().prev_bounds = Some(self.bounds());
            self.core_mut().bounds = self.window_mut().zoom_rect;
        }

        // Update frame and interior
        let bounds = self.bounds();
        self.window_mut().frame.set_bounds(bounds);
        // The frame draws a different zoom glyph once the window is zoomed:
        // an up arrow while it can still grow, both ways once it can only be
        // restored.
        self.window_mut().frame.set_zoomed(bounds == max_bounds);
        let mut interior_bounds = self.bounds();
        interior_bounds.grow(-1, -1);
        self.window_mut().interior.set_bounds(interior_bounds);
    }

    /// Validate window before closing with given command
    /// Matches Borland: TWindow inherits TGroup::valid() which validates all children
    /// Delegates to interior group to validate all children
    fn window_valid(&mut self, command: crate::core::command::CommandId) -> bool {
        self.window_mut().interior.valid(command)
    }

    fn window_get_palette(&self) -> Option<crate::core::palette::Palette> {
        use crate::core::palette::{Palette, palettes};
        if let Some(ref custom) = self.window().custom_palette {
            return Some(Palette::from_slice(custom));
        }
        match self.window().palette_type {
            WindowPaletteType::Blue => Some(Palette::from_slice(palettes::CP_BLUE_WINDOW)),
            WindowPaletteType::Cyan => Some(Palette::from_slice(palettes::CP_CYAN_WINDOW)),
            WindowPaletteType::Gray => Some(Palette::from_slice(palettes::CP_GRAY_WINDOW)),
            WindowPaletteType::Dialog => Some(Palette::from_slice(palettes::CP_GRAY_DIALOG)),
        }
    }

    fn window_init_after_add(&mut self) {
        // Initialize interior owner pointer now that Window is in final position
        self.window_mut().init_interior_owner();
    }

    fn window_constrain_to_parent_bounds(&mut self) {
        self.window_mut().constrain_to_limits();
    }

    fn window_set_parent_bounds(&mut self, bounds: Rect) {
        self.window_mut().explicit_drag_limits = Some(bounds);
    }
}

impl GroupLike for Window {
    fn group(&self) -> &Group {
        &self.interior
    }
    fn group_mut(&mut self) -> &mut Group {
        &mut self.interior
    }
}

impl WindowLike for Window {
    fn window(&self) -> &Window {
        self
    }
    fn window_mut(&mut self) -> &mut Window {
        self
    }
}

/// Generates the `impl View for $t` of a window-shaped type from its
/// [`WindowLike`] implementation.
///
/// Every `View` method is forwarded to the matching `window_*` base body, so
/// a wrapper can no longer forward selectively and fall back to a trait
/// default by accident. Overrides are written inline and replace the forward
/// for that method only; the base implementation stays reachable through its
/// `window_*` name:
///
/// ```ignore
/// impl_view_for_window!(Dialog {
///     fn handle_event(&mut self, event: &mut Event) {
///         self.window_handle_event(event); // TWindow::handleEvent(event)
///         // dialog-specific handling follows
///     }
///     fn get_palette(&self) -> Option<Palette> {
///         Some(Palette::from_slice(palettes::CP_GRAY_DIALOG))
///     }
/// });
/// ```
#[macro_export]
macro_rules! impl_view_for_window {
    ($t:ty) => {
        $crate::impl_view_for_window!($t {});
    };
    ($t:ty { $( fn $name:ident ( $($args:tt)* ) $( -> $ret:ty )? $body:block )* }) => {
        impl $crate::views::view::View for $t {
            $( fn $name ( $($args)* ) $( -> $ret )? $body )*
            $crate::impl_view_for_window!(@fwd core; $($name)*);
            $crate::impl_view_for_window!(@fwd core_mut; $($name)*);
            $crate::impl_view_for_window!(@fwd set_bounds; $($name)*);
            $crate::impl_view_for_window!(@fwd draw; $($name)*);
            $crate::impl_view_for_window!(@fwd handle_event; $($name)*);
            $crate::impl_view_for_window!(@fwd update_cursor; $($name)*);
            $crate::impl_view_for_window!(@fwd can_focus; $($name)*);
            $crate::impl_view_for_window!(@fwd set_focus; $($name)*);
            $crate::impl_view_for_window!(@fwd window_number; $($name)*);
            $crate::impl_view_for_window!(@fwd get_end_state; $($name)*);
            $crate::impl_view_for_window!(@fwd set_end_state; $($name)*);
            $crate::impl_view_for_window!(@fwd zoom; $($name)*);
            $crate::impl_view_for_window!(@fwd valid; $($name)*);
            $crate::impl_view_for_window!(@fwd set_parent_bounds; $($name)*);
            $crate::impl_view_for_window!(@fwd get_palette; $($name)*);
            $crate::impl_view_for_window!(@fwd init_after_add; $($name)*);
            $crate::impl_view_for_window!(@fwd constrain_to_parent_bounds; $($name)*);
            $crate::impl_view_for_window!(@fwd as_any; $($name)*);
            $crate::impl_view_for_window!(@fwd as_any_mut; $($name)*);
        }
    };

    // ---- skip a forward when the type overrides the method ----
    (@fwd core; core $($rest:ident)*) => {};
    (@fwd core_mut; core_mut $($rest:ident)*) => {};
    (@fwd set_bounds; set_bounds $($rest:ident)*) => {};
    (@fwd draw; draw $($rest:ident)*) => {};
    (@fwd handle_event; handle_event $($rest:ident)*) => {};
    (@fwd update_cursor; update_cursor $($rest:ident)*) => {};
    (@fwd can_focus; can_focus $($rest:ident)*) => {};
    (@fwd set_focus; set_focus $($rest:ident)*) => {};
    (@fwd window_number; window_number $($rest:ident)*) => {};
    (@fwd get_end_state; get_end_state $($rest:ident)*) => {};
    (@fwd set_end_state; set_end_state $($rest:ident)*) => {};
    (@fwd zoom; zoom $($rest:ident)*) => {};
    (@fwd valid; valid $($rest:ident)*) => {};
    (@fwd set_parent_bounds; set_parent_bounds $($rest:ident)*) => {};
    (@fwd get_palette; get_palette $($rest:ident)*) => {};
    (@fwd init_after_add; init_after_add $($rest:ident)*) => {};
    (@fwd constrain_to_parent_bounds; constrain_to_parent_bounds $($rest:ident)*) => {};
    (@fwd as_any; as_any $($rest:ident)*) => {};
    (@fwd as_any_mut; as_any_mut $($rest:ident)*) => {};
    // not this one: keep looking
    (@fwd $m:ident; $other:ident $($rest:ident)*) => {
        $crate::impl_view_for_window!(@fwd $m; $($rest)*);
    };

    // ---- the forwards themselves ----
    (@fwd core;) => {
        fn core(&self) -> &$crate::views::view::ViewCore {
            $crate::views::window::WindowLike::window(self).view_core()
        }
    };
    (@fwd core_mut;) => {
        fn core_mut(&mut self) -> &mut $crate::views::view::ViewCore {
            $crate::views::window::WindowLike::window_mut(self).view_core_mut()
        }
    };
    (@fwd set_bounds;) => {
        fn set_bounds(&mut self, bounds: $crate::core::geometry::Rect) {
            $crate::views::window::WindowLike::window_set_bounds(self, bounds)
        }
    };
    (@fwd draw;) => {
        fn draw(&mut self, terminal: &mut $crate::terminal::Terminal) {
            $crate::views::window::WindowLike::window_draw(self, terminal)
        }
    };
    (@fwd handle_event;) => {
        fn handle_event(&mut self, event: &mut $crate::core::event::Event) {
            $crate::views::window::WindowLike::window_handle_event(self, event)
        }
    };
    (@fwd update_cursor;) => {
        fn update_cursor(&self, terminal: &mut $crate::terminal::Terminal) {
            $crate::views::window::WindowLike::window_update_cursor(self, terminal)
        }
    };
    (@fwd can_focus;) => {
        fn can_focus(&self) -> bool {
            true
        }
    };
    (@fwd set_focus;) => {
        fn set_focus(&mut self, focused: bool) {
            $crate::views::window::WindowLike::window_set_focus(self, focused)
        }
    };
    (@fwd window_number;) => {
        fn window_number(&self) -> Option<u8> {
            $crate::views::window::WindowLike::window(self).number()
        }
    };
    (@fwd get_end_state;) => {
        fn get_end_state(&self) -> $crate::core::command::CommandId {
            $crate::views::group::GroupLike::end_state(self)
        }
    };
    (@fwd set_end_state;) => {
        fn set_end_state(&mut self, command: $crate::core::command::CommandId) {
            $crate::views::group::GroupLike::end_modal(self, command)
        }
    };
    (@fwd zoom;) => {
        fn zoom(&mut self, max_bounds: $crate::core::geometry::Rect) {
            $crate::views::window::WindowLike::window_zoom(self, max_bounds)
        }
    };
    (@fwd valid;) => {
        fn valid(&mut self, command: $crate::core::command::CommandId) -> bool {
            $crate::views::window::WindowLike::window_valid(self, command)
        }
    };
    (@fwd set_parent_bounds;) => {
        fn set_parent_bounds(&mut self, bounds: $crate::core::geometry::Rect) {
            $crate::views::window::WindowLike::window_set_parent_bounds(self, bounds)
        }
    };
    (@fwd get_palette;) => {
        fn get_palette(&self) -> Option<$crate::core::palette::Palette> {
            $crate::views::window::WindowLike::window_get_palette(self)
        }
    };
    (@fwd init_after_add;) => {
        fn init_after_add(&mut self) {
            $crate::views::window::WindowLike::window_init_after_add(self)
        }
    };
    (@fwd constrain_to_parent_bounds;) => {
        fn constrain_to_parent_bounds(&mut self) {
            $crate::views::window::WindowLike::window_constrain_to_parent_bounds(self)
        }
    };
    (@fwd as_any;) => {
        fn as_any(&self) -> &dyn ::std::any::Any {
            self
        }
    };
    (@fwd as_any_mut;) => {
        fn as_any_mut(&mut self) -> &mut dyn ::std::any::Any {
            self
        }
    };
}

impl_view_for_window!(Window);

/// Builder for creating windows with a fluent API.
///
/// # Examples
///
/// ```
/// use turbo_vision::views::window::WindowBuilder;
/// use turbo_vision::views::GroupLike;
/// use turbo_vision::views::button::ButtonBuilder;
/// use turbo_vision::core::geometry::Rect;
/// use turbo_vision::core::command::CM_OK;
///
/// // Create a resizable window (default)
/// let mut window = WindowBuilder::new()
///     .bounds(Rect::new(10, 5, 60, 20))
///     .title("My Window")
///     .build();
///
/// // Create a non-resizable window
/// let mut dialog = WindowBuilder::new()
///     .bounds(Rect::new(10, 5, 40, 15))
///     .title("Fixed Size")
///     .resizable(false)
///     .build();
///
/// // Add a button to the window
/// let ok_button = ButtonBuilder::new()
///     .bounds(Rect::new(10, 10, 20, 12))
///     .title("OK")
///     .command(CM_OK)
///     .build();
/// window.add(Box::new(ok_button));
/// ```
pub struct WindowBuilder {
    bounds: Option<Rect>,
    title: Option<String>,
    resizable: bool,
    palette_type: WindowPaletteType,
    grow_mode: crate::core::state::GrowFlags,
}

impl WindowBuilder {
    /// Creates a new WindowBuilder with default values.
    pub fn new() -> Self {
        Self {
            bounds: None,
            title: None,
            resizable: true, // Default to resizable (matches Borland TWindow with wfGrow)
            palette_type: WindowPaletteType::Blue,
            // Deliberately not gfGrowAll — see the comment in Window::new_with_palette.
            grow_mode: crate::core::state::GF_GROW_HI_X | crate::core::state::GF_GROW_HI_Y,
        }
    }

    /// Sets the window bounds (required).
    #[must_use]
    pub fn bounds(mut self, bounds: Rect) -> Self {
        self.bounds = Some(bounds);
        self
    }

    /// Sets the window title (required).
    #[must_use]
    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
        self
    }

    /// Sets whether the window is resizable (default: true).
    /// Resizable windows show single-line bottom corners and a resize handle.
    /// Non-resizable windows show double-line bottom corners (like TDialog).
    #[must_use]
    pub fn resizable(mut self, resizable: bool) -> Self {
        self.resizable = resizable;
        self
    }

    /// Sets the window palette type (default: Blue).
    #[must_use]
    pub fn palette_type(mut self, palette_type: WindowPaletteType) -> Self {
        self.palette_type = palette_type;
        self
    }

    /// Sets the window's grow mode flags (default:
    /// `GF_GROW_HI_X | GF_GROW_HI_Y`, so the window stretches to fill new
    /// desktop space rather than translating like Borland's literal
    /// `gfGrowAll`). Controls how the window's bounds move when its owner
    /// (the Desktop) is resized; see the comment in `Window::new_with_palette` and
    /// `View::grow_mode`.
    #[must_use]
    pub fn grow_mode(mut self, grow_mode: crate::core::state::GrowFlags) -> Self {
        self.grow_mode = grow_mode;
        self
    }

    /// Builds the Window.
    ///
    /// # Panics
    ///
    /// Panics if required fields (bounds, title) are not set.
    pub fn build(self) -> Window {
        let bounds = self.bounds.expect("Window bounds must be set");
        let title = self.title.expect("Window title must be set");

        let frame_palette = match self.palette_type {
            WindowPaletteType::Blue => super::frame::FramePaletteType::EditorWindow,
            WindowPaletteType::Cyan => super::frame::FramePaletteType::HelpWindow,
            WindowPaletteType::Gray | WindowPaletteType::Dialog => {
                super::frame::FramePaletteType::Dialog
            }
        };

        let resizable = match self.palette_type {
            WindowPaletteType::Dialog => false,
            _ => self.resizable,
        };

        let mut window =
            Window::new_with_palette(bounds, &title, frame_palette, self.palette_type, resizable);
        window.set_grow_mode(self.grow_mode);
        window
    }
}

impl Default for WindowBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn window_like_override_of_get_palette_is_used_by_window_draw() {
        use crate::core::palette::{Palette, palettes};

        struct RedWindow(Window);
        impl GroupLike for RedWindow {
            fn group(&self) -> &Group {
                &self.0.interior
            }
            fn group_mut(&mut self) -> &mut Group {
                &mut self.0.interior
            }
        }
        impl WindowLike for RedWindow {
            fn window(&self) -> &Window {
                &self.0
            }
            fn window_mut(&mut self) -> &mut Window {
                &mut self.0
            }
        }
        crate::impl_view_for_window!(RedWindow {
            fn get_palette(&self) -> Option<Palette> {
                Some(Palette::from_slice(palettes::CP_GRAY_DIALOG))
            }
        });

        let mut terminal = crate::test_util::test_terminal(80, 25);
        let mut plain = Window::new(Rect::new(0, 0, 20, 5), "a");
        let mut red = RedWindow(Window::new(Rect::new(0, 0, 20, 5), "a"));
        plain.set_focus(true);
        red.set_focus(true);

        plain.draw(&mut terminal);
        let plain_cell = terminal.read_cell(0, 0).unwrap();
        red.draw(&mut terminal);
        let red_cell = terminal.read_cell(0, 0).unwrap();

        assert_ne!(
            plain_cell.attr, red_cell.attr,
            "base window_draw must consult the overridden get_palette"
        );
    }

    #[test]
    fn test_new_with_type_gray() {
        let window = Window::new_with_type(
            Rect::new(5, 5, 40, 20),
            "Gray Panel",
            WindowPaletteType::Gray,
        );
        assert_eq!(window.bounds(), Rect::new(5, 5, 40, 20));
    }

    #[test]
    fn test_new_with_type_cyan() {
        let window = Window::new_with_type(
            Rect::new(5, 5, 40, 20),
            "Cyan Window",
            WindowPaletteType::Cyan,
        );
        assert_eq!(window.bounds(), Rect::new(5, 5, 40, 20));
    }

    #[test]
    fn test_new_with_type_blue() {
        let window = Window::new_with_type(
            Rect::new(5, 5, 40, 20),
            "Blue Window",
            WindowPaletteType::Blue,
        );
        assert_eq!(window.bounds(), Rect::new(5, 5, 40, 20));
    }

    #[test]
    fn test_set_focus_propagates_sf_active_to_window_and_frame() {
        use crate::core::state::SF_ACTIVE;

        let mut window = Window::new(Rect::new(0, 0, 40, 15), "Test");

        window.set_focus(true);
        assert_ne!(window.state() & SF_ACTIVE, 0);
        assert_ne!(window.frame.state() & SF_ACTIVE, 0);

        window.set_focus(false);
        assert_eq!(window.state() & SF_ACTIVE, 0);
        assert_eq!(window.frame.state() & SF_ACTIVE, 0);
    }

    #[test]
    fn test_auto_close_respects_valid() {
        use crate::core::state::SF_CLOSED;

        // A child view whose valid() vetoes the close
        struct Vetoer {
            core: ViewCore,
        }
        impl View for Vetoer {
            fn core(&self) -> &ViewCore {
                &self.core
            }

            fn core_mut(&mut self) -> &mut ViewCore {
                &mut self.core
            }

            fn draw(&mut self, _terminal: &mut crate::terminal::Terminal) {}
            fn handle_event(&mut self, _event: &mut Event) {}
            fn valid(&mut self, _command: crate::core::command::CommandId) -> bool {
                false
            }
            fn get_palette(&self) -> Option<crate::core::palette::Palette> {
                None
            }
        }

        // Window with a vetoing child: CM_CLOSE must NOT mark it closed
        let mut window = Window::new(Rect::new(0, 0, 40, 15), "Test");
        window.add(Box::new(Vetoer {
            core: ViewCore::new(Rect::new(0, 0, 5, 1)),
        }));
        let mut event = Event::command(CM_CLOSE);
        window.handle_event(&mut event);
        assert_eq!(window.state() & SF_CLOSED, 0);
        assert_eq!(event.what, EventType::Nothing); // event still consumed

        // Window whose children all validate: CM_CLOSE closes it
        let mut window = Window::new(Rect::new(0, 0, 40, 15), "Test");
        let mut event = Event::command(CM_CLOSE);
        window.handle_event(&mut event);
        assert_ne!(window.state() & SF_CLOSED, 0);
        assert_eq!(event.what, EventType::Nothing);
    }

    #[test]
    fn test_builder_with_palette_type() {
        let window = WindowBuilder::new()
            .bounds(Rect::new(5, 5, 40, 20))
            .title("Gray Window")
            .palette_type(WindowPaletteType::Gray)
            .build();
        assert_eq!(window.bounds(), Rect::new(5, 5, 40, 20));
    }

    #[test]
    fn keyboard_resize_mode_moves_resizes_and_restores() {
        use crate::core::command::CM_RESIZE;
        use crate::core::event::{Event, EventType, KB_DOWN, KB_ESC, KB_RIGHT};
        use crossterm::event::KeyModifiers;

        let mut window = Window::new(Rect::new(10, 5, 40, 15), "Test");
        window.set_focus(true);
        let original = window.bounds();

        // Enter keyboard move/resize mode
        let mut ev = Event::command(CM_RESIZE);
        window.handle_event(&mut ev);
        assert_eq!(ev.what, EventType::Nothing);

        // Arrow moves the whole window
        let mut ev = Event::keyboard(KB_RIGHT);
        window.handle_event(&mut ev);
        assert_eq!(window.bounds().a.x, 11);
        assert_eq!(window.bounds().b.x, 41);

        // Shift+arrow grows the bottom-right corner
        let mut ev = Event::keyboard(KB_DOWN);
        ev.key_modifiers = KeyModifiers::SHIFT;
        window.handle_event(&mut ev);
        assert_eq!(window.bounds().b.y, 16);

        // Esc restores the original bounds and leaves the mode
        let mut ev = Event::keyboard(KB_ESC);
        window.handle_event(&mut ev);
        assert_eq!(window.bounds(), original);

        // Mode is off: arrows no longer move the window
        let mut ev = Event::keyboard(KB_RIGHT);
        window.handle_event(&mut ev);
        assert_eq!(window.bounds(), original);
    }
}
