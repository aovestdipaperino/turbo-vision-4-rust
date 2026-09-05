// (C) 2025 - Enzo Lombardi

//! Group view - container for managing multiple child views with focus handling.

use super::view::{View, ViewCore, ViewId, write_line_to_terminal};
use crate::core::draw::DrawBuffer;
use crate::core::event::{Event, EventType, KB_SHIFT_TAB, KB_TAB};
use crate::core::geometry::Rect;
use crate::core::palette::Attr;
use crate::terminal::Terminal;

/// Group - a container for child views
/// Matches Borland: TGroup (tgroup.h/tgroup.cc)
pub struct Group {
    core: ViewCore,
    children: Vec<Box<dyn View>>,
    view_ids: Vec<ViewId>, // Parallel vec storing ID for each child
    focused: usize,
    background: Option<Attr>,
    end_state: crate::core::command::CommandId, // For execute() event loop (Borland: endState)
}

impl Group {
    pub fn new(bounds: Rect) -> Self {
        Self {
            core: ViewCore {
                bounds,
                palette_chain: None,
                grow_mode: 0,
                ..ViewCore::default()
            },
            children: Vec::new(),
            view_ids: Vec::new(),
            focused: 0,
            background: None,
            end_state: 0,
        }
    }

    pub fn with_background(bounds: Rect, background: Attr) -> Self {
        Self {
            core: ViewCore {
                bounds,
                palette_chain: None,
                grow_mode: 0,
                ..ViewCore::default()
            },
            children: Vec::new(),
            view_ids: Vec::new(),
            focused: 0,
            background: Some(background),
            end_state: 0,
        }
    }

    /// Set the grow mode of a child by index (Borland: child->growMode = ...).
    ///
    /// Convenience for callers that add children whose concrete type does not
    /// override `set_grow_mode()` — the call is then a no-op, matching the
    /// trait default.
    pub fn set_child_grow_mode(&mut self, index: usize, grow_mode: crate::core::state::GrowFlags) {
        if index < self.children.len() {
            self.children[index].set_grow_mode(grow_mode);
        }
    }

    pub fn add(&mut self, mut view: Box<dyn View>) -> ViewId {
        // Convert child's bounds from relative to absolute coordinates
        // Child bounds are specified relative to this Group's interior
        let child_bounds = view.bounds();
        let absolute_bounds = Rect::new(
            self.core.bounds.a.x + child_bounds.a.x,
            self.core.bounds.a.y + child_bounds.a.y,
            self.core.bounds.a.x + child_bounds.b.x,
            self.core.bounds.a.y + child_bounds.b.y,
        );
        view.set_bounds(absolute_bounds);

        let view_id = ViewId::new();
        self.children.push(view);
        self.view_ids.push(view_id);
        view_id
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
        if index < self.children.len() && self.children[index].can_focus() {
            self.clear_all_focus();
            self.focused = index;
            self.children[index].set_focus(true);
        }
    }

    /// Focus a child view by its ViewId
    /// Returns true if the view was found and focused, false otherwise
    pub fn focus_by_view_id(&mut self, view_id: ViewId) -> bool {
        if let Some(index) = self.view_ids.iter().position(|&id| id == view_id) {
            if self.children[index].can_focus() {
                self.clear_all_focus();
                self.focused = index;
                self.children[index].set_focus(true);
                return true;
            }
        }
        false
    }

    /// Bring a child view to the front (top of z-order)
    /// Matches Borland: TGroup::selectView() which reorders views
    /// Returns the new index of the moved child
    pub fn bring_to_front(&mut self, index: usize) -> usize {
        if index >= self.children.len() || index == self.children.len() - 1 {
            // Already at front or invalid index
            return index;
        }

        // Remove the view and its corresponding ID from their current position
        let view = self.children.remove(index);
        let view_id = self.view_ids.remove(index);

        // Add them to the end (front of z-order)
        self.children.push(view);
        self.view_ids.push(view_id);

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

        // Remove the view and its corresponding ID from their current position
        let view = self.children.remove(index);
        let view_id = self.view_ids.remove(index);

        // Insert them at position 1 (right after element 0, which is typically background)
        self.children.insert(1, view);
        self.view_ids.insert(1, view_id);

        // Update focused index if necessary
        if self.focused == index {
            self.focused = 1;
        } else if self.focused >= 1 && self.focused < index {
            // Views between positions 1 and index shifted up by one
            self.focused += 1;
        }

        1 // Always returns 1 (the back position after index 0)
    }

    /// Get the ViewId of a child at the given index.
    /// Returns None if the index is out of bounds.
    pub fn view_id_at(&self, index: usize) -> Option<ViewId> {
        self.view_ids.get(index).copied()
    }

