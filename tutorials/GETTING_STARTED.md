# Getting Started with AetherScript

## Introduction

Welcome to AetherScript! This tutorial will guide you through the basics of programming in AetherScript, a modern systems programming language with S-expression syntax and a powerful ownership system.

## Installation and Setup

### Prerequisites
- Rust toolchain (for building the compiler)
- LLVM 14+ (for code generation)
- A text editor or IDE

### Building the Compiler
```bash
git clone <repository-url>
cd aether-compiler
cargo build --release
```

### Running Your First Program
```bash
./target/release/aether-compiler compile examples/hello_world.aether
./hello_world
```

## Basic Concepts

### 1. S-Expression Syntax

AetherScript uses S-expression (symbolic expression) syntax, where all code is structured as nested lists in parentheses:

```aether
(KEYWORD
  (FIELD value)
  (FIELD value))
```

This provides:
- **Consistency**: All constructs follow the same pattern  
- **Metaprogramming**: Easy code manipulation and generation
- **Clarity**: Explicit structure makes parsing unambiguous

### 2. Type System

AetherScript has a strong static type system with several categories of types:

#### Primitive Types
```aether
INTEGER     ; Platform-dependent signed integer
INTEGER32   ; 32-bit signed integer  
INTEGER64   ; 64-bit signed integer
FLOAT       ; Platform-dependent floating point
STRING      ; UTF-8 encoded string
BOOLEAN     ; True/false value
```

#### Compound Types
```aether
(TYPE (ARRAY INTEGER))           ; Array of integers
(TYPE (MAP STRING INTEGER))      ; Map from strings to integers
(TYPE (POINTER INTEGER))         ; Raw pointer to integer
```

### 3. Ownership System

AetherScript's ownership system prevents memory leaks and data races through three ownership kinds:

- **Owned (`^`)**: Exclusive ownership, automatically cleaned up
- **Borrowed (`&`)**: Temporary immutable access  
- **Borrowed Mutable (`&mut`)**: Temporary mutable access
- **Shared (`~`)**: Reference-counted shared ownership

## Your First Program

Let's start with the classic "Hello World":

```aether
;; hello_world.aether
(DEFINE_MODULE
  (NAME 'hello_world')
  (INTENT "A simple hello world program")
  (CONTENT
    (DEFINE_FUNCTION
      (NAME 'main')
      (RETURNS INTEGER)
      (INTENT "Main entry point")
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

To compile and run:
```bash
aether-compiler compile hello_world.aether
./hello_world
```

## Working with Variables

### Constants
Constants are values that never change:

```aether
(DECLARE_CONSTANT
  (NAME 'PI')
  (TYPE FLOAT)
  (VALUE 3.14159)
  (INTENT "Mathematical constant pi")
)
```

### Variables
Variables can be modified after declaration:

```aether
(DECLARE_VARIABLE
  (NAME 'counter')
  (TYPE INTEGER)
  (INITIAL_VALUE 0)
)

