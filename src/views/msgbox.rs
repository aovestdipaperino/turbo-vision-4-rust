// (C) 2025 - Enzo Lombardi

//! `MsgBox` - message box utilities for displaying alerts and confirmations.

use super::button::{Button, width_for_title};
use super::dialog::Dialog;
use super::input_line::InputLine;
use super::label::Label;
use super::static_text::StaticText;
use crate::app::Application;
use crate::core::command::{CM_CANCEL, CM_NO, CM_OK, CM_YES, CommandId};
use crate::core::geometry::Rect;
use crate::core::state::{deprecated_aliases, flags};
use crate::i18n::tr;
use crate::views::group::GroupLike;
use crate::views::handle::Handle;
use std::time::Duration;

flags! {
    /// Message-box options (Borland: `mfXxxx`): a type in the low four bits,
    /// button flags above, and behaviour flags.
    pub struct MsgBox: u16 {
        const WARNING = 0x0000;
        const ERROR = 0x0001;
        const INFORMATION = 0x0002;
        const CONFIRMATION = 0x0003;
        const ABOUT = 0x0004;
        const YES_BUTTON = 0x0100;
        const NO_BUTTON = 0x0200;
        const OK_BUTTON = 0x0400;
        const CANCEL_BUTTON = 0x0800;
        /// Close the message box on its own after
        /// [`MESSAGE_BOX_AUTO_DISMISS_TIMEOUT`] if the user has not dismissed
        /// it. The result is the default button's command (OK when present,
        /// otherwise the first button).
        const AUTO_DISMISS = 0x1000;
        const YES_NO_CANCEL = 0x0E00;
        const OK_CANCEL = 0x0C00;
    }
}

/// The type bits of a `MsgBox` (Borland: `mfWarning`..`mfAbout`).
const MSGBOX_TYPE_MASK: MsgBox = MsgBox::from_bits(0x000F);

deprecated_aliases! { MsgBox:
    MF_WARNING => WARNING, MF_ERROR => ERROR, MF_INFORMATION => INFORMATION,
    MF_CONFIRMATION => CONFIRMATION, MF_ABOUT => ABOUT, MF_YES_BUTTON => YES_BUTTON,
    MF_NO_BUTTON => NO_BUTTON, MF_OK_BUTTON => OK_BUTTON, MF_CANCEL_BUTTON => CANCEL_BUTTON,
    MF_AUTO_DISMISS => AUTO_DISMISS, MF_YES_NO_CANCEL => YES_NO_CANCEL, MF_OK_CANCEL => OK_CANCEL,
}

/// How long an `MsgBox::AUTO_DISMISS` message box stays open before closing itself.
pub const MESSAGE_BOX_AUTO_DISMISS_TIMEOUT: Duration = Duration::from_secs(3);

/// Display a message box with the given message and options.
///
/// Long messages are word-wrapped to fit the dialog's text area —
/// `StaticText` itself only honours explicit `\n`, so without this the
/// tail of a long line would be clipped right at the frame. The
/// dialog stays at a fixed 60-column max width; long messages grow
/// the dialog vertically instead.
pub fn message_box(app: &mut Application, message: &str, options: MsgBox) -> CommandId {
    let (bounds, wrapped) = message_box_bounds(app.terminal.size(), message, options);
    message_box_rect(app, bounds, &wrapped, options)
}

