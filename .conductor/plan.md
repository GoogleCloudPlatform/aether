# Aether V2 Syntax Migration - Project Plan

## Development Commands

### Setup
```bash
cargo build
```

### Daily Development
```bash
cargo build           # Build compiler
cargo test            # Run all tests
cargo clippy          # Run linter
cargo fmt             # Format code
```

### Before Committing
```bash
cargo fmt && cargo clippy && cargo test
```

---

## Phase 1: V2 Lexer

### Task 1.1: Define V2 Token Types
- [x] **Write Tests**: Create `src/lexer/v2.rs` with tests for each V2 token type
  - Test delimiters: `{`, `}`, `[`, `]`, `(`, `)`, `;`, `:`, `,`, `.`
  - Test operators: `+`, `-`, `*`, `/`, `%`, `==`, `!=`, `<`, `>`, `<=`, `>=`, `&&`, `||`, `!`, `=`
  - Test arrow: `->`
  - Test ownership sigils: `^`, `&`, `~`
  - Test at symbol: `@`
- [x] **Implement**: Create new `TokenType` enum in `src/lexer/v2.rs`
- [x] **Verify**: All token type tests pass (44 tests)

**Notes:**
```
Created src/lexer/v2.rs with:
- TokenType enum with 30+ token variants
- Keyword enum with 40+ keywords
- Token struct with location and lexeme
- 44 unit tests covering all token types
```

---

### Task 1.2: Implement Keyword Recognition
- [x] **Write Tests**: Add tests for all V2 keywords
  - Declaration keywords: `module`, `import`, `func`, `let`, `const`, `struct`, `enum`
  - Modifier keywords: `mut`, `pub`
  - Control flow: `when`, `case`, `else`, `match`, `for`, `while`, `in`, `return`, `break`, `continue`
  - Error handling: `try`, `catch`, `finally`, `throw`
  - Resource: `resource`, `cleanup`, `guaranteed`
  - Types: `Int`, `Int64`, `Float`, `String`, `Bool`, `Void`, `Array`, `Map`, `Pointer`, `MutPointer`, `SizeT`
  - Literals: `true`, `false`, `nil`
  - Other: `as`, `range`
- [x] **Implement**: Create `Keyword` enum and keyword lookup table with 42 keywords
- [x] **Verify**: All keyword tests pass (18 new tests, 62 total)

**Notes:**
```
Implemented Lexer struct with:
- HashMap<String, Keyword> for O(1) keyword lookup
- initialize_keywords() populates 42 keywords
- read_identifier() distinguishes keywords from identifiers
- Special handling for true/false -> BoolLiteral
- tokenize() produces token stream from input
```

---

### Task 1.3: Implement Literal Tokenization
- [x] **Write Tests**: Add tests for literal tokenization
  - Integer literals: `0`, `42`, `1000000`
  - Float literals: `3.14`, `0.5`
  - String literals: `"hello"`, `"with\nnewline"`, `"with\"quote"`, escape sequences
  - Character literals: `'a'`, `'\n'`, `'\''`, `'\\'`
  - Error cases: unterminated strings, invalid escapes
- [x] **Implement**: Implement `read_number()`, `read_string()`, `read_char()` methods
- [x] **Verify**: All literal tests pass (24 new tests, 86 total)

**Notes:**
```
Implemented literal tokenization:
- read_number(): integers and floats with decimal point detection
- read_string(): double-quoted strings with escape sequences (\n, \t, \r, \\, \", \0)
- read_char(): single-quoted characters with same escape sequences
- Error handling for unterminated strings and invalid escapes
```

---

### Task 1.4: Implement Operator Tokenization
- [x] **Write Tests**: Add tests for multi-character operators
  - Two-character: `==`, `!=`, `<=`, `>=`, `&&`, `||`, `->`
  - Disambiguation: `=` vs `==`, `<` vs `<=`, `>` vs `>=`, `-` vs `->`
- [x] **Implement**: Implement lookahead logic for multi-character operators
- [x] **Verify**: All operator tests pass (20 new tests, 106 total)

**Notes:**
```
Implemented operator tokenization in next_token():
- All delimiters: { } [ ] ( ) ; : , . @
- Arithmetic operators: + - * / %
- Assignment: =
- Multi-character operators with lookahead:
  - == (vs =), != (vs !), <= (vs <), >= (vs >), -> (vs -)
  - && (vs &), || (single | is error)
- Ownership sigils: ^ & ~
- Integration tests for function signatures, braced expressions, etc.
```

