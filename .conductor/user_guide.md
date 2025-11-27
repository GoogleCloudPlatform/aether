# Aether V2 Syntax - User Guide

## Overview

Aether is a systems programming language with LLM-first design principles. It combines memory safety through an ownership system with explicit, unambiguous syntax optimized for AI code generation.

## Primary Users

**Large Language Models (LLMs)** are the primary users of Aether. The language is designed for:
- Reliable, deterministic code generation by AI systems
- Unambiguous syntax that eliminates parsing edge cases
- Explicit structure that reduces hallucination errors

Human developers interact with Aether primarily for:
- Reviewing and debugging LLM-generated code
- Writing compiler tooling and runtime components
- Maintaining the language specification

## The V2 Syntax

Aether V2 replaces the original S-expression (LISP-like) syntax with a Swift/Rust-inspired syntax that is more readable while maintaining strict unambiguity.

### Core Syntax Principles

1. **All binary operations require braces**: `{x + y}`, `{a * b}`
2. **All comparisons require braces**: `{x > 0}`, `{a == b}`
3. **All control flow uses `when` blocks**: No if/else statements
4. **All statements end with semicolons**: No exceptions
5. **Multi-argument functions use labeled parameters**: `func(arg1: val1, arg2: val2)`
6. **No special cases**: Rules apply uniformly everywhere

### Example: Hello World

```aether
module HelloWorld {
    /// A simple hello world program

    func main() -> Int {
        puts("Hello, World!");
        return 0;
    }
}
```

### Example: Conditional Logic

```aether
func classify(score: Int) -> String {
    when {
        case ({score > 90}): return "A";
        case ({score > 80}): return "B";
        case ({score > 70}): return "C";
        else: return "F";
    }
}
```

### Example: Arithmetic with Braces

```aether
func calculate(a: Int, b: Int, c: Int) -> Int {
    return {{a * b} + {c / 2}};
}
```

## Key Features

- **Memory Safety**: Ownership system with move, borrow, and shared semantics
- **Contract-Based Programming**: `@requires` and `@ensures` annotations for preconditions/postconditions
- **FFI Support**: `@extern` annotations for calling C libraries
- **Resource Management**: Scoped resource handling with guaranteed cleanup
- **Pattern Matching**: `match` expressions with exhaustive case handling

## Compilation

```bash
# Compile to executable
aether compile program.aether

# Type checking only
aether check program.aether

# Run directly
aether run program.aether
```

## Project Goal

This project migrates Aether from V1 (S-expression) syntax to V2 syntax by:
1. Replacing the V1 lexer/parser with a new V2 lexer/parser
2. Manually migrating all example files to V2 syntax
3. Updating documentation to reflect V2 syntax

V1 syntax will not be supported after migration.
