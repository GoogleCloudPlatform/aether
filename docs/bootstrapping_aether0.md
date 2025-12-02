# Aether-0: Subset for Bootstrapping

Aether-0 defines the minimal subset of the AetherScript language required to implement a self-hosting compiler. The goal of Aether-0 is to be simple enough to be implementable and verifiable, while still being expressive enough to represent basic compiler components (e.g., AST nodes, a simple lexer, and parser).

## Core Features of Aether-0

### 1. Basic Types
- `Int`: 64-bit signed integers.
- `Bool`: Boolean values (`true`, `false`).
- `String`: Immutable string literals.
- `Void`: Represents the absence of a value (e.g., for functions with no return).
- `Pointer<T>`: Generic pointers for low-level memory access (primarily for FFI to C).
- `Array<T>`: Simple fixed-size arrays.

### 2. Variables and Constants
- `let` (immutable) and `let mut` (mutable) variable declarations.
- `const` declarations for compile-time constants.

### 3. Operators
- **Arithmetic**: `+`, `-`, `*`, `/`, `%` (integer arithmetic only).
- **Comparison**: `==`, `!=`, `<`, `<=`, `>`, `>=`.
- **Logical**: `&&`, `||`, `!`.

### 4. Control Flow
- `if`/`else if`/`else` statements.
- `while` loops.
- `for` loops (range-based iteration only, e.g., `for i in 0..10`).
- `return` statements (with optional value).

### 5. Functions
- Function declarations with parameters and return types.
- No generic parameters for functions in Aether-0.
- No `where` clauses for functions in Aether-0.
- Simple function calls (no explicit type arguments).

### 6. Structs
- Basic struct definitions with named fields.
- No generic parameters for structs in Aether-0.
- Direct field access (`instance.field`).
- Struct construction (`MyStruct { field: value }`).

### 7. Modules
- Simple module declarations (`module MyModule;`).
- `import` statements for importing other modules.

### 8. External Functions (FFI)
- `@extern("C") func name(params) -> return_type;` syntax for interfacing with the underlying runtime or C libraries.

### 9. No Advanced Features
- No traits or trait implementations.
- No enums.
- No pattern matching.
- No concurrency primitives (`concurrent`, `spawn`, `await`).
- No ownership system enforcement (initial version assumes basic memory safety by underlying runtime).
- No formal contracts (`@pre`, `@post`, `@invariant`) within Aether-0 itself, as verification will be handled by the self-hosting compiler later. However, the self-hosting compiler *will* enforce contracts on the Aether-0 code it compiles.

## Implications for Bootstrapping

Developing the AetherScript compiler in AetherScript will start with this Aether-0 subset. The initial compiler (written in Rust) will compile Aether-0 code. As Aether-0 is expanded with more features, the self-hosted compiler can gradually be upgraded to support those features, eventually allowing it to compile its own more advanced versions.
