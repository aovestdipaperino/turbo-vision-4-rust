# Chapter 19: Localization

**Previous:** [Chapter 18 — Resources](chapter-18.md)

---

A program that speaks the user's language needs more than translated
words. The hot keys have to be chosen again. Longer text has to fit. Plurals
have to follow the language's own rules. And the user has to be able to pick
the language. Turbo Vision for Rust does this with
[Project Fluent](https://projectfluent.org), Mozilla's localization system,
through the `i18n` module.

The topics in this chapter include:

- What is translated, and by whom
- Translating an application, step by step
- Fluent messages: placeholders, plurals, several lines
- Choosing the language, and changing it while the program runs
- Hot keys in other languages
- Making room for longer text
- Help files per language
- Testing translations
- Adding a new language, to your application or to Turbo Vision

## What is translated, and by whom

The text on a Turbo Vision screen has two owners:

- **Turbo Vision's own text.** This is the OK and Cancel buttons, the message
  box titles, the Find and Replace boxes, the file and change-directory
  dialogs, the colour dialog, the editors' "Untitled" and "Save changes?",
  and the help and history windows. The crate ships these in English,
  Spanish (`es`) and Catalan (`ca`), in `locales/<locale>/turbo-vision.ftl`.
  You get them for free.
- **Your application's text.** This is your menus, status line, dialogs and
  messages. You write these as Fluent messages and add them to the same
  catalogue.

Both are looked up the same way: by an **id**, in the **current locale**.

```text
tr!("button-cancel")      en: ~C~ancel      es: ~C~ancelar      ca: ~C~ancel·la
tr!("menu-file")          your own: ~F~ile, ~A~rchivo, ~F~itxer
```

The catalogue belongs to the UI thread, like the command set. The program
starts in English. Nothing is translated until the application chooses a
locale, so an existing program behaves as before.

## Translating an application, step by step

### Step 1: write the messages

Write one Fluent file per language. Each line is `id = text`, and the text
keeps its `~` hot-key markers:

```ftl
# locales/en/app.ftl
menu-file = ~F~ile
menu-open = ~O~pen...
menu-exit = E~x~it
status-exit = ~Alt-X~ Exit
```

```ftl
# locales/ca/app.ftl
menu-file = ~F~itxer
menu-open = ~O~bre...
menu-exit = ~S~urt
status-exit = ~Alt-X~ Surt
```

Ids are lowercase words joined with `-`. Start them with the part of the
program they belong to (`menu-`, `status-`, `customer-dialog-`), so they
stay readable and never clash with the crate's ids (`button-`, `msgbox-`,
`file-dialog-` and so on).

### Step 2: add them, choose the locale, then build the views

```rust
use turbo_vision::app::Application;
use turbo_vision::core::command::CM_QUIT;
use turbo_vision::core::geometry::Rect;
use turbo_vision::core::menu_data::{Menu, MenuItemBuilder};
use turbo_vision::core::status_data::StatusItemBuilder;
use turbo_vision::tr;
use turbo_vision::views::menu_bar::{MenuBar, SubMenu};
use turbo_vision::views::status_line::StatusLine;

const EN: &str = "
menu-file = ~F~ile
menu-exit = E~x~it
status-exit = ~Alt-X~ Exit
";
const CA: &str = "
menu-file = ~F~itxer
menu-exit = ~S~urt
status-exit = ~Alt-X~ Surt
";

fn main() -> turbo_vision::core::error::Result<()> {
    let mut app = Application::new()?;

    // 1. Every language the program knows.
    app.add_translations("en", EN).expect("valid Fluent");
    app.add_translations("ca", CA).expect("valid Fluent");

    // 2. The user's language: LC_ALL, LC_MESSAGES or LANG; English if unset.
    app.set_locale_from_env();

    // 3. Only now build the views: each keeps the text it was made with.
    let (width, height) = app.terminal.size();
    let mut menu_bar = MenuBar::new(Rect::new(0, 0, width, 1));
    menu_bar.add_submenu(SubMenu::new(
        &tr!("menu-file"),
        Menu::from_items(vec![
            MenuItemBuilder::new()
                .text(tr!("menu-exit"))
                .command(CM_QUIT)
                .key("Alt+X")
                .build(),
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

    app.run();
    Ok(())
}
```

