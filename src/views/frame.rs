// (C) 2025 - Enzo Lombardi

//! Frame view - window border with title and close button.

use crate::core::geometry::Rect;
use crate::core::event::{Event, EventType, MB_LEFT_BUTTON};
use crate::core::draw::DrawBuffer;
use crate::core::palette::Attr;
use crate::core::command::CM_CLOSE;
use crate::core::state::{StateFlags, SF_ACTIVE, SF_DRAGGING, SF_RESIZING};
use crate::terminal::Terminal;
use super::view::{View, write_line_to_terminal};

pub struct Frame {
    bounds: Rect,
    title: String,
    /// Palette type for color mapping (Dialog vs Editor vs other window types)
    /// Matches Borland's view hierarchy palette mapping
    palette_type: FramePaletteType,
    /// State flags (active, dragging, etc.) - matches Borland's TView state
    state: StateFlags,
    /// Whether the frame is resizable (matches Borland's wfGrow flag)
    /// Resizable frames use single-line bottom corners and show resize handle
    resizable: bool,
    owner: Option<*const dyn View>,
}

/// Frame palette types for different window types
/// Matches Borland's palette hierarchy (cpDialog, cpBlueWindow, etc.)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FramePaletteType {
    Dialog,    // Uses cpDialog palette (LightGreen close button)
    Editor,    // Uses cpBlueWindow/cpCyanWindow palette (different colors)
}

impl Frame {
    pub fn new(bounds: Rect, title: &str, resizable: bool) -> Self {
        Self::with_palette(bounds, title, FramePaletteType::Dialog, resizable)
    }

    pub fn with_palette(bounds: Rect, title: &str, palette_type: FramePaletteType, resizable: bool) -> Self {
        Self {
            bounds,
            title: title.to_string(),
            palette_type,
            state: SF_ACTIVE,  // Default to active
            resizable,
            owner: None,
        }
    }

    /// Set the frame title
    /// Matches Borland: TFrame::setTitle() allows changing window title dynamically
    pub fn set_title(&mut self, title: &str) {
        self.title = title.to_string();
    }

