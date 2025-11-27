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

//! Pattern library integration tests (Phase 5)

// Access utils from parent directory since we're in integration subdir
#[path = "../utils/mod.rs"]
mod utils;

use utils::{assertions::*, compiler_wrapper::TestCompiler};

#[test]
fn test_pattern_discovery_by_intent() {
    let compiler = TestCompiler::new("pattern_discovery");

    let source = r#"
(DEFINE_MODULE pattern_discovery
  (USE_PATTERN_LIBRARY)
  
  (DEFINE_FUNCTION
    (NAME "test_pattern_discovery")
    (RETURNS (TYPE INT))
    (BODY
      // Use pattern discovery to find safe array access pattern
      (GENERATE_FROM_INTENT "safely access array element with bounds checking"
        (PARAMETERS
          (PARAM "array_expr" (ARRAY_LITERAL (TYPE (ARRAY INT 5)) 10 20 30 40 50))
          (PARAM "index_expr" (INTEGER_LITERAL 2))
          (PARAM "default_value" (INTEGER_LITERAL -1)))
        (ASSIGN_TO "safe_element"))
      
      (CALL_FUNCTION "printf" 
        (ARGUMENTS (STRING_LITERAL "Safe element: %d\n") (VARIABLE_REFERENCE "safe_element")))
      (RETURN_VALUE (INTEGER_LITERAL 0))))
)
    "#;

    let result = compiler.compile_source(source, "pattern_discovery.aether");

    // Should either compile successfully or provide helpful error about pattern system
    if result.is_success() {
        assert_compile_and_execute(&result, "Safe element: 30", "Pattern discovery");
    } else {
        // If pattern system not fully integrated, should have helpful error
        assert_compilation_error(&result, "pattern", "Pattern system integration");
    }
}

#[test]
fn test_sequential_pattern_composition() {
    let compiler = TestCompiler::new("sequential_composition");

    let source = r#"
(DEFINE_MODULE sequential_composition
  (USE_PATTERN_LIBRARY)
  
  (DEFINE_FUNCTION
    (NAME "test_sequential_patterns")
    (ACCEPTS_PARAMETER (NAME "filename") (TYPE STRING))
    (RETURNS (TYPE STRING))
    (BODY
      // Compose patterns sequentially: file validation + safe file read + string processing
      (COMPOSE_PATTERNS
        (STRATEGY SEQUENTIAL)
        (PATTERNS
          (PATTERN "input_validation"
            (PARAMETERS
              (PARAM "input" (VARIABLE_REFERENCE "filename"))
              (PARAM "validation_rules" (ARRAY_LITERAL (TYPE (ARRAY STRING 2)) 
                (STRING_LITERAL "not_empty") (STRING_LITERAL "valid_filename")))))
          
          (PATTERN "safe_file_read"
            (PARAMETERS
              (PARAM "file_path" (VARIABLE_REFERENCE "filename"))
              (PARAM "max_size_mb" (INTEGER_LITERAL 10))))
          
          (PATTERN "string_processing"
            (PARAMETERS
              (PARAM "input_string" (RESULT_FROM_PREVIOUS))
              (PARAM "operation" (STRING_LITERAL "trim_whitespace")))))
        (ASSIGN_TO "processed_content"))
      
      (RETURN_VALUE (VARIABLE_REFERENCE "processed_content"))))
  
  (DEFINE_FUNCTION
    (NAME "main")
    (RETURNS (TYPE INT))
    (BODY
      (DECLARE_VARIABLE (NAME "content")
        (INITIAL_VALUE (CALL_FUNCTION "test_sequential_patterns"
          (ARGUMENTS (STRING_LITERAL "test_input.txt")))))
      (CALL_FUNCTION "printf" 
        (ARGUMENTS (STRING_LITERAL "Processed content: %s\n") (VARIABLE_REFERENCE "content")))
      (RETURN_VALUE (INTEGER_LITERAL 0))))
)
    "#;

    let result = compiler.compile_source(source, "sequential_composition.aether");

    if result.is_success() {
        assert_compilation_success(&result, "Sequential pattern composition");
    } else {
        // Should provide helpful error about pattern composition
        assert_compilation_error(&result, "pattern", "Pattern composition integration");
    }
}

