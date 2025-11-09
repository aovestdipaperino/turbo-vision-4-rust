// (C) 2025 - Enzo Lombardi

//! Regression tests for palette colors to ensure they remain stable across changes.
//! These tests verify that the color mapping for key UI elements produces the expected
//! final color attributes when using the default palette system.

use turbo_vision::core::geometry::{Point, Rect};
use turbo_vision::core::menu_data::{Menu, MenuItem};
use turbo_vision::core::palette::palettes;
use turbo_vision::core::palette::{BUTTON_NORMAL, BUTTON_SELECTED, BUTTON_SHADOW, SCROLLBAR_PAGE};
use turbo_vision::views::button::Button;
use turbo_vision::views::dialog::Dialog;
use turbo_vision::views::input_line::InputLine;
use turbo_vision::views::label::Label;
use turbo_vision::views::menu_bar::MenuBar;
use turbo_vision::views::menu_box::MenuBox;
use turbo_vision::views::scrollbar::ScrollBar;
use turbo_vision::views::static_text::StaticText;
use turbo_vision::views::view::{OwnerType, View};

/// Helper to create a test button and verify its color mapping
fn verify_button_colors() -> Vec<(String, u8, u8)> {
    let mut button = Button::new(Rect::new(0, 0, 10, 3), "Test", 100, false);
    button.set_owner_type(OwnerType::Dialog);
    let mut results = Vec::new();

    // Button palette indices: [12, 12, 13, 13, 15, 14, 14, 9]
    // These should map through dialog palette to final colors

    // Test each button color state
    let states = [
        ("normal", 1),
        ("default", 2),
        ("focused", 3),
        ("disabled", 4),
        ("reserved5", 5),
        ("shortcut", 6),
        ("reserved7", 7),
        ("shadow", 8),
    ];

    for (name, index) in states {
        let color = button.map_color(index);
        results.push((name.to_string(), index, color.to_u8()));
    }

    results
}

/// Helper to create a test menu bar and verify its color mapping
fn verify_menu_colors() -> Vec<(String, u8, u8)> {
    let menu_bar = MenuBar::new(Rect::new(0, 0, 80, 1));
    let mut results = Vec::new();

    // MenuBar palette indices: [2, 3, 4, 5, 6, 7]
    // These are direct app indices (no dialog remapping for top-level views)

    let states = [
        ("normal", 1),
        ("selected", 2),
        ("disabled", 3),
        ("shortcut", 4),
    ];

    for (name, index) in states {
        let color = menu_bar.map_color(index);
        results.push((format!("menubar_{}", name), index, color.to_u8()));
    }

    // Also test MenuBox (popup menu) which uses the same palette
    let menu = Menu::from_items(vec![
        MenuItem::new("Test", 100, 0, 0), // text, command, key_code, help_ctx
    ]);
    let menu_box = MenuBox::new(Point::new(10, 5), menu);
    for (name, index) in states {
        let color = menu_box.map_color(index);
        results.push((format!("menubox_{}", name), index, color.to_u8()));
    }

    results
}

/// Helper to create a test input line and verify its color mapping
fn verify_input_line_colors() -> Vec<(String, u8, u8)> {
    use std::cell::RefCell;
    use std::rc::Rc;

    let data = Rc::new(RefCell::new("test".to_string()));
    let mut input_line = InputLine::new(Rect::new(0, 0, 20, 1), 100, data);
    input_line.set_owner_type(OwnerType::Dialog);
    let mut results = Vec::new();

    // InputLine palette indices: [19, 19, 20, 21]
    // These map through dialog palette to final colors

    let states = [("passive", 1), ("active", 2), ("selected", 3), ("arrow", 4)];

    for (name, index) in states {
        let color = input_line.map_color(index);
        results.push((format!("inputline_{}", name), index, color.to_u8()));
    }

    results
}

