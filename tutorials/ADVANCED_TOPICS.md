# Advanced AetherScript Topics

## Table of Contents
1. [Advanced Ownership Patterns](#advanced-ownership-patterns)
2. [Generic Programming](#generic-programming)
3. [Pattern Matching](#pattern-matching) 
4. [Contract Programming](#contract-programming)
5. [Metaprogramming](#metaprogramming)
6. [Performance Optimization](#performance-optimization)
7. [Interoperability](#interoperability)
8. [Concurrency Patterns](#concurrency-patterns)

## Advanced Ownership Patterns

### Lifetime Management

Understanding how ownership affects object lifetimes is crucial for writing efficient AetherScript code:

```aether
(DEFINE_FUNCTION
  (NAME 'demonstrate_lifetimes')
  (RETURNS INTEGER)
  (BODY
    (DECLARE_VARIABLE (NAME 'outer_data') (TYPE ^STRING) (INITIAL_VALUE (STRING_LITERAL "outer")))
    
    (BLOCK
      (DECLARE_VARIABLE (NAME 'inner_data') (TYPE ^STRING) (INITIAL_VALUE (STRING_LITERAL "inner")))
      
      ; This would be invalid - inner_data reference cannot outlive the inner scope
      ; (ASSIGN_VARIABLE 'outer_ref' (BORROW (VARIABLE_REFERENCE 'inner_data')))
      
      ; This is valid - we're using the reference within its lifetime
      (DECLARE_VARIABLE 
        (NAME 'inner_ref') 
        (TYPE &STRING) 
        (INITIAL_VALUE (BORROW (VARIABLE_REFERENCE 'inner_data')))
      )
      ; inner_data and inner_ref are cleaned up here
    )
    
    ; outer_data is still valid here
    (RETURN_VALUE (CALL_FUNCTION 'string_length' (ARGUMENTS (VARIABLE_REFERENCE 'outer_data'))))
  )
)
```

### Reference Counting with Shared Ownership

For cases where multiple owners are needed, use shared ownership:

```aether
(DEFINE_FUNCTION
  (NAME 'shared_ownership_example')
  (RETURNS INTEGER)
  (BODY
    ; Create shared data
    (DECLARE_VARIABLE 
      (NAME 'shared_data') 
      (TYPE ~STRING) 
      (INITIAL_VALUE (SHARE (STRING_LITERAL "shared between multiple owners")))
    )
    
    ; Multiple variables can reference the same data
    (DECLARE_VARIABLE 
      (NAME 'reference1') 
      (TYPE ~STRING) 
      (INITIAL_VALUE (CLONE (VARIABLE_REFERENCE 'shared_data')))
    )
    
    (DECLARE_VARIABLE 
      (NAME 'reference2') 
      (TYPE ~STRING) 
      (INITIAL_VALUE (CLONE (VARIABLE_REFERENCE 'shared_data')))
    )
    
    ; All references are automatically cleaned up via reference counting
    (RETURN_VALUE 0)
  )
)
```

### Move Semantics

Understanding when and how values are moved:

```aether
(DEFINE_FUNCTION
  (NAME 'move_semantics_demo')
  (RETURNS INTEGER)
  (BODY
    (DECLARE_VARIABLE (NAME 'original') (TYPE ^STRING) (INITIAL_VALUE (STRING_LITERAL "original")))
    
    ; Explicit move - original becomes unusable
    (DECLARE_VARIABLE 
      (NAME 'moved_to') 
      (TYPE ^STRING) 
      (INITIAL_VALUE (MOVE (VARIABLE_REFERENCE 'original')))
    )
    
    ; This would cause a compile error:
    ; (CALL_FUNCTION 'string_length' (ARGUMENTS (VARIABLE_REFERENCE 'original')))
    
    ; But this is fine:
    (RETURN_VALUE (CALL_FUNCTION 'string_length' (ARGUMENTS (VARIABLE_REFERENCE 'moved_to'))))
  )
)
```

## Generic Programming

### Generic Functions

Create functions that work with multiple types:

```aether
(DEFINE_GENERIC_FUNCTION
  (NAME 'swap')
  (TYPE_PARAMETERS (T))
  (ACCEPTS_PARAMETER (NAME 'a') (TYPE &mut T))
  (ACCEPTS_PARAMETER (NAME 'b') (TYPE &mut T))
  (RETURNS VOID)
  (INTENT "Swaps the values of two variables")
  (BODY
    (DECLARE_VARIABLE (NAME 'temp') (TYPE T) (INITIAL_VALUE (DEREFERENCE (VARIABLE_REFERENCE 'a'))))
    (ASSIGN_VARIABLE (DEREFERENCE (VARIABLE_REFERENCE 'a')) (DEREFERENCE (VARIABLE_REFERENCE 'b')))
    (ASSIGN_VARIABLE (DEREFERENCE (VARIABLE_REFERENCE 'b')) (VARIABLE_REFERENCE 'temp'))
  )
)
```

### Generic Data Structures

Define reusable data structures:

```aether
(DEFINE_GENERIC_STRUCT
  (NAME 'Option')
  (TYPE_PARAMETERS (T))
  (VARIANTS
    (VARIANT 'Some' (TYPE T))
    (VARIANT 'None')
  )
)

(DEFINE_GENERIC_FUNCTION
  (NAME 'unwrap_or')
  (TYPE_PARAMETERS (T))
  (ACCEPTS_PARAMETER (NAME 'option') (TYPE (Option T)))
  (ACCEPTS_PARAMETER (NAME 'default') (TYPE T))
  (RETURNS T)
  (BODY
    (MATCH_EXPRESSION
      (VALUE (VARIABLE_REFERENCE 'option'))
      (CASE 'Some' (BINDING 'value') (RETURN_VALUE (VARIABLE_REFERENCE 'value')))
      (CASE 'None' (RETURN_VALUE (VARIABLE_REFERENCE 'default')))
    )
  )
)
```

## Pattern Matching

### Advanced Pattern Matching

Pattern matching provides powerful ways to destructure data:

```aether
(DEFINE_ENUM
  (NAME 'Result')
  (TYPE_PARAMETERS (T E))
  (VARIANTS
    (VARIANT 'Ok' (TYPE T))
    (VARIANT 'Err' (TYPE E))
  )
)

(DEFINE_FUNCTION
  (NAME 'handle_result')
  (ACCEPTS_PARAMETER (NAME 'result') (TYPE (Result INTEGER STRING)))
  (RETURNS INTEGER)
  (BODY
    (MATCH_EXPRESSION
      (VALUE (VARIABLE_REFERENCE 'result'))
      
      ; Match successful result and extract value
      (CASE 'Ok' 
        (BINDING 'value') 
        (IF_STATEMENT
          (CONDITION (EXPRESSION_GREATER (VARIABLE_REFERENCE 'value') 0))
          (THEN_EXECUTE (RETURN_VALUE (VARIABLE_REFERENCE 'value')))
          (ELSE_EXECUTE (RETURN_VALUE 0))
        )
      )
      
      ; Match error and handle appropriately
      (CASE 'Err' 
        (BINDING 'error_msg')
        (EXPRESSION_STATEMENT 
          (CALL_FUNCTION 'puts' 
            (ARGUMENTS 
              (CALL_FUNCTION 'string_concat'
                (ARGUMENTS 
                  (STRING_LITERAL "Error: ")
                  (VARIABLE_REFERENCE 'error_msg')
                )
              )
            )
          )
        )
        (RETURN_VALUE -1)
      )
    )
  )
)
```

### Guard Conditions

Add conditional logic to pattern matches:

```aether
(MATCH_EXPRESSION
  (VALUE (VARIABLE_REFERENCE 'number'))
  (CASE (BINDING 'n') 
    (GUARD (EXPRESSION_GREATER (VARIABLE_REFERENCE 'n') 100))
    (RETURN_VALUE (STRING_LITERAL "Large number"))
  )
  (CASE (BINDING 'n')
    (GUARD (EXPRESSION_GREATER (VARIABLE_REFERENCE 'n') 0))
    (RETURN_VALUE (STRING_LITERAL "Positive number"))
  )
  (CASE (WILDCARD)
    (RETURN_VALUE (STRING_LITERAL "Zero or negative"))
  )
)
```

## Contract Programming

### Comprehensive Contract Example

Contracts provide formal specifications for function behavior:

```aether
(DEFINE_FUNCTION
  (NAME 'binary_search')
  (ACCEPTS_PARAMETER (NAME 'array') (TYPE &(ARRAY INTEGER)))
  (ACCEPTS_PARAMETER (NAME 'target') (TYPE INTEGER))
  (RETURNS INTEGER)
  (INTENT "Performs binary search on a sorted array")
  
  ; Preconditions - requirements for calling the function
  (PRECONDITION 
    (PREDICATE_GREATER (CALL_FUNCTION 'array_length' (ARGUMENTS (VARIABLE_REFERENCE 'array'))) 0)
    ASSERT_FAIL "Array must not be empty"
  )
  (PRECONDITION 
    (PREDICATE_SORTED (VARIABLE_REFERENCE 'array'))
    ASSERT_FAIL "Array must be sorted"
  )
  
  ; Postconditions - guarantees about the return value
  (POSTCONDITION 
    (PREDICATE_OR
      (PREDICATE_EQUALS 'result' -1)  ; Not found
      (PREDICATE_AND
        (PREDICATE_GREATER_EQUAL 'result' 0)
        (PREDICATE_LESS 'result' (CALL_FUNCTION 'array_length' (ARGUMENTS (VARIABLE_REFERENCE 'array'))))
        (PREDICATE_EQUALS 
          (CALL_FUNCTION 'array_get' (ARGUMENTS (VARIABLE_REFERENCE 'array') (VARIABLE_REFERENCE 'result')))
          (VARIABLE_REFERENCE 'target')
        )
      )
    )
    ASSERT_FAIL "Result must be valid index or -1"
  )
  
  (BODY
    (DECLARE_VARIABLE (NAME 'left') (TYPE INTEGER) (INITIAL_VALUE 0))
    (DECLARE_VARIABLE 
      (NAME 'right') 
      (TYPE INTEGER) 
      (INITIAL_VALUE (EXPRESSION_SUBTRACT (CALL_FUNCTION 'array_length' (ARGUMENTS (VARIABLE_REFERENCE 'array'))) 1))
    )
    
    (WHILE_LOOP
      (CONDITION (EXPRESSION_LESS_EQUAL (VARIABLE_REFERENCE 'left') (VARIABLE_REFERENCE 'right')))
      (BODY
        (DECLARE_VARIABLE 
          (NAME 'mid') 
          (TYPE INTEGER) 
          (INITIAL_VALUE 
            (EXPRESSION_DIVIDE 
              (EXPRESSION_ADD (VARIABLE_REFERENCE 'left') (VARIABLE_REFERENCE 'right'))
              2
            )
          )
        )
        
        (DECLARE_VARIABLE 
          (NAME 'mid_value') 
          (TYPE INTEGER) 
          (INITIAL_VALUE (CALL_FUNCTION 'array_get' (ARGUMENTS (VARIABLE_REFERENCE 'array') (VARIABLE_REFERENCE 'mid'))))
        )
        
        (IF_STATEMENT
          (CONDITION (EXPRESSION_EQUALS (VARIABLE_REFERENCE 'mid_value') (VARIABLE_REFERENCE 'target')))
          (THEN_EXECUTE (RETURN_VALUE (VARIABLE_REFERENCE 'mid')))
          (ELSE_EXECUTE
            (IF_STATEMENT
              (CONDITION (EXPRESSION_LESS (VARIABLE_REFERENCE 'mid_value') (VARIABLE_REFERENCE 'target')))
              (THEN_EXECUTE (ASSIGN_VARIABLE 'left' (EXPRESSION_ADD (VARIABLE_REFERENCE 'mid') 1)))
              (ELSE_EXECUTE (ASSIGN_VARIABLE 'right' (EXPRESSION_SUBTRACT (VARIABLE_REFERENCE 'mid') 1)))
            )
          )
        )
      )
    )
    
    (RETURN_VALUE -1)  ; Not found
  )
)
```

### Loop Invariants

Specify conditions that must hold during loop execution:

```aether
(WHILE_LOOP
  (CONDITION (EXPRESSION_LESS (VARIABLE_REFERENCE 'i') (VARIABLE_REFERENCE 'n')))
  (INVARIANT 
    (PREDICATE_AND
      (PREDICATE_GREATER_EQUAL (VARIABLE_REFERENCE 'i') 0)
      (PREDICATE_LESS_EQUAL (VARIABLE_REFERENCE 'i') (VARIABLE_REFERENCE 'n'))
      (PREDICATE_EQUALS (VARIABLE_REFERENCE 'sum') (MULTIPLY (VARIABLE_REFERENCE 'i') (VARIABLE_REFERENCE 'step')))
    )
  )
  (BODY
    (ASSIGN_VARIABLE 'sum' (EXPRESSION_ADD (VARIABLE_REFERENCE 'sum') (VARIABLE_REFERENCE 'step')))
    (ASSIGN_VARIABLE 'i' (EXPRESSION_ADD (VARIABLE_REFERENCE 'i') 1))
  )
)
```

## Metaprogramming

### Code Generation with Macros

AetherScript's S-expression syntax makes metaprogramming natural:

```aether
(DEFINE_MACRO
  (NAME 'define_getter_setter')
  (PARAMETERS (struct_name field_name field_type))
  (EXPANSION
    (BLOCK
      (DEFINE_FUNCTION
        (NAME (CONCAT 'get_ (SYMBOL_NAME field_name)))
        (ACCEPTS_PARAMETER (NAME 'self') (TYPE &(SYMBOL_NAME struct_name)))
        (RETURNS (SYMBOL_TYPE field_type))
        (BODY
          (RETURN_VALUE (FIELD_ACCESS (VARIABLE_REFERENCE 'self') (SYMBOL_NAME field_name)))
        )
      )
      
      (DEFINE_FUNCTION
        (NAME (CONCAT 'set_ (SYMBOL_NAME field_name)))
        (ACCEPTS_PARAMETER (NAME 'self') (TYPE &mut (SYMBOL_NAME struct_name)))
        (ACCEPTS_PARAMETER (NAME 'value') (TYPE (SYMBOL_TYPE field_type)))
        (RETURNS VOID)
        (BODY
          (ASSIGN_FIELD (VARIABLE_REFERENCE 'self') (SYMBOL_NAME field_name) (VARIABLE_REFERENCE 'value))
        )
      )
    )
  )
)

; Usage:
(DEFINE_STRUCT
  (NAME 'Person')
  (FIELDS
    (FIELD 'name' STRING)
    (FIELD 'age' INTEGER)
  )
)

(MACRO_CALL 'define_getter_setter' 'Person' 'name' STRING)
(MACRO_CALL 'define_getter_setter' 'Person' 'age' INTEGER)
```

### Compile-Time Computation

Perform computations at compile time:

```aether
(DEFINE_CONST_FUNCTION
  (NAME 'fibonacci_compile_time')
  (ACCEPTS_PARAMETER (NAME 'n') (TYPE INTEGER))
  (RETURNS INTEGER)
  (BODY
    (IF_STATEMENT
      (CONDITION (EXPRESSION_LESS_EQUAL (VARIABLE_REFERENCE 'n') 1))
      (THEN_EXECUTE (RETURN_VALUE (VARIABLE_REFERENCE 'n')))
      (ELSE_EXECUTE
        (RETURN_VALUE
          (EXPRESSION_ADD
            (CALL_FUNCTION 'fibonacci_compile_time' (ARGUMENTS (EXPRESSION_SUBTRACT (VARIABLE_REFERENCE 'n') 1)))
            (CALL_FUNCTION 'fibonacci_compile_time' (ARGUMENTS (EXPRESSION_SUBTRACT (VARIABLE_REFERENCE 'n') 2)))
          )
        )
      )
    )
  )
)

; This is computed at compile time
(DECLARE_CONSTANT
  (NAME 'FIB_10')
  (TYPE INTEGER)
  (VALUE (CALL_FUNCTION 'fibonacci_compile_time' (ARGUMENTS 10)))
)
```

## Performance Optimization

### Zero-Cost Abstractions

Write high-level code that compiles to efficient machine code:

```aether
(DEFINE_GENERIC_FUNCTION
  (NAME 'for_each')
  (TYPE_PARAMETERS (T F))
  (ACCEPTS_PARAMETER (NAME 'array') (TYPE &(ARRAY T)))
  (ACCEPTS_PARAMETER (NAME 'func') (TYPE F))
  (RETURNS VOID)
  (INTENT "Apply function to each element - zero runtime cost")
  (INLINE_HINT ALWAYS)  ; Hint to always inline this function
  (BODY
    (FOR_LOOP
      (INIT (DECLARE_VARIABLE (NAME 'i') (TYPE INTEGER) (INITIAL_VALUE 0)))
      (CONDITION (EXPRESSION_LESS (VARIABLE_REFERENCE 'i') (CALL_FUNCTION 'array_length' (ARGUMENTS (VARIABLE_REFERENCE 'array')))))
      (UPDATE (ASSIGN_VARIABLE 'i' (EXPRESSION_ADD (VARIABLE_REFERENCE 'i') 1)))
      (BODY
        (CALL_FUNCTION func 
          (ARGUMENTS (CALL_FUNCTION 'array_get' (ARGUMENTS (VARIABLE_REFERENCE 'array') (VARIABLE_REFERENCE 'i'))))
        )
      )
    )
  )
)
```

### Memory Layout Control

Control how data is laid out in memory:

```aether
(DEFINE_STRUCT
  (NAME 'OptimizedStruct')
  (MEMORY_LAYOUT PACKED)  ; Pack fields tightly
  (ALIGNMENT 16)          ; Align to 16-byte boundaries
  (FIELDS
    (FIELD 'small_field' INTEGER32)
    (FIELD 'large_field' INTEGER64)
    (FIELD 'flag' BOOLEAN)
  )
)
```

## Interoperability

### Calling C Functions

Interface with existing C libraries:

```aether
(DECLARE_EXTERNAL_FUNCTION
  (NAME 'strlen')
  (ACCEPTS_PARAMETER (NAME 'str') (TYPE (POINTER CHARACTER)))
  (RETURNS SIZET)
  (CALLING_CONVENTION C)
  (LIBRARY "libc")
)

(DEFINE_FUNCTION
  (NAME 'c_string_length')
  (ACCEPTS_PARAMETER (NAME 'aether_string') (TYPE &STRING))
  (RETURNS INTEGER)
  (BODY
    ; Convert AetherScript string to C string
    (DECLARE_VARIABLE 
      (NAME 'c_str') 
      (TYPE (POINTER CHARACTER)) 
      (INITIAL_VALUE (CALL_FUNCTION 'string_to_c_string' (ARGUMENTS (VARIABLE_REFERENCE 'aether_string'))))
    )
    
    ; Call C function
    (DECLARE_VARIABLE 
      (NAME 'length') 
      (TYPE SIZET) 
      (INITIAL_VALUE (CALL_FUNCTION 'strlen' (ARGUMENTS (VARIABLE_REFERENCE 'c_str'))))
    )
    
    ; Clean up and return
    (CALL_FUNCTION 'free_c_string' (ARGUMENTS (VARIABLE_REFERENCE 'c_str')))
    (RETURN_VALUE (TYPE_CAST (VARIABLE_REFERENCE 'length') INTEGER))
  )
)
```

### Creating C-Compatible APIs

Export AetherScript functions for use in C:

```aether
(DEFINE_FUNCTION
  (NAME 'add_numbers')
  (ACCEPTS_PARAMETER (NAME 'a') (TYPE INTEGER32))
  (ACCEPTS_PARAMETER (NAME 'b') (TYPE INTEGER32))
  (RETURNS INTEGER32)
  (EXPORT_SYMBOL "aether_add")  ; C-visible name
  (CALLING_CONVENTION C)
  (BODY
    (RETURN_VALUE (EXPRESSION_ADD (VARIABLE_REFERENCE 'a') (VARIABLE_REFERENCE 'b')))
  )
)
```

## Concurrency Patterns

### Safe Shared State

Use ownership to ensure thread safety:

```aether
(DEFINE_STRUCT
  (NAME 'SafeCounter')
  (FIELDS
    (FIELD 'value' (ATOMIC INTEGER))
    (FIELD 'mutex' MUTEX)
  )
)

(DEFINE_FUNCTION
  (NAME 'increment_counter')
  (ACCEPTS_PARAMETER (NAME 'counter') (TYPE ~SafeCounter))
  (RETURNS INTEGER)
  (BODY
    ; Lock mutex for exclusive access
    (WITH_LOCK (FIELD_ACCESS (VARIABLE_REFERENCE 'counter') 'mutex')
      (BODY
        (DECLARE_VARIABLE 
          (NAME 'old_value') 
          (TYPE INTEGER) 
          (INITIAL_VALUE (ATOMIC_LOAD (FIELD_ACCESS (VARIABLE_REFERENCE 'counter') 'value')))
        )
        
        (ATOMIC_STORE 
          (FIELD_ACCESS (VARIABLE_REFERENCE 'counter') 'value')
          (EXPRESSION_ADD (VARIABLE_REFERENCE 'old_value') 1)
        )
        
        (RETURN_VALUE (EXPRESSION_ADD (VARIABLE_REFERENCE 'old_value') 1))
      )
    )
  )
)
```

### Message Passing

Use channels for safe communication between threads:

```aether
(DEFINE_FUNCTION
  (NAME 'producer_consumer_example')
  (RETURNS INTEGER)
  (BODY
    ; Create a channel for communication
    (DECLARE_VARIABLE 
      (NAME 'channel') 
      (TYPE (CHANNEL INTEGER)) 
      (INITIAL_VALUE (CALL_FUNCTION 'create_channel' (ARGUMENTS 10)))  ; Buffer size 10
    )
    
    ; Spawn producer thread
    (SPAWN_THREAD
      (CLOSURE
        (CAPTURE (VARIABLE_REFERENCE 'channel'))
        (BODY
          (FOR_LOOP
            (INIT (DECLARE_VARIABLE (NAME 'i') (TYPE INTEGER) (INITIAL_VALUE 0)))
            (CONDITION (EXPRESSION_LESS (VARIABLE_REFERENCE 'i') 5))
            (UPDATE (ASSIGN_VARIABLE 'i' (EXPRESSION_ADD (VARIABLE_REFERENCE 'i') 1)))
            (BODY
              (CALL_FUNCTION 'channel_send' 
                (ARGUMENTS (VARIABLE_REFERENCE 'channel') (VARIABLE_REFERENCE 'i'))
              )
            )
          )
          (CALL_FUNCTION 'channel_close' (ARGUMENTS (VARIABLE_REFERENCE 'channel')))
        )
      )
    )
    
    ; Consumer loop
    (WHILE_LOOP
      (CONDITION TRUE)
      (BODY
        (MATCH_EXPRESSION
          (VALUE (CALL_FUNCTION 'channel_receive' (ARGUMENTS (VARIABLE_REFERENCE 'channel'))))
          (CASE 'Some' 
            (BINDING 'value')
            (EXPRESSION_STATEMENT 
              (CALL_FUNCTION 'puts' 
                (ARGUMENTS (CALL_FUNCTION 'int_to_string' (ARGUMENTS (VARIABLE_REFERENCE 'value'))))
              )
            )
          )
          (CASE 'None' (BREAK))  ; Channel closed
        )
      )
    )
    
    (RETURN_VALUE 0)
  )
)
```

These advanced topics demonstrate AetherScript's power for systems programming while maintaining safety and performance. The ownership system, contracts, and metaprogramming capabilities enable building robust, efficient applications.