/// Where [`message_box`] puts its box on a `screen` of that size, and the
/// message wrapped to fit it.
fn message_box_bounds(screen: (i16, i16), message: &str, options: MsgBox) -> (Rect, String) {
    let (screen_w, screen_h) = screen;

    // Fixed dialog width — keep modal dialogs neat and consistent.
    let target_w = 60usize;
    // Inner text area: x=3, x_end=W-2 in message_box_rect → 5 chars
    // of frame/margins. Leave one extra column of breathing room so a
    // wrapped line never sits flush against the right frame.
    let inner_w = target_w.saturating_sub(6).max(10);

    let wrapped = wrap_message(message, inner_w);

    let msg_width = wrapped
        .lines()
        .map(|l| l.chars().count())
        .max()
        .unwrap_or(20);
    let msg_height = wrapped.lines().count().max(1);

    // Wide enough for the buttons too, with their translated labels.
    let width = (msg_width + 6)
        .min(target_w)
        .max(30)
        .max(buttons_width(&message_box_buttons(options)) + 4);
    // Frame, one row of padding above and below the text, and the
    // button row only when there is at least one button to show.
    let chrome_rows = if has_buttons(options) { 6 } else { 4 };
    let min_height = chrome_rows + 1;
    let max_height = (screen_h as usize).saturating_sub(2).max(min_height);
    let height = (msg_height + chrome_rows).min(max_height).max(min_height);

    let x = (screen_w - width as i16) / 2;
    let y = (screen_h - height as i16) / 2;

    let bounds = Rect::new(x, y, x + width as i16, y + height as i16);
    (bounds, wrapped)
}

/// Word-wrap `message` so every line is at most `max_width` characters
/// wide. Existing newlines act as hard breaks; words longer than
/// `max_width` (e.g. file paths) are split character-wise rather than
/// dropped, so the user always sees the full content.
#[allow(
    unused_assignments,
    reason = "line_len / first_word_in_line are dead on the last iteration of the inner loop but used on every other"
)]
fn wrap_message(message: &str, max_width: usize) -> String {
    let max_width = max_width.max(1);
    let mut out = String::new();

    for (i, paragraph) in message.split('\n').enumerate() {
        if i > 0 {
            out.push('\n');
        }
        if paragraph.chars().count() <= max_width {
            out.push_str(paragraph);
            continue;
        }

        let mut line_len = 0usize;
        let mut first_word_in_line = true;

        for word in paragraph.split_whitespace() {
            let word_chars: Vec<char> = word.chars().collect();
            let word_len = word_chars.len();

            // Words longer than the wrap width can't fit on a single
            // line — break them forcibly so we never lose content.
            if word_len > max_width {
                if !first_word_in_line {
                    out.push('\n');
                    line_len = 0;
                    first_word_in_line = true;
                }
                let mut start = 0;
                while start < word_len {
                    let end = (start + max_width).min(word_len);
                    if start > 0 {
                        out.push('\n');
                    }
                    for ch in &word_chars[start..end] {
                        out.push(*ch);
                    }
                    start = end;
                }
                line_len = (word_len % max_width).max(if word_len.is_multiple_of(max_width) {
                    max_width
                } else {
                    0
                });
                first_word_in_line = false;
                continue;
            }

            let needed = if first_word_in_line {
                word_len
            } else {
                line_len + 1 + word_len
            };
            if needed > max_width {
                out.push('\n');
                out.push_str(word);
                line_len = word_len;
            } else {
                if !first_word_in_line {
                    out.push(' ');
                    line_len += 1;
                }
                out.push_str(word);
                line_len += word_len;
            }
            first_word_in_line = false;
        }
    }
    out
}

/// Display a message box at a specific location
pub fn message_box_rect(
    app: &mut Application,
    bounds: Rect,
    message: &str,
    options: MsgBox,
) -> CommandId {
    message_box_dialog(bounds, message, options).execute(app)
}

