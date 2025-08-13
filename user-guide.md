# AetherScript Language Reference Manual

**Version 1.0 (LLM-First Edition)**

**Motto:** *Explicitness is Clarity. Intent is Paramount. Verification is Built-In.*

### Table of Contents

1.  **Introduction to AetherScript**
    *   1.1. Purpose and Vision
    *   1.2. Core Design Philosophy
    *   1.3. The "User": LLM as Code Generator
    *   1.4. Benefits for LLM-Driven Development
    *   1.5. LLM-First Feature Overview
2.  **Lexical Structure**
    *   2.1. Character Set
    *   2.2. S-Expression Syntax
    *   2.3. Keywords
    *   2.4. Identifiers
    *   2.5. Literals
        *   2.5.1. Integer Literals
        *   2.5.2. Floating-Point Literals
        *   2.5.3. String Literals
        *   2.5.4. Boolean Literals
        *   2.5.5. The Null Value
    *   2.6. Comments (Annotation for Generators)
    *   2.7. Whitespace
3.  **Fundamental Constructs**
    *   3.1. Modules
        *   3.1.1. Defining a Module: `DEFINE_MODULE`
        *   3.1.2. Exporting Definitions: `EXPORT_FUNCTION`, `EXPORT_TYPE`, `EXPORT_CONSTANT`
        *   3.1.3. Importing Modules: `IMPORT_MODULE`
    *   3.2. Declarations
        *   3.2.1. Variable Declaration: `DECLARE_VARIABLE`
        *   3.2.2. Constant Declaration: `DECLARE_CONSTANT`
    *   3.3. Types
        *   3.3.1. Primitive Types
        *   3.3.2. Structured Types (Records/Structs): `DEFINE_TYPE`, `CONSTRUCT`
        *   3.3.3. Array Types: `ARRAY_LITERAL`, `ARRAY_ACCESS`, `ARRAY_LENGTH`
        *   3.3.4. Map Types (Dictionaries): `MAP_LITERAL`, `MAP_ACCESS`, `HAS_MAP_KEY`
        *   3.3.5. Enumeration Types: `DEFINE_ENUMERATION_TYPE`
        *   3.3.6. Type Aliasing: `DEFINE_TYPE_ALIAS`
        *   3.3.7. Pointer Types: `POINTER_TO`, `DEREFERENCE`, `ADDRESS_OF`
    *   3.4. Functions
        *   3.4.1. Defining Functions: `DEFINE_FUNCTION`
        *   3.4.2. Parameters: `ACCEPTS_PARAMETER`
        *   3.4.3. Return Types: `RETURNS`
        *   3.4.4. Function Body: `BODY`
        *   3.4.5. Calling Functions: `CALL_FUNCTION`
        *   3.4.6. Return Statements: `RETURN_VALUE`, `RETURN_VOID`
4.  **Expressions**
    *   4.1. General Structure: `(OPERATOR operand1 operand2 ...)`
    *   4.2. Arithmetic Expressions: `EXPRESSION_ADD`, `EXPRESSION_SUBTRACT`, etc.
    *   4.3. Comparison Predicates: `PREDICATE_EQUALS`, `PREDICATE_NOT_EQUALS`, etc.
    *   4.4. Logical Expressions: `LOGICAL_AND`, `LOGICAL_OR`, `LOGICAL_NOT`
    *   4.5. String Operations: `STRING_CONCAT`, `STRING_LENGTH`, `SUBSTRING`, etc.
    *   4.6. Type Conversion: `CAST_TO_TYPE`
    *   4.7. Accessing Composite Data
        *   4.7.1. Structure Field Access: `FIELD_ACCESS`
        *   4.7.2. Array Element Access
        *   4.7.3. Map Value Access
5.  **Statements and Control Flow**
    *   5.1. Assignment: `ASSIGN`
    *   5.2. Conditional Execution: `IF_CONDITION`
    *   5.3. Looping Constructs
        *   5.3.1. While Loop: `WHILE_LOOP`
        *   5.3.2. For Loop: `FOR_LOOP`
        *   5.3.3. For-Each Loop: `FOR_EACH_LOOP`
    *   5.4. Loop Control: `BREAK_LOOP`, `CONTINUE_LOOP`
6.  **Error Handling and LLM-Optimized Error System**
    *   6.1. Structured Error Handling: `TRY_EXECUTE`
    *   6.2. Catching Exceptions: `CATCH_EXCEPTION`
    *   6.3. Finalization: `FINALLY_EXECUTE`
    *   6.4. Throwing Exceptions: `THROW_EXCEPTION`
    *   6.5. Defining Custom Error Types
    *   6.6. LLM-Optimized Error Features
        *   6.6.1. Structured Error Format
        *   6.6.2. Auto-Fix Suggestions
        *   6.6.3. Intent Analysis for Errors
        *   6.6.4. Partial Compilation Support
        *   6.6.5. Error Recovery and Continuation
7.  **Enhanced Verification System**
    *   7.1. Contract-Based Programming
        *   7.1.1. Preconditions: `PRECONDITION`
        *   7.1.2. Postconditions: `POSTCONDITION`
        *   7.1.3. Invariants: `INVARIANT`
        *   7.1.4. Proof Hints and Obligations
    *   7.2. Behavioral Specifications
        *   7.2.1. Function Behavior: `SIDE_EFFECTS`
        *   7.2.2. Side Effect Declarations
        *   7.2.3. Determinism and Thread Safety
    *   7.3. SMT Solver Integration
        *   7.3.1. Verification Methods: `VERIFICATION_METHOD`
        *   7.3.2. Formal Verification with Z3
        *   7.3.3. Proof Generation and Validation
    *   7.4. Contract Propagation
        *   7.4.1. Inter-function Contract Flow
        *   7.4.2. Module-level Contracts
        *   7.4.3. Contract Inheritance
8.  **Resource Management**
    *   8.1. Resource Scopes: `RESOURCE_SCOPE`
        *   8.1.1. Resource Acquisition and Release
        *   8.1.2. Cleanup Ordering and Guarantees
        *   8.1.3. Exception-Safe Resource Management
    *   8.2. Resource Contracts
        *   8.2.1. Resource Specifications: `RESOURCE_CONTRACT`
        *   8.2.2. Memory and Time Limits
        *   8.2.3. Resource Leak Detection
    *   8.3. Advanced Resource Features
        *   8.3.1. Nested Resource Scopes
        *   8.3.2. Resource Usage Analysis
        *   8.3.3. Performance Monitoring
9. **Intent Specification**
    *   9.1. Purpose of Intent in LLM-First Design
    *   9.2. Intent Tags: `INTENT`
    *   9.3. Side Effect Declarations
10. **Foreign Function Interface (FFI) and Interoperability**
    *   10.1. Overview and Design Principles
    *   10.2. Declaring External Functions: `DECLARE_EXTERNAL_FUNCTION`
    *   10.3. External Type Mapping
    *   10.4. Memory Management Across FFI Boundaries
    *   10.5. C/C++ Interoperability
    *   10.6. Rust Interoperability
    *   10.7. Go Interoperability
    *   10.8. Calling AetherScript from External Code
    *   10.9. FFI Safety and Best Practices
11. **Standard Library (LLM-Optimized)**
    *   11.1. I/O Operations with Resource Management
    *   11.2. Collection Utilities
    *   11.3. Mathematical Functions with Contracts
    *   11.4. Date/Time Manipulation
    *   11.5. Networking Primitives with Safety
    *   11.6. Concurrency Primitives
12. **Complete Examples**
    *   12.1. Basic Function with Contracts
    *   12.2. Resource-Managed File Processing
    *   12.3. Multi-Module Application Example
    *   12.4. FFI Integration with Safety Contracts
    *   12.5. LLM Workflow Examples
13. **AetherScript Grammar (Extended BNF)**

---

### 1. Introduction to AetherScript

