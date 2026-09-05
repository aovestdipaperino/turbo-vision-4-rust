// (C) 2026 - Enzo Lombardi

//! VirtualListBox - list view over a lazy item provider.

use turbo_vision::core::command::CommandId;
use turbo_vision::core::draw::DrawBuffer;
use turbo_vision::core::event::Event;
use turbo_vision::core::geometry::Rect;
use turbo_vision::core::palette::{Attr, TvColor};
use turbo_vision::terminal::Terminal;
use turbo_vision::views::list_viewer::{ListViewer, ListViewerState};
use turbo_vision::views::view::write_line_to_terminal;
use turbo_vision::views::{View, ViewCore};

/// Lazy item source for [`VirtualListBox`].
///
/// Only the visible rows are materialized, so lists with millions of items
/// stay cheap (TV Tool Box called these "virtual list boxes").
pub trait ListProvider {
    /// Total number of items.
    fn len(&self) -> usize;

    /// Text for the item at `index` (`index < len()`).
    fn item(&self, index: usize) -> String;

    /// True when the list has no items.
    fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

impl ListProvider for Vec<String> {
    fn len(&self) -> usize {
        self.as_slice().len()
    }

    fn item(&self, index: usize) -> String {
        self[index].clone()
    }
}

/// List box that pulls items on demand from a [`ListProvider`].
///
/// Navigation matches the framework's other lists (arrows, PgUp/PgDn,
/// Home/End, mouse selection); Enter or double-click converts the event
/// into the constructor's command with the selected index in `event.info`
/// (clamped to `u16`).
///
/// # Examples
///
/// ```
/// use turbo_vision::core::geometry::Rect;
/// use turbo_vision_extras::{ListProvider, VirtualListBox};
///
/// struct Numbers;
/// impl ListProvider for Numbers {
///     fn len(&self) -> usize { 1_000_000 }
///     fn item(&self, i: usize) -> String { format!("Item {i}") }
/// }
///
/// let list = VirtualListBox::new(Rect::new(0, 0, 30, 10), Box::new(Numbers), 800);
/// assert_eq!(list.item_count(), 1_000_000);
/// ```
pub struct VirtualListBox {
    core: ViewCore,
    provider: Box<dyn ListProvider>,
    list_state: ListViewerState,
    on_select: CommandId,
    palette_chain: Option<turbo_vision::core::palette_chain::PaletteChainNode>,
}

impl std::fmt::Debug for VirtualListBox {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("VirtualListBox")
            .field("bounds", &self.core.bounds)
            .field("items", &self.provider.len())
            .finish()
    }
}

impl VirtualListBox {
    /// Create a list over `provider`; Enter/double-click emits `on_select`.
    pub fn new(bounds: Rect, provider: Box<dyn ListProvider>, on_select: CommandId) -> Self {
        let mut list_state = ListViewerState::new();
        list_state.set_range(provider.len());
        Self {
            core: ViewCore {
                bounds,
                state: 0,
                ..ViewCore::default()
            },
            provider,
            list_state,
            on_select,
            palette_chain: None,
        }
    }

    /// Replace the provider (e.g. after a query re-run).
    pub fn set_provider(&mut self, provider: Box<dyn ListProvider>) {
        self.list_state = ListViewerState::new();
        self.list_state.set_range(provider.len());
        self.provider = provider;
    }

    /// Total item count.
    pub fn item_count(&self) -> usize {
        self.provider.len()
    }

    /// Currently focused item index.
    pub fn selection(&self) -> Option<usize> {
        self.list_state.focused
    }

    /// Focus an item by index.
    pub fn set_selection(&mut self, index: usize) {
        if index < self.provider.len() {
            let visible = self.core.bounds.height_clamped() as usize;
            self.list_state.focus_item(index, visible);
        }
    }
}

impl View for VirtualListBox {
    fn core(&self) -> &ViewCore {
        &self.core
    }

    fn core_mut(&mut self) -> &mut ViewCore {
        &mut self.core
    }

