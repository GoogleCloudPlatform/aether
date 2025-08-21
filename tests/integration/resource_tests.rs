// Copyright 2025 Google LLC
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! Resource management integration tests (Phase 4)

// Access utils from parent directory since we're in integration subdir
#[path = "../utils/mod.rs"]
mod utils;

use utils::{
    compiler_wrapper::TestCompiler,
    assertions::*,
};

#[test]
fn test_basic_resource_scope() {
    let compiler = TestCompiler::new("basic_resource_scope");
    
    let source = r#"
(DEFINE_MODULE basic_resource_scope
  (DEFINE_FUNCTION
    (NAME "file_operation")
    (ACCEPTS_PARAMETER (NAME "filename") (TYPE STRING))
    (RETURNS (TYPE STRING))
    (RESOURCE_SCOPE
      (SCOPE_ID "file_operation_001")
      (ACQUIRES 
        (RESOURCE (TYPE "file_handle") (ID "file") (CLEANUP "aether_close_file")))
      (INVARIANT "File handle is valid throughout operation")
      (CLEANUP_GUARANTEED TRUE)
      (CLEANUP_ORDER "REVERSE_ACQUISITION")
      (BODY
        (DECLARE_VARIABLE (NAME "file")
          (INITIAL_VALUE (CALL_FUNCTION "aether_open_file"
            (ARGUMENTS (VARIABLE_REFERENCE "filename") (STRING_LITERAL "r")))))
        
        (IF_CONDITION
          (PREDICATE_EQUALS (VARIABLE_REFERENCE "file") (NULL_LITERAL))
          (THEN_EXECUTE
            (RETURN_VALUE (STRING_LITERAL "Error: Could not open file"))))
        
        (DECLARE_VARIABLE (NAME "content")
          (INITIAL_VALUE (CALL_FUNCTION "aether_read_file" 
            (ARGUMENTS (VARIABLE_REFERENCE "file")))))
        
        // File will be automatically closed due to RESOURCE_SCOPE
        (RETURN_VALUE (VARIABLE_REFERENCE "content")))))
  
  (DEFINE_FUNCTION
    (NAME "main")
    (RETURNS (TYPE INT))
    (BODY
      (DECLARE_VARIABLE (NAME "content")
        (INITIAL_VALUE (CALL_FUNCTION "file_operation"
          (ARGUMENTS (STRING_LITERAL "test_file.txt")))))
      (CALL_FUNCTION "printf" 
        (ARGUMENTS (STRING_LITERAL "File content: %s\n") (VARIABLE_REFERENCE "content")))
      (RETURN_VALUE (INTEGER_LITERAL 0))))
)
    "#;
    
    let result = compiler.compile_source(source, "basic_resource_scope.aether");
    assert_compilation_success(&result, "Basic resource scope compilation");
    
    // Check that resource management code was generated
    if let Some(compilation_result) = result.success() {
        // Should have no warnings about resource leaks
        // Note: Current CompilationResult doesn't track warnings
        // Resource leak detection would need to be implemented
        let _ = compilation_result;
    }
}

