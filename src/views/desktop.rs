// (C) 2025 - Enzo Lombardi

//! Desktop view - main workspace for managing application windows.

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

    pub fn add(&mut self, mut view: Box<dyn View>) {
        use crate::core::state::{OF_CENTERED, OF_CENTER_X, OF_CENTER_Y};

        // Apply automatic centering if OF_CENTERED flags are set
        // Matches Borland: TView with ofCentered is centered when inserted
        let options = view.options();
        if (options & OF_CENTERED) != 0 || (options & OF_CENTER_X) != 0 || (options & OF_CENTER_Y) != 0 {
            self.center_view(&mut *view, options);
        }

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

    /// Center a view within the desktop bounds based on its option flags
    /// Matches Borland: Views with ofCentered are automatically centered
    fn center_view(&self, view: &mut dyn View, options: u16) {
        use crate::core::state::{OF_CENTER_X, OF_CENTER_Y};

        let view_bounds = view.bounds();
        let desktop_bounds = self.bounds;

        let mut new_bounds = view_bounds;

        // Center horizontally if OF_CENTER_X is set
        if (options & OF_CENTER_X) != 0 {
            let view_width = view_bounds.width();
            let desktop_width = desktop_bounds.width();
            let center_x = (desktop_width - view_width) / 2;
            new_bounds.a.x = center_x;
            new_bounds.b.x = center_x + view_width;
        }

        // Center vertically if OF_CENTER_Y is set
        if (options & OF_CENTER_Y) != 0 {
            let view_height = view_bounds.height();
            let desktop_height = desktop_bounds.height();
            let center_y = (desktop_height - view_height) / 2;
            new_bounds.a.y = center_y;
            new_bounds.b.y = center_y + view_height;
        }

        view.set_bounds(new_bounds);
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
    /// Get desktop bounds for window operations
    /// Used by windows to determine maximum zoom size
    pub fn get_bounds(&self) -> Rect {
        self.bounds
    }

    /// Cycle to the next window (Borland: selectNext)
    /// Moves the current top window to the back, bringing the next window forward
    /// Matches Borland: cmNext command calls selectNext(False)
    pub fn select_next(&mut self) {
        use crate::core::state::OF_TOP_SELECT;

        // Need at least 2 windows (plus background) to cycle
        if self.children.len() <= 2 {
            return;
        }

        // Get the current top window (last in children list, excluding background)
        let top_window_idx = self.children.len() - 1;

        // Check if top window has OF_TOP_SELECT flag
        let has_top_select = {
            let options = self.children.child_at(top_window_idx).options();
            (options & OF_TOP_SELECT) != 0
        };

        if has_top_select {
            // Move top window behind all others (after background)
            // This is equivalent to Borland's: current->putInFrontOf(background)
            self.children.send_to_back(top_window_idx);
        }
    }

    /// Cycle to the previous window (Borland: selectPrev)
    /// Brings the bottom window to the top
    /// Matches Borland: cmPrev command calls current->putInFrontOf(background)
    pub fn select_prev(&mut self) {
        use crate::core::state::OF_TOP_SELECT;

        // Need at least 2 windows (plus background) to cycle
        if self.children.len() <= 2 {
            return;
        }

        // Get the bottom window (right after background)
        let bottom_window_idx = 1;

        // Check if it has OF_TOP_SELECT flag
        let has_top_select = {
            let options = self.children.child_at(bottom_window_idx).options();
            (options & OF_TOP_SELECT) != 0
        };

        if has_top_select {
            // Bring bottom window to front
            self.children.bring_to_front(bottom_window_idx);
        }
    }

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
        // When a window is clicked, bring it to the front if it has OF_TOP_SELECT flag
        // Matches Borland: TView::handleEvent() calls focus() -> select() -> makeFirst() if ofTopSelect set
        if !has_modal && event.what == EventType::MouseDown {
            use crate::core::state::OF_TOP_SELECT;
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
            // Only if the window has OF_TOP_SELECT flag set (matches Borland: ofTopSelect)
            if let Some(window_idx) = clicked_window {
                let last_idx = self.children.len() - 1;
                if window_idx != last_idx {
                    let window_options = self.children.child_at(window_idx).options();
                    if (window_options & OF_TOP_SELECT) != 0 {
                        // Bring window to front (Borland: makeFirst())
                        self.children.bring_to_front(window_idx);
                        // Note: We don't return here - let the event propagate to the window
                    }
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
