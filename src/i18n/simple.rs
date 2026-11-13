//! A reader for the plain subset of Fluent: `id = text` messages, text
//! continued on indented lines, `{ $name }` placeholders and comments. It is
//! the whole backend when the `i18n` feature is off, and the tests use it
//! with the feature on to list a file's ids.

use super::Arg;
use std::collections::HashMap;

/// One parsed resource: its messages, in file order.
pub(crate) type Messages = Vec<(String, String)>;

/// Parse `source`, or say which lines are not plain Fluent.
pub(crate) fn parse(source: &str) -> Result<Messages, Vec<String>> {
    let mut messages: Messages = Vec::new();
    let mut problems = Vec::new();
    // Blank lines seen inside the current message; they count only if an
    // indented line follows.
    let mut pending_blank_lines = 0;
    let mut in_message = false;

    for (index, line) in source.lines().enumerate() {
        let number = index + 1;
        if line.trim().is_empty() {
            pending_blank_lines += 1;
            continue;
        }
        if line.starts_with([' ', '\t']) {
            let text = line.trim();
            if in_message && !text.starts_with('.') {
                if let Some((_, value)) = messages.last_mut() {
                    if !value.is_empty() {
                        value.extend(std::iter::repeat_n('\n', pending_blank_lines + 1));
                    }
                    value.push_str(text);
                }
            } else if !in_message {
                problems.push(format!("line {number}: indented text outside a message"));
            }
            pending_blank_lines = 0;
            continue;
        }
        pending_blank_lines = 0;
        in_message = false;
        if line.starts_with('#') {
            continue;
        }
        match line.split_once('=') {
            Some((id, value)) if is_identifier(id.trim()) => {
                messages.push((id.trim().to_string(), value.trim().to_string()));
                in_message = true;
            }
            _ => problems.push(format!("line {number}: expected `id = text`")),
        }
    }

    if problems.is_empty() {
        Ok(messages)
    } else {
        Err(problems)
    }
}

/// A Fluent identifier: a letter, then letters, digits, `-` and `_`.
fn is_identifier(id: &str) -> bool {
    let mut chars = id.chars();
    chars.next().is_some_and(|c| c.is_ascii_alphabetic())
        && chars.all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
}

/// Replace each `{ $name }` of `pattern` with its argument; a placeholder
/// with no argument is kept as `{$name}`, the way Fluent shows it.
pub(crate) fn substitute(pattern: &str, args: &[(&str, Arg<'_>)]) -> String {
    let mut out = String::with_capacity(pattern.len());
    let mut rest = pattern;
    while let Some(open) = rest.find('{') {
        out.push_str(&rest[..open]);
        let Some(close) = rest[open..].find('}') else {
            // No closing brace: the rest is plain text.
            rest = &rest[open..];
            break;
        };
        let inner = rest[open + 1..open + close].trim();
        if let Some(name) = inner.strip_prefix('$') {
            if let Some((_, value)) = args.iter().find(|(arg, _)| *arg == name) {
                out.push_str(&value.to_string());
            } else {
                out.push_str("{$");
                out.push_str(name);
                out.push('}');
            }
        } else {
            // A string literal: `{ "{" }`.
            out.push_str(inner.trim_matches('"'));
        }
        rest = &rest[open + close + 1..];
    }
    out.push_str(rest);
    out
}

/// The feature-off store: each locale's messages, later ones winning.
#[derive(Default)]
#[cfg_attr(
    feature = "i18n",
    allow(dead_code, reason = "the backend only when the i18n feature is off")
)]
pub(crate) struct Store {
    /// Locales in the order their first resource was added.
    order: Vec<String>,
    messages: HashMap<String, HashMap<String, String>>,
    /// The current lookup chain.
    chain: Vec<String>,
}

#[cfg_attr(
    feature = "i18n",
    allow(dead_code, reason = "the backend only when the i18n feature is off")
)]
impl Store {
    pub(crate) fn add(&mut self, locale: &str, source: &str) -> Result<(), Vec<String>> {
        let parsed = parse(source)?;
        if !self.order.iter().any(|l| l == locale) {
            self.order.push(locale.to_string());
        }
        self.messages
            .entry(locale.to_string())
            .or_default()
            .extend(parsed);
        Ok(())
    }

    pub(crate) fn locales(&self) -> Vec<String> {
        self.order.clone()
    }

    pub(crate) fn select(&mut self, chain: &[String]) {
        self.chain = chain.to_vec();
    }

    fn pattern(&self, id: &str) -> Option<&String> {
        self.chain
            .iter()
            .find_map(|locale| self.messages.get(locale)?.get(id))
    }

    pub(crate) fn has(&self, id: &str) -> bool {
        self.pattern(id).is_some()
    }

    pub(crate) fn format(&self, id: &str, args: &[(&str, Arg<'_>)]) -> Option<String> {
        self.pattern(id).map(|pattern| substitute(pattern, args))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_messages_comments_and_continuations() {
        let messages = parse(
            "# comment\n## section\n\nhello = Hello\nlong =\n    one\n\n    two\n  .attr = skipped\nnext = { $n } left\n",
        )
        .unwrap();
        assert_eq!(
            messages,
            vec![
                ("hello".to_string(), "Hello".to_string()),
                ("long".to_string(), "one\n\ntwo".to_string()),
                ("next".to_string(), "{ $n } left".to_string()),
            ]
        );
    }

    #[test]
    fn reports_lines_that_are_not_messages() {
        let problems = parse("ok = fine\nnot a message\n  stray").unwrap_err();
        assert_eq!(
            problems,
            vec![
                "line 2: expected `id = text`".to_string(),
                "line 3: indented text outside a message".to_string(),
            ]
        );
        parse("  stray").unwrap_err();
        parse("1bad = x").unwrap_err();
    }

    #[test]
    fn substitutes_placeholders() {
        let args = [("name", Arg::from("a.txt")), ("n", Arg::from(3))];
        assert_eq!(substitute("{ $name }: {$n} {$x}", &args), "a.txt: 3 {$x}");
        assert_eq!(substitute("brace { \"{\" }", &[]), "brace {");
        assert_eq!(substitute("open { brace", &[]), "open { brace");
    }

    #[test]
    fn later_resources_win_and_the_chain_decides() {
        let mut store = Store::default();
        store.add("en", "a = A\nb = B").unwrap();
        store.add("ca", "a = Ca").unwrap();
        store.add("en", "b = Bee").unwrap();
        store.select(&["ca".to_string(), "en".to_string()]);
        assert_eq!(store.format("a", &[]).as_deref(), Some("Ca"));
        assert_eq!(store.format("b", &[]).as_deref(), Some("Bee"));
        assert_eq!(store.format("c", &[]), None);
        assert_eq!(store.locales(), vec!["en".to_string(), "ca".to_string()]);
    }
}
