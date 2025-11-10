// (C) 2025 - Enzo Lombardi

//! Help Table of Contents - hierarchical topic browser
//!
//! Matches Borland: THelpToc
//!
//! Provides a tree view of help topics organized hierarchically.

use crate::core::geometry::Rect;
use crate::core::event::Event;
use crate::core::command::{CM_OK, CM_CANCEL};
use crate::terminal::Terminal;
use super::dialog::Dialog;
use super::outline::{OutlineViewer, Node};
use super::button::Button;
use super::static_text::StaticText;
use super::View;
use super::help_file::HelpFile;
use std::rc::Rc;
use std::cell::RefCell;

/// Help Table of Contents
/// Matches Borland: THelpToc
pub struct HelpToc {
    dialog: Dialog,
    _outline_viewer_idx: usize,
    _help_file: Rc<RefCell<HelpFile>>,
    selected_topic: Option<String>,
}

impl HelpToc {
    /// Create a new help table of contents dialog
    pub fn new(bounds: Rect, title: &str, help_file: Rc<RefCell<HelpFile>>) -> Self {
        let mut dialog = Dialog::new(bounds, title);

        // Instructions
        dialog.add(Box::new(StaticText::new(
            Rect::new(2, 2, bounds.width() - 4, 3),
            "Browse help topics:"
        )));

        // Create outline viewer with topic tree
        let mut outline = OutlineViewer::new(
            Rect::new(2, 4, bounds.width() - 4, bounds.height() - 6),
            |title: &String| title.clone()
        );

        // Build topic tree from help file
        let help = help_file.borrow();
        let topic_ids = help.get_topic_ids();

        // For simplicity, create a flat list of topics
        // A real implementation could organize by category/hierarchy
        for id in topic_ids {
            if let Some(topic) = help.get_topic(&id) {
                let node = Rc::new(RefCell::new(Node::new(topic.title.clone())));
                outline.add_root(node);
            }
        }
        drop(help);

        let outline_viewer_idx = dialog.add(Box::new(outline));

        // Buttons
        dialog.add(Box::new(Button::new(
            Rect::new(bounds.width() - 24, bounds.height() - 4, bounds.width() - 14, bounds.height() - 2),
            "View",
            CM_OK,
            true
        )));

        dialog.add(Box::new(Button::new(
            Rect::new(bounds.width() - 12, bounds.height() - 4, bounds.width() - 2, bounds.height() - 2),
            "Close",
            CM_CANCEL,
            false
        )));

        Self {
            dialog,
            _outline_viewer_idx: outline_viewer_idx,
            _help_file: help_file,
            selected_topic: None,
        }
    }

    /// Execute the dialog modally
    /// Returns the selected topic title if View was pressed, None if closed
    pub fn execute(&mut self, app: &mut crate::app::Application) -> Option<String> {
        let result = self.dialog.execute(app);

        if result == CM_OK {
            // Get selected node from outline viewer
            // For now, return None (would need outline viewer access)
            // TODO: Access outline viewer to get selected node
            self.selected_topic.clone()
        } else {
            None
        }
    }

    /// Get the selected topic
    pub fn get_selected_topic(&self) -> Option<String> {
        self.selected_topic.clone()
    }
}

impl View for HelpToc {
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

/// Builder for creating help TOC dialogs with a fluent API.
pub struct HelpTocBuilder {
    bounds: Option<Rect>,
    title: Option<String>,
    help_file: Option<Rc<RefCell<HelpFile>>>,
}

impl HelpTocBuilder {
    pub fn new() -> Self {
        Self { bounds: None, title: None, help_file: None }
    }

    #[must_use]
    pub fn bounds(mut self, bounds: Rect) -> Self {
        self.bounds = Some(bounds);
        self
    }

    #[must_use]
    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
        self
    }

    #[must_use]
    pub fn help_file(mut self, help_file: Rc<RefCell<HelpFile>>) -> Self {
        self.help_file = Some(help_file);
        self
    }

    pub fn build(self) -> HelpToc {
        let bounds = self.bounds.expect("HelpToc bounds must be set");
        let title = self.title.expect("HelpToc title must be set");
        let help_file = self.help_file.expect("HelpToc help_file must be set");
        HelpToc::new(bounds, &title, help_file)
    }

    pub fn build_boxed(self) -> Box<HelpToc> {
        Box::new(self.build())
    }
}

impl Default for HelpTocBuilder {
    fn default() -> Self {
        Self::new()
    }
}
