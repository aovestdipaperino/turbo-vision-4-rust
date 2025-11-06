// (C) 2025 - Enzo Lombardi
//! ANSI dump utilities for debugging terminal output
//!
//! This module provides functionality to dump terminal buffers to ANSI text files,
//! which can be viewed with `cat` or any text editor that supports ANSI escape codes.
//!
//! # Examples
//!
//! ## Dumping the entire screen
//! ```no_run
//! use turbo_vision::terminal::Terminal;
//!
//! let terminal = Terminal::init().unwrap();
//! // ... draw some UI ...
//! terminal.dump_screen("debug_screen.ans").unwrap();
//! ```
//!
//! ## Dumping a specific view
//! ```no_run
//! use turbo_vision::prelude::*;
//! use turbo_vision::views::{dialog::Dialog, View};
//! use turbo_vision::terminal::Terminal;
//!
//! let mut terminal = Terminal::init().unwrap();
//! let dialog = Dialog::new(Rect::new(10, 5, 50, 15), "Test");
//! // ... draw the dialog ...
//! dialog.dump_to_file(&terminal, "debug_dialog.ans").unwrap();
//! ```
//!
//! ## Viewing the dumped files
//! On Unix-like systems, you can view the ANSI files with:
//! ```bash
//! cat debug_screen.ans
//! less -R debug_screen.ans  # For scrollable viewing
//! ```

use super::draw::Cell;
use super::palette::TvColor;
use std::io::{self, Write};
use std::fs::File;

/// Convert TvColor to RGB values for 24-bit ANSI codes
fn color_to_rgb(color: TvColor) -> (u8, u8, u8) {
    match color {
        TvColor::Black => (0, 0, 0),
        TvColor::Blue => (0, 0, 170),
        TvColor::Green => (0, 170, 0),
        TvColor::Cyan => (0, 170, 170),
        TvColor::Red => (170, 0, 0),
        TvColor::Magenta => (170, 0, 170),
        TvColor::Brown => (170, 85, 0),
        TvColor::LightGray => (170, 170, 170),
        TvColor::DarkGray => (85, 85, 85),
        TvColor::LightBlue => (85, 85, 255),
        TvColor::LightGreen => (85, 255, 85),
        TvColor::LightCyan => (85, 255, 255),
        TvColor::LightRed => (255, 85, 85),
        TvColor::LightMagenta => (255, 85, 255),
        TvColor::Yellow => (255, 255, 85),
        TvColor::White => (255, 255, 255),
    }
}

/// Dump a buffer to an ANSI text file.
///
/// Creates a file with ANSI color codes viewable with `cat`.
///
/// # Arguments
/// * `buffer` - The 2D cell buffer to dump
/// * `width` - Width of the region to dump
/// * `height` - Height of the region to dump
/// * `path` - File path where the dump will be saved
pub fn dump_buffer_to_file(
    buffer: &[Vec<Cell>],
    width: usize,
    height: usize,
    path: &str,
) -> io::Result<()> {
    let mut file = File::create(path)?;
    dump_buffer(&mut file, buffer, width, height)?;
    Ok(())
}

