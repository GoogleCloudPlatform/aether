# AetherScript Language Reference Manual

## Table of Contents
1. [Introduction](#introduction)
2. [Syntax Overview](#syntax-overview)
3. [Types](#types)
4. [Ownership System](#ownership-system)
5. [Modules](#modules)
6. [Functions](#functions)
7. [Variables and Constants](#variables-and-constants)
8. [Expressions](#expressions)
9. [Statements](#statements)
10. [Control Flow](#control-flow)
11. [Pattern Matching](#pattern-matching)
12. [Contracts and Verification](#contracts-and-verification)
13. [Foreign Function Interface (FFI)](#foreign-function-interface-ffi)
14. [Resource Management](#resource-management)
15. [Standard Library](#standard-library)
16. [Examples](#examples)

## Introduction

AetherScript is a modern systems programming language that combines memory safety through an ownership system with S-expression syntax for enhanced metaprogramming capabilities. The language is designed for high-performance applications while maintaining safety and expressiveness. The syntax is designed for LLM-first programming, where explicit intent annotations and structured code enhance AI comprehension.

### Key Features
- **Ownership System**: Move, borrow, and shared ownership semantics for memory safety
- **S-Expression Syntax**: Lisp-like syntax for consistent structure and metaprogramming
- **Static Type System**: Strong static typing with type inference
- **Zero-Cost Abstractions**: High-level features with minimal runtime overhead
- **Contract-Based Programming**: Built-in support for preconditions, postconditions, and invariants
- **LLM-First Design**: Explicit intent annotations and structured code for AI comprehension
- **Web Development Ready**: HTTP server capabilities with FFI networking integration

## Syntax Overview

AetherScript uses S-expression (symbolic expression) syntax, where all code is structured as nested lists enclosed in parentheses.

### Basic Structure
```aether
(KEYWORD
  (FIELD value)
  (FIELD value)
  ...)
```

### Comments
```aether
; This is a line comment
;; This is also a line comment (conventional for documentation)
```

## Types

### Primitive Types

#### Numeric Types
- `INTEGER` - Platform-dependent signed integer
- `INTEGER32` - 32-bit signed integer
- `INTEGER64` - 64-bit signed integer
- `FLOAT` - Platform-dependent floating point
- `FLOAT32` - 32-bit floating point
- `FLOAT64` - 64-bit floating point

#### Other Primitives
- `STRING` - UTF-8 encoded string
- `CHARACTER` - Single Unicode character
- `BOOLEAN` - True/false value
- `VOID` - Unit type (no value)

#### Size Types
- `SIZET` - Platform-dependent unsigned size type
- `UINTPTRT` - Unsigned integer pointer type

### Compound Types

#### Arrays
```aether
(TYPE (ARRAY INTEGER))           ; Array of integers
(TYPE (ARRAY STRING 10))         ; Fixed-size array of 10 strings
```

#### Maps
```aether
(TYPE (MAP STRING INTEGER))      ; Map from strings to integers
```

#### Pointers
```aether
(TYPE (POINTER INTEGER))         ; Raw pointer to integer
(TYPE (POINTER_MUT INTEGER))     ; Mutable raw pointer to integer
```

#### Function Types
```aether
(TYPE (FUNCTION (INTEGER STRING) INTEGER))  ; Function taking int,string -> int
```

### Generic Types
```aether
(TYPE (GENERIC List T))          ; Generic List of type T
```

### Named Types
```aether
(TYPE (NAMED MyStruct))          ; User-defined named type
```

## Ownership System

AetherScript's ownership system prevents data races and memory leaks through three ownership kinds:

### Ownership Kinds

#### Owned (`^`)
```aether
(TYPE ^STRING)                   ; Owned string - exclusive ownership
```
- Exclusive ownership
- Can be moved or borrowed
- Automatically cleaned up when out of scope

#### Borrowed (`&`)
```aether
(TYPE &STRING)                   ; Immutable reference to string
(TYPE &mut STRING)               ; Mutable reference to string
```
- Temporary access to owned data
- Cannot outlive the owner
- Multiple immutable borrows allowed
- Only one mutable borrow at a time

#### Shared (`~`)
```aether
(TYPE ~STRING)                   ; Reference-counted shared string
```
- Reference-counted shared ownership
- Multiple owners allowed
- Automatically cleaned up when last reference dropped

### Ownership Rules
1. Each value has exactly one owner at any time
2. When the owner goes out of scope, the value is dropped
3. References must not outlive the data they refer to
4. No data races: either multiple readers OR one writer

## Modules

### Module Definition
```aether
(DEFINE_MODULE
  (NAME 'my_module')
  (INTENT "Description of module purpose")
  (CONTENT
    ; Module contents go here
  )
)
```

### Module Imports
```aether
(IMPORT_MODULE "std.io")         ; Import standard I/O module
(IMPORT_MODULE "my_other_module")
```

## Functions

### Function Definition
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

### Function with Ownership Parameters
```aether
(DEFINE_FUNCTION
  (NAME 'take_ownership')
  (ACCEPTS_PARAMETER (NAME 'data') (TYPE ^STRING))
  (RETURNS INTEGER)
  (BODY
    ; Function takes ownership of 'data'
    (RETURN_VALUE (CALL_FUNCTION 'string_length' (ARGUMENTS (VARIABLE_REFERENCE 'data'))))
  )
)
```

### Function with Borrowing
```aether
(DEFINE_FUNCTION
  (NAME 'borrow_data')
  (ACCEPTS_PARAMETER (NAME 'data') (TYPE &STRING))
  (RETURNS INTEGER)
  (BODY
    ; Function borrows 'data' immutably
    (RETURN_VALUE (CALL_FUNCTION 'string_length' (ARGUMENTS (VARIABLE_REFERENCE 'data'))))
  )
)
```

## Variables and Constants

### Variable Declaration
```aether
(DECLARE_VARIABLE
  (NAME 'my_var')
  (TYPE INTEGER)
  (INITIAL_VALUE 42)
)
```

### Constant Declaration
```aether
(DECLARE_CONSTANT
  (NAME 'PI')
  (TYPE FLOAT)
  (VALUE 3.14159)
  (INTENT "Mathematical constant pi")
)
```

### Variable Assignment
```aether
(ASSIGN_VARIABLE 'my_var' 100)
```

## Expressions

### Arithmetic Expressions
```aether
(EXPRESSION_ADD 1 2)             ; Addition: 1 + 2
(EXPRESSION_SUBTRACT 5 3)        ; Subtraction: 5 - 3
(EXPRESSION_MULTIPLY 4 6)        ; Multiplication: 4 * 6
(EXPRESSION_DIVIDE 8 2)          ; Division: 8 / 2
(EXPRESSION_MODULO 10 3)         ; Modulo: 10 % 3
```

### Comparison Expressions
```aether
(EXPRESSION_EQUALS 5 5)          ; Equality: 5 == 5
(EXPRESSION_NOT_EQUALS 3 4)      ; Inequality: 3 != 4
(EXPRESSION_LESS 2 7)            ; Less than: 2 < 7
(EXPRESSION_LESS_EQUAL 3 3)      ; Less or equal: 3 <= 3
(EXPRESSION_GREATER 8 5)         ; Greater than: 8 > 5
(EXPRESSION_GREATER_EQUAL 6 6)   ; Greater or equal: 6 >= 6
```

### Logical Expressions
```aether
(EXPRESSION_AND TRUE FALSE)      ; Logical AND
(EXPRESSION_OR FALSE TRUE)       ; Logical OR
(EXPRESSION_NOT TRUE)            ; Logical NOT
```

### Function Calls
```aether
(CALL_FUNCTION 'my_function'
  (ARGUMENTS
    (VARIABLE_REFERENCE 'arg1')
    (INTEGER_LITERAL 42)
  )
)
```

### Memory Operations
```aether
(ADDRESS_OF (VARIABLE_REFERENCE 'my_var'))    ; Take address of variable
(DEREFERENCE (VARIABLE_REFERENCE 'my_ptr'))   ; Dereference pointer
```

## Statements

### Expression Statement
```aether
(EXPRESSION_STATEMENT
  (CALL_FUNCTION 'print' (ARGUMENTS (STRING_LITERAL "Hello World")))
)
```

### Block Statement
```aether
(BLOCK
  (DECLARE_VARIABLE (NAME 'x') (TYPE INTEGER) (INITIAL_VALUE 10))
  (ASSIGN_VARIABLE 'x' 20)
  (EXPRESSION_STATEMENT (CALL_FUNCTION 'print_int' (ARGUMENTS (VARIABLE_REFERENCE 'x'))))
)
```

### Return Statement
```aether
(RETURN_VALUE (INTEGER_LITERAL 42))
(RETURN_VOID)
```

## Control Flow

### If Statement
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

### While Loop
```aether
(WHILE_LOOP
  (CONDITION (EXPRESSION_LESS (VARIABLE_REFERENCE 'i') 10))
  (BODY
    (EXPRESSION_STATEMENT (CALL_FUNCTION 'print_int' (ARGUMENTS (VARIABLE_REFERENCE 'i'))))
    (ASSIGN_VARIABLE 'i' (EXPRESSION_ADD (VARIABLE_REFERENCE 'i') 1))
  )
)
```

### For Loop
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

## Pattern Matching

### Match Expression
```aether
(MATCH_EXPRESSION
  (VALUE (VARIABLE_REFERENCE 'option_value'))
  (CASE "Some"
    (BINDING 'value')
    (RETURN_VALUE (VARIABLE_REFERENCE 'value'))
  )
  (CASE "None"
    (RETURN_VALUE (INTEGER_LITERAL 0))
  )
)
```

## Contracts and Verification

### Function Preconditions
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

### Function Postconditions
```aether
(DEFINE_FUNCTION
  (NAME 'absolute_value')
  (ACCEPTS_PARAMETER (NAME 'x') (TYPE INTEGER))
  (RETURNS INTEGER)
  (POSTCONDITION (PREDICATE_GREATER_EQUAL 'result' 0) ASSERT_FAIL "Result must be non-negative")
  (BODY
    (IF_STATEMENT
      (CONDITION (EXPRESSION_LESS (VARIABLE_REFERENCE 'x') 0))
      (THEN_EXECUTE (RETURN_VALUE (EXPRESSION_MULTIPLY (VARIABLE_REFERENCE 'x') -1)))
      (ELSE_EXECUTE (RETURN_VALUE (VARIABLE_REFERENCE 'x')))
    )
  )
)
```

## Standard Library

### String Module (`std.string`)
- `string_length(s: &STRING) -> INTEGER` - Get string length
- `string_concat(a: &STRING, b: &STRING) -> ^STRING` - Concatenate strings
- `string_char_at(s: &STRING, index: INTEGER) -> CHARACTER` - Get character at index
- `string_to_upper(s: &STRING) -> ^STRING` - Convert to uppercase
- `string_to_lower(s: &STRING) -> ^STRING` - Convert to lowercase

### Array Module (`std.collections`)
- `array_create(size: INTEGER) -> ^(ARRAY T)` - Create new array
- `array_length(arr: &(ARRAY T)) -> INTEGER` - Get array length
- `array_get(arr: &(ARRAY T), index: INTEGER) -> &T` - Get element at index
- `array_set(arr: &mut (ARRAY T), index: INTEGER, value: T)` - Set element at index

### I/O Module (`std.io`)
- `print(s: &STRING)` - Print string to stdout
- `println(s: &STRING)` - Print string with newline
- `read_file(path: &STRING) -> ^STRING` - Read file contents
- `write_file(path: &STRING, content: &STRING) -> BOOLEAN` - Write to file

### Math Module (`std.math`)
- `abs(x: INTEGER) -> INTEGER` - Absolute value
- `min(a: T, b: T) -> T` - Minimum of two values
- `max(a: T, b: T) -> T` - Maximum of two values
- `sqrt(x: FLOAT) -> FLOAT` - Square root

### Memory Module (`std.memory`)
- `malloc(size: SIZET) -> POINTER` - Allocate memory
- `free(ptr: POINTER)` - Free allocated memory
- `realloc(ptr: POINTER, size: SIZET) -> POINTER` - Reallocate memory

## Examples

### Hello World
```aether
(DEFINE_MODULE
  (NAME 'hello_world')
  (CONTENT
    (DEFINE_FUNCTION
      (NAME 'main')
      (RETURNS INTEGER)
      (BODY
        (EXPRESSION_STATEMENT
          (CALL_FUNCTION 'println' (ARGUMENTS (STRING_LITERAL "Hello, World!")))
        )
        (RETURN_VALUE 0)
      )
    )
  )
)
```

### Factorial Function
```aether
(DEFINE_FUNCTION
  (NAME 'factorial')
  (ACCEPTS_PARAMETER (NAME 'n') (TYPE INTEGER))
  (RETURNS INTEGER)
  (PRECONDITION (PREDICATE_GREATER_EQUAL 'n' 0) ASSERT_FAIL "n must be non-negative")
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

### HTTP Blog Server Example
```aether
;; Complete working HTTP server serving styled blog content
(DEFINE_MODULE
  (NAME 'blog_listen')
  (INTENT "Blog server that listens on port 8080")
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
                (SOURCE_EXPRESSION "HTTP/1.1 200 OK\nContent-Type: text/html\n\n<!DOCTYPE html><html><head><title>AetherScript LLM Blog</title><style>body{font-family:sans-serif;max-width:800px;margin:20px auto;}</style></head><body><h1>AetherScript LLM Blog</h1><h2>Welcome to LLM-First Programming</h2><p>This blog demonstrates AetherScript's LLM-first design principles.</p></body></html>\n"))

        (CALL_FUNCTION tcp_write client_fd response 1300)
        (CALL_FUNCTION tcp_close client_fd)
        (RETURN_VALUE (CALL_FUNCTION server_loop server_fd))))

    (DEFINE_FUNCTION
      (NAME 'main')
      (RETURNS INTEGER)
      (INTENT "Start blog server on port 8080")
      (BODY
        (DECLARE_VARIABLE (NAME server_fd) (TYPE INTEGER))
        (ASSIGN (TARGET_VARIABLE server_fd)
                (SOURCE_EXPRESSION (CALL_FUNCTION tcp_listen 8080)))
        (RETURN_VALUE (CALL_FUNCTION server_loop server_fd))))))
```

### Ownership Example
```aether
(DEFINE_FUNCTION
  (NAME 'ownership_example')
  (RETURNS INTEGER)
  (BODY
    ; Create owned string
    (DECLARE_VARIABLE
      (NAME 'my_string')
      (TYPE ^STRING)
      (INITIAL_VALUE (STRING_LITERAL "Hello"))
    )

    ; Borrow the string immutably
    (DECLARE_VARIABLE
      (NAME 'string_ref')
      (TYPE &STRING)
      (INITIAL_VALUE (BORROW (VARIABLE_REFERENCE 'my_string')))
    )

    ; Use the borrowed reference
    (DECLARE_VARIABLE
      (NAME 'length')
      (TYPE INTEGER)
      (INITIAL_VALUE (CALL_FUNCTION 'string_length' (ARGUMENTS (VARIABLE_REFERENCE 'string_ref'))))
    )

    (RETURN_VALUE (VARIABLE_REFERENCE 'length'))
  )
)
```

### Error Handling Pattern
```aether
(DEFINE_FUNCTION
  (NAME 'safe_file_read')
  (ACCEPTS_PARAMETER (NAME 'filename') (TYPE &STRING))
  (RETURNS (TYPE (GENERIC Result ^STRING STRING)))
  (BODY
    (MATCH_EXPRESSION
      (VALUE (CALL_FUNCTION 'file_exists' (ARGUMENTS (VARIABLE_REFERENCE 'filename'))))
      (CASE TRUE
        (DECLARE_VARIABLE
          (NAME 'content')
          (TYPE ^STRING)
          (INITIAL_VALUE (CALL_FUNCTION 'read_file' (ARGUMENTS (VARIABLE_REFERENCE 'filename'))))
        )
        (RETURN_VALUE (CONSTRUCTOR 'Ok' (VARIABLE_REFERENCE 'content')))
      )
      (CASE FALSE
        (RETURN_VALUE (CONSTRUCTOR 'Err' (STRING_LITERAL "File not found")))
      )
    )
  )
)
```

## Foreign Function Interface (FFI)

### 13.1. Declaring External Functions

AetherScript provides comprehensive FFI support for integrating with C, Rust, and Go libraries:

```aether
(DECLARE_EXTERNAL_FUNCTION
  (NAME 'http_create_server')
  (LIBRARY "aether_runtime")
  (SYMBOL "http_create_server")
  (RETURNS INTEGER)
  (ACCEPTS_PARAMETER (NAME 'port') (TYPE INTEGER))
  (ACCEPTS_PARAMETER (NAME 'handler_ptr') (TYPE (POINTER VOID)))
  (INTENT "Creates HTTP server on specified port with callback handler")
)
```

### 13.2. Memory Management in FFI

AetherScript automatically handles memory allocation and deallocation for FFI calls:

```aether
;; Function returns allocated C string - AetherScript manages cleanup
(DECLARE_EXTERNAL_FUNCTION
  (NAME 'http_get_request_path')
  (LIBRARY "aether_runtime")
  (SYMBOL "http_get_request_path")
  (RETURNS (POINTER CHARACTER))
  (ACCEPTS_PARAMETER (NAME 'request_ctx') (TYPE (POINTER VOID)))
  (INTENT "Returns allocated path string - caller responsible for freeing")
)

;; Usage - AetherScript handles C string conversion
(DECLARE_VARIABLE
  (NAME 'path_c_str')
  (TYPE (POINTER CHARACTER))
  (INITIAL_VALUE (CALL_FUNCTION 'http_get_request_path' (ARGUMENTS (VARIABLE_REFERENCE 'context'))))
)

(DECLARE_VARIABLE
  (NAME 'path')
  (TYPE STRING)
  (INITIAL_VALUE (CALL_FUNCTION 'c_string_to_aether' (ARGUMENTS (VARIABLE_REFERENCE 'path_c_str'))))
)
```

### 13.3. Callback Functions

Pass AetherScript functions to external libraries as callbacks:

```aether
;; Define callback function
(DEFINE_FUNCTION
  (NAME 'http_request_handler')
  (ACCEPTS_PARAMETER (NAME 'request_context') (TYPE (POINTER VOID)))
  (RETURNS VOID)
  (BODY
    ;; Handle HTTP request
    (CALL_FUNCTION 'http_send_response'
      (ARGUMENTS
        (VARIABLE_REFERENCE 'request_context')
        (INTEGER_LITERAL 200)
        (STRING_LITERAL "text/html")
        (STRING_LITERAL "<h1>Hello from AetherScript!</h1>")
      )
    )
  )
)

;; Pass function pointer to external library
(DECLARE_VARIABLE
  (NAME 'server_handle')
  (TYPE INTEGER)
  (INITIAL_VALUE
    (CALL_FUNCTION 'http_create_server'
      (ARGUMENTS
        (INTEGER_LITERAL 8080)
        (ADDRESS_OF 'http_request_handler')  ;; Function pointer
      )
    )
  )
)
```

## Resource Management

### 14.1. RESOURCE_SCOPE

RESOURCE_SCOPE provides deterministic resource management with guaranteed cleanup:

```aether
(RESOURCE_SCOPE
  (SCOPE_ID "http_server_lifecycle")
  (ACQUIRES
    (RESOURCE
      (TYPE INTEGER)
      (ID "server_handle")
      (CLEANUP "http_stop_server")  ;; Cleanup function called automatically
    )
  )
  (CLEANUP_GUARANTEED TRUE)
  (CLEANUP_ORDER REVERSE_ACQUISITION)
  (BODY
    (DECLARE_VARIABLE
      (NAME 'server_handle')
      (TYPE INTEGER)
      (INITIAL_VALUE
        (CALL_FUNCTION 'http_create_server'
          (ARGUMENTS
            (INTEGER_LITERAL 8080)
            (ADDRESS_OF 'http_request_handler')
          )
        )
      )
    )

    ;; Server operations here
    ;; http_stop_server will be called automatically when scope exits
    (RETURN_VALUE 0)
  )
)
```

### 14.2. Exception-Safe Resource Management

RESOURCE_SCOPE works with exception handling to ensure cleanup even during errors:

```aether
(RESOURCE_SCOPE
  (SCOPE_ID "file_processing")
  (ACQUIRES
    (RESOURCE (TYPE (POINTER FILE)) (ID "file_handle") (CLEANUP "close_file"))
    (RESOURCE (TYPE (POINTER VOID)) (ID "buffer") (CLEANUP "free_memory"))
  )
  (CLEANUP_GUARANTEED TRUE)
  (BODY
    (TRY_EXECUTE
      (PROTECTED_BLOCK
        ;; Acquire resources
        (DECLARE_VARIABLE
          (NAME 'file_handle')
          (TYPE (POINTER FILE))
          (INITIAL_VALUE (CALL_FUNCTION 'open_file' (ARGUMENTS (STRING_LITERAL "data.txt"))))
        )

        (DECLARE_VARIABLE
          (NAME 'buffer')
          (TYPE (POINTER VOID))
          (INITIAL_VALUE (CALL_FUNCTION 'allocate_memory' (ARGUMENTS (INTEGER_LITERAL 1024))))
        )

        ;; Process file
        (CALL_FUNCTION 'read_file_data'
          (ARGUMENTS (VARIABLE_REFERENCE 'file_handle') (VARIABLE_REFERENCE 'buffer')))
      )
      (CATCH_EXCEPTION
        (EXCEPTION_TYPE 'io_error')
        (BINDING_VARIABLE (NAME 'error') (TYPE 'io_error'))
        (HANDLER_BLOCK
          ;; Resources are still cleaned up automatically
          (RETURN_VALUE -1)
        )
      )
      (FINALLY_EXECUTE
        (CLEANUP_BLOCK
          ;; Additional cleanup logic if needed
          (EXPRESSION_STATEMENT (CALL_FUNCTION 'log_info' (ARGUMENTS (STRING_LITERAL "File processing complete"))))
        )
      )
    )
  )
)
```

### 14.3. Resource Contracts

Specify resource usage limits and constraints:

```aether
(DEFINE_FUNCTION
  (NAME 'process_large_dataset')
  (RETURNS INTEGER)
  (RESOURCE_CONTRACT
    (MAX_MEMORY_MB 100)          ;; Maximum 100MB memory usage
    (MAX_EXECUTION_TIME_MS 5000) ;; Maximum 5 second execution time
    (ENFORCEMENT RUNTIME)        ;; Check limits at runtime
  )
  (BODY
    ;; Function implementation with automatic resource monitoring
    (RETURN_VALUE 0)
  )
)
```

---

This manual provides a comprehensive overview of the AetherScript language. For more detailed examples and advanced topics, consult the tutorial documentation and example programs.
