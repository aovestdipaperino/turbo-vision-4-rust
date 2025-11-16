// (C) 2025 - Enzo Lombardi

//! Geometric primitives - Point and Rect types for positioning and sizing views.

use std::fmt;

/// A point in 2D space
///
/// # Examples
///
/// ```
/// use turbo_vision::core::geometry::Point;
///
/// let p = Point::new(10, 20);
/// assert_eq!(p.x, 10);
/// assert_eq!(p.y, 20);
///
/// let origin = Point::zero();
/// assert_eq!(origin, Point::new(0, 0));
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Point {
    pub x: i16,
    pub y: i16,
}

impl Point {
    pub const fn new(x: i16, y: i16) -> Self {
        Self { x, y }
    }

    pub const fn zero() -> Self {
        Self { x: 0, y: 0 }
    }
}

impl fmt::Display for Point {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "({}, {})", self.x, self.y)
    }
}

/// A rectangle defined by two points (top-left inclusive, bottom-right exclusive)
///
/// # Examples
///
/// ```
/// use turbo_vision::core::geometry::{Point, Rect};
///
/// // Create a 10x10 rectangle at origin
/// let rect = Rect::new(0, 0, 10, 10);
/// assert_eq!(rect.width(), 10);
/// assert_eq!(rect.height(), 10);
///
/// // Check if a point is inside
/// assert!(rect.contains(Point::new(5, 5)));
/// assert!(!rect.contains(Point::new(10, 10))); // Bottom-right is exclusive
///
/// // Move and resize
/// let mut r = Rect::new(0, 0, 10, 10);
/// r.move_by(5, 5);
/// assert_eq!(r, Rect::new(5, 5, 15, 15));
///
/// r.grow(2, 2);  // Expand by 2 in all directions
/// assert_eq!(r, Rect::new(3, 3, 17, 17));
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Rect {
    pub a: Point,  // top-left (inclusive)
    pub b: Point,  // bottom-right (exclusive)
}

impl Rect {
    pub const fn new(x1: i16, y1: i16, x2: i16, y2: i16) -> Self {
        Self {
            a: Point::new(x1, y1),
            b: Point::new(x2, y2),
        }
    }

    pub const fn from_points(a: Point, b: Point) -> Self {
        Self { a, b }
    }

    pub const fn from_coords(x: i16, y: i16, width: i16, height: i16) -> Self {
        Self {
            a: Point::new(x, y),
            b: Point::new(x + width, y + height),
        }
    }

    /// Move the rectangle by the given delta
    pub fn move_by(&mut self, dx: i16, dy: i16) {
        self.a.x += dx;
        self.a.y += dy;
        self.b.x += dx;
        self.b.y += dy;
    }

    /// Grow (or shrink if negative) the rectangle by the given amount
    pub fn grow(&mut self, dx: i16, dy: i16) {
        self.a.x -= dx;
        self.a.y -= dy;
        self.b.x += dx;
        self.b.y += dy;
    }

    /// Check if a point is inside the rectangle
    /// For zero-width or zero-height rectangles (single row/column controls),
    /// the point must match the exact coordinate
    pub fn contains(&self, p: Point) -> bool {
        let in_x = if self.b.x > self.a.x {
            p.x >= self.a.x && p.x < self.b.x
        } else {
            p.x == self.a.x
        };

        let in_y = if self.b.y > self.a.y {
            p.y >= self.a.y && p.y < self.b.y
        } else {
            p.y == self.a.y
        };

        in_x && in_y
    }

    /// Check if the rectangle is empty (has zero or negative area)
    pub fn is_empty(&self) -> bool {
        self.b.x <= self.a.x || self.b.y <= self.a.y
    }

    /// Get the width of the rectangle
    pub fn width(&self) -> i16 {
        self.b.x - self.a.x
    }

    /// Get the height of the rectangle
    pub fn height(&self) -> i16 {
        self.b.y - self.a.y
    }

