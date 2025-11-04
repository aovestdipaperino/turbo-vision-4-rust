use crate::core::geometry::{Rect, Point};
use crate::core::event::{Event, EventType};
use crate::core::command::{CM_CLOSE, CM_CANCEL};
use crate::core::state::{StateFlags, SF_SHADOW, SF_DRAGGING, SF_RESIZING, SF_MODAL, SHADOW_ATTR};
use crate::core::palette::colors;
use crate::terminal::Terminal;
use super::view::{View, draw_shadow};
use super::frame::Frame;
use super::group::Group;

pub struct Window {
    bounds: Rect,
    frame: Frame,
    interior: Group,
    state: StateFlags,
    /// Drag start position (relative to mouse when drag started)
    drag_offset: Option<Point>,
    /// Resize start size (size when resize drag started)
    resize_start_size: Option<Point>,
    /// Minimum window size (matches Borland's minWinSize)
    min_size: Point,
    /// Previous bounds (for calculating union rect for redrawing)
    /// Matches Borland: TView::locate() calculates union of old and new bounds
    prev_bounds: Option<Rect>,
}

impl Window {
    pub fn new(bounds: Rect, title: &str) -> Self {
        let frame = Frame::new(bounds, title);

        // Interior bounds are ABSOLUTE (inset by 1 from window bounds for frame)
        let mut interior_bounds = bounds;
        interior_bounds.grow(-1, -1);
        let interior = Group::with_background(interior_bounds, colors::DIALOG_NORMAL);

        Self {
            bounds,
            frame,
            interior,
            state: SF_SHADOW, // Windows have shadows by default
            drag_offset: None,
            resize_start_size: None,
            min_size: Point::new(16, 6), // Minimum size: 16 wide, 6 tall (matches Borland's minWinSize)
            prev_bounds: None,
        }
    }

    pub fn add(&mut self, view: Box<dyn View>) {
        self.interior.add(view);
    }

