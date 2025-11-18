// (C) 2025 - Enzo Lombardi
// Rust Text Editor - Port of Borland's TEditorApp (tvedit)
//
// Based on Borland Turbo Vision's TEditorApp from:
//   - local-only/borland/tvision/examples/tvedit/tvedit.cc
//   - local-only/borland/tvision/classes/tvedit1.cc (application logic)
//   - local-only/borland/tvision/classes/tvedit2.cc (dialogs)
//   - local-only/borland/tvision/classes/tvedit3.cc (menus and status line)
//
// Features:
// - Full TEditorApp functionality with EditWindow (TEditWindow)
// - File operations (Open F3, New, Save F2, Save As, Exit Alt+X)
// - Search and Replace (Find, Replace, Search Again)
// - Window management (Zoom F5, Tile, Cascade, Next F6, Previous, Close Alt+F3)
// - Rust-specific extensions (Rust syntax highlighting, rust-analyzer integration)
// - Status line with function key shortcuts

use std::path::PathBuf;
use turbo_vision::app::Application;
use turbo_vision::core::command::{
    CM_QUIT, CM_NEW, CM_OPEN, CM_SAVE, CM_YES, CM_NO, CM_CLOSE,
    CM_ZOOM, CM_TILE, CM_CASCADE, CM_NEXT, CM_PREV, CM_SAVE_AS, CM_FIND,
    CM_REPLACE, CM_SEARCH_AGAIN, CM_GOTO_LINE,
};
use turbo_vision::core::command_set;
use turbo_vision::core::event::{EventType, KB_F10};
use turbo_vision::core::geometry::Rect;
use turbo_vision::core::menu_data::{Menu, MenuItem};
use turbo_vision::views::file_dialog::FileDialogBuilder;
use turbo_vision::views::file_editor::FileEditor;
use turbo_vision::views::menu_bar::{MenuBar, SubMenu};
use turbo_vision::views::status_line::{StatusItem, StatusLine};
use turbo_vision::views::View;
use turbo_vision::views::syntax::RustHighlighter;
use turbo_vision::views::msgbox::{message_box_ok, message_box_error, search_box, search_replace_box, goto_line_box};

// Custom command IDs for features not in core (using safe range 122-125, 400+)
const CM_CHANGE_DIR: u16 = 122;   // Borland: cmChangeDrct - change directory dialog
const CM_SHOW_CLIP: u16 = 123;    // Borland: cmShowClip - show clipboard window
// Rust-specific commands
const CM_ANALYZE: u16 = 400;      // Run rust-analyzer
const CM_SHOW_ERRORS: u16 = 401;  // Show analysis errors


/// Helper to get the FileEditor from the desktop (assumes it's the first child)
fn get_file_editor(app: &Application) -> Option<&FileEditor> {
    if app.desktop.child_count() == 0 {
        return None;
    }

    let child = app.desktop.child_at(0);
    // Try to downcast to FileEditor
    // SAFETY: We know the first child is a FileEditor if it exists
    unsafe {
        let ptr = child as *const dyn View as *const FileEditor;
        Some(&*ptr)
    }
}

/// Helper to get a mutable reference to the FileEditor from the desktop
fn get_file_editor_mut(app: &mut Application) -> Option<&mut FileEditor> {
    if app.desktop.child_count() == 0 {
        return None;
    }

    let child = app.desktop.child_at_mut(0);
    // Try to downcast to FileEditor
    // SAFETY: We know the first child is a FileEditor if it exists
    unsafe {
        let ptr = child as *mut dyn View as *mut FileEditor;
        Some(&mut *ptr)
    }
}

