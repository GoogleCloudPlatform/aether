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

//! V2 Parser for AetherScript
//!
//! Parses the new Swift/Rust-like V2 syntax into AST nodes.

use crate::ast::{Identifier, ImportStatement, Module};
use crate::error::{ParserError, SourceLocation};
use crate::lexer::v2::{Keyword, Token, TokenType};

/// V2 Parser for AetherScript
pub struct Parser {
    tokens: Vec<Token>,
    position: usize,
    errors: Vec<ParserError>,
}

impl Parser {
    /// Create a new V2 parser from a token stream
    pub fn new(tokens: Vec<Token>) -> Self {
        Self {
            tokens,
            position: 0,
            errors: Vec::new(),
        }
    }

    /// Get the current token without advancing
    pub fn peek(&self) -> &Token {
        self.tokens.get(self.position).unwrap_or_else(|| {
            // Return the last token (should be EOF)
            self.tokens.last().expect("Token stream should not be empty")
        })
    }

    /// Peek at the next token (one ahead of current)
    pub fn peek_next(&self) -> Option<&Token> {
        self.tokens.get(self.position + 1)
    }

    /// Advance to the next token and return the previous one
    pub fn advance(&mut self) -> &Token {
        if !self.is_at_end() {
            self.position += 1;
        }
        self.previous()
    }

    /// Get the previous token
    pub fn previous(&self) -> &Token {
        self.tokens.get(self.position.saturating_sub(1)).unwrap_or_else(|| {
            self.tokens.first().expect("Token stream should not be empty")
        })
    }

    /// Check if we've reached the end of the token stream
    pub fn is_at_end(&self) -> bool {
        matches!(self.peek().token_type, TokenType::Eof)
    }

    /// Check if the current token matches the expected type
    pub fn check(&self, token_type: &TokenType) -> bool {
        if self.is_at_end() {
            return false;
        }
        self.matches_token_type(&self.peek().token_type, token_type)
    }

    /// Check if the current token is a specific keyword
    pub fn check_keyword(&self, keyword: Keyword) -> bool {
        if self.is_at_end() {
            return false;
        }
        matches!(&self.peek().token_type, TokenType::Keyword(k) if *k == keyword)
    }

    /// Consume a token if it matches the expected type, otherwise return an error
    pub fn expect(&mut self, token_type: &TokenType, message: &str) -> Result<&Token, ParserError> {
        if self.check(token_type) {
            Ok(self.advance())
        } else {
            Err(ParserError::UnexpectedToken {
                expected: message.to_string(),
                found: format!("{:?}", self.peek().token_type),
                location: self.peek().location.clone(),
            })
        }
    }

    /// Consume a keyword if it matches, otherwise return an error
    pub fn expect_keyword(&mut self, keyword: Keyword, message: &str) -> Result<&Token, ParserError> {
        if self.check_keyword(keyword.clone()) {
            Ok(self.advance())
        } else {
            Err(ParserError::UnexpectedToken {
                expected: message.to_string(),
                found: format!("{:?}", self.peek().token_type),
                location: self.peek().location.clone(),
            })
        }
    }

    /// Check if current token matches any of the given types
    pub fn match_any(&mut self, types: &[TokenType]) -> bool {
        for t in types {
            if self.check(t) {
                self.advance();
                return true;
            }
        }
        false
    }

    /// Get collected errors
    pub fn errors(&self) -> &[ParserError] {
        &self.errors
    }

    /// Add an error to the error list
    pub fn add_error(&mut self, error: ParserError) {
        self.errors.push(error);
    }

    /// Get the current position
    pub fn current_position(&self) -> usize {
        self.position
    }

    /// Get the current location for error reporting
    pub fn current_location(&self) -> SourceLocation {
        self.peek().location.clone()
    }

    // ==================== PARSING METHODS ====================

