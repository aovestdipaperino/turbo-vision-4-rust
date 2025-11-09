// (C) 2025 - Enzo Lombardi

//! Regression tests for palette colors to ensure they remain stable across changes.
//! These tests verify that the color mapping for key UI elements produces the expected
//! final color attributes when using the default palette system.

#[cfg(test)]
mod tests {
    use crate::core::geometry::{Point, Rect};
    use crate::core::menu_data::{Menu, MenuItem};
    use crate::core::palette::palettes;
    use crate::views::button::Button;
    use crate::views::dialog::Dialog;
    use crate::views::input_line::InputLine;
    use crate::views::menu_bar::MenuBar;
    use crate::views::menu_box::MenuBox;
    use crate::views::scrollbar::ScrollBar;
    use crate::views::view::View;

    /// Helper to create a test button and verify its color mapping
    fn verify_button_colors() -> Vec<(String, u8, u8)> {
        let button = Button::new(Rect::new(0, 0, 10, 3), "Test", 100, false);
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

        // MenuBar palette indices: [2, 39, 3, 4]
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
        let input_line = InputLine::new(Rect::new(0, 0, 20, 1), 100, data);
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
        scrollbar.set_owner_type(crate::views::view::OwnerType::Window);
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

    #[test]
    fn test_button_palette_regression() {
        // Regression test: Button colors should remain stable
        // These are the expected final color values after palette remapping

        let colors = verify_button_colors();

        // Expected values based on current implementation:
        // CP_BUTTON[1]=12 -> CP_GRAY_DIALOG[12]=43 -> CP_APP_COLOR[43]=0x20 (Black on Green)
        assert_eq!(colors[0].2, 0x20, "Button normal color changed!");

        // CP_BUTTON[2]=12 -> same as normal
        assert_eq!(colors[1].2, 0x20, "Button default color changed!");

        // CP_BUTTON[3]=13 -> CP_GRAY_DIALOG[13]=44 -> CP_APP_COLOR[44]=0x2A (LightGreen on Green)
        assert_eq!(colors[2].2, 0x2A, "Button focused color changed!");

        // CP_BUTTON[4]=13 -> same as focused (for disabled state)
        assert_eq!(colors[3].2, 0x2A, "Button disabled color changed!");

        // CP_BUTTON[5]=15 -> CP_GRAY_DIALOG[15]=46 -> CP_APP_COLOR[46]=0x1F (White on Blue)
        assert_eq!(colors[4].2, 0x1F, "Button reserved5 color changed!");

        // CP_BUTTON[6]=14 -> CP_GRAY_DIALOG[14]=45 -> CP_APP_COLOR[45]=0x2F (White on Green)
        assert_eq!(colors[5].2, 0x2F, "Button shortcut color changed!");

        // CP_BUTTON[7]=14 -> same as shortcut
        assert_eq!(colors[6].2, 0x2F, "Button reserved7 color changed!");

        // CP_BUTTON[8]=9 -> CP_GRAY_DIALOG[9]=40 -> CP_APP_COLOR[40]=0x87 (LightGray on DarkGray)
        assert_eq!(colors[7].2, 0x87, "Button shadow color changed!");
    }

    #[test]
    fn test_menu_palette_regression() {
        // Regression test: Menu colors should remain stable
        // MenuBar uses direct app indices (no dialog remapping)

        let colors = verify_menu_colors();

        // MenuBar colors:
        // CP_MENU_BAR[1]=2 -> CP_APP_COLOR[2]=0x70 (Black on LightGray)
        assert_eq!(colors[0].2, 0x70, "MenuBar normal color changed!");

        // CP_MENU_BAR[2]=39 -> CP_APP_COLOR[39]=0x2F (White on Green)
        assert_eq!(colors[1].2, 0x2F, "MenuBar selected color changed!");

        // CP_MENU_BAR[3]=3 -> CP_APP_COLOR[3]=0x78 (DarkGray on LightGray)
        assert_eq!(colors[2].2, 0x78, "MenuBar disabled color changed!");

        // CP_MENU_BAR[4]=4 -> CP_APP_COLOR[4]=0x74 (Red on LightGray)
        assert_eq!(colors[3].2, 0x74, "MenuBar shortcut color changed!");

        // MenuBox should have the same colors as MenuBar
        assert_eq!(colors[4].2, 0x70, "MenuBox normal color changed!");
        assert_eq!(colors[5].2, 0x2F, "MenuBox selected color changed!");
        assert_eq!(colors[6].2, 0x78, "MenuBox disabled color changed!");
        assert_eq!(colors[7].2, 0x74, "MenuBox shortcut color changed!");
    }

    #[test]
    fn test_dialog_palette_regression() {
        // Regression test: Dialog colors should remain stable
        // Dialog uses CP_GRAY_DIALOG which maps indices 1-32 to app indices 32-63

        let colors = verify_dialog_colors();

        // Dialog palette remapping: Dialog itself returns dialog palette indices
        // Dialog[1] = 0x13
        assert_eq!(colors[0].2, 0x13, "Dialog frame color changed!");

        // Dialog[2] = 0x70
        assert_eq!(colors[1].2, 0x70, "Dialog frame active color changed!");

        // Dialog[3] = 0x74 (changed to support ScrollBar in Dialog)
        assert_eq!(colors[2].2, 0x74, "Dialog interior color changed!");

        // Dialog[4] = 0x74 (changed for consistency)
        assert_eq!(colors[3].2, 0x74, "Dialog text color changed!");

        // Dialog[5] = 0x7E (unchanged)
        assert_eq!(colors[4].2, 0x7E, "Dialog selected color changed!");

        // Dialog[6] = 0x20
        assert_eq!(colors[5].2, 0x20, "Dialog reserved6 color changed!");

        // Dialog[7] = 0x2B
        assert_eq!(colors[6].2, 0x2B, "Dialog label normal color changed!");

        // Dialog[8] = 0x2F
        assert_eq!(colors[7].2, 0x2F, "Dialog label selected color changed!");

        // Dialog[9] = 0x87
        assert_eq!(colors[8].2, 0x87, "Dialog label shortcut color changed!");
    }

    #[test]
    fn test_input_line_palette_regression() {
        // Regression test: InputLine colors should remain stable
        // These are the expected final color values after palette remapping

        let colors = verify_input_line_colors();

        // InputLine colors:
        // CP_INPUT_LINE[1]=19 -> CP_GRAY_DIALOG[19]=50 -> CP_APP_COLOR[50]=0x1E (Yellow on Blue)
        assert_eq!(colors[0].2, 0x1E, "InputLine passive color changed!");

        // CP_INPUT_LINE[2]=19 -> same as passive
        assert_eq!(colors[1].2, 0x1E, "InputLine active color changed!");

        // CP_INPUT_LINE[3]=20 -> CP_GRAY_DIALOG[20]=51 -> CP_APP_COLOR[51]=0x1F (White on Blue)
        assert_eq!(colors[2].2, 0x1F, "InputLine selected color changed!");

        // CP_INPUT_LINE[4]=21 -> CP_GRAY_DIALOG[21]=52 -> CP_APP_COLOR[52]=0x3E (Yellow on Cyan)
        assert_eq!(colors[3].2, 0x3E, "InputLine arrow color changed!");
    }

    #[test]
    fn test_scrollbar_palette_regression() {
        // Regression test: ScrollBar colors should remain stable
        // ScrollBar uses direct app palette indices (no dialog remapping)

        let colors = verify_scrollbar_colors();

        // ScrollBar colors:
        // CP_SCROLLBAR[1]=4 -> CP_APP_COLOR[4]=0x71 (Blue on LightGray)
        assert_eq!(colors[0].2, 0x71, "ScrollBar page color changed!");

        // CP_SCROLLBAR[2]=5 -> CP_APP_COLOR[5]=0x71 (Blue on LightGray)
        assert_eq!(colors[1].2, 0x71, "ScrollBar arrows color changed!");

        // CP_SCROLLBAR[3]=5 -> CP_APP_COLOR[5]=0x71 (Blue on LightGray)
        assert_eq!(colors[2].2, 0x71, "ScrollBar indicator color changed!");
    }

    #[test]
    fn test_scrollbar_context_colors() {
        // Test that ScrollBar shows different colors in Window vs Dialog contexts

        // Test ScrollBar in Window context (should be blue on gray)
        let mut scrollbar = ScrollBar::new_vertical(Rect::new(0, 0, 1, 10));
        scrollbar.set_owner_type(crate::views::view::OwnerType::Window);

        let window_color = scrollbar.map_color(1).to_u8();
        assert_eq!(
            window_color, 0x71,
            "ScrollBar in Window should be blue on gray"
        );

        // Test ScrollBar in Dialog context (should be red on gray)
        let mut scrollbar = ScrollBar::new_vertical(Rect::new(0, 0, 1, 10));
        scrollbar.set_owner_type(crate::views::view::OwnerType::Dialog);

        let dialog_color = scrollbar.map_color(1).to_u8();
        assert_eq!(
            dialog_color, 0x74,
            "ScrollBar in Dialog should be red on gray"
        );
    }

    #[test]
    fn test_palette_remapping_ranges() {
        // Test that the remapping ranges work as expected

        // Create a button to test dialog control remapping
        let button = Button::new(Rect::new(0, 0, 10, 3), "Test", 100, false);

        // Test that button colors are mapped correctly (already tested above in detail)
        // Just verify a few key indices to ensure remapping works
        assert_eq!(button.map_color(1).to_u8(), 0x20); // Normal button color
        assert_eq!(button.map_color(3).to_u8(), 0x2A); // Focused button color
        assert_eq!(button.map_color(8).to_u8(), 0x87); // Shadow color

        // Create a menu bar to test top-level view (no remapping for 1-8)
        let menu_bar = MenuBar::new(Rect::new(0, 0, 80, 1));

        // Test that indices 1-8 don't get remapped for top-level views
        for index in 1..=8 {
            let color = menu_bar.map_color(index);
            // MenuBar palette maps these, but then they should go direct to app
            let menu_mapped = match index {
                1 => 2,     // normal
                2 => 39,    // selected
                3 => 3,     // disabled
                4 => 4,     // shortcut
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
}