/// Helper to create a test scrollbar and verify its color mapping
fn verify_scrollbar_colors() -> Vec<(String, u8, u8)> {
    let mut scrollbar = ScrollBar::new_vertical(Rect::new(0, 0, 1, 10));
    // Set to Window context for this test (blue on gray)
    scrollbar.set_owner_type(OwnerType::Window);
    let mut results = Vec::new();

    // ScrollBar palette indices: [4, 5, 5]
    // These are direct app palette indices (not remapped through dialog)

    let states = [
        ("page", 1),      // Maps to index 4
        ("arrows", 2),    // Maps to index 5
        ("indicator", 3), // Maps to index 5
    ];

    for (name, index) in states {
        let color = scrollbar.map_color(index);
        results.push((format!("scrollbar_{}", name), index, color.to_u8()));
    }

    results
}

/// Helper to create a test dialog and verify its color mapping
fn verify_dialog_colors() -> Vec<(String, u8, u8)> {
    let dialog = Dialog::new(Rect::new(10, 5, 50, 20), "Test Dialog");
    let mut results = Vec::new();

    // Dialog uses CP_GRAY_DIALOG palette which maps to app indices 32-63
    // Test common dialog color indices

    let indices = [
        ("frame", 1),
        ("frame_active", 2),
        ("interior", 3),
        ("text", 4),
        ("selected", 5),
        ("reserved6", 6),
        ("label_normal", 7),
        ("label_selected", 8),
        ("label_shortcut", 9),
    ];

    for (name, index) in indices {
        let color = dialog.map_color(index);
        results.push((format!("dialog_{}", name), index, color.to_u8()));
    }

    results
}

/// Helper to create a test label and verify its color mapping
fn verify_label_colors() -> Vec<(String, u8, u8)> {
    let mut label = Label::new(Rect::new(0, 0, 20, 1), "Test ~L~abel");
    label.set_owner_type(OwnerType::Dialog);
    let mut results = Vec::new();

    // Label palette indices: [7, 8, 9, 9, 13, 13]
    // Matches Borland cpLabel "\x07\x08\x09\x09\x0D\x0D"
    // These map through dialog palette to final colors

    let states = [
        ("normal_fg", 1),
        ("normal_bg", 2),
        ("light_fg", 3),
        ("light_bg", 4),
        ("disabled_fg", 5),
        ("disabled_bg", 6),
    ];

    for (name, index) in states {
        let color = label.map_color(index);
        results.push((format!("label_{}", name), index, color.to_u8()));
    }

    results
}

/// Helper to create a test static text and verify its color mapping
fn verify_static_text_colors() -> Vec<(String, u8, u8)> {
    let mut static_text = StaticText::new(Rect::new(0, 0, 20, 3), "Test Text");
    static_text.set_owner_type(OwnerType::Dialog);
    let mut results = Vec::new();

    // StaticText palette: [6]
    // Matches Borland cpStaticText "\x06"
    // Maps through dialog palette to final color

    let color = static_text.map_color(1);
    results.push(("static_text_normal".to_string(), 1, color.to_u8()));

    results
}

#[test]
fn test_button_palette_regression() {
    // Regression test: Button colors should remain stable
    // These are the expected final color values after palette remapping
    // Updated to match Borland cpButton "\x0A\x0B\x0C\x0D\x0E\x0E\x0E\x0F"

    let colors = verify_button_colors();

    // CP_BUTTON[1]=10 -> CP_GRAY_DIALOG[10]=41 -> CP_APP_COLOR[41]=0x20 (Black on Green)
    assert_eq!(colors[0].2, 0x20, "Button normal color changed!");

    // CP_BUTTON[2]=11 -> CP_GRAY_DIALOG[11]=42 -> CP_APP_COLOR[42]=0x2B (LightGreen on Green)
    assert_eq!(colors[1].2, 0x2B, "Button default color changed!");

    // CP_BUTTON[3]=12 -> CP_GRAY_DIALOG[12]=43 -> CP_APP_COLOR[43]=0x2F (White on Green)
    assert_eq!(colors[2].2, 0x2F, "Button focused color changed!");

    // CP_BUTTON[4]=13 -> CP_GRAY_DIALOG[13]=44 -> CP_APP_COLOR[44]=0x78 (DarkGray on LightGray)
    assert_eq!(colors[3].2, 0x78, "Button disabled color changed!");

    // CP_BUTTON[5]=14 -> CP_GRAY_DIALOG[14]=45 -> CP_APP_COLOR[45]=0x2E (Yellow on Green)
    assert_eq!(colors[4].2, 0x2E, "Button reserved5 color changed!");

    // CP_BUTTON[6]=14 -> same as reserved5
    assert_eq!(colors[5].2, 0x2E, "Button shortcut color changed!");

    // CP_BUTTON[7]=14 -> same as reserved5
    assert_eq!(colors[6].2, 0x2E, "Button reserved7 color changed!");

    // CP_BUTTON[8]=15 -> CP_GRAY_DIALOG[15]=46 -> CP_APP_COLOR[46]=0x70 (Black on LightGray)
    assert_eq!(colors[7].2, 0x70, "Button shadow color changed!");
}

