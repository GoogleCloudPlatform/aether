# LLM-Optimized Error System for AetherScript

## Overview

Phase 3 of the FINAL_DESIGN.md implementation provides a structured error system designed specifically for LLM consumption. This system transforms traditional compiler errors into rich, structured formats with auto-fix suggestions, intent analysis, and partial compilation support.

## Key Features

### 1. Structured Error Format

Each error now includes:
- **Unique error codes** (e.g., "SEM-001", "TYPE-042")
- **Severity levels** (Fatal, Error, Warning, Info, Hint)
- **Enhanced location info** with code snippets
- **Detailed explanations** for LLM understanding
- **Auto-fix suggestions** with confidence scores
- **Related diagnostics** showing cause/effect relationships
- **LLM-specific context** including intent mismatches

```rust
pub struct StructuredError {
    pub error_code: String,
    pub severity: ErrorSeverity,
    pub location: ErrorLocation,
    pub message: String,
    pub explanation: String,
    pub fix_suggestions: Vec<FixSuggestion>,
    pub related: Vec<RelatedDiagnostic>,
    pub llm_context: LLMContext,
}
```

### 2. Auto-Fix Suggestions

The system provides multiple fix suggestions for each error:

```rust
pub struct FixSuggestion {
    pub description: String,
    pub confidence: f32,                    // 0.0 to 1.0
    pub modifications: Vec<CodeModification>,
    pub example: Option<String>,
    pub category: FixCategory,
}
```

Fix categories include:
- **Addition** - Add missing code
- **Correction** - Fix incorrect code
- **Removal** - Remove unnecessary code
- **Refactoring** - Improve code style
- **Safety** - Add safety checks
- **Performance** - Optimize code

### 3. Code Modifications

Precise instructions for fixing errors:

```rust
pub enum CodeModification {
    AddCode { before_line, after_line, code, indent_level },
    ReplaceCode { start_line, start_column, end_line, end_column, new_code },
    RemoveCode { start_line, start_column, end_line, end_column },
    AddImport { module, symbols },
    WrapCode { start_line, end_line, wrapper_type, parameters },
}
```

### 4. Intent Analysis

Detects when code behavior doesn't match stated intent:

```rust
pub struct IntentMismatch {
    pub stated_intent: String,
    pub detected_behavior: String,
    pub confidence: f32,
    pub evidence: Vec<String>,
}
```

The intent analyzer:
- Infers actual behavior from code analysis
- Compares against stated intent in metadata
- Detects side effects in "pure" functions
- Identifies missing error handling
- Suggests appropriate patterns

### 5. Enhanced Error Output

Errors are formatted in LLM-friendly S-expressions:

```lisp
(COMPILATION_ERROR
  (ERROR_CODE "TYPE-001")
  (SEVERITY "ERROR")
  (LOCATION
    (FILE "math.aether")
    (LINE 42)
    (COLUMN 12))
  (MESSAGE "Type mismatch: expected Float, found Integer")
  (EXPLANATION "The expression has type 'Integer' but type 'Float' was expected")
  (FIX_SUGGESTION_1
    (DESCRIPTION "Convert integer to float")
    (CONFIDENCE 0.95)
    (ADD_CODE "(CAST_TO_TYPE value FLOAT)"))
)
```

### 6. Partial Compilation Results

Even when errors occur, the system provides:

```lisp
(PARTIAL_COMPILATION_RESULT
  (SUCCESSFUL_MODULES ("user_auth" "data_validation"))
  (FAILED_MODULES
    ("payment_processing" 
      (REASON "Unverified invariant in process_payment")))
  (EXECUTABLE TRUE)
  (MISSING_FUNCTIONALITY ("payment processing"))
)
```

## Implementation Components

### Error Enhancement Module (`error/enhancement.rs`)

Converts traditional compiler errors into structured errors:
- Pattern matching for common error types
- Fix suggestion generation
- Code snippet extraction
- Learning hint addition

### Intent Analysis Module (`error/intent_analysis.rs`)

Analyzes functions for behavior/intent mismatches:
- Statement analyzers for behavior inference
- Pattern database for intent matching
- Side effect detection
- Algorithm pattern recognition

### Structured Error Types (`error/structured.rs`)

Core data structures for the enhanced error system:
- `StructuredError` - Main error type
- `FixSuggestion` - Auto-fix recommendations
- `CodeModification` - Precise fix instructions
- `LLMContext` - Additional context for LLMs

## Usage Example

```rust
// Original error
let error = SemanticError::UndefinedSymbol {
    symbol: "calculate_total".to_string(),
    location: SourceLocation { file: "billing.aether", line: 45, column: 8 },
};

// Enhanced error with fixes
let enhanced = enhancer.enhance_error(&CompilerError::SemanticError(error));

// Produces:
// - Error code: "SEM-001"
// - Fix suggestions:
//   1. Declare variable 'calculate_total' (confidence: 0.9)
//   2. Import from module 'std.math' (confidence: 0.7)
//   3. Did you mean 'calculate_tax'? (confidence: 0.8)
```

## Benefits for LLM Code Generation

1. **Precise Fix Instructions**: LLMs can apply fixes without guessing
2. **Intent Validation**: Ensures generated code matches requirements
3. **Pattern Learning**: LLMs learn from fix suggestions
4. **Partial Success**: Continue working even with some errors
5. **Context Preservation**: Related errors help understand root causes

## Integration with Verification

The error system integrates with Phase 2's verification:
- Contract violations produce structured errors
- Verification failures include counterexamples
- Proof obligations guide fix suggestions

## Testing

The system includes comprehensive tests:
```bash
cargo test error::enhancement
cargo test error::intent_analysis
cargo test error::structured
```

## Next Steps

With Phase 3 complete, the remaining phases are:
- Phase 4: Resource Management with RESOURCE_SCOPE
- Phase 5: Verified Pattern Library

The LLM-optimized error system ensures that AetherScript provides clear, actionable feedback that LLMs can understand and act upon, making the development cycle faster and more reliable.