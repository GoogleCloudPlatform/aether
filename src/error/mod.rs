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

//! Error handling for AetherScript compiler
//! 
//! Comprehensive error types and reporting
//! Now enhanced with LLM-first structured errors and auto-fix suggestions

mod reporter;
pub use reporter::DetailedErrorReporter;

pub mod structured;
pub mod enhancement;
pub mod intent_analysis;

use std::fmt;
use thiserror::Error;

/// Source location information for error reporting
#[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct SourceLocation {
    pub file: String,
    pub line: usize,
    pub column: usize,
    pub offset: usize,
}

impl SourceLocation {
    pub fn new(file: String, line: usize, column: usize, offset: usize) -> Self {
        Self {
            file,
            line,
            column,
            offset,
        }
    }

    pub fn unknown() -> Self {
        Self {
            file: "<unknown>".to_string(),
            line: 0,
            column: 0,
            offset: 0,
        }
    }
}

impl fmt::Display for SourceLocation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}:{}", self.file, self.line, self.column)
    }
}

/// Source span covering a range of characters
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceSpan {
    pub start: SourceLocation,
    pub end: SourceLocation,
}

impl SourceSpan {
    pub fn new(start: SourceLocation, end: SourceLocation) -> Self {
        Self { start, end }
    }

    pub fn single(location: SourceLocation) -> Self {
        Self {
            start: location.clone(),
            end: location,
        }
    }
}

/// Lexical analysis errors
#[derive(Error, Debug, Clone)]
pub enum LexerError {
    #[error("Unexpected character '{character}' at {location}")]
    UnexpectedCharacter {
        character: char,
        location: SourceLocation,
    },

    #[error("Unterminated string literal at {location}")]
    UnterminatedString { location: SourceLocation },

    #[error("Invalid escape sequence '\\{sequence}' at {location}")]
    InvalidEscapeSequence {
        sequence: String,
        location: SourceLocation,
    },

    #[error("Invalid number format '{value}' at {location}")]
    InvalidNumber {
        value: String,
        location: SourceLocation,
    },

    #[error("Invalid Unicode character at {location}")]
    InvalidUnicode { location: SourceLocation },

    #[error("Maximum nesting depth exceeded at {location}")]
    MaxNestingDepthExceeded { location: SourceLocation },
}

/// Parsing errors
#[derive(Error, Debug, Clone)]
pub enum ParserError {
    #[error("Unexpected token {found:?}, expected {expected} at {location}")]
    UnexpectedToken {
        found: String,
        expected: String,
        location: SourceLocation,
    },

    #[error("Unexpected end of file, expected {expected}")]
    UnexpectedEof { expected: String },

    #[error("Malformed S-expression at {location}: {reason}")]
    MalformedSExpression {
        reason: String,
        location: SourceLocation,
    },

    #[error("Missing required field '{field}' in {construct} at {location}")]
    MissingRequiredField {
        field: String,
        construct: String,
        location: SourceLocation,
    },

    #[error("Invalid construct '{construct}' at {location}")]
    InvalidConstruct {
        construct: String,
        location: SourceLocation,
    },
    
    #[error("Unimplemented feature '{feature}' at {location}")]
    Unimplemented {
        feature: String,
        location: SourceLocation,
    },

    #[error("Duplicate field '{field}' in {construct} at {location}")]
    DuplicateField {
        field: String,
        construct: String,
        location: SourceLocation,
    },

    #[error("Lexer error: {source}")]
    LexerError {
        #[from]
        source: LexerError,
    },
}

/// Semantic analysis errors
#[derive(Error, Debug, Clone)]
pub enum SemanticError {
    #[error("Undefined symbol '{symbol}' at {location}")]
    UndefinedSymbol {
        symbol: String,
        location: SourceLocation,
    },

    #[error("Type mismatch: expected {expected}, found {found} at {location}")]
    TypeMismatch {
        expected: String,
        found: String,
        location: SourceLocation,
    },

    #[error("Cannot assign to immutable variable '{variable}' at {location}")]
    AssignToImmutable {
        variable: String,
        location: SourceLocation,
    },

    #[error("Variable '{variable}' used before initialization at {location}")]
    UseBeforeInitialization {
        variable: String,
        location: SourceLocation,
    },

