# AetherScript V2 Examples Reorganization Plan

## Overview

This document outlines the reorganization of 78+ V1 examples into a comprehensive folder structure that illustrates every V2 language feature. All examples will be migrated from V1 (LISP-like S-expression) syntax to V2 (Swift/Rust-like) syntax.

## New Folder Structure

```
examples/
├── 01-basics/
│   ├── hello_world/          # Basic program structure
│   ├── simple_main/          # Minimal program
│   └── module_system/        # Module declarations
│
├── 02-variables/
│   ├── let_bindings/         # Immutable variables
│   ├── mutability/           # Mutable variables (var)
│   └── constants/            # Compile-time constants
│
├── 03-types/
│   ├── primitives/           # Int, Float, Bool, String
│   ├── type_aliases/         # Type aliasing
│   └── type_conversions/     # Type casting
│
├── 04-functions/
│   ├── basic_functions/      # Function declaration
│   ├── parameters/           # Parameter passing
│   ├── return_values/        # Return types
│   ├── closures/             # Lambda expressions
│   └── function_metadata/    # Function attributes
│
├── 05-operators/
│   ├── arithmetic/           # +, -, *, /, %
│   ├── comparison/           # ==, !=, <, >, <=, >=
│   └── logical/              # &&, ||, !
│
├── 06-control-flow/
│   ├── if_else/              # Conditionals
│   ├── loops/                # While, for loops
│   ├── foreach/              # Iterator loops
│   └── match/                # Pattern matching basics
│
├── 07-structs/
│   ├── basic_struct/         # Struct definition
│   ├── nested_structs/       # Struct composition
│   ├── struct_methods/       # Associated functions
│   └── struct_advanced/      # Complex struct patterns
│
├── 08-enums/
│   ├── basic_enum/           # Simple enums
│   ├── enum_with_data/       # Enums with associated values
│   └── enum_methods/         # Enum implementations
│
├── 09-pattern-matching/
│   ├── match_basics/         # Basic match expressions
│   ├── match_enums/          # Matching on enums
│   ├── match_structs/        # Struct destructuring
│   ├── match_guards/         # Match with guards
│   └── match_exhaustive/     # Exhaustiveness checking
│
├── 10-collections/
│   ├── arrays/               # Fixed-size arrays
│   ├── dynamic_arrays/       # Growable arrays
│   ├── maps/                 # Hash maps
│   └── iteration/            # Collection iteration
│
├── 11-memory/
│   ├── ownership/            # Ownership semantics
│   ├── pointers/             # Raw pointers
│   └── resource_management/  # RAII patterns
│
├── 12-error-handling/
│   ├── result_type/          # Result<T, E> usage
│   ├── error_propagation/    # Error chaining
│   └── panic/                # Unrecoverable errors
│
├── 13-strings/
│   ├── string_basics/        # String literals
│   ├── string_operations/    # Manipulation
│   └── string_formatting/    # Interpolation
│
├── 14-ffi/
│   ├── c_functions/          # Calling C functions
│   ├── c_types/              # C type interop
│   └── extern_blocks/        # External declarations
│
├── 15-stdlib/
│   ├── io/                   # Input/Output
│   ├── math/                 # Math functions
│   ├── string_utils/         # String utilities
│   └── collections/          # Collection helpers
│
└── 16-networking/
    ├── tcp_client/           # TCP connections
    ├── tcp_server/           # TCP listeners
    ├── http_client/          # HTTP requests
    └── http_server/          # HTTP servers
```

## Migration Mapping

### 01-basics/
| V1 Example | V2 Destination | Notes |
|------------|----------------|-------|
| hello_world | 01-basics/hello_world | Add print functionality demo |
| simple | 01-basics/simple_main | Minimal program |
| simple_main | 01-basics/simple_main | Merge with simple |
| module_system | 01-basics/module_system | Module imports |

