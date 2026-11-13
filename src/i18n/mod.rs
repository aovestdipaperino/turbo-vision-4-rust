//! Translations: the strings Turbo Vision shows, and your application's, in
//! the user's language, with [Project Fluent](https://projectfluent.org).
//!
//! A message has an id and a text per language, written in Fluent's `.ftl`
//! format:
//!
//! ```ftl
//! app-greeting = Hello, { $name }!
//! menu-file = ~F~ile
//! ```
//!
//! [`tr`] (or the [`tr!`](crate::tr) macro, for arguments) looks the id up in
//! the current locale, then in the locales it falls back to, and last in
//! English. Turbo Vision ships its own messages in English, Spanish (`es`)
//! and Catalan (`ca`); [`add_translations`] adds your application's, for any
//! language, and may also replace a built-in one.
//!
//! The locale starts as English. An application chooses it once, before it
//! builds its menus and dialogs: [`set_locale_from_env`] takes the user's
//! (`LC_ALL`, `LC_MESSAGES`, `LANG`), [`set_locale`] a given one. Views keep
//! the text they were built with, so changing the locale later affects the
//! dialogs opened afterwards.
//!
//! ```
//! use turbo_vision::i18n;
//! use turbo_vision::tr;
//!
//! i18n::add_translations("en", "app-greeting = Hello, { $name }!").unwrap();
//! i18n::add_translations("ca", "app-greeting = Hola, { $name }!").unwrap();
//!
//! i18n::set_locale("ca-ES");
//! assert_eq!(tr!("app-greeting", name = "Maria"), "Hola, Maria!");
//! assert_eq!(i18n::tr("button-cancel"), "~C~ancel·la"); // built in
//!
//! i18n::set_locale("de"); // no German: everything falls back to English
//! assert_eq!(tr!("app-greeting", name = "Maria"), "Hello, Maria!");
//! ```
//!
//! Like the command set, the catalogue is per thread: the UI thread's.
//!
//! With the `i18n` cargo feature off there is no Fluent: each message is
//! taken as plain text with `{ $name }` placeholders, which is enough for
//! the built-in English messages. Selectors and functions need the feature.

#[cfg(feature = "i18n")]
mod fluent_backend;
#[cfg(test)]
mod layout_check;
mod simple;

#[cfg(feature = "i18n")]
use fluent_backend as backend;
#[cfg(not(feature = "i18n"))]
use simple as backend;

use std::borrow::Cow;
use std::cell::RefCell;
use std::fmt;
use std::path::{Path, PathBuf};

/// The locale every lookup falls back to, and the one Turbo Vision starts in.
pub const DEFAULT_LOCALE: &str = "en";

/// The translations Turbo Vision ships, as (locale, Fluent source).
const BUILT_IN: &[(&str, &str)] = &[
    ("en", include_str!("../../locales/en/turbo-vision.ftl")),
    ("es", include_str!("../../locales/es/turbo-vision.ftl")),
    ("ca", include_str!("../../locales/ca/turbo-vision.ftl")),
];

/// The built-in Fluent source of `locale`, for tests elsewhere in the crate.
#[cfg(test)]
pub(crate) fn built_in_source(locale: &str) -> Option<&'static str> {
    BUILT_IN
        .iter()
        .find(|(tag, _)| *tag == locale)
        .map(|(_, source)| *source)
}

/// The locales Turbo Vision ships, for tests elsewhere in the crate.
#[cfg(test)]
pub(crate) fn built_in_locales() -> impl Iterator<Item = &'static str> {
    BUILT_IN.iter().map(|(tag, _)| *tag)
}

/// The messages of a plain Fluent `source`, for tests elsewhere in the crate.
#[cfg(test)]
pub(crate) use simple::parse as parse_plain;

/// A value for a message's `{ $name }` placeholder.
///
/// Built with `From`, usually by the [`tr!`](crate::tr) macro: text, or a
/// number that Fluent's selectors (`[one]`, `[other]`) can match on.
#[derive(Debug, Clone, PartialEq)]
pub enum Arg<'a> {
    /// Text, shown as it is.
    Text(Cow<'a, str>),
    /// A whole number.
    Int(i64),
    /// A number with a fraction.
    Float(f64),
}