    /// Remove a child at the specified index
    /// Matches Borland: TGroup::remove(TView *p) or TGroup::shutDown()
    pub fn remove(&mut self, index: usize) {
        if index < self.children.len() {
            let removed_focused = self.focused == index;
            self.children.remove(index);
            // `view_ids` is a parallel vec — must stay in lock-step with
            // `children`. Forgetting it leaves stale ids that point past the
            // end of `children`, so `child_by_id` indexes out of bounds.
            if index < self.view_ids.len() {
                self.view_ids.remove(index);
            }

            // Update focused index if needed
            if self.focused >= index && self.focused > 0 {
                self.focused -= 1;
            }

            // If we removed the last child, clear focus
            if self.children.is_empty() {
                self.focused = 0;
            } else if removed_focused {
                // The focused child was removed: re-establish focus on the
                // nearest focusable child so focus isn't silently lost.
                // Matches Borland: TGroup::remove() → resetCurrent()/focusNext.
                let len = self.children.len();
                let start = self.focused.min(len - 1);
                for k in 0..len {
                    let idx = (start + k) % len;
                    if self.children[idx].can_focus() {
                        self.focused = idx;
                        self.children[idx].set_focus(true);
                        break;
                    }
                }
            }
        }
    }

    /// Get an immutable reference to a child by its ViewId
    /// Returns None if the ViewId is not found
    pub fn child_by_id(&self, view_id: ViewId) -> Option<&dyn View> {
        self.view_ids
            .iter()
            .position(|&id| id == view_id)
            .map(|index| &*self.children[index])
    }

    /// Get a mutable reference to a child by its ViewId
    /// Returns None if the ViewId is not found
    pub fn child_by_id_mut(&mut self, view_id: ViewId) -> Option<&mut (dyn View + '_)> {
        if let Some(index) = self.view_ids.iter().position(|&id| id == view_id) {
            Some(&mut *self.children[index])
        } else {
            None
        }
    }

    /// Remove a child by its ViewId
    /// Returns true if a child was found and removed, false otherwise
    pub fn remove_by_id(&mut self, view_id: ViewId) -> bool {
        if let Some(index) = self.view_ids.iter().position(|&id| id == view_id) {
            // `remove()` already keeps `children` and `view_ids` in lock-step,
            // so we must NOT remove from `view_ids` again here — doing so drops
            // the wrong (now-shifted) id and can index out of bounds.
            self.remove(index);
            true
        } else {
            false
        }
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

    /// True if the child at `index` can take focus.
    ///
    /// Matches Borland TGroup::findNext: the view must be selectable and not
    /// disabled. (SF_VISIBLE is not checked because this port never sets it;
    /// hidden views are simply not added to the group.)
    fn child_focusable(&self, index: usize) -> bool {
        use crate::core::state::SF_DISABLED;
        let child = &self.children[index];
        child.can_focus() && (child.state() & SF_DISABLED) == 0
    }

    pub fn select_next(&mut self) {
        if self.children.is_empty() {
            return;
        }

        // Find the next focusable child WITHOUT dropping current focus:
        // if no other child qualifies, focus stays where it is (Borland's
        // focusNext is a no-op in that case)
        let start_index = self.focused;
        let mut candidate = self.focused;
        loop {
            candidate = (candidate + 1) % self.children.len();
            if candidate == start_index {
                return; // wrapped without finding another focusable child
            }
            if self.child_focusable(candidate) {
                break;
            }
        }

        if self.focused < self.children.len() {
            self.children[self.focused].set_focus(false);
        }
        self.focused = candidate;
        self.children[self.focused].set_focus(true);
    }

    pub fn select_previous(&mut self) {
        if self.children.is_empty() {
            return;
        }

        // Mirror image of select_next: scan backwards, keep focus if no
        // other focusable child exists
        let start_index = self.focused;
        let mut candidate = self.focused;
        loop {
            candidate = if candidate == 0 {
                self.children.len() - 1
            } else {
                candidate - 1
            };
            if candidate == start_index {
                return;
            }
            if self.child_focusable(candidate) {
                break;
            }
        }

        if self.focused < self.children.len() {
            self.children[self.focused].set_focus(false);
        }
        self.focused = candidate;
        self.children[self.focused].set_focus(true);
    }
}

/// The behaviour of Borland's `TGroup`, expressed as default methods over a
/// `Group` core so that a container type (`Window`, `Dialog`, ...) inherits
/// it and can still override any `View` hook.
///
/// Each inherited implementation has a `group_` name so an override can make
/// the base call, the way `TDialog::handleEvent` starts with
/// `TWindow::handleEvent(event)`. `execute`, `handle_event`, `valid` and
/// `get_palette` are called through `self`, so a type that overrides them in
/// its `View` impl is seen by this base code.
pub trait GroupLike: View {
    fn group(&self) -> &Group;
    fn group_mut(&mut self) -> &mut Group;

    // ---- inherited implementations, callable as base calls ----

