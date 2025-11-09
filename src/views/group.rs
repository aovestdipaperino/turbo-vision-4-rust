// (C) 2025 - Enzo Lombardi

//! Group view - container for managing multiple child views with focus handling.

use crate::core::geometry::Rect;
use crate::core::event::{Event, EventType, KB_TAB, KB_SHIFT_TAB};
use crate::core::draw::DrawBuffer;
use crate::core::palette::Attr;
use crate::terminal::Terminal;
use super::view::{View, write_line_to_terminal};

/// Group - a container for child views
/// Matches Borland: TGroup (tgroup.h/tgroup.cc)
pub struct Group {
    bounds: Rect,
    children: Vec<Box<dyn View>>,
    focused: usize,
    background: Option<Attr>,
    end_state: crate::core::command::CommandId,  // For execute() event loop (Borland: endState)
    owner: Option<*const dyn View>,  // Borland: TView::owner field
}

impl Group {
    pub fn new(bounds: Rect) -> Self {
        Self {
            bounds,
            children: Vec::new(),
            focused: 0,
            background: None,
            end_state: 0,
            owner: None,
        }
    }

    pub fn with_background(bounds: Rect, background: Attr) -> Self {
        Self {
            bounds,
            children: Vec::new(),
            focused: 0,
            background: Some(background),
            end_state: 0,
            owner: None,
        }
    }

    pub fn add(&mut self, mut view: Box<dyn View>) -> usize {
        // Set this group as the owner of the child view
        // Matches Borland: TGroup::insert() sets owner pointer
        view.set_owner(self as *const Self as *const dyn View);

        // Convert child's bounds from relative to absolute coordinates
        // Child bounds are specified relative to this Group's interior
        let child_bounds = view.bounds();
        let absolute_bounds = Rect::new(
            self.bounds.a.x + child_bounds.a.x,
            self.bounds.a.y + child_bounds.a.y,
            self.bounds.a.x + child_bounds.b.x,
            self.bounds.a.y + child_bounds.b.y,
        );
        view.set_bounds(absolute_bounds);
        self.children.push(view);
        self.children.len() - 1  // Return index of newly added child
    }

    pub fn set_initial_focus(&mut self) {
        if self.children.is_empty() {
            return;
        }

        // Find first focusable child and set focus
        for i in 0..self.children.len() {
            if self.children[i].can_focus() {
                self.focused = i;
                self.children[i].set_focus(true);
                break;
            }
        }
    }

    pub fn clear_all_focus(&mut self) {
        for child in &mut self.children {
            child.set_focus(false);
        }
    }

    pub fn len(&self) -> usize {
        self.children.len()
    }

    pub fn is_empty(&self) -> bool {
        self.children.is_empty()
    }

    pub fn child_at(&self, index: usize) -> &dyn View {
        &*self.children[index]
    }

    pub fn child_at_mut(&mut self, index: usize) -> &mut dyn View {
        &mut *self.children[index]
    }

    pub fn set_focus_to(&mut self, index: usize) {
        if index < self.children.len() {
            self.focused = index;
            self.children[index].set_focus(true);
        }
    }

    /// Bring a child view to the front (top of z-order)
    /// Matches Borland: TGroup::selectView() which reorders views
    /// Returns the new index of the moved child
    pub fn bring_to_front(&mut self, index: usize) -> usize {
        if index >= self.children.len() || index == self.children.len() - 1 {
            // Already at front or invalid index
            return index;
        }

        // Remove the view from its current position
        let view = self.children.remove(index);

        // Add it to the end (front of z-order)
        self.children.push(view);

        // Update focused index if necessary
        let new_index = self.children.len() - 1;
        if self.focused == index {
            self.focused = new_index;
        } else if self.focused > index {
            // Focused view shifted down by one
            self.focused -= 1;
        }

        new_index
    }

