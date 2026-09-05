// (C) 2025 - Enzo Lombardi

//! Frame view - window border with title, close button and zoom triangle.

use super::view::{View, ViewCore, write_line_to_terminal};
use crate::core::command::CM_CLOSE;
use crate::core::draw::DrawBuffer;
use crate::core::event::{Event, EventType, MB_LEFT_BUTTON};
use crate::core::geometry::Rect;
use crate::core::palette::Attr;
use crate::core::state::State;
use crate::terminal::Terminal;
use unicode_width::UnicodeWidthStr;

pub struct Frame {
    core: ViewCore,
    title: String,
    /// Window number shown right of the title (Borland: TFrame draws
    /// TWindow::number for Alt+1..9 selection); None = not numbered
    number: Option<u8>,
    /// Palette type — retained for API compatibility.
    #[allow(dead_code)]
    palette_type: FramePaletteType,
    /// Whether the frame is resizable (matches Borland's wfGrow flag)
    resizable: bool,
    /// True while a MouseDown that started on the close icon is outstanding.
    /// CM_CLOSE only fires when the matching MouseUp is also over the icon
    /// (matches Borland: TFrame tracks press-release on the close icon).
    close_pressed: bool,
    /// Whether the zoom icon is drawn at all (Borland: wfZoom). Defaults to
    /// `resizable`, since a fixed-size window has nothing to zoom to.
    zoomable: bool,
    /// The extent a zoom would fill, normally the desktop. The triangle is
    /// derived from this at draw time rather than remembered, so a window
    /// resized by anything other than a zoom (Tile, Cascade, a drag of the
    /// resize corner) still shows the right glyph.
    max_bounds: Option<Rect>,
    /// Fallback for a frame that was never told its maximum extent. Only
    /// [`Frame::set_zoomed`] writes it.
    zoomed_override: bool,
    /// The zoom icon's equivalent of `close_pressed`.
    zoom_pressed: bool,
}

/// Frame palette types for different window types
/// Matches Borland's palette hierarchy (cpDialog, cpBlueWindow, etc.)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FramePaletteType {
    Dialog,       // Uses cpDialog palette (LightGreen close button)
    EditorWindow, // Uses cpBlueWindow palette (blue window)
    HelpWindow,   // Uses cpCyanWindow palette (cyan help window)
}

impl Frame {
    pub fn new(bounds: Rect, title: &str, resizable: bool) -> Self {
        Self::with_palette(bounds, title, FramePaletteType::Dialog, resizable)
    }

    pub fn with_palette(
        bounds: Rect,
        title: &str,
        palette_type: FramePaletteType,
        resizable: bool,
    ) -> Self {
        Self {
            core: ViewCore {
                bounds,
                state: State::ACTIVE,
                palette_chain: None,
                ..ViewCore::default()
            },
            title: title.to_string(),
            number: None,
            palette_type,
            resizable,
            close_pressed: false,
            // Borland pairs wfGrow with wfZoom: a window that cannot be
            // resized has nothing to zoom to, and a dialog has neither.
            zoomable: resizable,
            max_bounds: None,
            zoomed_override: false,
            zoom_pressed: false,
        }
    }

    /// True if the given position is over the close icon `[■]` on the top
    /// frame row (columns 2..=4 relative to the frame's left edge).
    fn is_on_close_icon(&self, pos: crate::core::geometry::Point) -> bool {
        pos.y == 0 && pos.x >= 2 && pos.x <= 4
    }

    /// Leftmost column of the zoom icon `[\u{25B2}]`, three cells wide, sitting
    /// just inside the top-right corner.
    ///
    /// `None` when the frame is not zoomable or is too narrow to hold both
    /// icons and a title. Matches Borland TFrame::draw, which places the zoom
    /// icon at `width - 5`.
    fn zoom_icon_x(&self) -> Option<i16> {
        let width = self.core.bounds.width();
        if !self.zoomable || width <= 10 {
            return None;
        }
        Some(width - 5)
    }

    /// True if the given position is over the zoom icon on the top frame row.
    fn is_on_zoom_icon(&self, pos: crate::core::geometry::Point) -> bool {
        let Some(x) = self.zoom_icon_x() else {
            return false;
        };
        pos.y == 0 && pos.x >= x && pos.x <= x + 2
    }