    /// Parse a module definition
    /// Grammar: "module" IDENTIFIER "{" module_item* "}"
    pub fn parse_module(&mut self) -> Result<Module, ParserError> {
        let start_location = self.current_location();

        // Expect 'module' keyword
        self.expect_keyword(Keyword::Module, "expected 'module'")?;

        // Parse module name
        let name = self.parse_identifier()?;

        // Expect opening brace
        self.expect(&TokenType::LeftBrace, "expected '{' after module name")?;

        // Parse module items (imports, functions, structs, etc.)
        let mut imports = Vec::new();

        while !self.check(&TokenType::RightBrace) && !self.is_at_end() {
            if self.check_keyword(Keyword::Import) {
                imports.push(self.parse_import()?);
            } else {
                // For now, skip unknown items until we implement more parsing
                // In future tasks we'll add function, struct, enum parsing here
                break;
            }
        }

        // Expect closing brace
        self.expect(&TokenType::RightBrace, "expected '}' to close module")?;

        Ok(Module {
            name,
            intent: None,
            imports,
            exports: Vec::new(),
            type_definitions: Vec::new(),
            constant_declarations: Vec::new(),
            function_definitions: Vec::new(),
            external_functions: Vec::new(),
            source_location: start_location,
        })
    }

    /// Parse an import statement
    /// Grammar: "import" dotted_name ";"
    pub fn parse_import(&mut self) -> Result<ImportStatement, ParserError> {
        let start_location = self.current_location();

        // Expect 'import' keyword
        self.expect_keyword(Keyword::Import, "expected 'import'")?;

        // Parse dotted module name (e.g., std.io)
        let module_name = self.parse_dotted_identifier()?;

        // Expect semicolon
        self.expect(&TokenType::Semicolon, "expected ';' after import statement")?;

        Ok(ImportStatement {
            module_name,
            alias: None,
            source_location: start_location,
        })
    }

    /// Parse a single identifier
    fn parse_identifier(&mut self) -> Result<Identifier, ParserError> {
        let location = self.current_location();

        match &self.peek().token_type {
            TokenType::Identifier(name) => {
                let name = name.clone();
                self.advance();
                Ok(Identifier::new(name, location))
            }
            _ => Err(ParserError::UnexpectedToken {
                expected: "identifier".to_string(),
                found: format!("{:?}", self.peek().token_type),
                location,
            }),
        }
    }

    /// Parse a dotted identifier (e.g., std.io.File)
    fn parse_dotted_identifier(&mut self) -> Result<Identifier, ParserError> {
        let start_location = self.current_location();
        let mut parts = Vec::new();

        // First identifier
        let first = self.parse_identifier()?;
        parts.push(first.name);

        // Continue parsing .identifier sequences
        while self.check(&TokenType::Dot) {
            self.advance(); // consume dot
            let part = self.parse_identifier()?;
            parts.push(part.name);
        }

        Ok(Identifier::new(parts.join("."), start_location))
    }

    // ==================== HELPER METHODS ====================

