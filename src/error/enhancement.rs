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

//! Error Enhancement Module
//!
//! Converts traditional compiler errors into structured, LLM-friendly errors
//! with auto-fix suggestions and enhanced context.

use crate::error::structured::*;
use crate::error::{CompilerError, ParseError, SemanticError, SourceLocation, TypeError};
use std::collections::HashMap;
use std::fs;

/// Error enhancer that converts basic errors to structured errors
pub struct ErrorEnhancer {
    /// Error code generator
    code_generator: ErrorCodeGenerator,

    /// Pattern matcher for common errors
    patterns: ErrorPatternMatcher,

    /// Source file cache for snippets
    source_cache: HashMap<String, Vec<String>>,

    /// Configuration
    config: ErrorEnhancerConfig,
}

/// Configuration for error enhancement
#[derive(Debug, Clone)]
pub struct ErrorEnhancerConfig {
    /// Include code snippets
    pub include_snippets: bool,
    /// Maximum snippet lines
    pub snippet_context_lines: usize,
    /// Generate auto-fix suggestions
    pub generate_fixes: bool,
    /// Include learning hints
    pub include_learning_hints: bool,
}

impl Default for ErrorEnhancerConfig {
    fn default() -> Self {
        Self {
            include_snippets: true,
            snippet_context_lines: 3,
            generate_fixes: true,
            include_learning_hints: true,
        }
    }
}

impl ErrorEnhancer {
    pub fn new(config: ErrorEnhancerConfig) -> Self {
        Self {
            code_generator: ErrorCodeGenerator::new(),
            patterns: ErrorPatternMatcher::new(),
            source_cache: HashMap::new(),
            config,
        }
    }

    /// Enhance a compiler error
    pub fn enhance_error(&mut self, error: &CompilerError) -> StructuredError {
        match error {
            CompilerError::ParseError(e) => self.enhance_parse_error(e),
            CompilerError::TypeError(e) => self.enhance_type_error(e),
            CompilerError::SemanticError(e) => self.enhance_semantic_error(e),
            _ => self.enhance_generic_error(error),
        }
    }

    /// Enhance a parse error
    fn enhance_parse_error(&mut self, error: &ParseError) -> StructuredError {
        let code = self.code_generator.generate("PARSE");
        let location = self.create_location(&error.location);

        let mut builder = StructuredErrorBuilder::new()
            .error_code(code)
            .severity(ErrorSeverity::Error)
            .location(location)
            .message(error.message.clone())
            .explanation(self.explain_parse_error(error));

        // Add fixes based on error pattern
        if self.config.generate_fixes {
            let fixes = self.patterns.suggest_parse_fixes(error);
            for fix in fixes {
                builder = builder.add_fix(fix);
            }
        }

        // Add learning hints
        if self.config.include_learning_hints {
            builder = builder
                .add_learning_hint("Check parentheses balance and S-expression syntax".to_string());
        }

        builder
            .build()
            .unwrap_or_else(|_| panic!("Failed to build structured error"))
    }

    /// Enhance a type error
    fn enhance_type_error(&mut self, error: &TypeError) -> StructuredError {
        let code = self.code_generator.generate("TYPE");
        let location = self.create_location(&error.location);

        let mut builder = StructuredErrorBuilder::new()
            .error_code(code)
            .severity(ErrorSeverity::Error)
            .location(location)
            .message(format!(
                "Type mismatch: expected {}, found {}",
                error.expected, error.found
            ))
            .explanation(format!(
                "The expression has type '{}' but type '{}' was expected in this context",
                error.found, error.expected
            ));

        // Generate type conversion fixes
        if self.config.generate_fixes {
            let fixes = self.generate_type_conversion_fixes(&error.expected, &error.found);
            for fix in fixes {
                builder = builder.add_fix(fix);
            }
        }

        // Add pattern suggestion for common type errors
        if error.expected == "Boolean" && error.found.contains("Integer") {
            builder = builder.suggested_pattern("comparison_expression".to_string());
            builder = builder.add_learning_hint(
                "Use comparison operators (PREDICATE_EQUALS, PREDICATE_GREATER_THAN, etc.) to create Boolean expressions".to_string()
            );
        }

        builder.build().unwrap()
    }

