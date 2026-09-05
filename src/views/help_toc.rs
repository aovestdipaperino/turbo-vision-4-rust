// (C) 2025 - Enzo Lombardi

//! Help Table of Contents - hierarchical topic browser
//!
//! Matches Borland: THelpToc
//!
//! Provides a tree view of help topics organized hierarchically.

use super::ViewId;
use super::button::Button;
use super::dialog::Dialog;
use super::group::{Group, GroupLike};
use super::help_file::HelpFile;
use super::outline::{Node, OutlineViewer};
use super::static_text::StaticText;
use super::window::{Window, WindowLike};
use crate::core::command::{CM_CANCEL, CM_OK};
use crate::core::event::Event;
use crate::core::geometry::Rect;
use std::cell::RefCell;
use std::rc::Rc;

/// Help Table of Contents
/// Matches Borland: THelpToc
pub struct HelpToc {
    dialog: Dialog,
    _outline_viewer_id: ViewId,
    _help_file: Rc<RefCell<HelpFile>>,
    selected_topic: Option<String>,
}

impl HelpToc {
    /// Create a new help table of contents dialog
    pub fn new(bounds: Rect, title: &str, help_file: Rc<RefCell<HelpFile>>) -> Self {
        let mut dialog = Dialog::new(bounds, title);

        // Instructions
        dialog.add(StaticText::new(
            Rect::new(2, 2, bounds.width() - 4, 3),
            "Browse help topics:",
        ));

        // Create outline viewer with topic tree
        let mut outline = OutlineViewer::new(
            Rect::new(2, 4, bounds.width() - 4, bounds.height() - 6),
            |title: &String| title.clone(),
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

        let outline_viewer_id = dialog.add(outline);

        // Buttons
        dialog.add(Button::new(
            Rect::new(
                bounds.width() - 24,
                bounds.height() - 4,
                bounds.width() - 14,
                bounds.height() - 2,
            ),
            "View",
            CM_OK,
            true,
        ));

        dialog.add(Button::new(
            Rect::new(
                bounds.width() - 12,
                bounds.height() - 4,
                bounds.width() - 2,
                bounds.height() - 2,
            ),
            "Close",
            CM_CANCEL,
            false,
        ));

        Self {
            dialog,
            _outline_viewer_id: outline_viewer_id,
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

impl GroupLike for HelpToc {
    fn group(&self) -> &Group {
        self.dialog.group()
    }
    fn group_mut(&mut self) -> &mut Group {
        self.dialog.group_mut()
    }
}

impl WindowLike for HelpToc {
    fn window(&self) -> &Window {
        self.dialog.window()
    }
    fn window_mut(&mut self) -> &mut Window {
        self.dialog.window_mut()
    }
}

crate::impl_view_for_window!(HelpToc {
    fn handle_event(&mut self, event: &mut Event) {
        self.dialog.handle_event(event);
    }

    fn get_palette(&self) -> Option<crate::core::palette::Palette> {
        self.dialog.get_palette()
    }

    fn valid(&mut self, command: crate::core::command::CommandId) -> bool {
        self.dialog.valid(command)
    }
});

/// Builder for creating help TOC dialogs with a fluent API.
pub struct HelpTocBuilder {
    bounds: Option<Rect>,
    title: Option<String>,
    help_file: Option<Rc<RefCell<HelpFile>>>,
}

impl HelpTocBuilder {
    pub fn new() -> Self {
        Self {
            bounds: None,
            title: None,
            help_file: None,
        }
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