    fn group_set_bounds(&mut self, bounds: Rect) {
        // Calculate the offset (how much the group moved)
        let dx = bounds.a.x - self.bounds().a.x;
        let dy = bounds.a.y - self.bounds().a.y;

        // Calculate the size change (how much the group was resized)
        let dw = bounds.width() - self.bounds().width();
        let dh = bounds.height() - self.bounds().height();

        // Update our bounds
        self.core_mut().bounds = bounds;

        // Update all children's bounds. Every child shifts by the group's
        // offset (children store absolute coordinates); each edge additionally
        // follows the size delta only if the matching grow bit is set.
        // Matches Borland: TView::calcBounds() driven by growMode.
        use crate::core::state::{GF_GROW_HI_X, GF_GROW_HI_Y, GF_GROW_LO_X, GF_GROW_LO_Y};
        for child in &mut self.group_mut().children {
            let grow = child.grow_mode();
            let child_bounds = child.bounds();
            let new_bounds = Rect::new(
                child_bounds.a.x + dx + if grow & GF_GROW_LO_X != 0 { dw } else { 0 },
                child_bounds.a.y + dy + if grow & GF_GROW_LO_Y != 0 { dh } else { 0 },
                child_bounds.b.x + dx + if grow & GF_GROW_HI_X != 0 { dw } else { 0 },
                child_bounds.b.y + dy + if grow & GF_GROW_HI_Y != 0 { dh } else { 0 },
            );
            child.set_bounds(new_bounds);
        }
    }

    fn group_draw(&mut self, terminal: &mut Terminal) {
        // Draw background if specified
        if let Some(bg_attr) = self.group().background {
            let width = self.bounds().width_clamped() as usize;
            let height = self.bounds().height_clamped() as usize;

            for y in 0..height {
                let mut buf = DrawBuffer::new(width);
                buf.move_char(0, ' ', bg_attr, width);
                write_line_to_terminal(
                    terminal,
                    self.bounds().a.x,
                    self.bounds().a.y + y as i16,
                    &buf,
                );
            }
        }

        // Push clipping region for this group's bounds
        // Expand by 1 on all sides to allow children (like scrollbars) to overlap with parent's frame
        let mut clip_bounds = self.bounds();
        clip_bounds.grow(1, 1);
        terminal.push_clip(clip_bounds);

        // Build this Group's palette chain node for safe palette traversal.
        // Group is typically transparent (no palette), but carries the parent link.
        let my_chain_node = crate::core::palette_chain::PaletteChainNode::new(
            self.get_palette(),
            self.get_palette_chain().cloned(),
        );

        // Only draw children that intersect with this group's bounds
        // The clipping region ensures children can't render outside parent boundaries
        let my_bounds = self.bounds();
        for child in &mut self.group_mut().children {
            child.set_palette_chain(Some(my_chain_node.clone()));
            let child_bounds = child.bounds();
            if my_bounds.intersects(&child_bounds) {
                child.draw(terminal);
            }
        }

        // Pop clipping region
        terminal.pop_clip();
    }

