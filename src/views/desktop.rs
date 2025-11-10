// (C) 2025 - Enzo Lombardi

//! Desktop view - main workspace for managing application windows.

use crate::core::geometry::Rect;
use crate::core::event::Event;
use crate::terminal::Terminal;
use super::view::View;
use super::group::Group;
use super::background::Background;

pub struct Desktop {
    bounds: Rect,
    children: Group,
    owner: Option<*const dyn View>,
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
        let background = Box::new(Background::new(background_bounds, '░', crate::core::palette::colors::DESKTOP));
        children.add(background);

        Self {
            bounds,
            children,
            owner: None,
        }
    }

    /// Initialize the palette chain after Desktop is in its final memory location.
    /// Must be called after Desktop is constructed and in a stable location (not moved).
    /// Matches Borland: Desktop is the root of the palette chain with CP_APP_COLOR.
    pub fn init_palette_chain(&mut self) {
        // Set children's owner to this Desktop (for palette chain)
        // Desktop provides CP_APP_COLOR palette, making it the palette root
        // NOTE: We don't set owner pointer to avoid unsafe casting
    }

    pub fn add(&mut self, mut view: Box<dyn View>) -> usize {
        use crate::core::state::{OF_CENTERED, OF_CENTER_X, OF_CENTER_Y};

        // Set owner pointer to Desktop (for drag limits and bounds checking)
        // Matches Borland: TGroup::insert() sets view->owner = this
        // Safety: Desktop's address is stable for the lifetime of the view
        view.set_owner(self as *const dyn View);

        // Apply automatic centering if OF_CENTERED flags are set
        // Matches Borland: TView with ofCentered is centered when inserted
        let options = view.options();
        if (options & OF_CENTERED) != 0 || (options & OF_CENTER_X) != 0 || (options & OF_CENTER_Y) != 0 {
            self.center_view(&mut *view, options);
        }

        // Constrain window to Desktop bounds (prevents centering from placing window out of bounds)
        // This ensures windows with shadows don't extend below status bar
        // Matches Borland: TView::locate() constrains position to owner bounds
        view.constrain_to_parent_bounds();

        let index = self.children.add(view);

        // Initialize internal owner pointers after view is in final position
        // This is critical for views like Dialog that contain Groups by value
        let num_children = self.children.len();
        if num_children > 0 {
            let last_idx = num_children - 1;
            self.children.child_at_mut(last_idx).init_after_add();
        }

        // Focus on the newly added window (last child)
        if num_children > 0 {
            let last_idx = num_children - 1;
            if self.children.child_at(last_idx).can_focus() {
                // Clear focus from all children first
                self.children.clear_all_focus();
                // Then give focus to the new window
                self.children.set_focus_to(last_idx);
            }
        }
        index
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

    /// Get a mutable reference to a child view by index
    /// Note: Index 0 refers to the first window (background is at internal index 0)
    pub fn child_at_mut(&mut self, index: usize) -> &mut dyn View {
        self.children.child_at_mut(index + 1)  // +1 to skip background
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

    /// Cascade windows in a staircase pattern
    /// Matches Borland: TDesktop::cascade(const TRect &r)
    pub fn cascade(&mut self) {
        use crate::core::state::OF_TILEABLE;

        // Count tileable windows (skip background at index 0)
        let mut count = 0;
        for i in 1..self.children.len() {
            let child = self.children.child_at(i);
            let options = child.options();
            if (options & OF_TILEABLE) != 0 {
                count += 1;
            }
        }

        if count == 0 {
            return;
        }

        // Calculate cascade bounds (leave room for offset)
        let cascade_bounds = self.bounds;

        // Position windows in cascade (staircase) pattern
        // Each window is offset by its index from the base position
        let mut cascade_index: usize = count - 1;
        for i in 1..self.children.len() {
            let child = self.children.child_at(i);
            let options = child.options();
            if (options & OF_TILEABLE) != 0 {
                // Calculate new bounds with cascade offset
                let mut new_bounds = cascade_bounds;
                new_bounds.a.x += cascade_index as i16;
                new_bounds.a.y += cascade_index as i16;

                // Adjust size to account for offset (so window fits in desktop)
                new_bounds.b.x -= (count - 1 - cascade_index) as i16;
                new_bounds.b.y -= (count - 1 - cascade_index) as i16;

                self.children.child_at_mut(i).set_bounds(new_bounds);
                cascade_index = cascade_index.saturating_sub(1);
            }
        }
    }

    /// Tile windows in a grid pattern
    /// Matches Borland: TDesktop::tile(const TRect &r)
    pub fn tile(&mut self) {
        use crate::core::state::OF_TILEABLE;

        // Count tileable windows (skip background at index 0)
        let mut count = 0;
        for i in 1..self.children.len() {
            let child = self.children.child_at(i);
            let options = child.options();
            if (options & OF_TILEABLE) != 0 {
                count += 1;
            }
        }

        if count == 0 {
            return;
        }

        // Calculate grid dimensions (most square layout)
        let (cols, rows) = Self::calculate_grid_layout(count);

        let tile_bounds = self.bounds;
        let cell_width = tile_bounds.width() / cols as i16;
        let cell_height = tile_bounds.height() / rows as i16;

        // Position windows in grid
        let mut tile_index = 0;
        for i in 1..self.children.len() {
            let child = self.children.child_at(i);
            let options = child.options();
            if (options & OF_TILEABLE) != 0 {
                let col = tile_index % cols;
                let row = tile_index / cols;

                let new_bounds = Rect::new(
                    tile_bounds.a.x + (col as i16 * cell_width),
                    tile_bounds.a.y + (row as i16 * cell_height),
                    tile_bounds.a.x + ((col + 1) as i16 * cell_width),
                    tile_bounds.a.y + ((row + 1) as i16 * cell_height),
                );

                self.children.child_at_mut(i).set_bounds(new_bounds);
                tile_index += 1;
            }
        }
    }

    /// Calculate grid layout (rows x cols) that's most square
    /// Matches Borland: mostEqualDivisors()
    fn calculate_grid_layout(count: usize) -> (usize, usize) {
        if count == 0 {
            return (1, 1);
        }

        // Find the square root (approximately)
        let sqrt = (count as f64).sqrt() as usize;

        // Find divisors closest to square root
        let mut cols = sqrt;
        while count % cols != 0 && cols > 1 {
            cols -= 1;
        }

        if cols == 1 {
            // Prime number or couldn't find good divisor
            cols = sqrt;
            if cols * cols < count {
                cols += 1;
            }
        }

        let rows = (count + cols - 1) / cols; // Ceiling division

        (cols, rows)
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

    /// Zoom the topmost window
    /// Matches Borland: Desktop handles cmZoom and calls window->zoom()
    /// In Borland, TWindow::zoom() calls sizeLimits() which gets owner->size
    pub fn zoom_top_window(&mut self) {
        // Get the topmost window (last in children list, excluding background)
        if self.children.len() <= 1 {
            return; // No windows to zoom
        }

        let top_window_idx = self.children.len() - 1;

        // Call zoom on the topmost view (typically a Window)
        // This matches Borland: owner handles cmZoom, calls window->zoom()
        // window->zoom() uses sizeLimits() which returns owner->size as max
        // We pass desktop bounds (equivalent to owner->size in Borland)
        let desktop_bounds = self.bounds;
        self.children.child_at_mut(top_window_idx).zoom(desktop_bounds);
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

    fn set_owner(&mut self, owner: *const dyn View) {
        self.owner = Some(owner);
    }

    fn get_owner(&self) -> Option<*const dyn View> {
        self.owner
    }

    fn get_palette(&self) -> Option<crate::core::palette::Palette> {
        use crate::core::palette::{Palette, palettes};
        // Desktop uses the application palette directly (no remapping)
        Some(Palette::from_slice(palettes::CP_APP_COLOR))
    }
}

/// Builder for creating desktops with a fluent API.
pub struct DesktopBuilder {
    bounds: Option<Rect>,
}

impl DesktopBuilder {
    pub fn new() -> Self {
        Self { bounds: None }
    }

    #[must_use]
    pub fn bounds(mut self, bounds: Rect) -> Self {
        self.bounds = Some(bounds);
        self
    }

    pub fn build(self) -> Desktop {
        let bounds = self.bounds.expect("Desktop bounds must be set");
        Desktop::new(bounds)
    }

    pub fn build_boxed(self) -> Box<Desktop> {
        Box::new(self.build())
    }
}

impl Default for DesktopBuilder {
    fn default() -> Self {
        Self::new()
    }
}