/// The message box [`message_box_rect`] shows, ready to execute.
fn message_box_dialog(bounds: Rect, message: &str, options: MsgBox) -> Dialog {
    // Determine title based on message type
    let title = match options & MSGBOX_TYPE_MASK {
        MsgBox::WARNING => format!("\u{26A0} {}", tr("msgbox-warning")),
        MsgBox::ERROR => format!("\u{274C} {}", tr("msgbox-error")),
        MsgBox::INFORMATION => format!("\u{2139}\u{FE0F} {}", tr("msgbox-information")),
        MsgBox::CONFIRMATION => format!("\u{2753} {}", tr("msgbox-confirm")),
        MsgBox::ABOUT => format!("\u{2139}\u{FE0F} {}", tr("msgbox-about")),
        _ => tr("msgbox-message"),
    };

    let mut dialog = Dialog::new(bounds, &title);

    // Add static text with message (one row higher). Left-align so
    // wrapped lines hang from a consistent left margin instead of
    // each being independently centered (which makes a wrapped
    // diagnostic look like staggered poetry).
    // Without buttons the text area runs down to the bottom frame.
    let text_bottom = if has_buttons(options) {
        bounds.height() - 4
    } else {
        bounds.height() - 2
    };
    let text_bounds = Rect::new(3, 1, bounds.width() - 2, text_bottom);
    dialog.add(StaticText::new(text_bounds, message));

    let buttons = message_box_buttons(options);

    // Calculate button positions (one row higher)
    let button_y = bounds.height() - 4;
    let mut x = (bounds.width_clamped() as usize).saturating_sub(buttons_width(&buttons)) / 2;

    // Add buttons
    let is_default = buttons.len() == 1 || options.contains(MsgBox::OK_BUTTON);
    for (i, (label, cmd)) in buttons.iter().enumerate() {
        let button_width = label.chars().count() as i16;
        let button_bounds = Rect::new(x as i16, button_y, x as i16 + button_width, button_y + 2);
        let is_this_default = is_default && (i == 0 || *cmd == CM_OK);
        dialog.add(Button::new(button_bounds, label, *cmd, is_this_default));
        x += button_width as usize + 2;
    }

    if options.contains(MsgBox::AUTO_DISMISS) {
        dialog.set_auto_dismiss(
            MESSAGE_BOX_AUTO_DISMISS_TIMEOUT,
            auto_dismiss_command(&buttons.iter().map(|(_, cmd)| *cmd).collect::<Vec<_>>()),
        );
    }

    dialog.set_initial_focus();
    dialog
}

/// The buttons `options` asks for, as (label, command), in display order.
/// A label is its translated text after one space, and the button is as
/// wide as the label's characters, `~` included: one column each side.
fn message_box_buttons(options: MsgBox) -> Vec<(String, CommandId)> {
    [
        (MsgBox::YES_BUTTON, "button-yes", CM_YES),
        (MsgBox::NO_BUTTON, "button-no", CM_NO),
        (MsgBox::OK_BUTTON, "button-ok", CM_OK),
        (MsgBox::CANCEL_BUTTON, "button-cancel", CM_CANCEL),
    ]
    .into_iter()
    .filter(|(flag, _, _)| options.contains(*flag))
    .map(|(_, id, cmd)| (format!(" {}", tr(id)), cmd))
    .collect()
}

/// The width of a row of message box buttons, with their gaps.
fn buttons_width(buttons: &[(String, CommandId)]) -> usize {
    buttons
        .iter()
        .map(|(label, _)| label.chars().count() + 2)
        .sum()
}

/// Place `buttons` (title, command, default) on row `y`, each at least 10
/// wide and wider for a long translation, with two columns between them.
/// In English they start at `x`; wider buttons spread to both sides of
/// the same centre.
fn add_button_row(dialog: &mut Dialog, x: i16, y: i16, buttons: &[(String, CommandId, bool)]) {
    const LEGACY_WIDTH: i16 = 10;
    let widths: Vec<i16> = buttons
        .iter()
        .map(|(title, _, _)| width_for_title(title, LEGACY_WIDTH))
        .collect();
    let extra: i16 = widths.iter().map(|w| w - LEGACY_WIDTH).sum();
    let mut x = (x - extra / 2).max(1);
    for ((title, command, default), width) in buttons.iter().zip(widths) {
        dialog.add(Button::new(
            Rect::new(x, y, x + width, y + 2),
            title,
            *command,
            *default,
        ));
        x += width + 2;
    }
}

/// The OK and Cancel buttons of the input dialogs.
fn ok_cancel() -> [(String, CommandId, bool); 2] {
    [
        (format!(" {}", tr("button-ok")), CM_OK, true),
        (format!(" {}", tr("button-cancel")), CM_CANCEL, false),
    ]
}

