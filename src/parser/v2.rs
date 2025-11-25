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
    Block, CallingConvention, Expression, ExternalFunction, Function, FunctionMetadata, Identifier,
    ImportStatement, Module, Mutability, OwnershipKind, Parameter, PassingMode, PrimitiveType,
    Statement, TypeSpecifier,
};
use crate::error::{ParserError, SourceLocation};
use crate::lexer::v2::{Keyword, Token, TokenType};

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
                Keyword::String_ => {
                    self.advance();
                    return Ok(TypeSpecifier::Primitive {
                        type_name: PrimitiveType::String,
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
                    self.expect(&TokenType::Comma, "expected ',' between Map key and value types")?;
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

    /// Parse a block (function body)
    /// Grammar: "{" statement* "}"
    pub fn parse_block(&mut self) -> Result<Block, ParserError> {
        let start_location = self.current_location();

        // Expect opening brace
        self.expect(&TokenType::LeftBrace, "expected '{' to start block")?;

        // For now, just parse empty blocks or skip to closing brace
        // Statement parsing will be added in later tasks
        let statements = Vec::new();

        // Skip any tokens until we find the closing brace
        // This is temporary - we'll add proper statement parsing later
        let mut brace_depth = 1;
        while !self.is_at_end() && brace_depth > 0 {
            if self.check(&TokenType::LeftBrace) {
                brace_depth += 1;
                self.advance();
            } else if self.check(&TokenType::RightBrace) {
                brace_depth -= 1;
                if brace_depth > 0 {
                    self.advance();
                }
            } else {
                self.advance();
            }
        }

        // Expect closing brace
        self.expect(&TokenType::RightBrace, "expected '}' to close block")?;

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

            self.expect(&TokenType::RightParen, "expected ')' after annotation arguments")?;
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
            return Ok(AnnotationValue::Expression(expr_tokens.trim().to_string(), start));
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
    pub fn parse_external_function(&mut self, annotation: Annotation) -> Result<ExternalFunction, ParserError> {
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
        self.expect(&TokenType::Arrow, "expected '->' for extern function return type")?;
        let return_type = self.parse_type()?;

        // Expect semicolon (no body for extern functions)
        self.expect(&TokenType::Semicolon, "expected ';' after extern function declaration")?;

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

    // ==================== VARIABLE DECLARATION PARSING ====================

    /// Parse a variable declaration
    /// Grammar: "let" ["mut"] IDENTIFIER [":" type] ["=" expression] ";"
    pub fn parse_variable_declaration(&mut self) -> Result<Statement, ParserError> {
        let start_location = self.current_location();

        // Expect 'let' keyword
        self.expect_keyword(Keyword::Let, "expected 'let'")?;

        // Check for 'mut' keyword
        let mutability = if self.check_keyword(Keyword::Mut) {
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
        self.expect(&TokenType::Semicolon, "expected ';' after variable declaration")?;

        Ok(Statement::VariableDeclaration {
            name,
            type_spec: Box::new(type_spec),
            mutability,
            initial_value,
            intent: None,
            source_location: start_location,
        })
    }

    // ==================== EXPRESSION PARSING ====================

    /// Parse an expression (literals and identifiers for now)
    /// Binary expressions with braces will be added in Task 2.9+
    pub fn parse_expression(&mut self) -> Result<Expression, ParserError> {
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

        // Identifier (variable reference)
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

        Err(ParserError::UnexpectedToken {
            expected: "expression".to_string(),
            found: format!("{:?}", self.peek().token_type),
            location: start_location,
        })
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

    // ==================== TYPE PARSING TESTS ====================

    #[test]
    fn test_parse_type_int() {
        let mut parser = parser_from_source("Int");
        let result = parser.parse_type();

        assert!(result.is_ok());
        let type_spec = result.unwrap();
        assert!(matches!(type_spec, TypeSpecifier::Primitive { type_name: PrimitiveType::Integer, .. }));
    }

    #[test]
    fn test_parse_type_int64() {
        let mut parser = parser_from_source("Int64");
        let result = parser.parse_type();

        assert!(result.is_ok());
        let type_spec = result.unwrap();
        assert!(matches!(type_spec, TypeSpecifier::Primitive { type_name: PrimitiveType::Integer64, .. }));
    }

    #[test]
    fn test_parse_type_float() {
        let mut parser = parser_from_source("Float");
        let result = parser.parse_type();

        assert!(result.is_ok());
        let type_spec = result.unwrap();
        assert!(matches!(type_spec, TypeSpecifier::Primitive { type_name: PrimitiveType::Float, .. }));
    }

    #[test]
    fn test_parse_type_string() {
        let mut parser = parser_from_source("String");
        let result = parser.parse_type();

        assert!(result.is_ok());
        let type_spec = result.unwrap();
        assert!(matches!(type_spec, TypeSpecifier::Primitive { type_name: PrimitiveType::String, .. }));
    }

    #[test]
    fn test_parse_type_bool() {
        let mut parser = parser_from_source("Bool");
        let result = parser.parse_type();

        assert!(result.is_ok());
        let type_spec = result.unwrap();
        assert!(matches!(type_spec, TypeSpecifier::Primitive { type_name: PrimitiveType::Boolean, .. }));
    }

    #[test]
    fn test_parse_type_void() {
        let mut parser = parser_from_source("Void");
        let result = parser.parse_type();

        assert!(result.is_ok());
        let type_spec = result.unwrap();
        assert!(matches!(type_spec, TypeSpecifier::Primitive { type_name: PrimitiveType::Void, .. }));
    }

    #[test]
    fn test_parse_type_sizet() {
        let mut parser = parser_from_source("SizeT");
        let result = parser.parse_type();

        assert!(result.is_ok());
        let type_spec = result.unwrap();
        assert!(matches!(type_spec, TypeSpecifier::Primitive { type_name: PrimitiveType::SizeT, .. }));
    }

    #[test]
    fn test_parse_type_array() {
        let mut parser = parser_from_source("Array<Int>");
        let result = parser.parse_type();

        assert!(result.is_ok());
        let type_spec = result.unwrap();
        if let TypeSpecifier::Array { element_type, .. } = type_spec {
            assert!(matches!(*element_type, TypeSpecifier::Primitive { type_name: PrimitiveType::Integer, .. }));
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
        if let TypeSpecifier::Map { key_type, value_type, .. } = type_spec {
            assert!(matches!(*key_type, TypeSpecifier::Primitive { type_name: PrimitiveType::String, .. }));
            assert!(matches!(*value_type, TypeSpecifier::Primitive { type_name: PrimitiveType::Integer, .. }));
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
        if let TypeSpecifier::Pointer { target_type, is_mutable, .. } = type_spec {
            assert!(!is_mutable);
            assert!(matches!(*target_type, TypeSpecifier::Primitive { type_name: PrimitiveType::Integer, .. }));
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
        if let TypeSpecifier::Pointer { target_type, is_mutable, .. } = type_spec {
            assert!(is_mutable);
            assert!(matches!(*target_type, TypeSpecifier::Primitive { type_name: PrimitiveType::Void, .. }));
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
        if let TypeSpecifier::Owned { ownership, base_type, .. } = type_spec {
            assert_eq!(ownership, OwnershipKind::Owned);
            assert!(matches!(*base_type, TypeSpecifier::Primitive { type_name: PrimitiveType::String, .. }));
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
        if let TypeSpecifier::Owned { ownership, base_type, .. } = type_spec {
            assert_eq!(ownership, OwnershipKind::Borrowed);
            assert!(matches!(*base_type, TypeSpecifier::Primitive { type_name: PrimitiveType::Integer, .. }));
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
        if let TypeSpecifier::Owned { ownership, base_type, .. } = type_spec {
            assert_eq!(ownership, OwnershipKind::BorrowedMut);
            assert!(matches!(*base_type, TypeSpecifier::Primitive { type_name: PrimitiveType::Integer, .. }));
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
        if let TypeSpecifier::Owned { ownership, base_type, .. } = type_spec {
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
        if let TypeSpecifier::Generic { base_type, type_arguments, .. } = type_spec {
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
        if let TypeSpecifier::Owned { ownership, base_type, .. } = type_spec {
            assert_eq!(ownership, OwnershipKind::Owned);
            if let TypeSpecifier::Array { element_type, .. } = *base_type {
                if let TypeSpecifier::Owned { ownership: inner_ownership, .. } = *element_type {
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
        assert!(matches!(*func.return_type, TypeSpecifier::Primitive { type_name: PrimitiveType::Void, .. }));
    }

    #[test]
    fn test_parse_function_with_return_type() {
        let mut parser = parser_from_source("func answer() -> Int { }");
        let result = parser.parse_function();

        assert!(result.is_ok());
        let func = result.unwrap();
        assert_eq!(func.name.name, "answer");
        assert!(func.parameters.is_empty());
        assert!(matches!(*func.return_type, TypeSpecifier::Primitive { type_name: PrimitiveType::Integer, .. }));
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
        assert!(matches!(*func.parameters[0].param_type, TypeSpecifier::Primitive { type_name: PrimitiveType::String, .. }));
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
        assert!(matches!(*func.return_type, TypeSpecifier::Primitive { type_name: PrimitiveType::Integer, .. }));
    }

    #[test]
    fn test_parse_function_complex_types() {
        let mut parser = parser_from_source("func process(items: Array<Int>, config: Map<String, Int>) -> Bool { }");
        let result = parser.parse_function();

        assert!(result.is_ok());
        let func = result.unwrap();
        assert_eq!(func.name.name, "process");
        assert_eq!(func.parameters.len(), 2);
        assert!(matches!(*func.parameters[0].param_type, TypeSpecifier::Array { .. }));
        assert!(matches!(*func.parameters[1].param_type, TypeSpecifier::Map { .. }));
    }

    #[test]
    fn test_parse_function_ownership_types() {
        let mut parser = parser_from_source("func transfer(owned: ^String, borrowed: &Int) -> Void { }");
        let result = parser.parse_function();

        assert!(result.is_ok());
        let func = result.unwrap();
        assert_eq!(func.parameters.len(), 2);
        assert!(matches!(*func.parameters[0].param_type, TypeSpecifier::Owned { ownership: OwnershipKind::Owned, .. }));
        assert!(matches!(*func.parameters[1].param_type, TypeSpecifier::Owned { ownership: OwnershipKind::Borrowed, .. }));
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
        assert!(matches!(*func.return_type, TypeSpecifier::Pointer { is_mutable: false, .. }));
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
        assert!(matches!(&annotation.arguments[0].value, AnnotationValue::String(s) if s == "libc"));
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
        assert!(matches!(&annotation.arguments[1].value, AnnotationValue::String(s) if s == "malloc"));
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
        assert!(matches!(&annotation.arguments[0].value, AnnotationValue::Expression(expr, _) if expr.contains("n") && expr.contains(">") && expr.contains("0")));
    }

    #[test]
    fn test_parse_annotation_with_identifier_value() {
        let mut parser = parser_from_source("@category(math)");
        let result = parser.parse_annotation();

        assert!(result.is_ok());
        let annotation = result.unwrap();
        assert_eq!(annotation.name, "category");
        assert_eq!(annotation.arguments.len(), 1);
        assert!(matches!(&annotation.arguments[0].value, AnnotationValue::Identifier(s) if s == "math"));
    }

    #[test]
    fn test_parse_annotation_with_integer_value() {
        let mut parser = parser_from_source("@priority(10)");
        let result = parser.parse_annotation();

        assert!(result.is_ok());
        let annotation = result.unwrap();
        assert_eq!(annotation.name, "priority");
        assert_eq!(annotation.arguments.len(), 1);
        assert!(matches!(&annotation.arguments[0].value, AnnotationValue::Integer(10)));
    }

    #[test]
    fn test_parse_annotation_with_boolean_value() {
        let mut parser = parser_from_source("@deprecated(true)");
        let result = parser.parse_annotation();

        assert!(result.is_ok());
        let annotation = result.unwrap();
        assert_eq!(annotation.name, "deprecated");
        assert_eq!(annotation.arguments.len(), 1);
        assert!(matches!(&annotation.arguments[0].value, AnnotationValue::Boolean(true)));
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
        let mut parser = parser_from_source("@extern(library: \"libc\") func malloc(size: SizeT) -> Pointer<Void>;");

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
        let mut parser = parser_from_source("@extern(library: \"libc\") func malloc(size: SizeT) -> Pointer<Void>");

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
        if let Statement::VariableDeclaration { name, type_spec, mutability, initial_value, .. } = stmt {
            assert_eq!(name.name, "x");
            assert!(matches!(*type_spec, TypeSpecifier::Primitive { type_name: PrimitiveType::Integer, .. }));
            assert!(matches!(mutability, Mutability::Immutable));
            assert!(initial_value.is_some());
            let value = initial_value.unwrap();
            assert!(matches!(*value, Expression::IntegerLiteral { value: 42, .. }));
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
        if let Statement::VariableDeclaration { name, mutability, .. } = stmt {
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
            assert!(matches!(*value, Expression::StringLiteral { ref value, .. } if value == "hello"));
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
            assert!(matches!(*value, Expression::BooleanLiteral { value: true, .. }));
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
            assert!(matches!(*value, Expression::CharacterLiteral { value: 'a', .. }));
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
        if let Statement::VariableDeclaration { name, initial_value, .. } = stmt {
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
        if let Statement::VariableDeclaration { name, type_spec, initial_value, .. } = stmt {
            assert_eq!(name.name, "x");
            // Type should be _inferred placeholder
            assert!(matches!(*type_spec, TypeSpecifier::Named { ref name, .. } if name.name == "_inferred"));
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
        assert!(matches!(expr, Expression::StringLiteral { ref value, .. } if value == "hello world"));
    }

    #[test]
    fn test_parse_expression_boolean() {
        let mut parser = parser_from_source("true");
        let result = parser.parse_expression();

        assert!(result.is_ok());
        let expr = result.unwrap();
        assert!(matches!(expr, Expression::BooleanLiteral { value: true, .. }));
    }

    #[test]
    fn test_parse_expression_identifier() {
        let mut parser = parser_from_source("myVar");
        let result = parser.parse_expression();

        assert!(result.is_ok());
        let expr = result.unwrap();
        assert!(matches!(expr, Expression::Variable { ref name, .. } if name.name == "myVar"));
    }
}