### 02-variables/
| V1 Example | V2 Destination | Notes |
|------------|----------------|-------|
| mutability_demo | 02-variables/mutability | Show let vs var |
| constants_type_aliases | 02-variables/constants | Extract constants |

### 03-types/
| V1 Example | V2 Destination | Notes |
|------------|----------------|-------|
| type_conversions | 03-types/type_conversions | Type casting examples |
| constants_type_aliases | 03-types/type_aliases | Extract type aliases |

### 04-functions/
| V1 Example | V2 Destination | Notes |
|------------|----------------|-------|
| factorial | 04-functions/basic_functions | Recursion example |
| math_utils | 04-functions/basic_functions | Multiple functions |
| function_metadata_demo | 04-functions/function_metadata | Attributes |

### 05-operators/
| V1 Example | V2 Destination | Notes |
|------------|----------------|-------|
| arithmetic_operations | 05-operators/arithmetic | +, -, *, /, % |
| comparison_logic | 05-operators/comparison | Comparison ops |
| comparison_logic | 05-operators/logical | Logical ops |

### 06-control-flow/
| V1 Example | V2 Destination | Notes |
|------------|----------------|-------|
| control_flow | 06-control-flow/if_else | If/else |
| control_flow | 06-control-flow/loops | Loop constructs |
| foreach | 06-control-flow/foreach | For-each loops |

### 07-structs/
| V1 Example | V2 Destination | Notes |
|------------|----------------|-------|
| struct_test | 07-structs/basic_struct | Basic structs |
| struct_simple_nested | 07-structs/nested_structs | Composition |
| struct_advanced | 07-structs/struct_advanced | Complex patterns |
| struct_debug | 07-structs/struct_methods | Debug trait |

### 08-enums/
| V1 Example | V2 Destination | Notes |
|------------|----------------|-------|
| enum_test_basic | 08-enums/basic_enum | Simple enums |
| enum_test | 08-enums/enum_with_data | Associated data |

### 09-pattern-matching/
| V1 Example | V2 Destination | Notes |
|------------|----------------|-------|
| pattern_match_basic | 09-pattern-matching/match_basics | Basic match |
| pattern_match_enum | 09-pattern-matching/match_enums | Enum matching |
| pattern_match_struct | 09-pattern-matching/match_structs | Struct patterns |
| pattern_match_guard | 09-pattern-matching/match_guards | Guards |
| pattern_match_exhaustive | 09-pattern-matching/match_exhaustive | Exhaustiveness |
| pattern_match_nested | 09-pattern-matching/match_nested | Nested patterns |
| pattern_match_tuple | 09-pattern-matching/match_tuple | Tuple patterns |
| pattern_match_range | 09-pattern-matching/match_range | Range patterns |

### 10-collections/
| V1 Example | V2 Destination | Notes |
|------------|----------------|-------|
| arrays | 10-collections/arrays | Array operations |
| arrays_simple | 10-collections/arrays | Merge |
| maps | 10-collections/maps | Hash maps |
| maps_simple | 10-collections/maps | Merge |

### 11-memory/
| V1 Example | V2 Destination | Notes |
|------------|----------------|-------|
| ownership_demo | 11-memory/ownership | Ownership |
| pointers | 11-memory/pointers | Raw pointers |
| pointers_simple | 11-memory/pointers | Merge |
| resource_demo | 11-memory/resource_management | RAII |

### 12-error-handling/
| V1 Example | V2 Destination | Notes |
|------------|----------------|-------|
| error_handling | 12-error-handling/result_type | Result usage |
| error_simple | 12-error-handling/error_propagation | Error propagation |
| error_demo | 12-error-handling/panic | Panic handling |

### 13-strings/
| V1 Example | V2 Destination | Notes |
|------------|----------------|-------|
| string_operations | 13-strings/string_operations | String manipulation |
| string_helpers | 13-strings/string_utils | Helper functions |
| text_analyzer | 13-strings/text_analyzer | Analysis example |

