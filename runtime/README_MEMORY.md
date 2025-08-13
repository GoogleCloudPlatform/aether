# Aether Runtime Safe Memory Allocation

The Aether runtime provides a safe memory allocation system with built-in protection against common memory errors.

## Features

### 1. **Memory Tracking**
- All allocations are tracked in a global registry
- Memory usage statistics available at runtime
- Leak detection on demand

### 2. **Safety Checks**
- **Double-free protection**: Freed memory is marked and subsequent frees are ignored
- **Buffer overflow detection**: Guard bytes before and after each allocation
- **Magic number validation**: Detects corrupted headers or invalid pointers
- **Proper alignment**: Ensures allocations meet platform alignment requirements

### 3. **API Functions

```c
// Initialize the memory system (call once at program start)
void aether_memory_init();

// Allocate memory with safety checks
void* aether_safe_malloc(size_t size);

// Free memory with safety checks
void aether_safe_free(void* ptr);

// Reallocate memory (preserves data)
void* aether_safe_realloc(void* ptr, size_t new_size);

// Check for memory leaks (returns number of leaked allocations)
size_t aether_check_leaks();

// Get current memory usage in bytes
size_t aether_memory_usage();
```

## Memory Layout

Each allocation has the following structure:

```
[AllocHeader][Guard Bytes][User Data][Guard Bytes]
```

- **AllocHeader**: Contains size, magic number, and header guard bytes
- **Guard Bytes**: 8-byte patterns (0xDEADBEEF) to detect overflows
- **User Data**: The actual allocated memory for the user

## Usage Example

```aether
external FUNCTION aether_memory_init() FROM "aether_runtime"
external FUNCTION aether_safe_malloc(size: INTEGER) -> POINTER_TO_TYPE VOID FROM "aether_runtime"
external FUNCTION aether_safe_free(ptr: POINTER_TO_TYPE VOID) FROM "aether_runtime"
external FUNCTION aether_check_leaks() -> INTEGER FROM "aether_runtime"

FUNCTION main() -> INTEGER {
    // Initialize memory system
    aether_memory_init();
    
    // Allocate 100 bytes
    LET buffer: POINTER_TO_TYPE VOID = aether_safe_malloc(100);
    
    // Use the buffer...
    
    // Free when done
    aether_safe_free(buffer);
    
    // Check for leaks at program end
    LET leaks: INTEGER = aether_check_leaks();
    IF leaks > 0 {
        // Handle memory leaks
        RETURN 1;
    }
    
    RETURN 0;
}
```

## Error Messages

The system prints diagnostic messages to stderr when errors are detected:

- `AETHER MEMORY ERROR: Double free detected at <address>`
- `AETHER MEMORY ERROR: Invalid free of <address> (bad magic: <value>)`
- `AETHER MEMORY ERROR: Buffer underflow detected at <address>`
- `AETHER MEMORY ERROR: Buffer overflow detected at <address>`
- `AETHER MEMORY ERROR: Invalid realloc of <address>`
- `AETHER MEMORY: N allocation(s) leaked: ...`

## Implementation Details

- Thread-safe using Rust's `Mutex`
- Zero-size allocations return NULL
- Freed memory is not immediately returned to the OS (for detection)
- Compatible with existing runtime functions (arrays, strings)

## Performance Considerations

The safety checks add overhead compared to raw malloc/free:
- Additional memory per allocation (header + guard bytes)
- Mutex lock/unlock for tracking
- Guard byte validation on free

For performance-critical code, you can still use the original `aether_malloc`/`aether_free` functions, but without the safety guarantees.