    /// Get the width as a non-negative value, clamped to 0 if negative
    /// Use this when converting to usize to avoid panics from negative dimensions
    pub fn width_clamped(&self) -> i16 {
        self.width().max(0)
    }

    /// Get the height as a non-negative value, clamped to 0 if negative
    /// Use this when converting to usize to avoid panics from negative dimensions
    pub fn height_clamped(&self) -> i16 {
        self.height().max(0)
    }

    /// Get the size as a Point
    pub fn size(&self) -> Point {
        Point::new(self.width(), self.height())
    }

    /// Intersect this rectangle with another
    ///
    /// # Examples
    ///
    /// ```
    /// use turbo_vision::core::geometry::Rect;
    ///
    /// let r1 = Rect::new(0, 0, 10, 10);
    /// let r2 = Rect::new(5, 5, 15, 15);
    /// let intersection = r1.intersect(&r2);
    /// assert_eq!(intersection, Rect::new(5, 5, 10, 10));
    /// ```
    pub fn intersect(&self, other: &Rect) -> Rect {
        Rect {
            a: Point::new(self.a.x.max(other.a.x), self.a.y.max(other.a.y)),
            b: Point::new(self.b.x.min(other.b.x), self.b.y.min(other.b.y)),
        }
    }

    /// Check if this rectangle intersects (overlaps) with another
    pub fn intersects(&self, other: &Rect) -> bool {
        // Two rectangles intersect if they overlap on both axes
        !(self.b.x <= other.a.x || self.a.x >= other.b.x ||
          self.b.y <= other.a.y || self.a.y >= other.b.y)
    }

    /// Calculate the union of this rectangle with another
    /// Returns the smallest rectangle that contains both rectangles
    /// Matches Borland: Used in TView::locate() to calculate redraw region
    pub fn union(&self, other: &Rect) -> Rect {
        Rect {
            a: Point::new(self.a.x.min(other.a.x), self.a.y.min(other.a.y)),
            b: Point::new(self.b.x.max(other.b.x), self.b.y.max(other.b.y)),
        }
    }
}

impl Default for Rect {
    fn default() -> Self {
        Self::new(0, 0, 0, 0)
    }
}

