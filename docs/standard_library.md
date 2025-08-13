# AetherScript Standard Library Documentation

## Overview

The AetherScript Standard Library provides essential functionality for AetherScript programs with built-in safety guarantees and formal verification support. All standard library modules follow the language's core principles of explicitness, safety, and verifiability.

## Module Structure

The standard library is organized into the following modules:

### 1. **std.io** - I/O Operations
Provides safe file and console I/O operations with automatic resource management.

**Key Functions:**
- `read_file_safe(path: STRING, max_size: INT) -> STRING` - Read file with size limit
- `write_file_safe(path: STRING, content: STRING, append: BOOL) -> BOOL` - Write to file
- `print(text: STRING) -> VOID` - Print to stdout
- `println(text: STRING) -> VOID` - Print with newline
- `read_line() -> STRING` - Read from stdin
- `list_directory(path: STRING) -> ARRAY[STRING]` - List directory contents

**Features:**
- Automatic resource cleanup with RAII
- Size limits to prevent memory exhaustion
- Strong exception safety guarantees
- File operations wrapped in resource scopes

### 2. **std.collections** - Collection Utilities
Verified collection operations with formal correctness guarantees.

**Key Functions:**
- `sort_verified(array: ARRAY[INT], size: INT) -> VOID` - Sort with verified correctness
- `binary_search(array: ARRAY[INT], size: INT, target: INT) -> INT` - Binary search in sorted array
- `filter(input: ARRAY[INT], size: INT, predicate: FUNCTION, output: ARRAY[INT]) -> INT` - Filter elements
- `map(input: ARRAY[INT], size: INT, transform: FUNCTION, output: ARRAY[INT]) -> VOID` - Transform elements
- `reduce(array: ARRAY[INT], size: INT, initial: INT, accumulator: FUNCTION) -> INT` - Reduce to single value

**Features:**
- SMT solver verification for sorting correctness
- Complexity guarantees (O(n log n) for sort, O(log n) for binary search)
- Functional programming patterns
- Preconditions ensure array validity

### 3. **std.math** - Mathematical Functions
Mathematical operations with contracts and overflow protection.

**Key Functions:**
- `safe_add(a: INT, b: INT) -> INT` - Addition with overflow checking
- `safe_multiply(a: INT, b: INT) -> INT` - Multiplication with overflow checking
- `pow(base: FLOAT, exponent: FLOAT) -> FLOAT` - Power function
- `sqrt(x: FLOAT) -> FLOAT` - Square root with domain checking
- `sin/cos/tan(x: FLOAT) -> FLOAT` - Trigonometric functions
- `log(x: FLOAT) -> FLOAT` - Natural logarithm
- `exp(x: FLOAT) -> FLOAT` - Exponential function
- `min/max(a: INT, b: INT) -> INT` - Min/max operations
- `abs(x: INT) -> INT` - Absolute value

**Constants:**
- `PI = 3.14159265358979323846`
- `E = 2.71828182845904523536`
- `SQRT2 = 1.41421356237309504880`
- `LN2 = 0.69314718055994530942`

**Features:**
- Overflow protection with SMT verification
- Domain checking for functions like sqrt and log
- Range guarantees (e.g., sin/cos always in [-1, 1])
- Postconditions verify mathematical properties

### 4. **std.time** - Date/Time Manipulation
Time and date operations with timezone awareness.

**Types:**
- `timestamp` - Unix timestamp in seconds
- `datetime` - Structured date/time with timezone
- `duration` - Time duration with nanosecond precision

**Key Functions:**
- `now() -> timestamp` - Current Unix timestamp
- `now_utc() -> datetime` - Current UTC datetime
- `timestamp_to_datetime(ts: timestamp, tz_offset: INT) -> datetime` - Convert timestamp
- `datetime_to_timestamp(dt: datetime) -> timestamp` - Convert to timestamp
- `format_iso8601(dt: datetime) -> STRING` - Format as ISO 8601
- `parse_iso8601(str: STRING) -> datetime` - Parse ISO 8601
- `add_duration(ts: timestamp, dur: duration) -> timestamp` - Add duration
- `timestamp_compare(ts1: timestamp, ts2: timestamp) -> INT` - Compare timestamps

**Features:**
- Timezone-aware operations
- ISO 8601 formatting and parsing
- Overflow-safe duration arithmetic
- Validation of date components

### 5. **std.net** - Networking Primitives
Safe networking operations with timeouts and resource management.

**Types:**
- `ip_address` - IP address with port
- `socket` - Network socket handle
- `http_request` - HTTP request structure
- `http_response` - HTTP response structure

**Key Functions:**
- `tcp_connect(host: STRING, port: INT, timeout_ms: INT) -> socket` - Connect to TCP server
- `tcp_send(sock: socket, data: STRING) -> INT` - Send data
- `tcp_receive(sock: socket, max_size: INT, timeout_ms: INT) -> STRING` - Receive data
- `tcp_close(sock: socket) -> VOID` - Close socket
- `http_get(url: STRING, options: MAP) -> http_response` - HTTP GET request
- `http_post(url: STRING, body: STRING, content_type: STRING, options: MAP) -> http_response` - HTTP POST
- `resolve_hostname(hostname: STRING) -> ip_address` - DNS resolution