    fn draw(&mut self, terminal: &mut Terminal) {
        let width = self.core.bounds.width_clamped() as usize;
        let height = self.core.bounds.height_clamped() as usize;

        let normal = Attr::new(TvColor::Black, TvColor::Cyan);
        let selected = if self.is_focused() {
            Attr::new(TvColor::White, TvColor::Green)
        } else {
            Attr::new(TvColor::Black, TvColor::LightGray)
        };

        for row in 0..height {
            let mut buf = DrawBuffer::new(width);
            let idx = self.list_state.top_item + row;
            let attr = if Some(idx) == self.list_state.focused {
                selected
            } else {
                normal
            };
            buf.move_char(0, ' ', attr, width);
            if idx < self.provider.len() {
                // Only visible rows are materialized
                let text: String = self.provider.item(idx).chars().take(width).collect();
                buf.move_str(0, &text, attr);
            }
            write_line_to_terminal(
                terminal,
                self.core.bounds.a.x,
                self.core.bounds.a.y + row as i16,
                &buf,
            );
        }
    }

    fn handle_event(&mut self, event: &mut Event) {
        // Enter (or double-click via the list trait) selects the focused item
        if event.what == turbo_vision::core::event::EventType::Keyboard
            && event.key_code == turbo_vision::core::event::KB_ENTER
            && self.is_focused()
        {
            if let Some(index) = self.list_state.focused {
                *event =
                    Event::broadcast_with_info(self.on_select, index.min(u16::MAX as usize) as u16);
                return;
            }
        }
        self.handle_list_event(event);
    }

    fn can_focus(&self) -> bool {
        true
    }

    fn get_palette(&self) -> Option<turbo_vision::core::palette::Palette> {
        None
    }

    fn set_palette_chain(
        &mut self,
        node: Option<turbo_vision::core::palette_chain::PaletteChainNode>,
    ) {
        self.palette_chain = node;
    }

    fn get_palette_chain(&self) -> Option<&turbo_vision::core::palette_chain::PaletteChainNode> {
        self.palette_chain.as_ref()
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}

impl ListViewer for VirtualListBox {
    fn list_state(&self) -> &ListViewerState {
        &self.list_state
    }

    fn list_state_mut(&mut self) -> &mut ListViewerState {
        &mut self.list_state
    }

    fn get_text(&self, item: usize, _max_len: usize) -> String {
        if item < self.provider.len() {
            self.provider.item(item)
        } else {
            String::new()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use turbo_vision::core::event::{EventType, KB_DOWN, KB_END};

    struct Huge;
    impl ListProvider for Huge {
        fn len(&self) -> usize {
            1_000_000
        }
        fn item(&self, i: usize) -> String {
            format!("row {i}")
        }
    }

    #[test]
    fn navigates_huge_provider_lazily() {
        let mut list = VirtualListBox::new(Rect::new(0, 0, 20, 10), Box::new(Huge), 700);
        list.set_focus(true);
        assert_eq!(list.item_count(), 1_000_000);

        let mut ev = Event::keyboard(KB_DOWN);
        list.handle_event(&mut ev);
        assert_eq!(list.selection(), Some(1));
        assert_eq!(ev.what, EventType::Nothing);

        let mut ev = Event::keyboard(KB_END);
        list.handle_event(&mut ev);
        assert_eq!(list.selection(), Some(999_999));
    }

    #[test]
    fn vec_provider_works() {
        let items = vec!["a".to_string(), "b".to_string()];
        let list = VirtualListBox::new(Rect::new(0, 0, 20, 5), Box::new(items), 700);
        assert_eq!(list.item_count(), 2);
    }

    #[test]
    fn enter_broadcasts_selection() {
        use turbo_vision::core::event::KB_ENTER;
        let mut list = VirtualListBox::new(Rect::new(0, 0, 20, 10), Box::new(Huge), 700);
        list.set_focus(true);
        list.set_selection(42);
        let mut ev = Event::keyboard(KB_ENTER);
        list.handle_event(&mut ev);
        assert_eq!(ev.what, EventType::Broadcast);
        assert_eq!(ev.command, 700);
        assert_eq!(ev.info, 42);
    }
}
