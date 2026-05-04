// (C) 2025 - Enzo Lombardi

//! `FileEditorWindow` — `EditWindow` plus file binding (load/save) and
//! save-on-close prompt. Implements the [`Editor`] and [`FileEditor`] traits.
//!
//! Architecture:
//! `EditorWindow` (core editing) → `EditWindow` (frame/scrollbars) →
//! `FileEditorWindow` (adds file I/O + save-prompt)
//!
//! Matches Borland: `TFileEditor` (tfileedi.h, tfileedi.cc).

use std::path::PathBuf;
use std::time::SystemTime;

use crate::app::Application;
use crate::core::command::CommandId;
use crate::core::event::Event;
use crate::core::geometry::Rect;
use crate::core::state::StateFlags;
use crate::terminal::Terminal;

use super::edit_window::EditWindow;
use super::editor_traits::{confirm_save_on_close, Editor, ExternalState, FileEditor};
use super::view::View;

pub struct FileEditorWindow {
    edit_window: EditWindow,
    filename: Option<PathBuf>,
    /// mtime captured at the last successful load/save, used by
    /// [`FileEditor::poll_external_changes`] to detect outside edits.
    last_mtime: Option<SystemTime>,
}

impl FileEditorWindow {
    pub fn new(bounds: Rect, title: &str) -> Self {
        Self {
            edit_window: EditWindow::new(bounds, title),
            filename: None,
            last_mtime: None,
        }
    }

    /// Update the window's title bar from the current filename.
    pub fn refresh_title(&mut self) {
        let title = self.display_name();
        self.edit_window.set_title(&title);
    }

    pub fn set_text(&mut self, text: &str) {
        self.edit_window.editor_rc().borrow_mut().set_text(text);
    }

    pub fn edit_window(&self) -> &EditWindow {
        &self.edit_window
    }

    pub fn edit_window_mut(&mut self) -> &mut EditWindow {
        &mut self.edit_window
    }
}

impl Editor for FileEditorWindow {
    fn valid_close(&mut self, app: &mut Application, command: CommandId) -> bool {
        confirm_save_on_close(self, app, command)
    }

    fn undo(&mut self) {
        self.edit_window.editor_rc().borrow_mut().undo();
    }

    fn redo(&mut self) {
        self.edit_window.editor_rc().borrow_mut().redo();
    }

    fn can_undo(&self) -> bool {
        self.edit_window.editor_rc().borrow().can_undo()
    }

    fn can_redo(&self) -> bool {
        self.edit_window.editor_rc().borrow().can_redo()
    }

    fn cut(&mut self) -> bool {
        self.edit_window.editor_rc().borrow_mut().clip_cut()
    }

    fn copy(&mut self) -> bool {
        self.edit_window.editor_rc().borrow_mut().clip_copy()
    }

    fn paste(&mut self) -> bool {
        self.edit_window.editor_rc().borrow_mut().clip_paste()
    }

    fn select_all(&mut self) {
        self.edit_window.editor_rc().borrow_mut().select_all();
    }

    fn clear_selection(&mut self) {
        self.edit_window.editor_rc().borrow_mut().delete_selection();
    }

    fn has_selection(&self) -> bool {
        self.edit_window.editor_rc().borrow().has_selection()
    }
}

impl FileEditor for FileEditorWindow {
    fn file_path(&self) -> Option<PathBuf> {
        self.filename.clone()
    }

    fn set_file_path(&mut self, path: Option<PathBuf>) {
        self.filename = path;
    }

    fn is_dirty(&self) -> bool {
        self.edit_window.is_modified()
    }