/// A label's bounds at (`x`, `y`): at least `min` wide, wider for a long
/// translation.
fn label_bounds(text: &str, x: i16, y: i16, min: i16) -> Rect {
    // width_for_title adds 4 for a button's margins; a label needs one.
    let width = width_for_title(text, min + 3) - 3;
    Rect::new(x, y, x + width, y + 1)
}

/// Whether `options` asks for at least one button.
fn has_buttons(options: MsgBox) -> bool {
    options.intersects(MsgBox::YES_NO_CANCEL | MsgBox::OK_BUTTON)
}

/// The command an auto-dismissed message box reports: OK when present,
/// otherwise the first button, matching the default-button choice above.
fn auto_dismiss_command(commands: &[CommandId]) -> CommandId {
    if commands.contains(&CM_OK) {
        CM_OK
    } else {
        commands.first().copied().unwrap_or(CM_CANCEL)
    }
}

/// Display a simple message box with OK button
///
/// Returns `CM_OK` when dismissed.
///
/// # Example
/// ```ignore
/// use turbo_vision::views::msgbox::message_box_ok;
///
/// message_box_ok(&mut app, "File saved successfully!");
/// ```
pub fn message_box_ok(app: &mut Application, message: &str) -> CommandId {
    message_box(app, message, MsgBox::INFORMATION | MsgBox::OK_BUTTON)
}

/// Display an error message box with OK button
///
/// Returns `CM_OK` when dismissed.
///
/// # Example
/// ```ignore
/// use turbo_vision::views::msgbox::message_box_error;
///
/// message_box_error(&mut app, "Failed to open file");
/// ```
pub fn message_box_error(app: &mut Application, message: &str) -> CommandId {
    message_box(app, message, MsgBox::ERROR | MsgBox::OK_BUTTON)
}

/// Display a warning message box with OK button
///
/// Returns `CM_OK` when dismissed.
pub fn message_box_warning(app: &mut Application, message: &str) -> CommandId {
    message_box(app, message, MsgBox::WARNING | MsgBox::OK_BUTTON)
}

/// Display a confirmation dialog with Yes/No/Cancel buttons
///
/// Returns `CM_YES`, `CM_NO`, or `CM_CANCEL` based on user choice.
///
/// # Example
/// ```ignore
/// use turbo_vision::views::msgbox::{confirmation_box, CM_YES, CM_NO};
///
/// match confirmation_box(&mut app, "Save changes?") {
///     result if result == CM_YES => { /* save */ },
///     result if result == CM_NO => { /* don't save */ },
///     _ => { /* cancel */ },
/// }
/// ```
pub fn confirmation_box(app: &mut Application, message: &str) -> CommandId {
    message_box(app, message, MsgBox::CONFIRMATION | MsgBox::YES_NO_CANCEL)
}

/// Display a confirmation dialog with Yes/No buttons
///
/// Returns `CM_YES` or `CM_NO` based on user choice.
pub fn confirmation_box_yes_no(app: &mut Application, message: &str) -> CommandId {
    message_box(
        app,
        message,
        MsgBox::CONFIRMATION | MsgBox::YES_BUTTON | MsgBox::NO_BUTTON,
    )
}

/// Display a confirmation dialog with OK/Cancel buttons
///
/// Returns `CM_OK` or `CM_CANCEL` based on user choice.
pub fn confirmation_box_ok_cancel(app: &mut Application, message: &str) -> CommandId {
    message_box(app, message, MsgBox::CONFIRMATION | MsgBox::OK_CANCEL)
}

/// Display an input box that prompts the user for a string
pub fn input_box(
    app: &mut Application,
    title: &str,
    label: &str,
    initial: &str,
    max_length: usize,
) -> Option<String> {
    let (width, height) = input_box_size(label, max_length);
    let bounds = centered(app, width, height);
    input_box_rect(app, bounds, title, label, initial, max_length)
}

