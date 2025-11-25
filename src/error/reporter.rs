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

//! Error reporter with source context display
//! 
//! Provides detailed error messages with source code snippets

use crate::error::{ParserError, SemanticError, LexerError, SourceLocation};
use std::fs;
use std::collections::HashMap;

/// Error reporter that displays errors with source context
pub struct DetailedErrorReporter {
    /// Cache of loaded source files
    source_cache: HashMap<String, Vec<String>>,
}

impl DetailedErrorReporter {
    /// Create a new error reporter
    pub fn new() -> Self {
        Self {
            source_cache: HashMap::new(),
        }
    }

    /// Report a parser error with context
    pub fn report_parser_error(&mut self, error: &ParserError) {
        match error {
            ParserError::UnexpectedToken { found, expected, location } => {
                self.report_error_with_context(
                    "Unexpected token",
                    &format!("Expected {}, but found {}", expected, found),
                    location,
                    "error",
                );
            }
            ParserError::UnexpectedEof { expected } => {
                eprintln!("error: Unexpected end of file");
                eprintln!("  --> expected {}", expected);
            }
            ParserError::MalformedSExpression { reason, location } => {
                self.report_error_with_context(
                    "Malformed S-expression",
                    reason,
                    location,
                    "error",
                );
            }
            ParserError::MissingRequiredField { field, construct, location } => {
                self.report_error_with_context(
                    "Missing required field",
                    &format!("Field '{}' is required in {}", field, construct),
                    location,
                    "error",
                );
            }
            ParserError::InvalidConstruct { construct, location } => {
                self.report_error_with_context(
                    "Invalid construct",
                    &format!("'{}' is not valid here", construct),
                    location,
                    "error",
                );
            }
            ParserError::DuplicateField { field, construct, location } => {
                self.report_error_with_context(
                    "Duplicate field",
                    &format!("Field '{}' appears more than once in {}", field, construct),
                    location,
                    "error",
                );
            }
            ParserError::LexerError { source } => {
                self.report_lexer_error(source);
            }
            ParserError::Unimplemented { feature, location } => {
                self.report_error_with_context(
                    "Unimplemented feature",
                    &format!("Feature '{}' is not yet implemented", feature),
                    location,
                    "error",
                );
            }
            ParserError::SyntaxError { message, location, suggestion } => {
                self.report_error_with_context(
                    "Syntax error",
                    message,
                    location,
                    "error",
                );
                if let Some(hint) = suggestion {
                    eprintln!("  hint: {}", hint);
                }
            }
        }
    }

    /// Report a semantic error with context
    pub fn report_semantic_error(&mut self, error: &SemanticError) {
        match error {
            SemanticError::UndefinedSymbol { symbol, location } => {
                self.report_error_with_context(
                    "Undefined symbol",
                    &format!("'{}' is not defined", symbol),
                    location,
                    "error",
                );
            }
            SemanticError::TypeMismatch { expected, found, location } => {
                self.report_error_with_context(
                    "Type mismatch",
                    &format!("Expected type '{}', but found '{}'", expected, found),
                    location,
                    "error",
                );
            }
            SemanticError::DuplicateDefinition { symbol, location, previous_location } => {
                self.report_error_with_context(
                    "Duplicate definition",
                    &format!("'{}' is already defined", symbol),
                    location,
                    "error",
                );
                self.report_error_with_context(
                    "Previous definition",
                    &format!("'{}' was first defined here", symbol),
                    previous_location,
                    "note",
                );
            }
            _ => {
                eprintln!("error: {}", error);
            }
        }
    }

    /// Report a lexer error with context
    pub fn report_lexer_error(&mut self, error: &LexerError) {
        match error {
            LexerError::UnexpectedCharacter { character, location } => {
                self.report_error_with_context(
                    "Unexpected character",
                    &format!("Character '{}' is not valid here", character),
                    location,
                    "error",
                );
            }
            LexerError::UnterminatedString { location } => {
                self.report_error_with_context(
                    "Unterminated string",
                    "String literal is missing closing quote",
                    location,
                    "error",
                );
            }
            LexerError::InvalidEscapeSequence { sequence, location } => {
                self.report_error_with_context(
                    "Invalid escape sequence",
                    &format!("'\\{}' is not a valid escape sequence", sequence),
                    location,
                    "error",
                );
            }
            _ => {
                eprintln!("error: {}", error);
            }
        }
    }

    /// Report an error with source context
    fn report_error_with_context(
        &mut self,
        title: &str,
        message: &str,
        location: &SourceLocation,
        level: &str,
    ) {
        // Print error header
        eprintln!("{}: {}", level, title);
        eprintln!("  --> {}:{}:{}", location.file, location.line, location.column);
        eprintln!();

        // Try to load and display source context
        if let Ok(lines) = self.get_source_lines(&location.file) {
            let line_num = location.line;
            let col_num = location.column;
            
            // Calculate line number width for alignment
            let line_num_width = line_num.to_string().len().max(3);
            
            // Show context lines (1 before and 1 after if available)
            let start_line = line_num.saturating_sub(1).max(1);
            let end_line = (line_num + 1).min(lines.len());
            
            for i in start_line..=end_line {
                if i > lines.len() {
                    break;
                }
                
                let line = &lines[i - 1];
                let line_str = format!("{:>width$}", i, width = line_num_width);
                
                if i == line_num {
                    // Highlight the error line
                    eprintln!("{} |     {}", line_str, line);
                    
                    // Show caret pointing to error position
                    let padding = " ".repeat(line_num_width + 6 + col_num.saturating_sub(1));
                    let caret = "^".repeat(1); // Could calculate actual token length
                    eprintln!("{} {} {}", " ".repeat(line_num_width), "|", padding.clone() + &caret);
                    eprintln!("{} {} {} {}", " ".repeat(line_num_width), "|", padding, message);
                } else {
                    // Context lines
                    eprintln!("{} |     {}", line_str, line);
                }
            }
        } else {
            // Fallback if source is not available
            eprintln!("  {}", message);
        }
        eprintln!();
    }

    /// Get source lines from cache or load from file
    fn get_source_lines(&mut self, file_path: &str) -> Result<&Vec<String>, std::io::Error> {
        if !self.source_cache.contains_key(file_path) {
            let content = fs::read_to_string(file_path)?;
            let lines: Vec<String> = content.lines().map(|s| s.to_string()).collect();
            self.source_cache.insert(file_path.to_string(), lines);
        }
        Ok(self.source_cache.get(file_path).unwrap())
    }
}

impl Default for DetailedErrorReporter {
    fn default() -> Self {
        Self::new()
    }
}