    pub fn set_initial_focus(&mut self) {
        self.interior.set_initial_focus();
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

    /// Get the union rect of current and previous bounds (for redrawing)
    /// Matches Borland: TView::locate() calculates union rect
    /// Returns None if window hasn't moved yet
    pub fn get_redraw_union(&self) -> Option<Rect> {
        self.prev_bounds.map(|prev| {
            // Union of old and new bounds, including shadows
            let mut union = prev.union(&self.bounds);

            // Expand by 1 on right and bottom for shadow
            // Matches Borland: TView::shadowSize
            union.b.x += 1;
            union.b.y += 1;

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

    /// Helper method to get editor text from the first child (if it's an Editor)
    /// This is a pragmatic workaround for the editor demo where we know
    /// the window contains an Editor at index 0
    ///
    /// Returns None if there are no children
    pub fn get_editor_text_if_present(&self) -> Option<String> {
        if self.child_count() == 0 {
            return None;
        }

        // We know the child is an Editor, but we can't downcast without Any trait
        // So we use a workaround: Store the editor pointer and call get_text()
        // This requires unsafe, but it's contained and we know the types
        let view_ref = self.child_at(0);
        let view_ptr = view_ref as *const dyn crate::views::View;

        // SAFETY: We know from create_editor_window() that child 0 is always an Editor
        // This is a controlled demo scenario where we manage the window construction
        unsafe {
            let editor_ptr = view_ptr as *const crate::views::editor::Editor;
            if !editor_ptr.is_null() {
                Some((*editor_ptr).get_text())
            } else {
                None
            }
        }
    }

    /// Check if the editor (first child) is modified
    /// Returns None if there are no children
    pub fn is_editor_modified(&self) -> Option<bool> {
        if self.child_count() == 0 {
            return None;
        }

        let view_ref = self.child_at(0);
        let view_ptr = view_ref as *const dyn crate::views::View;

        // SAFETY: We know from create_editor_window() that child 0 is always an Editor
        unsafe {
            let editor_ptr = view_ptr as *const crate::views::editor::Editor;
            if !editor_ptr.is_null() {
                Some((*editor_ptr).is_modified())
            } else {
                None
            }
        }
    }

    /// Clear the modified flag on the editor (first child)
    /// Returns true if successful, false if there are no children
    pub fn clear_editor_modified(&mut self) -> bool {
        if self.child_count() == 0 {
            return false;
        }

        let view_ref = self.child_at_mut(0);
        let view_ptr = view_ref as *mut dyn crate::views::View;

        // SAFETY: We know from create_editor_window() that child 0 is always an Editor
        unsafe {
            let editor_ptr = view_ptr as *mut crate::views::editor::Editor;
            if !editor_ptr.is_null() {
                (*editor_ptr).clear_modified();
                true
            } else {
                false
            }
        }
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
    }

    fn draw(&mut self, terminal: &mut Terminal) {
        self.frame.draw(terminal);
        self.interior.draw(terminal);

        // Draw shadow if enabled
        if self.has_shadow() {
            draw_shadow(terminal, self.bounds, SHADOW_ATTR);
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
                self.drag_offset = Some(Point::new(
                    mouse_pos.x - self.bounds.a.x,
                    mouse_pos.y - self.bounds.a.y,
                ));
                self.state |= SF_DRAGGING;
            }
        }

        if frame_resizing && self.resize_start_size.is_none() {
            // Frame just started resizing - record initial size
            if event.what == EventType::MouseDown || event.what == EventType::MouseMove {
                let mouse_pos = event.mouse.pos;
                // Calculate offset from bottom-right corner
                // Borland: p = size - event.mouse.where (tview.cc:235)
                self.resize_start_size = Some(Point::new(
                    self.bounds.b.x - mouse_pos.x,
                    self.bounds.b.y - mouse_pos.y,
                ));
                self.state |= SF_RESIZING;
            }
        }

        // Handle mouse move during drag
        if frame_dragging && self.drag_offset.is_some() {
            if event.what == EventType::MouseMove {
                let mouse_pos = event.mouse.pos;
                let offset = self.drag_offset.unwrap();

                // Calculate new position
                let new_x = mouse_pos.x - offset.x;
                let new_y = mouse_pos.y - offset.y;

                // Save previous bounds for union rect calculation (Borland's locate pattern)
                self.prev_bounds = Some(self.bounds);

                // Update bounds (maintaining size)
                let width = self.bounds.width();
                let height = self.bounds.height();
                self.bounds = Rect::new(new_x, new_y, new_x + width as i16, new_y + height as i16);

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
                let new_width = (mouse_pos.x + offset.x - self.bounds.a.x) as u16;
                let new_height = (mouse_pos.y + offset.y - self.bounds.a.y) as u16;

                // Apply minimum size constraints (Borland: sizeLimits)
                let final_width = new_width.max(self.min_size.x as u16);
                let final_height = new_height.max(self.min_size.y as u16);

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

        // Handle CM_CLOSE command (Borland: twindow.cc lines 124-138)
        // Frame generates CM_CLOSE when close button is clicked
        // In Borland: TWindow::handleEvent calls close(), which calls valid(cmClose)
        // to allow subclasses to validate (e.g., prompt to save)
        if event.what == EventType::Command && event.command == CM_CLOSE {
            // Check if this window is modal
            if (self.state & SF_MODAL) != 0 {
                // Modal window: convert CM_CLOSE to CM_CANCEL
                // Borland: event.message.command = cmCancel; putEvent(event);
                *event = Event::command(CM_CANCEL);
                // Don't clear event - let it propagate to dialog's execute loop
            } else {
                // Non-modal window: DON'T close immediately
                // In Borland, close() calls valid(cmClose) first
                // Since we don't have subclassing/valid(), let CM_CLOSE propagate
                // to the application level where it can be validated
                // The application will set SF_CLOSED if the close is allowed
                // DON'T clear event - let it bubble to application
            }
            return;
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

    fn get_end_state(&self) -> crate::core::command::CommandId {
        self.interior.get_end_state()
    }

    fn set_end_state(&mut self, command: crate::core::command::CommandId) {
        self.interior.set_end_state(command);
    }
}