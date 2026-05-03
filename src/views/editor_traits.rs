// (C) 2026 - Enzo Lombardi

//! Window-level editor contracts that mirror Borland's TEditor / TFileEditor
//! hierarchy.
//!
//! `View::valid(cmd)` already covers no-app close validation, but a save-on-close
//! prompt needs `&mut Application` to display a modal dialog. These traits add
//! `valid_close(app, cmd)` for that purpose; the IDE event loop calls it when an
//! editor's frame requests a close (CM_CLOSE).
//!
//! Hierarchy:
//! - [`Editor`] — basic editor window. Default `valid_close` always allows close.
//! - [`FileEditor`] — adds an optional file path, dirty state, and load/save.
//!   Implementors typically delegate `valid_close` to [`confirm_save_on_close`].

use std::path::PathBuf;
use std::time::SystemTime;

use crate::app::Application;
use crate::core::command::CommandId;
use crate::views::view::View;

/// Result of probing the on-disk file behind a [`FileEditor`].
///
/// `last_known_mtime` is what the editor recorded the last time it touched
/// the file (load/save). [`FileEditor::poll_external_changes`] stats the path
/// and compares against that snapshot.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExternalState {
    /// No file path is bound yet, so there's nothing to compare.
    NoFile,
    /// File on disk matches the editor's last-known mtime.
    Unchanged,
    /// File on disk has a newer mtime than the editor's snapshot.
    Modified,
    /// File path is set but the file no longer exists on disk.
    Deleted,
}

/// Window-level editor contract. The default `valid_close` allows the close to
/// proceed; override to abort it (e.g. after asking the user to save).
pub trait Editor: View {
    /// Called by the event loop when this editor's frame requests a close.
    /// Returns true if the window should be removed.
    fn valid_close(&mut self, _app: &mut Application, _command: CommandId) -> bool {
        true
    }
}

/// Editor that is backed by an on-disk file. The path may be `None` for a new
/// untitled buffer; in that case [`FileEditor::prompt_save_as`] is invoked
/// when the user chooses to save.
pub trait FileEditor: Editor {
    /// Current file path. Cloned per call so implementations may store the path
    /// behind interior mutability (e.g. `Rc<RefCell<Option<PathBuf>>>`).
    fn file_path(&self) -> Option<PathBuf>;
    fn set_file_path(&mut self, path: Option<PathBuf>);

    /// True when in-memory contents differ from the on-disk file (or no file yet).
    fn is_dirty(&self) -> bool;

    fn save(&mut self) -> std::io::Result<()>;
    fn save_as(&mut self, path: PathBuf) -> std::io::Result<()>;
    fn load(&mut self, path: PathBuf) -> std::io::Result<()>;

    /// Reset to an empty Untitled buffer (no file path, no breakpoints, clean).
    fn new_buffer(&mut self);

    /// mtime recorded the last time the editor read or wrote the file. `None`
    /// when there's no file or the editor has never touched disk yet.
    fn last_known_mtime(&self) -> Option<SystemTime>;

    /// Probe the on-disk file and report whether it has changed since the last
    /// load/save. Pure metadata (`stat`); does not read contents or modify the
    /// editor's last-known mtime — call [`FileEditor::reload`] to do that.
    fn poll_external_changes(&self) -> ExternalState;

    /// Re-read the file from disk into the buffer and refresh the last-known
    /// mtime. Caller is responsible for prompting on dirty buffers.
    fn reload(&mut self) -> std::io::Result<()>;

    /// Friendly name used in dialogs and titles. Defaults to the file's basename
    /// or `"Untitled"`.
    fn display_name(&self) -> String {
        self.file_path()
            .as_deref()
            .and_then(|p| p.file_name())
            .and_then(|n| n.to_str())
            .map(|s| s.to_string())
            .unwrap_or_else(|| "Untitled".to_string())
    }

    /// Show a Save As dialog and persist the buffer to the chosen path.
    /// Returns true on a successful save, false on cancel or I/O error.
    fn prompt_save_as(&mut self, app: &mut Application) -> bool;
}

/// Standard "do you want to save?" dialog used by [`FileEditor`] implementors
/// from their [`Editor::valid_close`] override. Mirrors Borland's
/// `TFileEditor::valid(cmClose)`: YES → save, NO → discard, CANCEL → abort close.
pub fn confirm_save_on_close<E: FileEditor + ?Sized>(
    editor: &mut E,
    app: &mut Application,
    command: CommandId,
) -> bool {
    use crate::core::command::{CM_CLOSE, CM_NO, CM_YES};
    use crate::views::msgbox::confirmation_box;

    if command != CM_CLOSE || !editor.is_dirty() {
        return true;
    }

    let message = format!("{} has been modified.\n\nSave changes?", editor.display_name());
    match confirmation_box(app, &message) {
        c if c == CM_YES => {
            if editor.file_path().is_some() {
                editor.save().is_ok()
            } else {
                editor.prompt_save_as(app)
            }
        }
        c if c == CM_NO => true,
        _ => false,
    }
}
