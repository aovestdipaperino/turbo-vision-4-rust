// (C) 2025 - Enzo Lombardi

//! Syntax highlighting system - extensible token-based syntax coloring for editors.
// Syntax Highlighting System
//
// Provides extensible syntax highlighting for the Editor component.
// Supports multiple programming languages with token-based coloring.
//
// Architecture:
// - SyntaxHighlighter trait - Define highlighting rules for a language
// - TokenType enum - Classification of syntax elements
// - Token struct - Represents a highlighted span (start, end, type)
// - Built-in highlighters for common languages

use crate::core::palette::Attr;

/// Token types for syntax highlighting
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TokenType {
    /// Normal text (default)
    Normal,
    /// Keywords (if, for, while, etc.)
    Keyword,
    /// String literals ("text", 'c')
    String,
    /// Comments (// line, /* block */)
    Comment,
    /// Numeric literals (123, 0xFF, 3.14)
    Number,
    /// Operators (+, -, *, ==, etc.)
    Operator,
    /// Identifiers (variable names, function names)
    Identifier,
    /// Type names (struct, enum, class)
    Type,
    /// Preprocessor directives (#include, #define)
    Preprocessor,
    /// Function names
    Function,
    /// Special characters
    Special,
}

impl TokenType {
    /// Get the default color attribute for a token type
    pub fn default_color(&self) -> Attr {
        use crate::core::palette::colors::*;
        match self {
            TokenType::Normal => SYNTAX_NORMAL,
            TokenType::Keyword => SYNTAX_KEYWORD,
            TokenType::String => SYNTAX_STRING,
            TokenType::Comment => SYNTAX_COMMENT,
            TokenType::Number => SYNTAX_NUMBER,
            TokenType::Operator => SYNTAX_OPERATOR,
            TokenType::Identifier => SYNTAX_IDENTIFIER,
            TokenType::Type => SYNTAX_TYPE,
            TokenType::Preprocessor => SYNTAX_PREPROCESSOR,
            TokenType::Function => SYNTAX_FUNCTION,
            TokenType::Special => SYNTAX_SPECIAL,
        }
    }
}

/// Represents a highlighted token (span of text with a type)
#[derive(Debug, Clone)]
pub struct Token {
    /// Start column (character index)
    pub start: usize,
    /// End column (exclusive)
    pub end: usize,
    /// Token type
    pub token_type: TokenType,
}

impl Token {
    pub fn new(start: usize, end: usize, token_type: TokenType) -> Self {
        Token {
            start,
            end,
            token_type,
        }
    }
}

/// Trait for syntax highlighters
///
/// Implement this trait to add syntax highlighting for a language.
pub trait SyntaxHighlighter: Send + Sync {
    /// Get the language name
    fn language(&self) -> &str;

    /// Highlight a single line of text, returning tokens
    fn highlight_line(&self, line: &str, line_number: usize) -> Vec<Token>;

    /// Optional: Returns true if this is a multi-line context (e.g., inside a block comment)
    /// Default implementation returns false (no multi-line state)
    fn is_multiline_context(&self, _line_number: usize) -> bool {
        false
    }

    /// Optional: Update multi-line state after processing a line
    /// Default implementation does nothing
    fn update_multiline_state(&mut self, _line: &str, _line_number: usize) {}
}

/// Plain text highlighter (no highlighting)
pub struct PlainTextHighlighter;

impl SyntaxHighlighter for PlainTextHighlighter {
    fn language(&self) -> &str {
        "text"
    }

    fn highlight_line(&self, line: &str, _line_number: usize) -> Vec<Token> {
        if line.is_empty() {
            vec![]
        } else {
            vec![Token::new(0, line.len(), TokenType::Normal)]
        }
    }
}

/// Rust syntax highlighter
pub struct RustHighlighter {
    in_block_comment: bool,
}

impl RustHighlighter {
    pub fn new() -> Self {
        RustHighlighter {
            in_block_comment: false,
        }
    }

    fn is_rust_keyword(word: &str) -> bool {
        matches!(
            word,
            "as" | "async" | "await" | "break" | "const" | "continue" | "crate"
                | "dyn" | "else" | "enum" | "extern" | "false" | "fn" | "for"
                | "if" | "impl" | "in" | "let" | "loop" | "match" | "mod" | "move"
                | "mut" | "pub" | "ref" | "return" | "self" | "Self" | "static"
                | "struct" | "super" | "trait" | "true" | "type" | "unsafe" | "use"
                | "where" | "while"
        )
    }

    fn is_rust_type(word: &str) -> bool {
        // Common Rust types
        matches!(
            word,
            "i8" | "i16" | "i32" | "i64" | "i128" | "isize"
                | "u8" | "u16" | "u32" | "u64" | "u128" | "usize"
                | "f32" | "f64" | "bool" | "char" | "str"
                | "String" | "Vec" | "Option" | "Result" | "Box" | "Rc" | "Arc"
        ) || word.chars().next().map_or(false, |c| c.is_uppercase())
    }
}

impl SyntaxHighlighter for RustHighlighter {
    fn language(&self) -> &str {
        "rust"
    }