---

### Task 1.5: Implement Comment Handling
- [x] **Write Tests**: Add tests for comments
  - Line comments: `// comment`
  - Doc comments: `/// doc comment`
  - Comments at end of line
  - Comments on their own line
- [x] **Implement**: Implement `skip_line_comment()` method
- [x] **Verify**: All comment tests pass (11 new tests, 117 total)

**Notes:**
```
Implemented comment handling:
- Modified slash case in next_token() to check for //
- skip_line_comment() consumes characters until newline or EOF
- Both line comments (//) and doc comments (///) are skipped
- Single slash remains Slash operator
- Comments at end of file (no trailing newline) handled correctly
```

---

### Task 1.6: Implement Full Lexer Integration
- [x] **Write Tests**: Add integration tests for complete V2 code snippets
  - Hello world module
  - Function with parameters
  - When statement
  - Braced expressions
  - For loops
  - Struct definitions
  - Extern functions
  - Error handling (try/catch/finally)
  - Ownership types
  - Complex expressions
- [x] **Implement**: `tokenize()` method already implemented in previous tasks
- [x] **Verify**: All 126 integration tests pass

**Notes:**
```
Added 9 integration tests covering complete V2 code snippets:
- test_lexer_integration_hello_world: Module with function and string
- test_lexer_integration_function_with_params: Function with typed params
- test_lexer_integration_when_statement: When/case/else control flow
- test_lexer_integration_for_loop: For-in loop with range
- test_lexer_integration_struct_definition: Struct with fields
- test_lexer_integration_extern_function: @extern annotation
- test_lexer_integration_error_handling: try/catch/finally/throw
- test_lexer_integration_ownership_types: ^ & ~ sigils
- test_lexer_integration_complex_expressions: Nested braced expressions
```

---

## Phase 2: V2 Parser

### Task 2.1: Implement Parser Skeleton
- [x] **Write Tests**: Create `src/parser/v2.rs` with basic structure tests
  - Parser can be instantiated with token stream
  - Parser helper methods work (`peek`, `advance`, `expect`, `check`)
- [x] **Implement**: Create `Parser` struct with helper methods
- [x] **Verify**: All 18 skeleton tests pass

**Notes:**
```
Created src/parser/v2.rs with:
- Parser struct with tokens, position, errors
- Helper methods: peek, peek_next, advance, previous
- State checks: is_at_end, check, check_keyword
- Consume methods: expect, expect_keyword, match_any
- Error handling: add_error, errors
- Location tracking: current_location, current_position
- 18 unit tests covering all helper methods
```

---

### Task 2.2: Implement Module Parsing
- [x] **Write Tests**: Add tests for module parsing
  - Empty module: `module Foo { }`
  - Module with single import
  - Module with multiple imports
  - Module with dotted imports (e.g., std.io)
  - Error cases: missing name, braces, semicolons
- [x] **Implement**: Implement `parse_module()` method
- [x] **Verify**: All 12 module parsing tests pass (29 total)

**Notes:**
```
Implemented module parsing:
- parse_module(): Parses "module Name { ... }"
- parse_import(): Parses "import dotted.name;"
- parse_identifier(): Parses single identifier
- parse_dotted_identifier(): Parses dot-separated identifiers
- Supports multiline modules with comments
- Proper error messages for missing tokens
```

---

### Task 2.3: Implement Import Parsing
- [ ] **Write Tests**: Add tests for import statements
  - Simple import: `import std.io;`
  - Nested import: `import std.collections.HashMap;`
- [ ] **Implement**: Implement `parse_import()` method
- [ ] **Verify**: Import parsing tests pass

**Notes:**
```
```

---

### Task 2.4: Implement Type Parsing
- [x] **Write Tests**: Add tests for type specifiers
  - Primitive types: `Int`, `Int64`, `Float`, `String`, `Bool`, `Void`, `SizeT`
  - Generic types: `Array<Int>`, `Map<String, Int>`, nested arrays
  - Ownership types: `^String`, `&Int`, `&mut Int`, `~Resource`
  - Pointer types: `Pointer<Int>`, `MutPointer<Void>`
  - Named types: `MyCustomType`, `Result<Int, String>`
  - Complex combinations: `^Array<&Int>`
