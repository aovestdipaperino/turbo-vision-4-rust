// (C) 2025 - Enzo Lombardi
// Kitty Background Example
// Demonstrates using a Kitty image as the desktop background.
//
// Usage:
//   cargo run --example kitty_background
//
// Requirements:
// - Terminal with Kitty graphics protocol support (Kitty, WezTerm, Ghostty)
//
// Controls:
// - Alt-X or Esc: Exit
// - Drag windows to see the background

use turbo_vision::app::Application;
use turbo_vision::core::command::CM_QUIT;
use turbo_vision::core::event::{Event, EventType, KB_ALT_X, KB_ESC, KB_ESC_ESC};
use turbo_vision::core::geometry::Rect;
use turbo_vision::views::kitty_image::KittyImage;
use turbo_vision::views::label::LabelBuilder;
use turbo_vision::views::status_line::{StatusItem, StatusLine};
use turbo_vision::views::window::WindowBuilder;
use turbo_vision::views::View;

/// Generate a gray-on-gray pattern PNG
/// Creates a subtle checkerboard/texture pattern
fn generate_gray_pattern(width: u32, height: u32) -> Vec<u8> {
    // PNG signature
    let mut png_data: Vec<u8> = vec![0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A];

    // IHDR chunk (image header)
    let mut ihdr_data = Vec::new();
    ihdr_data.extend_from_slice(&width.to_be_bytes());
    ihdr_data.extend_from_slice(&height.to_be_bytes());
    ihdr_data.push(8);    // Bit depth
    ihdr_data.push(2);    // Color type (RGB)
    ihdr_data.push(0);    // Compression method
    ihdr_data.push(0);    // Filter method
    ihdr_data.push(0);    // Interlace method

    write_png_chunk(&mut png_data, b"IHDR", &ihdr_data);

    // Generate raw image data with gray pattern
    let mut raw_data = Vec::new();
    for y in 0..height {
        raw_data.push(0); // Filter type: None
        for x in 0..width {
            // Create a subtle texture pattern with multiple gray tones
            let pattern = create_gray_pattern(x, y);
            raw_data.push(pattern); // R
            raw_data.push(pattern); // G
            raw_data.push(pattern); // B
        }
    }

    // Compress with deflate (zlib format)
    let compressed = deflate_compress(&raw_data);

    // IDAT chunk (image data)
    write_png_chunk(&mut png_data, b"IDAT", &compressed);

    // IEND chunk (image end)
    write_png_chunk(&mut png_data, b"IEND", &[]);

    png_data
}

/// Create a subtle gray pattern value for a given pixel
fn create_gray_pattern(x: u32, y: u32) -> u8 {
    // Base gray level (medium gray, similar to Turbo Vision desktop)
    let base_gray: u8 = 85;

    // Create a subtle woven/textile pattern
    let pattern1 = ((x / 2) % 2) ^ ((y / 2) % 2); // 2x2 checkerboard
    let pattern2 = ((x / 4) % 2) ^ ((y / 4) % 2); // 4x4 checkerboard
    let pattern3 = ((x + y) % 4 == 0) as u32;     // Diagonal dots

    // Combine patterns for a subtle texture
    let variation = (pattern1 * 3 + pattern2 * 2 + pattern3 * 2) as u8;

    // Keep it subtle - vary only by a few shades
    base_gray.saturating_add(variation).min(95)
}

/// Write a PNG chunk with CRC
fn write_png_chunk(output: &mut Vec<u8>, chunk_type: &[u8; 4], data: &[u8]) {
    output.extend_from_slice(&(data.len() as u32).to_be_bytes());
    output.extend_from_slice(chunk_type);
    output.extend_from_slice(data);
    let mut crc_data = Vec::new();
    crc_data.extend_from_slice(chunk_type);
    crc_data.extend_from_slice(data);
    let crc = crc32(&crc_data);
    output.extend_from_slice(&crc.to_be_bytes());
}

/// Calculate CRC32 for PNG chunks
fn crc32(data: &[u8]) -> u32 {
    static CRC_TABLE: [u32; 256] = {
        let mut table = [0u32; 256];
        let mut i = 0;
        while i < 256 {
            let mut c = i as u32;
            let mut k = 0;
            while k < 8 {
                if c & 1 != 0 {
                    c = 0xEDB88320 ^ (c >> 1);
                } else {
                    c >>= 1;
                }
                k += 1;
            }
            table[i] = c;
            i += 1;
        }
        table
    };

    let mut crc = 0xFFFFFFFFu32;
    for byte in data {
        crc = CRC_TABLE[((crc ^ (*byte as u32)) & 0xFF) as usize] ^ (crc >> 8);
    }
    crc ^ 0xFFFFFFFF
}

