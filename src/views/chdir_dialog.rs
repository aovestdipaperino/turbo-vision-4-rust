// (C) 2025 - Enzo Lombardi

//! Change Directory Dialog - specialized dialog for directory selection
//!
//! Matches Borland: TChDirDialog (tchdirdi.cc)
//!
//! A dialog for navigating and selecting directories with a tree view,
//! input line, and control buttons.
//!
//! Layout matches Borland:
//! - Dialog bounds: 16, 2, 64, 21 (48 wide x 19 tall)
//! - Directory name input at top
//! - Directory tree listbox in middle
//! - Buttons (OK, Chdir, Revert) on right side

use crate::app::Application;
use crate::core::command::{CommandId, CM_OK};
use crate::core::event::{Event, EventType};
use crate::core::geometry::Rect;
use crate::terminal::Terminal;
use super::dialog::Dialog;
use super::input_line::InputLine;
use super::label::Label;
use super::button::Button;
use super::dir_listbox::DirListBox;
use super::scrollbar::ScrollBar;
use super::msgbox::message_box_error;
use super::View;
use std::path::PathBuf;
use std::cell::RefCell;
use std::rc::Rc;

// Custom commands for ChDirDialog
const CM_CHANGE_DIR: CommandId = 200;
const CM_REVERT: CommandId = 201;

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
struct SharedDirListBox(Rc<RefCell<DirListBox>>);

impl View for SharedDirListBox {
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

    fn can_focus(&self) -> bool {
        self.0.borrow().can_focus()
    }

    fn state(&self) -> crate::core::state::StateFlags {
        self.0.borrow().state()
    }