    /// Set whether the frame is resizable (matches Borland's wfGrow flag).
    pub fn set_resizable(&mut self, resizable: bool) {
        self.resizable = resizable;
    }

    /// Set whether the zoom icon is drawn (matches Borland's wfZoom flag).
    pub fn set_zoomable(&mut self, zoomable: bool) {
        self.zoomable = zoomable;
    }

    /// Tell the frame the extent a zoom would fill, normally the desktop rect.
    ///
    /// The zoom triangle is derived from this every time the frame draws, so it
    /// stays right through Tile, Cascade and a drag of the resize corner, none
    /// of which go through the zoom command.
    pub fn set_max_bounds(&mut self, max_bounds: Rect) {
        self.max_bounds = Some(max_bounds);
    }

    /// Tell the frame whether its window is zoomed.
    ///
    /// Only consulted when the frame has not been given its maximum extent with
    /// [`Frame::set_max_bounds`], which a window inside a desktop always is.
    #[deprecated(
        since = "2.4.1",
        note = "the zoom state is derived from the bounds; use set_max_bounds"
    )]
    pub fn set_zoomed(&mut self, zoomed: bool) {
        self.zoomed_override = zoomed;
    }

    /// Whether the window fills the extent a zoom would take it to, which is
    /// what decides between the "grow" and "restore" triangles.
    ///
    /// Compares sizes rather than positions, matching `Window::zoom`, so the
    /// glyph always agrees with what pressing it will do.
    pub fn is_zoomed(&self) -> bool {
        match self.max_bounds {
            Some(max) => {
                self.bounds().width() == max.width() && self.bounds().height() == max.height()
            }
            None => self.zoomed_override,
        }
    }

    /// Set the frame title
    /// Matches Borland: TFrame::setTitle() allows changing window title dynamically
    pub fn set_title(&mut self, title: &str) {
        self.title = title.to_string();
    }

    /// Set the window number displayed in the frame (Borland: TWindow::number).
    pub fn set_number(&mut self, number: Option<u8>) {
        self.number = number;
    }

    /// Get colors for frame elements based on palette type and state
    /// Matches Borland's getColor() with palette mapping (tframe.cc:43-64)
    /// Returns (frame_attr, close_icon_attr, title_attr)
    fn get_frame_colors(&self) -> (Attr, Attr, Attr) {
        use crate::core::palette::{FRAME_ACTIVE_BORDER, FRAME_ICON, FRAME_INACTIVE, FRAME_TITLE};

        // Borland determines cFrame based on state:
        // - Inactive: cFrame = 0x0101 (both bytes use palette[1])
        // - Dragging: cFrame = 0x0505 (both bytes use palette[5])
        // - Active:   cFrame = 0x0503 (low=palette[3], high=palette[5])

        let is_active = self.core.state.contains(State::ACTIVE);
        let is_dragging = self.core.state.contains(State::DRAGGING);

        if !is_active {
            // Inactive: cFrame = 0x0101, cTitle = 0x0002
            // Uses palette[1] for all elements
            let inactive_attr = self.map_color(FRAME_INACTIVE);
            (inactive_attr, inactive_attr, inactive_attr)
        } else if is_dragging {
            // Dragging: cFrame = 0x0505, cTitle = 0x0005
            // Uses palette[5] for all elements
            let dragging_attr = self.map_color(FRAME_ICON);
            (dragging_attr, dragging_attr, dragging_attr)
        } else {
            // Active: cFrame = 0x0503, cTitle = 0x0004
            // palette[3] = frame border
            // palette[5] = close icon (highlight)
            // palette[4] = title
            let frame_attr = self.map_color(FRAME_ACTIVE_BORDER);
            let close_icon_attr = self.map_color(FRAME_ICON);
            let title_attr = self.map_color(FRAME_TITLE);
            (frame_attr, close_icon_attr, title_attr)
        }
    }
}

impl View for Frame {
    fn core(&self) -> &ViewCore {
        &self.core
    }

    fn core_mut(&mut self) -> &mut ViewCore {
        &mut self.core
    }