- [x] **Implement**: Implement `parse_type()` method
- [x] **Verify**: All 21 type parsing tests pass (50 total)

**Notes:**
```
Implemented type parsing:
- parse_type(): Main type parsing method with recursive support
- Ownership sigils: ^ (owned), & (borrowed), &mut, ~ (shared)
- Primitive types: Int, Int64, Float, String, Bool, Void, SizeT
- Built-in generics: Array<T>, Map<K,V>, Pointer<T>, MutPointer<T>
- User-defined types with optional generic arguments
- Proper error handling for malformed types
```

---

### Task 2.5: Implement Function Parsing (Basic)
- [x] **Write Tests**: Add tests for basic function definitions
  - No params, no return: `func foo() { }`
  - With return type: `func foo() -> Int { }`
  - With parameters: `func add(a: Int, b: Int) -> Int { }`
  - Complex types, ownership types, pointer returns
  - Error cases: missing name, parens, body, param types
- [x] **Implement**: Implement `parse_function()` method (without annotations)
- [x] **Verify**: All 14 function parsing tests pass (64 total)

**Notes:**
```
Implemented function parsing:
- parse_function(): Parses "func name(params) -> Type { body }"
- parse_parameters(): Parses comma-separated parameter list
- parse_parameter(): Parses "name: Type" parameter
- parse_block(): Parses "{...}" (skips body content for now)
- Default return type is Void when not specified
- Supports all type features: primitives, generics, ownership
```

---

### Task 2.6: Implement Annotation Parsing
- [ ] **Write Tests**: Add tests for function annotations
  - Simple: `@extern(library: "libc")`
  - With multiple params: `@requires({n > 0}, "must be positive")`
  - Multiple annotations on one function
- [ ] **Implement**: Implement `parse_annotation()` method
- [ ] **Implement**: Integrate annotations into `parse_function()`
- [ ] **Verify**: Annotation parsing tests pass

**Notes:**
```
```

---

### Task 2.7: Implement Variable Declaration Parsing
- [ ] **Write Tests**: Add tests for variable declarations
  - Immutable: `let x: Int = 42;`
  - Mutable: `let mut counter: Int = 0;`
  - With complex expression: `let sum: Int = {a + b};`
- [ ] **Implement**: Implement `parse_var_declaration()` method
- [ ] **Verify**: Variable declaration tests pass

**Notes:**
```
```

---

### Task 2.8: Implement Assignment Parsing
- [ ] **Write Tests**: Add tests for assignments
  - Simple: `x = 10;`
  - With expression: `x = {x + 1};`
- [ ] **Implement**: Implement assignment parsing in `parse_statement()`
- [ ] **Verify**: Assignment tests pass

**Notes:**
```
```

---

### Task 2.9: Implement Expression Parsing (Braced)
- [ ] **Write Tests**: Add tests for braced expressions
  - Arithmetic: `{a + b}`, `{x * y}`, `{a / b}`, `{n % 2}`
  - Comparison: `{x > 0}`, `{a == b}`, `{n != 0}`
  - Logical: `{a && b}`, `{x || y}`, `{!flag}`
  - Nested: `{{a + b} * {c - d}}`
- [ ] **Implement**: Implement `parse_braced_expression()` method
- [ ] **Verify**: Braced expression tests pass

**Notes:**
```
```

---

### Task 2.10: Implement Function Call Parsing
- [ ] **Write Tests**: Add tests for function calls
  - No args: `foo();`
  - Single arg (no label): `print(message);`
  - Multiple args (labeled): `add(first: a, second: b);`
  - Method-style: `str.length();`
- [ ] **Implement**: Implement `parse_call_expression()` method
- [ ] **Verify**: Function call tests pass

**Notes:**
```
```

---

### Task 2.11: Implement When Statement Parsing
- [ ] **Write Tests**: Add tests for when statements
  - Single case with else: `when { case ({x > 0}): return "pos"; else: return "neg"; }`
  - Multiple cases: `when { case ({x > 90}): ...; case ({x > 80}): ...; else: ...; }`
  - Nested statements in cases
- [ ] **Implement**: Implement `parse_when_statement()` method
- [ ] **Verify**: When statement tests pass

**Notes:**
```
```

---

