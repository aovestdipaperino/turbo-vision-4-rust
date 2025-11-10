// (C) 2025 - Enzo Lombardi

//! FileList view - specialized list viewer for displaying and selecting files.
// FileList - Specialized list viewer for files
//
// Matches Borland: TFileList (views/tfillist.cc)
//
// A list viewer that displays files with attributes and optional icons.
// Designed to be used in file dialogs or as a standalone file browser.
//
// Features:
// - Displays files with visual indicators (directories shown as [dirname])
// - Supports wildcard filtering (*.rs, *.txt, etc.)
// - Parent directory (..) navigation
// - File info display (size, date, attributes)
// - Integrates with ListViewer trait for consistent navigation

use crate::core::geometry::Rect;
use crate::core::event::{Event, EventType};
use crate::core::state::StateFlags;
use crate::terminal::Terminal;
use super::view::View;
use super::list_viewer::{ListViewer, ListViewerState};
use std::path::{Path, PathBuf};
use std::fs;
use std::time::SystemTime;

/// File entry information
#[derive(Clone, Debug)]
pub struct FileEntry {
    pub name: String,
    pub path: PathBuf,
    pub is_dir: bool,
    pub size: u64,
    pub modified: Option<SystemTime>,
}

impl FileEntry {
    /// Create from a directory entry
    pub fn from_dir_entry(entry: &fs::DirEntry) -> std::io::Result<Self> {
        let metadata = entry.metadata()?;
        let name = entry.file_name().to_string_lossy().to_string();
        let path = entry.path();
        let is_dir = metadata.is_dir();
        let size = metadata.len();
        let modified = metadata.modified().ok();

        Ok(Self {
            name,
            path,
            is_dir,
            size,
            modified,
        })
    }

    /// Format for display in list
    pub fn display_name(&self) -> String {
        if self.is_dir {
            format!("[{}]", self.name)
        } else {
            self.name.clone()
        }
    }

    /// Get file size as human-readable string
    pub fn size_string(&self) -> String {
        if self.is_dir {
            "<DIR>".to_string()
        } else if self.size < 1024 {
            format!("{} B", self.size)
        } else if self.size < 1024 * 1024 {
            format!("{} KB", self.size / 1024)
        } else if self.size < 1024 * 1024 * 1024 {
            format!("{} MB", self.size / (1024 * 1024))
        } else {
            format!("{} GB", self.size / (1024 * 1024 * 1024))
        }
    }
}

/// FileList - Displays files and directories
///
/// Matches Borland: TFileList
pub struct FileList {
    bounds: Rect,
    state: StateFlags,
    list_state: ListViewerState,
    files: Vec<FileEntry>,
    current_path: PathBuf,
    wildcard: String,
    show_hidden: bool,
    owner: Option<*const dyn View>,
}

impl FileList {
    /// Create a new file list
    pub fn new(bounds: Rect, path: &Path) -> Self {
        Self {
            bounds,
            state: 0,
            list_state: ListViewerState::new(),
            files: Vec::new(),
            current_path: path.to_path_buf(),
            wildcard: "*".to_string(),
            show_hidden: false,
            owner: None,
        }
    }

    /// Set the wildcard filter (e.g., "*.rs", "*.txt")
    pub fn set_wildcard(&mut self, wildcard: &str) {
        self.wildcard = wildcard.to_string();
        self.refresh();
    }

    /// Set whether to show hidden files
    pub fn set_show_hidden(&mut self, show: bool) {
        self.show_hidden = show;
        self.refresh();
    }

    /// Get current path
    pub fn current_path(&self) -> &Path {
        &self.current_path
    }

    /// Change to a different directory
    pub fn change_dir(&mut self, path: &Path) -> std::io::Result<()> {
        let canonical = fs::canonicalize(path)?;
        self.current_path = canonical;
        self.refresh();
        Ok(())
    }

