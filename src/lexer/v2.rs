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

use crate::error::SourceLocation;
use serde::{Deserialize, Serialize};

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
}