/// Display an input box at a specific location
pub fn input_box_rect(
    app: &mut Application,
    bounds: Rect,
    title: &str,
    label: &str,
    initial: &str,
    max_length: usize,
) -> Option<String> {
    let (mut dialog, input) = input_dialog(bounds, title, label, initial, max_length);
    if dialog.execute(app) == CM_OK {
        dialog.get(input).map(|f| f.text().to_string())
    } else {
        None
    }
}

/// The dialog of [`input_box_rect`] and its input line.
fn input_dialog(
    bounds: Rect,
    title: &str,
    label: &str,
    initial: &str,
    max_length: usize,
) -> (Dialog, Handle<InputLine>) {
    let mut dialog = Dialog::new(bounds, title);

    // Add label
    let label_x = 2;
    let label_width = label.chars().count() as i16;
    let label_bounds = Rect::new(label_x, 2, label_x + label_width, 3);
    dialog.add(Label::new(label_bounds, label));

    // Add input line
    let input_x = label_x + label_width + 1;
    let input_width = (bounds.width() - input_x - 3).min(max_length as i16 + 2);
    let input_bounds = Rect::new(input_x, 2, input_x + input_width, 3);
    let mut input = InputLine::new(input_bounds, max_length);
    input.set_text(initial);
    let input = dialog.add_typed(input);

    // OK and Cancel
    add_button_row(
        &mut dialog,
        bounds.width() / 2 - 11,
        bounds.height() - 4,
        &ok_cancel(),
    );

    dialog.set_initial_focus();
    (dialog, input)
}

/// The size of [`input_box`]: room for the label and the text, 30 to 60
/// columns wide.
fn input_box_size(label: &str, max_length: usize) -> (i16, i16) {
    let width = (label.chars().count() + max_length + 12).clamp(30, 60);
    (i16::try_from(width).unwrap_or(60), 8)
}

/// The size of [`search_box`].
const SEARCH_BOX_SIZE: (i16, i16) = (50, 9);
/// The size of [`search_replace_box`].
const REPLACE_BOX_SIZE: (i16, i16) = (50, 13);
/// The size of [`goto_line_box`].
const GOTO_LINE_BOX_SIZE: (i16, i16) = (40, 8);

/// `width` by `height` bounds in the middle of the screen.
fn centered(app: &Application, width: i16, height: i16) -> Rect {
    let (screen_w, screen_h) = app.terminal.size();
    let x = (screen_w - width) / 2;
    let y = (screen_h - height) / 2;
    Rect::new(x, y, x + width, y + height)
}

/// Display a search dialog that prompts the user for search text
///
/// Returns `Some(search_text)` if OK was pressed, None if cancelled
///
/// # Example
/// ```ignore
/// use turbo_vision::views::msgbox::search_box;
///
/// if let Some(text) = search_box(&mut app, "Search") {
///     // Perform search with text
/// }
/// ```
pub fn search_box(app: &mut Application, title: &str) -> Option<String> {
    let (mut dialog, input) =
        search_dialog(centered(app, SEARCH_BOX_SIZE.0, SEARCH_BOX_SIZE.1), title);
    if dialog.execute(app) == CM_OK {
        let text = dialog
            .get(input)
            .map(|f| f.text().to_string())
            .unwrap_or_default();
        (!text.is_empty()).then_some(text)
    } else {
        None
    }
}

/// The dialog of [`search_box`] and its input line.
fn search_dialog(bounds: Rect, title: &str) -> (Dialog, Handle<InputLine>) {
    let width = bounds.width();
    let mut dialog = Dialog::new(bounds, title);

    // Add label
    let find = tr("find-label");
    dialog.add(Label::new(label_bounds(&find, 2, 2, 18), &find));

    // Add input line
    let input_bounds = Rect::new(2, 3, width - 4, 4);
    let input = dialog.add_typed(InputLine::new(input_bounds, 100));

    // OK and Cancel
    add_button_row(&mut dialog, 15, 5, &ok_cancel());

    dialog.set_initial_focus();
    (dialog, input)
}

