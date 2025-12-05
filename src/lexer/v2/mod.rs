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
    Semicolon,   // ;
    Colon,       // :
    DoubleColon, // ::
    Comma,       // ,
    Dot,         // .
    DotDot,      // ..
    DotDotEqual, // ..=
    Arrow,       // ->
    FatArrow,    // =>
    At,          // @
    Underscore,  // _

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
    Var,
    Const,
    Struct,
    Enum,
    Type,

    // Modifiers
    Mut,
    Pub,

    // Control Flow
    When,
    If,
    Case,
    Else,
    Match,
    For,
    While,
    In,
    Return,
    Break,
    Continue,
    Concurrent, // New async/await alternative
    Async,      // Future async/await support
    Await,      // Future async/await support

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
    Int32,
    Int64,
    Float,
    Float32,
    Float64,
    String_,
    Char,
    Bool,
    Void,
    Array,
    Map,
    Pointer,
    MutPointer,
    SizeT,
    // New fixed-width types
    UInt8,
    Int8,
    UInt16,
    Int16,
    UInt32,
    UInt64,

    // Literals
    True,
    False,
    Nil,

    // Generics
    Where,
    Trait,
    Impl,

    // Quantifiers (for axioms)
    ForAll,
    Exists,

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
        self.keywords.insert("var".to_string(), Keyword::Var);
        self.keywords.insert("const".to_string(), Keyword::Const);
        self.keywords.insert("struct".to_string(), Keyword::Struct);
        self.keywords.insert("enum".to_string(), Keyword::Enum);
        self.keywords.insert("type".to_string(), Keyword::Type);

        // Modifier keywords
        self.keywords.insert("mut".to_string(), Keyword::Mut);
        self.keywords.insert("pub".to_string(), Keyword::Pub);

        // Control flow keywords
        self.keywords.insert("when".to_string(), Keyword::When);
        self.keywords.insert("if".to_string(), Keyword::If);
        self.keywords.insert("case".to_string(), Keyword::Case);
        self.keywords.insert("else".to_string(), Keyword::Else);
        self.keywords.insert("match".to_string(), Keyword::Match);
        self.keywords.insert("for".to_string(), Keyword::For);
        self.keywords.insert("while".to_string(), Keyword::While);
        self.keywords.insert("in".to_string(), Keyword::In);
        self.keywords.insert("return".to_string(), Keyword::Return);
        self.keywords.insert("break".to_string(), Keyword::Break);
        self.keywords
            .insert("continue".to_string(), Keyword::Continue);
        self.keywords
            .insert("concurrent".to_string(), Keyword::Concurrent);
        self.keywords.insert("async".to_string(), Keyword::Async);
        self.keywords.insert("await".to_string(), Keyword::Await);

        // Error handling keywords
        self.keywords.insert("try".to_string(), Keyword::Try);
        self.keywords.insert("catch".to_string(), Keyword::Catch);
        self.keywords
            .insert("finally".to_string(), Keyword::Finally);
        self.keywords.insert("throw".to_string(), Keyword::Throw);

        // Resource management keywords
        self.keywords
            .insert("resource".to_string(), Keyword::Resource);
        self.keywords
            .insert("cleanup".to_string(), Keyword::Cleanup);
        self.keywords
            .insert("guaranteed".to_string(), Keyword::Guaranteed);

        // Type keywords
        self.keywords.insert("Int".to_string(), Keyword::Int);
        self.keywords.insert("Int32".to_string(), Keyword::Int32);
        self.keywords.insert("Int64".to_string(), Keyword::Int64);
        self.keywords.insert("Float".to_string(), Keyword::Float);
        self.keywords
            .insert("Float32".to_string(), Keyword::Float32);
        self.keywords
            .insert("Float64".to_string(), Keyword::Float64);
        self.keywords.insert("String".to_string(), Keyword::String_);
        self.keywords.insert("Char".to_string(), Keyword::Char);
        self.keywords.insert("Bool".to_string(), Keyword::Bool);
        self.keywords.insert("Void".to_string(), Keyword::Void);
        self.keywords.insert("Array".to_string(), Keyword::Array);
        self.keywords.insert("Map".to_string(), Keyword::Map);
        self.keywords
            .insert("Pointer".to_string(), Keyword::Pointer);
        self.keywords
            .insert("MutPointer".to_string(), Keyword::MutPointer);
        self.keywords.insert("SizeT".to_string(), Keyword::SizeT);
        
        // New fixed-width types
        self.keywords.insert("UInt8".to_string(), Keyword::UInt8);
        self.keywords.insert("Int8".to_string(), Keyword::Int8);
        self.keywords.insert("UInt16".to_string(), Keyword::UInt16);
        self.keywords.insert("Int16".to_string(), Keyword::Int16);
        self.keywords.insert("UInt32".to_string(), Keyword::UInt32);
        self.keywords.insert("UInt64".to_string(), Keyword::UInt64);

        // Literal keywords
        self.keywords.insert("true".to_string(), Keyword::True);
        self.keywords.insert("false".to_string(), Keyword::False);
        self.keywords.insert("nil".to_string(), Keyword::Nil);

        // Generic keywords
        self.keywords.insert("where".to_string(), Keyword::Where);
        self.keywords.insert("trait".to_string(), Keyword::Trait);
        self.keywords.insert("impl".to_string(), Keyword::Impl);

        // Quantifier keywords (for axioms)
        self.keywords.insert("forall".to_string(), Keyword::ForAll);
        self.keywords.insert("exists".to_string(), Keyword::Exists);

        // Other keywords
        self.keywords.insert("as".to_string(), Keyword::As);
        self.keywords.insert("range".to_string(), Keyword::Range);
    }

    /// Get the current source location
    fn current_location(&self) -> SourceLocation {
        SourceLocation::new(
            self.file_name.clone(),
            self.line,
            self.column,
            self.position,
        )
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

    /// Skip a block comment (/* ... */)
    fn skip_block_comment(&mut self) -> Result<(), LexerError> {
        // We're positioned after the first '/', consume the second '*'
        self.advance();

        let start_location = self.current_location();
        let mut depth = 1;

        // We need to consume the '*' that started the comment
        self.advance();

        while depth > 0 {
            match self.current_char {
                Some('/') => {
                    self.advance();
                    if self.current_char == Some('*') {
                        depth += 1;
                        self.advance();
                    }
                }
                Some('*') => {
                    self.advance();
                    if self.current_char == Some('/') {
                        depth -= 1;
                        self.advance();
                    }
                }
                Some(_) => {
                    self.advance();
                }
                None => {
                    return Err(LexerError::UnterminatedBlockComment {
                        location: start_location,
                    });
                }
            }
        }
        Ok(())
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

        // Check for underscore wildcard pattern
        if identifier == "_" {
            return Token::new(TokenType::Underscore, start_location, identifier);
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
                Ok(Token::new(
                    TokenType::LeftBracket,
                    location,
                    "[".to_string(),
                ))
            }
            Some(']') => {
                self.advance();
                Ok(Token::new(
                    TokenType::RightBracket,
                    location,
                    "]".to_string(),
                ))
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
                if self.current_char == Some(':') {
                    self.advance();
                    Ok(Token::new(
                        TokenType::DoubleColon,
                        location,
                        "::".to_string(),
                    ))
                } else {
                    Ok(Token::new(TokenType::Colon, location, ":".to_string()))
                }
            }
            Some(',') => {
                self.advance();
                Ok(Token::new(TokenType::Comma, location, ",".to_string()))
            }
            Some('.') => {
                self.advance();
                if self.current_char == Some('.') {
                    self.advance();
                    if self.current_char == Some('=') {
                        self.advance();
                        Ok(Token::new(
                            TokenType::DotDotEqual,
                            location,
                            "..=".to_string(),
                        ))
                    } else {
                        Ok(Token::new(TokenType::DotDot, location, "..".to_string()))
                    }
                } else {
                    Ok(Token::new(TokenType::Dot, location, ".".to_string()))
                }
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
                } else if self.current_char == Some('*') {
                    // Block comment (/* ... */)
                    self.skip_block_comment()?;
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

            // Equal, EqualEqual, or FatArrow
            Some('=') => {
                self.advance();
                if self.current_char == Some('=') {
                    self.advance();
                    Ok(Token::new(
                        TokenType::EqualEqual,
                        location,
                        "==".to_string(),
                    ))
                } else if self.current_char == Some('>') {
                    self.advance();
                    Ok(Token::new(TokenType::FatArrow, location, "=>".to_string()))
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
                    Ok(Token::new(
                        TokenType::GreaterEqual,
                        location,
                        ">=".to_string(),
                    ))
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
                    Err(LexerError::UnexpectedCharacter {
                        character: '|',
                        location,
                    })
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
            Some(ch) => Err(LexerError::UnexpectedCharacter {
                character: ch,
                location,
            }),
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
mod tests;
