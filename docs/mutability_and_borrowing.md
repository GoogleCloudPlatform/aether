# Mutability and Borrowing in AetherScript

This document describes the mutability and borrowing system implemented in AetherScript, which provides memory safety guarantees similar to Rust.

## Overview

AetherScript enforces strict rules about variable mutability and borrowing to prevent common memory safety issues:
- Use-after-move errors
- Data races
- Concurrent modification issues

## Mutability

By default, all variables in AetherScript are immutable:

```aether
(let ((x 42))
    ;; x is immutable
    ;; (set x 100)  ;; Error: cannot assign to immutable variable
    )
```

To create a mutable variable, use the `mut` keyword:

```aether
(let ((mut y 42))
    (set y 100)  ;; OK: y is mutable
    )
```

## Ownership Annotations

AetherScript supports four types of ownership annotations:

1. **Owned (`^T`)** - Exclusive ownership of a value
2. **Borrowed (`&T`)** - Immutable reference to a value
3. **Mutable Borrowed (`&mut T`)** - Mutable reference to a value
4. **Shared (`~T`)** - Shared ownership (reference counted)

Example:
```aether
(function take_ownership (^integer value) void
    ;; This function takes ownership of value
    )

(function borrow_value (&integer value) integer
    ;; This function borrows value immutably
    (+ value 1))

(function modify_value (&mut integer value) void
    ;; This function borrows value mutably
    (set value (* value 2)))
```

## Borrowing Rules

AetherScript enforces the following borrowing rules:

1. **Multiple immutable borrows are allowed**:
   ```aether
   (let ((x 42))
       (let ((ref1 &x)
             (ref2 &x))  ;; OK: multiple immutable borrows
           (+ ref1 ref2)))
   ```

2. **Mutable borrows are exclusive**:
   ```aether
   (let ((mut x 42))
       (let ((mut_ref &mut x))
           ;; Cannot create another borrow while mut_ref exists
           (set mut_ref 100)))
   ```

3. **Cannot mutably borrow while immutably borrowed**:
   ```aether
   (let ((mut x 42))
       (let ((ref &x))
           ;; (let ((mut_ref &mut x)) ...)  ;; Error: x is already borrowed
           ))
   ```

## Move Semantics

When a value is passed to a function that takes ownership (`^T`), the value is moved and cannot be used afterwards:

```aether
(let ((value 100))
    (consume_value value)  ;; value is moved here
    ;; (print_int value)  ;; Error: use after move
    )
```

## Implementation Details

The mutability and borrowing system is implemented through:

1. **Symbol Table Tracking**: Each symbol tracks:
   - `is_mutable`: Whether the variable can be modified
   - `is_moved`: Whether ownership has been transferred
   - `borrow_state`: Current borrowing state (none, borrowed, or mutably borrowed)

2. **Semantic Analysis**: The semantic analyzer enforces:
   - Immutability constraints during assignment
   - Move semantics when passing owned values
   - Borrowing rules when creating references

3. **Error Reporting**: Clear error messages for:
   - `UseAfterMove`: Attempting to use a moved value
   - `AssignToImmutable`: Attempting to modify an immutable variable
   - `InvalidOperation`: Violating borrowing rules

## Examples

See `examples/mutability_demo.aether` and `examples/ownership_demo.aether` for complete examples demonstrating these features.

## Future Enhancements

- Lifetime tracking for more complex borrowing patterns
- Automatic reference counting for shared ownership (`~T`)
- Integration with the type system for ownership-aware type checking
- Optimization based on ownership information