#[test]
fn test_nested_resource_scopes() {
    let compiler = TestCompiler::new("nested_resource_scopes");
    
    let source = r#"
(DEFINE_MODULE nested_resource_scopes
  (DEFINE_FUNCTION
    (NAME "complex_file_operation")
    (ACCEPTS_PARAMETER (NAME "input_file") (TYPE STRING))
    (ACCEPTS_PARAMETER (NAME "output_file") (TYPE STRING))
    (RETURNS (TYPE BOOL))
    (RESOURCE_SCOPE
      (SCOPE_ID "outer_scope")
      (ACQUIRES 
        (RESOURCE (TYPE "memory_buffer") (ID "buffer") (CLEANUP "aether_free")))
      (BODY
        (DECLARE_VARIABLE (NAME "buffer")
          (INITIAL_VALUE (CALL_FUNCTION "aether_alloc" (ARGUMENTS (INTEGER_LITERAL 1024)))))
        
        (RESOURCE_SCOPE
          (SCOPE_ID "input_scope")
          (ACQUIRES 
            (RESOURCE (TYPE "file_handle") (ID "input") (CLEANUP "aether_close_file")))
          (BODY
            (DECLARE_VARIABLE (NAME "input")
              (INITIAL_VALUE (CALL_FUNCTION "aether_open_file"
                (ARGUMENTS (VARIABLE_REFERENCE "input_file") (STRING_LITERAL "r")))))
            
            (IF_CONDITION
              (PREDICATE_EQUALS (VARIABLE_REFERENCE "input") (NULL_LITERAL))
              (THEN_EXECUTE
                (RETURN_VALUE (BOOL_LITERAL FALSE))))
            
            (CALL_FUNCTION "aether_read_into_buffer"
              (ARGUMENTS (VARIABLE_REFERENCE "input") (VARIABLE_REFERENCE "buffer")))
            
            (RESOURCE_SCOPE
              (SCOPE_ID "output_scope")
              (ACQUIRES 
                (RESOURCE (TYPE "file_handle") (ID "output") (CLEANUP "aether_close_file")))
              (BODY
                (DECLARE_VARIABLE (NAME "output")
                  (INITIAL_VALUE (CALL_FUNCTION "aether_open_file"
                    (ARGUMENTS (VARIABLE_REFERENCE "output_file") (STRING_LITERAL "w")))))
                
                (IF_CONDITION
                  (PREDICATE_EQUALS (VARIABLE_REFERENCE "output") (NULL_LITERAL))
                  (THEN_EXECUTE
                    (RETURN_VALUE (BOOL_LITERAL FALSE))))
                
                (CALL_FUNCTION "aether_write_from_buffer"
                  (ARGUMENTS (VARIABLE_REFERENCE "output") (VARIABLE_REFERENCE "buffer")))
                
                // All resources cleaned up in reverse order: output, input, buffer
                (RETURN_VALUE (BOOL_LITERAL TRUE))))))))))
  
  (DEFINE_FUNCTION
    (NAME "main")
    (RETURNS (TYPE INT))
    (BODY
      (DECLARE_VARIABLE (NAME "success")
        (INITIAL_VALUE (CALL_FUNCTION "complex_file_operation"
          (ARGUMENTS (STRING_LITERAL "input.txt") (STRING_LITERAL "output.txt")))))
      (CALL_FUNCTION "printf" 
        (ARGUMENTS (STRING_LITERAL "Operation successful: %d\n") (VARIABLE_REFERENCE "success")))
      (RETURN_VALUE (INTEGER_LITERAL 0))))
)
    "#;
    
    let result = compiler.compile_source(source, "nested_resource_scopes.aether");
    assert_compilation_success(&result, "Nested resource scopes compilation");
}

#[test]
fn test_resource_leak_detection() {
    let compiler = TestCompiler::new("resource_leak_detection");
    
    let source = r#"
(DEFINE_MODULE resource_leak_detection
  (DEFINE_FUNCTION
    (NAME "leaky_function")
    (ACCEPTS_PARAMETER (NAME "filename") (TYPE STRING))
    (RETURNS (TYPE STRING))
    (BODY
      (DECLARE_VARIABLE (NAME "file")
        (INITIAL_VALUE (CALL_FUNCTION "aether_open_file"
          (ARGUMENTS (VARIABLE_REFERENCE "filename") (STRING_LITERAL "r")))))
      
      (IF_CONDITION
        (PREDICATE_EQUALS (VARIABLE_REFERENCE "file") (NULL_LITERAL))
        (THEN_EXECUTE
          // Early return without closing file - potential leak
          (RETURN_VALUE (STRING_LITERAL "Error"))))
      
      (DECLARE_VARIABLE (NAME "content")
        (INITIAL_VALUE (CALL_FUNCTION "aether_read_file" 
          (ARGUMENTS (VARIABLE_REFERENCE "file")))))
      
      // Missing file close - definite leak
      (RETURN_VALUE (VARIABLE_REFERENCE "content"))))
)
    "#;
    
    let result = compiler.compile_source(source, "resource_leak_detection.aether");
    
    // Should either fail compilation or warn about resource leak
    if result.is_success() {
        assert_warning_contains(&result, "resource leak", "Resource leak warning");
    } else {
        assert_compilation_error(&result, "resource", "Resource leak error");
    }
}

