// HelpFile - Markdown-based help file manager
//
// Modern alternative to Borland's THelpFile (binary format)
//
// Uses markdown files for help content with topic markers:
// # Topic Name {#topic-id}
//
// This provides a maintainable, human-readable alternative to
// Borland's proprietary binary TPH format.

use std::collections::HashMap;
use std::fs;
use std::path::Path;

/// Help topic containing title and content
#[derive(Debug, Clone)]
pub struct HelpTopic {
    /// Topic ID (e.g., "file-open", "edit-copy")
    pub id: String,
    /// Topic title (from markdown heading)
    pub title: String,
    /// Topic content as lines of text
    pub content: Vec<String>,
    /// Cross-references to other topics
    pub links: Vec<String>,
}

impl HelpTopic {
    /// Create a new help topic
    pub fn new(id: String, title: String) -> Self {
        Self {
            id,
            title,
            content: Vec::new(),
            links: Vec::new(),
        }
    }

    /// Add a line to the topic content
    pub fn add_line(&mut self, line: String) {
        self.content.push(line);
    }

    /// Add a cross-reference link
    pub fn add_link(&mut self, topic_id: String) {
        if !self.links.contains(&topic_id) {
            self.links.push(topic_id);
        }
    }

    /// Get formatted content with line numbers for display
    pub fn get_formatted_content(&self) -> Vec<String> {
        let mut lines = vec![
            format!("═══ {} ═══", self.title),
            String::new(),
        ];
        lines.extend(self.content.clone());

        if !self.links.is_empty() {
            lines.push(String::new());
            lines.push("See also:".to_string());
            for link in &self.links {
                lines.push(format!("  → {}", link));
            }
        }

        lines
    }
}

/// HelpFile - Manages markdown help files
///
/// Modern alternative to Borland's THelpFile
pub struct HelpFile {
    /// File path
    path: String,
    /// Topics indexed by ID
    topics: HashMap<String, HelpTopic>,
    /// Default topic to show
    default_topic: Option<String>,
}

impl HelpFile {
    /// Create a new help file from a markdown file
    pub fn new(path: &str) -> std::io::Result<Self> {
        let mut help_file = Self {
            path: path.to_string(),
            topics: HashMap::new(),
            default_topic: None,
        };

        help_file.load()?;
        Ok(help_file)
    }

    /// Load and parse markdown file
    fn load(&mut self) -> std::io::Result<()> {
        let content = fs::read_to_string(&self.path)?;
        self.parse_markdown(&content);
        Ok(())
    }

    /// Parse markdown content into topics
    fn parse_markdown(&mut self, content: &str) {
        let mut current_topic: Option<HelpTopic> = None;

        for line in content.lines() {
            // Check for topic header: # Title {#topic-id}
            if let Some(topic) = self.parse_topic_header(line) {
                // Save previous topic if exists
                if let Some(topic) = current_topic.take() {
                    if self.default_topic.is_none() {
                        self.default_topic = Some(topic.id.clone());
                    }
                    self.topics.insert(topic.id.clone(), topic);
                }
                current_topic = Some(topic);
            } else if let Some(ref mut topic) = current_topic {
                // Check for cross-reference: [Link](#topic-id)
                if let Some(link_id) = self.parse_link(line) {
                    topic.add_link(link_id);
                }

                // Add line to current topic (skip empty first line)
                if !topic.content.is_empty() || !line.trim().is_empty() {
                    topic.add_line(line.to_string());
                }
            }
        }

        // Save last topic
        if let Some(topic) = current_topic {
            if self.default_topic.is_none() {
                self.default_topic = Some(topic.id.clone());
            }
            self.topics.insert(topic.id.clone(), topic);
        }
    }

    /// Parse topic header: # Title {#topic-id}
    fn parse_topic_header(&self, line: &str) -> Option<HelpTopic> {
        let trimmed = line.trim();
        if !trimmed.starts_with('#') {
            return None;
        }

        // Extract topic ID from {#id}
        if let Some(start) = trimmed.find("{#") {
            if let Some(end) = trimmed[start..].find('}') {
                let id = trimmed[start + 2..start + end].to_string();
                let title = trimmed[1..start].trim().to_string();
                return Some(HelpTopic::new(id, title));
            }
        }

        None
    }

    /// Parse cross-reference link: [Text](#topic-id)
    fn parse_link(&self, line: &str) -> Option<String> {
        if let Some(start) = line.find("](#") {
            if let Some(end) = line[start..].find(')') {
                let id = line[start + 3..start + end].to_string();
                return Some(id);
            }
        }
        None
    }

