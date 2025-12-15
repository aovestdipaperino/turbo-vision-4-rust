// (C) 2025 - Enzo Lombardi

//! ANSI escape sequence parser for loading colored text art.
//!
//! This module provides parsing of ANSI escape sequences from text files,
//! converting them to turbo-vision's Cell format for display.
//!
//! # Supported escape sequences
//!
//! - Basic 16-color: `\x1b[30-37m` (foreground), `\x1b[40-47m` (background)
//! - Bright colors: `\x1b[90-97m` (foreground), `\x1b[100-107m` (background)
//! - 256-color: `\x1b[38;5;Nm` (foreground), `\x1b[48;5;Nm` (background)
//! - True color RGB: `\x1b[38;2;R;G;Bm` (foreground), `\x1b[48;2;R;G;Bm` (background)
//! - Reset: `\x1b[0m`
//! - Bold/bright: `\x1b[1m`
//!
//! # Example
//!
//! ```rust
//! use turbo_vision::core::ansi::AnsiParser;
//! use turbo_vision::core::palette::{Attr, TvColor};
//!
//! let text = "\x1b[31mRed\x1b[0m Normal";
//! let parser = AnsiParser::new();
//! let cells = parser.parse_line(text);
//!
//! // cells[0..3] are red 'R', 'e', 'd'
//! // cells[4..] are normal colored ' ', 'N', 'o', 'r', 'm', 'a', 'l'
//! ```

use super::draw::Cell;
use super::palette::{Attr, TvColor};
use std::fs;
use std::io;
use std::path::Path;

/// A parsed ANSI art image consisting of lines of cells.
#[derive(Debug, Clone)]
pub struct AnsiImage {
    /// Lines of cells, each cell containing a character and its color attributes.
    pub lines: Vec<Vec<Cell>>,
    /// Width of the widest line (in characters).
    pub width: usize,
    /// Number of lines.
    pub height: usize,
}

impl AnsiImage {
    /// Creates an empty ANSI image.
    pub fn new() -> Self {
        Self {
            lines: Vec::new(),
            width: 0,
            height: 0,
        }
    }

    /// Loads an ANSI image from a file.
    ///
    /// # Arguments
    /// * `path` - Path to the ANSI art file
    ///
    /// # Returns
    /// The parsed ANSI image or an IO error.
    pub fn load<P: AsRef<Path>>(path: P) -> io::Result<Self> {
        let content = fs::read_to_string(path)?;
        Ok(Self::parse(&content))
    }

    /// Parses ANSI art from a string.
    pub fn parse(content: &str) -> Self {
        let parser = AnsiParser::new();
        let mut lines = Vec::new();
        let mut max_width = 0;

        for line in content.lines() {
            let cells = parser.parse_line(line);
            max_width = max_width.max(cells.len());
            lines.push(cells);
        }

        let height = lines.len();

        Self {
            lines,
            width: max_width,
            height,
        }
    }

    /// Gets a specific cell, or None if out of bounds.
    pub fn get(&self, x: usize, y: usize) -> Option<&Cell> {
        self.lines.get(y).and_then(|line| line.get(x))
    }

    /// Gets a cell at position, returning a space with default colors if out of bounds.
    pub fn get_or_default(&self, x: usize, y: usize, default_attr: Attr) -> Cell {
        self.get(x, y)
            .copied()
            .unwrap_or(Cell::new(' ', default_attr))
    }
}

impl Default for AnsiImage {
    fn default() -> Self {
        Self::new()
    }
}

/// Parser for ANSI escape sequences.
pub struct AnsiParser {
    /// Default foreground color.
    default_fg: TvColor,
    /// Default background color.
    default_bg: TvColor,
}

impl AnsiParser {
    /// Creates a new ANSI parser with default colors (light gray on black).
    pub fn new() -> Self {
        Self {
            default_fg: TvColor::LightGray,
            default_bg: TvColor::Black,
        }
    }

    /// Creates a new ANSI parser with custom default colors.
    pub fn with_defaults(fg: TvColor, bg: TvColor) -> Self {
        Self {
            default_fg: fg,
            default_bg: bg,
        }
    }

    /// Parses a single line of text containing ANSI escape sequences.
    ///
    /// Returns a vector of cells, one for each visible character.
    pub fn parse_line(&self, line: &str) -> Vec<Cell> {
        let mut cells = Vec::new();
        let mut current_fg = self.default_fg;
        let mut current_bg = self.default_bg;
        let mut bright = false;

        let mut chars = line.chars().peekable();

        while let Some(ch) = chars.next() {
            if ch == '\x1b' {
                // Start of escape sequence
                if chars.peek() == Some(&'[') {
                    chars.next(); // consume '['
                    let (new_fg, new_bg, new_bright) =
                        self.parse_sgr(&mut chars, current_fg, current_bg, bright);
                    current_fg = new_fg;
                    current_bg = new_bg;
                    bright = new_bright;
                }
            } else if ch != '\r' {
                // Regular character (skip carriage return)
                let fg = if bright {
                    Self::brighten(current_fg)
                } else {
                    current_fg
                };
                cells.push(Cell::new(ch, Attr::new(fg, current_bg)));
            }
        }

        cells
    }