### Task 2.12: Implement Loop Parsing
- [ ] **Write Tests**: Add tests for loops
  - While loop: `while ({i < 10}) { ... }`
  - For-in loop: `for item in items { ... }`
  - For-range loop: `for i in range(from: 0, to: 10) { ... }`
  - Break and continue
- [ ] **Implement**: Implement `parse_while_statement()` and `parse_for_statement()` methods
- [ ] **Verify**: Loop tests pass

**Notes:**
```
```

---

### Task 2.13: Implement Return Statement Parsing
- [ ] **Write Tests**: Add tests for return statements
  - With value: `return 42;`
  - With expression: `return {a + b};`
  - Without value: `return;`
- [ ] **Implement**: Implement `parse_return_statement()` method
- [ ] **Verify**: Return statement tests pass

**Notes:**
```
```

---

### Task 2.14: Implement Struct Parsing
- [ ] **Write Tests**: Add tests for struct definitions
  - Simple: `struct Point { x: Float; y: Float; }`
  - With doc comments
  - Empty struct
- [ ] **Implement**: Implement `parse_struct()` method
- [ ] **Verify**: Struct parsing tests pass

**Notes:**
```
```

---

### Task 2.15: Implement Enum Parsing
- [ ] **Write Tests**: Add tests for enum definitions
  - Simple: `enum Color { case Red; case Green; case Blue; }`
  - With associated values: `enum Result { case Ok(Int); case Error(String); }`
- [ ] **Implement**: Implement `parse_enum()` method
- [ ] **Verify**: Enum parsing tests pass

**Notes:**
```
```

---

### Task 2.16: Implement Match Expression Parsing
- [ ] **Write Tests**: Add tests for match expressions
  - Basic: `match result { case .Ok(let v): return v; case .Error(let e): return 0; }`
  - With wildcard: `case _: ...`
- [ ] **Implement**: Implement `parse_match_expression()` method
- [ ] **Verify**: Match expression tests pass

**Notes:**
```
```

---

### Task 2.17: Implement Try/Catch/Finally Parsing
- [ ] **Write Tests**: Add tests for error handling
  - Try-catch: `try { ... } catch Error as e { ... }`
  - With finally: `try { ... } catch Error as e { ... } finally { ... }`
  - Throw statement: `throw MyError("message");`
- [ ] **Implement**: Implement `parse_try_statement()` and `parse_throw_statement()` methods
- [ ] **Verify**: Error handling tests pass

**Notes:**
```
```

---

### Task 2.18: Implement External Function Parsing
- [ ] **Write Tests**: Add tests for extern functions
  - Basic: `@extern(library: "libc") func malloc(size: SizeT) -> Pointer<Void>;`
  - With symbol: `@extern(library: "libc", symbol: "free") func free(ptr: Pointer<Void>);`
- [ ] **Implement**: Implement `parse_extern_function()` method
- [ ] **Verify**: External function tests pass

**Notes:**
```
```

---

### Task 2.19: Implement Resource Scope Parsing
- [ ] **Write Tests**: Add tests for resource management
  - Basic resource: `resource file: FileHandle = openFile("data.txt") { cleanup: closeFile; };`
- [ ] **Implement**: Implement resource scope parsing
- [ ] **Verify**: Resource scope tests pass

**Notes:**
```
```

---

### Task 2.20: Parser Integration and Error Messages
- [ ] **Write Tests**: Add integration tests for complete programs
  - Hello world
  - Function with contracts
  - HTTP server example
- [ ] **Write Tests**: Add error message tests
  - Missing semicolon error
  - Missing brace error
  - Unlabeled argument error
- [ ] **Implement**: Polish error messages with source locations
- [ ] **Verify**: All parser tests pass
- [ ] **Verify**: `cargo test` shows >80% coverage for parser module

**Notes:**
```
```

---

## Phase 3: Pipeline Integration

### Task 3.1: Update Pipeline to Use V2 Lexer/Parser
- [ ] **Write Tests**: Add integration test that compiles a V2 file end-to-end
- [ ] **Implement**: Update `src/pipeline/mod.rs` Phase 1 to use new lexer/parser
- [ ] **Verify**: Integration test passes

**Notes:**
```
```

---

### Task 3.2: Verify AST Compatibility
- [ ] **Write Tests**: Create test that parses V2 code and verifies AST structure matches expected
- [ ] **Implement**: Fix any AST mapping issues discovered
- [ ] **Verify**: AST compatibility tests pass

