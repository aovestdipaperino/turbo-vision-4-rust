// (C) 2025 - Enzo Lombardi
//! The frame's zoom triangle must follow the window's size however it changed.
//!
//! The glyph used to be written only by the zoom command, so anything else that
//! resized a window — Tile, Cascade, a drag of the resize corner — left it
//! showing the wrong triangle until the next zoom toggle.

use turbo_vision::core::command::CM_ZOOM;
use turbo_vision::core::event::Event;
use turbo_vision::core::geometry::Rect;
use turbo_vision::views::View;
use turbo_vision::views::desktop::Desktop;
use turbo_vision::views::window::Window;

const DESKTOP: Rect = Rect {
    a: turbo_vision::core::geometry::Point { x: 0, y: 0 },
    b: turbo_vision::core::geometry::Point { x: 80, y: 24 },
};

fn desktop_with_windows(count: usize) -> Desktop {
    let mut desktop = Desktop::new(DESKTOP);
    for i in 0..count {
        let x = 2 + i as i16 * 4;
        desktop.add(Box::new(Window::new(
            Rect::new(x, x, x + 38, x + 10),
            "Window",
        )));
    }
    desktop
}

/// Zoom the top window through the command the frame icon sends.
fn zoom_top(desktop: &mut Desktop) {
    let mut zoom = Event::command(CM_ZOOM);
    desktop.handle_event(&mut zoom);
}

/// Every window's triangle must agree with its own size.
///
/// This is the whole point of the fix: the glyph is derived, so it can never
/// disagree with the bounds, whatever moved them.
fn assert_glyph_matches_bounds(desktop: &mut Desktop) {
    // The desktop's public indices already skip the background.
    for i in 0..desktop.child_count() {
        let child = desktop.child_at_mut(i);
        let Some(window) = child.as_any_mut().downcast_mut::<Window>() else {
            continue;
        };
        let bounds = window.bounds();
        let fills_desktop =
            bounds.width() == DESKTOP.width() && bounds.height() == DESKTOP.height();
        assert_eq!(
            window.is_zoomed(),
            fills_desktop,
            "window {i} at {bounds:?} reports the wrong zoom state"
        );
    }
}

#[test]
fn a_zoomed_window_reads_as_zoomed() {
    let mut desktop = desktop_with_windows(1);
    let child = desktop.child_at_mut(0);
    let window = child.as_any_mut().downcast_mut::<Window>().unwrap();
    assert!(!window.is_zoomed(), "starts at its own size");

    zoom_top(&mut desktop);
    assert_glyph_matches_bounds(&mut desktop);

    let child = desktop.child_at_mut(0);
    let window = child.as_any_mut().downcast_mut::<Window>().unwrap();
    assert!(window.is_zoomed(), "now fills the desktop");
}

#[test]
fn tiling_a_zoomed_window_flips_its_triangle_back() {
    let mut desktop = desktop_with_windows(2);
    zoom_top(&mut desktop);

    // Tile shrinks every window without going near the zoom command.
    desktop.tile();
    assert_glyph_matches_bounds(&mut desktop);

    // Two tiled windows split the desktop, so neither can fill it.
    for i in 0..desktop.child_count() {
        let child = desktop.child_at_mut(i);
        if let Some(window) = child.as_any_mut().downcast_mut::<Window>() {
            assert!(!window.is_zoomed(), "window {i} is tiled");
        }
    }
}

#[test]
fn cascading_keeps_every_triangle_honest() {
    let mut desktop = desktop_with_windows(3);
    zoom_top(&mut desktop);

    // Cascade lays the windows out in a staircase, each running to the
    // bottom-right corner, so the first one does still fill the desktop. What
    // matters is that each window's glyph matches its own size.
    desktop.cascade();
    assert_glyph_matches_bounds(&mut desktop);
}

#[test]
fn restoring_a_zoomed_window_flips_the_triangle_back() {
    let mut desktop = desktop_with_windows(1);
    zoom_top(&mut desktop);
    zoom_top(&mut desktop);
    assert_glyph_matches_bounds(&mut desktop);

    let child = desktop.child_at_mut(0);
    let window = child.as_any_mut().downcast_mut::<Window>().unwrap();
    assert!(!window.is_zoomed(), "back to its saved size");
}

/// Press and release the mouse on the top window's zoom icon, the way a real
/// click reaches the desktop.
fn click_zoom_icon(desktop: &mut Desktop) {
    use turbo_vision::core::event::{EventType, MB_LEFT_BUTTON};
    use turbo_vision::core::geometry::Point;

    let top = desktop.child_count() - 1;
    let bounds = desktop.child_at(top).bounds();
    // The icon is `[▲]` at width - 5; aim at the triangle itself.
    let icon = Point::new(bounds.a.x + bounds.width() - 4, bounds.a.y);

    let mut down = Event::mouse(EventType::MouseDown, icon, MB_LEFT_BUTTON, false);
    desktop.handle_event(&mut down);
    let mut up = Event::mouse(EventType::MouseUp, icon, MB_LEFT_BUTTON, false);
    desktop.handle_event(&mut up);
    assert_eq!(
        up.what,
        EventType::Nothing,
        "the zoom click must be consumed, not left for the application"
    );
}

#[test]
fn clicking_the_zoom_icon_zooms_the_window() {
    let mut desktop = desktop_with_windows(1);
    click_zoom_icon(&mut desktop);
    assert_glyph_matches_bounds(&mut desktop);

    let child = desktop.child_at_mut(0);
    let window = child.as_any_mut().downcast_mut::<Window>().unwrap();
    assert!(
        window.is_zoomed(),
        "a click on the icon must zoom the window"
    );

    click_zoom_icon(&mut desktop);
    let child = desktop.child_at_mut(0);
    let window = child.as_any_mut().downcast_mut::<Window>().unwrap();
    assert!(!window.is_zoomed(), "a second click restores it");
}
