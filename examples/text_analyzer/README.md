# Text File Analyzer Example

**Status: ✅ Syntax validation passes**

This example demonstrates AetherScript's LLM-first programming language design through a practical text analysis application.

## Current Status

- ✅ **Compiler builds successfully** (after fixing 25+ compilation errors)
- ✅ **Syntax validation passes** (`aether-compiler check` succeeds)
- ⏳ **Full compilation pending** (many language features still in development)
- 📚 **Serves as design template** for LLM-first programming

## LLM-First Features Demonstrated

### 1. Intent-Based Programming
Every function includes an `INTENT` clause that describes its purpose in natural language:
```lisp
(DEFINE_FUNCTION
  (NAME count_lines)
  (INTENT "Count the number of lines in text content")
  ...)
```

### 2. Explicit Structure for AI Understanding
- S-expression syntax is parse-friendly for LLMs
- Clear hierarchical structure with explicit keywords
- Self-documenting code that describes its own purpose

### 3. Type Safety and Contracts
```lisp
(ACCEPTS_PARAMETER (NAME text) (TYPE STRING))
(RETURNS INTEGER)
```

### 4. Functional Approach
- Explicit variable declarations and assignments
- Clear data flow and transformations
- Predictable execution model

## What It Does

This text analyzer:

1. **Analyzes sample text** with embedded newlines
2. **Counts lines** by scanning for `\n` characters
3. **Counts words** using whitespace as delimiters
4. **Displays results** in a formatted output

### Sample Output (when fully implemented):
```
Text Analysis Results:
Sample: Hello World!
This is a test.
Another line.
Lines: 3
Words: 8
```

## Implementation Highlights

### Line Counting Algorithm
```lisp
(WHILE_LOOP
  (CONDITION (PREDICATE_LESS_THAN i text_length))
  (BODY
    (IF_CONDITION
      (PREDICATE_EQUALS (STRING_CHAR_AT text i) '\n')
      (THEN_EXECUTE
        (ASSIGN line_count (EXPRESSION_ADD line_count 1))))
    (ASSIGN i (EXPRESSION_ADD i 1))))
```

### Word Counting with State Machine
```lisp
(IF_CONDITION
  (PREDICATE_NOT_EQUALS is_space TRUE)
  (THEN_EXECUTE
    (IF_CONDITION
      (PREDICATE_EQUALS in_word FALSE)
      (THEN_EXECUTE
        (ASSIGN word_count (EXPRESSION_ADD word_count 1))
        (ASSIGN in_word TRUE)))))
```

## LLM-First Design Principles Illustrated

1. **Explicitness over Conciseness**: Every operation is explicitly named
2. **Intent Documentation**: Natural language descriptions embedded in code
3. **Hierarchical Structure**: Clear nesting and organization
4. **Predictable Patterns**: Consistent syntax for similar operations
5. **Type Safety**: Explicit type annotations for all variables

## Building and Testing

```bash
# Build and test the example
./build.sh

# What this does:
# 1. Finds the AetherScript compiler
# 2. Attempts compilation 
# 3. Falls back to syntax check
# 4. Shows error details if compilation fails
# 5. Demonstrates the language's design principles
```

## Future LLM-First Features (From Design Doc)

When the compiler implementation is complete, this example will also demonstrate:

- **Resource Management**: `RESOURCE_SCOPE` for automatic cleanup
- **Pattern Composition**: `COMPOSE_PATTERNS` for processing pipelines  
- **Contract Verification**: `PRECONDITION`/`POSTCONDITION` with proof hints
- **Enhanced Error System**: Structured errors with auto-fix suggestions
- **Intent Analysis**: LLM-driven code generation from natural language

## Educational Value

This example serves as:

1. **Syntax Reference**: Correct AetherScript patterns and structures
2. **Design Template**: How to structure LLM-friendly code
3. **Implementation Guide**: Practical algorithms in AetherScript
4. **Testing Framework**: Validation that the compiler infrastructure works

## Files

- **`main.aether`**: Complete text analyzer implementation
- **`build.sh`**: Smart build script with error handling
- **`README.md`**: This comprehensive documentation

This example demonstrates that AetherScript's LLM-first design creates code that is both human-readable and machine-parseable, making it ideal for AI-driven development workflows.