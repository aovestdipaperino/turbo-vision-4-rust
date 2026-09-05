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
//! - `"a?c.txt"` - `?` matches any single character
//! - `"test"` - No wildcard characters: matches only a file named exactly "test"
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

/// FileDialog - A file selection dialog for opening/saving files
///
/// ## Usage
///
/// ```rust,ignore
/// // Open dialog (default)
/// let mut dialog = FileDialog::new(bounds, "Open File", "*.rs", None).build();
///
/// // Save dialog with custom button label
/// let mut dialog = FileDialog::new(bounds, "Save File", "*", None)
///     .with_button_label("~S~ave")
///     .build();
///
/// // Execute and get selected file path
/// match dialog.execute(&mut app) {
///     Some(path) => println!("Selected: {}", path.display()),
///     None => println!("Canceled"),
/// }
/// ```
///
/// ## Dialog Close Behavior
///
/// The FileDialog closes (returns a file path) for exactly 3 reasons:
/// 1. **File is double-clicked** in the ListBox → Dialog closes, returns file path
/// 2. **File is selected and Enter/OK is pressed** → Dialog closes, returns file path
/// 3. **User cancels** (close button, Cancel button, or double ESC) → Dialog closes, returns None
///
/// ## Folder Navigation
///
/// When a folder is selected and opened (double-click or Enter while focused on folder):
/// - The dialog stays OPEN
/// - Current directory is updated
/// - File list is refreshed with new directory contents
/// - Selection moves to first item in new directory
///
/// This applies to: "..", regular folders "[dirname]", and folder paths like "subdir/*.rs"
///
/// ## Button Labels
///
/// Customize the button label using `with_button_label()`:
/// - Default: `"~O~pen"` (Open button)
/// - For save: `"~S~ave"`
/// - For export: `"~E~xport"`
/// - Any other text: `"~C~ustom"`
///
/// The `~` character indicates the hotkey underline in the button text.
use super::View;
use super::button::Button;
use super::dialog::Dialog;
use super::group::{Group, GroupLike};
use super::input_line::InputLine;
use super::label::Label;
use super::listbox::ListBox;
use super::window::{Window, WindowLike};
use crate::app::ModalTick;
use crate::core::command::{CM_CANCEL, CM_FILE_FOCUSED, CM_OK, CommandId};
use crate::core::event::{Event, EventType};
use crate::core::geometry::Rect;
use std::cell::RefCell;
use std::fs;
use std::path::PathBuf;
use std::rc::Rc;

const CMD_FILE_SELECTED: u16 = 1000;

/// What a typed/selected name in the file dialog should do.
/// Mirrors the decision order of Borland's `TFileDialog::valid()`
/// (TFileDialog.cc:230-279): wildcard → refilter, existing directory →
/// navigate, otherwise → file result.
#[derive(Debug, PartialEq, Eq)]
enum SelectionAction {
    /// Navigate to the parent directory ("..").
    NavigateParent,
    /// Navigate into an existing directory (dialog stays open).
    Navigate(PathBuf),
    /// Apply a new wildcard filter, optionally after navigating.
    Refilter {
        dir: Option<PathBuf>,
        wildcard: String,
    },
    /// A file was chosen; close the dialog returning this path.
    SelectFile(PathBuf),
}

/// Classifies `file_name` relative to `current` following Borland's
/// `TFileDialog::valid()` order. Pure function so it can be unit tested.
fn classify_selection(current: &std::path::Path, file_name: &str) -> SelectionAction {
    if file_name == ".." {
        return SelectionAction::NavigateParent;
    }
    if file_name.starts_with('[') && file_name.ends_with(']') && file_name.len() >= 2 {
        // Listbox directory entry "[dirname]"
        return SelectionAction::Navigate(current.join(&file_name[1..file_name.len() - 1]));
    }

    // Split off a directory part, if any. `Path::join` handles absolute
    // paths by replacing the base.
    let (dir_part, file_part) = match file_name.rfind('/') {
        Some(pos) => (&file_name[..=pos], &file_name[pos + 1..]),
        None => ("", file_name),
    };
    let base = if dir_part.is_empty() {
        current.to_path_buf()
    } else {
        current.join(dir_part)
    };

    // 1. Wildcard → refilter (navigating first if a path was given)
    if file_part.contains('*') || file_part.contains('?') {
        return SelectionAction::Refilter {
            dir: if dir_part.is_empty() {
                None
            } else {
                Some(base)
            },
            wildcard: file_part.to_string(),
        };
    }

    let full = if file_part.is_empty() {
        base
    } else {
        base.join(file_part)
    };

    // 2. Existing directory → navigate into it
    if full.is_dir() {
        return SelectionAction::Navigate(full);
    }

    // 3. Otherwise it is a file result
    SelectionAction::SelectFile(full)
}

