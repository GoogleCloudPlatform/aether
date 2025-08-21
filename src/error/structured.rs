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

//! Structured Error System for LLM-First Language
//! 
//! This module implements the enhanced error system that provides structured,
//! machine-readable errors with auto-fix suggestions optimized for LLM consumption.

use crate::error::{CompilerError, SourceLocation};
use crate::ast::{Expression, Statement, TypeSpecifier};
use serde::{Serialize, Deserialize};
use std::collections::HashMap;

/// Structured compilation error with LLM-friendly format
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StructuredError {
    /// Unique error code (e.g., "SEM-001", "TYPE-042")
    pub error_code: String,
    
    /// Error severity
    pub severity: ErrorSeverity,
    
    /// Source location
    pub location: ErrorLocation,
    
    /// Human-readable error message
    pub message: String,
    
    /// Detailed explanation for LLM understanding
    pub explanation: String,
    
    /// Auto-fix suggestions
    pub fix_suggestions: Vec<FixSuggestion>,
    
    /// Related errors or warnings
    pub related: Vec<RelatedDiagnostic>,
    
    /// Additional context for LLM
    pub llm_context: LLMContext,
}

/// Error severity levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ErrorSeverity {
    /// Fatal error - compilation cannot continue
    Fatal,
    /// Error - must be fixed for successful compilation
    Error,
    /// Warning - should be addressed but doesn't block compilation
    Warning,
    /// Info - informational message
    Info,
    /// Hint - suggestion for improvement
    Hint,
}

/// Enhanced location information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorLocation {
    /// File path
    pub file: String,
    /// Line number (1-indexed)
    pub line: usize,
    /// Column number (1-indexed)
    pub column: usize,
    /// End line (for multi-line errors)
    pub end_line: Option<usize>,
    /// End column
    pub end_column: Option<usize>,
    /// Snippet of code around the error
    pub code_snippet: Option<CodeSnippet>,
}

/// Code snippet for context
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeSnippet {
    /// Lines of code (with line numbers)
    pub lines: Vec<(usize, String)>,
    /// Highlight ranges within the snippet
    pub highlights: Vec<HighlightRange>,
}