impl fmt::Display for Arg<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Arg::Text(text) => f.write_str(text),
            Arg::Int(n) => write!(f, "{n}"),
            Arg::Float(n) => write!(f, "{n}"),
        }
    }
}

impl<'a> From<&'a str> for Arg<'a> {
    fn from(text: &'a str) -> Self {
        Arg::Text(Cow::Borrowed(text))
    }
}

impl<'a> From<&'a String> for Arg<'a> {
    fn from(text: &'a String) -> Self {
        Arg::Text(Cow::Borrowed(text))
    }
}

impl From<String> for Arg<'_> {
    fn from(text: String) -> Self {
        Arg::Text(Cow::Owned(text))
    }
}

macro_rules! int_arg {
    ($($t:ty),*) => {$(
        impl From<$t> for Arg<'_> {
            fn from(n: $t) -> Self {
                Arg::Int(i64::from(n))
            }
        }
    )*};
}
int_arg!(i8, i16, i32, i64, u8, u16, u32);

impl From<usize> for Arg<'_> {
    fn from(n: usize) -> Self {
        i64::try_from(n).map_or(Arg::Float(n as f64), Arg::Int)
    }
}

impl From<u64> for Arg<'_> {
    fn from(n: u64) -> Self {
        i64::try_from(n).map_or(Arg::Float(n as f64), Arg::Int)
    }
}

impl From<f32> for Arg<'_> {
    fn from(n: f32) -> Self {
        Arg::Float(f64::from(n))
    }
}

impl From<f64> for Arg<'_> {
    fn from(n: f64) -> Self {
        Arg::Float(n)
    }
}

/// Why [`add_translations`] refused a resource.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TranslationError {
    /// The locale tag as given.
    pub locale: String,
    /// What was wrong, one entry per problem.
    pub problems: Vec<String>,
}

impl fmt::Display for TranslationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "translations for '{}' rejected: {}",
            self.locale,
            self.problems.join("; ")
        )
    }
}

impl std::error::Error for TranslationError {}

/// The thread's translations: every resource added, by locale, and the
/// lookup chain of the current locale.
struct Catalog {
    /// Requested locale, normalised (`ca-ES`).
    requested: String,
    /// Locales to look in, best first; always ends with `en`.
    chain: Vec<String>,
    /// The resources of each locale, in the order they were added.
    store: backend::Store,
}

impl Catalog {
    fn new() -> Self {
        let mut store = backend::Store::default();
        for (locale, source) in BUILT_IN {
            store
                .add(locale, source)
                .unwrap_or_else(|problems| panic!("built-in {locale}.ftl: {problems:?}"));
        }
        let mut catalog = Self {
            requested: DEFAULT_LOCALE.to_string(),
            chain: Vec::new(),
            store,
        };
        catalog.rebuild_chain();
        catalog
    }

    fn rebuild_chain(&mut self) {
        self.chain = fallback_chain(&self.requested, &self.store.locales());
        self.store.select(&self.chain);
    }
}

thread_local! {
    static CATALOG: RefCell<Catalog> = RefCell::new(Catalog::new());
}

/// The message `id` in the current locale, or `id` itself when no locale
/// has it.
///
/// ```
/// assert_eq!(turbo_vision::i18n::tr("button-ok"), "~O~K");
/// ```
#[must_use]
pub fn tr(id: &str) -> String {
    tr_args(id, &[])
}

