# AetherScript Standard Library

The AetherScript Standard Library provides essential functionality for common programming tasks, with a focus on safety, verification, and LLM-friendly design.

## Overview

The standard library is organized into the following modules:

### 1. **std.io** - Input/Output Operations
Safe file and console I/O with resource management guarantees.

**Key Functions:**
- `read_file_safe(path: string, max_size: int) -> string` - Read file with size limits
- `write_file_safe(path: string, content: string, append: bool) -> bool` - Write file safely
- `print(message: string) -> void` - Print without newline
- `println(message: string) -> void` - Print with newline
- `read_line() -> string` - Read line from stdin

**Features:**
- Automatic resource cleanup
- Size limits to prevent memory exhaustion
- Verified preconditions and postconditions

### 2. **std.collections** - Collection Utilities
Verified operations on arrays and collections.

**Key Functions:**
- `sort_verified(array: int[], size: int) -> void` - Sort with correctness proof
- `binary_search(array: int[], size: int, target: int) -> int` - Binary search
- `filter(array: int[], size: int, predicate: fn) -> int[]` - Filter elements
- `map(array: int[], size: int, transform: fn) -> int[]` - Transform elements
- `reduce(array: int[], size: int, initial: int, reducer: fn) -> int` - Reduce to value

**Features:**
- SMT solver verification for sorting
- Functional programming patterns
- Safe bounds checking

### 3. **std.math** - Mathematical Functions
Safe arithmetic and mathematical operations.

**Key Functions:**
- `safe_add(a: int, b: int) -> (int, bool)` - Addition with overflow check
- `safe_multiply(a: int, b: int) -> (int, bool)` - Multiplication with overflow check
- `pow_verified(base: int, exp: int) -> int` - Power with verification
- `sqrt_int(n: int) -> int` - Integer square root
- `gcd(a: int, b: int) -> int` - Greatest common divisor
- `min/max(a: int, b: int) -> int` - Minimum/maximum
- `sum_array(array: int[], size: int) -> int` - Array sum

**Features:**
- Overflow protection
- Contract verification
- Euclidean algorithms

### 4. **std.time** - Date/Time Operations
Comprehensive time manipulation and formatting.

**Key Functions:**
- `now() -> timestamp` - Current Unix timestamp
- `now_utc() -> datetime` - Current UTC datetime
- `format_iso8601(dt: datetime) -> string` - ISO 8601 formatting
- `add_duration(ts: timestamp, dur: duration) -> timestamp` - Time arithmetic
- `timestamp_to_datetime(ts: timestamp, tz_offset: int) -> datetime` - Conversion
- `duration_from_seconds(seconds: int) -> duration` - Create duration

**Types:**
- `timestamp` - Unix timestamp with nanosecond precision
- `datetime` - Structured date/time with components
- `duration` - Time duration

### 5. **std.net** - Networking Primitives
TCP/IP and HTTP networking with timeout support.

**Key Functions:**
- `tcp_connect(host: string, port: int, timeout_ms: int) -> socket` - TCP connect
- `tcp_send(socket: socket, data: string) -> int` - Send data
- `tcp_receive(socket: socket, max_size: int, timeout_ms: int) -> string` - Receive
- `http_get(url: string, options: map) -> http_response` - HTTP GET
- `http_post(url: string, body: string, content_type: string, options: map) -> http_response` - HTTP POST
- `resolve_hostname(hostname: string) -> ip_address` - DNS resolution

**Features:**
- Timeout support
- Error handling
- Resource cleanup

### 6. **std.concurrency** - Concurrency Primitives
Thread-safe concurrent programming support.

**Key Functions:**
- `thread_create(func: fn, stack_size: int) -> thread` - Create thread
- `thread_join(thread: thread, timeout_ms: int) -> bool` - Wait for thread
- `channel_create(capacity: int) -> channel` - Create channel
- `channel_send(ch: channel, value: int, timeout_ms: int) -> bool` - Send on channel
- `channel_receive(ch: channel, timeout_ms: int, out: *int) -> bool` - Receive
- `mutex_create() -> mutex` - Create mutex
- `mutex_lock(m: mutex, timeout_ms: int) -> bool` - Lock mutex
- `atomic_create(initial: int) -> atomic_int` - Create atomic integer
- `atomic_add(a: atomic_int, delta: int) -> int` - Atomic add

