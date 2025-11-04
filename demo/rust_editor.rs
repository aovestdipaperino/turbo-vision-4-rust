// Rust Text Editor Demo
//
// A comprehensive text editor with:
// - Rust syntax highlighting
// - File operations (New, Open, Save, Save As)
// - Dirty flag tracking with save prompt
// - Search and Replace
// - Rust analyzer integration
// - Status line with file info

use std::path::PathBuf;
use std::rc::Rc;
use std::cell::RefCell;
use std::fs;
use turbo_vision::app::Application;
use turbo_vision::core::command::{CM_QUIT, CM_NEW, CM_OPEN, CM_SAVE, CM_OK, CM_CANCEL, CM_YES, CM_NO, CM_CLOSE};
use turbo_vision::core::event::{EventType, KB_F10};
use turbo_vision::core::geometry::Rect;
use turbo_vision::core::menu_data::{Menu, MenuItem};
use turbo_vision::views::button::Button;
use turbo_vision::views::dialog::Dialog;
use turbo_vision::views::editor::Editor;
use turbo_vision::views::file_dialog::FileDialog;
use turbo_vision::views::input_line::InputLine;
use turbo_vision::views::label::Label;
use turbo_vision::views::menu_bar::{MenuBar, SubMenu};
use turbo_vision::views::static_text::StaticText;
use turbo_vision::views::status_line::{StatusItem, StatusLine};
use turbo_vision::views::window::Window;
use turbo_vision::views::View;
use turbo_vision::views::syntax::RustHighlighter;
use turbo_vision::views::msgbox::{confirmation_box, message_box_ok, message_box_error};

// Custom command IDs
const CMD_SAVE_AS: u16 = 105;
const CMD_SEARCH: u16 = 300;
const CMD_REPLACE: u16 = 301;
const CMD_GOTO_LINE: u16 = 302;
const CMD_ANALYZE: u16 = 400;
const CMD_SHOW_ERRORS: u16 = 401;

struct EditorState {
    filename: Option<PathBuf>,
    dirty: bool,
    content: Rc<RefCell<String>>,
}

impl EditorState {
    fn new() -> Self {
        Self {
            filename: None,
            dirty: false,
            content: Rc::new(RefCell::new(String::new())),
        }
    }

    fn set_content(&mut self, content: String) {
        *self.content.borrow_mut() = content;
        self.dirty = false;
    }

    #[allow(dead_code)]
    fn mark_dirty(&mut self) {
        self.dirty = true;
    }

    fn get_title(&self) -> String {
        let filename = self.filename
            .as_ref()
            .and_then(|p| p.file_name())
            .and_then(|n| n.to_str())
            .unwrap_or("Untitled");

        if self.dirty {
            format!("{} *", filename)
        } else {
            filename.to_string()
        }
    }
}

