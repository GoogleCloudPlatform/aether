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

//! Multi-file compilation integration tests

// Access utils from parent directory since we're in integration subdir
#[path = "../utils/mod.rs"]
mod utils;

use utils::{
    compiler_wrapper::TestCompiler,
    assertions::*,
    test_runner::TestResult,
};

#[test]
fn test_simple_two_file_project() {
    let compiler = TestCompiler::new("simple_two_file");
    
    let files = &[
        ("main.aether", r#"
(DEFINE_MODULE main
  (IMPORT_MODULE "math_lib")
  
  (DEFINE_FUNCTION
    (NAME "main")
    (RETURNS (TYPE INT))
    (BODY
      (DECLARE_VARIABLE (NAME "result")
        (INITIAL_VALUE (CALL_FUNCTION "math_lib.add" 
          (ARGUMENTS (INTEGER_LITERAL 5) (INTEGER_LITERAL 3)))))
      (CALL_FUNCTION "printf" 
        (ARGUMENTS (STRING_LITERAL "Result: %d\n") (VARIABLE_REFERENCE "result")))
      (RETURN_VALUE (INTEGER_LITERAL 0))))
)
        "#),
        ("math_lib.aether", r#"
(DEFINE_MODULE math_lib
  (EXPORT_FUNCTION "add")
  
  (DEFINE_FUNCTION
    (NAME "add")
    (ACCEPTS_PARAMETER (NAME "a") (TYPE INT))
    (ACCEPTS_PARAMETER (NAME "b") (TYPE INT))
    (RETURNS (TYPE INT))
    (INTENT "Add two integers safely")
    (PRECONDITION 
      (PREDICATE_LESS_THAN (VARIABLE_REFERENCE "a") (INTEGER_LITERAL 1000000))
      (PROOF_HINT "Input a is bounded"))
    (POSTCONDITION
      (PREDICATE_EQUALS 
        (VARIABLE_REFERENCE "RETURNED_VALUE")
        (EXPRESSION_ADD (VARIABLE_REFERENCE "a") (VARIABLE_REFERENCE "b")))
      (PROOF_HINT "Result equals sum of inputs"))
    (BODY
      (RETURN_VALUE 
        (EXPRESSION_ADD (VARIABLE_REFERENCE "a") (VARIABLE_REFERENCE "b")))))
)
        "#),
    ];
    
    let result = compiler.compile_project(files);
    assert_compile_and_execute(&result, "Result: 8", "Simple two-file project");
}

#[test]
fn test_complex_multi_module_dependency() {
    let compiler = TestCompiler::new("complex_multi_module");
    
    let files = &[
        ("main.aether", r#"
(DEFINE_MODULE main
  (IMPORT_MODULE "data_structures")
  (IMPORT_MODULE "algorithms")
  
  (DEFINE_FUNCTION
    (NAME "main")
    (RETURNS (TYPE INT))
    (BODY
      (DECLARE_VARIABLE (NAME "array")
        (INITIAL_VALUE (CALL_FUNCTION "data_structures.create_array" 
          (ARGUMENTS (INTEGER_LITERAL 5)))))
      
      (CALL_FUNCTION "data_structures.set_element"
        (ARGUMENTS (VARIABLE_REFERENCE "array") (INTEGER_LITERAL 0) (INTEGER_LITERAL 10)))
      (CALL_FUNCTION "data_structures.set_element"
        (ARGUMENTS (VARIABLE_REFERENCE "array") (INTEGER_LITERAL 1) (INTEGER_LITERAL 20)))
      (CALL_FUNCTION "data_structures.set_element"
        (ARGUMENTS (VARIABLE_REFERENCE "array") (INTEGER_LITERAL 2) (INTEGER_LITERAL 30)))
      
      (DECLARE_VARIABLE (NAME "sum")
        (INITIAL_VALUE (CALL_FUNCTION "algorithms.sum_array"
          (ARGUMENTS (VARIABLE_REFERENCE "array")))))
      
      (CALL_FUNCTION "printf" 
        (ARGUMENTS (STRING_LITERAL "Sum: %d\n") (VARIABLE_REFERENCE "sum")))
      (RETURN_VALUE (INTEGER_LITERAL 0))))
)
        "#),
        ("data_structures.aether", r#"
(DEFINE_MODULE data_structures
  (EXPORT_FUNCTION "create_array")
  (EXPORT_FUNCTION "set_element")
  (EXPORT_FUNCTION "get_element")
  
  (DEFINE_TYPE
    (NAME "Array")
    (STRUCTURED_TYPE
      (FIELD (NAME "data") (TYPE (POINTER INT)))
      (FIELD (NAME "size") (TYPE INT))
      (FIELD (NAME "capacity") (TYPE INT))))
  
  (DEFINE_FUNCTION
    (NAME "create_array")
    (ACCEPTS_PARAMETER (NAME "capacity") (TYPE INT))
    (RETURNS (TYPE (POINTER "Array")))
    (RESOURCE_SCOPE
      (ACQUIRES 
        (RESOURCE (TYPE "memory") (ID "array_memory") 
          (CLEANUP "aether_free")))
      (BODY
        (DECLARE_VARIABLE (NAME "arr")
          (INITIAL_VALUE (CALL_FUNCTION "aether_alloc" 
            (ARGUMENTS (EXPRESSION_MULTIPLY 
              (SIZEOF_TYPE "Array") (INTEGER_LITERAL 1))))))
        (ASSIGN (TARGET (FIELD_ACCESS (VARIABLE_REFERENCE "arr") "capacity"))
                (SOURCE (VARIABLE_REFERENCE "capacity")))
        (ASSIGN (TARGET (FIELD_ACCESS (VARIABLE_REFERENCE "arr") "size"))
                (SOURCE (INTEGER_LITERAL 0)))
        (ASSIGN (TARGET (FIELD_ACCESS (VARIABLE_REFERENCE "arr") "data"))
                (SOURCE (CALL_FUNCTION "aether_alloc"
                  (ARGUMENTS (EXPRESSION_MULTIPLY 
                    (SIZEOF_TYPE INT) (VARIABLE_REFERENCE "capacity"))))))
        (RETURN_VALUE (VARIABLE_REFERENCE "arr")))))
  
  (DEFINE_FUNCTION
    (NAME "set_element")
    (ACCEPTS_PARAMETER (NAME "arr") (TYPE (POINTER "Array")))
    (ACCEPTS_PARAMETER (NAME "index") (TYPE INT))
    (ACCEPTS_PARAMETER (NAME "value") (TYPE INT))
    (RETURNS (TYPE VOID))
    (PRECONDITION
      (LOGICAL_AND
        (PREDICATE_NOT_EQUALS (VARIABLE_REFERENCE "arr") (NULL_LITERAL))
        (PREDICATE_GREATER_THAN_OR_EQUAL_TO (VARIABLE_REFERENCE "index") (INTEGER_LITERAL 0))
        (PREDICATE_LESS_THAN (VARIABLE_REFERENCE "index") 
          (FIELD_ACCESS (VARIABLE_REFERENCE "arr") "capacity"))))
    (BODY
      (ASSIGN 
        (TARGET (ARRAY_ACCESS 
          (FIELD_ACCESS (VARIABLE_REFERENCE "arr") "data") 
          (VARIABLE_REFERENCE "index")))
        (SOURCE (VARIABLE_REFERENCE "value")))
      (IF_CONDITION
        (PREDICATE_GREATER_THAN_OR_EQUAL_TO (VARIABLE_REFERENCE "index")
          (FIELD_ACCESS (VARIABLE_REFERENCE "arr") "size"))
        (THEN_EXECUTE
          (ASSIGN (TARGET (FIELD_ACCESS (VARIABLE_REFERENCE "arr") "size"))
                  (SOURCE (EXPRESSION_ADD (VARIABLE_REFERENCE "index") (INTEGER_LITERAL 1))))))))
)
        "#),
        ("algorithms.aether", r#"
(DEFINE_MODULE algorithms
  (IMPORT_MODULE "data_structures")
  (EXPORT_FUNCTION "sum_array")
  (EXPORT_FUNCTION "binary_search")
  
  (DEFINE_FUNCTION
    (NAME "sum_array")
    (ACCEPTS_PARAMETER (NAME "arr") (TYPE (POINTER "data_structures.Array")))
    (RETURNS (TYPE INT))
    (INTENT "Calculate sum of all elements in array")
    (PRECONDITION
      (PREDICATE_NOT_EQUALS (VARIABLE_REFERENCE "arr") (NULL_LITERAL)))
    (POSTCONDITION
      (PREDICATE_GREATER_THAN_OR_EQUAL_TO (VARIABLE_REFERENCE "RETURNED_VALUE") (INTEGER_LITERAL 0)))
    (BODY
      (DECLARE_VARIABLE (NAME "sum") (INITIAL_VALUE (INTEGER_LITERAL 0)))
      (FOR_LOOP
        (INIT (DECLARE_VARIABLE (NAME "i") (INITIAL_VALUE (INTEGER_LITERAL 0))))
        (CONDITION (PREDICATE_LESS_THAN 
          (VARIABLE_REFERENCE "i") 
          (FIELD_ACCESS (VARIABLE_REFERENCE "arr") "size")))
        (UPDATE (ASSIGN (TARGET (VARIABLE_REFERENCE "i"))
                        (SOURCE (EXPRESSION_ADD (VARIABLE_REFERENCE "i") (INTEGER_LITERAL 1)))))
        (BODY
          (ASSIGN (TARGET (VARIABLE_REFERENCE "sum"))
                  (SOURCE (EXPRESSION_ADD 
                    (VARIABLE_REFERENCE "sum")
                    (ARRAY_ACCESS 
                      (FIELD_ACCESS (VARIABLE_REFERENCE "arr") "data")
                      (VARIABLE_REFERENCE "i")))))))
      (RETURN_VALUE (VARIABLE_REFERENCE "sum"))))
)
        "#),
    ];
    
    let result = compiler.compile_project(files);
    assert_compile_and_execute(&result, "Sum: 60", "Complex multi-module project");
}

#[test]
fn test_circular_dependency_detection() {
    let compiler = TestCompiler::new("circular_dependency");
    
    let files = &[
        ("module_a.aether", r#"
(DEFINE_MODULE module_a
  (IMPORT_MODULE "module_b")
  (EXPORT_FUNCTION "function_a")
  
  (DEFINE_FUNCTION
    (NAME "function_a")
    (RETURNS (TYPE INT))
    (BODY
      (RETURN_VALUE (CALL_FUNCTION "module_b.function_b"))))
)
        "#),
        ("module_b.aether", r#"
(DEFINE_MODULE module_b
  (IMPORT_MODULE "module_a")  
  (EXPORT_FUNCTION "function_b")
  
  (DEFINE_FUNCTION
    (NAME "function_b")
    (RETURNS (TYPE INT))
    (BODY
      (RETURN_VALUE (CALL_FUNCTION "module_a.function_a"))))
)
        "#),
    ];
    
    let result = compiler.compile_project(files);
    assert_compilation_error(&result, "circular dependency", "Circular dependency detection");
}

#[test]
fn test_standard_library_imports() {
    let compiler = TestCompiler::new("stdlib_imports");
    
    let files = &[
        ("main.aether", r#"
(DEFINE_MODULE main
  (IMPORT_MODULE "std_string")
  (IMPORT_MODULE "std_math")
  
  (DEFINE_FUNCTION
    (NAME "main")
    (RETURNS (TYPE INT))
    (BODY
      (DECLARE_VARIABLE (NAME "message")
        (INITIAL_VALUE (CALL_FUNCTION "std_string.concat"
          (ARGUMENTS (STRING_LITERAL "Hello, ") (STRING_LITERAL "World!")))))
      
      (DECLARE_VARIABLE (NAME "length")
        (INITIAL_VALUE (CALL_FUNCTION "std_string.length"
          (ARGUMENTS (VARIABLE_REFERENCE "message")))))
      
      (DECLARE_VARIABLE (NAME "sqrt_length")
        (INITIAL_VALUE (CALL_FUNCTION "std_math.sqrt"
          (ARGUMENTS (VARIABLE_REFERENCE "length")))))
      
      (CALL_FUNCTION "printf" 
        (ARGUMENTS 
          (STRING_LITERAL "Message: %s, Length: %d, Sqrt: %.2f\n") 
          (VARIABLE_REFERENCE "message")
          (VARIABLE_REFERENCE "length")
          (VARIABLE_REFERENCE "sqrt_length")))
      
      (RETURN_VALUE (INTEGER_LITERAL 0))))
)
        "#),
    ];
    
    let result = compiler.compile_project(files);
    assert_compilation_success(&result, "Standard library imports");
    
    let execution = result.execute();
    assert_execution_success(&execution, "Standard library execution");
    assert_output_contains(&execution, "Message: Hello, World!", "Standard library output");
}

#[test]
fn test_module_aliasing() {
    let compiler = TestCompiler::new("module_aliasing");
    
    let files = &[
        ("main.aether", r#"
(DEFINE_MODULE main
  (IMPORT_MODULE "very_long_module_name" (ALIAS "short"))
  
  (DEFINE_FUNCTION
    (NAME "main")
    (RETURNS (TYPE INT))
    (BODY
      (DECLARE_VARIABLE (NAME "result")
        (INITIAL_VALUE (CALL_FUNCTION "short.calculate"
          (ARGUMENTS (INTEGER_LITERAL 42)))))
      
      (CALL_FUNCTION "printf" 
        (ARGUMENTS (STRING_LITERAL "Result: %d\n") (VARIABLE_REFERENCE "result")))
      (RETURN_VALUE (INTEGER_LITERAL 0))))
)
        "#),
        ("very_long_module_name.aether", r#"
(DEFINE_MODULE very_long_module_name
  (EXPORT_FUNCTION "calculate")
  
  (DEFINE_FUNCTION
    (NAME "calculate")
    (ACCEPTS_PARAMETER (NAME "input") (TYPE INT))
    (RETURNS (TYPE INT))
    (BODY
      (RETURN_VALUE (EXPRESSION_MULTIPLY (VARIABLE_REFERENCE "input") (INTEGER_LITERAL 2)))))
)
        "#),
    ];
    
    let result = compiler.compile_project(files);
    assert_compile_and_execute(&result, "Result: 84", "Module aliasing");
}

#[test]
fn test_incremental_compilation() {
    let compiler = TestCompiler::new("incremental_compilation");
    
    // First compilation
    let files_v1 = &[
        ("main.aether", r#"
(DEFINE_MODULE main
  (IMPORT_MODULE "calculator")
  
  (DEFINE_FUNCTION
    (NAME "main")
    (RETURNS (TYPE INT))
    (BODY
      (DECLARE_VARIABLE (NAME "result")
        (INITIAL_VALUE (CALL_FUNCTION "calculator.add"
          (ARGUMENTS (INTEGER_LITERAL 10) (INTEGER_LITERAL 5)))))
      (CALL_FUNCTION "printf" 
        (ARGUMENTS (STRING_LITERAL "V1 Result: %d\n") (VARIABLE_REFERENCE "result")))
      (RETURN_VALUE (INTEGER_LITERAL 0))))
)
        "#),
        ("calculator.aether", r#"
(DEFINE_MODULE calculator
  (EXPORT_FUNCTION "add")
  
  (DEFINE_FUNCTION
    (NAME "add")
    (ACCEPTS_PARAMETER (NAME "a") (TYPE INT))
    (ACCEPTS_PARAMETER (NAME "b") (TYPE INT))
    (RETURNS (TYPE INT))
    (BODY
      (RETURN_VALUE (EXPRESSION_ADD (VARIABLE_REFERENCE "a") (VARIABLE_REFERENCE "b")))))
)
        "#),
    ];
    
    let result_v1 = compiler.compile_project(files_v1);
    assert_compile_and_execute(&result_v1, "V1 Result: 15", "First compilation");
    
    // Second compilation with modified calculator
    let files_v2 = &[
        ("main.aether", r#"
(DEFINE_MODULE main
  (IMPORT_MODULE "calculator")
  
  (DEFINE_FUNCTION
    (NAME "main")
    (RETURNS (TYPE INT))
    (BODY
      (DECLARE_VARIABLE (NAME "result")
        (INITIAL_VALUE (CALL_FUNCTION "calculator.multiply"
          (ARGUMENTS (INTEGER_LITERAL 10) (INTEGER_LITERAL 5)))))
      (CALL_FUNCTION "printf" 
        (ARGUMENTS (STRING_LITERAL "V2 Result: %d\n") (VARIABLE_REFERENCE "result")))
      (RETURN_VALUE (INTEGER_LITERAL 0))))
)
        "#),
        ("calculator.aether", r#"
(DEFINE_MODULE calculator
  (EXPORT_FUNCTION "add")
  (EXPORT_FUNCTION "multiply")
  
  (DEFINE_FUNCTION
    (NAME "add")
    (ACCEPTS_PARAMETER (NAME "a") (TYPE INT))
    (ACCEPTS_PARAMETER (NAME "b") (TYPE INT))
    (RETURNS (TYPE INT))
    (BODY
      (RETURN_VALUE (EXPRESSION_ADD (VARIABLE_REFERENCE "a") (VARIABLE_REFERENCE "b")))))
  
  (DEFINE_FUNCTION
    (NAME "multiply")
    (ACCEPTS_PARAMETER (NAME "a") (TYPE INT))
    (ACCEPTS_PARAMETER (NAME "b") (TYPE INT))
    (RETURNS (TYPE INT))
    (BODY
      (RETURN_VALUE (EXPRESSION_MULTIPLY (VARIABLE_REFERENCE "a") (VARIABLE_REFERENCE "b")))))
)
        "#),
    ];
    
    let result_v2 = compiler.compile_project(files_v2);
    assert_compile_and_execute(&result_v2, "V2 Result: 50", "Incremental compilation");
}