#[test]
fn test_menu_palette_regression() {
    // Regression test: Menu colors should remain stable
    // MenuBar uses direct app indices (no dialog remapping)

    let colors = verify_menu_colors();

    // MenuBar colors:
    // CP_MENU_BAR[1]=2 -> CP_APP_COLOR[2]=0x70 (Black on LightGray)
    assert_eq!(colors[0].2, 0x70, "MenuBar normal color changed!");

    // CP_MENU_BAR[2]=39 -> CP_APP_COLOR[39]=0x7F (White on LightGray)
    assert_eq!(colors[1].2, 0x7F, "MenuBar selected color changed!");

    // CP_MENU_BAR[3]=3 -> CP_APP_COLOR[3]=0x78 (DarkGray on LightGray)
    assert_eq!(colors[2].2, 0x78, "MenuBar disabled color changed!");

    // CP_MENU_BAR[4]=4 -> CP_APP_COLOR[4]=0x74 (Red on LightGray)
    assert_eq!(colors[3].2, 0x74, "MenuBar shortcut color changed!");

    // MenuBox should have the same colors as MenuBar
    assert_eq!(colors[4].2, 0x70, "MenuBox normal color changed!");
    assert_eq!(colors[5].2, 0x7F, "MenuBox selected color changed!");
    assert_eq!(colors[6].2, 0x78, "MenuBox disabled color changed!");
    assert_eq!(colors[7].2, 0x74, "MenuBox shortcut color changed!");
}

#[test]
fn test_dialog_palette_regression() {
    // Regression test: Dialog colors should remain stable
    // Dialog uses CP_GRAY_DIALOG which maps indices 1-32 to app indices 32-63
    // Updated to match Borland cpColor

    let colors = verify_dialog_colors();

    // Dialog[1] -> CP_GRAY_DIALOG[1]=32 -> CP_APP_COLOR[32]=0x70 (Black on LightGray)
    assert_eq!(colors[0].2, 0x70, "Dialog frame color changed!");

    // Dialog[2] -> CP_GRAY_DIALOG[2]=33 -> CP_APP_COLOR[33]=0x7F (White on LightGray)
    assert_eq!(colors[1].2, 0x7F, "Dialog frame active color changed!");

    // Dialog[3] -> CP_GRAY_DIALOG[3]=34 -> CP_APP_COLOR[34]=0x7A (LightGreen on LightGray)
    assert_eq!(colors[2].2, 0x7A, "Dialog interior color changed!");

    // Dialog[4] -> CP_GRAY_DIALOG[4]=35 -> CP_APP_COLOR[35]=0x13 (LightCyan on Blue)
    assert_eq!(colors[3].2, 0x13, "Dialog text color changed!");

    // Dialog[5] -> CP_GRAY_DIALOG[5]=36 -> CP_APP_COLOR[36]=0x13 (LightCyan on Blue)
    assert_eq!(colors[4].2, 0x13, "Dialog selected color changed!");

    // Dialog[6] -> CP_GRAY_DIALOG[6]=37 -> CP_APP_COLOR[37]=0x70 (Black on LightGray)
    assert_eq!(colors[5].2, 0x70, "Dialog reserved6 color changed!");

    // Dialog[7] -> CP_GRAY_DIALOG[7]=38 -> CP_APP_COLOR[38]=0x70 (Black on LightGray)
    assert_eq!(colors[6].2, 0x70, "Dialog label normal color changed!");

    // Dialog[8] -> CP_GRAY_DIALOG[8]=39 -> CP_APP_COLOR[39]=0x7F (White on LightGray)
    assert_eq!(colors[7].2, 0x7F, "Dialog label selected color changed!");

    // Dialog[9] -> CP_GRAY_DIALOG[9]=40 -> CP_APP_COLOR[40]=0x7E (Yellow on LightGray)
    assert_eq!(colors[8].2, 0x7E, "Dialog label shortcut color changed!");
}