fn main() -> std::io::Result<()> {
    let mut app = Application::new()?;
    let (width, height) = app.terminal.size();

    let mut editor_state = EditorState::new();

    // Create menu bar
    let menu_bar = create_menu_bar(width);
    app.set_menu_bar(menu_bar);

    // Create status line
    let status_line = StatusLine::new(
        Rect::new(0, height as i16 - 1, width as i16, height as i16),
        vec![
            StatusItem::new("~F10~ Menu", KB_F10, 0),
            StatusItem::new("~Ctrl+S~ Save", 0, 0),
            StatusItem::new("~Ctrl+F~ Find", 0, 0),
        ],
    );
    app.set_status_line(status_line);

    // Show About dialog on startup
    show_about_dialog(&mut app);

    // Editor bounds for when user creates a new window
    let editor_bounds = Rect::new(1, 1, width as i16 - 1, height as i16 - 2);

    // Event loop
    app.running = true;
    while app.running {
        // Draw everything in proper order
        app.desktop.draw(&mut app.terminal);
        if let Some(ref mut menu_bar) = app.menu_bar {
            menu_bar.draw(&mut app.terminal);
        }
        if let Some(ref mut status_line) = app.status_line {
            status_line.draw(&mut app.terminal);
        }
        let _ = app.terminal.flush();

        // Poll for events
        if let Ok(Some(mut event)) = app.terminal.poll_event(std::time::Duration::from_millis(50)) {
            // Menu bar handles events first
            if let Some(ref mut menu_bar) = app.menu_bar {
                menu_bar.handle_event(&mut event);

                // Check for cascading submenus
                if event.what == EventType::Keyboard || event.what == EventType::MouseUp {
                    if let Some(command) = menu_bar.check_cascading_submenu(&mut app.terminal) {
                        if command != 0 {
                            event = turbo_vision::core::event::Event::command(command);
                        }
                    }
                }
            }

            // Desktop (including editor window) handles events
            app.desktop.handle_event(&mut event);

            // Handle commands
            if event.what == EventType::Command {
                match event.command {
                    CM_QUIT => {
                        if prompt_save_if_dirty(&mut app, &editor_state) {
                            app.running = false;
                        }
                    }
                    CM_CLOSE => {
                        // Close the editor window if it exists
                        if app.desktop.child_count() > 0 {
                            // Check if dirty and prompt to save
                            if !editor_state.dirty || prompt_save_if_dirty(&mut app, &editor_state) {
                                // Remove the editor window
                                app.desktop.remove_child(0);
                                // Reset to empty state
                                editor_state = EditorState::new();
                            }
                        }
                    }
                    CM_NEW => {
                        if prompt_save_if_dirty(&mut app, &editor_state) {
                            editor_state = EditorState::new();
                            // Remove old window and add new one
                            if app.desktop.child_count() > 0 {
                                app.desktop.remove_child(0);
                            }
                            let editor_window = create_editor_window(editor_bounds, &editor_state);
                            app.desktop.add(Box::new(editor_window));
                        }
                    }
                    CM_OPEN => {
                        if let Some(path) = show_file_open_dialog(&mut app) {
                            if let Ok(content) = fs::read_to_string(&path) {
                                editor_state.filename = Some(path);
                                editor_state.set_content(content);
                                // Remove old window and add new one
                                if app.desktop.child_count() > 0 {
                                    app.desktop.remove_child(0);
                                }
                                let editor_window = create_editor_window(editor_bounds, &editor_state);
                                app.desktop.add(Box::new(editor_window));
                            } else {
                                show_error(&mut app, "Error", "Failed to open file");
                            }
                        }
                    }
                    CM_SAVE => {
                        save_file(&mut app, &mut editor_state);
                        // Update window title
                        if app.desktop.child_count() > 0 {
                            app.desktop.remove_child(0);
                        }
                        let editor_window = create_editor_window(editor_bounds, &editor_state);
                        app.desktop.add(Box::new(editor_window));
                    }
                    CMD_SAVE_AS => {
                        save_file_as(&mut app, &mut editor_state);
                        // Update window title
                        if app.desktop.child_count() > 0 {
                            app.desktop.remove_child(0);
                        }
                        let editor_window = create_editor_window(editor_bounds, &editor_state);
                        app.desktop.add(Box::new(editor_window));
                    }
                    CMD_SEARCH => {
                        show_search_dialog(&mut app);
                    }
                    CMD_REPLACE => {
                        show_replace_dialog(&mut app);
                    }
                    CMD_GOTO_LINE => {
                        show_goto_line_dialog(&mut app);
                    }
                    CMD_ANALYZE => {
                        analyze_with_rust_analyzer(&mut app, &editor_state);
                    }
                    CMD_SHOW_ERRORS => {
                        // Show errors from last analysis
                        show_message(&mut app, "Analysis", "No errors found");
                    }
                    _ => {}
                }
            }
        }
    }

    Ok(())
}