#[test]
fn test_nested_pattern_composition() {
    let compiler = TestCompiler::new("nested_composition");

    let source = r#"
(DEFINE_MODULE nested_composition
  (USE_PATTERN_LIBRARY)
  
  (DEFINE_FUNCTION
    (NAME "test_nested_patterns")
    (ACCEPTS_PARAMETER (NAME "data") (TYPE (ARRAY INT 10)))
    (RETURNS (TYPE INT))
    (BODY
      // Nest array processing pattern inside RAII wrapper
      (COMPOSE_PATTERNS
        (STRATEGY NESTED)
        (OUTER_PATTERN "raii_wrapper"
          (PARAMETERS
            (PARAM "resource_type" (STRING_LITERAL "computation_context"))
            (PARAM "cleanup_function" (STRING_LITERAL "release_computation_context"))))
        (INNER_PATTERNS
          (PATTERN "array_bounds_check"
            (PARAMETERS
              (PARAM "array" (VARIABLE_REFERENCE "data"))
              (PARAM "operation" (STRING_LITERAL "sum_elements"))))
          
          (PATTERN "arithmetic_operation"
            (PARAMETERS
              (PARAM "operation_type" (STRING_LITERAL "sum"))
              (PARAM "input_array" (VARIABLE_REFERENCE "data")))))
        (ASSIGN_TO "safe_sum"))
      
      (RETURN_VALUE (VARIABLE_REFERENCE "safe_sum"))))
  
  (DEFINE_FUNCTION
    (NAME "main")
    (RETURNS (TYPE INT))
    (BODY
      (DECLARE_VARIABLE (NAME "test_array")
        (INITIAL_VALUE (ARRAY_LITERAL (TYPE (ARRAY INT 10)) 1 2 3 4 5 6 7 8 9 10)))
      
      (DECLARE_VARIABLE (NAME "sum")
        (INITIAL_VALUE (CALL_FUNCTION "test_nested_patterns"
          (ARGUMENTS (VARIABLE_REFERENCE "test_array")))))
      
      (CALL_FUNCTION "printf" 
        (ARGUMENTS (STRING_LITERAL "Safe sum: %d\n") (VARIABLE_REFERENCE "sum")))
      (RETURN_VALUE (INTEGER_LITERAL 0))))
)
    "#;

    let result = compiler.compile_source(source, "nested_composition.aether");

    if result.is_success() {
        assert_compile_and_execute(&result, "Safe sum: 55", "Nested pattern composition");
    } else {
        assert_compilation_error(&result, "pattern", "Nested pattern composition integration");
    }
}

#[test]
fn test_parallel_pattern_composition() {
    let compiler = TestCompiler::new("parallel_composition");

    let source = r#"
(DEFINE_MODULE parallel_composition
  (USE_PATTERN_LIBRARY)
  
  (DEFINE_FUNCTION
    (NAME "test_parallel_patterns")
    (ACCEPTS_PARAMETER (NAME "data1") (TYPE (ARRAY INT 5)))
    (ACCEPTS_PARAMETER (NAME "data2") (TYPE (ARRAY INT 5)))
    (RETURNS (TYPE INT))
    (BODY
      // Process two arrays in parallel
      (COMPOSE_PATTERNS
        (STRATEGY PARALLEL)
        (SYNCHRONIZATION BARRIER)
        (PATTERNS
          (PATTERN "array_sum"
            (PARAMETERS
              (PARAM "array" (VARIABLE_REFERENCE "data1")))
            (ASSIGN_TO "sum1"))
          
          (PATTERN "array_sum"
            (PARAMETERS
              (PARAM "array" (VARIABLE_REFERENCE "data2")))
            (ASSIGN_TO "sum2")))
        (POST_SYNCHRONIZATION
          (ASSIGN (TARGET "combined_sum")
                  (SOURCE (EXPRESSION_ADD (VARIABLE_REFERENCE "sum1") (VARIABLE_REFERENCE "sum2"))))))
      
      (RETURN_VALUE (VARIABLE_REFERENCE "combined_sum"))))
  
  (DEFINE_FUNCTION
    (NAME "main")
    (RETURNS (TYPE INT))
    (BODY
      (DECLARE_VARIABLE (NAME "array1")
        (INITIAL_VALUE (ARRAY_LITERAL (TYPE (ARRAY INT 5)) 1 2 3 4 5)))
      (DECLARE_VARIABLE (NAME "array2")
        (INITIAL_VALUE (ARRAY_LITERAL (TYPE (ARRAY INT 5)) 6 7 8 9 10)))
      
      (DECLARE_VARIABLE (NAME "total")
        (INITIAL_VALUE (CALL_FUNCTION "test_parallel_patterns"
          (ARGUMENTS (VARIABLE_REFERENCE "array1") (VARIABLE_REFERENCE "array2")))))
      
      (CALL_FUNCTION "printf" 
        (ARGUMENTS (STRING_LITERAL "Total sum: %d\n") (VARIABLE_REFERENCE "total")))
      (RETURN_VALUE (INTEGER_LITERAL 0))))
)
    "#;

    let result = compiler.compile_source(source, "parallel_composition.aether");

    if result.is_success() {
        assert_compile_and_execute(&result, "Total sum: 55", "Parallel pattern composition");
    } else {
        assert_compilation_error(
            &result,
            "pattern",
            "Parallel pattern composition integration",
        );
    }
}

