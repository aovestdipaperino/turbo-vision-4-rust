// (C) 2025-2026 - Enzo Lombardi
// Pascal IDE Editor - Port of the Bruto-Pascal IDE editor
//
// Based on bruto-ide (https://github.com/aovestdipaperino/bruto-ide)
//
// Features:
// - Pascal syntax highlighting (from bruto-pascal-lang)
// - Menu bar and status line
// - Sample Pascal program loaded on startup

use turbo_vision::app::Application;
use turbo_vision::core::command::{CM_QUIT, CM_CLOSE};
use turbo_vision::core::event::KB_F10;
use turbo_vision::core::geometry::Rect;
use turbo_vision::core::menu_data::{Menu, MenuItem};
use turbo_vision::views::edit_window::EditWindow;
use turbo_vision::views::menu_bar::{MenuBar, SubMenu};
use turbo_vision::views::msgbox::message_box_ok;
use turbo_vision::views::status_line::{StatusItem, StatusLine};
use turbo_vision::views::syntax::{SyntaxHighlighter, Token, TokenType};

// ── Pascal Syntax Highlighter ────────────────────────────
// From bruto-pascal-lang (https://github.com/aovestdipaperino/bruto-pascal-lang)

struct PascalHighlighter {
    in_block_comment_brace: bool,
    in_block_comment_paren: bool,
}

impl PascalHighlighter {
    fn new() -> Self {
        Self { in_block_comment_brace: false, in_block_comment_paren: false }
    }

    fn is_keyword(word: &str) -> bool {
        matches!(
            word,
            "program" | "var" | "begin" | "end" | "if" | "then" | "else"
                | "while" | "do" | "for" | "to" | "downto" | "repeat" | "until"
                | "write" | "writeln" | "read" | "readln"
                | "div" | "mod" | "and" | "or" | "not"
                | "true" | "false" | "const" | "type" | "procedure" | "function"
                | "array" | "of" | "record" | "nil" | "case" | "with"
                | "new" | "dispose" | "forward"
        )
    }

    fn is_type_name(word: &str) -> bool {
        matches!(word, "integer" | "string" | "boolean" | "real" | "char" | "byte" | "word" | "longint" | "text")
    }
}

impl SyntaxHighlighter for PascalHighlighter {
    fn language(&self) -> &str { "pascal" }

    fn highlight_line(&self, line: &str, _line_number: usize) -> Vec<Token> {
        let mut tokens = Vec::new();
        let chars: Vec<char> = line.chars().collect();
        let len = chars.len();
        let mut i = 0;

        let mut in_brace = self.in_block_comment_brace;
        let mut in_paren = self.in_block_comment_paren;

        while i < len {
            if in_brace {
                let start = i;
                while i < len && chars[i] != '}' { i += 1; }
                if i < len { i += 1; in_brace = false; }
                tokens.push(Token::new(start, i, TokenType::Comment));
                continue;
            }
            if in_paren {
                let start = i;
                while i < len {
                    if i + 1 < len && chars[i] == '*' && chars[i + 1] == ')' {
                        i += 2; in_paren = false; break;
                    }
                    i += 1;
                }
                tokens.push(Token::new(start, i, TokenType::Comment));
                continue;
            }

            let ch = chars[i];

            if ch.is_whitespace() {
                let start = i;
                while i < len && chars[i].is_whitespace() { i += 1; }
                tokens.push(Token::new(start, i, TokenType::Normal));
                continue;
            }
            if ch == '/' && i + 1 < len && chars[i + 1] == '/' {
                tokens.push(Token::new(i, len, TokenType::Comment));
                break;
            }
            if ch == '{' {
                let start = i; i += 1;
                while i < len && chars[i] != '}' { i += 1; }
                if i < len { i += 1; } else { in_brace = true; }
                tokens.push(Token::new(start, i, TokenType::Comment));
                continue;
            }
            if ch == '(' && i + 1 < len && chars[i + 1] == '*' {
                let start = i; i += 2;
                while i < len {
                    if i + 1 < len && chars[i] == '*' && chars[i + 1] == ')' { i += 2; break; }
                    i += 1;
                }
                if i >= len && !(i >= 2 && chars[i - 2] == '*' && chars[i - 1] == ')') {
                    in_paren = true;
                }
                tokens.push(Token::new(start, i, TokenType::Comment));
                continue;
            }
            if ch == '\'' {
                let start = i; i += 1;
                while i < len {
                    if chars[i] == '\'' {
                        if i + 1 < len && chars[i + 1] == '\'' { i += 2; }
                        else { i += 1; break; }
                    } else { i += 1; }
                }
                tokens.push(Token::new(start, i, TokenType::String));
                continue;
            }
            if ch.is_ascii_digit() || (ch == '$' && i + 1 < len && chars[i + 1].is_ascii_hexdigit()) {
                let start = i;
                if ch == '$' { i += 1; while i < len && chars[i].is_ascii_hexdigit() { i += 1; } }
                else { while i < len && chars[i].is_ascii_digit() { i += 1; } }
                tokens.push(Token::new(start, i, TokenType::Number));
                continue;
            }
            if ch.is_ascii_alphabetic() || ch == '_' {
                let start = i;
                while i < len && (chars[i].is_ascii_alphanumeric() || chars[i] == '_') { i += 1; }
                let word: String = chars[start..i].iter().collect();
                let lower = word.to_lowercase();
                let token_type = if Self::is_keyword(&lower) { TokenType::Keyword }
                    else if Self::is_type_name(&lower) { TokenType::Type }
                    else { TokenType::Identifier };
                tokens.push(Token::new(start, i, token_type));
                continue;
            }
            if i + 1 < len {
                let two: String = chars[i..i + 2].iter().collect();
                if matches!(two.as_str(), ":=" | "<>" | "<=" | ">=") {
                    tokens.push(Token::new(i, i + 2, TokenType::Operator));
                    i += 2;
                    continue;
                }
            }
            if matches!(ch, '+' | '-' | '*' | '=' | '<' | '>' | '/') {
                tokens.push(Token::new(i, i + 1, TokenType::Operator));
                i += 1; continue;
            }
            if matches!(ch, ';' | ':' | '.' | ',' | '(' | ')' | '[' | ']') {
                tokens.push(Token::new(i, i + 1, TokenType::Special));
                i += 1; continue;
            }
            tokens.push(Token::new(i, i + 1, TokenType::Normal));
            i += 1;
        }
        tokens
    }