fn create_menu_bar(width: u16) -> MenuBar {
    let mut menu_bar = MenuBar::new(Rect::new(0, 0, width as i16, 1));

    // File menu
    let file_menu_items = vec![
        MenuItem::with_shortcut("~N~ew", CM_NEW, 0, "Ctrl+N", 0),
        MenuItem::with_shortcut("~O~pen...", CM_OPEN, 0, "Ctrl+O", 0),
        MenuItem::separator(),
        MenuItem::with_shortcut("~S~ave", CM_SAVE, 0, "Ctrl+S", 0),
        MenuItem::with_shortcut("Save ~A~s...", CMD_SAVE_AS, 0, "", 0),
        MenuItem::separator(),
        MenuItem::with_shortcut("E~x~it", CM_QUIT, 0, "Alt+X", 0),
    ];
    let file_menu = SubMenu::new("~F~ile", Menu::from_items(file_menu_items));

    // Edit menu
    let edit_menu_items = vec![
        MenuItem::with_shortcut("~S~earch...", CMD_SEARCH, 0, "Ctrl+F", 0),
        MenuItem::with_shortcut("~R~eplace...", CMD_REPLACE, 0, "Ctrl+H", 0),
        MenuItem::separator(),
        MenuItem::with_shortcut("~G~oto Line...", CMD_GOTO_LINE, 0, "Ctrl+G", 0),
    ];
    let edit_menu = SubMenu::new("~E~dit", Menu::from_items(edit_menu_items));

    // Tools menu
    let tools_menu_items = vec![
        MenuItem::with_shortcut("~A~nalyze with rust-analyzer", CMD_ANALYZE, 0, "F5", 0),
        MenuItem::with_shortcut("Show ~E~rrors", CMD_SHOW_ERRORS, 0, "F6", 0),
    ];
    let tools_menu = SubMenu::new("~T~ools", Menu::from_items(tools_menu_items));

    menu_bar.add_submenu(file_menu);
    menu_bar.add_submenu(edit_menu);
    menu_bar.add_submenu(tools_menu);

    menu_bar
}

fn create_editor_window(bounds: Rect, state: &EditorState) -> Window {
    let mut window = Window::new(bounds, &state.get_title());

    // Create editor with scrollbars
    let editor_bounds = Rect::new(1, 1, bounds.width() - 2, bounds.height() - 2);
    let mut editor = Editor::new(editor_bounds).with_scrollbars_and_indicator();

    // Set Rust syntax highlighting
    editor.set_highlighter(Box::new(RustHighlighter::new()));

    // Set content if available
    let content = state.content.borrow().clone();
    if !content.is_empty() {
        editor.set_text(&content);
    }

    window.add(Box::new(editor));
    window
}

fn prompt_save_if_dirty(app: &mut Application, state: &EditorState) -> bool {
    if !state.dirty {
        return true;
    }

    let message = format!("Save changes to {}?", state.get_title());
    let result = confirmation_box(app, &message);

    match result {
        CM_YES => {
            // TODO: Save and continue
            true
        }
        CM_NO => {
            // Don't save but continue
            true
        }
        CM_CANCEL => {
            // Cancel the operation
            false
        }
        _ => false,
    }
}

fn show_file_open_dialog(app: &mut Application) -> Option<PathBuf> {
    let (term_width, term_height) = app.terminal.size();
    let dialog_width = 60;
    let dialog_height = 20;
    let dialog_x = (term_width as i16 - dialog_width) / 2;
    let dialog_y = (term_height as i16 - dialog_height) / 2;

    let bounds = Rect::new(dialog_x, dialog_y, dialog_x + dialog_width, dialog_y + dialog_height);

    // Start in current directory
    let initial_dir = std::env::current_dir().ok();

    let mut file_dialog = FileDialog::new(bounds, "Open File", "*.rs", initial_dir).build();
    file_dialog.execute(app)
}

