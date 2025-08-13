# FFI Callback Support Summary

## Current Status

AetherScript can declare and use external functions that accept function pointers as parameters. However, actually passing AetherScript functions as callbacks to C code is not yet fully implemented.

## What Works

1. **Declaring External Functions with Function Pointer Parameters**
   - External functions can accept `(POINTER_TO VOID)` parameters that represent function pointers
   - Example: `qsort` from libc accepts a comparison function pointer

2. **Type System Support**
   - The AST includes a `Function` type specifier with parameter and return types
   - Function pointers can be represented as generic void pointers in FFI declarations

## What's Missing

1. **Function Reference Syntax**
   - No syntax for taking the address of a function (`&function_name`)
   - Function names cannot be passed directly as arguments

2. **Calling Convention Conversion**
   - No automatic trampolines for converting between AetherScript and C calling conventions
   - Would need to handle differences in parameter passing, stack management, etc.

3. **Type Safety**
   - Function pointer types are represented as `(POINTER_TO VOID)` losing type information
   - No compile-time checking of callback signatures

## Use Cases That Would Benefit

- Custom sorting with `qsort`
- Event handling in GUI libraries (GTK, Qt bindings)
- Async I/O completion handlers
- Signal handlers
- Iterator/visitor patterns
- Plugin systems

## Implementation Requirements

To fully support callbacks, AetherScript would need:

1. Syntax for function references/addresses
2. Function type checking in FFI calls
3. Calling convention adapters/trampolines
4. Runtime support for callback registration
5. Proper lifetime management for callbacks passed to C

## Test Files Created

- `test_ffi_callbacks.c` - C library with various callback-accepting functions
- `test_ffi_callbacks.aether` - Attempted full callback test (compilation fails)
- `test_ffi_callbacks_simple.aether` - Demonstrates current capabilities
- `libtest_callbacks.dylib` - Compiled callback test library