    /// Enhance a semantic error
    fn enhance_semantic_error(&mut self, error: &SemanticError) -> StructuredError {
        let code = self.code_generator.generate("SEM");

        match error {
            SemanticError::UndefinedSymbol { symbol, location } => {
                self.enhance_undefined_symbol(code, symbol, location)
            }
            SemanticError::TypeMismatch {
                expected,
                found,
                location,
            } => self.enhance_type_mismatch_semantic(code, expected, found, location),
            SemanticError::UseBeforeInitialization { variable, location } => {
                self.enhance_uninitialized_use(code, variable, location)
            }
            SemanticError::AssignToImmutable { variable, location } => {
                self.enhance_immutable_assign(code, variable, location)
            }
            SemanticError::InvalidContract {
                contract_type,
                reason,
                location,
            } => self.enhance_contract_error(code, contract_type, reason, location),
            _ => self.enhance_generic_semantic_error(code, error),
        }
    }

    /// Enhance undefined symbol error
    fn enhance_undefined_symbol(
        &mut self,
        code: String,
        symbol: &str,
        location: &SourceLocation,
    ) -> StructuredError {
        let loc = self.create_location(location);

        let mut builder = StructuredErrorBuilder::new()
            .error_code(code)
            .severity(ErrorSeverity::Error)
            .location(loc)
            .message(format!("Undefined symbol '{}'", symbol))
            .explanation(format!(
                "The symbol '{}' has not been declared in the current scope",
                symbol
            ));

        // Generate fix suggestions
        if self.config.generate_fixes {
            // Suggest variable declaration
            builder = builder.add_fix(FixSuggestion {
                description: format!("Declare variable '{}' before use", symbol),
                confidence: 0.9,
                modifications: vec![CodeModification::AddCode {
                    before_line: Some(location.line),
                    after_line: None,
                    code: format!(
                        "(DECLARE_VARIABLE (NAME \"{}\") (TYPE INTEGER) (MUTABLE TRUE))",
                        symbol
                    ),
                    indent_level: 1,
                }],
                example: None,
                category: FixCategory::Addition,
            });

            // Suggest parameter addition if in function
            builder = builder.add_fix(FixSuggestion {
                description: format!("Add '{}' as a function parameter", symbol),
                confidence: 0.7,
                modifications: vec![],
                example: Some(format!(
                    "(ACCEPTS_PARAMETER (NAME \"{}\") (TYPE <type>))",
                    symbol
                )),
                category: FixCategory::Addition,
            });

            // Check for typos and suggest corrections
            if let Some(similar) = self.find_similar_symbol(symbol) {
                builder = builder.add_fix(FixSuggestion {
                    description: format!("Did you mean '{}'?", similar),
                    confidence: 0.8,
                    modifications: vec![CodeModification::ReplaceCode {
                        start_line: location.line,
                        start_column: location.column,
                        end_line: location.line,
                        end_column: location.column + symbol.len(),
                        new_code: similar.clone(),
                    }],
                    example: None,
                    category: FixCategory::Correction,
                });
            }
        }

        builder.build().unwrap()
    }

    /// Enhance uninitialized use error
    fn enhance_uninitialized_use(
        &mut self,
        code: String,
        variable: &str,
        location: &SourceLocation,
    ) -> StructuredError {
        let loc = self.create_location(location);

        StructuredErrorBuilder::new()
            .error_code(code)
            .severity(ErrorSeverity::Error)
            .location(loc)
            .message(format!("Use of uninitialized variable '{}'", variable))
            .explanation(format!("Variable '{}' is used before being assigned a value", variable))
            .add_fix(FixSuggestion {
                description: "Initialize variable at declaration".to_string(),
                confidence: 0.95,
                modifications: vec![],
                example: Some(format!(
                    "(DECLARE_VARIABLE (NAME \"{}\") (TYPE <type>) (MUTABLE TRUE) (INITIAL_VALUE <value>))",
                    variable
                )),
                category: FixCategory::Correction,
            })
            .add_learning_hint("All variables must be initialized before use in AetherScript".to_string())
            .build()
            .unwrap()
    }