    /// Send a child view to the back (bottom of z-order, but after index 0)
    /// Matches Borland: current->putInFrontOf(background) for window cycling
    /// Returns the new index of the moved child (always 1 for desktop windows)
    pub fn send_to_back(&mut self, index: usize) -> usize {
        if index >= self.children.len() || index == 1 {
            // Already at back (position 1) or invalid index
            return index;
        }

        // Remove the view from its current position
        let view = self.children.remove(index);

        // Insert it at position 1 (right after element 0, which is typically background)
        self.children.insert(1, view);

        // Update focused index if necessary
        if self.focused == index {
            self.focused = 1;
        } else if self.focused >= 1 && self.focused < index {
            // Views between positions 1 and index shifted up by one
            self.focused += 1;
        }

        1 // Always returns 1 (the back position after index 0)
    }

    /// Remove a child at the specified index
    /// Matches Borland: TGroup::remove(TView *p) or TGroup::shutDown()
    pub fn remove(&mut self, index: usize) {
        if index < self.children.len() {
            self.children.remove(index);

            // Update focused index if needed
            if self.focused >= index && self.focused > 0 {
                self.focused -= 1;
            }

            // If we removed the last child, clear focus
            if self.children.is_empty() {
                self.focused = 0;
            }
        }
    }

    /// Execute a modal event loop
    /// Matches Borland: TGroup::execute() (tgroup.cc:182-195)
    ///
    /// This is the KEY method that makes modal views work.
    /// In Borland, TGroup has an execute() method with an event loop that calls
    /// getEvent() and handleEvent() until endState is set by endModal().
    ///
    /// The event loop:
    /// 1. Calls app.get_event() which handles drawing and returns events
    /// 2. Calls self.handle_event() to process the event
    /// 3. Continues until end_state is set (by endModal)
    ///
    /// This is used by Dialog, Window, and any other container that needs
    /// modal execution.
    pub fn execute(&mut self, app: &mut crate::app::Application) -> crate::core::command::CommandId {
        self.end_state = 0;

        loop {
            // Get event from Application (which handles drawing)
            // Matches Borland: TGroup::execute() calls getEvent(e)
            if let Some(mut event) = app.get_event() {
                // Handle the event
                // Matches Borland: TGroup::execute() calls handleEvent(e)
                self.handle_event(&mut event);
            }

            // Check if we should end the modal loop
            // Matches Borland: while( endState == 0 )
            // IMPORTANT: This must be OUTSIDE the event check, so we check
            // end_state even when there are no events (timeout)
            if self.end_state != 0 {
                break;
            }
        }

        // TODO: Borland also calls valid(endState) here
        // For now, just return endState
        self.end_state
    }

    /// End the modal event loop with a result code
    /// Matches Borland: TView::endModal(ushort command) (tview.cc:391-395)
    ///
    /// In Borland, views call endModal() to set endState and break out of
    /// the execute() event loop. This is typically called in response to
    /// button clicks (CM_OK, CM_CANCEL, etc.)
    pub fn end_modal(&mut self, command: crate::core::command::CommandId) {
        self.end_state = command;
    }

    /// Get the current end_state
    /// Used by containers that implement their own execute() loop
    /// to check if they should end the modal loop
    pub fn get_end_state(&self) -> crate::core::command::CommandId {
        self.end_state
    }

    /// Set the current end_state
    /// Used by modal views to signal they want to close
    pub fn set_end_state(&mut self, command: crate::core::command::CommandId) {
        self.end_state = command;
    }

    /// Broadcast an event to all children except the owner
    /// Matches Borland: TGroup::forEach with message() that takes receiver parameter
    ///
    /// The owner parameter prevents the broadcast from echoing back to the originator.
    /// This is essential for focus-list navigation commands and other broadcast patterns
    /// where the sender shouldn't receive its own message.
    ///
    /// # Arguments
    /// * `event` - The event to broadcast (typically EventType::Broadcast)
    /// * `owner_index` - Optional index of the child that originated the broadcast (will be skipped)
    ///
    /// # Reference
    /// Borland's message() function: `local-only/borland-tvision/include/tv/tvutil.h`
    /// TGroup::forEach pattern: `local-only/borland-tvision/classes/tgroup.cc:675-689`
    pub fn broadcast(&mut self, event: &mut Event, owner_index: Option<usize>) {
        for (i, child) in self.children.iter_mut().enumerate() {
            // Skip the owner if specified
            if let Some(owner) = owner_index {
                if i == owner {
                    continue;
                }
            }

            // Send event to this child
            // Note: Child handle_event may clear or transform the event
            // So we need to check if it's still active before continuing
            child.handle_event(event);

            // If event was cleared, stop broadcasting
            if event.what == EventType::Nothing {
                break;
            }
        }
    }

