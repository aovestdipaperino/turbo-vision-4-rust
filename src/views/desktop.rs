// (C) 2025 - Enzo Lombardi

//! Desktop view - main workspace for managing application windows.

use super::background::Background;
use super::group::{Group, GroupLike};
use super::view::{View, ViewCore, ViewId};
use crate::core::event::Event;
use crate::core::geometry::Rect;
use crate::core::state::Options;
use crate::core::state::State;
use crate::terminal::Terminal;

pub struct Desktop {
    core: ViewCore,
    children: Group,
}

impl Desktop {
    pub fn new(bounds: Rect) -> Self {
        // The child group fills the desktop's own space, so its children are
        // desktop-relative and it draws without a further origin push.
        let width = bounds.width();
        let height = bounds.height();
        let mut children = Group::new(Rect::new(0, 0, width, height));

        // Add background as first child (matches Borland's TDeskTop::TDeskTop)
        let background_bounds = Rect::new(0, 0, width, height);
        let background = Box::new(Background::new(
            background_bounds,
            '░',
            crate::core::palette::colors::DESKTOP,
        ));
        children.add(background);

        Self {
            core: ViewCore {
                bounds,
                palette_chain: None,
                ..ViewCore::default()
            },
            children,
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

    /// Add a window (Borland: `TDeskTop::insert`). Takes any view.
    pub fn add<V: View + 'static>(&mut self, view: V) -> ViewId {
        let mut view: Box<dyn View> = Box::new(view);

        // Drag and resize limits are the desktop's own extent
        view.set_owner_extent(self.extent());

        // Apply automatic centering if Options::CENTERED flags are set
        // Matches Borland: TView with ofCentered is centered when inserted
        let options = view.options();
        if options.contains(Options::CENTERED)
            || options.contains(Options::CENTER_X)
            || options.contains(Options::CENTER_Y)
        {
            self.center_view(&mut *view, options);
        }

        let view_id = self.children.add_boxed(view);

        // Keep the window (and its shadow) inside the desktop
        let num_children = self.children.len();
        if num_children > 0 {
            let last_idx = num_children - 1;
            self.children
                .child_at_mut(last_idx)
                .constrain_to_parent_bounds();
        }

        // Initialize internal owner pointers after view is in final position
        // This is critical for views like Dialog that contain Groups by value
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
        view_id
    }

    /// Center a view within the desktop bounds based on its option flags
    /// Matches Borland: Views with ofCentered are automatically centered
    fn center_view(&self, view: &mut dyn View, options: Options) {
        let view_bounds = view.bounds();
        let desktop_bounds = self.core.bounds;

        let mut new_bounds = view_bounds;

        // Center horizontally if Options::CENTER_X is set
        if options.contains(Options::CENTER_X) {
            let view_width = view_bounds.width();
            let desktop_width = desktop_bounds.width();
            let center_x = (desktop_width - view_width) / 2;
            new_bounds.a.x = center_x;
            new_bounds.b.x = center_x + view_width;
        }

        // Center vertically if Options::CENTER_Y is set
        if options.contains(Options::CENTER_Y) {
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
        self.children.child_at(index + 1) // +1 to skip background
    }

    /// Get a mutable reference to a child view by index
    /// Note: Index 0 refers to the first window (background is at internal index 0)
    pub fn child_at_mut(&mut self, index: usize) -> &mut dyn View {
        self.children.child_at_mut(index + 1) // +1 to skip background
    }

    /// True if a window with this `ViewId` is currently a child of the desktop.
    /// Lets owners that hold a saved id (e.g. an IDE that wants to know when a
    /// re-openable panel like Watches/Output has been closed) check presence
    /// without iterating + downcasting.
    pub fn contains_id(&self, view_id: ViewId) -> bool {
        self.children.child_by_id(view_id).is_some()
    }

    /// Get the `ViewId` of the topmost (most recently added) window, if any.
    pub fn top_view_id(&self) -> Option<ViewId> {
        let count = self.child_count();
        if count == 0 {
            return None;
        }
        self.children.view_id_at(count) // +1 background offset baked in
    }

    /// Get a child view by its `ViewId`.
    pub fn child_by_id(&self, view_id: ViewId) -> Option<&dyn View> {
        self.children.child_by_id(view_id)
    }

    /// Add a window and get a typed handle to it back; see `GroupLike::add_typed`.
    pub fn add_typed<T: View + 'static>(&mut self, view: T) -> super::handle::Handle<T> {
        super::handle::Handle::from_id(self.add(view))
    }

    /// The window behind a typed handle, if it is still on the desktop.
    pub fn get<T: View + 'static>(&self, handle: super::handle::Handle<T>) -> Option<&T> {
        self.children
            .child_by_id(handle.id())?
            .as_any()
            .downcast_ref::<T>()
    }

