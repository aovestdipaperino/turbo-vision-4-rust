// (C) 2025 - Enzo Lombardi

//! `HelpFile` - markdown-based help content loader and parser.
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

/// Styled text segment for rich text rendering
#[derive(Debug, Clone, PartialEq)]
pub enum TextSegment {
    /// Normal unstyled text
    Normal(String),
    /// Bold text (from **text**)
    Bold(String),
    /// Italic text (from *text*)
    Italic(String),
    /// Inline code (from `text`)
    Code(String),
    /// Hyperlink (from [text](#target))
    Link { text: String, target: String },
}

impl TextSegment {
    /// Get the display text for this segment
    pub fn text(&self) -> &str {
        match self {
            TextSegment::Normal(s) => s,
            TextSegment::Bold(s) => s,
            TextSegment::Italic(s) => s,
            TextSegment::Code(s) => s,
            TextSegment::Link { text, .. } => text,
        }
    }

    /// Display width of this segment in terminal cells, measured the way
    /// `DrawBuffer::move_str` draws it: wide characters take two cells,
    /// zero-width ones none. Not the byte length, so a `→` or an `é` counts
    /// once.
    pub fn width(&self) -> usize {
        display_width(self.text())
    }

    /// Get the display length of this segment, in cells; same as
    /// [`width`](Self::width).
    pub fn len(&self) -> usize {
        self.width()
    }

    /// Check if this segment is empty
    pub fn is_empty(&self) -> bool {
        self.text().is_empty()
    }
}

/// Width of `s` in terminal cells, one per character except wide (two) and
/// zero-width (none) ones; matches what `DrawBuffer::move_str` occupies.
pub(crate) fn display_width(s: &str) -> usize {
    use unicode_width::UnicodeWidthChar;
    s.chars().map(|c| c.width().unwrap_or(0)).sum()
}

/// Prefix drawn before each "See also" entry.
const SEE_ALSO_PREFIX: &str = "  \u{2192} "; // "  → "

/// Clamp a width to a cross-ref's `u8` length.
fn ref_len(width: usize) -> u8 {
    width.min(u8::MAX as usize) as u8
}

/// Cross-reference link within help content
/// Matches Borland: `TCrossRef` (helpbase.h)
#[derive(Debug, Clone)]
pub struct CrossRef {
    /// Line number (1-based, matching Borland convention)
    pub line: i16,
    /// Column offset within line (0-based)
    pub offset: i16,
    /// Length of the link text
    pub length: u8,
    /// Target topic ID to navigate to
    pub target: String,
}

impl CrossRef {
    /// Create a new cross-reference
    pub fn new(line: i16, offset: i16, length: u8, target: String) -> Self {
        Self {
            line,
            offset,
            length,
            target,
        }
    }
}

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
    /// Titles of the linked topics, by topic id, so "See also" can show
    /// "Edit Menu" rather than "edit". Filled by [`HelpFile`] once every topic
    /// is parsed; a link with no entry shows its id.
    pub link_titles: HashMap<String, String>,
}