    fn draw(&mut self, terminal: &mut Terminal) {
        let width = self.core.bounds.width_clamped() as usize;
        let height = self.core.bounds.height_clamped() as usize;

        // Don't render frames that are too small
        // Minimum: 2x2 (for top-left, top-right, bottom-left, bottom-right corners)
        if width < 2 || height < 2 {
            return;
        }

        // Get frame colors from palette mapping (matches Borland's getColor())
        let (frame_attr, close_icon_attr, title_attr) = self.get_frame_colors();

        // Top border with title - using double-line box drawing
        let mut buf = DrawBuffer::new(width);
        buf.put_char(0, '╔', frame_attr); // Double top-left corner
        buf.put_char(width - 1, '╗', frame_attr); // Double top-right corner
        for i in 1..width - 1 {
            buf.put_char(i, '═', frame_attr); // Double horizontal line
        }

        // Add close button at position 2: [■]
        // Matches Borland: closeIcon = "[~\xFE~]" where ~ toggles between cFrame low/high bytes
        // For active dialog: cFrame = 0x0503
        //   - '[' and ']' use low byte (03) -> cpDialog[3] -> frame_attr (White on LightGray)
        //   - '■' uses high byte (05) -> cpDialog[5] -> close_icon_attr (LightGreen on LightGray)
        // See local-only/about.png and tframe.cc:123 (b.moveCStr(2, closeIcon, cFrame))
        if width > 5 {
            buf.put_char(2, '[', frame_attr);
            buf.put_char(3, '■', close_icon_attr); // Uses palette highlight color
            buf.put_char(4, ']', frame_attr);
        }

        // Zoom icon just inside the top-right corner, mirroring the close
        // icon's bracket-and-glyph shape. The triangle points up while the
        // window can still grow, and down once it is zoomed and the click will
        // restore it. Matches Borland: zoomIcon at width - 5.
        if let Some(x) = self.zoom_icon_x() {
            // Owner-relative: the icon's x is already in the frame's space.
            let at = x as usize;
            let glyph = if self.is_zoomed() {
                '\u{25BC}'
            } else {
                '\u{25B2}'
            };
            buf.put_char(at, '[', frame_attr);
            buf.put_char(at + 1, glyph, close_icon_attr);
            buf.put_char(at + 2, ']', frame_attr);
        }

        // Add title after close button
        // Centered title, clamped clear of the close icon (left) and the
        // zoom/number area (right) — matches Borland TFrame::draw:
        // i = (width - l) >> 1 with l capped at width - 10
        let title_display_width = self.title.width().min(width.saturating_sub(10));
        if !self.title.is_empty() && width > title_display_width + 8 {
            let text = format!(" {} ", self.title);
            let start = ((width - title_display_width) / 2).max(6);
            buf.move_str(start, &text, title_attr);

            // Window number right of the title (Borland: TFrame::draw shows
            // TWindow::number when 1..=9 so Alt+digit selection is visible)
            if let Some(number) = self.number {
                if (1..=9).contains(&number) && width > title_display_width + 12 {
                    buf.move_str(
                        start + title_display_width + 2,
                        &format!(" {number} "),
                        title_attr,
                    );
                }
            }
        }
        write_line_to_terminal(terminal, 0, 0, &buf);

        // Middle rows - using double vertical lines
        let mut side_buf = DrawBuffer::new(width);
        side_buf.put_char(0, '║', frame_attr); // Double vertical line
        side_buf.put_char(width - 1, '║', frame_attr); // Double vertical line
        // Fill interior with background color from palette chain (matches Borland)
        // Maps through Frame's palette -> Window's palette -> App palette
        let interior_color = self.map_color(crate::core::palette::WINDOW_BACKGROUND);
        for i in 1..width - 1 {
            side_buf.put_char(i, ' ', interior_color);
        }
        for y in 1..height - 1 {
            write_line_to_terminal(terminal, 0, y as i16, &side_buf);
        }

        // Bottom border - using single-line for resizable, double-line for non-resizable
        // Matches Borland: resizable windows (wfGrow flag) use single-line bottom corners
        // to visually distinguish them and accommodate the resize handle
        let mut bottom_buf = DrawBuffer::new(width);
        if self.resizable {
            // Resizable: single-line bottom corners (matches Borland TWindow with wfGrow)
            bottom_buf.put_char(0, '└', frame_attr); // Single bottom-left corner
            bottom_buf.put_char(width - 1, '┘', frame_attr); // Single bottom-right corner
        } else {
            // Non-resizable: double-line bottom corners (matches Borland TDialog without wfGrow)
            bottom_buf.put_char(0, '╚', frame_attr); // Double bottom-left corner
            bottom_buf.put_char(width - 1, '╝', frame_attr); // Double bottom-right corner
        }
        for i in 1..width - 1 {
            bottom_buf.put_char(i, '═', frame_attr); // Double horizontal line
        }

        // Add resize handle for resizable windows when active
        // Matches Borland: dragIcon "~��~" at width-2 when (state & sfActive) && (flags & wfGrow)
        // See tframe.cc:142-144
        let is_active = self.core.state.contains(State::ACTIVE);
        if self.resizable && is_active && width >= 4 {
            // Resize handle at bottom-right corner (width-2 position)
            // Using ◢ (U+25E2) as resize indicator
            bottom_buf.put_char(width - 2, '◢', frame_attr);
        }

        write_line_to_terminal(terminal, 0, height as i16 - 1, &bottom_buf);
    }

