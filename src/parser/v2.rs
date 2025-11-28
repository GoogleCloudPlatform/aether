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

use crate::ast::{
    Argument, AssignmentTarget, Block, CallingConvention, Capture, CaptureMode,
    ConstantDeclaration, ElseIf, EnumVariant, ExportStatement, Expression, ExternalFunction, FieldValue, Function,
    FunctionCall as AstFunctionCall, FunctionMetadata, FunctionReference, Identifier,
    ImportStatement, LambdaBody, MatchArm, MatchCase, Module, Mutability, OwnershipKind, Parameter, PassingMode, Pattern, PrimitiveType, Program,
    Statement, StructField, TypeDefinition, TypeSpecifier,
};
use crate::error::{ParserError, SourceLocation};
use crate::lexer::v2::{Keyword, Token, TokenType};

/// Internal enum for binary operators during parsing
#[derive(Debug, Clone, Copy)]
enum BinaryOp {
    Add,
    Subtract,
    Multiply,
    Divide,
    Modulo,
    Equals,
    NotEquals,
    LessThan,
    LessEqual,
    GreaterThan,
    GreaterEqual,
    And,
    Or,
}

/// V2 Parser for AetherScript
pub struct Parser {
    tokens: Vec<Token>,
    position: usize,
    errors: Vec<ParserError>,
}

/// Parsed annotation (e.g., @extern, @requires)
#[derive(Debug, Clone)]
pub struct Annotation {
    pub name: String,
    pub arguments: Vec<AnnotationArgument>,
    pub source_location: SourceLocation,
}

/// Annotation argument (labeled or unlabeled)
#[derive(Debug, Clone)]
pub struct AnnotationArgument {
    pub label: Option<String>,
    pub value: AnnotationValue,
    pub source_location: SourceLocation,
}