    fn group_handle_event(&mut self, event: &mut Event) {
        use crate::core::state::{OF_POST_PROCESS, OF_PRE_PROCESS};

        // Mouse events: positional events (no three-phase processing)
        // Search in REVERSE order (top-most child first) - matches Borland's z-order
        // Matches Borland: TGroup::handleEvent() processes mouse events from front to back
        if event.what == EventType::MouseDown
            || event.what == EventType::MouseMove
            || event.what == EventType::MouseUp
        {
            let mouse_pos = event.mouse.pos;

            // For MouseMove and MouseUp, check if the focused child is dragging or resizing
            // If so, send the event to it even if mouse is outside its bounds
            // This allows dragging and resizing beyond window boundaries (matches Borland behavior)
            if (event.what == EventType::MouseMove || event.what == EventType::MouseUp)
                && self.group().focused < self.group_mut().children.len()
            {
                // Check if focused child is in dragging or resizing state
                let g = self.group_mut();
                let child_state = g.children[g.focused].state();
                if (child_state
                    & (crate::core::state::SF_DRAGGING | crate::core::state::SF_RESIZING))
                    != 0
                {
                    g.children[g.focused].handle_event(event);
                    return;
                }
            }

            // First pass: find which child contains the mouse (search in reverse z-order)
            let mut clicked_child_index: Option<usize> = None;
            for i in (0..self.group_mut().children.len()).rev() {
                let child_bounds = self.group_mut().children[i].bounds();
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
                    if let Some(link_id) = self.group_mut().children[i].label_link() {
                        // Find the child with the matching ViewId
                        if let Some(link_index) =
                            self.group().view_ids.iter().position(|&id| id == link_id)
                        {
                            if self.group_mut().children[link_index].can_focus() {
                                self.group_mut().clear_all_focus();
                                self.group_mut().focused = link_index;
                                self.group_mut().children[link_index].set_focus(true);
                                event.clear(); // Event consumed by focus transfer
                                return;
                            }
                        }
                    } else if self.group_mut().children[i].can_focus() {
                        // Regular focusable view - give it focus
                        self.group_mut().clear_all_focus();
                        self.group_mut().focused = i;
                        self.group_mut().children[i].set_focus(true);
                    }
                }

                // Second pass: handle the event
                self.group_mut().children[i].handle_event(event);

                // IMPORTANT: If the child converted the event to Broadcast (e.g., calculator buttons),
                // we need to handle that broadcast now (matches Borland's putEvent behavior)
                if event.what == EventType::Broadcast {
                    // Recursively call handle_event to process the broadcast
                    self.handle_event(event);
                    return;
                }

                // CRITICAL FIX: If the child converted MouseDown to Command (e.g., ListBox double-click),
                // DON'T return immediately. Instead, fall through to the command processing phase below
                // so the Command can be handled by the three-phase processing.
                // Matches Borland: Commands generated by mouse events flow through the event loop
                if event.what == EventType::Command {
                    // Fall through to command processing (don't return here)
                } else {
                    // For other event types, return after handling
                    return;
                }
            } else {
                // No child under the mouse: positional events must NOT be
                // forwarded to the focused child. Matches Borland:
                // TGroup::handleEvent() routes positional events only to
                // firstThat(hasMouse); if no subview contains the mouse the
                // event goes nowhere.
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
            for child in &mut self.group_mut().children {
                if event.what == EventType::Nothing {
                    break; // Event was handled
                }
                if (child.options() & OF_PRE_PROCESS) != 0 {
                    child.handle_event(event);
                }
            }

            // Phase 2: Focused
            // Give focused view a chance if event wasn't handled
            if event.what != EventType::Nothing
                && self.group().focused < self.group_mut().children.len()
            {
                {
                    let g = self.group_mut();
                    g.children[g.focused].handle_event(event);
                }
            }

            // Phase 3: PostProcess
            // Views with OF_POST_PROCESS get last chance (e.g., status line, buttons)
            if event.what != EventType::Nothing {
                for child in &mut self.group_mut().children {
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

            // Handle Tab key for focus navigation (after three-phase processing)
            // Only handle if event wasn't consumed by any child
            if event.what == EventType::Keyboard {
                if event.key_code == KB_TAB {
                    self.group_mut().select_next();
                    event.clear();
                    return;
                } else if event.key_code == KB_SHIFT_TAB {
                    self.group_mut().select_previous();
                    event.clear();
                    return;
                }
            }
        } else {
            // Broadcast events: send to ALL children
            // Other event types: send to focused child only
            if event.what == EventType::Broadcast {
                // Handle CM_FOCUS_LINK: Label hotkey requests focus on linked control
                if event.command == crate::core::command::CM_FOCUS_LINK {
                    let view_id = super::view::ViewId::from_u16(event.key_code);
                    if self.group_mut().focus_by_view_id(view_id) {
                        event.clear();
                    }
                    return;
                }
                // Matches Borland: TGroup::handleEvent() broadcasts to ALL
                // children via forEach(doHandleEvent) — delivery does not stop
                // when one child clears the event, so every child sees the
                // broadcast.
                for child in &mut self.group_mut().children {
                    child.handle_event(event);
                }
            } else {
                // Other event types: send to focused child only
                if self.group().focused < self.group_mut().children.len() {
                    {
                        let g = self.group_mut();
                        g.children[g.focused].handle_event(event);
                    }
                }
            }
        }
    }

    fn group_update_cursor(&self, terminal: &mut Terminal) {
        // Hide cursor by default
        let _ = terminal.hide_cursor();

        // Update cursor for the focused child (it can show it if needed)
        if self.group().focused < self.group().children.len() {
            self.group().children[self.group().focused].update_cursor(terminal);
        }
    }

    /// Validate group before performing command
    /// Matches Borland: TGroup::valid(ushort command)
    /// - If command is CM_RELEASED_FOCUS, validate current focused child if it has OF_VALIDATE
    /// - Otherwise, validate all children (return false if any child is invalid)
    fn group_valid(&mut self, command: crate::core::command::CommandId) -> bool {
        use crate::core::command::CM_RELEASED_FOCUS;
        use crate::core::state::OF_VALIDATE;

        if command == CM_RELEASED_FOCUS {
            // Validate only the currently focused child if it has OF_VALIDATE flag
            if self.group().focused < self.group_mut().children.len() {
                let g = self.group_mut();
                let child = &mut g.children[g.focused];
                if (child.options() & OF_VALIDATE) != 0 {
                    return child.valid(command);
                }
            }
            true
        } else {
            // Validate all children - return false if any child is invalid
            // Matches Borland: firstThat(isInvalid, &command) == nullptr
            for child in &mut self.group_mut().children {
                if !child.valid(command) {
                    return false;
                }
            }
            true
        }
    }

    // ---- modal loop (Borland: TGroup::execute) ----

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
    fn execute(&mut self, app: &mut crate::app::Application) -> crate::core::command::CommandId {
        self.group_mut().end_state = 0;

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
            if self.group().end_state != 0 {
                // Matches Borland: do { ... } while( !valid(endState) ) —
                // a failing validator vetoes the close and re-enters the loop
                let end_state = self.group().end_state;
                if self.valid(end_state) {
                    break;
                }
                self.group_mut().end_state = 0;
            }
        }

        self.group().end_state
    }

    /// End the modal event loop with a result code
    /// Matches Borland: TView::endModal(ushort command) (tview.cc:391-395)
    fn end_modal(&mut self, command: crate::core::command::CommandId) {
        self.group_mut().end_state = command;
    }

    /// The command the modal loop ended with, or 0 while it is still running.
    fn end_state(&self) -> crate::core::command::CommandId {
        self.group().end_state
    }

    // ---- child access, forwarded to Group's inherent methods ----

    fn add(&mut self, view: Box<dyn View>) -> ViewId {
        self.group_mut().add(view)
    }
    fn child_count(&self) -> usize {
        self.group().len()
    }
    fn child_at(&self, index: usize) -> &dyn View {
        self.group().child_at(index)
    }
    fn child_at_mut(&mut self, index: usize) -> &mut dyn View {
        self.group_mut().child_at_mut(index)
    }
    fn child_by_id(&self, id: ViewId) -> Option<&dyn View> {
        self.group().child_by_id(id)
    }
    fn child_by_id_mut(&mut self, id: ViewId) -> Option<&mut (dyn View + '_)> {
        self.group_mut().child_by_id_mut(id)
    }
    fn remove_by_id(&mut self, id: ViewId) -> bool {
        self.group_mut().remove_by_id(id)
    }
    fn set_initial_focus(&mut self) {
        self.group_mut().set_initial_focus();
    }
    fn set_focus_to(&mut self, index: usize) {
        self.group_mut().set_focus_to(index);
    }
    fn broadcast(&mut self, event: &mut Event, owner_index: Option<usize>) {
        self.group_mut().broadcast(event, owner_index);
    }
}

impl GroupLike for Group {
    fn group(&self) -> &Group {
        self
    }
    fn group_mut(&mut self) -> &mut Group {
        self
    }
}

impl View for Group {
    fn core(&self) -> &ViewCore {
        &self.core
    }

