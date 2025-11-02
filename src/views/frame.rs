use crate::core::geometry::Rect;
use crate::core::event::{Event, EventType, MB_LEFT_BUTTON};
use crate::core::draw::DrawBuffer;
use crate::core::palette::{colors, Attr, TvColor};
use crate::core::command::CM_CLOSE;
use crate::terminal::Terminal;
use super::view::{View, write_line_to_terminal};

pub struct Frame {
    bounds: Rect,
    title: String,
    /// Palette type for color mapping (Dialog vs Editor vs other window types)
    /// Matches Borland's view hierarchy palette mapping
    palette_type: FramePaletteType,
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
        }
    }

    /// Get colors for frame elements based on palette type
    /// Matches Borland's getColor() with palette mapping
    fn get_frame_colors(&self) -> (Attr, Attr) {
        // Borland: cFrame = 0x0503 for active dialogs
        // Low byte (03) = brackets, high byte (05) = close icon highlight
        match self.palette_type {
            FramePaletteType::Dialog => {
                // cpDialog palette mapping for color scheme:
                // Palette[3] (0x23: Cyan on Green) -> White on LightGray
                // Palette[5] (0x25: Magenta on Green) -> LightGreen on LightGray
                let frame_attr = colors::DIALOG_FRAME_ACTIVE;  // White on LightGray
                let close_icon_attr = Attr::new(TvColor::LightGreen, TvColor::LightGray);
                (frame_attr, close_icon_attr)
            }
            FramePaletteType::Editor => {
                // cpBlueWindow/cpCyanWindow palette mapping
                // TODO: Implement when editor windows are fully supported
                let frame_attr = Attr::new(TvColor::Yellow, TvColor::Blue);
                let close_icon_attr = Attr::new(TvColor::White, TvColor::Blue);
                (frame_attr, close_icon_attr)
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
        let (frame_attr, close_icon_attr) = self.get_frame_colors();

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
            buf.move_str(6, &format!(" {} ", self.title), colors::DIALOG_TITLE);
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
        // Handle mouse clicks on close button
        if event.what == EventType::MouseDown {
            let mouse_pos = event.mouse.pos;

            // Check if click is on the close button [■] at position (2,3,4) on the top line
            if event.mouse.buttons & MB_LEFT_BUTTON != 0
                && mouse_pos.y == self.bounds.a.y  // Top line of frame
                && mouse_pos.x >= self.bounds.a.x + 2  // Close button starts at position 2
                && mouse_pos.x <= self.bounds.a.x + 4  // Close button ends at position 4
            {
                // Generate close command
                *event = Event::command(CM_CLOSE);
            }
        }
    }
}
