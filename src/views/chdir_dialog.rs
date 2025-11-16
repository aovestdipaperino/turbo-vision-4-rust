// (C) 2025 - Enzo Lombardi

//! Change Directory Dialog - specialized dialog for directory selection
//!
//! Matches Borland: TChDirDialog (tchdirdi.cc)
//!
//! A dialog for navigating and selecting directories with a tree view,
//! input line, and control buttons.
//!
//! Layout (widened from Borland for more space):
//! - Dialog bounds: 5, 2, 75, 21 (70 wide x 19 tall)
//! - Directory name input at top
//! - Directory tree listbox in middle
//! - Buttons (OK, Chdir, Revert) on right side

use crate::app::Application;
use crate::core::command::{CommandId, CM_OK};
use crate::core::event::{Event, EventType};
use crate::core::geometry::{Point, Rect};
use crate::core::history::HistoryManager;
use crate::terminal::Terminal;
use super::dialog::Dialog;
use super::input_line::InputLine;
use super::label::Label;
use super::button::Button;
use super::dir_listbox::DirListBox;
use super::scrollbar::ScrollBar;
use super::history::History;
use super::msgbox::message_box_error;
use super::{View, ViewId};
use super::list_viewer::ListViewer;
use std::path::PathBuf;
use std::cell::RefCell;
use std::rc::Rc;

// Custom commands for ChDirDialog
const CM_CHANGE_DIR: CommandId = 200;
const CM_REVERT: CommandId = 201;

// History ID for directory paths
// Matches Borland: histId parameter in TChDirDialog constructor
const DEFAULT_HISTORY_ID: u16 = 10;

/// Wrapper that allows ScrollBar to be a child view
struct SharedScrollBar(Rc<RefCell<ScrollBar>>);

impl View for SharedScrollBar {
    fn bounds(&self) -> Rect {
        self.0.borrow().bounds()
    }

    fn set_bounds(&mut self, bounds: Rect) {
        self.0.borrow_mut().set_bounds(bounds);
    }

    fn draw(&mut self, terminal: &mut Terminal) {
        self.0.borrow_mut().draw(terminal);
    }

    fn handle_event(&mut self, event: &mut Event) {
        self.0.borrow_mut().handle_event(event);
    }

    fn get_palette(&self) -> Option<crate::core::palette::Palette> {
        self.0.borrow().get_palette()
    }

    fn get_owner_type(&self) -> super::view::OwnerType {
        self.0.borrow().get_owner_type()
    }

    fn set_owner_type(&mut self, owner_type: super::view::OwnerType) {
        self.0.borrow_mut().set_owner_type(owner_type);
    }
}

/// Wrapper that allows DirListBox to be a child view with shared access
/// Also broadcasts CM_FILE_FOCUSED when focused item changes (like FileList does)
/// Manages scrollbars connected to the listbox
struct SharedDirListBox {
    inner: Rc<RefCell<DirListBox>>,
    dir_input_data: Rc<RefCell<String>>,
    last_focused_path: Option<PathBuf>,
    v_scrollbar: Rc<RefCell<ScrollBar>>,
    h_scrollbar: Rc<RefCell<ScrollBar>>,
}

impl SharedDirListBox {
    fn new(
        inner: Rc<RefCell<DirListBox>>,
        dir_input_data: Rc<RefCell<String>>,
        v_scrollbar: Rc<RefCell<ScrollBar>>,
        h_scrollbar: Rc<RefCell<ScrollBar>>,
    ) -> Self {
        // Initialize with current focused entry
        let last_focused_path = inner.borrow().get_focused_entry().map(|e| e.path.clone());
        Self {
            inner,
            dir_input_data,
            last_focused_path,
            v_scrollbar,
            h_scrollbar,
        }
    }

    /// Update scrollbar positions based on listbox state
    fn update_scrollbars(&mut self) {
        // Get values from list state (can't hold reference due to borrow checker)
        let total_items;
        let top_item;
        let visible_items;
        {
            let listbox = self.inner.borrow();
            let list_state = listbox.list_state();
            total_items = list_state.range;
            top_item = list_state.top_item;
            visible_items = listbox.bounds().height_clamped() as usize;
        }

        // Update vertical scrollbar
        // Matches Borland: TScrollBar::setParams(value, min, max, pgSize, arStep)
        self.v_scrollbar.borrow_mut().set_params(
            top_item as i32,
            0,
            total_items.saturating_sub(visible_items) as i32,
            visible_items as i32,
            1,
        );

        // Horizontal scrollbar is typically not needed for directory names
        // but we'll set it to 0 for consistency
        self.h_scrollbar.borrow_mut().set_params(0, 0, 0, 1, 1);
    }
}

impl View for SharedDirListBox {
    fn bounds(&self) -> Rect {
        self.inner.borrow().bounds()
    }