    /// Get a topic by ID
    pub fn get_topic(&self, id: &str) -> Option<&HelpTopic> {
        self.topics.get(id)
    }

    /// Get the default topic
    pub fn get_default_topic(&self) -> Option<&HelpTopic> {
        if let Some(ref id) = self.default_topic {
            self.get_topic(id)
        } else {
            None
        }
    }

    /// Get all topic IDs
    pub fn get_topic_ids(&self) -> Vec<String> {
        let mut ids: Vec<String> = self.topics.keys().cloned().collect();
        ids.sort();
        ids
    }

    /// Check if a topic exists
    pub fn has_topic(&self, id: &str) -> bool {
        self.topics.contains_key(id)
    }

    /// Get the file path
    pub fn path(&self) -> &str {
        &self.path
    }

    /// Reload the help file from disk
    pub fn reload(&mut self) -> std::io::Result<()> {
        self.topics.clear();
        self.default_topic = None;
        self.load()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    fn create_test_help_file() -> NamedTempFile {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "# Introduction {{#intro}}").unwrap();
        writeln!(file, "").unwrap();
        writeln!(file, "Welcome to the help system!").unwrap();
        writeln!(file, "").unwrap();
        writeln!(file, "For more information, see [File Menu](#file-menu).").unwrap();
        writeln!(file, "").unwrap();
        writeln!(file, "# File Menu {{#file-menu}}").unwrap();
        writeln!(file, "").unwrap();
        writeln!(file, "The File menu contains:").unwrap();
        writeln!(file, "- Open: Open a file").unwrap();
        writeln!(file, "- Save: Save the file").unwrap();
        writeln!(file, "").unwrap();
        writeln!(file, "See also [Edit Menu](#edit-menu).").unwrap();
        writeln!(file, "").unwrap();
        writeln!(file, "# Edit Menu {{#edit-menu}}").unwrap();
        writeln!(file, "").unwrap();
        writeln!(file, "The Edit menu contains:").unwrap();
        writeln!(file, "- Copy: Copy text").unwrap();
        writeln!(file, "- Paste: Paste text").unwrap();
        file.flush().unwrap();
        file
    }

    #[test]
    fn test_help_file_load() {
        let file = create_test_help_file();
        let help = HelpFile::new(file.path().to_str().unwrap()).unwrap();

        assert_eq!(help.get_topic_ids().len(), 3);
        assert!(help.has_topic("intro"));
        assert!(help.has_topic("file-menu"));
        assert!(help.has_topic("edit-menu"));
    }

    #[test]
    fn test_help_topic_content() {
        let file = create_test_help_file();
        let help = HelpFile::new(file.path().to_str().unwrap()).unwrap();

        let topic = help.get_topic("intro").unwrap();
        assert_eq!(topic.title, "Introduction");
        assert!(topic.content.len() > 0);
        assert_eq!(topic.links.len(), 1);
        assert_eq!(topic.links[0], "file-menu");
    }

    #[test]
    fn test_default_topic() {
        let file = create_test_help_file();
        let help = HelpFile::new(file.path().to_str().unwrap()).unwrap();

        let default = help.get_default_topic().unwrap();
        assert_eq!(default.id, "intro");
    }

    #[test]
    fn test_formatted_content() {
        let file = create_test_help_file();
        let help = HelpFile::new(file.path().to_str().unwrap()).unwrap();

        let topic = help.get_topic("file-menu").unwrap();
        let formatted = topic.get_formatted_content();

        assert!(formatted[0].contains("File Menu"));
        assert!(formatted.iter().any(|line| line.contains("See also:")));
    }

    #[test]
    fn test_cross_references() {
        let file = create_test_help_file();
        let help = HelpFile::new(file.path().to_str().unwrap()).unwrap();

        let file_menu = help.get_topic("file-menu").unwrap();
        assert_eq!(file_menu.links.len(), 1);
        assert_eq!(file_menu.links[0], "edit-menu");
    }

    #[test]
    fn test_reload() {
        let file = create_test_help_file();
        let path = file.path().to_str().unwrap().to_string();
        let mut help = HelpFile::new(&path).unwrap();

        assert_eq!(help.get_topic_ids().len(), 3);

        // Reload should work
        help.reload().unwrap();
        assert_eq!(help.get_topic_ids().len(), 3);
    }
}
