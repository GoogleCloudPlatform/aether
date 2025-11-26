# Project Status

## Last Updated
2025-11-25

## Project Status
Phase 4 - Example Verification and Makefiles

## Current Phase
Phase 4: Example Verification and Makefiles

## Completed Tasks
- [x] Phase 1: V2 Lexer (Complete)
- [x] Phase 2: V2 Parser (Complete)
- [x] Phase 3: Pipeline Integration (Complete)
- [x] Phase 5: Test Suite Migration (Complete)
- [x] Phase 6.1: Remove V1 Lexer/Parser Code (Complete)
- [x] Task 4.1: Hello World Example Verified
- [x] Task 4.2: Constants Example Verified
- [x] Task 4.3: Let Bindings Example Verified
- [x] Task 4.3: Mutability Example Verified
- [x] Task 4.3: Simple Main Example Verified
- [x] Task 4.3: Module System Example Verified (fixed to return 0)
- [x] Task 4.3: Arithmetic Example Verified (added Makefile)
- [x] Task 4.3: Basic Functions Example Verified (added Makefile)
- [x] Task 4.4: Parameters Example Verified (added Makefile)
- [x] Task 4.5: Comparison Example Verified (added Makefile) - required implementing comparison operators
- [x] Task 4.6: Loops Example Verified (added Makefile) - required implementing var keyword
- [x] Task 4.7: Logical Example Verified (added Makefile) - required implementing logical operators
- [x] Task 4.8: Basic Struct Example Verified (added Makefile) - required implementing struct construction
- [x] Task 4.9: Nested Structs Example Verified (added Makefile) - required fixing nested field access in LLVM backend
- [x] Task 4.10: Struct Methods Example Verified (added Makefile)
- [x] Task 4.11: Basic Enum Example Verified (added Makefile) - required implementing enum variant expressions
- [x] Task 4.12: If/Else Example Verified (added Makefile)
- [x] Task 4.13: Primitives Example Verified
- [x] Task 4.14: Type Aliases Example Verified (added Makefile)
- [x] Task 4.15: Type Conversions Example Verified (added Makefile)
- [x] Task 4.16: Return Values Example Verified (added Makefile) - fixed by MIR return bug fix
- [x] Task 4.17: Foreach Example Verified (added Makefile) - implemented range syntax in for loops
- [x] Task 4.18: Arrays Example Verified (added Makefile) - implemented array literals and fixed function parameter types for arrays
- [x] Task 4.19: Iteration Example Verified (added Makefile) - uses range syntax
- [x] Task 4.20: Match Example Verified (added Makefile) - implemented match statement parsing and MIR lowering
- [x] Task 4.21: Match Basics Example Verified (added Makefile)

## Current Task
Task 4.22: Continue verifying remaining examples

## Not Yet Working (needs parser/semantic work)
- closures: needs lambda syntax (`=>`)
- strings/FFI: needs String to Pointer<Char> coercion
- enums with data: needs enum variant patterns with bindings

## Next Actions
1. Continue verifying examples in `examples/v2/`.
2. Add Makefiles to verified examples.

## Blockers
None

## Known Parser/Compiler Limitations
- [x] ~~`Int32`, `Float`, etc. types~~ (FIXED)
- [x] ~~`var` keyword not supported~~ (FIXED: added `var` keyword)
- [x] ~~Comparison operators (`<`, `>`, `==`, etc.)~~ (FIXED)
- [x] ~~Logical operators (`!`, `&&`, `||`)~~ (FIXED)
- [x] ~~Unary minus for negative literals not supported~~ (FIXED: added `-expr` syntax)
- [x] ~~Struct construction expressions not implemented~~ (FIXED: `Point { x: 1, y: 2 }` syntax)
- [x] ~~Nested struct field access broken~~ (FIXED: `rect.top_left.x` now works)
- [x] ~~Enum variant expressions not supported~~ (FIXED: `Color::Red` syntax now works)
- [x] ~~`match` expressions not supported~~ (FIXED: implemented Statement::Match with pattern matching on integer literals)
- [ ] Enums with associated data syntax (e.g., `Some(Int)`) not supported
- [x] ~~`return` inside `when` without `else` doesn't properly terminate~~ (FIXED: MIR lowering now checks for diverging blocks)

## Session Notes
- Resolved parser infinite loop for file-scoped modules.
- Fixed parsing of chained binary operators in braced expressions.
- Verified `constants` and `let_bindings` examples.
- **Fixed critical CSE optimization bug:** The Common Subexpression Elimination pass was incorrectly reusing values from reassigned variables. When `counter = 0` was computed, and later `return 0` appeared, CSE incorrectly replaced the constant with a copy from `counter` (which had since been reassigned to 12). Fixed by invalidating expressions computed by a local when that local is reassigned.
- Updated all Makefiles to use release build instead of debug.
- Fixed test compilation errors (parse_source signature, mutable pipeline).
- Verified additional examples: simple_main, module_system, arithmetic, basic_functions.
- **Implemented unary minus:** Added `-expr` parsing in parser, `Negate` handling in semantic analysis, and `lower_negate` in MIR lowering using `UnOp::Neg`.
- **Implemented struct construction:** Added `TypeName { field: value, ... }` parsing with smart lookahead to distinguish from blocks. Changed struct field definitions from semicolon-separated to comma-separated syntax. Verified with `basic_struct` example (returns 50).
- Updated struct-related tests to use comma syntax instead of semicolons.
- **Fixed nested struct field access:** The LLVM backend was not tracking the current struct type through projections. Fixed `generate_operand` to properly handle nested field access by tracking `current_struct_type` and loading pointers for struct fields. Verified with `nested_structs` example (returns 100).
- Verified `struct_methods` example (returns 11).
- **Implemented enum variant expressions:** Added `::` (DoubleColon) token to lexer and `parse_enum_variant_expression` function to parser. Now supports `Color::Red` and `Color::Red(value)` syntax. Verified with `basic_enum` example (returns 0).
- Verified `if_else` example (returns 7 = max(7,3)) and `primitives` example (returns 42).
- **Fixed return inside when-without-else:** MIR lowering was unconditionally setting `Goto` terminator after lowering if/while/loop blocks, overwriting any `Return` terminator. Added `current_block_diverges()` helper to check if block already has a `Return` terminator and skip setting the `Goto` in that case. Verified with `return_values` example (now returns 7 instead of 0).
- **Implemented array literals:** Added `looks_like_array_literal()` to distinguish `[1, 2, 3]` from capture lists, and `parse_array_literal()` to parse array literals. Array creation uses runtime functions `array_create`, `array_set`, `array_get`.
- **Fixed array parameter types:** The LLVM backend function type generation was missing cases for `Type::Array` and `Type::Map`, causing them to fall through to the default `i32`. Fixed by adding explicit cases to generate `ptr` type for arrays and maps. Verified with `arrays` example (returns 15 = sum of 1+2+3+4+5).
- Verified `iteration` example (returns 55 = 1+2+...+10).
- **Implemented match statements:** Added `Statement::Match` and `MatchArm` to AST, `parse_match_statement` to parser, semantic analysis handling, and MIR lowering using `SwitchInt` terminator. Supports integer literal patterns and wildcard (`_`) patterns. Verified with `match` example (returns 200) and `match_basics` example (returns 3).