/// The message `id` with its placeholders filled from `args`. The
/// [`tr!`](crate::tr) macro is the short way to call it.
#[must_use]
pub fn tr_args(id: &str, args: &[(&str, Arg<'_>)]) -> String {
    CATALOG.with(|catalog| {
        let catalog = catalog.borrow();
        catalog.store.format(id, args).unwrap_or_else(|| {
            log::warn!("i18n: no message '{id}' in {:?}", catalog.chain);
            id.to_string()
        })
    })
}

/// Whether some locale of the current chain has the message `id`.
#[must_use]
pub fn has_message(id: &str) -> bool {
    CATALOG.with(|catalog| catalog.borrow().store.has(id))
}

/// Use `tag` (`"ca"`, `"es-MX"`, `"pt_BR.UTF-8"`) for the messages looked up
/// from now on, and return the locale actually used first. A locale with no
/// translations falls back to its language (`es-MX` to `es`), then to
/// English.
pub fn set_locale(tag: &str) -> String {
    CATALOG.with(|catalog| {
        let mut catalog = catalog.borrow_mut();
        catalog.requested = normalize_tag(tag).unwrap_or_else(|| DEFAULT_LOCALE.to_string());
        catalog.rebuild_chain();
        catalog.chain[0].clone()
    })
}

/// Use the user's locale ([`system_locale`]), or English when none is set,
/// and return the locale actually used first.
pub fn set_locale_from_env() -> String {
    set_locale(&system_locale().unwrap_or_else(|| DEFAULT_LOCALE.to_string()))
}

/// The locale asked for last, normalised (`ca-ES`); `en` at start.
#[must_use]
pub fn requested_locale() -> String {
    CATALOG.with(|catalog| catalog.borrow().requested.clone())
}

/// The locales messages are looked up in, best first. The last one is
/// always `en`.
#[must_use]
pub fn locale_chain() -> Vec<String> {
    CATALOG.with(|catalog| catalog.borrow().chain.clone())
}

/// The locales that have translations, built in or added.
#[must_use]
pub fn available_locales() -> Vec<String> {
    CATALOG.with(|catalog| catalog.borrow().store.locales())
}

/// Add Fluent messages for `locale`. A message with the id of an earlier one,
/// built in or added, replaces it.
///
/// # Errors
///
/// When `locale` is not a language tag or `source` does not parse; nothing
/// is added then.
pub fn add_translations(locale: &str, source: &str) -> Result<(), TranslationError> {
    let error = |problems| TranslationError {
        locale: locale.to_string(),
        problems,
    };
    let tag = normalize_tag(locale).ok_or_else(|| error(vec!["not a language tag".into()]))?;
    CATALOG.with(|catalog| {
        let mut catalog = catalog.borrow_mut();
        catalog.store.add(&tag, source).map_err(error)?;
        catalog.rebuild_chain();
        Ok(())
    })
}

/// Load the Fluent file at `path` with [`add_translations`].
///
/// # Errors
///
/// When the file cannot be read, or [`add_translations`] refuses it.
pub fn add_translations_file(
    locale: &str,
    path: impl AsRef<Path>,
) -> Result<(), Box<dyn std::error::Error>> {
    let source = std::fs::read_to_string(path)?;
    add_translations(locale, &source)?;
    Ok(())
}

/// The user's locale from `LC_ALL`, `LC_MESSAGES` or `LANG` (the first one
/// set), normalised: `ca_ES.UTF-8@valencia` gives `ca-ES`. `None` when none
/// is set, or for the `C` and `POSIX` locales.
#[must_use]
pub fn system_locale() -> Option<String> {
    ["LC_ALL", "LC_MESSAGES", "LANG"]
        .into_iter()
        .filter_map(|var| std::env::var(var).ok())
        .find(|value| !value.is_empty())
        .and_then(|value| normalize_tag(&value))
}

/// The version of a file for the current locale: with `ca-ES` asked for,
/// the first of `help/manual.ca-ES.md`, `help/manual.ca.md` and
/// `help/manual.en.md` that exists, else `path` itself. The requested tag and
/// its shorter forms are tried even when no translations exist for them, so
/// a help file can be regional while the messages are not.
#[must_use]
pub fn localized_path(path: impl AsRef<Path>) -> PathBuf {
    let path = path.as_ref();
    let (Some(stem), Some(parent)) = (path.file_stem(), path.parent()) else {
        return path.to_path_buf();
    };
    let extension = path.extension();
    let mut locales: Vec<String> = Vec::new();
    let mut prefix = requested_locale();
    loop {
        locales.push(prefix.clone());
        match prefix.rfind('-') {
            Some(cut) => prefix.truncate(cut),
            None => break,
        }
    }
    for locale in locale_chain() {
        if !locales.contains(&locale) {
            locales.push(locale);
        }
    }
    locales
        .iter()
        .map(|locale| {
            let mut name = stem.to_os_string();
            name.push(".");
            name.push(locale);
            if let Some(extension) = extension {
                name.push(".");
                name.push(extension);
            }
            parent.join(name)
        })
        .find(|candidate| candidate.is_file())
        .unwrap_or_else(|| path.to_path_buf())
}

/// Turn a POSIX locale (`ca_ES.UTF-8@euro`) or a language tag (`ca-es`)
/// into the canonical tag (`ca-ES`). `None` for `C`, `POSIX` and anything
/// that is not a tag.
fn normalize_tag(raw: &str) -> Option<String> {
    let base = raw.split(['.', '@']).next()?.trim();
    if base.is_empty() || base == "C" || base == "POSIX" {
        return None;
    }
    let mut parts = base.split(['_', '-']);
    let language = parts.next()?;
    if !(2..=8).contains(&language.len()) || !language.chars().all(|c| c.is_ascii_alphabetic()) {
        return None;
    }
    let mut tag = language.to_ascii_lowercase();
    for part in parts {
        if part.is_empty() || part.len() > 8 || !part.chars().all(|c| c.is_ascii_alphanumeric()) {
            return None;
        }
        tag.push('-');
        match part.len() {
            // Region: ES, 419.
            2 | 3 => tag.push_str(&part.to_ascii_uppercase()),
            // Script: Latn.
            4 => {
                let mut chars = part.chars();
                tag.extend(chars.next().map(|c| c.to_ascii_uppercase()));
                tag.push_str(&chars.as_str().to_ascii_lowercase());
            }
            _ => tag.push_str(&part.to_ascii_lowercase()),
        }
    }
    Some(tag)
}

/// The locales to look in for `requested`, best first: the tag itself, then
/// each shorter prefix (`ca-ES-valencia`, `ca-ES`, `ca`), then any other
/// locale of the same language, then `en`. Only locales in `available`
/// count, except `en`, which always ends the chain.
fn fallback_chain(requested: &str, available: &[String]) -> Vec<String> {
    let mut chain: Vec<String> = Vec::new();
    let mut push = |locale: &str| {
        if !chain.iter().any(|l| l == locale) {
            chain.push(locale.to_string());
        }
    };
    let mut prefix = requested;
    loop {
        if available.iter().any(|l| l == prefix) {
            push(prefix);
        }
        match prefix.rfind('-') {
            Some(cut) => prefix = &prefix[..cut],
            None => break,
        }
    }
    for locale in available {
        if locale.split('-').next() == Some(prefix) {
            push(locale);
        }
    }
    push(DEFAULT_LOCALE);
    chain
}

/// Look up a message, with `name = value` placeholders.
///
/// ```
/// use turbo_vision::tr;
///
/// turbo_vision::i18n::add_translations("en", "files-left = { $count } files left").unwrap();
/// assert_eq!(tr!("files-left", count = 3), "3 files left");
/// assert_eq!(tr!("button-ok"), "~O~K");
/// ```
#[macro_export]
macro_rules! tr {
    ($id:expr $(,)?) => {
        $crate::i18n::tr($id)
    };
    ($id:expr, $($name:ident = $value:expr),+ $(,)?) => {
        $crate::i18n::tr_args(
            $id,
            &[$((stringify!($name), $crate::i18n::Arg::from($value))),+],
        )
    };
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn owned(locales: &[&str]) -> Vec<String> {
        locales.iter().map(ToString::to_string).collect()
    }

    #[test]
    fn normalizes_posix_locales_and_tags() {
        assert_eq!(
            normalize_tag("ca_ES.UTF-8@valencia").as_deref(),
            Some("ca-ES")
        );
        assert_eq!(normalize_tag("es-mx").as_deref(), Some("es-MX"));
        assert_eq!(normalize_tag("sr_latn_rs").as_deref(), Some("sr-Latn-RS"));
        assert_eq!(normalize_tag("es-419").as_deref(), Some("es-419"));
        assert_eq!(normalize_tag("de").as_deref(), Some("de"));
        assert_eq!(normalize_tag("C"), None);
        assert_eq!(normalize_tag("POSIX.UTF-8"), None);
        assert_eq!(normalize_tag(""), None);
        assert_eq!(normalize_tag("x"), None);
        assert_eq!(normalize_tag("ca ES"), None);
    }

    #[test]
    fn fallback_chain_goes_from_region_to_language_to_english() {
        let available = owned(&["en", "es", "ca", "ca-ES-valencia"]);
        assert_eq!(
            fallback_chain("ca-ES", &available),
            owned(&["ca", "ca-ES-valencia", "en"])
        );
        assert_eq!(
            fallback_chain("ca-ES-valencia", &available),
            owned(&["ca-ES-valencia", "ca", "en"])
        );
        assert_eq!(fallback_chain("es-MX", &available), owned(&["es", "en"]));
        assert_eq!(fallback_chain("de-DE", &available), owned(&["en"]));
        assert_eq!(fallback_chain("en-GB", &available), owned(&["en"]));
    }

    #[test]
    fn built_in_messages_are_english_by_default() {
        assert_eq!(requested_locale(), "en");
        assert_eq!(tr("button-cancel"), "~C~ancel");
        assert_eq!(tr("help-title"), "Help");
    }

    #[test]
    fn set_locale_switches_built_in_messages() {
        assert_eq!(set_locale("es_ES.UTF-8"), "es");
        assert_eq!(tr("button-cancel"), "~C~ancelar");
        assert_eq!(set_locale("ca"), "ca");
        assert_eq!(tr("button-cancel"), "~C~ancel·la");
        assert_eq!(set_locale("ja"), "en");
        assert_eq!(tr("button-cancel"), "~C~ancel");
    }

    #[test]
    fn unknown_ids_come_back_as_themselves() {
        assert_eq!(tr("no-such-message"), "no-such-message");
        assert!(!has_message("no-such-message"));
        assert!(has_message("button-ok"));
    }

    #[test]
    fn multiline_messages_keep_their_blank_lines() {
        assert_eq!(
            tr!("editor-save-changes", name = "notes.txt"),
            "notes.txt has been modified.\n\nSave changes?"
        );
        assert_eq!(
            tr("color-dialog-sample"),
            "Sample text with\nselected colors"
        );
    }

    #[test]
    fn placeables_are_not_wrapped_in_bidi_isolation_marks() {
        let text = tr!("color-selector-selected", color = 4, hex = "0x04");
        assert_eq!(text, "Selected: 4 (0x04)");
        assert!(!text.contains(['\u{2068}', '\u{2069}']));
    }

    #[test]
    fn added_translations_extend_and_override() {
        add_translations("en", "app-title = My App\nbutton-ok = Fine").unwrap();
        add_translations("pt-BR", "app-title = Meu App").unwrap();
        assert_eq!(tr("app-title"), "My App");
        assert_eq!(tr("button-ok"), "Fine");
        assert_eq!(set_locale("pt-BR"), "pt-BR");
        assert_eq!(tr("app-title"), "Meu App");
        assert_eq!(tr("button-ok"), "Fine"); // falls back to (overridden) English
        assert!(available_locales().contains(&"pt-BR".to_string()));
    }

    #[test]
    fn bad_translations_are_refused() {
        let err = add_translations("not a tag", "a = b").unwrap_err();
        assert_eq!(err.locale, "not a tag");
        let err = add_translations("fr", "this is not fluent").unwrap_err();
        assert!(!err.problems.is_empty());
        assert!(!available_locales().contains(&"fr".to_string()));
    }

    #[test]
    fn localized_path_prefers_the_locale_file() {
        let dir = tempfile::tempdir().unwrap();
        let base = dir.path().join("manual.md");
        std::fs::write(&base, "en").unwrap();
        std::fs::write(dir.path().join("manual.ca.md"), "ca").unwrap();
        assert_eq!(localized_path(&base), base);
        set_locale("ca-ES");
        assert_eq!(localized_path(&base), dir.path().join("manual.ca.md"));
        set_locale("es");
        assert_eq!(localized_path(&base), base);
        // A regional file counts even with no ca-ES translations.
        std::fs::write(dir.path().join("manual.ca-ES.md"), "ca-ES").unwrap();
        set_locale("ca_ES.UTF-8");
        assert_eq!(localized_path(&base), dir.path().join("manual.ca-ES.md"));
        set_locale("en");
    }

    /// The ids of each built-in locale, in file order.
    fn built_in_ids() -> Vec<(&'static str, Vec<String>)> {
        BUILT_IN
            .iter()
            .map(|(locale, source)| {
                let messages = simple::parse(source).unwrap_or_else(|p| panic!("{locale}: {p:?}"));
                (*locale, messages.into_iter().map(|(id, _)| id).collect())
            })
            .collect()
    }

    #[test]
    fn every_built_in_locale_has_exactly_the_english_ids() {
        let locales = built_in_ids();
        let (_, english) = &locales[0];
        for (locale, ids) in &locales[1..] {
            let missing: Vec<_> = english.iter().filter(|id| !ids.contains(id)).collect();
            let extra: Vec<_> = ids.iter().filter(|id| !english.contains(id)).collect();
            assert!(missing.is_empty(), "{locale}.ftl lacks {missing:?}");
            assert!(extra.is_empty(), "{locale}.ftl has unknown ids {extra:?}");
        }
    }

    /// The messages shown together in one built-in dialog: their hot keys
    /// must differ in every locale.
    const HOT_KEY_GROUPS: &[&[&str]] = &[
        &["button-yes", "button-no", "button-ok", "button-cancel"],
        &["find-label", "button-ok", "button-cancel"],
        &["find-label", "replace-label", "button-ok", "button-cancel"],
        &["goto-line-label", "button-ok", "button-cancel"],
        &[
            "file-dialog-name",
            "file-dialog-files",
            "file-dialog-show-hidden",
            "file-dialog-open",
            "button-cancel",
        ],
        &[
            "file-dialog-name",
            "file-dialog-files",
            "file-dialog-show-hidden",
            "file-dialog-save",
            "button-cancel",
        ],
        &[
            "chdir-name",
            "chdir-tree",
            "button-ok",
            "chdir-chdir",
            "chdir-revert",
        ],
        &["button-ok", "button-cancel"],
    ];

    /// The hot key of `text`: the letter between the `~`, lowercased.
    fn hot_key(text: &str) -> Option<String> {
        let start = text.find('~')? + 1;
        let end = start + text[start..].find('~')?;
        Some(text[start..end].to_lowercase())
    }

    #[test]
    fn hot_keys_are_unique_within_each_dialog_in_every_locale() {
        for (locale, source) in BUILT_IN {
            let messages: HashMap<String, String> =
                simple::parse(source).unwrap().into_iter().collect();
            for group in HOT_KEY_GROUPS {
                let mut seen: Vec<(String, &str)> = Vec::new();
                for id in *group {
                    let Some(key) = hot_key(&messages[*id]) else {
                        continue;
                    };
                    if let Some((_, other)) = seen.iter().find(|(k, _)| *k == key) {
                        panic!("{locale}: '{id}' and '{other}' both use Alt+{key}");
                    }
                    seen.push((key, id));
                }
            }
        }
    }

    /// Messages shown in a space of fixed width, with the most columns
    /// (widest line, `~` markers left out) each may take. The other
    /// messages sit in views sized from their text.
    const FIXED_SLOTS: &[(&str, usize, &str)] = &[
        // Help index: "Search:" label 8 wide before its input, "Topics:" 10.
        ("help-index-search", 7, "help_index.rs: Label (2, 4, 10, 5)"),
        ("help-index-topics", 9, "help_index.rs: Label (2, 6, 12, 7)"),
        // Colour dialog, as laid out in a 52-column dialog (the gallery's).
        (
            "color-dialog-prompt",
            46,
            "color_dialog.rs: StaticText (2, 2, w - 4)",
        ),
        (
            "color-dialog-foreground",
            18,
            "color_dialog.rs: StaticText (2, 4, 20, 5)",
        ),
        (
            "color-dialog-background",
            18,
            "color_dialog.rs: StaticText (2, 9, 20, 10)",
        ),
        (
            "color-dialog-preview",
            20,
            "color_dialog.rs: StaticText (28, 5, w - 4)",
        ),
        (
            "color-dialog-sample",
            20,
            "color_dialog.rs: StaticText (28, 6, w - 4)",
        ),
        (
            "color-selector-selected",
            24,
            "color_dialog.rs: ColorSelector (2, 5, 26, 8)",
        ),
        // Change directory: the buttons grow up to 14 columns, 10 of text.
        ("button-ok", 10, "chdir_dialog.rs: buttons at x 53..67"),
        ("chdir-chdir", 10, "chdir_dialog.rs: buttons at x 53..67"),
        ("chdir-revert", 10, "chdir_dialog.rs: buttons at x 53..67"),
        // Frame titles: the message box is at least 30 wide; its title has
        // an icon, the close box and the frame around it.
        ("msgbox-warning", 18, "msgbox.rs: title of a 30-column box"),
        ("msgbox-error", 18, "msgbox.rs: title of a 30-column box"),
        (
            "msgbox-information",
            18,
            "msgbox.rs: title of a 30-column box",
        ),
        ("msgbox-confirm", 18, "msgbox.rs: title of a 30-column box"),
        ("msgbox-about", 18, "msgbox.rs: title of a 30-column box"),
        ("msgbox-message", 18, "msgbox.rs: title of a 30-column box"),
        ("history-title", 20, "history_window.rs: 30-column window"),
        // The "Show hidden" box starts at column 14 or later and must end
        // before the buttons of a 60-column file dialog (column 45).
        (
            "file-dialog-show-hidden",
            27,
            "file_dialog.rs: CheckBox (14, 5, ..)",
        ),
    ];

    #[test]
    fn translations_fit_their_fixed_slots() {
        use unicode_width::UnicodeWidthStr;
        // The widest values the placeholders take in those slots.
        let args = [
            ("color", Arg::from(15)),
            ("hex", Arg::from("0x0F")),
            ("name", Arg::from("Untitled")),
        ];
        let mut too_wide = Vec::new();
        for (locale, source) in BUILT_IN {
            let messages: HashMap<String, String> =
                simple::parse(source).unwrap().into_iter().collect();
            for (id, max, slot) in FIXED_SLOTS {
                let text = simple::substitute(&messages[*id], &args).replace('~', "");
                let width = text.lines().map(UnicodeWidthStr::width).max().unwrap_or(0);
                if width > *max {
                    too_wide.push(format!(
                        "{locale}: '{id}' is {width} > {max} ({slot}): {text:?}"
                    ));
                }
            }
        }
        assert!(too_wide.is_empty(), "{}", too_wide.join("\n"));
    }

    #[test]
    fn every_hot_key_can_be_pressed_with_alt() {
        // Alt+letter exists for A-Z only: a hot key on `Á` or `Ç` could be
        // typed, but never reached with Alt from an input line.
        for (locale, source) in BUILT_IN {
            for (id, text) in simple::parse(source).unwrap() {
                if let Some(key) = hot_key(&text) {
                    let letter = key.chars().next().unwrap_or(' ');
                    assert!(
                        key.chars().count() == 1 && crate::core::event::alt_code(letter).is_some(),
                        "{locale}: '{id}' marks {key:?}, which has no Alt key"
                    );
                }
            }
        }
    }

    #[test]
    fn every_hot_key_message_has_one_marked_letter() {
        for (locale, source) in BUILT_IN {
            for (id, text) in simple::parse(source).unwrap() {
                let tildes = text.matches('~').count();
                assert!(
                    tildes == 0 || tildes == 2,
                    "{locale}: '{id}' has {tildes} '~'"
                );
            }
        }
    }

    #[test]
    fn the_simple_reader_agrees_with_fluent_on_built_in_messages() {
        // With the feature off, `simple` is the backend; it must give the
        // same text for the built-in files as Fluent does.
        let args = [
            ("name", Arg::from("a.txt")),
            ("color", Arg::from(4)),
            ("hex", Arg::from("0x04")),
        ];
        for (locale, source) in BUILT_IN {
            set_locale(locale);
            for (id, pattern) in simple::parse(source).unwrap() {
                assert_eq!(
                    tr_args(&id, &args),
                    simple::substitute(&pattern, &args),
                    "{locale}: {id}"
                );
            }
        }
    }

    #[test]
    fn arguments_display_as_text_or_numbers() {
        assert_eq!(Arg::from("a").to_string(), "a");
        assert_eq!(Arg::from(42u16).to_string(), "42");
        assert_eq!(Arg::from(usize::MAX), Arg::Float(usize::MAX as f64));
        assert_eq!(Arg::from(2.5).to_string(), "2.5");
    }
}
