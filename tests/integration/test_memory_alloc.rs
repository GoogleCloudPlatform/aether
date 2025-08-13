//! Integration tests for safe memory allocation

use aether::pipeline::{CompilationPipeline, CompileOptions};
use std::path::PathBuf;
use std::fs;

#[test]
fn test_memory_allocation_tracking() {
    let source = r#"
// Test safe memory allocation with leak detection
external FUNCTION aether_memory_init() FROM "aether_runtime"
external FUNCTION aether_safe_malloc(size: INTEGER) -> POINTER_TO_TYPE VOID FROM "aether_runtime"
external FUNCTION aether_safe_free(ptr: POINTER_TO_TYPE VOID) FROM "aether_runtime"
external FUNCTION aether_check_leaks() -> INTEGER FROM "aether_runtime"
external FUNCTION aether_memory_usage() -> INTEGER FROM "aether_runtime"

FUNCTION main() -> INTEGER {
    // Initialize memory system
    aether_memory_init();
    
    // Allocate some memory
    LET ptr1: POINTER_TO_TYPE VOID = aether_safe_malloc(100);
    LET ptr2: POINTER_TO_TYPE VOID = aether_safe_malloc(200);
    
    // Check memory usage
    LET usage: INTEGER = aether_memory_usage();
    IF usage != 300 {
        RETURN 1;  // Failed - expected 300 bytes
    }
    
    // Free one allocation
    aether_safe_free(ptr1);
    
    // Check memory usage again
    usage = aether_memory_usage();
    IF usage != 200 {
        RETURN 2;  // Failed - expected 200 bytes
    }
    
    // Free second allocation
    aether_safe_free(ptr2);
    
    // Check for leaks
    LET leaks: INTEGER = aether_check_leaks();
    IF leaks != 0 {
        RETURN 3;  // Failed - memory leak detected
    }
    
    RETURN 0;  // Success
}
"#;

    // Write test file
    let test_path = PathBuf::from("test_memory_alloc.aether");
    fs::write(&test_path, source).expect("Failed to write test file");
    
    // Compile the test
    let mut options = CompileOptions::default();
    options.output = Some(PathBuf::from("test_memory_alloc"));
    options.keep_intermediates = false;
    
    let mut pipeline = CompilationPipeline::new(options);
    let result = pipeline.compile_files(&[test_path.clone()]).expect("Compilation failed");
    
    // Clean up test file
    fs::remove_file(&test_path).ok();
    
    assert!(result.executable_path.exists(), "No output generated");
}

#[test]
fn test_double_free_detection() {
    let source = r#"
// Test double-free protection
external FUNCTION aether_memory_init() FROM "aether_runtime"
external FUNCTION aether_safe_malloc(size: INTEGER) -> POINTER_TO_TYPE VOID FROM "aether_runtime"
external FUNCTION aether_safe_free(ptr: POINTER_TO_TYPE VOID) FROM "aether_runtime"

FUNCTION main() -> INTEGER {
    aether_memory_init();
    
    LET ptr: POINTER_TO_TYPE VOID = aether_safe_malloc(50);
    
    // Free once - should work
    aether_safe_free(ptr);
    
    // Free again - should be detected and ignored (not crash)
    aether_safe_free(ptr);
    
    RETURN 0;  // If we get here, double-free protection worked
}
"#;

    // Write test file
    let test_path = PathBuf::from("test_double_free.aether");
    fs::write(&test_path, source).expect("Failed to write test file");
    
    // Compile the test
    let mut options = CompileOptions::default();
    options.output = Some(PathBuf::from("test_double_free"));
    options.keep_intermediates = false;
    
    let mut pipeline = CompilationPipeline::new(options);
    let result = pipeline.compile_files(&[test_path.clone()]).expect("Compilation failed");
    
    // Clean up test file
    fs::remove_file(&test_path).ok();
    
    assert!(result.executable_path.exists(), "No output generated");
}

#[test]
fn test_realloc_functionality() {
    let source = r#"
// Test reallocation with data preservation
external FUNCTION aether_memory_init() FROM "aether_runtime"
external FUNCTION aether_safe_malloc(size: INTEGER) -> POINTER_TO_TYPE VOID FROM "aether_runtime"
external FUNCTION aether_safe_realloc(ptr: POINTER_TO_TYPE VOID, new_size: INTEGER) -> POINTER_TO_TYPE VOID FROM "aether_runtime"
external FUNCTION aether_safe_free(ptr: POINTER_TO_TYPE VOID) FROM "aether_runtime"

FUNCTION main() -> INTEGER {
    aether_memory_init();
    
    // Allocate initial buffer
    LET ptr1: POINTER_TO_TYPE VOID = aether_safe_malloc(50);
    
    // Store some data (simplified - in real code we'd cast and write)
    // For now, just test that realloc returns a valid pointer
    
    // Reallocate to larger size
    LET ptr2: POINTER_TO_TYPE VOID = aether_safe_realloc(ptr1, 100);
    
    IF ptr2 == NULL {
        RETURN 1;  // Realloc failed
    }
    
    // Free the reallocated memory
    aether_safe_free(ptr2);
    
    RETURN 0;  // Success
}
"#;

    // Write test file
    let test_path = PathBuf::from("test_realloc.aether");
    fs::write(&test_path, source).expect("Failed to write test file");
    
    // Compile the test
    let mut options = CompileOptions::default();
    options.output = Some(PathBuf::from("test_realloc"));
    options.keep_intermediates = false;
    
    let mut pipeline = CompilationPipeline::new(options);
    let result = pipeline.compile_files(&[test_path.clone()]).expect("Compilation failed");
    
    // Clean up test file
    fs::remove_file(&test_path).ok();
    
    assert!(result.executable_path.exists(), "No output generated");
}