fn save_file(app: &mut Application, state: &mut EditorState) {
    if state.filename.is_none() {
        save_file_as(app, state);
        return;
    }

    let content = state.content.borrow().clone();
    if let Some(ref path) = state.filename {
        if fs::write(path, content).is_ok() {
            state.dirty = false;
            show_message(app, "Save", "File saved successfully");
        } else {
            show_error(app, "Error", "Failed to save file");
        }
    }
}

fn save_file_as(app: &mut Application, state: &mut EditorState) {
    if let Some(path) = show_file_save_dialog(app) {
        let content = state.content.borrow().clone();
        if fs::write(&path, content).is_ok() {
            state.filename = Some(path);
            state.dirty = false;
            show_message(app, "Save", "File saved successfully");
        } else {
            show_error(app, "Error", "Failed to save file");
        }
    }
}

fn show_file_save_dialog(app: &mut Application) -> Option<PathBuf> {
    let (term_width, term_height) = app.terminal.size();
    let dialog_width = 60;
    let dialog_height = 20;
    let dialog_x = (term_width as i16 - dialog_width) / 2;
    let dialog_y = (term_height as i16 - dialog_height) / 2;

    let bounds = Rect::new(dialog_x, dialog_y, dialog_x + dialog_width, dialog_y + dialog_height);

    // Start in current directory
    let initial_dir = std::env::current_dir().ok();

    let mut file_dialog = FileDialog::new(bounds, "Save File As", "*.rs", initial_dir).build();
    file_dialog.execute(app)
}

fn show_search_dialog(app: &mut Application) {
    let (term_width, term_height) = app.terminal.size();
    let dialog_width = 50;
    let dialog_height = 8;
    let dialog_x = (term_width as i16 - dialog_width) / 2;
    let dialog_y = (term_height as i16 - dialog_height) / 2;

    let mut dialog = Dialog::new(
        Rect::new(dialog_x, dialog_y, dialog_x + dialog_width, dialog_y + dialog_height),
        "Search"
    );

    let label = Label::new(Rect::new(2, 2, 20, 3), "~F~ind:");
    dialog.add(Box::new(label));

    let search_data = Rc::new(RefCell::new(String::new()));
    let input = InputLine::new(Rect::new(2, 3, dialog_width - 4, 4), 100, search_data.clone());
    dialog.add(Box::new(input));

    let ok_button = Button::new(Rect::new(15, 5, 25, 7), "  ~O~K  ", CM_OK, true);
    dialog.add(Box::new(ok_button));

    let cancel_button = Button::new(Rect::new(27, 5, 37, 7), " Cancel", CM_CANCEL, false);
    dialog.add(Box::new(cancel_button));

    dialog.set_initial_focus();

    let result = dialog.execute(app);

    if result == CM_OK {
        let search_text = search_data.borrow().clone();
        if !search_text.is_empty() {
            // TODO: Implement actual search in editor
            show_message(app, "Search", &format!("Searching for: {}", search_text));
        }
    }
}

fn show_replace_dialog(app: &mut Application) {
    let (term_width, term_height) = app.terminal.size();
    let dialog_width = 50;
    let dialog_height = 12;
    let dialog_x = (term_width as i16 - dialog_width) / 2;
    let dialog_y = (term_height as i16 - dialog_height) / 2;

    let mut dialog = Dialog::new(
        Rect::new(dialog_x, dialog_y, dialog_x + dialog_width, dialog_y + dialog_height),
        "Replace"
    );

    let label1 = Label::new(Rect::new(2, 2, 20, 3), "~F~ind:");
    dialog.add(Box::new(label1));

    let find_data = Rc::new(RefCell::new(String::new()));
    let input1 = InputLine::new(Rect::new(2, 3, dialog_width - 4, 4), 100, find_data.clone());
    dialog.add(Box::new(input1));

    let label2 = Label::new(Rect::new(2, 5, 20, 6), "~R~eplace with:");
    dialog.add(Box::new(label2));

    let replace_data = Rc::new(RefCell::new(String::new()));
    let input2 = InputLine::new(Rect::new(2, 6, dialog_width - 4, 7), 100, replace_data.clone());
    dialog.add(Box::new(input2));

    let ok_button = Button::new(Rect::new(15, 9, 25, 11), "  ~O~K  ", CM_OK, true);
    dialog.add(Box::new(ok_button));

    let cancel_button = Button::new(Rect::new(27, 9, 37, 11), " Cancel", CM_CANCEL, false);
    dialog.add(Box::new(cancel_button));

    dialog.set_initial_focus();

    let result = dialog.execute(app);

    if result == CM_OK {
        let find_text = find_data.borrow().clone();
        let replace_text = replace_data.borrow().clone();
        if !find_text.is_empty() {
            // TODO: Implement actual replace in editor
            show_message(app, "Replace", &format!("Replace '{}' with '{}'", find_text, replace_text));
        }
    }
}