/// Display a search and replace dialog that prompts for find and replace text
///
/// Returns `Some((find_text`, `replace_text`)) if OK was pressed, None if cancelled
///
/// # Example
/// ```ignore
/// use turbo_vision::views::msgbox::search_replace_box;
///
/// if let Some((find, replace)) = search_replace_box(&mut app, "Replace") {
///     // Perform search and replace
/// }
/// ```
pub fn search_replace_box(app: &mut Application, title: &str) -> Option<(String, String)> {
    let (mut dialog, find_input, replace_input) =
        search_replace_dialog(centered(app, REPLACE_BOX_SIZE.0, REPLACE_BOX_SIZE.1), title);
    if dialog.execute(app) == CM_OK {
        let text_of = |h| {
            dialog
                .get(h)
                .map(|f: &InputLine| f.text().to_string())
                .unwrap_or_default()
        };
        let find_text = text_of(find_input);
        (!find_text.is_empty()).then(|| {
            let replace_text = text_of(replace_input);
            (find_text, replace_text)
        })
    } else {
        None
    }
}

/// The dialog of [`search_replace_box`] and its find and replace lines.
fn search_replace_dialog(
    bounds: Rect,
    title: &str,
) -> (Dialog, Handle<InputLine>, Handle<InputLine>) {
    let width = bounds.width();
    let mut dialog = Dialog::new(bounds, title);

    // Add find label
    let find = tr("find-label");
    dialog.add(Label::new(label_bounds(&find, 2, 2, 18), &find));

    // Add find input line
    let input1_bounds = Rect::new(2, 3, width - 4, 4);
    let find_input = dialog.add_typed(InputLine::new(input1_bounds, 100));

    // Add replace label
    let replace = tr("replace-label");
    dialog.add(Label::new(label_bounds(&replace, 2, 5, 18), &replace));

    // Add replace input line
    let input2_bounds = Rect::new(2, 6, width - 4, 7);
    let replace_input = dialog.add_typed(InputLine::new(input2_bounds, 100));

    // OK and Cancel
    add_button_row(&mut dialog, 15, 9, &ok_cancel());

    dialog.set_initial_focus();
    (dialog, find_input, replace_input)
}

/// Display a goto line dialog that prompts for a line number
///
/// Returns `Some(line_number)` if OK was pressed, None if cancelled or invalid
///
/// # Example
/// ```ignore
/// use turbo_vision::views::msgbox::goto_line_box;
///
/// if let Some(line) = goto_line_box(&mut app, "Go to Line") {
///     // Jump to line number
/// }
/// ```
pub fn goto_line_box(app: &mut Application, title: &str) -> Option<usize> {
    let (width, height) = GOTO_LINE_BOX_SIZE;
    let (mut dialog, input) = goto_line_dialog(centered(app, width, height), title);
    if dialog.execute(app) == CM_OK {
        dialog
            .get(input)
            .and_then(|f| f.text().parse::<usize>().ok())
    } else {
        None
    }
}

/// The dialog of [`goto_line_box`] and its input line.
fn goto_line_dialog(bounds: Rect, title: &str) -> (Dialog, Handle<InputLine>) {
    let width = bounds.width();
    let mut dialog = Dialog::new(bounds, title);

    // Add label
    let line = format!(" {}", tr("goto-line-label"));
    dialog.add(Label::new(label_bounds(&line, 2, 2, 18), &line));

    // Add input line
    let input_bounds = Rect::new(2, 3, width - 4, 4);
    let input = dialog.add_typed(InputLine::new(input_bounds, 10));

    // OK and Cancel
    add_button_row(&mut dialog, 10, 5, &ok_cancel());

    dialog.set_initial_focus();
    (dialog, input)
}

/// The built-in dialogs, built but not shown, for the layout tests.
#[cfg(test)]
pub(crate) fn built_message_box(screen: (i16, i16), message: &str, options: MsgBox) -> Dialog {
    let (bounds, wrapped) = message_box_bounds(screen, message, options);
    message_box_dialog(bounds, &wrapped, options)
}