impl fmt::Display for Rect {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[{}, {}, {}, {}]", self.a.x, self.a.y, self.b.x, self.b.y)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_point() {
        let p = Point::new(10, 20);
        assert_eq!(p.x, 10);
        assert_eq!(p.y, 20);
    }

    #[test]
    fn test_rect_basic() {
        let r = Rect::new(1, 2, 11, 12);
        assert_eq!(r.width(), 10);
        assert_eq!(r.height(), 10);
        assert!(!r.is_empty());
    }

    #[test]
    fn test_rect_contains() {
        let r = Rect::new(0, 0, 10, 10);
        assert!(r.contains(Point::new(5, 5)));
        assert!(r.contains(Point::new(0, 0)));
        assert!(!r.contains(Point::new(10, 10)));
        assert!(!r.contains(Point::new(-1, 5)));
    }

    #[test]
    fn test_rect_contains_zero_dimensions() {
        // Zero-height rectangle (single row control like Label or InputLine)
        let single_row = Rect::new(2, 4, 15, 4);
        assert!(single_row.contains(Point::new(2, 4)));  // Left edge
        assert!(single_row.contains(Point::new(10, 4))); // Middle
        assert!(single_row.contains(Point::new(14, 4))); // Right edge-1
        assert!(!single_row.contains(Point::new(15, 4))); // Right edge (excluded)
        assert!(!single_row.contains(Point::new(5, 3))); // Row above
        assert!(!single_row.contains(Point::new(5, 5))); // Row below

        // Zero-width rectangle (single column)
        let single_col = Rect::new(5, 2, 5, 10);
        assert!(single_col.contains(Point::new(5, 2)));  // Top
        assert!(single_col.contains(Point::new(5, 5)));  // Middle
        assert!(single_col.contains(Point::new(5, 9)));  // Bottom-1
        assert!(!single_col.contains(Point::new(5, 10))); // Bottom (excluded)
        assert!(!single_col.contains(Point::new(4, 5))); // Column left
        assert!(!single_col.contains(Point::new(6, 5))); // Column right

        // Zero-width and zero-height (single point)
        let single_point = Rect::new(10, 10, 10, 10);
        assert!(single_point.contains(Point::new(10, 10)));
        assert!(!single_point.contains(Point::new(10, 11)));
        assert!(!single_point.contains(Point::new(11, 10)));
        assert!(!single_point.contains(Point::new(9, 10)));
        assert!(!single_point.contains(Point::new(10, 9)));
    }

    #[test]
    fn test_rect_move() {
        let mut r = Rect::new(0, 0, 10, 10);
        r.move_by(5, 5);
        assert_eq!(r, Rect::new(5, 5, 15, 15));
    }

    #[test]
    fn test_rect_grow() {
        let mut r = Rect::new(5, 5, 15, 15);
        r.grow(2, 2);
        assert_eq!(r, Rect::new(3, 3, 17, 17));
    }

    #[test]
    fn test_rect_intersect() {
        let r1 = Rect::new(0, 0, 10, 10);
        let r2 = Rect::new(5, 5, 15, 15);
        let intersection = r1.intersect(&r2);
        assert_eq!(intersection, Rect::new(5, 5, 10, 10));
    }

    #[test]
    fn test_rect_intersects() {
        let r1 = Rect::new(0, 0, 10, 10);

        // Overlapping rectangle
        let r2 = Rect::new(5, 5, 15, 15);
        assert!(r1.intersects(&r2));
        assert!(r2.intersects(&r1)); // Symmetric

        // Completely inside
        let r3 = Rect::new(2, 2, 8, 8);
        assert!(r1.intersects(&r3));

        // Adjacent but not overlapping (touching edges)
        let r4 = Rect::new(10, 0, 20, 10);
        assert!(!r1.intersects(&r4)); // b.x exclusive

        // Completely outside
        let r5 = Rect::new(20, 20, 30, 30);
        assert!(!r1.intersects(&r5));

        // Overlapping on one corner
        let r6 = Rect::new(8, 8, 15, 15);
        assert!(r1.intersects(&r6));
    }

    #[test]
    fn test_point_display() {
        let p = Point::new(10, 20);
        assert_eq!(format!("{}", p), "(10, 20)");

        let p2 = Point::new(-5, 0);
        assert_eq!(format!("{}", p2), "(-5, 0)");
    }

    #[test]
    fn test_rect_display() {
        let r = Rect::new(1, 2, 11, 12);
        assert_eq!(format!("{}", r), "[1, 2, 11, 12]");

        let r2 = Rect::new(0, 0, 80, 25);
        assert_eq!(format!("{}", r2), "[0, 0, 80, 25]");
    }

    #[test]
    fn test_width_height_clamped() {
        // Normal rectangle
        let r = Rect::new(0, 0, 10, 10);
        assert_eq!(r.width_clamped(), 10);
        assert_eq!(r.height_clamped(), 10);

        // Inverted rectangle (negative dimensions)
        let r2 = Rect::new(10, 10, 5, 5);
        assert_eq!(r2.width(), -5);  // Raw width is negative
        assert_eq!(r2.height(), -5); // Raw height is negative
        assert_eq!(r2.width_clamped(), 0);  // Clamped to 0
        assert_eq!(r2.height_clamped(), 0); // Clamped to 0

        // Partially inverted
        let r3 = Rect::new(5, 5, 2, 10);
        assert_eq!(r3.width(), -3);
        assert_eq!(r3.height(), 5);
        assert_eq!(r3.width_clamped(), 0);
        assert_eq!(r3.height_clamped(), 5);

        // Safe to convert to usize
        let width_usize = r2.width_clamped() as usize;
        let height_usize = r2.height_clamped() as usize;
        assert_eq!(width_usize, 0);
        assert_eq!(height_usize, 0);
    }
}