/// Simple zlib/deflate compression (store-only)
fn deflate_compress(data: &[u8]) -> Vec<u8> {
    let mut output = Vec::new();
    output.push(0x78);
    output.push(0x01);

    let chunks = data.chunks(65535);
    let chunk_count = data.chunks(65535).count();

    for (i, chunk) in chunks.enumerate() {
        let is_last = i == chunk_count - 1;
        output.push(if is_last { 0x01 } else { 0x00 });
        let len = chunk.len() as u16;
        output.push((len & 0xFF) as u8);
        output.push((len >> 8) as u8);
        output.push((!len & 0xFF) as u8);
        output.push((!len >> 8) as u8);
        output.extend_from_slice(chunk);
    }

    let adler = adler32(data);
    output.extend_from_slice(&adler.to_be_bytes());
    output
}

/// Calculate Adler-32 checksum
fn adler32(data: &[u8]) -> u32 {
    let mut a: u32 = 1;
    let mut b: u32 = 0;
    for byte in data {
        a = (a + *byte as u32) % 65521;
        b = (b + a) % 65521;
    }
    (b << 16) | a
}

fn main() -> turbo_vision::core::error::Result<()> {
    let mut app = Application::new()?;

    let (width, height) = app.terminal.size();

    // Create status line
    let status_line = StatusLine::new(
        Rect::new(0, height - 1, width, height),
        vec![
            StatusItem::new("~Alt-X~ Exit", KB_ALT_X, CM_QUIT),
            StatusItem::new("~Esc~ Exit", KB_ESC, CM_QUIT),
        ],
    );
    app.set_status_line(status_line);

    // Generate gray pattern for background
    // Make it large enough to tile or cover the screen
    let pattern_width = 256;
    let pattern_height = 256;
    let gray_pattern = generate_gray_pattern(pattern_width, pattern_height);

    // Create Kitty image as desktop background
    // Position it at (0, 1) to avoid menu bar area, spanning full desktop
    // Use negative z-index to place behind text and windows
    let background = KittyImage::from_bytes(
        Rect::new(0, 1, width, height - 1),
        gray_pattern,
    ).z_index(-1);

    // Add background to desktop first (will be behind windows)
    app.desktop.add(Box::new(background));

    // Create main info window (blue background contrasts with gray pattern)
    let window_width = 50;
    let window_height = 12;
    let window_x = (width - window_width) / 2;
    let window_y = (height - window_height) / 2 - 3;

    let mut window = WindowBuilder::new()
        .bounds(Rect::new(window_x, window_y, window_x + window_width, window_y + window_height))
        .title("Kitty Background Demo")
        .build();

    let label1 = LabelBuilder::new()
        .bounds(Rect::new(2, 2, 46, 2))
        .text("Desktop background using Kitty graphics!")
        .build();
    let label2 = LabelBuilder::new()
        .bounds(Rect::new(2, 4, 46, 4))
        .text("Try dragging the windows around.")
        .build();
    let label3 = LabelBuilder::new()
        .bounds(Rect::new(2, 6, 46, 6))
        .text("The gray pattern is a PNG image.")
        .build();
    let label4 = LabelBuilder::new()
        .bounds(Rect::new(2, 8, 46, 8))
        .text("Press Alt-X or Esc to exit.")
        .build();

    window.add(Box::new(label1));
    window.add(Box::new(label2));
    window.add(Box::new(label3));
    window.add(Box::new(label4));

    app.desktop.add(Box::new(window));

    // Create a small About window
    let about_width = 36;
    let about_height = 9;
    let about_x = width - about_width - 5;
    let about_y = height - about_height - 3;

    let mut about_window = WindowBuilder::new()
        .bounds(Rect::new(about_x, about_y, about_x + about_width, about_y + about_height))
        .title("About")
        .build();

    let about1 = LabelBuilder::new()
        .bounds(Rect::new(2, 2, 32, 2))
        .text("Turbo Vision for Rust")
        .build();
    let about2 = LabelBuilder::new()
        .bounds(Rect::new(2, 3, 32, 3))
        .text("Version 1.0.2")
        .build();
    let about3 = LabelBuilder::new()
        .bounds(Rect::new(2, 5, 32, 5))
        .text("Kitty Graphics Protocol Demo")
        .build();
    let about4 = LabelBuilder::new()
        .bounds(Rect::new(2, 6, 32, 6))
        .text("Drag me around!")
        .build();

    about_window.add(Box::new(about1));
    about_window.add(Box::new(about2));
    about_window.add(Box::new(about3));
    about_window.add(Box::new(about4));

    app.desktop.add(Box::new(about_window));

    // Run the application
    app.running = true;
    while app.running {
        app.desktop.draw(&mut app.terminal);
        let _ = app.terminal.flush();

        if let Ok(Some(mut event)) = app.terminal.poll_event(std::time::Duration::from_millis(50)) {
            if event.what == EventType::Keyboard {
                match event.key_code {
                    KB_ALT_X | KB_ESC | KB_ESC_ESC => {
                        event = Event::command(CM_QUIT);
                    }
                    _ => {}
                }
            }

            app.desktop.handle_event(&mut event);

            if event.what == EventType::Command && event.command == CM_QUIT {
                let _ = app.terminal.clear_kitty_images();
                app.running = false;
            }
        }
    }

    let _ = app.terminal.clear_kitty_images();
    app.terminal.shutdown()?;

    Ok(())
}
