// (C) 2025 - Enzo Lombardi

//! Window view - draggable, resizable window with frame and shadow.

use super::frame::Frame;
use super::group::Group;
use super::view::{View, ViewId};
use crate::core::command::{CM_CANCEL, CM_CLOSE};
use crate::core::event::{Event, EventType};
use crate::core::geometry::{Point, Rect};
use crate::core::state::{SF_DRAGGING, SF_MODAL, SF_RESIZING, SF_SHADOW, shadow_size, StateFlags};
use crate::terminal::Terminal;

pub struct Window {
    bounds: Rect,
    frame: Frame,
    interior: Group,
    /// Direct children of window (positioned relative to window frame, not interior)
    /// Used for scrollbars and other frame-relative elements
    frame_children: Vec<Box<dyn View>>,
    state: StateFlags,
    options: u16,
    /// Drag start position (relative to mouse when drag started)
    drag_offset: Option<Point>,
    /// Resize start size (size when resize drag started)
    resize_start_size: Option<Point>,
    /// Minimum window size (matches Borland's minWinSize)
    min_size: Point,
    /// Saved bounds for zoom/restore (matches Borland's zoomRect)
    zoom_rect: Rect,
    /// Previous bounds (for calculating union rect for redrawing)
    /// Matches Borland: TView::locate() calculates union of old and new bounds
    prev_bounds: Option<Rect>,
    /// Owner (parent) view - Borland: TView::owner
    owner: Option<*const dyn View>,
    /// Palette type (Dialog vs Editor window)
    palette_type: WindowPaletteType,
    /// Explicit drag limits (for modal dialogs not added to desktop)
    /// Used when owner is None but we still want to constrain dragging
    explicit_drag_limits: Option<Rect>,
}

#[derive(Clone, Copy)]
pub enum WindowPaletteType {
    Blue,   // Uses CP_BLUE_WINDOW
    Cyan,   // Uses CP_CYAN_WINDOW
    Gray,   // Uses CP_GRAY_WINDOW
    Dialog, // Uses CP_GRAY_DIALOG
}