    fn core_mut(&mut self) -> &mut ViewCore {
        &mut self.core
    }

    fn set_bounds(&mut self, bounds: Rect) {
        self.group_set_bounds(bounds)
    }

    fn draw(&mut self, terminal: &mut Terminal) {
        self.group_draw(terminal)
    }

    fn handle_event(&mut self, event: &mut Event) {
        self.group_handle_event(event)
    }

    fn update_cursor(&self, terminal: &mut Terminal) {
        self.group_update_cursor(terminal)
    }

    fn valid(&mut self, command: crate::core::command::CommandId) -> bool {
        self.group_valid(command)
    }

    fn get_end_state(&self) -> crate::core::command::CommandId {
        self.end_state()
    }

    fn set_end_state(&mut self, command: crate::core::command::CommandId) {
        self.end_modal(command)
    }

    fn get_palette(&self) -> Option<crate::core::palette::Palette> {
        // TGroup has no palette (returns empty palette in Borland)
        // Returning None achieves the same effect - skip to parent's palette
        None
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}

/// Builder for creating groups with a fluent API.
pub struct GroupBuilder {
    bounds: Option<Rect>,
    background: Option<Attr>,
}

impl GroupBuilder {
    pub fn new() -> Self {
        Self {
            bounds: None,
            background: None,
        }
    }

    #[must_use]
    pub fn bounds(mut self, bounds: Rect) -> Self {
        self.bounds = Some(bounds);
        self
    }

    #[must_use]
    pub fn background(mut self, background: Attr) -> Self {
        self.background = Some(background);
        self
    }

    pub fn build(self) -> Group {
        let bounds = self.bounds.expect("Group bounds must be set");
        if let Some(bg) = self.background {
            Group::with_background(bounds, bg)
        } else {
            Group::new(bounds)
        }
    }

    pub fn build_boxed(self) -> Box<Group> {
        Box::new(self.build())
    }
}

impl Default for GroupBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn group_like_handle_event_dispatches_to_the_outer_override() {
        use std::cell::Cell;
        use std::rc::Rc;

        struct Counting {
            group: Group,
            seen: Rc<Cell<u32>>,
        }
        impl View for Counting {
            fn core(&self) -> &ViewCore {
                self.group.core()
            }
            fn core_mut(&mut self) -> &mut ViewCore {
                self.group.core_mut()
            }
            fn draw(&mut self, t: &mut Terminal) {
                self.group_draw(t)
            }
            fn handle_event(&mut self, e: &mut Event) {
                self.seen.set(self.seen.get() + 1);
                self.group_handle_event(e); // base call
                if e.what == EventType::Command && e.command == 42 {
                    self.end_modal(42);
                    e.clear();
                }
            }
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
        impl GroupLike for Counting {
            fn group(&self) -> &Group {
                &self.group
            }
            fn group_mut(&mut self) -> &mut Group {
                &mut self.group
            }
        }

        let seen = Rc::new(Cell::new(0));
        let mut c = Counting {
            group: Group::new(Rect::new(0, 0, 10, 10)),
            seen: Rc::clone(&seen),
        };
        let mut ev = Event::command(42);
        // drive one iteration of the loop body by hand, as execute() needs an Application
        c.handle_event(&mut ev);
        assert_eq!(seen.get(), 1);
        assert_eq!(c.end_state(), 42);
        assert_eq!(ev.what, EventType::Nothing);
    }

    // Helper to count how many times draw is called on views
    struct DrawCountView {
        core: ViewCore,
        draw_count: std::cell::RefCell<usize>,
    }

