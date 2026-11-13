// (C) 2025 - Enzo Lombardi
// Translations: an application in English, Spanish and Catalan.
//
// The application's own texts are Fluent messages, added with
// `add_translations`; Turbo Vision's (OK, Cancel, the file dialog, the
// message box titles) come with the crate. It starts in the user's locale
// (LANG=ca_ES.UTF-8 cargo run --example i18n) and the Language menu
// switches while it runs: the menu bar and status line are built again,
// and every dialog opened afterwards is in the new language.

use turbo_vision::app::{AppHandler, Application};
use turbo_vision::core::command::{CM_QUIT, CM_USER, CM_YES, CommandId};
use turbo_vision::core::event::Event;
use turbo_vision::core::geometry::Rect;
use turbo_vision::core::menu_data::{Menu, MenuItem, MenuItemBuilder};
use turbo_vision::core::status_data::StatusItemBuilder;
use turbo_vision::i18n;
use turbo_vision::tr;
use turbo_vision::views::file_dialog::FileDialog;
use turbo_vision::views::menu_bar::{MenuBar, SubMenu};
use turbo_vision::views::msgbox::{confirmation_box_yes_no, message_box_ok};
use turbo_vision::views::status_line::StatusLine;

const CM_OPEN_FILE: CommandId = CM_USER;
const CM_ASK: CommandId = CM_USER + 1;
const CM_LANG_EN: CommandId = CM_USER + 10;
const CM_LANG_ES: CommandId = CM_USER + 11;
const CM_LANG_CA: CommandId = CM_USER + 12;

const EN: &str = "
menu-file = ~F~ile
menu-open = ~O~pen...
menu-ask = ~A~sk a question
menu-exit = E~x~it
menu-language = ~L~anguage
status-exit = ~Alt-X~ Exit
question = Do you like this language?
answer = { $yes ->
    [yes] Glad to hear it.
   *[no] Try another one in the Language menu.
}
opened = { $name } chosen.
";

const ES: &str = "
menu-file = ~A~rchivo
menu-open = A~b~rir...
menu-ask = ~P~reguntar
menu-exit = ~S~alir
menu-language = ~I~dioma
status-exit = ~Alt-X~ Salir
question = ¿Le gusta este idioma?
answer = { $yes ->
    [yes] Nos alegra saberlo.
   *[no] Pruebe otro en el menú Idioma.
}
opened = Ha elegido { $name }.
";

const CA: &str = "
menu-file = ~F~itxer
menu-open = ~O~bre...
menu-ask = ~P~regunta
menu-exit = ~S~urt
menu-language = ~I~dioma
status-exit = ~Alt-X~ Surt
question = Us agrada aquest idioma?
answer = { $yes ->
    [yes] Ens n'alegrem.
   *[no] Proveu-ne un altre al menú Idioma.
}
opened = Heu triat { $name }.
";

/// The menu bar and status line in the current locale.
fn build_chrome(app: &mut Application) {
    let (width, height) = app.terminal.size();
    let item = |text: &str, command| MenuItemBuilder::new().text(text).command(command).build();

    let mut menu_bar = MenuBar::new(Rect::new(0, 0, width, 1));
    menu_bar.add_submenu(SubMenu::new(
        &tr!("menu-file"),
        Menu::from_items(vec![
            item(&tr!("menu-open"), CM_OPEN_FILE),
            item(&tr!("menu-ask"), CM_ASK),
            MenuItem::separator(),
            MenuItemBuilder::new()
                .text(tr!("menu-exit"))
                .command(CM_QUIT)
                .key("Alt+X")
                .build(),
        ]),
    ));
    // Each language is named in itself, so it can be found from any other.
    menu_bar.add_submenu(SubMenu::new(
        &tr!("menu-language"),
        Menu::from_items(vec![
            item("~E~nglish", CM_LANG_EN),
            item("E~s~pañol", CM_LANG_ES),
            item("~C~atalà", CM_LANG_CA),
        ]),
    ));
    app.set_menu_bar(menu_bar);

    app.set_status_line(StatusLine::new(
        Rect::new(0, height - 1, width, height),
        vec![
            StatusItemBuilder::new()
                .text(tr!("status-exit"))
                .key("Alt+X")
                .command(CM_QUIT)
                .build(),
        ],
    ));
}

struct Demo;

impl AppHandler for Demo {
    fn handle_command(&mut self, app: &mut Application, command: CommandId, _: &Event) -> bool {
        let locale = match command {
            CM_LANG_EN => "en",
            CM_LANG_ES => "es",
            CM_LANG_CA => "ca",
            CM_OPEN_FILE => {
                // The title is the application's; the rest of the dialog
                // is translated by Turbo Vision.
                let title = tr!("menu-open").replace(['~', '.'], "");
                let dialog = FileDialog::new(Rect::new(5, 2, 69, 20), &title, "*", None);
                if let Some(path) = dialog.build().execute(app) {
                    let name = path.display().to_string();
                    message_box_ok(app, &tr!("opened", name = name));
                }
                return true;
            }
            CM_ASK => {
                let yes = confirmation_box_yes_no(app, &tr!("question")) == CM_YES;
                message_box_ok(app, &tr!("answer", yes = if yes { "yes" } else { "no" }));
                return true;
            }
            _ => return false,
        };
        app.set_locale(locale);
        build_chrome(app);
        true
    }
}

fn main() -> turbo_vision::core::error::Result<()> {
    let mut app = Application::new()?;
    for (locale, source) in [("en", EN), ("es", ES), ("ca", CA)] {
        app.add_translations(locale, source)
            .unwrap_or_else(|e| panic!("{e}"));
    }
    app.set_locale_from_env();
    log::info!("locale chain: {:?}", i18n::locale_chain());
    build_chrome(&mut app);
    app.run_with(&mut Demo);
    Ok(())
}
