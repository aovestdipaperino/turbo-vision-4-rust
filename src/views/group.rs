// (C) 2025 - Enzo Lombardi

//! Group view - container for managing multiple child views with focus handling.

use super::view::{View, ViewCore, ViewId, write_line_to_terminal};
use crate::core::draw::DrawBuffer;
use crate::core::event::{Event, EventType, KB_SHIFT_TAB, KB_TAB};
use crate::core::geometry::{Point, Rect};
use crate::core::palette::Attr;
use crate::core::state::Options;
use crate::core::state::{Grow, State};
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
    /// How far past its own extent this group lets a child paint. Zero for an
    /// ordinary group, so an oversized child is clipped instead of drawing
    /// over whatever surrounds the group (a window's frame, say). The desktop
    /// raises it, because a window draws its shadow outside its own bounds.
    child_overhang: Point,
}

impl Group {
    pub fn new(bounds: Rect) -> Self {
        Self {
            core: ViewCore {
                bounds,
                palette_chain: None,
                grow_mode: Grow::empty(),
                ..ViewCore::default()
            },
            children: Vec::new(),
            view_ids: Vec::new(),
            focused: 0,
            background: None,
            end_state: 0,
            child_overhang: Point::new(0, 0),
        }
    }

    /// Let this group's children paint up to `overhang` cells past its extent.
    ///
    /// Defaults to nothing: a child larger than the group is clipped to it, so
    /// it cannot erase the frame of the window that owns the group. Raise it
    /// only where an overhang is part of the design, as the desktop does for
    /// the shadow a window casts outside its own bounds.
    pub fn set_child_overhang(&mut self, overhang: Point) {
        self.child_overhang = overhang;
    }

    /// How far past its extent this group lets a child paint.
    pub fn child_overhang(&self) -> Point {
        self.child_overhang
    }