impl HelpTopic {
    /// Create a new help topic
    pub fn new(id: String, title: String) -> Self {
        Self {
            id,
            title,
            content: Vec::new(),
            links: Vec::new(),
            link_titles: HashMap::new(),
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

    /// What a "See also" entry for `id` shows: the linked topic's title, or
    /// the id when that topic is unknown.
    pub fn link_label<'a>(&'a self, id: &'a str) -> &'a str {
        self.link_titles.get(id).map_or(id, String::as_str)
    }

    /// Get formatted content with line numbers for display
    pub fn get_formatted_content(&self) -> Vec<String> {
        let mut lines = vec![format!("═══ {} ═══", self.title), String::new()];
        lines.extend(self.content.clone());

        if !self.links.is_empty() {
            lines.push(String::new());
            lines.push(crate::i18n::tr("help-see-also"));
            for link in &self.links {
                lines.push(format!("{SEE_ALSO_PREFIX}{}", self.link_label(link)));
            }
        }

        lines
    }

    /// Get formatted content with cross-references extracted
    /// Returns (`formatted_lines`, `cross_refs`)
    /// Cross-refs have line numbers relative to the formatted output (1-based)
    pub fn get_content_with_refs(&self) -> (Vec<String>, Vec<CrossRef>) {
        let mut lines = Vec::new();
        let mut refs = Vec::new();

        // Header line
        lines.push(format!("═══ {} ═══", self.title));
        lines.push(String::new());

        // Process content lines, extracting links
        for content_line in &self.content {
            let (processed_line, line_refs) =
                Self::process_line_links(content_line, (lines.len() + 1) as i16);
            lines.push(processed_line);
            refs.extend(line_refs);
        }

        // "See also" section with clickable links
        if !self.links.is_empty() {
            lines.push(String::new());
            lines.push(crate::i18n::tr("help-see-also"));
            let offset = display_width(SEE_ALSO_PREFIX) as i16;
            for link in &self.links {
                let line_num = (lines.len() + 1) as i16;
                let label = self.link_label(link);
                refs.push(CrossRef::new(
                    line_num,
                    offset,
                    ref_len(display_width(label)),
                    link.clone(),
                ));
                lines.push(format!("{SEE_ALSO_PREFIX}{label}"));
            }
        }

        (lines, refs)
    }

    /// Process a single line, extracting markdown links [text](#target)
    /// Returns the display text (with link markers removed) and any cross-refs found
    fn process_line_links(line: &str, line_num: i16) -> (String, Vec<CrossRef>) {
        let mut result = String::new();
        let mut refs = Vec::new();
        let mut remaining = line;

        while let Some(link_start) = remaining.find('[') {
            // Add text before the link
            result.push_str(&remaining[..link_start]);

            // Find the closing bracket and target
            let after_bracket = &remaining[link_start + 1..];
            if let Some(text_end) = after_bracket.find("](#") {
                let link_text = &after_bracket[..text_end];
                let after_target_start = &after_bracket[text_end + 3..];

                if let Some(target_end) = after_target_start.find(')') {
                    let target = &after_target_start[..target_end];

                    // Record the cross-reference at current position
                    let offset = display_width(&result) as i16;
                    refs.push(CrossRef::new(
                        line_num,
                        offset,
                        ref_len(display_width(link_text)),
                        target.to_string(),
                    ));

                    // Add the link text (displayed without markdown syntax)
                    result.push_str(link_text);

                    // Continue after the link
                    remaining = &after_target_start[target_end + 1..];
                    continue;
                }
            }

            // Not a valid link, keep the bracket and continue
            result.push('[');
            remaining = after_bracket;
        }

        // Add any remaining text
        result.push_str(remaining);

        (result, refs)
    }

    /// Get the number of cross-references in this topic
    pub fn num_cross_refs(&self) -> usize {
        let (_, refs) = self.get_content_with_refs();
        refs.len()
    }

    /// Parse a line into styled text segments
    /// Handles: **bold**, *italic*, `code`, and [link](#target)
    pub fn parse_line_segments(line: &str) -> Vec<TextSegment> {
        let mut segments = Vec::new();
        let mut remaining = line;
        let mut current_text = String::new();

        while !remaining.is_empty() {
            // Check for bold (**text**)
            if remaining.starts_with("**") {
                // Flush any accumulated normal text
                if !current_text.is_empty() {
                    segments.push(TextSegment::Normal(std::mem::take(&mut current_text)));
                }

                if let Some(end) = remaining[2..].find("**") {
                    let bold_text = &remaining[2..2 + end];
                    segments.push(TextSegment::Bold(bold_text.to_string()));
                    remaining = &remaining[2 + end + 2..];
                    continue;
                }
            }

            // Check for code (`text`)
            if remaining.starts_with('`') {
                // Flush any accumulated normal text
                if !current_text.is_empty() {
                    segments.push(TextSegment::Normal(std::mem::take(&mut current_text)));
                }

                if let Some(end) = remaining[1..].find('`') {
                    let code_text = &remaining[1..=end];
                    segments.push(TextSegment::Code(code_text.to_string()));
                    remaining = &remaining[1 + end + 1..];
                    continue;
                }
            }

            // Check for link ([text](#target))
            if remaining.starts_with('[') {
                // Flush any accumulated normal text
                if !current_text.is_empty() {
                    segments.push(TextSegment::Normal(std::mem::take(&mut current_text)));
                }

                if let Some(text_end) = remaining[1..].find("](#") {
                    let link_text = &remaining[1..=text_end];
                    let after_target = &remaining[1 + text_end + 3..];

                    if let Some(target_end) = after_target.find(')') {
                        let target = &after_target[..target_end];
                        segments.push(TextSegment::Link {
                            text: link_text.to_string(),
                            target: target.to_string(),
                        });
                        remaining = &after_target[target_end + 1..];
                        continue;
                    }
                }
            }

            // Check for italic (*text*) - but not if it's **
            if remaining.starts_with('*') && !remaining.starts_with("**") {
                // Flush any accumulated normal text
                if !current_text.is_empty() {
                    segments.push(TextSegment::Normal(std::mem::take(&mut current_text)));
                }

                // Find closing * that isn't **
                let search = &remaining[1..];
                let mut found_end = None;
                let mut pos = 0;
                while pos < search.len() {
                    if search[pos..].starts_with('*') && !search[pos..].starts_with("**") {
                        found_end = Some(pos);
                        break;
                    }
                    pos += 1;
                }

                if let Some(end) = found_end {
                    let italic_text = &remaining[1..=end];
                    segments.push(TextSegment::Italic(italic_text.to_string()));
                    remaining = &remaining[1 + end + 1..];
                    continue;
                }
            }

            // Not a special marker, accumulate as normal text
            current_text.push(remaining.chars().next().unwrap());
            remaining = &remaining[remaining.chars().next().unwrap().len_utf8()..];
        }

        // Flush any remaining normal text
        if !current_text.is_empty() {
            segments.push(TextSegment::Normal(current_text));
        }

        // Coalesce empty result to single empty Normal
        if segments.is_empty() {
            segments.push(TextSegment::Normal(String::new()));
        }

        segments
    }

    /// Get content as lines of styled segments for rich text rendering
    /// Returns (`line_segments`, `cross_refs`) where each line is a `Vec<TextSegment>`
    pub fn get_styled_content(&self) -> (Vec<Vec<TextSegment>>, Vec<CrossRef>) {
        let mut all_segments = Vec::new();
        let mut refs = Vec::new();

        // Header line (just normal text for now)
        all_segments.push(vec![TextSegment::Normal(format!("═══ {} ═══", self.title))]);
        all_segments.push(vec![TextSegment::Normal(String::new())]);

        // Process content lines
        for content_line in &self.content {
            let segments = Self::parse_line_segments(content_line);

            // Track cross-refs from links
            let line_num = (all_segments.len() + 1) as i16;
            let mut offset = 0i16;
            for seg in &segments {
                if let TextSegment::Link { text, target } = seg {
                    refs.push(CrossRef::new(
                        line_num,
                        offset,
                        ref_len(display_width(text)),
                        target.clone(),
                    ));
                }
                offset += seg.width() as i16;
            }

            all_segments.push(segments);
        }

        // "See also" section
        if !self.links.is_empty() {
            all_segments.push(vec![TextSegment::Normal(String::new())]);
            all_segments.push(vec![TextSegment::Normal(crate::i18n::tr("help-see-also"))]);
            // The link starts where the prefix ends, in cells.
            let offset = display_width(SEE_ALSO_PREFIX) as i16;
            for link in &self.links {
                let line_num = (all_segments.len() + 1) as i16;
                let label = self.link_label(link).to_string();
                refs.push(CrossRef::new(
                    line_num,
                    offset,
                    ref_len(display_width(&label)),
                    link.clone(),
                ));
                all_segments.push(vec![
                    TextSegment::Normal(SEE_ALSO_PREFIX.to_string()),
                    TextSegment::Link {
                        text: label,
                        target: link.clone(),
                    },
                ]);
            }
        }

        (all_segments, refs)
    }
}

/// `HelpFile` - Manages markdown help files
///
/// Modern alternative to Borland's `THelpFile`
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
    pub fn new(path: impl AsRef<std::path::Path>) -> std::io::Result<Self> {
        let path_ref = path.as_ref();
        let mut help_file = Self {
            path: path_ref.to_string_lossy().to_string(),
            topics: HashMap::new(),
            default_topic: None,
        };

        help_file.load()?;
        Ok(help_file)
    }

    /// Create a help file from an in-memory markdown string
    pub fn from_content(content: &str) -> Self {
        let mut help_file = Self {
            path: String::new(),
            topics: HashMap::new(),
            default_topic: None,
        };
        help_file.parse_markdown(content);
        help_file
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
            if let Some(topic) = Self::parse_topic_header(line) {
                // Save previous topic if exists
                if let Some(topic) = current_topic.take() {
                    if self.default_topic.is_none() {
                        self.default_topic = Some(topic.id.clone());
                    }
                    self.topics.insert(topic.id.clone(), topic);
                }
                current_topic = Some(topic);
            } else if let Some(ref mut topic) = current_topic {
                // Record every cross-reference on the line: [Link](#topic-id)
                for link_id in Self::parse_links(line) {
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

        self.resolve_link_titles();
    }

    /// Record, in every topic, the titles of the topics it links to. Runs
    /// after parsing, since a link may point at a topic further down.
    fn resolve_link_titles(&mut self) {
        let titles: HashMap<String, String> = self
            .topics
            .values()
            .map(|t| (t.id.clone(), t.title.clone()))
            .collect();
        for topic in self.topics.values_mut() {
            topic.link_titles = topic
                .links
                .iter()
                .filter_map(|id| titles.get(id).map(|title| (id.clone(), title.clone())))
                .collect();
        }
    }

    /// Parse topic header: # Title {#topic-id}
    fn parse_topic_header(line: &str) -> Option<HelpTopic> {
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

    /// The targets of the cross-reference links `[Text](#topic-id)` on a
    /// line, in order. Found the way the inline links are drawn, so the
    /// "See also" list names exactly the links the text shows; repeats are
    /// left for `add_link` to drop.
    fn parse_links(line: &str) -> impl Iterator<Item = String> {
        HelpTopic::process_line_links(line, 0)
            .1
            .into_iter()
            .map(|r| r.target)
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

/// Builder for creating help files with a fluent API.
pub struct HelpFileBuilder {
    path: Option<String>,
}

impl HelpFileBuilder {
    pub fn new() -> Self {
        Self { path: None }
    }

    #[must_use]
    pub fn path(mut self, path: impl Into<String>) -> Self {
        self.path = Some(path.into());
        self
    }

    pub fn build(self) -> std::io::Result<HelpFile> {
        let path = self.path.expect("HelpFile path must be set");
        HelpFile::new(&path)
    }

    pub fn build_rc(self) -> std::io::Result<std::rc::Rc<std::cell::RefCell<HelpFile>>> {
        Ok(std::rc::Rc::new(std::cell::RefCell::new(self.build()?)))
    }
}

impl Default for HelpFileBuilder {
    fn default() -> Self {
        Self::new()
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
        writeln!(file).unwrap();
        writeln!(file, "Welcome to the help system!").unwrap();
        writeln!(file).unwrap();
        writeln!(file, "For more information, see [File Menu](#file-menu).").unwrap();
        writeln!(file).unwrap();
        writeln!(file, "# File Menu {{#file-menu}}").unwrap();
        writeln!(file).unwrap();
        writeln!(file, "The File menu contains:").unwrap();
        writeln!(file, "- Open: Open a file").unwrap();
        writeln!(file, "- Save: Save the file").unwrap();
        writeln!(file).unwrap();
        writeln!(file, "See also [Edit Menu](#edit-menu).").unwrap();
        writeln!(file).unwrap();
        writeln!(file, "# Edit Menu {{#edit-menu}}").unwrap();
        writeln!(file).unwrap();
        writeln!(file, "The Edit menu contains:").unwrap();
        writeln!(file, "- Copy: Copy text").unwrap();
        writeln!(file, "- Paste: Paste text").unwrap();
        file.flush().unwrap();
        file
    }

    #[test]
    fn every_link_on_a_line_is_recorded() {
        let help = HelpFile::from_content(
            "# Two {#two}\n\nSee [A](#a) and [B](#b).\n\n\
             # Brackets {#brackets}\n\nNotes ([C](#c)) and [D](#d)], then [C](#c) again.\n",
        );
        assert_eq!(help.get_topic("two").unwrap().links, ["a", "b"]);
        // A `)` or `]` right after a link ends it; it is not part of the id,
        // and a repeated link is recorded once.
        assert_eq!(help.get_topic("brackets").unwrap().links, ["c", "d"]);
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
        assert!(!topic.content.is_empty());
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
    fn test_get_content_with_refs() {
        let file = create_test_help_file();
        let help = HelpFile::new(file.path().to_str().unwrap()).unwrap();

        let intro = help.get_topic("intro").unwrap();
        let (lines, refs) = intro.get_content_with_refs();

        // Should have header, empty line, content lines, and see also section
        assert!(lines.len() >= 4);
        assert!(lines[0].contains("Introduction"));

        // Should have at least one cross-reference (from inline link and see also)
        assert!(!refs.is_empty());

        // Check that inline link "[File Menu](#file-menu)" was processed
        // The link should appear in the content without markdown syntax
        let has_inline_ref = refs.iter().any(|r| r.target == "file-menu");
        assert!(has_inline_ref, "Should have file-menu cross-ref");

        // Check that cross-ref has valid position
        let file_menu_ref = refs.iter().find(|r| r.target == "file-menu").unwrap();
        assert!(file_menu_ref.line > 0, "Line should be positive (1-based)");
        assert!(file_menu_ref.length > 0, "Length should be positive");
    }

    #[test]
    fn test_process_line_links() {
        // Test the link parsing function directly
        let line = "See [File Menu](#file-menu) and [Edit](#edit) for details.";
        let (result, refs) = HelpTopic::process_line_links(line, 5);

        // The result should have the markdown stripped
        assert_eq!(result, "See File Menu and Edit for details.");

        // Should have two cross-references
        assert_eq!(refs.len(), 2);

        // First ref: "File Menu" at position 4
        assert_eq!(refs[0].target, "file-menu");
        assert_eq!(refs[0].offset, 4);
        assert_eq!(refs[0].length, 9); // "File Menu"
        assert_eq!(refs[0].line, 5);

        // Second ref: "Edit" at position 18
        assert_eq!(refs[1].target, "edit");
        assert_eq!(refs[1].offset, 18);
        assert_eq!(refs[1].length, 4); // "Edit"
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

    #[test]
    fn test_parse_line_segments_bold() {
        let segments = HelpTopic::parse_line_segments("Press **F1** for help");
        assert_eq!(segments.len(), 3);
        assert_eq!(segments[0], TextSegment::Normal("Press ".to_string()));
        assert_eq!(segments[1], TextSegment::Bold("F1".to_string()));
        assert_eq!(segments[2], TextSegment::Normal(" for help".to_string()));
    }

    #[test]
    fn test_parse_line_segments_italic() {
        let segments = HelpTopic::parse_line_segments("Choose *File > Open* from menu");
        assert_eq!(segments.len(), 3);
        assert_eq!(segments[0], TextSegment::Normal("Choose ".to_string()));
        assert_eq!(segments[1], TextSegment::Italic("File > Open".to_string()));
        assert_eq!(segments[2], TextSegment::Normal(" from menu".to_string()));
    }

    #[test]
    fn test_parse_line_segments_code() {
        let segments = HelpTopic::parse_line_segments("Use `Ctrl+C` to copy");
        assert_eq!(segments.len(), 3);
        assert_eq!(segments[0], TextSegment::Normal("Use ".to_string()));
        assert_eq!(segments[1], TextSegment::Code("Ctrl+C".to_string()));
        assert_eq!(segments[2], TextSegment::Normal(" to copy".to_string()));
    }

    #[test]
    fn test_parse_line_segments_link() {
        let segments = HelpTopic::parse_line_segments("See [File Menu](#file-menu) for details");
        assert_eq!(segments.len(), 3);
        assert_eq!(segments[0], TextSegment::Normal("See ".to_string()));
        assert_eq!(
            segments[1],
            TextSegment::Link {
                text: "File Menu".to_string(),
                target: "file-menu".to_string(),
            }
        );
        assert_eq!(segments[2], TextSegment::Normal(" for details".to_string()));
    }

    #[test]
    fn test_parse_line_segments_mixed() {
        let segments = HelpTopic::parse_line_segments("Press **F1** or `?` for [Help](#help)");
        assert_eq!(segments.len(), 6);
        assert_eq!(segments[0], TextSegment::Normal("Press ".to_string()));
        assert_eq!(segments[1], TextSegment::Bold("F1".to_string()));
        assert_eq!(segments[2], TextSegment::Normal(" or ".to_string()));
        assert_eq!(segments[3], TextSegment::Code("?".to_string()));
        assert_eq!(segments[4], TextSegment::Normal(" for ".to_string()));
        assert_eq!(
            segments[5],
            TextSegment::Link {
                text: "Help".to_string(),
                target: "help".to_string(),
            }
        );
    }

    #[test]
    fn test_parse_line_segments_plain() {
        let segments = HelpTopic::parse_line_segments("Just plain text");
        assert_eq!(segments.len(), 1);
        assert_eq!(
            segments[0],
            TextSegment::Normal("Just plain text".to_string())
        );
    }

    #[test]
    fn see_also_shows_the_target_title_and_falls_back_to_the_id() {
        let help = HelpFile::from_content(
            "# Intro {#intro}\n\nSee [Edit](#edit).\nAnd [Gone](#missing).\n\n# Edit Menu {#edit}\n\nText.\n",
        );
        let intro = help.get_topic("intro").unwrap();
        assert_eq!(intro.links, vec!["edit".to_string(), "missing".to_string()]);
        let (lines, refs) = intro.get_styled_content();
        let n = lines.len();
        assert_eq!(
            lines[n - 2][1],
            TextSegment::Link {
                text: "Edit Menu".to_string(),
                target: "edit".to_string(),
            },
            "the linked topic's title"
        );
        assert_eq!(
            lines[n - 1][1],
            TextSegment::Link {
                text: "missing".to_string(),
                target: "missing".to_string(),
            },
            "an unknown topic shows its id"
        );
        let last = refs.last().unwrap();
        assert_eq!((last.target.as_str(), last.length), ("missing", 7));
        let edit = &refs[refs.len() - 2];
        assert_eq!((edit.target.as_str(), edit.length), ("edit", 9));
    }

    #[test]
    fn test_parse_line_segments_empty() {
        let segments = HelpTopic::parse_line_segments("");
        assert_eq!(segments.len(), 1);
        assert_eq!(segments[0], TextSegment::Normal(String::new()));
    }
}