/// Glob matcher supporting `*` (any run of chars) and `?` (any single char)
/// anywhere in the pattern. Case-sensitive, like the underlying OS.
fn glob_match(pattern: &str, name: &str) -> bool {
    fn matches(p: &[char], n: &[char]) -> bool {
        match p.first() {
            None => n.is_empty(),
            Some('*') => matches(&p[1..], n) || (!n.is_empty() && matches(p, &n[1..])),
            Some('?') => !n.is_empty() && matches(&p[1..], &n[1..]),
            Some(&c) => n.first() == Some(&c) && matches(&p[1..], &n[1..]),
        }
    }
    let p: Vec<char> = pattern.chars().collect();
    let n: Vec<char> = name.chars().collect();
    matches(&p, &n)
}

// Child indices in the dialog
const CHILD_LISTBOX: usize = 4; // ListBox
const CHILD_OK_BUTTON: usize = 5; // Open button

pub struct FileDialog {
    dialog: Dialog,
    current_path: PathBuf,
    wildcard: String,
    file_name_data: Rc<RefCell<String>>,
    files: Vec<String>,
    selected_file_index: usize, // Track ListBox selection
    title: String,              // Store title for rebuilds
    button_label: String,       // "Open", "Save", etc.
    /// The file the user picked; `execute` returns it once the modal loop
    /// ends with `CM_OK`.
    selected: Option<PathBuf>,
    /// Set when the list contents changed under the double buffer; the
    /// modal tick turns it into `Terminal::force_full_redraw`.
    needs_full_redraw: bool,
}