#### 1.1. Purpose and Vision

AetherScript is a programming language designed from the ground up to be **optimally generated by Large Language Models (LLMs)**. In the era of AI-driven development where expressing intent in natural language can materialize into executable code, AetherScript serves as an intermediate representation between human intent and machine execution.

Unlike traditional programming languages designed for human readability and writability, AetherScript prioritizes:
- **LLM comprehension and generation reliability**
- **Built-in verification and safety guarantees**
- **Intent-driven semantic richness**
- **Automatic error recovery and suggestions**

#### 1.2. Core Design Philosophy

AetherScript is built upon these foundational principles:

*   **Extreme Explicitness:** Every operation, type, scope, and behavior is explicitly stated, eliminating ambiguity for LLM generators and compilers.
*   **Verification-First:** Contracts, invariants, and behavioral specifications are first-class language constructs, enabling formal verification and automatic safety guarantees.
*   **Intent-Driven Syntax:** Keywords and structures directly reflect the *purpose* and *intent* of code, not just mechanical operations, facilitating natural language to code translation.
*   **Error-Aware:** The error system is designed for LLM consumption, providing structured feedback with auto-fix suggestions and partial compilation support.
*   **Resource-Safe:** Deterministic resource management prevents common classes of bugs and enables predictable performance characteristics.

#### 1.3. The "User": LLM as Code Generator

The primary "user" of AetherScript is an LLM. This manual serves as both:
1. A specification for LLM training and fine-tuning
2. A reference for LLM-generated code validation
3. A guide for human developers integrating with LLM-generated AetherScript

Humans interact with AetherScript primarily through:
- **Intent specification** in natural language
- **High-level code review** and verification
- **Tool development** for the AetherScript ecosystem

#### 1.4. Benefits for LLM-Driven Development

*   **Reduced Generation Errors:** Explicit syntax and semantic richness guide LLMs to produce correct code
*   **Automatic Verification:** Built-in contracts catch errors at compile-time, reducing runtime failures
*   **Intent Preservation:** Semantic metadata ensures generated code matches intended behavior
*   **Iterative Refinement:** Structured error feedback enables LLMs to improve code through multiple iterations
*   **Performance Predictability:** Resource contracts and complexity annotations enable performance guarantees

#### 1.5. LLM-First Feature Overview

AetherScript includes several features specifically designed for LLM code generation:

**Enhanced Verification System:**
- Contract-based programming with automatic proof generation
- SMT solver integration for formal verification
- Behavioral specifications for function properties
- Contract propagation across function boundaries

**LLM-Optimized Error System:**
- Structured error format designed for LLM consumption
- Auto-fix suggestions with example code
- Intent mismatch detection and correction
- Partial compilation support for error recovery

**Resource Management:**
- Deterministic resource scopes with guaranteed cleanup
- Resource contracts with memory and time limits
- Exception-safe resource handling
- Automatic leak detection and prevention

---

### 2. Lexical Structure

#### 2.1. Character Set
AetherScript source code is encoded in UTF-8, supporting international characters in string literals and comments.

#### 2.2. S-Expression Syntax
AetherScript uses S-expressions for uniform, unambiguous syntax:

`(KEYWORD_OR_OPERATOR argument1 argument2 ... argumentN)`

This structure simplifies:
- **LLM parsing and generation**
- **Compiler implementation**
- **Tool development**
- **Syntax validation**

#### 2.3. Keywords
Keywords define language constructs and are written in `UPPERCASE_SNAKE_CASE`:
- **Basic:** `DEFINE_MODULE`, `DEFINE_FUNCTION`, `DECLARE_VARIABLE`
- **Verification:** `PRECONDITION`, `POSTCONDITION`, `SIDE_EFFECTS`
- **Resources:** `RESOURCE_SCOPE`, `RESOURCE_CONTRACT`
- **Error Handling:** `TRY_EXECUTE`, `CATCH_EXCEPTION`, `STRUCTURED_ERROR`

#### 2.4. Identifiers
Identifiers use `lowercase_snake_case` for consistency:
- Must start with lowercase letter (`a-z`)
- Can contain letters, digits, underscores
- Examples: `user_name`, `calculate_distance`, `file_processor`

#### 2.5. Literals

##### 2.5.1. Integer Literals
Whole numbers with optional sign: `123`, `0`, `-42`

##### 2.5.2. Floating-Point Literals
Decimal numbers: `3.14159`, `-0.001`, `1.0`, `2.5E-10`

##### 2.5.3. String Literals
UTF-8 text in double quotes: `"Hello, AetherScript!"`, `"Error: File not found"`
Escape sequences: `\n`, `\t`, `\\`, `\"`

##### 2.5.4. Boolean Literals
Truth values: `TRUE`, `FALSE`

##### 2.5.5. The Null Value
Absence of value: `NULL_LITERAL`

#### 2.6. Comments
Single-line comments for human annotation:
```aetherscript
; This is a comment explaining the code for human readers
(DECLARE_VARIABLE (NAME "counter") (TYPE INT)) ; Variable for iteration counting
```

#### 2.7. Whitespace
Whitespace separates tokens and can be used for human-readable formatting without semantic significance.

---

### 3. Fundamental Constructs

#### 3.1. Modules

##### 3.1.1. Defining a Module: `DEFINE_MODULE`
```aetherscript
(DEFINE_MODULE
  (NAME "module_name")
  (INTENT "High-level description of module purpose")
  (CONTENT
    ; Type definitions, functions, constants, etc.
  )
)
```

##### 3.1.2. Exporting Definitions
```aetherscript
(EXPORT_FUNCTION "function_name")
(EXPORT_TYPE "type_name")
(EXPORT_CONSTANT "constant_name")
```

##### 3.1.3. Importing Modules: `IMPORT_MODULE`
```aetherscript
(IMPORT_MODULE "module_name")
(IMPORT_MODULE "long_module_name" (ALIAS "short"))
```

#### 3.2. Declarations

##### 3.2.1. Variable Declaration: `DECLARE_VARIABLE`
```aetherscript
(DECLARE_VARIABLE
  (NAME "variable_name")
  (TYPE type_specifier)
  (INITIAL_VALUE expression)
  (INTENT "Purpose of this variable")
)
```

##### 3.2.2. Constant Declaration: `DECLARE_CONSTANT`
```aetherscript
(DECLARE_CONSTANT
  (NAME "constant_name")
  (TYPE type_specifier)
  (VALUE literal_or_expression)
  (INTENT "Purpose of this constant")
)
```

#### 3.3. Types

##### 3.3.1. Primitive Types
- `INT`: Integer numbers
- `FLOAT`: Floating-point numbers
- `STRING`: Text sequences
- `BOOL`: Boolean values
- `VOID`: No return value

##### 3.3.2. Structured Types: `DEFINE_TYPE`
```aetherscript
(DEFINE_TYPE
  (NAME "point_2d")
  (STRUCTURED_TYPE
    (FIELD (NAME "x") (TYPE FLOAT))
    (FIELD (NAME "y") (TYPE FLOAT))
  )
  (INTENT "Represents a point in 2D space")
)
```

Creating instances:
```aetherscript
(CONSTRUCT "point_2d"
  (FIELD_VALUE (NAME "x") (VALUE 1.0))
  (FIELD_VALUE (NAME "y") (VALUE 2.0))
)
```

##### 3.3.3. Array Types
```aetherscript
; Array literal
(ARRAY_LITERAL (TYPE (ARRAY INT 5)) 1 2 3 4 5)

; Array access
(ARRAY_ACCESS (VARIABLE_REFERENCE "my_array") (INTEGER_LITERAL 0))

; Array length
(ARRAY_LENGTH (VARIABLE_REFERENCE "my_array"))
```