    /// Parse SGR (Select Graphic Rendition) parameters.
    fn parse_sgr(
        &self,
        chars: &mut std::iter::Peekable<std::str::Chars>,
        mut fg: TvColor,
        mut bg: TvColor,
        mut bright: bool,
    ) -> (TvColor, TvColor, bool) {
        let mut params = Vec::new();
        let mut current_param = String::new();

        // Parse until 'm' or invalid character
        loop {
            match chars.peek() {
                Some(&'m') => {
                    chars.next();
                    if !current_param.is_empty() {
                        params.push(current_param);
                    }
                    break;
                }
                Some(&';') => {
                    chars.next();
                    params.push(current_param);
                    current_param = String::new();
                }
                Some(&c) if c.is_ascii_digit() => {
                    chars.next();
                    current_param.push(c);
                }
                _ => {
                    // Invalid sequence, consume until 'm' or give up
                    while let Some(&c) = chars.peek() {
                        chars.next();
                        if c == 'm' || !c.is_ascii() {
                            break;
                        }
                    }
                    return (fg, bg, bright);
                }
            }
        }

        // Process parameters
        let mut i = 0;
        while i < params.len() {
            let code: u8 = params[i].parse().unwrap_or(0);
            match code {
                0 => {
                    // Reset
                    fg = self.default_fg;
                    bg = self.default_bg;
                    bright = false;
                }
                1 => {
                    // Bold/bright
                    bright = true;
                }
                22 => {
                    // Normal intensity (not bold)
                    bright = false;
                }
                30..=37 => {
                    // Standard foreground colors
                    fg = Self::ansi_to_tv_color(code - 30);
                }
                38 => {
                    // Extended foreground color
                    if i + 1 < params.len() {
                        let mode: u8 = params[i + 1].parse().unwrap_or(0);
                        match mode {
                            5 if i + 2 < params.len() => {
                                // 256-color mode
                                let color_idx: u8 = params[i + 2].parse().unwrap_or(0);
                                fg = Self::ansi256_to_tv_color(color_idx);
                                i += 2;
                            }
                            2 if i + 4 < params.len() => {
                                // True color RGB
                                let r: u8 = params[i + 2].parse().unwrap_or(0);
                                let g: u8 = params[i + 3].parse().unwrap_or(0);
                                let b: u8 = params[i + 4].parse().unwrap_or(0);
                                fg = TvColor::from_rgb(r, g, b);
                                i += 4;
                            }
                            _ => {}
                        }
                    }
                }
                39 => {
                    // Default foreground
                    fg = self.default_fg;
                }
                40..=47 => {
                    // Standard background colors
                    bg = Self::ansi_to_tv_color(code - 40);
                }
                48 => {
                    // Extended background color
                    if i + 1 < params.len() {
                        let mode: u8 = params[i + 1].parse().unwrap_or(0);
                        match mode {
                            5 if i + 2 < params.len() => {
                                // 256-color mode
                                let color_idx: u8 = params[i + 2].parse().unwrap_or(0);
                                bg = Self::ansi256_to_tv_color(color_idx);
                                i += 2;
                            }
                            2 if i + 4 < params.len() => {
                                // True color RGB
                                let r: u8 = params[i + 2].parse().unwrap_or(0);
                                let g: u8 = params[i + 3].parse().unwrap_or(0);
                                let b: u8 = params[i + 4].parse().unwrap_or(0);
                                bg = TvColor::from_rgb(r, g, b);
                                i += 4;
                            }
                            _ => {}
                        }
                    }
                }
                49 => {
                    // Default background
                    bg = self.default_bg;
                }
                90..=97 => {
                    // Bright foreground colors
                    fg = Self::ansi_to_tv_color_bright(code - 90);
                }
                100..=107 => {
                    // Bright background colors
                    bg = Self::ansi_to_tv_color_bright(code - 100);
                }
                _ => {}
            }
            i += 1;
        }

        (fg, bg, bright)
    }

    /// Convert ANSI color code (0-7) to `TvColor`.
    fn ansi_to_tv_color(code: u8) -> TvColor {
        match code {
            0 => TvColor::Black,
            1 => TvColor::Red,
            2 => TvColor::Green,
            3 => TvColor::Brown, // ANSI yellow maps to TV brown (dark yellow)
            4 => TvColor::Blue,
            5 => TvColor::Magenta,
            6 => TvColor::Cyan,
            _ => TvColor::LightGray, // 7 and any unexpected values
        }
    }

