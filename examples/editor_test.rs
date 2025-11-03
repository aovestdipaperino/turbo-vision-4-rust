// Minimal editor test to diagnose crash
use turbo_vision::app::Application;
use turbo_vision::core::command::CM_CANCEL;
use turbo_vision::core::geometry::Rect;
use turbo_vision::views::dialog::Dialog;
use turbo_vision::views::editor::Editor;
use turbo_vision::views::button::Button;

fn main() -> std::io::Result<()> {
    let mut app = Application::new()?;
    let (width, height) = app.terminal.size();

    let dialog_width = 60.min(width as i16 - 4);
    let dialog_height = 20.min(height as i16 - 2);
    let dialog_x = ((width as i16 - dialog_width) / 2).max(0);
    let dialog_y = ((height as i16 - dialog_height) / 2).max(0);

    let mut dialog = Dialog::new(
        Rect::new(dialog_x, dialog_y, dialog_x + dialog_width, dialog_y + dialog_height),
        "Editor Test"
    );

    let editor_bounds = Rect::new(2, 2, dialog_width - 2, dialog_height - 3);
    let mut editor = Editor::new(editor_bounds);

    // Simple ASCII text first
    editor.set_text("Hello World\nSecond Line\nThird Line");

    dialog.add(Box::new(editor));

    let button_y = dialog_height - 2;
    let close_button = Button::new(
        Rect::new((dialog_width / 2) - 5, button_y, (dialog_width / 2) + 5, button_y + 2),
        "Close",
        CM_CANCEL,
        false
    );
    dialog.add(Box::new(close_button));

    dialog.set_initial_focus();
    let _result = dialog.execute(&mut app);

    Ok(())
}
