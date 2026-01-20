// (C) 2025 - Enzo Lombardi
// Kitty Biorhythm Example
// Displays biorhythm chart using Kitty graphics protocol.
//
// Usage:
//   cargo run --example kitty_biorhythm
//
// Shows biorhythm cycles for today based on birthdate 12/14/1972:
// - Physical (23-day cycle) - Red
// - Emotional (28-day cycle) - Green
// - Intellectual (33-day cycle) - Blue

use turbo_vision::app::Application;
use turbo_vision::core::command::CM_QUIT;
use turbo_vision::core::event::{Event, EventType, KB_ALT_X, KB_ESC, KB_ESC_ESC};
use turbo_vision::core::geometry::Rect;
use turbo_vision::views::kitty_image::KittyImage;
use turbo_vision::views::label::LabelBuilder;
use turbo_vision::views::status_line::{StatusItem, StatusLine};
use turbo_vision::views::window::WindowBuilder;
use turbo_vision::views::View;

use chrono::{Local, NaiveDate};
use std::f64::consts::PI;

// Biorhythm cycle lengths in days
const PHYSICAL_CYCLE: f64 = 23.0;
const EMOTIONAL_CYCLE: f64 = 28.0;
const INTELLECTUAL_CYCLE: f64 = 33.0;

/// Calculate biorhythm value (-1.0 to 1.0) for a given cycle
fn biorhythm_value(days_alive: i64, cycle_length: f64) -> f64 {
    (2.0 * PI * days_alive as f64 / cycle_length).sin()
}