Run it with `LANG=ca_ES.UTF-8` and the menu reads *Fitxer*, the status
line *Alt-X Surt*. Every message box and file dialog the program opens
says *D'acord* and *Cancel·la*.

Keep the files in the program with `include_str!`, as the crate does with
its own. Or load them at run time with
`i18n::add_translations_file("ca", "locales/ca/app.ftl")`, so users can
correct a translation without a new build.

### Step 3: look messages up where the text is needed

`tr!` returns a `String`. Pass it where you would have written the literal.

```rust
dialog.add(Label::new(Rect::new(2, 1, 12, 2), &tr!("customer-name")));
message_box_ok(app, &tr!("customer-saved", name = customer.name.as_str()));
```

A message that is missing in the current language comes from the next one
in the chain (see below). If no language has it, the id itself comes back,
so a forgotten message shows up on screen as `customer-saved` instead of
crashing the program. A warning goes to the log too.

## Fluent messages

Fluent keeps grammar in the translation, where it belongs, instead of in
your code. This section covers the parts you will use most. The
[Fluent syntax guide](https://projectfluent.org/fluent/guide/) has the rest.

### Placeholders

A message names its variable parts. The code fills them by name, so a
translator can move them:

```ftl
customer-saved = { $name } has been saved.
```

```ftl
customer-saved = S'ha desat { $name }.
```

```rust
use turbo_vision::tr;

turbo_vision::i18n::add_translations("en", "customer-saved = { $name } has been saved.").unwrap();
assert_eq!(tr!("customer-saved", name = "Anna"), "Anna has been saved.");
```

`tr!` accepts text (`&str`, `String`) and numbers (any integer type, `f32`,
`f64`). Never build a sentence out of pieces with `format!`, such as
`format!("{} {}", tr!("saved"), name)`. Word order changes from language to
language, and only the translator knows it.

### Plurals and other choices

A *selector* picks a variant by the value of a variable. With a number,
the variants are the language's plural categories (`zero`, `one`, `two`,
`few`, `many`, `other`). Each language uses only the ones it has. The
variant marked `*` is the default.

```ftl
files-left = { $count ->
    [one] One file left.
   *[other] { $count } files left.
}
```

```rust
use turbo_vision::tr;

turbo_vision::i18n::add_translations(
    "en",
    "files-left = { $count ->\n    [one] One file left.\n   *[other] { $count } files left.\n}",
)
.unwrap();
assert_eq!(tr!("files-left", count = 1), "One file left.");
assert_eq!(tr!("files-left", count = 3), "3 files left.");
```

A selector can also match a word that your code passes in, such as
`{ $gender -> [female] … [male] … *[other] … }`, or `yes` and `no` as in
`examples/i18n.rs`.

### Several lines

Indented lines continue a message. A blank line inside it is kept, which is
how a message box gets its paragraphs:

```ftl
editor-save-changes =
    { $name } has been modified.

    Save changes?
```

Comments start with `#`. `##` and `###` mark sections. A comment right above
a message is the place to tell the translator what a placeholder holds, or
how much room there is.

## Choosing the language

| Call | Chooses |
|------|---------|
| `app.set_locale_from_env()` | The user's locale: `LC_ALL`, then `LC_MESSAGES`, then `LANG`. `C`, `POSIX` and unset mean English. |
| `app.set_locale("ca")` | A given locale. The tag can be `"ca"`, `"es-MX"`, `"pt_BR.UTF-8"`. |
| `i18n::set_locale(..)`, `i18n::set_locale_from_env()` | The same, without an `Application` (in tests, before it exists). |

Each returns the locale actually used first. The tag is normalised:
`ca_ES.UTF-8@valencia` becomes `ca-ES`. Messages are then looked up along
a **fallback chain**:

```text
requested ca-ES-valencia  →  ca-ES-valencia (if any) → ca-ES (if any) → ca → en → the id
requested es-MX           →  es → en → the id
requested de              →  en → the id
```

So one Spanish file serves Spain and Mexico. A `es-MX` file that changes
only a few words can sit on top of it, and English is always the last
resort. `i18n::locale_chain()` shows the chain in use, and
`i18n::available_locales()` lists the languages that have messages.

### Changing the language while the program runs