    #[error("Variable '{variable}' used after move at {location}")]
    UseAfterMove {
        variable: String,
        location: SourceLocation,
    },

    #[error("Function '{function}' called with wrong number of arguments: expected {expected}, got {found} at {location}")]
    ArgumentCountMismatch {
        function: String,
        expected: usize,
        found: usize,
        location: SourceLocation,
    },

    #[error("Duplicate definition of '{symbol}' at {location} (previously defined at {previous_location})")]
    DuplicateDefinition {
        symbol: String,
        location: SourceLocation,
        previous_location: SourceLocation,
    },

    #[error("Circular dependency detected involving module '{module}' at {location}")]
    CircularDependency {
        module: String,
        location: SourceLocation,
    },

    #[error("Contract violation: {condition} at {location}")]
    ContractViolation {
        condition: String,
        location: SourceLocation,
    },

    #[error("Cannot export private symbol '{symbol}' at {location}")]
    ExportPrivateSymbol {
        symbol: String,
        location: SourceLocation,
    },

    #[error("Constraint violation: {constraint} cannot be satisfied by type '{found_type}' at {location}")]
    ConstraintViolation {
        constraint: String,
        found_type: String,
        location: SourceLocation,
    },

    #[error("Generic type instantiation error for '{base_type}': expected {expected_args} type arguments, found {found_args} at {location}")]
    GenericInstantiationError {
        base_type: String,
        expected_args: usize,
        found_args: usize,
        location: SourceLocation,
    },

    #[error("Unsupported feature: {feature} at {location}")]
    UnsupportedFeature {
        feature: String,
        location: SourceLocation,
    },

    #[error("Code generation error: {message}")]
    CodeGenError {
        message: String,
    },

    #[error("Invalid contract {contract_type}: {reason} at {location}")]
    InvalidContract {
        contract_type: String,
        reason: String,
        location: SourceLocation,
    },

    #[error("Invalid type '{type_name}': {reason} at {location}")]
    InvalidType {
        type_name: String,
        reason: String,
        location: SourceLocation,
    },

    #[error("Invalid FFI declaration: {message} at {location}")]
    InvalidFFI {
        message: String,
        location: SourceLocation,
    },

    #[error("Internal semantic error: {message}")]
    Internal {
        message: String,
    },

    #[error("Verification error: {message} at {location}")]
    VerificationError {
        message: String,
        location: SourceLocation,
    },

    #[error("Feature not implemented: {feature} at {location}")]
    NotImplemented {
        feature: String,
        location: SourceLocation,
    },
    
    #[error("IO error: {message}")]
    IoError {
        message: String,
    },

    #[error("Import error for module '{module}': {reason} at {location}")]
    ImportError {
        module: String,
        reason: String,
        location: SourceLocation,
    },

    #[error("Duplicate catch clause for exception type '{exception_type}' at {location}")]
    DuplicateCatchClause {
        exception_type: String,
        location: SourceLocation,
    },

    #[error("Invalid operation '{operation}': {reason} at {location}")]
    InvalidOperation {
        operation: String,
        reason: String,
        location: SourceLocation,
    },
    
    #[error("Resource leak detected: {resource_type} '{binding}' not released at {location}")]
    ResourceLeak {
        resource_type: String,
        binding: String,
        location: SourceLocation,
    },
    
    #[error("Missing field '{field_name}' in struct '{struct_name}' at {location}")]
    MissingField {
        struct_name: String,
        field_name: String,
        location: SourceLocation,
    },
    
    #[error("Unknown field '{field_name}' in struct '{struct_name}' at {location}")]
    UnknownField {
        struct_name: String,
        field_name: String,
        location: SourceLocation,
    },
    
    #[error("Internal error: {message} at {location}")]
    InternalError {
        message: String,
        location: SourceLocation,
    },
    
    #[error("Malformed construct '{construct}': {reason} at {location}")]
    MalformedConstruct {
        construct: String,
        reason: String,
        location: SourceLocation,
    },
    
    #[error("Missing value for enum variant '{variant}' of type '{enum_name}' at {location}")]
    MissingEnumVariantValue {
        variant: String,
        enum_name: String,
        location: SourceLocation,
    },
    
