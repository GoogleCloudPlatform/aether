# Resource Management in AetherScript

## Overview

Phase 4 of the FINAL_DESIGN.md implementation provides deterministic resource management through explicit resource scopes. This system ensures that resources are always properly acquired and released, preventing leaks and use-after-free errors that LLMs might otherwise introduce.

## Key Features

### 1. RESOURCE_SCOPE Construct

The `RESOURCE_SCOPE` construct provides explicit resource management with guaranteed cleanup:

```lisp
(RESOURCE_SCOPE
  (NAME "scope_identifier")
  (ACQUIRE_RESOURCE
    (RESOURCE_TYPE "file_handle")
    (RESOURCE_BINDING "file")
    (VALUE (CALL_FUNCTION (NAME "open_file") ...))
    (CLEANUP "file_close"))
  (BODY
    ;; Use resources here
    ;; Cleanup is guaranteed even if exceptions occur
  ))
```

### 2. Resource Types

Built-in resource types with automatic mapping:
- `file_handle` - File system handles
- `memory_buffer` - Allocated memory blocks
- `tcp_socket` / `udp_socket` - Network connections
- `mutex` / `semaphore` - Synchronization primitives
- `database_connection` - Database connections
- `thread` - Thread handles

### 3. Cleanup Strategies

Multiple cleanup ordering strategies:
- **ReverseAcquisition** (default) - LIFO order
- **ForwardAcquisition** - FIFO order
- **DependencyBased** - Based on resource dependencies
- **Parallel** - Concurrent cleanup where safe

### 4. Resource Analysis

The ResourceAnalyzer provides:
- **Leak Detection** - Identifies resources not released
- **Double Release Detection** - Catches multiple releases
- **Use After Release** - Detects access after cleanup
- **Usage Pattern Analysis** - Tracks resource utilization
- **Optimization Suggestions** - Recommends pooling, lazy acquisition

### 5. Resource Contracts

Functions can declare resource limits:

```lisp
(RESOURCE_CONTRACT
  (MAX_FILE_HANDLES 10)
  (MAX_MEMORY_MB 100)
  (MAX_THREADS 4))
```

## Implementation Details

### AST Nodes

New AST nodes in `ast/resource.rs`:
- `ResourceScope` - Main scope construct
- `ResourceAcquisition` - Resource acquisition spec
- `CleanupSpecification` - How to clean up resources
- `ResourceContract` - Resource usage limits

### Analysis

Resource analysis in `resource/analysis.rs`:
- Tracks resource lifecycle
- Validates proper usage
- Generates optimization suggestions
- Reports violations as structured errors

### Parser Support

Extended parser with:
- `RESOURCE_SCOPE` keyword
- `ACQUIRE_RESOURCE` for acquisitions
- `CLEANUP` specifications
- `CLEANUP_ORDER` strategies

### Semantic Analysis

Semantic analyzer:
- Validates resource scopes
- Adds resource bindings to symbol table
- Integrates with resource analyzer
- Reports resource errors

## Usage Examples

### Basic File Handling

```lisp
(RESOURCE_SCOPE
  (NAME "file_operation")
  (ACQUIRE_RESOURCE
    (RESOURCE_TYPE "file_handle")
    (RESOURCE_BINDING "input")
    (VALUE (CALL_FUNCTION (NAME "open_file") 
      (ARGUMENT "data.txt")
      (ARGUMENT "r")))
    (CLEANUP "file_close"))
  (BODY
    (DECLARE_VARIABLE
      (NAME "content")
      (TYPE STRING)
      (INITIAL_VALUE (CALL_FUNCTION (NAME "read_all") (ARGUMENT input))))
    (RETURN_VALUE content)))
```

### Nested Resource Scopes

```lisp
(RESOURCE_SCOPE
  (NAME "outer_scope")
  (ACQUIRE_RESOURCE
    (RESOURCE_TYPE "database_connection")
    (RESOURCE_BINDING "db")
    (VALUE (CALL_FUNCTION (NAME "connect_db") ...))
    (CLEANUP "disconnect_db"))
  (BODY
    (RESOURCE_SCOPE
      (NAME "inner_scope")
      (ACQUIRE_RESOURCE
        (RESOURCE_TYPE "database_statement")
        (RESOURCE_BINDING "stmt")
        (VALUE (CALL_FUNCTION (NAME "prepare_statement") ...))
        (CLEANUP "close_statement"))
      (BODY
        ;; Use both db and stmt here
      ))))
```

### Error Handling with Resources

```lisp
(TRY_EXECUTE
  (PROTECTED_BLOCK
    (RESOURCE_SCOPE
      (NAME "protected_resources")
      (ACQUIRE_RESOURCE ...)
      (BODY
        ;; May throw exceptions
        ;; Resources still cleaned up
      )))
  (CATCH_EXCEPTION
    (EXCEPTION_TYPE "ResourceError")
    (HANDLER_BLOCK
      ;; Resources already cleaned up
    )))
```

## Benefits for LLM Code Generation

1. **Explicit Lifecycle** - Clear acquire/release boundaries
2. **No Manual Cleanup** - Automatic cleanup prevents leaks
3. **Scope Visualization** - Resources tied to lexical scopes
4. **Error Resilience** - Cleanup happens even on exceptions
5. **Static Analysis** - Compile-time leak detection

## Resource Optimization

The analyzer provides optimization suggestions:

### Resource Pooling
```
(OPTIMIZATION
  (TYPE UsePool)
  (RESOURCE "file_handle")
  (DESCRIPTION "Resource 'file_handle' acquired 15 times/sec. Consider pooling.")
  (MEMORY_SAVED_MB 5.2)
  (LATENCY_REDUCED_MS 12.5))
```

### Lazy Acquisition
```
(OPTIMIZATION
  (TYPE LazyAcquisition)
  (RESOURCE "memory_buffer")
  (DESCRIPTION "Resource 'buffer' acquired but never used in 30% of executions"))
```

## Integration with Other Phases

### With Verification (Phase 2)
- Resource invariants verified
- Cleanup guarantees proven
- Contract compliance checked

### With Error System (Phase 3)
- Structured errors for resource violations
- Auto-fix suggestions for leaks
- Intent analysis for resource usage

### With Pattern Library (Phase 5)
- Common resource patterns
- Verified cleanup sequences
- Reusable resource management templates

## Testing

Comprehensive tests in:
- `resource/analysis.rs` - Unit tests for analyzer
- `examples/resource_demo.aether` - Usage examples
- Integration tests for full pipeline

## Future Enhancements

1. **Resource Pools** - Built-in pooling support
2. **Async Resources** - Asynchronous acquisition/release
3. **Resource Metrics** - Runtime usage tracking
4. **Custom Resources** - User-defined resource types
5. **Resource Transactions** - All-or-nothing acquisition

## Summary

The resource management system in AetherScript ensures that LLM-generated code handles resources correctly by default. Through explicit scopes, automatic cleanup, and compile-time analysis, it eliminates entire classes of resource-related bugs while maintaining the clarity that LLMs need for effective code generation.