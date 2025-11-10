// (C) 2025 - Enzo Lombardi

//! Help Index - searchable index of help topics
//!
//! Matches Borland: THelpIndex
//!
//! Provides a searchable list of all help topics with filtering capability.

use crate::core::geometry::Rect;
use crate::core::event::{Event, EventType};
use crate::core::command::{CM_OK, CM_CANCEL};
use crate::terminal::Terminal;
use super::dialog::Dialog;
use super::input_line::InputLine;
use super::listbox::ListBox;
use super::button::Button;
use super::label::Label;
use super::static_text::StaticText;
use super::View;
use super::help_file::HelpFile;
use std::rc::Rc;
use std::cell::RefCell;

const CMD_TOPIC_SELECTED: u16 = 1002;

/// Help Index - searchable topic list
/// Matches Borland: THelpIndex
pub struct HelpIndex {
    dialog: Dialog,
    _help_file: Rc<RefCell<HelpFile>>,
    #[allow(dead_code)]
    all_topics: Vec<(String, String)>, // (id, title)
    filtered_topics: Vec<(String, String)>,
    _search_input_idx: usize,
    _topic_list_idx: usize,
    selected_topic: Option<String>,
}

impl HelpIndex {
    /// Create a new help index dialog
    pub fn new(bounds: Rect, title: &str, help_file: Rc<RefCell<HelpFile>>) -> Self {
        let mut dialog = Dialog::new(bounds, title);

        // Instructions
        dialog.add(Box::new(StaticText::new(
            Rect::new(2, 2, bounds.width() - 4, 3),
            "Search for a topic:"
        )));

        // Search input
        let search_label = Label::new(Rect::new(2, 4, 10, 5), "Search:");
        dialog.add(Box::new(search_label));

        let search_data = Rc::new(RefCell::new(String::new()));
        let search_input = InputLine::new(Rect::new(10, 4, bounds.width() - 4, 5), 100, search_data.clone());
        let search_input_idx = dialog.add(Box::new(search_input));

        // Topic list
        let list_label = Label::new(Rect::new(2, 6, 12, 7), "Topics:");
        dialog.add(Box::new(list_label));

        let topic_list = ListBox::new(
            Rect::new(2, 7, bounds.width() - 4, bounds.height() - 6),
            CMD_TOPIC_SELECTED
        );
        let topic_list_idx = dialog.add(Box::new(topic_list));

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

        // Get all topics from help file
        let help = help_file.borrow();
        let topic_ids = help.get_topic_ids();

        let mut all_topics = Vec::new();
        for id in topic_ids {
            if let Some(topic) = help.get_topic(&id) {
                all_topics.push((id.clone(), topic.title.clone()));
            }
        }
        drop(help);

        // Sort by title
        all_topics.sort_by(|a, b| a.1.cmp(&b.1));

        let filtered_topics = all_topics.clone();

        let mut index = Self {
            dialog,
            _help_file: help_file,
            all_topics,
            filtered_topics,
            _search_input_idx: search_input_idx,
            _topic_list_idx: topic_list_idx,
            selected_topic: None,
        };

        index.update_topic_list();
        index
    }

    /// Update the topic list based on current filter
    fn update_topic_list(&mut self) {
        // For now, show all topics (filtering not yet implemented)
        // TODO: Add actual search filtering based on search_input text
        let _topic_titles: Vec<String> = self.filtered_topics
            .iter()
            .map(|(_, title)| title.clone())
            .collect();

        // Note: We'd need to access the ListBox child to update it
        // This is a simplified implementation
    }

    /// Filter topics based on search text
    #[allow(dead_code)]
    fn filter_topics(&mut self, search_text: &str) {
        if search_text.is_empty() {
            self.filtered_topics = self.all_topics.clone();
        } else {
            let search_lower = search_text.to_lowercase();
            self.filtered_topics = self.all_topics
                .iter()
                .filter(|(id, title)| {
                    title.to_lowercase().contains(&search_lower) ||
                    id.to_lowercase().contains(&search_lower)
                })
                .cloned()
                .collect();
        }
        self.update_topic_list();
    }

    /// Execute the dialog modally
    /// Returns the selected topic ID if View was pressed, None if closed
    pub fn execute(&mut self, app: &mut crate::app::Application) -> Option<String> {
        let result = self.dialog.execute(app);

        if result == CM_OK && !self.filtered_topics.is_empty() {
            // Get selected index from listbox (simplified - would need actual listbox access)
            // For now, return the first topic
            Some(self.filtered_topics[0].0.clone())
        } else {
            None
        }
    }

    /// Get the selected topic ID
    pub fn get_selected_topic(&self) -> Option<String> {
        self.selected_topic.clone()
    }
}

impl View for HelpIndex {
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
        // Handle topic selection
        if event.what == EventType::Command && event.command == CMD_TOPIC_SELECTED {
            // Topic was double-clicked or Enter pressed
            // Convert this to CM_OK to close the dialog
            *event = Event::command(CM_OK);
        }

        // Let dialog handle events
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

/// Builder for creating help index dialogs with a fluent API.
pub struct HelpIndexBuilder {
    bounds: Option<Rect>,
    title: Option<String>,
    help_file: Option<Rc<RefCell<HelpFile>>>,
}

impl HelpIndexBuilder {
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

    pub fn build(self) -> HelpIndex {
        let bounds = self.bounds.expect("HelpIndex bounds must be set");
        let title = self.title.expect("HelpIndex title must be set");
        let help_file = self.help_file.expect("HelpIndex help_file must be set");
        HelpIndex::new(bounds, &title, help_file)
    }

    pub fn build_boxed(self) -> Box<HelpIndex> {
        Box::new(self.build())
    }
}

impl Default for HelpIndexBuilder {
    fn default() -> Self {
        Self::new()
    }
}