##### 3.3.4. Map Types
```aetherscript
; Map literal
(MAP_LITERAL (KEY_TYPE STRING) (VALUE_TYPE INT)
  (ENTRY (KEY "alice") (VALUE 30))
  (ENTRY (KEY "bob") (VALUE 25))
)

; Map access
(MAP_ACCESS (VARIABLE_REFERENCE "user_ages") (STRING_LITERAL "alice"))
```

#### 3.4. Functions

##### 3.4.1. Defining Functions: `DEFINE_FUNCTION`
```aetherscript
(DEFINE_FUNCTION
  (NAME "function_name")
  (INTENT "Detailed description of function purpose and behavior")
  (ACCEPTS_PARAMETER (NAME "param1") (TYPE INT) (INTENT "Purpose of param1"))
  (ACCEPTS_PARAMETER (NAME "param2") (TYPE STRING) (INTENT "Purpose of param2"))
  (RETURNS (TYPE FLOAT))

  ; Verification and metadata
  (PRECONDITION
    (PREDICATE_GREATER_THAN (VARIABLE_REFERENCE "param1") (INTEGER_LITERAL 0))
    (PROOF_HINT "param1 must be positive"))

  (POSTCONDITION
    (PREDICATE_GREATER_THAN (VARIABLE_REFERENCE "RETURNED_VALUE") (FLOAT_LITERAL 0.0))
    (PROOF_HINT "Result is always positive"))

  (SIDE_EFFECTS NONE)

  (BODY
    ; Function implementation
    (RETURN_VALUE expression)
  )
)
```

##### 3.4.5. Calling Functions: `CALL_FUNCTION`
```aetherscript
(CALL_FUNCTION "function_name"
  (ARGUMENTS
    (INTEGER_LITERAL 42)
    (STRING_LITERAL "hello")
  )
)
```

---

### 4. Expressions

#### 4.1. General Structure
All expressions use prefix notation: `(OPERATOR operand1 operand2 ...)`

#### 4.2. Arithmetic Expressions
```aetherscript
(EXPRESSION_ADD (VARIABLE_REFERENCE "x") (INTEGER_LITERAL 5))
(EXPRESSION_SUBTRACT (VARIABLE_REFERENCE "a") (VARIABLE_REFERENCE "b"))
(EXPRESSION_MULTIPLY (FLOAT_LITERAL 3.14) (VARIABLE_REFERENCE "radius"))
(EXPRESSION_DIVIDE (VARIABLE_REFERENCE "total") (VARIABLE_REFERENCE "count"))
(EXPRESSION_MODULO (VARIABLE_REFERENCE "number") (INTEGER_LITERAL 10))
```

#### 4.3. Comparison Predicates
```aetherscript
(PREDICATE_EQUALS (VARIABLE_REFERENCE "status") (STRING_LITERAL "ready"))
(PREDICATE_NOT_EQUALS (VARIABLE_REFERENCE "count") (INTEGER_LITERAL 0))
(PREDICATE_LESS_THAN (VARIABLE_REFERENCE "age") (INTEGER_LITERAL 18))
(PREDICATE_GREATER_THAN_OR_EQUAL_TO (VARIABLE_REFERENCE "score") (INTEGER_LITERAL 90))
```

#### 4.4. Logical Expressions
```aetherscript
(LOGICAL_AND
  (PREDICATE_GREATER_THAN (VARIABLE_REFERENCE "age") (INTEGER_LITERAL 18))
  (PREDICATE_EQUALS (VARIABLE_REFERENCE "citizenship") (STRING_LITERAL "US")))

(LOGICAL_OR
  (PREDICATE_EQUALS (VARIABLE_REFERENCE "role") (STRING_LITERAL "admin"))
  (PREDICATE_EQUALS (VARIABLE_REFERENCE "role") (STRING_LITERAL "moderator")))

(LOGICAL_NOT (VARIABLE_REFERENCE "is_disabled"))
```

#### 4.7. Accessing Composite Data

##### 4.7.1. Structure Field Access
```aetherscript
(FIELD_ACCESS (VARIABLE_REFERENCE "point") "x")
```

---

### 5. Statements and Control Flow

#### 5.1. Assignment: `ASSIGN`
```aetherscript
(ASSIGN (TARGET "variable_name") (SOURCE expression))
```

#### 5.2. Conditional Execution: `IF_CONDITION`
```aetherscript
(IF_CONDITION (PREDICATE_GREATER_THAN (VARIABLE_REFERENCE "temperature") (FLOAT_LITERAL 30.0))
  (THEN_EXECUTE
    (CALL_FUNCTION "log_warning" (ARGUMENTS (STRING_LITERAL "High temperature!"))))
  (ELSE_EXECUTE
    (CALL_FUNCTION "log_info" (ARGUMENTS (STRING_LITERAL "Normal temperature")))))
```

#### 5.3. Looping Constructs

##### 5.3.1. While Loop: `WHILE_LOOP`
```aetherscript
(WHILE_LOOP
  (CONDITION (PREDICATE_LESS_THAN (VARIABLE_REFERENCE "i") (INTEGER_LITERAL 10)))
  (INVARIANT
    (PREDICATE_GREATER_THAN_OR_EQUAL_TO (VARIABLE_REFERENCE "i") (INTEGER_LITERAL 0))
    (PROOF_HINT "i remains non-negative"))
  (BODY
    (CALL_FUNCTION "process_item" (ARGUMENTS (VARIABLE_REFERENCE "i")))
    (ASSIGN (TARGET "i") (SOURCE (EXPRESSION_ADD (VARIABLE_REFERENCE "i") (INTEGER_LITERAL 1)))))
)
```

##### 5.3.2. For Loop: `FOR_LOOP`
```aetherscript
(FOR_LOOP
  (INIT (DECLARE_VARIABLE (NAME "i") (INITIAL_VALUE (INTEGER_LITERAL 0))))
  (CONDITION (PREDICATE_LESS_THAN (VARIABLE_REFERENCE "i") (INTEGER_LITERAL 10)))
  (UPDATE (ASSIGN (TARGET "i") (SOURCE (EXPRESSION_ADD (VARIABLE_REFERENCE "i") (INTEGER_LITERAL 1)))))
  (BODY
    (CALL_FUNCTION "process_item" (ARGUMENTS (VARIABLE_REFERENCE "i"))))
)
```

---

### 6. Error Handling and LLM-Optimized Error System

#### 6.1. Structured Error Handling: `TRY_EXECUTE`
```aetherscript
(TRY_EXECUTE
  (PROTECTED_BLOCK
    ; Code that might throw exceptions
    (CALL_FUNCTION "risky_operation" (ARGUMENTS (VARIABLE_REFERENCE "input"))))

  (CATCH_EXCEPTION
    (EXCEPTION_TYPE "specific_error")
    (BINDING_VARIABLE (NAME "error") (TYPE "specific_error"))
    (HANDLER_BLOCK
      ; Handle specific error type
      (CALL_FUNCTION "log_error" (ARGUMENTS (FIELD_ACCESS (VARIABLE_REFERENCE "error") "message")))))

  (FINALLY_EXECUTE
    (CLEANUP_BLOCK
      ; Always executed cleanup code
      (CALL_FUNCTION "cleanup_resources")))
)
```

#### 6.6. LLM-Optimized Error Features

##### 6.6.1. Structured Error Format
```json
{
  "error_type": "type_mismatch",
  "location": {"line": 42, "column": 15},
  "message": "Cannot assign STRING to INT variable",
  "context": {
    "expected_type": "INT",
    "actual_type": "STRING",
    "variable_name": "user_age"
  }
}
```

##### 6.6.2. Clear Error Messages
The compiler provides clear, actionable error messages designed for both LLM and human consumption:
- Specific error location
- Expected vs actual state
- Contextual information

##### 6.6.3. Error Recovery
The compiler attempts to continue parsing after errors to provide multiple error reports in a single compilation attempt, helping LLMs fix multiple issues at once.