(ASSIGN_VARIABLE 'counter' 10)
```

## Functions

Functions are the building blocks of AetherScript programs:

```aether
(DEFINE_FUNCTION
  (NAME 'add_numbers')
  (ACCEPTS_PARAMETER (NAME 'a') (TYPE INTEGER))
  (ACCEPTS_PARAMETER (NAME 'b') (TYPE INTEGER))
  (RETURNS INTEGER)
  (INTENT "Adds two integers together")
  (BODY
    (RETURN_VALUE 
      (EXPRESSION_ADD 
        (VARIABLE_REFERENCE 'a') 
        (VARIABLE_REFERENCE 'b')
      )
    )
  )
)
```

## Control Flow

### If Statements
```aether
(IF_STATEMENT
  (CONDITION (EXPRESSION_GREATER (VARIABLE_REFERENCE 'x') 0))
  (THEN_EXECUTE
    (EXPRESSION_STATEMENT 
      (CALL_FUNCTION 'puts' (ARGUMENTS (STRING_LITERAL "Positive")))
    )
  )
  (ELSE_EXECUTE
    (EXPRESSION_STATEMENT 
      (CALL_FUNCTION 'puts' (ARGUMENTS (STRING_LITERAL "Not positive")))
    )
  )
)
```

### While Loops
```aether
(WHILE_LOOP
  (CONDITION (EXPRESSION_LESS (VARIABLE_REFERENCE 'i') 10))
  (BODY
    (EXPRESSION_STATEMENT 
      (CALL_FUNCTION 'puts' 
        (ARGUMENTS (CALL_FUNCTION 'int_to_string' (ARGUMENTS (VARIABLE_REFERENCE 'i'))))
      )
    )
    (ASSIGN_VARIABLE 'i' (EXPRESSION_ADD (VARIABLE_REFERENCE 'i') 1))
  )
)
```

## Understanding Ownership

One of AetherScript's key features is its ownership system. Here's how it works:

### Owned Values (`^`)
```aether
(DECLARE_VARIABLE 
  (NAME 'my_string') 
  (TYPE ^STRING) 
  (INITIAL_VALUE (STRING_LITERAL "Hello"))
)
```

This creates an owned string. The variable `my_string` has exclusive ownership and is responsible for cleanup.

### Borrowed References (`&`)
```aether
(DEFINE_FUNCTION
  (NAME 'get_length')
  (ACCEPTS_PARAMETER (NAME 'text') (TYPE &STRING))
  (RETURNS INTEGER)
  (BODY
    (RETURN_VALUE (CALL_FUNCTION 'string_length' (ARGUMENTS (VARIABLE_REFERENCE 'text'))))
  )
)
```

This function borrows a string reference. It can read the string but not modify it, and doesn't take ownership.

### Moving Ownership
```aether
(DEFINE_FUNCTION
  (NAME 'consume_string')
  (ACCEPTS_PARAMETER (NAME 'text') (TYPE ^STRING))
  (RETURNS INTEGER)
  (BODY
    ; This function takes ownership of the string
    (RETURN_VALUE (CALL_FUNCTION 'string_length' (ARGUMENTS (VARIABLE_REFERENCE 'text'))))
  )
)