    /// Enhance immutable assignment error
    fn enhance_immutable_assign(
        &mut self,
        code: String,
        variable: &str,
        location: &SourceLocation,
    ) -> StructuredError {
        let loc = self.create_location(location);

        StructuredErrorBuilder::new()
            .error_code(code)
            .severity(ErrorSeverity::Error)
            .location(loc)
            .message(format!("Cannot assign to immutable variable '{}'", variable))
            .explanation("This variable was declared as immutable and cannot be modified after initialization".to_string())
            .add_fix(FixSuggestion {
                description: "Make the variable mutable at declaration".to_string(),
                confidence: 0.9,
                modifications: vec![],
                example: Some(format!("(DECLARE_VARIABLE (NAME \"{}\") (TYPE <type>) (MUTABLE TRUE))", variable)),
                category: FixCategory::Correction,
            })
            .add_fix(FixSuggestion {
                description: "Create a new variable instead".to_string(),
                confidence: 0.7,
                modifications: vec![
                    CodeModification::ReplaceCode {
                        start_line: location.line,
                        start_column: location.column,
                        end_line: location.line,
                        end_column: location.column + variable.len(),
                        new_code: format!("{}_new", variable),
                    }
                ],
                example: None,
                category: FixCategory::Refactoring,
            })
            .build()
            .unwrap()
    }

    /// Enhance contract error
    fn enhance_contract_error(
        &mut self,
        code: String,
        contract_type: &str,
        reason: &str,
        location: &SourceLocation,
    ) -> StructuredError {
        let loc = self.create_location(location);

        let mut builder = StructuredErrorBuilder::new()
            .error_code(code)
            .severity(ErrorSeverity::Error)
            .location(loc)
            .message(format!("Invalid {} contract", contract_type))
            .explanation(reason.to_string());

        // Add contract-specific fixes
        match contract_type {
            "precondition" => {
                builder = builder.add_fix(FixSuggestion {
                    description: "Add PROOF_HINT to explain the precondition".to_string(),
                    confidence: 0.8,
                    modifications: vec![],
                    example: Some(
                        "(PROOF_HINT \"Explanation of why this condition is necessary\")"
                            .to_string(),
                    ),
                    category: FixCategory::Addition,
                });
            }
            "postcondition" => {
                builder = builder.add_fix(FixSuggestion {
                    description: "Use RETURNED_VALUE to reference the function result".to_string(),
                    confidence: 0.9,
                    modifications: vec![],
                    example: Some("(PREDICATE_EQUALS RETURNED_VALUE expected_value)".to_string()),
                    category: FixCategory::Correction,
                });
            }
            _ => {}
        }

        builder
            .add_learning_hint(
                "Contracts must be verifiable at compile time or runtime".to_string(),
            )
            .build()
            .unwrap()
    }

    /// Create enhanced location with code snippet
    fn create_location(&mut self, source_loc: &SourceLocation) -> ErrorLocation {
        let mut location = ErrorLocation::from(source_loc);

        if self.config.include_snippets {
            if let Some(snippet) = self.create_code_snippet(source_loc) {
                location.code_snippet = Some(snippet);
            }
        }

        location
    }

    /// Create code snippet around error
    fn create_code_snippet(&mut self, location: &SourceLocation) -> Option<CodeSnippet> {
        // Load source file if not cached
        if !self.source_cache.contains_key(&location.file) {
            if let Ok(content) = fs::read_to_string(&location.file) {
                let lines: Vec<String> = content.lines().map(|s| s.to_string()).collect();
                self.source_cache.insert(location.file.clone(), lines);
            }
        }

        // Get lines from cache
        let lines = self.source_cache.get(&location.file)?;

        // Calculate snippet range
        let start_line = location
            .line
            .saturating_sub(self.config.snippet_context_lines);
        let end_line = (location.line + self.config.snippet_context_lines).min(lines.len());

        // Extract snippet lines
        let mut snippet_lines = Vec::new();
        for line_num in start_line..=end_line {
            if line_num > 0 && line_num <= lines.len() {
                snippet_lines.push((line_num, lines[line_num - 1].clone()));
            }
        }

        // Create highlight for error location
        let highlights = vec![HighlightRange {
            start_line: location.line,
            start_column: location.column,
            end_line: location.line,
            end_column: location.column + 10, // Approximate
            style: HighlightStyle::Error,
        }];

        Some(CodeSnippet {
            lines: snippet_lines,
            highlights,
        })
    }

