// (C) 2025 - Enzo Lombardi

//! The built-in dialogs keep their controls apart in every language.
//!
//! A translated button or label grows to show its text. These tests build
//! each dialog of the crate in every built-in locale (every entry of
//! `BUILT_IN`), and in a made-up
//! locale whose every message is half as long again (the room German or
//! Catalan text can take), and check that no control overlaps another or
//! the frame. A text too long for its place is cut; it never covers a
//! neighbour.

use crate::core::geometry::Rect;
use crate::core::palette::Attr;
use crate::views::GroupLike;
use crate::views::chdir_dialog::ChDirDialog;
use crate::views::color_dialog::ColorDialog;
use crate::views::file_dialog::FileDialog;
use crate::views::help_file::HelpFile;
use crate::views::help_index::HelpIndex;
use crate::views::help_toc::HelpToc;
use crate::views::msgbox;
use std::cell::RefCell;
use std::io::Write;
use std::rc::Rc;

/// The made-up locale of long messages.
const LONG: &str = "qps";

/// Every built-in English message, each line half as long again (at least
/// three columns more), as a Fluent resource for [`LONG`].
fn long_messages() -> String {
    use unicode_width::UnicodeWidthStr;
    let english = super::built_in_source("en").expect("English messages");
    let mut ftl = String::new();
    for (id, text) in super::parse_plain(english).expect("plain Fluent") {
        ftl.push_str(&id);
        ftl.push_str(" =");
        for line in text.lines() {
            let extra = (line.replace('~', "").width() / 2).max(3);
            ftl.push_str("\n    ");
            ftl.push_str(line);
            ftl.push_str(&"+".repeat(extra));
        }
        ftl.push('\n');
    }
    ftl
}

/// What is wrong with the layout of `dialog`: children outside the area
/// inside its frame, and children that overlap.
fn layout_problems<G: GroupLike + ?Sized>(dialog: &G) -> Vec<String> {
    // Children are placed relative to the area inside the frame.
    let outer = dialog.bounds();
    let inside = Rect::new(0, 0, outer.width() - 2, outer.height() - 2);
    let children: Vec<(String, Rect)> = (0..dialog.child_count())
        .map(|i| (format!("child {i}"), dialog.child_at(i).bounds()))
        .filter(|(_, r)| !r.is_empty())
        .collect();
    let mut problems = Vec::new();
    for (i, (name, r)) in children.iter().enumerate() {
        if r.intersect(&inside) != *r {
            problems.push(format!("{name} {r:?} leaves the frame {inside:?}"));
        }
        for (other, o) in &children[i + 1..] {
            if r.intersects(o) {
                problems.push(format!("{name} {r:?} overlaps {other} {o:?}"));
            }
        }
    }
    problems
}

/// A help file of one topic, for the help dialogs.
fn help_file() -> (tempfile::NamedTempFile, Rc<RefCell<HelpFile>>) {
    let mut file = tempfile::NamedTempFile::new().unwrap();
    writeln!(file, "# Topic {{#topic}}\n\nText.").unwrap();
    file.flush().unwrap();
    let help = HelpFile::new(file.path()).unwrap();
    (file, Rc::new(RefCell::new(help)))
}

/// The layout problems of every built-in dialog, in the current locale.
fn problems_of_every_dialog() -> Vec<String> {
    let mut found = Vec::new();
    let mut check = |what: &str, problems: Vec<String>| {
        found.extend(problems.into_iter().map(|p| format!("{what}: {p}")));
    };

    let screen = (80, 25);
    for (name, options) in [
        (
            "message box OK",
            msgbox::MsgBox::INFORMATION | msgbox::MsgBox::OK_BUTTON,
        ),
        (
            "message box OK/Cancel",
            msgbox::MsgBox::CONFIRMATION | msgbox::MsgBox::OK_CANCEL,
        ),
        (
            "message box Yes/No",
            msgbox::MsgBox::CONFIRMATION | msgbox::MsgBox::YES_BUTTON | msgbox::MsgBox::NO_BUTTON,
        ),
        (
            "message box Yes/No/Cancel",
            msgbox::MsgBox::CONFIRMATION | msgbox::MsgBox::YES_NO_CANCEL,
        ),
    ] {
        for message in ["Saved.", &"A long message that wraps. ".repeat(6)] {
            check(
                name,
                layout_problems(&msgbox::built_message_box(screen, message, options)),
            );
        }
    }
    let at = |width, height| Rect::new(0, 0, width, height);
    for (label, max) in [("~N~ew name", 40), ("~V~alue", 5)] {
        check(
            "input box",
            layout_problems(&msgbox::built_input_box(label, max)),
        );
    }
    check("search box", layout_problems(&msgbox::built_search_box()));
    check("replace box", layout_problems(&msgbox::built_replace_box()));
    // The Go to line box is checked once fix/goto-line-box-height, which
    // makes it tall enough for its buttons, is merged.

    // The gallery's colour dialog.
    check(
        "colour dialog",
        layout_problems(&ColorDialog::new(
            at(52, 17),
            "Colours",
            Attr::from_u8(0x1F),
        )),
    );
    check("change directory", layout_problems(&ChDirDialog::new(None)));
    for (title, hidden) in [("Open", false), ("Open", true)] {
        let dialog = FileDialog::new(at(60, 20), title, "*", None)
            .with_hidden_toggle(hidden)
            .build();
        check("file dialog", layout_problems(&dialog));
    }
    let (_file, help) = help_file();
    check(
        "help index",
        layout_problems(&HelpIndex::new(at(50, 18), "Index", Rc::clone(&help))),
    );
    check(
        "help contents",
        layout_problems(&HelpToc::new(at(50, 18), "Contents", help)),
    );
    found
}

#[test]
fn built_in_dialogs_keep_their_controls_apart_in_every_locale() {
    super::add_translations(LONG, &long_messages()).unwrap();
    let mut found = Vec::new();
    // Every shipped language, so a new one is checked as soon as it is in
    // BUILT_IN, and the made-up long one.
    for locale in super::built_in_locales().chain([LONG]) {
        super::set_locale(locale);
        found.extend(
            problems_of_every_dialog()
                .into_iter()
                .map(|p| format!("{locale}: {p}")),
        );
    }
    super::set_locale("en");
    assert!(found.is_empty(), "\n{}", found.join("\n"));
}