    fn save(&mut self) -> std::io::Result<()> {
        match self.filename.clone() {
            Some(path) => {
                self.edit_window.save_file()?;
                self.last_mtime = read_mtime(&path);
                Ok(())
            }
            None => Err(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "no filename set; use save_as",
            )),
        }
    }

    fn save_as(&mut self, path: PathBuf) -> std::io::Result<()> {
        self.edit_window.save_as(&path)?;
        self.last_mtime = read_mtime(&path);
        self.filename = Some(path);
        Ok(())
    }

    fn load(&mut self, path: PathBuf) -> std::io::Result<()> {
        self.edit_window.load_file(&path)?;
        self.last_mtime = read_mtime(&path);
        self.filename = Some(path);
        Ok(())
    }

    fn new_buffer(&mut self) {
        self.edit_window.editor_rc().borrow_mut().set_text("");
        self.edit_window.editor_rc().borrow_mut().clear_modified();
        self.filename = None;
        self.last_mtime = None;
    }

    fn last_known_mtime(&self) -> Option<SystemTime> {
        self.last_mtime
    }

    fn poll_external_changes(&self) -> ExternalState {
        let Some(path) = self.filename.as_ref() else {
            return ExternalState::NoFile;
        };
        match read_mtime(path) {
            Some(disk) => match self.last_mtime {
                Some(known) if disk == known => ExternalState::Unchanged,
                Some(_) | None => ExternalState::Modified,
            },
            None => ExternalState::Deleted,
        }
    }

    fn reload(&mut self) -> std::io::Result<()> {
        let Some(path) = self.filename.clone() else {
            return Err(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "no filename to reload",
            ));
        };
        self.edit_window.load_file(&path)?;
        self.last_mtime = read_mtime(&path);
        Ok(())
    }

    fn prompt_save_as(&mut self, app: &mut Application) -> bool {
        use super::file_dialog::FileDialogBuilder;

        let (tw, th) = app.terminal.size();
        let dw = 64i16.min(tw as i16 - 4);
        let dh = 18i16.min(th as i16 - 4);
        let x = ((tw as i16) - dw) / 2;
        let y = ((th as i16) - dh) / 2;
        let bounds = Rect::new(x, y, x + dw, y + dh);
        let mut dialog = FileDialogBuilder::new()
            .bounds(bounds)
            .title("Save As")
            .wildcard("*")
            .button_label("~S~ave")
            .build();
        match dialog.execute(app) {
            Some(path) => self.save_as(path).is_ok(),
            None => false,
        }
    }
}

impl View for FileEditorWindow {
    fn bounds(&self) -> Rect {
        self.edit_window.bounds()
    }

    fn set_bounds(&mut self, bounds: Rect) {
        self.edit_window.set_bounds(bounds);
    }

    fn draw(&mut self, terminal: &mut Terminal) {
        self.edit_window.draw(terminal);
    }

    fn handle_event(&mut self, event: &mut Event) {
        self.edit_window.handle_event(event);
    }

    fn can_focus(&self) -> bool {
        self.edit_window.can_focus()
    }

    fn options(&self) -> u16 {
        self.edit_window.options()
    }

    fn set_options(&mut self, options: u16) {
        self.edit_window.set_options(options);
    }

    fn state(&self) -> StateFlags {
        self.edit_window.state()
    }

    fn set_state(&mut self, state: StateFlags) {
        self.edit_window.set_state(state);
    }

    fn get_palette(&self) -> Option<crate::core::palette::Palette> {
        self.edit_window.get_palette()
    }

    fn set_palette_chain(&mut self, node: Option<crate::core::palette_chain::PaletteChainNode>) {
        self.edit_window.set_palette_chain(node);
    }

    fn get_palette_chain(&self) -> Option<&crate::core::palette_chain::PaletteChainNode> {
        self.edit_window.get_palette_chain()
    }
}

/// Builder for creating file editors with a fluent API.
pub struct FileEditorBuilder {
    bounds: Option<Rect>,
    title: String,
}

impl FileEditorBuilder {
    pub fn new() -> Self {
        Self {
            bounds: None,
            title: "Untitled".to_string(),
        }
    }

    #[must_use]
    pub fn bounds(mut self, bounds: Rect) -> Self {
        self.bounds = Some(bounds);
        self
    }

    #[must_use]
    pub fn title(mut self, title: &str) -> Self {
        self.title = title.to_string();
        self
    }

    pub fn build(self) -> FileEditorWindow {
        let bounds = self.bounds.expect("FileEditorWindow bounds must be set");
        FileEditorWindow::new(bounds, &self.title)
    }

    pub fn build_boxed(self) -> Box<FileEditorWindow> {
        Box::new(self.build())
    }
}

impl Default for FileEditorBuilder {
    fn default() -> Self {
        Self::new()
    }
}

fn read_mtime(path: &std::path::Path) -> Option<SystemTime> {
    std::fs::metadata(path).ok().and_then(|m| m.modified().ok())
}
