// (C) 2025 - Enzo Lombardi

//! Palette Themes Demo - demonstrates app-level palette customization
//!
//! This example shows how to customize the application palette to create different themes.
//! All views automatically use the new colors through the palette mapping system.
//!
//! ## How It Works
//!
//! 1. Create a custom palette as a Vec<u8> with 63 bytes (one per palette entry)
//! 2. Each byte encodes a color as: (foreground << 4) | background
//! 3. Call `app.set_palette(Some(palette))` to activate the theme (redraw is automatic!)
//! 4. All views (frames, buttons, editors, lists, etc.) automatically use the new palette
//!
//! ## Themes Demonstrated
//!
//! - **Default** - Classic Borland Turbo Vision colors
//! - **Dark** - Dark backgrounds with bright text for low-light environments
//! - **High-Contrast** - Black/white for maximum visibility and accessibility
//! - **Solarized** - Earth tones inspired by the Solarized color scheme
//!
//! Click the numbered buttons (1-4) to switch between themes, or Q to quit.

use std::cell::RefCell;
use std::rc::Rc;
use turbo_vision::app::Application;
use turbo_vision::core::command::CM_QUIT;
use turbo_vision::core::geometry::Rect;
use turbo_vision::views::button::ButtonBuilder;
use turbo_vision::views::dialog::Dialog;
use turbo_vision::views::input_line::InputLineBuilder;
use turbo_vision::views::label::LabelBuilder;
use turbo_vision::views::listbox::ListBox;
use turbo_vision::views::memo::MemoBuilder;

// Custom command IDs for theme switching
const CMD_THEME_DEFAULT: u16 = 1001;
const CMD_THEME_DARK: u16 = 1002;
const CMD_THEME_CONTRAST: u16 = 1003;
const CMD_THEME_SOLAR: u16 = 1004;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut app = Application::new()?;

    // Show the demo dialog - loops to allow theme switching
    loop {
        let mut dialog = create_theme_dialog("Palette Themes Demo (Click buttons to switch themes)");

        // Run the dialog modally
        let result = dialog.execute(&mut app);

        // Handle theme switching
        match result {
            CMD_THEME_DEFAULT => {
                // Reset to default Borland palette
                app.set_palette(None);
            }
            CMD_THEME_DARK => {
                app.set_palette(Some(palette_dark()));
            }
            CMD_THEME_CONTRAST => {
                app.set_palette(Some(palette_high_contrast()));
            }
            CMD_THEME_SOLAR => {
                app.set_palette(Some(palette_solarized()));
            }
            CM_QUIT | _ => break,
        }
    }

    Ok(())
}

fn create_theme_dialog(title: &str) -> Dialog {
    let mut dialog = Dialog::new(Rect::new(8, 2, 72, 26), title);

    // Add various components to show theme colors

    // Label + Input line
    let label1 = LabelBuilder::new().bounds(Rect::new(2, 2, 25, 3)).text("~I~nput Field:").build();
    dialog.add(Box::new(label1));

    let input_data = Rc::new(RefCell::new(String::new()));
    let input = InputLineBuilder::new().bounds(Rect::new(2, 3, 59, 4)).max_length(55).data(input_data).build();
    dialog.add(Box::new(input));

    // Memo (editor)
    let label2 = LabelBuilder::new().bounds(Rect::new(2, 5, 25, 6)).text("~M~emo:").build();
    dialog.add(Box::new(label2));

    let mut memo = MemoBuilder::new().bounds(Rect::new(2, 6, 59, 11)).build();
    memo.set_text("This is a memo field.\nYou can edit text here.\nNotice how colors change with themes!");
    dialog.add(Box::new(memo));

    // ListBox
    let label3 = LabelBuilder::new().bounds(Rect::new(2, 12, 25, 13)).text("~L~ist:").build();
    dialog.add(Box::new(label3));

    let mut listbox = ListBox::new(Rect::new(2, 13, 59, 17), 0);
    listbox.add_item("Normal Item 1".to_string());
    listbox.add_item("Normal Item 2".to_string());
    listbox.add_item("Normal Item 3".to_string());
    dialog.add(Box::new(listbox));

    // Buttons to switch themes (arranged in two rows)
    let btn_default = ButtonBuilder::new().bounds(Rect::new(2, 18, 2 + 13, 20)).title("~1~ Default").command(CMD_THEME_DEFAULT).build();
    dialog.add(Box::new(btn_default));

    let btn_dark = ButtonBuilder::new().bounds(Rect::new(17, 18, 17 + 13, 20)).title("~2~ Dark").command(CMD_THEME_DARK).build();
    dialog.add(Box::new(btn_dark));

    let btn_contrast = ButtonBuilder::new().bounds(Rect::new(32, 18, 32 + 13, 20)).title("~3~ Contrast").command(CMD_THEME_CONTRAST).build();
    dialog.add(Box::new(btn_contrast));

    let btn_solar = ButtonBuilder::new().bounds(Rect::new(47, 18, 47 + 13, 20)).title("~4~ Solarized").command(CMD_THEME_SOLAR).build();
    dialog.add(Box::new(btn_solar));

    // Close button
    let btn_close = ButtonBuilder::new().bounds(Rect::new(47, 20, 47 + 13, 22)).title("~Q~uit").command(CM_QUIT).build();
    dialog.add(Box::new(btn_close));

    dialog
}