    impl DrawCountView {
        fn new(bounds: Rect) -> Self {
            Self {
                core: ViewCore {
                    bounds,
                    ..ViewCore::default()
                },
                draw_count: std::cell::RefCell::new(0),
            }
        }
    }

    impl View for DrawCountView {
        fn core(&self) -> &ViewCore {
            &self.core
        }

        fn core_mut(&mut self) -> &mut ViewCore {
            &mut self.core
        }

        fn draw(&mut self, _terminal: &mut Terminal) {
            *self.draw_count.borrow_mut() += 1;
        }

        fn handle_event(&mut self, _event: &mut Event) {}

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

    // Test view that records events, can take focus, and stores a grow mode
    struct RecorderView {
        core: ViewCore,
        events: std::rc::Rc<std::cell::RefCell<Vec<EventType>>>,
    }

    impl RecorderView {
        fn new(bounds: Rect) -> Self {
            Self {
                core: ViewCore {
                    bounds,
                    state: 0,
                    grow_mode: 0,
                    ..ViewCore::default()
                },
                events: std::rc::Rc::new(std::cell::RefCell::new(Vec::new())),
            }
        }
    }

    impl View for RecorderView {
        fn core(&self) -> &ViewCore {
            &self.core
        }

        fn core_mut(&mut self) -> &mut ViewCore {
            &mut self.core
        }

        fn draw(&mut self, _terminal: &mut Terminal) {}

        fn handle_event(&mut self, event: &mut Event) {
            self.events.borrow_mut().push(event.what);
        }

        fn can_focus(&self) -> bool {
            true
        }

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

    #[test]
    fn test_mouse_down_outside_children_not_sent_to_focused() {
        use crate::core::geometry::Point;

        let mut group = Group::new(Rect::new(0, 0, 80, 25));
        let child = RecorderView::new(Rect::new(0, 0, 10, 5));
        let events = child.events.clone();
        group.add(Box::new(child));
        group.set_initial_focus();

        // MouseDown on empty group area (outside the child at 0,0-10,5)
        let mut event = Event::mouse(EventType::MouseDown, Point::new(50, 20), 1, false);
        group.handle_event(&mut event);

        // The focused child must NOT have received the positional event
        assert!(events.borrow().is_empty());

        // But a click ON the child is still delivered (focus-on-click intact)
        let mut event = Event::mouse(EventType::MouseDown, Point::new(5, 2), 1, false);
        group.handle_event(&mut event);
        assert_eq!(events.borrow().as_slice(), &[EventType::MouseDown]);
    }

    #[test]
    fn test_grow_modes_on_resize() {
        use crate::core::state::{GF_GROW_ALL, GF_GROW_HI_X, GF_GROW_HI_Y};

        let mut group = Group::new(Rect::new(0, 0, 40, 20));

        // Fixed child (grow_mode = 0, Borland default)
        group.add(Box::new(RecorderView::new(Rect::new(1, 1, 11, 3))));

        // Right/bottom-growing child (gfGrowHiX | gfGrowHiY)
        let mut growing = RecorderView::new(Rect::new(1, 5, 11, 7));
        growing.set_grow_mode(GF_GROW_HI_X | GF_GROW_HI_Y);
        group.add(Box::new(growing));

        // Fully growing child (gfGrowAll — moves with the far edge)
        let mut all = RecorderView::new(Rect::new(30, 15, 39, 19));
        all.set_grow_mode(GF_GROW_ALL);
        group.add(Box::new(all));

        // Resize the group: +10 wide, +5 tall (no move)
        group.set_bounds(Rect::new(0, 0, 50, 25));

        // Fixed child: unchanged
        assert_eq!(group.child_at(0).bounds(), Rect::new(1, 1, 11, 3));
        // HiX|HiY child: only b edge moved
        assert_eq!(group.child_at(1).bounds(), Rect::new(1, 5, 21, 12));
        // GrowAll child: both edges moved
        assert_eq!(group.child_at(2).bounds(), Rect::new(40, 20, 49, 24));

        // Moving the group (no size change) shifts all children equally
        group.set_bounds(Rect::new(5, 2, 55, 27));
        assert_eq!(group.child_at(0).bounds(), Rect::new(6, 3, 16, 5));
        assert_eq!(group.child_at(1).bounds(), Rect::new(6, 7, 26, 14));
        assert_eq!(group.child_at(2).bounds(), Rect::new(45, 22, 54, 26));
    }

    #[test]
    fn test_focus_restored_after_removing_focused_child() {
        use crate::core::state::SF_FOCUSED;

        let mut group = Group::new(Rect::new(0, 0, 80, 25));
        group.add(Box::new(RecorderView::new(Rect::new(0, 0, 10, 2))));
        group.add(Box::new(RecorderView::new(Rect::new(0, 3, 10, 5))));
        group.add(Box::new(RecorderView::new(Rect::new(0, 6, 10, 8))));

        group.set_focus_to(1);
        assert!(group.child_at(1).is_focused());

        // Remove the focused child — focus must land on a remaining child
        group.remove(1);
        assert_eq!(group.len(), 2);
        let focused_count = (0..group.len())
            .filter(|&i| (group.child_at(i).state() & SF_FOCUSED) != 0)
            .count();
        assert_eq!(focused_count, 1);
        assert!(group.focused_child().unwrap().is_focused());
    }

