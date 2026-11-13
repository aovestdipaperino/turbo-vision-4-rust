//! Checks for translation files, for an application's tests.
//!
//! Turbo Vision checks its own `.ftl` files with these functions. An
//! application calls them the same way, from a test, for its own files:
//!
//! ```
//! use turbo_vision::i18n::{check_hot_key_groups, check_translation};
//!
//! const EN: &str = "menu-file = ~F~ile\nmenu-edit = ~E~dit\nsaved = Saved { $name }.";
//! const CA: &str = "menu-file = ~F~itxer\nmenu-edit = ~E~dita\nsaved = S'ha desat { $name }.";
//!
//! // In a #[test]: every language has exactly the English messages.
//! let report = check_translation(EN, CA);
//! assert!(report.is_ok(), "ca.ftl:\n{report}");
//!
//! // And the menu's hot keys differ, in every language.
//! for source in [EN, CA] {
//!     let clashes = check_hot_key_groups(source, &[&["menu-file", "menu-edit"]]);
//!     assert!(clashes.is_empty(), "{clashes:?}");
//! }
//! ```

use std::fmt;

/// What [`check_translation`] found wrong with a translation. Each list
/// holds one line per problem, ready to show in a test failure.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct TranslationReport {
    /// Lines that are not valid Fluent, in either file. When the
    /// translation does not parse, the other lists may be incomplete.
    pub syntax: Vec<String>,
    /// Ids of the reference that the translation lacks. They show in the
    /// reference language until they are translated.
    pub missing: Vec<String>,
    /// Ids of the translation that the reference does not have: a typo, or
    /// a message removed from the reference. They are never shown.
    pub extra: Vec<String>,
    /// Placeholders the translation uses that the reference message does
    /// not have: no value is ever given for them, so the user sees
    /// `{$name}`.
    pub placeholders: Vec<String>,
    /// Badly marked hot keys, in either file: more than one `~` letter,
    /// an unclosed `~`, or a marked letter with no Alt key (A to Z only).
    pub hot_keys: Vec<String>,
}

impl TranslationReport {
    /// Whether nothing is wrong.
    #[must_use]
    pub fn is_ok(&self) -> bool {
        self.problems().is_empty()
    }

    /// Every problem, one line each, grouped by kind.
    #[must_use]
    pub fn problems(&self) -> Vec<String> {
        let mut all = Vec::new();
        all.extend(self.syntax.iter().map(|p| format!("syntax: {p}")));
        all.extend(self.missing.iter().map(|id| format!("missing: {id}")));
        all.extend(
            self.extra
                .iter()
                .map(|id| format!("not in the reference: {id}")),
        );
        all.extend(
            self.placeholders
                .iter()
                .map(|p| format!("placeholder: {p}")),
        );
        all.extend(self.hot_keys.iter().map(|p| format!("hot key: {p}")));
        all
    }
}

impl fmt::Display for TranslationReport {
    /// One problem per line; nothing when the translation is fine.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.problems().join("\n"))
    }
}

/// Compare `translation` with `reference` (the English file, usually): both
/// must be valid Fluent, have the same message ids, use no placeholder the
/// reference lacks, and mark their hot keys properly.
///
/// A translation may leave out a placeholder: a singular form often says
/// "one file" instead of `{ $count } files`. Terms (`-brand = ..`) and
/// attributes (`.title = ..`) are not compared; they belong to each
/// language.
///
/// Call it with the same file twice to check the reference alone.
///
/// The syntax is checked with the reader in use: Fluent with the `i18n`
/// feature, and without it the plain subset (`id = text`, indented
/// continuation lines, `{ $name }`), which has no terms and no selectors.
#[must_use]
pub fn check_translation(reference: &str, translation: &str) -> TranslationReport {
    let mut report = TranslationReport::default();
    report.syntax.extend(
        syntax_problems(reference)
            .into_iter()
            .map(|p| format!("reference: {p}")),
    );
    report.syntax.extend(syntax_problems(translation));

    let reference = messages(reference);
    let translation = messages(translation);
    let find = |messages: &[Message], id: &str| messages.iter().position(|m| m.id == id);

    for message in &reference {
        if find(&translation, &message.id).is_none() {
            report.missing.push(message.id.clone());
        }
    }
    for message in &translation {
        let Some(index) = find(&reference, &message.id) else {
            report.extra.push(message.id.clone());
            continue;
        };
        let known = placeholders(&reference[index].text);
        for name in placeholders(&message.text) {
            if !known.contains(&name) {
                report.placeholders.push(format!(
                    "{} (line {}) uses {{ ${name} }}, which the reference message does not have",
                    message.id, message.line
                ));
            }
        }
    }
    for message in reference.iter().chain(&translation) {
        if let Some(problem) = hot_key_problem(&message.text) {
            report
                .hot_keys
                .push(format!("{} (line {}) {problem}", message.id, message.line));
        }
    }
    report
}