    /// Convert ANSI bright color code (0-7) to `TvColor`.
    fn ansi_to_tv_color_bright(code: u8) -> TvColor {
        match code {
            0 => TvColor::DarkGray,
            1 => TvColor::LightRed,
            2 => TvColor::LightGreen,
            3 => TvColor::Yellow,
            4 => TvColor::LightBlue,
            5 => TvColor::LightMagenta,
            6 => TvColor::LightCyan,
            _ => TvColor::White, // 7 and any unexpected values
        }
    }

    /// Convert 256-color palette index to `TvColor`.
    fn ansi256_to_tv_color(idx: u8) -> TvColor {
        match idx {
            // Standard colors (0-7)
            0 => TvColor::Black,
            1 => TvColor::Red,
            2 => TvColor::Green,
            3 => TvColor::Brown,
            4 => TvColor::Blue,
            5 => TvColor::Magenta,
            6 => TvColor::Cyan,
            7 => TvColor::LightGray,
            // Bright colors (8-15)
            8 => TvColor::DarkGray,
            9 => TvColor::LightRed,
            10 => TvColor::LightGreen,
            11 => TvColor::Yellow,
            12 => TvColor::LightBlue,
            13 => TvColor::LightMagenta,
            14 => TvColor::LightCyan,
            15 => TvColor::White,
            // 216-color cube (16-231) and grayscale (232-255)
            // Map to closest TvColor
            16..=231 => {
                // 6x6x6 color cube
                let idx = idx - 16;
                let r = (idx / 36) * 51;
                let g = ((idx % 36) / 6) * 51;
                let b = (idx % 6) * 51;
                TvColor::from_rgb(r, g, b)
            }
            232..=255 => {
                // Grayscale (24 shades)
                let gray = (idx - 232) * 10 + 8;
                TvColor::from_rgb(gray, gray, gray)
            }
        }
    }

    /// Make a color brighter (for bold attribute).
    fn brighten(color: TvColor) -> TvColor {
        match color {
            TvColor::Black => TvColor::DarkGray,
            TvColor::Red => TvColor::LightRed,
            TvColor::Green => TvColor::LightGreen,
            TvColor::Brown => TvColor::Yellow,
            TvColor::Blue => TvColor::LightBlue,
            TvColor::Magenta => TvColor::LightMagenta,
            TvColor::Cyan => TvColor::LightCyan,
            TvColor::LightGray => TvColor::White,
            // Already bright colors stay the same
            other => other,
        }
    }
}

impl Default for AnsiParser {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_plain_text() {
        let parser = AnsiParser::new();
        let cells = parser.parse_line("Hello");
        assert_eq!(cells.len(), 5);
        assert_eq!(cells[0].ch, 'H');
        assert_eq!(cells[4].ch, 'o');
    }

    #[test]
    fn test_parse_red_text() {
        let parser = AnsiParser::new();
        let cells = parser.parse_line("\x1b[31mRed\x1b[0m");
        assert_eq!(cells.len(), 3);
        assert_eq!(cells[0].ch, 'R');
        assert_eq!(cells[0].attr.fg, TvColor::Red);
    }

    #[test]
    fn test_parse_bright_color() {
        let parser = AnsiParser::new();
        let cells = parser.parse_line("\x1b[91mBright\x1b[0m");
        assert_eq!(cells[0].attr.fg, TvColor::LightRed);
    }

    #[test]
    fn test_parse_background_color() {
        let parser = AnsiParser::new();
        let cells = parser.parse_line("\x1b[44mBlue BG\x1b[0m");
        assert_eq!(cells[0].attr.bg, TvColor::Blue);
    }

    #[test]
    fn test_parse_reset() {
        let parser = AnsiParser::new();
        let cells = parser.parse_line("\x1b[31mRed\x1b[0mNormal");
        assert_eq!(cells[0].attr.fg, TvColor::Red);
        assert_eq!(cells[3].attr.fg, TvColor::LightGray);
    }

    #[test]
    fn test_parse_bold() {
        let parser = AnsiParser::new();
        let cells = parser.parse_line("\x1b[1;34mBold Blue\x1b[0m");
        assert_eq!(cells[0].attr.fg, TvColor::LightBlue);
    }

    #[test]
    fn test_ansi_image_parse() {
        let content = "Line 1\n\x1b[31mLine 2\x1b[0m\nLine 3";
        let image = AnsiImage::parse(content);
        assert_eq!(image.height, 3);
        assert_eq!(image.lines[1][0].attr.fg, TvColor::Red);
    }
}