**Notes:**
```
```

---

### Task 3.3: Verify Semantic Analysis Works
- [ ] **Write Tests**: Create test that runs semantic analysis on V2-parsed code
- [ ] **Implement**: Fix any issues discovered
- [ ] **Verify**: Semantic analysis tests pass

**Notes:**
```
```

---

### Task 3.4: Verify Full Compilation Works
- [ ] **Write Tests**: Create test that compiles V2 code to executable and runs it
- [ ] **Implement**: Fix any issues discovered
- [ ] **Verify**: Full compilation test passes

**Notes:**
```
```

---

## Phase 4: Example Migration

### Task 4.1: Migrate Hello World Example
- [ ] **Migrate**: Convert `examples/hello_world.aether` to V2 syntax
- [ ] **Verify**: Compiles and runs correctly

**Notes:**
```
```

---

### Task 4.2: Migrate Basic Examples
- [ ] **Migrate**: Convert basic examples to V2 syntax
  - `examples/arithmetic.aether`
  - `examples/variables.aether`
  - `examples/functions.aether`
  - `examples/types.aether`
- [ ] **Verify**: All compile and run correctly

**Notes:**
```
```

---

### Task 4.3: Migrate Control Flow Examples
- [ ] **Migrate**: Convert control flow examples to V2 syntax
  - `examples/conditionals.aether`
  - `examples/loops.aether`
  - `examples/pattern_matching.aether`
- [ ] **Verify**: All compile and run correctly

**Notes:**
```
```

---

### Task 4.4: Migrate HTTP Server Examples
- [ ] **Migrate**: Convert HTTP examples to V2 syntax
  - `examples/blog_listen.aether`
  - `examples/http_server.aether`
  - Any other networking examples
- [ ] **Verify**: All compile and run correctly

**Notes:**
```
```

---

### Task 4.5: Migrate Remaining Examples
- [ ] **Migrate**: Convert all remaining examples to V2 syntax
- [ ] **Verify**: All compile and run correctly
- [ ] **Count**: Document total examples migrated

**Notes:**
```
```

---

## Phase 5: Test Suite Migration

### Task 5.1: Update Parser Test Inputs
- [ ] **Migrate**: Update test input strings in `src/parser/tests.rs` to V2 syntax
- [ ] **Verify**: All parser tests pass

**Notes:**
```
```

---

### Task 5.2: Update Integration Test Inputs
- [ ] **Migrate**: Update any integration tests that use V1 syntax
- [ ] **Verify**: All integration tests pass

**Notes:**
```
```

---

### Task 5.3: Final Test Suite Verification
- [ ] **Verify**: Run `cargo test` - all 360+ tests pass
- [ ] **Verify**: Run `cargo clippy` - no warnings
- [ ] **Verify**: Run coverage report - >80% coverage

**Notes:**
```
```

---

## Phase 6: Cleanup and Documentation

### Task 6.1: Remove V1 Lexer/Parser Code
- [ ] **Implement**: Delete old V1 lexer code
- [ ] **Implement**: Delete old V1 parser code
- [ ] **Verify**: Build succeeds with only V2 code

**Notes:**
```
```

---

### Task 6.2: Update Documentation
- [ ] **Update**: Update `LANGUAGE_REFERENCE.md` to show V2 syntax
- [ ] **Update**: Update `README.md` examples to V2 syntax
- [ ] **Update**: Update any other documentation files

**Notes:**
```
```

---

### Task 6.3: Final Verification
- [ ] **Verify**: `cargo build --release` succeeds
- [ ] **Verify**: `cargo test` - all tests pass
- [ ] **Verify**: All examples compile and run
- [ ] **Verify**: CLI commands work (`aether compile`, `aether check`, `aether run`)

**Notes:**
```
```

---

## Summary

| Phase | Tasks | Description |
|-------|-------|-------------|
| 1 | 1.1 - 1.6 | V2 Lexer Implementation |
| 2 | 2.1 - 2.20 | V2 Parser Implementation |
| 3 | 3.1 - 3.4 | Pipeline Integration |
| 4 | 4.1 - 4.5 | Example Migration |
| 5 | 5.1 - 5.3 | Test Suite Migration |
| 6 | 6.1 - 6.3 | Cleanup and Documentation |

**Total Tasks:** 38