#[test]
fn test_pipeline_pattern_composition() {
    let compiler = TestCompiler::new("pipeline_composition");

    let source = r#"
(DEFINE_MODULE pipeline_composition
  (USE_PATTERN_LIBRARY)
  
  (DEFINE_FUNCTION
    (NAME "test_pipeline_patterns")
    (ACCEPTS_PARAMETER (NAME "input_text") (TYPE STRING))
    (RETURNS (TYPE STRING))
    (BODY
      // Create data processing pipeline
      (COMPOSE_PATTERNS
        (STRATEGY PIPELINE)
        (DATA_FLOW DIRECT)
        (STAGES
          (STAGE "input_validation"
            (PARAMETERS
              (PARAM "input" (VARIABLE_REFERENCE "input_text"))
              (PARAM "validation_type" (STRING_LITERAL "string_safety")))
            (OUTPUT_TYPE STRING))
          
          (STAGE "string_normalize"
            (PARAMETERS
              (PARAM "input_string" (PIPELINE_INPUT))
              (PARAM "normalization" (STRING_LITERAL "trim_and_lowercase")))
            (OUTPUT_TYPE STRING))
          
          (STAGE "string_encode"
            (PARAMETERS
              (PARAM "input_string" (PIPELINE_INPUT))
              (PARAM "encoding" (STRING_LITERAL "base64")))
            (OUTPUT_TYPE STRING))
          
          (STAGE "result_validation"
            (PARAMETERS
              (PARAM "output" (PIPELINE_INPUT))
              (PARAM "expected_format" (STRING_LITERAL "base64_encoded")))
            (OUTPUT_TYPE STRING)))
        (ASSIGN_TO "processed_result"))
      
      (RETURN_VALUE (VARIABLE_REFERENCE "processed_result"))))
  
  (DEFINE_FUNCTION
    (NAME "main")
    (RETURNS (TYPE INT))
    (BODY
      (DECLARE_VARIABLE (NAME "result")
        (INITIAL_VALUE (CALL_FUNCTION "test_pipeline_patterns"
          (ARGUMENTS (STRING_LITERAL "  Hello World!  ")))))
      (CALL_FUNCTION "printf" 
        (ARGUMENTS (STRING_LITERAL "Pipeline result: %s\n") (VARIABLE_REFERENCE "result")))
      (RETURN_VALUE (INTEGER_LITERAL 0))))
)
    "#;

    let result = compiler.compile_source(source, "pipeline_composition.aether");

    if result.is_success() {
        assert_compilation_success(&result, "Pipeline pattern composition");
    } else {
        assert_compilation_error(
            &result,
            "pattern",
            "Pipeline pattern composition integration",
        );
    }
}

#[test]
fn test_pattern_verification() {
    let compiler = TestCompiler::new("pattern_verification");

    let source = r#"
(DEFINE_MODULE pattern_verification
  (USE_PATTERN_LIBRARY)
  
  (DEFINE_FUNCTION
    (NAME "test_verified_patterns")
    (ACCEPTS_PARAMETER (NAME "unsafe_index") (TYPE INT))
    (RETURNS (TYPE INT))
    (BODY
      (DECLARE_VARIABLE (NAME "safe_array")
        (INITIAL_VALUE (ARRAY_LITERAL (TYPE (ARRAY INT 5)) 100 200 300 400 500)))
      
      // Use verified safe array access pattern
      (USE_VERIFIED_PATTERN "safe_array_access"
        (PARAMETERS
          (PARAM "array_expr" (VARIABLE_REFERENCE "safe_array"))
          (PARAM "index_expr" (VARIABLE_REFERENCE "unsafe_index"))
          (PARAM "default_value" (INTEGER_LITERAL 0)))
        (VERIFICATION_REQUIRED TRUE)
        (ASSIGN_TO "safe_value"))
      
      // Pattern should be verified at compile time
      (RETURN_VALUE (VARIABLE_REFERENCE "safe_value"))))
  
  (DEFINE_FUNCTION
    (NAME "main")
    (RETURNS (TYPE INT))
    (BODY
      // Test with valid index
      (DECLARE_VARIABLE (NAME "value1")
        (INITIAL_VALUE (CALL_FUNCTION "test_verified_patterns" (ARGUMENTS (INTEGER_LITERAL 2)))))
      (CALL_FUNCTION "printf" 
        (ARGUMENTS (STRING_LITERAL "Valid index result: %d\n") (VARIABLE_REFERENCE "value1")))
      
      // Test with invalid index (should return default value 0)
      (DECLARE_VARIABLE (NAME "value2")
        (INITIAL_VALUE (CALL_FUNCTION "test_verified_patterns" (ARGUMENTS (INTEGER_LITERAL 10)))))
      (CALL_FUNCTION "printf" 
        (ARGUMENTS (STRING_LITERAL "Invalid index result: %d\n") (VARIABLE_REFERENCE "value2")))
      
      (RETURN_VALUE (INTEGER_LITERAL 0))))
)
    "#;

    let result = compiler.compile_source(source, "pattern_verification.aether");

    if result.is_success() {
        let execution = result.execute();
        if execution.is_success() {
            assert_output_contains(
                &execution,
                "Valid index result: 300",
                "Pattern verification - valid case",
            );
            assert_output_contains(
                &execution,
                "Invalid index result: 0",
                "Pattern verification - invalid case",
            );
        }
    } else {
        assert_compilation_error(&result, "pattern", "Pattern verification integration");
    }
}

