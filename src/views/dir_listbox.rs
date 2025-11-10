// (C) 2025 - Enzo Lombardi

//! DirListBox view - directory tree navigation and selection.
// DirListBox - Directory tree viewer
//
// Matches Borland: TDirListBox (views/tdirlist.cc)
//
// A hierarchical tree view of the directory structure, showing parent
// directories and subdirectories with visual tree indicators.
//
// Features:
// - Hierarchical directory tree display
// - Visual tree structure (├─, └─, │, etc.)
// - Navigate up and down the directory tree
// - Expand/collapse directories
// - Current path tracking
//
// Display format:
//   C:\
//   ├─Users
//   │ ├─alice
//   │ └─bob
//   └─Program Files

use crate::core::geometry::Rect;
use crate::core::event::{Event, EventType, KB_ENTER};
use crate::core::state::StateFlags;
use crate::terminal::Terminal;
use super::view::View;
use super::list_viewer::{ListViewer, ListViewerState};
use std::path::{Path, PathBuf};
use std::fs;

/// Directory entry in the tree
#[derive(Clone, Debug)]
pub struct DirEntry {
    /// Directory name
    pub name: String,
    /// Full path
    pub path: PathBuf,
    /// Nesting level (0 = root)
    pub level: usize,
    /// Whether this is the last child at its level
    pub is_last: bool,
}

impl DirEntry {
    /// Format with tree characters
    fn display_text(&self, parent_continues: &[bool]) -> String {
        let mut result = String::new();

        // Add vertical lines for parent levels
        for i in 0..self.level {
            if i < parent_continues.len() && parent_continues[i] {
                result.push_str("│ ");
            } else {
                result.push_str("  ");
            }
        }

        // Add branch for current level
        if self.level > 0 {
            if self.is_last {
                result.push_str("└─");
            } else {
                result.push_str("├─");
            }
        }

        result.push_str(&self.name);
        result
    }
}

/// DirListBox - Hierarchical directory tree viewer
///
/// Matches Borland: TDirListBox
pub struct DirListBox {
    bounds: Rect,
    state: StateFlags,
    list_state: ListViewerState,
    entries: Vec<DirEntry>,
    current_path: PathBuf,
    root_path: PathBuf,
    owner: Option<*const dyn View>,
}

impl DirListBox {
    /// Create a new directory list box
    pub fn new(bounds: Rect, path: &Path) -> Self {
        let mut dlb = Self {
            bounds,
            state: 0,
            list_state: ListViewerState::new(),
            entries: Vec::new(),
            current_path: path.to_path_buf(),
            root_path: Self::find_root(path),
            owner: None,
        };
        dlb.rebuild_tree();
        dlb
    }

    /// Find the root path (drive root on Windows, / on Unix)
    fn find_root(path: &Path) -> PathBuf {
        let mut current = path;
        while let Some(parent) = current.parent() {
            current = parent;
        }
        current.to_path_buf()
    }

    /// Get the currently selected directory path
    pub fn current_path(&self) -> &Path {
        &self.current_path
    }

    /// Get the focused directory entry
    pub fn get_focused_entry(&self) -> Option<&DirEntry> {
        let idx = self.list_state.focused?;
        self.entries.get(idx)
    }

    /// Navigate to a different directory
    pub fn change_dir(&mut self, path: &Path) -> std::io::Result<()> {
        if path.is_dir() {
            self.current_path = fs::canonicalize(path)?;
            self.rebuild_tree();
            Ok(())
        } else {
            Err(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "Not a directory",
            ))
        }
    }

    /// Rebuild the directory tree from root to current path
    fn rebuild_tree(&mut self) {
        self.entries.clear();

        // Build path from root to current directory
        let mut path_components = Vec::new();
        let mut current = self.current_path.clone();

        while current != self.root_path {
            if let Some(name) = current.file_name() {
                path_components.push((name.to_string_lossy().to_string(), current.clone()));
            }
            if let Some(parent) = current.parent() {
                current = parent.to_path_buf();
            } else {
                break;
            }
        }
        path_components.reverse();

        // Add root
        let root_name = self.root_path.to_string_lossy().to_string();
        self.entries.push(DirEntry {
            name: if root_name.is_empty() {
                "/".to_string()
            } else {
                root_name
            },
            path: self.root_path.clone(),
            level: 0,
            is_last: path_components.is_empty(),
        });

        // Add path components
        for (i, (name, path)) in path_components.iter().enumerate() {
            let is_last = i == path_components.len() - 1;
            self.entries.push(DirEntry {
                name: name.clone(),
                path: path.clone(),
                level: i + 1,
                is_last,
            });
        }

        // Add subdirectories of current directory
        if let Ok(entries) = fs::read_dir(&self.current_path) {
            let mut subdirs: Vec<_> = entries
                .filter_map(|e| e.ok())
                .filter_map(|e| {
                    let path = e.path();
                    if path.is_dir() {
                        Some((e.file_name().to_string_lossy().to_string(), path))
                    } else {
                        None
                    }
                })
                .collect();

            subdirs.sort_by(|a, b| a.0.to_lowercase().cmp(&b.0.to_lowercase()));

            let current_level = path_components.len() + 1;
            for (i, (name, path)) in subdirs.iter().enumerate() {
                let is_last = i == subdirs.len() - 1;
                self.entries.push(DirEntry {
                    name: name.clone(),
                    path: path.clone(),
                    level: current_level,
                    is_last,
                });
            }
        }

        // Update list state
        self.list_state.set_range(self.entries.len());

        // Focus the current directory entry
        if let Some(idx) = self.entries.iter().position(|e| e.path == self.current_path) {
            self.list_state.focused = Some(idx);
        } else {
            self.list_state.focused = Some(0);
        }
    }

    /// Enter the focused directory
    pub fn enter_focused_dir(&mut self) -> std::io::Result<()> {
        if let Some(entry) = self.get_focused_entry() {
            let path = entry.path.clone();
            self.change_dir(&path)?;
        }
        Ok(())
    }

    /// Navigate to parent directory
    pub fn parent_dir(&mut self) -> std::io::Result<()> {
        let parent = self.current_path.parent().map(|p| p.to_path_buf());
        if let Some(parent) = parent {
            self.change_dir(&parent)?;
        }
        Ok(())
    }

    /// Get parent continuation flags for rendering
    fn get_parent_continues(&self, entry: &DirEntry) -> Vec<bool> {
        let mut continues = vec![false; entry.level];

        // Find which parent levels have more siblings after them
        let entry_idx = self.entries.iter().position(|e| e.path == entry.path).unwrap_or(0);

        for i in 0..entry.level {
            // Check if there are more entries at level i after this entry's ancestor at level i
            let has_more = self.entries[entry_idx + 1..]
                .iter()
                .any(|e| e.level == i);
            continues[i] = has_more;
        }

        continues
    }
}

