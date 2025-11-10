// (C) 2025 - Enzo Lombardi
//! File Dialog - A file selection dialog for opening files
//!
//! ## Current Implementation
//!
//! The FileDialog provides a classic file selection interface with:
//! - Input line for typing filenames
//! - ListBox showing files and directories in the current path
//! - **Mouse support**: Click on files to select, double-click to open
//! - **Keyboard navigation**: Arrow keys, PgUp/PgDn, Home/End
//! - Directory navigation (double-click directories or select and press Enter)
//! - Wildcard filtering (e.g., "*.rs" shows only Rust files)
//! - Parent directory navigation via ".."
//!
//! ## Usage
//!
//! Users can select files in multiple ways:
//! 1. **Click a file** once to select it (updates the input field)
//! 2. **Double-click a file** to select and open it
//! 3. **Use arrow keys** to navigate the list, press Enter to select
//! 4. **Type a filename** directly in the input box
//!
//! Directory navigation:
//! - Press Enter on a folder (`[dirname]`) to navigate into it
//! - Press Enter on `..` to go to parent directory
//! - Click on folders to navigate (single click selects, double-click or Enter opens)
//! - The dialog stays open while navigating directories
//!
//! Wildcard patterns:
//! - `"*"` - Shows all files
//! - `"*.rs"` - Shows only files ending with `.rs`
//! - `"*.toml"` - Shows only files ending with `.toml`
//! - `"test"` - Shows files containing "test" in their name
//!
//! **Note**: Directories are always shown regardless of the wildcard pattern.
//!
//! ## Implementation Notes
//!
//! The FileDialog tracks ListBox selection state by intercepting keyboard and mouse
//! events before passing them to the dialog. This allows it to:
//! - Maintain a shadow selection index
//! - Update the InputLine when files are selected
//! - Handle directory navigation seamlessly
//!
//! ### Architecture
//!
//! The Dialog/Window/Group hierarchy now provides `child_at_mut()` methods for accessing
//! child views. This architectural improvement allows components to:
//! - Query child view state after adding them to containers
//! - Modify child views dynamically
//! - Create more sophisticated interactions between parent and child views
//!
//! The current FileDialog implementation uses event interception for simplicity and
//! performance, but could alternatively use direct child access if needed for more
//! complex scenarios.

use crate::core::geometry::Rect;
use crate::core::event::{Event, EventType};
use crate::core::command::{CM_OK, CM_CANCEL, CM_FILE_FOCUSED, CommandId};
use crate::terminal::Terminal;
use super::dialog::Dialog;
use super::input_line::InputLine;
use super::listbox::ListBox;
use super::button::Button;
use super::label::Label;
use super::View;
use std::path::PathBuf;
use std::fs;
use std::rc::Rc;
use std::cell::RefCell;

const CMD_FILE_SELECTED: u16 = 1000;

// Child indices in the dialog
const CHILD_LISTBOX: usize = 4; // ListBox
const CHILD_OK_BUTTON: usize = 5; // Open button

pub struct FileDialog {
    dialog: Dialog,
    current_path: PathBuf,
    wildcard: String,
    file_name_data: Rc<RefCell<String>>,
    files: Vec<String>,
    selected_file_index: usize,  // Track ListBox selection
}

impl FileDialog {
    pub fn new(bounds: Rect, title: &str, wildcard: &str, initial_dir: Option<PathBuf>) -> Self {
        let dialog = Dialog::new(bounds, title);

        let current_path = initial_dir.unwrap_or_else(|| {
            std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."))
        });

        let file_name_data = Rc::new(RefCell::new(String::new()));

        Self {
            dialog,
            current_path,
            wildcard: wildcard.to_string(),
            file_name_data,
            files: Vec::new(),
            selected_file_index: 0,
        }
    }