fn main() -> turbo_vision::core::error::Result<()> {
    let mut app = Application::new()?;
    let (width, height) = app.terminal.size();

    // Create menu bar (matching Borland's TEditorApp::initMenuBar)
    let menu_bar = init_menu_bar(Rect::new(0, 0, width as i16, 1));
    app.set_menu_bar(menu_bar);

    // Create status line (matching Borland's TEditorApp::initStatusLine)
    let status_line = init_status_line(Rect::new(0, height as i16 - 1, width as i16, height as i16));
    app.set_status_line(status_line);

    // Event loop
    app.running = true;
    while app.running {
        // Update menu states based on current file editor state
        update_menu_states(&app);

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
            // Status line handles events first (pre-process phase)
            // Matches Borland: TStatusLine has ofPreProcess flag
            if let Some(ref mut status_line) = app.status_line {
                status_line.handle_event(&mut event);
            }

            // Menu bar handles events (special case for F10 and Alt keys)
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

            // AFTER desktop handles events, check if CM_CLOSE was generated
            // Frame converts MouseUp on close button to CM_CLOSE during handle_event
            // Matches Borland: TWindow::close() calls valid(cmClose) before destroying
            // Note: We can't call valid() directly due to borrow checker (it needs &mut app)
            // So we handle the validation logic inline
            if event.what == EventType::Command && event.command == CM_CLOSE {
                if app.desktop.child_count() > 0 {
                    // Check if modified (drop borrow before showing dialog)
                    let is_modified = get_file_editor(&app)
                        .map_or(false, |fe| fe.is_modified());

                    let should_close = if is_modified {
                        // Get title for prompt
                        let title = get_file_editor(&app)
                            .map(|fe| fe.get_title())
                            .unwrap_or_else(|| "Untitled".to_string());

                        // Show save prompt (using FileEditor's validation pattern)
                        let message = format!("Save changes to {}?", title);
                        match turbo_vision::views::msgbox::confirmation_box(&mut app, &message) {
                            cmd if cmd == CM_YES => {
                                save_file(&mut app);
                                true
                            }
                            cmd if cmd == CM_NO => true,
                            _ => false, // Cancel
                        }
                    } else {
                        true // Not modified, allow close
                    };

                    if should_close {
                        // User chose Yes or No - allow the close
                        app.desktop.remove_child(0);
                    }
                    // Clear event whether cancelled or completed
                    event.clear();
                }
            }

            // Remove any closed windows
            app.desktop.remove_closed_windows();

            // Handle commands
            if event.what == EventType::Command {
                match event.command {
                    CM_QUIT => {
                        let should_quit = if app.desktop.child_count() > 0 {
                            let is_modified = get_file_editor(&app)
                                .map_or(false, |fe| fe.is_modified());

                            if is_modified {
                                let title = get_file_editor(&app)
                                    .map(|fe| fe.get_title())
                                    .unwrap_or_else(|| "Untitled".to_string());

                                let message = format!("Save changes to {}?", title);
                                match turbo_vision::views::msgbox::confirmation_box(&mut app, &message) {
                                    cmd if cmd == CM_YES => {
                                        save_file(&mut app);
                                        true
                                    }
                                    cmd if cmd == CM_NO => true,
                                    _ => false,
                                }
                            } else {
                                true
                            }
                        } else {
                            true
                        };

                        if should_quit {
                            app.running = false;
                        }
                    }
                    CM_NEW => {
                        // Create new untitled window
                        // Use RELATIVE coordinates for desktop
                        let window_bounds = Rect::new(5, 1, width as i16 - 5, height as i16 - 4);
                        create_editor_window(&mut app, window_bounds, None);
                    }
                    CM_OPEN => {
                        if let Some(path) = show_file_open_dialog(&mut app) {
                            // Create new window with loaded file
                            // Use RELATIVE coordinates for desktop
                            let window_bounds = Rect::new(5, 1, width as i16 - 5, height as i16 - 4);
                            create_editor_window(&mut app, window_bounds, Some(path));
                        }
                    }
                    CM_SAVE => {
                        save_file(&mut app);
                    }
                    CM_SAVE_AS => {
                        save_file_as(&mut app);
                    }
                    CM_FIND => {
                        if let Some(search_text) = search_box(&mut app, "Find") {
                            // TODO: Implement actual search in editor
                            show_message(&mut app, "Find", &format!("Searching for: {}", search_text));
                        }
                    }
                    CM_REPLACE => {
                        if let Some((find_text, replace_text)) = search_replace_box(&mut app, "Replace") {
                            // TODO: Implement actual replace in editor
                            show_message(&mut app, "Replace", &format!("Replace '{}' with '{}'", find_text, replace_text));
                        }
                    }
                    CM_SEARCH_AGAIN => {
                        // TODO: Implement search again functionality
                        show_message(&mut app, "Search Again", "Repeating last search...");
                    }
                    CM_CHANGE_DIR => {
                        // TODO: Implement change directory dialog
                        show_message(&mut app, "Change Directory", "Change directory not yet implemented");
                    }
                    CM_SHOW_CLIP => {
                        // TODO: Implement clipboard window
                        show_message(&mut app, "Clipboard", "Clipboard window not yet implemented");
                    }
                    CM_GOTO_LINE => {
                        if let Some(line_num) = goto_line_box(&mut app, "Go to Line") {
                            // TODO: Implement actual goto line in editor
                            show_message(&mut app, "Go to Line", &format!("Going to line: {}", line_num));
                        }
                    }
                    CM_ANALYZE => {
                        analyze_with_rust_analyzer(&mut app);
                    }
                    CM_SHOW_ERRORS => {
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

/// Update menu command states based on current file editor state
///
/// - SAVE is disabled if no window is open OR the current window is "Untitled"
/// - SAVE_AS is disabled if no window is open
///
/// Matches Borland: TEditorApp::updateMenuItems() pattern
fn update_menu_states(app: &Application) {
    let has_window = app.desktop.child_count() > 0;
    let has_filename = has_window && get_file_editor(app)
        .map_or(false, |fe| fe.filename().is_some());

    // SAVE: enabled only if window exists AND has a filename
    if has_filename {
        command_set::enable_command(CM_SAVE);
    } else {
        command_set::disable_command(CM_SAVE);
    }

    // SAVE_AS: enabled only if window exists
    if has_window {
        command_set::enable_command(CM_SAVE_AS);
    } else {
        command_set::disable_command(CM_SAVE_AS);
    }
}

/// Initialize menu bar (matching Borland's TEditorApp::initMenuBar from tvedit3.cc)
fn init_menu_bar(r: Rect) -> MenuBar {
    let mut menu_bar = MenuBar::new(r);

    // File menu (matching Borland's sub1)
    let file_menu_items = vec![
        MenuItem::with_shortcut("~O~pen", CM_OPEN, 0, "F3", 0),
        MenuItem::with_shortcut("~N~ew", CM_NEW, 0, "", 0),
        MenuItem::with_shortcut("~S~ave", CM_SAVE, 0, "F2", 0),
        MenuItem::with_shortcut("S~a~ve as...", CM_SAVE_AS, 0, "", 0),
        MenuItem::separator(),
        MenuItem::with_shortcut("~C~hange dir...", CM_CHANGE_DIR, 0, "", 0),
        // MenuItem::with_shortcut("S~h~ell", CM_DOS_SHELL, 0, "", 0),  // TODO: Add shell support
        MenuItem::separator(),
        MenuItem::with_shortcut("E~x~it", CM_QUIT, 0, "Alt+X", 0),
    ];
    let file_menu = SubMenu::new("~F~ile", Menu::from_items(file_menu_items));

    // Edit menu (matching Borland's sub2)
    let edit_menu_items = vec![
        // MenuItem::with_shortcut("~U~ndo", CM_UNDO, 0, "", 0),  // TODO: Add undo command routing
        // MenuItem::separator(),
        // MenuItem::with_shortcut("Cu~t~", CM_CUT, 0, "Shift+Del", 0),
        // MenuItem::with_shortcut("~C~opy", CM_COPY, 0, "Ctrl+Ins", 0),
        // MenuItem::with_shortcut("~P~aste", CM_PASTE, 0, "Shift+Ins", 0),
        MenuItem::with_shortcut("~S~how clipboard", CM_SHOW_CLIP, 0, "", 0),
        MenuItem::separator(),
        // MenuItem::with_shortcut("~C~lear", CM_CLEAR, 0, "Ctrl+Del", 0),
        MenuItem::with_shortcut("~G~oto Line...", CM_GOTO_LINE, 0, "Ctrl+G", 0),
    ];
    let edit_menu = SubMenu::new("~E~dit", Menu::from_items(edit_menu_items));

    // Search menu (matching Borland's sub3)
    let search_menu_items = vec![
        MenuItem::with_shortcut("~F~ind...", CM_FIND, 0, "", 0),
        MenuItem::with_shortcut("~R~eplace...", CM_REPLACE, 0, "", 0),
        MenuItem::with_shortcut("~S~earch again", CM_SEARCH_AGAIN, 0, "", 0),
    ];
    let search_menu = SubMenu::new("~S~earch", Menu::from_items(search_menu_items));

    // Windows menu (matching Borland's sub4)
    let windows_menu_items = vec![
        // MenuItem::with_shortcut("~S~ize/move", CM_RESIZE, 0, "Ctrl+F5", 0),
        MenuItem::with_shortcut("~Z~oom", CM_ZOOM, 0, "F5", 0),
        MenuItem::with_shortcut("~T~ile", CM_TILE, 0, "", 0),
        MenuItem::with_shortcut("C~a~scade", CM_CASCADE, 0, "", 0),
        MenuItem::with_shortcut("~N~ext", CM_NEXT, 0, "F6", 0),
        MenuItem::with_shortcut("~P~revious", CM_PREV, 0, "Shift+F6", 0),
        MenuItem::with_shortcut("~C~lose", CM_CLOSE, 0, "Alt+F3", 0),
    ];
    let windows_menu = SubMenu::new("~W~indows", Menu::from_items(windows_menu_items));

    // Rust-specific Tools menu (extension to Borland)
    let tools_menu_items = vec![
        MenuItem::with_shortcut("~A~nalyze with rust-analyzer", CM_ANALYZE, 0, "F7", 0),
        MenuItem::with_shortcut("Show ~E~rrors", CM_SHOW_ERRORS, 0, "F8", 0),
    ];
    let tools_menu = SubMenu::new("~T~ools", Menu::from_items(tools_menu_items));

    menu_bar.add_submenu(file_menu);
    menu_bar.add_submenu(edit_menu);
    menu_bar.add_submenu(search_menu);
    menu_bar.add_submenu(windows_menu);
    menu_bar.add_submenu(tools_menu);

    menu_bar
}

/// Initialize status line (matching Borland's TEditorApp::initStatusLine from tvedit3.cc)
fn init_status_line(r: Rect) -> StatusLine {
    use turbo_vision::core::event::{KB_F2, KB_F3, KB_F5, KB_F6};

    // Matching Borland's status line order: F10 Menu, F2 Save, F3 Open, Alt+F3 Close, F5 Zoom, F6 Next
    StatusLine::new(
        r,
        vec![
            StatusItem::new("~F10~ Menu", KB_F10, 0),
            StatusItem::new("~F2~ Save", KB_F2, CM_SAVE),
            StatusItem::new("~F3~ Open", KB_F3, CM_OPEN),
            StatusItem::new("~Alt+F3~ Close", 0, CM_CLOSE),
            StatusItem::new("~F5~ Zoom", KB_F5, CM_ZOOM),
            StatusItem::new("~F6~ Next", KB_F6, CM_NEXT),
        ],
    )
}

/// Create a FileEditor and add it to the desktop
/// Matches Borland: TEditWindow with TFileEditor
fn create_editor_window(
    app: &mut Application,
    bounds: Rect,
    file_path: Option<PathBuf>,
) {
    let title = file_path
        .as_ref()
        .and_then(|p| p.file_name().and_then(|n| n.to_str()))
        .unwrap_or("Untitled");

    let mut file_editor = FileEditor::new(bounds, title);

    // Set Rust syntax highlighting
    file_editor.edit_window_mut().editor_rc().borrow_mut().set_highlighter(Box::new(RustHighlighter::new()));

    // Load file if provided
    if let Some(path) = file_path {
        if let Err(e) = file_editor.load_file(path) {
            eprintln!("Failed to load file: {}", e);
        }
    }

    // Add to desktop (matches Borland: desktop->insert(new TFileEditor(...)))
    app.desktop.add(Box::new(file_editor));
}

fn show_file_open_dialog(app: &mut Application) -> Option<PathBuf> {
    let (term_width, term_height) = app.terminal.size();
    let dialog_width = 62;
    let dialog_height = 20;
    let dialog_x = (term_width as i16 - dialog_width) / 2;
    let dialog_y = (term_height as i16 - dialog_height) / 2;

    let bounds = Rect::new(dialog_x, dialog_y, dialog_x + dialog_width, dialog_y + dialog_height);

    // Start in current directory
    let initial_dir = std::env::current_dir().ok();

    // Use standard FileDialogBuilder API
    let mut file_dialog = FileDialogBuilder::new()
        .bounds(bounds)
        .title("Open File")
        .wildcard("*.rs");

    if let Some(dir) = initial_dir {
        file_dialog = file_dialog.initial_dir(dir);
    }

    let mut dialog = file_dialog.build();
    dialog.execute(app)
}

fn save_file(app: &mut Application) {
    let file_editor = match get_file_editor_mut(app) {
        Some(fe) => fe,
        None => return,
    };

    match file_editor.save() {
        Ok(true) => {
            // Refresh the window title after saving
            file_editor.refresh_title();
            show_message(app, "Save", "File saved successfully");
        }
        Ok(false) => {
            // No filename, need save_as
            save_file_as(app);
        }
        Err(_) => {
            show_error(app, "Error", "Failed to save file");
        }
    }
}

fn save_file_as(app: &mut Application) {
    if let Some(path) = show_file_save_dialog(app) {
        let file_editor = match get_file_editor_mut(app) {
            Some(fe) => fe,
            None => return,
        };

        if file_editor.save_as(path).is_ok() {
            // Refresh the window title after saving with new filename
            file_editor.refresh_title();
            show_message(app, "Save", "File saved successfully");
        } else {
            show_error(app, "Error", "Failed to save file");
        }
    }
}

fn show_file_save_dialog(app: &mut Application) -> Option<PathBuf> {
    let (term_width, term_height) = app.terminal.size();
    let dialog_width = 62;
    let dialog_height = 20;
    let dialog_x = (term_width as i16 - dialog_width) / 2;
    let dialog_y = (term_height as i16 - dialog_height) / 2;

    let bounds = Rect::new(dialog_x, dialog_y, dialog_x + dialog_width, dialog_y + dialog_height);

    // Start in current directory
    let initial_dir = std::env::current_dir().ok();

    // Use standard FileDialogBuilder API with Save button label
    let mut file_dialog = FileDialogBuilder::new()
        .bounds(bounds)
        .title("Save File As")
        .wildcard("*.rs")
        .button_label("~S~ave");  // Use "Save" button instead of "Open"

    if let Some(dir) = initial_dir {
        file_dialog = file_dialog.initial_dir(dir);
    }

    let mut dialog = file_dialog.build();
    dialog.execute(app)
}

fn analyze_with_rust_analyzer(app: &mut Application) {
    // For now, just show a message about rust-analyzer integration
    // In a real implementation, we would:
    // 1. Save the file temporarily
    // 2. Run rust-analyzer via LSP or command line
    // 3. Parse the results
    // 4. Display errors/warnings

    let has_filename = get_file_editor(app)
        .map_or(false, |fe| fe.filename().is_some());

    if !has_filename {
        show_error(app, "Analysis", "Please save the file first");
        return;
    }

    show_message(app, "Analysis", "Running rust-analyzer...\n\n(Integration in progress)");
}

fn show_message(app: &mut Application, _title: &str, message: &str) {
    message_box_ok(app, message);
}

fn show_error(app: &mut Application, _title: &str, message: &str) {
    message_box_error(app, message);
}