    fn highlight_line(&self, line: &str, _line_number: usize) -> Vec<Token> {
        let mut tokens = Vec::new();
        let chars: Vec<char> = line.chars().collect();
        let mut i = 0;

        // Check for block comment continuation
        if self.in_block_comment {
            // Find end of block comment
            if let Some(end_pos) = line.find("*/") {
                tokens.push(Token::new(0, end_pos + 2, TokenType::Comment));
                i = end_pos + 2;
            } else {
                // Entire line is comment
                tokens.push(Token::new(0, line.len(), TokenType::Comment));
                return tokens;
            }
        }

        while i < chars.len() {
            let ch = chars[i];

            // Line comment
            if i + 1 < chars.len() && ch == '/' && chars[i + 1] == '/' {
                tokens.push(Token::new(i, chars.len(), TokenType::Comment));
                break;
            }

            // Block comment start
            if i + 1 < chars.len() && ch == '/' && chars[i + 1] == '*' {
                let start = i;
                i += 2;
                // Find end of block comment
                let mut found_end = false;
                while i + 1 < chars.len() {
                    if chars[i] == '*' && chars[i + 1] == '/' {
                        i += 2;
                        found_end = true;
                        break;
                    }
                    i += 1;
                }
                if !found_end {
                    i = chars.len();
                }
                tokens.push(Token::new(start, i, TokenType::Comment));
                continue;
            }

            // String literal
            if ch == '"' {
                let start = i;
                i += 1;
                while i < chars.len() {
                    if chars[i] == '\\' && i + 1 < chars.len() {
                        i += 2; // Skip escaped character
                    } else if chars[i] == '"' {
                        i += 1;
                        break;
                    } else {
                        i += 1;
                    }
                }
                tokens.push(Token::new(start, i, TokenType::String));
                continue;
            }

            // Character literal
            if ch == '\'' {
                let start = i;
                i += 1;
                while i < chars.len() {
                    if chars[i] == '\\' && i + 1 < chars.len() {
                        i += 2; // Skip escaped character
                    } else if chars[i] == '\'' {
                        i += 1;
                        break;
                    } else {
                        i += 1;
                    }
                }
                tokens.push(Token::new(start, i, TokenType::String));
                continue;
            }

            // Number literal
            if ch.is_ascii_digit() {
                let start = i;
                while i < chars.len() && (chars[i].is_ascii_alphanumeric() || chars[i] == '_' || chars[i] == '.') {
                    i += 1;
                }
                tokens.push(Token::new(start, i, TokenType::Number));
                continue;
            }

            // Operator
            if matches!(ch, '+' | '-' | '*' | '/' | '%' | '=' | '<' | '>' | '!' | '&' | '|' | '^' | '~') {
                let start = i;
                i += 1;
                // Handle multi-character operators
                while i < chars.len() && matches!(chars[i], '=' | '&' | '|' | '<' | '>') {
                    i += 1;
                }
                tokens.push(Token::new(start, i, TokenType::Operator));
                continue;
            }

            // Identifier or keyword
            if ch.is_alphabetic() || ch == '_' {
                let start = i;
                while i < chars.len() && (chars[i].is_alphanumeric() || chars[i] == '_') {
                    i += 1;
                }
                let word: String = chars[start..i].iter().collect();

                let token_type = if Self::is_rust_keyword(&word) {
                    TokenType::Keyword
                } else if Self::is_rust_type(&word) {
                    TokenType::Type
                } else {
                    TokenType::Identifier
                };

                tokens.push(Token::new(start, i, token_type));
                continue;
            }

            // Skip other characters (whitespace, punctuation)
            i += 1;
        }

        tokens
    }

    fn is_multiline_context(&self, _line_number: usize) -> bool {
        self.in_block_comment
    }

    fn update_multiline_state(&mut self, line: &str, _line_number: usize) {
        // Check if we enter or exit a block comment
        let mut chars = line.chars().peekable();
        while let Some(ch) = chars.next() {
            if ch == '/' && chars.peek() == Some(&'*') {
                self.in_block_comment = true;
                chars.next();
            } else if ch == '*' && chars.peek() == Some(&'/') {
                self.in_block_comment = false;
                chars.next();
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_token_type_default_colors() {
        // Verify each token type has a color
        assert_ne!(TokenType::Keyword.default_color(), Attr::new(TvColor::Black, TvColor::Black));
        assert_ne!(TokenType::String.default_color(), Attr::new(TvColor::Black, TvColor::Black));
    }

    #[test]
    fn test_plain_text_highlighter() {
        let highlighter = PlainTextHighlighter;
        let tokens = highlighter.highlight_line("Hello, world!", 0);
        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0].token_type, TokenType::Normal);
    }

    #[test]
    fn test_rust_highlighter_keywords() {
        let highlighter = RustHighlighter::new();
        let tokens = highlighter.highlight_line("fn main() {", 0);

        // Should have tokens for "fn", "main", and possibly others
        assert!(!tokens.is_empty());

        // Find "fn" token
        let fn_token = tokens.iter().find(|t| t.token_type == TokenType::Keyword);
        assert!(fn_token.is_some(), "Should find 'fn' keyword");
    }

    #[test]
    fn test_rust_highlighter_strings() {
        let highlighter = RustHighlighter::new();
        let tokens = highlighter.highlight_line(r#"let s = "hello";"#, 0);

        // Find string token
        let string_token = tokens.iter().find(|t| t.token_type == TokenType::String);
        assert!(string_token.is_some(), "Should find string literal");
    }

    #[test]
    fn test_rust_highlighter_comments() {
        let highlighter = RustHighlighter::new();
        let tokens = highlighter.highlight_line("// This is a comment", 0);

        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0].token_type, TokenType::Comment);
    }

    #[test]
    fn test_rust_highlighter_numbers() {
        let highlighter = RustHighlighter::new();
        let tokens = highlighter.highlight_line("let x = 42;", 0);

        // Find number token
        let number_token = tokens.iter().find(|t| t.token_type == TokenType::Number);
        assert!(number_token.is_some(), "Should find number literal");
    }

    #[test]
    fn test_rust_highlighter_types() {
        let highlighter = RustHighlighter::new();
        let tokens = highlighter.highlight_line("let x: i32 = 0;", 0);

        // Find type token
        let type_token = tokens.iter().find(|t| t.token_type == TokenType::Type);
        assert!(type_token.is_some(), "Should find type name");
    }
}