    pub fn build(mut self) -> Self {
        let bounds = self.dialog.bounds();
        let dialog_width = bounds.width();

        // Label for file name input
        let name_label = Label::new(Rect::new(2, 1, 12, 2), "~N~ame:");
        self.dialog.add(Box::new(name_label));

        // File name input line
        let file_input = InputLine::new(
            Rect::new(12, 1, dialog_width - 4, 2),
            255,
            self.file_name_data.clone()
        );
        self.dialog.add(Box::new(file_input));

        // Current path label
        let path_str = format!(" {}", self.current_path.display());
        let path_label = Label::new(Rect::new(2, 3, dialog_width - 4, 4), &path_str);
        self.dialog.add(Box::new(path_label));

        // Label for files list
        let files_label = Label::new(Rect::new(2, 5, 12, 6), "~F~iles:");
        self.dialog.add(Box::new(files_label));

        // File list box - will be populated after reading directory
        let mut file_list = ListBox::new(
            Rect::new(2, 6, dialog_width - 4, bounds.height() - 6),
            CMD_FILE_SELECTED,
        );

        // Load directory contents first
        self.read_directory();

        // Populate the list box with files
        file_list.set_items(self.files.clone());
        self.dialog.add(Box::new(file_list));

        // Buttons
        let button_y = bounds.height() - 4;
        let button_spacing = 14;
        let mut button_x = 2;

        let open_button = Button::new(
            Rect::new(button_x, button_y, button_x + 12, button_y + 2),
            "  ~O~pen  ",
            CM_OK,
            true,
        );
        self.dialog.add(Box::new(open_button));
        button_x += button_spacing;

        let cancel_button = Button::new(
            Rect::new(button_x, button_y, button_x + 12, button_y + 2),
            " ~C~ancel ",
            CM_CANCEL,
            false,
        );
        self.dialog.add(Box::new(cancel_button));

        // Set focus to the listbox by default (better UX for file selection)
        self.dialog.set_initial_focus();
        // TODO: Need a way to set focus to a specific child index
        // For now, initial focus goes to first focusable (input), user can Tab to listbox

        self
    }

    pub fn execute(&mut self, app: &mut crate::app::Application) -> Option<PathBuf> {
        loop {
            // Update OK button state based on input field
            self.update_ok_button_state();

            // Draw desktop first (background), then dialog on top
            // This matches Borland's pattern where getEvent() triggers full screen redraw
            app.desktop.draw(&mut app.terminal);
            self.dialog.draw(&mut app.terminal);
            self.dialog.update_cursor(&mut app.terminal);
            let _ = app.terminal.flush();

            // Get event
            if let Some(mut event) = app.terminal.poll_event(std::time::Duration::from_millis(50)).ok().flatten() {
                // Handle double ESC to close
                if event.what == EventType::Keyboard && event.key_code == crate::core::event::KB_ESC_ESC {
                    return None;
                }

                // Let the dialog (and its children) handle the event first
                self.dialog.handle_event(&mut event);

                // After event is processed, check if ListBox selection changed
                // Matches Borland: TFileList::focusItem() broadcasts cmFileFocused when selection changes
                // We read the ListBox selection after it has processed navigation events
                self.sync_inputline_with_listbox();

                // Check if dialog should close
                if event.what == EventType::Command {
                    match event.command {
                        CM_OK => {
                            // Get file name from input field
                            let file_name = self.file_name_data.borrow().clone();
                            if !file_name.is_empty() {
                                // Matches Borland: TFileDialog::valid() checks wildcards and directories
                                // (tfiledia.cc:98-124)

                                // Check if input contains wildcards
                                if self.contains_wildcards(&file_name) {
                                    // Update wildcard filter and reload list
                                    self.wildcard = file_name.clone();
                                    self.read_directory();
                                    self.rebuild_and_redraw(&mut app.terminal);
                                    // Stay open - don't return
                                    continue;
                                }

                                // Check if it's a directory navigation request or file selection
                                if let Some(path) = self.handle_selection(&file_name, &mut app.terminal) {
                                    // File selected - return it
                                    return Some(path);
                                }
                                // Directory navigation - continue loop (handle_selection returns None)
                            }
                            // If input is empty, do nothing (don't close dialog)
                            // This effectively disables the OK button when input is empty
                        }
                        CM_CANCEL | crate::core::command::CM_CLOSE => {
                            return None;
                        }
                        CMD_FILE_SELECTED => {
                            // User double-clicked or pressed Enter on a file in the list
                            // Matches Borland: cmFileDoubleClicked is converted to cmOK,
                            // which triggers valid() that reads from the input field

                            // The input field has ALREADY been updated by the cmFileFocused broadcast
                            // when the item was focused (see track_listbox_events)
                            // So we just read what's already there and handle it
                            let file_name = self.file_name_data.borrow().clone();

                            if !file_name.is_empty() {
                                // Handle the selection (navigate dirs or return file)
                                // Matches Borland: TFileDialog::valid() reads from input field
                                if let Some(path) = self.handle_selection(&file_name, &mut app.terminal) {
                                    // File selected - return it
                                    return Some(path);
                                }
                                // Directory navigation - continue loop (valid() returns False)
                            }
                        }
                        _ => {}
                    }
                }
            }
        }
    }

