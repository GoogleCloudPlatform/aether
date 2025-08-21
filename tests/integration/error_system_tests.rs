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

//! LLM-optimized error system integration tests (Phase 3)

// Access utils from parent directory since we're in integration subdir
#[path = "../utils/mod.rs"]
mod utils;

use utils::{
    compiler_wrapper::TestCompiler,
    assertions::*,
};

#[test]
fn test_structured_error_format() {
    let compiler = TestCompiler::new("structured_errors");
    
    let source = r#"
(DEFINE_MODULE structured_errors
  (DEFINE_FUNCTION
    (NAME "type_mismatch_function")
    (ACCEPTS_PARAMETER (NAME "number") (TYPE INT))
    (RETURNS (TYPE STRING))
    (BODY
      // This should cause a type error - returning INT instead of STRING
      (RETURN_VALUE (VARIABLE_REFERENCE "number"))))
)
    "#;
    
    let result = compiler.compile_source(source, "structured_errors.aether");
    assert_compilation_failure(&result, "Structured error generation");
    
    // Check that error contains structured information
    if let Some(error) = result.error() {
        let error_msg = format!("{}", error);
        
        // Should contain error code
        assert!(error_msg.contains("TYPE-") || error_msg.contains("SEM-"), 
                "Error should contain structured error code");
        
        // Should contain location information
        assert!(error_msg.contains("line") || error_msg.contains("column"), 
                "Error should contain location information");
        
        // Should contain fix suggestion
        assert!(error_msg.contains("suggestion") || error_msg.contains("fix") || error_msg.contains("try"), 
                "Error should contain fix suggestion");
    }
}

#[test]
fn test_auto_fix_suggestions() {
    let compiler = TestCompiler::new("auto_fix");
    
    let source = r#"
(DEFINE_MODULE auto_fix
  (DEFINE_FUNCTION
    (NAME "undefined_variable_function")
    (RETURNS (TYPE INT))
    (BODY
      // Using undefined variable 'result' - should suggest declaration
      (RETURN_VALUE (VARIABLE_REFERENCE "result"))))
)
    "#;
    
    let result = compiler.compile_source(source, "auto_fix.aether");
    assert_compilation_failure(&result, "Auto-fix suggestion generation");
    
    if let Some(error) = result.error() {
        let error_msg = format!("{}", error);
        
        // Should suggest variable declaration
        assert!(error_msg.to_lowercase().contains("declare") || 
                error_msg.to_lowercase().contains("define") ||
                error_msg.to_lowercase().contains("variable"),
                "Error should suggest variable declaration: {}", error_msg);
        
        // Should provide example fix
        assert!(error_msg.contains("DECLARE_VARIABLE") || 
                error_msg.contains("example") ||
                error_msg.contains("try:"),
                "Error should provide example fix: {}", error_msg);
    }
}

