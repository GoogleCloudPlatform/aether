# AetherScript Compiler Technical Design Document

## Table of Contents

1. [Executive Summary](#executive-summary)
2. [Design Goals](#design-goals)
3. [Architecture Overview](#architecture-overview)
4. [Compiler Pipeline](#compiler-pipeline)
5. [Core Components](#core-components)
6. [Memory Management](#memory-management)
7. [Type System Implementation](#type-system-implementation)
8. [Code Generation Strategy](#code-generation-strategy)
9. [Runtime System](#runtime-system)
10. [Foreign Function Interface (FFI)](#foreign-function-interface-ffi)
11. [Error Handling and Diagnostics](#error-handling-and-diagnostics)
12. [Optimization Framework](#optimization-framework)
13. [Testing and Verification](#testing-and-verification)
14. [Performance Considerations](#performance-considerations)
15. [Security Considerations](#security-considerations)
16. [Build System and Distribution](#build-system-and-distribution)

## Executive Summary

The AetherScript compiler is designed to compile a highly explicit, S-expression-based programming language optimized for LLM code generation. The compiler prioritizes reliability and performance while providing seamless interoperability with C, C++, Rust, and Go through a sophisticated FFI system.

Key features:
- Multi-stage compilation pipeline with extensive verification
- LLVM-based backend for optimal code generation
- Zero-copy interop with native languages
- Comprehensive error recovery and diagnostics
- Formal verification capabilities through metadata assertions

## Design Goals

### Primary Goals
1. **Reliability**: Zero undefined behavior, comprehensive error handling, formal verification support
2. **Performance**: Near-native execution speed, minimal runtime overhead, efficient memory usage
3. **Interoperability**: Seamless FFI with C/C++/Rust/Go, shared memory model, zero-copy data exchange
4. **Verifiability**: Full support for preconditions, postconditions, and invariants
5. **Determinism**: Reproducible builds, predictable performance characteristics

### Non-Goals
- Human-friendly syntax (explicitly designed for LLM generation)
- Dynamic typing or runtime reflection
- Garbage collection (using deterministic memory management instead)

## Architecture Overview

```
┌─────────────────────────────────────────────────────────────┐
│                     Frontend                                 │
├─────────────────┬───────────────┬───────────────────────────┤
│   Lexer         │    Parser     │    Semantic Analyzer      │
│ (S-Expression)  │  (AST Builder)│  (Type Checker/Resolver)  │
└─────────────────┴───────────────┴───────────────────────────┘
                            │
                            ▼
┌─────────────────────────────────────────────────────────────┐
│                  Middle-End (MIR)                            │
├─────────────────┬───────────────┬───────────────────────────┤
│ MIR Generator   │  Optimizer    │   Metadata Processor      │
│                 │  (Multiple     │   (Intent/Contract        │
│                 │   Passes)      │    Verification)          │
└─────────────────┴───────────────┴───────────────────────────┘
                            │
                            ▼
┌─────────────────────────────────────────────────────────────┐
│                     Backend                                  │
├─────────────────┬───────────────┬───────────────────────────┤
│ LLVM IR Gen     │ Native Code   │    Runtime Library        │
│                 │ Generation    │    Generation             │
└─────────────────┴───────────────┴───────────────────────────┘
```

## Compiler Pipeline

### 1. Lexical Analysis
- **Input**: UTF-8 encoded source files
- **Output**: Token stream
- **Implementation**: Hand-written lexer optimized for S-expressions
- **Features**:
  - Zero-allocation tokenization where possible
  - Precise source location tracking
  - Parallel file processing

### 2. Parsing
- **Input**: Token stream
- **Output**: Abstract Syntax Tree (AST)
- **Implementation**: Recursive descent parser
- **Features**:
  - Error recovery for better diagnostics
  - Incremental parsing support
  - AST validation during construction

### 3. Semantic Analysis
- **Input**: AST
- **Output**: Typed AST with resolved symbols
- **Phases**:
  1. Symbol table construction
  2. Type inference and checking
  3. Contract validation
  4. Module dependency resolution
- **Features**:
  - Bidirectional type checking
  - Generic type instantiation
  - Compile-time constant evaluation

### 4. MIR Generation
- **Input**: Typed AST
- **Output**: Middle Intermediate Representation (MIR)
- **Purpose**: Optimization and analysis platform
- **Features**:
  - SSA form for data flow analysis
  - Control flow graph construction
  - Metadata preservation

### 5. Optimization
- **Input**: MIR
- **Output**: Optimized MIR
- **Passes**:
  1. Dead code elimination
  2. Constant folding/propagation
  3. Inlining (guided by metadata)
  4. Loop optimization
  5. Memory access optimization
  6. Contract assertion optimization

### 6. Code Generation
- **Input**: Optimized MIR
- **Output**: LLVM IR or direct machine code
- **Targets**: x86-64, ARM64, WASM
- **Features**:
  - Efficient FFI thunk generation
  - Stack frame optimization
  - SIMD utilization where applicable

## Core Components

### AST Node Structure
```rust
pub enum ASTNode {
    Module {
        name: Identifier,
        intent: String,
        content: Vec<ModuleContent>,
        source_loc: SourceLocation,
    },
    Function {
        name: Identifier,
        params: Vec<Parameter>,
        return_type: Type,
        metadata: FunctionMetadata,
        body: Block,
        source_loc: SourceLocation,
    },
    // ... other node types
}

pub struct FunctionMetadata {
    intent: Option<String>,
    preconditions: Vec<Assertion>,
    postconditions: Vec<Assertion>,
    algorithm_hint: Option<AlgorithmHint>,
    complexity: Option<ComplexitySpec>,
    throws: Vec<Type>,
}
```

### Type System Core
```rust
pub enum Type {
    Primitive(PrimitiveType),
    Structured { fields: Vec<Field> },
    Array { element: Box<Type>, size: Option<usize> },
    Map { key: Box<Type>, value: Box<Type> },
    Enum { variants: Vec<EnumVariant> },
    Alias { name: String, target: Box<Type> },
    Function { params: Vec<Type>, returns: Box<Type> },
    Pointer { target: Box<Type>, is_mut: bool },
}
```

### Symbol Table
- Hierarchical scope management
- Module-level and function-level tables
- Efficient lookup with caching
- Support for qualified names

## Memory Management

### Strategy
1. **Stack-first allocation**: Prefer stack allocation for all fixed-size types
2. **Region-based allocation**: Group related heap allocations
3. **Reference counting**: For shared mutable data
4. **Linear types**: Optional move semantics for zero-copy operations

### Memory Layout
```
┌─────────────────┐
│   Stack Frame   │
├─────────────────┤
│ Local Variables │ <- Fixed size, known at compile time
├─────────────────┤
│ Temp Allocations│ <- Function-scoped heap allocations
├─────────────────┤
│ Return Address  │
└─────────────────┘

Heap Layout:
┌─────────────────┐
│  Region Header  │
├─────────────────┤
│  Allocation 1   │ <- Grouped by lifetime
│  Allocation 2   │
│      ...        │
└─────────────────┘
```

## Type System Implementation

### Type Checking Algorithm
1. **Bidirectional type checking**:
   - Synthesis: Infer type from expression
   - Checking: Verify expression against expected type

2. **Constraint solving**:
   - Collect constraints during inference
   - Solve using unification algorithm
   - Report meaningful errors on failure

### Type Safety Guarantees
- No implicit conversions
- Explicit nullability
- Array bounds checking (optimized away when possible)
- No uninitialized memory access

## Code Generation Strategy

### LLVM Integration
```rust
pub struct CodeGenerator {
    context: LLVMContext,
    module: LLVMModule,
    builder: LLVMBuilder,
    target_machine: LLVMTargetMachine,
}

impl CodeGenerator {
    pub fn generate_function(&mut self, func: &MIRFunction) -> LLVMValue {
        // 1. Create function prototype
        // 2. Generate entry block
        // 3. Emit precondition checks
        // 4. Generate function body
        // 5. Emit postcondition checks
        // 6. Add debug information
    }
}
```

### Optimization Levels
- **O0**: Debug mode - no optimizations, full debug info
- **O1**: Basic optimizations - dead code elimination, simple inlining
- **O2**: Standard optimizations - loop optimizations, vectorization
- **O3**: Aggressive optimizations - whole program optimization

## Runtime System

### Core Runtime Components
1. **Memory allocator**: Custom allocator with region support
2. **Exception handling**: Zero-cost exceptions using LLVM landingpad
3. **FFI dispatcher**: Efficient foreign function call mechanism
4. **Standard library**: Minimal runtime with essential functions

### Runtime Layout
```
libaether_runtime.a
├── allocator.o      # Memory management
├── exception.o      # Exception handling
├── ffi.o           # FFI support
├── io.o            # Basic I/O
└── startup.o       # Program initialization
```

## Foreign Function Interface (FFI)

### Design Principles
1. **Zero-copy data exchange**: Share memory layouts with C ABI
2. **Type safety**: Compile-time verification of FFI calls
3. **Minimal overhead**: Direct calls without trampolines when possible

### C/C++ Interop
```aetherscript
(DECLARE_EXTERNAL_FUNCTION
  (NAME 'malloc')
  (LIBRARY "libc")
  (RETURNS (TYPE (POINTER_TO 'VOID')))
  (ACCEPTS_PARAMETER (NAME 'size') (TYPE 'SIZE_T'))
  (CALLING_CONVENTION 'C')
)
```

Generated C Header:
```c
// aether_bindings.h
typedef struct {
    float x_coordinate;
    float y_coordinate;
} aether_point_2d;

extern aether_point_2d aether_create_point(float x, float y);
```

### Rust Interop
```rust
// Generated Rust bindings
#[repr(C)]
pub struct AetherPoint2D {
    pub x_coordinate: f32,
    pub y_coordinate: f32,
}

extern "C" {
    fn aether_create_point(x: f32, y: f32) -> AetherPoint2D;
}
```

### Go Interop
```go
// Generated Go bindings
// #include "aether_bindings.h"
import "C"

type Point2D struct {
    X float32
    Y float32
}

func CreatePoint(x, y float32) Point2D {
    p := C.aether_create_point(C.float(x), C.float(y))
    return Point2D{X: float32(p.x_coordinate), Y: float32(p.y_coordinate)}
}
```

### Memory Layout Compatibility
- Structures: C-compatible layout with explicit padding
- Arrays: Contiguous memory with length prefix
- Strings: UTF-8 with null termination for C compatibility
- Maps: Hash table with stable iteration order

## Error Handling and Diagnostics

### Error Categories
1. **Syntax Errors**: Malformed S-expressions
2. **Type Errors**: Type mismatches, undefined types
3. **Contract Violations**: Failed pre/postconditions
4. **Semantic Errors**: Undefined variables, invalid operations
5. **Resource Errors**: Out of memory, file not found

### Error Reporting
```
Error: Type mismatch in function call
  ┌─ user_service.aether:42:15
  │
42│ (CALL_FUNCTION (FUNCTION_REFERENCE 'add_integers')
  │                ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ expected function 'add_integers'
43│   (ARGUMENTS
44│     (ARGUMENT (NAME 'a') (VALUE "not a number"))
  │                                 ^^^^^^^^^^^^^^ expected INTEGER, found STRING
  │
  = help: 'add_integers' expects parameter 'a' to be of type INTEGER
  = note: consider using (TO_INTEGER "not a number") if conversion is intended
```

### Recovery Strategies
- Continue parsing after errors
- Maintain partial type information
- Suggest fixes based on context
- Generate placeholder nodes for IDE support

## Optimization Framework

### Analysis Passes
1. **Data Flow Analysis**: Live variable analysis, reaching definitions
2. **Control Flow Analysis**: Dominance, loop detection
3. **Alias Analysis**: Points-to analysis for optimization
4. **Effect Analysis**: Pure function detection

### Optimization Passes
1. **Early Optimizations**:
   - Constant folding
   - Dead code elimination
   - Simple inlining

2. **Mid-level Optimizations**:
   - Common subexpression elimination
   - Loop invariant code motion
   - Strength reduction

3. **Late Optimizations**:
   - Register allocation
   - Instruction selection
   - Peephole optimization

### Metadata-Guided Optimization
```rust
fn optimize_with_hints(func: &MIRFunction) -> MIRFunction {
    match func.metadata.algorithm_hint {
        Some(AlgorithmHint::TailRecursive) => optimize_tail_calls(func),
        Some(AlgorithmHint::Vectorizable) => auto_vectorize(func),
        Some(AlgorithmHint::Memoizable) => add_memoization(func),
        _ => func.clone()
    }
}
```

## Testing and Verification

### Compiler Testing
1. **Unit Tests**: Test individual compiler components
2. **Integration Tests**: End-to-end compilation tests
3. **Regression Tests**: Prevent reintroduction of bugs
4. **Fuzzing**: Random program generation
5. **Differential Testing**: Compare with reference implementation

### Verification Framework
```rust
pub struct Verifier {
    smt_solver: Z3Solver,
}

impl Verifier {
    pub fn verify_function(&self, func: &TypedFunction) -> VerificationResult {
        // 1. Convert preconditions to SMT formulas
        // 2. Generate verification conditions
        // 3. Check postconditions hold
        // 4. Verify invariants maintained
    }
}
```

### Contract Enforcement
- Debug builds: Runtime assertion checks
- Release builds: Compile-time verification where possible
- Optional runtime checks based on configuration

## Performance Considerations

### Compile-Time Performance
- Parallel compilation at module level
- Incremental compilation support
- Caching of type checking results
- Lazy loading of dependencies

### Runtime Performance
- Zero-cost abstractions
- Inline caching for dynamic dispatch
- Profile-guided optimization support
- SIMD instruction utilization

### Memory Efficiency
- Small AST nodes (64 bytes or less)
- String interning for identifiers
- Compact type representations
- Efficient symbol table implementation

## Security Considerations

### Compiler Security
- Input validation for untrusted code
- Resource limits during compilation
- Sandboxed execution for macros
- Secure handling of file paths

### Runtime Security
- Stack canaries for buffer overflow detection
- ASLR support
- Control flow integrity
- Safe FFI boundary checks

## Build System and Distribution

### Build Configuration
```toml
[build]
compiler = "aether"
version = "0.1.0"
target = "x86_64-unknown-linux-gnu"

[dependencies]
llvm = "17.0"
z3 = "4.12"

[profile.debug]
opt-level = 0
debug = true
assertions = true

[profile.release]
opt-level = 3
lto = true
strip = true
```

### Distribution Formats
1. **Source Distribution**: Complete compiler source
2. **Binary Distribution**: Pre-built compiler binaries
3. **Docker Images**: Containerized build environments
4. **Package Managers**: apt, brew, cargo integration

### Compiler Driver
```bash
aether compile main.aether -o main
aether compile --target=wasm32 web.aether -o web.wasm
aether verify contracts.aether
aether doc generate stdlib/
```

## Implementation Roadmap

### Phase 1: Core Compiler (Months 1-3)
- Lexer and parser
- Basic type checking
- Simple code generation

### Phase 2: Advanced Features (Months 4-6)
- Full type system
- Optimization framework
- FFI implementation

### Phase 3: Production Ready (Months 7-9)
- Performance tuning
- Comprehensive testing
- Documentation
- Standard library

### Phase 4: Extended Features (Months 10-12)
- Advanced optimizations
- Formal verification
- IDE support
- Debugger integration