; Usage:
(DECLARE_VARIABLE (NAME 'my_text') (TYPE ^STRING) (INITIAL_VALUE (STRING_LITERAL "Hello")))
(DECLARE_VARIABLE 
  (NAME 'length') 
  (TYPE INTEGER) 
  (INITIAL_VALUE (CALL_FUNCTION 'consume_string' (ARGUMENTS (VARIABLE_REFERENCE 'my_text'))))
)
; my_text cannot be used after this point - ownership was moved
```

## Contracts and Safety

AetherScript supports contract-based programming for enhanced safety:

```aether
(DEFINE_FUNCTION
  (NAME 'safe_divide')
  (ACCEPTS_PARAMETER (NAME 'numerator') (TYPE INTEGER))
  (ACCEPTS_PARAMETER (NAME 'denominator') (TYPE INTEGER))
  (RETURNS INTEGER)
  (PRECONDITION (PREDICATE_NOT_EQUALS 'denominator' 0) ASSERT_FAIL "Division by zero")
  (POSTCONDITION (PREDICATE_GREATER_EQUAL 'result' 0) ASSERT_WARN "Unexpected negative result")
  (BODY
    (RETURN_VALUE (EXPRESSION_DIVIDE (VARIABLE_REFERENCE 'numerator') (VARIABLE_REFERENCE 'denominator')))
  )
)
```

Contracts help catch errors early and document function behavior.

## Complete Example: Factorial Calculator

Here's a complete program that demonstrates many AetherScript features:

```aether
(DEFINE_MODULE
  (NAME 'factorial_calculator')
  (INTENT "Demonstrates recursion, contracts, and I/O")
  (CONTENT
    
    (DECLARE_CONSTANT
      (NAME 'MAX_INPUT')
      (TYPE INTEGER)
      (VALUE 20)
      (INTENT "Maximum input to prevent overflow")
    )
    
    (DEFINE_FUNCTION
      (NAME 'factorial')
      (ACCEPTS_PARAMETER (NAME 'n') (TYPE INTEGER))
      (RETURNS INTEGER)
      (INTENT "Computes factorial recursively")
      (PRECONDITION (PREDICATE_GREATER_EQUAL 'n' 0) ASSERT_FAIL "n must be non-negative")
      (PRECONDITION (PREDICATE_LESS_EQUAL 'n' (VARIABLE_REFERENCE 'MAX_INPUT')) ASSERT_FAIL "n too large")
      (POSTCONDITION (PREDICATE_GREATER 'result' 0) ASSERT_FAIL "factorial must be positive")
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

    (DEFINE_FUNCTION
      (NAME 'main')
      (RETURNS INTEGER)
      (INTENT "Main program entry point")
      (BODY
        (EXPRESSION_STATEMENT 
          (CALL_FUNCTION 'puts' (ARGUMENTS (STRING_LITERAL "Factorial Calculator")))
        )
        
        ; Calculate factorial of 5
        (DECLARE_VARIABLE (NAME 'input') (TYPE INTEGER) (INITIAL_VALUE 5))
        (DECLARE_VARIABLE 
          (NAME 'result') 
          (TYPE INTEGER) 
          (INITIAL_VALUE (CALL_FUNCTION 'factorial' (ARGUMENTS (VARIABLE_REFERENCE 'input'))))
        )
        
        ; Display result
        (EXPRESSION_STATEMENT
          (CALL_FUNCTION 'puts' 
            (ARGUMENTS 
              (CALL_FUNCTION 'string_concat'
                (ARGUMENTS
                  (STRING_LITERAL "Factorial of 5 is: ")
                  (CALL_FUNCTION 'int_to_string' (ARGUMENTS (VARIABLE_REFERENCE 'result')))
                )
              )
            )
          )
        )
        
        (RETURN_VALUE 0)
      )
    )
  )
)
```

## Next Steps

1. **Explore Examples**: Check out the `examples/` directory for more sample programs
2. **Read the Language Reference**: See `LANGUAGE_REFERENCE.md` for comprehensive documentation
3. **Try the Standard Library**: Experiment with string, array, and I/O operations
4. **Build Something**: Start with a simple project like a calculator or text processor

## CLI Commands

The AetherScript compiler provides several useful commands:

```bash
# Compile a program
aether-compiler compile program.aether

# Type check without compilation
aether-compiler check program.aether

# View the AST
aether-compiler ast program.aether

# View tokens
aether-compiler tokens program.aether

# Compile and run
aether-compiler run program.aether

# Get help
aether-compiler --help
```

## Common Patterns

### Error Handling
```aether
; Check conditions before proceeding
(IF_STATEMENT
  (CONDITION (EXPRESSION_EQUALS (VARIABLE_REFERENCE 'input') NULL_VALUE))
  (THEN_EXECUTE (RETURN_VALUE -1))
  (ELSE_EXECUTE
    ; Proceed with normal processing
    (RETURN_VALUE (CALL_FUNCTION 'process_input' (ARGUMENTS (VARIABLE_REFERENCE 'input'))))
  )
)
```

### Resource Management
```aether
; Use ownership to ensure cleanup
(DEFINE_FUNCTION
  (NAME 'process_file')
  (ACCEPTS_PARAMETER (NAME 'filename') (TYPE &STRING))
  (RETURNS INTEGER)
  (BODY
    (DECLARE_VARIABLE 
      (NAME 'content') 
      (TYPE ^STRING) 
      (INITIAL_VALUE (CALL_FUNCTION 'read_file' (ARGUMENTS (VARIABLE_REFERENCE 'filename'))))
    )
    ; Process content...
    ; content is automatically cleaned up when function returns
    (RETURN_VALUE 0)
  )
)
```

Happy coding with AetherScript!