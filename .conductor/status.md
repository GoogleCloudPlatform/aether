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

## Current Task
Task 4.3: Continue verifying remaining examples

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
- [ ] Enum variant expressions not supported (`Color::Red` syntax needs implementation)

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