    fn handle_event(&mut self, event: &mut Event) {
        // Note: no State::ACTIVE gate here — the owning Window only forwards
        // events to its own frame, and an inactive window can still receive
        // the click that activates it.

        // Double-click on the title row zooms the window
        // (Borland: TFrame::handleEvent converts it to cmZoom)
        if event.what == EventType::MouseDown
            && (event.mouse.buttons & MB_LEFT_BUTTON) != 0
            && event.mouse.double_click
            && event.mouse.pos.y == 0
            && !self.is_on_close_icon(event.mouse.pos)
            && !self.is_on_zoom_icon(event.mouse.pos)
        {
            *event = crate::core::event::Event::command(crate::core::command::CM_ZOOM);
            return;
        }

        if event.what == EventType::MouseDown && (event.mouse.buttons & MB_LEFT_BUTTON) != 0 {
            let mouse_pos = event.mouse.pos;

            // Any new press resets icon tracking; each is re-armed below only
            // when the press lands on that icon.
            self.close_pressed = false;
            self.zoom_pressed = false;

            // Check if click is on the resize corner (bottom-right, matching Borland tframe.cc:214)
            // Borland: mouse.x >= size.x - 2 && mouse.y >= size.y - 1
            // Only allow resize on resizable frames (matches Borland's wfGrow flag check)
            if self.resizable
                && mouse_pos.x >= self.extent().b.x - 2
                && mouse_pos.y >= self.extent().b.y - 1
            {
                // Resize corner - set resizing state
                self.core.state |= State::RESIZING;
                // DON'T clear event - let Window handle it to initialize resize_start_size
                return;
            }

            // Check if click is on the top frame line (title bar)
            if mouse_pos.y == 0 {
                // Check if click is on the close button [■] at position (2,3,4)
                if mouse_pos.x >= 2 && mouse_pos.x <= 4 {
                    // Close button area - arm press tracking, don't start
                    // drag, and consume the press so it doesn't leak to other
                    // views. Close fires only on the matching MouseUp.
                    self.close_pressed = true;
                    event.clear();
                    return;
                }

                // The zoom icon works the same way: arm the press, and fire
                // CM_ZOOM only if the release lands on it too.
                if self.is_on_zoom_icon(mouse_pos) {
                    self.zoom_pressed = true;
                    event.clear();
                    return;
                }

                // Click on title bar (not close button) - prepare for drag
                // In Borland, this calls dragWindow() which then calls owner->dragView()
                // Set dragging state and let Window handle the MouseDown event

                // Set dragging state
                self.core.state |= State::DRAGGING;
                // DON'T clear event - let Window handle it to initialize drag_offset
                return;
            }
        } else if event.what == EventType::MouseUp {
            // Handle close-icon release FIRST (before drag/resize cleanup).
            // CM_CLOSE fires only when the press ALSO started on the icon
            // (matches Borland: TFrame tracks press-release on the icon).
            let mouse_pos = event.mouse.pos;

            if self.close_pressed {
                self.close_pressed = false;
                if self.is_on_close_icon(mouse_pos) {
                    // Generate close command
                    *event = Event::command(CM_CLOSE);
                } else {
                    // Press started on the icon but was released elsewhere:
                    // cancel the close and consume the release.
                    event.clear();
                }
                // Also clear drag/resize state if set
                self.core.state &= !(State::DRAGGING | State::RESIZING);
                return;
            }

            if self.zoom_pressed {
                self.zoom_pressed = false;
                if self.is_on_zoom_icon(mouse_pos) {
                    *event = Event::command(crate::core::command::CM_ZOOM);
                } else {
                    event.clear();
                }
                self.core.state &= !(State::DRAGGING | State::RESIZING);
                return;
            }

            // End dragging or resizing
            if self.core.state.contains(State::DRAGGING) {
                self.core.state &= !State::DRAGGING;
                event.clear();
            } else if self.core.state.contains(State::RESIZING) {
                self.core.state &= !State::RESIZING;
                event.clear();
            }
        }
    }

