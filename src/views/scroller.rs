// (C) 2025 - Enzo Lombardi

//! Scroller view - scrollable viewport base for text viewers and editors.

use crate::core::geometry::{Point, Rect};
use crate::core::event::Event;
use crate::terminal::Terminal;
use super::view::View;
use super::scrollbar::ScrollBar;

/// Scroller is a base class for scrollable views.
/// It manages scroll offsets (delta) and content size (limit),
/// and coordinates with horizontal and vertical scrollbars.
pub struct Scroller {
    bounds: Rect,
    delta: Point,       // Current scroll offset
    limit: Point,       // Maximum scroll range (content size)
    h_scrollbar: Option<Box<ScrollBar>>,
    v_scrollbar: Option<Box<ScrollBar>>,
    owner: Option<*const dyn View>,
}

impl Scroller {
    pub fn new(bounds: Rect, h_scrollbar: Option<Box<ScrollBar>>, v_scrollbar: Option<Box<ScrollBar>>) -> Self {
        let mut scroller = Self {
            bounds,
            delta: Point::zero(),
            limit: Point::zero(),
            h_scrollbar,
            v_scrollbar,
            owner: None,
        };
        scroller.update_scrollbars();
        scroller
    }

    /// Set the scroll offset
    pub fn scroll_to(&mut self, x: i16, y: i16) {
        self.delta.x = x.max(0).min(self.limit.x);
        self.delta.y = y.max(0).min(self.limit.y);
        self.update_scrollbars();
    }

    /// Set the content size limit
    pub fn set_limit(&mut self, x: i16, y: i16) {
        self.limit.x = x.max(0);
        self.limit.y = y.max(0);

        // Adjust delta if it exceeds new limit
        self.delta.x = self.delta.x.min(self.limit.x);
        self.delta.y = self.delta.y.min(self.limit.y);

        self.update_scrollbars();
    }

    /// Get current scroll offset
    pub fn get_delta(&self) -> Point {
        self.delta
    }

    /// Get content size limit
    pub fn get_limit(&self) -> Point {
        self.limit
    }

    /// Update scrollbar positions to match current delta
    fn update_scrollbars(&mut self) {
        if let Some(ref mut h_bar) = self.h_scrollbar {
            h_bar.set_params(
                self.delta.x as i32,
                0,
                self.limit.x as i32,
                self.bounds.width() as i32,
                1,
            );
        }

        if let Some(ref mut v_bar) = self.v_scrollbar {
            v_bar.set_params(
                self.delta.y as i32,
                0,
                self.limit.y as i32,
                self.bounds.height() as i32,
                1,
            );
        }
    }

    /// Draw the scroller (draws scrollbars, subclasses override to draw content)
    pub fn draw_scrollbars(&mut self, terminal: &mut Terminal) {
        if let Some(ref mut h_bar) = self.h_scrollbar {
            h_bar.draw(terminal);
        }

        if let Some(ref mut v_bar) = self.v_scrollbar {
            v_bar.draw(terminal);
        }
    }

    /// Handle scrollbar events
    pub fn handle_scrollbar_events(&mut self, event: &mut Event) {
        let old_delta = self.delta;

        // Let scrollbars handle the event
        if let Some(ref mut h_bar) = self.h_scrollbar {
            h_bar.handle_event(event);
            self.delta.x = h_bar.get_value() as i16;
        }

        if let Some(ref mut v_bar) = self.v_scrollbar {
            v_bar.handle_event(event);
            self.delta.y = v_bar.get_value() as i16;
        }

        // If delta changed, the event was handled
        if old_delta != self.delta {
            event.clear();
        }
    }
}

impl View for Scroller {
    fn bounds(&self) -> Rect {
        self.bounds
    }

    fn set_bounds(&mut self, bounds: Rect) {
        self.bounds = bounds;

        // Update scrollbar positions (they are typically at edges)
        if let Some(ref mut h_bar) = self.h_scrollbar {
            let h_bounds = Rect::new(
                bounds.a.x,
                bounds.b.y - 1,
                bounds.b.x - 1,
                bounds.b.y,
            );
            h_bar.set_bounds(h_bounds);
        }

        if let Some(ref mut v_bar) = self.v_scrollbar {
            let v_bounds = Rect::new(
                bounds.b.x - 1,
                bounds.a.y,
                bounds.b.x,
                bounds.b.y - 1,
            );
            v_bar.set_bounds(v_bounds);
        }

        self.update_scrollbars();
    }

    fn draw(&mut self, terminal: &mut Terminal) {
        // Default implementation: draw scrollbars only
        // Subclasses should override this to draw content + scrollbars
        self.draw_scrollbars(terminal);
    }

    fn handle_event(&mut self, event: &mut Event) {
        self.handle_scrollbar_events(event);
    }

    fn set_owner(&mut self, owner: *const dyn View) {
        self.owner = Some(owner);
    }

    fn get_owner(&self) -> Option<*const dyn View> {
        self.owner
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scroller_scroll_to() {
        let scroller = Scroller::new(Rect::new(0, 0, 80, 25), None, None);
        let mut scroller = scroller;
        scroller.set_limit(100, 100);

        scroller.scroll_to(10, 20);
        assert_eq!(scroller.get_delta(), Point::new(10, 20));

        // Test clamping to limit
        scroller.scroll_to(150, 150);
        assert_eq!(scroller.get_delta(), Point::new(100, 100));

        // Test clamping to zero
        scroller.scroll_to(-10, -10);
        assert_eq!(scroller.get_delta(), Point::new(0, 0));
    }

    #[test]
    fn test_scroller_set_limit() {
        let scroller = Scroller::new(Rect::new(0, 0, 80, 25), None, None);
        let mut scroller = scroller;

        // First set a large limit
        scroller.set_limit(100, 100);
        scroller.scroll_to(50, 50);
        assert_eq!(scroller.get_delta(), Point::new(50, 50));

        // Reducing limit should clamp delta
        scroller.set_limit(30, 30);
        assert_eq!(scroller.get_delta(), Point::new(30, 30));
        assert_eq!(scroller.get_limit(), Point::new(30, 30));
    }
}