    /// Sync the InputLine with the current ListBox selection
    /// Matches Borland: TFileList::focusItem() broadcasts cmFileFocused when selection changes
    /// We read the ListBox selection after it has processed events
    fn sync_inputline_with_listbox(&mut self) {
        // Get the current selection from the ListBox
        if CHILD_LISTBOX >= self.dialog.child_count() {
            return;
        }

        let listbox = self.dialog.child_at(CHILD_LISTBOX);
        let new_selection = listbox.get_list_selection();

        // Only update if selection actually changed
        if new_selection != self.selected_file_index {
            self.selected_file_index = new_selection;

            // Get the selected item text
            if self.selected_file_index < self.files.len() {
                let selected = self.files[self.selected_file_index].clone();

                // Format the input field text based on selection type
                // Matches Borland: TFileInputLine::handleEvent() (tfileinp.cc:35-45)
                let display_text = if selected.starts_with('[') && selected.ends_with(']') {
                    // Directory selected - show "dirname/*.txt" format
                    let dir_name = &selected[1..selected.len() - 1];
                    format!("{}/{}", dir_name, self.wildcard)
                } else if selected == ".." {
                    // Parent directory - just show ".."
                    selected.clone()
                } else {
                    // Regular file - show just the filename
                    selected.clone()
                };

                // Update the shared data field directly (Borland pattern)
                // InputLine will observe this change via its broadcast handler
                *self.file_name_data.borrow_mut() = display_text;

                // Broadcast to notify InputLine to update its display
                // Matches Borland: message(owner, evBroadcast, cmFileFocused, this)
                // InputLine will only update display if NOT focused (prevents interrupting typing)
                let mut broadcast = Event::broadcast(CM_FILE_FOCUSED);
                self.dialog.handle_event(&mut broadcast);
            }
        }
    }

    fn handle_selection(&mut self, file_name: &str, terminal: &mut Terminal) -> Option<PathBuf> {
        // Check if input contains directory/wildcard format (e.g., "dirname/*.txt")
        // Matches Borland: TFileDialog::valid() parsing (tfiledia.cc:98-124)
        if let Some(slash_pos) = file_name.rfind('/') {
            let dir_part = &file_name[..slash_pos];
            let file_part = &file_name[slash_pos + 1..];

            // Navigate to the directory
            if !dir_part.is_empty() {
                self.current_path.push(dir_part);
            }

            // Update wildcard if file_part contains wildcards
            if self.contains_wildcards(file_part) {
                self.wildcard = file_part.to_string();
            }

            // rebuild_and_redraw will call read_directory() internally
            self.rebuild_and_redraw(terminal);
            return None;
        }

        if file_name == ".." {
            // Go to parent directory
            if let Some(parent) = self.current_path.parent() {
                self.current_path = parent.to_path_buf();
                // rebuild_and_redraw will call read_directory() internally
                self.rebuild_and_redraw(terminal);
            }
            None
        } else if file_name.starts_with('[') && file_name.ends_with(']') {
            // Directory selected - navigate into it
            let dir_name = &file_name[1..file_name.len() - 1];
            self.current_path.push(dir_name);
            // rebuild_and_redraw will call read_directory() internally
            self.rebuild_and_redraw(terminal);
            None
        } else {
            // File selected - update input and return
            *self.file_name_data.borrow_mut() = file_name.to_string();
            Some(self.current_path.join(file_name))
        }
    }

