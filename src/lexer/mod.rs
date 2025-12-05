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

//! Lexical analysis for AetherScript
//!
//! Tokenizes S-expression based AetherScript source code

#![allow(dead_code)]

pub mod v2;

use crate::error::{LexerError, SourceLocation};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use unicode_segmentation::UnicodeSegmentation;

/// Token types for AetherScript
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum TokenType {
    // Structural tokens
    LeftParen,
    RightParen,

    // Literals
    Integer(i64),
    Float(f64),
    String(String),
    Character(char),
    Boolean(bool),
    Identifier(String),

    // Keywords
    Keyword(String),

    // Special values
    NullValue,

    // Ownership annotations
    Caret,            // ^ for owned
    Ampersand,        // & for borrowed
    Tilde,            // ~ for shared
    Lifetime(String), // 'a, 'b, etc.

    // Comments and whitespace
    Comment(String),
    Whitespace,

    // End of file
    Eof,
}

/// A token with its type and location information
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

/// Lexer for AetherScript source code
pub struct Lexer {
    input: Vec<char>,
    position: usize,
    current_char: Option<char>,
    line: usize,
    column: usize,
    file_name: String,
    keywords: HashMap<String, String>,
}

impl Lexer {
    /// Create a new lexer for the given input
    pub fn new(input: &str, file_name: String) -> Self {
        let chars: Vec<char> = input.graphemes(true).flat_map(|g| g.chars()).collect();
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

    /// Initialize the keywords map with all AetherScript keywords
    fn initialize_keywords(&mut self) {
        let keywords = [
            // Module and import keywords
            "DEFINE_MODULE",
            "IMPORT_MODULE",
            "EXPORTS_FUNCTION",
            "EXPORTS_TYPE",
            "EXPORTS_CONSTANT",
            // Declaration keywords
            "DECLARE_VARIABLE",
            "DECLARE_CONSTANT",
            "DECLARE_EXTERNAL_FUNCTION",
            // Type definition keywords
            "DEFINE_STRUCTURED_TYPE",
            "DEFINE_ENUMERATION_TYPE",
            "DEFINE_TYPE_ALIAS",
            "DEFINE_FUNCTION",
            // Enumeration and pattern matching keywords
            "VARIANTS",
            "VARIANT",
            "HOLDS",
            "MATCH_EXPRESSION",
            "CASE",
            // Type keywords
            "INTEGER",
            "FLOAT",
            "STRING",
            "CHAR",
            "BOOLEAN",
            "VOID",
            "ARRAY_OF_TYPE",
            "MAP_FROM_TYPE_TO_TYPE",
            "POINTER_TO",
            // Function keywords
            "ACCEPTS_PARAMETER",
            "RETURNS",
            "BODY",
            "CALL_FUNCTION",
            "RETURN_VALUE",
            "RETURN_VOID",
            // Expression keywords
            "EXPRESSION_ADD",
            "EXPRESSION_SUBTRACT",
            "EXPRESSION_MULTIPLY",
            "EXPRESSION_DIVIDE",
            "EXPRESSION_INTEGER_DIVIDE",
            "EXPRESSION_MODULO",
            "EXPRESSION_NEGATE",
            // Predicate keywords
            "PREDICATE_EQUALS",
            "PREDICATE_NOT_EQUALS",
            "PREDICATE_LESS_THAN",
            "PREDICATE_LESS_THAN_OR_EQUAL_TO",
            "PREDICATE_GREATER_THAN",
            "PREDICATE_GREATER_THAN_OR_EQUAL_TO",
            // Logical keywords
            "LOGICAL_AND",
            "LOGICAL_OR",
            "LOGICAL_NOT",
            // String operations
            "STRING_CONCAT",
            "STRING_LENGTH",
            "STRING_CHAR_AT",
            "SUBSTRING",
            "STRING_EQUALS",
            "STRING_CONTAINS",
            // Type conversion
            "CAST_TO_TYPE",
            "TO_STRING",
            "TO_INTEGER",
            "TO_FLOAT",
            // Control flow keywords
            "IF_CONDITION",
            "THEN_EXECUTE",
            "ELSE_IF_CONDITION",
            "ELSE_EXECUTE",
            "LOOP_WHILE_CONDITION",
            "LOOP_FOR_EACH_ELEMENT",
            "LOOP_FIXED_ITERATIONS",
            "COUNTER",
            "FROM",
            "TO",
            "STEP",
            "DO",
            "BREAK_LOOP",
            "CONTINUE_LOOP",
            // Assignment and access keywords
            "ASSIGN",
            "TARGET_VARIABLE",
            "SOURCE_EXPRESSION",
            "GET_FIELD_VALUE",
            "GET_ARRAY_ELEMENT",
            "SET_ARRAY_ELEMENT",
            "GET_MAP_VALUE",
            "SET_MAP_VALUE",
            // Error handling keywords
            "TRY_EXECUTE",
            "CATCH_EXCEPTION",
            "FINALLY_EXECUTE",
            "THROW_EXCEPTION",
            // Metadata keywords
            "INTENT",
            "PRECONDITION",
            "POSTCONDITION",
            "INVARIANT",
            "ALGORITHM_HINT",
            "PERFORMANCE_EXPECTATION",
            "COMPLEXITY_EXPECTATION",
            // Pointer operations
            "ADDRESS_OF",
            "DEREFERENCE",
            "POINTER_ADD",
            // Mutability
            "mut",
            // FFI keywords
            "LIBRARY",
            "SYMBOL",
            "CALLING_CONVENTION",
            "CONVENTION",
            "THREAD_SAFE",
            "MAY_BLOCK",
            "VARIADIC",
            // Construction keywords
            "CONSTRUCT",
            "FIELD_VALUE",
            "ARRAY_LITERAL",
            "ARRAY_LENGTH",
            "MAP_LITERAL",
            // Misc keywords
            "NAME",
            "TYPE",
            "VALUE",
            "MUTABILITY",
            "MUTABLE",
            "IMMUTABLE",
            "FIELD",
            "PARAMETER",
            "ARGUMENT",
            "ELEMENTS",
            "ENTRY",
            "KEY",
            "OWNERSHIP",
            "LIFETIME",
            "PASSING",
            "BY_VALUE",
            "BY_REFERENCE",
            "GENERIC_PARAMETERS",
            "PARAM",
            // Content and container keywords
            "CONTENT",
            "ARGUMENTS",
            "CONDITION",
            "BOOLEAN_EXPRESSION",
            "ITERATION_BODY",
            "ELEMENT_VARIABLE",
            "INDEX_VARIABLE",
            "COLLECTION",
            "PROTECTED_BLOCK",
            "HANDLER_BLOCK",
            "CLEANUP_BLOCK",
            // Wildcard pattern
            "_",
        ];

        for keyword in keywords {
            self.keywords
                .insert(keyword.to_string(), keyword.to_string());
        }
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

    /// Read a number (integer or float)
    fn read_number(&mut self) -> Result<Token, LexerError> {
        let start_location = self.current_location();
        let mut number_str = String::new();
        let mut is_float = false;

        // Handle negative numbers
        if self.current_char == Some('-') {
            number_str.push('-');
            self.advance();
        }

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

        // Handle scientific notation
        if let Some(ch) = self.current_char {
            if ch == 'e' || ch == 'E' {
                is_float = true;
                number_str.push(ch);
                self.advance();

                // Optional + or - after e/E
                if let Some(sign) = self.current_char {
                    if sign == '+' || sign == '-' {
                        number_str.push(sign);
                        self.advance();
                    }
                }

                // Read exponent digits
                while let Some(ch) = self.current_char {
                    if ch.is_ascii_digit() {
                        number_str.push(ch);
                        self.advance();
                    } else {
                        break;
                    }
                }
            }
        }

        if is_float {
            match number_str.parse::<f64>() {
                Ok(value) => Ok(Token::new(
                    TokenType::Float(value),
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
                    TokenType::Integer(value),
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
            lexeme.push(ch);

            if ch == '"' {
                // End of string
                self.advance();
                return Ok(Token::new(
                    TokenType::String(string_value),
                    start_location,
                    lexeme,
                ));
            } else if ch == '\\' {
                // Handle escape sequences
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
                self.advance();
            }
        }

        Err(LexerError::UnterminatedString {
            location: start_location,
        })
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

        // Check for special values
        let token_type = match identifier.as_str() {
            "TRUE" => TokenType::Boolean(true),
            "FALSE" => TokenType::Boolean(false),
            "NULL_VALUE" => TokenType::NullValue,
            _ => {
                // Check if it's a keyword
                if self.keywords.contains_key(&identifier) {
                    TokenType::Keyword(identifier.clone())
                } else {
                    TokenType::Identifier(identifier.clone())
                }
            }
        };

        Token::new(token_type, start_location, identifier)
    }

    /// Read a quoted identifier (surrounded by single quotes)
    fn read_quoted_identifier(&mut self) -> Result<Token, LexerError> {
        let start_location = self.current_location();
        let mut identifier = String::new();
        let mut lexeme = String::new();

        // Skip opening quote
        lexeme.push('\'');
        self.advance();

        while let Some(ch) = self.current_char {
            lexeme.push(ch);

            if ch == '\'' {
                // End of quoted identifier
                self.advance();

                // Check if it's a keyword or identifier
                let token_type = if self.keywords.contains_key(&identifier) {
                    TokenType::Keyword(identifier.clone())
                } else {
                    TokenType::Identifier(identifier.clone())
                };

                return Ok(Token::new(token_type, start_location, lexeme));
            } else if ch == '\n' || ch == '\r' {
                return Err(LexerError::UnterminatedString {
                    location: start_location,
                });
            } else {
                identifier.push(ch);
                self.advance();
            }
        }

        Err(LexerError::UnterminatedString {
            location: start_location,
        })
    }

    /// Read quoted content (character literal or quoted identifier)
    fn read_quoted_content(&mut self) -> Result<Token, LexerError> {
        let start_location = self.current_location();
        let mut content = String::new();
        let mut lexeme = String::new();

        // Skip opening quote
        lexeme.push('\'');
        self.advance();

        // Read content between quotes
        while let Some(ch) = self.current_char {
            if ch == '\'' {
                // End of quoted content
                lexeme.push(ch);
                self.advance();

                // Determine if this is a character literal or quoted identifier
                if content.len() == 1 {
                    // Single character - treat as character literal
                    // Safe: we just verified content.len() == 1
                    let character = content
                        .chars()
                        .next()
                        .expect("content verified to have exactly 1 char");
                    return Ok(Token::new(
                        TokenType::Character(character),
                        start_location,
                        lexeme,
                    ));
                } else {
                    // Multiple characters - treat as quoted identifier
                    let token_type = if self.keywords.contains_key(&content) {
                        TokenType::Keyword(content.clone())
                    } else {
                        TokenType::Identifier(content.clone())
                    };

                    return Ok(Token::new(token_type, start_location, lexeme));
                }
            } else if ch == '\\' {
                // Handle escape sequences for character literals
                lexeme.push(ch);
                self.advance();
                match self.current_char {
                    Some('n') => {
                        content.push('\n');
                        lexeme.push('n');
                    }
                    Some('t') => {
                        content.push('\t');
                        lexeme.push('t');
                    }
                    Some('r') => {
                        content.push('\r');
                        lexeme.push('r');
                    }
                    Some('\\') => {
                        content.push('\\');
                        lexeme.push('\\');
                    }
                    Some('\'') => {
                        content.push('\'');
                        lexeme.push('\'');
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
                content.push(ch);
                lexeme.push(ch);
                self.advance();
            }
        }

        Err(LexerError::UnterminatedString {
            location: start_location,
        })
    }

    /// Read a comment
    fn read_comment(&mut self) -> Token {
        let start_location = self.current_location();
        let mut comment = String::new();

        // Skip the semicolon
        self.advance();

        while let Some(ch) = self.current_char {
            if ch == '\n' || ch == '\r' {
                break;
            }
            comment.push(ch);
            self.advance();
        }

        Token::new(
            TokenType::Comment(comment.trim().to_string()),
            start_location,
            format!(";{}", comment),
        )
    }

    /// Get the next token from the input
    pub fn next_token(&mut self) -> Result<Token, LexerError> {
        loop {
            match self.current_char {
                None => {
                    return Ok(Token::new(
                        TokenType::Eof,
                        self.current_location(),
                        String::new(),
                    ));
                }
                Some('(') => {
                    let location = self.current_location();
                    self.advance();
                    return Ok(Token::new(TokenType::LeftParen, location, "(".to_string()));
                }
                Some(')') => {
                    let location = self.current_location();
                    self.advance();
                    return Ok(Token::new(TokenType::RightParen, location, ")".to_string()));
                }
                Some(';') => {
                    return Ok(self.read_comment());
                }
                Some('"') => {
                    return self.read_string();
                }
                Some('\'') => {
                    return self.read_quoted_content();
                }
                Some(ch) if ch.is_whitespace() => {
                    self.skip_whitespace();
                    continue;
                }
                Some(ch)
                    if ch.is_ascii_digit()
                        || (ch == '-' && self.peek().is_some_and(|c| c.is_ascii_digit())) =>
                {
                    return self.read_number();
                }
                Some(ch) if ch.is_ascii_alphabetic() || ch == '_' => {
                    return Ok(self.read_identifier());
                }
                Some('^') => {
                    let location = self.current_location();
                    self.advance();
                    return Ok(Token::new(TokenType::Caret, location, "^".to_string()));
                }
                Some('&') => {
                    let location = self.current_location();
                    self.advance();
                    return Ok(Token::new(TokenType::Ampersand, location, "&".to_string()));
                }
                Some('~') => {
                    let location = self.current_location();
                    self.advance();
                    return Ok(Token::new(TokenType::Tilde, location, "~".to_string()));
                }
                Some(ch) => {
                    let location = self.current_location();
                    return Err(LexerError::UnexpectedCharacter {
                        character: ch,
                        location,
                    });
                }
            }
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

    /// Peek at the next token without consuming it
    pub fn peek_token(&mut self) -> Result<Token, LexerError> {
        let saved_position = self.position;
        let saved_current_char = self.current_char;
        let saved_line = self.line;
        let saved_column = self.column;

        let token = self.next_token();

        // Restore lexer state
        self.position = saved_position;
        self.current_char = saved_current_char;
        self.line = saved_line;
        self.column = saved_column;

        token
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_tokens() {
        let mut lexer = Lexer::new("( )", "test.aether".to_string());
        let tokens = lexer.tokenize().unwrap();

        assert_eq!(tokens.len(), 3); // ( ) EOF
        assert!(matches!(tokens[0].token_type, TokenType::LeftParen));
        assert!(matches!(tokens[1].token_type, TokenType::RightParen));
        assert!(matches!(tokens[2].token_type, TokenType::Eof));
    }

    #[test]
    fn test_numbers() {
        let mut lexer = Lexer::new("42 -17 3.14 -2.5 1.5e10 2E-3", "test.aether".to_string());
        let tokens = lexer.tokenize().unwrap();

        assert!(matches!(tokens[0].token_type, TokenType::Integer(42)));
        assert!(matches!(tokens[1].token_type, TokenType::Integer(-17)));
        assert!(
            matches!(tokens[2].token_type, TokenType::Float(f) if (f - 3.14).abs() < f64::EPSILON)
        );
        assert!(
            matches!(tokens[3].token_type, TokenType::Float(f) if (f - (-2.5)).abs() < f64::EPSILON)
        );
        assert!(
            matches!(tokens[4].token_type, TokenType::Float(f) if (f - 1.5e10).abs() < f64::EPSILON)
        );
        assert!(
            matches!(tokens[5].token_type, TokenType::Float(f) if (f - 2E-3).abs() < f64::EPSILON)
        );
    }

    #[test]
    fn test_strings() {
        let mut lexer = Lexer::new(
            r#""hello" "world\n" "test\"quote""#,
            "test.aether".to_string(),
        );
        let tokens = lexer.tokenize().unwrap();

        assert!(matches!(tokens[0].token_type, TokenType::String(ref s) if s == "hello"));
        assert!(matches!(tokens[1].token_type, TokenType::String(ref s) if s == "world\n"));
        assert!(matches!(tokens[2].token_type, TokenType::String(ref s) if s == "test\"quote"));
    }

    #[test]
    fn test_identifiers_and_keywords() {
        let mut lexer = Lexer::new(
            "DEFINE_FUNCTION my_var TRUE FALSE NULL_VALUE",
            "test.aether".to_string(),
        );
        let tokens = lexer.tokenize().unwrap();

        assert!(
            matches!(tokens[0].token_type, TokenType::Keyword(ref k) if k == "DEFINE_FUNCTION")
        );
        assert!(matches!(tokens[1].token_type, TokenType::Identifier(ref i) if i == "my_var"));
        assert!(matches!(tokens[2].token_type, TokenType::Boolean(true)));
        assert!(matches!(tokens[3].token_type, TokenType::Boolean(false)));
        assert!(matches!(tokens[4].token_type, TokenType::NullValue));
    }

    #[test]
    fn test_comments() {
        let mut lexer = Lexer::new("; This is a comment\n(", "test.aether".to_string());
        let tokens = lexer.tokenize().unwrap();

        assert!(
            matches!(tokens[0].token_type, TokenType::Comment(ref c) if c == "This is a comment")
        );
        assert!(matches!(tokens[1].token_type, TokenType::LeftParen));
    }

    #[test]
    fn test_error_cases() {
        // Unterminated string
        let mut lexer = Lexer::new(r#""unterminated"#, "test.aether".to_string());
        assert!(matches!(
            lexer.tokenize(),
            Err(LexerError::UnterminatedString { .. })
        ));

        // Invalid escape sequence
        let mut lexer = Lexer::new(r#""\x""#, "test.aether".to_string());
        assert!(matches!(
            lexer.tokenize(),
            Err(LexerError::InvalidEscapeSequence { .. })
        ));

        // Unexpected character
        let mut lexer = Lexer::new("@", "test.aether".to_string());
        assert!(matches!(
            lexer.tokenize(),
            Err(LexerError::UnexpectedCharacter { .. })
        ));
    }

    #[test]
    fn test_peek_token() {
        let mut lexer = Lexer::new("(", "test.aether".to_string());

        let peeked = lexer.peek_token().unwrap();
        assert!(matches!(peeked.token_type, TokenType::LeftParen));

        let actual = lexer.next_token().unwrap();
        assert!(matches!(actual.token_type, TokenType::LeftParen));
    }

    #[test]
    fn test_quoted_identifiers() {
        let mut lexer = Lexer::new("'hello_world' 'DEFINE_FUNCTION'", "test.aether".to_string());
        let tokens = lexer.tokenize().unwrap();

        assert!(matches!(tokens[0].token_type, TokenType::Identifier(ref i) if i == "hello_world"));
        assert!(
            matches!(tokens[1].token_type, TokenType::Keyword(ref k) if k == "DEFINE_FUNCTION")
        );
    }

    #[test]
    fn test_source_locations() {
        let mut lexer = Lexer::new("(\n  hello\n)", "test.aether".to_string());
        let tokens = lexer.tokenize().unwrap();

        assert_eq!(tokens[0].location.line, 1);
        assert_eq!(tokens[0].location.column, 1);

        assert_eq!(tokens[1].location.line, 2);
        assert_eq!(tokens[1].location.column, 3);

        assert_eq!(tokens[2].location.line, 3);
        assert_eq!(tokens[2].location.column, 1);
    }
}
