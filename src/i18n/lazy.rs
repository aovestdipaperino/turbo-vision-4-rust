//! Messages translated when they are shown, not when they are made.
//!
//! [`tr`](super::tr) translates at once, in the current locale of the
//! calling thread. That is right for text about to go on screen, and wrong
//! for text made earlier or elsewhere:
//!
//! - **On another thread.** The catalogue is per thread. A worker thread
//!   has only Turbo Vision's messages, in English, and none of the
//!   application's: `tr!` there returns the id itself.
//! - **Before the locale is chosen**, such as defaults in a struct built at
//!   start-up, or in a `const` table.
//! - **Kept across a change of language**, such as the last error, a status
//!   message or a log that is shown again later.
//!
//! A [`LazyMessage`] holds the id and the arguments, and is translated each
//! time it is displayed, with the locale of the thread that displays it.

use super::Arg;
use std::borrow::Cow;
use std::fmt;

/// A message id and its arguments, translated when it is displayed.
///
/// Make one with [`tr_lazy!`](crate::tr_lazy) or [`LazyMessage::new`],
/// keep it or send it to the UI thread (it is `Send`), and turn it into
/// text there with [`resolve`](Self::resolve), `to_string()` or `{}`:
///
/// ```
/// use turbo_vision::i18n::{self, LazyMessage};
/// use turbo_vision::tr_lazy;
///
/// i18n::add_translations("en", "app-copied = Copied { $count } files.").unwrap();
/// i18n::add_translations("ca", "app-copied = S'han copiat { $count } fitxers.").unwrap();
///
/// // Made on a worker thread, which has no application messages...
/// let done: LazyMessage = std::thread::spawn(|| tr_lazy!("app-copied", count = 3))
///     .join()
///     .unwrap();
///
/// // ...and translated on the UI thread, in its language.
/// i18n::set_locale("ca");
/// assert_eq!(done.to_string(), "S'han copiat 3 fitxers.");
/// i18n::set_locale("en");
/// assert_eq!(done.resolve(), "Copied 3 files.");
/// ```
#[derive(Debug, Clone, PartialEq)]
pub struct LazyMessage {
    id: Cow<'static, str>,
    args: Vec<(&'static str, LazyArg)>,
}

/// A value for a placeholder of a [`LazyMessage`]: a plain value, or
/// another message, translated together with the one that holds it.
#[derive(Debug, Clone, PartialEq)]
pub enum LazyArg {
    /// Text or a number, as for [`tr!`](crate::tr).
    Value(Arg<'static>),
    /// A message translated in the same locale as the one that holds it.
    Message(LazyMessage),
}

impl<T: Into<Arg<'static>>> From<T> for LazyArg {
    fn from(value: T) -> Self {
        LazyArg::Value(value.into())
    }
}

impl From<LazyMessage> for LazyArg {
    fn from(message: LazyMessage) -> Self {
        LazyArg::Message(message)
    }
}

impl LazyMessage {
    /// The message `id`, with no arguments yet. It is `const`, so a table
    /// of messages can be a `const` or a `static`:
    ///
    /// ```
    /// use turbo_vision::i18n::LazyMessage;
    ///
    /// const CANCELLED: LazyMessage = LazyMessage::new("button-cancel");
    /// assert_eq!(CANCELLED.resolve(), "~C~ancel");
    /// ```
    #[must_use]
    pub const fn new(id: &'static str) -> Self {
        Self {
            id: Cow::Borrowed(id),
            args: Vec::new(),
        }
    }

    /// The message whose id is only known at run time.
    #[must_use]
    pub fn from_id(id: impl Into<String>) -> Self {
        Self {
            id: Cow::Owned(id.into()),
            args: Vec::new(),
        }
    }

    /// The same message with `{ $name }` set to `value`: text, a number, or
    /// another [`LazyMessage`]. Setting a name again replaces its value.
    ///
    /// A borrowed `&str` must be `'static`; pass a `String` otherwise
    /// (`path.display().to_string()`).
    #[must_use]
    pub fn arg(mut self, name: &'static str, value: impl Into<LazyArg>) -> Self {
        let value = value.into();
        match self.args.iter_mut().find(|(n, _)| *n == name) {
            Some((_, old)) => *old = value,
            None => self.args.push((name, value)),
        }
        self
    }

    /// The message id.
    #[must_use]
    pub fn id(&self) -> &str {
        &self.id
    }