    fn get_palette(&self) -> Option<crate::core::palette::Palette> {
        // Frame is transparent in the palette chain. Frame indices (1-3) are
        // already in the Window's index space (1-8), so they pass straight
        // through to the owner (Window), whose palette maps them to app
        // palette positions. This matches Borland's cpFrame which maps
        // Frame indices to Window indices.
        None
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}

/// Builder for creating frames with a fluent API.
///
/// # Examples
///
/// ```ignore
/// use turbo_vision::views::frame::{FrameBuilder, FramePaletteType};
/// use turbo_vision::core::geometry::Rect;
///
/// // Create a basic dialog frame
/// let frame = FrameBuilder::new()
///     .bounds(Rect::new(0, 0, 60, 20))
///     .title("My Dialog")
///     .build();
///
/// // Create a resizable editor frame
/// let frame = FrameBuilder::new()
///     .bounds(Rect::new(0, 0, 80, 25))
///     .title("EditorWindow")
///     .palette_type(FramePaletteType::EditorWindow)
///     .resizable(true)
///     .build();
/// ```
pub struct FrameBuilder {
    bounds: Option<Rect>,
    title: Option<String>,
    palette_type: FramePaletteType,
    resizable: bool,
}

impl FrameBuilder {
    /// Creates a new FrameBuilder with default values.
    pub fn new() -> Self {
        Self {
            bounds: None,
            title: None,
            palette_type: FramePaletteType::Dialog,
            resizable: false,
        }
    }

    /// Sets the frame bounds (required).
    #[must_use]
    pub fn bounds(mut self, bounds: Rect) -> Self {
        self.bounds = Some(bounds);
        self
    }

    /// Sets the frame title (required).
    #[must_use]
    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
        self
    }

    /// Sets the frame palette type (default: Dialog).
    #[must_use]
    pub fn palette_type(mut self, palette_type: FramePaletteType) -> Self {
        self.palette_type = palette_type;
        self
    }

    /// Sets whether the frame is resizable (default: false).
    #[must_use]
    pub fn resizable(mut self, resizable: bool) -> Self {
        self.resizable = resizable;
        self
    }

    /// Builds the Frame.
    ///
    /// # Panics
    ///
    /// Panics if required fields (bounds, title) are not set.
    pub fn build(self) -> Frame {
        let bounds = self.bounds.expect("Frame bounds must be set");
        let title = self.title.expect("Frame title must be set");
        Frame::with_palette(bounds, &title, self.palette_type, self.resizable)
    }

    /// Builds the Frame as a Box.
    pub fn build_boxed(self) -> Box<Frame> {
        Box::new(self.build())
    }
}