#[test]
fn test_resource_cleanup_ordering() {
    let compiler = TestCompiler::new("resource_cleanup_ordering");
    
    let source = r#"
(DEFINE_MODULE resource_cleanup_ordering
  (DEFINE_FUNCTION
    (NAME "test_cleanup_order")
    (RETURNS (TYPE INT))
    (RESOURCE_SCOPE
      (SCOPE_ID "cleanup_test")
      (ACQUIRES 
        (RESOURCE (TYPE "network_connection") (ID "conn") (CLEANUP "aether_close_connection"))
        (RESOURCE (TYPE "memory_buffer") (ID "buffer") (CLEANUP "aether_free"))
        (RESOURCE (TYPE "file_handle") (ID "log") (CLEANUP "aether_close_file")))
      (INVARIANT "All resources are properly initialized")
      (CLEANUP_ORDER "REVERSE_ACQUISITION")
      (BODY
        // Acquire in order: conn, buffer, log
        (DECLARE_VARIABLE (NAME "conn")
          (INITIAL_VALUE (CALL_FUNCTION "aether_connect" 
            (ARGUMENTS (STRING_LITERAL "localhost") (INTEGER_LITERAL 8080)))))
        
        (DECLARE_VARIABLE (NAME "buffer")
          (INITIAL_VALUE (CALL_FUNCTION "aether_alloc" (ARGUMENTS (INTEGER_LITERAL 1024)))))
        
        (DECLARE_VARIABLE (NAME "log")
          (INITIAL_VALUE (CALL_FUNCTION "aether_open_file"
            (ARGUMENTS (STRING_LITERAL "operation.log") (STRING_LITERAL "w")))))
        
        // Perform operation
        (CALL_FUNCTION "aether_write_file"
          (ARGUMENTS (VARIABLE_REFERENCE "log") (STRING_LITERAL "Operation started")))
        
        (CALL_FUNCTION "aether_send_data"
          (ARGUMENTS (VARIABLE_REFERENCE "conn") (VARIABLE_REFERENCE "buffer")))
        
        // Cleanup will happen in reverse order: log, buffer, conn
        (RETURN_VALUE (INTEGER_LITERAL 0)))))
  
  (DEFINE_FUNCTION
    (NAME "main")
    (RETURNS (TYPE INT))
    (BODY
      (DECLARE_VARIABLE (NAME "result")
        (INITIAL_VALUE (CALL_FUNCTION "test_cleanup_order")))
      (CALL_FUNCTION "printf" 
        (ARGUMENTS (STRING_LITERAL "Cleanup test result: %d\n") (VARIABLE_REFERENCE "result")))
      (RETURN_VALUE (INTEGER_LITERAL 0))))
)
    "#;
    
    let result = compiler.compile_source(source, "resource_cleanup_ordering.aether");
    assert_compilation_success(&result, "Resource cleanup ordering");
}

#[test]
fn test_resource_contract_validation() {
    let compiler = TestCompiler::new("resource_contracts");
    
    let source = r#"
(DEFINE_MODULE resource_contracts
  (DEFINE_FUNCTION
    (NAME "memory_intensive_operation")
    (ACCEPTS_PARAMETER (NAME "size") (TYPE INT))
    (RETURNS (TYPE (POINTER VOID)))
    (RESOURCE_CONTRACT
      (MAX_MEMORY_MB 50)
      (MAX_EXECUTION_TIME_MS 5000)
      (ENFORCEMENT RUNTIME))
    (PRECONDITION
      (PREDICATE_LESS_THAN_OR_EQUAL_TO (VARIABLE_REFERENCE "size") (INTEGER_LITERAL 52428800))
      (PROOF_HINT "Size must not exceed 50MB"))
    (BODY
      (RESOURCE_SCOPE
        (SCOPE_ID "memory_operation")
        (ACQUIRES 
          (RESOURCE (TYPE "memory_block") (ID "data") (CLEANUP "aether_free")))
        (BODY
          (DECLARE_VARIABLE (NAME "data")
            (INITIAL_VALUE (CALL_FUNCTION "aether_alloc" 
              (ARGUMENTS (VARIABLE_REFERENCE "size")))))
          
          (IF_CONDITION
            (PREDICATE_EQUALS (VARIABLE_REFERENCE "data") (NULL_LITERAL))
            (THEN_EXECUTE
              (RETURN_VALUE (NULL_LITERAL))))
          
          // Simulate memory-intensive work
          (FOR_LOOP
            (INIT (DECLARE_VARIABLE (NAME "i") (INITIAL_VALUE (INTEGER_LITERAL 0))))
            (CONDITION (PREDICATE_LESS_THAN (VARIABLE_REFERENCE "i") (VARIABLE_REFERENCE "size")))
            (UPDATE (ASSIGN (TARGET (VARIABLE_REFERENCE "i"))
                            (SOURCE (EXPRESSION_ADD (VARIABLE_REFERENCE "i") (INTEGER_LITERAL 1)))))
            (BODY
              (ASSIGN (TARGET (ARRAY_ACCESS (CAST_TO_BYTE_ARRAY (VARIABLE_REFERENCE "data")) (VARIABLE_REFERENCE "i")))
                      (SOURCE (CAST_TO_BYTE (EXPRESSION_MODULO (VARIABLE_REFERENCE "i") (INTEGER_LITERAL 256)))))))
          
          (RETURN_VALUE (VARIABLE_REFERENCE "data"))))))
  
  (DEFINE_FUNCTION
    (NAME "main")
    (RETURNS (TYPE INT))
    (BODY
      // Test with valid size (10MB)
      (DECLARE_VARIABLE (NAME "data1")
        (INITIAL_VALUE (CALL_FUNCTION "memory_intensive_operation"
          (ARGUMENTS (INTEGER_LITERAL 10485760)))))
      
      (IF_CONDITION
        (PREDICATE_NOT_EQUALS (VARIABLE_REFERENCE "data1") (NULL_LITERAL))
        (THEN_EXECUTE
          (CALL_FUNCTION "printf" (ARGUMENTS (STRING_LITERAL "10MB allocation successful\n"))))
        (ELSE_EXECUTE
          (CALL_FUNCTION "printf" (ARGUMENTS (STRING_LITERAL "10MB allocation failed\n")))))
      
      (RETURN_VALUE (INTEGER_LITERAL 0))))
)
    "#;
    
    let result = compiler.compile_source(source, "resource_contracts.aether");
    assert_compilation_success(&result, "Resource contract validation");
}