    fn set_bounds(&mut self, bounds: Rect) {
        self.inner.borrow_mut().set_bounds(bounds);
    }

    fn draw(&mut self, terminal: &mut Terminal) {
        // Update scrollbars before drawing
        self.update_scrollbars();

        self.inner.borrow_mut().draw(terminal);
    }

    fn handle_event(&mut self, event: &mut Event) {
        // Track the old scroll position
        let old_top_item = self.inner.borrow().list_state().top_item;

        // Handle scrollbar events first (mouse clicks on scrollbars)
        // Matches Borland: TScroller::handleEvent() processes scrollbar events
        if event.what == EventType::MouseDown || event.what == EventType::MouseMove {
            let v_bounds = self.v_scrollbar.borrow().bounds();
            let h_bounds = self.h_scrollbar.borrow().bounds();

            if v_bounds.contains(event.mouse.pos) {
                // Let vertical scrollbar handle the event
                self.v_scrollbar.borrow_mut().handle_event(event);

                // Get the new scroll value and update listbox
                let new_top = self.v_scrollbar.borrow().get_value() as usize;
                {
                    let mut listbox = self.inner.borrow_mut();
                    listbox.list_state_mut().top_item = new_top;
                }

                // Update scrollbars to reflect new position
                self.update_scrollbars();
                return;
            } else if h_bounds.contains(event.mouse.pos) {
                // Let horizontal scrollbar handle the event
                self.h_scrollbar.borrow_mut().handle_event(event);
                // Horizontal scrolling not used for directory names
                return;
            }
        }

        // Track focused entry before event
        let path_before = self.inner.borrow().get_focused_entry().map(|e| e.path.clone());

        // Let DirListBox handle the event
        self.inner.borrow_mut().handle_event(event);

        // Check if scroll position changed (from keyboard navigation)
        let new_top_item = self.inner.borrow().list_state().top_item;
        if old_top_item != new_top_item {
            // Scroll position changed - update scrollbars
            self.update_scrollbars();
        }

        // Check if focused entry changed
        let path_after = self.inner.borrow().get_focused_entry().map(|e| e.path.clone());

        if path_before != path_after {
            // Focused entry changed - update input data
            // Matches Borland: message(owner, evBroadcast, cmFileFocused, this)
            if let Some(ref new_path) = path_after {
                *self.dir_input_data.borrow_mut() = new_path.to_string_lossy().to_string();
            }

            self.last_focused_path = path_after;
        }

        // Handle broadcast commands from Chdir and Revert buttons
        if event.what == EventType::Broadcast {
            match event.command {
                CM_CHANGE_DIR => {
                    // Navigate to the selected directory in listbox
                    // Matches Borland: gets focused item from dirList, updates current dir
                    // Extract path first to avoid overlapping borrows
                    let new_path = self.inner.borrow().get_focused_entry().map(|e| e.path.clone());

                    if let Some(new_path) = new_path {
                        // Update listbox to show the new directory
                        if self.inner.borrow_mut().change_dir(&new_path).is_ok() {
                            // Update input line with the new path
                            *self.dir_input_data.borrow_mut() = new_path.to_string_lossy().to_string();
                            // Update scrollbars after directory change
                            self.update_scrollbars();
                        }
                    }
                    event.clear();
                }
                CM_REVERT => {
                    // Revert to current working directory
                    // Matches Borland: resets dialog to show current directory
                    if let Ok(current_dir) = std::env::current_dir() {
                        *self.dir_input_data.borrow_mut() = current_dir.to_string_lossy().to_string();
                        // Update dir listbox to show current directory
                        let _ = self.inner.borrow_mut().change_dir(&current_dir);
                        // Update scrollbars after directory change
                        self.update_scrollbars();
                    }
                    event.clear();
                }
                _ => {}
            }
        }
    }

    fn can_focus(&self) -> bool {
        self.inner.borrow().can_focus()
    }

    fn state(&self) -> crate::core::state::StateFlags {
        self.inner.borrow().state()
    }

    fn set_state(&mut self, state: crate::core::state::StateFlags) {
        self.inner.borrow_mut().set_state(state);
    }

    fn get_palette(&self) -> Option<crate::core::palette::Palette> {
        self.inner.borrow().get_palette()
    }

    fn get_owner_type(&self) -> super::view::OwnerType {
        self.inner.borrow().get_owner_type()
    }

    fn set_owner_type(&mut self, owner_type: super::view::OwnerType) {
        self.inner.borrow_mut().set_owner_type(owner_type);
    }
}