/// Generate a biorhythm chart as a PNG image
fn generate_biorhythm_png(birth_date: NaiveDate, width: u32, height: u32) -> Vec<u8> {
    let today = Local::now().date_naive();
    let days_alive = (today - birth_date).num_days();

    // Chart parameters
    let margin_left: u32 = 30;
    let margin_right: u32 = 10;
    let margin_top: u32 = 20;
    let margin_bottom: u32 = 25;
    let chart_width = width - margin_left - margin_right;
    let chart_height = height - margin_top - margin_bottom;

    // Days to show: 15 days before and after today
    let days_before = 15i64;
    let days_after = 15i64;
    let total_days = days_before + days_after + 1;

    // Create image buffer (RGB)
    let mut pixels: Vec<(u8, u8, u8)> = vec![(32, 32, 48); (width * height) as usize];

    // Helper to set a pixel
    let set_pixel = |pixels: &mut Vec<(u8, u8, u8)>, x: i32, y: i32, color: (u8, u8, u8)| {
        if x >= 0 && x < width as i32 && y >= 0 && y < height as i32 {
            pixels[(y as u32 * width + x as u32) as usize] = color;
        }
    };

    // Draw chart background (darker area)
    for y in margin_top..(height - margin_bottom) {
        for x in margin_left..(width - margin_right) {
            pixels[(y * width + x) as usize] = (24, 24, 36);
        }
    }

    // Draw grid lines
    let grid_color = (48, 48, 64);

    // Horizontal grid (at -1, -0.5, 0, 0.5, 1)
    for i in 0..=4 {
        let y = margin_top + (chart_height as f64 * i as f64 / 4.0) as u32;
        for x in margin_left..(width - margin_right) {
            set_pixel(&mut pixels, x as i32, y as i32, grid_color);
        }
    }

    // Vertical grid (every 5 days)
    for day in (-days_before..=days_after).step_by(5) {
        let x = margin_left + ((day + days_before) as f64 / total_days as f64 * chart_width as f64) as u32;
        for y in margin_top..(height - margin_bottom) {
            set_pixel(&mut pixels, x as i32, y as i32, grid_color);
        }
    }

    // Draw center line (y=0) brighter
    let center_y = margin_top + chart_height / 2;
    for x in margin_left..(width - margin_right) {
        set_pixel(&mut pixels, x as i32, center_y as i32, (80, 80, 100));
    }

    // Draw today line (vertical)
    let today_x = margin_left + (days_before as f64 / total_days as f64 * chart_width as f64) as u32;
    for y in margin_top..(height - margin_bottom) {
        set_pixel(&mut pixels, today_x as i32, y as i32, (100, 100, 120));
    }

    // Colors for each cycle
    let physical_color = (255, 80, 80);    // Red
    let emotional_color = (80, 255, 80);   // Green
    let intellectual_color = (80, 80, 255); // Blue

    // Draw smooth biorhythm curves using sub-pixel sampling
    let draw_curve = |pixels: &mut Vec<(u8, u8, u8)>, cycle: f64, color: (u8, u8, u8)| {
        let mut prev_x: Option<i32> = None;
        let mut prev_y: Option<i32> = None;

        // Sample at sub-pixel resolution for smooth curves
        let samples = (chart_width * 4) as i32;

        for i in 0..=samples {
            let t = i as f64 / samples as f64;
            let day_offset = t * total_days as f64 - days_before as f64;
            let value = (2.0 * PI * (days_alive as f64 + day_offset) / cycle).sin();

            // Map to pixel coordinates
            let x = margin_left as i32 + (t * chart_width as f64) as i32;
            let y = margin_top as i32 + (chart_height as f64 * (1.0 - value) / 2.0) as i32;

            // Draw line from previous point to current point
            if let (Some(px), Some(py)) = (prev_x, prev_y) {
                // Bresenham-style line drawing for smooth connection
                let dx = (x - px).abs();
                let dy = (y - py).abs();
                let steps = dx.max(dy).max(1);

                for step in 0..=steps {
                    let t = step as f64 / steps as f64;
                    let lx = px + ((x - px) as f64 * t) as i32;
                    let ly = py + ((y - py) as f64 * t) as i32;

                    // Draw 2-pixel thick line for visibility
                    set_pixel(pixels, lx, ly, color);
                    set_pixel(pixels, lx, ly + 1, color);
                    set_pixel(pixels, lx + 1, ly, color);
                }
            }

            prev_x = Some(x);
            prev_y = Some(y);
        }
    };

    draw_curve(&mut pixels, PHYSICAL_CYCLE, physical_color);
    draw_curve(&mut pixels, EMOTIONAL_CYCLE, emotional_color);
    draw_curve(&mut pixels, INTELLECTUAL_CYCLE, intellectual_color);

    // Draw today's values as dots
    let physical_today = biorhythm_value(days_alive, PHYSICAL_CYCLE);
    let emotional_today = biorhythm_value(days_alive, EMOTIONAL_CYCLE);
    let intellectual_today = biorhythm_value(days_alive, INTELLECTUAL_CYCLE);

    // Draw legend dots at today line
    let draw_dot = |pixels: &mut Vec<(u8, u8, u8)>, value: f64, color: (u8, u8, u8)| {
        let y = margin_top as i32 + (chart_height as f64 * (1.0 - value) / 2.0) as i32;
        for dy in -2..=2 {
            for dx in -2..=2 {
                if dx * dx + dy * dy <= 5 {
                    set_pixel(pixels, today_x as i32 + dx, y + dy, color);
                }
            }
        }
    };

    draw_dot(&mut pixels, physical_today, physical_color);
    draw_dot(&mut pixels, emotional_today, emotional_color);
    draw_dot(&mut pixels, intellectual_today, intellectual_color);

    // Convert to PNG
    create_png(&pixels, width, height)
}

