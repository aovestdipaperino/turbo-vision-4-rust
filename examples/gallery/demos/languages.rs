//! Text in the user's language, with Project Fluent. Turbo Vision's own
//! buttons and dialogs come in English, Spanish and Catalan; the
//! application adds its messages as Fluent text and looks them up by id.
//! Pick a language, then Ask or Open: the dialog's title, buttons and
//! labels follow. A language without a translation falls back to English.
//!
//! Parameters:
//! - `i18n::add_translations(locale, ftl)`: the application's messages for
//!   one language, as `id = text` lines; a built-in id is replaced.
//! - `i18n::set_locale(tag)`: "ca", "es-MX", "ca_ES.UTF-8"; looks in
//!   ca-ES, then ca, then en. `set_locale_from_env()` reads LANG.
//! - `tr!(id, name = value)`: the message in the current language, its
//!   `{ $name }` placeholders filled in.
//!
//! See also: Message boxes, File dialogs

use crate::panel::Panel;
use turbo_vision::app::Application;
use turbo_vision::core::command::{CM_USER, CommandId};
use turbo_vision::core::geometry::Rect;
use turbo_vision::i18n;
use turbo_vision::tr;
use turbo_vision::views::button::Button;
use turbo_vision::views::file_dialog::FileDialog;
use turbo_vision::views::msgbox::{confirmation_box_yes_no, message_box_ok};

const ENGLISH: CommandId = CM_USER + 100;
const SPANISH: CommandId = CM_USER + 101;
const CATALAN: CommandId = CM_USER + 102;
const ASK: CommandId = CM_USER + 103;
const OPEN: CommandId = CM_USER + 104;

/// The demo's own messages; the dialogs' buttons are built in.
const MESSAGES: &[(&str, &str)] = &[
    (
        "en",
        "demo-now = Language: { $locale }\ndemo-question = Keep this language?",
    ),
    (
        "es",
        "demo-now = Idioma: { $locale }\ndemo-question = ¿Mantener este idioma?",
    ),
    (
        "ca",
        "demo-now = Idioma: { $locale }\ndemo-question = Voleu mantenir aquest idioma?",
    ),
];

pub fn build(panel: &mut Panel) {
    for (locale, source) in MESSAGES {
        i18n::add_translations(locale, source).expect("valid Fluent");
    }
    // Each language is named in itself, so it can be found from any other.
    panel.add(Button::new(
        Rect::new(0, 0, 13, 2),
        "~E~nglish",
        ENGLISH,
        true,
    ));
    panel.add(Button::new(
        Rect::new(15, 0, 28, 2),
        "E~s~pañol",
        SPANISH,
        false,
    ));
    panel.add(Button::new(
        Rect::new(30, 0, 43, 2),
        "~C~atalà",
        CATALAN,
        false,
    ));
    panel.add(Button::new(Rect::new(0, 3, 13, 5), "~A~sk", ASK, false));
    panel.add(Button::new(
        Rect::new(15, 3, 28, 5),
        "~O~pen...",
        OPEN,
        false,
    ));
}

pub fn handle(app: &mut Application, command: CommandId) -> bool {
    let locale = match command {
        ENGLISH => "en",
        SPANISH => "es",
        CATALAN => "ca",
        ASK => {
            confirmation_box_yes_no(app, &tr!("demo-question"));
            return true;
        }
        OPEN => {
            // The title is the caller's; the rest is translated.
            let title = tr!("file-dialog-open").replace('~', "");
            FileDialog::new(Rect::new(8, 2, 72, 20), &title, "*", None)
                .build()
                .execute(app);
            return true;
        }
        _ => return false,
    };
    // Views keep the text they were built with: set the locale, then open
    // the dialogs.
    let used = app.set_locale(locale);
    message_box_ok(app, &tr!("demo-now", locale = used));
    true
}