impl Window {
    /// Create a new TWindow with blue palette (default Borland TWindow behavior)
    /// Matches Borland: TWindow constructor sets palette(wpBlueWindow)
    /// For TDialog (gray palette), use new_for_dialog() instead
    pub fn new(bounds: Rect, title: &str) -> Self {
        Self::new_with_palette(
            bounds,
            title,
            super::frame::FramePaletteType::Editor,
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

        let mut window = Self {
            bounds,
            frame,
            interior,
            frame_children: Vec::new(),
            state: SF_SHADOW, // Windows have shadows by default
            options: OF_SELECTABLE | OF_TOP_SELECT | OF_TILEABLE, // Matches Borland: TWindow/TEditWindow flags
            drag_offset: None,
            resize_start_size: None,
            min_size: Point::new(16, 6), // Minimum size: 16 wide, 6 tall (matches Borland's minWinSize)
            zoom_rect: bounds,           // Initialize to current bounds
            prev_bounds: None,
            owner: None,
            palette_type: window_palette,
            explicit_drag_limits: None,
        };

        // Set the interior's owner to the window for palette chain resolution
        window.interior.set_owner(&window as *const _ as *const dyn View);

        window
    }

    pub fn add(&mut self, mut view: Box<dyn View>) -> ViewId {
        // Set the owner type based on palette type for correct color remapping
        let owner_type = match self.palette_type {
            WindowPaletteType::Dialog => super::view::OwnerType::Dialog,
            WindowPaletteType::Cyan => super::view::OwnerType::CyanWindow,
            _ => super::view::OwnerType::Window,
        };
        view.set_owner_type(owner_type);

        // Add to interior group (which will set owner pointer for palette chain)
        self.interior.add(view)
    }

    /// Add a child positioned relative to the window frame (not interior)
    /// Used for scrollbars and other frame-edge elements
    /// Matches Borland: TWindow is a TGroup, all children use window-relative coords
    pub fn add_frame_child(&mut self, mut view: Box<dyn View>) -> usize {
        // Set the owner type based on palette type for correct color remapping
        let owner_type = match self.palette_type {
            WindowPaletteType::Dialog => super::view::OwnerType::Dialog,
            WindowPaletteType::Cyan => super::view::OwnerType::CyanWindow,
            _ => super::view::OwnerType::Window,
        };
        view.set_owner_type(owner_type);

        // Set owner pointer for palette chain
        view.set_owner(self as *const _ as *const dyn View);

        // Convert from relative to absolute coordinates (relative to window frame)
        let child_bounds = view.bounds();
        let absolute_bounds = Rect::new(
            self.bounds.a.x + child_bounds.a.x,
            self.bounds.a.y + child_bounds.a.y,
            self.bounds.a.x + child_bounds.b.x,
            self.bounds.a.y + child_bounds.b.y,
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

    pub fn set_initial_focus(&mut self) {
        self.interior.set_initial_focus();
    }

    /// Set the window title
    /// Matches Borland: TWindow allows title mutation via setTitle()
    /// The frame will be redrawn on the next draw() call
    pub fn set_title(&mut self, title: &str) {
        self.frame.set_title(title);
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

    /// Get drag limits from owner (parent bounds) or explicit limits
    /// Matches Borland: TFrame::dragWindow() gets limits = owner->owner->getExtent()
    /// Returns parent bounds if owner exists, explicit limits if set, otherwise unrestricted
    fn get_drag_limits(&self) -> Rect {
        // First check explicit drag limits (for modal dialogs)
        if let Some(limits) = self.explicit_drag_limits {
            return limits;
        }

        // Then check owner pointer
        if let Some(owner_ptr) = self.owner {
            // Safety: owner pointer is valid during the window's lifetime
            unsafe { (*owner_ptr).bounds() }
        } else {
            // No owner - unrestricted movement
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
        let width = self.bounds.width();
        let height = self.bounds.height();

        // Account for shadow when constraining edges
        let (shadow_x, shadow_y) = if (self.state & SF_SHADOW) != 0 {
            shadow_size()
        } else {
            (0, 0)
        };

        let mut new_x = self.bounds.a.x;
        let mut new_y = self.bounds.a.y;

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
        if new_x != self.bounds.a.x || new_y != self.bounds.a.y {
            self.bounds = Rect::new(new_x, new_y, new_x + width, new_y + height);

            // Update frame and interior bounds
            self.frame.set_bounds(self.bounds);
            let mut interior_bounds = self.bounds;
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

    /// Get the number of child views in the interior
    pub fn child_count(&self) -> usize {
        self.interior.len()
    }

    /// Get a reference to a child view by index
    pub fn child_at(&self, index: usize) -> &dyn View {
        self.interior.child_at(index)
    }

    /// Get a mutable reference to a child view by index
    pub fn child_at_mut(&mut self, index: usize) -> &mut dyn View {
        self.interior.child_at_mut(index)
    }

    /// Get an immutable reference to a child by its ViewId
    /// Returns None if the ViewId is not found
    pub fn child_by_id(&self, view_id: ViewId) -> Option<&dyn View> {
        self.interior.child_by_id(view_id)
    }

    /// Get a mutable reference to a child by its ViewId
    /// Returns None if the ViewId is not found
    pub fn child_by_id_mut(&mut self, view_id: ViewId) -> Option<&mut (dyn View + '_)> {
        self.interior.child_by_id_mut(view_id)
    }

    /// Remove a child by its ViewId
    /// Returns true if a child was found and removed, false otherwise
    pub fn remove_by_id(&mut self, view_id: ViewId) -> bool {
        self.interior.remove_by_id(view_id)
    }

    /// Get the union rect of current and previous bounds (for redrawing)
    /// Matches Borland: TView::locate() calculates union rect
    /// Returns None if window hasn't moved yet
    pub fn get_redraw_union(&self) -> Option<Rect> {
        self.prev_bounds.map(|prev| {
            // Union of old and new bounds, including shadows
            let mut union = prev.union(&self.bounds);

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

    /// Execute a modal event loop
    /// Delegates to the interior Group's execute() method
    /// Matches Borland: Window and Dialog both inherit TGroup's execute()
    pub fn execute(&mut self, app: &mut crate::app::Application) -> crate::core::command::CommandId {
        self.interior.execute(app)
    }

    /// End the modal event loop
    /// Delegates to the interior Group's end_modal() method
    pub fn end_modal(&mut self, command: crate::core::command::CommandId) {
        self.interior.end_modal(command);
    }

    /// Get the current end_state from the interior Group
    /// Used by Dialog to check if the modal loop should end
    pub fn get_end_state(&self) -> crate::core::command::CommandId {
        self.interior.get_end_state()
    }

    /// Set the end_state in the interior Group
    /// Used by modal dialogs to signal they want to close
    pub fn set_end_state(&mut self, command: crate::core::command::CommandId) {
        self.interior.set_end_state(command);
    }

    /// Initialize the interior's owner pointer after Window is in its final memory location.
    /// Must be called after any operation that moves the Window (adding to parent, etc.)
    /// This ensures the interior Group has a valid pointer to this Window.
    pub fn init_interior_owner(&mut self) {
        // NOTE: We don't set interior's owner pointer to avoid unsafe casting
        // Color palette resolution is handled without needing parent pointers
    }
}

impl View for Window {
    fn bounds(&self) -> Rect {
        self.bounds
    }

    fn set_bounds(&mut self, bounds: Rect) {
        self.bounds = bounds;
        self.frame.set_bounds(bounds);

        // Update interior bounds (absolute, inset by 1 for frame)
        let mut interior_bounds = bounds;
        interior_bounds.grow(-1, -1);
        self.interior.set_bounds(interior_bounds);

        // NOTE: We do NOT automatically update frame_children here
        // Subclasses like EditWindow handle frame_children positioning manually
        // because scrollbars need to be repositioned based on new window SIZE, not just offset
    }

    fn draw(&mut self, terminal: &mut Terminal) {
        self.frame.draw(terminal);
        self.interior.draw(terminal);

        // Draw frame children (scrollbars, etc.) after interior so they appear on top
        for child in &mut self.frame_children {
            child.draw(terminal);
        }

        // Draw shadow if enabled
        if self.has_shadow() {
            self.draw_shadow(terminal);
        }
    }

    fn update_cursor(&self, terminal: &mut Terminal) {
        // Propagate cursor update to interior group
        self.interior.update_cursor(terminal);
    }

    fn handle_event(&mut self, event: &mut Event) {
        // First, let the frame handle the event (for close button clicks, drag start, etc.)
        self.frame.handle_event(event);

        // Check if frame started dragging or resizing
        let frame_dragging = (self.frame.state() & SF_DRAGGING) != 0;
        let frame_resizing = (self.frame.state() & SF_RESIZING) != 0;

        if frame_dragging && self.drag_offset.is_none() {
            // Frame just started dragging - record offset
            if event.what == EventType::MouseDown || event.what == EventType::MouseMove {
                let mouse_pos = event.mouse.pos;
                self.drag_offset = Some(Point::new(mouse_pos.x - self.bounds.a.x, mouse_pos.y - self.bounds.a.y));
                self.state |= SF_DRAGGING;
                event.clear(); // Mark event as handled
                return;
            }
        }

        if frame_resizing && self.resize_start_size.is_none() {
            // Frame just started resizing - record initial size
            if event.what == EventType::MouseDown || event.what == EventType::MouseMove {
                let mouse_pos = event.mouse.pos;
                // Calculate offset from bottom-right corner
                // Borland: p = size - event.mouse.where (tview.cc:235)
                self.resize_start_size = Some(Point::new(self.bounds.b.x - mouse_pos.x, self.bounds.b.y - mouse_pos.y));
                self.state |= SF_RESIZING;
                event.clear(); // Mark event as handled
                return;
            }
        }

        // Handle mouse move during drag
        if frame_dragging && self.drag_offset.is_some() {
            if event.what == EventType::MouseMove {
                let mouse_pos = event.mouse.pos;
                let offset = self.drag_offset.unwrap();

                // Calculate new position
                let mut new_x = mouse_pos.x - offset.x;
                let mut new_y = mouse_pos.y - offset.y;

                // Get drag limits from owner (parent bounds)
                // Matches Borland: TView::moveGrow() constrains position to limits
                let limits = self.get_drag_limits();
                let width = self.bounds.width();
                let height = self.bounds.height();

                // Account for shadow when constraining edges
                let (shadow_x, shadow_y) = if (self.state & SF_SHADOW) != 0 {
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
                self.prev_bounds = Some(self.bounds);

                // Update bounds (maintaining size)
                self.bounds = Rect::new(new_x, new_y, new_x + width, new_y + height);

                // Update frame and interior bounds
                self.frame.set_bounds(self.bounds);
                let mut interior_bounds = self.bounds;
                interior_bounds.grow(-1, -1);
                self.interior.set_bounds(interior_bounds);

                event.clear(); // Mark event as handled
                return;
            }
        }

        // Handle mouse move during resize
        if frame_resizing && self.resize_start_size.is_some() {
            if event.what == EventType::MouseMove {
                let mouse_pos = event.mouse.pos;
                let offset = self.resize_start_size.unwrap();

                // Calculate new size (Borland: event.mouse.where += p, then use as size)
                // Ensure positive before casting to u16 to avoid wraparound
                let new_width = (mouse_pos.x + offset.x - self.bounds.a.x).max(0) as u16;
                let new_height = (mouse_pos.y + offset.y - self.bounds.a.y).max(0) as u16;

                // Apply size constraints (Borland: sizeLimits)
                let (min, max) = self.size_limits();
                let mut final_width = new_width.max(min.x as u16).min(max.x as u16);
                let mut final_height = new_height.max(min.y as u16).min(max.y as u16);

                // Constrain size to not exceed parent bounds
                // Borland: TView::moveGrow() constrains both position and size to limits
                let limits = self.get_drag_limits();
                let max_width = (limits.b.x - self.bounds.a.x).max(0) as u16;
                let max_height = (limits.b.y - self.bounds.a.y).max(0) as u16;
                final_width = final_width.min(max_width);
                final_height = final_height.min(max_height);

                // Save previous bounds for union rect calculation
                self.prev_bounds = Some(self.bounds);

                // Update bounds (maintaining position, changing size)
                self.bounds.b.x = self.bounds.a.x + final_width as i16;
                self.bounds.b.y = self.bounds.a.y + final_height as i16;

                // Update frame and interior bounds
                self.frame.set_bounds(self.bounds);
                let mut interior_bounds = self.bounds;
                interior_bounds.grow(-1, -1);
                self.interior.set_bounds(interior_bounds);

                event.clear(); // Mark event as handled
                return;
            }
        }

        // Check if frame ended dragging
        if !frame_dragging && self.drag_offset.is_some() {
            self.drag_offset = None;
            self.state &= !SF_DRAGGING;
        }

        // Check if frame ended resizing
        if !frame_resizing && self.resize_start_size.is_some() {
            self.resize_start_size = None;
            self.state &= !SF_RESIZING;
        }

        // Handle ESC key for modal windows
        // Modal windows should close when ESC or ESC ESC is pressed
        if event.what == EventType::Keyboard {
            let is_esc = event.key_code == crate::core::event::KB_ESC;
            let is_esc_esc = event.key_code == crate::core::event::KB_ESC_ESC;

            if (is_esc || is_esc_esc) && (self.state & SF_MODAL) != 0 {
                // Modal window: ESC ends the modal loop with CM_CANCEL
                self.end_modal(CM_CANCEL);
                event.clear();
                return;
            }
        }

        // Handle CM_CLOSE command (Borland: twindow.cc lines 104-118, 70-78)
        // Frame generates CM_CLOSE when close button is clicked
        if event.what == EventType::Command && event.command == CM_CLOSE {
            // Check if this window is modal
            if (self.state & SF_MODAL) != 0 {
                // Modal window: end modal loop with CM_CANCEL
                // Borland: event.message.command = cmCancel; putEvent(event);
                self.end_modal(CM_CANCEL);
                event.clear();
            } else {
                // Non-modal window: Let the event bubble up to the application level
                // The application will handle validation (showing "Save changes?" dialog)
                // and removal of the window.
                //
                // Note: In Borland, TWindow::close() calls valid(cmClose) and destroys itself.
                // In our Rust architecture, we can't show dialogs in valid() because we don't
                // have access to Application/Terminal. So we let CM_CLOSE bubble up to the
                // application where it can show dialogs and handle the removal.
                //
                // DO NOT clear the event - application needs to see it!
                // DO NOT mark as SF_CLOSED here - application will remove the window after validation
            }
            return; // Don't pass CM_CLOSE to interior
        }

        // Then let the interior handle it (if not already handled)
        self.interior.handle_event(event);
    }

    fn can_focus(&self) -> bool {
        true
    }

    fn set_focus(&mut self, focused: bool) {
        // Propagate focus to the interior group
        // When the window gets focus, set focus on its first focusable child
        if focused {
            self.interior.set_initial_focus();
        } else {
            self.interior.clear_all_focus();
        }
    }

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

    fn get_end_state(&self) -> crate::core::command::CommandId {
        self.interior.get_end_state()
    }

    fn set_end_state(&mut self, command: crate::core::command::CommandId) {
        self.interior.set_end_state(command);
    }

    /// Zoom (maximize) or restore window
    /// Matches Borland: TWindow::zoom() toggles between current size and maximum size
    /// In Borland, this is called by owner in response to cmZoom command
    fn zoom(&mut self, max_bounds: Rect) {
        let (_min, _max_size) = self.size_limits();
        let current_size = Point::new(self.bounds.width(), self.bounds.height());

        // If not at max size, zoom to max
        if current_size.x != max_bounds.width() || current_size.y != max_bounds.height() {
            // Save current bounds for restore
            self.zoom_rect = self.bounds;

            // Save previous bounds for redraw union
            self.prev_bounds = Some(self.bounds);

            // Zoom to max size (typically desktop bounds)
            self.bounds = max_bounds;
        } else {
            // Restore to saved bounds
            self.prev_bounds = Some(self.bounds);
            self.bounds = self.zoom_rect;
        }

        // Update frame and interior
        self.frame.set_bounds(self.bounds);
        let mut interior_bounds = self.bounds;
        interior_bounds.grow(-1, -1);
        self.interior.set_bounds(interior_bounds);
    }

    /// Validate window before closing with given command
    /// Matches Borland: TWindow inherits TGroup::valid() which validates all children
    /// Delegates to interior group to validate all children
    fn valid(&mut self, command: crate::core::command::CommandId) -> bool {
        self.interior.valid(command)
    }

    fn set_owner(&mut self, owner: *const dyn View) {
        self.owner = Some(owner);
        // Do NOT set interior.owner here - Window might still move!
        // Instead, init_interior_owner() must be called after Window is in final position
    }

    fn get_owner(&self) -> Option<*const dyn View> {
        self.owner
    }

    fn get_palette(&self) -> Option<crate::core::palette::Palette> {
        use crate::core::palette::{Palette, palettes};
        match self.palette_type {
            WindowPaletteType::Blue => Some(Palette::from_slice(palettes::CP_BLUE_WINDOW)),
            WindowPaletteType::Cyan => Some(Palette::from_slice(palettes::CP_CYAN_WINDOW)),
            WindowPaletteType::Gray => Some(Palette::from_slice(palettes::CP_GRAY_WINDOW)),
            WindowPaletteType::Dialog => Some(Palette::from_slice(palettes::CP_GRAY_DIALOG)),
        }
    }

    fn init_after_add(&mut self) {
        // Initialize interior owner pointer now that Window is in final position
        self.init_interior_owner();
    }

    fn constrain_to_parent_bounds(&mut self) {
        self.constrain_to_limits();
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}

/// Builder for creating windows with a fluent API.
///
/// # Examples
///
/// ```
/// use turbo_vision::views::window::WindowBuilder;
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
}

impl WindowBuilder {
    /// Creates a new WindowBuilder with default values.
    pub fn new() -> Self {
        Self {
            bounds: None,
            title: None,
            resizable: true, // Default to resizable (matches Borland TWindow with wfGrow)
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

    /// Builds the Window.
    ///
    /// # Panics
    ///
    /// Panics if required fields (bounds, title) are not set.
    pub fn build(self) -> Window {
        let bounds = self.bounds.expect("Window bounds must be set");
        let title = self.title.expect("Window title must be set");

        Window::new_with_palette(
            bounds,
            &title,
            super::frame::FramePaletteType::Editor,
            WindowPaletteType::Blue,
            self.resizable,
        )
    }
}

impl Default for WindowBuilder {
    fn default() -> Self {
        Self::new()
    }
}