    /// Generate type conversion fixes
    fn generate_type_conversion_fixes(&self, expected: &str, found: &str) -> Vec<FixSuggestion> {
        let mut fixes = Vec::new();

        // Integer to Float conversion
        if expected == "Float" && found == "Integer" {
            fixes.push(FixSuggestion {
                description: "Convert integer to float".to_string(),
                confidence: 0.95,
                modifications: vec![],
                example: Some("(CAST_TO_TYPE value FLOAT)".to_string()),
                category: FixCategory::Correction,
            });
        }

        // String to Integer/Float parsing
        if found == "String" && (expected == "Integer" || expected == "Float") {
            fixes.push(FixSuggestion {
                description: format!("Parse string to {}", expected.to_lowercase()),
                confidence: 0.8,
                modifications: vec![],
                example: Some(format!(
                    "(CALL_FUNCTION (NAME \"parse_{}\") (ARGUMENT value))",
                    expected.to_lowercase()
                )),
                category: FixCategory::Correction,
            });
        }

        // Null handling
        if found.contains("Null") {
            fixes.push(FixSuggestion {
                description: "Add null check before use".to_string(),
                confidence: 0.9,
                modifications: vec![],
                example: Some("(IF_CONDITION (PREDICATE_NOT_NULL value) ...)".to_string()),
                category: FixCategory::Safety,
            });
        }

        fixes
    }

    /// Find similar symbol names (for typo correction)
    fn find_similar_symbol(&self, symbol: &str) -> Option<String> {
        // In a real implementation, this would check the symbol table
        // For now, return some common corrections
        let common_symbols = vec![
            ("lenght", "length"),
            ("widht", "width"),
            ("heigth", "height"),
            ("retrn", "return"),
            ("fucntion", "function"),
        ];

        for (typo, correct) in common_symbols {
            if symbol == typo {
                return Some(correct.to_string());
            }
        }

        None
    }

    /// Explain parse error in detail
    fn explain_parse_error(&self, error: &ParseError) -> String {
        if error.message.contains("expected ')'") {
            "S-expressions must have balanced parentheses. Each opening '(' needs a matching ')'"
                .to_string()
        } else if error.message.contains("unexpected token") {
            "The parser encountered a token that doesn't fit the expected syntax pattern"
                .to_string()
        } else {
            "The source code doesn't match AetherScript's S-expression syntax".to_string()
        }
    }

    /// Enhance type mismatch from semantic analysis
    fn enhance_type_mismatch_semantic(
        &mut self,
        code: String,
        expected: &str,
        found: &str,
        location: &SourceLocation,
    ) -> StructuredError {
        let loc = self.create_location(location);

        let mut builder = StructuredErrorBuilder::new()
            .error_code(code)
            .severity(ErrorSeverity::Error)
            .location(loc)
            .message(format!(
                "Type mismatch: expected {}, found {}",
                expected, found
            ))
            .explanation(format!(
                "The expression has type '{}' but type '{}' was required in this context",
                found, expected
            ));

        if self.config.generate_fixes {
            let fixes = self.generate_type_conversion_fixes(expected, found);
            for fix in fixes {
                builder = builder.add_fix(fix);
            }
        }

        builder.build().unwrap()
    }

    /// Enhance generic semantic error
    fn enhance_generic_semantic_error(
        &mut self,
        code: String,
        error: &SemanticError,
    ) -> StructuredError {
        let location = match error {
            SemanticError::DuplicateDefinition { location, .. } => location,
            SemanticError::InvalidType { location, .. } => location,
            SemanticError::ArgumentCountMismatch { location, .. } => location,
            SemanticError::InvalidOperation { location, .. } => location,
            _ => &SourceLocation::unknown(),
        };

        let loc = self.create_location(location);

        StructuredErrorBuilder::new()
            .error_code(code)
            .severity(ErrorSeverity::Error)
            .location(loc)
            .message(format!("{}", error))
            .explanation("See documentation for more details".to_string())
            .build()
            .unwrap()
    }

