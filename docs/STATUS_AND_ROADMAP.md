# AetherScript Compiler Status and Roadmap

*Generated: November 2025*

## Table of Contents
1. [What's Implemented](#whats-implemented)
2. [What's NOT Implemented](#whats-not-implemented)
3. [Recommendations](#recommendations)

---

## What's Implemented

### V2 Lexer & Parser
The compiler uses a modern curly-brace syntax similar to Rust/Swift.

```aether
module example {
    func add(a: Int, b: Int) -> Int {
        return {a + b};
    }

    func main() -> Int {
        let result: Int = add(10, 20);
        return result;
    }
}
```

### Semantic Analysis
Full type checking, symbol resolution, and scope management.

```aether
module type_checking {
    func example() -> Int {
        let x: Int = 42;
        let y: Float64 = 3.14;
        // let z: Int = y;  // Error: type mismatch
        return x;
    }
}
```

### MIR (Mid-level IR) Lowering
AST is lowered to a control-flow graph representation for optimization.

### LLVM Backend
Native code generation via LLVM. Supports multiple targets.

```bash
# Compile to native executable
aether-compiler compile program.aes -o program

# Generate LLVM IR
aether-compiler compile program.aes --emit-llvm
```

### Ownership System
Rust-inspired borrow checking with moves and borrows.

```aether
module ownership {
    func take_ownership(s: String) -> Void {
        // s is moved here, caller can no longer use it
    }

    func borrow(s: &String) -> Int {
        // s is borrowed, caller retains ownership
        return 0;
    }

    func main() -> Int {
        var name: String = "Alice";
        borrow(&name);      // Borrow - name still valid
        take_ownership(name); // Move - name no longer valid
        // let x = name;    // Error: use after move
        return 0;
    }
}
```

### Async/Concurrency (Partial)
Thread pool runtime exists but parallel execution within blocks is broken.

```aether
module async_example {
    func compute(n: Int) -> Int {
        return {n * 2};
    }

    func main() -> Int {
        var result1: Int = 0;
        var result2: Int = 0;

        concurrent {
            result1 = compute(10);
            result2 = compute(20);
        }
        // INTENDED: Both computations run in parallel
        // ACTUAL: Statements run sequentially in one spawned task

        return {result1 + result2};
    }
}
```

**Note:** See "Concurrent Block Parallelism" in What's NOT Implemented.

### FFI (C Interop)
Call C functions and pass structs across the FFI boundary.

```aether
module ffi_example {
    @extern("C")
    extern func printf(format: Pointer<Int8>, ...) -> Int;

    @extern("C")
    extern func malloc(size: SizeT) -> Pointer<Void>;

    struct Point {
        x: Float64,
        y: Float64
    }

    @extern("C")
    extern func process_point(p: Point) -> Float64;
}
```

### LSP Server (Basic)
Language Server Protocol support for IDE integration.

**Implemented:**
- Diagnostics (error reporting)
- Hover (type information)
- Go-to-definition

```bash
# Start LSP server
aether-compiler lsp
```

### Optimization Passes

**Implemented:**
- Dead Code Elimination
- Constant Propagation
- Function Inlining

```aether
module optimizable {
    func unused() -> Int {
        return 42;  // DCE removes this
    }

    func constants() -> Int {
        let x: Int = 10;
        let y: Int = 20;
        return {x + y};  // Constant folded to 30
    }
}
```

### Runtime Library

| Module | Features |
|--------|----------|
| `memory` | Allocation, deallocation, reference counting |
| `string` | Creation, concatenation, slicing |
| `collections` | Arrays, maps |
| `io` | File operations, console I/O |
| `http` | HTTP client |
| `json` | JSON parsing and serialization |
| `network` | TCP/UDP sockets |
| `time` | Timestamps, duration |
| `math` | Basic math functions |

---

## What's NOT Implemented

### 1. Formal Verification (Partial - Z3 Backend Working)

The SMT solver backend using Z3 is now functional. The solver can verify mathematical properties.

```aether
module contracts {
    // Z3 can verify properties like: x > 0 => x + 1 > 0
    @pre("x > 0")
    @post("result > x")
    func double_positive(x: Int) -> Int {
        return {x * 2};
    }
}
```

**Current status:**
- ✅ Z3 SMT solver integrated (`src/verification/solver.rs`)
- ✅ Formula conversion working (Int, Bool, comparisons, arithmetic, logic)
- ✅ Quantifiers supported (forall, exists)
- ⚠️ Contract parsing → verification pipeline needs wiring
- ⚠️ Runtime contract enforcement not implemented

**Requires:** Z3 installed (`brew install z3`) and build env vars:
```bash
Z3_SYS_Z3_HEADER=/opt/homebrew/opt/z3/include/z3.h cargo build
```

**Location:** `src/verification/solver.rs` - real Z3 implementation

### 2. Concurrent Block Parallelism (Broken)

Statements inside `concurrent {}` blocks run sequentially, not in parallel.

```aether
module concurrency_issue {
    func slow_compute(n: Int) -> Int {
        // Imagine this takes 1 second
        return {n * 2};
    }

    func main() -> Int {
        var result1: Int = 0;
        var result2: Int = 0;

        concurrent {
            result1 = slow_compute(10);  // Takes 1 second
            result2 = slow_compute(20);  // Takes 1 second
        }
        // EXPECTED: ~1 second total (parallel)
        // ACTUAL: ~2 seconds total (sequential)

        return {result1 + result2};
    }
}
```

**Root cause:** The codegen outlines the entire block as ONE task function, spawns it, then immediately awaits. Statements inside execute sequentially.

**Location:** `src/llvm_backend/mod.rs:908-968` - Terminator::Concurrent handling

**What's needed:**
1. Parse individual statements as separate tasks
2. Spawn all tasks first
3. Await all at block end

**Also missing: Fire-and-forget**

No way to spawn a task without waiting for it:

```aether
// NOT SUPPORTED - would need something like:
spawn { background_task(); }  // Don't wait
// or
let handle = async { compute(); };  // Get handle, await later
```

### 4. Enum Methods (Broken)

Calling methods on enum values fails with "Expected integer value for switch".

```aether
module enum_methods {
    enum Status {
        Active,
        Inactive,
        Pending
    }

    impl Status {
        func is_active(self) -> Bool {
            match self {
                Status::Active => true,
                _ => false
            }
        }
    }

    func main() -> Int {
        let s: Status = Status::Active;
        // let active: Bool = s.is_active();  // FAILS: codegen error
        return 0;
    }
}
```

**Error:** `Expected integer value for switch`

**Location:** LLVM backend enum dispatch logic

### 5. Result Type Parsing (Broken)

The `Result<T, E>` type fails to parse correctly.

```aether
module result_example {
    enum Error {
        NotFound,
        InvalidInput
    }

    // FAILS: Parser error
    // func find(id: Int) -> Result<String, Error> {
    //     when {id > 0} {
    //         return Ok("found");
    //     } else {
    //         return Err(Error::NotFound);
    //     }
    // }

    // Workaround: Use custom enum
    enum FindResult {
        Found(String),
        NotFound
    }
}
```

**Location:** Parser handling of generic enum types

### 6. Array/Tuple Aggregates (TODO)

Array literals and tuple construction have incomplete codegen.

```aether
module aggregates {
    func main() -> Int {
        // Basic arrays work
        let arr: Array<Int> = [1, 2, 3, 4, 5];

        // Complex aggregates may fail
        // let matrix: Array<Array<Int>> = [[1, 2], [3, 4]];  // TODO
        // let tuple: (Int, String) = (42, "hello");  // TODO

        return arr[0];
    }
}
```

**Location:** `src/llvm_backend/mod.rs:1961-1975`

### 7. Exception Handling (TODO)

Unwind paths for exceptions are not implemented.

```aether
module exceptions {
    func might_fail() -> Int {
        // throw/catch not implemented
        // Panics terminate the program
        return 0;
    }
}
```

**Location:** `src/llvm_backend/mod.rs:843`

### 8. Generics/Templates (Missing)

No support for generic type parameters.

```aether
module generics {
    // NOT SUPPORTED
    // func identity<T>(x: T) -> T {
    //     return x;
    // }

    // NOT SUPPORTED
    // struct Container<T> {
    //     value: T
    // }

    // Workaround: Write specific versions
    func identity_int(x: Int) -> Int {
        return x;
    }

    func identity_string(x: String) -> String {
        return x;
    }
}
```

### 9. Traits/Interfaces (TODO)

Interface-based polymorphism is not implemented.

```aether
module traits {
    // NOT SUPPORTED
    // trait Printable {
    //     func to_string(self) -> String;
    // }

    // impl Printable for Int {
    //     func to_string(self) -> String {
    //         return "number";
    //     }
    // }
}
```

**Location:** `src/types/mod.rs:1181` - "TODO: Implement proper trait checking"

### 10. REPL Mode (Missing)

No interactive Read-Eval-Print Loop.

```bash
# NOT AVAILABLE
# aether repl
# > let x = 42
# > x + 10
# 52
```

### 11. Debugger Integration (Limited)

DWARF debug info is generated but tooling is limited.

```bash
# Debug info is emitted but integration is basic
aether-compiler compile --debug program.aes -o program
lldb ./program  # Works but source mapping may be incomplete
```

### 12. Package Manager (Incomplete)

Registry and resolver code exists but is not functional.

```bash
# NOT AVAILABLE
# aether pkg init
# aether pkg add some-library
# aether pkg publish
```

**Location:** `src/package/` - code exists but not integrated

---

## Recommendations

### High Priority: Compiler Completeness

#### 1. Fix Enum Methods Codegen

**Problem:** Methods on enum values fail at code generation.

**Impact:** Blocks idiomatic enum usage patterns.

**Example that should work:**
```aether
module fixed_enums {
    enum Option {
        Some(Int),
        None
    }

    impl Option {
        func unwrap_or(self, default: Int) -> Int {
            match self {
                Option::Some(v) => v,
                Option::None => default
            }
        }

        func is_some(self) -> Bool {
            match self {
                Option::Some(_) => true,
                Option::None => false
            }
        }
    }

    func main() -> Int {
        let opt: Option = Option::Some(42);
        return opt.unwrap_or(0);  // Should return 42
    }
}
```

**Suggested fix:** Review switch codegen in `src/llvm_backend/mod.rs` for enum discriminant handling.

---

#### 2. Fix Result Type Parsing

**Problem:** `Result<T, E>` generic syntax fails to parse.

**Impact:** Blocks standard error handling patterns.

**Example that should work:**
```aether
module fixed_result {
    enum Error {
        IoError(String),
        ParseError(String)
    }

    func read_file(path: String) -> Result<String, Error> {
        when {path == ""} {
            return Err(Error::IoError("empty path"));
        } else {
            return Ok("file contents");
        }
    }

    func main() -> Int {
        let result: Result<String, Error> = read_file("test.txt");
        match result {
            Ok(contents) => {
                // use contents
                return 0;
            },
            Err(e) => {
                return 1;
            }
        }
    }
}
```

**Suggested fix:** Review generic type parsing in `src/parser/v2/mod.rs`.

---

#### 3. Implement Generics

**Problem:** No generic functions or types.

**Impact:** Forces code duplication, limits stdlib expressiveness.

**Example that should work:**
```aether
module generics {
    // Generic function
    func swap<T>(a: &T, b: &T) -> Void {
        let temp: T = *a;
        *a = *b;
        *b = temp;
    }

    // Generic struct
    struct Pair<A, B> {
        first: A,
        second: B
    }

    // Generic enum
    enum Option<T> {
        Some(T),
        None
    }

    func main() -> Int {
        var x: Int = 1;
        var y: Int = 2;
        swap<Int>(&x, &y);

        let pair: Pair<Int, String> = Pair { first: 42, second: "hello" };
        let opt: Option<Int> = Option::Some(10);

        return x;  // Returns 2
    }
}
```

**Implementation approach:**
1. Add type parameters to AST nodes
2. Implement monomorphization in semantic analysis
3. Generate specialized code for each instantiation

---

### Medium Priority: Developer Experience

#### 4. Enhance LSP

**Current state:** Basic diagnostics, hover, go-to-definition.

**Missing features:**

```
Auto-completion:
  let x: St|  ->  [String, Status, Struct...]

Rename refactoring:
  Rename 'oldName' to 'newName' across all files

Find references:
  Where is 'functionName' called?

Signature help:
  func foo(a: Int, b: String) -> Bool
           ^cursor here shows parameter info
```

---

#### 5. REPL Mode

**Example session:**
```
$ aether repl
Aether REPL v0.1.0
> let x = 42
x: Int = 42

> x * 2
84

> func double(n: Int) -> Int { return {n * 2}; }
double: (Int) -> Int

> double(x)
84

> :type double
(Int) -> Int

> :quit
```

---

#### 6. Better Error Messages

**Current:**
```
Error: Type mismatch at line 10
```

**Improved:**
```
error[E0308]: mismatched types
  --> src/main.aes:10:15
   |
10 |     let x: Int = "hello";
   |            ---   ^^^^^^^ expected `Int`, found `String`
   |            |
   |            expected due to this
   |
help: consider using parse to convert
   |
10 |     let x: Int = "hello".parse();
   |                         ++++++++
```

---

### Lower Priority: Nice to Have

#### 7. Trait System

```aether
module traits {
    trait Display {
        func fmt(self) -> String;
    }

    trait Add<Rhs> {
        type Output;
        func add(self, rhs: Rhs) -> Self::Output;
    }

    struct Point {
        x: Float64,
        y: Float64
    }

    impl Display for Point {
        func fmt(self) -> String {
            return "Point(x, y)";  // string interpolation would help
        }
    }

    impl Add<Point> for Point {
        type Output = Point;
        func add(self, rhs: Point) -> Point {
            return Point {
                x: {self.x + rhs.x},
                y: {self.y + rhs.y}
            };
        }
    }
}
```

---

#### 8. Async/Await Syntax Sugar

**Current:**
```aether
func fetch_data() -> Int {
    var result: Int = 0;
    concurrent {
        result = slow_operation();
    }
    return result;
}
```

**Proposed:**
```aether
async func fetch_data() -> Int {
    let result: Int = await slow_operation();
    return result;
}

func main() -> Int {
    let data: Int = await fetch_data();
    return data;
}
```

---

#### 9. Package Manager

```bash
# Initialize a new package
$ aether pkg init my-library
Created my-library/
  - aether.toml
  - src/lib.aes

# Add a dependency
$ aether pkg add json-parser
Added json-parser v1.2.0

# Build and run
$ aether build
$ aether run

# Publish to registry
$ aether pkg publish
Published my-library v0.1.0
```

**aether.toml:**
```toml
[package]
name = "my-library"
version = "0.1.0"
authors = ["Developer <dev@example.com>"]

[dependencies]
json-parser = "1.2.0"
http-client = { version = "2.0", features = ["tls"] }

[dev-dependencies]
test-framework = "0.5"
```

---

#### 10. Formal Verification

```aether
module verified {
    // Pre/post conditions checked at compile time via SMT solver
    @pre("n >= 0")
    @post("result >= n")
    @post("result == n * 2")
    func double(n: Int) -> Int {
        return {n * 2};
    }

    // Loop invariants
    @invariant("sum >= 0")
    @invariant("i <= n")
    func sum_to_n(n: Int) -> Int {
        var sum: Int = 0;
        var i: Int = 0;
        while {i < n} {
            sum = {sum + i};
            i = {i + 1};
        }
        return sum;
    }

    // Termination proofs
    @decreases("n")
    func factorial(n: Int) -> Int {
        when {n <= 1} {
            return 1;
        } else {
            return {n * factorial({n - 1})};
        }
    }
}
```

---

## Summary

| Priority | Feature | Effort | Impact |
|----------|---------|--------|--------|
| **High** | Fix Concurrent Parallelism | Medium | Enables actual concurrency |
| **High** | Fix Enum Methods | Medium | Unblocks enum patterns |
| **High** | Fix Result Type | Low | Unblocks error handling |
| **High** | Implement Generics | High | Foundation for stdlib |
| **Medium** | Wire Contract→SMT Pipeline | Medium | Full verification |
| **Medium** | Enhance LSP | Medium | Developer productivity |
| **Medium** | REPL Mode | Medium | Learning/prototyping |
| **Medium** | Better Errors | Medium | Developer experience |
| **Low** | Trait System | High | Polymorphism |
| **Low** | Async/Await Sugar | Medium | Cleaner async code |
| **Low** | Package Manager | High | Ecosystem growth |

**Recently Completed:**
- ✅ Z3 SMT solver backend (formal verification foundation)

---

## Getting Started

```bash
# Install Z3 (required for formal verification)
brew install z3  # macOS

# Build the compiler (with Z3 support)
Z3_SYS_Z3_HEADER=/opt/homebrew/opt/z3/include/z3.h cargo build --release

# Or build without verification features
cargo build --release

# Compile an example
./target/release/aether-compiler compile examples/v2/01-basics/hello_world/main.aes -o hello

# Run it
./hello
```

For more examples, see the `examples/v2/` directory.
