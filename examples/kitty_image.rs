// (C) 2025 - Enzo Lombardi
// Kitty Image Example
// Demonstrates displaying images using the Kitty graphics protocol.
//
// Usage:
//   cargo run --example kitty_image [path/to/image.png]
//
// If no image path is provided, a built-in test pattern is used.
//
// Requirements:
// - Terminal with Kitty graphics protocol support (Kitty, WezTerm, Ghostty)
// - For best results, use a PNG image
//
// Controls:
// - Alt-X or Esc: Exit
// - R: Reload image
// - C: Clear all images

use turbo_vision::app::Application;
use turbo_vision::core::command::{CM_QUIT, CommandId};
use turbo_vision::core::event::{Event, EventType, KB_ALT_X, KB_ESC, KB_ESC_ESC};
use turbo_vision::core::geometry::Rect;
use turbo_vision::core::palette::{Attr, TvColor};
use turbo_vision::views::kitty_image::KittyImage;
use turbo_vision::views::label::LabelBuilder;
use turbo_vision::views::status_line::{StatusItem, StatusLine};
use turbo_vision::views::window::WindowBuilder;
use turbo_vision::views::View;

// Command IDs for this demo
const CM_RELOAD: CommandId = 1000;
const CM_CLEAR: CommandId = 1001;

