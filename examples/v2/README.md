# AetherScript V2 Examples

This directory contains comprehensive examples demonstrating every feature of the AetherScript V2 syntax.

## Directory Structure

```
examples/v2/
├── 01-basics/          # Program structure and modules
├── 02-variables/       # let, var, const
├── 03-types/           # Primitive types and type system
├── 04-functions/       # Functions, parameters, closures
├── 05-operators/       # Arithmetic, comparison, logical
├── 06-control-flow/    # when/else, while, for, match
├── 07-structs/         # Struct definition and usage
├── 08-enums/           # Enum definition and pattern matching
├── 09-pattern-matching/ # Advanced pattern matching
├── 10-collections/     # Arrays, maps, iteration
├── 11-memory/          # Ownership and pointers
├── 12-error-handling/  # Result type and error propagation
├── 13-strings/         # String operations
├── 14-ffi/             # C function interop
├── 15-stdlib/          # Standard library usage
└── 16-networking/      # Network concepts
```

## Running Examples

To compile and run an example:

```bash
# Check syntax
cargo run -- check examples/v2/01-basics/hello_world/main.aes

# Compile to executable
cargo run -- compile examples/v2/01-basics/hello_world/main.aes -o hello

# Run the executable
./hello
```

## Quick Start

Start with these examples in order:

1. `01-basics/hello_world/` - Your first program
2. `02-variables/let_bindings/` - Variable declaration
3. `04-functions/basic_functions/` - Defining functions
4. `06-control-flow/if_else/` - Conditionals
5. `07-structs/basic_struct/` - Custom types
6. `08-enums/enum_with_data/` - Algebraic data types

## Key Syntax Notes

- **Binary expressions need braces**: `return {a + b};`
- **Use `when` instead of `if`**: `when condition { }`
- **Type annotations required**: `let x: Int = 10;`
- **Enum variants use `::`**: `Option::Some(42)`
- **Match arms use `=>`**: `1 => { return "one"; }`