impl ListViewer for DirListBox {
    fn list_state(&self) -> &ListViewerState {
        &self.list_state
    }

    fn list_state_mut(&mut self) -> &mut ListViewerState {
        &mut self.list_state
    }

    fn get_text(&self, item: usize, _max_len: usize) -> String {
        if let Some(entry) = self.entries.get(item) {
            let continues = self.get_parent_continues(entry);
            entry.display_text(&continues)
        } else {
            String::new()
        }
    }
}

impl View for DirListBox {
    fn bounds(&self) -> Rect {
        self.bounds
    }

    fn set_bounds(&mut self, bounds: Rect) {
        self.bounds = bounds;
    }

    fn draw(&mut self, terminal: &mut Terminal) {
        let width = self.bounds.width() as usize;
        let height = self.bounds.height() as usize;

        self.list_state.set_range(self.entries.len());

        for y in 0..height {
            let item_idx = self.list_state.top_item + y;

            let (text, color) = if item_idx < self.entries.len() {
                use crate::core::palette::colors::{LISTBOX_FOCUSED, LISTBOX_NORMAL};
                let text = self.get_text(item_idx, width);
                let is_focused = self.is_focused() && Some(item_idx) == self.list_state.focused;
                let color = if is_focused {
                    LISTBOX_FOCUSED
                } else {
                    LISTBOX_NORMAL
                };
                (text, color)
            } else {
                use crate::core::palette::colors::LISTBOX_NORMAL;
                (String::new(), LISTBOX_NORMAL)
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

        // Handle Enter to navigate into directory
        if event.what == EventType::Keyboard && event.key_code == KB_ENTER {
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
        None  // DirListBox uses hardcoded listbox colors
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    #[test]
    fn test_dir_listbox_creation() {
        let bounds = Rect::new(0, 0, 40, 10);
        let path = env::current_dir().unwrap();
        let dlb = DirListBox::new(bounds, &path);

        assert!(dlb.entries.len() > 0, "Should have at least root entry");
        assert_eq!(dlb.current_path(), path.as_path());
    }

    #[test]
    fn test_find_root() {
        let path = env::current_dir().unwrap();
        let root = DirListBox::find_root(&path);

        // Root should have no parent
        assert!(root.parent().is_none());
    }

    #[test]
    fn test_dir_entry_display() {
        let entry = DirEntry {
            name: "subdir".to_string(),
            path: PathBuf::from("/path/to/subdir"),
            level: 1,
            is_last: false,
        };

        let continues = vec![true];
        let text = entry.display_text(&continues);
        assert!(text.contains("├─") || text.contains("└─"));
        assert!(text.contains("subdir"));
    }

    #[test]
    fn test_parent_navigation() {
        let path = env::current_dir().unwrap();
        let bounds = Rect::new(0, 0, 40, 10);
        let mut dlb = DirListBox::new(bounds, &path);

        let original_path = dlb.current_path().to_path_buf();

        // Try to go to parent
        if original_path.parent().is_some() {
            let result = dlb.parent_dir();
            assert!(result.is_ok());
            assert_ne!(dlb.current_path(), original_path.as_path());
        }
    }
}

/// Builder for creating directory list boxes with a fluent API.
pub struct DirListBoxBuilder {
    bounds: Option<Rect>,
    path: Option<PathBuf>,
}

impl DirListBoxBuilder {
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

    pub fn build(self) -> DirListBox {
        let bounds = self.bounds.expect("DirListBox bounds must be set");
        let path = self.path.expect("DirListBox path must be set");
        DirListBox::new(bounds, &path)
    }

    pub fn build_boxed(self) -> Box<DirListBox> {
        Box::new(self.build())
    }
}

impl Default for DirListBoxBuilder {
    fn default() -> Self {
        Self::new()
    }
}