    /// Enhance generic compiler error
    fn enhance_generic_error(&mut self, error: &CompilerError) -> StructuredError {
        let code = self.code_generator.generate("GEN");

        StructuredErrorBuilder::new()
            .error_code(code)
            .severity(ErrorSeverity::Error)
            .location(ErrorLocation {
                file: "<unknown>".to_string(),
                line: 1,
                column: 1,
                end_line: None,
                end_column: None,
                code_snippet: None,
            })
            .message(format!("{}", error))
            .explanation("An error occurred during compilation".to_string())
            .build()
            .unwrap()
    }
}

/// Pattern matcher for common error patterns
struct ErrorPatternMatcher {
    parse_patterns: Vec<ParseErrorPattern>,
}

struct ParseErrorPattern {
    matcher: fn(&ParseError) -> bool,
    fixes: fn(&ParseError) -> Vec<FixSuggestion>,
}

impl ErrorPatternMatcher {
    fn new() -> Self {
        let mut parse_patterns = Vec::new();

        // Missing closing parenthesis
        parse_patterns.push(ParseErrorPattern {
            matcher: |e| e.message.contains("expected ')'"),
            fixes: |e| {
                vec![FixSuggestion {
                    description: "Add missing closing parenthesis".to_string(),
                    confidence: 0.9,
                    modifications: vec![CodeModification::AddCode {
                        before_line: None,
                        after_line: Some(e.location.line),
                        code: ")".to_string(),
                        indent_level: 0,
                    }],
                    example: None,
                    category: FixCategory::Correction,
                }]
            },
        });

        // Missing opening parenthesis
        parse_patterns.push(ParseErrorPattern {
            matcher: |e| e.message.contains("unexpected") && e.message.contains("expected '('"),
            fixes: |e| {
                vec![FixSuggestion {
                    description: "Add missing opening parenthesis".to_string(),
                    confidence: 0.9,
                    modifications: vec![CodeModification::AddCode {
                        before_line: Some(e.location.line),
                        after_line: None,
                        code: "(".to_string(),
                        indent_level: 0,
                    }],
                    example: None,
                    category: FixCategory::Correction,
                }]
            },
        });

        Self { parse_patterns }
    }

    fn suggest_parse_fixes(&self, error: &ParseError) -> Vec<FixSuggestion> {
        for pattern in &self.parse_patterns {
            if (pattern.matcher)(error) {
                return (pattern.fixes)(error);
            }
        }
        Vec::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_enhancement() {
        let mut enhancer = ErrorEnhancer::new(ErrorEnhancerConfig::default());

        let semantic_error = SemanticError::UndefinedSymbol {
            symbol: "foo".to_string(),
            location: SourceLocation {
                file: "test.aether".to_string(),
                line: 10,
                column: 5,
                offset: 0,
            },
        };

        let structured = enhancer.enhance_error(&CompilerError::SemanticError(semantic_error));

        assert!(structured.error_code.starts_with("SEM-"));
        assert_eq!(structured.message, "Undefined symbol 'foo'");
        assert!(!structured.fix_suggestions.is_empty());
    }

    #[test]
    fn test_type_error_enhancement() {
        let mut enhancer = ErrorEnhancer::new(ErrorEnhancerConfig::default());

        let type_error = TypeError {
            expected: "Float".to_string(),
            found: "Integer".to_string(),
            location: SourceLocation {
                file: "math.aether".to_string(),
                line: 20,
                column: 10,
                offset: 0,
            },
        };

        let structured = enhancer.enhance_error(&CompilerError::TypeError(type_error));

        assert!(structured.error_code.starts_with("TYPE-"));
        assert!(structured.message.contains("Type mismatch"));

        // Should have type conversion fix
        let has_cast_fix = structured.fix_suggestions.iter().any(|fix| {
            fix.example
                .as_ref()
                .map_or(false, |ex| ex.contains("CAST_TO_TYPE"))
        });
        assert!(has_cast_fix);
    }
}