    /// Draw views starting from a specific index
    /// Used for Borland's drawUnderRect pattern where we only redraw views
    /// that come after (on top of) a moved view
    /// Matches Borland: TGroup::drawSubViews(TView *p, TView *bottom)
    pub fn draw_sub_views(&mut self, terminal: &mut Terminal, start_index: usize, clip: Rect) {
        // Set clip region to the affected area
        terminal.push_clip(clip);

        // Draw all children from start_index onwards that intersect the clip region
        for i in start_index..self.children.len() {
            let child_bounds = self.children[i].bounds();
            if clip.intersects(&child_bounds) {
                self.children[i].draw(terminal);
            }
        }

        terminal.pop_clip();
    }

    /// Get a reference to the currently focused child view, if any
    pub fn focused_child(&self) -> Option<&dyn View> {
        if self.focused < self.children.len() {
            Some(&*self.children[self.focused])
        } else {
            None
        }
    }

    pub fn select_next(&mut self) {
        if self.children.is_empty() {
            return;
        }

        // Clear focus from current child
        if self.focused < self.children.len() {
            self.children[self.focused].set_focus(false);
        }

        let start_index = self.focused;
        loop {
            self.focused = (self.focused + 1) % self.children.len();
            if self.children[self.focused].can_focus() {
                self.children[self.focused].set_focus(true);
                break;
            }
            // Prevent infinite loop if no focusable children
            if self.focused == start_index {
                break;
            }
        }
    }

    pub fn select_previous(&mut self) {
        if self.children.is_empty() {
            return;
        }

        // Clear focus from current child
        if self.focused < self.children.len() {
            self.children[self.focused].set_focus(false);
        }

        let start_index = self.focused;
        loop {
            // Move to previous, wrapping around
            if self.focused == 0 {
                self.focused = self.children.len() - 1;
            } else {
                self.focused -= 1;
            }

            if self.children[self.focused].can_focus() {
                self.children[self.focused].set_focus(true);
                break;
            }
            // Prevent infinite loop if no focusable children
            if self.focused == start_index {
                break;
            }
        }
    }
}

impl View for Group {
    fn bounds(&self) -> Rect {
        self.bounds
    }

    fn set_bounds(&mut self, bounds: Rect) {
        // Calculate the offset (how much the group moved)
        let dx = bounds.a.x - self.bounds.a.x;
        let dy = bounds.a.y - self.bounds.a.y;

        // Update our bounds
        self.bounds = bounds;

        // Update all children's bounds by the same offset
        for child in &mut self.children {
            let child_bounds = child.bounds();
            let new_bounds = Rect::new(
                child_bounds.a.x + dx,
                child_bounds.a.y + dy,
                child_bounds.b.x + dx,
                child_bounds.b.y + dy,
            );
            child.set_bounds(new_bounds);
        }
    }

    fn draw(&mut self, terminal: &mut Terminal) {
        // Draw background if specified
        if let Some(bg_attr) = self.background {
            let width = self.bounds.width() as usize;
            let height = self.bounds.height() as usize;

            for y in 0..height {
                let mut buf = DrawBuffer::new(width);
                buf.move_char(0, ' ', bg_attr, width);
                write_line_to_terminal(
                    terminal,
                    self.bounds.a.x,
                    self.bounds.a.y + y as i16,
                    &buf,
                );
            }
        }

        // Push clipping region for this group's bounds
        terminal.push_clip(self.bounds);

        // Only draw children that intersect with this group's bounds
        // The clipping region ensures children can't render outside parent boundaries
        for child in &mut self.children {
            let child_bounds = child.bounds();
            if self.bounds.intersects(&child_bounds) {
                child.draw(terminal);
            }
        }

        // Pop clipping region
        terminal.pop_clip();
    }

