# Async I/O Example

This example demonstrates the `concurrent` block syntax in AetherScript V2.

## Current Status

- **Syntax**: The `concurrent` keyword is supported and parsed correctly.
- **Type System**: Function calls inside `concurrent` blocks correctly return `Future<T>` instead of `T`.
- **Execution**: Currently, the LLVM backend lowers `concurrent` blocks to synchronous execution with a warning.
  - **Limitation**: The current implementation terminates the function after the concurrent block executes (due to a `Return` terminator in the lowered block), so code after the `concurrent` block may not execute.
- **Stdlib**: `std.io` is currently in V1 syntax, so we use `extern` declarations for printing in this example.

## Usage

```bash
make run
```

Expected output (synchronous execution):
```
Starting async I/O example
Starting task 1
Finished task 1
Starting task 2
Finished task 2
```
(Note: "Concurrent block finished" is not printed due to the limitation mentioned above).