**Features:**
- Deadlock prevention
- Timeout support
- Memory safety

## Design Principles

### 1. **Safety First**
- All operations have explicit error handling
- Resources are automatically cleaned up
- Size limits prevent resource exhaustion

### 2. **Verification**
- Critical operations include formal contracts
- SMT solver integration for correctness proofs
- Preconditions and postconditions

### 3. **LLM-Friendly**
- Clear function names and semantics
- Consistent patterns across modules
- Explicit error propagation

### 4. **Resource Management**
- RAII (Resource Acquisition Is Initialization)
- Automatic cleanup with RESOURCE_SCOPE
- Exception-safe resource handling

## Usage Examples

### Basic I/O
```aetherscript
(TRY_EXECUTE
  (PROTECTED_BLOCK
    (DECLARE_VARIABLE (NAME "content") (TYPE STRING)
      (INITIAL_VALUE (CALL_FUNCTION "std.io.read_file_safe"
        (ARGUMENTS 
          (STRING_LITERAL "data.txt")
          (INTEGER_LITERAL 1048576))))) ; 1MB limit
    (CALL_FUNCTION "std.io.println"
      (ARGUMENTS (VARIABLE_REFERENCE "content"))))
  (CATCH_EXCEPTION
    (EXCEPTION_TYPE "io_error")
    (BINDING_VARIABLE (NAME "e") (TYPE "io_error"))
    (HANDLER_BLOCK
      (CALL_FUNCTION "std.io.println"
        (ARGUMENTS (STRING_LITERAL "Failed to read file"))))))
```

### Verified Sorting
```aetherscript
(DECLARE_VARIABLE (NAME "numbers") (TYPE (ARRAY INT 10)))
; ... initialize array ...
(CALL_FUNCTION "std.collections.sort_verified"
  (ARGUMENTS
    (VARIABLE_REFERENCE "numbers")
    (INTEGER_LITERAL 10)))
; Array is now sorted and verified by SMT solver
```

### Concurrent Processing
```aetherscript
(DECLARE_VARIABLE (NAME "channel") (TYPE "channel")
  (INITIAL_VALUE (CALL_FUNCTION "std.concurrency.channel_create"
    (ARGUMENTS (INTEGER_LITERAL 10)))))

(DECLARE_VARIABLE (NAME "worker") (TYPE "thread")
  (INITIAL_VALUE (CALL_FUNCTION "std.concurrency.thread_create"
    (ARGUMENTS 
      (LAMBDA () 
        ; Worker thread code
        (CALL_FUNCTION "std.concurrency.channel_send"
          (ARGUMENTS 
            (VARIABLE_REFERENCE "channel")
            (INTEGER_LITERAL 42)
            (INTEGER_LITERAL -1))))
      (INTEGER_LITERAL 65536)))))
```

## Runtime Support

The standard library is backed by a comprehensive runtime written in Rust, providing:

- Memory safety
- Thread safety
- OS integration
- Performance optimization

Runtime modules:
- `io.rs` - File and console I/O
- `collections.rs` - Collection algorithms
- `math.rs` - Mathematical operations
- `time.rs` - Time/date handling
- `network.rs` - TCP/IP and HTTP
- `concurrency.rs` - Threading and synchronization
- `memory.rs` - Memory management

## Testing

Run the test suite:
```bash
./test_stdlib_examples.sh
```

Individual example programs:
```bash
./run_example.sh stdlib_demo
./run_example.sh stdlib_file_processing
./run_example.sh stdlib_concurrent_tasks
# etc.
```

## Future Enhancements

Planned additions:
- Cryptographic primitives
- JSON/XML parsing
- Regular expressions
- Database connectivity
- Advanced data structures
- Compression algorithms
- Image processing

## Contributing

When adding new standard library modules:

1. Design with safety and verification in mind
2. Provide comprehensive contracts
3. Implement runtime support in Rust
4. Create example programs
5. Add integration tests
6. Document thoroughly

The standard library is a critical component of AetherScript, designed to make LLM-assisted programming safer and more reliable.