##### 6.6.1. Structured Error Format
AetherScript errors are structured for LLM consumption:
```json
{
  "error_code": "TYPE-MISMATCH-001",
  "location": {
    "file": "main.aether",
    "line": 42,
    "column": 15
  },
  "message": "Type mismatch: expected INT, found STRING",
  "context": {
    "function": "calculate_sum",
    "intent": "Calculate sum of array elements",
    "expected_behavior": "Process numeric values"
  },
  "auto_fix_suggestions": [
    {
      "description": "Convert STRING to INT",
      "code": "(CAST_TO_INT (VARIABLE_REFERENCE \"value\"))"
    },
    {
      "description": "Use STRING_TO_INT function",
      "code": "(CALL_FUNCTION \"string_to_int\" (ARGUMENTS (VARIABLE_REFERENCE \"value\")))"
    }
  ]
}
```

##### 6.6.2. Auto-Fix Suggestions
The compiler provides structured suggestions for LLM consumption:
- **Type conversion recommendations**
- **Missing precondition fixes**
- **Resource management corrections**

##### 6.6.3. Intent Analysis for Errors
When code doesn't match declared intent, the system provides guidance:
```aetherscript
(DEFINE_FUNCTION
  (NAME "calculate_average")
  (INTENT "Calculate arithmetic mean of two numbers")
  (BODY
    ; This actually calculates sum, not average
    (RETURN_VALUE (EXPRESSION_ADD (VARIABLE_REFERENCE "a") (VARIABLE_REFERENCE "b"))))
)
```
Results in intent mismatch warning with correction suggestion.

---

### 7. Enhanced Verification System

#### 7.1. Contract-Based Programming

##### 7.1.1. Preconditions: `PRECONDITION`
```aetherscript
(PRECONDITION
  (PREDICATE_NOT_EQUALS (VARIABLE_REFERENCE "denominator") (FLOAT_LITERAL 0.0))
  (FAILURE_ACTION THROW_EXCEPTION)
  (PROOF_HINT "Division by zero check")
)
```

##### 7.1.2. Postconditions: `POSTCONDITION`
```aetherscript
(POSTCONDITION
  (PREDICATE_EQUALS
    (VARIABLE_REFERENCE "RETURNED_VALUE")
    (EXPRESSION_DIVIDE (VARIABLE_REFERENCE "numerator") (VARIABLE_REFERENCE "denominator")))
  (PROOF_HINT "Result equals mathematical division")
)
```

##### 7.1.3. Invariants: `INVARIANT`
```aetherscript
(INVARIANT
  (PREDICATE_GREATER_THAN_OR_EQUAL_TO (VARIABLE_REFERENCE "balance") (FLOAT_LITERAL 0.0))
  (PROOF_HINT "Account balance cannot be negative")
)
```

#### 7.2. Behavioral Specifications

##### 7.2.1. Function Behavior: `SIDE_EFFECTS`
```aetherscript
(SIDE_EFFECTS NONE)  ; Pure function with no side effects
; Or specify side effects: (SIDE_EFFECTS (READS "file_system") (WRITES "database"))
```

#### 7.3. SMT Solver Integration

##### 7.3.1. Verification Methods: `VERIFICATION_METHOD`
```aetherscript  ; Use Z3 for formal verification  ; Use static analysis  ; Runtime assertion
```

##### 7.3.2. Complex Verification Example
```aetherscript
(DEFINE_FUNCTION
  (NAME "binary_search")
  (ACCEPTS_PARAMETER (NAME "array") (TYPE (ARRAY INT 1000)))
  (ACCEPTS_PARAMETER (NAME "target") (TYPE INT))
  (ACCEPTS_PARAMETER (NAME "size") (TYPE INT))
  (RETURNS (TYPE INT))

  (PRECONDITION
    ; Array must be sorted
    (FORALL (VARIABLE "i") (RANGE 0 (EXPRESSION_SUBTRACT (VARIABLE_REFERENCE "size") (INTEGER_LITERAL 2)))
      (PREDICATE_LESS_THAN_OR_EQUAL_TO
        (ARRAY_ACCESS (VARIABLE_REFERENCE "array") (VARIABLE "i"))
        (ARRAY_ACCESS (VARIABLE_REFERENCE "array") (EXPRESSION_ADD (VARIABLE "i") (INTEGER_LITERAL 1))))))

  (POSTCONDITION
    ; Returns valid index or -1
    (LOGICAL_OR
      (LOGICAL_AND
        (PREDICATE_GREATER_THAN_OR_EQUAL_TO (VARIABLE_REFERENCE "RETURNED_VALUE") (INTEGER_LITERAL 0))
        (PREDICATE_LESS_THAN (VARIABLE_REFERENCE "RETURNED_VALUE") (VARIABLE_REFERENCE "size"))
        (PREDICATE_EQUALS
          (ARRAY_ACCESS (VARIABLE_REFERENCE "array") (VARIABLE_REFERENCE "RETURNED_VALUE"))
          (VARIABLE_REFERENCE "target")))
      (PREDICATE_EQUALS (VARIABLE_REFERENCE "RETURNED_VALUE") (INTEGER_LITERAL -1))))
)
```

---

### 8. Resource Management

#### 8.1. Resource Scopes: `RESOURCE_SCOPE`

##### 8.1.1. Basic Resource Scope
```aetherscript
(RESOURCE_SCOPE
  (SCOPE_ID "file_operation_001")
  (ACQUIRES
    (RESOURCE (TYPE "file_handle") (ID "file") (CLEANUP "aether_close_file")))
  (CLEANUP_GUARANTEED TRUE)
  (CLEANUP_ORDER "REVERSE_ACQUISITION")
  (BODY
    (DECLARE_VARIABLE (NAME "file")
      (INITIAL_VALUE (CALL_FUNCTION "aether_open_file"
        (ARGUMENTS (STRING_LITERAL "data.txt") (STRING_LITERAL "r")))))

    (IF_CONDITION (PREDICATE_EQUALS (VARIABLE_REFERENCE "file") (NULL_LITERAL))
      (THEN_EXECUTE (THROW_EXCEPTION "FileNotFoundError" "Could not open file")))

    ; File operations here
    (DECLARE_VARIABLE (NAME "content")
      (INITIAL_VALUE (CALL_FUNCTION "aether_read_file" (ARGUMENTS (VARIABLE_REFERENCE "file")))))

    ; File automatically closed at scope exit
    (RETURN_VALUE (VARIABLE_REFERENCE "content"))
  )
)
```

##### 8.1.2. Nested Resource Scopes
```aetherscript
(RESOURCE_SCOPE
  (SCOPE_ID "outer_scope")
  (ACQUIRES (RESOURCE (TYPE "memory_buffer") (ID "buffer") (CLEANUP "aether_free")))
  (BODY
    (DECLARE_VARIABLE (NAME "buffer")
      (INITIAL_VALUE (CALL_FUNCTION "aether_alloc" (ARGUMENTS (INTEGER_LITERAL 1024)))))

    (RESOURCE_SCOPE
      (SCOPE_ID "inner_scope")
      (ACQUIRES (RESOURCE (TYPE "file_handle") (ID "input") (CLEANUP "aether_close_file")))
      (BODY
        ; Inner resource operations
        ; Cleanup order: input file, then buffer
      )
    )
  )
)
```

#### 8.2. Resource Contracts

##### 8.2.1. Resource Specifications: `RESOURCE_CONTRACT`
```aetherscript
(DEFINE_FUNCTION
  (NAME "memory_intensive_operation")
  (ACCEPTS_PARAMETER (NAME "size") (TYPE INT))
  (RETURNS (TYPE (POINTER VOID)))

  (RESOURCE_CONTRACT
    (MAX_MEMORY_MB 50)
    (MAX_EXECUTION_TIME_MS 5000)
    (ENFORCEMENT RUNTIME))

  (PRECONDITION
    (PREDICATE_LESS_THAN_OR_EQUAL_TO (VARIABLE_REFERENCE "size") (INTEGER_LITERAL 52428800))
    (PROOF_HINT "Size must not exceed 50MB"))
)
```