    fn update_ok_button_state(&mut self) {
        use crate::core::state::SF_DISABLED;

        // Check if input field is empty
        let is_empty = self.file_name_data.borrow().is_empty();

        // Get the OK button and update its disabled state
        // Matches Borland's TView::setState(sfDisabled, enable) pattern
        if CHILD_OK_BUTTON < self.dialog.child_count() {
            let ok_button = self.dialog.child_at_mut(CHILD_OK_BUTTON);
            ok_button.set_state_flag(SF_DISABLED, is_empty);
        }
    }

    fn rebuild_and_redraw(&mut self, _terminal: &mut Terminal) {
        // Create a new dialog with updated file list
        let old_bounds = self.dialog.bounds();
        let old_title = "Open File"; // TODO: Store title

        *self = Self::new(old_bounds, old_title, &self.wildcard.clone(), Some(self.current_path.clone())).build();

        // Reset focus to listbox after directory navigation
        // Matches Borland: fileList->select() calls owner->setCurrent(this, normalSelect)
        // (tfiledia.cc:275,287 and tview.cc:658-664)
        // This properly updates both the Group's focused index AND the child's focus state
        if CHILD_LISTBOX < self.dialog.child_count() {
            self.dialog.set_focus_to_child(CHILD_LISTBOX);
            // Also ensure listbox selection is at index 0
            self.dialog.child_at_mut(CHILD_LISTBOX).set_list_selection(0);
        }

        // Reset selection index
        self.selected_file_index = 0;

        // CRITICAL: Broadcast initial selection after directory navigation
        // Matches Borland: TFileList::readDirectory() broadcasts cmFileFocused after newList()
        // (tfilelis.cc:588-595) and TFileList::setState() broadcasts on focus (tfilelis.cc:146-149)
        if !self.files.is_empty() {
            let first_item = self.files[0].clone();

            // Format the display text for the input field
            let display_text = if first_item.starts_with('[') && first_item.ends_with(']') {
                // Directory selected - show "dirname/*.txt" format
                let dir_name = &first_item[1..first_item.len() - 1];
                format!("{}/{}", dir_name, self.wildcard)
            } else if first_item == ".." {
                // Parent directory - just show ".."
                first_item.clone()
            } else {
                // Regular file - show just the filename
                first_item.clone()
            };

            // Update the shared data field
            *self.file_name_data.borrow_mut() = display_text;

            // Broadcast to notify InputLine to update its display
            let mut broadcast = Event::broadcast(CM_FILE_FOCUSED);
            self.dialog.handle_event(&mut broadcast);
        } else {
            // No files - clear the input
            *self.file_name_data.borrow_mut() = String::new();
        }
    }

    fn read_directory(&mut self) {
        self.files.clear();

        // Add parent directory entry
        if self.current_path.parent().is_some() {
            self.files.push("..".to_string());
        }

        // Read directory contents
        if let Ok(entries) = fs::read_dir(&self.current_path) {
            let mut dirs = Vec::new();
            let mut regular_files = Vec::new();

            for entry in entries.flatten() {
                if let Ok(metadata) = entry.metadata() {
                    let name = entry.file_name().to_string_lossy().to_string();

                    if metadata.is_dir() {
                        dirs.push(format!("[{}]", name));
                    } else if self.matches_wildcard(&name) {
                        regular_files.push(name);
                    }
                }
            }

            // Sort and combine: directories first, then files
            dirs.sort();
            regular_files.sort();
            self.files.extend(dirs);
            self.files.extend(regular_files);
        }
    }

    fn contains_wildcards(&self, name: &str) -> bool {
        // Check if the name contains wildcard characters
        // Matches Borland: IsWild() checks for '*' and '?' (tfiledia.cc:42-47)
        name.contains('*') || name.contains('?')
    }

