# Rust Editor Help {#intro}

Welcome to **Rust Editor**, a demonstration text editor built with turbo-vision.

This editor showcases the capabilities of the Turbo Vision framework for Rust,
including windows, menus, dialogs, and the help system you're reading now.

## Quick Start

- Press **F3** to open a file
- Press **F2** to save
- Press **Alt+X** to exit
- Press **F1** for help (this window)

## Navigation

Use [File Menu](#file-menu) to open and save files.
Use [Edit Menu](#edit-menu) for clipboard operations.
Use [Window Menu](#window-menu) to manage editor windows.

---

# File Menu {#file-menu}

The File menu provides operations for working with files.

## New {#file-new}

Creates a new empty editor window.

**Shortcut:** None

## Open {#file-open}

Opens the file selection dialog to choose a file to edit.

**Shortcut:** F3

The [Open Dialog](#open-dialog) allows you to:
- Navigate directories
- Filter files by extension
- Preview file information

## Save {#file-save}

Saves the current file to disk.

**Shortcut:** F2

If the file has never been saved, you'll be prompted to choose a filename.

## Save As {#file-save-as}

Saves the current file with a new name.

**Shortcut:** None

Opens a dialog to choose the new filename and location.

## Exit {#file-exit}

Exits the application.

**Shortcut:** Alt+X

If there are unsaved changes, you'll be prompted to save first.

---

# Edit Menu {#edit-menu}

The Edit menu provides clipboard and editing operations.

## Undo {#edit-undo}

Undoes the last editing operation.

**Shortcut:** Ctrl+Z

## Cut {#edit-cut}

Cuts the selected text to the clipboard.

**Shortcut:** Ctrl+X or Shift+Delete

## Copy {#edit-copy}

Copies the selected text to the clipboard.

**Shortcut:** Ctrl+C or Ctrl+Insert

## Paste {#edit-paste}

Pastes text from the clipboard at the cursor position.

**Shortcut:** Ctrl+V or Shift+Insert

## Select All {#edit-select-all}

Selects all text in the current document.

**Shortcut:** Ctrl+A

---

# Search Menu {#search-menu}

The Search menu provides find and replace operations.

## Find {#search-find}

Opens the Find dialog to search for text.

**Shortcut:** Ctrl+F

Options:
- Case sensitive search
- Whole words only
- Regular expressions

## Replace {#search-replace}

Opens the Find and Replace dialog.

**Shortcut:** Ctrl+H

Allows replacing found text with new text.

## Find Next {#search-find-next}

Finds the next occurrence of the search text.

**Shortcut:** F3 (after initial search)

## Find Previous {#search-find-prev}

Finds the previous occurrence of the search text.

**Shortcut:** Shift+F3

---

# Window Menu {#window-menu}

The Window menu provides operations for managing editor windows.

## Tile {#window-tile}

Arranges all open windows in a non-overlapping grid pattern.

This is useful when you want to see multiple files at once.

## Cascade {#window-cascade}

Arranges all open windows in an overlapping cascade pattern.

Each window is offset from the previous, allowing you to see
the title bars of all windows.

## Close {#window-close}

Closes the current window.

**Shortcut:** Alt+F3

If the file has unsaved changes, you'll be prompted to save.

## Next {#window-next}

Switches to the next window.

**Shortcut:** F6

## Previous {#window-prev}

Switches to the previous window.

**Shortcut:** Shift+F6

---

# Open Dialog {#open-dialog}

The Open dialog allows you to select a file to open.

## Components

- **Name field**: Enter a filename or wildcard pattern (e.g., `*.rs`)
- **File list**: Shows files matching the current pattern
- **Directory tree**: Navigate to different folders

## Navigation

- Use arrow keys to move between files
- Press Enter to open the selected file
- Double-click a file to open it
- Double-click a directory to enter it

## Filters

Type a pattern in the Name field to filter files:
- `*.rs` - Show only Rust files
- `*.txt` - Show only text files
- `*.*` - Show all files

---

# Keyboard Reference {#keyboard}

## General

| Key | Action |
|-----|--------|
| F1 | Help |
| F2 | Save |
| F3 | Open |
| F6 | Next window |
| Shift+F6 | Previous window |
| Alt+F3 | Close window |
| Alt+X | Exit |

## Editing

| Key | Action |
|-----|--------|
| Ctrl+Z | Undo |
| Ctrl+X | Cut |
| Ctrl+C | Copy |
| Ctrl+V | Paste |
| Ctrl+A | Select all |

## Navigation

| Key | Action |
|-----|--------|
| Home | Start of line |
| End | End of line |
| Ctrl+Home | Start of file |
| Ctrl+End | End of file |
| PgUp | Page up |
| PgDn | Page down |

---

# About {#about}

**Rust Editor** is a demonstration application for the turbo-vision framework.

Version: 1.0.0

turbo-vision is a Rust port of Borland's classic Turbo Vision text-mode UI
framework. It provides a complete set of views (widgets), windows, dialogs,
menus, and other components for building text-mode applications.

For more information, visit the project repository.

---

# Help Navigation {#help-nav}

## Using This Help System

- Press **TAB** to move to the next link
- Press **Shift+TAB** to move to the previous link
- Press **ENTER** to follow the selected link
- Press **ESC** to close the help window

## Links

Links appear highlighted. Click on them or press ENTER when selected
to navigate to that topic.

See also:
- [Introduction](#intro)
- [Keyboard Reference](#keyboard)
