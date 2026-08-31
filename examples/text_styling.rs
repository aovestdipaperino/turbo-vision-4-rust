// (C) 2026 - Enzo Lombardi
// Text Styling Demo
//
// Prints a table of text-style samples straight to the terminal, using the
// same SGR emission path the framework uses for rendering (`ansi_dump`).
// Your terminal renders the real bold / italic / underline / reverse / dim /
// strikethrough attributes.
//
// Run with:
//   cargo run --example text_styling

use std::io::{self, Write};

use turbo_vision::core::ansi_dump::dump_buffer;
use turbo_vision::core::draw::Cell;
use turbo_vision::core::palette::{Attr, Style, TvColor};

/// Write `text` into `row` starting at column `col`, using `attr`.
fn put(row: &mut [Cell], col: usize, text: &str, attr: Attr) {
    for (i, ch) in text.chars().enumerate() {
        if col + i < row.len() {
            row[col + i] = Cell::new(ch, attr);
        }
    }
}

fn main() -> io::Result<()> {
    const WIDTH: usize = 60;

    // A neutral base: light-gray on black.
    let base = Attr::new(TvColor::LightGray, TvColor::Black);
    let label = Attr::new(TvColor::DarkGray, TvColor::Black);

    // (name, styled attr) pairs. Sample text uses the same base colors so the
    // only visible difference is the style flag.
    let sample = Attr::new(TvColor::White, TvColor::Black);
    let rows: Vec<(&str, Attr)> = vec![
        ("normal", sample),
        ("bold", sample.bold()),
        ("dim", sample.dim()),
        ("italic", sample.italic()),
        ("underline", sample.underline()),
        ("reverse", sample.reverse()),
        ("strikethrough", sample.strikethrough()),
        ("bold + italic", sample.bold().italic()),
        ("bold + underline", sample.bold().underline()),
        (
            "italic + underline + strike",
            sample.with_style(Style::ITALIC | Style::UNDERLINE | Style::STRIKETHROUGH),
        ),
        ("bold yellow", Attr::new(TvColor::Yellow, TvColor::Black).bold()),
        ("underline cyan", Attr::new(TvColor::Cyan, TvColor::Black).underline()),
    ];

    let mut buffer: Vec<Vec<Cell>> = Vec::new();

    // Header
    let mut header = vec![Cell::new(' ', base); WIDTH];
    put(&mut header, 2, "turbo-vision text styling", base.bold());
    buffer.push(header);
    buffer.push(vec![Cell::new(' ', base); WIDTH]); // blank line

    for (name, attr) in &rows {
        let mut line = vec![Cell::new(' ', base); WIDTH];
        put(&mut line, 2, name, label);
        put(&mut line, 30, "The quick brown fox", *attr);
        buffer.push(line);
    }

    let height = buffer.len();
    let stdout = io::stdout();
    let mut lock = stdout.lock();
    writeln!(lock)?;
    dump_buffer(&mut lock, &buffer, WIDTH, height)?;
    writeln!(lock)?;
    Ok(())
}