    fn handle_event(&mut self, event: &mut Event) {
        use crate::core::state::{OF_PRE_PROCESS, OF_POST_PROCESS};

        // Handle Tab key for focus navigation (before three-phase)
        if event.what == EventType::Keyboard {
            if event.key_code == KB_TAB {
                self.select_next();
                event.clear();
                return;
            } else if event.key_code == KB_SHIFT_TAB {
                self.select_previous();
                event.clear();
                return;
            }
        }

        // Mouse events: positional events (no three-phase processing)
        // Search in REVERSE order (top-most child first) - matches Borland's z-order
        // Matches Borland: TGroup::handleEvent() processes mouse events from front to back
        if event.what == EventType::MouseDown || event.what == EventType::MouseMove || event.what == EventType::MouseUp {
            let mouse_pos = event.mouse.pos;

            // For MouseMove and MouseUp, check if the focused child is dragging
            // If so, send the event to it even if mouse is outside its bounds
            // This allows dragging beyond window boundaries (matches Borland behavior)
            if (event.what == EventType::MouseMove || event.what == EventType::MouseUp) && self.focused < self.children.len() {
                // Check if focused child is in dragging state (has SF_DRAGGING flag)
                if (self.children[self.focused].state() & crate::core::state::SF_DRAGGING) != 0 {
                    self.children[self.focused].handle_event(event);
                    return;
                }
            }

            // First pass: find which child contains the mouse (search in reverse z-order)
            let mut clicked_child_index: Option<usize> = None;
            for i in (0..self.children.len()).rev() {
                let child_bounds = self.children[i].bounds();
                if child_bounds.contains(mouse_pos) {
                    clicked_child_index = Some(i);
                    break;
                }
            }

            // If a child was clicked, handle focus and events
            if let Some(i) = clicked_child_index {
                if event.what == EventType::MouseDown {
                    // Check if this is a label with a link (Borland: TLabel::focusLink)
                    // If so, focus the linked control instead of the label
                    if let Some(link_index) = self.children[i].label_link() {
                        if link_index < self.children.len() && self.children[link_index].can_focus() {
                            self.clear_all_focus();
                            self.focused = link_index;
                            self.children[link_index].set_focus(true);
                            event.clear();  // Event consumed by focus transfer
                            return;
                        }
                    } else if self.children[i].can_focus() {
                        // Regular focusable view - give it focus
                        self.clear_all_focus();
                        self.focused = i;
                        self.children[i].set_focus(true);
                    }
                }

                // Second pass: handle the event
                self.children[i].handle_event(event);

                // IMPORTANT: If the child converted the event to Broadcast (e.g., calculator buttons),
                // we need to handle that broadcast now (matches Borland's putEvent behavior)
                if event.what == EventType::Broadcast {
                    // Recursively call handle_event to process the broadcast
                    self.handle_event(event);
                }
                return;
            }
        }

        // Keyboard and Command events: use three-phase processing (matches Borland)
        // Phase 1: PreProcess - views with OF_PRE_PROCESS flag (e.g., buttons for Space/Enter)
        // Phase 2: Focused - currently focused view gets first chance
        // Phase 3: PostProcess - views with OF_POST_PROCESS flag (e.g., status line for help keys)

        if event.what == EventType::Keyboard || event.what == EventType::Command {
            // Phase 1: PreProcess
            // Views with OF_PRE_PROCESS get first chance at the event
            for child in &mut self.children {
                if event.what == EventType::Nothing {
                    break; // Event was handled
                }
                if (child.options() & OF_PRE_PROCESS) != 0 {
                    child.handle_event(event);
                }
            }

            // Phase 2: Focused
            // Give focused view a chance if event wasn't handled
            if event.what != EventType::Nothing && self.focused < self.children.len() {
                self.children[self.focused].handle_event(event);
            }

            // Phase 3: PostProcess
            // Views with OF_POST_PROCESS get last chance (e.g., status line, buttons)
            if event.what != EventType::Nothing {
                for child in &mut self.children {
                    if event.what == EventType::Nothing {
                        break; // Event was handled
                    }
                    if (child.options() & OF_POST_PROCESS) != 0 {
                        child.handle_event(event);
                    }
                }

                // IMPORTANT: If a PostProcess view converted the event to Broadcast,
                // we need to handle that broadcast now (matches Borland's putEvent behavior)
                // For example, calculator buttons convert MouseDown to Broadcast
                if event.what == EventType::Broadcast {
                    // Recursively call handle_event to process the broadcast
                    self.handle_event(event);
                }
            }
        } else {
            // Broadcast events: send to ALL children
            // Other event types: send to focused child only
            if event.what == EventType::Broadcast {
                // Matches Borland: TGroup::handleEvent() broadcasts to all children via forEach
                for child in &mut self.children {
                    if event.what == EventType::Nothing {
                        break; // Event was handled
                    }
                    child.handle_event(event);
                }
            } else {
                // Other event types: send to focused child only
                if self.focused < self.children.len() {
                    self.children[self.focused].handle_event(event);
                }
            }
        }
    }