impl Default for FrameBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::geometry::Point;

    fn frame() -> Frame {
        Frame::new(Rect::new(0, 0, 40, 10), "Test", false)
    }

    fn mouse(what: EventType, x: i16, y: i16) -> Event {
        Event::mouse(what, Point::new(x, y), MB_LEFT_BUTTON, false)
    }

    #[test]
    fn test_close_button_press_and_release_generates_close() {
        let mut f = frame();

        // Press on the close icon: event is consumed (does not leak)
        let mut down = mouse(EventType::MouseDown, 3, 0);
        f.handle_event(&mut down);
        assert_eq!(down.what, EventType::Nothing);

        // Release on the icon: CM_CLOSE is generated
        let mut up = mouse(EventType::MouseUp, 3, 0);
        f.handle_event(&mut up);
        assert_eq!(up.what, EventType::Command);
        assert_eq!(up.command, CM_CLOSE);
    }

    #[test]
    fn test_release_on_close_icon_without_press_does_not_close() {
        let mut f = frame();

        // MouseUp over the icon with no prior press on it — must NOT close
        let mut up = mouse(EventType::MouseUp, 3, 0);
        f.handle_event(&mut up);
        assert_ne!(up.what, EventType::Command);
    }

    #[test]
    fn test_press_on_icon_release_elsewhere_cancels_close() {
        let mut f = frame();

        let mut down = mouse(EventType::MouseDown, 3, 0);
        f.handle_event(&mut down);

        // Release away from the icon: close is cancelled, release consumed
        let mut up = mouse(EventType::MouseUp, 20, 5);
        f.handle_event(&mut up);
        assert_eq!(up.what, EventType::Nothing);

        // A later release over the icon must not close either
        let mut up2 = mouse(EventType::MouseUp, 3, 0);
        f.handle_event(&mut up2);
        assert_ne!(up2.what, EventType::Command);
    }

    #[test]
    fn test_press_elsewhere_disarms_close_tracking() {
        let mut f = frame();

        // Arm, then press somewhere else on the title bar
        let mut down = mouse(EventType::MouseDown, 3, 0);
        f.handle_event(&mut down);
        let mut down2 = mouse(EventType::MouseDown, 20, 0);
        f.handle_event(&mut down2);
        // End the drag started by the title-bar press
        let mut up_drag = mouse(EventType::MouseUp, 20, 0);
        f.handle_event(&mut up_drag);

        // Release over the icon: the icon press was superseded — no close
        let mut up = mouse(EventType::MouseUp, 3, 0);
        f.handle_event(&mut up);
        assert_ne!(up.what, EventType::Command);
    }

    // --- Zoom icon -------------------------------------------------------

    fn zoomable_frame() -> Frame {
        Frame::new(Rect::new(0, 0, 40, 10), "Title", true)
    }

    fn press_at(frame: &mut Frame, x: i16, y: i16) -> Event {
        let mut e = Event::nothing();
        e.what = EventType::MouseDown;
        e.mouse.buttons = MB_LEFT_BUTTON;
        e.mouse.pos = crate::core::geometry::Point::new(x, y);
        frame.handle_event(&mut e);
        e
    }

    fn release_at(frame: &mut Frame, x: i16, y: i16) -> Event {
        let mut e = Event::nothing();
        e.what = EventType::MouseUp;
        e.mouse.pos = crate::core::geometry::Point::new(x, y);
        frame.handle_event(&mut e);
        e
    }

    #[test]
    fn the_zoom_icon_sits_five_cells_from_the_right_edge() {
        let frame = zoomable_frame();
        assert_eq!(frame.zoom_icon_x(), Some(35));
        assert!(frame.is_on_zoom_icon(crate::core::geometry::Point::new(36, 0)));
        assert!(!frame.is_on_zoom_icon(crate::core::geometry::Point::new(36, 1)));
    }

    #[test]
    fn a_fixed_size_frame_shows_no_zoom_icon() {
        // Borland pairs wfZoom with wfGrow, and a dialog has neither.
        let frame = Frame::new(Rect::new(0, 0, 40, 10), "Title", false);
        assert_eq!(frame.zoom_icon_x(), None);
    }

    #[test]
    fn a_narrow_frame_shows_no_zoom_icon() {
        let frame = Frame::new(Rect::new(0, 0, 10, 5), "T", true);
        assert_eq!(frame.zoom_icon_x(), None, "no room beside the close icon");
    }

    #[test]
    fn press_and_release_on_the_zoom_icon_zooms() {
        let mut frame = zoomable_frame();
        let down = press_at(&mut frame, 36, 0);
        assert_eq!(down.what, EventType::Nothing, "the press is consumed");
        let up = release_at(&mut frame, 36, 0);
        assert_eq!(up.what, EventType::Command);
        assert_eq!(up.command, crate::core::command::CM_ZOOM);
    }

    #[test]
    fn releasing_off_the_zoom_icon_cancels() {
        let mut frame = zoomable_frame();
        press_at(&mut frame, 36, 0);
        let up = release_at(&mut frame, 20, 0);
        assert_ne!(up.what, EventType::Command, "dragged off the icon");
    }

    #[test]
    fn a_press_on_the_zoom_icon_does_not_start_a_drag() {
        let mut frame = zoomable_frame();
        press_at(&mut frame, 36, 0);
        assert!(
            !frame.state().contains(State::DRAGGING),
            "the title bar drag must not begin on an icon"
        );
    }

    #[test]
    fn double_clicking_the_zoom_icon_is_left_to_the_press_tracking() {
        // A double-click anywhere else on the title bar zooms directly; on the
        // icon itself the press/release pair already does it, so the shortcut
        // must not fire twice.
        let mut frame = zoomable_frame();
        let mut e = Event::nothing();
        e.what = EventType::MouseDown;
        e.mouse.buttons = MB_LEFT_BUTTON;
        e.mouse.double_click = true;
        e.mouse.pos = crate::core::geometry::Point::new(36, 0);
        frame.handle_event(&mut e);
        assert_ne!(e.what, EventType::Command);
    }

    /// The deprecated override still works for a bare frame that was never
    /// told the extent a zoom would fill.
    #[test]
    #[allow(
        deprecated,
        reason = "covers the fallback set_zoomed still stands in for"
    )]
    fn the_triangle_follows_the_zoomed_state() {
        let mut frame = zoomable_frame();
        assert!(!frame.is_zoomed());
        frame.set_zoomed(true);
        assert!(frame.is_zoomed());
    }

    #[test]
    fn hiding_the_zoom_icon_also_stops_its_clicks() {
        let mut frame = zoomable_frame();
        frame.set_zoomable(false);
        assert_eq!(frame.zoom_icon_x(), None);
        press_at(&mut frame, 36, 0);
        let up = release_at(&mut frame, 36, 0);
        assert_ne!(up.command, crate::core::command::CM_ZOOM);
    }

    #[test]
    fn the_triangle_is_derived_from_the_bounds_not_remembered() {
        let mut frame = zoomable_frame();
        frame.set_max_bounds(Rect::new(0, 0, 80, 25));
        assert!(!frame.is_zoomed(), "a 40x10 window in an 80x25 desktop");

        frame.set_bounds(Rect::new(0, 0, 80, 25));
        assert!(frame.is_zoomed(), "filling the desktop reads as zoomed");
    }

    #[test]
    fn shrinking_a_zoomed_window_flips_the_triangle_back() {
        // Regression: the flag used to be written only by the zoom command, so
        // Tile or Cascade left a shrunken window still showing the restore
        // triangle until the next zoom toggle.
        let mut frame = zoomable_frame();
        frame.set_max_bounds(Rect::new(0, 0, 80, 25));
        frame.set_bounds(Rect::new(0, 0, 80, 25));
        assert!(frame.is_zoomed());

        // What Tile does: move and resize the window, nothing more.
        frame.set_bounds(Rect::new(0, 0, 40, 25));
        assert!(!frame.is_zoomed(), "half the desktop is not zoomed");
    }

    #[test]
    fn a_moved_window_of_full_size_still_reads_as_zoomed() {
        // Sizes decide it, not positions, matching Window::zoom; the glyph has
        // to agree with what pressing it will do.
        let mut frame = zoomable_frame();
        frame.set_max_bounds(Rect::new(0, 0, 40, 10));
        frame.set_bounds(Rect::new(5, 5, 45, 15));
        assert!(frame.is_zoomed());
    }

    #[test]
    fn a_growing_desktop_unzooms_a_window_that_no_longer_fills_it() {
        let mut frame = zoomable_frame();
        frame.set_max_bounds(Rect::new(0, 0, 40, 10));
        assert!(frame.is_zoomed());
        // The terminal was made bigger.
        frame.set_max_bounds(Rect::new(0, 0, 100, 30));
        assert!(!frame.is_zoomed());
    }

    #[test]
    fn a_frame_with_no_desktop_falls_back_to_what_it_was_told() {
        let mut frame = zoomable_frame();
        assert!(!frame.is_zoomed());
        #[allow(deprecated)]
        frame.set_zoomed(true);
        assert!(frame.is_zoomed(), "no max bounds, so the override stands");

        // Once the extent is known it wins, override or not.
        frame.set_max_bounds(Rect::new(0, 0, 80, 25));
        assert!(!frame.is_zoomed());
    }
}