    pub fn with_background(bounds: Rect, background: Attr) -> Self {
        Self {
            core: ViewCore {
                bounds,
                palette_chain: None,
                grow_mode: Grow::empty(),
                ..ViewCore::default()
            },
            children: Vec::new(),
            view_ids: Vec::new(),
            focused: 0,
            background: Some(background),
            end_state: 0,
            child_overhang: Point::new(0, 0),
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

    /// Add an already boxed child. `GroupLike::add` takes any view and boxes
    /// it; this is the primitive underneath.
    pub fn add_boxed(&mut self, view: Box<dyn View>) -> ViewId {
        // The child's bounds are relative to this group and stay that way
        // (Borland: TView::origin is owner-relative).
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
    /// Redraw the children from `start_index` on that intersect `clip`
    /// (`clip` in this group's space).
    pub fn draw_sub_views(&mut self, terminal: &mut Terminal, start_index: usize, clip: Rect) {
        terminal.push_clip(clip);
        for i in start_index..self.children.len() {
            let child_bounds = self.children[i].bounds();
            if clip.intersects(&child_bounds) {
                terminal.push_origin(child_bounds.a);
                self.children[i].draw(terminal);
                terminal.pop_origin();
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
    /// disabled. (State::VISIBLE is not checked because this port never sets it;
    /// hidden views are simply not added to the group.)
    fn child_focusable(&self, index: usize) -> bool {
        let child = &self.children[index];
        child.can_focus() && !child.state().contains(State::DISABLED)
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
        // Children are owner-relative, so a move leaves them alone; only a
        // size change reaches them, edge by edge, through their grow bits.
        // Matches Borland: TGroup::changeBounds() -> TView::calcBounds().
        let dw = bounds.width() - self.bounds().width();
        let dh = bounds.height() - self.bounds().height();
        self.core_mut().bounds = bounds;
        if dw == 0 && dh == 0 {
            return;
        }
        for child in &mut self.group_mut().children {
            let grow = child.grow_mode();
            let child_bounds = child.bounds();
            let new_bounds = Rect::new(
                child_bounds.a.x + if grow.contains(Grow::LO_X) { dw } else { 0 },
                child_bounds.a.y + if grow.contains(Grow::LO_Y) { dh } else { 0 },
                child_bounds.b.x + if grow.contains(Grow::HI_X) { dw } else { 0 },
                child_bounds.b.y + if grow.contains(Grow::HI_Y) { dh } else { 0 },
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
                write_line_to_terminal(terminal, 0, y as i16, &buf);
            }
        }

        // Clip children to this group's extent. A group that expects an
        // overhang (the desktop, whose windows cast a shadow outside their own
        // bounds) asks for it; everything else clips tight, so a child too big
        // for its group cannot paint over the frame around it.
        let overhang = self.group().child_overhang;
        let mut clip_bounds = self.extent();
        clip_bounds.grow(overhang.x, overhang.y);
        terminal.push_clip(clip_bounds);

        // Build this Group's palette chain node for safe palette traversal.
        // Group is typically transparent (no palette), but carries the parent link.
        let my_chain_node = crate::core::palette_chain::PaletteChainNode::new(
            self.get_palette(),
            self.get_palette_chain().cloned(),
        );

        // Each child draws in its own space: push its origin around draw().
        let extent = self.extent();
        for child in &mut self.group_mut().children {
            child.set_palette_chain(Some(my_chain_node.clone()));
            let child_bounds = child.bounds();
            if extent.intersects(&child_bounds) {
                terminal.push_origin(child_bounds.a);
                child.draw(terminal);
                terminal.pop_origin();
            }
        }

        // Pop clipping region
        terminal.pop_clip();
    }

    fn group_handle_event(&mut self, event: &mut Event) {
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
                if child_state.intersects(State::DRAGGING | State::RESIZING) {
                    let focused = g.focused;
                    self.dispatch_to_child(focused, event);
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

                // Second pass: handle the event, in the child's own space
                self.dispatch_to_child(i, event);

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
        // Phase 1: PreProcess - views with Options::PRE_PROCESS flag (e.g., buttons for Space/Enter)
        // Phase 2: Focused - currently focused view gets first chance
        // Phase 3: PostProcess - views with Options::POST_PROCESS flag (e.g., status line for help keys)

        if event.what == EventType::Keyboard || event.what == EventType::Command {
            // Phase 1: PreProcess
            // Views with Options::PRE_PROCESS get first chance at the event
            for i in 0..self.group().children.len() {
                if event.what == EventType::Nothing {
                    break; // Event was handled
                }
                if self.group().children[i].options().contains(Options::PRE_PROCESS) {
                    self.dispatch_to_child(i, event);
                }
            }

            // Phase 2: Focused
            // Give focused view a chance if event wasn't handled
            if event.what != EventType::Nothing
                && self.group().focused < self.group_mut().children.len()
            {
                let focused = self.group().focused;
                self.dispatch_to_child(focused, event);
            }

            // Phase 3: PostProcess
            // Views with Options::POST_PROCESS get last chance (e.g., status line, buttons)
            if event.what != EventType::Nothing {
                for i in 0..self.group().children.len() {
                    if event.what == EventType::Nothing {
                        break; // Event was handled
                    }
                    if self.group().children[i].options().contains(Options::POST_PROCESS) {
                        self.dispatch_to_child(i, event);
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
                for i in 0..self.group().children.len() {
                    self.dispatch_to_child(i, event);
                }
            } else {
                // Other event types (mouse wheel among them): focused child only
                if self.group().focused < self.group_mut().children.len() {
                    let focused = self.group().focused;
                    self.dispatch_to_child(focused, event);
                }
            }
        }
    }

    /// Hand a positional event to a child in the child's own coordinate
    /// space, then put the position back into this group's space whatever
    /// the child turned the event into. That last step is how a control
    /// reports an anchor upward (History, ComboBox) without an owner chain:
    /// every group on the way back adds its child's origin.
    fn dispatch_to_child(&mut self, index: usize, event: &mut Event) {
        let origin = self.group().children[index].bounds().a;
        event.mouse.pos.x -= origin.x;
        event.mouse.pos.y -= origin.y;
        self.group_mut().children[index].handle_event(event);
        event.mouse.pos.x += origin.x;
        event.mouse.pos.y += origin.y;
    }

    fn group_update_cursor(&self, terminal: &mut Terminal) {
        // Hide cursor by default
        let _ = terminal.hide_cursor();

        // Update cursor for the focused child (it can show it if needed)
        if self.group().focused < self.group().children.len() {
            let child = &self.group().children[self.group().focused];
            terminal.push_origin(child.bounds().a);
            child.update_cursor(terminal);
            terminal.pop_origin();
        }
    }

    /// Validate group before performing command
    /// Matches Borland: TGroup::valid(ushort command)
    /// - If command is CM_RELEASED_FOCUS, validate current focused child if it has Options::VALIDATE
    /// - Otherwise, validate all children (return false if any child is invalid)
    fn group_valid(&mut self, command: crate::core::command::CommandId) -> bool {
        use crate::core::command::CM_RELEASED_FOCUS;

        if command == CM_RELEASED_FOCUS {
            // Validate only the currently focused child if it has Options::VALIDATE flag
            if self.group().focused < self.group_mut().children.len() {
                let g = self.group_mut();
                let child = &mut g.children[g.focused];
                if child.options().contains(Options::VALIDATE) {
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

    /// Add a child (Borland: `TGroup::insert`). Takes any view; a
    /// `Box<dyn View>` is accepted too, so older `add(Box::new(v))` calls
    /// still compile.
    fn add<V: View + 'static>(&mut self, view: V) -> ViewId
    where
        Self: Sized,
    {
        self.add_boxed(Box::new(view))
    }
    /// The object-safe primitive behind `add`.
    fn add_boxed(&mut self, view: Box<dyn View>) -> ViewId {
        self.group_mut().add_boxed(view)
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

    // ---- typed child access (Borland: a typed `TView*` to a child) ----
    // `where Self: Sized` keeps the trait usable as `dyn GroupLike`.

    fn add_typed<T: View + 'static>(&mut self, view: T) -> super::handle::Handle<T>
    where
        Self: Sized,
    {
        super::handle::Handle::from_id(self.add_boxed(Box::new(view)))
    }
    fn get<T: View + 'static>(&self, handle: super::handle::Handle<T>) -> Option<&T>
    where
        Self: Sized,
    {
        self.child_by_id(handle.id())?.as_any().downcast_ref::<T>()
    }
    fn get_mut<T: View + 'static>(&mut self, handle: super::handle::Handle<T>) -> Option<&mut T>
    where
        Self: Sized,
    {
        self.child_by_id_mut(handle.id())?
            .as_any_mut()
            .downcast_mut::<T>()
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

    fn as_group(&self) -> Option<&dyn GroupLike> {
        Some(self)
    }

    fn as_group_mut(&mut self) -> Option<&mut dyn GroupLike> {
        Some(self)
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
    use crate::core::event::MB_LEFT_BUTTON;

    /// Paint every cell of a test terminal, so a later assertion can tell an
    /// untouched cell from one a view drew over.
    #[cfg(test)]
    fn fill(terminal: &mut Terminal, ch: char) {
        use crate::core::palette::TvColor;
        let (w, h) = terminal.size();
        for y in 0..h {
            let mut buf = DrawBuffer::new(w as usize);
            buf.move_char(0, ch, Attr::new(TvColor::White, TvColor::Black), w as usize);
            terminal.write_line(0, y, &buf.data);
        }
    }

    /// A child too big for the group it sits in must be clipped to the group,
    /// not allowed to paint over whatever surrounds it. Reported as #108: an
    /// ASCII table in a window shrunk below its content erased the window's
    /// right border and bottom edge.
    #[test]
    fn a_child_larger_than_its_group_is_clipped_to_the_group() {
        use crate::test_util::test_terminal;
        use crate::views::static_text::StaticText;

        let mut terminal = test_terminal(40, 6);
        // Paint the whole screen so anything the group leaves alone stays '#'.
        fill(&mut terminal, '#');

        // A 10x3 group holding a child twice its width and a row too tall.
        let mut group = Group::new(Rect::new(5, 1, 15, 4));
        group.add(StaticText::new(Rect::new(0, 0, 20, 4), "XXXXXXXXXXXXXXXXXXXX"));
        terminal.draw_view(&mut group);

        assert_eq!(
            terminal.read_cell(14, 1).map(|c| c.ch),
            Some('X'),
            "the child paints inside the group"
        );
        assert_eq!(
            terminal.read_cell(15, 1).map(|c| c.ch),
            Some('#'),
            "the column just past the group's right edge is untouched"
        );
        assert_eq!(
            terminal.read_cell(16, 1).map(|c| c.ch),
            Some('#'),
            "and so is the one after it"
        );
        assert_eq!(
            terminal.read_cell(5, 4).map(|c| c.ch),
            Some('#'),
            "the row just past the group's bottom edge is untouched"
        );
    }

    /// The desktop is the exception: a window draws its shadow outside its own
    /// bounds, so the group holding the windows allows that much overhang.
    #[test]
    fn a_group_can_allow_its_children_an_overhang() {
        use crate::test_util::test_terminal;
        use crate::views::static_text::StaticText;

        let mut terminal = test_terminal(40, 6);
        fill(&mut terminal, '#');

        let mut group = Group::new(Rect::new(5, 1, 15, 4));
        group.set_child_overhang(Point::new(2, 1));
        group.add(StaticText::new(Rect::new(0, 0, 20, 4), "XXXXXXXXXXXXXXXXXXXX"));
        terminal.draw_view(&mut group);

        assert_eq!(
            terminal.read_cell(16, 1).map(|c| c.ch),
            Some('X'),
            "two columns of overhang are allowed"
        );
        assert_eq!(
            terminal.read_cell(17, 1).map(|c| c.ch),
            Some('#'),
            "but no more than that"
        );
    }

    #[test]
    fn add_accepts_unboxed_and_boxed_views() {
        use crate::views::static_text::StaticText;
        let mut g = Group::new(Rect::new(0, 0, 40, 10));
        g.add(StaticText::new(Rect::new(0, 0, 5, 1), "a"));
        let boxed: Box<dyn View> = Box::new(StaticText::new(Rect::new(0, 1, 5, 2), "b"));
        g.add(boxed);
        assert_eq!(g.len(), 2);
    }

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
                    state: State::empty(),
                    grow_mode: Grow::empty(),
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
        group.add(child);
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
        let mut group = Group::new(Rect::new(0, 0, 40, 20));

        // Fixed child (grow_mode = 0, Borland default)
        group.add(RecorderView::new(Rect::new(1, 1, 11, 3)));

        // Right/bottom-growing child (gfGrowHiX | gfGrowHiY)
        let mut growing = RecorderView::new(Rect::new(1, 5, 11, 7));
        growing.set_grow_mode(Grow::HI_X | Grow::HI_Y);
        group.add(growing);

        // Fully growing child (gfGrowAll — moves with the far edge)
        let mut all = RecorderView::new(Rect::new(30, 15, 39, 19));
        all.set_grow_mode(Grow::ALL);
        group.add(all);

        // Resize the group: +10 wide, +5 tall (no move)
        group.set_bounds(Rect::new(0, 0, 50, 25));

        // Fixed child: unchanged
        assert_eq!(group.child_at(0).bounds(), Rect::new(1, 1, 11, 3));
        // HiX|HiY child: only b edge moved
        assert_eq!(group.child_at(1).bounds(), Rect::new(1, 5, 21, 12));
        // GrowAll child: both edges moved
        assert_eq!(group.child_at(2).bounds(), Rect::new(40, 20, 49, 24));

        // Moving the group (no size change) leaves owner-relative children alone
        group.set_bounds(Rect::new(5, 2, 55, 27));
        assert_eq!(group.child_at(0).bounds(), Rect::new(1, 1, 11, 3));
        assert_eq!(group.child_at(1).bounds(), Rect::new(1, 5, 21, 12));
        assert_eq!(group.child_at(2).bounds(), Rect::new(40, 20, 49, 24));
    }

    #[test]
    fn test_focus_restored_after_removing_focused_child() {
        let mut group = Group::new(Rect::new(0, 0, 80, 25));
        group.add(RecorderView::new(Rect::new(0, 0, 10, 2)));
        group.add(RecorderView::new(Rect::new(0, 3, 10, 5)));
        group.add(RecorderView::new(Rect::new(0, 6, 10, 8)));

        group.set_focus_to(1);
        assert!(group.child_at(1).is_focused());

        // Remove the focused child — focus must land on a remaining child
        group.remove(1);
        assert_eq!(group.len(), 2);
        let focused_count = (0..group.len())
            .filter(|&i| group.child_at(i).state().contains(State::FOCUSED))
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
        group.add(Consumer {
            core: ViewCore::new(Rect::new(0, 0, 5, 1)),
        });
        // Second child records what it receives
        let recorder = RecorderView::new(Rect::new(0, 2, 5, 3));
        let events = recorder.events.clone();
        group.add(recorder);

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

        assert_eq!(group.children.len(), 1);
        assert_eq!(group.children[0].bounds(), Rect::new(5, 5, 15, 15));
        assert!(group.extent().intersects(&group.children[0].bounds()));
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

        // Bounds stay owner-relative; the child overlaps the extent edge
        assert_eq!(group.children[0].bounds(), Rect::new(15, 15, 25, 25));
        assert!(group.extent().intersects(&group.children[0].bounds()));

        // Note: The child will be drawn, but the Terminal's write methods
        // will clip at the terminal boundaries. For proper parent clipping,
        // we would need to implement a clipping region in Terminal.
        // For now, we just verify that intersecting children would be drawn.
    }

    #[test]
    fn add_keeps_the_child_s_owner_relative_bounds() {
        let mut group = Group::new(Rect::new(20, 30, 60, 80));
        group.add(DrawCountView::new(Rect::new(5, 10, 15, 20)));
        assert_eq!(group.children[0].bounds(), Rect::new(5, 10, 15, 20));
    }

    #[test]
    fn moving_a_group_leaves_its_children_where_they_were() {
        let mut group = Group::new(Rect::new(20, 30, 60, 80));
        group.add(DrawCountView::new(Rect::new(5, 10, 15, 20)));
        group.set_bounds(Rect::new(0, 0, 40, 50));
        assert_eq!(group.children[0].bounds(), Rect::new(5, 10, 15, 20));
    }

    /// Writes one marker character at its own local (0, 0) and records the
    /// local mouse position of every MouseDown it receives.
    struct Probe {
        core: ViewCore,
        marker: char,
        clicks: std::rc::Rc<std::cell::RefCell<Vec<Point>>>,
    }
    impl View for Probe {
        fn core(&self) -> &ViewCore {
            &self.core
        }
        fn core_mut(&mut self) -> &mut ViewCore {
            &mut self.core
        }
        fn draw(&mut self, terminal: &mut Terminal) {
            terminal.write_cell(
                0,
                0,
                crate::core::draw::Cell::new(self.marker, crate::core::palette::Attr::from_u8(7)),
            );
        }
        fn handle_event(&mut self, event: &mut Event) {
            if event.what == EventType::MouseDown {
                self.clicks.borrow_mut().push(event.mouse.pos);
                event.clear();
            }
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
    fn probe(bounds: Rect, marker: char) -> (Probe, std::rc::Rc<std::cell::RefCell<Vec<Point>>>) {
        let clicks = std::rc::Rc::new(std::cell::RefCell::new(Vec::new()));
        (
            Probe {
                core: ViewCore::new(bounds),
                marker,
                clicks: clicks.clone(),
            },
            clicks,
        )
    }

    #[test]
    fn a_group_draws_its_children_translated_by_its_own_origin_and_theirs() {
        let mut outer = Group::new(Rect::new(10, 2, 40, 12));
        let mut inner = Group::new(Rect::new(3, 1, 20, 8));
        inner.add(probe(Rect::new(2, 2, 6, 3), 'X').0);
        outer.add(inner);
        let mut t = crate::test_util::test_terminal(60, 20);
        // The owner of `outer` pushes its origin, as any group would.
        t.push_origin(outer.bounds().a);
        outer.draw(&mut t);
        t.pop_origin();
        assert_eq!(t.read_cell(15, 5).unwrap().ch, 'X');
    }

    #[test]
    fn a_group_delivers_mouse_positions_in_the_child_s_own_space() {
        let mut group = Group::new(Rect::new(10, 2, 40, 12));
        let (p, clicks) = probe(Rect::new(4, 3, 14, 6), 'X');
        group.add(p);
        let mut event = Event::mouse(EventType::MouseDown, Point::new(6, 4), MB_LEFT_BUTTON, false);
        group.handle_event(&mut event);
        assert_eq!(clicks.borrow().as_slice(), &[Point::new(2, 1)]);
        assert_eq!(event.what, EventType::Nothing);
    }

    #[test]
    fn a_group_restores_the_owner_space_position_after_dispatch() {
        struct ToCommand(ViewCore);
        impl View for ToCommand {
            fn core(&self) -> &ViewCore {
                &self.0
            }
            fn core_mut(&mut self) -> &mut ViewCore {
                &mut self.0
            }
            fn draw(&mut self, _: &mut Terminal) {}
            fn handle_event(&mut self, event: &mut Event) {
                // Like History: turn the click into a command, keep the anchor.
                event.what = EventType::Command;
                event.command = crate::core::command::CM_SHOW_HISTORY;
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
        let mut group = Group::new(Rect::new(0, 0, 40, 12));
        group.add(ToCommand(ViewCore::new(Rect::new(4, 3, 14, 6))));
        let mut event = Event::mouse(EventType::MouseDown, Point::new(6, 4), MB_LEFT_BUTTON, false);
        group.handle_event(&mut event);
        assert_eq!(event.what, EventType::Command);
        assert_eq!(event.mouse.pos, Point::new(6, 4));
    }

    #[test]
    fn test_multiple_children_clipping() {
        // Create a group at (0, 0) with size 50x50
        let mut group = Group::new(Rect::new(0, 0, 50, 50));

        // Child 1: Inside (10, 10, 20, 20) -> absolute (10, 10, 20, 20)
        group.add(DrawCountView::new(Rect::new(10, 10, 20, 20)));

        // Child 2: Completely outside (100, 100, 110, 110) -> absolute (100, 100, 110, 110)
        group.add(DrawCountView::new(Rect::new(100, 100, 110, 110)));

        // Child 3: Partially outside (40, 40, 60, 60) -> absolute (40, 40, 60, 60)
        group.add(DrawCountView::new(Rect::new(40, 40, 60, 60)));

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
        let id1 = group.add(crate::views::background::Background::new(
            Rect::new(0, 0, 10, 5),
            ' ',
            crate::core::palette::Attr::new(
                crate::core::palette::TvColor::White,
                crate::core::palette::TvColor::Blue,
            ),
        ));
        let id2 = group.add(crate::views::background::Background::new(
            Rect::new(0, 0, 10, 5),
            ' ',
            crate::core::palette::Attr::new(
                crate::core::palette::TvColor::White,
                crate::core::palette::TvColor::Blue,
            ),
        ));
        let id3 = group.add(crate::views::background::Background::new(
            Rect::new(0, 0, 10, 5),
            ' ',
            crate::core::palette::Attr::new(
                crate::core::palette::TvColor::White,
                crate::core::palette::TvColor::Blue,
            ),
        ));

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
        let id1 = group.add(crate::views::background::Background::new(
            Rect::new(0, 0, 10, 5),
            ' ',
            crate::core::palette::Attr::new(
                crate::core::palette::TvColor::White,
                crate::core::palette::TvColor::Blue,
            ),
        ));
        let id2 = group.add(crate::views::background::Background::new(
            Rect::new(0, 0, 10, 5),
            ' ',
            crate::core::palette::Attr::new(
                crate::core::palette::TvColor::White,
                crate::core::palette::TvColor::Blue,
            ),
        ));
        let id3 = group.add(crate::views::background::Background::new(
            Rect::new(0, 0, 10, 5),
            ' ',
            crate::core::palette::Attr::new(
                crate::core::palette::TvColor::White,
                crate::core::palette::TvColor::Blue,
            ),
        ));

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