    fn update_cursor(&self, terminal: &mut Terminal) {
        // Hide cursor by default
        let _ = terminal.hide_cursor();

        // Update cursor for the focused child (it can show it if needed)
        if self.focused < self.children.len() {
            self.children[self.focused].update_cursor(terminal);
        }
    }

    fn get_end_state(&self) -> crate::core::command::CommandId {
        self.end_state
    }

    fn set_end_state(&mut self, command: crate::core::command::CommandId) {
        self.end_state = command;
    }

    /// Validate group before performing command
    /// Matches Borland: TGroup::valid(ushort command)
    /// - If command is CM_RELEASED_FOCUS, validate current focused child if it has OF_VALIDATE
    /// - Otherwise, validate all children (return false if any child is invalid)
    fn valid(&mut self, command: crate::core::command::CommandId) -> bool {
        use crate::core::command::CM_RELEASED_FOCUS;
        use crate::core::state::OF_VALIDATE;

        if command == CM_RELEASED_FOCUS {
            // Validate only the currently focused child if it has OF_VALIDATE flag
            if self.focused < self.children.len() {
                let child = &mut self.children[self.focused];
                if (child.options() & OF_VALIDATE) != 0 {
                    return child.valid(command);
                }
            }
            true
        } else {
            // Validate all children - return false if any child is invalid
            // Matches Borland: firstThat(isInvalid, &command) == nullptr
            for child in &mut self.children {
                if !child.valid(command) {
                    return false;
                }
            }
            true
        }
    }

    fn set_owner(&mut self, owner: *const dyn View) {
        use std::io::Write;
        let mut log = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open("calc.log")
            .ok();

        if let Some(ref mut log) = log {
            writeln!(log, "Group::set_owner called on {:p}, owner={:?}", self, owner).ok();
        }

        self.owner = Some(owner);

        if let Some(ref mut log) = log {
            writeln!(log, "Group::set_owner done, self.owner={:?}", self.owner).ok();
        }
    }

