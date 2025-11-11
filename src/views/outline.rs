// (C) 2025 - Enzo Lombardi

//! Outline view - generic hierarchical tree control for displaying expandable/collapsible data.
//!
//! Matches Borland: TOutline, TOutlineViewer, TNode (from Turbo Vision Professional)
//!
//! Provides a hierarchical tree view with:
//! - Expand/collapse nodes
//! - Visual tree structure (├─, └─, │)
//! - Keyboard navigation
//! - Custom node data

use crate::core::geometry::Rect;
use crate::core::event::{Event, EventType, KB_ENTER, KB_LEFT, KB_RIGHT};
use crate::core::state::StateFlags;
use crate::core::draw::DrawBuffer;
use crate::terminal::Terminal;
use super::view::{View, write_line_to_terminal};
use super::list_viewer::{ListViewer, ListViewerState};
use std::rc::Rc;
use std::cell::RefCell;

/// A node in the tree
/// Matches Borland: TNode
pub struct Node<T> {
    /// Node data (user-provided)
    pub data: T,
    /// Child nodes
    pub children: Vec<Rc<RefCell<Node<T>>>>,
    /// Whether this node is expanded (showing children)
    pub expanded: bool,
}

impl<T> Node<T> {
    /// Create a new leaf node (no children)
    pub fn new(data: T) -> Self {
        Self {
            data,
            children: Vec::new(),
            expanded: false,
        }
    }

    /// Create a new node with children
    pub fn with_children(data: T, children: Vec<Rc<RefCell<Node<T>>>>) -> Self {
        Self {
            data,
            children,
            expanded: false,
        }
    }

    /// Add a child node
    pub fn add_child(&mut self, child: Rc<RefCell<Node<T>>>) {
        self.children.push(child);
    }

    /// Check if this node has children
    pub fn has_children(&self) -> bool {
        !self.children.is_empty()
    }

    /// Toggle expanded state
    pub fn toggle(&mut self) {
        self.expanded = !self.expanded;
    }
}

/// Flattened node for display (with nesting level and display text)
struct DisplayNode {
    /// Display text
    text: String,
    /// Nesting level (0 = root)
    level: usize,
    /// Node index in the original tree (for lookup)
    node_index: usize,
    /// Whether this node has children
    has_children: bool,
    /// Whether this node is expanded
    expanded: bool,
    /// Whether this is the last child at its level
    is_last: bool,
    /// Track which parent levels continue (for vertical lines)
    parent_continues: Vec<bool>,
}

impl DisplayNode {
    /// Format with tree characters
    fn display_text(&self) -> String {
        let mut result = String::new();

        // Add vertical lines for parent levels
        for i in 0..self.level {
            if i < self.parent_continues.len() && self.parent_continues[i] {
                result.push_str("│ ");
            } else {
                result.push_str("  ");
            }
        }

        // Add branch and expansion indicator
        if self.level > 0 {
            if self.is_last {
                result.push_str("└─");
            } else {
                result.push_str("├─");
            }
        }

        // Add expansion indicator
        if self.has_children {
            if self.expanded {
                result.push_str("[-] ");
            } else {
                result.push_str("[+] ");
            }
        }

        result.push_str(&self.text);
        result
    }
}

/// OutlineViewer - displays a hierarchical tree of nodes
/// Matches Borland: TOutlineViewer
pub struct OutlineViewer<T> {
    bounds: Rect,
    state: StateFlags,
    /// Root nodes
    roots: Vec<Rc<RefCell<Node<T>>>>,
    /// Flattened list for display
    display_nodes: Vec<DisplayNode>,
    /// All nodes (flattened)
    all_nodes: Vec<Rc<RefCell<Node<T>>>>,
    /// List viewer state for scrolling/selection
    list_state: ListViewerState,
    /// Function to convert data to display string
    format_fn: Box<dyn Fn(&T) -> String>,
    owner: Option<*const dyn View>,
    owner_type: super::view::OwnerType,
}