    #[error("Unexpected value for enum variant '{variant}' of type '{enum_name}' which has no associated type at {location}")]
    UnexpectedEnumVariantValue {
        variant: String,
        enum_name: String,
        location: SourceLocation,
    },
}

impl From<std::io::Error> for SemanticError {
    fn from(err: std::io::Error) -> Self {
        SemanticError::IoError {
            message: err.to_string(),
        }
    }
}

/// Code generation errors
#[derive(Error, Debug, Clone)]
pub enum CodegenError {
    #[error("LLVM error: {message}")]
    LLVMError { message: String },

    #[error("Unsupported target architecture: {arch}")]
    UnsupportedArchitecture { arch: String },

    #[error("FFI type mapping failed for type '{type_name}': {reason}")]
    FFITypeMappingFailed {
        type_name: String,
        reason: String,
    },

    #[error("Link error: {message}")]
    LinkError { message: String },

    #[error("Optimization failed: {message}")]
    OptimizationFailed { message: String },
}

/// Compiler errors encompassing all phases
#[derive(Error, Debug, Clone)]
pub enum CompilerError {
    #[error("Lexer error: {source}")]
    Lexer {
        #[from]
        source: LexerError,
    },

    #[error("Parser error: {source}")]
    Parser {
        #[from]
        source: ParserError,
    },

    #[error("Semantic error: {source}")]
    Semantic {
        #[from]
        source: SemanticError,
    },

    #[error("Code generation error: {source}")]
    Codegen {
        #[from]
        source: CodegenError,
    },

    #[error("I/O error: {message}")]
    IoError { message: String },

    #[error("Internal compiler error: {message}")]
    Internal { message: String },
    
    // Wrapper types for new error system compatibility
    #[error("Parse error: {0}")]
    ParseError(ParseError),
    
    #[error("Type error: {0}")]
    TypeError(TypeError),
    
    #[error("Semantic error occurred")]
    SemanticError(SemanticError),
}

impl From<std::io::Error> for CompilerError {
    fn from(err: std::io::Error) -> Self {
        CompilerError::IoError {
            message: err.to_string(),
        }
    }
}

/// Parse error for structured error conversion
#[derive(Error, Debug, Clone)]
#[error("{message} at {location}")]
pub struct ParseError {
    pub message: String,
    pub location: SourceLocation,
}

/// Type error for structured error conversion  
#[derive(Error, Debug, Clone)]
#[error("Type mismatch: expected {expected}, found {found} at {location}")]
pub struct TypeError {
    pub expected: String,
    pub found: String,
    pub location: SourceLocation,
}

/// Diagnostic severity levels
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Severity {
    Error,
    Warning,
    Info,
    Hint,
}

impl fmt::Display for Severity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Severity::Error => write!(f, "error"),
            Severity::Warning => write!(f, "warning"),
            Severity::Info => write!(f, "info"),
            Severity::Hint => write!(f, "hint"),
        }
    }
}

/// A diagnostic message with location and severity
#[derive(Debug, Clone)]
pub struct Diagnostic {
    pub severity: Severity,
    pub message: String,
    pub location: Option<SourceSpan>,
    pub help: Option<String>,
    pub note: Option<String>,
}

impl Diagnostic {
    pub fn error(message: String, location: Option<SourceSpan>) -> Self {
        Self {
            severity: Severity::Error,
            message,
            location,
            help: None,
            note: None,
        }
    }

    pub fn warning(message: String, location: Option<SourceSpan>) -> Self {
        Self {
            severity: Severity::Warning,
            message,
            location,
            help: None,
            note: None,
        }
    }

    pub fn with_help(mut self, help: String) -> Self {
        self.help = Some(help);
        self
    }

    pub fn with_note(mut self, note: String) -> Self {
        self.note = Some(note);
        self
    }
}

/// Error reporter for displaying diagnostics
pub struct ErrorReporter {
    use_colors: bool,
}

impl ErrorReporter {
    pub fn new(use_colors: bool) -> Self {
        Self { use_colors }
    }