    /// Refresh the file list
    pub fn refresh(&mut self) {
        self.files.clear();

        // Add parent directory entry if not at root
        if self.current_path.parent().is_some() {
            self.files.push(FileEntry {
                name: "..".to_string(),
                path: self.current_path.parent().unwrap().to_path_buf(),
                is_dir: true,
                size: 0,
                modified: None,
            });
        }

        // Read directory entries
        if let Ok(entries) = fs::read_dir(&self.current_path) {
            let mut file_entries: Vec<FileEntry> = entries
                .filter_map(|e| e.ok())
                .filter_map(|e| FileEntry::from_dir_entry(&e).ok())
                .filter(|entry| {
                    // Filter hidden files
                    if !self.show_hidden && entry.name.starts_with('.') {
                        return false;
                    }
                    // Always show directories
                    if entry.is_dir {
                        return true;
                    }
                    // Filter files by wildcard
                    self.matches_wildcard(&entry.name)
                })
                .collect();

            // Sort: directories first, then files, both alphabetically
            file_entries.sort_by(|a, b| {
                match (a.is_dir, b.is_dir) {
                    (true, false) => std::cmp::Ordering::Less,
                    (false, true) => std::cmp::Ordering::Greater,
                    _ => a.name.to_lowercase().cmp(&b.name.to_lowercase()),
                }
            });

            self.files.extend(file_entries);
        }

        // Update list state
        self.list_state.set_range(self.files.len());
        if let Some(focused) = self.list_state.focused {
            if focused >= self.files.len() {
                self.list_state.focused = Some(0);
            }
        }
    }

    /// Check if filename matches wildcard pattern
    fn matches_wildcard(&self, filename: &str) -> bool {
        if self.wildcard == "*" {
            return true;
        }

        // Simple wildcard matching: "*.ext" -> ends with ".ext"
        if let Some(pattern) = self.wildcard.strip_prefix('*') {
            return filename.ends_with(pattern);
        }

        // Exact match
        filename == self.wildcard
    }

    /// Get the currently focused file entry
    pub fn get_focused_entry(&self) -> Option<&FileEntry> {
        let idx = self.list_state.focused?;
        self.files.get(idx)
    }

    /// Get the selected file path (returns None if directory is selected)
    pub fn get_selected_file(&self) -> Option<PathBuf> {
        let entry = self.get_focused_entry()?;
        if entry.is_dir {
            None
        } else {
            Some(entry.path.clone())
        }
    }

    /// Navigate into the focused directory
    pub fn enter_focused_dir(&mut self) -> std::io::Result<bool> {
        let path = if let Some(entry) = self.get_focused_entry() {
            if entry.is_dir {
                Some(entry.path.clone())
            } else {
                None
            }
        } else {
            None
        };

        if let Some(path) = path {
            self.change_dir(&path)?;
            Ok(true)
        } else {
            Ok(false)
        }
    }

    /// Get file count
    pub fn file_count(&self) -> usize {
        self.files.len()
    }
}

impl ListViewer for FileList {
    fn list_state(&self) -> &ListViewerState {
        &self.list_state
    }

    fn list_state_mut(&mut self) -> &mut ListViewerState {
        &mut self.list_state
    }

    fn get_text(&self, item: usize, _max_len: usize) -> String {
        self.files
            .get(item)
            .map(|f| f.display_name())
            .unwrap_or_default()
    }
}

impl View for FileList {
    fn bounds(&self) -> Rect {
        self.bounds
    }

    fn set_bounds(&mut self, bounds: Rect) {
        self.bounds = bounds;
    }

    fn draw(&mut self, terminal: &mut Terminal) {
        let width = self.bounds.width() as usize;
        let height = self.bounds.height() as usize;

        self.list_state.set_range(self.files.len());

        for y in 0..height {
            let item_idx = self.list_state.top_item + y;

            let (text, color) = if item_idx < self.files.len() {
                let text = self.get_text(item_idx, width);
                let is_focused = self.is_focused() && Some(item_idx) == self.list_state.focused;
                let color = if is_focused {
                    crate::core::palette::colors::LISTBOX_FOCUSED
                } else {
                    crate::core::palette::colors::LISTBOX_NORMAL
                };
                (text, color)
            } else {
                (String::new(), crate::core::palette::colors::LISTBOX_NORMAL)
            };

            let padded = format!("{:width$}", text, width = width);

            for (x, ch) in padded.chars().take(width).enumerate() {
                terminal.write_cell(
                    (self.bounds.a.x + x as i16) as u16,
                    (self.bounds.a.y + y as i16) as u16,
                    crate::core::draw::Cell::new(ch, color),
                );
            }
        }
    }