    /// Helper to compare token types, handling variants with data
    fn matches_token_type(&self, actual: &TokenType, expected: &TokenType) -> bool {
        use std::mem::discriminant;

        // For tokens with data, compare discriminants (type match without value match)
        // This allows check(&TokenType::Identifier("".to_string())) to match any identifier
        match (actual, expected) {
            // For tokens with data, just match on the variant type
            (TokenType::IntegerLiteral(_), TokenType::IntegerLiteral(_)) => true,
            (TokenType::FloatLiteral(_), TokenType::FloatLiteral(_)) => true,
            (TokenType::StringLiteral(_), TokenType::StringLiteral(_)) => true,
            (TokenType::CharLiteral(_), TokenType::CharLiteral(_)) => true,
            (TokenType::BoolLiteral(_), TokenType::BoolLiteral(_)) => true,
            (TokenType::Identifier(_), TokenType::Identifier(_)) => true,
            (TokenType::Keyword(a), TokenType::Keyword(b)) => a == b, // Keywords need exact match

            // For tokens without data, compare discriminants
            _ => discriminant(actual) == discriminant(expected),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer::v2::Lexer;

    /// Helper to create a parser from source code
    fn parser_from_source(source: &str) -> Parser {
        let mut lexer = Lexer::new(source, "test.aether".to_string());
        let tokens = lexer.tokenize().unwrap();
        Parser::new(tokens)
    }

    // ==================== BASIC PARSER TESTS ====================

    #[test]
    fn test_parser_new() {
        let parser = parser_from_source("let x");
        assert_eq!(parser.position, 0);
        assert!(parser.errors.is_empty());
    }

    #[test]
    fn test_parser_peek() {
        let parser = parser_from_source("let x");
        assert!(matches!(parser.peek().token_type, TokenType::Keyword(Keyword::Let)));
    }

    #[test]
    fn test_parser_peek_next() {
        let parser = parser_from_source("let x");
        let next = parser.peek_next().unwrap();
        assert!(matches!(next.token_type, TokenType::Identifier(ref s) if s == "x"));
    }

    #[test]
    fn test_parser_advance() {
        let mut parser = parser_from_source("let x");

        // First advance returns "let"
        let tok = parser.advance();
        assert!(matches!(tok.token_type, TokenType::Keyword(Keyword::Let)));

        // Now peek should be "x"
        assert!(matches!(parser.peek().token_type, TokenType::Identifier(ref s) if s == "x"));
    }

    #[test]
    fn test_parser_previous() {
        let mut parser = parser_from_source("let x");
        parser.advance();

        let prev = parser.previous();
        assert!(matches!(prev.token_type, TokenType::Keyword(Keyword::Let)));
    }

    #[test]
    fn test_parser_is_at_end() {
        let mut parser = parser_from_source("x");

        assert!(!parser.is_at_end());
        parser.advance(); // x
        assert!(parser.is_at_end()); // Now at EOF
    }

    #[test]
    fn test_parser_check() {
        let parser = parser_from_source("let x = 42");

        assert!(parser.check(&TokenType::Keyword(Keyword::Let)));
        assert!(!parser.check(&TokenType::Identifier("x".to_string())));
    }

    #[test]
    fn test_parser_check_keyword() {
        let parser = parser_from_source("func main");

        assert!(parser.check_keyword(Keyword::Func));
        assert!(!parser.check_keyword(Keyword::Let));
    }

    #[test]
    fn test_parser_expect_success() {
        let mut parser = parser_from_source("let x");

        let result = parser.expect(&TokenType::Keyword(Keyword::Let), "expected 'let'");
        assert!(result.is_ok());

        // Position should have advanced
        assert!(matches!(parser.peek().token_type, TokenType::Identifier(ref s) if s == "x"));
    }

    #[test]
    fn test_parser_expect_failure() {
        let mut parser = parser_from_source("let x");

        let result = parser.expect(&TokenType::Keyword(Keyword::Func), "expected 'func'");
        assert!(result.is_err());

        // Position should NOT have advanced
        assert!(matches!(parser.peek().token_type, TokenType::Keyword(Keyword::Let)));
    }

    #[test]
    fn test_parser_expect_keyword_success() {
        let mut parser = parser_from_source("module Test");

        let result = parser.expect_keyword(Keyword::Module, "expected 'module'");
        assert!(result.is_ok());
    }

    #[test]
    fn test_parser_expect_keyword_failure() {
        let mut parser = parser_from_source("module Test");

        let result = parser.expect_keyword(Keyword::Func, "expected 'func'");
        assert!(result.is_err());
    }

    #[test]
    fn test_parser_match_any() {
        let mut parser = parser_from_source("+ - *");

        // Should match Plus
        assert!(parser.match_any(&[TokenType::Plus, TokenType::Minus]));

        // Now at Minus, should match
        assert!(parser.match_any(&[TokenType::Plus, TokenType::Minus]));

        // Now at Star, should NOT match
        assert!(!parser.match_any(&[TokenType::Plus, TokenType::Minus]));
    }

    #[test]
    fn test_parser_current_location() {
        let parser = parser_from_source("let x");
        let loc = parser.current_location();

        assert_eq!(loc.line, 1);
        assert_eq!(loc.column, 1);
    }

    #[test]
    fn test_parser_add_error() {
        let mut parser = parser_from_source("let x");

        assert!(parser.errors().is_empty());

        parser.add_error(ParserError::UnexpectedToken {
            expected: "test".to_string(),
            found: "other".to_string(),
            location: parser.current_location(),
        });

        assert_eq!(parser.errors().len(), 1);
    }

    #[test]
    fn test_parser_empty_input() {
        let parser = parser_from_source("");

        // Should just have EOF
        assert!(parser.is_at_end());
    }

    #[test]
    fn test_parser_multiline_input() {
        let mut parser = parser_from_source("let\nx\n=\n42");

        // Should be able to parse through newlines
        parser.expect_keyword(Keyword::Let, "let").unwrap();

        // Next should be identifier
        assert!(matches!(parser.peek().token_type, TokenType::Identifier(ref s) if s == "x"));
    }

    #[test]
    fn test_parser_with_comments() {
        let parser = parser_from_source("// comment\nlet x");

        // Comments should be skipped, first token is 'let'
        assert!(parser.check_keyword(Keyword::Let));
    }

    // ==================== MODULE PARSING TESTS ====================

    #[test]
    fn test_parse_empty_module() {
        let mut parser = parser_from_source("module Test { }");
        let result = parser.parse_module();

        assert!(result.is_ok());
        let module = result.unwrap();
        assert_eq!(module.name.name, "Test");
        assert!(module.imports.is_empty());
        assert!(module.function_definitions.is_empty());
    }

    #[test]
    fn test_parse_module_with_single_import() {
        let mut parser = parser_from_source("module Test { import std; }");
        let result = parser.parse_module();

        assert!(result.is_ok());
        let module = result.unwrap();
        assert_eq!(module.name.name, "Test");
        assert_eq!(module.imports.len(), 1);
        assert_eq!(module.imports[0].module_name.name, "std");
    }

    #[test]
    fn test_parse_module_with_dotted_import() {
        let mut parser = parser_from_source("module Test { import std.io; }");
        let result = parser.parse_module();

        assert!(result.is_ok());
        let module = result.unwrap();
        assert_eq!(module.imports.len(), 1);
        assert_eq!(module.imports[0].module_name.name, "std.io");
    }

    #[test]
    fn test_parse_module_with_multiple_imports() {
        let mut parser = parser_from_source(
            "module Test { import std.io; import std.collections; import math; }"
        );
        let result = parser.parse_module();

        assert!(result.is_ok());
        let module = result.unwrap();
        assert_eq!(module.imports.len(), 3);
        assert_eq!(module.imports[0].module_name.name, "std.io");
        assert_eq!(module.imports[1].module_name.name, "std.collections");
        assert_eq!(module.imports[2].module_name.name, "math");
    }

    #[test]
    fn test_parse_module_with_deeply_nested_import() {
        let mut parser = parser_from_source(
            "module Test { import std.collections.hashmap.HashMap; }"
        );
        let result = parser.parse_module();

        assert!(result.is_ok());
        let module = result.unwrap();
        assert_eq!(module.imports[0].module_name.name, "std.collections.hashmap.HashMap");
    }

    #[test]
    fn test_parse_module_error_missing_name() {
        let mut parser = parser_from_source("module { }");
        let result = parser.parse_module();

        assert!(result.is_err());
    }

    #[test]
    fn test_parse_module_error_missing_open_brace() {
        let mut parser = parser_from_source("module Test }");
        let result = parser.parse_module();

        assert!(result.is_err());
    }

    #[test]
    fn test_parse_module_error_missing_close_brace() {
        let mut parser = parser_from_source("module Test {");
        let result = parser.parse_module();

        assert!(result.is_err());
    }

    #[test]
    fn test_parse_import_error_missing_semicolon() {
        let mut parser = parser_from_source("module Test { import std }");
        let result = parser.parse_module();

        assert!(result.is_err());
    }

    #[test]
    fn test_parse_module_multiline() {
        let source = r#"
module MyModule {
    import std.io;
    import std.collections;
}
"#;
        let mut parser = parser_from_source(source);
        let result = parser.parse_module();

        assert!(result.is_ok());
        let module = result.unwrap();
        assert_eq!(module.name.name, "MyModule");
        assert_eq!(module.imports.len(), 2);
    }

    #[test]
    fn test_parse_module_with_comments() {
        let source = r#"
// Module documentation
module Test {
    // Import the standard library
    import std;
}
"#;
        let mut parser = parser_from_source(source);
        let result = parser.parse_module();

        assert!(result.is_ok());
        let module = result.unwrap();
        assert_eq!(module.name.name, "Test");
        assert_eq!(module.imports.len(), 1);
    }
}