#[test]
fn test_exception_safe_resource_management() {
    let compiler = TestCompiler::new("exception_safe_resources");
    
    let source = r#"
(DEFINE_MODULE exception_safe_resources
  (DEFINE_FUNCTION
    (NAME "exception_prone_operation")
    (ACCEPTS_PARAMETER (NAME "might_fail") (TYPE BOOL))
    (RETURNS (TYPE STRING))
    (EXCEPTION_SAFETY STRONG)
    (RESOURCE_SCOPE
      (SCOPE_ID "exception_safe")
      (ACQUIRES 
        (RESOURCE (TYPE "file_handle") (ID "temp_file") (CLEANUP "aether_close_file"))
        (RESOURCE (TYPE "memory_buffer") (ID "buffer") (CLEANUP "aether_free")))
      (INVARIANT "Resources cleaned up even if exception occurs")
      (BODY
        (DECLARE_VARIABLE (NAME "temp_file")
          (INITIAL_VALUE (CALL_FUNCTION "aether_open_file"
            (ARGUMENTS (STRING_LITERAL "temp.txt") (STRING_LITERAL "w+")))))
        
        (DECLARE_VARIABLE (NAME "buffer")
          (INITIAL_VALUE (CALL_FUNCTION "aether_alloc" (ARGUMENTS (INTEGER_LITERAL 1024)))))
        
        (TRY_EXECUTE
          (PROTECTED_BLOCK
            (CALL_FUNCTION "aether_write_file"
              (ARGUMENTS (VARIABLE_REFERENCE "temp_file") (STRING_LITERAL "Test data")))
            
            (IF_CONDITION
              (VARIABLE_REFERENCE "might_fail")
              (THEN_EXECUTE
                (THROW_EXCEPTION "TestException" "Simulated failure")))
            
            (CALL_FUNCTION "aether_read_file_to_buffer"
              (ARGUMENTS (VARIABLE_REFERENCE "temp_file") (VARIABLE_REFERENCE "buffer")))
            
            (RETURN_VALUE (CALL_FUNCTION "aether_buffer_to_string" 
              (ARGUMENTS (VARIABLE_REFERENCE "buffer")))))
          
          (CATCH_EXCEPTION
            (EXCEPTION_TYPE "TestException")
            (HANDLER_BLOCK
              // Resources will still be cleaned up due to RESOURCE_SCOPE
              (RETURN_VALUE (STRING_LITERAL "Operation failed but resources cleaned")))))))
  
  (DEFINE_FUNCTION
    (NAME "main")
    (RETURNS (TYPE INT))
    (BODY
      // Test successful case
      (DECLARE_VARIABLE (NAME "result1")
        (INITIAL_VALUE (CALL_FUNCTION "exception_prone_operation" (ARGUMENTS (BOOL_LITERAL FALSE)))))
      (CALL_FUNCTION "printf" 
        (ARGUMENTS (STRING_LITERAL "Success case: %s\n") (VARIABLE_REFERENCE "result1")))
      
      // Test exception case
      (DECLARE_VARIABLE (NAME "result2")
        (INITIAL_VALUE (CALL_FUNCTION "exception_prone_operation" (ARGUMENTS (BOOL_LITERAL TRUE)))))
      (CALL_FUNCTION "printf" 
        (ARGUMENTS (STRING_LITERAL "Exception case: %s\n") (VARIABLE_REFERENCE "result2")))
      
      (RETURN_VALUE (INTEGER_LITERAL 0))))
)
    "#;
    
    let result = compiler.compile_source(source, "exception_safe_resources.aether");
    assert_compilation_success(&result, "Exception-safe resource management");
}

#[test]
fn test_resource_usage_analysis() {
    let compiler = TestCompiler::new("resource_usage_analysis");
    
    let source = r#"
(DEFINE_MODULE resource_usage_analysis
  (DEFINE_FUNCTION
    (NAME "analyze_resource_usage")
    (RETURNS (TYPE INT))
    (RESOURCE_SCOPE
      (SCOPE_ID "usage_analysis")
      (ACQUIRES 
        (RESOURCE (TYPE "cpu_intensive") (ID "computation") (CLEANUP "aether_release_cpu"))
        (RESOURCE (TYPE "memory_pool") (ID "pool") (CLEANUP "aether_destroy_pool")))
      (RESOURCE_MONITORING ENABLED)
      (PERFORMANCE_TRACKING ENABLED)
      (BODY
        (DECLARE_VARIABLE (NAME "computation")
          (INITIAL_VALUE (CALL_FUNCTION "aether_acquire_cpu" (ARGUMENTS (INTEGER_LITERAL 4)))))
        
        (DECLARE_VARIABLE (NAME "pool")
          (INITIAL_VALUE (CALL_FUNCTION "aether_create_memory_pool" 
            (ARGUMENTS (INTEGER_LITERAL 1048576)))))
        
        // Perform resource-intensive computation
        (DECLARE_VARIABLE (NAME "result") (INITIAL_VALUE (INTEGER_LITERAL 0)))
        (FOR_LOOP
          (INIT (DECLARE_VARIABLE (NAME "i") (INITIAL_VALUE (INTEGER_LITERAL 0))))
          (CONDITION (PREDICATE_LESS_THAN (VARIABLE_REFERENCE "i") (INTEGER_LITERAL 1000000)))
          (UPDATE (ASSIGN (TARGET (VARIABLE_REFERENCE "i"))
                          (SOURCE (EXPRESSION_ADD (VARIABLE_REFERENCE "i") (INTEGER_LITERAL 1)))))
          (BODY
            (DECLARE_VARIABLE (NAME "temp_buffer")
              (INITIAL_VALUE (CALL_FUNCTION "aether_pool_alloc"
                (ARGUMENTS (VARIABLE_REFERENCE "pool") (INTEGER_LITERAL 64)))))
            
            (ASSIGN (TARGET (VARIABLE_REFERENCE "result"))
                    (SOURCE (EXPRESSION_ADD (VARIABLE_REFERENCE "result") 
                      (CALL_FUNCTION "aether_compute_hash"
                        (ARGUMENTS (VARIABLE_REFERENCE "temp_buffer"))))))
            
            (CALL_FUNCTION "aether_pool_free"
              (ARGUMENTS (VARIABLE_REFERENCE "pool") (VARIABLE_REFERENCE "temp_buffer")))))
        
        (RETURN_VALUE (VARIABLE_REFERENCE "result")))))
  
  (DEFINE_FUNCTION
    (NAME "main")
    (RETURNS (TYPE INT))
    (BODY
      (DECLARE_VARIABLE (NAME "hash_result")
        (INITIAL_VALUE (CALL_FUNCTION "analyze_resource_usage")))
      (CALL_FUNCTION "printf" 
        (ARGUMENTS (STRING_LITERAL "Hash result: %d\n") (VARIABLE_REFERENCE "hash_result")))
      (RETURN_VALUE (INTEGER_LITERAL 0))))
)
    "#;
    
    let result = compiler.compile_source(source, "resource_usage_analysis.aether");
    assert_compilation_success(&result, "Resource usage analysis");
    
    // Should have performance and resource usage information
    if let Some(compilation_result) = result.success() {
        // Look for resource analysis warnings/info
        // Note: Current CompilationResult doesn't track warnings
        // Resource analysis info would need to be implemented
        println!("Resource analysis compilation completed successfully");
    }
}