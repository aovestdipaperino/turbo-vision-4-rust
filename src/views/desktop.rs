// (C) 2025 - Enzo Lombardi
use crate::core::geometry::Rect;
use crate::core::event::Event;
use crate::core::palette::colors;
use crate::terminal::Terminal;
use super::view::View;
use super::group::Group;
use super::background::Background;

pub struct Desktop {
    bounds: Rect,
    children: Group,
}

impl Desktop {
    pub fn new(bounds: Rect) -> Self {
        let mut children = Group::new(bounds);

        // Add background as first child (matches Borland's TDeskTop::TDeskTop)
        // Background fills the entire desktop area
        // NOTE: Must use relative coordinates (0, 0) because Group.add() converts to absolute
        let width = bounds.width();
        let height = bounds.height();
        let background_bounds = Rect::new(0, 0, width, height);
        let background = Box::new(Background::new(background_bounds, '░', colors::DESKTOP));
        children.add(background);

        Self {
            bounds,
            children,
        }
    }

    pub fn add(&mut self, view: Box<dyn View>) {
        self.children.add(view);
        // Focus on the newly added window (last child)
        let num_children = self.children.len();
        if num_children > 0 {
            let last_idx = num_children - 1;
            if self.children.child_at(last_idx).can_focus() {
                // Clear focus from all children first
                self.children.clear_all_focus();
                // Then give focus to the new window
                self.children.set_focus_to(last_idx);
            }
        }
    }

    /// Get the number of child views (windows) on the desktop
    /// Note: Subtracts 1 because the background is also a child
    pub fn child_count(&self) -> usize {
        self.children.len().saturating_sub(1)
    }

    /// Get a reference to a child view by index
    /// Note: Index 0 refers to the first window (background is at internal index 0)
    pub fn child_at(&self, index: usize) -> &dyn View {
        self.children.child_at(index + 1)  // +1 to skip background
    }

    /// Remove a child view by index
    /// Note: Index 0 refers to the first window (background is at internal index 0)
    /// Used by Application::exec_view() to remove modal dialogs after they close
    pub fn remove_child(&mut self, index: usize) {
        self.children.remove(index + 1);  // +1 to skip background
    }

    /// Draw views in the affected rectangle (Borland's drawUnderRect pattern)
    /// This is called when a window moves to redraw only the affected area
    /// Matches Borland: TView::drawUnderRect() (tview.cc:304-308)
    pub fn draw_under_rect(&mut self, terminal: &mut Terminal, rect: Rect, start_from_window: usize) {
        // +1 to account for background being at index 0
        let start_index = start_from_window + 1;

        // Draw background in the affected rect first
        terminal.push_clip(rect);
        self.children.child_at_mut(0).draw(terminal);
        terminal.pop_clip();

        // Then draw all windows from start_index onwards in the affected rect
        self.children.draw_sub_views(terminal, start_index, rect);
    }

    /// Check for moved windows and redraw affected areas
    /// Matches Borland: TProgram::idle() checks for moved views and calls drawUnderRect
    /// This is called after event handling to redraw areas exposed by window movement
    /// Returns true if any windows were moved and redrawn
    pub fn handle_moved_windows(&mut self, terminal: &mut Terminal) -> bool {
        let mut had_movement = false;

        // Check each window (skip background at index 0)
        // We iterate in reverse because we need to check from front to back (z-order)
        for i in 1..self.children.len() {
            // Check if this view has moved
            if let Some(union_rect) = self.children.child_at(i).get_redraw_union() {
                // This window moved - redraw the union rect area
                // Start from the moved window's position (all views behind it)
                // Matches Borland: TView::locate() → TView::drawUnderRect()
                self.draw_under_rect(terminal, union_rect, i - 1); // -1 because Desktop uses window indices, not internal indices

                // Clear the movement tracking after redrawing
                self.children.child_at_mut(i).clear_move_tracking();

                had_movement = true;
            }
        }

        had_movement
    }
}

impl Desktop {
    /// Get a mutable reference to a window by index (for movement tracking)
    /// Returns None if index is out of bounds
    /// Index 0 refers to first window (background is at internal index 0)
    pub fn window_at_mut(&mut self, index: usize) -> Option<&mut dyn View> {
        let internal_index = index + 1; // +1 to skip background
        if internal_index < self.children.len() {
            Some(self.children.child_at_mut(internal_index))
        } else {
            None
        }
    }

