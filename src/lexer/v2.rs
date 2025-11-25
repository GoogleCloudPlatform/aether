// Copyright 2025 Google LLC
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! V2 Lexical analysis for AetherScript
//!
//! Tokenizes the new Swift/Rust-like V2 syntax

use crate::error::{LexerError, SourceLocation};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// V2 Token types for AetherScript
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum TokenType {
    // Delimiters
    LeftParen,    // (
    RightParen,   // )
    LeftBrace,    // {
    RightBrace,   // }
    LeftBracket,  // [
    RightBracket, // ]

    // Punctuation
    Semicolon, // ;
    Colon,     // :
    Comma,     // ,
    Dot,       // .
    Arrow,     // ->
    At,        // @

    // Operators - Arithmetic
    Plus,    // +
    Minus,   // -
    Star,    // *
    Slash,   // /
    Percent, // %

    // Operators - Comparison
    EqualEqual,   // ==
    BangEqual,    // !=
    Less,         // <
    LessEqual,    // <=
    Greater,      // >
    GreaterEqual, // >=

    // Operators - Logical
    AmpAmp,   // &&
    PipePipe, // ||
    Bang,     // !

    // Operators - Assignment
    Equal, // =

    // Ownership Sigils
    Caret,     // ^ (owned)
    Ampersand, // & (borrowed, also used in &&)
    Tilde,     // ~ (shared)

    // Literals
    IntegerLiteral(i64),
    FloatLiteral(f64),
    StringLiteral(String),
    CharLiteral(char),
    BoolLiteral(bool),

    // Identifiers and Keywords
    Identifier(String),
    Keyword(Keyword),

    // Special
    Eof,
}

/// V2 Keywords
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Keyword {
    // Declarations
    Module,
    Import,
    Func,
    Let,
    Const,
    Struct,
    Enum,

    // Modifiers
    Mut,
    Pub,

    // Control Flow
    When,
    Case,
    Else,
    Match,
    For,
    While,
    In,
    Return,
    Break,
    Continue,

    // Error Handling
    Try,
    Catch,
    Finally,
    Throw,

    // Resource Management
    Resource,
    Cleanup,
    Guaranteed,

    // Types
    Int,
    Int64,
    Float,
    String_,
    Bool,
    Void,
    Array,
    Map,
    Pointer,
    MutPointer,
    SizeT,

    // Literals
    True,
    False,
    Nil,

    // Other
    As,
    Range,
}

/// A V2 token with its type and location information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Token {
    pub token_type: TokenType,
    pub location: SourceLocation,
    pub lexeme: String,
}

impl Token {
    pub fn new(token_type: TokenType, location: SourceLocation, lexeme: String) -> Self {
        Self {
            token_type,
            location,
            lexeme,
        }
    }
}

/// V2 Lexer for AetherScript
pub struct Lexer {
    input: Vec<char>,
    position: usize,
    current_char: Option<char>,
    line: usize,
    column: usize,
    file_name: String,
    keywords: HashMap<String, Keyword>,
}

impl Lexer {
    /// Create a new V2 lexer for the given input
    pub fn new(input: &str, file_name: String) -> Self {
        let chars: Vec<char> = input.chars().collect();
        let current_char = chars.first().copied();

        let mut lexer = Self {
            input: chars,
            position: 0,
            current_char,
            line: 1,
            column: 1,
            file_name,
            keywords: HashMap::new(),
        };

        lexer.initialize_keywords();
        lexer
    }

    /// Initialize the keyword lookup table
    fn initialize_keywords(&mut self) {
        // Declaration keywords
        self.keywords.insert("module".to_string(), Keyword::Module);
        self.keywords.insert("import".to_string(), Keyword::Import);
        self.keywords.insert("func".to_string(), Keyword::Func);
        self.keywords.insert("let".to_string(), Keyword::Let);
        self.keywords.insert("const".to_string(), Keyword::Const);
        self.keywords.insert("struct".to_string(), Keyword::Struct);
        self.keywords.insert("enum".to_string(), Keyword::Enum);

        // Modifier keywords
        self.keywords.insert("mut".to_string(), Keyword::Mut);
        self.keywords.insert("pub".to_string(), Keyword::Pub);

        // Control flow keywords
        self.keywords.insert("when".to_string(), Keyword::When);
        self.keywords.insert("case".to_string(), Keyword::Case);
        self.keywords.insert("else".to_string(), Keyword::Else);
        self.keywords.insert("match".to_string(), Keyword::Match);
        self.keywords.insert("for".to_string(), Keyword::For);
        self.keywords.insert("while".to_string(), Keyword::While);
        self.keywords.insert("in".to_string(), Keyword::In);
        self.keywords.insert("return".to_string(), Keyword::Return);
        self.keywords.insert("break".to_string(), Keyword::Break);
        self.keywords.insert("continue".to_string(), Keyword::Continue);

        // Error handling keywords
        self.keywords.insert("try".to_string(), Keyword::Try);
        self.keywords.insert("catch".to_string(), Keyword::Catch);
        self.keywords.insert("finally".to_string(), Keyword::Finally);
        self.keywords.insert("throw".to_string(), Keyword::Throw);

        // Resource management keywords
        self.keywords.insert("resource".to_string(), Keyword::Resource);
        self.keywords.insert("cleanup".to_string(), Keyword::Cleanup);
        self.keywords.insert("guaranteed".to_string(), Keyword::Guaranteed);

        // Type keywords
        self.keywords.insert("Int".to_string(), Keyword::Int);
        self.keywords.insert("Int64".to_string(), Keyword::Int64);
        self.keywords.insert("Float".to_string(), Keyword::Float);
        self.keywords.insert("String".to_string(), Keyword::String_);
        self.keywords.insert("Bool".to_string(), Keyword::Bool);
        self.keywords.insert("Void".to_string(), Keyword::Void);
        self.keywords.insert("Array".to_string(), Keyword::Array);
        self.keywords.insert("Map".to_string(), Keyword::Map);
        self.keywords.insert("Pointer".to_string(), Keyword::Pointer);
        self.keywords.insert("MutPointer".to_string(), Keyword::MutPointer);
        self.keywords.insert("SizeT".to_string(), Keyword::SizeT);

        // Literal keywords
        self.keywords.insert("true".to_string(), Keyword::True);
        self.keywords.insert("false".to_string(), Keyword::False);
        self.keywords.insert("nil".to_string(), Keyword::Nil);

        // Other keywords
        self.keywords.insert("as".to_string(), Keyword::As);
        self.keywords.insert("range".to_string(), Keyword::Range);
    }

    /// Get the current source location
    fn current_location(&self) -> SourceLocation {
        SourceLocation::new(self.file_name.clone(), self.line, self.column, self.position)
    }

    /// Advance to the next character
    fn advance(&mut self) {
        if self.current_char == Some('\n') {
            self.line += 1;
            self.column = 1;
        } else {
            self.column += 1;
        }

        self.position += 1;
        self.current_char = self.input.get(self.position).copied();
    }

    /// Peek at the next character without advancing
    fn peek(&self) -> Option<char> {
        self.input.get(self.position + 1).copied()
    }

    /// Skip whitespace characters
    fn skip_whitespace(&mut self) {
        while let Some(ch) = self.current_char {
            if ch.is_whitespace() {
                self.advance();
            } else {
                break;
            }
        }
    }

    /// Skip a line comment (// or ///) until end of line or EOF
    fn skip_line_comment(&mut self) {
        // We're positioned after the first '/', consume the second '/'
        self.advance();

        // Skip until newline or EOF
        while let Some(ch) = self.current_char {
            if ch == '\n' {
                self.advance(); // consume the newline
                break;
            }
            self.advance();
        }
    }

    /// Read a number (integer or float)
    fn read_number(&mut self) -> Result<Token, LexerError> {
        let start_location = self.current_location();
        let mut number_str = String::new();
        let mut is_float = false;

        // Read digits before decimal point
        while let Some(ch) = self.current_char {
            if ch.is_ascii_digit() {
                number_str.push(ch);
                self.advance();
            } else if ch == '.' && !is_float && self.peek().is_some_and(|c| c.is_ascii_digit()) {
                is_float = true;
                number_str.push(ch);
                self.advance();
            } else {
                break;
            }
        }

        if is_float {
            match number_str.parse::<f64>() {
                Ok(value) => Ok(Token::new(
                    TokenType::FloatLiteral(value),
                    start_location,
                    number_str,
                )),
                Err(_) => Err(LexerError::InvalidNumber {
                    value: number_str,
                    location: start_location,
                }),
            }
        } else {
            match number_str.parse::<i64>() {
                Ok(value) => Ok(Token::new(
                    TokenType::IntegerLiteral(value),
                    start_location,
                    number_str,
                )),
                Err(_) => Err(LexerError::InvalidNumber {
                    value: number_str,
                    location: start_location,
                }),
            }
        }
    }