    pub fn get_mut<T: View + 'static>(
        &mut self,
        handle: super::handle::Handle<T>,
    ) -> Option<&mut T> {
        self.children
            .child_by_id_mut(handle.id())?
            .as_any_mut()
            .downcast_mut::<T>()
    }

    /// The desktop's child list as a group, for owner-side bookkeeping such
    /// as delivering a history selection to the linked input.
    pub(crate) fn children_mut(&mut self) -> &mut Group {
        &mut self.children
    }

    /// Remove a child view by its `ViewId`. Returns true if it was present.
    pub fn remove_child_by_id(&mut self, view_id: ViewId) -> bool {
        self.children.remove_by_id(view_id)
    }

    /// Get a mutable child view by its `ViewId`.
    pub fn child_by_id_mut(&mut self, view_id: ViewId) -> Option<&mut (dyn View + '_)> {
        self.children.child_by_id_mut(view_id)
    }

    /// Remove a child view by index
    /// Note: Index 0 refers to the first window (background is at internal index 0)
    /// Used by Application::exec_view() to remove modal dialogs after they close
    pub fn remove_child(&mut self, index: usize) {
        self.children.remove(index + 1); // +1 to skip background
    }

    /// Draw views in the affected rectangle (Borland's drawUnderRect pattern)
    /// This is called when a window moves to redraw only the affected area
    /// Matches Borland: TView::drawUnderRect() (tview.cc:304-308)
    pub fn draw_under_rect(
        &mut self,
        terminal: &mut Terminal,
        rect: Rect,
        start_from_window: usize,
    ) {
        // +1 to account for background being at index 0
        let start_index = start_from_window + 1;

        // Draw background in the affected rect first
        terminal.push_clip(rect);
        super::view::draw_child(terminal, self.children.child_at_mut(0));
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
        self.core.bounds
    }

    /// Check if any child window is tileable
    /// Used for enabling/disabling tile/cascade commands
    /// Matches Borland: deskTop->firstThat(isTileable, 0) != 0
    pub fn has_tileable_windows(&self) -> bool {
        // Skip background (index 0)
        for i in 1..self.children.len() {
            let child = self.children.child_at(i);
            if child.options().contains(Options::TILEABLE) {
                return true;
            }
        }
        false
    }

    /// Count tileable windows on desktop
    /// Used for tile/cascade algorithms
    pub fn count_tileable_windows(&self) -> usize {
        let mut count = 0;
        for i in 1..self.children.len() {
            let child = self.children.child_at(i);
            if child.options().contains(Options::TILEABLE) {
                count += 1;
            }
        }
        count
    }

    /// Cascade windows in a staircase pattern (using full desktop bounds)
    /// Matches Borland: TDesktop::cascade(const TRect &r)
    pub fn cascade(&mut self) {
        self.cascade_with_rect(self.extent());
    }

    /// Cascade windows in a staircase pattern within specified rect
    /// Matches Borland: TDesktop::cascade(const TRect &r)
    pub fn cascade_with_rect(&mut self, rect: Rect) {
        // Count tileable windows (skip background at index 0)
        let mut count = 0;
        for i in 1..self.children.len() {
            let child = self.children.child_at(i);
            let options = child.options();
            if options.contains(Options::TILEABLE) {
                count += 1;
            }
        }

        if count == 0 {
            return;
        }

        // Calculate cascade bounds (leave room for offset)
        let cascade_bounds = rect;

        // Position windows in cascade (staircase) pattern
        // Bottom window (first in z-order) has no offset (leftmost)
        // Top window (last in z-order) has maximum offset (rightmost)
        let mut cascade_index: usize = 0;
        for i in 1..self.children.len() {
            let child = self.children.child_at(i);
            let options = child.options();
            if options.contains(Options::TILEABLE) {
                // Matches Borland doCascade: each window's origin steps down
                // the staircase while every window extends to the rect's
                // bottom-right corner
                let mut new_bounds = cascade_bounds;
                new_bounds.a.x += cascade_index as i16;
                new_bounds.a.y += cascade_index as i16;

                self.children.child_at_mut(i).set_bounds(new_bounds);
                cascade_index += 1;
            }
        }
    }

    /// Tile windows in a grid pattern (using full desktop bounds)
    /// Matches Borland: TDesktop::tile(const TRect &r)
    pub fn tile(&mut self) {
        self.tile_with_rect(self.extent());
    }

    /// Tile windows in a grid pattern within specified rect
    /// Matches Borland: TDesktop::tile(const TRect &r)
    pub fn tile_with_rect(&mut self, rect: Rect) {
        // Count tileable windows (skip background at index 0)
        let mut count = 0;
        for i in 1..self.children.len() {
            let child = self.children.child_at(i);
            let options = child.options();
            if options.contains(Options::TILEABLE) {
                count += 1;
            }
        }

        if count == 0 {
            return;
        }

        // Borland's exact-fill layout: mostEqualDivisors picks the grid,
        // leftover windows get an extra row in the first columns, and
        // dividerLoc spreads remainder cells so the rect is covered exactly
        let (num_cols, num_rows) = Self::most_equal_divisors(count, false);
        let left_over = count % num_cols;

        let mut tile_index = 0;
        for i in 1..self.children.len() {
            let child = self.children.child_at(i);
            let options = child.options();
            if options.contains(Options::TILEABLE) {
                let new_bounds =
                    Self::calc_tile_rect(tile_index, rect, num_cols, num_rows, left_over);
                self.children.child_at_mut(i).set_bounds(new_bounds);
                tile_index += 1;
            }
        }
    }

    /// Split `[lo, hi)` into `num` near-equal parts; boundary of part `pos`.
    /// Matches Borland: dividerLoc()
    fn divider_loc(lo: i16, hi: i16, num: usize, pos: usize) -> i16 {
        lo + ((hi - lo) as i32 * pos as i32 / num as i32) as i16
    }

    /// Grid dimensions `(cols, rows)` as equal as possible.
    /// Matches Borland: mostEqualDivisors()
    fn most_equal_divisors(n: usize, favor_y: bool) -> (usize, usize) {
        let mut i = (n as f64).sqrt() as usize;
        if i == 0 {
            i = 1;
        }
        if n % i != 0 && n % (i + 1) == 0 {
            i += 1;
        }
        if i < n / i {
            i = n / i;
        }
        if favor_y { (n / i, i) } else { (i, n / i) }
    }

    /// Tile cell for window `pos`. Matches Borland: calcTileRect()
    fn calc_tile_rect(
        pos: usize,
        r: Rect,
        num_cols: usize,
        num_rows: usize,
        left_over: usize,
    ) -> Rect {
        let d = (num_cols - left_over) * num_rows;
        let (x, y, rows_here) = if pos < d {
            (pos / num_rows, pos % num_rows, num_rows)
        } else {
            (
                (pos - d) / (num_rows + 1) + (num_cols - left_over),
                (pos - d) % (num_rows + 1),
                num_rows + 1,
            )
        };
        Rect::new(
            Self::divider_loc(r.a.x, r.b.x, num_cols, x),
            Self::divider_loc(r.a.y, r.b.y, rows_here, y),
            Self::divider_loc(r.a.x, r.b.x, num_cols, x + 1),
            Self::divider_loc(r.a.y, r.b.y, rows_here, y + 1),
        )
    }

    /// Cycle to the next window (Borland: selectNext)
    /// Moves the current top window to the back, bringing the next window forward
    /// Matches Borland: cmNext command calls selectNext(False)
    pub fn select_next(&mut self) {
        // Need at least 2 windows (plus background) to cycle
        if self.children.len() <= 2 {
            return;
        }

        // Get the current top window (last in children list, excluding background)
        let top_window_idx = self.children.len() - 1;

        // Check if top window has Options::TOP_SELECT flag
        let has_top_select = {
            let options = self.children.child_at(top_window_idx).options();
            options.contains(Options::TOP_SELECT)
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
        // Need at least 2 windows (plus background) to cycle
        if self.children.len() <= 2 {
            return;
        }

        // Get the bottom window (right after background)
        let bottom_window_idx = 1;

        // Check if it has Options::TOP_SELECT flag
        let has_top_select = {
            let options = self.children.child_at(bottom_window_idx).options();
            options.contains(Options::TOP_SELECT)
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
        // We pass the desktop extent (owner->size in Borland)
        let desktop_extent = self.extent();
        self.children
            .child_at_mut(top_window_idx)
            .zoom(desktop_extent);
    }

    /// Bring a specific window to the front of the Z-order by its ViewId.
    /// Returns true if the window was found and moved, false if not found.
    /// Matches Borland: TView::makeFirst() / TView::select()
    pub fn bring_to_front(&mut self, view_id: ViewId) -> bool {
        // Search children (skip background at index 0) for matching ViewId
        let found_index =
            (1..self.children.len()).find(|&i| self.children.view_id_at(i) == Some(view_id));

        let index = match found_index {
            Some(i) => i,
            None => return false,
        };

        let last_idx = self.children.len() - 1;
        if index == last_idx {
            return true; // Already on top
        }

        // Unfocus current top window
        self.children.child_at_mut(last_idx).set_focus(false);

        // Bring the target window to front
        self.children.bring_to_front(index);

        // Focus the new top window
        let new_top = self.children.len() - 1;
        self.children.child_at_mut(new_top).set_focus(true);

        true
    }

    /// Remove closed windows (those with State::CLOSED flag)
    /// In Borland, views call CLY_destroy() which removes them from the owner
    /// In Rust, views set State::CLOSED flag and the parent removes them
    /// This is called after event handling in the main loop
    /// Returns true if any windows were removed
    /// Returns the ids of the windows removed, so the caller can tell which
    /// ones went away (`AppHandler::window_closed`); empty when nothing closed.
    pub fn remove_closed_windows(&mut self) -> Vec<ViewId> {
        let mut removed = Vec::new();

        // Remove windows marked as closed (skip background at index 0)
        // We need to iterate in reverse to avoid index shifting issues
        let mut i = self.children.len();
        while i > 1 {
            // Don't remove background at index 0
            i -= 1;
            if self.children.child_at(i).state().contains(State::CLOSED) {
                if let Some(id) = self.children.view_id_at(i) {
                    removed.push(id);
                }
                self.children.remove(i);
            }
        }

        removed
    }
}

impl View for Desktop {
    fn core(&self) -> &ViewCore {
        &self.core
    }

    fn core_mut(&mut self) -> &mut ViewCore {
        &mut self.core
    }

    fn set_bounds(&mut self, bounds: Rect) {
        self.core.bounds = bounds;
        self.children.set_bounds(self.extent());
    }

    fn draw(&mut self, terminal: &mut Terminal) {
        // Just draw all children (background is the first child, windows come after)
        // This matches Borland's TDeskTop which is a TGroup with TBackground as first child
        self.children.draw(terminal);
    }

    fn handle_event(&mut self, event: &mut Event) {
        use crate::core::event::EventType;

        // cmZoom toggles the top window between zoomed and saved bounds
        // (Borland: TWindow::handleEvent cmZoom; here the desktop owns the
        // maximum extent)
        if event.what == EventType::Command && event.command == crate::core::command::CM_ZOOM {
            self.zoom_top_window();
            event.clear();
            return;
        }

        // Alt+1..9 window selection (Borland: TWindow handles
        // cmSelectWindowNum; here the desktop resolves the number since it
        // owns z-order and focus)
        if event.what == EventType::Broadcast
            && event.command == crate::core::command::CM_SELECT_WINDOW_NUM
        {
            let wanted = event.info as u8;
            let count = self.child_count();
            for i in 0..count {
                if self.child_at(i).window_number() == Some(wanted) {
                    if let Some(id) = self.children.view_id_at(i + 1) {
                        self.bring_to_front(id);
                        event.clear();
                    }
                    return;
                }
            }
            return;
        }

        // Check if the topmost window is modal
        // Modal windows capture all events - clicks on other windows have no effect
        // Matches Borland: TGroup::execView() creates modal scope
        let has_modal = if self.children.len() > 1 {
            let top_window_idx = self.children.len() - 1;
            self.children
                .child_at(top_window_idx)
                .state()
                .contains(State::MODAL)
        } else {
            false
        };

        // Handle z-order changes on mouse down (only when no modal window is present)
        // When a window is clicked, bring it to the front if it has Options::TOP_SELECT flag
        // Matches Borland: TView::handleEvent() calls focus() -> select() -> makeFirst() if ofTopSelect set
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
            // Only if the window has Options::TOP_SELECT flag set (matches Borland: ofTopSelect)
            if let Some(window_idx) = clicked_window {
                let last_idx = self.children.len() - 1;
                if window_idx != last_idx {
                    let window_options = self.children.child_at(window_idx).options();
                    if window_options.contains(Options::TOP_SELECT) {
                        // Bring window to front (Borland: makeFirst())
                        self.children.bring_to_front(window_idx);
                        // Note: We don't return here - let the event propagate to the window
                    }
                }
            }
        }

        // Handle desktop-level commands
        // Matches Borland: TDesktop::handleEvent (tdesktop.cc:103-133)
        if event.what == EventType::Command {
            use crate::core::command::{CM_NEXT, CM_PREV};

            match event.command {
                CM_NEXT => {
                    // Cycle to next window (send top window to back)
                    // Matches Borland: cmNext command calls selectNext(False)
                    if self.children.len() > 2 {
                        // Clear focus from current top window
                        let old_top_idx = self.children.len() - 1;
                        let old_state = self.children.child_at(old_top_idx).state();
                        if old_state.contains(State::FOCUSED) {
                            self.children.child_at_mut(old_top_idx).set_focus(false);
                        }

                        // Move top window to back
                        self.children.send_to_back(old_top_idx);

                        // Focus the new top window
                        let new_top_idx = self.children.len() - 1;
                        self.children.child_at_mut(new_top_idx).set_focus(true);
                    }
                    event.clear();
                    return;
                }
                CM_PREV => {
                    // Cycle to previous window (bring bottom window to front)
                    // Matches Borland: cmPrev calls current->putInFrontOf(background)
                    if self.children.len() > 2 {
                        // Clear focus from current top window
                        let old_top_idx = self.children.len() - 1;
                        let old_state = self.children.child_at(old_top_idx).state();
                        if old_state.contains(State::FOCUSED) {
                            self.children.child_at_mut(old_top_idx).set_focus(false);
                        }

                        // Bring bottom window (after background) to front
                        self.children.bring_to_front(1);

                        // Focus the new top window
                        let new_top_idx = self.children.len() - 1;
                        self.children.child_at_mut(new_top_idx).set_focus(true);
                    }
                    event.clear();
                    return;
                }
                _ => {}
            }
        }

        // If there's a modal window, only send events to it
        // Matches Borland: modal views block events to views behind them
        if has_modal {
            let modal_idx = self.children.len() - 1;
            super::view::dispatch_to_child(self.children.child_at_mut(modal_idx), event);
        } else {
            self.children.handle_event(event);
        }
    }

    fn get_palette(&self) -> Option<crate::core::palette::Palette> {
        use crate::core::palette::{Palette, palettes};
        // Desktop uses the application palette directly (no remapping)
        let app_palette_data = palettes::get_app_palette();
        Some(Palette::from_slice(&app_palette_data))
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::views::window::Window;

    /// The background must follow the desktop's bottom-right corner on a
    /// resize (Borland: TBackground's gfGrowHiX | gfGrowHiY). Before the
    /// grow mode was set, `Group::set_bounds` only translated the
    /// background, so a widened terminal left its new columns unpainted -
    /// a black band down the right edge of the desktop.
    #[test]
    fn background_follows_the_desktop_on_resize() {
        let mut desktop = Desktop::new(Rect::new(0, 0, 78, 25));
        assert_eq!(
            desktop.children.child_at(0).bounds(),
            Rect::new(0, 0, 78, 25)
        );

        // Widen and shorten, the way Application::update_desktop_bounds
        // does once a menu bar and a status line take a row each.
        desktop.set_bounds(Rect::new(0, 1, 80, 24));
        assert_eq!(
            desktop.children.child_at(0).bounds(),
            Rect::new(0, 0, 80, 23),
            "background did not track the resized desktop"
        );

        // And back the other way: a narrower terminal must not leave the
        // background hanging past the desktop's edges either.
        desktop.set_bounds(Rect::new(0, 1, 60, 20));
        assert_eq!(
            desktop.children.child_at(0).bounds(),
            Rect::new(0, 0, 60, 19)
        );
    }

    #[test]
    fn test_bring_to_front_by_view_id() {
        let mut desktop = Desktop::new(Rect::new(0, 1, 80, 24));

        let id1 = desktop.add(Window::new(Rect::new(5, 5, 30, 15), "Win 1"));
        let id2 = desktop.add(Window::new(Rect::new(10, 6, 35, 16), "Win 2"));
        let _id3 = desktop.add(Window::new(Rect::new(15, 7, 40, 17), "Win 3"));

        // Win 3 is on top (last added). Bring Win 1 to front.
        assert!(desktop.bring_to_front(id1));

        // Win 1 should now be the top window (last child, excluding background).
        // Verify by bringing id2 to front and confirming it also works.
        assert!(desktop.bring_to_front(id2));

        // Non-existent ViewId returns false
        let fake_id = crate::views::view::ViewId::from_u16(9999);
        assert!(!desktop.bring_to_front(fake_id));
    }

    #[test]
    fn test_id_tracking_survives_sibling_removal() {
        // Regression: Application::exec_view used to track the modal view by
        // index, so removing a lower sibling made it remove the wrong window
        let mut desktop = Desktop::new(Rect::new(0, 1, 80, 24));

        let id1 = desktop.add(Window::new(Rect::new(5, 5, 30, 15), "Win 1"));
        let modal_id = desktop.add(Window::new(Rect::new(10, 6, 35, 16), "Modal"));
        assert_eq!(desktop.top_view_id(), Some(modal_id));

        // Removing a lower sibling must not disturb identity lookups
        assert!(desktop.remove_child_by_id(id1));
        assert!(desktop.contains_id(modal_id));
        assert!(desktop.child_by_id(modal_id).is_some());
        assert!(desktop.child_by_id_mut(modal_id).is_some());

        assert!(desktop.remove_child_by_id(modal_id));
        assert!(!desktop.contains_id(modal_id));
        assert_eq!(desktop.top_view_id(), None);
    }

    #[test]
    fn test_alt_digit_selects_numbered_window() {
        use crate::core::command::CM_SELECT_WINDOW_NUM;
        use crate::core::event::{Event, EventType};

        let mut desktop = Desktop::new(Rect::new(0, 1, 80, 24));
        let mut w1 = Window::new(Rect::new(5, 5, 30, 15), "One");
        w1.set_number(1);
        let mut w2 = Window::new(Rect::new(10, 6, 35, 16), "Two");
        w2.set_number(2);
        let id1 = desktop.add(w1);
        let _id2 = desktop.add(w2);

        // Window 2 is on top; broadcast selects window 1
        let mut event = Event::broadcast_with_info(CM_SELECT_WINDOW_NUM, 1);
        desktop.handle_event(&mut event);
        assert_eq!(event.what, EventType::Nothing);
        assert_eq!(desktop.top_view_id(), Some(id1));

        // Unknown number leaves the event unconsumed and z-order unchanged
        let mut event = Event::broadcast_with_info(CM_SELECT_WINDOW_NUM, 7);
        desktop.handle_event(&mut event);
        assert_eq!(event.what, EventType::Broadcast);
        assert_eq!(desktop.top_view_id(), Some(id1));
    }

    #[test]
    fn test_tile_fills_rect_exactly() {
        let mut desktop = Desktop::new(Rect::new(0, 0, 80, 24));
        for i in 0..3 {
            let mut w = Window::new(Rect::new(i, i, i + 20, i + 10), "w");
            let opts = w.options();
            w.set_options(opts | Options::TILEABLE);
            desktop.add(w);
        }
        desktop.tile();

        // Borland layout for 3 windows: 3 columns x 1 row, no holes, and
        // the union of cells covers the desktop exactly
        let b0 = desktop.child_at(0).bounds();
        let b1 = desktop.child_at(1).bounds();
        let b2 = desktop.child_at(2).bounds();
        assert_eq!(b0, Rect::new(0, 0, 26, 24));
        assert_eq!(b1, Rect::new(26, 0, 53, 24));
        assert_eq!(b2, Rect::new(53, 0, 80, 24));
    }

    #[test]
    fn test_cascade_extends_to_corner() {
        let mut desktop = Desktop::new(Rect::new(0, 0, 80, 24));
        for i in 0..3 {
            let mut w = Window::new(Rect::new(i, i, i + 20, i + 10), "w");
            let opts = w.options();
            w.set_options(opts | Options::TILEABLE);
            desktop.add(w);
        }
        desktop.cascade();

        // Each window's origin steps down the staircase; ALL extend to the
        // rect's bottom-right corner (Borland doCascade)
        for i in 0..3 {
            let b = desktop.child_at(i).bounds();
            assert_eq!((b.a.x, b.a.y), (i as i16, i as i16));
            assert_eq!((b.b.x, b.b.y), (80, 24));
        }
    }
}