#[test]
fn test_input_line_palette_regression() {
    // Regression test: InputLine colors should remain stable
    // These are the expected final color values after palette remapping
    // Updated to match Borland cpColor

    let colors = verify_input_line_colors();

    // CP_INPUT_LINE[1]=19 -> CP_GRAY_DIALOG[19]=50 -> CP_APP_COLOR[50]=0x1F (White on Blue)
    assert_eq!(colors[0].2, 0x1F, "InputLine passive color changed!");

    // CP_INPUT_LINE[2]=19 -> same as passive
    assert_eq!(colors[1].2, 0x1F, "InputLine active color changed!");

    // CP_INPUT_LINE[3]=20 -> CP_GRAY_DIALOG[20]=51 -> CP_APP_COLOR[51]=0x2F (White on Green)
    assert_eq!(colors[2].2, 0x2F, "InputLine selected color changed!");

    // CP_INPUT_LINE[4]=21 -> CP_GRAY_DIALOG[21]=52 -> CP_APP_COLOR[52]=0x1A (LightGreen on Blue)
    assert_eq!(colors[3].2, 0x1A, "InputLine arrow color changed!");
}

#[test]
fn test_scrollbar_palette_regression() {
    // Regression test: ScrollBar colors should remain stable
    // ScrollBar uses direct app palette indices (no dialog remapping)
    // Updated to match Borland cpColor

    let colors = verify_scrollbar_colors();

    // ScrollBar colors for Window context:
    // CP_SCROLLBAR[1]=4 -> CP_APP_COLOR[4]=0x74 (Red on LightGray)
    assert_eq!(colors[0].2, 0x74, "ScrollBar page color changed!");

    // CP_SCROLLBAR[2]=5 -> CP_APP_COLOR[5]=0x20 (Black on Green)
    assert_eq!(colors[1].2, 0x20, "ScrollBar arrows color changed!");

    // CP_SCROLLBAR[3]=5 -> CP_APP_COLOR[5]=0x20 (Black on Green)
    assert_eq!(colors[2].2, 0x20, "ScrollBar indicator color changed!");
}

#[test]
fn test_scrollbar_context_colors() {
    // Test that ScrollBar shows different colors in Window vs Dialog contexts
    // Updated to match Borland cpColor

    // Test ScrollBar in Window context
    let mut scrollbar = ScrollBar::new_vertical(Rect::new(0, 0, 1, 10));
    scrollbar.set_owner_type(OwnerType::Window);

    let window_color = scrollbar.map_color(SCROLLBAR_PAGE).to_u8();
    assert_eq!(
        window_color, 0x74,
        "ScrollBar in Window context color changed"
    );

    // Test ScrollBar in Dialog context
    let mut scrollbar = ScrollBar::new_vertical(Rect::new(0, 0, 1, 10));
    scrollbar.set_owner_type(OwnerType::Dialog);

    let dialog_color = scrollbar.map_color(SCROLLBAR_PAGE).to_u8();
    // In dialog context, it remaps through dialog palette
    // CP_SCROLLBAR[1]=4 -> CP_GRAY_DIALOG[4]=35 -> CP_APP_COLOR[35]=0x13
    assert_eq!(
        dialog_color, 0x13,
        "ScrollBar in Dialog context color changed"
    );
}