    /// Read a string literal
    fn read_string(&mut self) -> Result<Token, LexerError> {
        let start_location = self.current_location();
        let mut string_value = String::new();
        let mut lexeme = String::new();

        // Skip opening quote
        lexeme.push('"');
        self.advance();

        while let Some(ch) = self.current_char {
            if ch == '"' {
                // End of string
                lexeme.push(ch);
                self.advance();
                return Ok(Token::new(
                    TokenType::StringLiteral(string_value),
                    start_location,
                    lexeme,
                ));
            } else if ch == '\\' {
                // Handle escape sequences
                lexeme.push(ch);
                self.advance();
                match self.current_char {
                    Some('n') => {
                        string_value.push('\n');
                        lexeme.push('n');
                    }
                    Some('t') => {
                        string_value.push('\t');
                        lexeme.push('t');
                    }
                    Some('r') => {
                        string_value.push('\r');
                        lexeme.push('r');
                    }
                    Some('\\') => {
                        string_value.push('\\');
                        lexeme.push('\\');
                    }
                    Some('"') => {
                        string_value.push('"');
                        lexeme.push('"');
                    }
                    Some('0') => {
                        string_value.push('\0');
                        lexeme.push('0');
                    }
                    Some(other) => {
                        return Err(LexerError::InvalidEscapeSequence {
                            sequence: other.to_string(),
                            location: self.current_location(),
                        });
                    }
                    None => {
                        return Err(LexerError::UnterminatedString {
                            location: start_location,
                        });
                    }
                }
                self.advance();
            } else if ch == '\n' || ch == '\r' {
                return Err(LexerError::UnterminatedString {
                    location: start_location,
                });
            } else {
                string_value.push(ch);
                lexeme.push(ch);
                self.advance();
            }
        }

        Err(LexerError::UnterminatedString {
            location: start_location,
        })
    }

    /// Read a character literal
    fn read_char(&mut self) -> Result<Token, LexerError> {
        let start_location = self.current_location();
        let mut lexeme = String::new();

        // Skip opening quote
        lexeme.push('\'');
        self.advance();

        let char_value = match self.current_char {
            Some('\\') => {
                // Handle escape sequences
                lexeme.push('\\');
                self.advance();
                match self.current_char {
                    Some('n') => {
                        lexeme.push('n');
                        self.advance();
                        '\n'
                    }
                    Some('t') => {
                        lexeme.push('t');
                        self.advance();
                        '\t'
                    }
                    Some('r') => {
                        lexeme.push('r');
                        self.advance();
                        '\r'
                    }
                    Some('\\') => {
                        lexeme.push('\\');
                        self.advance();
                        '\\'
                    }
                    Some('\'') => {
                        lexeme.push('\'');
                        self.advance();
                        '\''
                    }
                    Some('0') => {
                        lexeme.push('0');
                        self.advance();
                        '\0'
                    }
                    Some(other) => {
                        return Err(LexerError::InvalidEscapeSequence {
                            sequence: other.to_string(),
                            location: self.current_location(),
                        });
                    }
                    None => {
                        return Err(LexerError::UnterminatedString {
                            location: start_location,
                        });
                    }
                }
            }
            Some(ch) if ch != '\'' => {
                lexeme.push(ch);
                self.advance();
                ch
            }
            _ => {
                return Err(LexerError::UnterminatedString {
                    location: start_location,
                });
            }
        };

        // Expect closing quote
        match self.current_char {
            Some('\'') => {
                lexeme.push('\'');
                self.advance();
                Ok(Token::new(
                    TokenType::CharLiteral(char_value),
                    start_location,
                    lexeme,
                ))
            }
            _ => Err(LexerError::UnterminatedString {
                location: start_location,
            }),
        }
    }

    /// Read an identifier or keyword
    fn read_identifier(&mut self) -> Token {
        let start_location = self.current_location();
        let mut identifier = String::new();

        while let Some(ch) = self.current_char {
            if ch.is_ascii_alphanumeric() || ch == '_' {
                identifier.push(ch);
                self.advance();
            } else {
                break;
            }
        }

        // Check if it's a keyword
        let token_type = if let Some(keyword) = self.keywords.get(&identifier) {
            // Special handling for true/false/nil
            match keyword {
                Keyword::True => TokenType::BoolLiteral(true),
                Keyword::False => TokenType::BoolLiteral(false),
                Keyword::Nil => TokenType::Identifier("nil".to_string()), // Could be a special NilLiteral
                _ => TokenType::Keyword(keyword.clone()),
            }
        } else {
            TokenType::Identifier(identifier.clone())
        };

        Token::new(token_type, start_location, identifier)
    }

    /// Get the next token from the input
    pub fn next_token(&mut self) -> Result<Token, LexerError> {
        self.skip_whitespace();

        let location = self.current_location();

        match self.current_char {
            None => Ok(Token::new(TokenType::Eof, location, String::new())),

            // Identifiers and keywords
            Some(ch) if ch.is_ascii_alphabetic() || ch == '_' => Ok(self.read_identifier()),

            // Numbers
            Some(ch) if ch.is_ascii_digit() => self.read_number(),

            // String literals
            Some('"') => self.read_string(),

            // Character literals
            Some('\'') => self.read_char(),

            // Delimiters
            Some('{') => {
                self.advance();
                Ok(Token::new(TokenType::LeftBrace, location, "{".to_string()))
            }
            Some('}') => {
                self.advance();
                Ok(Token::new(TokenType::RightBrace, location, "}".to_string()))
            }
            Some('[') => {
                self.advance();
                Ok(Token::new(TokenType::LeftBracket, location, "[".to_string()))
            }
            Some(']') => {
                self.advance();
                Ok(Token::new(TokenType::RightBracket, location, "]".to_string()))
            }
            Some('(') => {
                self.advance();
                Ok(Token::new(TokenType::LeftParen, location, "(".to_string()))
            }
            Some(')') => {
                self.advance();
                Ok(Token::new(TokenType::RightParen, location, ")".to_string()))
            }
            Some(';') => {
                self.advance();
                Ok(Token::new(TokenType::Semicolon, location, ";".to_string()))
            }
            Some(':') => {
                self.advance();
                Ok(Token::new(TokenType::Colon, location, ":".to_string()))
            }
            Some(',') => {
                self.advance();
                Ok(Token::new(TokenType::Comma, location, ",".to_string()))
            }
            Some('.') => {
                self.advance();
                Ok(Token::new(TokenType::Dot, location, ".".to_string()))
            }
            Some('@') => {
                self.advance();
                Ok(Token::new(TokenType::At, location, "@".to_string()))
            }

            // Arithmetic operators
            Some('+') => {
                self.advance();
                Ok(Token::new(TokenType::Plus, location, "+".to_string()))
            }
            Some('*') => {
                self.advance();
                Ok(Token::new(TokenType::Star, location, "*".to_string()))
            }
            // Slash, or line comment, or doc comment
            Some('/') => {
                self.advance();
                if self.current_char == Some('/') {
                    // Line comment (// or ///)
                    self.skip_line_comment();
                    // Continue to get the next token after the comment
                    self.next_token()
                } else {
                    Ok(Token::new(TokenType::Slash, location, "/".to_string()))
                }
            }
            Some('%') => {
                self.advance();
                Ok(Token::new(TokenType::Percent, location, "%".to_string()))
            }

            // Minus or Arrow
            Some('-') => {
                self.advance();
                if self.current_char == Some('>') {
                    self.advance();
                    Ok(Token::new(TokenType::Arrow, location, "->".to_string()))
                } else {
                    Ok(Token::new(TokenType::Minus, location, "-".to_string()))
                }
            }

            // Equal or EqualEqual
            Some('=') => {
                self.advance();
                if self.current_char == Some('=') {
                    self.advance();
                    Ok(Token::new(TokenType::EqualEqual, location, "==".to_string()))
                } else {
                    Ok(Token::new(TokenType::Equal, location, "=".to_string()))
                }
            }

            // Bang or BangEqual
            Some('!') => {
                self.advance();
                if self.current_char == Some('=') {
                    self.advance();
                    Ok(Token::new(TokenType::BangEqual, location, "!=".to_string()))
                } else {
                    Ok(Token::new(TokenType::Bang, location, "!".to_string()))
                }
            }

            // Less or LessEqual
            Some('<') => {
                self.advance();
                if self.current_char == Some('=') {
                    self.advance();
                    Ok(Token::new(TokenType::LessEqual, location, "<=".to_string()))
                } else {
                    Ok(Token::new(TokenType::Less, location, "<".to_string()))
                }
            }

            // Greater or GreaterEqual
            Some('>') => {
                self.advance();
                if self.current_char == Some('=') {
                    self.advance();
                    Ok(Token::new(TokenType::GreaterEqual, location, ">=".to_string()))
                } else {
                    Ok(Token::new(TokenType::Greater, location, ">".to_string()))
                }
            }

            // Ampersand or AmpAmp
            Some('&') => {
                self.advance();
                if self.current_char == Some('&') {
                    self.advance();
                    Ok(Token::new(TokenType::AmpAmp, location, "&&".to_string()))
                } else {
                    Ok(Token::new(TokenType::Ampersand, location, "&".to_string()))
                }
            }

            // Pipe or PipePipe
            Some('|') => {
                self.advance();
                if self.current_char == Some('|') {
                    self.advance();
                    Ok(Token::new(TokenType::PipePipe, location, "||".to_string()))
                } else {
                    // Single pipe is not a valid token in V2 syntax
                    Err(LexerError::UnexpectedCharacter { character: '|', location })
                }
            }

            // Ownership sigils
            Some('^') => {
                self.advance();
                Ok(Token::new(TokenType::Caret, location, "^".to_string()))
            }
            Some('~') => {
                self.advance();
                Ok(Token::new(TokenType::Tilde, location, "~".to_string()))
            }

            // Unknown character
            Some(ch) => Err(LexerError::UnexpectedCharacter { character: ch, location }),
        }
    }