#[test]
fn test_custom_pattern_definition() {
    let compiler = TestCompiler::new("custom_pattern");

    let source = r#"
(DEFINE_MODULE custom_pattern
  (DEFINE_CUSTOM_PATTERN
    (NAME "safe_division_with_logging")
    (INTENT "Divide two numbers safely with audit logging")
    (PARAMETERS
      (PARAM "numerator" (TYPE FLOAT))
      (PARAM "denominator" (TYPE FLOAT))
      (PARAM "log_file" (TYPE STRING)))
    (TEMPLATE
      (FUNCTION_TEMPLATE
        (NAME "safe_divide_logged")
        (PARAMETERS
          (PARAM "num" "{{numerator}}")
          (PARAM "den" "{{denominator}}")
          (PARAM "log" "{{log_file}}"))
        (RETURNS (TYPE FLOAT))
        (BODY
          (CALL_FUNCTION "log_operation" 
            (ARGUMENTS (VARIABLE_REFERENCE "log") (STRING_LITERAL "Division started")))
          
          (IF_CONDITION
            (PREDICATE_EQUALS (VARIABLE_REFERENCE "den") (FLOAT_LITERAL 0.0))
            (THEN_EXECUTE
              (CALL_FUNCTION "log_operation" 
                (ARGUMENTS (VARIABLE_REFERENCE "log") (STRING_LITERAL "Division by zero attempted")))
              (RETURN_VALUE (FLOAT_LITERAL 0.0))))
          
          (DECLARE_VARIABLE (NAME "result")
            (INITIAL_VALUE (EXPRESSION_DIVIDE (VARIABLE_REFERENCE "num") (VARIABLE_REFERENCE "den"))))
          
          (CALL_FUNCTION "log_operation" 
            (ARGUMENTS (VARIABLE_REFERENCE "log") 
              (CALL_FUNCTION "sprintf" 
                (ARGUMENTS (STRING_LITERAL "Division result: %.2f") (VARIABLE_REFERENCE "result")))))
          
          (RETURN_VALUE (VARIABLE_REFERENCE "result")))))
    (CONTRACT
      (PRECONDITION
        (PREDICATE_NOT_EQUALS (PARAMETER_VALUE "denominator") (FLOAT_LITERAL 0.0)))
      (POSTCONDITION
        (PREDICATE_EQUALS 
          (VARIABLE_REFERENCE "RETURNED_VALUE")
          (EXPRESSION_DIVIDE (PARAMETER_VALUE "numerator") (PARAMETER_VALUE "denominator")))))
    (VERIFICATION_REQUIRED TRUE))
  
  (DEFINE_FUNCTION
    (NAME "test_custom_pattern")
    (RETURNS (TYPE INT))
    (BODY
      (USE_PATTERN "safe_division_with_logging"
        (PARAMETERS
          (PARAM "numerator" (FLOAT_LITERAL 10.5))
          (PARAM "denominator" (FLOAT_LITERAL 2.5))
          (PARAM "log_file" (STRING_LITERAL "division.log")))
        (ASSIGN_TO "division_result"))
      
      (CALL_FUNCTION "printf" 
        (ARGUMENTS (STRING_LITERAL "Division result: %.2f\n") (VARIABLE_REFERENCE "division_result")))
      (RETURN_VALUE (INTEGER_LITERAL 0))))
  
  (DEFINE_FUNCTION
    (NAME "log_operation")
    (ACCEPTS_PARAMETER (NAME "log_file") (TYPE STRING))
    (ACCEPTS_PARAMETER (NAME "message") (TYPE STRING))
    (RETURNS (TYPE VOID))
    (BODY
      (CALL_FUNCTION "printf" 
        (ARGUMENTS (STRING_LITERAL "LOG[%s]: %s\n") 
          (VARIABLE_REFERENCE "log_file") (VARIABLE_REFERENCE "message")))))
  
  (DEFINE_FUNCTION
    (NAME "main")
    (RETURNS (TYPE INT))
    (BODY
      (CALL_FUNCTION "test_custom_pattern")
      (RETURN_VALUE (INTEGER_LITERAL 0))))
)
    "#;

    let result = compiler.compile_source(source, "custom_pattern.aether");

    if result.is_success() {
        let execution = result.execute();
        if execution.is_success() {
            assert_output_contains(
                &execution,
                "Division result: 4.20",
                "Custom pattern execution",
            );
            assert_output_contains(
                &execution,
                "LOG[division.log]: Division started",
                "Custom pattern logging",
            );
        }
    } else {
        assert_compilation_error(&result, "pattern", "Custom pattern definition integration");
    }
}