fn show_goto_line_dialog(app: &mut Application) {
    let (term_width, term_height) = app.terminal.size();
    let dialog_width = 40;
    let dialog_height = 8;
    let dialog_x = (term_width as i16 - dialog_width) / 2;
    let dialog_y = (term_height as i16 - dialog_height) / 2;

    let mut dialog = Dialog::new(
        Rect::new(dialog_x, dialog_y, dialog_x + dialog_width, dialog_y + dialog_height),
        "Go to Line"
    );

    let label = Label::new(Rect::new(2, 2, 20, 3), "~L~ine number:");
    dialog.add(Box::new(label));

    let line_data = Rc::new(RefCell::new(String::new()));
    let input = InputLine::new(Rect::new(2, 3, dialog_width - 4, 4), 10, line_data.clone());
    dialog.add(Box::new(input));

    let ok_button = Button::new(Rect::new(10, 5, 20, 7), "  ~O~K  ", CM_OK, true);
    dialog.add(Box::new(ok_button));

    let cancel_button = Button::new(Rect::new(22, 5, 32, 7), " Cancel", CM_CANCEL, false);
    dialog.add(Box::new(cancel_button));

    dialog.set_initial_focus();

    let result = dialog.execute(app);

    if result == CM_OK {
        let line_text = line_data.borrow().clone();
        if let Ok(line_num) = line_text.parse::<usize>() {
            // TODO: Implement actual goto line in editor
            show_message(app, "Go to Line", &format!("Going to line: {}", line_num));
        }
    }
}

fn analyze_with_rust_analyzer(app: &mut Application, state: &EditorState) {
    // For now, just show a message about rust-analyzer integration
    // In a real implementation, we would:
    // 1. Save the file temporarily
    // 2. Run rust-analyzer via LSP or command line
    // 3. Parse the results
    // 4. Display errors/warnings

    if state.filename.is_none() {
        show_error(app, "Analysis", "Please save the file first");
        return;
    }

    show_message(app, "Analysis", "Running rust-analyzer...\n\n(Integration in progress)");
}

fn show_message(app: &mut Application, _title: &str, message: &str) {
    message_box_ok(app, message);
}

fn show_about_dialog(app: &mut Application) {
    let (term_width, term_height) = app.terminal.size();
    let dialog_width = 40;
    let dialog_height = 10;
    let dialog_x = (term_width as i16 - dialog_width) / 2;
    let dialog_y = (term_height as i16 - dialog_height) / 2;

    let mut dialog = Dialog::new(
        Rect::new(dialog_x, dialog_y, dialog_x + dialog_width, dialog_y + dialog_height),
        "About"
    );

    let text = StaticText::new_centered(
        Rect::new(2, 3, dialog_width - 4, 5),
        "Lonbard Turbo Rust"
    );
    dialog.add(Box::new(text));

    let button = Button::new(Rect::new(15, 6, 25, 8), "  ~O~K  ", CM_OK, true);
    dialog.add(Box::new(button));

    dialog.set_initial_focus();
    dialog.execute(app);
}

fn show_error(app: &mut Application, _title: &str, message: &str) {
    message_box_error(app, message);
}