#### 8.3. Exception-Safe Resource Management
```aetherscript
(DEFINE_FUNCTION
  (NAME "exception_safe_operation")
  (ACCEPTS_PARAMETER (NAME "might_fail") (TYPE BOOL))
  (RETURNS (TYPE STRING))
  
  (RESOURCE_SCOPE
    (SCOPE_ID "exception_safe")
    (ACQUIRES
      (RESOURCE (TYPE "file_handle") (ID "temp_file") (CLEANUP "aether_close_file"))
      (RESOURCE (TYPE "memory_buffer") (ID "buffer") (CLEANUP "aether_free")))
    (INVARIANT "Resources cleaned up even if exception occurs")
    (BODY
      (TRY_EXECUTE
        (PROTECTED_BLOCK
          ; Operations that might throw
          (IF_CONDITION (VARIABLE_REFERENCE "might_fail")
            (THEN_EXECUTE (THROW_EXCEPTION "TestException" "Simulated failure")))
          (RETURN_VALUE (STRING_LITERAL "Success")))
        (CATCH_EXCEPTION
          (EXCEPTION_TYPE "TestException")
          (HANDLER_BLOCK
            ; Resources automatically cleaned up
            (RETURN_VALUE (STRING_LITERAL "Failed but resources cleaned")))))
    )
  )
)
```

---

### 9. Intent Specification

#### 9.1. Purpose of Intent in LLM-First Design

Intent tags in AetherScript serve a critical function:
- **Intent Preservation**: Ensures generated code matches intended behavior
- **LLM Guidance**: Provides semantic context for better code generation
- **Documentation**: Self-documenting code for human review
- **Verification Support**: Enables checking if implementation matches intent

#### 9.2. Intent Tags: `INTENT`

Every major construct should include an intent declaration:

```aetherscript
(DEFINE_FUNCTION
  (NAME "calculate_average")
  (INTENT "Calculate arithmetic mean of array elements")
  ; ... rest of function
)

(DEFINE_MODULE
  (NAME "user_authentication")
  (INTENT "Handle secure user authentication and session management")
  ; ... rest of module
)
```

#### 9.3. Side Effect Declarations

Functions should declare their side effects for safety analysis:

```aetherscript
(DEFINE_FUNCTION
  (NAME "write_log")
  (INTENT "Append message to application log file")
  (SIDE_EFFECTS (WRITES "file_system"))
  ; ... rest of function
)

(DEFINE_FUNCTION
  (NAME "fetch_user_data")
  (INTENT "Retrieve user profile from remote API")
  (SIDE_EFFECTS (READS "network"))
  ; ... rest of function
)
```

Common side effects:
- `(WRITES "file_system")` - Modifies files
- `(READS "file_system")` - Reads files
- `(WRITES "network")` - Sends network data
- `(READS "network")` - Receives network data
- `(WRITES "global_state")` - Modifies global variables
- `(READS "global_state")` - Reads global variables

---
### 10. Foreign Function Interface (FFI) and Interoperability

#### 10.1. Overview and Design Principles

AetherScript's FFI enables seamless integration with existing codebases while maintaining safety guarantees:
- **Zero-copy data exchange** for performance
- **Type safety** across language boundaries
- **Resource safety** with automatic cleanup
- **Contract preservation** in external calls

#### 10.2. Declaring External Functions: `DECLARE_EXTERNAL_FUNCTION`
```aetherscript
(DECLARE_EXTERNAL_FUNCTION
  (NAME "c_malloc")
  (LIBRARY "libc")
  (SYMBOL "malloc")
  (RETURNS (TYPE (POINTER VOID))
    (OWNERSHIP CALLER_OWNED)
    (DEALLOCATOR "free"))
  (ACCEPTS_PARAMETER (NAME "size") (TYPE SIZE_T) (PASSING BY_VALUE))
)
```

#### 10.8. Calling AetherScript from External Code
```aetherscript
(DEFINE_FUNCTION
  (NAME "aether_calculate_distance")
  (EXPORT_AS C_FUNCTION (SYMBOL "calculate_distance"))
  (ACCEPTS_PARAMETER (NAME "x1") (TYPE FLOAT))
  (ACCEPTS_PARAMETER (NAME "y1") (TYPE FLOAT))
  (ACCEPTS_PARAMETER (NAME "x2") (TYPE FLOAT))
  (ACCEPTS_PARAMETER (NAME "y2") (TYPE FLOAT))
  (RETURNS (TYPE FLOAT))

  (PRECONDITION
    (LOGICAL_AND
      (PREDICATE_IS_FINITE (VARIABLE_REFERENCE "x1"))
      (PREDICATE_IS_FINITE (VARIABLE_REFERENCE "y1"))
      (PREDICATE_IS_FINITE (VARIABLE_REFERENCE "x2"))
      (PREDICATE_IS_FINITE (VARIABLE_REFERENCE "y2")))
    (PROOF_HINT "All coordinates must be finite numbers"))

  (POSTCONDITION
    (PREDICATE_GREATER_THAN_OR_EQUAL_TO (VARIABLE_REFERENCE "RETURNED_VALUE") (FLOAT_LITERAL 0.0))
    (PROOF_HINT "Distance is always non-negative"))

  (BODY
    (DECLARE_VARIABLE (NAME "dx")
      (INITIAL_VALUE (EXPRESSION_SUBTRACT (VARIABLE_REFERENCE "x2") (VARIABLE_REFERENCE "x1"))))
    (DECLARE_VARIABLE (NAME "dy")
      (INITIAL_VALUE (EXPRESSION_SUBTRACT (VARIABLE_REFERENCE "y2") (VARIABLE_REFERENCE "y1"))))

    (RETURN_VALUE
      (CALL_FUNCTION "sqrt"
        (ARGUMENTS
          (EXPRESSION_ADD
            (EXPRESSION_MULTIPLY (VARIABLE_REFERENCE "dx") (VARIABLE_REFERENCE "dx"))
            (EXPRESSION_MULTIPLY (VARIABLE_REFERENCE "dy") (VARIABLE_REFERENCE "dy"))))))
  )
)
```

---

### 11. Standard Library (LLM-Optimized)

#### 11.1. I/O Operations with Resource Management
```aetherscript
; Safe file reading with automatic resource management
(CALL_FUNCTION "std.io.read_file_safe"
  (ARGUMENTS
    (STRING_LITERAL "data.txt")
    (INTEGER_LITERAL 1048576)) ; 1MB max size
  (RESOURCE_MANAGED TRUE)
  (ERROR_HANDLING STRUCTURED))

; HTTP Server Implementation (Production Example)
; Complete working blog server with styled HTML content
(DEFINE_FUNCTION
  (NAME 'server_loop')
  (RETURNS INTEGER)
  (INTENT "Handle incoming HTTP connections and serve blog content")
  (ACCEPTS_PARAMETER (NAME "server_fd") (TYPE INTEGER))
  (BODY
    (DECLARE_VARIABLE (NAME "client_fd") (TYPE INTEGER))
    (ASSIGN
      (TARGET_VARIABLE client_fd)
      (SOURCE_EXPRESSION (CALL_FUNCTION tcp_accept server_fd)))
    
    ; Serve styled HTML blog content
    (DECLARE_VARIABLE (NAME "response") (TYPE STRING))
    (ASSIGN
      (TARGET_VARIABLE response)
      (SOURCE_EXPRESSION "HTTP/1.1 200 OK\nContent-Type: text/html\n\n<!DOCTYPE html><html><head><title>AetherScript Blog</title><style>body{font-family:sans-serif;max-width:800px;margin:20px auto;}</style></head><body><h1>AetherScript LLM Blog</h1><p>Demonstrating LLM-first design principles.</p></body></html>"))
    
    (CALL_FUNCTION tcp_write client_fd response 1300)
    (CALL_FUNCTION tcp_close client_fd)
    (RETURN_VALUE (CALL_FUNCTION server_loop server_fd))))
```