    /// The message in the calling thread's current locale, its arguments
    /// filled in. Messages given as arguments are translated first, in the
    /// same locale.
    #[must_use]
    pub fn resolve(&self) -> String {
        let args: Vec<(&str, Arg<'_>)> = self
            .args
            .iter()
            .map(|(name, value)| {
                let value = match value {
                    LazyArg::Value(arg) => arg.clone(),
                    LazyArg::Message(message) => Arg::from(message.resolve()),
                };
                (*name, value)
            })
            .collect();
        super::tr_args(&self.id, &args)
    }
}

impl fmt::Display for LazyMessage {
    /// The message in the current locale: [`resolve`](Self::resolve).
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.resolve())
    }
}

impl From<&LazyMessage> for String {
    fn from(message: &LazyMessage) -> Self {
        message.resolve()
    }
}

/// A [`LazyMessage`]: like [`tr!`](crate::tr), with the same arguments, but
/// translated when it is displayed instead of now.
///
/// ```
/// use turbo_vision::tr_lazy;
///
/// let saved = tr_lazy!("editor-save-changes", name = "notes.txt");
/// assert_eq!(saved.id(), "editor-save-changes");
/// assert!(saved.to_string().starts_with("notes.txt has been modified."));
/// ```
#[macro_export]
macro_rules! tr_lazy {
    ($id:expr $(,)?) => {
        $crate::i18n::LazyMessage::new($id)
    };
    ($id:expr, $($name:ident = $value:expr),+ $(,)?) => {
        $crate::i18n::LazyMessage::new($id)$(.arg(stringify!($name), $value))+
    };
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::i18n::{add_translations, set_locale};
    use std::sync::mpsc;

    fn app_messages() {
        add_translations(
            "en",
            "lazy-saved = Saved { $name }.\n\
             lazy-failed = Could not save { $name }: { $reason }\n\
             lazy-disk-full = the disk is full",
        )
        .unwrap();
        add_translations(
            "ca",
            "lazy-saved = S'ha desat { $name }.\n\
             lazy-failed = No s'ha pogut desar { $name }: { $reason }\n\
             lazy-disk-full = el disc és ple",
        )
        .unwrap();
    }

    #[test]
    fn a_message_from_a_worker_thread_is_translated_on_the_ui_thread() {
        app_messages();
        set_locale("ca");
        let (send, receive) = mpsc::channel();
        std::thread::spawn(move || {
            // The worker has no application messages: tr! gives the id.
            assert_eq!(crate::tr!("lazy-saved", name = "a.txt"), "lazy-saved");
            send.send(tr_lazy!("lazy-saved", name = "a.txt")).unwrap();
        })
        .join()
        .unwrap();
        let message = receive.recv().unwrap();
        assert_eq!(message.to_string(), "S'ha desat a.txt.");
        set_locale("en");
    }

    #[test]
    fn a_kept_message_follows_a_change_of_language() {
        app_messages();
        let message = tr_lazy!("lazy-saved", name = "a.txt");
        set_locale("en");
        assert_eq!(message.resolve(), "Saved a.txt.");
        set_locale("ca");
        assert_eq!(message.resolve(), "S'ha desat a.txt.");
        set_locale("en");
    }

    #[test]
    fn a_message_argument_is_translated_in_the_same_locale() {
        app_messages();
        set_locale("ca");
        let failed = tr_lazy!(
            "lazy-failed",
            name = "a.txt",
            reason = tr_lazy!("lazy-disk-full")
        );
        assert_eq!(
            failed.resolve(),
            "No s'ha pogut desar a.txt: el disc és ple"
        );
        set_locale("en");
        assert_eq!(failed.resolve(), "Could not save a.txt: the disk is full");
    }

    #[test]
    fn a_const_table_of_messages_is_translated_when_used() {
        const BUTTONS: [LazyMessage; 2] = [
            LazyMessage::new("button-ok"),
            LazyMessage::new("button-cancel"),
        ];
        set_locale("es");
        let shown: Vec<String> = BUTTONS.iter().map(String::from).collect();
        assert_eq!(shown, ["~A~ceptar", "~C~ancelar"]);
        set_locale("en");
    }

    #[test]
    fn arguments_take_numbers_text_and_replace_earlier_values() {
        let message = LazyMessage::from_id(String::from("lazy-saved"))
            .arg("name", "a.txt")
            .arg("name", String::from("b.txt"))
            .arg("count", 3u16);
        assert_eq!(message.id(), "lazy-saved");
        assert_eq!(
            message.args,
            [
                ("name", LazyArg::Value(Arg::from(String::from("b.txt")))),
                ("count", LazyArg::Value(Arg::Int(3))),
            ]
        );
    }

    #[test]
    fn a_missing_message_shows_its_id_like_tr() {
        assert_eq!(tr_lazy!("lazy-nowhere").to_string(), "lazy-nowhere");
    }

    #[test]
    fn a_lazy_message_can_cross_threads() {
        fn send_and_sync<T: Send + Sync>() {}
        send_and_sync::<LazyMessage>();
    }
}