    fn set_state(&mut self, state: crate::core::state::StateFlags) {
        self.0.borrow_mut().set_state(state);
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

/// Change Directory Dialog
///
/// Matches Borland: TChDirDialog (tchdirdi.cc)
///
/// Dialog for selecting and changing to a directory. Shows a hierarchical
/// directory tree and allows navigation through directories.
pub struct ChDirDialog {
    dialog: Dialog,
    dir_input_data: Rc<RefCell<String>>,
    dir_listbox: Rc<RefCell<DirListBox>>,
    #[allow(dead_code)] // Will be used for navigation implementation
    dir_list_idx: usize,
    #[allow(dead_code)] // Will be used for input updates
    dir_input_idx: usize,
    #[allow(dead_code)] // Will be used for button state management
    ok_button_idx: usize,
    #[allow(dead_code)] // Will be used for button state management
    chdir_button_idx: usize,
    selected_directory: Option<PathBuf>,
}

impl ChDirDialog {
    /// Create a new change directory dialog
    ///
    /// Matches Borland constructor:
    /// `TChDirDialog::TChDirDialog( ushort opts, ushort histId )`
    ///
    /// Dialog layout (Borland coordinates):
    /// - Dialog: TRect( 16, 2, 64, 21 ) = 48 wide x 19 tall
    /// - Input line: TRect( 3, 3, 30, 4 )
    /// - Label "Directory name": (2, 2)
    /// - History button: TRect( 30, 3, 33, 4 )
    /// - Vertical scrollbar: TRect( 32, 6, 33, 16 )
    /// - Horizontal scrollbar: TRect( 3, 16, 32, 17 )
    /// - Dir listbox: TRect( 3, 6, 32, 16 )
    /// - Label "Directory tree": (2, 5)
    /// - OK button: TRect( 35, 6, 45, 8 )
    /// - Chdir button: TRect( 35, 9, 45, 11 )
    /// - Revert button: TRect( 35, 12, 45, 14 )
    pub fn new() -> Self {
        // Borland dialog bounds: TRect( 16, 2, 64, 21 )
        // This is absolute screen coordinates, will be centered by ofCentered flag
        let dialog_bounds = Rect::new(16, 2, 64, 21);
        let mut dialog = Dialog::new(dialog_bounds, "Change Directory");

        // Get current directory for initial value
        let current_dir = std::env::current_dir()
            .unwrap_or_else(|_| PathBuf::from("/"))
            .to_string_lossy()
            .to_string();
        let dir_input_data = Rc::new(RefCell::new(current_dir.clone()));

        // Directory name input line - Borland: TRect( 3, 3, 30, 4 )
        let input_bounds = Rect::new(3, 3, 30, 4);
        let dir_input = InputLine::new(input_bounds, 255, Rc::clone(&dir_input_data));
        let dir_input_idx = dialog.add(Box::new(dir_input));

        // Label "Directory name" - Borland: (2, 2)
        let label_bounds = Rect::new(2, 2, 20, 3);
        let dir_label = Label::new(label_bounds, "Directory ~n~ame");
        dialog.add(Box::new(dir_label));

        // TODO: History button - Borland: TRect( 30, 3, 33, 4 )
        // Skipping history button for now as it requires History implementation

        // Vertical scrollbar - Borland: TRect( 32, 6, 33, 16 )
        let v_scrollbar_bounds = Rect::new(32, 6, 33, 16);
        let v_scrollbar = ScrollBar::new_vertical(v_scrollbar_bounds);
        let v_scrollbar_rc = Rc::new(RefCell::new(v_scrollbar));
        dialog.add(Box::new(SharedScrollBar(Rc::clone(&v_scrollbar_rc))));

        // Horizontal scrollbar - Borland: TRect( 3, 16, 32, 17 )
        let h_scrollbar_bounds = Rect::new(3, 16, 32, 17);
        let h_scrollbar = ScrollBar::new_horizontal(h_scrollbar_bounds);
        let h_scrollbar_rc = Rc::new(RefCell::new(h_scrollbar));
        dialog.add(Box::new(SharedScrollBar(Rc::clone(&h_scrollbar_rc))));

        // Directory listbox - Borland: TRect( 3, 6, 32, 16 )
        let listbox_bounds = Rect::new(3, 6, 32, 16);
        let current_path = PathBuf::from(&current_dir);
        let dir_list = DirListBox::new(listbox_bounds, &current_path);
        let dir_listbox = Rc::new(RefCell::new(dir_list));
        let dir_list_idx = dialog.add(Box::new(SharedDirListBox(Rc::clone(&dir_listbox))));

        // Label "Directory tree" - Borland: (2, 5)
        let tree_label_bounds = Rect::new(2, 5, 20, 6);
        let tree_label = Label::new(tree_label_bounds, "Directory ~t~ree");
        dialog.add(Box::new(tree_label));

        // OK button - Borland: TRect( 35, 6, 45, 8 )
        let ok_bounds = Rect::new(35, 6, 45, 8);
        let ok_button = Button::new(ok_bounds, "~O~K", CM_OK, true);
        let ok_button_idx = dialog.add(Box::new(ok_button));

        // Chdir button - Borland: TRect( 35, 9, 45, 11 )
        let chdir_bounds = Rect::new(35, 9, 45, 11);
        let chdir_button = Button::new(chdir_bounds, "~C~hdir", CM_CHANGE_DIR, false);
        let chdir_button_idx = dialog.add(Box::new(chdir_button));

        // Revert button - Borland: TRect( 35, 12, 45, 14 )
        let revert_bounds = Rect::new(35, 12, 45, 14);
        let revert_button = Button::new(revert_bounds, "~R~evert", CM_REVERT, false);
        dialog.add(Box::new(revert_button));

        // TODO: Optional Help button - Borland: TRect( 35, 15, 45, 17 )
        // Skipping help button for now

        Self {
            dialog,
            dir_input_data,
            dir_listbox,
            dir_list_idx,
            dir_input_idx,
            ok_button_idx,
            chdir_button_idx,
            selected_directory: None,
        }
    }

    /// Execute the dialog modally
    ///
    /// Returns the selected directory if OK was pressed, None if cancelled
    ///
    /// Matches Borland: user interacts with dialog, OK/Cancel to exit
    /// The valid() method validates the directory before closing
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

                // Valid directory - return success
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

        // Handle custom commands
        if event.what == EventType::Command {
            match event.command {
                CM_CHANGE_DIR => {
                    // Navigate to the selected directory in listbox
                    // Matches Borland: gets focused item from dirList, updates current dir
                    if let Some(entry) = self.dir_listbox.borrow().get_focused_entry() {
                        let new_path = entry.path.clone();

                        // Update listbox to show the new directory
                        if self.dir_listbox.borrow_mut().change_dir(&new_path).is_ok() {
                            // Update input line with the new path
                            *self.dir_input_data.borrow_mut() = new_path.to_string_lossy().to_string();
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
                        let _ = self.dir_listbox.borrow_mut().change_dir(&current_dir);
                    }
                    event.clear();
                }
                _ => {}
            }
        }
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
pub struct ChDirDialogBuilder {}

impl ChDirDialogBuilder {
    /// Creates a new ChDirDialogBuilder
    pub fn new() -> Self {
        Self {}
    }

    /// Builds the ChDirDialog with Borland standard layout
    pub fn build(self) -> ChDirDialog {
        ChDirDialog::new()
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