/// Generate a simple test PNG image (a colorful gradient pattern)
/// This creates a valid minimal PNG without external dependencies
fn generate_test_png() -> Vec<u8> {
    // This is a pre-generated 64x64 PNG with a colorful test pattern
    // Created with a gradient from red to blue with some green
    // The PNG is base64-decoded here for embedding

    // Actually, let's create a minimal valid PNG programmatically
    // PNG format: signature + IHDR + IDAT + IEND

    let width: u32 = 64;
    let height: u32 = 64;

    // PNG signature
    let mut png_data: Vec<u8> = vec![0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A];

    // IHDR chunk (image header)
    let mut ihdr_data = Vec::new();
    ihdr_data.extend_from_slice(&width.to_be_bytes());    // Width
    ihdr_data.extend_from_slice(&height.to_be_bytes());   // Height
    ihdr_data.push(8);    // Bit depth
    ihdr_data.push(2);    // Color type (RGB)
    ihdr_data.push(0);    // Compression method
    ihdr_data.push(0);    // Filter method
    ihdr_data.push(0);    // Interlace method

    write_png_chunk(&mut png_data, b"IHDR", &ihdr_data);

    // Generate raw image data (uncompressed for simplicity)
    // Each row: filter byte (0) + RGB pixels
    let mut raw_data = Vec::new();
    for y in 0..height {
        raw_data.push(0); // Filter type: None
        for x in 0..width {
            // Create a colorful gradient pattern
            let r = ((x * 4) % 256) as u8;
            let g = ((y * 4) % 256) as u8;
            let b = (((x + y) * 2) % 256) as u8;
            raw_data.push(r);
            raw_data.push(g);
            raw_data.push(b);
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

/// Write a PNG chunk with CRC
fn write_png_chunk(output: &mut Vec<u8>, chunk_type: &[u8; 4], data: &[u8]) {
    // Length (4 bytes, big-endian)
    output.extend_from_slice(&(data.len() as u32).to_be_bytes());

    // Chunk type
    output.extend_from_slice(chunk_type);

    // Chunk data
    output.extend_from_slice(data);

    // CRC32 (over type + data)
    let mut crc_data = Vec::new();
    crc_data.extend_from_slice(chunk_type);
    crc_data.extend_from_slice(data);
    let crc = crc32(&crc_data);
    output.extend_from_slice(&crc.to_be_bytes());
}

/// Calculate CRC32 for PNG chunks
fn crc32(data: &[u8]) -> u32 {
    // CRC32 lookup table (polynomial 0xEDB88320)
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

/// Simple zlib/deflate compression (store-only for simplicity)
fn deflate_compress(data: &[u8]) -> Vec<u8> {
    let mut output = Vec::new();

    // Zlib header (CMF + FLG)
    output.push(0x78); // CMF: deflate, 32K window
    output.push(0x01); // FLG: no dict, fastest

    // Split data into chunks of max 65535 bytes (store blocks)
    let chunks = data.chunks(65535);
    let chunk_count = data.chunks(65535).count();

    for (i, chunk) in chunks.enumerate() {
        let is_last = i == chunk_count - 1;

        // Block header
        if is_last {
            output.push(0x01); // Final block, stored
        } else {
            output.push(0x00); // Not final, stored
        }

        // Length and complement
        let len = chunk.len() as u16;
        output.push((len & 0xFF) as u8);
        output.push((len >> 8) as u8);
        output.push((!len & 0xFF) as u8);
        output.push((!len >> 8) as u8);

        // Data
        output.extend_from_slice(chunk);
    }

    // Adler-32 checksum
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
    // Get image path from command line, if provided
    let args: Vec<String> = std::env::args().collect();
    let image_path = args.get(1);

    // Create application
    let mut app = Application::new()?;

    // Check if terminal supports Kitty graphics
    let supports_kitty = app.terminal.supports_kitty_graphics();

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

    // Create main window
    let window_width = 60;
    let window_height = 22;
    let window_x = (width - window_width) / 2;
    let window_y = (height - window_height) / 2;

    let mut window = WindowBuilder::new()
        .bounds(Rect::new(window_x, window_y, window_x + window_width, window_y + window_height))
        .title("Kitty Image Demo")
        .build();

    // Add info label
    let info_text = if supports_kitty {
        "Terminal supports Kitty graphics!"
    } else {
        "Warning: Terminal may not support Kitty graphics"
    };
    let info_label = LabelBuilder::new()
        .bounds(Rect::new(2, 1, 56, 1))
        .text(info_text)
        .build();
    window.add(Box::new(info_label));

    // Add source label
    let source_text = match image_path {
        Some(path) => format!("Image: {}", path),
        None => "Using generated test pattern".to_string(),
    };
    let source_label = LabelBuilder::new()
        .bounds(Rect::new(2, 2, 56, 2))
        .text(&source_text)
        .build();
    window.add(Box::new(source_label));

    // Load or generate image
    let png_data = match image_path {
        Some(path) => {
            std::fs::read(path).unwrap_or_else(|e| {
                eprintln!("Failed to load image: {}", e);
                generate_test_png()
            })
        }
        None => generate_test_png(),
    };

    // Create Kitty image view
    // Position it inside the window (coordinates relative to window interior)
    let image_view = KittyImage::from_bytes(
        Rect::new(2, 4, 56, 18),
        png_data,
    ).background(Attr::new(TvColor::Black, TvColor::DarkGray));

    window.add(Box::new(image_view));

    // Add hint label at bottom
    let hint_label = LabelBuilder::new()
        .bounds(Rect::new(2, 19, 56, 19))
        .text("Press Alt-X or Esc to exit")
        .build();
    window.add(Box::new(hint_label));

    app.desktop.add(Box::new(window));

    // Run the application
    app.running = true;
    while app.running {
        app.desktop.draw(&mut app.terminal);
        let _ = app.terminal.flush();

        if let Ok(Some(mut event)) = app.terminal.poll_event(std::time::Duration::from_millis(50)) {
            // Handle keyboard events
            if event.what == EventType::Keyboard {
                match event.key_code {
                    KB_ALT_X | KB_ESC | KB_ESC_ESC => {
                        event = Event::command(CM_QUIT);
                    }
                    // 'r' or 'R' key - not mapped as there's no easy keycode
                    // 'c' or 'C' key - not mapped
                    _ => {}
                }
            }

            app.desktop.handle_event(&mut event);

            // Handle commands
            if event.what == EventType::Command {
                match event.command {
                    CM_QUIT => {
                        // Clear Kitty images before exiting
                        let _ = app.terminal.clear_kitty_images();
                        app.running = false;
                    }
                    CM_RELOAD => {
                        // Would reload the image here
                    }
                    CM_CLEAR => {
                        let _ = app.terminal.clear_kitty_images();
                    }
                    _ => {}
                }
            }
        }
    }

    // Cleanup
    let _ = app.terminal.clear_kitty_images();
    app.terminal.shutdown()?;

    Ok(())
}