/// Create a PNG from RGB pixel data
fn create_png(pixels: &[(u8, u8, u8)], width: u32, height: u32) -> Vec<u8> {
    // PNG signature
    let mut png_data: Vec<u8> = vec![0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A];

    // IHDR chunk
    let mut ihdr_data = Vec::new();
    ihdr_data.extend_from_slice(&width.to_be_bytes());
    ihdr_data.extend_from_slice(&height.to_be_bytes());
    ihdr_data.push(8);    // Bit depth
    ihdr_data.push(2);    // Color type (RGB)
    ihdr_data.push(0);    // Compression
    ihdr_data.push(0);    // Filter
    ihdr_data.push(0);    // Interlace

    write_png_chunk(&mut png_data, b"IHDR", &ihdr_data);

    // Generate raw image data
    let mut raw_data = Vec::new();
    for y in 0..height {
        raw_data.push(0); // Filter type: None
        for x in 0..width {
            let (r, g, b) = pixels[(y * width + x) as usize];
            raw_data.push(r);
            raw_data.push(g);
            raw_data.push(b);
        }
    }

    // Compress and write IDAT
    let compressed = deflate_compress(&raw_data);
    write_png_chunk(&mut png_data, b"IDAT", &compressed);

    // IEND chunk
    write_png_chunk(&mut png_data, b"IEND", &[]);

    png_data
}

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

    // Birthdate: December 14, 1972
    let birth_date = NaiveDate::from_ymd_opt(1972, 12, 14).unwrap();
    let today = Local::now().date_naive();
    let days_alive = (today - birth_date).num_days();

    // Calculate today's values
    let physical = biorhythm_value(days_alive, PHYSICAL_CYCLE);
    let emotional = biorhythm_value(days_alive, EMOTIONAL_CYCLE);
    let intellectual = biorhythm_value(days_alive, INTELLECTUAL_CYCLE);

    // Create status line
    let status_line = StatusLine::new(
        Rect::new(0, height - 1, width, height),
        vec![
            StatusItem::new("~Alt-X~ Exit", KB_ALT_X, CM_QUIT),
            StatusItem::new("~Esc~ Exit", KB_ESC, CM_QUIT),
        ],
    );
    app.set_status_line(status_line);

    // Generate biorhythm chart
    let chart_pixel_width = 400u32;
    let chart_pixel_height = 200u32;
    let chart_png = generate_biorhythm_png(birth_date, chart_pixel_width, chart_pixel_height);

    // Create window
    let window_width = 60;
    let window_height = 20;
    let window_x = (width - window_width) / 2;
    let window_y = (height - window_height) / 2;

    let mut window = WindowBuilder::new()
        .bounds(Rect::new(window_x, window_y, window_x + window_width, window_y + window_height))
        .title("Biorhythm Chart")
        .build();

    // Add info labels
    let date_label = LabelBuilder::new()
        .bounds(Rect::new(2, 1, 56, 1))
        .text(&format!("Birthdate: 12/14/1972  Today: {}", today.format("%m/%d/%Y")))
        .build();

    let days_label = LabelBuilder::new()
        .bounds(Rect::new(2, 2, 56, 2))
        .text(&format!("Days alive: {}", days_alive))
        .build();

    window.add(Box::new(date_label));
    window.add(Box::new(days_label));

    // Add biorhythm chart image
    let chart_view = KittyImage::from_bytes(
        Rect::new(2, 4, 56, 14),
        chart_png,
    );
    window.add(Box::new(chart_view));

    // Add legend
    let physical_label = LabelBuilder::new()
        .bounds(Rect::new(2, 15, 20, 15))
        .text(&format!("Physical: {:+.0}%", physical * 100.0))
        .build();

    let emotional_label = LabelBuilder::new()
        .bounds(Rect::new(20, 15, 38, 15))
        .text(&format!("Emotional: {:+.0}%", emotional * 100.0))
        .build();

    let intellectual_label = LabelBuilder::new()
        .bounds(Rect::new(38, 15, 56, 15))
        .text(&format!("Intellectual: {:+.0}%", intellectual * 100.0))
        .build();

    window.add(Box::new(physical_label));
    window.add(Box::new(emotional_label));
    window.add(Box::new(intellectual_label));

    // Color legend
    let legend_label = LabelBuilder::new()
        .bounds(Rect::new(2, 16, 56, 16))
        .text("Red=Physical  Green=Emotional  Blue=Intellectual")
        .build();
    window.add(Box::new(legend_label));

    app.desktop.add(Box::new(window));

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