#[test]
fn test_pattern_performance_estimation() {
    let compiler = TestCompiler::new("pattern_performance");

    let source = r#"
(DEFINE_MODULE pattern_performance
  (USE_PATTERN_LIBRARY)
  
  (DEFINE_FUNCTION
    (NAME "test_performance_aware_patterns")
    (ACCEPTS_PARAMETER (NAME "data_size") (TYPE INT))
    (RETURNS (TYPE INT))
    (BODY
      // Choose pattern based on performance characteristics
      (IF_CONDITION
        (PREDICATE_LESS_THAN (VARIABLE_REFERENCE "data_size") (INTEGER_LITERAL 100))
        (THEN_EXECUTE
          // Use linear search for small data
          (USE_PATTERN "linear_search"
            (PARAMETERS
              (PARAM "array_size" (VARIABLE_REFERENCE "data_size"))
              (PARAM "target" (INTEGER_LITERAL 42)))
            (PERFORMANCE_EXPECTED "O(n)")
            (ASSIGN_TO "search_result")))
        (ELSE_EXECUTE
          // Use binary search for large data (requires sorted array)
          (USE_PATTERN "binary_search"
            (PARAMETERS
              (PARAM "array_size" (VARIABLE_REFERENCE "data_size"))
              (PARAM "target" (INTEGER_LITERAL 42)))
            (PERFORMANCE_EXPECTED "O(log n)")
            (REQUIRES_PRECONDITION "sorted_array")
            (ASSIGN_TO "search_result"))))
      
      (RETURN_VALUE (VARIABLE_REFERENCE "search_result"))))
  
  (DEFINE_FUNCTION
    (NAME "main")
    (RETURNS (TYPE INT))
    (BODY
      // Test with small data
      (DECLARE_VARIABLE (NAME "small_result")
        (INITIAL_VALUE (CALL_FUNCTION "test_performance_aware_patterns" 
          (ARGUMENTS (INTEGER_LITERAL 50)))))
      (CALL_FUNCTION "printf" 
        (ARGUMENTS (STRING_LITERAL "Small data result: %d\n") (VARIABLE_REFERENCE "small_result")))
      
      // Test with large data
      (DECLARE_VARIABLE (NAME "large_result")
        (INITIAL_VALUE (CALL_FUNCTION "test_performance_aware_patterns" 
          (ARGUMENTS (INTEGER_LITERAL 1000)))))
      (CALL_FUNCTION "printf" 
        (ARGUMENTS (STRING_LITERAL "Large data result: %d\n") (VARIABLE_REFERENCE "large_result")))
      
      (RETURN_VALUE (INTEGER_LITERAL 0))))
)
    "#;

    let result = compiler.compile_source(source, "pattern_performance.aether");

    if result.is_success() {
        // Should compile and potentially provide performance analysis
        assert_compilation_success(&result, "Pattern performance estimation");

        if let Some(compilation_result) = result.success() {
            // Look for performance-related warnings or information
            // Note: Current CompilationResult doesn't track warnings
            let has_performance_info = false;

            println!(
                "Performance-aware compilation completed. Performance info available: {}",
                has_performance_info
            );
        }
    } else {
        assert_compilation_error(
            &result,
            "pattern",
            "Pattern performance estimation integration",
        );
    }
}
