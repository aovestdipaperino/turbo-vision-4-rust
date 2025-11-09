// (C) 2025 - Enzo Lombardi

//! Frame view - window border with title and close button.

use crate::core::geometry::Rect;
use crate::core::event::{Event, EventType, MB_LEFT_BUTTON};
use crate::core::draw::DrawBuffer;
use crate::core::palette::{Attr, TvColor};
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
    pub fn new(bounds: Rect, title: &str) -> Self {
        Self::with_palette(bounds, title, FramePaletteType::Dialog)
    }

    pub fn with_palette(bounds: Rect, title: &str, palette_type: FramePaletteType) -> Self {
        Self {
            bounds,
            title: title.to_string(),
            palette_type,
            state: SF_ACTIVE,  // Default to active
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
        // Borland determines cFrame based on state:
        // - Inactive: cFrame = 0x0101 (both bytes use palette[1])
        // - Dragging: cFrame = 0x0505 (both bytes use palette[5])
        // - Active:   cFrame = 0x0503 (low=palette[3], high=palette[5])

        let is_active = (self.state & SF_ACTIVE) != 0;
        let is_dragging = (self.state & SF_DRAGGING) != 0;

        match self.palette_type {
            FramePaletteType::Dialog => {
                if !is_active {
                    // Inactive: cFrame = 0x0101, cTitle = 0x0002
                    // cpDialog[1] = 0x21 (Blue on Green) -> mapped to DarkGray on LightGray
                    let inactive_attr = Attr::new(TvColor::DarkGray, TvColor::LightGray);
                    (inactive_attr, inactive_attr, inactive_attr)
                } else if is_dragging {
                    // Dragging: cFrame = 0x0505, cTitle = 0x0005
                    // cpDialog[5] = 0x25 (Magenta on Green) -> mapped to LightGreen on LightGray
                    let dragging_attr = Attr::new(TvColor::LightGreen, TvColor::LightGray);
                    (dragging_attr, dragging_attr, dragging_attr)
                } else {
                    // Active: cFrame = 0x0503, cTitle = 0x0004
                    // cpDialog[3] = 0x23 (Cyan on Green) -> White on LightGray (frame)
                    // cpDialog[5] = 0x25 (Magenta on Green) -> LightGreen on LightGray (highlight)
                    // cpDialog[4] = 0x24 (Red on Green) -> White on LightGray (title)
                    let frame_attr = Attr::new(TvColor::White, TvColor::LightGray);  // White on LightGray
                    let close_icon_attr = Attr::new(TvColor::LightGreen, TvColor::LightGray);
                    let title_attr = Attr::new(TvColor::White, TvColor::LightGray);  // White on LightGray
                    (frame_attr, close_icon_attr, title_attr)
                }
            }
            FramePaletteType::Editor => {
                // cpBlueWindow palette mapping (Borland TWindow with wpBlueWindow)
                // See TWindow.cc line 38: palette(wpBlueWindow)
                // TWindow frame is White on Blue background

                if !is_active {
                    // Inactive: LightGreen on Blue (all elements)
                    let inactive_attr = Attr::new(TvColor::LightGreen, TvColor::Blue);
                    (inactive_attr, inactive_attr, inactive_attr)
                } else if is_dragging {
                    // Dragging: White on Blue (maintains blue background while dragging)
                    let dragging_attr = Attr::new(TvColor::White, TvColor::Blue);
                    (dragging_attr, dragging_attr, dragging_attr)
                } else {
                    // Active: White on Blue for frame and close icon, Yellow on Blue for title
                    let frame_attr = Attr::new(TvColor::White, TvColor::Blue);  // Border
                    let close_icon_attr = Attr::new(TvColor::White, TvColor::Blue);  // Close icon
                    let title_attr = Attr::new(TvColor::Yellow, TvColor::Blue);  // Title
                    (frame_attr, close_icon_attr, title_attr)
                }
            }
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
        let width = self.bounds.width() as usize;
        let height = self.bounds.height() as usize;

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
        for i in 1..width - 1 {
            side_buf.put_char(i, ' ', frame_attr);
        }
        for y in 1..height - 1 {
            write_line_to_terminal(terminal, self.bounds.a.x, self.bounds.a.y + y as i16, &side_buf);
        }

        // Bottom border - using double-line box drawing
        let mut bottom_buf = DrawBuffer::new(width);
        bottom_buf.put_char(0, '╚', frame_attr);  // Double bottom-left corner
        bottom_buf.put_char(width - 1, '╝', frame_attr);  // Double bottom-right corner
        for i in 1..width - 1 {
            bottom_buf.put_char(i, '═', frame_attr);  // Double horizontal line
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
            if mouse_pos.x >= self.bounds.b.x - 2 && mouse_pos.y >= self.bounds.b.y - 1 {
                // Resize corner - set resizing state
                self.state |= SF_RESIZING;
                event.clear(); // Mark event as handled
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
                // For now, we'll mark this event as consumed and set a flag
                // The Window needs to track drag state and handle MouseMove events

                // Set dragging state
                self.state |= SF_DRAGGING;
                event.clear(); // Mark event as handled
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
        None  // Frame uses hardcoded colors based on FramePaletteType
    }
}