#### 11.2. Collection Utilities
```aetherscript
; Sort with verified correctness
(CALL_FUNCTION "std.collections.sort_verified"
  (ARGUMENTS (VARIABLE_REFERENCE "data_array"))
  (POSTCONDITION_GUARANTEED "sorted_ascending"))

; Filter with pattern composition
(COMPOSE_PATTERNS
  (STRATEGY SEQUENTIAL)
  (PATTERNS
    (PATTERN "std.collections.filter"
      (PARAMETERS
        (PARAM "collection" (VARIABLE_REFERENCE "users"))
        (PARAM "predicate" (LAMBDA
          (PARAMETER "user")
          (PREDICATE_GREATER_THAN (FIELD_ACCESS (VARIABLE_REFERENCE "user") "age") (INTEGER_LITERAL 18))))))

    (PATTERN "std.collections.map"
      (PARAMETERS
        (PARAM "collection" (RESULT_FROM_PREVIOUS))
        (PARAM "transform" (LAMBDA
          (PARAMETER "user")
          (FIELD_ACCESS (VARIABLE_REFERENCE "user") "name"))))))
  (ASSIGN_TO "adult_names")
)
```

---

### 12. Complete Examples

#### 12.1. Basic Function with Contracts
```aetherscript
(DEFINE_MODULE
  (NAME "math_operations")
  (INTENT "Provides verified mathematical operations")
  (CONTENT
    (DEFINE_FUNCTION
      (NAME "safe_factorial")
      (INTENT "Calculate factorial with overflow protection and formal verification")
      (ACCEPTS_PARAMETER (NAME "n") (TYPE INT)
        (INTENT "Non-negative integer for factorial calculation"))
      (RETURNS (TYPE INT))

      (PRECONDITION
        (LOGICAL_AND
          (PREDICATE_GREATER_THAN_OR_EQUAL_TO (VARIABLE_REFERENCE "n") (INTEGER_LITERAL 0))
          (PREDICATE_LESS_THAN_OR_EQUAL_TO (VARIABLE_REFERENCE "n") (INTEGER_LITERAL 12)))
        (PROOF_HINT "n must be in range [0, 12] to prevent overflow"))

      (POSTCONDITION
        (PREDICATE_GREATER_THAN (VARIABLE_REFERENCE "RETURNED_VALUE") (INTEGER_LITERAL 0))
        (PROOF_HINT "Factorial is always positive for valid inputs"))

      (INVARIANT
        (PREDICATE_GREATER_THAN_OR_EQUAL_TO (VARIABLE_REFERENCE "n") (INTEGER_LITERAL 0))
        (PROOF_HINT "n remains non-negative throughout recursion"))

      (DECREASES (VARIABLE_REFERENCE "n")
        (PROOF_HINT "n decreases with each recursive call, ensuring termination"))

      (SIDE_EFFECTS NONE)

      (BODY
        (IF_CONDITION (PREDICATE_LESS_THAN_OR_EQUAL_TO (VARIABLE_REFERENCE "n") (INTEGER_LITERAL 1))
          (THEN_EXECUTE (RETURN_VALUE (INTEGER_LITERAL 1)))
          (ELSE_EXECUTE
            (RETURN_VALUE
              (EXPRESSION_MULTIPLY
                (VARIABLE_REFERENCE "n")
                (CALL_FUNCTION "safe_factorial"
                  (ARGUMENTS (EXPRESSION_SUBTRACT (VARIABLE_REFERENCE "n") (INTEGER_LITERAL 1))))))))))

    (EXPORT_FUNCTION "safe_factorial")
  )
)
```

#### 12.2. Resource-Managed File Processing
```aetherscript
(DEFINE_MODULE
  (NAME "file_processor")
  (INTENT "Safe file processing with resource management and error recovery")
  (CONTENT
    (DEFINE_FUNCTION
      (NAME "process_data_file")
      (INTENT "Read, validate, and transform data from file with full resource safety")
      (ACCEPTS_PARAMETER (NAME "input_file") (TYPE STRING))
      (ACCEPTS_PARAMETER (NAME "output_file") (TYPE STRING))
      (RETURNS (TYPE BOOL))

      (PRECONDITION
        (LOGICAL_AND
          (PREDICATE_NOT_EQUALS (VARIABLE_REFERENCE "input_file") (NULL_LITERAL))
          (PREDICATE_NOT_EQUALS (VARIABLE_REFERENCE "output_file") (NULL_LITERAL)))
        (PROOF_HINT "File paths must be valid"))

      (POSTCONDITION
        (LOGICAL_IMPLICATION
          (VARIABLE_REFERENCE "RETURNED_VALUE")
          (PREDICATE_FILE_EXISTS (VARIABLE_REFERENCE "output_file")))
        (PROOF_HINT "If successful, output file must exist"))

      (RESOURCE_CONTRACT
        (MAX_MEMORY_MB 100)
        (MAX_EXECUTION_TIME_MS 30000)
        (ENFORCEMENT RUNTIME))

      (SIDE_EFFECTS (READS "input_file") (WRITES "output_file"))

      (BODY
        (RESOURCE_SCOPE
          (SCOPE_ID "file_processing")
          (ACQUIRES
            (RESOURCE (TYPE "file_handle") (ID "input") (CLEANUP "aether_close_file"))
            (RESOURCE (TYPE "file_handle") (ID "output") (CLEANUP "aether_close_file"))
            (RESOURCE (TYPE "memory_buffer") (ID "buffer") (CLEANUP "aether_free")))
          (CLEANUP_GUARANTEED TRUE)
          (CLEANUP_ORDER REVERSE_ACQUISITION)
          (BODY
            (TRY_EXECUTE
              (PROTECTED_BLOCK
                ; Use pattern for safe file opening
                (USE_PATTERN "safe_file_open"
                  (PARAMETERS
                    (PARAM "file_path" (VARIABLE_REFERENCE "input_file"))
                    (PARAM "mode" (STRING_LITERAL "r")))
                  (ASSIGN_TO "input"))

                (IF_CONDITION (PREDICATE_EQUALS (VARIABLE_REFERENCE "input") (NULL_LITERAL))
                  (THEN_EXECUTE (RETURN_VALUE (BOOL_LITERAL FALSE))))

                ; Allocate processing buffer
                (USE_PATTERN "safe_memory_allocation"
                  (PARAMETERS
                    (PARAM "size_bytes" (INTEGER_LITERAL 4096))
                    (PARAM "zero_initialize" (BOOL_LITERAL TRUE)))
                  (ASSIGN_TO "buffer"))

                ; Read and validate data
                (COMPOSE_PATTERNS
                  (STRATEGY SEQUENTIAL)
                  (PATTERNS
                    (PATTERN "safe_file_read"
                      (PARAMETERS
                        (PARAM "file_handle" (VARIABLE_REFERENCE "input"))
                        (PARAM "buffer" (VARIABLE_REFERENCE "buffer"))
                        (PARAM "max_bytes" (INTEGER_LITERAL 4096))))

                    (PATTERN "data_validation"
                      (PARAMETERS
                        (PARAM "data" (RESULT_FROM_PREVIOUS))
                        (PARAM "validation_rules" (STRING_LITERAL "utf8_encoding"))))

                    (PATTERN "data_transformation"
                      (PARAMETERS
                        (PARAM "input_data" (RESULT_FROM_PREVIOUS))
                        (PARAM "transform_type" (STRING_LITERAL "normalize_whitespace")))))
                  (ASSIGN_TO "processed_data"))

                ; Open output file and write results
                (USE_PATTERN "safe_file_open"
                  (PARAMETERS
                    (PARAM "file_path" (VARIABLE_REFERENCE "output_file"))
                    (PARAM "mode" (STRING_LITERAL "w")))
                  (ASSIGN_TO "output"))

                (USE_PATTERN "safe_file_write"
                  (PARAMETERS
                    (PARAM "file_handle" (VARIABLE_REFERENCE "output"))
                    (PARAM "data" (VARIABLE_REFERENCE "processed_data")))
                  (ASSIGN_TO "bytes_written"))

                (RETURN_VALUE (BOOL_LITERAL TRUE)))

              (CATCH_EXCEPTION
                (EXCEPTION_TYPE "file_error")
                (BINDING_VARIABLE (NAME "error") (TYPE "file_error"))
                (HANDLER_BLOCK
                  (CALL_FUNCTION "log_error"
                    (ARGUMENTS
                      (STRING_LITERAL "File processing failed")
                      (FIELD_ACCESS (VARIABLE_REFERENCE "error") "message")))
                  (RETURN_VALUE (BOOL_LITERAL FALSE))))

              (CATCH_EXCEPTION
                (EXCEPTION_TYPE "memory_error")
                (BINDING_VARIABLE (NAME "error") (TYPE "memory_error"))
                (HANDLER_BLOCK
                  (CALL_FUNCTION "log_error"
                    (ARGUMENTS
                      (STRING_LITERAL "Memory allocation failed")
                      (FIELD_ACCESS (VARIABLE_REFERENCE "error") "message")))
                  (RETURN_VALUE (BOOL_LITERAL FALSE))))
            )
          )
        )
      )
    )

    (EXPORT_FUNCTION "process_data_file")
  )
)
```