    fn is_multiline_context(&self, _line_number: usize) -> bool {
        self.in_block_comment_brace || self.in_block_comment_paren
    }

    fn update_multiline_state(&mut self, line: &str, _line_number: usize) {
        let chars: Vec<char> = line.chars().collect();
        let len = chars.len();
        let mut i = 0;
        while i < len {
            if self.in_block_comment_brace {
                if chars[i] == '}' { self.in_block_comment_brace = false; }
                i += 1; continue;
            }
            if self.in_block_comment_paren {
                if i + 1 < len && chars[i] == '*' && chars[i + 1] == ')' {
                    self.in_block_comment_paren = false; i += 2; continue;
                }
                i += 1; continue;
            }
            if chars[i] == '\'' {
                i += 1;
                while i < len && chars[i] != '\'' { i += 1; }
                if i < len { i += 1; }
                continue;
            }
            if chars[i] == '/' && i + 1 < len && chars[i + 1] == '/' { break; }
            if chars[i] == '{' { self.in_block_comment_brace = true; i += 1; continue; }
            if chars[i] == '(' && i + 1 < len && chars[i + 1] == '*' {
                self.in_block_comment_paren = true; i += 2; continue;
            }
            i += 1;
        }
    }
}

// ── Sample Program ───────────────────────────────────────
// From bruto-pascal-lang (https://github.com/aovestdipaperino/bruto-pascal-lang)

const SAMPLE_PROGRAM: &str = r#"program Demo;
const
  Size = 5;

type
  Vector = array[1..5] of integer;
  Point = record
    x, y: real;
  end;

var
  nums: Vector;
  pt: Point;
  total, i: integer;
  avg: real;
  msg: string;
  p: ^integer;

procedure Fill(var a: Vector; n: integer);
var
  k: integer;
begin
  for k := 1 to n do
    a[k] := k * k
end;

function Sum(var a: Vector; n: integer): integer;
var
  s, j: integer;
begin
  s := 0;
  for j := 1 to n do
    s := s + a[j];
  Sum := s
end;

begin
  { Arrays and procedures }
  Fill(nums, Size);
  total := Sum(nums, Size);
  avg := total / Size;
  writeln('Squares: 1..', Size);
  for i := 1 to Size do
    write(nums[i], ' ');
  writeln;

  { Real arithmetic }
  writeln('Sum = ', total);
  writeln('Avg = ', avg);

  { Records }
  pt.x := avg;
  pt.y := 3.14;
  writeln('Point = (', pt.x, ', ', pt.y, ')');

  { Strings }
  msg := 'Hello' + ' ' + 'Pascal!';
  writeln(msg, ' len=', length(msg));

  { Heap pointers }
  new(p);
  p^ := 42;
  writeln('Heap value: ', p^);
  dispose(p);

  { Repeat-until }
  i := 1;
  repeat
    i := i * 2
  until i > 100;
  writeln('First power of 2 > 100: ', i);

  { ord / chr }
  writeln('ord(A) = ', ord('A'));
  write('chr(90) = ');
  writeln(chr(90));

  if total = 55 then
    writeln('All correct!')
  else
    writeln('Something is wrong!')
end.
"#;

// ── Main ─────────────────────────────────────────────────

fn main() -> turbo_vision::core::error::Result<()> {
    let mut app = Application::new()?;
    let (width, height) = app.terminal.size();
    let w = width as i16;
    let h = height as i16;

    // Menu bar
    let mut menu_bar = MenuBar::new(Rect::new(0, 0, w, 1));
    menu_bar.add_submenu(SubMenu::new("~F~ile", Menu::from_items(vec![
        MenuItem::with_shortcut("~C~lose", CM_CLOSE, 0, "Alt+F3", 0),
        MenuItem::separator(),
        MenuItem::with_shortcut("E~x~it", CM_QUIT, 0, "Alt+X", 0),
    ])));
    app.set_menu_bar(menu_bar);

    // Status line
    app.set_status_line(StatusLine::new(
        Rect::new(0, h - 1, w, h),
        vec![
            StatusItem::new("~F10~ Menu", KB_F10, 0),
            StatusItem::new("~Alt+X~ Exit", 0x012D, CM_QUIT),
        ],
    ));

    // Editor window with Pascal syntax highlighting and sample program
    let edit_window = EditWindow::new(Rect::new(2, 1, w - 2, h - 2), "Untitled.pas");
    edit_window.editor_rc().borrow_mut().set_highlighter(Box::new(PascalHighlighter::new()));
    edit_window.editor_rc().borrow_mut().set_text(SAMPLE_PROGRAM);
    app.desktop.add(Box::new(edit_window));

    // Show about dialog at startup
    message_box_ok(
        &mut app,
        "\x03Bruto Pascal IDE\n\n\
         Version 0.1.0\n\n\
         A Mini-Pascal IDE built with\n\
         Turbo Vision for Rust\n\n\
         \x03(c) 2026 Enzo Lombardi",
    );

    app.run();

    Ok(())
}