/// Dark theme with darker backgrounds and brighter foregrounds
/// Good for reducing eye strain in low-light environments
#[rustfmt::skip]
fn palette_dark() -> Vec<u8> {
    // Dark theme: Use darker backgrounds (Black, DarkGray) and brighter text
    // Background: 0=Black, 8=DarkGray, 1=Blue
    // Foreground: 15=White, 14=Yellow, 11=LightCyan, 10=LightGreen
    vec![
        0x08, 0x0F, 0x08, 0x0E, 0x0B, 0x0A, 0x0C, 0x01, // 1-8: Desktop (DarkGray bg)
        0xF1, 0xE1, 0xF3, 0xF3, 0xF1, 0x08, 0x00,       // 9-15: Menu (bright on dark)
        0xF1, 0xE1, 0xA1, 0xF3, 0xF3, 0xF1, 0xE1, 0x00, // 16-23: Cyan Window
        0x08, 0xF8, 0xA8, 0xF3, 0xF3, 0x08, 0xF8, 0x00, // 24-31: Gray Window (dark)
        0x08, 0xF8, 0xA8, 0xF3, 0xF3, 0x08, 0x08, 0xF8, // 32-39: Dialog (dark gray)
        0xF8, 0xE2, 0xE2, 0xA2, 0x08, 0xA8, 0x08, 0xF3, // 40-47: Dialog controls
        0xF2, 0xA2, 0xF1, 0xA1, 0xE1, 0xE2, 0x08, 0xF3, // 48-55: InputLine, Button
        0xF3, 0xF3, 0xA1, 0xF2, 0xF3, 0xF3, 0x08, 0x00, // 56-63: Dialog remaining
    ]
}

/// High-contrast theme for accessibility
/// Strong contrast between text and background for better visibility
#[rustfmt::skip]
fn palette_high_contrast() -> Vec<u8> {
    // High contrast: Black text on White, or White text on Black
    // 0=Black, F=White, E=Yellow for highlights
    vec![
        0x0F, 0xF0, 0x0F, 0xE0, 0xF0, 0xE0, 0xF0, 0xF0, // 1-8: Desktop
        0x0F, 0xE0, 0x0F, 0x0F, 0x0F, 0x0F, 0x00,       // 9-15: Menu
        0x0F, 0x0F, 0xE0, 0x0F, 0x0F, 0x0F, 0x0F, 0x00, // 16-23: Cyan Window
        0xF0, 0x0F, 0xE0, 0x0F, 0x0F, 0xF0, 0x0F, 0x00, // 24-31: Gray Window
        0xF0, 0x0F, 0xE0, 0x0F, 0x0F, 0xF0, 0xF0, 0x0F, // 32-39: Dialog
        0x0F, 0xE0, 0xF0, 0xE0, 0xF0, 0xE0, 0xF0, 0x0F, // 40-47: Dialog controls
        0x0F, 0xE0, 0x0F, 0xE0, 0xE0, 0xE0, 0xF0, 0x0F, // 48-55: InputLine, Button
        0x0F, 0x0F, 0xE0, 0x0F, 0x0F, 0x0F, 0xF0, 0x00, // 56-63: Dialog remaining
    ]
}

/// Solarized-inspired theme
/// Warm earth tones with muted colors for comfortable viewing
#[rustfmt::skip]
fn palette_solarized() -> Vec<u8> {
    // Solarized-inspired: Brown/Yellow backgrounds, Cyan/Green/Magenta text
    // Background: 6=Brown, 3=Cyan (muted), 7=LightGray
    // Foreground: 3=Cyan, 2=Green, 11=LightCyan, 14=Yellow
    vec![
        0x36, 0x37, 0x36, 0xE6, 0x33, 0x23, 0x36, 0x37, // 1-8: Desktop (Brown bg)
        0x36, 0xE6, 0x3E, 0x3E, 0x36, 0x36, 0x00,       // 9-15: Menu
        0x36, 0xE6, 0x26, 0x3E, 0x3E, 0x36, 0xE6, 0x00, // 16-23: Cyan Window
        0x37, 0x27, 0xE7, 0x3E, 0x3E, 0x37, 0x27, 0x00, // 24-31: Gray Window
        0x37, 0x27, 0xE7, 0x3E, 0x3E, 0x37, 0x37, 0x27, // 32-39: Dialog
        0x27, 0x32, 0x22, 0x32, 0x37, 0x27, 0x37, 0x3E, // 40-47: Dialog controls
        0x36, 0x26, 0x36, 0x26, 0xE6, 0x32, 0x37, 0x3E, // 48-55: InputLine, Button
        0x3E, 0x3E, 0x26, 0x36, 0x3E, 0x3E, 0x37, 0x00, // 56-63: Dialog remaining
    ]
}