    fn matches_wildcard(&self, name: &str) -> bool {
        if self.wildcard == "*" || self.wildcard.is_empty() {
            return true;
        }

        // Simple wildcard matching (*.ext)
        if let Some(ext) = self.wildcard.strip_prefix("*.") {
            name.ends_with(&format!(".{}", ext))
        } else {
            name.contains(&self.wildcard)
        }
    }

    pub fn get_selected_file(&self) -> Option<PathBuf> {
        let file_name = self.file_name_data.borrow().clone();
        if !file_name.is_empty() {
            Some(self.current_path.join(file_name))
        } else {
            None
        }
    }

    /// Get the current directory being browsed
    /// Useful for ChDirDialog to get the selected directory
    pub fn get_current_directory(&self) -> PathBuf {
        self.current_path.clone()
    }

    /// Get the end state (command that closed the dialog)
    pub fn get_end_state(&self) -> CommandId {
        self.dialog.get_end_state()
    }
}

impl View for FileDialog {
    fn bounds(&self) -> Rect {
        self.dialog.bounds()
    }

    fn set_bounds(&mut self, bounds: Rect) {
        self.dialog.set_bounds(bounds);
    }

    fn draw(&mut self, terminal: &mut Terminal) {
        self.dialog.draw(terminal);
    }

    fn handle_event(&mut self, event: &mut Event) {
        self.dialog.handle_event(event);
    }

    fn get_palette(&self) -> Option<crate::core::palette::Palette> {
        self.dialog.get_palette()
    }
}

/// Builder for creating file dialogs with a fluent API.
///
/// # Examples
///
/// ```ignore
/// use turbo_vision::views::file_dialog::FileDialogBuilder;
/// use turbo_vision::core::geometry::Rect;
/// use std::path::PathBuf;
///
/// // Create a basic file dialog
/// let dialog = FileDialogBuilder::new()
///     .bounds(Rect::new(10, 5, 70, 20))
///     .title("Open File")
///     .wildcard("*.rs")
///     .build();
///
/// // Create a file dialog with initial directory
/// let dialog = FileDialogBuilder::new()
///     .bounds(Rect::new(10, 5, 70, 20))
///     .title("Select File")
///     .wildcard("*")
///     .initial_dir(PathBuf::from("/home/user/documents"))
///     .build();
/// ```
pub struct FileDialogBuilder {
    bounds: Option<Rect>,
    title: Option<String>,
    wildcard: String,
    initial_dir: Option<PathBuf>,
}

impl FileDialogBuilder {
    /// Creates a new FileDialogBuilder with default values.
    pub fn new() -> Self {
        Self {
            bounds: None,
            title: None,
            wildcard: "*".to_string(),
            initial_dir: None,
        }
    }

    /// Sets the file dialog bounds (required).
    #[must_use]
    pub fn bounds(mut self, bounds: Rect) -> Self {
        self.bounds = Some(bounds);
        self
    }

    /// Sets the dialog title (required).
    #[must_use]
    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
        self
    }

    /// Sets the wildcard filter (default: "*").
    /// Examples: "*.rs", "*.toml", "*"
    #[must_use]
    pub fn wildcard(mut self, wildcard: impl Into<String>) -> Self {
        self.wildcard = wildcard.into();
        self
    }

    /// Sets the initial directory (optional).
    /// If not set, uses the current working directory.
    #[must_use]
    pub fn initial_dir(mut self, dir: PathBuf) -> Self {
        self.initial_dir = Some(dir);
        self
    }

    /// Builds the FileDialog.
    ///
    /// # Panics
    ///
    /// Panics if required fields (bounds, title) are not set.
    pub fn build(self) -> FileDialog {
        let bounds = self.bounds.expect("FileDialog bounds must be set");
        let title = self.title.expect("FileDialog title must be set");
        FileDialog::new(bounds, &title, &self.wildcard, self.initial_dir)
    }

    /// Builds the FileDialog as a Box.
    pub fn build_boxed(self) -> Box<FileDialog> {
        Box::new(self.build())
    }
}

impl Default for FileDialogBuilder {
    fn default() -> Self {
        Self::new()
    }
}