    /// Tokenize the entire input and return a vector of tokens
    pub fn tokenize(&mut self) -> Result<Vec<Token>, LexerError> {
        let mut tokens = Vec::new();

        loop {
            let token = self.next_token()?;
            let is_eof = matches!(token.token_type, TokenType::Eof);
            tokens.push(token);

            if is_eof {
                break;
            }
        }

        Ok(tokens)
    }

    /// Look up a keyword by name (for testing)
    pub fn lookup_keyword(&self, name: &str) -> Option<&Keyword> {
        self.keywords.get(name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Helper to create a simple location for tests
    fn test_location() -> SourceLocation {
        SourceLocation::new("test.aether".to_string(), 1, 1, 0)
    }

    // ==================== DELIMITER TESTS ====================

    #[test]
    fn test_left_brace_token() {
        let token = Token::new(TokenType::LeftBrace, test_location(), "{".to_string());
        assert!(matches!(token.token_type, TokenType::LeftBrace));
        assert_eq!(token.lexeme, "{");
    }

    #[test]
    fn test_right_brace_token() {
        let token = Token::new(TokenType::RightBrace, test_location(), "}".to_string());
        assert!(matches!(token.token_type, TokenType::RightBrace));
        assert_eq!(token.lexeme, "}");
    }

    #[test]
    fn test_left_bracket_token() {
        let token = Token::new(TokenType::LeftBracket, test_location(), "[".to_string());
        assert!(matches!(token.token_type, TokenType::LeftBracket));
        assert_eq!(token.lexeme, "[");
    }

    #[test]
    fn test_right_bracket_token() {
        let token = Token::new(TokenType::RightBracket, test_location(), "]".to_string());
        assert!(matches!(token.token_type, TokenType::RightBracket));
        assert_eq!(token.lexeme, "]");
    }

    #[test]
    fn test_left_paren_token() {
        let token = Token::new(TokenType::LeftParen, test_location(), "(".to_string());
        assert!(matches!(token.token_type, TokenType::LeftParen));
        assert_eq!(token.lexeme, "(");
    }

    #[test]
    fn test_right_paren_token() {
        let token = Token::new(TokenType::RightParen, test_location(), ")".to_string());
        assert!(matches!(token.token_type, TokenType::RightParen));
        assert_eq!(token.lexeme, ")");
    }

    #[test]
    fn test_semicolon_token() {
        let token = Token::new(TokenType::Semicolon, test_location(), ";".to_string());
        assert!(matches!(token.token_type, TokenType::Semicolon));
        assert_eq!(token.lexeme, ";");
    }

    #[test]
    fn test_colon_token() {
        let token = Token::new(TokenType::Colon, test_location(), ":".to_string());
        assert!(matches!(token.token_type, TokenType::Colon));
        assert_eq!(token.lexeme, ":");
    }

    #[test]
    fn test_comma_token() {
        let token = Token::new(TokenType::Comma, test_location(), ",".to_string());
        assert!(matches!(token.token_type, TokenType::Comma));
        assert_eq!(token.lexeme, ",");
    }

    #[test]
    fn test_dot_token() {
        let token = Token::new(TokenType::Dot, test_location(), ".".to_string());
        assert!(matches!(token.token_type, TokenType::Dot));
        assert_eq!(token.lexeme, ".");
    }

    // ==================== OPERATOR TESTS ====================

    #[test]
    fn test_arrow_token() {
        let token = Token::new(TokenType::Arrow, test_location(), "->".to_string());
        assert!(matches!(token.token_type, TokenType::Arrow));
        assert_eq!(token.lexeme, "->");
    }

    #[test]
    fn test_at_token() {
        let token = Token::new(TokenType::At, test_location(), "@".to_string());
        assert!(matches!(token.token_type, TokenType::At));
        assert_eq!(token.lexeme, "@");
    }

    #[test]
    fn test_plus_token() {
        let token = Token::new(TokenType::Plus, test_location(), "+".to_string());
        assert!(matches!(token.token_type, TokenType::Plus));
        assert_eq!(token.lexeme, "+");
    }

    #[test]
    fn test_minus_token() {
        let token = Token::new(TokenType::Minus, test_location(), "-".to_string());
        assert!(matches!(token.token_type, TokenType::Minus));
        assert_eq!(token.lexeme, "-");
    }

    #[test]
    fn test_star_token() {
        let token = Token::new(TokenType::Star, test_location(), "*".to_string());
        assert!(matches!(token.token_type, TokenType::Star));
        assert_eq!(token.lexeme, "*");
    }

    #[test]
    fn test_slash_token() {
        let token = Token::new(TokenType::Slash, test_location(), "/".to_string());
        assert!(matches!(token.token_type, TokenType::Slash));
        assert_eq!(token.lexeme, "/");
    }

    #[test]
    fn test_percent_token() {
        let token = Token::new(TokenType::Percent, test_location(), "%".to_string());
        assert!(matches!(token.token_type, TokenType::Percent));
        assert_eq!(token.lexeme, "%");
    }

    #[test]
    fn test_equal_equal_token() {
        let token = Token::new(TokenType::EqualEqual, test_location(), "==".to_string());
        assert!(matches!(token.token_type, TokenType::EqualEqual));
        assert_eq!(token.lexeme, "==");
    }

    #[test]
    fn test_bang_equal_token() {
        let token = Token::new(TokenType::BangEqual, test_location(), "!=".to_string());
        assert!(matches!(token.token_type, TokenType::BangEqual));
        assert_eq!(token.lexeme, "!=");
    }

    #[test]
    fn test_less_token() {
        let token = Token::new(TokenType::Less, test_location(), "<".to_string());
        assert!(matches!(token.token_type, TokenType::Less));
        assert_eq!(token.lexeme, "<");
    }

    #[test]
    fn test_less_equal_token() {
        let token = Token::new(TokenType::LessEqual, test_location(), "<=".to_string());
        assert!(matches!(token.token_type, TokenType::LessEqual));
        assert_eq!(token.lexeme, "<=");
    }

    #[test]
    fn test_greater_token() {
        let token = Token::new(TokenType::Greater, test_location(), ">".to_string());
        assert!(matches!(token.token_type, TokenType::Greater));
        assert_eq!(token.lexeme, ">");
    }

    #[test]
    fn test_greater_equal_token() {
        let token = Token::new(TokenType::GreaterEqual, test_location(), ">=".to_string());
        assert!(matches!(token.token_type, TokenType::GreaterEqual));
        assert_eq!(token.lexeme, ">=");
    }

    #[test]
    fn test_amp_amp_token() {
        let token = Token::new(TokenType::AmpAmp, test_location(), "&&".to_string());
        assert!(matches!(token.token_type, TokenType::AmpAmp));
        assert_eq!(token.lexeme, "&&");
    }

    #[test]
    fn test_pipe_pipe_token() {
        let token = Token::new(TokenType::PipePipe, test_location(), "||".to_string());
        assert!(matches!(token.token_type, TokenType::PipePipe));
        assert_eq!(token.lexeme, "||");
    }

    #[test]
    fn test_bang_token() {
        let token = Token::new(TokenType::Bang, test_location(), "!".to_string());
        assert!(matches!(token.token_type, TokenType::Bang));
        assert_eq!(token.lexeme, "!");
    }

    #[test]
    fn test_equal_token() {
        let token = Token::new(TokenType::Equal, test_location(), "=".to_string());
        assert!(matches!(token.token_type, TokenType::Equal));
        assert_eq!(token.lexeme, "=");
    }

    // ==================== OWNERSHIP SIGIL TESTS ====================

    #[test]
    fn test_caret_token() {
        let token = Token::new(TokenType::Caret, test_location(), "^".to_string());
        assert!(matches!(token.token_type, TokenType::Caret));
        assert_eq!(token.lexeme, "^");
    }

    #[test]
    fn test_ampersand_token() {
        let token = Token::new(TokenType::Ampersand, test_location(), "&".to_string());
        assert!(matches!(token.token_type, TokenType::Ampersand));
        assert_eq!(token.lexeme, "&");
    }

    #[test]
    fn test_tilde_token() {
        let token = Token::new(TokenType::Tilde, test_location(), "~".to_string());
        assert!(matches!(token.token_type, TokenType::Tilde));
        assert_eq!(token.lexeme, "~");
    }

    // ==================== LITERAL TESTS ====================

    #[test]
    fn test_integer_literal_token() {
        let token = Token::new(
            TokenType::IntegerLiteral(42),
            test_location(),
            "42".to_string(),
        );
        assert!(matches!(token.token_type, TokenType::IntegerLiteral(42)));
        assert_eq!(token.lexeme, "42");
    }

    #[test]
    fn test_float_literal_token() {
        let token = Token::new(
            TokenType::FloatLiteral(3.14),
            test_location(),
            "3.14".to_string(),
        );
        if let TokenType::FloatLiteral(f) = token.token_type {
            assert!((f - 3.14).abs() < f64::EPSILON);
        } else {
            panic!("Expected FloatLiteral");
        }
        assert_eq!(token.lexeme, "3.14");
    }

    #[test]
    fn test_string_literal_token() {
        let token = Token::new(
            TokenType::StringLiteral("hello".to_string()),
            test_location(),
            "\"hello\"".to_string(),
        );
        assert!(matches!(token.token_type, TokenType::StringLiteral(ref s) if s == "hello"));
        assert_eq!(token.lexeme, "\"hello\"");
    }

    #[test]
    fn test_char_literal_token() {
        let token = Token::new(
            TokenType::CharLiteral('a'),
            test_location(),
            "'a'".to_string(),
        );
        assert!(matches!(token.token_type, TokenType::CharLiteral('a')));
        assert_eq!(token.lexeme, "'a'");
    }

    #[test]
    fn test_bool_literal_true_token() {
        let token = Token::new(
            TokenType::BoolLiteral(true),
            test_location(),
            "true".to_string(),
        );
        assert!(matches!(token.token_type, TokenType::BoolLiteral(true)));
        assert_eq!(token.lexeme, "true");
    }

    #[test]
    fn test_bool_literal_false_token() {
        let token = Token::new(
            TokenType::BoolLiteral(false),
            test_location(),
            "false".to_string(),
        );
        assert!(matches!(token.token_type, TokenType::BoolLiteral(false)));
        assert_eq!(token.lexeme, "false");
    }

    // ==================== IDENTIFIER TESTS ====================

    #[test]
    fn test_identifier_token() {
        let token = Token::new(
            TokenType::Identifier("myVar".to_string()),
            test_location(),
            "myVar".to_string(),
        );
        assert!(matches!(token.token_type, TokenType::Identifier(ref s) if s == "myVar"));
        assert_eq!(token.lexeme, "myVar");
    }

    // ==================== KEYWORD TESTS ====================

    #[test]
    fn test_keyword_module() {
        let token = Token::new(
            TokenType::Keyword(Keyword::Module),
            test_location(),
            "module".to_string(),
        );
        assert!(matches!(
            token.token_type,
            TokenType::Keyword(Keyword::Module)
        ));
    }

    #[test]
    fn test_keyword_func() {
        let token = Token::new(
            TokenType::Keyword(Keyword::Func),
            test_location(),
            "func".to_string(),
        );
        assert!(matches!(
            token.token_type,
            TokenType::Keyword(Keyword::Func)
        ));
    }

    #[test]
    fn test_keyword_let() {
        let token = Token::new(
            TokenType::Keyword(Keyword::Let),
            test_location(),
            "let".to_string(),
        );
        assert!(matches!(token.token_type, TokenType::Keyword(Keyword::Let)));
    }

    #[test]
    fn test_keyword_when() {
        let token = Token::new(
            TokenType::Keyword(Keyword::When),
            test_location(),
            "when".to_string(),
        );
        assert!(matches!(
            token.token_type,
            TokenType::Keyword(Keyword::When)
        ));
    }

    #[test]
    fn test_keyword_return() {
        let token = Token::new(
            TokenType::Keyword(Keyword::Return),
            test_location(),
            "return".to_string(),
        );
        assert!(matches!(
            token.token_type,
            TokenType::Keyword(Keyword::Return)
        ));
    }

    // ==================== EOF TEST ====================

    #[test]
    fn test_eof_token() {
        let token = Token::new(TokenType::Eof, test_location(), "".to_string());
        assert!(matches!(token.token_type, TokenType::Eof));
    }

    // ==================== TOKEN TYPE EQUALITY TESTS ====================

    #[test]
    fn test_token_type_equality() {
        assert_eq!(TokenType::LeftBrace, TokenType::LeftBrace);
        assert_ne!(TokenType::LeftBrace, TokenType::RightBrace);
        assert_eq!(TokenType::IntegerLiteral(42), TokenType::IntegerLiteral(42));
        assert_ne!(TokenType::IntegerLiteral(42), TokenType::IntegerLiteral(43));
        assert_eq!(
            TokenType::Keyword(Keyword::Func),
            TokenType::Keyword(Keyword::Func)
        );
        assert_ne!(
            TokenType::Keyword(Keyword::Func),
            TokenType::Keyword(Keyword::Let)
        );
    }

    // ==================== LEXER KEYWORD RECOGNITION TESTS ====================

    #[test]
    fn test_lexer_keyword_lookup_declaration() {
        let lexer = Lexer::new("", "test.aether".to_string());
        assert_eq!(lexer.lookup_keyword("module"), Some(&Keyword::Module));
        assert_eq!(lexer.lookup_keyword("import"), Some(&Keyword::Import));
        assert_eq!(lexer.lookup_keyword("func"), Some(&Keyword::Func));
        assert_eq!(lexer.lookup_keyword("let"), Some(&Keyword::Let));
        assert_eq!(lexer.lookup_keyword("const"), Some(&Keyword::Const));
        assert_eq!(lexer.lookup_keyword("struct"), Some(&Keyword::Struct));
        assert_eq!(lexer.lookup_keyword("enum"), Some(&Keyword::Enum));
    }

    #[test]
    fn test_lexer_keyword_lookup_modifiers() {
        let lexer = Lexer::new("", "test.aether".to_string());
        assert_eq!(lexer.lookup_keyword("mut"), Some(&Keyword::Mut));
        assert_eq!(lexer.lookup_keyword("pub"), Some(&Keyword::Pub));
    }

    #[test]
    fn test_lexer_keyword_lookup_control_flow() {
        let lexer = Lexer::new("", "test.aether".to_string());
        assert_eq!(lexer.lookup_keyword("when"), Some(&Keyword::When));
        assert_eq!(lexer.lookup_keyword("case"), Some(&Keyword::Case));
        assert_eq!(lexer.lookup_keyword("else"), Some(&Keyword::Else));
        assert_eq!(lexer.lookup_keyword("match"), Some(&Keyword::Match));
        assert_eq!(lexer.lookup_keyword("for"), Some(&Keyword::For));
        assert_eq!(lexer.lookup_keyword("while"), Some(&Keyword::While));
        assert_eq!(lexer.lookup_keyword("in"), Some(&Keyword::In));
        assert_eq!(lexer.lookup_keyword("return"), Some(&Keyword::Return));
        assert_eq!(lexer.lookup_keyword("break"), Some(&Keyword::Break));
        assert_eq!(lexer.lookup_keyword("continue"), Some(&Keyword::Continue));
    }

    #[test]
    fn test_lexer_keyword_lookup_error_handling() {
        let lexer = Lexer::new("", "test.aether".to_string());
        assert_eq!(lexer.lookup_keyword("try"), Some(&Keyword::Try));
        assert_eq!(lexer.lookup_keyword("catch"), Some(&Keyword::Catch));
        assert_eq!(lexer.lookup_keyword("finally"), Some(&Keyword::Finally));
        assert_eq!(lexer.lookup_keyword("throw"), Some(&Keyword::Throw));
    }

    #[test]
    fn test_lexer_keyword_lookup_resource() {
        let lexer = Lexer::new("", "test.aether".to_string());
        assert_eq!(lexer.lookup_keyword("resource"), Some(&Keyword::Resource));
        assert_eq!(lexer.lookup_keyword("cleanup"), Some(&Keyword::Cleanup));
        assert_eq!(lexer.lookup_keyword("guaranteed"), Some(&Keyword::Guaranteed));
    }

    #[test]
    fn test_lexer_keyword_lookup_types() {
        let lexer = Lexer::new("", "test.aether".to_string());
        assert_eq!(lexer.lookup_keyword("Int"), Some(&Keyword::Int));
        assert_eq!(lexer.lookup_keyword("Int64"), Some(&Keyword::Int64));
        assert_eq!(lexer.lookup_keyword("Float"), Some(&Keyword::Float));
        assert_eq!(lexer.lookup_keyword("String"), Some(&Keyword::String_));
        assert_eq!(lexer.lookup_keyword("Bool"), Some(&Keyword::Bool));
        assert_eq!(lexer.lookup_keyword("Void"), Some(&Keyword::Void));
        assert_eq!(lexer.lookup_keyword("Array"), Some(&Keyword::Array));
        assert_eq!(lexer.lookup_keyword("Map"), Some(&Keyword::Map));
        assert_eq!(lexer.lookup_keyword("Pointer"), Some(&Keyword::Pointer));
        assert_eq!(lexer.lookup_keyword("MutPointer"), Some(&Keyword::MutPointer));
        assert_eq!(lexer.lookup_keyword("SizeT"), Some(&Keyword::SizeT));
    }

    #[test]
    fn test_lexer_keyword_lookup_literals() {
        let lexer = Lexer::new("", "test.aether".to_string());
        assert_eq!(lexer.lookup_keyword("true"), Some(&Keyword::True));
        assert_eq!(lexer.lookup_keyword("false"), Some(&Keyword::False));
        assert_eq!(lexer.lookup_keyword("nil"), Some(&Keyword::Nil));
    }

    #[test]
    fn test_lexer_keyword_lookup_other() {
        let lexer = Lexer::new("", "test.aether".to_string());
        assert_eq!(lexer.lookup_keyword("as"), Some(&Keyword::As));
        assert_eq!(lexer.lookup_keyword("range"), Some(&Keyword::Range));
    }

    #[test]
    fn test_lexer_keyword_lookup_not_keyword() {
        let lexer = Lexer::new("", "test.aether".to_string());
        assert_eq!(lexer.lookup_keyword("myVar"), None);
        assert_eq!(lexer.lookup_keyword("foo"), None);
        assert_eq!(lexer.lookup_keyword("bar123"), None);
    }

    #[test]
    fn test_lexer_tokenize_single_keyword() {
        let mut lexer = Lexer::new("func", "test.aether".to_string());
        let tokens = lexer.tokenize().unwrap();

        assert_eq!(tokens.len(), 2); // func + EOF
        assert!(matches!(tokens[0].token_type, TokenType::Keyword(Keyword::Func)));
        assert_eq!(tokens[0].lexeme, "func");
        assert!(matches!(tokens[1].token_type, TokenType::Eof));
    }

    #[test]
    fn test_lexer_tokenize_multiple_keywords() {
        let mut lexer = Lexer::new("func let return", "test.aether".to_string());
        let tokens = lexer.tokenize().unwrap();

        assert_eq!(tokens.len(), 4); // func let return EOF
        assert!(matches!(tokens[0].token_type, TokenType::Keyword(Keyword::Func)));
        assert!(matches!(tokens[1].token_type, TokenType::Keyword(Keyword::Let)));
        assert!(matches!(tokens[2].token_type, TokenType::Keyword(Keyword::Return)));
        assert!(matches!(tokens[3].token_type, TokenType::Eof));
    }

    #[test]
    fn test_lexer_tokenize_keywords_and_identifiers() {
        let mut lexer = Lexer::new("func myFunction let x", "test.aether".to_string());
        let tokens = lexer.tokenize().unwrap();

        assert_eq!(tokens.len(), 5); // func myFunction let x EOF
        assert!(matches!(tokens[0].token_type, TokenType::Keyword(Keyword::Func)));
        assert!(matches!(tokens[1].token_type, TokenType::Identifier(ref s) if s == "myFunction"));
        assert!(matches!(tokens[2].token_type, TokenType::Keyword(Keyword::Let)));
        assert!(matches!(tokens[3].token_type, TokenType::Identifier(ref s) if s == "x"));
        assert!(matches!(tokens[4].token_type, TokenType::Eof));
    }

    #[test]
    fn test_lexer_tokenize_true_false() {
        let mut lexer = Lexer::new("true false", "test.aether".to_string());
        let tokens = lexer.tokenize().unwrap();

        assert_eq!(tokens.len(), 3); // true false EOF
        assert!(matches!(tokens[0].token_type, TokenType::BoolLiteral(true)));
        assert!(matches!(tokens[1].token_type, TokenType::BoolLiteral(false)));
    }

    #[test]
    fn test_lexer_tokenize_type_keywords() {
        let mut lexer = Lexer::new("Int String Bool Void", "test.aether".to_string());
        let tokens = lexer.tokenize().unwrap();

        assert_eq!(tokens.len(), 5); // Int String Bool Void EOF
        assert!(matches!(tokens[0].token_type, TokenType::Keyword(Keyword::Int)));
        assert!(matches!(tokens[1].token_type, TokenType::Keyword(Keyword::String_)));
        assert!(matches!(tokens[2].token_type, TokenType::Keyword(Keyword::Bool)));
        assert!(matches!(tokens[3].token_type, TokenType::Keyword(Keyword::Void)));
    }

    #[test]
    fn test_lexer_tokenize_control_flow_keywords() {
        let mut lexer = Lexer::new("when case else match for while in", "test.aether".to_string());
        let tokens = lexer.tokenize().unwrap();

        assert_eq!(tokens.len(), 8); // 7 keywords + EOF
        assert!(matches!(tokens[0].token_type, TokenType::Keyword(Keyword::When)));
        assert!(matches!(tokens[1].token_type, TokenType::Keyword(Keyword::Case)));
        assert!(matches!(tokens[2].token_type, TokenType::Keyword(Keyword::Else)));
        assert!(matches!(tokens[3].token_type, TokenType::Keyword(Keyword::Match)));
        assert!(matches!(tokens[4].token_type, TokenType::Keyword(Keyword::For)));
        assert!(matches!(tokens[5].token_type, TokenType::Keyword(Keyword::While)));
        assert!(matches!(tokens[6].token_type, TokenType::Keyword(Keyword::In)));
    }

    #[test]
    fn test_lexer_tokenize_identifier_with_underscore() {
        let mut lexer = Lexer::new("my_var _private __dunder__", "test.aether".to_string());
        let tokens = lexer.tokenize().unwrap();

        assert_eq!(tokens.len(), 4);
        assert!(matches!(tokens[0].token_type, TokenType::Identifier(ref s) if s == "my_var"));
        assert!(matches!(tokens[1].token_type, TokenType::Identifier(ref s) if s == "_private"));
        assert!(matches!(tokens[2].token_type, TokenType::Identifier(ref s) if s == "__dunder__"));
    }

    #[test]
    fn test_lexer_tokenize_with_newlines() {
        let mut lexer = Lexer::new("func\nlet\nreturn", "test.aether".to_string());
        let tokens = lexer.tokenize().unwrap();

        assert_eq!(tokens.len(), 4);
        assert!(matches!(tokens[0].token_type, TokenType::Keyword(Keyword::Func)));
        assert_eq!(tokens[0].location.line, 1);
        assert!(matches!(tokens[1].token_type, TokenType::Keyword(Keyword::Let)));
        assert_eq!(tokens[1].location.line, 2);
        assert!(matches!(tokens[2].token_type, TokenType::Keyword(Keyword::Return)));
        assert_eq!(tokens[2].location.line, 3);
    }

    #[test]
    fn test_lexer_all_keywords_count() {
        let lexer = Lexer::new("", "test.aether".to_string());
        // Count total keywords: 7 decl + 2 mod + 10 control + 4 error + 3 resource + 11 types + 3 literals + 2 other = 42
        assert_eq!(lexer.keywords.len(), 42);
    }

    // ==================== LITERAL TOKENIZATION TESTS ====================

    #[test]
    fn test_lexer_tokenize_integer_literal() {
        let mut lexer = Lexer::new("42", "test.aether".to_string());
        let tokens = lexer.tokenize().unwrap();

        assert_eq!(tokens.len(), 2); // 42 + EOF
        assert!(matches!(tokens[0].token_type, TokenType::IntegerLiteral(42)));
        assert_eq!(tokens[0].lexeme, "42");
    }

    #[test]
    fn test_lexer_tokenize_integer_zero() {
        let mut lexer = Lexer::new("0", "test.aether".to_string());
        let tokens = lexer.tokenize().unwrap();

        assert!(matches!(tokens[0].token_type, TokenType::IntegerLiteral(0)));
    }

    #[test]
    fn test_lexer_tokenize_large_integer() {
        let mut lexer = Lexer::new("1000000", "test.aether".to_string());
        let tokens = lexer.tokenize().unwrap();

        assert!(matches!(tokens[0].token_type, TokenType::IntegerLiteral(1000000)));
    }

    #[test]
    fn test_lexer_tokenize_float_literal() {
        let mut lexer = Lexer::new("3.14", "test.aether".to_string());
        let tokens = lexer.tokenize().unwrap();

        assert_eq!(tokens.len(), 2);
        if let TokenType::FloatLiteral(f) = tokens[0].token_type {
            assert!((f - 3.14).abs() < f64::EPSILON);
        } else {
            panic!("Expected FloatLiteral");
        }
        assert_eq!(tokens[0].lexeme, "3.14");
    }

    #[test]
    fn test_lexer_tokenize_float_leading_zero() {
        let mut lexer = Lexer::new("0.5", "test.aether".to_string());
        let tokens = lexer.tokenize().unwrap();

        if let TokenType::FloatLiteral(f) = tokens[0].token_type {
            assert!((f - 0.5).abs() < f64::EPSILON);
        } else {
            panic!("Expected FloatLiteral");
        }
    }

    #[test]
    fn test_lexer_tokenize_multiple_numbers() {
        let mut lexer = Lexer::new("42 3.14 100", "test.aether".to_string());
        let tokens = lexer.tokenize().unwrap();

        assert_eq!(tokens.len(), 4);
        assert!(matches!(tokens[0].token_type, TokenType::IntegerLiteral(42)));
        if let TokenType::FloatLiteral(f) = tokens[1].token_type {
            assert!((f - 3.14).abs() < f64::EPSILON);
        } else {
            panic!("Expected FloatLiteral");
        }
        assert!(matches!(tokens[2].token_type, TokenType::IntegerLiteral(100)));
    }

    #[test]
    fn test_lexer_tokenize_string_simple() {
        let mut lexer = Lexer::new("\"hello\"", "test.aether".to_string());
        let tokens = lexer.tokenize().unwrap();

        assert_eq!(tokens.len(), 2);
        assert!(matches!(tokens[0].token_type, TokenType::StringLiteral(ref s) if s == "hello"));
        assert_eq!(tokens[0].lexeme, "\"hello\"");
    }

    #[test]
    fn test_lexer_tokenize_string_with_spaces() {
        let mut lexer = Lexer::new("\"hello world\"", "test.aether".to_string());
        let tokens = lexer.tokenize().unwrap();

        assert!(matches!(tokens[0].token_type, TokenType::StringLiteral(ref s) if s == "hello world"));
    }

    #[test]
    fn test_lexer_tokenize_string_empty() {
        let mut lexer = Lexer::new("\"\"", "test.aether".to_string());
        let tokens = lexer.tokenize().unwrap();

        assert!(matches!(tokens[0].token_type, TokenType::StringLiteral(ref s) if s.is_empty()));
    }

    #[test]
    fn test_lexer_tokenize_string_escape_newline() {
        let mut lexer = Lexer::new("\"hello\\nworld\"", "test.aether".to_string());
        let tokens = lexer.tokenize().unwrap();

        assert!(matches!(tokens[0].token_type, TokenType::StringLiteral(ref s) if s == "hello\nworld"));
    }

    #[test]
    fn test_lexer_tokenize_string_escape_tab() {
        let mut lexer = Lexer::new("\"hello\\tworld\"", "test.aether".to_string());
        let tokens = lexer.tokenize().unwrap();

        assert!(matches!(tokens[0].token_type, TokenType::StringLiteral(ref s) if s == "hello\tworld"));
    }

    #[test]
    fn test_lexer_tokenize_string_escape_quote() {
        let mut lexer = Lexer::new("\"say \\\"hi\\\"\"", "test.aether".to_string());
        let tokens = lexer.tokenize().unwrap();

        assert!(matches!(tokens[0].token_type, TokenType::StringLiteral(ref s) if s == "say \"hi\""));
    }

    #[test]
    fn test_lexer_tokenize_string_escape_backslash() {
        let mut lexer = Lexer::new("\"path\\\\file\"", "test.aether".to_string());
        let tokens = lexer.tokenize().unwrap();

        assert!(matches!(tokens[0].token_type, TokenType::StringLiteral(ref s) if s == "path\\file"));
    }

    #[test]
    fn test_lexer_tokenize_char_simple() {
        let mut lexer = Lexer::new("'a'", "test.aether".to_string());
        let tokens = lexer.tokenize().unwrap();

        assert_eq!(tokens.len(), 2);
        assert!(matches!(tokens[0].token_type, TokenType::CharLiteral('a')));
        assert_eq!(tokens[0].lexeme, "'a'");
    }

    #[test]
    fn test_lexer_tokenize_char_escape_newline() {
        let mut lexer = Lexer::new("'\\n'", "test.aether".to_string());
        let tokens = lexer.tokenize().unwrap();

        assert!(matches!(tokens[0].token_type, TokenType::CharLiteral('\n')));
    }

    #[test]
    fn test_lexer_tokenize_char_escape_tab() {
        let mut lexer = Lexer::new("'\\t'", "test.aether".to_string());
        let tokens = lexer.tokenize().unwrap();

        assert!(matches!(tokens[0].token_type, TokenType::CharLiteral('\t')));
    }

    #[test]
    fn test_lexer_tokenize_char_escape_single_quote() {
        let mut lexer = Lexer::new("'\\''", "test.aether".to_string());
        let tokens = lexer.tokenize().unwrap();

        assert!(matches!(tokens[0].token_type, TokenType::CharLiteral('\'')));
    }

    #[test]
    fn test_lexer_tokenize_char_escape_backslash() {
        let mut lexer = Lexer::new("'\\\\'", "test.aether".to_string());
        let tokens = lexer.tokenize().unwrap();

        assert!(matches!(tokens[0].token_type, TokenType::CharLiteral('\\')));
    }

    #[test]
    fn test_lexer_tokenize_mixed_literals() {
        let mut lexer = Lexer::new("42 \"hello\" 'x' 3.14", "test.aether".to_string());
        let tokens = lexer.tokenize().unwrap();

        assert_eq!(tokens.len(), 5);
        assert!(matches!(tokens[0].token_type, TokenType::IntegerLiteral(42)));
        assert!(matches!(tokens[1].token_type, TokenType::StringLiteral(ref s) if s == "hello"));
        assert!(matches!(tokens[2].token_type, TokenType::CharLiteral('x')));
        if let TokenType::FloatLiteral(f) = tokens[3].token_type {
            assert!((f - 3.14).abs() < f64::EPSILON);
        } else {
            panic!("Expected FloatLiteral");
        }
    }

    #[test]
    fn test_lexer_tokenize_literals_with_keywords() {
        let mut lexer = Lexer::new("let x 42 \"hello\"", "test.aether".to_string());
        let tokens = lexer.tokenize().unwrap();

        assert_eq!(tokens.len(), 5); // let x 42 "hello" EOF
        assert!(matches!(tokens[0].token_type, TokenType::Keyword(Keyword::Let)));
        assert!(matches!(tokens[1].token_type, TokenType::Identifier(ref s) if s == "x"));
        assert!(matches!(tokens[2].token_type, TokenType::IntegerLiteral(42)));
        assert!(matches!(tokens[3].token_type, TokenType::StringLiteral(ref s) if s == "hello"));
    }

    #[test]
    fn test_lexer_error_unterminated_string() {
        let mut lexer = Lexer::new("\"unterminated", "test.aether".to_string());
        let result = lexer.tokenize();

        assert!(matches!(result, Err(LexerError::UnterminatedString { .. })));
    }

    #[test]
    fn test_lexer_error_unterminated_string_newline() {
        let mut lexer = Lexer::new("\"hello\nworld\"", "test.aether".to_string());
        let result = lexer.tokenize();

        assert!(matches!(result, Err(LexerError::UnterminatedString { .. })));
    }

    #[test]
    fn test_lexer_error_invalid_escape_sequence() {
        let mut lexer = Lexer::new("\"\\x\"", "test.aether".to_string());
        let result = lexer.tokenize();

        assert!(matches!(result, Err(LexerError::InvalidEscapeSequence { .. })));
    }

    #[test]
    fn test_lexer_error_unterminated_char() {
        let mut lexer = Lexer::new("'a", "test.aether".to_string());
        let result = lexer.tokenize();

        assert!(matches!(result, Err(LexerError::UnterminatedString { .. })));
    }

    // ==================== OPERATOR TOKENIZATION TESTS ====================

    #[test]
    fn test_lexer_tokenize_delimiters() {
        let mut lexer = Lexer::new("{ } [ ] ( ) ; : , . @", "test.aether".to_string());
        let tokens = lexer.tokenize().unwrap();

        assert_eq!(tokens.len(), 12); // 11 delimiters + EOF
        assert!(matches!(tokens[0].token_type, TokenType::LeftBrace));
        assert!(matches!(tokens[1].token_type, TokenType::RightBrace));
        assert!(matches!(tokens[2].token_type, TokenType::LeftBracket));
        assert!(matches!(tokens[3].token_type, TokenType::RightBracket));
        assert!(matches!(tokens[4].token_type, TokenType::LeftParen));
        assert!(matches!(tokens[5].token_type, TokenType::RightParen));
        assert!(matches!(tokens[6].token_type, TokenType::Semicolon));
        assert!(matches!(tokens[7].token_type, TokenType::Colon));
        assert!(matches!(tokens[8].token_type, TokenType::Comma));
        assert!(matches!(tokens[9].token_type, TokenType::Dot));
        assert!(matches!(tokens[10].token_type, TokenType::At));
    }

    #[test]
    fn test_lexer_tokenize_arithmetic_operators() {
        let mut lexer = Lexer::new("+ - * / %", "test.aether".to_string());
        let tokens = lexer.tokenize().unwrap();

        assert_eq!(tokens.len(), 6); // 5 operators + EOF
        assert!(matches!(tokens[0].token_type, TokenType::Plus));
        assert!(matches!(tokens[1].token_type, TokenType::Minus));
        assert!(matches!(tokens[2].token_type, TokenType::Star));
        assert!(matches!(tokens[3].token_type, TokenType::Slash));
        assert!(matches!(tokens[4].token_type, TokenType::Percent));
    }

    #[test]
    fn test_lexer_tokenize_comparison_operators() {
        let mut lexer = Lexer::new("== != < <= > >=", "test.aether".to_string());
        let tokens = lexer.tokenize().unwrap();

        assert_eq!(tokens.len(), 7); // 6 operators + EOF
        assert!(matches!(tokens[0].token_type, TokenType::EqualEqual));
        assert!(matches!(tokens[1].token_type, TokenType::BangEqual));
        assert!(matches!(tokens[2].token_type, TokenType::Less));
        assert!(matches!(tokens[3].token_type, TokenType::LessEqual));
        assert!(matches!(tokens[4].token_type, TokenType::Greater));
        assert!(matches!(tokens[5].token_type, TokenType::GreaterEqual));
    }

    #[test]
    fn test_lexer_tokenize_logical_operators() {
        let mut lexer = Lexer::new("&& || !", "test.aether".to_string());
        let tokens = lexer.tokenize().unwrap();

        assert_eq!(tokens.len(), 4); // 3 operators + EOF
        assert!(matches!(tokens[0].token_type, TokenType::AmpAmp));
        assert!(matches!(tokens[1].token_type, TokenType::PipePipe));
        assert!(matches!(tokens[2].token_type, TokenType::Bang));
    }

    #[test]
    fn test_lexer_tokenize_assignment_operator() {
        let mut lexer = Lexer::new("=", "test.aether".to_string());
        let tokens = lexer.tokenize().unwrap();

        assert_eq!(tokens.len(), 2);
        assert!(matches!(tokens[0].token_type, TokenType::Equal));
        assert_eq!(tokens[0].lexeme, "=");
    }

    #[test]
    fn test_lexer_tokenize_arrow_operator() {
        let mut lexer = Lexer::new("->", "test.aether".to_string());
        let tokens = lexer.tokenize().unwrap();

        assert_eq!(tokens.len(), 2);
        assert!(matches!(tokens[0].token_type, TokenType::Arrow));
        assert_eq!(tokens[0].lexeme, "->");
    }

    #[test]
    fn test_lexer_tokenize_ownership_sigils() {
        let mut lexer = Lexer::new("^ & ~", "test.aether".to_string());
        let tokens = lexer.tokenize().unwrap();

        assert_eq!(tokens.len(), 4); // 3 sigils + EOF
        assert!(matches!(tokens[0].token_type, TokenType::Caret));
        assert!(matches!(tokens[1].token_type, TokenType::Ampersand));
        assert!(matches!(tokens[2].token_type, TokenType::Tilde));
    }

    #[test]
    fn test_lexer_disambiguate_equal_vs_equalequal() {
        let mut lexer = Lexer::new("= == =", "test.aether".to_string());
        let tokens = lexer.tokenize().unwrap();

        assert_eq!(tokens.len(), 4);
        assert!(matches!(tokens[0].token_type, TokenType::Equal));
        assert!(matches!(tokens[1].token_type, TokenType::EqualEqual));
        assert!(matches!(tokens[2].token_type, TokenType::Equal));
    }

    #[test]
    fn test_lexer_disambiguate_less_vs_lessequal() {
        let mut lexer = Lexer::new("< <= <", "test.aether".to_string());
        let tokens = lexer.tokenize().unwrap();

        assert_eq!(tokens.len(), 4);
        assert!(matches!(tokens[0].token_type, TokenType::Less));
        assert!(matches!(tokens[1].token_type, TokenType::LessEqual));
        assert!(matches!(tokens[2].token_type, TokenType::Less));
    }

    #[test]
    fn test_lexer_disambiguate_greater_vs_greaterequal() {
        let mut lexer = Lexer::new("> >= >", "test.aether".to_string());
        let tokens = lexer.tokenize().unwrap();

        assert_eq!(tokens.len(), 4);
        assert!(matches!(tokens[0].token_type, TokenType::Greater));
        assert!(matches!(tokens[1].token_type, TokenType::GreaterEqual));
        assert!(matches!(tokens[2].token_type, TokenType::Greater));
    }

    #[test]
    fn test_lexer_disambiguate_minus_vs_arrow() {
        let mut lexer = Lexer::new("- -> -", "test.aether".to_string());
        let tokens = lexer.tokenize().unwrap();

        assert_eq!(tokens.len(), 4);
        assert!(matches!(tokens[0].token_type, TokenType::Minus));
        assert!(matches!(tokens[1].token_type, TokenType::Arrow));
        assert!(matches!(tokens[2].token_type, TokenType::Minus));
    }

    #[test]
    fn test_lexer_disambiguate_ampersand_vs_ampamp() {
        let mut lexer = Lexer::new("& && &", "test.aether".to_string());
        let tokens = lexer.tokenize().unwrap();

        assert_eq!(tokens.len(), 4);
        assert!(matches!(tokens[0].token_type, TokenType::Ampersand));
        assert!(matches!(tokens[1].token_type, TokenType::AmpAmp));
        assert!(matches!(tokens[2].token_type, TokenType::Ampersand));
    }

    #[test]
    fn test_lexer_disambiguate_bang_vs_bangequal() {
        let mut lexer = Lexer::new("! != !", "test.aether".to_string());
        let tokens = lexer.tokenize().unwrap();

        assert_eq!(tokens.len(), 4);
        assert!(matches!(tokens[0].token_type, TokenType::Bang));
        assert!(matches!(tokens[1].token_type, TokenType::BangEqual));
        assert!(matches!(tokens[2].token_type, TokenType::Bang));
    }

    #[test]
    fn test_lexer_tokenize_function_signature() {
        let mut lexer = Lexer::new("func foo(x: Int) -> Int", "test.aether".to_string());
        let tokens = lexer.tokenize().unwrap();

        assert_eq!(tokens.len(), 10); // func foo ( x : Int ) -> Int EOF
        assert!(matches!(tokens[0].token_type, TokenType::Keyword(Keyword::Func)));
        assert!(matches!(tokens[1].token_type, TokenType::Identifier(ref s) if s == "foo"));
        assert!(matches!(tokens[2].token_type, TokenType::LeftParen));
        assert!(matches!(tokens[3].token_type, TokenType::Identifier(ref s) if s == "x"));
        assert!(matches!(tokens[4].token_type, TokenType::Colon));
        assert!(matches!(tokens[5].token_type, TokenType::Keyword(Keyword::Int)));
        assert!(matches!(tokens[6].token_type, TokenType::RightParen));
        assert!(matches!(tokens[7].token_type, TokenType::Arrow));
        assert!(matches!(tokens[8].token_type, TokenType::Keyword(Keyword::Int)));
    }

    #[test]
    fn test_lexer_tokenize_braced_expression() {
        let mut lexer = Lexer::new("{a + b}", "test.aether".to_string());
        let tokens = lexer.tokenize().unwrap();

        assert_eq!(tokens.len(), 6); // { a + b } EOF
        assert!(matches!(tokens[0].token_type, TokenType::LeftBrace));
        assert!(matches!(tokens[1].token_type, TokenType::Identifier(ref s) if s == "a"));
        assert!(matches!(tokens[2].token_type, TokenType::Plus));
        assert!(matches!(tokens[3].token_type, TokenType::Identifier(ref s) if s == "b"));
        assert!(matches!(tokens[4].token_type, TokenType::RightBrace));
    }

    #[test]
    fn test_lexer_tokenize_variable_declaration() {
        let mut lexer = Lexer::new("let x: Int = 42;", "test.aether".to_string());
        let tokens = lexer.tokenize().unwrap();

        assert_eq!(tokens.len(), 8); // let x : Int = 42 ; EOF
        assert!(matches!(tokens[0].token_type, TokenType::Keyword(Keyword::Let)));
        assert!(matches!(tokens[1].token_type, TokenType::Identifier(ref s) if s == "x"));
        assert!(matches!(tokens[2].token_type, TokenType::Colon));
        assert!(matches!(tokens[3].token_type, TokenType::Keyword(Keyword::Int)));
        assert!(matches!(tokens[4].token_type, TokenType::Equal));
        assert!(matches!(tokens[5].token_type, TokenType::IntegerLiteral(42)));
        assert!(matches!(tokens[6].token_type, TokenType::Semicolon));
    }

    #[test]
    fn test_lexer_tokenize_comparison_expression() {
        let mut lexer = Lexer::new("{x > 0}", "test.aether".to_string());
        let tokens = lexer.tokenize().unwrap();

        assert_eq!(tokens.len(), 6); // { x > 0 } EOF
        assert!(matches!(tokens[0].token_type, TokenType::LeftBrace));
        assert!(matches!(tokens[1].token_type, TokenType::Identifier(ref s) if s == "x"));
        assert!(matches!(tokens[2].token_type, TokenType::Greater));
        assert!(matches!(tokens[3].token_type, TokenType::IntegerLiteral(0)));
        assert!(matches!(tokens[4].token_type, TokenType::RightBrace));
    }

    #[test]
    fn test_lexer_tokenize_annotation() {
        let mut lexer = Lexer::new("@requires({n > 0})", "test.aether".to_string());
        let tokens = lexer.tokenize().unwrap();

        assert_eq!(tokens.len(), 10); // @ requires ( { n > 0 } ) EOF
        assert!(matches!(tokens[0].token_type, TokenType::At));
        assert!(matches!(tokens[1].token_type, TokenType::Identifier(ref s) if s == "requires"));
        assert!(matches!(tokens[2].token_type, TokenType::LeftParen));
        assert!(matches!(tokens[3].token_type, TokenType::LeftBrace));
        assert!(matches!(tokens[4].token_type, TokenType::Identifier(ref s) if s == "n"));
        assert!(matches!(tokens[5].token_type, TokenType::Greater));
        assert!(matches!(tokens[6].token_type, TokenType::IntegerLiteral(0)));
        assert!(matches!(tokens[7].token_type, TokenType::RightBrace));
        assert!(matches!(tokens[8].token_type, TokenType::RightParen));
        assert!(matches!(tokens[9].token_type, TokenType::Eof));
    }

    #[test]
    fn test_lexer_tokenize_generic_type() {
        let mut lexer = Lexer::new("Array<Int>", "test.aether".to_string());
        let tokens = lexer.tokenize().unwrap();

        assert_eq!(tokens.len(), 5); // Array < Int > EOF
        assert!(matches!(tokens[0].token_type, TokenType::Keyword(Keyword::Array)));
        assert!(matches!(tokens[1].token_type, TokenType::Less));
        assert!(matches!(tokens[2].token_type, TokenType::Keyword(Keyword::Int)));
        assert!(matches!(tokens[3].token_type, TokenType::Greater));
    }

    #[test]
    fn test_lexer_tokenize_complex_expression() {
        let mut lexer = Lexer::new("{{a * b} + {c / 2}}", "test.aether".to_string());
        let tokens = lexer.tokenize().unwrap();

        // { { a * b } + { c / 2 } } EOF = 14 tokens
        assert_eq!(tokens.len(), 14);
        assert!(matches!(tokens[0].token_type, TokenType::LeftBrace));
        assert!(matches!(tokens[1].token_type, TokenType::LeftBrace));
        assert!(matches!(tokens[2].token_type, TokenType::Identifier(ref s) if s == "a"));
        assert!(matches!(tokens[3].token_type, TokenType::Star));
        assert!(matches!(tokens[4].token_type, TokenType::Identifier(ref s) if s == "b"));
        assert!(matches!(tokens[5].token_type, TokenType::RightBrace));
        assert!(matches!(tokens[6].token_type, TokenType::Plus));
    }

    // ==================== COMMENT TOKENIZATION TESTS ====================

    #[test]
    fn test_lexer_skip_line_comment() {
        let mut lexer = Lexer::new("// this is a comment", "test.aether".to_string());
        let tokens = lexer.tokenize().unwrap();

        assert_eq!(tokens.len(), 1); // Just EOF
        assert!(matches!(tokens[0].token_type, TokenType::Eof));
    }

    #[test]
    fn test_lexer_skip_line_comment_with_code_before() {
        let mut lexer = Lexer::new("let x // comment", "test.aether".to_string());
        let tokens = lexer.tokenize().unwrap();

        assert_eq!(tokens.len(), 3); // let x EOF
        assert!(matches!(tokens[0].token_type, TokenType::Keyword(Keyword::Let)));
        assert!(matches!(tokens[1].token_type, TokenType::Identifier(ref s) if s == "x"));
        assert!(matches!(tokens[2].token_type, TokenType::Eof));
    }

    #[test]
    fn test_lexer_skip_line_comment_with_code_after() {
        let mut lexer = Lexer::new("// comment\nlet x", "test.aether".to_string());
        let tokens = lexer.tokenize().unwrap();

        assert_eq!(tokens.len(), 3); // let x EOF
        assert!(matches!(tokens[0].token_type, TokenType::Keyword(Keyword::Let)));
        assert!(matches!(tokens[1].token_type, TokenType::Identifier(ref s) if s == "x"));
    }

    #[test]
    fn test_lexer_skip_multiple_line_comments() {
        let mut lexer = Lexer::new("// comment 1\n// comment 2\nlet x", "test.aether".to_string());
        let tokens = lexer.tokenize().unwrap();

        assert_eq!(tokens.len(), 3); // let x EOF
        assert!(matches!(tokens[0].token_type, TokenType::Keyword(Keyword::Let)));
    }

    #[test]
    fn test_lexer_skip_doc_comment() {
        let mut lexer = Lexer::new("/// this is a doc comment", "test.aether".to_string());
        let tokens = lexer.tokenize().unwrap();

        assert_eq!(tokens.len(), 1); // Just EOF
        assert!(matches!(tokens[0].token_type, TokenType::Eof));
    }

    #[test]
    fn test_lexer_skip_doc_comment_with_function() {
        let mut lexer = Lexer::new("/// Adds two numbers\nfunc add", "test.aether".to_string());
        let tokens = lexer.tokenize().unwrap();

        assert_eq!(tokens.len(), 3); // func add EOF
        assert!(matches!(tokens[0].token_type, TokenType::Keyword(Keyword::Func)));
        assert!(matches!(tokens[1].token_type, TokenType::Identifier(ref s) if s == "add"));
    }

    #[test]
    fn test_lexer_slash_not_comment() {
        // Single slash should be Slash operator, not comment
        let mut lexer = Lexer::new("a / b", "test.aether".to_string());
        let tokens = lexer.tokenize().unwrap();

        assert_eq!(tokens.len(), 4); // a / b EOF
        assert!(matches!(tokens[0].token_type, TokenType::Identifier(ref s) if s == "a"));
        assert!(matches!(tokens[1].token_type, TokenType::Slash));
        assert!(matches!(tokens[2].token_type, TokenType::Identifier(ref s) if s == "b"));
    }

    #[test]
    fn test_lexer_comment_does_not_break_lines() {
        let mut lexer = Lexer::new("let x = 1; // assign\nlet y = 2;", "test.aether".to_string());
        let tokens = lexer.tokenize().unwrap();

        // let x = 1 ; let y = 2 ; EOF = 11 tokens
        assert_eq!(tokens.len(), 11);
        assert!(matches!(tokens[0].token_type, TokenType::Keyword(Keyword::Let)));
        assert!(matches!(tokens[5].token_type, TokenType::Keyword(Keyword::Let)));
    }

    #[test]
    fn test_lexer_comment_with_special_chars() {
        let mut lexer = Lexer::new("// comment with special chars: @#$%^&*()!", "test.aether".to_string());
        let tokens = lexer.tokenize().unwrap();

        assert_eq!(tokens.len(), 1); // Just EOF
    }

    #[test]
    fn test_lexer_empty_comment() {
        let mut lexer = Lexer::new("//\nlet x", "test.aether".to_string());
        let tokens = lexer.tokenize().unwrap();

        assert_eq!(tokens.len(), 3); // let x EOF
        assert!(matches!(tokens[0].token_type, TokenType::Keyword(Keyword::Let)));
    }

    #[test]
    fn test_lexer_comment_at_end_of_file() {
        let mut lexer = Lexer::new("let x // no newline at end", "test.aether".to_string());
        let tokens = lexer.tokenize().unwrap();

        assert_eq!(tokens.len(), 3); // let x EOF
        assert!(matches!(tokens[0].token_type, TokenType::Keyword(Keyword::Let)));
        assert!(matches!(tokens[1].token_type, TokenType::Identifier(ref s) if s == "x"));
    }
}
