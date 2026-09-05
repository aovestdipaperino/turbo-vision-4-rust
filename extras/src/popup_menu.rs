// (C) 2026 - Enzo Lombardi

//! Popup (context) menus and check-mark menu items (TV Tool Box style).

use turbo_vision::core::command::CommandId;
use turbo_vision::core::geometry::Point;
use turbo_vision::core::menu_data::{Menu, MenuItem};
use turbo_vision::terminal::Terminal;
use turbo_vision::views::MenuBox;

/// Prefix marking a checked menu item.
const CHECK_PREFIX: &str = "✓ ";
/// Prefix keeping unchecked items aligned with checked ones.
const UNCHECK_PREFIX: &str = "  ";

/// Run `menu` as a modal popup at `position` (e.g. the mouse location).
///
/// Returns the selected command, or `None` when the menu was dismissed.
/// This is the context-menu entry point the classic add-on kits provided:
/// it reuses the framework's `MenuBox` so navigation, accelerators, and
/// drawing all match drop-down menus.
///
/// # Examples
///
/// ```ignore
/// use turbo_vision::core::menu_data::{Menu, MenuItem};
/// use turbo_vision_extras::popup_menu;
///
/// let menu = Menu::from_items(vec![
///     MenuItemBuilder::new().text("~C~opy").command(21).build(),
///     MenuItemBuilder::new().text("~P~aste").command(22).build(),
/// ]);
/// if let Some(cmd) = popup_menu(&mut terminal, event.mouse.pos, menu) {
///     // dispatch cmd
/// }
/// ```
pub fn popup_menu(terminal: &mut Terminal, position: Point, menu: Menu) -> Option<CommandId> {
    let mut menu_box = MenuBox::new(position, menu);
    match menu_box.execute(terminal) {
        0 => None,
        command => Some(command),
    }
}

/// Set or clear the check mark on a menu item (TV Tool Box check-mark menus).
///
/// Checked items are rendered with a leading `✓`; unchecked items get a
/// two-space prefix so captions stay aligned. Separators and submenus are
/// left untouched.
pub fn set_menu_item_checked(menu: &mut Menu, index: usize, checked: bool) {
    if let Some(MenuItem::Regular { text, .. }) = menu.items.get_mut(index) {
        let bare = text
            .strip_prefix(CHECK_PREFIX)
            .or_else(|| text.strip_prefix(UNCHECK_PREFIX))
            .unwrap_or(text)
            .to_string();
        *text = if checked {
            format!("{CHECK_PREFIX}{bare}")
        } else {
            format!("{UNCHECK_PREFIX}{bare}")
        };
    }
}

/// True when the menu item at `index` carries a check mark.
pub fn is_menu_item_checked(menu: &Menu, index: usize) -> bool {
    matches!(
        menu.items.get(index),
        Some(MenuItem::Regular { text, .. }) if text.starts_with(CHECK_PREFIX)
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use turbo_vision::core::menu_data::MenuItemBuilder;

    fn menu() -> Menu {
        Menu::from_items(vec![
            MenuItemBuilder::new()
                .text("Word wrap")
                .command(200)
                .build(),
            MenuItem::separator(),
            MenuItemBuilder::new()
                .text("Line numbers")
                .command(201)
                .build(),
        ])
    }

    #[test]
    fn check_toggle_round_trips() {
        let mut m = menu();
        assert!(!is_menu_item_checked(&m, 0));

        set_menu_item_checked(&mut m, 0, true);
        assert!(is_menu_item_checked(&m, 0));
        assert_eq!(m.items[0].text(), "✓ Word wrap");

        // Re-checking doesn't stack prefixes
        set_menu_item_checked(&mut m, 0, true);
        assert_eq!(m.items[0].text(), "✓ Word wrap");

        set_menu_item_checked(&mut m, 0, false);
        assert!(!is_menu_item_checked(&m, 0));
        assert_eq!(m.items[0].text(), "  Word wrap");
    }

    #[test]
    fn separators_are_ignored() {
        let mut m = menu();
        set_menu_item_checked(&mut m, 1, true);
        assert!(!is_menu_item_checked(&m, 1));
    }
}
