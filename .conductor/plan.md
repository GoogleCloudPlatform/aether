# Aether V2 Syntax Migration - Project Plan

## Development Commands

### Setup
```bash
car go build
```

### Daily Development
```bash
car go build           # Build compiler
car go test            # Run all tests
car go clippy          # Run linter
car go fmt             # Format code
```

### Before Committing
```bash
car go fmt && car go clippy && car go test
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

### Task 2.3: Implement Import Parsing
- [x] **Write Tests**: Add tests for import statements
  - Simple import: `import std.io;`
  - Nested import: `import std.collections.HashMap;`
- [x] **Implement**: Implement `parse_import()` method
- [x] **Verify**: Import parsing tests pass

**Notes:**
```
```

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

### Task 2.6: Implement Annotation Parsing
- [x] **Write Tests**: Add tests for function annotations
  - Simple: `@extern(library: "libc")`
  - With multiple params: `@requires({n > 0}, "must be positive")`
  - Multiple annotations on one function
- [x] **Implement**: Implement `parse_annotation()` method
- [x] **Implement**: Integrate annotations into `parse_function()`
- [x] **Verify**: Annotation parsing tests pass

**Notes:**
```
```

### Task 2.7: Implement Variable Declaration Parsing
- [x] **Write Tests**: Add tests for variable declarations
  - Immutable: `let x: Int = 42;`
  - Mutable: `let mut counter: Int = 0;`
  - With complex expression: `let sum: Int = {a + b};`
- [x] **Implement**: Implement `parse_var_declaration()` method
- [x] **Verify**: Variable declaration tests pass

**Notes:**
```
```

### Task 2.8: Implement Assignment Parsing
- [x] **Write Tests**: Add tests for assignments
  - Simple: `x = 10;`
  - With expression: `x = {x + 1};`
- [x] **Implement**: Implement assignment parsing in `parse_statement()`
- [x] **Verify**: Assignment tests pass

**Notes:**
```
```

### Task 2.9: Implement Expression Parsing (Braced)
- [x] **Write Tests**: Add tests for braced expressions
  - Arithmetic: `{a + b}`, `{x * y}`, `{a / b}`, `{n % 2}`
  - Comparison: `{x > 0}`, `{a == b}`, `{n != 0}`
  - Logical: `{a && b}`, `{x || y}`, `{!flag}`
  - Nested: `{{a + b} * {c - d}}`
- [x] **Implement**: Implement `parse_braced_expression()` method
- [x] **Verify**: Braced expression tests pass

**Notes:**
```
```

### Task 2.10: Implement Function Call Parsing
- [x] **Write Tests**: Add tests for function calls
  - No args: `foo();`
  - Single arg (no label): `print(message);`
  - Multiple args (labeled): `add(first: a, second: b);`
  - Method-style: `str.length();`
- [x] **Implement**: Implement `parse_call_expression()` method
- [x] **Verify**: Function call tests pass

**Notes:**
```
```

### Task 2.11: Implement When Statement Parsing
- [x] **Write Tests**: Add tests for when statements
  - Single case with else: `when { case ({x > 0}): return "pos"; else: return "neg"; }`
  - Multiple cases: `when { case ({x > 90}): ...; case ({x > 80}): ...; else: ...; }`
  - Nested statements in cases
- [x] **Implement**: Implement `parse_when_statement()` method
- [x] **Verify**: When statement tests pass

**Notes:**
```
```

### Task 2.12: Implement Loop Parsing
- [x] **Write Tests**: Add tests for loops
  - While loop: `while ({i < 10}) { ... }`
  - For-in loop: `for item in items { ... }`
  - For-range loop: `for i in range(from: 0, to: 10) { ... }`
  - Break and continue
- [x] **Implement**: Implement `parse_while_statement()` and `parse_for_statement()` methods
- [x] **Verify**: Loop tests pass

**Notes:**
```
```

### Task 2.13: Implement Return Statement Parsing
- [x] **Write Tests**: Add tests for return statements
  - With value: `return 42;`
  - With expression: `return {a + b};`
  - Without value: `return;`
- [x] **Implement**: Implement `parse_return_statement()` method
- [x] **Verify**: Return statement tests pass

**Notes:**
```
```

### Task 2.14: Implement Struct Parsing
- [x] **Write Tests**: Add tests for struct definitions
  - Simple: `struct Point { x: Float; y: Float; }`
  - With doc comments
  - Empty struct
- [x] **Implement**: Implement `parse_struct()` method
- [x] **Verify**: Struct parsing tests pass

**Notes:**
```
```

### Task 2.15: Implement Enum Parsing
- [x] **Write Tests**: Add tests for enum definitions
  - Simple: `enum Color { case Red; case Green; case Blue; }`
  - With associated values: `enum Result { case Ok(Int); case Error(String); }`
- [x] **Implement**: Implement `parse_enum()` method
- [x] **Verify**: Enum parsing tests pass

**Notes:**
```
```

### Task 2.16: Implement Match Expression Parsing
- [x] **Write Tests**: Add tests for match expressions
  - Basic: `match result { case .Ok(let v): return v; case .Error(let e): return 0; }`
  - With wildcard: `case _: ...`
- [x] **Implement**: Implement `parse_match_expression()` method
- [x] **Verify**: Match expression tests pass

**Notes:**
```
```

### Task 2.17: Implement Try/Catch/Finally Parsing
- [x] **Write Tests**: Add tests for error handling
  - Try-catch: `try { ... } catch Error as e { ... }`
  - With finally: `try { ... } catch Error as e { ... } finally { ... }`
  - Throw statement: `throw MyError("message");`
- [x] **Implement**: Implement `parse_try_statement()` and `parse_throw_statement()` methods
- [x] **Verify**: Error handling tests pass

**Notes:**
```
```

### Task 2.18: Implement External Function Parsing
- [x] **Write Tests**: Add tests for extern functions
  - Basic: `@extern(library: "libc") func malloc(size: SizeT) -> Pointer<Void>;`
  - With symbol: `@extern(library: "libc", symbol: "free") func free(ptr: Pointer<Void>);`
- [x] **Implement**: Implement `parse_extern_function()` method
- [x] **Verify**: External function tests pass

**Notes:**
```
```

### Task 2.19: Implement Resource Scope Parsing
- [x] **Write Tests**: Add tests for resource management
  - Basic resource: `resource file: FileHandle = openFile("data.txt") { cleanup: closeFile; };`
- [x] **Implement**: Implement resource scope parsing
- [x] **Verify**: Resource scope tests pass

**Notes:**
```
```

### Task 2.20: Parser Integration and Error Messages
- [x] **Write Tests**: Add integration tests for complete programs
  - Hello world
  - Function with contracts
  - HTTP server example
- [x] **Write Tests**: Add error message tests
  - Missing semicolon error
  - Missing brace error
  - Unlabeled argument error
- [x] **Implement**: Polish error messages with source locations
- [x] **Verify**: All parser tests pass
- [x] **Verify**: `cargo test` shows >80% coverage for parser module

**Notes:**
```
```

---

## Phase 3: Pipeline Integration

### Task 3.1: Update Pipeline to Use V2 Lexer/Parser
- [x] **Write Tests**: Add integration test that compiles a V2 file end-to-end
- [x] **Implement**: Update `src/pipeline/mod.rs` Phase 1 to use new lexer/parser
- [x] **Verify**: Integration test passes

**Notes:**
```
```

### Task 3.2: Verify AST Compatibility
- [x] **Write Tests**: Create test that parses V2 code and verifies AST structure matches expected
- [x] **Implement**: Fix any AST mapping issues discovered
- [x] **Verify**: AST compatibility tests pass

**Notes:**
```
```

### Task 3.3: Verify Semantic Analysis Works
- [x] **Write Tests**: Create test that runs semantic analysis on V2-parsed code
- [x] **Implement**: Fix any issues discovered
- [x] **Verify**: Semantic analysis tests pass

**Notes:**
```
```

### Task 3.4: Verify Full Compilation Works
- [x] **Write Tests**: Create test that compiles V2 code to executable and runs it
- [x] **Implement**: Fix any issues discovered
- [x] **Verify**: Full compilation test passes

**Notes:**
```
```

---

## Phase 4: Example Verification and Makefiles

### Task 4.1: Hello World
- [x] **Verify**: `examples/v2/01-basics/hello_world` compiles and runs.
- [x] **Makefile**: Create and verify Makefile.

### Task 4.2: Constants
- [x] **Verify**: `examples/v2/02-variables/constants` compiles and runs.
- [x] **Makefile**: Create/verify Makefile.
- [x] **Fix**: Resolve panic/crash.

### Task 4.3: Verify Remaining Examples
- [x] `02-variables/let_bindings`
- [x] `02-variables/mutability`
- [ ] **Iterate**: Go through each example folder in `examples/v2`.
- [ ] **Verify**: Ensure it compiles and runs.
- [ ] **Makefile**: Add standard Makefile.
- [ ] **Commit**: Commit after each success.

---

## Phase 5: Test Suite Migration

### Task 5.1: Update Parser Test Inputs
- [x] **Migrate**: Update test input strings in `src/parser/tests.rs` to V2 syntax
- [x] **Verify**: All parser tests pass

**Notes:**
```
```

### Task 5.2: Update Integration Test Inputs
- [x] **Migrate**: Update any integration tests that use V1 syntax
- [x] **Verify**: All integration tests pass

**Notes:**
```
```

### Task 5.3: Final Test Suite Verification
- [x] **Verify**: Run `cargo test` - all 360+ tests pass
- [x] **Verify**: Run `cargo clippy` - no warnings
- [x] **Verify**: Run coverage report - >80% coverage

**Notes:**
```
```

## Phase 6: Cleanup and Documentation

### Task 6.1: Remove V1 Lexer/Parser Code
- [x] **Implement**: Delete old V1 lexer code
- [x] **Implement**: Delete old V1 parser code
- [x] **Verify**: Build succeeds with only V2 code

**Notes:**
```
```

### Task 6.2: Update Documentation
- [ ] **Update**: Update `LANGUAGE_REFERENCE.md` to show V2 syntax
- [ ] **Update**: Update `README.md` examples to V2 syntax
- [ ] **Update**: Update any other documentation files

**Notes:**
```
```

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
| 4 | 4.1 - 4.3 | Example Verification and Makefiles |
| 5 | 5.1 - 5.3 | Test Suite Migration |
| 6 | 6.1 - 6.3 | Cleanup and Documentation |

**Total Tasks:** 38

```