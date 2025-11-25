# AetherScript Documentation Index for LLM Agents

## Read These Files in Order

### 1. Quick Start (read first)
- **`docs/llm-cheatsheet.md`** - One-page syntax reference, common mistakes
- **`examples/hello_v2.aes`** - Working example program

### 2. Complete Reference (read for detailed understanding)
- **`docs/llm-handbook.md`** - Comprehensive language guide with all features

### 3. Grammar and Parsing (for advanced understanding)
- **`src/lexer/v2.rs`** - Token types and lexer (search for `pub enum TokenType`)
- **`src/parser/v2.rs`** - Parser implementation (search for `pub fn parse_`)

### 4. AST Structure (for code analysis)
- **`src/ast/mod.rs`** - AST node definitions

---

## Critical Syntax Rules

1. **Binary expressions need braces**: `return {a + b};` not `return a + b;`
2. **Use `when` not `if`** for conditionals
3. **All variables need type annotations**: `let x: Int = 10;`
4. **Enum variants use `::`**: `Option::Some(x)`
5. **File extension**: `.aes`

---

## Example Program Template

```aether
module my_program;

// Optional: external function declarations
@extern("C")
func puts(s: *Char) -> Int;

// Struct definitions
struct MyStruct {
    field: Int,
}

// Enum definitions
enum MyEnum {
    VariantA,
    VariantB(Int),
}

// Helper functions
func helper(x: Int) -> Int {
    return {x * 2};
}

// Entry point
func main() -> Int {
    let value: Int = helper(21);
    return value;
}
```

---

## Verification

To check if generated code is correct:
```bash
cargo run -- check path/to/file.aes
cargo run -- compile path/to/file.aes -o output
```