impl FileDialog {
    pub fn new(bounds: Rect, title: &str, wildcard: &str, initial_dir: Option<PathBuf>) -> Self {
        let mut dialog = Dialog::new(bounds, title);
        dialog.set_resizable(true);

        let current_path = initial_dir
            .unwrap_or_else(|| std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")));

        let file_name_data = Rc::new(RefCell::new(String::new()));

        Self {
            dialog,
            current_path,
            wildcard: wildcard.to_string(),
            file_name_data,
            files: Vec::new(),
            selected_file_index: 0,
            title: title.to_string(),
            button_label: "~O~pen".to_string(), // Default to "Open"
            selected: None,
            needs_full_redraw: false,
        }
    }

    /// Set the button label (e.g., "~S~ave" for save dialogs)
    pub fn with_button_label(mut self, label: &str) -> Self {
        self.button_label = label.to_string();
        self
    }

    pub fn build(mut self) -> Self {
        let bounds = self.dialog.bounds();
        let dialog_width = bounds.width();

        // Reserve space for buttons on the right
        let content_width = dialog_width - 15;

        // Label for file name input
        let name_label = Label::new(Rect::new(2, 1, 12, 1), "~N~ame:");
        self.dialog.add(Box::new(name_label));

        // File name input line
        let file_input = InputLine::new(
            Rect::new(12, 1, content_width, 2),
            255,
            self.file_name_data.clone(),
        );
        self.dialog.add(Box::new(file_input));

        // Current path label
        let path_str = format!(" {}", self.current_path.display());
        let path_label = Label::new(Rect::new(2, 3, content_width, 3), &path_str);
        self.dialog.add(Box::new(path_label));

        // Label for files list
        let files_label = Label::new(Rect::new(2, 5, 12, 5), "~F~iles:");
        self.dialog.add(Box::new(files_label));

        // File list box - leave space on right for buttons
        let mut file_list = ListBox::new(
            Rect::new(2, 6, content_width, bounds.height() - 2),
            CMD_FILE_SELECTED,
        );

        // Load directory contents first
        self.read_directory();

        // Populate the list box with files
        file_list.set_items(self.files.clone());
        self.dialog.add(Box::new(file_list));

        // Mirror the initial listbox selection into file_name_data so the input
        // field and OK button reflect a real selection on first frame, instead
        // of waiting for the user to click an item.
        if let Some(first_item) = self.files.first() {
            let display_text = if first_item.starts_with('[') && first_item.ends_with(']') {
                let dir_name = &first_item[1..first_item.len() - 1];
                format!("{}/{}", dir_name, self.wildcard)
            } else {
                first_item.clone()
            };
            *self.file_name_data.borrow_mut() = display_text;
        }

        // Buttons on the right side (vertically stacked)
        let button_x = dialog_width - 14; // 15 = 12 (button width) + 2 (right margin) + 1 (end space)
        let mut button_y = 6;

        // Create button with appropriate label (Open, Save, etc.)
        // Pad the button label to maintain consistent button width
        let button_text = format!("  {}  ", self.button_label);
        let open_button = Button::new(
            Rect::new(button_x, button_y, button_x + 11, button_y + 2),
            &button_text,
            CM_OK,
            true,
        );
        self.dialog.add(Box::new(open_button));
        button_y += 3;

        let cancel_button = Button::new(
            Rect::new(button_x, button_y, button_x + 11, button_y + 2),
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
        use crate::core::state::SF_MODAL;

        // Set modal flag - file dialogs are modal
        // Matches Borland: TFileDialog in modal state
        let old_state = self.dialog.state();
        self.dialog.set_state(old_state | SF_MODAL);
        self.selected = None;
        self.update_ok_button_state();

        // The loop is Application::execute_modal; events reach
        // FileDialog::handle_event (below), which does the file-dialog work.
        // The tick keeps the OK button in step with the input and applies a
        // deferred full redraw after the list contents changed.
        let end_state = app.execute_modal(self, |app, dialog| {
            dialog.update_ok_button_state();
            if dialog.needs_full_redraw {
                dialog.needs_full_redraw = false;
                app.terminal.force_full_redraw();
            }
            ModalTick::Continue
        });

        if end_state == CM_OK {
            self.selected.take()
        } else {
            None
        }
    }

    /// The file-dialog reaction to an event the inner `Dialog` has already
    /// processed: mirror the list selection into the input, apply wildcard
    /// filters, navigate into directories, or record the chosen file.
    /// (Borland: TFileDialog::handleEvent / valid(cmFileOpen).)
    fn after_event(&mut self, event: &mut Event) {
        // Check if dialog wants to close (e.g., close button clicked)
        // Dialog::handle_event() calls end_modal() which sets the end_state
        // Matches Borland: TDialog::execute() checks endState after handleEvent
        let end_state = self.dialog.end_state();

        // IMPORTANT: Only close dialog for CM_CANCEL and CM_CLOSE
        // For CM_OK, we need to check if it's a wildcard pattern FIRST
        // If it's a wildcard, we stay open and update the filter
        // Only close if it's actually a file selection (after wildcard check)
        if end_state == crate::core::command::CM_CANCEL
            || end_state == crate::core::command::CM_CLOSE
        {
            // CLOSE CONDITION 3: User cancels via cancel button or close
            // button; the modal loop sees the end state and returns.
            return;
        }

        // After event is processed, check if ListBox selection changed
        // Matches Borland: TFileList::focusItem() broadcasts cmFileFocused when selection changes
        // We read the ListBox selection after it has processed navigation events
        self.sync_inputline_with_listbox();

        // Effective command: Dialog::handle_event clears `event` for CM_OK/
        // CM_CANCEL/CM_YES/CM_NO and stashes the command in end_state, so we
        // pull it from there when the event field is no longer a Command.
        // ListBox commands (>= 1000, e.g. CMD_FILE_SELECTED) survive in `event`.
        let effective_command = if event.what == EventType::Command {
            Some(event.command)
        } else if end_state != 0 {
            Some(end_state)
        } else {
            None
        };

        if let Some(cmd) = effective_command {
            match cmd {
                CM_OK => {
                    // User clicked OK button or pressed Enter (while not in listbox)
                    // Matches Borland: TFileDialog::valid(cmFileOpen) (tfiledia.cc:251-302)
                    let file_name = self.file_name_data.borrow().clone();
                    if !file_name.is_empty() {
                        // Check if input contains wildcards (*.txt, *.rs, etc)
                        if self.contains_wildcards(&file_name) {
                            // Borland pattern: Update wildcard and reload list, keep dialog open
                            // Matches: strcpy(wildCard, name); fileList->readDirectory()
                            self.wildcard = file_name.clone();
                            self.read_directory();

                            // Update ListBox items directly (don't rebuild entire dialog)
                            // Downcast to ListBox to call set_items()
                            if CHILD_LISTBOX < self.dialog.child_count() {
                                let view = self.dialog.child_at_mut(CHILD_LISTBOX);
                                if let Some(listbox) = view.as_any_mut().downcast_mut::<ListBox>() {
                                    listbox.set_items(self.files.clone());
                                    listbox.set_selection(0);
                                }
                            }

                            // Update input field to show the wildcard pattern
                            *self.file_name_data.borrow_mut() = self.wildcard.clone();

                            // Reset selection tracking
                            self.selected_file_index = 0;

                            // CRITICAL: Clear the end_state that was set by Dialog.handle_event()
                            // Dialog called end_modal(CM_OK) but we're staying open for wildcard filter
                            self.dialog.end_modal(0);

                            // Force full redraw to ensure ListBox visual updates
                            // The Terminal's double-buffering system needs this to guarantee
                            // that all changed cells are resent to the actual terminal
                            self.needs_full_redraw = true;

                            // Stay open - the modal loop continues
                            return;
                        }

                        // Check if it's a directory navigation request or file selection
                        if let Some(path) = self.handle_selection(&file_name) {
                            // CLOSE CONDITION 2: File selected and OK pressed;
                            // end_state is already CM_OK, so the loop returns it.
                            self.selected = Some(path);
                            return;
                        }
                        // Directory/folder selected - navigate into it (stay open)
                        // CRITICAL: Clear the end_state so the loop continues
                        // Dialog called end_modal(CM_OK) but we're navigating into folder
                        self.dialog.end_modal(0);
                        // Loop continues with new directory contents
                    } else {
                        // If input is empty, do nothing (don't close dialog)
                        // This effectively disables the OK button when input is empty
                        // Clear end_state so dialog stays open
                        self.dialog.end_modal(0);
                    }
                }
                CM_CANCEL | crate::core::command::CM_CLOSE => {
                    // CLOSE CONDITION 3: User cancels via Cancel button;
                    // the end state closes the modal loop.
                }
                CMD_FILE_SELECTED => {
                    // User double-clicked or pressed Enter on an item in the listbox
                    // The input field has ALREADY been updated by sync_inputline_with_listbox()
                    // So we just read what's already there and handle it
                    let file_name = self.file_name_data.borrow().clone();

                    if !file_name.is_empty() {
                        // Handle the selection (navigate into folder or return file)
                        if let Some(path) = self.handle_selection(&file_name) {
                            // CLOSE CONDITION 1: File double-clicked or Enter pressed on file
                            self.selected = Some(path);
                            self.dialog.end_modal(CM_OK);
                        }
                        // Folder/directory selected - navigate into it (stay open)
                        // Loop continues with new directory contents
                    }
                    // Consumed here: the loop would otherwise re-dispatch a
                    // command that survived the first pass.
                    event.clear();
                }
                _ => {}
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

        let new_selection = self
            .dialog
            .child_at(CHILD_LISTBOX)
            .as_any()
            .downcast_ref::<ListBox>()
            .and_then(ListBox::get_selection)
            .unwrap_or(0);

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

    fn handle_selection(&mut self, file_name: &str) -> Option<PathBuf> {
        // Determines whether a selection is:
        // - A folder to navigate into (returns None, dialog stays open)
        // - A file to return (returns Some(path), closes dialog)
        //
        // Returns None when a folder/wildcard is selected → dialog stays open
        // Returns Some(path) when a file is selected → closes dialog with file path
        //
        // Matches Borland's TFileDialog::valid() order (TFileDialog.cc:230-279):
        // wildcard → refilter; existing directory → navigate; otherwise treat
        // as a file result (navigating to its parent dir if a path was given).
        match classify_selection(&self.current_path, file_name) {
            SelectionAction::NavigateParent => {
                if let Some(parent) = self.current_path.parent() {
                    self.current_path = parent.to_path_buf();
                    self.rebuild_and_redraw();
                }
                None
            }
            SelectionAction::Navigate(dir) => {
                self.current_path = dir;
                self.rebuild_and_redraw();
                None
            }
            SelectionAction::Refilter { dir, wildcard } => {
                if let Some(dir) = dir {
                    self.current_path = dir;
                }
                self.wildcard = wildcard;
                self.rebuild_and_redraw();
                None
            }
            SelectionAction::SelectFile(path) => {
                // Navigate to the file's parent directory (relevant when the
                // user typed a path like "src/main.rs" or an absolute path),
                // and return the joined path as the selection.
                if let Some(parent) = path.parent() {
                    self.current_path = parent.to_path_buf();
                }
                let name = path
                    .file_name()
                    .map(|n| n.to_string_lossy().to_string())
                    .unwrap_or_else(|| file_name.to_string());
                *self.file_name_data.borrow_mut() = name;
                Some(path)
            }
        }
    }

    fn update_ok_button_state(&mut self) {
        use crate::core::state::SF_DISABLED;

        let file_name = self.file_name_data.borrow().clone();

        // OK button is enabled whenever the input is non-empty. The CM_OK handler
        // routes the action: handle_selection navigates into directories and `..`,
        // applies wildcard patterns, or returns a file path. Disabling for those
        // cases blocked the obvious "select folder, click Open to enter it" flow.
        let should_disable = file_name.is_empty();

        // Get the OK button and update its disabled state
        // Matches Borland's TView::setState(sfDisabled, enable) pattern
        if CHILD_OK_BUTTON < self.dialog.child_count() {
            let ok_button = self.dialog.child_at_mut(CHILD_OK_BUTTON);
            ok_button.set_state_flag(SF_DISABLED, should_disable);
        }
    }

    fn rebuild_and_redraw(&mut self) {
        // Create a new dialog with updated file list
        let old_bounds = self.dialog.bounds();
        let old_title = self.title.clone();
        let old_button_label = self.button_label.clone();

        *self = Self::new(
            old_bounds,
            &old_title,
            &self.wildcard.clone(),
            Some(self.current_path.clone()),
        )
        .with_button_label(&old_button_label)
        .build();

        // Reset focus to listbox after directory navigation
        // Matches Borland: fileList->select() calls owner->setCurrent(this, normalSelect)
        // (tfiledia.cc:275,287 and tview.cc:658-664)
        // This properly updates both the Group's focused index AND the child's focus state
        if CHILD_LISTBOX < self.dialog.child_count() {
            self.dialog.set_focus_to_child(CHILD_LISTBOX);
            // Also ensure listbox selection is at index 0
            if let Some(listbox) = self
                .dialog
                .child_at_mut(CHILD_LISTBOX)
                .as_any_mut()
                .downcast_mut::<ListBox>()
            {
                listbox.set_selection(0);
            }
        }

        // Reset selection index
        self.selected_file_index = 0;

        // CRITICAL: Broadcast initial selection after directory navigation
        // Matches Borland: TFileList::readDirectory() broadcasts cmFileFocused after newList()
        // (tfilelis.cc:588-595) and TFileList::setState() broadcasts on focus (tfilelis.cc:146-149)
        if !self.files.is_empty() {
            let first_item = self.files[0].clone();

            // Format the display text for the input field
            // IMPORTANT: After applying a wildcard filter, keep the wildcard pattern
            // in the input field so the user can see what filter is active.
            // This matches user expectations: when they press OK with "*.txt",
            // the dialog applies the filter and shows "*.txt" in the input field.
            let display_text = if first_item.starts_with('[') && first_item.ends_with(']') {
                // Directory selected - show "dirname/*.txt" format
                let dir_name = &first_item[1..first_item.len() - 1];
                format!("{}/{}", dir_name, self.wildcard)
            } else if first_item == ".." {
                // Parent directory - just show ".."
                first_item.clone()
            } else {
                // Regular file - show the wildcard pattern if one was explicitly set
                // (user typed a wildcard and pressed OK to apply the filter)
                // Otherwise show the filename
                if self.wildcard.contains('*') || self.wildcard.contains('?') {
                    // Wildcard is active - show it in the input field
                    self.wildcard.clone()
                } else {
                    // Regular file selection mode - show the filename
                    first_item.clone()
                }
            };

            // Update the shared data field
            *self.file_name_data.borrow_mut() = display_text;

            // Broadcast to notify InputLine to update its display
            let mut broadcast = Event::broadcast(CM_FILE_FOCUSED);
            self.dialog.handle_event(&mut broadcast);
        } else {
            // No files - show the wildcard pattern if one was applied
            if self.wildcard.contains('*') || self.wildcard.contains('?') {
                *self.file_name_data.borrow_mut() = self.wildcard.clone();
            } else {
                *self.file_name_data.borrow_mut() = String::new();
            }
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
        // "*.*" means all files (DOS/Windows convention), "*"/"" match all.
        if self.wildcard == "*.*" || self.wildcard == "*" || self.wildcard.is_empty() {
            return true;
        }

        // Proper glob matching with `*` and `?` anywhere in the pattern.
        // A bare name with no wildcard characters is an exact match, not a
        // substring filter.
        glob_match(&self.wildcard, name)
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
        self.dialog.end_state()
    }
}

impl GroupLike for FileDialog {
    fn group(&self) -> &Group {
        self.dialog.group()
    }
    fn group_mut(&mut self) -> &mut Group {
        self.dialog.group_mut()
    }
}

impl WindowLike for FileDialog {
    fn window(&self) -> &Window {
        self.dialog.window()
    }
    fn window_mut(&mut self) -> &mut Window {
        self.dialog.window_mut()
    }
}

crate::impl_view_for_window!(FileDialog {
    fn handle_event(&mut self, event: &mut Event) {
        self.dialog.handle_event(event);
        self.after_event(event);
    }

    fn get_palette(&self) -> Option<crate::core::palette::Palette> {
        self.dialog.get_palette()
    }

    fn valid(&mut self, command: CommandId) -> bool {
        self.dialog.valid(command)
    }
});

#[cfg(test)]
mod forwarding_tests {
    use super::*;
    use crate::core::state::SF_MODAL;

    #[test]
    fn file_dialog_reports_the_inner_dialogs_state_and_end_state() {
        let mut fd = FileDialog::new(Rect::new(0, 0, 60, 20), "Open", "*.rs", None);
        fd.set_state(fd.state() | SF_MODAL);
        assert_ne!(View::state(&fd) & SF_MODAL, 0);
        assert!(fd.can_focus());
        fd.end_modal(CM_OK);
        assert_eq!(fd.end_state(), CM_OK);
        assert!(fd.as_any().downcast_ref::<FileDialog>().is_some());
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
    button_label: String,
    resizable: bool,
}

impl FileDialogBuilder {
    /// Creates a new FileDialogBuilder with default values.
    pub fn new() -> Self {
        Self {
            bounds: None,
            title: None,
            wildcard: "*".to_string(),
            initial_dir: None,
            button_label: "~O~pen".to_string(),
            resizable: true, // Resizable by default
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

    /// Sets the button label (optional, default: "~O~pen").
    /// Examples: "~S~ave", "~E~xport", "~C~hoose"
    /// The ~ character indicates the hotkey underline.
    #[must_use]
    pub fn button_label(mut self, label: impl Into<String>) -> Self {
        self.button_label = label.into();
        self
    }

    /// Sets whether the file dialog is resizable (default: true).
    #[must_use]
    pub fn resizable(mut self, resizable: bool) -> Self {
        self.resizable = resizable;
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
        let mut fd = FileDialog::new(bounds, &title, &self.wildcard, self.initial_dir)
            .with_button_label(&self.button_label)
            .build();
        fd.dialog.set_resizable(self.resizable);
        fd
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn glob_star_matches_prefix_not_substring() {
        assert!(glob_match("foo*", "foo.txt"));
        assert!(glob_match("foo*", "foo"));
        assert!(!glob_match("foo*", "xfoo"));
        assert!(!glob_match("foo*", "xfoo.txt"));
    }

    #[test]
    fn glob_question_mark_matches_single_char() {
        assert!(glob_match("a?c.txt", "abc.txt"));
        assert!(glob_match("a?c.txt", "axc.txt"));
        assert!(!glob_match("a?c.txt", "ac.txt"));
        assert!(!glob_match("a?c.txt", "abbc.txt"));
    }

    #[test]
    fn glob_star_anywhere() {
        assert!(glob_match("*.rs", "main.rs"));
        assert!(!glob_match("*.rs", "main.rss"));
        assert!(glob_match("a*b*c", "aXXbYYc"));
        assert!(!glob_match("a*b*c", "aXXbYY"));
        // Case-sensitive like the OS
        assert!(!glob_match("*.RS", "main.rs"));
    }

    #[test]
    fn bare_name_is_exact_match_not_substring() {
        let dialog = FileDialog::new(Rect::new(0, 0, 60, 20), "t", "test", None);
        assert!(dialog.matches_wildcard("test"));
        assert!(!dialog.matches_wildcard("my_test.rs"));
        assert!(!dialog.matches_wildcard("testing"));
    }

    #[test]
    fn star_dot_star_matches_everything() {
        let dialog = FileDialog::new(Rect::new(0, 0, 60, 20), "t", "*.*", None);
        assert!(dialog.matches_wildcard("main.rs"));
        assert!(dialog.matches_wildcard("Makefile"));
    }

    #[test]
    fn classify_parent_and_bracket_dirs() {
        let cur = Path::new("/base");
        assert_eq!(
            classify_selection(cur, ".."),
            SelectionAction::NavigateParent
        );
        assert_eq!(
            classify_selection(cur, "[src]"),
            SelectionAction::Navigate(PathBuf::from("/base/src"))
        );
    }

    #[test]
    fn classify_wildcard_refilters() {
        let cur = Path::new("/base");
        assert_eq!(
            classify_selection(cur, "*.rs"),
            SelectionAction::Refilter {
                dir: None,
                wildcard: "*.rs".to_string()
            }
        );
        // "dir/*.txt" navigates and refilters
        assert_eq!(
            classify_selection(cur, "src/*.txt"),
            SelectionAction::Refilter {
                dir: Some(PathBuf::from("/base/src/")),
                wildcard: "*.txt".to_string()
            }
        );
    }

    #[test]
    fn classify_existing_directory_navigates() {
        let tmp = std::env::temp_dir();
        let sub = tmp.join("tv_fd_test_dir");
        std::fs::create_dir_all(&sub).unwrap();
        let action = classify_selection(&tmp, "tv_fd_test_dir");
        assert_eq!(action, SelectionAction::Navigate(sub.clone()));
        let _ = std::fs::remove_dir(&sub);
    }

    #[test]
    fn classify_typed_path_returns_file_not_navigation() {
        // Typing "src/main.rs" must yield the FILE, not discard main.rs.
        let cur = Path::new("/base");
        assert_eq!(
            classify_selection(cur, "src/main.rs"),
            SelectionAction::SelectFile(PathBuf::from("/base/src/main.rs"))
        );
        // Absolute path keeps the file too.
        assert_eq!(
            classify_selection(cur, "/etc/hosts.bak"),
            SelectionAction::SelectFile(PathBuf::from("/etc/hosts.bak"))
        );
    }

    #[test]
    fn handle_selection_typed_path_returns_joined_path() {
        // End-to-end through handle_selection with a real temp tree:
        // typing "sub/file.txt" returns the file and navigates to its parent.
        let tmp = std::env::temp_dir().join("tv_fd_test_sel");
        let sub = tmp.join("sub");
        std::fs::create_dir_all(&sub).unwrap();
        std::fs::write(sub.join("file.txt"), b"x").unwrap();

        let mut dialog =
            FileDialog::new(Rect::new(0, 0, 60, 20), "t", "*", Some(tmp.clone())).build();
        let result = dialog.handle_selection("sub/file.txt");
        assert_eq!(result, Some(sub.join("file.txt")));
        assert_eq!(dialog.get_current_directory(), sub);
        assert_eq!(*dialog.file_name_data.borrow(), "file.txt");

        let _ = std::fs::remove_dir_all(&tmp);
    }
}