#[test]
fn test_palette_remapping_ranges() {
    // Test that the remapping ranges work as expected
    // Updated to match Borland cpColor

    // Create a button to test dialog control remapping
    let mut button = Button::new(Rect::new(0, 0, 10, 3), "Test", 100, false);
    button.set_owner_type(OwnerType::Dialog);

    // Test that button colors are mapped correctly (already tested above in detail)
    // Just verify a few key indices to ensure remapping works
    assert_eq!(button.map_color(BUTTON_NORMAL).to_u8(), 0x20); // Normal button color
    assert_eq!(button.map_color(BUTTON_SELECTED).to_u8(), 0x2F); // Focused button color (updated)
    assert_eq!(button.map_color(BUTTON_SHADOW).to_u8(), 0x70); // Shadow color (updated)

    // Create a menu bar to test top-level view (no remapping for 1-8)
    let menu_bar = MenuBar::new(Rect::new(0, 0, 80, 1));

    // Test that indices 1-8 don't get remapped for top-level views
    for index in 1..=8 {
        let color = menu_bar.map_color(index);
        // MenuBar palette maps these, but then they should go direct to app
        let menu_mapped = match index {
            1 => 2,  // normal
            2 => 39, // selected
            3 => 3,  // disabled
            4 => 4,  // shortcut
            _ => index, // MenuBar only defines 4 entries
        };

        // For indices > 4, MenuBar palette returns 0, so we expect error color (0x0F)
        if index <= 4 {
            let expected_color = palettes::CP_APP_COLOR[(menu_mapped - 1) as usize];
            assert_eq!(
                color.to_u8(),
                expected_color,
                "Top-level view remapping incorrect for index {}",
                index
            );
        }
    }
}

#[test]
fn test_label_palette_regression() {
    // Regression test: Label colors should remain stable
    // These are the expected final color values after palette remapping
    // Label palette matches Borland cpLabel "\x07\x08\x09\x09\x0D\x0D"

    let colors = verify_label_colors();

    // CP_LABEL[1]=7 -> CP_GRAY_DIALOG[7]=38 -> CP_APP_COLOR[38]=0x70 (Black on LightGray)
    assert_eq!(colors[0].2, 0x70, "Label normal fg color changed!");

    // CP_LABEL[2]=8 -> CP_GRAY_DIALOG[8]=39 -> CP_APP_COLOR[39]=0x7F (White on LightGray)
    assert_eq!(colors[1].2, 0x7F, "Label normal bg color changed!");

    // CP_LABEL[3]=9 -> CP_GRAY_DIALOG[9]=40 -> CP_APP_COLOR[40]=0x7E (Yellow on LightGray)
    assert_eq!(colors[2].2, 0x7E, "Label light fg color changed!");

    // CP_LABEL[4]=9 -> same as light fg
    assert_eq!(colors[3].2, 0x7E, "Label light bg color changed!");

    // CP_LABEL[5]=13 -> CP_GRAY_DIALOG[13]=44 -> CP_APP_COLOR[44]=0x78 (DarkGray on LightGray)
    assert_eq!(colors[4].2, 0x78, "Label disabled fg color changed!");

    // CP_LABEL[6]=13 -> same as disabled fg
    assert_eq!(colors[5].2, 0x78, "Label disabled bg color changed!");
}

#[test]
fn test_static_text_palette_regression() {
    // Regression test: StaticText colors should remain stable
    // StaticText palette matches Borland cpStaticText "\x06"

    let colors = verify_static_text_colors();

    // CP_STATIC_TEXT[1]=6 -> CP_GRAY_DIALOG[6]=37 -> CP_APP_COLOR[37]=0x70 (Black on LightGray)
    assert_eq!(colors[0].2, 0x70, "StaticText normal color changed!");
}