    #[test]
    fn test_broadcast_delivered_to_all_children() {
        // A child that clears broadcast events (simulates a "consumer")
        struct Consumer {
            core: ViewCore,
        }
        impl View for Consumer {
            fn core(&self) -> &ViewCore {
                &self.core
            }

            fn core_mut(&mut self) -> &mut ViewCore {
                &mut self.core
            }

            fn draw(&mut self, _terminal: &mut Terminal) {}
            fn handle_event(&mut self, event: &mut Event) {
                if event.what == EventType::Broadcast {
                    event.clear();
                }
            }
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

        let mut group = Group::new(Rect::new(0, 0, 80, 25));
        // First child consumes broadcasts
        group.add(Box::new(Consumer {
            core: ViewCore::new(Rect::new(0, 0, 5, 1)),
        }));
        // Second child records what it receives
        let recorder = RecorderView::new(Rect::new(0, 2, 5, 3));
        let events = recorder.events.clone();
        group.add(Box::new(recorder));

        let mut event = Event::broadcast(9999);
        group.handle_event(&mut event);

        // The second child was still visited even though the first cleared
        // the event (Borland delivers broadcasts to every child)
        assert_eq!(events.borrow().len(), 1);
    }

    #[test]
    fn test_child_completely_outside_parent_not_drawn() {
        // Create a group at (10, 10) with size 20x20
        let group = Group::new(Rect::new(10, 10, 30, 30));

        // Add a child completely outside the parent bounds (to the right)
        let child_bounds = Rect::new(100, 15, 110, 20);

        // Verify the child is outside parent bounds
        assert!(!group.bounds().intersects(&child_bounds));
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
        assert!(group.bounds().intersects(&group.children[0].bounds()));
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
        assert!(group.bounds().intersects(&group.children[0].bounds()));

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
        assert!(group.bounds().intersects(&group.children[0].bounds()));

        // Child 2: completely outside, should NOT intersect
        assert!(!group.bounds().intersects(&group.children[1].bounds()));

        // Child 3: partially outside, should intersect
        assert!(group.bounds().intersects(&group.children[2].bounds()));
    }

    #[test]
    fn test_child_by_id() {
        // Create a group and add children
        let mut group = Group::new(Rect::new(0, 0, 50, 50));

        let child1 = Box::new(DrawCountView::new(Rect::new(0, 0, 10, 10)));
        let id1 = group.add(child1);

        let child2 = Box::new(DrawCountView::new(Rect::new(20, 0, 30, 10)));
        let id2 = group.add(child2);

        let child3 = Box::new(DrawCountView::new(Rect::new(40, 0, 50, 10)));
        let id3 = group.add(child3);

        // Test accessing children by ID (immutable)
        assert!(group.child_by_id(id1).is_some());
        assert!(group.child_by_id(id2).is_some());
        assert!(group.child_by_id(id3).is_some());

        // Test that invalid ID returns None
        let invalid_id = ViewId::new();
        assert!(group.child_by_id(invalid_id).is_none());
    }

    #[test]
    fn test_child_by_id_mut() {
        // Create a group and add a child
        let mut group = Group::new(Rect::new(0, 0, 50, 50));

        let child = Box::new(DrawCountView::new(Rect::new(0, 0, 10, 10)));
        let child_id = group.add(child);

        // Test accessing child by ID (mutable)
        let child_ref = group.child_by_id_mut(child_id);
        assert!(child_ref.is_some());

        // Test that invalid ID returns None
        let invalid_id = ViewId::new();
        assert!(group.child_by_id_mut(invalid_id).is_none());
    }

    #[test]
    fn test_remove_by_id() {
        // Create a group and add multiple children
        let mut group = Group::new(Rect::new(0, 0, 50, 50));

        let child1 = Box::new(DrawCountView::new(Rect::new(0, 0, 10, 10)));
        let id1 = group.add(child1);

        let child2 = Box::new(DrawCountView::new(Rect::new(20, 0, 30, 10)));
        let id2 = group.add(child2);

        let child3 = Box::new(DrawCountView::new(Rect::new(40, 0, 50, 10)));
        let id3 = group.add(child3);

        // Verify all children are present
        assert_eq!(group.len(), 3);
        assert!(group.child_by_id(id1).is_some());
        assert!(group.child_by_id(id2).is_some());
        assert!(group.child_by_id(id3).is_some());

        // Remove middle child by ID
        let removed = group.remove_by_id(id2);
        assert!(removed);
        assert_eq!(group.len(), 2);

        // Verify id2 is gone but id1 and id3 are still there
        assert!(group.child_by_id(id1).is_some());
        assert!(group.child_by_id(id2).is_none());
        assert!(group.child_by_id(id3).is_some());

        // Try to remove invalid ID
        let invalid_id = ViewId::new();
        let not_removed = group.remove_by_id(invalid_id);
        assert!(!not_removed);
        assert_eq!(group.len(), 2);
    }

    #[test]
    fn test_bring_to_front_syncs_view_ids() {
        use crate::core::geometry::Rect;

        let mut group = Group::new(Rect::new(0, 0, 80, 25));
        let id1 = group.add(Box::new(crate::views::background::Background::new(
            Rect::new(0, 0, 10, 5),
            ' ',
            crate::core::palette::Attr::new(
                crate::core::palette::TvColor::White,
                crate::core::palette::TvColor::Blue,
            ),
        )));
        let id2 = group.add(Box::new(crate::views::background::Background::new(
            Rect::new(0, 0, 10, 5),
            ' ',
            crate::core::palette::Attr::new(
                crate::core::palette::TvColor::White,
                crate::core::palette::TvColor::Blue,
            ),
        )));
        let id3 = group.add(Box::new(crate::views::background::Background::new(
            Rect::new(0, 0, 10, 5),
            ' ',
            crate::core::palette::Attr::new(
                crate::core::palette::TvColor::White,
                crate::core::palette::TvColor::Blue,
            ),
        )));

        // Bring first child to front
        group.bring_to_front(0);

        // After bring_to_front(0): order should be [id2, id3, id1]
        // Verify child_by_id still works correctly
        assert!(group.child_by_id(id1).is_some());
        assert!(group.child_by_id(id2).is_some());
        assert!(group.child_by_id(id3).is_some());

        // The brought-to-front child (id1) should now be at the last index
        // Verify by checking that view_ids[2] == id1
        // We can test this indirectly: remove_by_id should still find the right child
        assert!(group.remove_by_id(id1));
        assert_eq!(group.len(), 2);
        // id2 and id3 should still be findable
        assert!(group.child_by_id(id2).is_some());
        assert!(group.child_by_id(id3).is_some());
    }

    #[test]
    fn test_send_to_back_syncs_view_ids() {
        use crate::core::geometry::Rect;

        let mut group = Group::new(Rect::new(0, 0, 80, 25));
        let id1 = group.add(Box::new(crate::views::background::Background::new(
            Rect::new(0, 0, 10, 5),
            ' ',
            crate::core::palette::Attr::new(
                crate::core::palette::TvColor::White,
                crate::core::palette::TvColor::Blue,
            ),
        )));
        let id2 = group.add(Box::new(crate::views::background::Background::new(
            Rect::new(0, 0, 10, 5),
            ' ',
            crate::core::palette::Attr::new(
                crate::core::palette::TvColor::White,
                crate::core::palette::TvColor::Blue,
            ),
        )));
        let id3 = group.add(Box::new(crate::views::background::Background::new(
            Rect::new(0, 0, 10, 5),
            ' ',
            crate::core::palette::Attr::new(
                crate::core::palette::TvColor::White,
                crate::core::palette::TvColor::Blue,
            ),
        )));

        // Send last child to back (position 1, after index 0)
        group.send_to_back(2);

        // After send_to_back(2): order should be [id1, id3, id2]
        // All IDs should still be findable
        assert!(group.child_by_id(id1).is_some());
        assert!(group.child_by_id(id2).is_some());
        assert!(group.child_by_id(id3).is_some());

        // Remove id3 (should be at index 1 now) to verify sync
        assert!(group.remove_by_id(id3));
        assert_eq!(group.len(), 2);
        assert!(group.child_by_id(id1).is_some());
        assert!(group.child_by_id(id2).is_some());
    }

    #[test]
    fn test_child_by_id_fragility_fix() {
        // This test demonstrates the fragility fix that child_by_id() solves
        let mut group = Group::new(Rect::new(0, 0, 50, 50));

        let child1 = Box::new(DrawCountView::new(Rect::new(0, 0, 10, 10)));
        let id1 = group.add(child1);

        let child2 = Box::new(DrawCountView::new(Rect::new(20, 0, 30, 10)));
        let id2 = group.add(child2);

        let child3 = Box::new(DrawCountView::new(Rect::new(40, 0, 50, 10)));
        let id3 = group.add(child3);

        // With indices, we would have: index 0 = id1, index 1 = id2, index 2 = id3
        // If we stored index 1 for "the button" and then inserted a new child before it,
        // our stored index 1 would now point to the new child, not the button!

        // But with ViewIds, the IDs are stable regardless of insertion order
        assert!(group.child_by_id(id1).is_some());
        assert!(group.child_by_id(id2).is_some());
        assert!(group.child_by_id(id3).is_some());

        // If we insert a new child at the beginning (simulating reordering)
        let new_child = Box::new(DrawCountView::new(Rect::new(0, 20, 10, 30)));
        let new_id = group.add(new_child);

        // The old IDs still work correctly because they're not affected by reordering
        assert!(group.child_by_id(id1).is_some());
        assert!(group.child_by_id(id2).is_some());
        assert!(group.child_by_id(id3).is_some());
        assert!(group.child_by_id(new_id).is_some());
    }
}