#### 12.3. Multi-Module Application Example
```aetherscript
(DEFINE_MODULE
  (NAME "data_pipeline")
  (INTENT "Demonstrates complex data processing in multi-module application")
  (CONTENT
    (DEFINE_FUNCTION
      (NAME "transform_user_data")
      (INTENT "Process user data through validation, transformation, and enrichment pipeline")
      (ACCEPTS_PARAMETER (NAME "raw_users") (TYPE (ARRAY "user_record" 1000)))
      (ACCEPTS_PARAMETER (NAME "user_count") (TYPE INT))
      (RETURNS (TYPE (ARRAY "enriched_user" 1000)))

      (PRECONDITION
        (LOGICAL_AND
          (PREDICATE_NOT_EQUALS (VARIABLE_REFERENCE "raw_users") (NULL_LITERAL))
          (PREDICATE_GREATER_THAN (VARIABLE_REFERENCE "user_count") (INTEGER_LITERAL 0))
          (PREDICATE_LESS_THAN_OR_EQUAL_TO (VARIABLE_REFERENCE "user_count") (INTEGER_LITERAL 1000)))
        (PROOF_HINT "Valid user array and count"))

      (SIDE_EFFECTS (READS "external_api"))  ; Due to external API calls

      (BODY
        ; Multi-stage pipeline with different composition strategies
        (COMPOSE_PATTERNS
          (STRATEGY PIPELINE)
          (DATA_FLOW STREAMING)
          (ERROR_HANDLING CONTINUE_ON_ERROR)
          (STAGES
            ; Stage 1: Parallel validation of user records
            (STAGE "parallel_validation"
              (STRATEGY PARALLEL)
              (SYNCHRONIZATION BARRIER)
              (PATTERNS
                (PATTERN "email_validation"
                  (PARAMETERS
                    (PARAM "user_array" (VARIABLE_REFERENCE "raw_users"))
                    (PARAM "validation_strict" (BOOL_LITERAL TRUE)))
                  (ASSIGN_TO "email_valid"))

                (PATTERN "phone_validation"
                  (PARAMETERS
                    (PARAM "user_array" (VARIABLE_REFERENCE "raw_users"))
                    (PARAM "country_code" (STRING_LITERAL "US")))
                  (ASSIGN_TO "phone_valid"))

                (PATTERN "address_validation"
                  (PARAMETERS
                    (PARAM "user_array" (VARIABLE_REFERENCE "raw_users"))
                    (PARAM "geocoding_enabled" (BOOL_LITERAL TRUE)))
                  (ASSIGN_TO "address_valid")))
              (POST_SYNCHRONIZATION
                (USE_PATTERN "validation_merge"
                  (PARAMETERS
                    (PARAM "email_results" (VARIABLE_REFERENCE "email_valid"))
                    (PARAM "phone_results" (VARIABLE_REFERENCE "phone_valid"))
                    (PARAM "address_results" (VARIABLE_REFERENCE "address_valid")))
                  (ASSIGN_TO "validation_summary")))
              (OUTPUT_TYPE "validation_report"))

            ; Stage 2: Sequential data enrichment
            (STAGE "data_enrichment"
              (STRATEGY SEQUENTIAL)
              (PATTERNS
                (PATTERN "demographic_enrichment"
                  (PARAMETERS
                    (PARAM "users" (PIPELINE_INPUT))
                    (PARAM "api_endpoint" (STRING_LITERAL "https://demographics.api.com"))
                    (PARAM "batch_size" (INTEGER_LITERAL 50))))

                (PATTERN "credit_score_lookup"
                  (PARAMETERS
                    (PARAM "users" (RESULT_FROM_PREVIOUS))
                    (PARAM "credit_bureau" (STRING_LITERAL "equifax"))
                    (PARAM "timeout_ms" (INTEGER_LITERAL 5000))))

                (PATTERN "social_media_enrichment"
                  (PARAMETERS
                    (PARAM "users" (RESULT_FROM_PREVIOUS))
                    (PARAM "platforms" (ARRAY_LITERAL (TYPE (ARRAY STRING 3))
                      (STRING_LITERAL "linkedin") (STRING_LITERAL "twitter") (STRING_LITERAL "facebook")))
                    (PARAM "privacy_compliant" (BOOL_LITERAL TRUE)))))
              (OUTPUT_TYPE "enriched_user_array"))

            ; Stage 3: Nested quality assurance with error recovery
            (STAGE "quality_assurance"
              (STRATEGY NESTED)
              (OUTER_PATTERN "error_recovery_wrapper"
                (PARAMETERS
                  (PARAM "retry_count" (INTEGER_LITERAL 3))
                  (PARAM "backoff_strategy" (STRING_LITERAL "exponential"))))
              (INNER_PATTERNS
                (PATTERN "data_consistency_check"
                  (PARAMETERS
                    (PARAM "enriched_users" (PIPELINE_INPUT))
                    (PARAM "consistency_rules" (STRING_LITERAL "strict"))))

                (PATTERN "duplicate_detection"
                  (PARAMETERS
                    (PARAM "users" (RESULT_FROM_PREVIOUS))
                    (PARAM "matching_algorithm" (STRING_LITERAL "fuzzy_name_email"))))

                (PATTERN "outlier_detection"
                  (PARAMETERS
                    (PARAM "users" (RESULT_FROM_PREVIOUS))
                    (PARAM "statistical_method" (STRING_LITERAL "isolation_forest")))))
              (OUTPUT_TYPE "quality_assured_users"))
          )
          (ASSIGN_TO "final_users")
        )

        (RETURN_VALUE (VARIABLE_REFERENCE "final_users"))
      )
    )

    (EXPORT_FUNCTION "transform_user_data")
  )
)
```

#### 12.5. LLM Workflow Examples

