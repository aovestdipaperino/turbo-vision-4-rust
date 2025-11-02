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
        self.children.handle_event(event);
    }
}