    fn handle_event(&mut self, event: &mut Event) {
        if !self.is_focused() {
            return;
        }

        // Use default ListViewer navigation
        self.handle_list_event(event);

        // Handle Enter to navigate directories
        if event.what == EventType::Keyboard && event.key_code == crate::core::event::KB_ENTER {
            let _ = self.enter_focused_dir();
            event.clear();
        }
    }

    fn can_focus(&self) -> bool {
        true
    }

    fn state(&self) -> StateFlags {
        self.state
    }

    fn set_state(&mut self, state: StateFlags) {
        self.state = state;
    }

    fn set_owner(&mut self, owner: *const dyn View) {
        self.owner = Some(owner);
    }

    fn get_owner(&self) -> Option<*const dyn View> {
        self.owner
    }

    fn get_palette(&self) -> Option<crate::core::palette::Palette> {
        use crate::core::palette::{palettes, Palette};
        Some(Palette::from_slice(palettes::CP_LISTBOX))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    #[test]
    fn test_file_list_creation() {
        let bounds = Rect::new(0, 0, 40, 10);
        let path = env::current_dir().unwrap();
        let mut list = FileList::new(bounds, &path);
        list.refresh();

        assert!(list.file_count() > 0, "Should find at least some files");
        assert_eq!(list.current_path(), path.as_path());
    }

    #[test]
    fn test_wildcard_filtering() {
        let bounds = Rect::new(0, 0, 40, 10);
        let path = env::current_dir().unwrap();
        let mut list = FileList::new(bounds, &path);

        // Filter for Rust files
        list.set_wildcard("*.rs");

        // All non-directory entries should be .rs files
        for entry in &list.files {
            if !entry.is_dir {
                assert!(entry.name.ends_with(".rs"), "Should only show .rs files");
            }
        }
    }

    #[test]
    fn test_file_entry_display() {
        let entry = FileEntry {
            name: "test.txt".to_string(),
            path: PathBuf::from("test.txt"),
            is_dir: false,
            size: 1024,
            modified: None,
        };
        assert_eq!(entry.display_name(), "test.txt");

        let dir_entry = FileEntry {
            name: "mydir".to_string(),
            path: PathBuf::from("mydir"),
            is_dir: true,
            size: 0,
            modified: None,
        };
        assert_eq!(dir_entry.display_name(), "[mydir]");
    }

    #[test]
    fn test_size_formatting() {
        let small = FileEntry {
            name: "small".to_string(),
            path: PathBuf::from("small"),
            is_dir: false,
            size: 512,
            modified: None,
        };
        assert_eq!(small.size_string(), "512 B");

        let kb = FileEntry {
            name: "kb".to_string(),
            path: PathBuf::from("kb"),
            is_dir: false,
            size: 2048,
            modified: None,
        };
        assert_eq!(kb.size_string(), "2 KB");

        let dir = FileEntry {
            name: "dir".to_string(),
            path: PathBuf::from("dir"),
            is_dir: true,
            size: 0,
            modified: None,
        };
        assert_eq!(dir.size_string(), "<DIR>");
    }
}

/// Builder for creating file lists with a fluent API.
pub struct FileListBuilder {
    bounds: Option<Rect>,
    path: Option<PathBuf>,
}

impl FileListBuilder {
    pub fn new() -> Self {
        Self { bounds: None, path: None }
    }

    #[must_use]
    pub fn bounds(mut self, bounds: Rect) -> Self {
        self.bounds = Some(bounds);
        self
    }

    #[must_use]
    pub fn path(mut self, path: impl Into<PathBuf>) -> Self {
        self.path = Some(path.into());
        self
    }

    pub fn build(self) -> FileList {
        let bounds = self.bounds.expect("FileList bounds must be set");
        let path = self.path.expect("FileList path must be set");
        FileList::new(bounds, &path)
    }

    pub fn build_boxed(self) -> Box<FileList> {
        Box::new(self.build())
    }
}

impl Default for FileListBuilder {
    fn default() -> Self {
        Self::new()
    }
}