    fn get_owner(&self) -> Option<*const dyn View> {
        self.owner
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Helper to count how many times draw is called on views
    struct DrawCountView {
        bounds: Rect,
        draw_count: std::cell::RefCell<usize>,
    }

    impl DrawCountView {
        fn new(bounds: Rect) -> Self {
            Self {
                bounds,
                draw_count: std::cell::RefCell::new(0),
            }
        }
    }

    impl View for DrawCountView {
        fn bounds(&self) -> Rect {
            self.bounds
        }

        fn set_bounds(&mut self, bounds: Rect) {
            self.bounds = bounds;
        }

        fn draw(&mut self, _terminal: &mut Terminal) {
            *self.draw_count.borrow_mut() += 1;
        }

        fn handle_event(&mut self, _event: &mut Event) {}
    }

    #[test]
    fn test_child_completely_outside_parent_not_drawn() {
        // Create a group at (10, 10) with size 20x20
        let group = Group::new(Rect::new(10, 10, 30, 30));

        // Add a child completely outside the parent bounds (to the right)
        let child_bounds = Rect::new(100, 15, 110, 20);

        // Verify the child is outside parent bounds
        assert!(!group.bounds.intersects(&child_bounds));
    }

    #[test]
    fn test_child_inside_parent_is_drawn() {
        // Create a group at (10, 10) with size 20x20
        let mut group = Group::new(Rect::new(10, 10, 30, 30));

        // Add a child at relative position (5, 5) which becomes absolute (15, 15)
        // This is inside the parent bounds (10, 10, 30, 30)
        let child = Box::new(DrawCountView::new(Rect::new(5, 5, 15, 15)));
        group.add(child);

        // Verify the child was converted to absolute coordinates
        assert_eq!(group.children.len(), 1);
        assert_eq!(group.children[0].bounds(), Rect::new(15, 15, 25, 25));

        // Verify child intersects with parent (so it would be drawn)
        assert!(group.bounds.intersects(&group.children[0].bounds()));
    }

    #[test]
    fn test_child_partially_outside_parent() {
        // Create a group at (10, 10) with size 20x20 (bounds: 10-30, 10-30)
        let mut group = Group::new(Rect::new(10, 10, 30, 30));

        // Add a child at relative position (15, 15) with size 10x10
        // Absolute bounds: (25, 25, 35, 35)
        // This extends beyond parent (30, 30), so partially outside
        let child = Box::new(DrawCountView::new(Rect::new(15, 15, 25, 25)));
        group.add(child);

        // Verify conversion to absolute
        assert_eq!(group.children[0].bounds(), Rect::new(25, 25, 35, 35));

        // Verify child still intersects with parent (partially visible)
        assert!(group.bounds.intersects(&group.children[0].bounds()));

        // Note: The child will be drawn, but the Terminal's write methods
        // will clip at the terminal boundaries. For proper parent clipping,
        // we would need to implement a clipping region in Terminal.
        // For now, we just verify that intersecting children would be drawn.
    }

    #[test]
    fn test_coordinate_conversion_on_add() {
        // Create a group at (20, 30) with size 40x50
        let mut group = Group::new(Rect::new(20, 30, 60, 80));

        // Add a child with relative coordinates (5, 10)
        let child = Box::new(DrawCountView::new(Rect::new(5, 10, 15, 20)));
        group.add(child);

        // Verify the child's bounds were converted to absolute
        // Relative (5, 10, 15, 20) + Group origin (20, 30) = Absolute (25, 40, 35, 50)
        assert_eq!(group.children[0].bounds(), Rect::new(25, 40, 35, 50));
    }

    #[test]
    fn test_multiple_children_clipping() {
        // Create a group at (0, 0) with size 50x50
        let mut group = Group::new(Rect::new(0, 0, 50, 50));

        // Child 1: Inside (10, 10, 20, 20) -> absolute (10, 10, 20, 20)
        group.add(Box::new(DrawCountView::new(Rect::new(10, 10, 20, 20))));

        // Child 2: Completely outside (100, 100, 110, 110) -> absolute (100, 100, 110, 110)
        group.add(Box::new(DrawCountView::new(Rect::new(100, 100, 110, 110))));

        // Child 3: Partially outside (40, 40, 60, 60) -> absolute (40, 40, 60, 60)
        group.add(Box::new(DrawCountView::new(Rect::new(40, 40, 60, 60))));

        assert_eq!(group.children.len(), 3);

        // Verify intersections
        // Child 1: completely inside, should intersect
        assert!(group.bounds.intersects(&group.children[0].bounds()));

        // Child 2: completely outside, should NOT intersect
        assert!(!group.bounds.intersects(&group.children[1].bounds()));

        // Child 3: partially outside, should intersect
        assert!(group.bounds.intersects(&group.children[2].bounds()));
    }
}