Views keep the text they were built with. A label does not look its text up
again when it draws. So changing the language is two steps: set the locale,
then build again the views that show translated text. Dialogs opened after
the change are in the new language automatically, because they are built
when they open.

```rust
fn handle_command(&mut self, app: &mut Application, command: CommandId, _: &Event) -> bool {
    let locale = match command {
        CM_LANG_EN => "en",
        CM_LANG_CA => "ca",
        _ => return false,
    };
    app.set_locale(locale);
    build_menu_bar_and_status_line(app); // the same function main calls
    true
}
```

Name each language in itself in the language menu: *English*, *Español*,
*Català*. That way a user who opened the wrong language by mistake can still
find their own. `examples/i18n.rs` (`cargo run --example i18n`) is a
complete program that does this. The gallery's *Languages* demo
(`cargo run --example gallery`) shows the built-in dialogs switching.

Windows that are already open keep their text. Either rebuild them too,
which is easiest with a function that makes the window from its data, or
tell the user that the change applies to new windows.

### Replacing a built-in message

Your messages are added after the crate's. A message with the same id
replaces the earlier one, in that language only:

```rust
use turbo_vision::i18n;

i18n::add_translations("en", "button-ok = ~A~ccept").unwrap();
assert_eq!(i18n::tr("button-ok"), "~A~ccept");
```

The ids of the built-in messages, with a comment for each group, are in
`locales/en/turbo-vision.ftl`.

## Hot keys in other languages

The `~` letter is part of the translation. The translator chooses it again
for each language, and the same rules hold as in English:

1. **One marked letter per text.** `~A~rxiu`, not `~A~r~x~iu`.
2. **Unique within a dialog or a menu.** In a Yes/No/Cancel box in Catalan,
   *Sí*, *No* and *Cancel·la* use `S`, `N` and `C`. If a translated label
   and a button both took `C`, the first one would win and the other could
   not be reached by keyboard.
3. **An ASCII letter, A to Z.** Alt plus a letter exists only for those
   letters. A hot key on `Á`, `Ç` or `Ñ` could still be typed, but not
   pressed with Alt from an input line. Mark a plain letter instead:
   `Á~r~bol`, not `~Á~rbol`.
4. **A letter the word contains.** The marker must sit on a letter of the
   text, so the user can see which key it is.

Global keys keep their meaning in every language: Alt+X, F1, F10 and
Alt+1…9. Do not translate the key names in `StatusItemBuilder::key` and
`MenuItemBuilder::key` (`"Alt+X"`, `"Ctrl+S"`). Those are parsed, not shown.
The text shown beside them (`~Alt-X~ Surt`) is translated.

## Making room for longer text

Translations are often longer than the English. *OK* is 2 columns, but
*Aceptar* and *D'acord* are 7. *Show hidden* (11) becomes *Mostra els
amagats* (18). German is often half as long again. In a terminal every
column is visible, so plan for it:

- **Size from the text, not from the English.** `button::width_for_title`
  gives the width a button needs for a title: its letters plus four columns
  for the margins and the shadow. A label needs its text's width plus one
  column. The built-in dialogs do this. In English they keep their old
  layout, and in other languages they grow.
- **Measure in columns, not bytes or chars.** `·`, `é` and `ñ` are one
  column each, while CJK characters are two. `unicode_width` (which the
  crate uses) gives the right number.
- **Leave slack where you can't grow.** For a fixed area, such as a column
  of buttons beside a list, decide how much room each text has and say so
  in a comment above the message in the English file.
- **Wrap long messages.** `message_box` wraps its text and grows its height.
  Keep sentences in messages, not in hand-placed `StaticText` lines.

```rust
use turbo_vision::views::button::width_for_title;

assert_eq!(width_for_title("~O~K", 10), 10); // short: the minimum
assert_eq!(width_for_title("~C~ancel·la", 10), 13); // 9 columns of text + 4
```

## Help files per language

`app.set_help_file("help/manual.md")` loads the version for the current
locale when it exists. With Catalan chosen, it looks for
`help/manual.ca-ES.md`, then `help/manual.ca.md`, then `help/manual.en.md`,
and falls back to `help/manual.md`. Keep the topic ids (`{#file-open}`) the
same in every language, so `register_help_context` works for all of them.
Translate only the titles and the text. `i18n::localized_path` does the
same lookup for any file of yours.