    pub fn report_diagnostic(&self, diagnostic: &Diagnostic) {
        let severity_color = if self.use_colors {
            match diagnostic.severity {
                Severity::Error => "\x1b[31m",   // Red
                Severity::Warning => "\x1b[33m", // Yellow
                Severity::Info => "\x1b[34m",    // Blue
                Severity::Hint => "\x1b[36m",    // Cyan
            }
        } else {
            ""
        };

        let reset_color = if self.use_colors { "\x1b[0m" } else { "" };

        if let Some(span) = &diagnostic.location {
            eprintln!(
                "{}{}:{}{} {}{}",
                severity_color,
                diagnostic.severity,
                reset_color,
                span.start,
                diagnostic.message,
                reset_color
            );
        } else {
            eprintln!(
                "{}{}: {}",
                severity_color, diagnostic.severity, reset_color
            );
        }

        if let Some(help) = &diagnostic.help {
            eprintln!("  {} help: {}", if self.use_colors { "\x1b[32m=" } else { "=" }, help);
        }

        if let Some(note) = &diagnostic.note {
            eprintln!("  {} note: {}", if self.use_colors { "\x1b[36m=" } else { "=" }, note);
        }

        if self.use_colors {
            eprint!("\x1b[0m");
        }
    }

    pub fn report_error(&self, error: &CompilerError) {
        let diagnostic = match error {
            CompilerError::Lexer { source } => self.lexer_error_to_diagnostic(source),
            CompilerError::Parser { source } => self.parser_error_to_diagnostic(source),
            CompilerError::Semantic { source } => self.semantic_error_to_diagnostic(source),
            CompilerError::Codegen { source } => self.codegen_error_to_diagnostic(source),
            CompilerError::IoError { message } => Diagnostic::error(
                message.clone(),
                None,
            ),
            CompilerError::Internal { message } => Diagnostic::error(
                format!("Internal compiler error: {}", message),
                None,
            ).with_note("This is a bug in the compiler. Please report it.".to_string()),
            CompilerError::ParseError(e) => Diagnostic::error(
                format!("Parse error: {}", e),
                None,
            ),
            CompilerError::TypeError(e) => Diagnostic::error(
                format!("Type error: {}", e),
                None,
            ),
            CompilerError::SemanticError(e) => self.semantic_error_to_diagnostic(&e),
        };

        self.report_diagnostic(&diagnostic);
    }

    fn lexer_error_to_diagnostic(&self, error: &LexerError) -> Diagnostic {
        match error {
            LexerError::UnexpectedCharacter { character, location } => {
                Diagnostic::error(
                    format!("Unexpected character '{}'", character),
                    Some(SourceSpan::single(location.clone())),
                )
            }
            LexerError::UnterminatedString { location } => {
                Diagnostic::error(
                    "Unterminated string literal".to_string(),
                    Some(SourceSpan::single(location.clone())),
                ).with_help("Add closing quote '\"' to terminate the string".to_string())
            }
            LexerError::InvalidEscapeSequence { sequence, location } => {
                Diagnostic::error(
                    format!("Invalid escape sequence '\\{}'", sequence),
                    Some(SourceSpan::single(location.clone())),
                ).with_help("Valid escape sequences are: \\n, \\t, \\\\, \\\"".to_string())
            }
            _ => Diagnostic::error(error.to_string(), None),
        }
    }

    fn parser_error_to_diagnostic(&self, error: &ParserError) -> Diagnostic {
        match error {
            ParserError::UnexpectedToken { found, expected, location } => {
                Diagnostic::error(
                    format!("Unexpected token '{}', expected {}", found, expected),
                    Some(SourceSpan::single(location.clone())),
                )
            }
            ParserError::MissingRequiredField { field, construct, location } => {
                Diagnostic::error(
                    format!("Missing required field '{}' in {}", field, construct),
                    Some(SourceSpan::single(location.clone())),
                ).with_help(format!("Add the '{}' field to the {} construct", field, construct))
            }
            _ => Diagnostic::error(error.to_string(), None),
        }
    }

    fn semantic_error_to_diagnostic(&self, error: &SemanticError) -> Diagnostic {
        match error {
            SemanticError::UndefinedSymbol { symbol, location } => {
                Diagnostic::error(
                    format!("Undefined symbol '{}'", symbol),
                    Some(SourceSpan::single(location.clone())),
                ).with_help("Check the spelling or ensure the symbol is declared".to_string())
            }
            SemanticError::TypeMismatch { expected, found, location } => {
                Diagnostic::error(
                    format!("Type mismatch: expected {}, found {}", expected, found),
                    Some(SourceSpan::single(location.clone())),
                ).with_help("Consider using explicit type conversion".to_string())
            }
            _ => Diagnostic::error(error.to_string(), None),
        }
    }

