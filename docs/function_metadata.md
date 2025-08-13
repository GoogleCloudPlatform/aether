# Function Metadata in AetherScript

Function metadata provides a way to specify contracts, performance expectations, and other properties of functions in AetherScript. This enables better documentation, optimization opportunities, and runtime verification.

## Overview

Function metadata can include:
- **Preconditions** - Conditions that must be true before function execution
- **Postconditions** - Conditions that must be true after function execution
- **Invariants** - Conditions that must remain true throughout function execution
- **Performance Expectations** - Expected performance characteristics
- **Complexity Expectations** - Time and space complexity
- **Algorithm Hints** - Hints about the algorithm used
- **Thread Safety** - Whether the function is thread-safe
- **Blocking Behavior** - Whether the function may block

## Syntax

### Preconditions

```aether
(function safe_divide (integer a) (integer b) integer
    (precondition (!= b 0) ASSERT_FAIL "Division by zero")
    (/ a b))
```

### Postconditions

```aether
(function abs (integer x) integer
    (postcondition (>= (return_value) 0) ASSERT_FAIL "Result must be non-negative")
    (if (< x 0) (- x) x))
```

### Performance Expectations

```aether
(function fast_lookup (string key) value
    (performance_expectation LATENCY_MS 0.01 "99th percentile")
    ;; implementation
    )
```

Available metrics:
- `LATENCY_MS` - Expected latency in milliseconds
- `THROUGHPUT_OPS` - Expected throughput in operations per second
- `MEMORY_BYTES` - Expected memory usage in bytes

### Complexity Expectations

```aether
(function merge_sort (array arr) array
    (complexity_expectation TIME BIG_O "n log n")
    (complexity_expectation SPACE BIG_O "n")
    ;; implementation
    )
```

Complexity types:
- `TIME` - Time complexity
- `SPACE` - Space complexity

Notations:
- `BIG_O` - Upper bound (O notation)
- `BIG_THETA` - Tight bound (Θ notation)
- `BIG_OMEGA` - Lower bound (Ω notation)

### Algorithm Hints

```aether
(function binary_search (array arr) (value target) integer
    (algorithm_hint "binary search")
    ;; implementation
    )
```

### Thread Safety and Blocking

```aether
(function atomic_increment (ref integer counter) void
    (thread_safe true)
    (may_block false)
    ;; implementation
    )
```

## Failure Actions

Contract assertions can specify what happens when they fail:

- `ASSERT_FAIL` - Terminate the program with an assertion failure
- `LOG_WARNING` - Log a warning and continue execution
- `THROW_EXCEPTION` - Throw an exception

## Complete Example

```aether
(function binary_search (array integer sorted_arr) (integer target) integer
    ;; Preconditions
    (precondition (> (array_length sorted_arr) 0) ASSERT_FAIL "Array must not be empty")
    (precondition (is_sorted sorted_arr) ASSERT_FAIL "Array must be sorted")
    
    ;; Postconditions
    (postcondition 
        (or (= (return_value) -1)
            (= (array_access sorted_arr (return_value)) target))
        ASSERT_FAIL "Must return valid index or -1")
    
    ;; Performance characteristics
    (algorithm_hint "binary search")
    (complexity_expectation TIME BIG_O "log n")
    (performance_expectation LATENCY_MS 0.1 "worst case")
    
    ;; Thread safety
    (thread_safe true)
    (may_block false)
    
    ;; Implementation
    (let ((low 0)
          (high (- (array_length sorted_arr) 1)))
        ;; ... binary search implementation ...
        ))
```

## Benefits

1. **Documentation** - Metadata serves as executable documentation
2. **Verification** - Contracts can be verified at runtime or compile-time
3. **Optimization** - Performance hints enable better optimization decisions
4. **Safety** - Thread safety annotations prevent data races
5. **Analysis** - Complexity information aids in algorithm selection

## Implementation Status

The parser now fully supports all metadata fields. The semantic analyzer validates and stores this metadata. Future work includes:
- Runtime contract enforcement
- Static contract verification
- Performance monitoring based on expectations
- Optimization based on algorithm hints