## Testing translations

The catalogue is per thread, and Rust runs each test on its own thread. So
a test can choose a language without disturbing the others:

```rust
use turbo_vision::i18n;

i18n::set_locale("ca");
assert_eq!(i18n::tr("button-cancel"), "~C~ancel·la");
i18n::set_locale("en"); // tidy, though the next test has a fresh thread
```

To check a screen in a language, build it after `set_locale`, run it on the
test terminal (`AGENTS.md`, "Test a screen without a terminal"), and read
the cells back.

Three checks catch most translation bugs. The crate runs them on its own
files in `src/i18n/mod.rs`, and they are worth copying for yours:

- **Completeness:** every language has exactly the ids of the English file,
  no fewer and no more.
- **Hot keys:** for each dialog or menu, the marked letters of the texts it
  shows are different, in every language, and each is A to Z.
- **Room:** each text shown in a fixed area fits it, in every language.

## Without the `i18n` feature

Fluent comes with the `i18n` cargo feature, which is on by default. A program
that wants no extra dependencies (an embedded or WASM build, say) can turn
it off:

```toml
turbo-vision = { version = "4", default-features = false, features = ["native"] }
```

Everything keeps working, with simple messages: each `id = text`, text
continued on indented lines, and `{ $name }` placeholders. Selectors
(plurals) and Fluent functions need the feature. Without it, a message that
uses them is not resolved, and part of its source text shows on screen.

## Adding a new language

A language has two sets of messages: Turbo Vision's own (the OK and Cancel
buttons, the message boxes, the file dialog and the rest) and your
application's. Your application's are always yours to write, as in
"Translating an application, step by step". For Turbo Vision's own there
are two ways, and you can do both:

- **Ship it in your application**, when you need the language now and the
  crate does not have it yet. Only your program gets it.
- **Add it to Turbo Vision**, so every program built on the crate gets it.
  It takes an hour or so: the crate has about forty short messages.

Until one of the two is done, a language the crate does not ship still
works. Your own messages are in it, and the built-in dialogs fall back to
English.

### In your application

Say the program needs German (`de`):

1. **Copy the crate's English file into your program**, next to your own
   messages: `locales/de/turbo-vision.ftl`. It is in the crate's source
   (`locales/en/turbo-vision.ftl`), and in the copy Cargo keeps under
   `~/.cargo/registry/src/`. Copy it from the version of the crate you use.
2. **Translate it**, following steps 2 to 4 of the next section: keep every
   id and placeholder, choose the hot keys again, and mind the room.
3. **Add it before you choose the locale**, like your own messages:

   ```rust
   use turbo_vision::app::Application;

   // In the program: include_str!("../locales/de/turbo-vision.ftl").
   const TURBO_VISION_DE: &str = "button-ok = ~O~K\nbutton-cancel = ~A~bbrechen";
   // And your own German messages.
   const APP_DE: &str = "menu-file = ~D~atei";

   fn main() -> turbo_vision::core::error::Result<()> {
       let mut app = Application::new()?;
       app.add_translations("de", TURBO_VISION_DE).expect("valid Fluent");
       app.add_translations("de", APP_DE).expect("valid Fluent");
       app.set_locale_from_env(); // LANG=de_DE.UTF-8 gives German
       // ... build the menus, the status line and the windows
       Ok(())
   }
   ```

4. **Add it to your language menu**, named in itself: *Deutsch*.

A later version of the crate may add messages. Those show in English in
your German until you translate them. The ids are in the new version's
English file. When the translation is done, consider sending it to the crate
as the next section describes. Then it is kept up to date with the crate.

### In Turbo Vision itself

Here is the whole process, for French (`fr`) as an example.

1. **Copy the English file.** It is the reference, with a comment above
   each group:

   ```text
   mkdir locales/fr
   cp locales/en/turbo-vision.ftl locales/fr/turbo-vision.ftl
   ```

2. **Translate the text after each `=`.** Keep every id, and keep each
   `{ $name }` placeholder (you can move it). Translate the comments at the
   top of the file, and keep the `##` section headings so the file reads
   like the others. For a regional variant, name the folder after its tag
   (`pt-BR`).