    fn codegen_error_to_diagnostic(&self, error: &CodegenError) -> Diagnostic {
        Diagnostic::error(error.to_string(), None)
    }
}

/// Error recovery strategies for the parser
pub struct ErrorRecovery;

impl ErrorRecovery {
    /// Skip tokens until a synchronization point is found
    pub fn synchronize_at_s_expression() {
        // Implementation will be added in parser task
    }

    /// Suggest corrections for common typos
    pub fn suggest_correction(input: &str, candidates: &[&str]) -> Option<String> {
        candidates
            .iter()
            .min_by_key(|candidate| edit_distance(input, candidate))
            .filter(|candidate| edit_distance(input, candidate) <= 2)
            .map(|s| s.to_string())
    }
}

/// Simple edit distance calculation for suggestions
fn edit_distance(a: &str, b: &str) -> usize {
    let a_chars: Vec<char> = a.chars().collect();
    let b_chars: Vec<char> = b.chars().collect();
    let a_len = a_chars.len();
    let b_len = b_chars.len();

    let mut dp = vec![vec![0; b_len + 1]; a_len + 1];

    // Initialize base cases
    for (i, row) in dp.iter_mut().enumerate().take(a_len + 1) {
        row[0] = i;
    }
    for j in 0..=b_len {
        dp[0][j] = j;
    }

    // Fill the DP table
    for i in 1..=a_len {
        for j in 1..=b_len {
            if a_chars[i - 1] == b_chars[j - 1] {
                dp[i][j] = dp[i - 1][j - 1];
            } else {
                dp[i][j] = 1 + std::cmp::min(
                    std::cmp::min(dp[i - 1][j], dp[i][j - 1]),
                    dp[i - 1][j - 1],
                );
            }
        }
    }

    dp[a_len][b_len]
}

/// Implement From for Vec<SemanticError> to CompilerError
impl From<Vec<SemanticError>> for CompilerError {
    fn from(errors: Vec<SemanticError>) -> Self {
        if errors.is_empty() {
            CompilerError::Internal {
                message: "Empty error list provided".to_string(),
            }
        } else if errors.len() == 1 {
            CompilerError::Semantic {
                source: errors.into_iter().next().unwrap(),
            }
        } else {
            // Combine multiple errors into a single message
            let messages: Vec<String> = errors.iter().map(|e| e.to_string()).collect();
            CompilerError::Internal {
                message: format!("Multiple semantic errors:\n{}", messages.join("\n")),
            }
        }
    }
}

/// Implement From for String to CompilerError
impl From<String> for CompilerError {
    fn from(message: String) -> Self {
        CompilerError::Internal { message }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_source_location_display() {
        let loc = SourceLocation::new("test.aether".to_string(), 10, 5, 100);
        assert_eq!(loc.to_string(), "test.aether:10:5");
    }

    #[test]
    fn test_edit_distance() {
        assert_eq!(edit_distance("hello", "hello"), 0);
        assert_eq!(edit_distance("hello", "helo"), 1);
        assert_eq!(edit_distance("hello", "world"), 4);
        assert_eq!(edit_distance("", "test"), 4);
    }

    #[test]
    fn test_error_suggestion() {
        let candidates = &["DEFINE_FUNCTION", "DEFINE_MODULE", "DECLARE_VARIABLE"];
        assert_eq!(
            ErrorRecovery::suggest_correction("DEFINE_FUNCTIO", candidates),
            Some("DEFINE_FUNCTION".to_string())
        );
        assert_eq!(
            ErrorRecovery::suggest_correction("xyz", candidates),
            None
        );
    }

    #[test]
    fn test_diagnostic_creation() {
        let loc = SourceLocation::new("test.aether".to_string(), 1, 1, 0);
        let span = SourceSpan::single(loc);
        let diag = Diagnostic::error("Test error".to_string(), Some(span))
            .with_help("This is helpful".to_string())
            .with_note("This is a note".to_string());

        assert_eq!(diag.severity, Severity::Error);
        assert_eq!(diag.message, "Test error");
        assert!(diag.help.is_some());
        assert!(diag.note.is_some());
    }
}
