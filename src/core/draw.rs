// (C) 2025 - Enzo Lombardi

//! Drawing primitives - Cell and DrawBuffer types for efficient line-based rendering.

use super::palette::Attr;

/// A single character cell with attributes
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Cell {
    pub ch: char,
    pub attr: Attr,
}

impl Cell {
    pub const fn new(ch: char, attr: Attr) -> Self {
        Self { ch, attr }
    }
}

/// Buffer for efficient line-based drawing
pub struct DrawBuffer {
    pub data: Vec<Cell>,
}

impl DrawBuffer {
    /// Create a new draw buffer with the given width
    pub fn new(width: usize) -> Self {
        Self {
            data: vec![Cell::new(' ', Attr::from_u8(0x07)); width],
        }
    }

    /// Fill a range with a single character and attribute
    pub fn move_char(&mut self, pos: usize, ch: char, attr: Attr, count: usize) {
        let end = (pos + count).min(self.data.len());
        for i in pos..end {
            self.data[i] = Cell::new(ch, attr);
        }
    }

    /// Write a string with the given attribute
    pub fn move_str(&mut self, pos: usize, s: &str, attr: Attr) {
        let mut i = pos;
        for ch in s.chars() {
            if i >= self.data.len() {
                break;
            }
            self.data[i] = Cell::new(ch, attr);
            i += 1;
        }
    }

    /// Copy cells from another buffer
    pub fn move_buf(&mut self, pos: usize, src: &[Cell], count: usize) {
        let end = (pos + count).min(self.data.len()).min(pos + src.len());
        self.data[pos..end].copy_from_slice(&src[..(end - pos)]);
    }

    /// Put a single character at a position
    pub fn put_char(&mut self, pos: usize, ch: char, attr: Attr) {
        if pos < self.data.len() {
            self.data[pos] = Cell::new(ch, attr);
        }
    }

    /// Change the attribute of a cell without changing its character
    /// Matches Borland: TDrawBuffer::putAttribute (help.cc:109)
    pub fn put_attribute(&mut self, pos: usize, attr: Attr) {
        if pos < self.data.len() {
            self.data[pos].attr = attr;
        }
    }

    /// Get the length of the buffer
    pub fn len(&self) -> usize {
        self.data.len()
    }

    /// Check if the buffer is empty
    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    /// Write a string with shortcut highlighting
    /// Format: "~X~" highlights X with shortcut_attr, rest uses normal_attr
    /// Example: "~F~ile" displays "File" with "F" highlighted
    pub fn move_str_with_shortcut(&mut self, mut pos: usize, s: &str, normal_attr: Attr, shortcut_attr: Attr) -> usize {
        let mut chars = s.chars();
        let start_pos = pos;

        // Parse string character by character, handling ~X~ shortcut highlighting
        while let Some(ch) = chars.next() {
            if pos >= self.data.len() {
                break;
            }

            if ch == '~' {
                // Read characters until closing ~ and render with shortcut color
                while let Some(shortcut_ch) = chars.next() {
                    if shortcut_ch == '~' {
                        break;  // Found closing tilde
                    }
                    if pos < self.data.len() {
                        self.data[pos] = Cell::new(shortcut_ch, shortcut_attr);
                        pos += 1;
                    }
                }
            } else {
                self.data[pos] = Cell::new(ch, normal_attr);
                pos += 1;
            }
        }

        pos - start_pos  // Return number of characters written
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::palette::TvColor;

    #[test]
    fn test_draw_buffer_basic() {
        let buf = DrawBuffer::new(10);
        assert_eq!(buf.len(), 10);
        assert!(!buf.is_empty());
    }

    #[test]
    fn test_move_char() {
        let mut buf = DrawBuffer::new(10);
        let attr = Attr::new(TvColor::White, TvColor::Black);
        buf.move_char(2, 'X', attr, 3);
        assert_eq!(buf.data[1].ch, ' ');
        assert_eq!(buf.data[2].ch, 'X');
        assert_eq!(buf.data[3].ch, 'X');
        assert_eq!(buf.data[4].ch, 'X');
        assert_eq!(buf.data[5].ch, ' ');
    }

    #[test]
    fn test_move_str() {
        let mut buf = DrawBuffer::new(20);
        let attr = Attr::new(TvColor::White, TvColor::Black);
        buf.move_str(0, "Hello, World!", attr);
        assert_eq!(buf.data[0].ch, 'H');
        assert_eq!(buf.data[6].ch, ' ');
        assert_eq!(buf.data[12].ch, '!');
    }
}
