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

#![allow(dead_code)]

use crate::ast::{
    Argument, AssignmentTarget, Block, CallingConvention, Capture, CaptureMode, CatchClause,
    ConstantDeclaration, ContractAssertion, ElseIf, EnumVariant, ExportStatement, Expression, ExternalFunction, FailureAction, FieldValue, Function,
    FunctionCall as AstFunctionCall, FunctionMetadata, FunctionReference, GenericParameter, Identifier,
    ImportStatement, LambdaBody, MatchArm, MatchCase, Module, Mutability, OwnershipKind, Parameter, PassingMode, Pattern, PrimitiveType, Program,
    Quantifier, QuantifierKind, QuantifierVariable,
    Statement, StructField, TraitAxiom, TraitDefinition, TraitMethod, TypeDefinition, TypeSpecifier, PerformanceMetric, PerformanceExpectation, ComplexityExpectation, ComplexityType, ComplexityNotation,
    WhereClause,
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
    Float(f64),
    Boolean(bool),
    Identifier(String),
    Expression(Box<Expression>),
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
                    | Keyword::If
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
        let mut trait_definitions = Vec::new();
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
                    &mut trait_definitions,
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
                    &mut trait_definitions,
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
            trait_definitions,
            constant_declarations,
            function_definitions,
            external_functions,
            source_location: start_location,
        };

        let errors = self.take_errors();
        (Some(module), errors)
    }

    /// Extract contract condition and runtime_check flag from annotation arguments
    /// Syntax: @pre({condition}) or @pre({condition}, check=runtime)
    fn extract_contract_args(args: &[AnnotationArgument]) -> (Option<Box<Expression>>, bool) {
        let mut condition: Option<Box<Expression>> = None;
        let mut runtime_check = false;

        for arg in args {
            // Check for the condition expression (unlabeled argument)
            if arg.label.is_none() {
                if let AnnotationValue::Expression(expr) = &arg.value {
                    condition = Some(expr.clone());
                }
            }
            // Check for check=runtime
            else if arg.label.as_deref() == Some("check") {
                if let AnnotationValue::Identifier(id) = &arg.value {
                    if id == "runtime" {
                        runtime_check = true;
                    }
                }
            }
        }

        (condition, runtime_check)
    }

    /// Apply annotations to a function
    fn apply_annotations(&self, func: &mut Function, annotations: Vec<Annotation>) -> Result<(), ParserError> {
        for ann in annotations {
            match ann.name.as_str() {
                "intent" => {
                    if let Some(arg) = ann.arguments.first() {
                        if let AnnotationValue::String(s) = &arg.value {
                            func.intent = Some(s.clone());
                        }
                    }
                }
                "pre" | "requires" => {
                    let (condition, runtime_check) = Self::extract_contract_args(&ann.arguments);
                    if let Some(expr) = condition {
                        func.metadata.preconditions.push(ContractAssertion {
                            condition: expr,
                            failure_action: FailureAction::ThrowException,
                            message: None,
                            runtime_check,
                            source_location: ann.source_location.clone(),
                        });
                    }
                }
                "post" | "ensures" => {
                    let (condition, runtime_check) = Self::extract_contract_args(&ann.arguments);
                    if let Some(expr) = condition {
                        func.metadata.postconditions.push(ContractAssertion {
                            condition: expr,
                            failure_action: FailureAction::ThrowException,
                            message: None,
                            runtime_check,
                            source_location: ann.source_location.clone(),
                        });
                    }
                }
                "invariant" => {
                    let (condition, runtime_check) = Self::extract_contract_args(&ann.arguments);
                    if let Some(expr) = condition {
                        func.metadata.invariants.push(ContractAssertion {
                            condition: expr,
                            failure_action: FailureAction::ThrowException,
                            message: None,
                            runtime_check,
                            source_location: ann.source_location.clone(),
                        });
                    }
                }
                "algo" => {
                    if let Some(arg) = ann.arguments.first() {
                        if let AnnotationValue::String(s) = &arg.value {
                            func.metadata.algorithm_hint = Some(s.clone());
                        }
                    }
                }
                "perf" => {
                    let mut metric = None;
                    let mut target = 0.0;
                    let mut context = None;
                    
                    for arg in &ann.arguments {
                         if let Some(label) = &arg.label {
                             match label.as_str() {
                                 "metric" => {
                                     if let AnnotationValue::String(s) = &arg.value {
                                         metric = match s.as_str() {
                                             "LatencyMs" => Some(PerformanceMetric::LatencyMs),
                                             "ThroughputOpsPerSec" => Some(PerformanceMetric::ThroughputOpsPerSec),
                                             "MemoryUsageBytes" => Some(PerformanceMetric::MemoryUsageBytes),
                                             _ => None,
                                         };
                                     }
                                 }
                                 "target" => {
                                     if let AnnotationValue::Float(f) = &arg.value {
                                         target = *f;
                                     } else if let AnnotationValue::Integer(i) = &arg.value {
                                         target = *i as f64;
                                     }
                                 }
                                 "context" => {
                                     if let AnnotationValue::String(s) = &arg.value {
                                         context = Some(s.clone());
                                     }
                                 }
                                 _ => {}
                             }
                         }
                    }
                    if let Some(m) = metric {
                        func.metadata.performance_expectation = Some(PerformanceExpectation {
                            metric: m,
                            target_value: target,
                            context,
                        });
                    }
                }
                "complexity" => {
                    let mut comp_type = ComplexityType::Time;
                    let mut notation = ComplexityNotation::BigO;
                    let mut value = "".to_string();

                    for arg in &ann.arguments {
                        if let Some(label) = &arg.label {
                            match label.as_str() {
                                "type" => {
                                    if let AnnotationValue::String(s) = &arg.value {
                                        comp_type = match s.as_str() {
                                            "Time" => ComplexityType::Time,
                                            "Space" => ComplexityType::Space,
                                            _ => ComplexityType::Time,
                                        };
                                    }
                                }
                                "notation" => {
                                    if let AnnotationValue::String(s) = &arg.value {
                                        notation = match s.as_str() {
                                            "BigO" => ComplexityNotation::BigO,
                                            "BigTheta" => ComplexityNotation::BigTheta,
                                            "BigOmega" => ComplexityNotation::BigOmega,
                                            _ => ComplexityNotation::BigO,
                                        };
                                    }
                                }
                                "value" => {
                                    if let AnnotationValue::String(s) = &arg.value {
                                        value = s.clone();
                                    }
                                }
                                _ => {}
                            }
                        }
                    }
                    func.metadata.complexity_expectation = Some(ComplexityExpectation {
                        complexity_type: comp_type,
                        notation,
                        value,
                    });
                }
                "thread_safe" => {
                    if let Some(arg) = ann.arguments.first() {
                        if let AnnotationValue::Boolean(b) = &arg.value {
                            func.metadata.thread_safe = Some(*b);
                        }
                    }
                }
                "may_block" => {
                    if let Some(arg) = ann.arguments.first() {
                        if let AnnotationValue::Boolean(b) = &arg.value {
                            func.metadata.may_block = Some(*b);
                        }
                    }
                }
                _ => {}
            }
        }
        Ok(())
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
        let mut trait_definitions = Vec::new();
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
                    &mut trait_definitions,
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
                    &mut trait_definitions,
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
            trait_definitions,
            constant_declarations,
            function_definitions,
            external_functions,
            source_location: start_location,
        })
    }

    /// Parse a single module item (import, function, struct, enum, trait, extern)
    fn parse_module_item(
        &mut self,
        imports: &mut Vec<ImportStatement>,
        function_definitions: &mut Vec<Function>,
        external_functions: &mut Vec<ExternalFunction>,
        type_definitions: &mut Vec<TypeDefinition>,
        trait_definitions: &mut Vec<TraitDefinition>,
        constant_declarations: &mut Vec<ConstantDeclaration>,
        exports: &mut Vec<ExportStatement>,
    ) -> Result<(), ParserError> {
        // Collect annotations
        let mut annotations = Vec::new();
        while self.check(&TokenType::At) {
            annotations.push(self.parse_annotation()?);
        }

        // Check for visibility modifier
        let is_public = if self.check_keyword(Keyword::Pub) {
            self.advance();
            true
        } else {
            false
        };

        if self.check_keyword(Keyword::Import) {
            imports.push(self.parse_import()?);
        } else if self.check_keyword(Keyword::Func) {
            // Check for @extern in annotations
            if let Some(extern_attr) = annotations.iter().find(|a| a.name == "extern") {
                external_functions.push(self.parse_external_function(extern_attr.clone())?);
            } else {
                let mut func = self.parse_function()?;
                // Apply annotations to func.metadata
                self.apply_annotations(&mut func, annotations)?;
                
                if is_public {
                    exports.push(ExportStatement::Function {
                        name: func.name.clone(),
                        source_location: func.source_location.clone(),
                    });
                }
                function_definitions.push(func);
            }
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
        } else if self.check_keyword(Keyword::Trait) {
            let trait_def = self.parse_trait_definition()?;
            if is_public {
                exports.push(ExportStatement::Type {
                    name: trait_def.name.clone(),
                    source_location: trait_def.source_location.clone(),
                });
            }
            trait_definitions.push(trait_def);
        } else {
            return Err(ParserError::UnexpectedToken {
                expected: "import, func, struct, enum, const, trait, or annotation".to_string(),
                found: format!("{:?}", self.peek().token_type),
                location: self.current_location(),
            });
        }

        Ok(())
    }

    /// Parse an import statement
    /// Grammar: "import" dotted_name ["as" identifier] ";"
    pub fn parse_import(&mut self) -> Result<ImportStatement, ParserError> {
        let start_location = self.current_location();

        // Expect 'import' keyword
        self.expect_keyword(Keyword::Import, "expected 'import'")?;

        // Parse dotted module name (e.g., std.io)
        let module_name = self.parse_dotted_identifier()?;

        // Check for alias
        let alias = if self.check_keyword(Keyword::As) {
            self.advance(); // consume 'as'
            Some(self.parse_identifier()?)
        } else {
            None
        };

        // Expect semicolon
        self.expect(&TokenType::Semicolon, "expected ';' after import statement")?;

        Ok(ImportStatement {
            module_name,
            alias,
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

    /// Parse generic type parameters for functions, structs, and enums
    /// Grammar: "<" generic_param ("," generic_param)* ">"
    /// generic_param: IDENTIFIER
    /// Returns empty Vec if no generic parameters are present
    pub fn parse_generic_parameters(&mut self) -> Result<Vec<GenericParameter>, ParserError> {
        // Check if there are generic parameters
        if !self.check(&TokenType::Less) {
            return Ok(Vec::new());
        }

        self.advance(); // consume '<'

        let mut params = Vec::new();

        // Parse first parameter
        let start_location = self.current_location();
        let name = self.parse_identifier()?;
        params.push(GenericParameter {
            name,
            constraints: Vec::new(),
            default_type: None,
            source_location: start_location,
        });

        // Parse additional parameters
        while self.check(&TokenType::Comma) {
            self.advance(); // consume ','
            let param_location = self.current_location();
            let param_name = self.parse_identifier()?;
            params.push(GenericParameter {
                name: param_name,
                constraints: Vec::new(),
                default_type: None,
                source_location: param_location,
            });
        }

        self.expect(&TokenType::Greater, "expected '>' to close generic parameters")?;

        Ok(params)
    }

    /// Parse an optional where clause for generic constraints
    /// Grammar: ("where" where_constraint ("," where_constraint)*)?
    /// where_constraint: IDENTIFIER ":" constraint ("+" constraint)*
    /// constraint: IDENTIFIER
    /// Returns empty Vec if no where clause is present
    pub fn parse_where_clause(&mut self) -> Result<Vec<WhereClause>, ParserError> {
        // Check if there's a where clause
        if !self.check_keyword(Keyword::Where) {
            return Ok(Vec::new());
        }

        self.advance(); // consume 'where'

        let mut clauses = Vec::new();

        // Parse first constraint
        clauses.push(self.parse_where_constraint()?);

        // Parse additional constraints separated by commas
        while self.check(&TokenType::Comma) {
            self.advance(); // consume ','
            clauses.push(self.parse_where_constraint()?);
        }

        Ok(clauses)
    }

    /// Parse a single where constraint: T: Display + Debug
    fn parse_where_constraint(&mut self) -> Result<WhereClause, ParserError> {
        let start_location = self.current_location();

        // Parse the type parameter name
        let type_param = self.parse_identifier()?;

        // Expect colon
        self.expect(&TokenType::Colon, "expected ':' after type parameter in where clause")?;

        // Parse first constraint (trait bound)
        let mut constraints = Vec::new();
        constraints.push(self.parse_identifier()?);

        // Parse additional constraints separated by +
        while self.check(&TokenType::Plus) {
            self.advance(); // consume '+'
            constraints.push(self.parse_identifier()?);
        }

        Ok(WhereClause {
            type_param,
            constraints,
            source_location: start_location,
        })
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
                lifetime: None,
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
                                lifetime: None,
                                source_location: start_location,
                            });            }
            let base_type = self.parse_type()?;
                            return Ok(TypeSpecifier::Owned {
                                base_type: Box::new(base_type),
                                ownership: OwnershipKind::Borrowed,
                                lifetime: None,
                                source_location: start_location,
                            });        }

        if self.check(&TokenType::Tilde) {
            self.advance();
            let base_type = self.parse_type()?;
            return Ok(TypeSpecifier::Owned {
                base_type: Box::new(base_type),
                ownership: OwnershipKind::Shared,
                lifetime: None,
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
        let name = self.parse_dotted_identifier()?;

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
    /// Grammar: "func" IDENTIFIER generic_params? "(" params? ")" ("->" type)? where_clause? block
    pub fn parse_function(&mut self) -> Result<Function, ParserError> {
        let start_location = self.current_location();

        // Expect 'func' keyword
        self.expect_keyword(Keyword::Func, "expected 'func'")?;

        // Parse function name
        let name = self.parse_identifier()?;

        // Parse optional generic parameters
        let generic_parameters = self.parse_generic_parameters()?;

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

        // Parse optional where clause
        let where_clause = self.parse_where_clause()?;

        // Parse function body
        let body = self.parse_block()?;

        Ok(Function {
            name,
            intent: None,
            generic_parameters,
            lifetime_parameters: Vec::new(),
            where_clause,
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
            is_async: false,
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

    /// Parse an annotation argument (key: value, key=value, or just value)
    fn parse_annotation_argument(&mut self) -> Result<AnnotationArgument, ParserError> {
        let start_location = self.current_location();

        // Check if this is a labeled argument (key: value or key=value)
        if let TokenType::Identifier(name) = &self.peek().token_type {
            let name_clone = name.clone();
            if let Some(next) = self.peek_next() {
                if matches!(next.token_type, TokenType::Colon) || matches!(next.token_type, TokenType::Equal) {
                    // Labeled argument
                    self.advance(); // consume identifier
                    self.advance(); // consume colon or equal
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

        // Float literal
        if let TokenType::FloatLiteral(f) = &self.peek().token_type {
            let value = *f;
            self.advance();
            return Ok(AnnotationValue::Float(value));
        }

        // Boolean literal
        if let TokenType::BoolLiteral(b) = &self.peek().token_type {
            let value = *b;
            self.advance();
            return Ok(AnnotationValue::Boolean(value));
        }

        // Braced expression (for contracts like @requires({n > 0}))
        if self.check(&TokenType::LeftBrace) {
            let expr = self.parse_braced_expression()?;
            return Ok(AnnotationValue::Expression(Box::new(expr)));
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

        // Extract optional variadic flag
        let variadic = annotation
            .arguments
            .iter()
            .find(|a| a.label.as_deref() == Some("variadic"))
            .and_then(|a| match &a.value {
                AnnotationValue::Boolean(b) => Some(*b),
                _ => None,
            })
            .unwrap_or(false);

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
            variadic,
            ownership_info: None,
            source_location: start_location,
        })
    }

    // ==================== STRUCT PARSING ====================

    /// Parse a struct definition
    /// Grammar: "struct" IDENTIFIER generic_params? "{" field* "}"
    pub fn parse_struct(&mut self) -> Result<TypeDefinition, ParserError> {
        let start_location = self.current_location();

        self.expect_keyword(Keyword::Struct, "expected 'struct'")?;

        let name = self.parse_identifier()?;

        // Parse optional generic parameters
        let generic_parameters = self.parse_generic_parameters()?;

        // Parse optional where clause
        let where_clause = self.parse_where_clause()?;

        self.expect(&TokenType::LeftBrace, "expected '{' after struct name")?;

        let mut fields = Vec::new();

        while !self.check(&TokenType::RightBrace) && !self.is_at_end() {
            fields.push(self.parse_struct_field()?);
        }

        self.expect(&TokenType::RightBrace, "expected '}' to close struct")?;

        Ok(TypeDefinition::Structured {
            name,
            intent: None,
            generic_parameters,
            lifetime_parameters: vec![],
            where_clause,
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
    /// Grammar: "enum" IDENTIFIER generic_params? where_clause? "{" ("case" IDENTIFIER ["(" type ")"] ";")* "}"
    pub fn parse_enum(&mut self) -> Result<TypeDefinition, ParserError> {
        let start_location = self.current_location();

        self.expect_keyword(Keyword::Enum, "expected 'enum'")?;

        let name = self.parse_identifier()?;

        // Parse optional generic parameters
        let generic_parameters = self.parse_generic_parameters()?;

        // Parse optional where clause
        let where_clause = self.parse_where_clause()?;

        self.expect(&TokenType::LeftBrace, "expected '{' after enum name")?;

        let mut variants = Vec::new();

        while !self.check(&TokenType::RightBrace) && !self.is_at_end() {
            variants.push(self.parse_enum_variant()?);
        }

        self.expect(&TokenType::RightBrace, "expected '}' to close enum")?;

        Ok(TypeDefinition::Enumeration {
            name,
            intent: None,
            generic_parameters,
            lifetime_parameters: vec![],
            where_clause,
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

    // ==================== TRAIT DEFINITION PARSING ====================

    /// Parse a trait definition
    /// Grammar: "trait" IDENTIFIER ["<" generic_params ">"] [where_clause] "{" trait_method* "}"
    pub fn parse_trait_definition(&mut self) -> Result<TraitDefinition, ParserError> {
        let start_location = self.current_location();

        self.expect_keyword(Keyword::Trait, "expected 'trait'")?;

        let name = self.parse_identifier()?;

        // Parse optional generic parameters
        let generic_parameters = self.parse_generic_parameters()?;

        // Parse optional where clause
        let where_clause = self.parse_where_clause()?;

        self.expect(&TokenType::LeftBrace, "expected '{' after trait name")?;

        let mut axioms = Vec::new();
        let mut methods = Vec::new();

        while !self.check(&TokenType::RightBrace) && !self.is_at_end() {
            // Check if this is an axiom (@axiom ...)
            if self.check(&TokenType::At) {
                // Peek ahead to see if this is @axiom
                if let Some(next) = self.peek_next() {
                    if let TokenType::Identifier(name) = &next.token_type {
                        if name == "axiom" {
                            axioms.push(self.parse_axiom()?);
                            continue;
                        }
                    }
                }
            }
            // Otherwise parse as a method
            methods.push(self.parse_trait_method()?);
        }

        self.expect(&TokenType::RightBrace, "expected '}' after trait methods")?;

        Ok(TraitDefinition {
            name,
            generic_parameters,
            where_clause,
            axioms,
            methods,
            source_location: start_location,
        })
    }

    /// Parse a trait method (signature with optional default implementation)
    /// Grammar: "fn" IDENTIFIER ["<" generic_params ">"] "(" params ")" "->" type (";" | block)
    fn parse_trait_method(&mut self) -> Result<TraitMethod, ParserError> {
        let start_location = self.current_location();

        self.expect_keyword(Keyword::Func, "expected 'fn' for trait method")?;

        let name = self.parse_identifier()?;

        // Parse optional generic parameters for the method
        let generic_parameters = self.parse_generic_parameters()?;

        // Parse parameters
        self.expect(&TokenType::LeftParen, "expected '(' after method name")?;
        let mut parameters = Vec::new();
        while !self.check(&TokenType::RightParen) && !self.is_at_end() {
            parameters.push(self.parse_parameter()?);
            if !self.check(&TokenType::RightParen) {
                self.expect(&TokenType::Comma, "expected ',' between parameters")?;
            }
        }
        self.expect(&TokenType::RightParen, "expected ')' after parameters")?;

        // Parse return type
        self.expect(&TokenType::Arrow, "expected '->' before return type")?;
        let return_type = Box::new(self.parse_type()?);

        // Check for default implementation (block) or just declaration (semicolon)
        let default_body = if self.check(&TokenType::LeftBrace) {
            Some(self.parse_block()?)
        } else {
            self.expect(&TokenType::Semicolon, "expected ';' or '{' after method signature")?;
            None
        };

        Ok(TraitMethod {
            name,
            generic_parameters,
            parameters,
            return_type,
            default_body,
            source_location: start_location,
        })
    }

    /// Parse a trait axiom
    /// Grammar: "@axiom" [name ":"] [quantifiers "=>"] expression
    /// Quantifiers: "forall" var_decl {"," var_decl} | "exists" var_decl {"," var_decl}
    fn parse_axiom(&mut self) -> Result<TraitAxiom, ParserError> {
        let start_location = self.current_location();

        // Consume the @axiom annotation - we already checked for it
        self.expect(&TokenType::At, "expected '@'")?;
        self.expect_identifier_value("axiom")?;

        // Check for optional name: "name: ..."
        let name = if let TokenType::Identifier(ident) = &self.peek().token_type {
            // Check if followed by colon (to distinguish from expression start)
            if self.peek_next().map(|t| matches!(t.token_type, TokenType::Colon)).unwrap_or(false) {
                let name = Identifier::new(ident.clone(), self.current_location());
                self.advance(); // consume identifier
                self.advance(); // consume colon
                Some(name)
            } else {
                None
            }
        } else {
            None
        };

        // Parse optional quantifiers
        let mut quantifiers = Vec::new();
        while self.check_keyword(Keyword::ForAll) || self.check_keyword(Keyword::Exists) {
            quantifiers.push(self.parse_quantifier()?);
        }

        // Parse the condition expression
        let condition = Box::new(self.parse_expression()?);

        Ok(TraitAxiom {
            name,
            quantifiers,
            condition,
            source_location: start_location,
        })
    }

    /// Parse a quantifier (forall or exists with variable bindings)
    /// Grammar: ("forall" | "exists") var_decl {"," var_decl} "=>"
    fn parse_quantifier(&mut self) -> Result<Quantifier, ParserError> {
        let start_location = self.current_location();

        let kind = if self.check_keyword(Keyword::ForAll) {
            self.advance();
            QuantifierKind::ForAll
        } else if self.check_keyword(Keyword::Exists) {
            self.advance();
            QuantifierKind::Exists
        } else {
            return Err(ParserError::UnexpectedToken {
                expected: "expected 'forall' or 'exists'".to_string(),
                found: format!("{:?}", self.peek().token_type),
                location: self.peek().location.clone(),
            });
        };

        // Parse variable bindings: x: Type, y: Type, ...
        let mut variables = Vec::new();
        loop {
            let var_start = self.current_location();
            let var_name = self.parse_identifier()?;
            self.expect(&TokenType::Colon, "expected ':' after quantifier variable")?;
            let var_type = Box::new(self.parse_type()?);

            variables.push(QuantifierVariable {
                name: var_name,
                var_type,
                source_location: var_start,
            });

            if self.check(&TokenType::Comma) {
                self.advance(); // consume comma
            } else {
                break;
            }
        }

        // Expect "=>" after quantifier bindings
        self.expect(&TokenType::FatArrow, "expected '=>' after quantifier variables")?;

        Ok(Quantifier {
            kind,
            variables,
            source_location: start_location,
        })
    }

    /// Check if the next token is an identifier with a specific value
    fn expect_identifier_value(&mut self, expected: &str) -> Result<(), ParserError> {
        match &self.peek().token_type {
            TokenType::Identifier(name) if name == expected => {
                self.advance();
                Ok(())
            }
            _ => Err(ParserError::UnexpectedToken {
                expected: format!("expected '{}'", expected),
                found: format!("{:?}", self.peek().token_type),
                location: self.peek().location.clone(),
            }),
        }
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

    /// Parse an if statement
    /// Grammar: "if" expression block ["else" "if" expression block]* ["else" block]
    pub fn parse_if_statement(&mut self) -> Result<Statement, ParserError> {
        let start_location = self.current_location();

        self.expect_keyword(Keyword::If, "expected 'if'")?;

        // Parse condition
        let condition = self.parse_expression()?;

        // Parse then block
        let then_block = self.parse_block()?;

        // Parse else-ifs and else
        let mut else_ifs = Vec::new();
        let mut else_block = None;

        while self.check_keyword(Keyword::Else) {
            self.advance(); // consume 'else'

            if self.check_keyword(Keyword::If) {
                // else if
                let else_if_location = self.current_location();
                self.advance(); // consume 'if'

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

    /// Parse a try statement
    /// Grammar: "try" block ("catch" type ["as" name] block)* ["finally" block]
    pub fn parse_try_statement(&mut self) -> Result<Statement, ParserError> {
        let start_location = self.current_location();

        self.expect_keyword(Keyword::Try, "expected 'try'")?;
        let protected_block = self.parse_block()?;

        let mut catch_clauses = Vec::new();
        while self.check_keyword(Keyword::Catch) {
            let catch_loc = self.current_location();
            self.advance();

            let exception_type = Box::new(self.parse_type()?);
            
            let binding_variable = if self.check_keyword(Keyword::As) {
                self.advance();
                Some(self.parse_identifier()?)
            } else {
                None
            };

            let handler_block = self.parse_block()?;
            
            catch_clauses.push(CatchClause {
                exception_type,
                binding_variable,
                handler_block,
                source_location: catch_loc,
            });
        }

        let finally_block = if self.check_keyword(Keyword::Finally) {
            self.advance();
            Some(self.parse_block()?)
        } else {
            None
        };

        Ok(Statement::TryBlock {
            protected_block,
            catch_clauses,
            finally_block,
            source_location: start_location,
        })
    }

    /// Parse a throw statement
    /// Grammar: "throw" expression ";"
    pub fn parse_throw_statement(&mut self) -> Result<Statement, ParserError> {
        let start_location = self.current_location();
        self.expect_keyword(Keyword::Throw, "expected 'throw'")?;
        let exception = self.parse_expression()?;
        self.expect(&TokenType::Semicolon, "expected ';' after throw")?;
        
        Ok(Statement::Throw {
            exception: Box::new(exception),
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
        if self.check_keyword(Keyword::If) {
            return self.parse_if_statement();
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
        if self.check_keyword(Keyword::Try) {
            return self.parse_try_statement();
        }
        if self.check_keyword(Keyword::Throw) {
            return self.parse_throw_statement();
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

    /// Parse function arguments with optional labels: (val, label: val)
    /// Returns vector of (label, value) tuples
    fn parse_argument_list(&mut self) -> Result<Vec<(Option<String>, Expression)>, ParserError> {
        self.expect(&TokenType::LeftParen, "expected '('")?;
        let mut args = Vec::new();

        if !self.check(&TokenType::RightParen) {
            loop {
                // Check for label: label: expr
                // Need to look ahead: identifier + colon
                let mut label = None;
                if let TokenType::Identifier(name) = &self.peek().token_type {
                    if let Some(next) = self.peek_next() {
                        if matches!(next.token_type, TokenType::Colon) {
                            label = Some(name.clone());
                            self.advance(); // consume label
                            self.advance(); // consume colon
                        }
                    }
                }

                let value = self.parse_expression()?;
                args.push((label, value));

                if self.check(&TokenType::Comma) {
                    self.advance();
                } else {
                    break;
                }
            }
        }

        self.expect(&TokenType::RightParen, "expected ')'")?;
        Ok(args)
    }

    /// Parse an expression using precedence-based parsing
    /// Supports both braced expressions `{a + b}` (for backward compatibility)
    /// and unbraced expressions `a + b`
    pub fn parse_expression(&mut self) -> Result<Expression, ParserError> {
        // Use the precedence-based expression parser, starting with range (lowest precedence)
        self.parse_range_expression_full()
    }

    /// Parse range expression: expr..expr or expr..=expr (lowest precedence)
    fn parse_range_expression_full(&mut self) -> Result<Expression, ParserError> {
        let start_location = self.current_location();

        // Check for prefix range: ..end or ..=end
        if self.check(&TokenType::DotDot) {
            self.advance();
            let end = self.parse_or_expression()?;
            return Ok(Expression::Range {
                start: None,
                end: Some(Box::new(end)),
                inclusive: false,
                source_location: start_location,
            });
        }
        if self.check(&TokenType::DotDotEqual) {
            self.advance();
            let end = self.parse_or_expression()?;
            return Ok(Expression::Range {
                start: None,
                end: Some(Box::new(end)),
                inclusive: true,
                source_location: start_location,
            });
        }

        // Parse the left operand
        let left = self.parse_or_expression()?;

        // Check for postfix range: start.. or start..end
        if self.check(&TokenType::DotDot) {
            self.advance();
            // Check if there's an end value
            if self.is_expression_start() {
                let end = self.parse_or_expression()?;
                return Ok(Expression::Range {
                    start: Some(Box::new(left)),
                    end: Some(Box::new(end)),
                    inclusive: false,
                    source_location: start_location,
                });
            } else {
                return Ok(Expression::Range {
                    start: Some(Box::new(left)),
                    end: None,
                    inclusive: false,
                    source_location: start_location,
                });
            }
        }
        if self.check(&TokenType::DotDotEqual) {
            self.advance();
            let end = self.parse_or_expression()?;
            return Ok(Expression::Range {
                start: Some(Box::new(left)),
                end: Some(Box::new(end)),
                inclusive: true,
                source_location: start_location,
            });
        }

        Ok(left)
    }

    /// Check if current token can start an expression
    fn is_expression_start(&self) -> bool {
        match &self.peek().token_type {
            TokenType::IntegerLiteral(_)
            | TokenType::FloatLiteral(_)
            | TokenType::StringLiteral(_)
            | TokenType::CharLiteral(_)
            | TokenType::BoolLiteral(_)
            | TokenType::Identifier(_)
            | TokenType::LeftParen
            | TokenType::LeftBrace
            | TokenType::LeftBracket
            | TokenType::Bang
            | TokenType::Minus => true,
            TokenType::Keyword(k) => matches!(k, Keyword::Match),
            _ => false,
        }
    }

    /// Legacy parse_expression implementation kept for reference but no longer used
    /// Handles special cases like address-of and move that aren't part of normal expressions
    fn parse_special_expression(&mut self) -> Result<Expression, ParserError> {
        let start_location = self.current_location();

        // Address of: &expr or &mut expr
        if self.check(&TokenType::Ampersand) {
            self.advance(); // consume '&'

            let is_mut = if self.check_keyword(Keyword::Mut) {
                self.advance(); // consume 'mut'
                true
            } else {
                false
            };

            let operand = self.parse_expression()?;

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
            return Ok(operand); // Treating ^ as a semantic marker
        }

        // Prefix range expression: ..end or ..=end
        if self.check(&TokenType::DotDot) || self.check(&TokenType::DotDotEqual) {
            return self.parse_range_expression(None, start_location);
        }

        // Fall back to precedence-based parsing
        self.parse_or_expression()
    }

    /// Parse a braced binary expression: `{left op right}`
    /// Kept for backward compatibility - braces are now optional
    fn parse_braced_expression(&mut self) -> Result<Expression, ParserError> {
        self.expect(&TokenType::LeftBrace, "expected '{'")?;

        // Parse the inner expression using precedence-based parsing
        let expr = self.parse_or_expression()?;

        self.expect(
            &TokenType::RightBrace,
            "expected '}' after expression",
        )?;

        Ok(expr)
    }

    // ==================== PRECEDENCE-BASED EXPRESSION PARSING ====================
    // Precedence (lowest to highest):
    // 1. || (logical or)
    // 2. && (logical and)
    // 3. == != (equality)
    // 4. < > <= >= (comparison)
    // 5. + - (additive)
    // 6. * / % (multiplicative)
    // 7. unary (! -)
    // 8. primary (literals, identifiers, function calls)

    /// Parse logical OR expression: expr || expr
    fn parse_or_expression(&mut self) -> Result<Expression, ParserError> {
        let start_location = self.current_location();
        let mut left = self.parse_and_expression()?;

        while self.check(&TokenType::PipePipe) {
            self.advance(); // consume ||
            let right = self.parse_and_expression()?;
            left = self.build_binary_expression(left, BinaryOp::Or, right, start_location.clone());
        }

        Ok(left)
    }

    /// Parse logical AND expression: expr && expr
    fn parse_and_expression(&mut self) -> Result<Expression, ParserError> {
        let start_location = self.current_location();
        let mut left = self.parse_equality_expression()?;

        while self.check(&TokenType::AmpAmp) {
            self.advance(); // consume &&
            let right = self.parse_equality_expression()?;
            left = self.build_binary_expression(left, BinaryOp::And, right, start_location.clone());
        }

        Ok(left)
    }

    /// Parse equality expression: expr == expr, expr != expr
    fn parse_equality_expression(&mut self) -> Result<Expression, ParserError> {
        let start_location = self.current_location();
        let mut left = self.parse_comparison_expression()?;

        loop {
            if self.check(&TokenType::EqualEqual) {
                self.advance();
                let right = self.parse_comparison_expression()?;
                left = self.build_binary_expression(left, BinaryOp::Equals, right, start_location.clone());
            } else if self.check(&TokenType::BangEqual) {
                self.advance();
                let right = self.parse_comparison_expression()?;
                left = self.build_binary_expression(left, BinaryOp::NotEquals, right, start_location.clone());
            } else {
                break;
            }
        }

        Ok(left)
    }

    /// Parse comparison expression: expr < expr, expr > expr, etc.
    fn parse_comparison_expression(&mut self) -> Result<Expression, ParserError> {
        let start_location = self.current_location();
        let mut left = self.parse_additive_expression()?;

        loop {
            if self.check(&TokenType::Less) {
                self.advance();
                let right = self.parse_additive_expression()?;
                left = self.build_binary_expression(left, BinaryOp::LessThan, right, start_location.clone());
            } else if self.check(&TokenType::LessEqual) {
                self.advance();
                let right = self.parse_additive_expression()?;
                left = self.build_binary_expression(left, BinaryOp::LessEqual, right, start_location.clone());
            } else if self.check(&TokenType::Greater) {
                self.advance();
                let right = self.parse_additive_expression()?;
                left = self.build_binary_expression(left, BinaryOp::GreaterThan, right, start_location.clone());
            } else if self.check(&TokenType::GreaterEqual) {
                self.advance();
                let right = self.parse_additive_expression()?;
                left = self.build_binary_expression(left, BinaryOp::GreaterEqual, right, start_location.clone());
            } else {
                break;
            }
        }

        Ok(left)
    }

    /// Parse additive expression: expr + expr, expr - expr
    fn parse_additive_expression(&mut self) -> Result<Expression, ParserError> {
        let start_location = self.current_location();
        let mut left = self.parse_multiplicative_expression()?;

        loop {
            if self.check(&TokenType::Plus) {
                self.advance();
                let right = self.parse_multiplicative_expression()?;
                left = self.build_binary_expression(left, BinaryOp::Add, right, start_location.clone());
            } else if self.check(&TokenType::Minus) {
                self.advance();
                let right = self.parse_multiplicative_expression()?;
                left = self.build_binary_expression(left, BinaryOp::Subtract, right, start_location.clone());
            } else {
                break;
            }
        }

        Ok(left)
    }

    /// Parse multiplicative expression: expr * expr, expr / expr, expr % expr
    fn parse_multiplicative_expression(&mut self) -> Result<Expression, ParserError> {
        let start_location = self.current_location();
        let mut left = self.parse_unary_expression()?;

        loop {
            if self.check(&TokenType::Star) {
                self.advance();
                let right = self.parse_unary_expression()?;
                left = self.build_binary_expression(left, BinaryOp::Multiply, right, start_location.clone());
            } else if self.check(&TokenType::Slash) {
                self.advance();
                let right = self.parse_unary_expression()?;
                left = self.build_binary_expression(left, BinaryOp::Divide, right, start_location.clone());
            } else if self.check(&TokenType::Percent) {
                self.advance();
                let right = self.parse_unary_expression()?;
                left = self.build_binary_expression(left, BinaryOp::Modulo, right, start_location.clone());
            } else {
                break;
            }
        }

        Ok(left)
    }

    /// Parse unary expression: !expr, -expr, &expr, &mut expr
    fn parse_unary_expression(&mut self) -> Result<Expression, ParserError> {
        let start_location = self.current_location();

        if self.check(&TokenType::Bang) {
            self.advance();
            let operand = self.parse_unary_expression()?;
            return Ok(Expression::LogicalNot {
                operand: Box::new(operand),
                source_location: start_location,
            });
        }

        if self.check(&TokenType::Minus) {
            self.advance();
            let operand = self.parse_unary_expression()?;
            return Ok(Expression::Negate {
                operand: Box::new(operand),
                source_location: start_location,
            });
        }

        // Address of: &expr or &mut expr
        if self.check(&TokenType::Ampersand) {
            self.advance(); // consume '&'

            let is_mut = if self.check_keyword(Keyword::Mut) {
                self.advance(); // consume 'mut'
                true
            } else {
                false
            };

            let operand = self.parse_unary_expression()?;

            return Ok(Expression::AddressOf {
                operand: Box::new(operand),
                mutability: is_mut,
                source_location: start_location,
            });
        }

        self.parse_postfix_expression()
    }

    /// Parse postfix expression: function calls, array indexing, field access
    fn parse_postfix_expression(&mut self) -> Result<Expression, ParserError> {
        let start_location = self.current_location();
        let mut expr = self.parse_atomic_expression()?;

        // Handle postfix operators: (args), [index], and .field
        loop {
            if self.check(&TokenType::LeftParen) {
                // Function call: expr(args)
                let args_with_labels = self.parse_argument_list()?;

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

                // Convert expressions to Argument structs
                let arguments: Vec<Argument> = args_with_labels
                    .into_iter()
                    .enumerate()
                    .map(|(i, (label, value))| Argument {
                        parameter_name: Identifier::new(
                            label.unwrap_or_else(|| format!("arg_{}", i)),
                            start_location.clone(),
                        ),
                        value: Box::new(value),
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
                self.advance();
                let index = self.parse_expression()?;
                self.expect(&TokenType::RightBracket, "expected ']'")?;

                expr = Expression::ArrayAccess {
                    array: Box::new(expr),
                    index: Box::new(index),
                    source_location: start_location.clone(),
                };
            } else if self.check(&TokenType::Dot) {
                // Field access or method call: expr.field or expr.method(args)
                self.advance();
                let member_name = self.parse_identifier()?;

                // Check if this is a method call (followed by '(')
                if self.check(&TokenType::LeftParen) {
                    // Method call: expr.method(args)
                    let args_with_labels = self.parse_argument_list()?;

                    // Convert expressions to Argument structs
                    let arguments: Vec<Argument> = args_with_labels
                        .into_iter()
                        .enumerate()
                        .map(|(i, (label, value))| Argument {
                            parameter_name: Identifier::new(
                                label.unwrap_or_else(|| format!("arg_{}", i)),
                                start_location.clone(),
                            ),
                            value: Box::new(value),
                            source_location: start_location.clone(),
                        })
                        .collect();

                    expr = Expression::MethodCall {
                        receiver: Box::new(expr),
                        method_name: member_name,
                        arguments,
                        source_location: start_location.clone(),
                    };
                } else {
                    // Field access: expr.field
                    expr = Expression::FieldAccess {
                        instance: Box::new(expr),
                        field_name: member_name,
                        source_location: start_location.clone(),
                    };
                }
            } else {
                break;
            }
        }

        Ok(expr)
    }

    /// Parse atomic expression: literals, identifiers, parenthesized expressions
    fn parse_atomic_expression(&mut self) -> Result<Expression, ParserError> {
        let start_location = self.current_location();

        // Braced expression: { expr } (kept for backward compatibility)
        if self.check(&TokenType::LeftBrace) {
            if self.looks_like_map_literal() {
                return self.parse_map_literal();
            }
            return self.parse_braced_expression();
        }

        // Parenthesized expression or lambda: (x) => x + 1
        if self.check(&TokenType::LeftParen) {
            return self.parse_paren_expr_or_lambda();
        }

        // Array literal or closure with capture list: [x, y](params) => body
        if self.check(&TokenType::LeftBracket) {
            if self.looks_like_array_literal() {
                return self.parse_array_literal();
            } else {
                // Parse as capture list for closure
                let captures = self.parse_capture_list()?;
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

        // Identifier (variable reference, struct construction, enum variant)
        if let TokenType::Identifier(name) = &self.peek().token_type {
            let name = name.clone();
            self.advance();

            // Check for enum variant: EnumType::Variant
            if self.check(&TokenType::DoubleColon) {
                return self.parse_enum_variant_expression(name, start_location);
            }

            // Check for struct construction: TypeName { field: value, ... }
            if self.check(&TokenType::LeftBrace) && self.looks_like_struct_construction() {
                return self.parse_struct_construction(name, start_location);
            }

            return Ok(Expression::Variable {
                name: Identifier {
                    name,
                    source_location: start_location.clone(),
                },
                source_location: start_location,
            });
        }

        // Match expression
        if self.check_keyword(Keyword::Match) {
            return self.parse_match_expression();
        }

        Err(ParserError::UnexpectedToken {
            expected: "expression".to_string(),
            found: format!("{:?}", self.peek().token_type),
            location: start_location,
        })
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
                    let args_with_labels = self.parse_argument_list()?;

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
                    let arguments: Vec<Argument> = args_with_labels
                        .into_iter()
                        .enumerate()
                        .map(|(i, (label, value))| Argument {
                            parameter_name: Identifier::new(
                                label.unwrap_or_else(|| format!("arg_{}", i)),
                                start_location.clone(),
                            ),
                            value: Box::new(value),
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
                    let args_with_labels = self.parse_argument_list()?;

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

                    // Convert to Argument structs
                    let arguments: Vec<Argument> = args_with_labels
                        .into_iter()
                        .enumerate()
                        .map(|(i, (label, value))| Argument {
                            parameter_name: Identifier::new(
                                label.unwrap_or_else(|| format!("arg_{}", i)),
                                start_location.clone(),
                            ),
                            value: Box::new(value),
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
                            let args_with_labels = self.parse_argument_list()?;

                            // Convert expressions to Argument structs
                            let arguments: Vec<Argument> = args_with_labels
                                .into_iter()
                                .enumerate()
                                .map(|(i, (label, value))| Argument {
                                    parameter_name: Identifier::new(
                                        label.unwrap_or_else(|| format!("arg_{}", i)),
                                        start_location.clone(),
                                    ),
                                    value: Box::new(value),
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
        let element_type = if let Some(first) = elements.first() {
            match &**first {
                Expression::IntegerLiteral { source_location, .. } => Box::new(TypeSpecifier::Primitive {
                    type_name: PrimitiveType::Integer,
                    source_location: source_location.clone(),
                }),
                Expression::FloatLiteral { source_location, .. } => Box::new(TypeSpecifier::Primitive {
                    type_name: PrimitiveType::Float,
                    source_location: source_location.clone(),
                }),
                Expression::StringLiteral { source_location, .. } => Box::new(TypeSpecifier::Primitive {
                    type_name: PrimitiveType::String,
                    source_location: source_location.clone(),
                }),
                Expression::BooleanLiteral { source_location, .. } => Box::new(TypeSpecifier::Primitive {
                    type_name: PrimitiveType::Boolean,
                    source_location: source_location.clone(),
                }),
                _ => Box::new(TypeSpecifier::Primitive {
                    type_name: PrimitiveType::Integer,
                    source_location: start_location.clone(),
                }),
            }
        } else {
            Box::new(TypeSpecifier::Primitive {
                type_name: PrimitiveType::Integer,
                source_location: start_location.clone(),
            })
        };

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
        
        // Check for { Identifier : ... } or { StringLiteral : ... }
        if let Some(key_candidate) = self.peek_at(self.position + 1) {
            match &key_candidate.token_type {
                TokenType::Identifier(_) | TokenType::StringLiteral(_) => {
                    if let Some(colon_candidate) = self.peek_at(self.position + 2) {
                        if matches!(colon_candidate.token_type, TokenType::Colon) {
                            return true;
                        }
                    }
                },
                _ => { /* not an identifier or string literal, so not a simple map key */ }
            }
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
mod tests;