    /// Get colors for frame elements based on palette type and state
    /// Matches Borland's getColor() with palette mapping (tframe.cc:43-64)
    /// Returns (frame_attr, close_icon_attr, title_attr)
    fn get_frame_colors(&self) -> (Attr, Attr, Attr) {
        use crate::core::palette::{FRAME_INACTIVE, FRAME_ACTIVE_BORDER, FRAME_TITLE, FRAME_ICON};

        // Borland determines cFrame based on state:
        // - Inactive: cFrame = 0x0101 (both bytes use palette[1])
        // - Dragging: cFrame = 0x0505 (both bytes use palette[5])
        // - Active:   cFrame = 0x0503 (low=palette[3], high=palette[5])

        let is_active = (self.state & SF_ACTIVE) != 0;
        let is_dragging = (self.state & SF_DRAGGING) != 0;

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
    fn bounds(&self) -> Rect {
        self.bounds
    }

    fn set_bounds(&mut self, bounds: Rect) {
        self.bounds = bounds;
    }

    fn draw(&mut self, terminal: &mut Terminal) {
        let width = self.bounds.width_clamped() as usize;
        let height = self.bounds.height_clamped() as usize;

        // Don't render frames that are too small
        // Minimum: 2x2 (for top-left, top-right, bottom-left, bottom-right corners)
        if width < 2 || height < 2 {
            return;
        }

        // Get frame colors from palette mapping (matches Borland's getColor())
        let (frame_attr, close_icon_attr, title_attr) = self.get_frame_colors();

        // Top border with title - using double-line box drawing
        let mut buf = DrawBuffer::new(width);
        buf.put_char(0, '╔', frame_attr);  // Double top-left corner
        buf.put_char(width - 1, '╗', frame_attr);  // Double top-right corner
        for i in 1..width - 1 {
            buf.put_char(i, '═', frame_attr);  // Double horizontal line
        }

        // Add close button at position 2: [■]
        // Matches Borland: closeIcon = "[~\xFE~]" where ~ toggles between cFrame low/high bytes
        // For active dialog: cFrame = 0x0503
        //   - '[' and ']' use low byte (03) -> cpDialog[3] -> frame_attr (White on LightGray)
        //   - '■' uses high byte (05) -> cpDialog[5] -> close_icon_attr (LightGreen on LightGray)
        // See local-only/about.png and tframe.cc:123 (b.moveCStr(2, closeIcon, cFrame))
        if width > 5 {
            buf.put_char(2, '[', frame_attr);
            buf.put_char(3, '■', close_icon_attr);  // Uses palette highlight color
            buf.put_char(4, ']', frame_attr);
        }

        // Add title after close button
        if !self.title.is_empty() && width > self.title.len() + 8 {
            buf.move_str(6, &format!(" {} ", self.title), title_attr);
        }
        write_line_to_terminal(terminal, self.bounds.a.x, self.bounds.a.y, &buf);

        // Middle rows - using double vertical lines
        let mut side_buf = DrawBuffer::new(width);
        side_buf.put_char(0, '║', frame_attr);  // Double vertical line
        side_buf.put_char(width - 1, '║', frame_attr);  // Double vertical line
        // Fill interior with background color from palette chain (matches Borland)
        // Maps through Frame's palette -> Window's palette -> App palette
        let interior_color = self.map_color(crate::core::palette::WINDOW_BACKGROUND);
        for i in 1..width - 1 {
            side_buf.put_char(i, ' ', interior_color);
        }
        for y in 1..height - 1 {
            write_line_to_terminal(terminal, self.bounds.a.x, self.bounds.a.y + y as i16, &side_buf);
        }

        // Bottom border - using single-line for resizable, double-line for non-resizable
        // Matches Borland: resizable windows (wfGrow flag) use single-line bottom corners
        // to visually distinguish them and accommodate the resize handle
        let mut bottom_buf = DrawBuffer::new(width);
        if self.resizable {
            // Resizable: single-line bottom corners (matches Borland TWindow with wfGrow)
            bottom_buf.put_char(0, '└', frame_attr);  // Single bottom-left corner
            bottom_buf.put_char(width - 1, '┘', frame_attr);  // Single bottom-right corner
        } else {
            // Non-resizable: double-line bottom corners (matches Borland TDialog without wfGrow)
            bottom_buf.put_char(0, '╚', frame_attr);  // Double bottom-left corner
            bottom_buf.put_char(width - 1, '╝', frame_attr);  // Double bottom-right corner
        }
        for i in 1..width - 1 {
            bottom_buf.put_char(i, '═', frame_attr);  // Double horizontal line
        }

        // Add resize handle for resizable windows when active
        // Matches Borland: dragIcon "~��~" at width-2 when (state & sfActive) && (flags & wfGrow)
        // See tframe.cc:142-144
        let is_active = (self.state & SF_ACTIVE) != 0;
        if self.resizable && is_active && width >= 4 {
            // Resize handle at bottom-right corner (width-2 position)
            // Using ◢ (U+25E2) as resize indicator
            bottom_buf.put_char(width - 2, '◢', frame_attr);
        }

        write_line_to_terminal(terminal, self.bounds.a.x, self.bounds.a.y + height as i16 - 1, &bottom_buf);
    }

    fn handle_event(&mut self, event: &mut Event) {
        // Note: Removed SF_ACTIVE check - all frames are created active and never deactivated
        // The check was preventing event handling in some edge cases

        if event.what == EventType::MouseDown && (event.mouse.buttons & MB_LEFT_BUTTON) != 0 {
            let mouse_pos = event.mouse.pos;

            // Check if click is on the resize corner (bottom-right, matching Borland tframe.cc:214)
            // Borland: mouse.x >= size.x - 2 && mouse.y >= size.y - 1
            // Only allow resize on resizable frames (matches Borland's wfGrow flag check)
            if self.resizable && mouse_pos.x >= self.bounds.b.x - 2 && mouse_pos.y >= self.bounds.b.y - 1 {
                // Resize corner - set resizing state
                self.state |= SF_RESIZING;
                // DON'T clear event - let Window handle it to initialize resize_start_size
                return;
            }

            // Check if click is on the top frame line (title bar)
            if mouse_pos.y == self.bounds.a.y {
                // Check if click is on the close button [■] at position (2,3,4)
                if mouse_pos.x >= self.bounds.a.x + 2 && mouse_pos.x <= self.bounds.a.x + 4 {
                    // Close button area - don't start drag, wait for mouse up
                    return;
                }

                // Click on title bar (not close button) - prepare for drag
                // In Borland, this calls dragWindow() which then calls owner->dragView()
                // Set dragging state and let Window handle the MouseDown event

                // Set dragging state
                self.state |= SF_DRAGGING;
                // DON'T clear event - let Window handle it to initialize drag_offset
                return;
            }
        } else if event.what == EventType::MouseUp {
            // Handle mouse up on close button FIRST (before drag/resize cleanup)
            // This ensures close button works even if there was accidental mouse movement
            let mouse_pos = event.mouse.pos;

            if mouse_pos.y == self.bounds.a.y
                && mouse_pos.x >= self.bounds.a.x + 2
                && mouse_pos.x <= self.bounds.a.x + 4
            {
                // Generate close command
                *event = Event::command(CM_CLOSE);
                // Also clear drag/resize state if set
                self.state &= !(SF_DRAGGING | SF_RESIZING);
                return;
            }

            // End dragging or resizing
            if (self.state & SF_DRAGGING) != 0 {
                self.state &= !SF_DRAGGING;
                event.clear();
            } else if (self.state & SF_RESIZING) != 0 {
                self.state &= !SF_RESIZING;
                event.clear();
            }
        }
    }

    fn state(&self) -> StateFlags {
        self.state
    }

    fn set_state(&mut self, state: StateFlags) {
        self.state = state;
    }

    fn set_owner(&mut self, owner: *const dyn View) {
        self.owner = Some(owner);
    }

    fn get_owner(&self) -> Option<*const dyn View> {
        self.owner
    }

    fn get_palette(&self) -> Option<crate::core::palette::Palette> {
        use crate::core::palette::{palettes, Palette};
        match self.palette_type {
            FramePaletteType::Dialog => Some(Palette::from_slice(palettes::CP_GRAY_DIALOG)),
            FramePaletteType::Editor => Some(Palette::from_slice(palettes::CP_BLUE_WINDOW)),
        }
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
///     .title("Editor")
///     .palette_type(FramePaletteType::Editor)
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