/// Range to highlight in code snippet
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HighlightRange {
    pub start_line: usize,
    pub start_column: usize,
    pub end_line: usize,
    pub end_column: usize,
    pub style: HighlightStyle,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum HighlightStyle {
    Error,
    Warning,
    Info,
    Suggestion,
}

/// Auto-fix suggestion
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FixSuggestion {
    /// Description of the fix
    pub description: String,
    
    /// Confidence level (0.0 to 1.0)
    pub confidence: f32,
    
    /// Code modifications to apply
    pub modifications: Vec<CodeModification>,
    
    /// Example of fixed code
    pub example: Option<String>,
    
    /// Category of fix
    pub category: FixCategory,
}

/// Code modification instruction
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CodeModification {
    /// Add code at a specific location
    AddCode {
        before_line: Option<usize>,
        after_line: Option<usize>,
        code: String,
        indent_level: usize,
    },
    
    /// Replace code range
    ReplaceCode {
        start_line: usize,
        start_column: usize,
        end_line: usize,
        end_column: usize,
        new_code: String,
    },
    
    /// Remove code range
    RemoveCode {
        start_line: usize,
        start_column: usize,
        end_line: usize,
        end_column: usize,
    },
    
    /// Add import/include
    AddImport {
        module: String,
        symbols: Vec<String>,
    },
    
    /// Wrap code in a construct
    WrapCode {
        start_line: usize,
        end_line: usize,
        wrapper_type: WrapperType,
        parameters: HashMap<String, String>,
    },
}

/// Types of code wrappers
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WrapperType {
    TryCatch,
    IfCondition,
    WhileLoop,
    Function,
    ResourceScope,
}

/// Category of fix
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum FixCategory {
    /// Add missing code
    Addition,
    /// Fix incorrect code
    Correction,
    /// Remove unnecessary code
    Removal,
    /// Refactor for better style
    Refactoring,
    /// Add safety checks
    Safety,
    /// Performance improvement
    Performance,
}

/// Related diagnostic information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RelatedDiagnostic {
    pub location: ErrorLocation,
    pub message: String,
    pub relationship: DiagnosticRelationship,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum DiagnosticRelationship {
    /// This is the cause of the main error
    Cause,
    /// This is affected by the main error
    Consequence,
    /// Similar error elsewhere
    Similar,
    /// Previous definition/declaration
    PreviousDefinition,
    /// Suggested alternative
    Alternative,
}

/// LLM-specific context
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LLMContext {
    /// Intent mismatch information
    pub intent_mismatch: Option<IntentMismatch>,
    
    /// Pattern that could be applied
    pub suggested_pattern: Option<String>,
    
    /// Relevant documentation links
    pub documentation: Vec<String>,
    
    /// Learning hints for the LLM
    pub learning_hints: Vec<String>,
    
    /// Verification status
    pub verification_status: Option<VerificationStatus>,
}

/// Intent mismatch details
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntentMismatch {
    pub stated_intent: String,
    pub detected_behavior: String,
    pub confidence: f32,
    pub evidence: Vec<String>,
}

/// Verification status for contracts
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerificationStatus {
    pub contract_type: String,
    pub verification_result: String,
    pub counterexample: Option<String>,
}

/// Error code generator
pub struct ErrorCodeGenerator {
    next_codes: HashMap<String, usize>,
}

impl ErrorCodeGenerator {
    pub fn new() -> Self {
        Self {
            next_codes: HashMap::new(),
        }
    }
    
    /// Generate a unique error code for a category
    pub fn generate(&mut self, category: &str) -> String {
        let counter = self.next_codes.entry(category.to_string()).or_insert(1);
        let code = format!("{}-{:03}", category, counter);
        *counter += 1;
        code
    }
}

/// Structured error builder
pub struct StructuredErrorBuilder {
    error_code: Option<String>,
    severity: ErrorSeverity,
    location: Option<ErrorLocation>,
    message: Option<String>,
    explanation: Option<String>,
    fix_suggestions: Vec<FixSuggestion>,
    related: Vec<RelatedDiagnostic>,
    llm_context: LLMContext,
}

impl StructuredErrorBuilder {
    pub fn new() -> Self {
        Self {
            error_code: None,
            severity: ErrorSeverity::Error,
            location: None,
            message: None,
            explanation: None,
            fix_suggestions: Vec::new(),
            related: Vec::new(),
            llm_context: LLMContext {
                intent_mismatch: None,
                suggested_pattern: None,
                documentation: Vec::new(),
                learning_hints: Vec::new(),
                verification_status: None,
            },
        }
    }
    
    pub fn error_code(mut self, code: String) -> Self {
        self.error_code = Some(code);
        self
    }
    
    pub fn severity(mut self, severity: ErrorSeverity) -> Self {
        self.severity = severity;
        self
    }
    
    pub fn location(mut self, location: ErrorLocation) -> Self {
        self.location = Some(location);
        self
    }
    
    pub fn message(mut self, message: String) -> Self {
        self.message = Some(message);
        self
    }
    
    pub fn explanation(mut self, explanation: String) -> Self {
        self.explanation = Some(explanation);
        self
    }
    
    pub fn add_fix(mut self, fix: FixSuggestion) -> Self {
        self.fix_suggestions.push(fix);
        self
    }
    
    pub fn add_related(mut self, related: RelatedDiagnostic) -> Self {
        self.related.push(related);
        self
    }
    
    pub fn intent_mismatch(mut self, mismatch: IntentMismatch) -> Self {
        self.llm_context.intent_mismatch = Some(mismatch);
        self
    }
    
    pub fn suggested_pattern(mut self, pattern: String) -> Self {
        self.llm_context.suggested_pattern = Some(pattern);
        self
    }
    
    pub fn add_documentation(mut self, doc: String) -> Self {
        self.llm_context.documentation.push(doc);
        self
    }
    
    pub fn add_learning_hint(mut self, hint: String) -> Self {
        self.llm_context.learning_hints.push(hint);
        self
    }
    
    pub fn build(self) -> Result<StructuredError, String> {
        Ok(StructuredError {
            error_code: self.error_code.ok_or("Missing error code")?,
            severity: self.severity,
            location: self.location.ok_or("Missing location")?,
            message: self.message.ok_or("Missing message")?,
            explanation: self.explanation.unwrap_or_default(),
            fix_suggestions: self.fix_suggestions,
            related: self.related,
            llm_context: self.llm_context,
        })
    }
}

/// Convert source location to enhanced error location
impl From<&SourceLocation> for ErrorLocation {
    fn from(loc: &SourceLocation) -> Self {
        ErrorLocation {
            file: loc.file.clone(),
            line: loc.line,
            column: loc.column,
            end_line: None,
            end_column: None,
            code_snippet: None,
        }
    }
}

/// Common error patterns with auto-fix suggestions
pub struct ErrorPatterns {
    patterns: HashMap<String, ErrorPattern>,
}

#[derive(Clone)]
struct ErrorPattern {
    code_prefix: String,
    matcher: fn(&CompilerError) -> bool,
    fixer: fn(&CompilerError) -> Vec<FixSuggestion>,
}

impl ErrorPatterns {
    pub fn new() -> Self {
        let mut patterns = HashMap::new();
        
        // Register common patterns
        patterns.insert("type_mismatch".to_string(), ErrorPattern {
            code_prefix: "TYPE".to_string(),
            matcher: |e| matches!(e, CompilerError::TypeError(_)),
            fixer: Self::fix_type_mismatch,
        });
        
        patterns.insert("undefined_symbol".to_string(), ErrorPattern {
            code_prefix: "SEM".to_string(),
            matcher: |e| matches!(e, CompilerError::SemanticError(_)),
            fixer: Self::fix_undefined_symbol,
        });
        
        Self { patterns }
    }
    
    /// Generate fix suggestions for type mismatches
    fn fix_type_mismatch(error: &CompilerError) -> Vec<FixSuggestion> {
        let mut fixes = Vec::new();
        
        // Add type cast suggestion
        fixes.push(FixSuggestion {
            description: "Add explicit type cast".to_string(),
            confidence: 0.8,
            modifications: vec![
                CodeModification::WrapCode {
                    start_line: 1, // Would be extracted from error
                    end_line: 1,
                    wrapper_type: WrapperType::Function,
                    parameters: HashMap::from([
                        ("function".to_string(), "CAST_TO_TYPE".to_string()),
                    ]),
                },
            ],
            example: Some("(CAST_TO_TYPE value TargetType)".to_string()),
            category: FixCategory::Correction,
        });
        
        fixes
    }
    
    /// Generate fix suggestions for undefined symbols
    fn fix_undefined_symbol(error: &CompilerError) -> Vec<FixSuggestion> {
        let mut fixes = Vec::new();
        
        // Suggest variable declaration
        fixes.push(FixSuggestion {
            description: "Declare the variable before use".to_string(),
            confidence: 0.9,
            modifications: vec![
                CodeModification::AddCode {
                    before_line: Some(1), // Would be extracted
                    after_line: None,
                    code: "(DECLARE_VARIABLE (NAME \"var_name\") (TYPE INTEGER) (MUTABLE TRUE))".to_string(),
                    indent_level: 1,
                },
            ],
            example: None,
            category: FixCategory::Addition,
        });
        
        // Suggest import
        fixes.push(FixSuggestion {
            description: "Import from module".to_string(),
            confidence: 0.7,
            modifications: vec![
                CodeModification::AddImport {
                    module: "std.collections".to_string(),
                    symbols: vec!["symbol_name".to_string()],
                },
            ],
            example: Some("(IMPORT_MODULE (NAME \"std.collections\"))".to_string()),
            category: FixCategory::Addition,
        });
        
        fixes
    }
}

/// Partial compilation result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PartialCompilationResult {
    /// Modules that compiled successfully
    pub successful_modules: Vec<String>,
    
    /// Modules that failed with reasons
    pub failed_modules: Vec<(String, String)>,
    
    /// Whether partial executable can be generated
    pub executable: bool,
    
    /// Missing functionality in partial build
    pub missing_functionality: Vec<String>,
    
    /// Suggested next steps
    pub next_steps: Vec<String>,
}