/// Dump a buffer to any writer with ANSI color codes.
///
/// Optimizes output by only emitting color codes when colors change.
///
/// # Arguments
/// * `writer` - Output writer (file, stdout, or any `Write` implementor)
/// * `buffer` - The 2D cell buffer to dump
/// * `width` - Width of the region to dump
/// * `height` - Height of the region to dump
pub fn dump_buffer<W: Write>(
    writer: &mut W,
    buffer: &[Vec<Cell>],
    width: usize,
    height: usize,
) -> io::Result<()> {
    for row in buffer.iter().take(height.min(buffer.len())) {
        let mut last_fg = None;
        let mut last_bg = None;

        for x in 0..width.min(row.len()) {
            let cell = row[x];

            // Only emit color codes when colors change
            let need_fg_change = Some(cell.attr.fg) != last_fg;
            let need_bg_change = Some(cell.attr.bg) != last_bg;

            if need_fg_change || need_bg_change {
                if need_fg_change && need_bg_change {
                    let (fg_r, fg_g, fg_b) = color_to_rgb(cell.attr.fg);
                    let (bg_r, bg_g, bg_b) = color_to_rgb(cell.attr.bg);
                    write!(
                        writer,
                        "\x1b[38;2;{};{};{};48;2;{};{};{}m",
                        fg_r, fg_g, fg_b, bg_r, bg_g, bg_b
                    )?;
                } else if need_fg_change {
                    let (fg_r, fg_g, fg_b) = color_to_rgb(cell.attr.fg);
                    write!(writer, "\x1b[38;2;{};{};{}m", fg_r, fg_g, fg_b)?;
                } else {
                    let (bg_r, bg_g, bg_b) = color_to_rgb(cell.attr.bg);
                    write!(writer, "\x1b[48;2;{};{};{}m", bg_r, bg_g, bg_b)?;
                }
                last_fg = Some(cell.attr.fg);
                last_bg = Some(cell.attr.bg);
            }

            write!(writer, "{}", cell.ch)?;
        }

        // Reset colors at end of line
        writeln!(writer, "\x1b[0m")?;
    }

    Ok(())
}

/// Dump a rectangular region of a buffer.
///
/// Useful for dumping individual views or UI components.
///
/// # Arguments
/// * `writer` - Output writer (file, stdout, or any `Write` implementor)
/// * `buffer` - The 2D cell buffer to dump from
/// * `x` - Starting X coordinate
/// * `y` - Starting Y coordinate
/// * `width` - Width of the region
/// * `height` - Height of the region
pub fn dump_buffer_region<W: Write>(
    writer: &mut W,
    buffer: &[Vec<Cell>],
    x: usize,
    y: usize,
    width: usize,
    height: usize,
) -> io::Result<()> {
    for row in buffer.iter().take((y + height).min(buffer.len())).skip(y) {
        let mut last_fg = None;
        let mut last_bg = None;

        for col in x..(x + width).min(row.len()) {
            let cell = row[col];

            let need_fg_change = Some(cell.attr.fg) != last_fg;
            let need_bg_change = Some(cell.attr.bg) != last_bg;

            if need_fg_change || need_bg_change {
                if need_fg_change && need_bg_change {
                    let (fg_r, fg_g, fg_b) = color_to_rgb(cell.attr.fg);
                    let (bg_r, bg_g, bg_b) = color_to_rgb(cell.attr.bg);
                    write!(
                        writer,
                        "\x1b[38;2;{};{};{};48;2;{};{};{}m",
                        fg_r, fg_g, fg_b, bg_r, bg_g, bg_b
                    )?;
                } else if need_fg_change {
                    let (fg_r, fg_g, fg_b) = color_to_rgb(cell.attr.fg);
                    write!(writer, "\x1b[38;2;{};{};{}m", fg_r, fg_g, fg_b)?;
                } else {
                    let (bg_r, bg_g, bg_b) = color_to_rgb(cell.attr.bg);
                    write!(writer, "\x1b[48;2;{};{};{}m", bg_r, bg_g, bg_b)?;
                }
                last_fg = Some(cell.attr.fg);
                last_bg = Some(cell.attr.bg);
            }

            write!(writer, "{}", cell.ch)?;
        }

        writeln!(writer, "\x1b[0m")?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::palette::Attr;

    #[test]
    fn test_dump_simple_buffer() {
        let cells = vec![
            Cell::new('H', Attr::new(TvColor::White, TvColor::Blue)),
            Cell::new('i', Attr::new(TvColor::White, TvColor::Blue)),
        ];

        let buffer = vec![cells];
        let mut output = Vec::new();

        dump_buffer(&mut output, &buffer, 2, 1).unwrap();

        let result = String::from_utf8(output).unwrap();
        assert!(result.contains("Hi"));
        assert!(result.contains("\x1b[")); // Contains ANSI codes
    }
}