/// The hot-key clashes in `source`: for each group of messages shown
/// together (the labels and buttons of one dialog, the items of one menu),
/// two messages that mark the same letter. Ids of a group that `source`
/// lacks are reported too.
///
/// ```
/// use turbo_vision::i18n::check_hot_key_groups;
///
/// let ca = "dialog-name = ~N~om\ndialog-new = ~N~ou";
/// assert_eq!(
///     check_hot_key_groups(ca, &[&["dialog-name", "dialog-new"]]),
///     ["'dialog-new' and 'dialog-name' both use Alt+N"]
/// );
/// ```
#[must_use]
pub fn check_hot_key_groups(source: &str, groups: &[&[&str]]) -> Vec<String> {
    let messages = messages(source);
    let mut problems = Vec::new();
    for group in groups {
        let mut seen: Vec<(char, &str)> = Vec::new();
        for id in *group {
            let Some(message) = messages.iter().find(|m| m.id == *id) else {
                problems.push(format!("'{id}' is not in the file"));
                continue;
            };
            let Some(key) = hot_key(&message.text) else {
                continue;
            };
            if let Some((_, other)) = seen.iter().find(|(k, _)| *k == key) {
                problems.push(format!("'{id}' and '{other}' both use Alt+{key}"));
            } else {
                seen.push((key, id));
            }
        }
    }
    problems
}

/// The marked letter of `text`, uppercased: what Alt is pressed with.
pub(crate) fn hot_key(text: &str) -> Option<char> {
    let start = text.find('~')? + 1;
    let end = start + text[start..].find('~')?;
    let mut marked = text[start..end].chars();
    let key = marked.next()?;
    marked
        .next()
        .is_none()
        .then(|| key.to_uppercase().next().unwrap_or(key))
}

/// What is wrong with the hot-key markers of `text`, if anything.
fn hot_key_problem(text: &str) -> Option<String> {
    let tildes = text.matches('~').count();
    if tildes % 2 == 1 {
        return Some("has a `~` with no closing `~`".to_string());
    }
    // A message with variants ([one] .., *[other] ..) marks each variant.
    let has_variants = text.contains("->");
    if tildes > 2 && !has_variants {
        return Some(format!(
            "marks {} letters; a text has one hot key",
            tildes / 2
        ));
    }
    for marked in text.split('~').skip(1).step_by(2) {
        let mut chars = marked.chars();
        let letter = chars.next();
        if chars.next().is_some() || letter.is_none() {
            return Some(format!("marks {marked:?}; mark a single letter"));
        }
        let letter = letter.unwrap_or(' ');
        if crate::core::event::alt_code(letter).is_none() {
            return Some(format!(
                "marks {letter:?}, which has no Alt key; mark a letter from A to Z"
            ));
        }
    }
    None
}

/// The `$name`s a message uses, in order, each once.
fn placeholders(text: &str) -> Vec<String> {
    let mut names: Vec<String> = Vec::new();
    for (start, _) in text.match_indices('$') {
        let name: String = text[start + 1..]
            .chars()
            .take_while(|c| c.is_ascii_alphanumeric() || *c == '-' || *c == '_')
            .collect();
        if !name.is_empty() && !names.contains(&name) {
            names.push(name);
        }
    }
    names
}

/// A message of a file: its id, its value with the variant lines, and the
/// line where it starts.
struct Message {
    id: String,
    text: String,
    line: usize,
}

/// The messages of `source`, read leniently: whatever does not parse is
/// reported by [`syntax_problems`], and the rest is still compared.
fn messages(source: &str) -> Vec<Message> {
    let mut messages: Vec<Message> = Vec::new();
    // Whether indented lines continue a message (not a term or a comment).
    let mut in_message = false;
    for (index, line) in source.lines().enumerate() {
        if line.trim().is_empty() {
            continue;
        }
        if line.starts_with([' ', '\t']) {
            let text = line.trim();
            if in_message && !text.starts_with('.') {
                if let Some(message) = messages.last_mut() {
                    message.text.push('\n');
                    message.text.push_str(text);
                }
            }
            continue;
        }
        in_message = false;
        let Some((id, value)) = line.split_once('=') else {
            continue;
        };
        let id = id.trim();
        let is_message_id = id.chars().next().is_some_and(|c| c.is_ascii_alphabetic())
            && id
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_');
        if is_message_id && !line.starts_with('#') {
            messages.push(Message {
                id: id.to_string(),
                text: value.trim().to_string(),
                line: index + 1,
            });
            in_message = true;
        }
    }
    messages
}

/// Why `source` is not valid Fluent, one line per error; empty when it is.
#[cfg(feature = "i18n")]
fn syntax_problems(source: &str) -> Vec<String> {
    match fluent_bundle::FluentResource::try_new(source.to_string()) {
        Ok(_) => Vec::new(),
        Err((_, errors)) => errors
            .iter()
            .map(|error| {
                let line = source[..error.pos.start.min(source.len())]
                    .lines()
                    .count()
                    .max(1);
                format!("line {line}: {:?}", error.kind)
            })
            .collect(),
    }
}