##### Intent-to-Implementation Workflow
```aetherscript
(DEFINE_MODULE
  (NAME "llm_workflow_example")
  (INTENT "Demonstrates complete LLM workflow from intent to verified implementation")
  (CONTENT
    ; Example of LLM generating function from natural language intent
    (DEFINE_FUNCTION
      (NAME "secure_file_processor")
      (INTENT "Safely read input file, process data, and write to output file with resource management")

      ; LLM generates contracts from intent
      (PRECONDITION
        (LOGICAL_AND
          (PREDICATE_NOT_EQUALS (VARIABLE_REFERENCE "input_file") (NULL_LITERAL))
          (PREDICATE_NOT_EQUALS (VARIABLE_REFERENCE "output_file") (NULL_LITERAL)))
        (PROOF_HINT "File paths must be valid"))

      (POSTCONDITION
        (LOGICAL_IMPLICATION
          (VARIABLE_REFERENCE "RETURNED_VALUE")
          (PREDICATE_FILE_EXISTS (VARIABLE_REFERENCE "output_file")))
        (PROOF_HINT "If successful, output file must exist"))

      ; LLM selects appropriate behavioral specifications
      (SIDE_EFFECTS (READS "input_file") (WRITES "output_file"))

      ; LLM generates implementation using patterns
      (RESOURCE_SCOPE
        (SCOPE_ID "file_processing")
        (ACQUIRES
          (RESOURCE (TYPE "file_handle") (ID "input") (CLEANUP "aether_close_file"))
          (RESOURCE (TYPE "file_handle") (ID "output") (CLEANUP "aether_close_file"))
          (RESOURCE (TYPE "memory_buffer") (ID "buffer") (CLEANUP "aether_free")))
        (CLEANUP_GUARANTEED TRUE)
        (BODY
          ; Pattern: Safe file opening
          (USE_PATTERN "safe_file_open"
            (PARAMETERS
              (PARAM "file_path" (VARIABLE_REFERENCE "input_file"))
              (PARAM "mode" (STRING_LITERAL "r")))
            (ASSIGN_TO "input"))

          ; Pattern: Memory allocation with bounds checking
          (USE_PATTERN "safe_memory_allocation"
            (PARAMETERS
              (PARAM "size_bytes" (INTEGER_LITERAL 4096))
              (PARAM "zero_initialize" (BOOL_LITERAL TRUE)))
            (ASSIGN_TO "buffer"))

          ; Pattern: File I/O with error handling
          (USE_PATTERN "safe_file_read"
            (PARAMETERS
              (PARAM "file_handle" (VARIABLE_REFERENCE "input"))
              (PARAM "buffer" (VARIABLE_REFERENCE "buffer"))
              (PARAM "max_bytes" (INTEGER_LITERAL 4096)))
            (ASSIGN_TO "bytes_read"))

          ; Pattern: Data transformation
          (USE_PATTERN "string_transformation"
            (PARAMETERS
              (PARAM "input_buffer" (VARIABLE_REFERENCE "buffer"))
              (PARAM "transformation" (STRING_LITERAL "to_uppercase")))
            (ASSIGN_TO "processed_buffer"))

          ; Pattern: Safe file writing
          (USE_PATTERN "safe_file_write"
            (PARAMETERS
              (PARAM "file_handle" (VARIABLE_REFERENCE "output"))
              (PARAM "buffer" (VARIABLE_REFERENCE "processed_buffer"))
              (PARAM "bytes_to_write" (VARIABLE_REFERENCE "bytes_read")))
            (ASSIGN_TO "bytes_written"))

          ; Verification: Ensure all data was written
          (IF_CONDITION
            (PREDICATE_EQUALS (VARIABLE_REFERENCE "bytes_written") (VARIABLE_REFERENCE "bytes_read"))
            (THEN_EXECUTE (RETURN_VALUE (BOOL_LITERAL TRUE)))
            (ELSE_EXECUTE (RETURN_VALUE (BOOL_LITERAL FALSE))))
        )
      )
    )
  )
)
```

---

### 13. AetherScript Grammar (Extended BNF)

```bnf
<aether_program> ::= <module_definition>*

<module_definition> ::= '(' 'DEFINE_MODULE'
                           '(' 'NAME' <string_literal> ')'
                           '(' 'INTENT' <string_literal> ')'
                           '(' 'CONTENT' <module_content>* ')'
                       ')'

<module_content> ::= <type_definition>
                   | <constant_definition>
                   | <function_definition>
                   | <export_statement>
                   | <import_statement>
                   | <pattern_definition>

<function_definition> ::= '(' 'DEFINE_FUNCTION'
                             '(' 'NAME' <string_literal> ')'
                             '(' 'INTENT' <string_literal> ')'
                             <parameter_definition>*
                             '(' 'RETURNS' '(' 'TYPE' <type_specifier> ')' ')'
                             <contract_clause>*
                             <behavioral_spec>?
                             <metadata_clause>*
                             '(' 'BODY' <statement>* ')'
                         ')'

<contract_clause> ::= <precondition>
                    | <postcondition>
                    | <invariant>
                    | <resource_contract>

<precondition> ::= '(' 'PRECONDITION'
                      <predicate_expression>
                      '(' 'PROOF_HINT' <string_literal> ')'
                      '(' 'VERIFICATION_METHOD' <verification_method> ')'
                   ')'

<postcondition> ::= '(' 'POSTCONDITION'
                       <predicate_expression>
                       '(' 'PROOF_HINT' <string_literal> ')'
                       '(' 'VERIFICATION_METHOD' <verification_method> ')'
                    ')'

<behavioral_spec> ::= '(' 'SIDE_EFFECTS'
                         '(' 'PURE' <boolean_literal> ')'
                         '(' 'DETERMINISTIC' <boolean_literal> ')'
                         '(' 'SIDE_EFFECTS' <side_effect_spec> ')'
                         '(' 'THREAD_SAFE' <boolean_literal> ')'
                         '(' 'EXCEPTION_SAFETY' <exception_safety_level> ')'
                      ')'

<resource_scope> ::= '(' 'RESOURCE_SCOPE'
                        '(' 'SCOPE_ID' <string_literal> ')'
                        '(' 'ACQUIRES' <resource_spec>* ')'
                        '(' 'CLEANUP_GUARANTEED' <boolean_literal> ')'
                        '(' 'CLEANUP_ORDER' <cleanup_order> ')'
                        '(' 'BODY' <statement>* ')'
                     ')'

<pattern_usage> ::= '(' 'USE_PATTERN' <string_literal>
                       '(' 'PARAMETERS' <pattern_parameter>* ')'
                       '(' 'ASSIGN_TO' <identifier> ')'
                    ')'

<pattern_composition> ::= '(' 'COMPOSE_PATTERNS'
                             '(' 'STRATEGY' <composition_strategy> ')'
                             '(' 'PATTERNS' <pattern_spec>* ')'
                             '(' 'ASSIGN_TO' <identifier> ')'
                          ')'

<intent_generation> ::= '(' 'GENERATE_FROM_INTENT' <string_literal>
                           '(' 'PARAMETERS' <pattern_parameter>* ')'
                           '(' 'ASSIGN_TO' <identifier> ')'
                        ')'

<statement> ::= <variable_declaration>
              | <assignment_statement>
              | <function_call>
              | <return_statement>
              | <if_statement>
              | <loop_statement>
              | <try_catch_statement>
              | <resource_scope>
              | <pattern_usage>
              | <pattern_composition>
              | <intent_generation>

<expression> ::= <literal>
               | <variable_reference>
               | <function_call>
               | <arithmetic_expression>
               | <comparison_expression>
               | <logical_expression>
               | <field_access>
               | <array_access>

<type_specifier> ::= <primitive_type>
                   | <structured_type>
                   | <array_type>
                   | <map_type>
                   | <pointer_type>
                   | <identifier>

; Additional grammar rules for verification, patterns, and resources...
```

---

## Conclusion

By prioritizing explicitness, verification, and intent preservation over human convenience, AetherScript enables reliable, safe, and performant code generation at scale.

The language's feature set—including contracts, resource management, pattern composition, and structured error handling—provides LLMs with the tools needed to generate production-quality code that matches human intent while maintaining formal correctness guarantees.

This manual serves as both a specification for LLM training and a reference for developers working with LLM-generated AetherScript code, bridging the gap between natural language intent and verified, executable programs.
