// (C) 2026 - Enzo Lombardi
// Extras Controls Example
// Demonstrates the TV Tool Box-style value controls from
// turbo-vision-extras: ComboBox, SpinControl, Slider, and Gauge.
//
// Tab moves between controls. The combo box opens with Down or a click;
// the spinner reacts to Up/Down/PgUp/PgDn and its ▲/▼ cells; the slider
// follows Left/Right/Home/End and mouse clicks on the track.

use std::cell::RefCell;
use std::rc::Rc;
use turbo_vision::views::GroupLike;

use turbo_vision::app::Application;
use turbo_vision::core::command::CM_QUIT;
use turbo_vision::core::event::{KB_ALT_X, KB_ESC_ESC};
use turbo_vision::core::geometry::Rect;
use turbo_vision::views::static_text::StaticText;
use turbo_vision::views::status_line::{StatusItem, StatusLine};
use turbo_vision::views::window::WindowBuilder;
use turbo_vision_extras::{ComboBox, Gauge, Slider, SpinControl};

fn main() -> turbo_vision::core::error::Result<()> {
    let mut app = Application::new()?;

    let (width, height) = app.terminal.size();
    app.set_status_line(StatusLine::new(
        Rect::new(0, height - 1, width, height),
        vec![
            StatusItem::new("~Alt-X~ Exit", KB_ALT_X, CM_QUIT),
            StatusItem::new("~Esc-Esc~ Exit", KB_ESC_ESC, CM_QUIT),
        ],
    ));

    let mut window = WindowBuilder::new()
        .bounds(Rect::new(10, 3, 66, 18))
        .title("Extra Controls")
        .build();

    // SpinControl — numeric value with steppers
    window.add(StaticText::new(Rect::new(2, 5, 16, 6), "Tab size:"));
    let tab_size = Rc::new(RefCell::new(4));
    window.add(SpinControl::new(
        Rect::new(16, 5, 26, 6),
        1,
        16,
        tab_size.clone(),
    ));

    // Slider
    window.add(StaticText::new(Rect::new(2, 8, 16, 9), "Volume:"));
    let mut slider = Slider::new(Rect::new(16, 8, 50, 9), 0, 100);
    slider.set_value(65);
    slider.set_step(5);
    window.add(slider);

    // Gauge
    window.add(StaticText::new(Rect::new(2, 11, 16, 12), "Progress:"));
    let mut gauge = Gauge::new(Rect::new(16, 11, 50, 12), 100);
    gauge.set_value(65);
    window.add(gauge);

    // ComboBox — added LAST so its drop-down draws on top of the controls
    // below it (children paint in add order)
    window.add(StaticText::new(Rect::new(2, 2, 16, 3), "Theme:"));
    let theme = Rc::new(RefCell::new("Classic Blue".to_string()));
    window.add(ComboBox::new(
        Rect::new(16, 2, 42, 3),
        vec![
            "Classic Blue".into(),
            "Monochrome".into(),
            "Solarized".into(),
            "High Contrast".into(),
        ],
        theme.clone(),
    ));

    app.desktop.add(window);
    app.run();

    // Shared values are read back after the app exits
    println!("Theme:    {}", theme.borrow());
    println!("Tab size: {}", tab_size.borrow());

    Ok(())
}