### 14-ffi/
| V1 Example | V2 Destination | Notes |
|------------|----------------|-------|
| ffi_demo | 14-ffi/c_functions | Basic FFI |
| ffi_advanced | 14-ffi/c_types | Advanced types |
| ffi_strlen_test | 14-ffi/c_functions | String interop |
| test_printf | 14-ffi/c_functions | printf example |

### 15-stdlib/
| V1 Example | V2 Destination | Notes |
|------------|----------------|-------|
| stdlib_demo | 15-stdlib/io | IO operations |
| stdlib_printf | 15-stdlib/io | Printf |
| stdlib_math | 15-stdlib/math | Math functions |
| stdlib_string | 15-stdlib/string_utils | String utils |
| stdlib_time | 15-stdlib/time | Time operations |
| stdlib_full | 15-stdlib/complete | Full stdlib demo |

### 16-networking/
| V1 Example | V2 Destination | Notes |
|------------|----------------|-------|
| tcp_test_simple | 16-networking/tcp_client | TCP basics |
| http_demo | 16-networking/http_client | HTTP client |
| http_server | 16-networking/http_server | Basic server |
| http_server_stay | 16-networking/http_server | Persistent server |
| simple_http_server | 16-networking/http_server | Merge |
| debug_http_server | 16-networking/http_server | Debug mode |
| http_blog_server | 16-networking/http_server | Blog example |
| blog_http_simple | 16-networking/http_server | Merge |
| blog_server_loop | 16-networking/http_server | Merge |
| blog_server_running | 16-networking/http_server | Merge |
| blog_listen | 16-networking/http_server | Merge |
| simple_llm_blog | 16-networking/http_server | LLM integration |
| llm_blog_server | 16-networking/http_server | LLM server |

## V1 to V2 Syntax Reference

### Module Declaration
```
# V1 (S-expression)
(DEFINE_MODULE
  (NAME 'hello_world')
  (CONTENT ...))

# V2 (Modern)
module hello_world;
```

### Function Declaration
```
# V1
(DEFINE_FUNCTION
  (NAME 'add')
  (PARAMS (PARAM 'a' INTEGER) (PARAM 'b' INTEGER))
  (RETURNS INTEGER)
  (BODY (RETURN_VALUE (ADD (REFERENCE 'a') (REFERENCE 'b')))))

# V2
func add(a: Int, b: Int) -> Int {
    return {a + b};
}
```

### Variable Binding
```
# V1
(BIND 'x' INTEGER (LITERAL_INT 10))

# V2
let x: Int = 10;
```

### Struct Definition
```
# V1
(DEFINE_STRUCT
  (NAME 'Point')
  (FIELDS
    (FIELD 'x' INTEGER)
    (FIELD 'y' INTEGER)))

# V2
struct Point {
    x: Int,
    y: Int,
}
```

### Enum Definition
```
# V1
(DEFINE_ENUM
  (NAME 'Color')
  (VARIANTS
    (VARIANT 'Red')
    (VARIANT 'Green')
    (VARIANT 'Blue')))

# V2
enum Color {
    Red,
    Green,
    Blue,
}
```

### Pattern Matching
```
# V1
(MATCH (REFERENCE 'x')
  (ARM (PATTERN (LITERAL_INT 1)) (BODY ...))
  (ARM (PATTERN (WILDCARD)) (BODY ...)))

# V2
match x {
    1 => { ... }
    _ => { ... }
}
```

## Implementation Plan

1. **Phase 1: Core Infrastructure** (Current Task)
   - Remove V1 syntax support from compiler
   - Remove syntax version detection
   - Update CLI to remove --syntax flag

2. **Phase 2: Folder Structure**
   - Create new directory hierarchy
   - Set up README.md for each category

3. **Phase 3: Example Migration**
   - Migrate basics and fundamentals first
   - Progress through increasingly complex features
   - Test each migrated example compiles and runs

4. **Phase 4: Documentation**
   - Add comments explaining V2 syntax in each example
   - Create master index of all examples
   - Update main README with new structure