/// Why `source` is not plain Fluent, the only kind read without the `i18n`
/// feature.
#[cfg(not(feature = "i18n"))]
fn syntax_problems(source: &str) -> Vec<String> {
    super::simple::parse(source).err().unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;

    const EN: &str = "
# A comment
menu-file = ~F~ile
files-copied = { $count ->
    [one] One file copied to { $folder }.
   *[other] { $count } files copied to { $folder }.
    }
help = Press F1
    for help.
";

    #[test]
    fn a_complete_translation_is_ok() {
        let ca = "
menu-file = ~F~itxer
files-copied = { $count ->
    [one] S'ha copiat un fitxer a { $folder }.
   *[other] S'han copiat { $count } fitxers a { $folder }.
    }
help = Premeu F1
    per a l'ajuda.
";
        let report = check_translation(EN, ca);
        assert!(report.is_ok(), "{report}");
        assert_eq!(report.to_string(), "");
    }

    #[test]
    fn missing_and_extra_ids_are_reported() {
        let report = check_translation(
            EN,
            "menu-file = ~F~itxer\nmenu-fitxer = ~F~itxer\nhelp = Ajuda",
        );
        assert_eq!(report.missing, ["files-copied"]);
        assert_eq!(report.extra, ["menu-fitxer"]);
        assert!(!report.is_ok());
        assert_eq!(
            report.problems(),
            ["missing: files-copied", "not in the reference: menu-fitxer"]
        );
    }

    #[test]
    fn a_placeholder_the_reference_lacks_is_reported() {
        let report = check_translation(
            EN,
            "menu-file = ~F~itxer\nfiles-copied = Copiats { $count } a { $carpeta }.\nhelp = Ajuda",
        );
        assert_eq!(
            report.placeholders,
            ["files-copied (line 2) uses { $carpeta }, which the reference message does not have"]
        );
    }

    #[test]
    fn leaving_a_placeholder_out_is_allowed() {
        let report = check_translation(
            EN,
            "menu-file = ~F~itxer\nfiles-copied = Fet.\nhelp = Ajuda",
        );
        assert!(report.is_ok(), "{report}");
    }

    #[test]
    fn badly_marked_hot_keys_are_reported() {
        let en = "a = ~O~pen\nb = ~S~ave\nc = ~Q~uit\nd = ~Ex~it\ne = ~C~lose";
        let xx = "a = ~O~bre\nb = ~A~r~x~iu\nc = ~Á~rbre\nd = ~S~urt\ne = ~T~anca";
        let report = check_translation(en, xx);
        assert_eq!(
            report.hot_keys,
            [
                "d (line 4) marks \"Ex\"; mark a single letter",
                "b (line 2) marks 2 letters; a text has one hot key",
                "c (line 3) marks 'Á', which has no Alt key; mark a letter from A to Z",
            ]
        );
        let report = check_translation("a = ~O~pen", "a = ~O~bre~");
        assert_eq!(
            report.hot_keys,
            ["a (line 1) has a `~` with no closing `~`"]
        );
    }

    #[test]
    fn invalid_fluent_is_reported_with_its_line() {
        let report = check_translation(EN, "menu-file = ~F~itxer\nthis is not fluent\n");
        assert_eq!(report.syntax.len(), 1, "{report}");
        assert!(report.syntax[0].starts_with("line 2"), "{report}");
        let report = check_translation("not fluent either", "a = b");
        assert!(
            report.syntax[0].starts_with("reference: line 1"),
            "{report}"
        );
    }

    #[test]
    fn terms_attributes_and_comments_are_not_messages() {
        let en =
            "-brand = Turbo\n# about = not a message\nabout = About { -brand }\n    .title = About";
        let ca = "-brand = Turbo\nabout = Quant a { -brand }\n    .title = Quant a";
        let report = check_translation(en, ca);
        assert!(
            report.missing.is_empty() && report.extra.is_empty(),
            "{report}"
        );
        assert_eq!(messages(en).len(), 1);
        // Terms are Fluent; the plain reader used without the feature
        // cannot read them, and says so.
        if cfg!(feature = "i18n") {
            assert!(report.is_ok(), "{report}");
        } else {
            assert!(!report.syntax.is_empty(), "{report}");
        }
    }

    #[test]
    fn hot_key_groups_report_clashes_and_unknown_ids() {
        let source = "name = ~N~om\nnew = ~n~ou\nopen = ~O~bre\nplain = Text";
        assert_eq!(
            check_hot_key_groups(source, &[&["name", "new", "plain"], &["open", "gone"]]),
            [
                "'new' and 'name' both use Alt+N",
                "'gone' is not in the file"
            ]
        );
        assert!(check_hot_key_groups(source, &[&["name", "open"]]).is_empty());
    }

    #[test]
    fn hot_key_is_the_marked_letter_uppercased() {
        assert_eq!(hot_key("E~x~it"), Some('X'));
        assert_eq!(hot_key("Plain"), None);
        assert_eq!(hot_key("~Ex~it"), None);
    }
}