/// Enhanced error formatter for LLM consumption
pub struct LLMErrorFormatter {
    include_snippets: bool,
    include_fixes: bool,
    max_fixes_per_error: usize,
}

impl LLMErrorFormatter {
    pub fn new() -> Self {
        Self {
            include_snippets: true,
            include_fixes: true,
            max_fixes_per_error: 3,
        }
    }
    
    /// Format error for LLM consumption
    pub fn format_error(&self, error: &StructuredError) -> String {
        let mut output = String::new();
        
        // Header
        output.push_str(&format!("(COMPILATION_ERROR\n"));
        output.push_str(&format!("  (ERROR_CODE \"{}\")\n", error.error_code));
        output.push_str(&format!("  (SEVERITY \"{}\")\n", 
            match error.severity {
                ErrorSeverity::Fatal => "FATAL",
                ErrorSeverity::Error => "ERROR",
                ErrorSeverity::Warning => "WARNING",
                ErrorSeverity::Info => "INFO",
                ErrorSeverity::Hint => "HINT",
            }
        ));
        
        // Location
        output.push_str(&format!("  (LOCATION\n"));
        output.push_str(&format!("    (FILE \"{}\")\n", error.location.file));
        output.push_str(&format!("    (LINE {})\n", error.location.line));
        output.push_str(&format!("    (COLUMN {}))\n", error.location.column));
        
        // Message and explanation
        output.push_str(&format!("  (MESSAGE \"{}\")\n", error.message));
        if !error.explanation.is_empty() {
            output.push_str(&format!("  (EXPLANATION \"{}\")\n", error.explanation));
        }
        
        // Fix suggestions
        if self.include_fixes && !error.fix_suggestions.is_empty() {
            for (i, fix) in error.fix_suggestions.iter()
                .take(self.max_fixes_per_error)
                .enumerate() 
            {
                output.push_str(&format!("  (FIX_SUGGESTION_{}\n", i + 1));
                output.push_str(&format!("    (DESCRIPTION \"{}\")\n", fix.description));
                output.push_str(&format!("    (CONFIDENCE {:.2})\n", fix.confidence));
                
                // Include first modification as example
                if let Some(mod_example) = fix.modifications.first() {
                    match mod_example {
                        CodeModification::AddCode { code, .. } => {
                            output.push_str(&format!("    (ADD_CODE \"{}\")\n", code));
                        }
                        CodeModification::ReplaceCode { new_code, .. } => {
                            output.push_str(&format!("    (REPLACE_WITH \"{}\")\n", new_code));
                        }
                        _ => {}
                    }
                }
                output.push_str("  )\n");
            }
        }
        
        output.push_str(")\n");
        output
    }
    
