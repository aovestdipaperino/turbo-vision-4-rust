# The strings Turbo Vision itself shows: buttons, message boxes and the
# built-in dialogs. This English file is the reference: every other locale
# defines the same ids (the tests in src/i18n/mod.rs check it).
#
# A `~` pair marks the hot key: `~O~K` is reached with Alt+O. Keep one per
# string, and keep the letters of one dialog different from each other
# (the groups are listed in those tests).

## Buttons shared by every dialog

button-ok = ~O~K
button-cancel = ~C~ancel
button-yes = ~Y~es
button-no = ~N~o

## Message box titles (an icon is put in front of them)

msgbox-warning = Warning
msgbox-error = Error
msgbox-information = Information
msgbox-confirm = Confirm
msgbox-about = About
msgbox-message = Message

## Find, Replace and Go to line

find-label = ~F~ind:
replace-label = ~R~eplace with:
goto-line-label = ~L~ine number:

## File dialog

file-dialog-name = ~N~ame:
file-dialog-files = ~F~iles:
file-dialog-show-hidden = Show ~h~idden
file-dialog-open = ~O~pen
file-dialog-save = ~S~ave
file-dialog-save-as-title = Save As

## Change directory dialog

chdir-title = Change Directory
chdir-name = Directory ~n~ame
chdir-tree = Directory ~t~ree
chdir-chdir = ~C~hdir
chdir-revert = ~R~evert
chdir-invalid-directory = Invalid directory

## Editors

editor-untitled = Untitled
# $name: the file name, or the text of editor-untitled.
editor-save-changes =
    { $name } has been modified.

    Save changes?

## Colour dialog

color-dialog-prompt = Select foreground and background colors:
color-dialog-foreground = Foreground:
color-dialog-background = Background:
color-dialog-preview = Preview:
color-dialog-sample =
    Sample text with
    selected colors
# $color: the colour number (0-15); $hex: the same in hexadecimal, as 0x0F.
color-selector-selected = Selected: { $color } ({ $hex })

## Help

help-title = Help
help-see-also = See also:
help-index-prompt = Search for a topic:
help-index-search = Search:
help-index-topics = Topics:
help-toc-prompt = Browse help topics:
help-view = View
help-close = Close

## History drop-down

history-title = History