#[test]
fn test_partial_compilation_success() {
    let compiler = TestCompiler::new("partial_compilation");
    
    let files = &[
        ("main.aether", r#"
(DEFINE_MODULE main
  (IMPORT_MODULE "working_module")
  (IMPORT_MODULE "broken_module")  // This module has errors
  
  (DEFINE_FUNCTION
    (NAME "main")
    (RETURNS (TYPE INT))
    (BODY
      // Use only the working module
      (DECLARE_VARIABLE (NAME "result")
        (INITIAL_VALUE (CALL_FUNCTION "working_module.calculate"
          (ARGUMENTS (INTEGER_LITERAL 42)))))
      (CALL_FUNCTION "printf" 
        (ARGUMENTS (STRING_LITERAL "Result: %d\n") (VARIABLE_REFERENCE "result")))
      (RETURN_VALUE (INTEGER_LITERAL 0))))
)
        "#),
        ("working_module.aether", r#"
(DEFINE_MODULE working_module
  (EXPORT_FUNCTION "calculate")
  
  (DEFINE_FUNCTION
    (NAME "calculate")
    (ACCEPTS_PARAMETER (NAME "input") (TYPE INT))
    (RETURNS (TYPE INT))
    (BODY
      (RETURN_VALUE (EXPRESSION_MULTIPLY (VARIABLE_REFERENCE "input") (INTEGER_LITERAL 2)))))
)
        "#),
        ("broken_module.aether", r#"
(DEFINE_MODULE broken_module
  (EXPORT_FUNCTION "broken_function")
  
  (DEFINE_FUNCTION
    (NAME "broken_function")
    (RETURNS (TYPE INT))
    (BODY
      // Type error: returning string instead of int
      (RETURN_VALUE (STRING_LITERAL "error"))))
)
        "#),
    ];
    
    let result = compiler.compile_project(files);
    
    // Should either succeed with warnings or fail with partial compilation info
    if result.is_success() {
        // If compilation succeeds, should have warnings about broken module
        assert_warning_contains(&result, "broken_module", "Partial compilation warning");
        
        // Should still be able to execute using working parts
        let execution = result.execute();
        assert_execution_success(&execution, "Partial execution");
        assert_output_contains(&execution, "Result: 84", "Partial execution output");
    } else {
        // If compilation fails, error should indicate partial compilation possibility
        if let Some(error) = result.error() {
            let error_msg = format!("{}", error);
            assert!(error_msg.contains("partial") || 
                    error_msg.contains("working") ||
                    error_msg.contains("successful"),
                    "Error should mention partial compilation: {}", error_msg);
        }
    }
}

#[test]
fn test_llm_friendly_error_messages() {
    let compiler = TestCompiler::new("llm_friendly");
    
    let source = r#"
(DEFINE_MODULE llm_friendly
  (DEFINE_FUNCTION
    (NAME "complex_error_function")
    (ACCEPTS_PARAMETER (NAME "data") (TYPE (ARRAY INT 10)))
    (RETURNS (TYPE STRING))
    (BODY
      // Multiple errors to test LLM-friendly reporting
      (DECLARE_VARIABLE (NAME "index") (INITIAL_VALUE (STRING_LITERAL "not_a_number"))) // Type error
      (DECLARE_VARIABLE (NAME "element") 
        (INITIAL_VALUE (ARRAY_ACCESS (VARIABLE_REFERENCE "data") (VARIABLE_REFERENCE "index")))) // Type error in array access
      (RETURN_VALUE (VARIABLE_REFERENCE "undefined_var")))) // Undefined variable
)
    "#;
    
    let result = compiler.compile_source(source, "llm_friendly.aether");
    assert_compilation_failure(&result, "LLM-friendly error generation");
    
    if let Some(error) = result.error() {
        let error_msg = format!("{}", error);
        
        // Should explain the problem clearly
        assert!(error_msg.contains("type") || error_msg.contains("TYPE"),
                "Error should mention type issues: {}", error_msg);
        
        // Should provide context
        assert!(error_msg.contains("array") || error_msg.contains("ARRAY"),
                "Error should provide context about array access: {}", error_msg);
        
        // Should be structured for LLM consumption
        assert!(error_msg.contains("{") || error_msg.contains("Error:") || error_msg.contains("Code:"),
                "Error should be structured: {}", error_msg);
    }
}

#[test]
fn test_intent_mismatch_error_reporting() {
    let compiler = TestCompiler::new("intent_mismatch_error");
    
    let source = r#"
(DEFINE_MODULE intent_mismatch_error
  (DEFINE_FUNCTION
    (NAME "sort_array")
    (ACCEPTS_PARAMETER (NAME "array") (TYPE (ARRAY INT 10)))
    (RETURNS (TYPE (ARRAY INT 10)))
    (INTENT "Sort array in ascending order")
    (BODY
      // Function claims to sort but actually reverses - intent mismatch
      (DECLARE_VARIABLE (NAME "result") (INITIAL_VALUE (VARIABLE_REFERENCE "array")))
      (FOR_LOOP
        (INIT (DECLARE_VARIABLE (NAME "i") (INITIAL_VALUE (INTEGER_LITERAL 0))))
        (CONDITION (PREDICATE_LESS_THAN (VARIABLE_REFERENCE "i") (INTEGER_LITERAL 5)))
        (UPDATE (ASSIGN (TARGET (VARIABLE_REFERENCE "i"))
                        (SOURCE (EXPRESSION_ADD (VARIABLE_REFERENCE "i") (INTEGER_LITERAL 1)))))
        (BODY
          // Swap elements to reverse instead of sort
          (DECLARE_VARIABLE (NAME "temp")
            (INITIAL_VALUE (ARRAY_ACCESS (VARIABLE_REFERENCE "result") (VARIABLE_REFERENCE "i"))))
          (ASSIGN (TARGET (ARRAY_ACCESS (VARIABLE_REFERENCE "result") (VARIABLE_REFERENCE "i")))
                  (SOURCE (ARRAY_ACCESS (VARIABLE_REFERENCE "result") 
                    (EXPRESSION_SUBTRACT (INTEGER_LITERAL 9) (VARIABLE_REFERENCE "i")))))
          (ASSIGN (TARGET (ARRAY_ACCESS (VARIABLE_REFERENCE "result") 
                    (EXPRESSION_SUBTRACT (INTEGER_LITERAL 9) (VARIABLE_REFERENCE "i"))))
                  (SOURCE (VARIABLE_REFERENCE "temp")))))
      (RETURN_VALUE (VARIABLE_REFERENCE "result"))))
)
    "#;
    
    let result = compiler.compile_source(source, "intent_mismatch_error.aether");
    
    // Should compile but warn about intent mismatch
    if result.is_success() {
        assert_warning_contains(&result, "intent", "Intent mismatch warning");
    }
    
    // Error/warning should be LLM-friendly
    let has_llm_friendly_message = if let Some(_compilation_result) = result.success() {
        // Note: Current CompilationResult doesn't track warnings
        // This check is currently disabled
        false
    } else {
        if let Some(error) = result.error() {
            let error_msg = format!("{}", error);
            error_msg.contains("intent") && 
            (error_msg.contains("mismatch") || error_msg.contains("differs") || error_msg.contains("expected"))
        } else {
            false
        }
    };
    
    assert!(has_llm_friendly_message, "Should have LLM-friendly intent mismatch message");
}

#[test]
fn test_error_recovery_and_continuation() {
    let compiler = TestCompiler::new("error_recovery");
    
    let source = r#"
(DEFINE_MODULE error_recovery
  // First function has error
  (DEFINE_FUNCTION
    (NAME "broken_function")
    (RETURNS (TYPE INT))
    (BODY
      (RETURN_VALUE (UNDEFINED_IDENTIFIER "this_will_cause_error"))))
  
  // Second function is correct
  (DEFINE_FUNCTION
    (NAME "working_function")
    (ACCEPTS_PARAMETER (NAME "x") (TYPE INT))
    (RETURNS (TYPE INT))
    (BODY
      (RETURN_VALUE (EXPRESSION_MULTIPLY (VARIABLE_REFERENCE "x") (INTEGER_LITERAL 2)))))
  
  // Third function is also correct
  (DEFINE_FUNCTION
    (NAME "another_working_function")
    (ACCEPTS_PARAMETER (NAME "a") (TYPE INT))
    (ACCEPTS_PARAMETER (NAME "b") (TYPE INT))
    (RETURNS (TYPE INT))
    (BODY
      (RETURN_VALUE (EXPRESSION_ADD (VARIABLE_REFERENCE "a") (VARIABLE_REFERENCE "b")))))
)
    "#;
    
    let result = compiler.compile_source(source, "error_recovery.aether");
    assert_compilation_failure(&result, "Error recovery test");
    
    if let Some(error) = result.error() {
        let error_msg = format!("{}", error);
        
        // Error should mention recovery or continuation
        let mentions_recovery = error_msg.contains("continue") || 
                               error_msg.contains("recovery") ||
                               error_msg.contains("other functions") ||
                               error_msg.contains("remaining");
                               
        // Error should indicate how many functions were processed
        let mentions_progress = error_msg.contains("function") && 
                               (error_msg.contains("1") || error_msg.contains("2") || error_msg.contains("3"));
        
        assert!(mentions_recovery || mentions_progress,
                "Error should mention recovery or progress: {}", error_msg);
    }
}

#[test]
fn test_cascading_error_prevention() {
    let compiler = TestCompiler::new("cascading_errors");
    
    let source = r#"
(DEFINE_MODULE cascading_errors
  (DEFINE_FUNCTION
    (NAME "function_with_cascading_errors")
    (RETURNS (TYPE INT))
    (BODY
      // Primary error: undefined variable
      (DECLARE_VARIABLE (NAME "result") 
        (INITIAL_VALUE (VARIABLE_REFERENCE "undefined_var")))
      
      // These would cause secondary errors due to the first error
      (DECLARE_VARIABLE (NAME "doubled")
        (INITIAL_VALUE (EXPRESSION_MULTIPLY (VARIABLE_REFERENCE "result") (INTEGER_LITERAL 2))))
      
      (DECLARE_VARIABLE (NAME "tripled")
        (INITIAL_VALUE (EXPRESSION_MULTIPLY (VARIABLE_REFERENCE "result") (INTEGER_LITERAL 3))))
      
      (RETURN_VALUE (EXPRESSION_ADD (VARIABLE_REFERENCE "doubled") (VARIABLE_REFERENCE "tripled")))))
)
    "#;
    
    let result = compiler.compile_source(source, "cascading_errors.aether");
    assert_compilation_failure(&result, "Cascading error prevention");
    
    if let Some(error) = result.error() {
        let error_msg = format!("{}", error);
        
        // Should focus on the primary error, not flood with secondary errors
        let error_count = error_msg.matches("error").count() + error_msg.matches("Error").count();
        
        // Should not have too many error messages (indicating good error recovery)
        assert!(error_count <= 5, 
                "Should not have excessive error messages (found {}): {}", error_count, error_msg);
        
        // Should mention the primary issue
        assert!(error_msg.contains("undefined") || error_msg.contains("undefined_var"),
                "Should identify primary error: {}", error_msg);
    }
}

#[test]
fn test_contextual_error_information() {
    let compiler = TestCompiler::new("contextual_errors");
    
    let source = r#"
(DEFINE_MODULE contextual_errors
  (DEFINE_FUNCTION
    (NAME "function_with_context")
    (ACCEPTS_PARAMETER (NAME "user_input") (TYPE STRING))
    (RETURNS (TYPE INT))
    (INTENT "Convert user input string to integer")
    (BODY
      // Error: trying to directly return string as int
      (RETURN_VALUE (VARIABLE_REFERENCE "user_input"))))
)
    "#;
    
    let result = compiler.compile_source(source, "contextual_errors.aether");
    assert_compilation_failure(&result, "Contextual error information");
    
    if let Some(error) = result.error() {
        let error_msg = format!("{}", error);
        
        // Should provide context about the intended operation
        assert!(error_msg.contains("string") && error_msg.contains("int"),
                "Error should mention type conflict: {}", error_msg);
        
        // Should reference the function's intent
        assert!(error_msg.contains("intent") || error_msg.contains("convert"),
                "Error should reference function intent: {}", error_msg);
        
        // Should suggest conversion
        assert!(error_msg.contains("convert") || error_msg.contains("parse") || error_msg.contains("cast"),
                "Error should suggest conversion: {}", error_msg);
    }
}