**Features:**
- Automatic resource cleanup
- Configurable timeouts
- Memory limits for responses
- Strong exception safety
- Resource contracts enforce limits

### 6. **std.concurrency** - Concurrency Primitives
Thread-safe concurrency operations with deadlock prevention.

**Types:**
- `thread` - Thread handle
- `mutex` - Mutual exclusion lock
- `channel` - Message passing channel
- `atomic_int` - Lock-free atomic integer

**Key Functions:**
- `thread_create(function: FUNCTION, stack_size: INT) -> thread` - Create thread
- `thread_join(thread: thread, timeout_ms: INT) -> BOOL` - Wait for thread
- `mutex_create() -> mutex` - Create mutex
- `mutex_lock(mutex: mutex, timeout_ms: INT) -> BOOL` - Lock with timeout
- `mutex_unlock(mutex: mutex) -> VOID` - Unlock mutex
- `channel_create(capacity: INT) -> channel` - Create channel
- `channel_send(channel: channel, message: INT, timeout_ms: INT) -> BOOL` - Send message
- `channel_receive(channel: channel, timeout_ms: INT, message: PTR[INT]) -> BOOL` - Receive
- `atomic_load/store/add` - Atomic operations

**Features:**
- Deadlock prevention (no recursive locks)
- Configurable timeouts
- Both buffered and unbuffered channels
- Lock-free atomic operations
- Thread-safe by design

## Usage Examples

### Basic I/O
```aetherscript
(IMPORT_MODULE "std.io")

; Write and read a file
(DECLARE_VARIABLE (NAME "success") (TYPE BOOL)
  (INITIAL_VALUE (CALL_FUNCTION "std.io.write_file_safe"
    (ARGUMENTS 
      (STRING_LITERAL "output.txt")
      (STRING_LITERAL "Hello, AetherScript!")
      (BOOL_LITERAL FALSE)))))

(DECLARE_VARIABLE (NAME "content") (TYPE STRING)
  (INITIAL_VALUE (CALL_FUNCTION "std.io.read_file_safe"
    (ARGUMENTS 
      (STRING_LITERAL "output.txt")
      (INTEGER_LITERAL 1024)))))
```

### Verified Sorting
```aetherscript
(IMPORT_MODULE "std.collections")

(DECLARE_VARIABLE (NAME "numbers") (TYPE (ARRAY INT 10))
  (INITIAL_VALUE (ARRAY_LITERAL 
    (TYPE (ARRAY INT 10))
    (INTEGER_LITERAL 5)
    (INTEGER_LITERAL 2)
    (INTEGER_LITERAL 8)
    (INTEGER_LITERAL 1)
    (INTEGER_LITERAL 9))))

; Sort with formal verification
(CALL_FUNCTION "std.collections.sort_verified"
  (ARGUMENTS 
    (VARIABLE_REFERENCE "numbers")
    (INTEGER_LITERAL 5)))
```

### Safe Math Operations
```aetherscript
(IMPORT_MODULE "std.math")

; Safe addition with overflow protection
(DECLARE_VARIABLE (NAME "sum") (TYPE INT)
  (INITIAL_VALUE (CALL_FUNCTION "std.math.safe_add"
    (ARGUMENTS 
      (INTEGER_LITERAL 2147483640)
      (INTEGER_LITERAL 5)))))

; Mathematical functions with contracts
(DECLARE_VARIABLE (NAME "root") (TYPE FLOAT)
  (INITIAL_VALUE (CALL_FUNCTION "std.math.sqrt"
    (ARGUMENTS (FLOAT_LITERAL 16.0)))))
```

### Concurrent Programming
```aetherscript
(IMPORT_MODULE "std.concurrency")

; Create a channel for communication
(DECLARE_VARIABLE (NAME "ch") (TYPE "channel")
  (INITIAL_VALUE (CALL_FUNCTION "std.concurrency.channel_create"
    (ARGUMENTS (INTEGER_LITERAL 10)))))

; Send message
(CALL_FUNCTION "std.concurrency.channel_send"
  (ARGUMENTS 
    (VARIABLE_REFERENCE "ch")
    (INTEGER_LITERAL 42)
    (INTEGER_LITERAL 1000)))
```

## Runtime Support

The standard library is backed by a Rust runtime that provides:

1. **Memory Safety** - All allocations tracked and freed automatically
2. **Thread Safety** - Concurrent operations use proper synchronization
3. **Resource Management** - RAII patterns ensure cleanup
4. **Performance** - Zero-cost abstractions where possible
5. **Verification** - Runtime checks enforce contracts

## Integration with Compiler

The AetherScript compiler automatically:
- Resolves standard library imports
- Links with the runtime library
- Verifies contracts at compile time when possible
- Generates efficient code using LLVM
- Provides helpful error messages for contract violations

## Best Practices

1. **Always use safe variants** - Prefer `safe_add` over raw addition
2. **Check return values** - Many operations return success/failure
3. **Use resource scopes** - Let the library manage resources
4. **Specify limits** - Always provide size/time limits
5. **Handle errors** - Use try/catch for I/O and network operations
6. **Verify assumptions** - Use contracts to document expectations