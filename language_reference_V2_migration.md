# AetherScript V2 Syntax Migration Guide

## Overview

This document demonstrates the migration from AetherScript's S-expression syntax (V1) to the new structured Swift-like syntax (V2). The new syntax is designed to be more human-readable while maintaining the explicitness and unambiguity that makes it ideal for LLM code generation.

### Key Principles of V2 Syntax

1. **ALL binary operations require braces**: `{x + y}`
2. **ALL comparisons require braces**: `{x > 0}`
3. **ALL control flow uses `when` blocks** (no if/else)
4. **ALL statements end with semicolons**
5. **ALL multi-argument functions use labeled parameters**
6. **NO special cases** - complex rules apply everywhere

---

## Table of Contents

1. [Module Definition](#module-definition)
2. [Function Definition](#function-definition)
3. [Variables and Constants](#variables-and-constants)
4. [Types](#types)
5. [Expressions](#expressions)
6. [Control Flow](#control-flow)
7. [Pattern Matching](#pattern-matching)
8. [Error Handling](#error-handling)
9. [FFI (Foreign Function Interface)](#ffi-foreign-function-interface)
10. [Resource Management](#resource-management)

---

## Module Definition

### Old Syntax (S-expressions)
```aether
(DEFINE_MODULE
  (NAME 'my_module')
  (INTENT "Description of module purpose")
  (CONTENT
    ; Module contents go here
  )
)
```

### New Syntax (V2)
```aether
module MyModule {
    /// Description of module purpose

    // Module contents go here
}
```

### Import Statements

**Old:**
```aether
(IMPORT_MODULE "std.io")
(IMPORT_MODULE "my_other_module")
```

**New:**
```aether
import std.io;
import my_other_module;
```

---

## Function Definition

### Basic Function

**Old Syntax:**
```aether
(DEFINE_FUNCTION
  (NAME 'add_numbers')
  (ACCEPTS_PARAMETER (NAME 'a') (TYPE INTEGER))
  (ACCEPTS_PARAMETER (NAME 'b') (TYPE INTEGER))
  (RETURNS INTEGER)
  (INTENT "Adds two integers together")
  (BODY
    (RETURN_VALUE (EXPRESSION_ADD (VARIABLE_REFERENCE 'a') (VARIABLE_REFERENCE 'b')))
  )
)
```

**New Syntax:**
```aether
/// Adds two integers together
func addNumbers(a: Int, b: Int) -> Int {
    return {a + b};
}
```

### Function with Contracts

**Old Syntax:**
```aether
(DEFINE_FUNCTION
  (NAME 'safe_divide')
  (ACCEPTS_PARAMETER (NAME 'a') (TYPE INTEGER))
  (ACCEPTS_PARAMETER (NAME 'b') (TYPE INTEGER))
  (RETURNS INTEGER)
  (PRECONDITION (PREDICATE_NOT_EQUALS 'b' 0) ASSERT_FAIL "Division by zero")
  (BODY
    (RETURN_VALUE (EXPRESSION_DIVIDE (VARIABLE_REFERENCE 'a') (VARIABLE_REFERENCE 'b')))
  )
)
```

**New Syntax:**
```aether
@requires({b != 0}, "Division by zero")
@ensures({result != 0})
func safeDivide(a: Int, b: Int) -> Int {
    return {a / b};
}
```

### Complex Function with Full Contracts

**Old Syntax:**
```aether
(DEFINE_FUNCTION
  (NAME 'factorial')
  (ACCEPTS_PARAMETER (NAME 'n') (TYPE INTEGER))
  (RETURNS INTEGER)
  (PRECONDITION (PREDICATE_GREATER_EQUAL 'n' 0) ASSERT_FAIL "n must be non-negative")
  (POSTCONDITION (PREDICATE_GREATER 'result' 0) ASSERT_FAIL "Result must be positive")
  (BODY
    (IF_STATEMENT
      (CONDITION (EXPRESSION_LESS_EQUAL (VARIABLE_REFERENCE 'n') 1))
      (THEN_EXECUTE (RETURN_VALUE 1))
      (ELSE_EXECUTE
        (RETURN_VALUE
          (EXPRESSION_MULTIPLY
            (VARIABLE_REFERENCE 'n')
            (CALL_FUNCTION 'factorial'
              (ARGUMENTS (EXPRESSION_SUBTRACT (VARIABLE_REFERENCE 'n') 1))
            )
          )
        )
      )
    )
  )
)
```

**New Syntax:**
```aether
@requires({n >= 0}, "n must be non-negative")
@ensures({result > 0}, "Result must be positive")
func factorial(n: Int) -> Int {
    when {
        case ({n <= 1}): return 1;
        else: return {n * factorial({n - 1})};
    }
}
```

---

## Variables and Constants

### Variable Declaration

**Old Syntax:**
```aether
(DECLARE_VARIABLE
  (NAME 'my_var')
  (TYPE INTEGER)
  (INITIAL_VALUE 42)
)
```

**New Syntax:**
```aether
let myVar: Int = 42;
```

### Mutable Variable

**Old Syntax:**
```aether
(DECLARE_VARIABLE
  (NAME 'counter')
  (TYPE INTEGER)
  (INITIAL_VALUE 0)
)
(ASSIGN_VARIABLE 'counter' 10)
```

**New Syntax:**
```aether
let mut counter: Int = 0;
counter = 10;
```

### Constant Declaration

**Old Syntax:**
```aether
(DECLARE_CONSTANT
  (NAME 'PI')
  (TYPE FLOAT)
  (VALUE 3.14159)
  (INTENT "Mathematical constant pi")
)
```

**New Syntax:**
```aether
/// Mathematical constant pi
const PI: Float = 3.14159;
```

---

## Types

### Primitive Types

**Old Syntax:**
```aether
(TYPE INTEGER)
(TYPE FLOAT)
(TYPE STRING)
(TYPE BOOLEAN)
(TYPE VOID)
```

**New Syntax:**
```aether
Int
Float
String
Bool
Void
```

### Array Types

**Old Syntax:**
```aether
(TYPE (ARRAY INTEGER))           ; Dynamic array
(TYPE (ARRAY STRING 10))         ; Fixed-size array
```

**New Syntax:**
```aether
Array<Int>                       // Dynamic array
Array<String, 10>                // Fixed-size array
```

### Map Types

**Old Syntax:**
```aether
(TYPE (MAP STRING INTEGER))      ; Map from strings to integers
```

**New Syntax:**
```aether
Map<String, Int>                 // Map from strings to integers
```

### Pointer Types

**Old Syntax:**
```aether
(TYPE (POINTER INTEGER))         ; Raw pointer to integer
(TYPE (POINTER_MUT INTEGER))     ; Mutable raw pointer to integer
```

**New Syntax:**
```aether
Pointer<Int>                     // Raw pointer to integer
MutPointer<Int>                  // Mutable raw pointer to integer
```

### Structured Types (Structs)

**Old Syntax:**
```aether
(DEFINE_STRUCTURED_TYPE
  (NAME 'Point')
  (FIELD (NAME 'x') (TYPE FLOAT))
  (FIELD (NAME 'y') (TYPE FLOAT))
)
```

**New Syntax:**
```aether
struct Point {
    x: Float;
    y: Float;
}
```

### Enumeration Types

**Old Syntax:**
```aether
(DEFINE_ENUMERATION_TYPE
  (NAME Result)
  (INTENT "Represents success or failure")
  (VARIANTS
    (VARIANT Ok (HOLDS INTEGER))
    (VARIANT Error (HOLDS STRING))
  )
)
```

**New Syntax:**
```aether
/// Represents success or failure
enum Result {
    case Ok(Int);
    case Error(String);
}
```

---

## Expressions

### Arithmetic Operations

**Old Syntax:**
```aether
(EXPRESSION_ADD 1 2)
(EXPRESSION_SUBTRACT 5 3)
(EXPRESSION_MULTIPLY 4 6)
(EXPRESSION_DIVIDE 8 2)
(EXPRESSION_MODULO 10 3)
```

**New Syntax:**
```aether
{1 + 2};
{5 - 3};
{4 * 6};
{8 / 2};
{10 % 3};
```

### Complex Expressions

**Old Syntax:**
```aether
(EXPRESSION_ADD
  (EXPRESSION_MULTIPLY (VARIABLE_REFERENCE 'a') (VARIABLE_REFERENCE 'b'))
  (EXPRESSION_DIVIDE (VARIABLE_REFERENCE 'c') 2)
)
```

**New Syntax:**
```aether
{{a * b} + {c / 2}};
```

### Comparison Operations

**Old Syntax:**
```aether
(EXPRESSION_EQUALS 5 5)
(EXPRESSION_NOT_EQUALS 3 4)
(EXPRESSION_LESS 2 7)
(EXPRESSION_LESS_EQUAL 3 3)
(EXPRESSION_GREATER 8 5)
(EXPRESSION_GREATER_EQUAL 6 6)
```

**New Syntax:**
```aether
{5 == 5};
{3 != 4};
{2 < 7};
{3 <= 3};
{8 > 5};
{6 >= 6};
```

### Logical Operations

**Old Syntax:**
```aether
(EXPRESSION_AND TRUE FALSE)
(EXPRESSION_OR FALSE TRUE)
(EXPRESSION_NOT TRUE)
```

**New Syntax:**
```aether
{true && false};
{false || true};
{!true};
```

### Function Calls

**Old Syntax:**
```aether
(CALL_FUNCTION 'my_function'
  (ARGUMENTS
    (VARIABLE_REFERENCE 'arg1')
    (INTEGER_LITERAL 42)
  )
)
```

**New Syntax:**
```aether
// Single argument (no label required)
myFunction(arg1);

// Multiple arguments (labels required)
myFunction(first: arg1, second: 42);
```

### String Operations

**Old Syntax:**
```aether
(STRING_CONCAT (VARIABLE_REFERENCE 's1') (VARIABLE_REFERENCE 's2'))
(STRING_LENGTH (VARIABLE_REFERENCE 'str'))
(STRING_CHAR_AT (VARIABLE_REFERENCE 'str') 0)
```

**New Syntax:**
```aether
{s1.concat(s2)};
str.length();
str.charAt(0);
```

---

## Control Flow

### Conditional Execution (if/when)

**Old Syntax:**
```aether
(IF_STATEMENT
  (CONDITION (EXPRESSION_GREATER (VARIABLE_REFERENCE 'x') 0))
  (THEN_EXECUTE
    (EXPRESSION_STATEMENT (CALL_FUNCTION 'print' (ARGUMENTS (STRING_LITERAL "Positive"))))
  )
  (ELSE_EXECUTE
    (EXPRESSION_STATEMENT (CALL_FUNCTION 'print' (ARGUMENTS (STRING_LITERAL "Not positive"))))
  )
)
```

**New Syntax:**
```aether
when {
    case ({x > 0}): print("Positive");
    else: print("Not positive");
}
```

### Multi-Case Conditionals

**Old Syntax:**
```aether
(IF_STATEMENT
  (CONDITION (EXPRESSION_GREATER (VARIABLE_REFERENCE 'score') 90))
  (THEN_EXECUTE (RETURN_VALUE "A"))
  (ELSE_EXECUTE
    (IF_STATEMENT
      (CONDITION (EXPRESSION_GREATER (VARIABLE_REFERENCE 'score') 80))
      (THEN_EXECUTE (RETURN_VALUE "B"))
      (ELSE_EXECUTE (RETURN_VALUE "C"))
    )
  )
)
```

**New Syntax:**
```aether
when {
    case ({score > 90}): return "A";
    case ({score > 80}): return "B";
    else: return "C";
}
```

### While Loop

**Old Syntax:**
```aether
(WHILE_LOOP
  (CONDITION (EXPRESSION_LESS (VARIABLE_REFERENCE 'i') 10))
  (BODY
    (EXPRESSION_STATEMENT (CALL_FUNCTION 'print_int' (ARGUMENTS (VARIABLE_REFERENCE 'i'))))
    (ASSIGN_VARIABLE 'i' (EXPRESSION_ADD (VARIABLE_REFERENCE 'i') 1))
  )
)
```

**New Syntax:**
```aether
while ({i < 10}) {
    printInt(i);
    i = {i + 1};
}
```

### For Loop

**Old Syntax:**
```aether
(FOR_LOOP
  (INIT (DECLARE_VARIABLE (NAME 'i') (TYPE INTEGER) (INITIAL_VALUE 0)))
  (CONDITION (EXPRESSION_LESS (VARIABLE_REFERENCE 'i') 10))
  (UPDATE (ASSIGN_VARIABLE 'i' (EXPRESSION_ADD (VARIABLE_REFERENCE 'i') 1)))
  (BODY
    (EXPRESSION_STATEMENT (CALL_FUNCTION 'print_int' (ARGUMENTS (VARIABLE_REFERENCE 'i'))))
  )
)
```

**New Syntax:**
```aether
for i in range(from: 0, to: 10) {
    printInt(i);
}
```

### For-Each Loop

**Old Syntax:**
```aether
(FOR_EACH_LOOP
  (ELEMENT 'item')
  (COLLECTION (VARIABLE_REFERENCE 'items'))
  (BODY
    (EXPRESSION_STATEMENT (CALL_FUNCTION 'process' (ARGUMENTS (VARIABLE_REFERENCE 'item'))))
  )
)
```

**New Syntax:**
```aether
for item in items {
    process(item);
}
```

---

## Pattern Matching

### Basic Match Expression

**Old Syntax:**
```aether
(MATCH_EXPRESSION result
  (CASE (Ok value)
    (STRING_CONCAT "Success: " (TO_STRING value)))
  (CASE (Error msg)
    (STRING_CONCAT "Error: " msg))
)
```

**New Syntax:**
```aether
match result {
    case .Ok(let value): {
        return {"Success: ".concat(value.toString())};
    };
    case .Error(let msg): {
        return {"Error: ".concat(msg)};
    };
}
```

### Pattern Matching with Wildcards

**Old Syntax:**
```aether
(MATCH_EXPRESSION option_value
  (CASE (Some value)
    (RETURN_VALUE (VARIABLE_REFERENCE 'value'))
  )
  (CASE _
    (RETURN_VALUE 0)
  )
)
```

**New Syntax:**
```aether
match optionValue {
    case .Some(let value): return value;
    case _: return 0;
}
```

---

## Error Handling

### Try-Catch-Finally

**Old Syntax:**
```aether
(TRY_EXECUTE
  (PROTECTED_BLOCK
    (CALL_FUNCTION 'risky_operation' (ARGUMENTS (VARIABLE_REFERENCE 'input')))
  )
  (CATCH_EXCEPTION
    (EXCEPTION_TYPE 'FileError')
    (BINDING_VARIABLE (NAME 'error') (TYPE 'FileError'))
    (HANDLER_BLOCK
      (CALL_FUNCTION 'log_error' (ARGUMENTS (FIELD_ACCESS (VARIABLE_REFERENCE 'error') 'message')))
    )
  )
  (FINALLY_EXECUTE
    (CLEANUP_BLOCK
      (CALL_FUNCTION 'cleanup_resources'))
  )
)
```

**New Syntax:**
```aether
try {
    riskyOperation(input);
} catch FileError as error {
    logError(message: error.message);
} finally {
    cleanupResources();
}
```

---

## FFI (Foreign Function Interface)

### Declaring External Functions

**Old Syntax:**
```aether
(DECLARE_EXTERNAL_FUNCTION
  (NAME 'tcp_listen')
  (LIBRARY "aether_runtime")
  (RETURNS INTEGER)
  (ACCEPTS_PARAMETER (NAME "port") (TYPE INTEGER))
)

(DECLARE_EXTERNAL_FUNCTION
  (NAME 'malloc')
  (LIBRARY "libc")
  (SYMBOL "malloc")
  (RETURNS (POINTER VOID))
  (ACCEPTS_PARAMETER (NAME "size") (TYPE SIZET))
)
```

**New Syntax:**
```aether
@extern(library: "aether_runtime")
func tcpListen(port: Int) -> Int;

@extern(library: "libc", symbol: "malloc")
func malloc(size: SizeT) -> Pointer<Void>;
```

### Complex FFI with Ownership

**Old Syntax:**
```aether
(DECLARE_EXTERNAL_FUNCTION
  (NAME 'c_malloc')
  (LIBRARY "libc")
  (SYMBOL "malloc")
  (RETURNS (TYPE (POINTER VOID))
    (OWNERSHIP CALLER_OWNED)
    (DEALLOCATOR "free"))
  (ACCEPTS_PARAMETER (NAME "size") (TYPE SIZE_T) (PASSING BY_VALUE))
)
```

**New Syntax:**
```aether
@extern(library: "libc", symbol: "malloc")
@returns(ownership: .callerOwned, deallocator: "free")
func cMalloc(size: SizeT) -> Pointer<Void>;
```

---

## Resource Management

### Resource Scopes

**Old Syntax:**
```aether
(RESOURCE_SCOPE
  (SCOPE_ID "file_operation")
  (ACQUIRES
    (RESOURCE (TYPE "file_handle") (ID "file") (CLEANUP "close_file"))
  )
  (CLEANUP_GUARANTEED TRUE)
  (BODY
    (DECLARE_VARIABLE (NAME 'file')
      (INITIAL_VALUE (CALL_FUNCTION 'open_file' (ARGUMENTS (STRING_LITERAL "data.txt")))))

    (IF_CONDITION (PREDICATE_EQUALS (VARIABLE_REFERENCE 'file') (NULL_LITERAL))
      (THEN_EXECUTE (THROW_EXCEPTION "FileNotFoundError" "Could not open file")))

    (DECLARE_VARIABLE (NAME 'content')
      (INITIAL_VALUE (CALL_FUNCTION 'read_file' (ARGUMENTS (VARIABLE_REFERENCE 'file')))))

    (RETURN_VALUE (VARIABLE_REFERENCE 'content'))
  )
)
```

**New Syntax:**
```aether
@resource_scope(id: "file_operation")
func readFileContent() -> String {
    resource file: FileHandle = openFile("data.txt") {
        cleanup: closeFile;
        guaranteed: true;
    };

    when {
        case ({file == nil}): {
            throw FileNotFoundError("Could not open file");
        };
        else: {
            let content: String = readFile(file);
            return content;
        };
    }
}
```

---

## Complete Example: Hello World

### Old Syntax
```aether
(DEFINE_MODULE
  (NAME 'hello_world')
  (INTENT "A simple hello world program")
  (CONTENT
    (DEFINE_FUNCTION
      (NAME 'main')
      (RETURNS INTEGER)
      (INTENT "Main entry point that prints hello world")
      (BODY
        (EXPRESSION_STATEMENT
          (CALL_FUNCTION 'puts' (ARGUMENTS (STRING_LITERAL "Hello, World!")))
        )
        (RETURN_VALUE 0)
      )
    )
  )
)
```

### New Syntax
```aether
module HelloWorld {
    /// A simple hello world program

    /// Main entry point that prints hello world
    func main() -> Int {
        puts("Hello, World!");
        return 0;
    }
}
```

---

## Complete Example: HTTP Server

### Old Syntax
```aether
(DEFINE_MODULE
  (NAME 'blog_server')
  (INTENT "Simple HTTP blog server")
  (CONTENT
    (DECLARE_EXTERNAL_FUNCTION
      (NAME 'tcp_listen')
      (LIBRARY "aether_runtime")
      (RETURNS INTEGER)
      (ACCEPTS_PARAMETER (NAME "port") (TYPE INTEGER)))

    (DECLARE_EXTERNAL_FUNCTION
      (NAME 'tcp_accept')
      (LIBRARY "aether_runtime")
      (RETURNS INTEGER)
      (ACCEPTS_PARAMETER (NAME "listener_id") (TYPE INTEGER)))

    (DECLARE_EXTERNAL_FUNCTION
      (NAME 'tcp_write')
      (LIBRARY "aether_runtime")
      (RETURNS INTEGER)
      (ACCEPTS_PARAMETER (NAME "socket_id") (TYPE INTEGER))
      (ACCEPTS_PARAMETER (NAME "data") (TYPE STRING))
      (ACCEPTS_PARAMETER (NAME "data_size") (TYPE INTEGER)))

    (DEFINE_FUNCTION
      (NAME 'server_loop')
      (RETURNS INTEGER)
      (INTENT "Handle incoming connections and serve blog content")
      (ACCEPTS_PARAMETER (NAME "server_fd") (TYPE INTEGER))
      (BODY
        (DECLARE_VARIABLE (NAME "client_fd") (TYPE INTEGER))
        (ASSIGN (TARGET_VARIABLE client_fd)
                (SOURCE_EXPRESSION (CALL_FUNCTION tcp_accept server_fd)))

        (DECLARE_VARIABLE (NAME "response") (TYPE STRING))
        (ASSIGN (TARGET_VARIABLE response)
                (SOURCE_EXPRESSION "HTTP/1.1 200 OK\nContent-Type: text/html\n\n<html><body><h1>Blog</h1></body></html>"))

        (CALL_FUNCTION tcp_write client_fd response 1000)
        (RETURN_VALUE (CALL_FUNCTION server_loop server_fd))))

    (DEFINE_FUNCTION
      (NAME 'main')
      (RETURNS INTEGER)
      (BODY
        (DECLARE_VARIABLE (NAME server_fd) (TYPE INTEGER))
        (ASSIGN (TARGET_VARIABLE server_fd)
                (SOURCE_EXPRESSION (CALL_FUNCTION tcp_listen 8080)))
        (RETURN_VALUE (CALL_FUNCTION server_loop server_fd))))
  )
)
```

### New Syntax
```aether
module BlogServer {
    /// Simple HTTP blog server

    @extern(library: "aether_runtime")
    func tcpListen(port: Int) -> Int;

    @extern(library: "aether_runtime")
    func tcpAccept(listenerId: Int) -> Int;

    @extern(library: "aether_runtime")
    func tcpWrite(socketId: Int, data: String, dataSize: Int) -> Int;

    /// Handle incoming connections and serve blog content
    func serverLoop(serverFd: Int) -> Int {
        let clientFd: Int = tcpAccept(serverFd);

        let response: String = "HTTP/1.1 200 OK\nContent-Type: text/html\n\n<html><body><h1>Blog</h1></body></html>";

        tcpWrite(socketId: clientFd, data: response, dataSize: 1000);
        return serverLoop(serverFd);
    }

    func main() -> Int {
        let serverFd: Int = tcpListen(8080);
        return serverLoop(serverFd);
    }
}
```

---

## Key Migration Notes

### 1. Expression Grouping
- **V1**: Nested S-expressions naturally group operations
- **V2**: Explicit braces required for ALL operations: `{a + b}`

### 2. Control Flow
- **V1**: `IF_STATEMENT` with nested conditions
- **V2**: `when` blocks with cases (no if/else)

### 3. Function Calls
- **V1**: `(CALL_FUNCTION 'name' (ARGUMENTS ...))`
- **V2**: Single arg: `func(arg)`, Multiple: `func(arg1: value1, arg2: value2)`

### 4. Naming Conventions
- **V1**: `UPPERCASE_SNAKE_CASE` for keywords, 'single_quotes' for identifiers
- **V2**: `camelCase` for functions/variables, `PascalCase` for types

### 5. Comments
- **V1**: `;` or `;;` for comments
- **V2**: `//` for line comments, `///` for doc comments

### 6. Statement Termination
- **V1**: S-expressions self-terminate with `)`
- **V2**: ALL statements require semicolons `;`

### 7. Type Annotations
- **V1**: `(TYPE INTEGER)` wrapped in expressions
- **V2**: `: Int` inline type annotations

---

## Benefits of V2 Syntax

1. **More Readable**: Familiar to developers from Swift/Rust/TypeScript backgrounds
2. **Still Unambiguous**: Strict rules eliminate parsing ambiguity
3. **LLM-Optimized**: Consistent structure makes generation reliable
4. **Shorter**: Less verbose than S-expressions
5. **Better IDE Support**: Easier to build tooling for conventional syntax

## Backwards Compatibility

The compiler could potentially support both syntaxes during a transition period:
- V1 files: `.aether` extension
- V2 files: `.aether2` or `.aeth` extension

A transpiler tool could automatically convert V1 to V2 syntax.