impl<T: 'static> OutlineViewer<T> {
    /// Create a new outline viewer
    /// format_fn converts node data to display string
    pub fn new<F>(bounds: Rect, format_fn: F) -> Self
    where
        F: Fn(&T) -> String + 'static,
    {
        Self {
            bounds,
            state: 0,
            roots: Vec::new(),
            display_nodes: Vec::new(),
            all_nodes: Vec::new(),
            list_state: ListViewerState::new(),
            format_fn: Box::new(format_fn),
            owner: None,
            owner_type: super::view::OwnerType::None,
        }
    }

    /// Set the root nodes of the tree
    pub fn set_roots(&mut self, roots: Vec<Rc<RefCell<Node<T>>>>) {
        self.roots = roots;
        self.rebuild_display();
    }

    /// Add a root node
    pub fn add_root(&mut self, root: Rc<RefCell<Node<T>>>) {
        self.roots.push(root);
        self.rebuild_display();
    }

    /// Rebuild the flattened display list
    fn rebuild_display(&mut self) {
        self.display_nodes.clear();
        self.all_nodes.clear();

        // Clone roots to avoid borrow checker issues
        let roots = self.roots.clone();
        for (i, root) in roots.iter().enumerate() {
            let is_last_root = i == roots.len() - 1;
            self.flatten_node(root.clone(), 0, is_last_root, &mut Vec::new());
        }

        // Update list state with new count
        self.list_state.set_range(self.display_nodes.len());
    }

    /// Recursively flatten a node and its visible children
    fn flatten_node(
        &mut self,
        node: Rc<RefCell<Node<T>>>,
        level: usize,
        is_last: bool,
        parent_continues: &mut Vec<bool>,
    ) {
        let node_index = self.all_nodes.len();
        self.all_nodes.push(node.clone());

        let node_borrow = node.borrow();
        let text = (self.format_fn)(&node_borrow.data);
        let has_children = node_borrow.has_children();
        let expanded = node_borrow.expanded;

        self.display_nodes.push(DisplayNode {
            text,
            level,
            node_index,
            has_children,
            expanded,
            is_last,
            parent_continues: parent_continues.clone(),
        });

        // If expanded, show children
        if expanded && has_children {
            let children_len = node_borrow.children.len();
            drop(node_borrow); // Release borrow before recursing

            // Track which parent levels have more siblings
            if level > 0 {
                parent_continues.push(!is_last);
            }

            for (i, child) in node.borrow().children.iter().enumerate() {
                let child_is_last = i == children_len - 1;
                self.flatten_node(child.clone(), level + 1, child_is_last, parent_continues);
            }

            if level > 0 {
                parent_continues.pop();
            }
        }
    }

    /// Get the currently selected node
    pub fn selected_node(&self) -> Option<Rc<RefCell<Node<T>>>> {
        if let Some(focused) = self.list_state.focused {
            if focused < self.display_nodes.len() {
                let node_index = self.display_nodes[focused].node_index;
                if node_index < self.all_nodes.len() {
                    return Some(self.all_nodes[node_index].clone());
                }
            }
        }
        None
    }

    /// Expand the currently selected node
    fn expand_selected(&mut self) {
        if let Some(node) = self.selected_node() {
            let mut node_borrow = node.borrow_mut();
            if node_borrow.has_children() && !node_borrow.expanded {
                node_borrow.expanded = true;
                drop(node_borrow);
                self.rebuild_display();
            }
        }
    }

    /// Collapse the currently selected node
    fn collapse_selected(&mut self) {
        if let Some(node) = self.selected_node() {
            let mut node_borrow = node.borrow_mut();
            if node_borrow.has_children() && node_borrow.expanded {
                node_borrow.expanded = false;
                drop(node_borrow);
                self.rebuild_display();
            }
        }
    }

    /// Toggle expand/collapse on the currently selected node
    fn toggle_selected(&mut self) {
        if let Some(node) = self.selected_node() {
            let mut node_borrow = node.borrow_mut();
            if node_borrow.has_children() {
                node_borrow.toggle();
                drop(node_borrow);
                self.rebuild_display();
            }
        }
    }
}

impl<T: 'static> View for OutlineViewer<T> {
    fn bounds(&self) -> Rect {
        self.bounds
    }

    fn set_bounds(&mut self, bounds: Rect) {
        self.bounds = bounds;
    }

    fn draw(&mut self, terminal: &mut Terminal) {
        let width = self.bounds.width() as usize;
        let height = self.bounds.height() as usize;

        use crate::core::palette::colors::{LISTBOX_FOCUSED, LISTBOX_NORMAL, LISTBOX_SELECTED_FOCUSED, LISTBOX_SELECTED};
        let color_normal = if self.is_focused() {
            LISTBOX_FOCUSED
        } else {
            LISTBOX_NORMAL
        };
        let color_selected = if self.is_focused() {
            LISTBOX_SELECTED_FOCUSED
        } else {
            LISTBOX_SELECTED
        };

        // Draw visible items
        for i in 0..height {
            let mut buf = DrawBuffer::new(width);
            let item_idx = self.list_state.top_item + i;

            if item_idx < self.display_nodes.len() {
                let is_selected = Some(item_idx) == self.list_state.focused;
                let color = if is_selected { color_selected } else { color_normal };

                let display_node = &self.display_nodes[item_idx];
                let text = display_node.display_text();
                buf.move_str(0, &text, color);

                // Fill rest of line with spaces
                let text_len = text.len();
                if text_len < width {
                    buf.move_char(text_len, ' ', color, width - text_len);
                }
            } else {
                // Empty line
                buf.move_char(0, ' ', color_normal, width);
            }

            write_line_to_terminal(terminal, self.bounds.a.x, self.bounds.a.y + i as i16, &buf);
        }
    }

    fn handle_event(&mut self, event: &mut Event) {
        // Handle expand/collapse keys first
        if event.what == EventType::Keyboard {
            match event.key_code {
                KB_RIGHT => {
                    self.expand_selected();
                    event.clear();
                    return;
                }
                KB_LEFT => {
                    self.collapse_selected();
                    event.clear();
                    return;
                }
                KB_ENTER => {
                    self.toggle_selected();
                    event.clear();
                    return;
                }
                _ => {}
            }
        }

        // Let list viewer handle standard navigation (arrows, page up/down, etc.)
        self.handle_list_event(event);
    }

    fn state(&self) -> StateFlags {
        self.state
    }

    fn set_state(&mut self, state: StateFlags) {
        self.state = state;
    }

    fn can_focus(&self) -> bool {
        true
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

    fn get_owner_type(&self) -> super::view::OwnerType {
        self.owner_type
    }

    fn set_owner_type(&mut self, owner_type: super::view::OwnerType) {
        self.owner_type = owner_type;
    }
}

// Implement ListViewer trait to get standard list navigation
impl<T: 'static> ListViewer for OutlineViewer<T> {
    fn list_state(&self) -> &ListViewerState {
        &self.list_state
    }

    fn list_state_mut(&mut self) -> &mut ListViewerState {
        &mut self.list_state
    }

    fn get_text(&self, item: usize, _max_len: usize) -> String {
        if item < self.display_nodes.len() {
            self.display_nodes[item].display_text()
        } else {
            String::new()
        }
    }
}