#[cfg(test)]
pub(crate) fn built_input_box(label: &str, max: usize) -> Dialog {
    let (width, height) = input_box_size(label, max);
    input_dialog(Rect::new(0, 0, width, height), "Input", label, "", max).0
}

#[cfg(test)]
pub(crate) fn built_search_box() -> Dialog {
    let (width, height) = SEARCH_BOX_SIZE;
    search_dialog(Rect::new(0, 0, width, height), "Find").0
}

#[cfg(test)]
pub(crate) fn built_replace_box() -> Dialog {
    let (width, height) = REPLACE_BOX_SIZE;
    search_replace_dialog(Rect::new(0, 0, width, height), "Replace").0
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::event::{Event, KB_ESC};
    use crate::views::View;

    /// The text of screen row `y`.
    fn screen_row(app: &Application, y: i16) -> String {
        let (width, _) = app.terminal.size();
        (0..width)
            .filter_map(|x| app.terminal.read_cell(x, y))
            .map(|cell| cell.ch)
            .collect()
    }

    fn screen(app: &Application) -> Vec<String> {
        let (_, height) = app.terminal.size();
        (0..height).map(|y| screen_row(app, y)).collect()
    }

    fn test_app() -> Application {
        Application::with_terminal(crate::test_util::test_terminal(80, 25))
    }

    #[test]
    fn a_catalan_confirmation_shows_catalan_buttons_in_order() {
        crate::i18n::set_locale("ca");
        let mut app = test_app();
        app.terminal
            .event_injector()
            .send(Event::keyboard(KB_ESC))
            .unwrap();
        // The three flags, not YES_NO_CANCEL: that constant is 0x0E00
        // (No, OK, Cancel) where Borland's mfYesNoCancel is 0x0B00.
        let yes_no_cancel = MsgBox::YES_BUTTON | MsgBox::NO_BUTTON | MsgBox::CANCEL_BUTTON;
        let options = MsgBox::CONFIRMATION | yes_no_cancel;
        assert_eq!(message_box(&mut app, "Voleu desar?", options), CM_CANCEL);
        let rows = screen(&app);
        assert!(rows.iter().any(|r| r.contains("Confirmació")), "{rows:#?}");
        let buttons = rows
            .iter()
            .find(|r| r.contains("Sí"))
            .unwrap_or_else(|| panic!("a row of buttons: {rows:#?}"));
        let (si, no, cancel) = (
            buttons.find("Sí").unwrap(),
            buttons.find(" No ").unwrap(),
            buttons.find("Cancel·la").unwrap(),
        );
        assert!(si < no && no < cancel, "in order, apart: {buttons:?}");
        crate::i18n::set_locale("en");
    }

    #[test]
    fn a_spanish_search_box_fits_its_longer_labels() {
        crate::i18n::set_locale("es");
        let mut app = test_app();
        app.terminal
            .event_injector()
            .send(Event::keyboard(KB_ESC))
            .unwrap();
        assert_eq!(search_replace_box(&mut app, "Reemplazar"), None);
        let rows = screen(&app);
        assert!(
            rows.iter().any(|r| r.contains("Reemplazar por:")),
            "{rows:#?}"
        );
        let buttons = rows
            .iter()
            .find(|r| r.contains("Aceptar"))
            .expect("buttons");
        assert!(buttons.contains("Cancelar"), "{buttons:?}");
        crate::i18n::set_locale("en");
    }

    #[test]
    fn button_rows_keep_the_english_layout_and_grow_for_translations() {
        let mut dialog = Dialog::new(Rect::new(0, 0, 50, 9), "t");
        let rects = |dialog: &Dialog| -> Vec<Rect> {
            let group = dialog.group();
            (0..group.len())
                .map(|i| group.child_at(i).bounds())
                .collect()
        };
        add_button_row(&mut dialog, 15, 5, &ok_cancel());
        let english = rects(&dialog);
        let origin = dialog.bounds().a;
        let rel = |r: &Rect| (r.a.x - origin.x, r.b.x - origin.x);
        assert_eq!(
            english.iter().map(rel).collect::<Vec<_>>(),
            [(15, 25), (27, 37)]
        );

        let mut dialog = Dialog::new(Rect::new(0, 0, 50, 9), "t");
        let long = [
            (" ~V~ery long OK text".to_string(), CM_OK, true),
            (" ~C~ancel".to_string(), CM_CANCEL, false),
        ];
        add_button_row(&mut dialog, 15, 5, &long);
        let grown: Vec<_> = rects(&dialog).iter().map(rel).collect();
        assert_eq!(
            grown[0].1 - grown[0].0,
            21,
            "17 columns of text, 4 of margin and shadow"
        );
        assert_eq!(grown[1].0, grown[0].1 + 2, "two columns apart");
    }

    #[test]
    fn message_boxes_widen_for_their_buttons() {
        let options =
            MsgBox::CONFIRMATION | MsgBox::YES_BUTTON | MsgBox::NO_BUTTON | MsgBox::CANCEL_BUTTON;
        crate::i18n::set_locale("en");
        let english = buttons_width(&message_box_buttons(options));
        crate::i18n::set_locale("ca");
        let catalan = buttons_width(&message_box_buttons(options));
        crate::i18n::set_locale("en");
        assert!(catalan > english);
        assert_eq!(
            message_box_buttons(MsgBox::OK_CANCEL),
            vec![
                (" ~O~K".to_string(), CM_OK),
                (" ~C~ancel".to_string(), CM_CANCEL)
            ]
        );
    }

    #[test]
    fn auto_dismiss_prefers_ok_then_first_button() {
        assert_eq!(auto_dismiss_command(&[CM_YES, CM_NO, CM_OK]), CM_OK);
        assert_eq!(auto_dismiss_command(&[CM_YES, CM_NO]), CM_YES);
        assert_eq!(auto_dismiss_command(&[]), CM_CANCEL);
    }

    #[test]
    fn has_buttons_reflects_button_flags_only() {
        assert!(has_buttons(MsgBox::INFORMATION | MsgBox::OK_BUTTON));
        assert!(has_buttons(MsgBox::CONFIRMATION | MsgBox::YES_NO_CANCEL));
        assert!(!has_buttons(MsgBox::INFORMATION | MsgBox::AUTO_DISMISS));
    }

    #[test]
    fn auto_dismiss_flag_does_not_overlap_type_or_button_bits() {
        assert!((MsgBox::AUTO_DISMISS & MSGBOX_TYPE_MASK).is_empty());
        assert!(!MsgBox::AUTO_DISMISS.intersects(MsgBox::YES_NO_CANCEL | MsgBox::OK_BUTTON));
    }

    #[test]
    fn wraps_long_lines_at_word_boundaries() {
        let out = wrap_message(
            "Parse error: line 1:1: expected 'program', found identifier 'hello'",
            30,
        );
        for line in out.lines() {
            assert!(line.chars().count() <= 30, "line too long: {line:?}");
        }
        // Round-tripping by re-joining whitespace must give the original
        // message back — wrapping must not lose or reorder words.
        let original_words: Vec<&str> =
            "Parse error: line 1:1: expected 'program', found identifier 'hello'"
                .split_whitespace()
                .collect();
        let wrapped_words: Vec<&str> = out.split_whitespace().collect();
        assert_eq!(original_words, wrapped_words);
    }

    #[test]
    fn preserves_existing_newlines() {
        let out = wrap_message("first paragraph\nsecond paragraph", 40);
        assert_eq!(out, "first paragraph\nsecond paragraph");
    }

    #[test]
    fn breaks_overlong_words_character_wise() {
        let path = "averylongfilenameWithoutSpaces.txt";
        let out = wrap_message(path, 10);
        for line in out.lines() {
            assert!(line.chars().count() <= 10);
        }
        let recombined: String = out.lines().collect();
        assert_eq!(recombined, path);
    }
}