/// Annotation argument value types
#[derive(Debug, Clone)]
pub enum AnnotationValue {
    String(String),
    Integer(i64),
    Boolean(bool),
    Identifier(String),
    Expression(String, SourceLocation), // Raw expression string for contracts
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
            self.tokens
                .last()
                .expect("Token stream should not be empty")
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
        self.tokens
            .get(self.position.saturating_sub(1))
            .unwrap_or_else(|| {
                self.tokens
                    .first()
                    .expect("Token stream should not be empty")
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
    pub fn expect_keyword(
        &mut self,
        keyword: Keyword,
        message: &str,
    ) -> Result<&Token, ParserError> {
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

    /// Check if there are any collected errors
    pub fn has_errors(&self) -> bool {
        !self.errors.is_empty()
    }

    /// Take all collected errors, leaving the list empty
    pub fn take_errors(&mut self) -> Vec<ParserError> {
        std::mem::take(&mut self.errors)
    }

    // ==================== ERROR RECOVERY ====================

    /// Synchronize to a safe recovery point after an error
    /// Skips tokens until we find a statement boundary or declaration start
    fn synchronize(&mut self) {
        self.advance();

        while !self.is_at_end() {
            // If we just passed a semicolon, we're at a statement boundary
            if matches!(self.previous().token_type, TokenType::Semicolon) {
                return;
            }

            // If we see a keyword that starts a declaration, stop here
            match &self.peek().token_type {
                TokenType::Keyword(kw) => match kw {
                    Keyword::Func
                    | Keyword::Struct
                    | Keyword::Enum
                    | Keyword::Let
                    | Keyword::Var
                    | Keyword::Const
                    | Keyword::Import
                    | Keyword::Module
                    | Keyword::When
                    | Keyword::While
                    | Keyword::Return => return,
                    _ => {}
                },
                TokenType::At => return, // Annotation starts a new declaration
                TokenType::RightBrace => return, // End of block
                _ => {}
            }

            self.advance();
        }
    }

    /// Synchronize to the end of a block (closing brace or EOF)
    fn synchronize_to_block_end(&mut self) {
        let mut brace_depth = 1;

        while !self.is_at_end() && brace_depth > 0 {
            match &self.peek().token_type {
                TokenType::LeftBrace => brace_depth += 1,
                TokenType::RightBrace => brace_depth -= 1,
                _ => {}
            }
            if brace_depth > 0 {
                self.advance();
            }
        }
    }

    /// Synchronize to the next module-level item
    fn synchronize_to_module_item(&mut self) {
        while !self.is_at_end() {
            match &self.peek().token_type {
                TokenType::Keyword(kw) => match kw {
                    Keyword::Func | Keyword::Struct | Keyword::Enum | Keyword::Import | Keyword::Const => return,
                    _ => {}
                },
                TokenType::At => return,         // Annotation
                TokenType::RightBrace => { self.advance(); return; }
                _ => {}
            }
            self.advance();
        }
    }

    /// Get the current position
    pub fn current_position(&self) -> usize {
        self.position
    }

    /// Get the current location for error reporting
    pub fn current_location(&self) -> SourceLocation {
        self.peek().location.clone()
    }

    /// Check if what follows looks like a struct construction: { field: value, ... }
    /// This requires looking ahead: after '{', if we see 'identifier :' it's struct construction.
    /// Called when current token is '{'.
    /// Note: Empty braces `{ }` are NOT treated as struct construction because they are
    /// ambiguous (could be a block in control flow like `when x { }`).
    fn looks_like_struct_construction(&self) -> bool {
        // Current token should be '{'
        if !matches!(self.peek().token_type, TokenType::LeftBrace) {
            return false;
        }

        // Look at token after '{'
        if let Some(after_brace) = self.peek_at(self.position + 1) {
            // Check for field: value pattern
            // Only treat as struct construction if we see `identifier :` pattern
            if let TokenType::Identifier(_) = &after_brace.token_type {
                // Check if identifier is followed by ':'
                if let Some(after_ident) = self.peek_at(self.position + 2) {
                    if matches!(after_ident.token_type, TokenType::Colon) {
                        return true;
                    }
                }
            }
        }

        false
    }

    /// Check if what follows looks like an array literal: [expr, expr, ...]
    /// vs a capture list: [ident, ident, ...](params) => body
    /// Called when current token is '['.
    ///
    /// Array literal detection: if the first element after '[' is NOT an identifier,
    /// or if ']' is NOT followed by '(', it's an array literal.
    fn looks_like_array_literal(&self) -> bool {
        // Current token should be '['
        if !matches!(self.peek().token_type, TokenType::LeftBracket) {
            return false;
        }

        // Empty brackets [] - treat as empty array
        if let Some(after_bracket) = self.peek_at(self.position + 1) {
            if matches!(after_bracket.token_type, TokenType::RightBracket) {
                // Check if followed by '(' - if so, it's an empty capture list
                if let Some(after_close) = self.peek_at(self.position + 2) {
                    if matches!(after_close.token_type, TokenType::LeftParen) {
                        return false; // capture list
                    }
                }
                return true; // empty array
            }
        }

        // Check the first element after '['
        if let Some(first_elem) = self.peek_at(self.position + 1) {
            // If it's not an identifier (or & for capture by ref), it's definitely an array
            match &first_elem.token_type {
                TokenType::Identifier(_) | TokenType::Ampersand => {
                    // Could be either - need to scan to find ']' and check what follows
                    // For simplicity, scan ahead to find the matching ']'
                    let mut depth = 1;
                    let mut pos = self.position + 1;
                    while depth > 0 {
                        if let Some(tok) = self.peek_at(pos) {
                            match &tok.token_type {
                                TokenType::LeftBracket => depth += 1,
                                TokenType::RightBracket => depth -= 1,
                                _ => {}
                            }
                            pos += 1;
                        } else {
                            break;
                        }
                    }
                    // pos is now just after ']'
                    // Check if followed by '('
                    if let Some(after_close) = self.peek_at(pos) {
                        if matches!(after_close.token_type, TokenType::LeftParen) {
                            return false; // capture list followed by params
                        }
                    }
                    return true; // not followed by '(', so it's an array
                }
                _ => return true, // non-identifier first element = array literal
            }
        }

        true // default to array
    }

    /// Peek at a specific position in the token stream
    fn peek_at(&self, pos: usize) -> Option<&Token> {
        self.tokens.get(pos)
    }

    // ==================== ERROR HELPERS ====================

    /// Create a syntax error with a helpful message and optional suggestion
    fn syntax_error(&self, message: &str, suggestion: Option<&str>) -> ParserError {
        ParserError::SyntaxError {
            message: message.to_string(),
            location: self.current_location(),
            suggestion: suggestion.map(|s| s.to_string()),
        }
    }

    /// Create an error for a missing semicolon
    fn missing_semicolon_error(&self) -> ParserError {
        self.syntax_error(
            "Missing semicolon",
            Some("Add ';' at the end of the statement"),
        )
    }

    /// Create an error for a missing type annotation
    fn missing_type_error(&self, context: &str) -> ParserError {
        self.syntax_error(
            &format!("Missing type annotation in {}", context),
            Some("Add a type annotation like ': Int' or ': String'"),
        )
    }

    /// Create a contextual unexpected token error with suggestions
    fn unexpected_token_error(&self, context: &str) -> ParserError {
        let token = &self.peek().token_type;
        let (message, suggestion) = match token {
            TokenType::Semicolon => (
                format!("Unexpected semicolon in {}", context),
                Some("Remove the extra semicolon or check your syntax"),
            ),
            TokenType::RightBrace => (
                format!("Unexpected '}}' in {}", context),
                Some("Check for missing statements or unbalanced braces"),
            ),
            TokenType::RightParen => (
                format!("Unexpected ')' in {}", context),
                Some("Check for missing expressions or unbalanced parentheses"),
            ),
            TokenType::Keyword(kw) => (
                format!("Unexpected keyword '{:?}' in {}", kw, context),
                Some("Keywords cannot be used here. Did you forget a semicolon?"),
            ),
            TokenType::Eof => (
                format!("Unexpected end of file in {}", context),
                Some("Check for unclosed braces or missing code"),
            ),
            _ => (format!("Unexpected token {:?} in {}", token, context), None),
        };

        ParserError::SyntaxError {
            message,
            location: self.current_location(),
            suggestion: suggestion.map(|s| s.to_string()),
        }
    }

    /// Get a description of what kind of expression was expected
    fn expected_expression_hint(&self) -> &'static str {
        match &self.peek().token_type {
            TokenType::RightParen => "expression before ')'",
            TokenType::RightBracket => "expression before ']'",
            TokenType::RightBrace => "expression before '}'",
            TokenType::Semicolon => "expression before ';'",
            TokenType::Comma => "expression before ','",
            _ => "expression",
        }
    }

    // ==================== PARSING METHODS ====================

    /// Parse a complete program (one or more modules)
    pub fn parse_program(&mut self) -> Result<Program, ParserError> {
        let start_location = self.current_location();
        let mut modules = Vec::new();

        // Parse modules until EOF
        while !self.is_at_end() {
            modules.push(self.parse_module()?);
        }

        Ok(Program {
            modules,
            source_location: start_location,
        })
    }

    /// Parse a module with full error recovery, returning both the result and all errors
    /// This is useful for IDE/editor integration where you want to report all errors
    pub fn parse_module_with_recovery(&mut self) -> (Option<Module>, Vec<ParserError>) {
        let start_location = self.current_location();

        // Expect 'module' keyword
        if let Err(e) = self.expect_keyword(Keyword::Module, "expected 'module'") {
            return (None, vec![e]);
        }

        // Parse module name
        let name = match self.parse_identifier() {
            Ok(n) => n,
            Err(e) => return (None, vec![e]),
        };

        let mut imports = Vec::new();
        let mut function_definitions = Vec::new();
        let mut external_functions = Vec::new();
        let mut type_definitions = Vec::new();
        let mut constant_declarations = Vec::new();
        let mut exports = Vec::new();

        // Support both "module name;" (file-scoped) and "module name { }" (inline)
        if self.check(&TokenType::Semicolon) {
            self.advance();

            while !self.is_at_end() {
                if let Err(e) = self.parse_module_item(
                    &mut imports,
                    &mut function_definitions,
                    &mut external_functions,
                    &mut type_definitions,
                    &mut constant_declarations,
                    &mut exports,
                ) {
                    self.add_error(e);
                    self.synchronize_to_module_item();
                }
            }
        } else if self.check(&TokenType::LeftBrace) {
            self.advance();

            while !self.check(&TokenType::RightBrace) && !self.is_at_end() {
                if let Err(e) = self.parse_module_item(
                    &mut imports,
                    &mut function_definitions,
                    &mut external_functions,
                    &mut type_definitions,
                    &mut constant_declarations,
                    &mut exports,
                ) {
                    self.add_error(e);
                    self.synchronize_to_module_item();
                }
            }

            if let Err(e) = self.expect(&TokenType::RightBrace, "expected '}' to close module") {
                self.add_error(e);
            }
        } else {
            self.add_error(ParserError::UnexpectedToken {
                expected: "';' or '{' after module name".to_string(),
                found: format!("{:?}", self.peek().token_type),
                location: self.current_location(),
            });
        }

        let module = Module {
            name,
            intent: None,
            imports,
            exports,
            type_definitions,
            constant_declarations,
            function_definitions,
            external_functions,
            source_location: start_location,
        };

        let errors = self.take_errors();
        (Some(module), errors)
    }

    /// Parse a module definition
    /// Grammar: "module" IDENTIFIER "{" module_item* "}"
    pub fn parse_module(&mut self) -> Result<Module, ParserError> {
        let start_location = self.current_location();

        // Expect 'module' keyword
        self.expect_keyword(Keyword::Module, "expected 'module'")?;

        // Parse module name
        let name = self.parse_identifier()?;

        // Parse module items (imports, functions, structs, etc.)
        let mut imports = Vec::new();
        let mut function_definitions = Vec::new();
        let mut external_functions = Vec::new();
        let mut type_definitions = Vec::new();
        let mut constant_declarations = Vec::new();
        let mut exports = Vec::new();

        // Support both "module name;" (file-scoped) and "module name { }" (inline)
        if self.check(&TokenType::Semicolon) {
            // File-scoped module: "module name;" followed by items at top level
            self.advance(); // consume semicolon

            // Parse remaining items until EOF with error recovery
            while !self.is_at_end() {
                if let Err(e) = self.parse_module_item(
                    &mut imports,
                    &mut function_definitions,
                    &mut external_functions,
                    &mut type_definitions,
                    &mut constant_declarations,
                    &mut exports,
                ) {
                    self.add_error(e);
                    self.synchronize_to_module_item();
                }
            }
        } else if self.check(&TokenType::LeftBrace) {
            // Inline module: "module name { items }"
            self.advance(); // consume '{'

            while !self.check(&TokenType::RightBrace) && !self.is_at_end() {
                if let Err(e) = self.parse_module_item(
                    &mut imports,
                    &mut function_definitions,
                    &mut external_functions,
                    &mut type_definitions,
                    &mut constant_declarations,
                    &mut exports,
                ) {
                    self.add_error(e);
                    self.synchronize_to_module_item();
                }
            }

            // Expect closing brace
            self.expect(&TokenType::RightBrace, "expected '}' to close module")?;
        } else {
            return Err(ParserError::UnexpectedToken {
                expected: "';' or '{' after module name".to_string(),
                found: format!("{:?}", self.peek().token_type),
                location: self.current_location(),
            });
        }

        // If we collected errors during parsing, return the first one
        // (the module is still partially parsed)
        if let Some(first_error) = self.errors.first() {
            return Err(first_error.clone());
        }

        Ok(Module {
            name,
            intent: None,
            imports,
            exports,
            type_definitions,
            constant_declarations,
            function_definitions,
            external_functions,
            source_location: start_location,
        })
    }

    /// Parse a single module item (import, function, struct, enum, extern)
    fn parse_module_item(
        &mut self,
        imports: &mut Vec<ImportStatement>,
        function_definitions: &mut Vec<Function>,
        external_functions: &mut Vec<ExternalFunction>,
        type_definitions: &mut Vec<TypeDefinition>,
        constant_declarations: &mut Vec<ConstantDeclaration>,
        exports: &mut Vec<ExportStatement>,
    ) -> Result<(), ParserError> {
        // Skip visibility modifier if present
        let is_public = if self.check_keyword(Keyword::Pub) {
            self.advance();
            true
        } else {
            false
        };

        // Check for annotation (could be @extern)
        if self.check(&TokenType::At) {
            let annotation = self.parse_annotation()?;

            // Check if this is @extern followed by func
            if annotation.name == "extern" && self.check_keyword(Keyword::Func) {
                external_functions.push(self.parse_external_function(annotation)?);
            } else if self.check_keyword(Keyword::Func) {
                // Annotated function - parse but ignore annotation for now
                // (AST Function struct doesn't have annotations field)
                let func = self.parse_function()?;
                if is_public {
                    exports.push(ExportStatement::Function {
                        name: func.name.clone(),
                        source_location: func.source_location.clone(),
                    });
                }
                function_definitions.push(func);
            } else {
                return Err(ParserError::UnexpectedToken {
                    expected: "func after annotation".to_string(),
                    found: format!("{:?}", self.peek().token_type),
                    location: self.current_location(),
                });
            }
        } else if self.check_keyword(Keyword::Import) {
            imports.push(self.parse_import()?);
        } else if self.check_keyword(Keyword::Func) {
            let func = self.parse_function()?;
            if is_public {
                exports.push(ExportStatement::Function {
                    name: func.name.clone(),
                    source_location: func.source_location.clone(),
                });
            }
            function_definitions.push(func);
        } else if self.check_keyword(Keyword::Struct) {
            let type_def = self.parse_struct()?;
            if is_public {
                match &type_def {
                    TypeDefinition::Structured { name, source_location, .. } => {
                        exports.push(ExportStatement::Type {
                            name: name.clone(),
                            source_location: source_location.clone(),
                        });
                    }
                    _ => {}
                }
            }
            type_definitions.push(type_def);
        } else if self.check_keyword(Keyword::Enum) {
            let type_def = self.parse_enum()?;
            if is_public {
                match &type_def {
                    TypeDefinition::Enumeration { name, source_location, .. } => {
                        exports.push(ExportStatement::Type {
                            name: name.clone(),
                            source_location: source_location.clone(),
                        });
                    }
                    _ => {}
                }
            }
            type_definitions.push(type_def);
        } else if self.check_keyword(Keyword::Const) {
            let constant = self.parse_constant_declaration()?;
            if is_public {
                exports.push(ExportStatement::Constant {
                    name: constant.name.clone(),
                    source_location: constant.source_location.clone(),
                });
            }
            constant_declarations.push(constant);
        } else {
            return Err(ParserError::UnexpectedToken {
                expected: "import, func, struct, enum, const, or annotation".to_string(),
                found: format!("{:?}", self.peek().token_type),
                location: self.current_location(),
            });
        }

        Ok(())
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

    /// Parse a constant declaration
    /// Grammar: "const" IDENTIFIER ":" type "=" expression ";"
    fn parse_constant_declaration(&mut self) -> Result<ConstantDeclaration, ParserError> {
        let start_location = self.current_location();

        self.expect_keyword(Keyword::Const, "expected 'const'")?;
        let name = self.parse_identifier()?;
        self.expect(&TokenType::Colon, "expected ':' after constant name")?;
        let type_spec = self.parse_type()?;
        self.expect(&TokenType::Equal, "expected '=' after type")?;
        let value = self.parse_expression()?;
        self.expect(&TokenType::Semicolon, "expected ';' after constant declaration")?;

        Ok(ConstantDeclaration {
            name,
            type_spec: Box::new(type_spec),
            value: Box::new(value),
            intent: None,
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

    /// Parse a type specifier
    /// Grammar: ownership_type | primitive_type | generic_type | named_type
    pub fn parse_type(&mut self) -> Result<TypeSpecifier, ParserError> {
        let start_location = self.current_location();

        // Check for ownership sigils first: ^ & ~
        if self.check(&TokenType::Caret) {
            self.advance();
            let base_type = self.parse_type()?;
            return Ok(TypeSpecifier::Owned {
                base_type: Box::new(base_type),
                ownership: OwnershipKind::Owned,
                source_location: start_location,
            });
        }

        if self.check(&TokenType::Ampersand) {
            self.advance();
            // Check for &mut
            if self.check_keyword(Keyword::Mut) {
                self.advance();
                let base_type = self.parse_type()?;
                return Ok(TypeSpecifier::Owned {
                    base_type: Box::new(base_type),
                    ownership: OwnershipKind::BorrowedMut,
                    source_location: start_location,
                });
            }
            let base_type = self.parse_type()?;
            return Ok(TypeSpecifier::Owned {
                base_type: Box::new(base_type),
                ownership: OwnershipKind::Borrowed,
                source_location: start_location,
            });
        }

        if self.check(&TokenType::Tilde) {
            self.advance();
            let base_type = self.parse_type()?;
            return Ok(TypeSpecifier::Owned {
                base_type: Box::new(base_type),
                ownership: OwnershipKind::Shared,
                source_location: start_location,
            });
        }

        // Check for primitive types and built-in generic types
        if let TokenType::Keyword(keyword) = &self.peek().token_type {
            match keyword {
                // Primitive types
                Keyword::Int => {
                    self.advance();
                    return Ok(TypeSpecifier::Primitive {
                        type_name: PrimitiveType::Integer,
                        source_location: start_location,
                    });
                }
                Keyword::Int32 => {
                    self.advance();
                    return Ok(TypeSpecifier::Primitive {
                        type_name: PrimitiveType::Integer32,
                        source_location: start_location,
                    });
                }
                Keyword::Int64 => {
                    self.advance();
                    return Ok(TypeSpecifier::Primitive {
                        type_name: PrimitiveType::Integer64,
                        source_location: start_location,
                    });
                }
                Keyword::Float => {
                    self.advance();
                    return Ok(TypeSpecifier::Primitive {
                        type_name: PrimitiveType::Float,
                        source_location: start_location,
                    });
                }
                Keyword::Float32 => {
                    self.advance();
                    return Ok(TypeSpecifier::Primitive {
                        type_name: PrimitiveType::Float32,
                        source_location: start_location,
                    });
                }
                Keyword::Float64 => {
                    self.advance();
                    return Ok(TypeSpecifier::Primitive {
                        type_name: PrimitiveType::Float64,
                        source_location: start_location,
                    });
                }
                Keyword::String_ => {
                    self.advance();
                    return Ok(TypeSpecifier::Primitive {
                        type_name: PrimitiveType::String,
                        source_location: start_location,
                    });
                }
                Keyword::Char => {
                    self.advance();
                    return Ok(TypeSpecifier::Primitive {
                        type_name: PrimitiveType::Char,
                        source_location: start_location,
                    });
                }
                Keyword::Bool => {
                    self.advance();
                    return Ok(TypeSpecifier::Primitive {
                        type_name: PrimitiveType::Boolean,
                        source_location: start_location,
                    });
                }
                Keyword::Void => {
                    self.advance();
                    return Ok(TypeSpecifier::Primitive {
                        type_name: PrimitiveType::Void,
                        source_location: start_location,
                    });
                }
                Keyword::SizeT => {
                    self.advance();
                    return Ok(TypeSpecifier::Primitive {
                        type_name: PrimitiveType::SizeT,
                        source_location: start_location,
                    });
                }

                // Built-in generic types
                Keyword::Array => {
                    self.advance();
                    self.expect(&TokenType::Less, "expected '<' after Array")?;
                    let element_type = self.parse_type()?;
                    self.expect(&TokenType::Greater, "expected '>' to close Array type")?;
                    return Ok(TypeSpecifier::Array {
                        element_type: Box::new(element_type),
                        size: None,
                        source_location: start_location,
                    });
                }
                Keyword::Map => {
                    self.advance();
                    self.expect(&TokenType::Less, "expected '<' after Map")?;
                    let key_type = self.parse_type()?;
                    self.expect(
                        &TokenType::Comma,
                        "expected ',' between Map key and value types",
                    )?;
                    let value_type = self.parse_type()?;
                    self.expect(&TokenType::Greater, "expected '>' to close Map type")?;
                    return Ok(TypeSpecifier::Map {
                        key_type: Box::new(key_type),
                        value_type: Box::new(value_type),
                        source_location: start_location,
                    });
                }
                Keyword::Pointer => {
                    self.advance();
                    self.expect(&TokenType::Less, "expected '<' after Pointer")?;
                    let target_type = self.parse_type()?;
                    self.expect(&TokenType::Greater, "expected '>' to close Pointer type")?;
                    return Ok(TypeSpecifier::Pointer {
                        target_type: Box::new(target_type),
                        is_mutable: false,
                        source_location: start_location,
                    });
                }
                Keyword::MutPointer => {
                    self.advance();
                    self.expect(&TokenType::Less, "expected '<' after MutPointer")?;
                    let target_type = self.parse_type()?;
                    self.expect(&TokenType::Greater, "expected '>' to close MutPointer type")?;
                    return Ok(TypeSpecifier::Pointer {
                        target_type: Box::new(target_type),
                        is_mutable: true,
                        source_location: start_location,
                    });
                }
                _ => {}
            }
        }

        // Named type (user-defined) - possibly with generic arguments
        let name = self.parse_identifier()?;

        // Check for generic type arguments
        if self.check(&TokenType::Less) {
            self.advance();
            let mut type_arguments = Vec::new();

            // Parse first type argument
            type_arguments.push(Box::new(self.parse_type()?));

            // Parse additional type arguments
            while self.check(&TokenType::Comma) {
                self.advance();
                type_arguments.push(Box::new(self.parse_type()?));
            }

            self.expect(&TokenType::Greater, "expected '>' to close generic type")?;

            return Ok(TypeSpecifier::Generic {
                base_type: name,
                type_arguments,
                source_location: start_location,
            });
        }

        Ok(TypeSpecifier::Named {
            name,
            source_location: start_location,
        })
    }

    /// Parse a function definition
    /// Grammar: "func" IDENTIFIER "(" params? ")" ("->" type)? block
    pub fn parse_function(&mut self) -> Result<Function, ParserError> {
        let start_location = self.current_location();

        // Expect 'func' keyword
        self.expect_keyword(Keyword::Func, "expected 'func'")?;

        // Parse function name
        let name = self.parse_identifier()?;

        // Expect opening parenthesis
        self.expect(&TokenType::LeftParen, "expected '(' after function name")?;

        // Parse parameters
        let parameters = self.parse_parameters()?;

        // Expect closing parenthesis
        self.expect(&TokenType::RightParen, "expected ')' after parameters")?;

        // Parse optional return type
        let return_type = if self.check(&TokenType::Arrow) {
            self.advance();
            self.parse_type()?
        } else {
            // Default to Void if no return type specified
            TypeSpecifier::Primitive {
                type_name: PrimitiveType::Void,
                source_location: self.current_location(),
            }
        };

        // Parse function body
        let body = self.parse_block()?;

        Ok(Function {
            name,
            intent: None,
            generic_parameters: Vec::new(),
            parameters,
            return_type: Box::new(return_type),
            metadata: FunctionMetadata {
                preconditions: Vec::new(),
                postconditions: Vec::new(),
                invariants: Vec::new(),
                algorithm_hint: None,
                performance_expectation: None,
                complexity_expectation: None,
                throws_exceptions: Vec::new(),
                thread_safe: None,
                may_block: None,
            },
            body,
            export_info: None,
            source_location: start_location,
        })
    }

    /// Parse function parameters
    /// Grammar: param ("," param)*
    /// param: IDENTIFIER ":" type
    fn parse_parameters(&mut self) -> Result<Vec<Parameter>, ParserError> {
        let mut parameters = Vec::new();

        // Check for empty parameter list
        if self.check(&TokenType::RightParen) {
            return Ok(parameters);
        }

        // Parse first parameter
        parameters.push(self.parse_parameter()?);

        // Parse additional parameters
        while self.check(&TokenType::Comma) {
            self.advance();
            parameters.push(self.parse_parameter()?);
        }

        Ok(parameters)
    }

    /// Parse a single parameter
    /// Grammar: IDENTIFIER ":" type
    fn parse_parameter(&mut self) -> Result<Parameter, ParserError> {
        let start_location = self.current_location();

        // Parse parameter name
        let name = self.parse_identifier()?;

        // Expect colon
        self.expect(&TokenType::Colon, "expected ':' after parameter name")?;

        // Parse parameter type
        let param_type = self.parse_type()?;

        Ok(Parameter {
            name,
            param_type: Box::new(param_type),
            intent: None,
            constraint: None,
            passing_mode: PassingMode::ByValue,
            source_location: start_location,
        })
    }

    /// Parse a block of statements
    /// Grammar: "{" statement* "}"
    pub fn parse_block(&mut self) -> Result<Block, ParserError> {
        let start_location = self.current_location();

        self.expect(&TokenType::LeftBrace, "expected '{' to start block")?;

        let mut statements = Vec::new();

        while !self.check(&TokenType::RightBrace) && !self.is_at_end() {
            statements.push(self.parse_statement()?);
        }

        self.expect(&TokenType::RightBrace, "expected '}' to end block")?;

        Ok(Block {
            statements,
            source_location: start_location,
        })
    }

    /// Parse an annotation
    /// Grammar: "@" IDENTIFIER ("(" annotation_args ")")?
    pub fn parse_annotation(&mut self) -> Result<Annotation, ParserError> {
        let start_location = self.current_location();

        // Expect '@'
        self.expect(&TokenType::At, "expected '@'")?;

        // Parse annotation name
        let name = self.parse_identifier()?;

        // Parse optional arguments
        let mut arguments = Vec::new();
        if self.check(&TokenType::LeftParen) {
            self.advance();

            // Parse arguments until closing paren
            if !self.check(&TokenType::RightParen) {
                arguments.push(self.parse_annotation_argument()?);

                while self.check(&TokenType::Comma) {
                    self.advance();
                    arguments.push(self.parse_annotation_argument()?);
                }
            }

            self.expect(
                &TokenType::RightParen,
                "expected ')' after annotation arguments",
            )?;
        }

        Ok(Annotation {
            name: name.name,
            arguments,
            source_location: start_location,
        })
    }

    /// Parse an annotation argument (key: value or just value)
    fn parse_annotation_argument(&mut self) -> Result<AnnotationArgument, ParserError> {
        let start_location = self.current_location();

        // Check if this is a labeled argument (key: value)
        if let TokenType::Identifier(name) = &self.peek().token_type {
            let name_clone = name.clone();
            if let Some(next) = self.peek_next() {
                if matches!(next.token_type, TokenType::Colon) {
                    // Labeled argument
                    self.advance(); // consume identifier
                    self.advance(); // consume colon
                    let value = self.parse_annotation_value()?;
                    return Ok(AnnotationArgument {
                        label: Some(name_clone),
                        value,
                        source_location: start_location,
                    });
                }
            }
        }

        // Unlabeled argument
        let value = self.parse_annotation_value()?;
        Ok(AnnotationArgument {
            label: None,
            value,
            source_location: start_location,
        })
    }

    /// Parse an annotation value (string literal, identifier, or braced expression)
    fn parse_annotation_value(&mut self) -> Result<AnnotationValue, ParserError> {
        // String literal
        if let TokenType::StringLiteral(s) = &self.peek().token_type {
            let value = s.clone();
            self.advance();
            return Ok(AnnotationValue::String(value));
        }

        // Integer literal
        if let TokenType::IntegerLiteral(n) = &self.peek().token_type {
            let value = *n;
            self.advance();
            return Ok(AnnotationValue::Integer(value));
        }

        // Boolean literal
        if let TokenType::BoolLiteral(b) = &self.peek().token_type {
            let value = *b;
            self.advance();
            return Ok(AnnotationValue::Boolean(value));
        }

        // Braced expression (for contracts like @requires({n > 0}))
        if self.check(&TokenType::LeftBrace) {
            let start = self.current_location();
            self.advance();

            // Collect tokens until matching brace
            let mut expr_tokens = String::new();
            let mut brace_depth = 1;

            while !self.is_at_end() && brace_depth > 0 {
                if self.check(&TokenType::LeftBrace) {
                    brace_depth += 1;
                    expr_tokens.push('{');
                } else if self.check(&TokenType::RightBrace) {
                    brace_depth -= 1;
                    if brace_depth > 0 {
                        expr_tokens.push('}');
                    }
                } else {
                    expr_tokens.push_str(&self.peek().lexeme);
                    expr_tokens.push(' ');
                }
                if brace_depth > 0 {
                    self.advance();
                }
            }

            self.expect(&TokenType::RightBrace, "expected '}' to close expression")?;
            return Ok(AnnotationValue::Expression(
                expr_tokens.trim().to_string(),
                start,
            ));
        }

        // Identifier
        if let TokenType::Identifier(name) = &self.peek().token_type {
            let value = name.clone();
            self.advance();
            return Ok(AnnotationValue::Identifier(value));
        }

        Err(ParserError::UnexpectedToken {
            expected: "annotation value".to_string(),
            found: format!("{:?}", self.peek().token_type),
            location: self.current_location(),
        })
    }

    /// Parse an external function declaration
    /// Grammar: "@extern" "(" args ")" "func" IDENTIFIER "(" params ")" "->" type ";"
    pub fn parse_external_function(
        &mut self,
        annotation: Annotation,
    ) -> Result<ExternalFunction, ParserError> {
        let start_location = annotation.source_location.clone();

        // Extract library from annotation
        let library = annotation
            .arguments
            .iter()
            .find(|a| a.label.as_deref() == Some("library"))
            .and_then(|a| match &a.value {
                AnnotationValue::String(s) => Some(s.clone()),
                _ => None,
            })
            .unwrap_or_else(|| "STATIC".to_string());

        // Extract optional symbol
        let symbol = annotation
            .arguments
            .iter()
            .find(|a| a.label.as_deref() == Some("symbol"))
            .and_then(|a| match &a.value {
                AnnotationValue::String(s) => Some(s.clone()),
                _ => None,
            });

        // Expect 'func' keyword
        self.expect_keyword(Keyword::Func, "expected 'func' after @extern annotation")?;

        // Parse function name
        let name = self.parse_identifier()?;

        // Parse parameters
        self.expect(&TokenType::LeftParen, "expected '(' after function name")?;
        let parameters = self.parse_parameters()?;
        self.expect(&TokenType::RightParen, "expected ')' after parameters")?;

        // Parse return type (required for extern functions)
        self.expect(
            &TokenType::Arrow,
            "expected '->' for extern function return type",
        )?;
        let return_type = self.parse_type()?;

        // Expect semicolon (no body for extern functions)
        self.expect(
            &TokenType::Semicolon,
            "expected ';' after extern function declaration",
        )?;

        Ok(ExternalFunction {
            name,
            library,
            symbol,
            parameters,
            return_type: Box::new(return_type),
            calling_convention: CallingConvention::C,
            thread_safe: false,
            may_block: false,
            variadic: false,
            ownership_info: None,
            source_location: start_location,
        })
    }

    // ==================== STRUCT PARSING ====================

    /// Parse a struct definition
    /// Grammar: "struct" IDENTIFIER "{" field* "}"
    pub fn parse_struct(&mut self) -> Result<TypeDefinition, ParserError> {
        let start_location = self.current_location();

        self.expect_keyword(Keyword::Struct, "expected 'struct'")?;

        let name = self.parse_identifier()?;

        self.expect(&TokenType::LeftBrace, "expected '{' after struct name")?;

        let mut fields = Vec::new();

        while !self.check(&TokenType::RightBrace) && !self.is_at_end() {
            fields.push(self.parse_struct_field()?);
        }

        self.expect(&TokenType::RightBrace, "expected '}' to close struct")?;

        Ok(TypeDefinition::Structured {
            name,
            intent: None,
            generic_parameters: vec![],
            fields,
            export_as: None,
            source_location: start_location,
        })
    }

    /// Parse a struct field
    /// Grammar: IDENTIFIER ":" type [","]
    fn parse_struct_field(&mut self) -> Result<StructField, ParserError> {
        let start_location = self.current_location();

        let name = self.parse_identifier()?;

        self.expect(&TokenType::Colon, "expected ':' after field name")?;

        let field_type = self.parse_type()?;

        // Accept comma or semicolon as field separator (optional for last field)
        if self.check(&TokenType::Comma) || self.check(&TokenType::Semicolon) {
            self.advance();
        }

        Ok(StructField {
            name,
            field_type: Box::new(field_type),
            source_location: start_location,
        })
    }

    // ==================== ENUM PARSING ====================

    /// Parse an enum definition
    /// Grammar: "enum" IDENTIFIER "{" ("case" IDENTIFIER ["(" type ")"] ";")* "}"
    pub fn parse_enum(&mut self) -> Result<TypeDefinition, ParserError> {
        let start_location = self.current_location();

        self.expect_keyword(Keyword::Enum, "expected 'enum'")?;

        let name = self.parse_identifier()?;

        self.expect(&TokenType::LeftBrace, "expected '{' after enum name")?;

        let mut variants = Vec::new();

        while !self.check(&TokenType::RightBrace) && !self.is_at_end() {
            variants.push(self.parse_enum_variant()?);
        }

        self.expect(&TokenType::RightBrace, "expected '}' to close enum")?;

        Ok(TypeDefinition::Enumeration {
            name,
            intent: None,
            generic_parameters: vec![],
            variants,
            source_location: start_location,
        })
    }

    /// Parse an enum variant
    /// Grammar: ["case"] IDENTIFIER ["(" type ["," type]* ")"] [";" | ","]
    fn parse_enum_variant(&mut self) -> Result<EnumVariant, ParserError> {
        let start_location = self.current_location();

        // Expect 'case' keyword
        self.expect_keyword(Keyword::Case, "expected 'case' before enum variant")?;

        let name = self.parse_identifier()?;

        // Parse optional associated types
        let associated_types = if self.check(&TokenType::LeftParen) {
            self.advance();
            let mut types = Vec::new();
            loop {
                types.push(self.parse_type()?);
                if self.check(&TokenType::Comma) {
                    self.advance();
                } else {
                    break;
                }
            }
            self.expect(&TokenType::RightParen, "expected ')' after associated types")?;
            types
        } else {
            Vec::new()
        };

        // Expect delimiter (semicolon or comma)
        if self.check(&TokenType::Semicolon) {
            self.advance();
        } else if self.check(&TokenType::Comma) {
            self.advance();
        } else {
            return Err(ParserError::UnexpectedToken {
                expected: "; or ,".to_string(),
                found: format!("{:?}", self.peek().token_type),
                location: self.current_location(),
            });
        }

        Ok(EnumVariant {
            name,
            associated_types,
            source_location: start_location,
        })
    }

    // ==================== VARIABLE DECLARATION PARSING ====================

    /// Parse a variable declaration
    /// Grammar: "let" ["mut"] IDENTIFIER [":" type] ["=" expression] ";"
    pub fn parse_variable_declaration(&mut self) -> Result<Statement, ParserError> {
        let start_location = self.current_location();

        // Check for 'let' or 'var' keyword
        let is_var = self.check_keyword(Keyword::Var);
        if is_var {
            self.advance(); // consume 'var'
        } else {
            self.expect_keyword(Keyword::Let, "expected 'let' or 'var'")?;
        }

        // Check for 'mut' keyword (only valid after 'let', 'var' is implicitly mutable)
        let mutability = if is_var {
            Mutability::Mutable
        } else if self.check_keyword(Keyword::Mut) {
            self.advance();
            Mutability::Mutable
        } else {
            Mutability::Immutable
        };

        // Parse variable name
        let name = self.parse_identifier()?;

        // Parse optional type annotation
        let type_spec = if self.check(&TokenType::Colon) {
            self.advance();
            self.parse_type()?
        } else {
            // If no type annotation, we need an initializer to infer type
            // For now, default to a placeholder type that semantic analysis will resolve
            TypeSpecifier::Named {
                name: Identifier {
                    name: "_inferred".to_string(),
                    source_location: start_location.clone(),
                },
                source_location: start_location.clone(),
            }
        };

        // Parse optional initializer
        let initial_value = if self.check(&TokenType::Equal) {
            self.advance();
            Some(Box::new(self.parse_expression()?))
        } else {
            None
        };

        // Expect semicolon
        self.expect(
            &TokenType::Semicolon,
            "expected ';' after variable declaration",
        )?;

        Ok(Statement::VariableDeclaration {
            name,
            type_spec: Box::new(type_spec),
            mutability,
            initial_value,
            intent: None,
            source_location: start_location,
        })
    }

    // ==================== ASSIGNMENT PARSING ====================

    /// Check if the current statement looks like an assignment
    fn looks_like_assignment(&self) -> bool {
        let mut i = self.position;
        
        // Must start with identifier
        if i >= self.tokens.len() { return false; }
        if !matches!(self.tokens[i].token_type, TokenType::Identifier(_)) {
            return false;
        }
        i += 1;
        
        while i < self.tokens.len() {
            match &self.tokens[i].token_type {
                TokenType::Dot => {
                    i += 1;
                    // Expect identifier after dot
                    if i < self.tokens.len() && matches!(self.tokens[i].token_type, TokenType::Identifier(_)) {
                        i += 1;
                    } else {
                        return false; 
                    }
                },
                TokenType::LeftBracket => {
                    // Skip until RightBracket (balanced)
                    i += 1;
                    let mut depth = 1;
                    while i < self.tokens.len() && depth > 0 {
                        match &self.tokens[i].token_type {
                            TokenType::LeftBracket => depth += 1,
                            TokenType::RightBracket => depth -= 1,
                            _ => {}
                        }
                        i += 1;
                    }
                },
                TokenType::Equal => return true, // Found assignment operator
                
                TokenType::LeftParen => return false, // Method/Function call
                TokenType::Semicolon => return false, // Expression statement
                _ => return false, // Unexpected token for assignment target
            }
        }
        false
    }

    /// Parse an assignment statement
    /// Grammar: assignment_target "=" expression ";"
    pub fn parse_assignment(&mut self) -> Result<Statement, ParserError> {
        let start_location = self.current_location();

        // Parse the target (for now, just variable names)
        let target = self.parse_assignment_target()?;

        // Expect '='
        self.expect(&TokenType::Equal, "expected '=' in assignment")?;

        // Parse the value expression
        let value = self.parse_expression()?;

        // Expect semicolon
        self.expect(&TokenType::Semicolon, "expected ';' after assignment")?;

        Ok(Statement::Assignment {
            target,
            value: Box::new(value),
            source_location: start_location,
        })
    }

    /// Parse an assignment target
    /// For now, supports simple variables. Will expand for array[i], struct.field, etc.
    fn parse_assignment_target(&mut self) -> Result<AssignmentTarget, ParserError> {
        let start_location = self.current_location();

        // Simple variable target
        if let TokenType::Identifier(name) = &self.peek().token_type {
            let name = name.clone();
            self.advance();

            // Check for array index access: name[index]
            if self.check(&TokenType::LeftBracket) {
                self.advance();
                let index = self.parse_expression()?;
                self.expect(&TokenType::RightBracket, "expected ']' after array index")?;

                return Ok(AssignmentTarget::ArrayElement {
                    array: Box::new(Expression::Variable {
                        name: Identifier {
                            name,
                            source_location: start_location.clone(),
                        },
                        source_location: start_location.clone(),
                    }),
                    index: Box::new(index),
                });
            }

            // Check for field access: name.field
            if self.check(&TokenType::Dot) {
                self.advance();
                let field_name = self.parse_identifier()?;

                return Ok(AssignmentTarget::StructField {
                    instance: Box::new(Expression::Variable {
                        name: Identifier {
                            name,
                            source_location: start_location.clone(),
                        },
                        source_location: start_location.clone(),
                    }),
                    field_name,
                });
            }

            return Ok(AssignmentTarget::Variable {
                name: Identifier {
                    name,
                    source_location: start_location,
                },
            });
        }

        Err(ParserError::UnexpectedToken {
            expected: "assignment target".to_string(),
            found: format!("{:?}", self.peek().token_type),
            location: start_location,
        })
    }

    // ==================== CONTROL FLOW PARSING ====================

    /// Parse a return statement
    /// Grammar: "return" [expression] ";"
    pub fn parse_return_statement(&mut self) -> Result<Statement, ParserError> {
        let start_location = self.current_location();

        self.expect_keyword(Keyword::Return, "expected 'return'")?;

        // Check for optional return value
        let value = if !self.check(&TokenType::Semicolon) {
            Some(Box::new(self.parse_expression()?))
        } else {
            None
        };

        self.expect(&TokenType::Semicolon, "expected ';' after return statement")?;

        Ok(Statement::Return {
            value,
            source_location: start_location,
        })
    }

    /// Parse a when statement (V2 if/else)
    /// Grammar: "when" expression block ["else" "when" expression block]* ["else" block]
    pub fn parse_when_statement(&mut self) -> Result<Statement, ParserError> {
        let start_location = self.current_location();

        self.expect_keyword(Keyword::When, "expected 'when'")?;

        // Parse condition
        let condition = self.parse_expression()?;

        // Parse then block
        let then_block = self.parse_block()?;

        // Parse else-ifs and else
        let mut else_ifs = Vec::new();
        let mut else_block = None;

        while self.check_keyword(Keyword::Else) {
            self.advance(); // consume 'else'

            if self.check_keyword(Keyword::When) {
                // else when (else if)
                let else_if_location = self.current_location();
                self.advance(); // consume 'when'

                let else_if_condition = self.parse_expression()?;
                let else_if_block = self.parse_block()?;

                else_ifs.push(ElseIf {
                    condition: Box::new(else_if_condition),
                    block: else_if_block,
                    source_location: else_if_location,
                });
            } else {
                // Final else block
                else_block = Some(self.parse_block()?);
                break;
            }
        }

        Ok(Statement::If {
            condition: Box::new(condition),
            then_block,
            else_ifs,
            else_block,
            source_location: start_location,
        })
    }

    /// Parse a while loop
    /// Grammar: "while" expression block
    pub fn parse_while_loop(&mut self) -> Result<Statement, ParserError> {
        let start_location = self.current_location();

        self.expect_keyword(Keyword::While, "expected 'while'")?;

        let condition = self.parse_expression()?;
        let body = self.parse_block()?;

        Ok(Statement::WhileLoop {
            condition: Box::new(condition),
            invariant: None,
            body,
            label: None,
            source_location: start_location,
        })
    }

    /// Parse a for loop
    /// Grammar: "for" identifier "in" expression block
    pub fn parse_for_loop(&mut self) -> Result<Statement, ParserError> {
        let start_location = self.current_location();

        self.expect_keyword(Keyword::For, "expected 'for'")?;

        // Parse the element binding (loop variable)
        let element_binding = self.parse_identifier()?;

        // Optional type annotation: for x: T in ...
        let element_type = if self.check(&TokenType::Colon) {
            self.advance();
            Some(Box::new(self.parse_type()?))
        } else {
            None
        };

        // Expect 'in' keyword
        self.expect_keyword(Keyword::In, "expected 'in' after loop variable")?;

        // Parse the start of the range or collection expression
        let from_expr = self.parse_primary_expression()?;

        // Check if this is a range expression (.. or ..=)
        if self.check(&TokenType::DotDot) || self.check(&TokenType::DotDotEqual) {
            let inclusive = self.check(&TokenType::DotDotEqual);
            self.advance(); // consume .. or ..=

            // Parse the end of the range
            let to_expr = self.parse_primary_expression()?;

            // Parse the loop body
            let body = self.parse_block()?;

            Ok(Statement::FixedIterationLoop {
                counter: element_binding,
                from_value: Box::new(from_expr),
                to_value: Box::new(to_expr),
                step_value: None,
                inclusive,
                body,
                label: None,
                source_location: start_location,
            })
        } else {
            // This is a for-each loop over a collection
            let collection = Box::new(from_expr);

            // Parse the loop body
            let body = self.parse_block()?;

            let default_type = Box::new(TypeSpecifier::Primitive {
                type_name: PrimitiveType::Integer,
                source_location: start_location.clone(),
            });

            Ok(Statement::ForEachLoop {
                collection,
                element_binding,
                element_type: element_type.unwrap_or(default_type),
                index_binding: None,
                body,
                label: None,
                source_location: start_location,
            })
        }
    }

    /// Parse a match statement
    /// Grammar: "match" expr "{" (pattern "=>" block)* "}"
    pub fn parse_match_statement(&mut self) -> Result<Statement, ParserError> {
        let start_location = self.current_location();

        self.expect_keyword(Keyword::Match, "expected 'match'")?;

        // Parse the value being matched
        let value = self.parse_expression()?;

        // Expect opening brace
        self.expect(&TokenType::LeftBrace, "expected '{' after match value")?;

        // Parse match arms
        let mut arms = Vec::new();
        while !self.check(&TokenType::RightBrace) && !self.is_at_end() {
            let arm_location = self.current_location();

            // Parse the pattern
            let pattern = self.parse_pattern()?;

            // Check for optional guard: `if { condition }`
            let guard = if self.check_keyword(Keyword::If) {
                self.advance(); // consume 'if'
                // Parse the guard condition (a block expression like `{x < 0}`)
                let guard_expr = self.parse_expression()?;
                Some(Box::new(guard_expr))
            } else {
                None
            };

            // Expect =>
            self.expect(&TokenType::FatArrow, "expected '=>' after pattern")?;

            // Parse the body block
            let body = self.parse_block()?;

            arms.push(MatchArm {
                pattern,
                guard,
                body,
                source_location: arm_location,
            });
        }

        // Expect closing brace
        self.expect(&TokenType::RightBrace, "expected '}' after match arms")?;

        Ok(Statement::Match {
            value: Box::new(value),
            arms,
            source_location: start_location,
        })
    }

    /// Parse a break statement
    /// Grammar: "break" ";"
    pub fn parse_break_statement(&mut self) -> Result<Statement, ParserError> {
        let start_location = self.current_location();

        self.expect_keyword(Keyword::Break, "expected 'break'")?;
        self.expect(&TokenType::Semicolon, "expected ';' after break")?;

        Ok(Statement::Break {
            target_label: None,
            source_location: start_location,
        })
    }

    /// Parse a continue statement
    /// Grammar: "continue" ";"
    pub fn parse_continue_statement(&mut self) -> Result<Statement, ParserError> {
        let start_location = self.current_location();

        self.expect_keyword(Keyword::Continue, "expected 'continue'")?;
        self.expect(&TokenType::Semicolon, "expected ';' after continue")?;

        Ok(Statement::Continue {
            target_label: None,
            source_location: start_location,
        })
    }

    /// Parse a concurrent block
    /// Grammar: "concurrent" block
    pub fn parse_concurrent_block(&mut self) -> Result<Statement, ParserError> {
        let start_location = self.current_location();

        self.expect_keyword(Keyword::Concurrent, "expected 'concurrent'")?;

        let block = self.parse_block()?;

        Ok(Statement::Concurrent {
            block,
            source_location: start_location,
        })
    }

    /// Parse any statement based on the leading token
    pub fn parse_statement(&mut self) -> Result<Statement, ParserError> {
        // Check for keywords that start statements
        if self.check_keyword(Keyword::Let) || self.check_keyword(Keyword::Var) {
            return self.parse_variable_declaration();
        }
        if self.check_keyword(Keyword::Return) {
            return self.parse_return_statement();
        }
        if self.check_keyword(Keyword::When) {
            return self.parse_when_statement();
        }
        if self.check_keyword(Keyword::While) {
            return self.parse_while_loop();
        }
        if self.check_keyword(Keyword::For) {
            return self.parse_for_loop();
        }
        if self.check_keyword(Keyword::Match) {
            return self.parse_match_statement();
        }
        if self.check_keyword(Keyword::Break) {
            return self.parse_break_statement();
        }
        if self.check_keyword(Keyword::Continue) {
            return self.parse_continue_statement();
        }
        if self.check_keyword(Keyword::Concurrent) {
            return self.parse_concurrent_block();
        }

        // Otherwise, try to parse as assignment or expression statement
        // Check if it looks like an assignment
        if self.looks_like_assignment() {
            return self.parse_assignment();
        }

        // Default: expression statement
        let start_location = self.current_location();
        let expr = self.parse_expression()?;
        self.expect(&TokenType::Semicolon, "expected ';' after expression")?;

        Ok(Statement::Expression {
            expr: Box::new(expr),
            source_location: start_location,
        })
    }

    // ==================== EXPRESSION PARSING ====================

    /// Parse an expression
    /// Supports: literals, identifiers, braced binary expressions `{a + b}`
    pub fn parse_expression(&mut self) -> Result<Expression, ParserError> {
        let start_location = self.current_location();

        // Braced binary expression: {left op right} or Map Literal
        if self.check(&TokenType::LeftBrace) {
            if self.looks_like_map_literal() {
                return self.parse_map_literal();
            }
            return self.parse_braced_expression();
        }

        // Logical NOT: !expr
        if self.check(&TokenType::Bang) {
            self.advance(); // consume '!'
            let operand = self.parse_expression()?;
            return Ok(Expression::LogicalNot {
                operand: Box::new(operand),
                source_location: start_location,
            });
        }

        // Unary minus: -expr
        if self.check(&TokenType::Minus) {
            self.advance(); // consume '-'
            let operand = self.parse_expression()?;
            return Ok(Expression::Negate {
                operand: Box::new(operand),
                source_location: start_location,
            });
        }

        // Address of: &expr or &mut expr
        if self.check(&TokenType::Ampersand) {
            self.advance(); // consume '&'
            
            // Check for 'mut' (which is a keyword, but we need to see if it's handled by parser)
            // In V2, mut is a keyword.
            let is_mut = if self.check_keyword(Keyword::Mut) {
                self.advance(); // consume 'mut'
                true
            } else {
                false
            };
            
            let operand = self.parse_expression()?;
            
            // If it was &mut, we might need a different expression type or flag
            // For now, let's map it to AddressOf with a potential flag if supported,
            // or just AddressOf for now. 
            // Wait, AddressOf usually implies immutable borrow. 
            // If the AST supports MutableBorrow, we should use that.
            // Let's check AST.
            return Ok(Expression::AddressOf {
                operand: Box::new(operand),
                mutability: is_mut,
                source_location: start_location,
            });
        }
        
        // Move: ^expr
        if self.check(&TokenType::Caret) {
            self.advance(); // consume '^'
            let operand = self.parse_expression()?;
            // Move is explicit in V2
            // Map to a Move expression if it exists, or maybe just the expression itself if move is default?
            // Actually, `^` might be a dereference in some languages, but here it says "owned".
            // Let's see if we have a Move expression. If not, maybe it's just a marker.
            // But wait, `^` is also XOR.
            // If used as prefix, it's Move/Owned.
            // Let's check if we have an AST node for this.
            // For now, let's assume it's just an expression wrapper or we need to handle it.
            // If no AST node, maybe it's just processed as part of the type checking/move semantics?
            // Let's look for "Move" in AST.
            return Ok(operand); // Just return the operand for now, treating ^ as a semantic marker handled elsewhere? 
            // Or we need to implement it.
        }

        // Prefix range expression: ..end or ..=end
        if self.check(&TokenType::DotDot) || self.check(&TokenType::DotDotEqual) {
            return self.parse_range_expression(None, start_location);
        }

        // Integer literal (with range check)
        if let TokenType::IntegerLiteral(value) = &self.peek().token_type {
            let value = *value;
            self.advance();
            let expr = Expression::IntegerLiteral {
                value,
                source_location: start_location.clone(),
            };
            // Check for range: integer..end or integer..=end
            if self.check(&TokenType::DotDot) || self.check(&TokenType::DotDotEqual) {
                return self.parse_range_expression(Some(expr), start_location);
            }
            return Ok(expr);
        }

        // Float literal
        if let TokenType::FloatLiteral(value) = &self.peek().token_type {
            let value = *value;
            self.advance();
            return Ok(Expression::FloatLiteral {
                value,
                source_location: start_location,
            });
        }

        // String literal
        if let TokenType::StringLiteral(value) = &self.peek().token_type {
            let value = value.clone();
            self.advance();
            return Ok(Expression::StringLiteral {
                value,
                source_location: start_location,
            });
        }

        // Character literal
        if let TokenType::CharLiteral(value) = &self.peek().token_type {
            let value = *value;
            self.advance();
            return Ok(Expression::CharacterLiteral {
                value,
                source_location: start_location,
            });
        }

        // Boolean literal
        if let TokenType::BoolLiteral(value) = &self.peek().token_type {
            let value = *value;
            self.advance();
            return Ok(Expression::BooleanLiteral {
                value,
                source_location: start_location,
            });
        }

        // Identifier (variable reference, struct construction, enum variant, or function call)
        if let TokenType::Identifier(name) = &self.peek().token_type {
            let name = name.clone();
            self.advance();

            // Check for enum variant: EnumType::Variant or EnumType::Variant(value)
            if self.check(&TokenType::DoubleColon) {
                return self.parse_enum_variant_expression(name, start_location);
            }

            // Check for struct construction: TypeName { field: value, ... }
            // Only parse as struct construction if it looks like one (has field: value syntax)
            if self.check(&TokenType::LeftBrace) && self.looks_like_struct_construction() {
                return self.parse_struct_construction(name, start_location);
            }

            let mut expr = Expression::Variable {
                name: Identifier {
                    name,
                    source_location: start_location.clone(),
                },
                source_location: start_location.clone(),
            };

            // Handle postfix operators: (args), [index], and .field
            loop {
                if self.check(&TokenType::LeftParen) {
                    // Function call: expr(args)
                    self.advance(); // consume '('
                    let mut arg_exprs = Vec::new();

                    if !self.check(&TokenType::RightParen) {
                        // Check for labeled argument: label: expr
                        let is_labeled = match &self.peek().token_type {
                            TokenType::Identifier(_) | TokenType::Keyword(_) => {
                                self.peek_next().map(|t| t.token_type == TokenType::Colon).unwrap_or(false)
                            },
                            _ => false
                        };

                        if is_labeled {
                            self.advance(); // consume label
                            self.advance(); // consume colon
                        }
                        
                        arg_exprs.push(self.parse_expression()?);
                        
                        while self.check(&TokenType::Comma) {
                            self.advance(); // consume ','
                            
                            let is_labeled = match &self.peek().token_type {
                                TokenType::Identifier(_) | TokenType::Keyword(_) => {
                                    self.peek_next().map(|t| t.token_type == TokenType::Colon).unwrap_or(false)
                                },
                                _ => false
                            };

                            if is_labeled {
                                self.advance(); // consume label
                                self.advance(); // consume colon
                            }
                            
                            arg_exprs.push(self.parse_expression()?);
                        }
                    }
                    self.expect(&TokenType::RightParen, "expected ')'")?;

                    // Extract function name from variable expression
                    let function_name = match &expr {
                        Expression::Variable { name, .. } => name.clone(),
                        _ => {
                            return Err(ParserError::UnexpectedToken {
                                expected: "function name".to_string(),
                                found: "complex expression".to_string(),
                                location: start_location.clone(),
                            });
                        }
                    };

                    // Convert expressions to Argument structs with placeholder names
                    let arguments: Vec<Argument> = arg_exprs
                        .into_iter()
                        .enumerate()
                        .map(|(i, e)| Argument {
                            parameter_name: Identifier::new(
                                format!("arg_{}", i),
                                start_location.clone(),
                            ),
                            value: Box::new(e),
                            source_location: start_location.clone(),
                        })
                        .collect();

                    expr = Expression::FunctionCall {
                        call: AstFunctionCall {
                            function_reference: FunctionReference::Local {
                                name: function_name,
                            },
                            arguments,
                            variadic_arguments: Vec::new(),
                        },
                        source_location: start_location.clone(),
                    };
                } else if self.check(&TokenType::LeftBracket) {
                    // Array indexing: expr[index]
                    self.advance(); // consume '['
                    let index = self.parse_expression()?;
                    self.expect(&TokenType::RightBracket, "expected ']'")?;
                    expr = Expression::ArrayAccess {
                        array: Box::new(expr),
                        index: Box::new(index),
                        source_location: start_location.clone(),
                    };
                } else if self.check(&TokenType::Dot) {
                    // Field access or method call: expr.field or expr.method(args)
                    self.advance(); // consume '.'
                    if let TokenType::Identifier(member_name) = &self.peek().token_type {
                        let member_name = member_name.clone();
                        let member_loc = self.current_location();
                        self.advance();

                        // Check if this is a method call (followed by '(')
                        if self.check(&TokenType::LeftParen) {
                            // Method call: expr.method(args)
                            self.advance(); // consume '('
                            let mut arg_exprs = Vec::new();

                            if !self.check(&TokenType::RightParen) {
                                // Parse first argument
                                arg_exprs.push(self.parse_expression()?);

                                // Parse remaining arguments
                                while self.check(&TokenType::Comma) {
                                    self.advance(); // consume ','
                                    arg_exprs.push(self.parse_expression()?);
                                }
                            }
                            self.expect(
                                &TokenType::RightParen,
                                "expected ')' after method arguments",
                            )?;

                            // Convert expressions to Argument structs
                            let arguments: Vec<Argument> = arg_exprs
                                .into_iter()
                                .enumerate()
                                .map(|(i, e)| Argument {
                                    parameter_name: Identifier::new(
                                        format!("arg_{}", i),
                                        start_location.clone(),
                                    ),
                                    value: Box::new(e),
                                    source_location: start_location.clone(),
                                })
                                .collect();

                            expr = Expression::MethodCall {
                                receiver: Box::new(expr),
                                method_name: Identifier {
                                    name: member_name,
                                    source_location: member_loc,
                                },
                                arguments,
                                source_location: start_location.clone(),
                            };
                        } else {
                            // Field access: expr.field
                            expr = Expression::FieldAccess {
                                instance: Box::new(expr),
                                field_name: Identifier {
                                    name: member_name,
                                    source_location: member_loc,
                                },
                                source_location: start_location.clone(),
                            };
                        }
                    } else {
                        return Err(ParserError::UnexpectedToken {
                            expected: "field name or method name".to_string(),
                            found: format!("{:?}", self.peek().token_type),
                            location: self.current_location(),
                        });
                    }
                } else {
                    break;
                }
            }

            return Ok(expr);
        }

        // Array literal or closure with capture list
        if self.check(&TokenType::LeftBracket) {
            // Determine if this is an array literal or capture list
            // Array literal: [1, 2, 3] or [expr, expr, ...]
            // Capture list: [x, y](params) => body (followed by '(')
            if self.looks_like_array_literal() {
                return self.parse_array_literal();
            } else {
                // Parse as capture list for closure
                let captures = self.parse_capture_list()?;
                // After capture list, we must have parameters in parens
                if !self.check(&TokenType::LeftParen) {
                    return Err(ParserError::SyntaxError {
                        message: "expected '(' after capture list".to_string(),
                        location: self.current_location(),
                        suggestion: Some("use [captures](params) => body syntax".to_string()),
                    });
                }
                return self.parse_paren_expr_or_lambda_with_captures(captures);
            }
        }

        // Parenthesized expression or lambda
        if self.check(&TokenType::LeftParen) {
            return self.parse_paren_expr_or_lambda();
        }

        Err(ParserError::UnexpectedToken {
            expected: "expression".to_string(),
            found: format!("{:?}", self.peek().token_type),
            location: start_location,
        })
    }

    /// Parse a braced binary expression: `{left op right}`
    /// V2 syntax requires binary operations to be wrapped in braces
    fn parse_braced_expression(&mut self) -> Result<Expression, ParserError> {
        let start_location = self.current_location();

        self.expect(&TokenType::LeftBrace, "expected '{'")?;

        // Parse left operand (must be a primary expression, not another binary)
        let mut left = self.parse_primary_expression()?;

        // Loop to handle chained binary operations
        while !self.check(&TokenType::RightBrace) && !self.is_at_end() {
            let operator = self.parse_binary_operator()?;
            let right = self.parse_primary_expression()?; // Use primary expression for right operand for now
            left = self.build_binary_expression(left, operator, right, start_location.clone());
        }

        self.expect(
            &TokenType::RightBrace,
            "expected '}' after binary expression",
        )?;

        Ok(left)
    }

    /// Parse a primary expression (non-binary): literals, identifiers
    fn parse_primary_expression(&mut self) -> Result<Expression, ParserError> {
        let start_location = self.current_location();

        // Check for Map literal: { key: value, ... }
        // Distinguished from braced expression { expr } by looking ahead for a colon
        if self.check(&TokenType::LeftBrace) {
            // If it's an empty brace {}, it could be an empty map or an empty block (which isn't an expression usually)
            // In Aether V2, {} is ambiguous. Let's assume it's an empty Map if context allows, or empty block.
            // Actually, `{ expr }` is a braced expression. `{ stmt; }` is a block.
            // `{ key: value }` is a map.
            // `{}` is typically an empty Map literal in this context.
            
            // Lookahead to see if it's a map literal
            if self.looks_like_map_literal() {
                return self.parse_map_literal();
            }
            
            // Otherwise parse as braced expression
            return self.parse_braced_expression();
        }

        // Logical NOT: !expr
        if self.check(&TokenType::Bang) {
            self.advance(); // consume '!'
            let operand = self.parse_primary_expression()?;
            return Ok(Expression::LogicalNot {
                operand: Box::new(operand),
                source_location: start_location,
            });
        }

        // Unary minus: -expr
        if self.check(&TokenType::Minus) {
            self.advance(); // consume '-'
            let operand = self.parse_primary_expression()?;
            return Ok(Expression::Negate {
                operand: Box::new(operand),
                source_location: start_location,
            });
        }

        // Integer literal
        if let TokenType::IntegerLiteral(value) = &self.peek().token_type {
            let value = *value;
            self.advance();
            return Ok(Expression::IntegerLiteral {
                value,
                source_location: start_location,
            });
        }

        // Float literal
        if let TokenType::FloatLiteral(value) = &self.peek().token_type {
            let value = *value;
            self.advance();
            return Ok(Expression::FloatLiteral {
                value,
                source_location: start_location,
            });
        }

        // String literal
        if let TokenType::StringLiteral(value) = &self.peek().token_type {
            let value = value.clone();
            self.advance();
            return Ok(Expression::StringLiteral {
                value,
                source_location: start_location,
            });
        }

        // Character literal
        if let TokenType::CharLiteral(value) = &self.peek().token_type {
            let value = *value;
            self.advance();
            return Ok(Expression::CharacterLiteral {
                value,
                source_location: start_location,
            });
        }

        // Boolean literal
        if let TokenType::BoolLiteral(value) = &self.peek().token_type {
            let value = *value;
            self.advance();
            return Ok(Expression::BooleanLiteral {
                value,
                source_location: start_location,
            });
        }

        // Keyword 'range' treated as identifier for function call
        if self.check_keyword(Keyword::Range) {
            let start_location = self.current_location();
            self.advance(); // consume 'range'
            let name = "range".to_string();
            
            let mut expr = Expression::Variable {
                name: Identifier {
                    name: name.clone(),
                    source_location: start_location.clone(),
                },
                source_location: start_location.clone(),
            };
            
            // Handle postfix operators (copied from Identifier block)
            loop {
                if self.check(&TokenType::LeftParen) {
                    // Function call: expr(args)
                    self.advance(); // consume '('
                    let mut arg_exprs = Vec::new();

                    if !self.check(&TokenType::RightParen) {
                        // Check for labeled argument: label: expr
                        let is_labeled = if let TokenType::Identifier(_) = &self.peek().token_type {
                            self.peek_next().unwrap().token_type == TokenType::Colon
                        } else {
                            false
                        };

                        if is_labeled {
                            self.advance(); // consume label
                            self.advance(); // consume colon
                        }
                        
                        arg_exprs.push(self.parse_expression()?);
                        
                        while self.check(&TokenType::Comma) {
                            self.advance(); // consume ','
                            
                            let is_labeled = if let TokenType::Identifier(_) = &self.peek().token_type {
                                self.peek_next().unwrap().token_type == TokenType::Colon
                            } else {
                                false
                            };

                            if is_labeled {
                                self.advance(); // consume label
                                self.advance(); // consume colon
                            }
                            
                            arg_exprs.push(self.parse_expression()?);
                        }
                    }
                    self.expect(&TokenType::RightParen, "expected ')'")?;

                    // Extract function name
                    let function_name = match &expr {
                        Expression::Variable { name, .. } => name.clone(),
                        _ => {
                            return Err(ParserError::UnexpectedToken {
                                expected: "function name".to_string(),
                                found: "complex expression".to_string(),
                                location: start_location.clone(),
                            });
                        }
                    };

                    // Convert to Argument structs (simplified)
                    let arguments: Vec<Argument> = arg_exprs
                        .into_iter()
                        .enumerate()
                        .map(|(i, e)| Argument {
                            parameter_name: Identifier::new(
                                format!("arg_{}", i),
                                start_location.clone(),
                            ),
                            value: Box::new(e),
                            source_location: start_location.clone(),
                        })
                        .collect();

                    expr = Expression::FunctionCall {
                        call: AstFunctionCall {
                            function_reference: FunctionReference::Local {
                                name: function_name,
                            },
                            arguments,
                            variadic_arguments: Vec::new(),
                        },
                        source_location: start_location.clone(),
                    };
                } else {
                    break;
                }
            }
            return Ok(expr);
        }

        // Identifier (variable reference, struct construction, enum variant, or function call)
        if let TokenType::Identifier(name) = &self.peek().token_type {
            let name = name.clone();
            self.advance();

            // Check for enum variant: EnumType::Variant or EnumType::Variant(value)
            if self.check(&TokenType::DoubleColon) {
                return self.parse_enum_variant_expression(name, start_location);
            }

            // Check for struct construction: TypeName { field: value, ... }
            // Only parse as struct construction if it looks like one (has field: value syntax)
            if self.check(&TokenType::LeftBrace) && self.looks_like_struct_construction() {
                return self.parse_struct_construction(name, start_location);
            }

            let mut expr = Expression::Variable {
                name: Identifier {
                    name,
                    source_location: start_location.clone(),
                },
                source_location: start_location.clone(),
            };

            // Handle postfix operators: (args), [index], and .field
            loop {
                if self.check(&TokenType::LeftParen) {
                    // Function call: expr(args)
                    self.advance(); // consume '('
                    let mut arg_exprs = Vec::new();

                    if !self.check(&TokenType::RightParen) {
                        arg_exprs.push(self.parse_expression()?);
                        while self.check(&TokenType::Comma) {
                            self.advance(); // consume ','
                            arg_exprs.push(self.parse_expression()?);
                        }
                    }
                    self.expect(&TokenType::RightParen, "expected ')'")?;

                    // Extract function name from variable expression
                    let function_name = match &expr {
                        Expression::Variable { name, .. } => name.clone(),
                        _ => {
                            return Err(ParserError::UnexpectedToken {
                                expected: "function name".to_string(),
                                found: "complex expression".to_string(),
                                location: start_location.clone(),
                            });
                        }
                    };

                    // Convert expressions to Argument structs with placeholder names
                    let arguments: Vec<Argument> = arg_exprs
                        .into_iter()
                        .enumerate()
                        .map(|(i, e)| Argument {
                            parameter_name: Identifier::new(
                                format!("arg_{}", i),
                                start_location.clone(),
                            ),
                            value: Box::new(e),
                            source_location: start_location.clone(),
                        })
                        .collect();

                    expr = Expression::FunctionCall {
                        call: AstFunctionCall {
                            function_reference: FunctionReference::Local {
                                name: function_name,
                            },
                            arguments,
                            variadic_arguments: Vec::new(),
                        },
                        source_location: start_location.clone(),
                    };
                } else if self.check(&TokenType::LeftBracket) {
                    // Array indexing: expr[index]
                    self.advance(); // consume '['
                    let index = self.parse_expression()?;
                    self.expect(&TokenType::RightBracket, "expected ']'")?;
                    expr = Expression::ArrayAccess {
                        array: Box::new(expr),
                        index: Box::new(index),
                        source_location: start_location.clone(),
                    };
                } else if self.check(&TokenType::Dot) {
                    // Field access or method call: expr.field or expr.method(args)
                    self.advance(); // consume '.'
                    if let TokenType::Identifier(member_name) = &self.peek().token_type {
                        let member_name = member_name.clone();
                        let member_loc = self.current_location();
                        self.advance();

                        // Check if this is a method call (followed by '(')
                        if self.check(&TokenType::LeftParen) {
                            // Method call: expr.method(args)
                            self.advance(); // consume '('
                            let mut arg_exprs = Vec::new();

                            if !self.check(&TokenType::RightParen) {
                                // Parse first argument
                                arg_exprs.push(self.parse_expression()?);

                                // Parse remaining arguments
                                while self.check(&TokenType::Comma) {
                                    self.advance(); // consume ','
                                    arg_exprs.push(self.parse_expression()?);
                                }
                            }
                            self.expect(
                                &TokenType::RightParen,
                                "expected ')' after method arguments",
                            )?;

                            // Convert expressions to Argument structs
                            let arguments: Vec<Argument> = arg_exprs
                                .into_iter()
                                .enumerate()
                                .map(|(i, e)| Argument {
                                    parameter_name: Identifier::new(
                                        format!("arg_{}", i),
                                        start_location.clone(),
                                    ),
                                    value: Box::new(e),
                                    source_location: start_location.clone(),
                                })
                                .collect();

                            expr = Expression::MethodCall {
                                receiver: Box::new(expr),
                                method_name: Identifier {
                                    name: member_name,
                                    source_location: member_loc,
                                },
                                arguments,
                                source_location: start_location.clone(),
                            };
                        } else {
                            // Field access: expr.field
                            expr = Expression::FieldAccess {
                                instance: Box::new(expr),
                                field_name: Identifier {
                                    name: member_name,
                                    source_location: member_loc,
                                },
                                source_location: start_location.clone(),
                            };
                        }
                    } else {
                        return Err(ParserError::UnexpectedToken {
                            expected: "field name or method name".to_string(),
                            found: format!("{:?}", self.peek().token_type),
                            location: self.current_location(),
                        });
                    }
                } else {
                    break;
                }
            }

            return Ok(expr);
        }

        // Match expression
        if self.check_keyword(Keyword::Match) {
            return self.parse_match_expression();
        }

        // Nested braced expression
        if self.peek().token_type == TokenType::LeftBrace {
            return self.parse_braced_expression();
        }

        // Parenthesized expression or lambda
        if self.check(&TokenType::LeftParen) {
            return self.parse_paren_expr_or_lambda();
        }

        Err(ParserError::UnexpectedToken {
            expected: "primary expression".to_string(),
            found: format!("{:?}", self.peek().token_type),
            location: start_location,
        })
    }

    /// Parse either a parenthesized expression or a lambda (without captures)
    /// Grammar: "(" expression ")" | "(" params ")" [":" type] "=>" body
    fn parse_paren_expr_or_lambda(&mut self) -> Result<Expression, ParserError> {
        self.parse_paren_expr_or_lambda_with_captures(Vec::new())
    }

    /// Parse either a parenthesized expression or a lambda with optional captures
    /// Grammar: "(" expression ")" | "(" params ")" [":" type] "=>" body
    fn parse_paren_expr_or_lambda_with_captures(
        &mut self,
        captures: Vec<Capture>,
    ) -> Result<Expression, ParserError> {
        let start_location = self.current_location();
        self.expect(&TokenType::LeftParen, "expected '('")?;

        // Check for empty parens - must be a zero-parameter lambda
        if self.check(&TokenType::RightParen) {
            self.advance(); // consume ')'
            return self.parse_lambda_after_params(captures, vec![], start_location);
        }

        // Try to determine if this is a lambda or parenthesized expression
        // Lambda params look like: identifier [: type] [, ...]
        // We need to look ahead to see if this is a param list or expression
        if self.is_lambda_param_start() {
            // Parse as lambda parameters
            let params = self.parse_lambda_params()?;
            self.expect(
                &TokenType::RightParen,
                "expected ')' after lambda parameters",
            )?;
            return self.parse_lambda_after_params(captures, params, start_location);
        }

        // If we have captures but this doesn't look like a lambda, that's an error
        if !captures.is_empty() {
            return Err(ParserError::SyntaxError {
                message: "capture list must be followed by lambda parameters".to_string(),
                location: start_location,
                suggestion: Some("use [captures](params) => body syntax".to_string()),
            });
        }

        // Parse as parenthesized expression
        let inner = self.parse_expression()?;
        self.expect(&TokenType::RightParen, "expected ')'")?;
        Ok(inner)
    }

    /// Check if the current position looks like the start of a lambda parameter
    fn is_lambda_param_start(&self) -> bool {
        // Lambda parameter starts with identifier followed by `:` (type annotation)
        // or identifier followed by `,` or `)`
        // Regular expressions can also start with identifier, so we need to look ahead
        if let TokenType::Identifier(_) = &self.peek().token_type {
            // Check what follows the identifier
            if let Some(next) = self.tokens.get(self.position + 1) {
                match &next.token_type {
                    // identifier: type - definitely a lambda param
                    TokenType::Colon => return true,
                    // identifier, or identifier) followed by => - lambda
                    TokenType::Comma | TokenType::RightParen => {
                        // Look further to see if there's a `=>` after the `)`
                        let mut depth = 1;
                        let mut i = self.position + 1;
                        while i < self.tokens.len() {
                            match &self.tokens[i].token_type {
                                TokenType::LeftParen => depth += 1,
                                TokenType::RightParen => {
                                    depth -= 1;
                                    if depth == 0 {
                                        // Check if => follows
                                        if i + 1 < self.tokens.len() {
                                            if matches!(
                                                self.tokens[i + 1].token_type,
                                                TokenType::FatArrow
                                            ) {
                                                return true;
                                            }
                                        }
                                        break;
                                    }
                                }
                                _ => {}
                            }
                            i += 1;
                        }
                    }
                    _ => {}
                }
            }
        }
        false
    }

    /// Parse lambda parameters (without parentheses)
    fn parse_lambda_params(&mut self) -> Result<Vec<Parameter>, ParserError> {
        let mut params = Vec::new();

        loop {
            let param_loc = self.current_location();

            // Parse parameter name
            let name = self.parse_identifier()?;

            // Optional type annotation
            let param_type = if self.check(&TokenType::Colon) {
                self.advance();
                Box::new(self.parse_type()?)
            } else {
                // Inferred type - use Integer as placeholder
                Box::new(TypeSpecifier::Primitive {
                    type_name: PrimitiveType::Integer,
                    source_location: param_loc.clone(),
                })
            };

            params.push(Parameter {
                name,
                param_type,
                intent: None,
                constraint: None,
                passing_mode: PassingMode::ByValue,
                source_location: param_loc,
            });

            if !self.check(&TokenType::Comma) {
                break;
            }
            self.advance(); // consume ','
        }

        Ok(params)
    }

    /// Parse the rest of a lambda after parameters have been parsed
    fn parse_lambda_after_params(
        &mut self,
        captures: Vec<Capture>,
        parameters: Vec<Parameter>,
        start_location: SourceLocation,
    ) -> Result<Expression, ParserError> {
        // Optional return type: -> Type
        let return_type = if self.check(&TokenType::Arrow) {
            self.advance();
            Some(Box::new(self.parse_type()?))
        } else {
            None
        };

        // Expect => for lambda body
        self.expect(&TokenType::FatArrow, "expected '=>' for lambda body")?;

        // Parse body - either a block or a single expression
        let body = if self.check(&TokenType::LeftBrace) {
            self.parse_lambda_block()?
        } else {
            LambdaBody::Expression(Box::new(self.parse_expression()?))
        };

        Ok(Expression::Lambda {
            captures,
            parameters,
            return_type,
            body,
            source_location: start_location,
        })
    }

    /// Parse a lambda block - can be either:
    /// - A single expression (implicit return): `{expr}`
    /// - Multiple statements with explicit return: `{ stmt; stmt; return expr; }`
    fn parse_lambda_block(&mut self) -> Result<LambdaBody, ParserError> {
        let block_start = self.current_location();
        self.expect(&TokenType::LeftBrace, "expected '{'")?;

        // Check for empty block
        if self.check(&TokenType::RightBrace) {
            let end_loc = self.current_location();
            self.advance();
            return Ok(LambdaBody::Block(Block {
                statements: vec![],
                source_location: block_start,
            }));
        }

        // Try to parse as a single expression first
        // Save position to backtrack if needed
        let saved_pos = self.position;

        // Try parsing an expression
        if let Ok(expr) = self.parse_expression() {
            // Check if this is followed by `}` (single expression body)
            if self.check(&TokenType::RightBrace) {
                self.advance(); // consume '}'
                // Wrap the expression in an implicit return statement
                let return_stmt = Statement::Return {
                    value: Some(Box::new(expr)),
                    source_location: block_start.clone(),
                };
                return Ok(LambdaBody::Block(Block {
                    statements: vec![return_stmt],
                    source_location: block_start,
                }));
            }
            // Not a single expression body, backtrack and parse as statements
            self.position = saved_pos;
        } else {
            // Expression parse failed, backtrack
            self.position = saved_pos;
        }

        // Parse as regular block with statements
        let mut statements = Vec::new();
        while !self.check(&TokenType::RightBrace) && !self.is_at_end() {
            statements.push(self.parse_statement()?);
        }
        self.expect(&TokenType::RightBrace, "expected '}' after lambda block")?;

        Ok(LambdaBody::Block(Block {
            statements,
            source_location: block_start,
        }))
    }

    /// Parse a struct construction expression: TypeName { field: value, field2: value2, ... }
    fn parse_struct_construction(
        &mut self,
        type_name: String,
        start_location: SourceLocation,
    ) -> Result<Expression, ParserError> {
        self.expect(&TokenType::LeftBrace, "expected '{'")?;

        let mut field_values = Vec::new();

        // Parse field: value pairs
        while !self.check(&TokenType::RightBrace) && !self.is_at_end() {
            let field_loc = self.current_location();

            // Parse field name
            let field_name = if let TokenType::Identifier(name) = &self.peek().token_type {
                let name = name.clone();
                self.advance();
                Identifier {
                    name,
                    source_location: field_loc.clone(),
                }
            } else {
                return Err(ParserError::UnexpectedToken {
                    expected: "field name".to_string(),
                    found: format!("{:?}", self.peek().token_type),
                    location: field_loc,
                });
            };

            // Expect colon
            self.expect(&TokenType::Colon, "expected ':' after field name")?;

            // Parse field value
            let value = Box::new(self.parse_expression()?);

            field_values.push(FieldValue {
                field_name,
                value,
                source_location: field_loc,
            });

            // Accept comma as field separator (optional for last field)
            if self.check(&TokenType::Comma) {
                self.advance();
            }
        }

        self.expect(&TokenType::RightBrace, "expected '}'")?;

        Ok(Expression::StructConstruct {
            type_name: Identifier {
                name: type_name,
                source_location: start_location.clone(),
            },
            field_values,
            source_location: start_location,
        })
    }

    /// Parse an enum variant expression: EnumType::Variant or EnumType::Variant(value)
    fn parse_enum_variant_expression(
        &mut self,
        enum_name: String,
        start_location: SourceLocation,
    ) -> Result<Expression, ParserError> {
        self.expect(&TokenType::DoubleColon, "expected '::'")?;

        // Parse variant name
        let variant_name = if let TokenType::Identifier(name) = &self.peek().token_type {
            let name = name.clone();
            self.advance();
            Identifier {
                name,
                source_location: self.current_location(),
            }
        } else {
            return Err(ParserError::UnexpectedToken {
                expected: "variant name".to_string(),
                found: format!("{:?}", self.peek().token_type),
                location: self.current_location(),
            });
        };

        // Check for associated value: Variant(value)
        let values = if self.check(&TokenType::LeftParen) {
            self.advance(); // consume '('
            let mut exprs = Vec::new();
            if !self.check(&TokenType::RightParen) {
                loop {
                    exprs.push(self.parse_expression()?);
                    if self.check(&TokenType::Comma) {
                        self.advance();
                    } else {
                        break;
                    }
                }
            }
            self.expect(&TokenType::RightParen, "expected ')'")?;
            exprs
        } else {
            Vec::new()
        };

        Ok(Expression::EnumVariant {
            enum_name: Identifier {
                name: enum_name,
                source_location: start_location.clone(),
            },
            variant_name,
            values,
            source_location: start_location,
        })
    }

    /// Parse an array literal: [expr, expr, ...]
    fn parse_array_literal(&mut self) -> Result<Expression, ParserError> {
        let start_location = self.current_location();

        self.expect(&TokenType::LeftBracket, "expected '['")?;

        let mut elements = Vec::new();

        // Parse elements until we see ]
        while !self.check(&TokenType::RightBracket) {
            let element = self.parse_expression()?;
            elements.push(Box::new(element));

            // Expect comma or closing bracket
            if !self.check(&TokenType::RightBracket) {
                self.expect(&TokenType::Comma, "expected ',' or ']'")?;
            }
        }

        self.expect(&TokenType::RightBracket, "expected ']'")?;

        // Infer element type from first element or default to Int
        let element_type = Box::new(TypeSpecifier::Primitive {
            type_name: PrimitiveType::Integer,
            source_location: start_location.clone(),
        });

        Ok(Expression::ArrayLiteral {
            element_type,
            elements,
            source_location: start_location,
        })
    }

    /// Check if the current token sequence looks like a map literal
    /// { key: value } or {} (empty map)
    fn looks_like_map_literal(&self) -> bool {
        if !self.check(&TokenType::LeftBrace) {
            return false;
        }
        
        // Empty map {}
        if let Some(next) = self.tokens.get(self.position + 1) {
            if matches!(next.token_type, TokenType::RightBrace) {
                return true; 
            }
        }
        
        // Map with entries: { key: value
        // We need to skip the key expression to find the colon
        // This is hard without full backtracking.
        // Simple heuristic: if next token is a string/int literal or identifier, 
        // and followed by colon (after skipping potential complex key), it's a map.
        
        // For now, let's just check if we can find a colon at the right nesting level
        let mut depth = 0;
        let mut i = self.position + 1;
        while i < self.tokens.len() {
            match &self.tokens[i].token_type {
                TokenType::LeftBrace => depth += 1,
                TokenType::RightBrace => {
                    if depth == 0 {
                        return false; // End of brace before finding colon
                    }
                    depth -= 1;
                },
                TokenType::Colon => {
                    if depth == 0 {
                        return true; // Found top-level colon
                    }
                },
                TokenType::Semicolon => return false, // Semicolon implies block/statements
                _ => {}
            }
            i += 1;
        }
        
        false
    }

    /// Parse a map literal: { key: value, ... }
    fn parse_map_literal(&mut self) -> Result<Expression, ParserError> {
        let start_location = self.current_location();
        self.expect(&TokenType::LeftBrace, "expected '{'")?;

        let mut entries = Vec::new();

        // Check for empty map
        if self.check(&TokenType::RightBrace) {
            self.advance();
            let key_type = Box::new(TypeSpecifier::Primitive { 
                type_name: PrimitiveType::String,
                source_location: start_location.clone() 
            });
            let value_type = Box::new(TypeSpecifier::Primitive { 
                type_name: PrimitiveType::Integer,
                source_location: start_location.clone() 
            });
            return Ok(Expression::MapLiteral {
                key_type,
                value_type,
                entries,
                source_location: start_location,
            });
        }

        while !self.check(&TokenType::RightBrace) && !self.is_at_end() {
            // Parse key expression
            let key = self.parse_expression()?;
            
            self.expect(&TokenType::Colon, "expected ':' after map key")?;
            
            // Parse value expression
            let value = self.parse_expression()?;
            
            entries.push(crate::ast::MapEntry {
                key: Box::new(key),
                value: Box::new(value),
                source_location: self.current_location(),
            });
            
            if self.check(&TokenType::Comma) {
                self.advance();
            } else {
                break;
            }
        }
        
        self.expect(&TokenType::RightBrace, "expected '}' after map entries")?;
        
        // Inferred types (String -> Integer for now as default)
        // In a real compiler, we'd infer from context or first entry
        let key_type = Box::new(TypeSpecifier::Primitive { 
            type_name: PrimitiveType::String,
            source_location: start_location.clone() 
        });
        let value_type = Box::new(TypeSpecifier::Primitive { 
            type_name: PrimitiveType::Integer,
            source_location: start_location.clone() 
        });

        Ok(Expression::MapLiteral {
            key_type,
            value_type,
            entries,
            source_location: start_location,
        })
    }

    /// Parse a capture list: [x, &y, &mut z]
    /// Returns a vector of captures
    fn parse_capture_list(&mut self) -> Result<Vec<Capture>, ParserError> {
        let mut captures = Vec::new();

        // Expect opening bracket
        self.expect(
            &TokenType::LeftBracket,
            "expected '[' to start capture list",
        )?;

        // Parse captures until we see ]
        while !self.check(&TokenType::RightBracket) {
            let capture_location = self.current_location();

            // Check for capture mode prefix
            let mode = if self.check(&TokenType::Ampersand) {
                self.advance();
                // Check for &mut
                if self.check_keyword(Keyword::Mut) {
                    self.advance();
                    CaptureMode::ByMutableReference
                } else {
                    CaptureMode::ByReference
                }
            } else {
                CaptureMode::ByValue
            };

            // Parse the identifier
            let name = self.parse_identifier()?;

            captures.push(Capture {
                name,
                mode,
                source_location: capture_location,
            });

            // Expect comma or end
            if !self.check(&TokenType::RightBracket) {
                self.expect(&TokenType::Comma, "expected ',' between captures")?;
            }
        }

        // Expect closing bracket
        self.expect(&TokenType::RightBracket, "expected ']' to end capture list")?;

        Ok(captures)
    }

    /// Parse a range expression: start..end or start..=end
    /// Called with optional start expression; parses .. or ..= and optional end
    fn parse_range_expression(
        &mut self,
        start: Option<Expression>,
        start_location: SourceLocation,
    ) -> Result<Expression, ParserError> {
        // Determine if inclusive (..=) or exclusive (..)
        let inclusive = if self.check(&TokenType::DotDotEqual) {
            self.advance();
            true
        } else if self.check(&TokenType::DotDot) {
            self.advance();
            false
        } else {
            return Err(ParserError::UnexpectedToken {
                expected: "'..' or '..='".to_string(),
                found: format!("{:?}", self.peek().token_type),
                location: self.current_location(),
            });
        };

        // Parse optional end expression
        // End is present if followed by a primary expression (not operator, delimiter, etc.)
        let end = if self.is_range_end_start() {
            Some(Box::new(self.parse_range_operand()?))
        } else {
            None
        };

        Ok(Expression::Range {
            start: start.map(Box::new),
            end,
            inclusive,
            source_location: start_location,
        })
    }

    /// Check if the current token could start a range end expression
    fn is_range_end_start(&self) -> bool {
        matches!(
            &self.peek().token_type,
            TokenType::IntegerLiteral(_)
                | TokenType::FloatLiteral(_)
                | TokenType::Identifier(_)
                | TokenType::LeftParen
                | TokenType::LeftBrace
        )
    }

    /// Parse a simple expression suitable for range operand
    fn parse_range_operand(&mut self) -> Result<Expression, ParserError> {
        let start_location = self.current_location();

        // Integer literal
        if let TokenType::IntegerLiteral(value) = &self.peek().token_type {
            let value = *value;
            self.advance();
            return Ok(Expression::IntegerLiteral {
                value,
                source_location: start_location,
            });
        }

        // Identifier
        if let TokenType::Identifier(name) = &self.peek().token_type {
            let name = name.clone();
            self.advance();
            return Ok(Expression::Variable {
                name: Identifier {
                    name,
                    source_location: start_location.clone(),
                },
                source_location: start_location,
            });
        }

        // Parenthesized expression
        if self.check(&TokenType::LeftParen) {
            return self.parse_paren_expr_or_lambda();
        }

        // Braced expression
        if self.check(&TokenType::LeftBrace) {
            return self.parse_braced_expression();
        }

        Err(ParserError::UnexpectedToken {
            expected: "range end value".to_string(),
            found: format!("{:?}", self.peek().token_type),
            location: start_location,
        })
    }

    /// Parse a match expression
    /// Grammar: "match" expression "{" match_arm* "}"
    fn parse_match_expression(&mut self) -> Result<Expression, ParserError> {
        let start_location = self.current_location();

        // Consume 'match' keyword
        self.expect_keyword(Keyword::Match, "expected 'match'")?;

        // Parse the value being matched
        let value = Box::new(self.parse_expression()?);

        // Expect opening brace
        self.expect(&TokenType::LeftBrace, "expected '{' after match value")?;

        // Parse match arms
        let mut cases = Vec::new();
        while !self.check(&TokenType::RightBrace) && !self.is_at_end() {
            cases.push(self.parse_match_arm()?);

            // Optional comma between arms
            if self.check(&TokenType::Comma) {
                self.advance();
            }
        }

        // Expect closing brace
        self.expect(&TokenType::RightBrace, "expected '}' to close match")?;

        Ok(Expression::Match {
            value,
            cases,
            source_location: start_location,
        })
    }

    /// Parse a match arm
    /// Grammar: pattern "=>" expression
    fn parse_match_arm(&mut self) -> Result<MatchCase, ParserError> {
        let start_location = self.current_location();

        // Parse the pattern
        let pattern = self.parse_pattern()?;

        // Expect '=>'
        self.expect(&TokenType::FatArrow, "expected '=>' after pattern")?;

        // Parse the body expression
        let body = Box::new(self.parse_expression()?);

        Ok(MatchCase {
            pattern,
            body,
            source_location: start_location,
        })
    }

    /// Parse a pattern for match expressions
    /// Grammar: "_" | identifier | literal | enum_variant
    fn parse_pattern(&mut self) -> Result<Pattern, ParserError> {
        let start_location = self.current_location();

        // Wildcard pattern: "_" (as Underscore token)
        if self.check(&TokenType::Underscore) {
            self.advance();
            return Ok(Pattern::Wildcard {
                binding: None,
                source_location: start_location,
            });
        }

        // Identifier pattern (could be variable binding or enum variant)
        if let TokenType::Identifier(name) = &self.peek().token_type {
            let name = name.clone();
            self.advance();

            // Check if this is a qualified enum variant pattern: EnumName::Variant
            if self.check(&TokenType::DoubleColon) {
                self.advance(); // consume '::'
                let variant_name = self.parse_identifier()?;

                // Check for binding: EnumName::Variant(x) or EnumName::Variant(_)
                let bindings = if self.check(&TokenType::LeftParen) {
                    self.advance(); // consume '('
                    let mut bindings_vec = Vec::new();
                    
                    if !self.check(&TokenType::RightParen) {
                        loop {
                            if self.check(&TokenType::Underscore) {
                                self.advance();
                                // Represent wildcard binding as "_"
                                bindings_vec.push(Identifier::new("_".to_string(), self.current_location()));
                            } else {
                                bindings_vec.push(self.parse_identifier()?);
                            }
                            
                            if self.check(&TokenType::Comma) {
                                self.advance();
                            } else {
                                break;
                            }
                        }
                    }
                    self.expect(&TokenType::RightParen, "expected ')'")?;
                    bindings_vec
                } else {
                    Vec::new()
                };

                return Ok(Pattern::EnumVariant {
                    enum_name: Some(Identifier::new(name, start_location.clone())),
                    variant_name,
                    bindings,
                    nested_pattern: None,
                    source_location: start_location,
                });
            }

            // Check for Struct pattern: Name { field: pattern, ... }
            if self.check(&TokenType::LeftBrace) {
                self.advance(); // consume '{'
                let mut fields = Vec::new();
                
                while !self.check(&TokenType::RightBrace) && !self.is_at_end() {
                    // Field name
                    let field_name = self.parse_identifier()?;
                    
                    let pattern = if self.check(&TokenType::Colon) {
                        self.advance(); // consume ':'
                        self.parse_pattern()?
                    } else {
                        // Shorthand: field name is also the binding
                        // Treat as wildcard pattern with binding
                        Pattern::Wildcard {
                            binding: Some(field_name.clone()),
                            source_location: field_name.source_location.clone(),
                        }
                    };
                    
                    fields.push((field_name, pattern));
                    
                    if self.check(&TokenType::Comma) {
                        self.advance();
                    } else {
                        break;
                    }
                }
                
                self.expect(&TokenType::RightBrace, "expected '}' after struct pattern fields")?;
                
                return Ok(Pattern::Struct {
                    struct_name: Identifier::new(name, start_location.clone()),
                    fields,
                    source_location: start_location,
                });
            }

            // Check if this is an enum variant pattern: Name(binding) or Name binding
            if self.check(&TokenType::LeftParen) {
                // Enum variant with parenthesized binding: Some(x)
                self.advance(); // consume '('

                let bindings = if !self.check(&TokenType::RightParen) {
                    let mut bindings_vec = Vec::new();
                    loop {
                        bindings_vec.push(self.parse_identifier()?);
                        if self.check(&TokenType::Comma) {
                            self.advance();
                        } else {
                            break;
                        }
                    }
                    bindings_vec
                } else {
                    Vec::new()
                };

                self.expect(&TokenType::RightParen, "expected ')'")?;

                return Ok(Pattern::EnumVariant {
                    enum_name: None,
                    variant_name: Identifier::new(name, start_location.clone()),
                    bindings,
                    nested_pattern: None,
                    source_location: start_location,
                });
            } else if let TokenType::Identifier(_) = &self.peek().token_type {
                // Enum variant with space binding: Some x
                let binding = self.parse_identifier()?;

                return Ok(Pattern::EnumVariant {
                    enum_name: None,
                    variant_name: Identifier::new(name, start_location.clone()),
                    bindings: vec![binding],
                    nested_pattern: None,
                    source_location: start_location,
                });
            }

            // Simple variable binding pattern
            return Ok(Pattern::Wildcard {
                binding: Some(Identifier::new(name, start_location.clone())),
                source_location: start_location,
            });
        }

        // Literal patterns
        if let TokenType::IntegerLiteral(value) = &self.peek().token_type {
            let value = *value;
            self.advance();
            return Ok(Pattern::Literal {
                value: Box::new(Expression::IntegerLiteral {
                    value,
                    source_location: start_location.clone(),
                }),
                source_location: start_location,
            });
        }

        if let TokenType::StringLiteral(value) = &self.peek().token_type {
            let value = value.clone();
            self.advance();
            return Ok(Pattern::Literal {
                value: Box::new(Expression::StringLiteral {
                    value,
                    source_location: start_location.clone(),
                }),
                source_location: start_location,
            });
        }

        if let TokenType::BoolLiteral(value) = &self.peek().token_type {
            let value = *value;
            self.advance();
            return Ok(Pattern::Literal {
                value: Box::new(Expression::BooleanLiteral {
                    value,
                    source_location: start_location.clone(),
                }),
                source_location: start_location,
            });
        }

        Err(ParserError::UnexpectedToken {
            expected: "pattern (_, identifier, or literal)".to_string(),
            found: format!("{:?}", self.peek().token_type),
            location: start_location,
        })
    }

    /// Parse a binary operator token
    fn parse_binary_operator(&mut self) -> Result<BinaryOp, ParserError> {
        let location = self.current_location();
        let token = self.peek();

        let op = match &token.token_type {
            TokenType::Plus => BinaryOp::Add,
            TokenType::Minus => BinaryOp::Subtract,
            TokenType::Star => BinaryOp::Multiply,
            TokenType::Slash => BinaryOp::Divide,
            TokenType::Percent => BinaryOp::Modulo,
            TokenType::EqualEqual => BinaryOp::Equals,
            TokenType::BangEqual => BinaryOp::NotEquals,
            TokenType::Less => BinaryOp::LessThan,
            TokenType::LessEqual => BinaryOp::LessEqual,
            TokenType::Greater => BinaryOp::GreaterThan,
            TokenType::GreaterEqual => BinaryOp::GreaterEqual,
            TokenType::AmpAmp => BinaryOp::And,
            TokenType::PipePipe => BinaryOp::Or,
            _ => {
                return Err(ParserError::UnexpectedToken {
                    expected: "binary operator".to_string(),
                    found: format!("{:?}", token.token_type),
                    location,
                });
            }
        };

        self.advance();
        Ok(op)
    }

    /// Build the appropriate Expression variant from operator
    fn build_binary_expression(
        &self,
        left: Expression,
        op: BinaryOp,
        right: Expression,
        source_location: SourceLocation,
    ) -> Expression {
        match op {
            BinaryOp::Add => Expression::Add {
                left: Box::new(left),
                right: Box::new(right),
                source_location,
            },
            BinaryOp::Subtract => Expression::Subtract {
                left: Box::new(left),
                right: Box::new(right),
                source_location,
            },
            BinaryOp::Multiply => Expression::Multiply {
                left: Box::new(left),
                right: Box::new(right),
                source_location,
            },
            BinaryOp::Divide => Expression::Divide {
                left: Box::new(left),
                right: Box::new(right),
                source_location,
            },
            BinaryOp::Modulo => Expression::Modulo {
                left: Box::new(left),
                right: Box::new(right),
                source_location,
            },
            BinaryOp::Equals => Expression::Equals {
                left: Box::new(left),
                right: Box::new(right),
                source_location,
            },
            BinaryOp::NotEquals => Expression::NotEquals {
                left: Box::new(left),
                right: Box::new(right),
                source_location,
            },
            BinaryOp::LessThan => Expression::LessThan {
                left: Box::new(left),
                right: Box::new(right),
                source_location,
            },
            BinaryOp::LessEqual => Expression::LessThanOrEqual {
                left: Box::new(left),
                right: Box::new(right),
                source_location,
            },
            BinaryOp::GreaterThan => Expression::GreaterThan {
                left: Box::new(left),
                right: Box::new(right),
                source_location,
            },
            BinaryOp::GreaterEqual => Expression::GreaterThanOrEqual {
                left: Box::new(left),
                right: Box::new(right),
                source_location,
            },
            BinaryOp::And => Expression::LogicalAnd {
                operands: vec![left, right],
                source_location,
            },
            BinaryOp::Or => Expression::LogicalOr {
                operands: vec![left, right],
                source_location,
            },
        }
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
        assert!(matches!(
            parser.peek().token_type,
            TokenType::Keyword(Keyword::Let)
        ));
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
        assert!(matches!(
            parser.peek().token_type,
            TokenType::Keyword(Keyword::Let)
        ));
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
            "module Test { import std.io; import std.collections; import math; }",
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
        let mut parser =
            parser_from_source("module Test { import std.collections.hashmap.HashMap; }");
        let result = parser.parse_module();

        assert!(result.is_ok());
        let module = result.unwrap();
        assert_eq!(
            module.imports[0].module_name.name,
            "std.collections.hashmap.HashMap"
        );
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

    // ==================== TYPE PARSING TESTS ====================

    #[test]
    fn test_parse_type_int() {
        let mut parser = parser_from_source("Int");
        let result = parser.parse_type();

        assert!(result.is_ok());
        let type_spec = result.unwrap();
        assert!(matches!(
            type_spec,
            TypeSpecifier::Primitive {
                type_name: PrimitiveType::Integer,
                ..
            }
        ));
    }

    #[test]
    fn test_parse_type_int64() {
        let mut parser = parser_from_source("Int64");
        let result = parser.parse_type();

        assert!(result.is_ok());
        let type_spec = result.unwrap();
        assert!(matches!(
            type_spec,
            TypeSpecifier::Primitive {
                type_name: PrimitiveType::Integer64,
                ..
            }
        ));
    }

    #[test]
    fn test_parse_type_float() {
        let mut parser = parser_from_source("Float");
        let result = parser.parse_type();

        assert!(result.is_ok());
        let type_spec = result.unwrap();
        assert!(matches!(
            type_spec,
            TypeSpecifier::Primitive {
                type_name: PrimitiveType::Float,
                ..
            }
        ));
    }

    #[test]
    fn test_parse_type_string() {
        let mut parser = parser_from_source("String");
        let result = parser.parse_type();

        assert!(result.is_ok());
        let type_spec = result.unwrap();
        assert!(matches!(
            type_spec,
            TypeSpecifier::Primitive {
                type_name: PrimitiveType::String,
                ..
            }
        ));
    }

    #[test]
    fn test_parse_type_bool() {
        let mut parser = parser_from_source("Bool");
        let result = parser.parse_type();

        assert!(result.is_ok());
        let type_spec = result.unwrap();
        assert!(matches!(
            type_spec,
            TypeSpecifier::Primitive {
                type_name: PrimitiveType::Boolean,
                ..
            }
        ));
    }

    #[test]
    fn test_parse_type_void() {
        let mut parser = parser_from_source("Void");
        let result = parser.parse_type();

        assert!(result.is_ok());
        let type_spec = result.unwrap();
        assert!(matches!(
            type_spec,
            TypeSpecifier::Primitive {
                type_name: PrimitiveType::Void,
                ..
            }
        ));
    }

    #[test]
    fn test_parse_type_sizet() {
        let mut parser = parser_from_source("SizeT");
        let result = parser.parse_type();

        assert!(result.is_ok());
        let type_spec = result.unwrap();
        assert!(matches!(
            type_spec,
            TypeSpecifier::Primitive {
                type_name: PrimitiveType::SizeT,
                ..
            }
        ));
    }

    #[test]
    fn test_parse_type_array() {
        let mut parser = parser_from_source("Array<Int>");
        let result = parser.parse_type();

        assert!(result.is_ok());
        let type_spec = result.unwrap();
        if let TypeSpecifier::Array { element_type, .. } = type_spec {
            assert!(matches!(
                *element_type,
                TypeSpecifier::Primitive {
                    type_name: PrimitiveType::Integer,
                    ..
                }
            ));
        } else {
            panic!("Expected Array type");
        }
    }

    #[test]
    fn test_parse_type_nested_array() {
        let mut parser = parser_from_source("Array<Array<Int>>");
        let result = parser.parse_type();

        assert!(result.is_ok());
        let type_spec = result.unwrap();
        if let TypeSpecifier::Array { element_type, .. } = type_spec {
            assert!(matches!(*element_type, TypeSpecifier::Array { .. }));
        } else {
            panic!("Expected nested Array type");
        }
    }

    #[test]
    fn test_parse_type_map() {
        let mut parser = parser_from_source("Map<String, Int>");
        let result = parser.parse_type();

        assert!(result.is_ok());
        let type_spec = result.unwrap();
        if let TypeSpecifier::Map {
            key_type,
            value_type,
            ..
        } = type_spec
        {
            assert!(matches!(
                *key_type,
                TypeSpecifier::Primitive {
                    type_name: PrimitiveType::String,
                    ..
                }
            ));
            assert!(matches!(
                *value_type,
                TypeSpecifier::Primitive {
                    type_name: PrimitiveType::Integer,
                    ..
                }
            ));
        } else {
            panic!("Expected Map type");
        }
    }

    #[test]
    fn test_parse_type_pointer() {
        let mut parser = parser_from_source("Pointer<Int>");
        let result = parser.parse_type();

        assert!(result.is_ok());
        let type_spec = result.unwrap();
        if let TypeSpecifier::Pointer {
            target_type,
            is_mutable,
            ..
        } = type_spec
        {
            assert!(!is_mutable);
            assert!(matches!(
                *target_type,
                TypeSpecifier::Primitive {
                    type_name: PrimitiveType::Integer,
                    ..
                }
            ));
        } else {
            panic!("Expected Pointer type");
        }
    }

    #[test]
    fn test_parse_type_mut_pointer() {
        let mut parser = parser_from_source("MutPointer<Void>");
        let result = parser.parse_type();

        assert!(result.is_ok());
        let type_spec = result.unwrap();
        if let TypeSpecifier::Pointer {
            target_type,
            is_mutable,
            ..
        } = type_spec
        {
            assert!(is_mutable);
            assert!(matches!(
                *target_type,
                TypeSpecifier::Primitive {
                    type_name: PrimitiveType::Void,
                    ..
                }
            ));
        } else {
            panic!("Expected MutPointer type");
        }
    }

    #[test]
    fn test_parse_type_owned() {
        let mut parser = parser_from_source("^String");
        let result = parser.parse_type();

        assert!(result.is_ok());
        let type_spec = result.unwrap();
        if let TypeSpecifier::Owned {
            ownership,
            base_type,
            ..
        } = type_spec
        {
            assert_eq!(ownership, OwnershipKind::Owned);
            assert!(matches!(
                *base_type,
                TypeSpecifier::Primitive {
                    type_name: PrimitiveType::String,
                    ..
                }
            ));
        } else {
            panic!("Expected Owned type");
        }
    }

    #[test]
    fn test_parse_type_borrowed() {
        let mut parser = parser_from_source("&Int");
        let result = parser.parse_type();

        assert!(result.is_ok());
        let type_spec = result.unwrap();
        if let TypeSpecifier::Owned {
            ownership,
            base_type,
            ..
        } = type_spec
        {
            assert_eq!(ownership, OwnershipKind::Borrowed);
            assert!(matches!(
                *base_type,
                TypeSpecifier::Primitive {
                    type_name: PrimitiveType::Integer,
                    ..
                }
            ));
        } else {
            panic!("Expected Borrowed type");
        }
    }

    #[test]
    fn test_parse_type_borrowed_mut() {
        let mut parser = parser_from_source("&mut Int");
        let result = parser.parse_type();

        assert!(result.is_ok());
        let type_spec = result.unwrap();
        if let TypeSpecifier::Owned {
            ownership,
            base_type,
            ..
        } = type_spec
        {
            assert_eq!(ownership, OwnershipKind::BorrowedMut);
            assert!(matches!(
                *base_type,
                TypeSpecifier::Primitive {
                    type_name: PrimitiveType::Integer,
                    ..
                }
            ));
        } else {
            panic!("Expected BorrowedMut type");
        }
    }

    #[test]
    fn test_parse_type_shared() {
        let mut parser = parser_from_source("~Resource");
        let result = parser.parse_type();

        assert!(result.is_ok());
        let type_spec = result.unwrap();
        if let TypeSpecifier::Owned {
            ownership,
            base_type,
            ..
        } = type_spec
        {
            assert_eq!(ownership, OwnershipKind::Shared);
            // Resource is a user-defined type
            assert!(matches!(*base_type, TypeSpecifier::Named { .. }));
        } else {
            panic!("Expected Shared type");
        }
    }

    #[test]
    fn test_parse_type_named() {
        let mut parser = parser_from_source("MyCustomType");
        let result = parser.parse_type();

        assert!(result.is_ok());
        let type_spec = result.unwrap();
        if let TypeSpecifier::Named { name, .. } = type_spec {
            assert_eq!(name.name, "MyCustomType");
        } else {
            panic!("Expected Named type");
        }
    }

    #[test]
    fn test_parse_type_generic_named() {
        let mut parser = parser_from_source("Result<Int, String>");
        let result = parser.parse_type();

        assert!(result.is_ok());
        let type_spec = result.unwrap();
        if let TypeSpecifier::Generic {
            base_type,
            type_arguments,
            ..
        } = type_spec
        {
            assert_eq!(base_type.name, "Result");
            assert_eq!(type_arguments.len(), 2);
        } else {
            panic!("Expected Generic type");
        }
    }

    #[test]
    fn test_parse_type_complex_ownership() {
        let mut parser = parser_from_source("^Array<&Int>");
        let result = parser.parse_type();

        assert!(result.is_ok());
        let type_spec = result.unwrap();
        if let TypeSpecifier::Owned {
            ownership,
            base_type,
            ..
        } = type_spec
        {
            assert_eq!(ownership, OwnershipKind::Owned);
            if let TypeSpecifier::Array { element_type, .. } = *base_type {
                if let TypeSpecifier::Owned {
                    ownership: inner_ownership,
                    ..
                } = *element_type
                {
                    assert_eq!(inner_ownership, OwnershipKind::Borrowed);
                } else {
                    panic!("Expected borrowed element type");
                }
            } else {
                panic!("Expected Array base type");
            }
        } else {
            panic!("Expected Owned type");
        }
    }

    #[test]
    fn test_parse_type_error_missing_generic_close() {
        let mut parser = parser_from_source("Array<Int");
        let result = parser.parse_type();

        assert!(result.is_err());
    }

    #[test]
    fn test_parse_type_error_missing_map_comma() {
        let mut parser = parser_from_source("Map<String Int>");
        let result = parser.parse_type();

        assert!(result.is_err());
    }

    // ==================== FUNCTION PARSING TESTS ====================

    #[test]
    fn test_parse_function_no_params_no_return() {
        let mut parser = parser_from_source("func foo() { }");
        let result = parser.parse_function();

        assert!(result.is_ok());
        let func = result.unwrap();
        assert_eq!(func.name.name, "foo");
        assert!(func.parameters.is_empty());
        // Default return type should be Void
        assert!(matches!(
            *func.return_type,
            TypeSpecifier::Primitive {
                type_name: PrimitiveType::Void,
                ..
            }
        ));
    }

    #[test]
    fn test_parse_function_with_return_type() {
        let mut parser = parser_from_source("func answer() -> Int { }");
        let result = parser.parse_function();

        assert!(result.is_ok());
        let func = result.unwrap();
        assert_eq!(func.name.name, "answer");
        assert!(func.parameters.is_empty());
        assert!(matches!(
            *func.return_type,
            TypeSpecifier::Primitive {
                type_name: PrimitiveType::Integer,
                ..
            }
        ));
    }

    #[test]
    fn test_parse_function_single_param() {
        let mut parser = parser_from_source("func greet(name: String) { }");
        let result = parser.parse_function();

        assert!(result.is_ok());
        let func = result.unwrap();
        assert_eq!(func.name.name, "greet");
        assert_eq!(func.parameters.len(), 1);
        assert_eq!(func.parameters[0].name.name, "name");
        assert!(matches!(
            *func.parameters[0].param_type,
            TypeSpecifier::Primitive {
                type_name: PrimitiveType::String,
                ..
            }
        ));
    }

    #[test]
    fn test_parse_function_multiple_params() {
        let mut parser = parser_from_source("func add(a: Int, b: Int) -> Int { }");
        let result = parser.parse_function();

        assert!(result.is_ok());
        let func = result.unwrap();
        assert_eq!(func.name.name, "add");
        assert_eq!(func.parameters.len(), 2);
        assert_eq!(func.parameters[0].name.name, "a");
        assert_eq!(func.parameters[1].name.name, "b");
        assert!(matches!(
            *func.return_type,
            TypeSpecifier::Primitive {
                type_name: PrimitiveType::Integer,
                ..
            }
        ));
    }

    #[test]
    fn test_parse_function_complex_types() {
        let mut parser = parser_from_source(
            "func process(items: Array<Int>, config: Map<String, Int>) -> Bool { }",
        );
        let result = parser.parse_function();

        assert!(result.is_ok());
        let func = result.unwrap();
        assert_eq!(func.name.name, "process");
        assert_eq!(func.parameters.len(), 2);
        assert!(matches!(
            *func.parameters[0].param_type,
            TypeSpecifier::Array { .. }
        ));
        assert!(matches!(
            *func.parameters[1].param_type,
            TypeSpecifier::Map { .. }
        ));
    }

    #[test]
    fn test_parse_function_ownership_types() {
        let mut parser =
            parser_from_source("func transfer(owned: ^String, borrowed: &Int) -> Void { }");
        let result = parser.parse_function();

        assert!(result.is_ok());
        let func = result.unwrap();
        assert_eq!(func.parameters.len(), 2);
        assert!(matches!(
            *func.parameters[0].param_type,
            TypeSpecifier::Owned {
                ownership: OwnershipKind::Owned,
                ..
            }
        ));
        assert!(matches!(
            *func.parameters[1].param_type,
            TypeSpecifier::Owned {
                ownership: OwnershipKind::Borrowed,
                ..
            }
        ));
    }

    #[test]
    fn test_parse_function_with_body_content() {
        // Body content is skipped for now, but structure should parse
        let mut parser = parser_from_source("func main() { let x = 42; return x; }");
        let result = parser.parse_function();

        assert!(result.is_ok());
        let func = result.unwrap();
        assert_eq!(func.name.name, "main");
    }

    #[test]
    fn test_parse_function_multiline() {
        let source = r#"
func calculate(
    a: Int,
    b: Int,
    c: Int
) -> Int {
    // body
}
"#;
        let mut parser = parser_from_source(source);
        let result = parser.parse_function();

        assert!(result.is_ok());
        let func = result.unwrap();
        assert_eq!(func.name.name, "calculate");
        assert_eq!(func.parameters.len(), 3);
    }

    #[test]
    fn test_parse_function_error_missing_name() {
        let mut parser = parser_from_source("func () { }");
        let result = parser.parse_function();

        assert!(result.is_err());
    }

    #[test]
    fn test_parse_function_error_missing_open_paren() {
        let mut parser = parser_from_source("func foo) { }");
        let result = parser.parse_function();

        assert!(result.is_err());
    }

    #[test]
    fn test_parse_function_error_missing_close_paren() {
        let mut parser = parser_from_source("func foo( { }");
        let result = parser.parse_function();

        assert!(result.is_err());
    }

    #[test]
    fn test_parse_function_error_missing_body() {
        let mut parser = parser_from_source("func foo()");
        let result = parser.parse_function();

        assert!(result.is_err());
    }

    #[test]
    fn test_parse_function_error_missing_param_type() {
        let mut parser = parser_from_source("func foo(x) { }");
        let result = parser.parse_function();

        assert!(result.is_err());
    }

    #[test]
    fn test_parse_function_pointer_return() {
        let mut parser = parser_from_source("func allocate(size: SizeT) -> Pointer<Void> { }");
        let result = parser.parse_function();

        assert!(result.is_ok());
        let func = result.unwrap();
        assert!(matches!(
            *func.return_type,
            TypeSpecifier::Pointer {
                is_mutable: false,
                ..
            }
        ));
    }

    // ==================== Annotation Parsing Tests ====================

    #[test]
    fn test_parse_annotation_simple() {
        let mut parser = parser_from_source("@test");
        let result = parser.parse_annotation();

        assert!(result.is_ok());
        let annotation = result.unwrap();
        assert_eq!(annotation.name, "test");
        assert!(annotation.arguments.is_empty());
    }

    #[test]
    fn test_parse_annotation_with_labeled_string_arg() {
        let mut parser = parser_from_source("@extern(library: \"libc\")");
        let result = parser.parse_annotation();

        assert!(result.is_ok());
        let annotation = result.unwrap();
        assert_eq!(annotation.name, "extern");
        assert_eq!(annotation.arguments.len(), 1);
        assert_eq!(annotation.arguments[0].label, Some("library".to_string()));
        assert!(
            matches!(&annotation.arguments[0].value, AnnotationValue::String(s) if s == "libc")
        );
    }

    #[test]
    fn test_parse_annotation_with_multiple_args() {
        let mut parser = parser_from_source("@extern(library: \"libc\", symbol: \"malloc\")");
        let result = parser.parse_annotation();

        assert!(result.is_ok());
        let annotation = result.unwrap();
        assert_eq!(annotation.name, "extern");
        assert_eq!(annotation.arguments.len(), 2);
        assert_eq!(annotation.arguments[0].label, Some("library".to_string()));
        assert_eq!(annotation.arguments[1].label, Some("symbol".to_string()));
        assert!(
            matches!(&annotation.arguments[1].value, AnnotationValue::String(s) if s == "malloc")
        );
    }

    #[test]
    fn test_parse_annotation_with_braced_expression() {
        let mut parser = parser_from_source("@requires({n > 0})");
        let result = parser.parse_annotation();

        assert!(result.is_ok());
        let annotation = result.unwrap();
        assert_eq!(annotation.name, "requires");
        assert_eq!(annotation.arguments.len(), 1);
        assert!(annotation.arguments[0].label.is_none());
        assert!(
            matches!(&annotation.arguments[0].value, AnnotationValue::Expression(expr, _) if expr.contains("n") && expr.contains(">") && expr.contains("0"))
        );
    }

    #[test]
    fn test_parse_annotation_with_identifier_value() {
        let mut parser = parser_from_source("@category(math)");
        let result = parser.parse_annotation();

        assert!(result.is_ok());
        let annotation = result.unwrap();
        assert_eq!(annotation.name, "category");
        assert_eq!(annotation.arguments.len(), 1);
        assert!(
            matches!(&annotation.arguments[0].value, AnnotationValue::Identifier(s) if s == "math")
        );
    }

    #[test]
    fn test_parse_annotation_with_integer_value() {
        let mut parser = parser_from_source("@priority(10)");
        let result = parser.parse_annotation();

        assert!(result.is_ok());
        let annotation = result.unwrap();
        assert_eq!(annotation.name, "priority");
        assert_eq!(annotation.arguments.len(), 1);
        assert!(matches!(
            &annotation.arguments[0].value,
            AnnotationValue::Integer(10)
        ));
    }

    #[test]
    fn test_parse_annotation_with_boolean_value() {
        let mut parser = parser_from_source("@deprecated(true)");
        let result = parser.parse_annotation();

        assert!(result.is_ok());
        let annotation = result.unwrap();
        assert_eq!(annotation.name, "deprecated");
        assert_eq!(annotation.arguments.len(), 1);
        assert!(matches!(
            &annotation.arguments[0].value,
            AnnotationValue::Boolean(true)
        ));
    }

    #[test]
    fn test_parse_annotation_empty_parens() {
        let mut parser = parser_from_source("@marker()");
        let result = parser.parse_annotation();

        assert!(result.is_ok());
        let annotation = result.unwrap();
        assert_eq!(annotation.name, "marker");
        assert!(annotation.arguments.is_empty());
    }

    #[test]
    fn test_parse_annotation_error_missing_name() {
        let mut parser = parser_from_source("@()");
        let result = parser.parse_annotation();

        assert!(result.is_err());
    }

    #[test]
    fn test_parse_annotation_error_missing_close_paren() {
        let mut parser = parser_from_source("@test(x: 1");
        let result = parser.parse_annotation();

        assert!(result.is_err());
    }

    #[test]
    fn test_parse_external_function_basic() {
        let mut parser = parser_from_source(
            "@extern(library: \"libc\") func malloc(size: SizeT) -> Pointer<Void>;",
        );

        // First parse the annotation
        let annotation = parser.parse_annotation().unwrap();
        assert_eq!(annotation.name, "extern");

        // Then parse the external function
        let result = parser.parse_external_function(annotation);

        assert!(result.is_ok());
        let ext_func = result.unwrap();
        assert_eq!(ext_func.name.name, "malloc");
        assert_eq!(ext_func.library, "libc");
        assert_eq!(ext_func.parameters.len(), 1);
        assert_eq!(ext_func.parameters[0].name.name, "size");
    }

    #[test]
    fn test_parse_external_function_with_symbol() {
        let mut parser = parser_from_source("@extern(library: \"c\", symbol: \"_malloc\") func malloc(size: SizeT) -> Pointer<Void>;");

        let annotation = parser.parse_annotation().unwrap();
        let result = parser.parse_external_function(annotation);

        assert!(result.is_ok());
        let ext_func = result.unwrap();
        assert_eq!(ext_func.name.name, "malloc");
        assert_eq!(ext_func.library, "c");
        assert_eq!(ext_func.symbol.as_deref(), Some("_malloc"));
    }

    #[test]
    fn test_parse_external_function_multiple_params() {
        let mut parser = parser_from_source("@extern(library: \"libc\") func memcpy(dest: Pointer<Void>, src: Pointer<Void>, n: SizeT) -> Pointer<Void>;");

        let annotation = parser.parse_annotation().unwrap();
        let result = parser.parse_external_function(annotation);

        assert!(result.is_ok());
        let ext_func = result.unwrap();
        assert_eq!(ext_func.name.name, "memcpy");
        assert_eq!(ext_func.parameters.len(), 3);
    }

    #[test]
    fn test_parse_external_function_error_missing_semicolon() {
        let mut parser = parser_from_source(
            "@extern(library: \"libc\") func malloc(size: SizeT) -> Pointer<Void>",
        );

        let annotation = parser.parse_annotation().unwrap();
        let result = parser.parse_external_function(annotation);

        assert!(result.is_err());
    }

    // ==================== Variable Declaration Tests ====================

    #[test]
    fn test_parse_variable_declaration_with_type_and_value() {
        let mut parser = parser_from_source("let x: Int = 42;");
        let result = parser.parse_variable_declaration();

        assert!(result.is_ok());
        let stmt = result.unwrap();
        if let Statement::VariableDeclaration {
            name,
            type_spec,
            mutability,
            initial_value,
            ..
        } = stmt
        {
            assert_eq!(name.name, "x");
            assert!(matches!(
                *type_spec,
                TypeSpecifier::Primitive {
                    type_name: PrimitiveType::Integer,
                    ..
                }
            ));
            assert!(matches!(mutability, Mutability::Immutable));
            assert!(initial_value.is_some());
            let value = initial_value.unwrap();
            assert!(matches!(
                *value,
                Expression::IntegerLiteral { value: 42, .. }
            ));
        } else {
            panic!("Expected VariableDeclaration");
        }
    }

    #[test]
    fn test_parse_variable_declaration_mutable() {
        let mut parser = parser_from_source("let mut counter: Int = 0;");
        let result = parser.parse_variable_declaration();

        assert!(result.is_ok());
        let stmt = result.unwrap();
        if let Statement::VariableDeclaration {
            name, mutability, ..
        } = stmt
        {
            assert_eq!(name.name, "counter");
            assert!(matches!(mutability, Mutability::Mutable));
        } else {
            panic!("Expected VariableDeclaration");
        }
    }

    #[test]
    fn test_parse_variable_declaration_string_value() {
        let mut parser = parser_from_source("let name: String = \"hello\";");
        let result = parser.parse_variable_declaration();

        assert!(result.is_ok());
        let stmt = result.unwrap();
        if let Statement::VariableDeclaration { initial_value, .. } = stmt {
            assert!(initial_value.is_some());
            let value = initial_value.unwrap();
            assert!(
                matches!(*value, Expression::StringLiteral { ref value, .. } if value == "hello")
            );
        } else {
            panic!("Expected VariableDeclaration");
        }
    }

    #[test]
    fn test_parse_variable_declaration_float_value() {
        let mut parser = parser_from_source("let pi: Float = 3.14;");
        let result = parser.parse_variable_declaration();

        assert!(result.is_ok());
        let stmt = result.unwrap();
        if let Statement::VariableDeclaration { initial_value, .. } = stmt {
            assert!(initial_value.is_some());
            let value = initial_value.unwrap();
            assert!(matches!(*value, Expression::FloatLiteral { .. }));
        } else {
            panic!("Expected VariableDeclaration");
        }
    }

    #[test]
    fn test_parse_variable_declaration_bool_value() {
        let mut parser = parser_from_source("let flag: Bool = true;");
        let result = parser.parse_variable_declaration();

        assert!(result.is_ok());
        let stmt = result.unwrap();
        if let Statement::VariableDeclaration { initial_value, .. } = stmt {
            assert!(initial_value.is_some());
            let value = initial_value.unwrap();
            assert!(matches!(
                *value,
                Expression::BooleanLiteral { value: true, .. }
            ));
        } else {
            panic!("Expected VariableDeclaration");
        }
    }

    #[test]
    fn test_parse_variable_declaration_char_value() {
        let mut parser = parser_from_source("let ch: Char = 'a';");
        let result = parser.parse_variable_declaration();

        assert!(result.is_ok());
        let stmt = result.unwrap();
        if let Statement::VariableDeclaration { initial_value, .. } = stmt {
            assert!(initial_value.is_some());
            let value = initial_value.unwrap();
            assert!(matches!(
                *value,
                Expression::CharacterLiteral { value: 'a', .. }
            ));
        } else {
            panic!("Expected VariableDeclaration");
        }
    }

    #[test]
    fn test_parse_variable_declaration_no_initializer() {
        let mut parser = parser_from_source("let x: Int;");
        let result = parser.parse_variable_declaration();

        assert!(result.is_ok());
        let stmt = result.unwrap();
        if let Statement::VariableDeclaration {
            name,
            initial_value,
            ..
        } = stmt
        {
            assert_eq!(name.name, "x");
            assert!(initial_value.is_none());
        } else {
            panic!("Expected VariableDeclaration");
        }
    }

    #[test]
    fn test_parse_variable_declaration_type_inference() {
        let mut parser = parser_from_source("let x = 42;");
        let result = parser.parse_variable_declaration();

        assert!(result.is_ok());
        let stmt = result.unwrap();
        if let Statement::VariableDeclaration {
            name,
            type_spec,
            initial_value,
            ..
        } = stmt
        {
            assert_eq!(name.name, "x");
            // Type should be _inferred placeholder
            assert!(
                matches!(*type_spec, TypeSpecifier::Named { ref name, .. } if name.name == "_inferred")
            );
            assert!(initial_value.is_some());
        } else {
            panic!("Expected VariableDeclaration");
        }
    }

    #[test]
    fn test_parse_variable_declaration_variable_reference() {
        let mut parser = parser_from_source("let y: Int = x;");
        let result = parser.parse_variable_declaration();

        assert!(result.is_ok());
        let stmt = result.unwrap();
        if let Statement::VariableDeclaration { initial_value, .. } = stmt {
            assert!(initial_value.is_some());
            let value = initial_value.unwrap();
            assert!(matches!(*value, Expression::Variable { ref name, .. } if name.name == "x"));
        } else {
            panic!("Expected VariableDeclaration");
        }
    }

    #[test]
    fn test_parse_variable_declaration_error_missing_semicolon() {
        let mut parser = parser_from_source("let x: Int = 42");
        let result = parser.parse_variable_declaration();

        assert!(result.is_err());
    }

    #[test]
    fn test_parse_variable_declaration_error_missing_name() {
        let mut parser = parser_from_source("let : Int = 42;");
        let result = parser.parse_variable_declaration();

        assert!(result.is_err());
    }

    // ==================== Expression Parsing Tests ====================

    #[test]
    fn test_parse_expression_integer() {
        let mut parser = parser_from_source("42");
        let result = parser.parse_expression();

        assert!(result.is_ok());
        let expr = result.unwrap();
        assert!(matches!(expr, Expression::IntegerLiteral { value: 42, .. }));
    }

    #[test]
    fn test_parse_expression_float() {
        let mut parser = parser_from_source("3.14");
        let result = parser.parse_expression();

        assert!(result.is_ok());
        let expr = result.unwrap();
        assert!(matches!(expr, Expression::FloatLiteral { .. }));
    }

    #[test]
    fn test_parse_expression_string() {
        let mut parser = parser_from_source("\"hello world\"");
        let result = parser.parse_expression();

        assert!(result.is_ok());
        let expr = result.unwrap();
        assert!(
            matches!(expr, Expression::StringLiteral { ref value, .. } if value == "hello world")
        );
    }

    #[test]
    fn test_parse_expression_boolean() {
        let mut parser = parser_from_source("true");
        let result = parser.parse_expression();

        assert!(result.is_ok());
        let expr = result.unwrap();
        assert!(matches!(
            expr,
            Expression::BooleanLiteral { value: true, .. }
        ));
    }

    #[test]
    fn test_parse_expression_identifier() {
        let mut parser = parser_from_source("myVar");
        let result = parser.parse_expression();

        assert!(result.is_ok());
        let expr = result.unwrap();
        assert!(matches!(expr, Expression::Variable { ref name, .. } if name.name == "myVar"));
    }

    // ==================== Assignment Parsing Tests ====================

    #[test]
    fn test_parse_assignment_simple() {
        let mut parser = parser_from_source("x = 42;");
        let result = parser.parse_assignment();

        assert!(result.is_ok());
        let stmt = result.unwrap();
        if let Statement::Assignment { target, value, .. } = stmt {
            assert!(matches!(target, AssignmentTarget::Variable { ref name } if name.name == "x"));
            assert!(matches!(
                *value,
                Expression::IntegerLiteral { value: 42, .. }
            ));
        } else {
            panic!("Expected Assignment");
        }
    }

    #[test]
    fn test_parse_assignment_string_value() {
        let mut parser = parser_from_source("name = \"hello\";");
        let result = parser.parse_assignment();

        assert!(result.is_ok());
        let stmt = result.unwrap();
        if let Statement::Assignment { target, value, .. } = stmt {
            assert!(
                matches!(target, AssignmentTarget::Variable { ref name } if name.name == "name")
            );
            assert!(
                matches!(*value, Expression::StringLiteral { ref value, .. } if value == "hello")
            );
        } else {
            panic!("Expected Assignment");
        }
    }

    #[test]
    fn test_parse_assignment_variable_value() {
        let mut parser = parser_from_source("y = x;");
        let result = parser.parse_assignment();

        assert!(result.is_ok());
        let stmt = result.unwrap();
        if let Statement::Assignment { value, .. } = stmt {
            assert!(matches!(*value, Expression::Variable { ref name, .. } if name.name == "x"));
        } else {
            panic!("Expected Assignment");
        }
    }

    #[test]
    fn test_parse_assignment_array_element() {
        let mut parser = parser_from_source("arr[0] = 42;");
        let result = parser.parse_assignment();

        assert!(result.is_ok());
        let stmt = result.unwrap();
        if let Statement::Assignment { target, .. } = stmt {
            if let AssignmentTarget::ArrayElement { array, index } = target {
                assert!(
                    matches!(*array, Expression::Variable { ref name, .. } if name.name == "arr")
                );
                assert!(matches!(
                    *index,
                    Expression::IntegerLiteral { value: 0, .. }
                ));
            } else {
                panic!("Expected ArrayElement target");
            }
        } else {
            panic!("Expected Assignment");
        }
    }

    #[test]
    fn test_parse_assignment_struct_field() {
        let mut parser = parser_from_source("point.x = 10;");
        let result = parser.parse_assignment();

        assert!(result.is_ok());
        let stmt = result.unwrap();
        if let Statement::Assignment { target, .. } = stmt {
            if let AssignmentTarget::StructField {
                instance,
                field_name,
            } = target
            {
                assert!(
                    matches!(*instance, Expression::Variable { ref name, .. } if name.name == "point")
                );
                assert_eq!(field_name.name, "x");
            } else {
                panic!("Expected StructField target");
            }
        } else {
            panic!("Expected Assignment");
        }
    }

    #[test]
    fn test_parse_assignment_error_missing_equals() {
        let mut parser = parser_from_source("x 42;");
        let result = parser.parse_assignment();

        assert!(result.is_err());
    }

    #[test]
    fn test_parse_assignment_error_missing_semicolon() {
        let mut parser = parser_from_source("x = 42");
        let result = parser.parse_assignment();

        assert!(result.is_err());
    }

    #[test]
    fn test_parse_assignment_error_missing_value() {
        let mut parser = parser_from_source("x = ;");
        let result = parser.parse_assignment();

        assert!(result.is_err());
    }

    // ==================== Binary Expression Parsing Tests ====================

    #[test]
    fn test_parse_binary_add() {
        let mut parser = parser_from_source("{1 + 2}");
        let result = parser.parse_expression();

        assert!(result.is_ok());
        let expr = result.unwrap();
        if let Expression::Add { left, right, .. } = expr {
            assert!(matches!(*left, Expression::IntegerLiteral { value: 1, .. }));
            assert!(matches!(
                *right,
                Expression::IntegerLiteral { value: 2, .. }
            ));
        } else {
            panic!("Expected Add expression, got {:?}", expr);
        }
    }

    #[test]
    fn test_parse_binary_subtract() {
        let mut parser = parser_from_source("{10 - 5}");
        let result = parser.parse_expression();

        assert!(result.is_ok());
        let expr = result.unwrap();
        assert!(matches!(expr, Expression::Subtract { .. }));
    }

    #[test]
    fn test_parse_binary_multiply() {
        let mut parser = parser_from_source("{3 * 4}");
        let result = parser.parse_expression();

        assert!(result.is_ok());
        let expr = result.unwrap();
        assert!(matches!(expr, Expression::Multiply { .. }));
    }

    #[test]
    fn test_parse_binary_divide() {
        let mut parser = parser_from_source("{10 / 2}");
        let result = parser.parse_expression();

        assert!(result.is_ok());
        let expr = result.unwrap();
        assert!(matches!(expr, Expression::Divide { .. }));
    }

    #[test]
    fn test_parse_binary_modulo() {
        let mut parser = parser_from_source("{7 % 3}");
        let result = parser.parse_expression();

        assert!(result.is_ok());
        let expr = result.unwrap();
        assert!(matches!(expr, Expression::Modulo { .. }));
    }

    #[test]
    fn test_parse_binary_equals() {
        let mut parser = parser_from_source("{x == y}");
        let result = parser.parse_expression();

        assert!(result.is_ok());
        let expr = result.unwrap();
        assert!(matches!(expr, Expression::Equals { .. }));
    }

    #[test]
    fn test_parse_binary_not_equals() {
        let mut parser = parser_from_source("{a != b}");
        let result = parser.parse_expression();

        assert!(result.is_ok());
        let expr = result.unwrap();
        assert!(matches!(expr, Expression::NotEquals { .. }));
    }

    #[test]
    fn test_parse_binary_less_than() {
        let mut parser = parser_from_source("{x < 10}");
        let result = parser.parse_expression();

        assert!(result.is_ok());
        let expr = result.unwrap();
        assert!(matches!(expr, Expression::LessThan { .. }));
    }

    #[test]
    fn test_parse_binary_less_equal() {
        let mut parser = parser_from_source("{x <= 10}");
        let result = parser.parse_expression();

        assert!(result.is_ok());
        let expr = result.unwrap();
        assert!(matches!(expr, Expression::LessThanOrEqual { .. }));
    }

    #[test]
    fn test_parse_binary_greater_than() {
        let mut parser = parser_from_source("{x > 0}");
        let result = parser.parse_expression();

        assert!(result.is_ok());
        let expr = result.unwrap();
        assert!(matches!(expr, Expression::GreaterThan { .. }));
    }

    #[test]
    fn test_parse_binary_greater_equal() {
        let mut parser = parser_from_source("{x >= 0}");
        let result = parser.parse_expression();

        assert!(result.is_ok());
        let expr = result.unwrap();
        assert!(matches!(expr, Expression::GreaterThanOrEqual { .. }));
    }

    #[test]
    fn test_parse_binary_and() {
        let mut parser = parser_from_source("{a && b}");
        let result = parser.parse_expression();

        assert!(result.is_ok());
        let expr = result.unwrap();
        assert!(matches!(expr, Expression::LogicalAnd { .. }));
    }

    #[test]
    fn test_parse_binary_or() {
        let mut parser = parser_from_source("{a || b}");
        let result = parser.parse_expression();

        assert!(result.is_ok());
        let expr = result.unwrap();
        assert!(matches!(expr, Expression::LogicalOr { .. }));
    }

    #[test]
    fn test_parse_binary_nested() {
        // {1 + {2 * 3}} - nested binary expressions
        let mut parser = parser_from_source("{1 + {2 * 3}}");
        let result = parser.parse_expression();

        assert!(result.is_ok());
        let expr = result.unwrap();
        if let Expression::Add { left, right, .. } = expr {
            assert!(matches!(*left, Expression::IntegerLiteral { value: 1, .. }));
            assert!(matches!(*right, Expression::Multiply { .. }));
        } else {
            panic!("Expected Add with nested Multiply");
        }
    }

    #[test]
    fn test_parse_binary_with_variables() {
        let mut parser = parser_from_source("{x + y}");
        let result = parser.parse_expression();

        assert!(result.is_ok());
        let expr = result.unwrap();
        if let Expression::Add { left, right, .. } = expr {
            assert!(matches!(*left, Expression::Variable { ref name, .. } if name.name == "x"));
            assert!(matches!(*right, Expression::Variable { ref name, .. } if name.name == "y"));
        } else {
            panic!("Expected Add expression");
        }
    }

    #[test]
    fn test_parse_binary_error_missing_close_brace() {
        let mut parser = parser_from_source("{1 + 2");
        let result = parser.parse_expression();

        assert!(result.is_err());
    }

    #[test]
    fn test_parse_binary_error_missing_operator() {
        let mut parser = parser_from_source("{1 2}");
        let result = parser.parse_expression();

        assert!(result.is_err());
    }

    #[test]
    fn test_parse_binary_error_missing_right_operand() {
        let mut parser = parser_from_source("{1 + }");
        let result = parser.parse_expression();

        assert!(result.is_err());
    }

    // ==================== Control Flow Tests ====================

    #[test]
    fn test_parse_return_with_value() {
        let mut parser = parser_from_source("return 42;");
        let result = parser.parse_return_statement();

        assert!(result.is_ok());
        let stmt = result.unwrap();
        if let Statement::Return { value, .. } = stmt {
            assert!(value.is_some());
            assert!(matches!(
                *value.unwrap(),
                Expression::IntegerLiteral { value: 42, .. }
            ));
        } else {
            panic!("Expected Return statement");
        }
    }

    #[test]
    fn test_parse_return_without_value() {
        let mut parser = parser_from_source("return;");
        let result = parser.parse_return_statement();

        assert!(result.is_ok());
        let stmt = result.unwrap();
        if let Statement::Return { value, .. } = stmt {
            assert!(value.is_none());
        } else {
            panic!("Expected Return statement");
        }
    }

    #[test]
    fn test_parse_return_with_expression() {
        let mut parser = parser_from_source("return {x + y};");
        let result = parser.parse_return_statement();

        assert!(result.is_ok());
        let stmt = result.unwrap();
        if let Statement::Return { value, .. } = stmt {
            assert!(value.is_some());
            assert!(matches!(*value.unwrap(), Expression::Add { .. }));
        } else {
            panic!("Expected Return statement");
        }
    }

    #[test]
    fn test_parse_when_simple() {
        let mut parser = parser_from_source("when true { }");
        let result = parser.parse_when_statement();

        assert!(result.is_ok());
        let stmt = result.unwrap();
        if let Statement::If {
            condition,
            else_ifs,
            else_block,
            ..
        } = stmt
        {
            assert!(matches!(
                *condition,
                Expression::BooleanLiteral { value: true, .. }
            ));
            assert!(else_ifs.is_empty());
            assert!(else_block.is_none());
        } else {
            panic!("Expected If statement");
        }
    }

    #[test]
    fn test_parse_when_with_else() {
        let mut parser = parser_from_source("when x { } else { }");
        let result = parser.parse_when_statement();

        assert!(result.is_ok());
        let stmt = result.unwrap();
        if let Statement::If { else_block, .. } = stmt {
            assert!(else_block.is_some());
        } else {
            panic!("Expected If statement");
        }
    }

    #[test]
    fn test_parse_when_with_else_when() {
        let mut parser = parser_from_source("when x { } else when y { } else { }");
        let result = parser.parse_when_statement();

        assert!(result.is_ok());
        let stmt = result.unwrap();
        if let Statement::If {
            else_ifs,
            else_block,
            ..
        } = stmt
        {
            assert_eq!(else_ifs.len(), 1);
            assert!(else_block.is_some());
        } else {
            panic!("Expected If statement");
        }
    }

    #[test]
    fn test_parse_when_with_body() {
        let mut parser = parser_from_source("when {x > 0} { return x; }");
        let result = parser.parse_when_statement();

        assert!(result.is_ok());
        let stmt = result.unwrap();
        if let Statement::If { then_block, .. } = stmt {
            assert_eq!(then_block.statements.len(), 1);
        } else {
            panic!("Expected If statement");
        }
    }

    #[test]
    fn test_parse_while_loop() {
        let mut parser = parser_from_source("while true { }");
        let result = parser.parse_while_loop();

        assert!(result.is_ok());
        let stmt = result.unwrap();
        if let Statement::WhileLoop { condition, .. } = stmt {
            assert!(matches!(
                *condition,
                Expression::BooleanLiteral { value: true, .. }
            ));
        } else {
            panic!("Expected WhileLoop statement");
        }
    }

    #[test]
    fn test_parse_while_with_condition() {
        let mut parser = parser_from_source("while {i < 10} { }");
        let result = parser.parse_while_loop();

        assert!(result.is_ok());
        let stmt = result.unwrap();
        assert!(matches!(stmt, Statement::WhileLoop { .. }));
    }

    #[test]
    fn test_parse_while_with_body() {
        let mut parser = parser_from_source("while {x > 0} { x = {x - 1}; }");
        let result = parser.parse_while_loop();

        assert!(result.is_ok());
        let stmt = result.unwrap();
        if let Statement::WhileLoop { body, .. } = stmt {
            assert_eq!(body.statements.len(), 1);
        } else {
            panic!("Expected WhileLoop statement");
        }
    }

    #[test]
    fn test_parse_break() {
        let mut parser = parser_from_source("break;");
        let result = parser.parse_break_statement();

        assert!(result.is_ok());
        assert!(matches!(result.unwrap(), Statement::Break { .. }));
    }

    #[test]
    fn test_parse_continue() {
        let mut parser = parser_from_source("continue;");
        let result = parser.parse_continue_statement();

        assert!(result.is_ok());
        assert!(matches!(result.unwrap(), Statement::Continue { .. }));
    }

    #[test]
    fn test_parse_block_empty() {
        let mut parser = parser_from_source("{ }");
        let result = parser.parse_block();

        assert!(result.is_ok());
        let block = result.unwrap();
        assert!(block.statements.is_empty());
    }

    #[test]
    fn test_parse_block_with_statements() {
        let mut parser = parser_from_source("{ let x: Int = 1; let y: Int = 2; }");
        let result = parser.parse_block();

        assert!(result.is_ok());
        let block = result.unwrap();
        assert_eq!(block.statements.len(), 2);
    }

    #[test]
    fn test_parse_statement_let() {
        let mut parser = parser_from_source("let x: Int = 42;");
        let result = parser.parse_statement();

        assert!(result.is_ok());
        assert!(matches!(
            result.unwrap(),
            Statement::VariableDeclaration { .. }
        ));
    }

    #[test]
    fn test_parse_statement_return() {
        let mut parser = parser_from_source("return 0;");
        let result = parser.parse_statement();

        assert!(result.is_ok());
        assert!(matches!(result.unwrap(), Statement::Return { .. }));
    }

    #[test]
    fn test_parse_statement_when() {
        let mut parser = parser_from_source("when true { }");
        let result = parser.parse_statement();

        assert!(result.is_ok());
        assert!(matches!(result.unwrap(), Statement::If { .. }));
    }

    #[test]
    fn test_parse_statement_while() {
        let mut parser = parser_from_source("while true { }");
        let result = parser.parse_statement();

        assert!(result.is_ok());
        assert!(matches!(result.unwrap(), Statement::WhileLoop { .. }));
    }

    #[test]
    fn test_parse_statement_assignment() {
        let mut parser = parser_from_source("x = 42;");
        let result = parser.parse_statement();

        assert!(result.is_ok());
        assert!(matches!(result.unwrap(), Statement::Assignment { .. }));
    }

    // ==================== Struct Parsing Tests ====================

    #[test]
    fn test_parse_struct_simple() {
        let mut parser = parser_from_source("struct Point { x: Float, y: Float }");
        let result = parser.parse_struct();

        assert!(result.is_ok());
        let typedef = result.unwrap();
        if let TypeDefinition::Structured { name, fields, .. } = typedef {
            assert_eq!(name.name, "Point");
            assert_eq!(fields.len(), 2);
            assert_eq!(fields[0].name.name, "x");
            assert_eq!(fields[1].name.name, "y");
        } else {
            panic!("Expected Structured type");
        }
    }

    #[test]
    fn test_parse_struct_empty() {
        let mut parser = parser_from_source("struct Empty { }");
        let result = parser.parse_struct();

        assert!(result.is_ok());
        let typedef = result.unwrap();
        if let TypeDefinition::Structured { name, fields, .. } = typedef {
            assert_eq!(name.name, "Empty");
            assert!(fields.is_empty());
        } else {
            panic!("Expected Structured type");
        }
    }

    #[test]
    fn test_parse_struct_single_field() {
        let mut parser = parser_from_source("struct Wrapper { value: Int }");
        let result = parser.parse_struct();

        assert!(result.is_ok());
        let typedef = result.unwrap();
        if let TypeDefinition::Structured { fields, .. } = typedef {
            assert_eq!(fields.len(), 1);
            assert_eq!(fields[0].name.name, "value");
        } else {
            panic!("Expected Structured type");
        }
    }

    #[test]
    fn test_parse_struct_complex_types() {
        let mut parser =
            parser_from_source("struct Data { items: Array<Int>, lookup: Map<String, Int> }");
        let result = parser.parse_struct();

        assert!(result.is_ok());
        let typedef = result.unwrap();
        if let TypeDefinition::Structured { fields, .. } = typedef {
            assert_eq!(fields.len(), 2);
            assert!(matches!(*fields[0].field_type, TypeSpecifier::Array { .. }));
            assert!(matches!(*fields[1].field_type, TypeSpecifier::Map { .. }));
        } else {
            panic!("Expected Structured type");
        }
    }

    #[test]
    fn test_parse_struct_error_missing_brace() {
        let mut parser = parser_from_source("struct Point x: Float; }");
        let result = parser.parse_struct();

        assert!(result.is_err());
    }

    #[test]
    fn test_parse_struct_error_missing_field_type() {
        let mut parser = parser_from_source("struct Point { x; }");
        let result = parser.parse_struct();

        assert!(result.is_err());
    }

    #[test]
    fn test_parse_struct_single_field_no_trailing_comma() {
        // With comma-separated fields and optional trailing comma, this should succeed
        let mut parser = parser_from_source("struct Point { x: Float }");
        let result = parser.parse_struct();

        assert!(result.is_ok());
    }

    // ==================== Enum Parsing Tests ====================

    #[test]
    fn test_parse_enum_simple() {
        let mut parser = parser_from_source("enum Color { case Red; case Green; case Blue; }");
        let result = parser.parse_enum();

        assert!(result.is_ok());
        let typedef = result.unwrap();
        if let TypeDefinition::Enumeration { name, variants, .. } = typedef {
            assert_eq!(name.name, "Color");
            assert_eq!(variants.len(), 3);
            assert_eq!(variants[0].name.name, "Red");
            assert_eq!(variants[1].name.name, "Green");
            assert_eq!(variants[2].name.name, "Blue");
        } else {
            panic!("Expected Enumeration type");
        }
    }

    #[test]
    fn test_parse_enum_empty() {
        let mut parser = parser_from_source("enum Empty { }");
        let result = parser.parse_enum();

        assert!(result.is_ok());
        let typedef = result.unwrap();
        if let TypeDefinition::Enumeration { variants, .. } = typedef {
            assert!(variants.is_empty());
        } else {
            panic!("Expected Enumeration type");
        }
    }

    #[test]
    fn test_parse_enum_with_associated_type() {
        let mut parser = parser_from_source("enum Result { case Ok(Int); case Error(String); }");
        let result = parser.parse_enum();

        assert!(result.is_ok());
        let typedef = result.unwrap();
        if let TypeDefinition::Enumeration { name, variants, .. } = typedef {
            assert_eq!(name.name, "Result");
            assert_eq!(variants.len(), 2);
            assert!(!variants[0].associated_types.is_empty());
            assert!(!variants[1].associated_types.is_empty());
        } else {
            panic!("Expected Enumeration type");
        }
    }

    #[test]
    fn test_parse_enum_mixed_variants() {
        let mut parser = parser_from_source("enum Option { case Some(Int); case None; }");
        let result = parser.parse_enum();

        assert!(result.is_ok());
        let typedef = result.unwrap();
        if let TypeDefinition::Enumeration { variants, .. } = typedef {
            assert_eq!(variants.len(), 2);
            assert!(!variants[0].associated_types.is_empty());
            assert!(variants[1].associated_types.is_empty());
        } else {
            panic!("Expected Enumeration type");
        }
    }

    #[test]
    fn test_parse_enum_error_missing_case() {
        let mut parser = parser_from_source("enum Color { Red; }");
        let result = parser.parse_enum();

        assert!(result.is_err());
    }

    #[test]
    fn test_parse_enum_error_missing_semicolon() {
        let mut parser = parser_from_source("enum Color { case Red }");
        let result = parser.parse_enum();

        assert!(result.is_err());
    }

    // ==================== Integration Tests ====================

    #[test]
    fn test_integration_function_with_body() {
        let source = r#"
func add(a: Int, b: Int) -> Int {
    let result: Int = {a + b};
    return result;
}
"#;
        let mut parser = parser_from_source(source);
        let result = parser.parse_function();

        assert!(result.is_ok());
        let func = result.unwrap();
        assert_eq!(func.name.name, "add");
        assert_eq!(func.parameters.len(), 2);
        assert_eq!(func.body.statements.len(), 2);
    }

    #[test]
    fn test_integration_function_with_control_flow() {
        let source = r#"
func abs(n: Int) -> Int {
    when {n < 0} {
        return {0 - n};
    } else {
        return n;
    }
}
"#;
        let mut parser = parser_from_source(source);
        let result = parser.parse_function();

        assert!(result.is_ok());
        let func = result.unwrap();
        assert_eq!(func.name.name, "abs");
        assert_eq!(func.body.statements.len(), 1);
        assert!(matches!(func.body.statements[0], Statement::If { .. }));
    }

    #[test]
    fn test_integration_function_with_loop() {
        let source = r#"
func sum(n: Int) -> Int {
    let mut total: Int = 0;
    let mut i: Int = 0;
    while {i < n} {
        total = {total + i};
        i = {i + 1};
    }
    return total;
}
"#;
        let mut parser = parser_from_source(source);
        let result = parser.parse_function();

        assert!(result.is_ok());
        let func = result.unwrap();
        assert_eq!(func.body.statements.len(), 4); // 2 lets + while + return
    }

    #[test]
    fn test_integration_struct_and_function() {
        let struct_source = "struct Point { x: Float, y: Float }";
        let mut parser = parser_from_source(struct_source);
        let struct_result = parser.parse_struct();
        assert!(struct_result.is_ok());

        let func_source = r#"
func distance(p: Point) -> Float {
    return {p.x + p.y};
}
"#;
        let mut parser = parser_from_source(func_source);
        let func_result = parser.parse_function();
        assert!(func_result.is_ok(), "Parse error: {:?}", func_result.err());
    }

    #[test]
    fn test_integration_nested_control_flow() {
        let source = r#"
func classify(n: Int) -> Int {
    when {n > 0} {
        when {n > 100} {
            return 2;
        } else {
            return 1;
        }
    } else when {n < 0} {
        return 0;
    } else {
        return 0;
    }
}
"#;
        let mut parser = parser_from_source(source);
        let result = parser.parse_function();

        assert!(result.is_ok());
        let func = result.unwrap();
        if let Statement::If {
            else_ifs,
            else_block,
            ..
        } = &func.body.statements[0]
        {
            assert_eq!(else_ifs.len(), 1);
            assert!(else_block.is_some());
        } else {
            panic!("Expected If statement");
        }
    }

    #[test]
    fn test_integration_module_with_import() {
        let source = r#"
module math;

import std.io;
"#;
        let mut parser = parser_from_source(source);
        let result = parser.parse_module();

        assert!(result.is_ok(), "Parse error: {:?}", result.err());
        let module = result.unwrap();
        assert_eq!(module.name.name, "math");
        assert_eq!(module.imports.len(), 1);
    }

    #[test]
    fn test_integration_complex_expressions() {
        let source = r#"
func compute(a: Int, b: Int, c: Int) -> Int {
    let x: Int = {a + b};
    let y: Int = {x * c};
    let z: Int = {{a + b} * {c - 1}};
    return z;
}
"#;
        let mut parser = parser_from_source(source);
        let result = parser.parse_function();

        assert!(result.is_ok(), "Parse error: {:?}", result.err());
        let func = result.unwrap();
        assert_eq!(func.body.statements.len(), 4);
    }

    #[test]
    fn test_integration_array_operations() {
        let source = r#"
func process(items: Array<Int>) -> Int {
    let first: Int = items[0];
    items[0] = 42;
    return first;
}
"#;
        let mut parser = parser_from_source(source);
        let result = parser.parse_function();

        assert!(result.is_ok(), "Parse error: {:?}", result.err());
        let func = result.unwrap();
        assert_eq!(func.body.statements.len(), 3);
    }

    #[test]
    fn test_integration_struct_field_access() {
        let source = r#"
func swap(p: Point) -> Point {
    let temp: Float = p.x;
    p.x = p.y;
    p.y = temp;
    return p;
}
"#;
        let mut parser = parser_from_source(source);
        let result = parser.parse_function();

        assert!(result.is_ok());
        let func = result.unwrap();
        assert_eq!(func.body.statements.len(), 4);
    }

    #[test]
    fn test_integration_ownership_types() {
        let source = r#"
func take_owned(data: ^Data) -> Int {
    return 0;
}
"#;
        let mut parser = parser_from_source(source);
        let result = parser.parse_function();

        assert!(result.is_ok());
        let func = result.unwrap();
        assert!(matches!(
            *func.parameters[0].param_type,
            TypeSpecifier::Owned { .. }
        ));
    }

    #[test]
    fn test_integration_borrowed_types() {
        let source = r#"
func read_only(data: &Data) -> Int {
    return 0;
}
"#;
        let mut parser = parser_from_source(source);
        let result = parser.parse_function();

        assert!(result.is_ok());
        let func = result.unwrap();
        assert!(matches!(
            *func.parameters[0].param_type,
            TypeSpecifier::Owned {
                ownership: OwnershipKind::Borrowed,
                ..
            }
        ));
    }

    #[test]
    fn test_integration_external_function() {
        let source = r#"@extern(library: "libc") func puts(s: Pointer<Void>) -> Int;"#;
        let mut parser = parser_from_source(source);

        let annotation = parser.parse_annotation().unwrap();
        let result = parser.parse_external_function(annotation);

        assert!(result.is_ok());
        let ext_func = result.unwrap();
        assert_eq!(ext_func.name.name, "puts");
        assert_eq!(ext_func.library, "libc");
    }

    // ==================== ERROR RECOVERY TESTS ====================

    #[test]
    fn test_error_recovery_unexpected_token() {
        // Simple error: unexpected token at module level
        let source = r#"
module test;

123;

func good() -> Int {
    return 0;
}
"#;
        let mut parser = parser_from_source(source);
        let (result, errors) = parser.parse_module_with_recovery();

        // Should have parsed something
        assert!(result.is_some());
        let module = result.unwrap();

        // Should have at least one error
        assert!(!errors.is_empty(), "Should have collected errors");

        // The good function should still be parsed
        assert!(module.function_definitions.len() >= 1);
        assert!(module
            .function_definitions
            .iter()
            .any(|f| f.name.name == "good"));
    }

    #[test]
    fn test_error_recovery_with_valid_first_function() {
        // First function valid, second has extra tokens
        let source = r#"
module test;

func first() -> Int {
    return 1;
}

struct;

func third() -> Int {
    return 3;
}
"#;
        let mut parser = parser_from_source(source);
        let (result, errors) = parser.parse_module_with_recovery();

        assert!(result.is_some());
        // Should have errors from the incomplete struct
        assert!(!errors.is_empty());

        let module = result.unwrap();
        let func_names: Vec<_> = module
            .function_definitions
            .iter()
            .map(|f| f.name.name.as_str())
            .collect();

        // First function should definitely be parsed
        assert!(func_names.contains(&"first"), "Should have parsed 'first'");
    }

    #[test]
    fn test_synchronize_simple() {
        // Test synchronization with simple invalid token
        let source = r#"
module test;

999;

func valid() -> Int {
    return 42;
}
"#;
        let mut parser = parser_from_source(source);
        let (result, errors) = parser.parse_module_with_recovery();

        assert!(result.is_some());
        assert!(!errors.is_empty());

        // The valid function should be parsed after recovery
        let module = result.unwrap();
        assert!(module
            .function_definitions
            .iter()
            .any(|f| f.name.name == "valid"));
    }

    #[test]
    fn test_error_recovery_inline_module() {
        // Test recovery in inline module syntax
        let source = r#"
module test {
    func good() -> Int {
        return 1;
    }

    123;

    func also_good() -> Int {
        return 2;
    }
}
"#;
        let mut parser = parser_from_source(source);
        let (result, errors) = parser.parse_module_with_recovery();

        assert!(result.is_some());
        // Should have errors from the invalid token
        assert!(!errors.is_empty());

        let module = result.unwrap();
        // Should have parsed the good functions
        assert!(module.function_definitions.len() >= 1);
    }

    #[test]
    fn test_synchronization_methods() {
        // Test that synchronization skips to the next declaration
        let source = r#"
module test;

456;

func next_item() -> Int {
    return 0;
}
"#;
        let mut parser = parser_from_source(source);

        // Skip the module declaration
        parser.expect_keyword(Keyword::Module, "").unwrap();
        parser.parse_identifier().unwrap();
        parser.expect(&TokenType::Semicolon, "").unwrap();

        // Synchronize should skip the invalid token and stop at 'func'
        parser.synchronize_to_module_item();

        // Should now be at 'func' keyword
        assert!(parser.check_keyword(Keyword::Func));
    }

    #[test]
    fn test_has_errors_and_take_errors() {
        let source = "module test;";
        let mut parser = parser_from_source(source);

        assert!(!parser.has_errors());

        parser.add_error(ParserError::UnexpectedEof {
            expected: "test".to_string(),
        });

        assert!(parser.has_errors());
        assert_eq!(parser.errors().len(), 1);

        let taken = parser.take_errors();
        assert_eq!(taken.len(), 1);
        assert!(!parser.has_errors());
    }

    // ==================== MATCH EXPRESSION TESTS ====================

    #[test]
    fn test_match_simple() {
        let source = r#"match x { 1 => 10, 2 => 20, _ => 0 }"#;
        let mut parser = parser_from_source(source);
        let result = parser.parse_match_expression();

        assert!(result.is_ok());
        let expr = result.unwrap();
        if let Expression::Match { cases, .. } = expr {
            assert_eq!(cases.len(), 3);
        } else {
            panic!("Expected Match expression");
        }
    }

    #[test]
    fn test_match_with_variable_binding() {
        let source = r#"match value { x => x }"#;
        let mut parser = parser_from_source(source);
        let result = parser.parse_match_expression();

        assert!(result.is_ok());
        let expr = result.unwrap();
        if let Expression::Match { cases, .. } = expr {
            assert_eq!(cases.len(), 1);
            // x is a variable binding pattern
            assert!(matches!(cases[0].pattern, Pattern::Wildcard { .. }));
        } else {
            panic!("Expected Match expression");
        }
    }

    #[test]
    fn test_match_with_enum_variant() {
        let source = r#"match opt { Some(x) => x, None => 0 }"#;
        let mut parser = parser_from_source(source);
        let result = parser.parse_match_expression();

        assert!(result.is_ok());
        let expr = result.unwrap();
        if let Expression::Match { cases, .. } = expr {
            assert_eq!(cases.len(), 2);
            // First case: Some(x)
            if let Pattern::EnumVariant {
                variant_name,
                bindings,
                ..
            } = &cases[0].pattern
            {
                assert_eq!(variant_name.name, "Some");
                assert_eq!(bindings.len(), 1);
                assert_eq!(bindings[0].name, "x");
            } else {
                panic!("Expected EnumVariant pattern");
            }
        } else {
            panic!("Expected Match expression");
        }
    }

    #[test]
    fn test_match_with_literal_patterns() {
        let source = r#"match s { "hello" => 1, "world" => 2, _ => 0 }"#;
        let mut parser = parser_from_source(source);
        let result = parser.parse_match_expression();

        assert!(result.is_ok());
        let expr = result.unwrap();
        if let Expression::Match { cases, .. } = expr {
            assert_eq!(cases.len(), 3);
            // First two should be literal patterns
            assert!(matches!(cases[0].pattern, Pattern::Literal { .. }));
            assert!(matches!(cases[1].pattern, Pattern::Literal { .. }));
            // Last should be wildcard
            assert!(matches!(
                cases[2].pattern,
                Pattern::Wildcard { binding: None, .. }
            ));
        } else {
            panic!("Expected Match expression");
        }
    }

    #[test]
    fn test_match_with_bool_patterns() {
        let source = r#"match flag { true => 1, false => 0 }"#;
        let mut parser = parser_from_source(source);
        let result = parser.parse_match_expression();

        assert!(result.is_ok());
        let expr = result.unwrap();
        if let Expression::Match { cases, .. } = expr {
            assert_eq!(cases.len(), 2);
            assert!(matches!(cases[0].pattern, Pattern::Literal { .. }));
            assert!(matches!(cases[1].pattern, Pattern::Literal { .. }));
        } else {
            panic!("Expected Match expression");
        }
    }

    #[test]
    fn test_fat_arrow_token() {
        let source = "=>";
        let mut lexer = crate::lexer::v2::Lexer::new(source, "test".to_string());
        let tokens = lexer.tokenize().unwrap();
        assert!(matches!(tokens[0].token_type, TokenType::FatArrow));
    }

    #[test]
    fn test_underscore_token() {
        let source = "_";
        let mut lexer = crate::lexer::v2::Lexer::new(source, "test".to_string());
        let tokens = lexer.tokenize().unwrap();
        assert!(matches!(tokens[0].token_type, TokenType::Underscore));
    }

    #[test]
    fn test_underscore_in_identifier() {
        let source = "_foo";
        let mut lexer = crate::lexer::v2::Lexer::new(source, "test".to_string());
        let tokens = lexer.tokenize().unwrap();
        // _foo should be an identifier, not underscore
        assert!(matches!(tokens[0].token_type, TokenType::Identifier(_)));
    }

    // ==================== For Loop Tests ====================

    #[test]
    fn test_for_loop_simple() {
        let source = "for x in items { }";
        let mut parser = parser_from_source(source);
        let result = parser.parse_for_loop();

        assert!(result.is_ok());
        let stmt = result.unwrap();
        if let Statement::ForEachLoop {
            element_binding,
            collection,
            ..
        } = stmt
        {
            assert_eq!(element_binding.name, "x");
            assert!(matches!(*collection, Expression::Variable { .. }));
        } else {
            panic!("Expected ForEachLoop statement");
        }
    }

    #[test]
    fn test_for_loop_with_type_annotation() {
        let source = "for x: Int in numbers { }";
        let mut parser = parser_from_source(source);
        let result = parser.parse_for_loop();

        assert!(result.is_ok());
        let stmt = result.unwrap();
        if let Statement::ForEachLoop {
            element_binding,
            element_type,
            ..
        } = stmt
        {
            assert_eq!(element_binding.name, "x");
            assert!(matches!(
                *element_type,
                TypeSpecifier::Primitive { .. } | TypeSpecifier::Named { .. }
            ));
        } else {
            panic!("Expected ForEachLoop statement");
        }
    }

    #[test]
    fn test_for_loop_with_body() {
        let source = "for item in list { let x = item; }";
        let mut parser = parser_from_source(source);
        let result = parser.parse_for_loop();

        assert!(result.is_ok());
        let stmt = result.unwrap();
        if let Statement::ForEachLoop { body, .. } = stmt {
            assert_eq!(body.statements.len(), 1);
        } else {
            panic!("Expected ForEachLoop statement");
        }
    }

    #[test]
    fn test_for_loop_in_statement() {
        let source = "for i in items { break; }";
        let mut parser = parser_from_source(source);
        let result = parser.parse_statement();

        assert!(result.is_ok(), "Error: {:?}", result.err());
        let stmt = result.unwrap();
        assert!(matches!(stmt, Statement::ForEachLoop { .. }));
    }

    #[test]
    fn test_for_loop_with_function_call_collection() {
        let source = "for x in get_items() { }";
        let mut parser = parser_from_source(source);
        let result = parser.parse_for_loop();

        assert!(result.is_ok());
        let stmt = result.unwrap();
        if let Statement::ForEachLoop { collection, .. } = stmt {
            assert!(matches!(*collection, Expression::FunctionCall { .. }));
        } else {
            panic!("Expected ForEachLoop statement");
        }
    }

    #[test]
    fn test_for_loop_nested() {
        let source = "for i in outer { for j in inner { } }";
        let mut parser = parser_from_source(source);
        let result = parser.parse_for_loop();

        assert!(result.is_ok());
        let stmt = result.unwrap();
        if let Statement::ForEachLoop { body, .. } = stmt {
            assert_eq!(body.statements.len(), 1);
            assert!(matches!(body.statements[0], Statement::ForEachLoop { .. }));
        } else {
            panic!("Expected ForEachLoop statement");
        }
    }

    // ==================== Lambda Tests ====================

    #[test]
    fn test_lambda_zero_params() {
        let source = "() => 42";
        let mut parser = parser_from_source(source);
        let result = parser.parse_expression();

        assert!(result.is_ok(), "Error: {:?}", result.err());
        let expr = result.unwrap();
        if let Expression::Lambda {
            parameters, body, ..
        } = expr
        {
            assert_eq!(parameters.len(), 0);
            assert!(matches!(body, LambdaBody::Expression(_)));
        } else {
            panic!("Expected Lambda expression, got {:?}", expr);
        }
    }

    #[test]
    fn test_lambda_single_param_typed() {
        let source = "(x: Int) => x";
        let mut parser = parser_from_source(source);
        let result = parser.parse_expression();

        assert!(result.is_ok(), "Error: {:?}", result.err());
        let expr = result.unwrap();
        if let Expression::Lambda {
            parameters, body, ..
        } = expr
        {
            assert_eq!(parameters.len(), 1);
            assert_eq!(parameters[0].name.name, "x");
            assert!(matches!(body, LambdaBody::Expression(_)));
        } else {
            panic!("Expected Lambda expression, got {:?}", expr);
        }
    }

    #[test]
    fn test_lambda_multiple_params() {
        // Expression body with a simple identifier
        let source = "(x: Int, y: Int) => x";
        let mut parser = parser_from_source(source);
        let result = parser.parse_expression();

        assert!(result.is_ok(), "Error: {:?}", result.err());
        let expr = result.unwrap();
        if let Expression::Lambda {
            parameters, body, ..
        } = expr
        {
            assert_eq!(parameters.len(), 2);
            assert_eq!(parameters[0].name.name, "x");
            assert_eq!(parameters[1].name.name, "y");
            assert!(matches!(body, LambdaBody::Expression(_)));
        } else {
            panic!("Expected Lambda expression, got {:?}", expr);
        }
    }

    #[test]
    fn test_lambda_with_block_body() {
        let source = "(x: Int) => { let y = x; return y; }";
        let mut parser = parser_from_source(source);
        let result = parser.parse_expression();

        assert!(result.is_ok(), "Error: {:?}", result.err());
        let expr = result.unwrap();
        if let Expression::Lambda { body, .. } = expr {
            assert!(matches!(body, LambdaBody::Block(_)));
            if let LambdaBody::Block(block) = body {
                assert_eq!(block.statements.len(), 2);
            }
        } else {
            panic!("Expected Lambda expression, got {:?}", expr);
        }
    }

    #[test]
    fn test_lambda_with_return_type() {
        let source = "(x: Int) -> Int => x";
        let mut parser = parser_from_source(source);
        let result = parser.parse_expression();

        assert!(result.is_ok(), "Error: {:?}", result.err());
        let expr = result.unwrap();
        if let Expression::Lambda { return_type, .. } = expr {
            assert!(return_type.is_some());
        } else {
            panic!("Expected Lambda expression, got {:?}", expr);
        }
    }

    #[test]
    fn test_parenthesized_expression() {
        // (42) should be parsed as a parenthesized expression, not a lambda
        let source = "(42)";
        let mut parser = parser_from_source(source);
        let result = parser.parse_expression();

        assert!(result.is_ok(), "Error: {:?}", result.err());
        let expr = result.unwrap();
        assert!(
            matches!(expr, Expression::IntegerLiteral { value: 42, .. }),
            "Expected IntegerLiteral, got {:?}",
            expr
        );
    }

    #[test]
    fn test_parenthesized_binary_expression() {
        let source = "({a + b})";
        let mut parser = parser_from_source(source);
        let result = parser.parse_expression();

        assert!(result.is_ok(), "Error: {:?}", result.err());
        // The inner expression should be an Add
        let expr = result.unwrap();
        assert!(matches!(expr, Expression::Add { .. }));
    }

    // ==================== Method Call Tests ====================

    #[test]
    fn test_method_call_no_args() {
        let source = "obj.method()";
        let mut parser = parser_from_source(source);
        let result = parser.parse_expression();

        assert!(result.is_ok(), "Error: {:?}", result.err());
        let expr = result.unwrap();
        if let Expression::MethodCall {
            receiver,
            method_name,
            arguments,
            ..
        } = expr
        {
            assert!(matches!(*receiver, Expression::Variable { .. }));
            assert_eq!(method_name.name, "method");
            assert_eq!(arguments.len(), 0);
        } else {
            panic!("Expected MethodCall expression, got {:?}", expr);
        }
    }

    #[test]
    fn test_method_call_with_args() {
        let source = "list.push(42)";
        let mut parser = parser_from_source(source);
        let result = parser.parse_expression();

        assert!(result.is_ok(), "Error: {:?}", result.err());
        let expr = result.unwrap();
        if let Expression::MethodCall {
            receiver,
            method_name,
            arguments,
            ..
        } = expr
        {
            assert!(matches!(*receiver, Expression::Variable { .. }));
            assert_eq!(method_name.name, "push");
            assert_eq!(arguments.len(), 1);
        } else {
            panic!("Expected MethodCall expression, got {:?}", expr);
        }
    }

    #[test]
    fn test_method_call_multiple_args() {
        let source = "map.insert(key, value)";
        let mut parser = parser_from_source(source);
        let result = parser.parse_expression();

        assert!(result.is_ok(), "Error: {:?}", result.err());
        let expr = result.unwrap();
        if let Expression::MethodCall {
            method_name,
            arguments,
            ..
        } = expr
        {
            assert_eq!(method_name.name, "insert");
            assert_eq!(arguments.len(), 2);
        } else {
            panic!("Expected MethodCall expression, got {:?}", expr);
        }
    }

    #[test]
    fn test_method_call_chained() {
        let source = "obj.first().second()";
        let mut parser = parser_from_source(source);
        let result = parser.parse_expression();

        assert!(result.is_ok(), "Error: {:?}", result.err());
        let expr = result.unwrap();
        // Outer call should be second()
        if let Expression::MethodCall {
            receiver,
            method_name,
            ..
        } = expr
        {
            assert_eq!(method_name.name, "second");
            // Inner call should be first()
            if let Expression::MethodCall {
                method_name: inner_method,
                ..
            } = *receiver
            {
                assert_eq!(inner_method.name, "first");
            } else {
                panic!("Expected inner MethodCall");
            }
        } else {
            panic!("Expected MethodCall expression, got {:?}", expr);
        }
    }

    #[test]
    fn test_field_access_still_works() {
        let source = "obj.field";
        let mut parser = parser_from_source(source);
        let result = parser.parse_expression();

        assert!(result.is_ok(), "Error: {:?}", result.err());
        let expr = result.unwrap();
        assert!(
            matches!(expr, Expression::FieldAccess { .. }),
            "Expected FieldAccess, got {:?}",
            expr
        );
    }

    #[test]
    fn test_method_call_on_field() {
        let source = "obj.field.method()";
        let mut parser = parser_from_source(source);
        let result = parser.parse_expression();

        assert!(result.is_ok(), "Error: {:?}", result.err());
        let expr = result.unwrap();
        if let Expression::MethodCall {
            receiver,
            method_name,
            ..
        } = expr
        {
            assert_eq!(method_name.name, "method");
            // Receiver should be a field access
            assert!(
                matches!(*receiver, Expression::FieldAccess { .. }),
                "Expected FieldAccess receiver"
            );
        } else {
            panic!("Expected MethodCall expression, got {:?}", expr);
        }
    }

    // ==================== Range Expression Tests ====================

    #[test]
    fn test_range_exclusive() {
        let source = "0..10";
        let mut parser = parser_from_source(source);
        let result = parser.parse_expression();

        assert!(result.is_ok(), "Error: {:?}", result.err());
        let expr = result.unwrap();
        if let Expression::Range {
            start,
            end,
            inclusive,
            ..
        } = expr
        {
            assert!(start.is_some());
            assert!(end.is_some());
            assert!(!inclusive);
        } else {
            panic!("Expected Range expression, got {:?}", expr);
        }
    }

    #[test]
    fn test_range_inclusive() {
        let source = "0..=10";
        let mut parser = parser_from_source(source);
        let result = parser.parse_expression();

        assert!(result.is_ok(), "Error: {:?}", result.err());
        let expr = result.unwrap();
        if let Expression::Range {
            start,
            end,
            inclusive,
            ..
        } = expr
        {
            assert!(start.is_some());
            assert!(end.is_some());
            assert!(inclusive);
        } else {
            panic!("Expected Range expression, got {:?}", expr);
        }
    }

    #[test]
    fn test_range_prefix() {
        let source = "..10";
        let mut parser = parser_from_source(source);
        let result = parser.parse_expression();

        assert!(result.is_ok(), "Error: {:?}", result.err());
        let expr = result.unwrap();
        if let Expression::Range { start, end, .. } = expr {
            assert!(start.is_none());
            assert!(end.is_some());
        } else {
            panic!("Expected Range expression, got {:?}", expr);
        }
    }

    #[test]
    fn test_range_postfix() {
        let source = "0..";
        let mut parser = parser_from_source(source);
        let result = parser.parse_expression();

        assert!(result.is_ok(), "Error: {:?}", result.err());
        let expr = result.unwrap();
        if let Expression::Range { start, end, .. } = expr {
            assert!(start.is_some());
            assert!(end.is_none());
        } else {
            panic!("Expected Range expression, got {:?}", expr);
        }
    }

    #[test]
    fn test_dotdot_token() {
        let source = "..";
        let mut lexer = crate::lexer::v2::Lexer::new(source, "test".to_string());
        let tokens = lexer.tokenize().unwrap();
        assert!(matches!(tokens[0].token_type, TokenType::DotDot));
    }

    #[test]
    fn test_dotdotequal_token() {
        let source = "..=";
        let mut lexer = crate::lexer::v2::Lexer::new(source, "test".to_string());
        let tokens = lexer.tokenize().unwrap();
        assert!(matches!(tokens[0].token_type, TokenType::DotDotEqual));
    }

    // ========== Edge Case Tests ==========

    // Range edge cases
    #[test]
    fn test_range_prefix_in_expression() {
        // Prefix range: ..end directly parsed
        let source = "..10";
        let mut parser = parser_from_source(source);
        let result = parser.parse_expression();
        assert!(result.is_ok());
        let expr = result.unwrap();
        if let Expression::Range {
            start,
            end,
            inclusive,
            ..
        } = expr
        {
            assert!(start.is_none());
            assert!(end.is_some());
            assert!(!inclusive);
        } else {
            panic!("Expected Range expression");
        }
    }

    #[test]
    fn test_range_prefix_inclusive() {
        // Prefix inclusive range: ..=end
        let source = "..=5";
        let mut parser = parser_from_source(source);
        let result = parser.parse_expression();
        assert!(result.is_ok());
        let expr = result.unwrap();
        if let Expression::Range {
            start, inclusive, ..
        } = expr
        {
            assert!(start.is_none());
            assert!(inclusive);
        } else {
            panic!("Expected Range expression");
        }
    }

    #[test]
    fn test_for_loop_with_range() {
        // For loop iterating over a range - now produces FixedIterationLoop
        let source = "for i in 0..10 { x = i; }";
        let mut parser = parser_from_source(source);
        let result = parser.parse_for_loop();
        assert!(result.is_ok());
        let stmt = result.unwrap();
        if let Statement::FixedIterationLoop { counter, inclusive, .. } = stmt {
            assert_eq!(counter.name, "i");
            assert!(!inclusive); // exclusive range
        } else {
            panic!("Expected FixedIterationLoop statement");
        }
    }

    #[test]
    fn test_for_loop_with_inclusive_range() {
        // For loop with inclusive range - now produces FixedIterationLoop
        let source = "for i in 1..=5 { x = i; }";
        let mut parser = parser_from_source(source);
        let result = parser.parse_for_loop();
        assert!(result.is_ok());
        let stmt = result.unwrap();
        if let Statement::FixedIterationLoop { counter, inclusive, .. } = stmt {
            assert_eq!(counter.name, "i");
            assert!(inclusive); // inclusive range
        } else {
            panic!("Expected FixedIterationLoop statement");
        }
    }

    // Lambda edge cases
    #[test]
    fn test_lambda_typed_param_direct() {
        // Lambda with typed parameter, parsed directly
        let source = "(x: Int) => x";
        let mut parser = parser_from_source(source);
        let result = parser.parse_expression();
        assert!(result.is_ok());
        let expr = result.unwrap();
        if let Expression::Lambda { parameters, .. } = expr {
            assert_eq!(parameters.len(), 1);
            assert_eq!(parameters[0].name.name, "x");
        } else {
            panic!("Expected Lambda expression");
        }
    }

    #[test]
    fn test_lambda_untyped_param_direct() {
        // Lambda with untyped parameter, parsed directly
        let source = "(x) => x";
        let mut parser = parser_from_source(source);
        let result = parser.parse_expression();
        assert!(result.is_ok());
        let expr = result.unwrap();
        if let Expression::Lambda { parameters, .. } = expr {
            assert_eq!(parameters.len(), 1);
            assert_eq!(parameters[0].name.name, "x");
        } else {
            panic!("Expected Lambda expression");
        }
    }

    #[test]
    fn test_lambda_with_block_body_direct() {
        // Lambda with block body, parsed directly
        let source = "(x: Int) => { return {x + 1}; }";
        let mut parser = parser_from_source(source);
        let result = parser.parse_expression();
        assert!(result.is_ok());
        let expr = result.unwrap();
        if let Expression::Lambda { body, .. } = expr {
            assert!(matches!(body, LambdaBody::Block(_)));
        } else {
            panic!("Expected Lambda expression");
        }
    }

    // Method call edge cases
    #[test]
    fn test_method_call_in_braced_expression() {
        // Method call inside a braced binary expression
        let source = "{a.len() + b.len()}";
        let mut parser = parser_from_source(source);
        let result = parser.parse_expression();
        assert!(result.is_ok());
        let expr = result.unwrap();
        if let Expression::Add { left, right, .. } = expr {
            assert!(matches!(*left, Expression::MethodCall { .. }));
            assert!(matches!(*right, Expression::MethodCall { .. }));
        } else {
            panic!("Expected Add expression");
        }
    }

    #[test]
    fn test_method_call_deeply_chained() {
        // Deeply chained method calls - parsed directly
        let source = "a.first().second().third().fourth()";
        let mut parser = parser_from_source(source);
        let result = parser.parse_expression();
        assert!(result.is_ok());
        let expr = result.unwrap();
        // Should be MethodCall for fourth()
        if let Expression::MethodCall {
            method_name,
            receiver,
            ..
        } = expr
        {
            assert_eq!(method_name.name, "fourth");
            // Receiver should be MethodCall for third()
            assert!(matches!(*receiver, Expression::MethodCall { .. }));
        } else {
            panic!("Expected MethodCall expression");
        }
    }

    #[test]
    fn test_method_call_with_expression_arg() {
        // Method call with braced expression as argument
        let source = "list.get({i + 1})";
        let mut parser = parser_from_source(source);
        let result = parser.parse_expression();
        assert!(result.is_ok());
        let expr = result.unwrap();
        if let Expression::MethodCall {
            method_name,
            arguments,
            ..
        } = expr
        {
            assert_eq!(method_name.name, "get");
            assert_eq!(arguments.len(), 1);
        } else {
            panic!("Expected MethodCall expression");
        }
    }

    // Match expression edge cases
    #[test]
    fn test_match_with_bool_result() {
        // Match expression with bool patterns - parsed directly
        let source = "match x { 1 => true, _ => false }";
        let mut parser = parser_from_source(source);
        let result = parser.parse_match_expression();
        assert!(result.is_ok());
        let expr = result.unwrap();
        if let Expression::Match { cases, .. } = expr {
            assert_eq!(cases.len(), 2);
        } else {
            panic!("Expected Match expression");
        }
    }

    #[test]
    fn test_match_with_multiple_enum_variants() {
        // Match with multiple enum variant patterns - parsed directly
        let source = "match opt { Some(x) => x, None => 0 }";
        let mut parser = parser_from_source(source);
        let result = parser.parse_match_expression();
        assert!(result.is_ok());
        let expr = result.unwrap();
        if let Expression::Match { cases, .. } = expr {
            assert_eq!(cases.len(), 2);
            // First case: Some(x)
            assert!(matches!(cases[0].pattern, Pattern::EnumVariant { .. }));
        } else {
            panic!("Expected Match expression");
        }
    }

    // Combined feature tests
    #[test]
    fn test_combined_for_range() {
        // For loop with integer range - produces FixedIterationLoop
        let source = "for i in 0..10 { x = i; }";
        let mut parser = parser_from_source(source);
        let result = parser.parse_for_loop();
        assert!(result.is_ok());
        let stmt = result.unwrap();
        if let Statement::FixedIterationLoop { counter, inclusive, .. } = stmt {
            assert_eq!(counter.name, "i");
            assert!(!inclusive);
        } else {
            panic!("Expected FixedIterationLoop statement");
        }
    }

    #[test]
    fn test_combined_method_with_lambda() {
        // Method call with lambda as argument - parsed directly
        // Note: uses braced expression for lambda body to work with parser
        let source = "list.map((x) => x)";
        let mut parser = parser_from_source(source);
        let result = parser.parse_expression();
        assert!(result.is_ok());
        let expr = result.unwrap();
        if let Expression::MethodCall {
            method_name,
            arguments,
            ..
        } = expr
        {
            assert_eq!(method_name.name, "map");
            assert_eq!(arguments.len(), 1);
            // The argument should be a lambda
            assert!(matches!(&*arguments[0].value, Expression::Lambda { .. }));
        } else {
            panic!("Expected MethodCall expression");
        }
    }

    #[test]
    fn test_combined_match_multiple_cases() {
        // Match with wildcard and enum patterns
        let source = "match status { Ok(v) => v, Err(e) => 0, _ => default }";
        let mut parser = parser_from_source(source);
        let result = parser.parse_match_expression();
        assert!(result.is_ok());
        let expr = result.unwrap();
        if let Expression::Match { cases, .. } = expr {
            assert_eq!(cases.len(), 3);
        } else {
            panic!("Expected Match expression");
        }
    }

    #[test]
    fn test_combined_nested_for_range() {
        // For loop with integer range - verify statement structure
        let source = "for i in 1..=100 { count = {count + 1}; }";
        let mut parser = parser_from_source(source);
        let result = parser.parse_for_loop();
        assert!(result.is_ok());
        let stmt = result.unwrap();
        if let Statement::FixedIterationLoop {
            counter,
            body,
            inclusive,
            ..
        } = stmt
        {
            assert_eq!(counter.name, "i");
            assert!(inclusive); // inclusive range
            assert!(!body.statements.is_empty());
        } else {
            panic!("Expected FixedIterationLoop statement");
        }
    }

    #[test]
    fn test_return_lambda_direct() {
        // Return statement with lambda value - parsed via return parsing
        let source = "return (x) => x;";
        let mut parser = parser_from_source(source);
        let result = parser.parse_return_statement();
        assert!(result.is_ok());
        let stmt = result.unwrap();
        if let Statement::Return {
            value: Some(expr), ..
        } = stmt
        {
            assert!(matches!(*expr, Expression::Lambda { .. }));
        } else {
            panic!("Expected Return statement with Lambda");
        }
    }

    #[test]
    fn test_method_call_result() {
        // Method call on expression
        let source = "callbacks.get(0)";
        let mut parser = parser_from_source(source);
        let result = parser.parse_expression();
        assert!(result.is_ok());
        let expr = result.unwrap();
        assert!(matches!(expr, Expression::MethodCall { .. }));
    }

    #[test]
    fn test_complex_expression_chain() {
        // Complex chain: field access + method call + array access
        let source = "obj.field.method()[0]";
        let mut parser = parser_from_source(source);
        let result = parser.parse_expression();
        assert!(result.is_ok());
        let expr = result.unwrap();
        // Outermost should be array access
        assert!(matches!(expr, Expression::ArrayAccess { .. }));
    }

    // ========== Closure Capture Tests ==========

    #[test]
    fn test_closure_single_capture_by_value() {
        // Closure with single capture by value
        let source = "[x](y) => y";
        let mut parser = parser_from_source(source);
        let result = parser.parse_expression();
        assert!(result.is_ok(), "Error: {:?}", result.err());
        let expr = result.unwrap();
        if let Expression::Lambda {
            captures,
            parameters,
            ..
        } = expr
        {
            assert_eq!(captures.len(), 1);
            assert_eq!(captures[0].name.name, "x");
            assert!(matches!(captures[0].mode, CaptureMode::ByValue));
            assert_eq!(parameters.len(), 1);
        } else {
            panic!("Expected Lambda expression");
        }
    }

    #[test]
    fn test_closure_single_capture_by_reference() {
        // Closure with single capture by reference
        let source = "[&x](y) => y";
        let mut parser = parser_from_source(source);
        let result = parser.parse_expression();
        assert!(result.is_ok(), "Error: {:?}", result.err());
        let expr = result.unwrap();
        if let Expression::Lambda { captures, .. } = expr {
            assert_eq!(captures.len(), 1);
            assert_eq!(captures[0].name.name, "x");
            assert!(matches!(captures[0].mode, CaptureMode::ByReference));
        } else {
            panic!("Expected Lambda expression");
        }
    }

    #[test]
    fn test_closure_capture_by_mut_reference() {
        // Closure with capture by mutable reference
        let source = "[&mut counter]() => counter";
        let mut parser = parser_from_source(source);
        let result = parser.parse_expression();
        assert!(result.is_ok(), "Error: {:?}", result.err());
        let expr = result.unwrap();
        if let Expression::Lambda {
            captures,
            parameters,
            ..
        } = expr
        {
            assert_eq!(captures.len(), 1);
            assert_eq!(captures[0].name.name, "counter");
            assert!(matches!(captures[0].mode, CaptureMode::ByMutableReference));
            assert_eq!(parameters.len(), 0);
        } else {
            panic!("Expected Lambda expression");
        }
    }

    #[test]
    fn test_closure_multiple_captures() {
        // Closure with multiple captures of different modes
        let source = "[x, &y, &mut z](a: Int) => a";
        let mut parser = parser_from_source(source);
        let result = parser.parse_expression();
        assert!(result.is_ok(), "Error: {:?}", result.err());
        let expr = result.unwrap();
        if let Expression::Lambda {
            captures,
            parameters,
            ..
        } = expr
        {
            assert_eq!(captures.len(), 3);
            assert_eq!(captures[0].name.name, "x");
            assert!(matches!(captures[0].mode, CaptureMode::ByValue));
            assert_eq!(captures[1].name.name, "y");
            assert!(matches!(captures[1].mode, CaptureMode::ByReference));
            assert_eq!(captures[2].name.name, "z");
            assert!(matches!(captures[2].mode, CaptureMode::ByMutableReference));
            assert_eq!(parameters.len(), 1);
        } else {
            panic!("Expected Lambda expression");
        }
    }

    #[test]
    fn test_closure_empty_captures() {
        // Closure with empty capture list (explicit no captures)
        let source = "[](x) => x";
        let mut parser = parser_from_source(source);
        let result = parser.parse_expression();
        assert!(result.is_ok(), "Error: {:?}", result.err());
        let expr = result.unwrap();
        if let Expression::Lambda {
            captures,
            parameters,
            ..
        } = expr
        {
            assert_eq!(captures.len(), 0);
            assert_eq!(parameters.len(), 1);
        } else {
            panic!("Expected Lambda expression");
        }
    }

    #[test]
    fn test_closure_with_block_body() {
        // Closure with captures and block body
        let source = "[total](x: Int) => { return {total + x}; }";
        let mut parser = parser_from_source(source);
        let result = parser.parse_expression();
        assert!(result.is_ok(), "Error: {:?}", result.err());
        let expr = result.unwrap();
        if let Expression::Lambda { captures, body, .. } = expr {
            assert_eq!(captures.len(), 1);
            assert!(matches!(body, LambdaBody::Block(_)));
        } else {
            panic!("Expected Lambda expression");
        }
    }

    #[test]
    fn test_closure_with_return_type() {
        // Closure with captures and explicit return type
        let source = "[state](x: Int) -> Int => x";
        let mut parser = parser_from_source(source);
        let result = parser.parse_expression();
        assert!(result.is_ok(), "Error: {:?}", result.err());
        let expr = result.unwrap();
        if let Expression::Lambda {
            captures,
            return_type,
            ..
        } = expr
        {
            assert_eq!(captures.len(), 1);
            assert!(return_type.is_some());
        } else {
            panic!("Expected Lambda expression");
        }
    }

    #[test]
    fn test_lambda_without_captures_still_works() {
        // Verify lambdas without captures still work (backward compatibility)
        let source = "(x: Int) => x";
        let mut parser = parser_from_source(source);
        let result = parser.parse_expression();
        assert!(result.is_ok());
        let expr = result.unwrap();
        if let Expression::Lambda { captures, .. } = expr {
            assert_eq!(captures.len(), 0);
        } else {
            panic!("Expected Lambda expression");
        }
    }
}
