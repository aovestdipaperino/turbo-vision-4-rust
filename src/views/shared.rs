// (C) 2026 - Enzo Lombardi

//! `Shared<T>` lets one view be both a child owned by a `Group` and a handle
//! held by its parent struct. Borland does this with a raw `TEditor*` into
//! the owner's child list; Rust needs `Rc<RefCell<T>>`. This is the single
//! forwarding wrapper that replaces the per-type `SharedScrollBar`,
//! `SharedEditor`, `SharedIndicator`, `SharedHelpViewer` and
//! `SharedTerminalWidget` newtypes.

use super::view::{View, ViewId};
use crate::core::command::CommandId;
use crate::core::event::Event;
use crate::core::geometry::Rect;
use crate::core::palette::Palette;
use crate::core::palette_chain::PaletteChainNode;
use crate::core::state::{GrowFlags, StateFlags};
use crate::terminal::Terminal;
use std::cell::RefCell;
use std::rc::Rc;

/// A `View` that forwards everything to an `Rc<RefCell<T>>` so the same view
/// can be inserted into a group and still be reached by its creator.
pub struct Shared<T: View> {
    inner: Rc<RefCell<T>>,
    /// Mirror of the inner view's chain so `get_palette_chain` can hand out
    /// a reference (a `RefCell` borrow cannot escape).
    palette_chain: Option<PaletteChainNode>,
}

impl<T: View> Shared<T> {
    pub fn new(inner: Rc<RefCell<T>>) -> Self {
        Self {
            inner,
            palette_chain: None,
        }
    }

    pub fn inner(&self) -> &Rc<RefCell<T>> {
        &self.inner
    }
}

impl<T: View> View for Shared<T> {
    fn bounds(&self) -> Rect {
        self.inner.borrow().bounds()
    }
    fn set_bounds(&mut self, bounds: Rect) {
        self.inner.borrow_mut().set_bounds(bounds);
    }
    fn draw(&mut self, terminal: &mut Terminal) {
        self.inner.borrow_mut().draw(terminal);
    }
    fn handle_event(&mut self, event: &mut Event) {
        self.inner.borrow_mut().handle_event(event);
    }
    fn can_focus(&self) -> bool {
        self.inner.borrow().can_focus()
    }
    fn set_focus(&mut self, focused: bool) {
        self.inner.borrow_mut().set_focus(focused);
    }
    fn is_focused(&self) -> bool {
        self.inner.borrow().is_focused()
    }
    fn window_number(&self) -> Option<u8> {
        self.inner.borrow().window_number()
    }
    fn options(&self) -> u16 {
        self.inner.borrow().options()
    }
    fn set_options(&mut self, options: u16) {
        self.inner.borrow_mut().set_options(options);
    }
    fn state(&self) -> StateFlags {
        self.inner.borrow().state()
    }
    fn set_state(&mut self, state: StateFlags) {
        self.inner.borrow_mut().set_state(state);
    }
    fn grow_mode(&self) -> GrowFlags {
        self.inner.borrow().grow_mode()
    }
    fn set_grow_mode(&mut self, grow_mode: GrowFlags) {
        self.inner.borrow_mut().set_grow_mode(grow_mode);
    }
    fn update_cursor(&self, terminal: &mut Terminal) {
        self.inner.borrow().update_cursor(terminal);
    }
    fn zoom(&mut self, max_bounds: Rect) {
        self.inner.borrow_mut().zoom(max_bounds);
    }
    fn valid(&mut self, command: CommandId) -> bool {
        self.inner.borrow_mut().valid(command)
    }
    fn is_default_button(&self) -> bool {
        self.inner.borrow().is_default_button()
    }
    fn button_command(&self) -> Option<u16> {
        self.inner.borrow().button_command()
    }
    fn set_list_selection(&mut self, index: usize) {
        self.inner.borrow_mut().set_list_selection(index);
    }
    fn get_list_selection(&self) -> usize {
        self.inner.borrow().get_list_selection()
    }
    fn get_end_state(&self) -> CommandId {
        self.inner.borrow().get_end_state()
    }
    fn set_end_state(&mut self, command: CommandId) {
        self.inner.borrow_mut().set_end_state(command);
    }
    fn label_link(&self) -> Option<ViewId> {
        self.inner.borrow().label_link()
    }
    fn init_after_add(&mut self) {
        self.inner.borrow_mut().init_after_add();
    }
    fn constrain_to_parent_bounds(&mut self) {
        self.inner.borrow_mut().constrain_to_parent_bounds();
    }
    fn set_parent_bounds(&mut self, bounds: Rect) {
        self.inner.borrow_mut().set_parent_bounds(bounds);
    }
    fn get_palette(&self) -> Option<Palette> {
        self.inner.borrow().get_palette()
    }

    fn set_palette_chain(&mut self, node: Option<PaletteChainNode>) {
        self.palette_chain = node.clone();
        self.inner.borrow_mut().set_palette_chain(node);
    }

    fn get_palette_chain(&self) -> Option<&PaletteChainNode> {
        self.palette_chain.as_ref()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::state::{SF_FOCUSED, SF_VISIBLE};
    use crate::views::button::Button;
    use crate::views::scrollbar::ScrollBar;

    #[test]
    fn shared_forwards_state_and_bounds_both_ways() {
        let inner = Rc::new(RefCell::new(Button::new(
            Rect::new(0, 0, 10, 2),
            "ok",
            1,
            false,
        )));
        let mut shared = Shared::new(Rc::clone(&inner));

        shared.set_state(SF_VISIBLE | SF_FOCUSED);
        assert_eq!(inner.borrow().state(), SF_VISIBLE | SF_FOCUSED);

        inner.borrow_mut().set_bounds(Rect::new(5, 5, 6, 15));
        assert_eq!(shared.bounds(), Rect::new(5, 5, 6, 15));
    }

    #[test]
    fn shared_keeps_an_observable_palette_chain() {
        let inner = Rc::new(RefCell::new(ScrollBar::new_vertical(Rect::new(
            0, 0, 1, 10,
        ))));
        let mut shared = Shared::new(Rc::clone(&inner));
        assert!(shared.get_palette_chain().is_none());

        shared.set_palette_chain(Some(PaletteChainNode::new(None, None)));
        assert!(shared.get_palette_chain().is_some());
        assert!(inner.borrow().get_palette_chain().is_some());
    }
}