/// Change Directory Dialog
///
/// Matches Borland: TChDirDialog (tchdirdi.cc)
///
/// Dialog for selecting and changing to a directory. Shows a hierarchical
/// directory tree and allows navigation through directories.
pub struct ChDirDialog {
    dialog: Dialog,
    dir_input_data: Rc<RefCell<String>>,
    history_id: u16,
    #[allow(dead_code)] // Will be used for navigation implementation
    dir_list_id: ViewId,
    #[allow(dead_code)] // Will be used for input updates
    dir_input_id: ViewId,
    #[allow(dead_code)] // Will be used for button state management
    ok_button_id: ViewId,
    #[allow(dead_code)] // Will be used for button state management
    chdir_button_id: ViewId,
    selected_directory: Option<PathBuf>,
}

impl ChDirDialog {
    /// Create a new change directory dialog
    ///
    /// # Arguments
    /// * `history_id` - Optional history ID for storing directory history (defaults to 10)
    ///
    /// Matches Borland constructor:
    /// `TChDirDialog::TChDirDialog( ushort opts, ushort histId )`
    ///
    /// Dialog layout (widened from Borland for more space):
    /// - Dialog: TRect( 5, 2, 75, 21 ) = 70 wide x 19 tall (widened for more space)
    /// - Input line: TRect( 3, 3, 48, 4 )
    /// - Label "Directory name": (2, 2)
    /// - History button: TRect( 48, 3, 51, 4 )
    /// - Vertical scrollbar: TRect( 50, 6, 51, 16 )
    /// - Horizontal scrollbar: TRect( 3, 16, 50, 17 )
    /// - Dir listbox: TRect( 3, 6, 50, 16 )
    /// - Label "Directory tree": (2, 5)
    /// - OK button: TRect( 53, 6, 63, 8 )
    /// - Chdir button: TRect( 53, 9, 63, 11 )
    /// - Revert button: TRect( 53, 12, 63, 14 )
    pub fn new(history_id: Option<u16>) -> Self {
        let history_id = history_id.unwrap_or(DEFAULT_HISTORY_ID);
        // Widened dialog bounds: TRect( 5, 2, 75, 21 ) - 70 columns instead of 48
        // This is absolute screen coordinates, will be centered by ofCentered flag
        let dialog_bounds = Rect::new(5, 2, 75, 21);
        let mut dialog = Dialog::new(dialog_bounds, "Change Directory");

        // Get current directory for initial value
        let current_dir = std::env::current_dir()
            .unwrap_or_else(|_| PathBuf::from("/"))
            .to_string_lossy()
            .to_string();
        let dir_input_data = Rc::new(RefCell::new(current_dir.clone()));

        // Directory name input line - widened: TRect( 3, 3, 48, 4 )
        let input_bounds = Rect::new(3, 3, 48, 4);
        let dir_input = InputLine::new(input_bounds, 255, Rc::clone(&dir_input_data));
        let dir_input_id = dialog.add(Box::new(dir_input));

        // Label "Directory name" - Borland: (2, 2)
        let label_bounds = Rect::new(2, 2, 20, 2);
        let dir_label = Label::new(label_bounds, "Directory ~n~ame");
        dialog.add(Box::new(dir_label));

        // History button - adjusted: TRect( 48, 3, 51, 4 )
        // Shows a dropdown button (▼) that displays previous directories
        let history_button = History::new(Point::new(48, 3), history_id);
        dialog.add(Box::new(history_button));

        // Vertical scrollbar - adjusted: TRect( 50, 6, 51, 16 )
        let v_scrollbar_bounds = Rect::new(50, 6, 51, 16);
        let v_scrollbar = ScrollBar::new_vertical(v_scrollbar_bounds);
        let v_scrollbar_rc = Rc::new(RefCell::new(v_scrollbar));
        dialog.add(Box::new(SharedScrollBar(Rc::clone(&v_scrollbar_rc))));

        // Horizontal scrollbar - adjusted: TRect( 3, 16, 50, 17 )
        let h_scrollbar_bounds = Rect::new(3, 16, 50, 17);
        let h_scrollbar = ScrollBar::new_horizontal(h_scrollbar_bounds);
        let h_scrollbar_rc = Rc::new(RefCell::new(h_scrollbar));
        dialog.add(Box::new(SharedScrollBar(Rc::clone(&h_scrollbar_rc))));

        // Directory listbox - widened: TRect( 3, 6, 50, 16 )
        let listbox_bounds = Rect::new(3, 6, 50, 16);
        let current_path = PathBuf::from(&current_dir);
        let dir_list = DirListBox::new(listbox_bounds, &current_path);
        let dir_listbox = Rc::new(RefCell::new(dir_list));
        let shared_listbox = SharedDirListBox::new(
            Rc::clone(&dir_listbox),
            Rc::clone(&dir_input_data),
            Rc::clone(&v_scrollbar_rc),
            Rc::clone(&h_scrollbar_rc),
        );
        let dir_list_id = dialog.add(Box::new(shared_listbox));

        // Label "Directory tree" - Borland: (2, 5)
        let tree_label_bounds = Rect::new(2, 5, 20, 5);
        let tree_label = Label::new(tree_label_bounds, "Directory ~t~ree");
        dialog.add(Box::new(tree_label));

        // OK button - adjusted: TRect( 53, 6, 63, 8 )
        let ok_bounds = Rect::new(53, 6, 63, 8);
        let ok_button = Button::new(ok_bounds, "~O~K", CM_OK, true);
        let ok_button_id = dialog.add(Box::new(ok_button));

        // Chdir button - adjusted: TRect( 53, 9, 63, 11 )
        let chdir_bounds = Rect::new(53, 9, 63, 11);
        let mut chdir_button = Button::new(chdir_bounds, "~C~hdir", CM_CHANGE_DIR, false);
        chdir_button.set_broadcast(true);  // Broadcast instead of ending dialog
        chdir_button.set_selectable(false);  // Not part of focus cycle
        let chdir_button_id = dialog.add(Box::new(chdir_button));

        // Revert button - adjusted: TRect( 53, 12, 63, 14 )
        let revert_bounds = Rect::new(53, 12, 63, 14);
        let mut revert_button = Button::new(revert_bounds, "~R~evert", CM_REVERT, false);
        revert_button.set_broadcast(true);  // Broadcast instead of ending dialog
        revert_button.set_selectable(false);  // Not part of focus cycle
        dialog.add(Box::new(revert_button));

        // Help button is intentionally NOT implemented
        // Borland: TRect( 35, 15, 45, 17 ) - optional, requires help system
        // Will be added when application-wide help system is implemented

        Self {
            dialog,
            dir_input_data,
            history_id,
            dir_list_id,
            dir_input_id,
            ok_button_id,
            chdir_button_id,
            selected_directory: None,
        }
    }