    /// Remove closed windows (those with SF_CLOSED flag)
    /// In Borland, views call CLY_destroy() which removes them from the owner
    /// In Rust, views set SF_CLOSED flag and the parent removes them
    /// This is called after event handling in the main loop
    /// Returns true if any windows were removed
    pub fn remove_closed_windows(&mut self) -> bool {
        use crate::core::state::SF_CLOSED;

        let mut had_removals = false;

        // Remove windows marked as closed (skip background at index 0)
        // We need to iterate in reverse to avoid index shifting issues
        let mut i = self.children.len();
        while i > 1 {  // Don't remove background at index 0
            i -= 1;
            if (self.children.child_at(i).state() & SF_CLOSED) != 0 {
                self.children.remove(i);
                had_removals = true;
            }
        }

        had_removals
    }

    /// Get the first window as a Window type (for editor demo use case)
    /// This is a pragmatic helper that uses unsafe downcasting
    /// Returns None if there are no windows
    pub fn get_first_window_as_window(&self) -> Option<&crate::views::window::Window> {
        if self.child_count() == 0 {
            return None;
        }

        let view_ref = self.child_at(0);
        let view_ptr = view_ref as *const dyn View;

        // SAFETY: In the editor demo, we know the first child is always a Window
        unsafe {
            let window_ptr = view_ptr as *const crate::views::window::Window;
            if !window_ptr.is_null() {
                Some(&*window_ptr)
            } else {
                None
            }
        }
    }

    /// Get the first window as a mutable Window type (for editor demo use case)
    /// This is a pragmatic helper that uses unsafe downcasting
    /// Returns None if there are no windows
    pub fn get_first_window_as_window_mut(&mut self) -> Option<&mut crate::views::window::Window> {
        if self.child_count() == 0 {
            return None;
        }

        let view_ref = self.window_at_mut(0).unwrap();
        let view_ptr = view_ref as *mut dyn View;

        // SAFETY: In the editor demo, we know the first child is always a Window
        unsafe {
            let window_ptr = view_ptr as *mut crate::views::window::Window;
            if !window_ptr.is_null() {
                Some(&mut *window_ptr)
            } else {
                None
            }
        }
    }
}

impl View for Desktop {
    fn bounds(&self) -> Rect {
        self.bounds
    }

    fn set_bounds(&mut self, bounds: Rect) {
        self.bounds = bounds;
        self.children.set_bounds(bounds);
    }

    fn draw(&mut self, terminal: &mut Terminal) {
        // Just draw all children (background is the first child, windows come after)
        // This matches Borland's TDeskTop which is a TGroup with TBackground as first child
        self.children.draw(terminal);
    }

    fn handle_event(&mut self, event: &mut Event) {
        use crate::core::event::EventType;
        use crate::core::state::SF_MODAL;

        // Check if the topmost window is modal
        // Modal windows capture all events - clicks on other windows have no effect
        // Matches Borland: TGroup::execView() creates modal scope
        let has_modal = if self.children.len() > 1 {
            let top_window_idx = self.children.len() - 1;
            (self.children.child_at(top_window_idx).state() & SF_MODAL) != 0
        } else {
            false
        };

        // Handle z-order changes on mouse down (only when no modal window is present)
        // When a window is clicked, bring it to the front (unless it's already on top)
        // Matches Borland: TGroup::selectView() called on mouse events
        if !has_modal && event.what == EventType::MouseDown {
            let mouse_pos = event.mouse.pos;

            // Find which window was clicked (search in reverse z-order, skip background at 0)
            let mut clicked_window: Option<usize> = None;
            for i in (1..self.children.len()).rev() {
                let child_bounds = self.children.child_at(i).bounds();
                if child_bounds.contains(mouse_pos) {
                    clicked_window = Some(i);
                    break;
                }
            }

            // If a window was clicked and it's not already on top, bring it to front
            if let Some(window_idx) = clicked_window {
                let last_idx = self.children.len() - 1;
                if window_idx != last_idx {
                    // Bring window to front
                    self.children.bring_to_front(window_idx);
                    // Note: We don't return here - let the event propagate to the window
                }
            }
        }

        // If there's a modal window, only send events to it
        // Matches Borland: modal views block events to views behind them
        if has_modal {
            let modal_idx = self.children.len() - 1;
            self.children.child_at_mut(modal_idx).handle_event(event);
        } else {
            self.children.handle_event(event);
        }
    }
}