    /// Format partial compilation result
    pub fn format_partial_result(&self, result: &PartialCompilationResult) -> String {
        let mut output = String::new();
        
        output.push_str("(PARTIAL_COMPILATION_RESULT\n");
        
        // Successful modules
        output.push_str("  (SUCCESSFUL_MODULES");
        for module in &result.successful_modules {
            output.push_str(&format!(" \"{}\"", module));
        }
        output.push_str(")\n");
        
        // Failed modules
        output.push_str("  (FAILED_MODULES\n");
        for (module, reason) in &result.failed_modules {
            output.push_str(&format!("    (\"{}\" (REASON \"{}\"))\n", module, reason));
        }
        output.push_str("  )\n");
        
        output.push_str(&format!("  (EXECUTABLE {})\n", 
            if result.executable { "TRUE" } else { "FALSE" }));
        
        // Missing functionality
        if !result.missing_functionality.is_empty() {
            output.push_str("  (MISSING_FUNCTIONALITY");
            for func in &result.missing_functionality {
                output.push_str(&format!(" \"{}\"", func));
            }
            output.push_str(")\n");
        }
        
        output.push_str(")\n");
        output
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_structured_error_builder() {
        let error = StructuredErrorBuilder::new()
            .error_code("TYPE-001".to_string())
            .severity(ErrorSeverity::Error)
            .location(ErrorLocation {
                file: "test.aether".to_string(),
                line: 10,
                column: 15,
                end_line: None,
                end_column: None,
                code_snippet: None,
            })
            .message("Type mismatch".to_string())
            .explanation("Expected Integer but found String".to_string())
            .add_fix(FixSuggestion {
                description: "Convert to integer".to_string(),
                confidence: 0.9,
                modifications: vec![],
                example: Some("(CAST_TO_TYPE value INTEGER)".to_string()),
                category: FixCategory::Correction,
            })
            .build()
            .unwrap();
        
        assert_eq!(error.error_code, "TYPE-001");
        assert_eq!(error.severity, ErrorSeverity::Error);
        assert_eq!(error.fix_suggestions.len(), 1);
    }
    
    #[test]
    fn test_error_formatter() {
        let error = StructuredError {
            error_code: "SEM-001".to_string(),
            severity: ErrorSeverity::Error,
            location: ErrorLocation {
                file: "main.aether".to_string(),
                line: 42,
                column: 12,
                end_line: None,
                end_column: None,
                code_snippet: None,
            },
            message: "Undefined variable 'x'".to_string(),
            explanation: "Variable 'x' was not declared before use".to_string(),
            fix_suggestions: vec![],
            related: vec![],
            llm_context: LLMContext {
                intent_mismatch: None,
                suggested_pattern: None,
                documentation: vec![],
                learning_hints: vec![],
                verification_status: None,
            },
        };
        
        let formatter = LLMErrorFormatter::new();
        let formatted = formatter.format_error(&error);
        
        assert!(formatted.contains("ERROR_CODE \"SEM-001\""));
        assert!(formatted.contains("LINE 42"));
        assert!(formatted.contains("Undefined variable 'x'"));
    }
}