3. **Choose the hot keys again.** Keep one `~` letter per text, from A to Z,
   and different within each dialog. The groups of messages shown together
   are listed in `HOT_KEY_GROUPS` in `src/i18n/mod.rs`. For example, the file
   dialog shows *Name*, *Files*, *Show hidden*, *Open* and *Cancel*.

4. **Mind the room.** The texts with a fixed amount of space, and how many
   columns each may take, are listed in `FIXED_SLOTS` in the same file.
   Everything else is sized from its text, and grows: a longer *Cancel*
   makes its button wider.

5. **Register the language.** Add one line to `BUILT_IN` in
   `src/i18n/mod.rs`:

   ```rust
   ("fr", include_str!("../../locales/fr/turbo-vision.ftl")),
   ```

   That is the only code to change. The tests below find the new language
   through `BUILT_IN`.

6. **Run the tests.** `cargo test i18n` checks, for every language in
   `BUILT_IN`:

   - that the file parses, and has exactly the English ids;
   - that no dialog's hot keys clash, and that every hot key has an Alt key;
   - that the texts in fixed areas fit;
   - that every built-in dialog, built in that language, keeps its
     controls apart: no button grown for a long text covers a list, a field
     or the frame (`src/i18n/layout_check.rs`).

   A failure names the locale, the id or the dialog, and the problem:

   ```text
   fr: 'file-dialog-files' and 'file-dialog-name' both use Alt+n
   fr: 'help-index-search' is 11 > 7 (help_index.rs: Label (2, 4, 10, 5)): "Rechercher:"
   fr: file dialog: child 4 Rect { .. } overlaps child 5 Rect { .. }
   ```

   The layout check also runs in a made-up language whose every message is
   half as long again. The crate's dialogs pass it, so an overlap in yours
   means a text far longer than usual. Shorten it, or report the dialog.

7. **Look at it.** `LANG=fr_FR.UTF-8 cargo run --example i18n` opens the
   example in French. The menus stay in English, because the example has
   no French messages of its own, but every built-in dialog is in French.
   Open the message boxes, the Find and Go to line boxes and the file
   dialog, and check that nothing is cut off or crowded.

8. **Show it in the gallery.** Add a button named in the language itself
   (*Français*) to the Languages demo (`examples/gallery/demos/languages.rs`),
   with its two `demo-` messages, so the language can be tried there.

9. **Record it** in `CHANGELOG.md` under `[Unreleased]`, *Added*, and in its
   copy `website/docs/reference/changelog.md`. Add the language to the lists
   of shipped languages in this chapter (the first section and the summary)
   and in `AGENTS.md`, then run `python3 website/sync_docs.py`.

When the crate gains a new message, it is added in English and in every
shipped language in the same change. The completeness test fails until it
is. If you can't translate a message, copy the English text and mark it
with a `# TODO: translate` comment, so a speaker can find it.

## Summary

1. **Two owners:** Turbo Vision ships its own text (in `en`, `es`, `ca`), and
   the application adds its own. Both are Fluent messages looked up by id.
2. **Order matters:** add the translations, choose the locale, then build
   the views. Views keep their text, so changing the language means building
   the translated views again.
3. **Fallback:** `ca-ES` → `ca` → `en` → the id. A half-translated file still
   works.
4. **Grammar stays in the translation:** use placeholders and selectors, not
   `format!` on translated pieces.
5. **Hot keys are translated:** one letter A to Z per text, unique in each
   dialog or menu.
6. **Text grows:** size views from their text (`width_for_title`) and check
   the fixed areas.
7. **Test every language:** completeness, hot keys and room, as the crate
   does for its own files.
8. **A new language** goes into your application with `add_translations`
   and a translated copy of `locales/en/turbo-vision.ftl`, or into the crate
   with that file and one line in `BUILT_IN`; the tests then check it.

**Key Files to Explore**:
- `src/i18n/mod.rs` - The catalogue, `tr`, `tr!`, the locale chain, and the tests for the built-in languages
- `locales/en/turbo-vision.ftl` - The ids and English text of every built-in message
- `examples/i18n.rs` - An application in three languages that switches while it runs
- `examples/gallery/demos/languages.rs` - The gallery's Languages demo
- [Chapter 18 — Resources](chapter-18.md) - Keeping UI definitions apart from code