    /// Execute the dialog modally
    ///
    /// Returns the selected directory if OK was pressed, None if cancelled
    ///
    /// Matches Borland: user interacts with dialog, OK/Cancel to exit
    /// The valid() method validates the directory before closing
    /// History is automatically updated on success
    pub fn execute(&mut self, app: &mut Application) -> Option<PathBuf> {
        loop {
            let end_state = self.dialog.execute(app);

            if end_state == CM_OK {
                // Validate the directory path from the input line
                let dir_path = self.dir_input_data.borrow().clone();
                let path = PathBuf::from(&dir_path);

                // Try to change directory (validates that it exists and is accessible)
                if let Err(e) = std::env::set_current_dir(&path) {
                    // Invalid directory - show error and re-execute dialog
                    // Matches Borland: valid() returns False, shows error, and keeps dialog open
                    let error_msg = format!("Invalid directory\n\n{}", e);
                    message_box_error(app, &error_msg);

                    // Re-execute the dialog to let user try again
                    continue;
                }

                // Valid directory - add to history and return success
                // Matches Borland: historyAdd is called in handleEvent on cmReleasedFocus
                HistoryManager::add(self.history_id, dir_path.clone());

                self.selected_directory = Some(path.clone());
                return Some(path);
            } else {
                // User cancelled
                return None;
            }
        }
    }

    /// Get the selected directory
    pub fn get_directory(&self) -> Option<PathBuf> {
        self.selected_directory.clone()
    }

    /// Get the end state (command that closed the dialog)
    pub fn get_end_state(&self) -> CommandId {
        self.dialog.get_end_state()
    }
}

impl View for ChDirDialog {
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

    fn can_focus(&self) -> bool {
        true
    }

    fn state(&self) -> crate::core::state::StateFlags {
        self.dialog.state()
    }

    fn set_state(&mut self, state: crate::core::state::StateFlags) {
        self.dialog.set_state(state);
    }

    fn get_palette(&self) -> Option<crate::core::palette::Palette> {
        self.dialog.get_palette()
    }
}

/// Builder for creating change directory dialogs with a fluent API.
pub struct ChDirDialogBuilder {
    history_id: Option<u16>,
}

impl ChDirDialogBuilder {
    /// Creates a new ChDirDialogBuilder
    pub fn new() -> Self {
        Self { history_id: None }
    }

    /// Sets a custom history ID for directory history
    #[must_use]
    pub fn history_id(mut self, history_id: u16) -> Self {
        self.history_id = Some(history_id);
        self
    }

    /// Builds the ChDirDialog with Borland standard layout
    pub fn build(self) -> ChDirDialog {
        ChDirDialog::new(self.history_id)
    }

    /// Builds the ChDirDialog as a Box
    pub fn build_boxed(self) -> Box<ChDirDialog> {
        Box::new(self.build())
    }
}

impl Default for ChDirDialogBuilder {
    fn default() -> Self {
        Self::new()
    }
}
