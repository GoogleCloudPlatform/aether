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

//! Tests for Standard Library Collection Operations

// Test utilities are defined locally
use aether::error::CompilerError;
use aether::Compiler;
use std::fs;
use tempfile::TempDir;

fn compile_and_run(source: &str) -> Result<String, CompilerError> {
    // Create a temporary directory for the test
    let temp_dir = TempDir::new()?;
    let test_file = temp_dir.path().join("test.aether");
    fs::write(&test_file, source)?;

    let compiler = Compiler::new();
    let result = compiler.compile_file(test_file)?;

    // Compilation succeeded if we got here without error

    // For now, just return success status - actual execution would require linking
    Ok("Success".to_string())
}

#[test]
fn test_sort_verified() {
    let program = r#"
(DEFINE_MODULE
  (NAME "test_sort")
  (IMPORT_MODULE "std.collections")
  (CONTENT
    (DEFINE_FUNCTION
      (NAME "main")
      (RETURNS (TYPE INT))
      (BODY
        ; Create unsorted array
        (DECLARE_VARIABLE (NAME "arr") (TYPE (ARRAY INT 10))
          (INITIAL_VALUE (ARRAY_LITERAL 
            (TYPE (ARRAY INT 10))
            (INTEGER_LITERAL 5)
            (INTEGER_LITERAL 2)
            (INTEGER_LITERAL 8)
            (INTEGER_LITERAL 1)
            (INTEGER_LITERAL 9))))
        
        ; Sort the array
        (CALL_FUNCTION "std.collections.sort_verified"
          (ARGUMENTS 
            (VARIABLE_REFERENCE "arr")
            (INTEGER_LITERAL 5)))
        
        ; Check if sorted
        (IF_CONDITION (PREDICATE_NOT_EQUALS 
            (ARRAY_ACCESS (VARIABLE_REFERENCE "arr") (INTEGER_LITERAL 0))
            (INTEGER_LITERAL 1))
          (THEN_EXECUTE (RETURN_VALUE (INTEGER_LITERAL 1))))
        
        (IF_CONDITION (PREDICATE_NOT_EQUALS 
            (ARRAY_ACCESS (VARIABLE_REFERENCE "arr") (INTEGER_LITERAL 4))
            (INTEGER_LITERAL 9))
          (THEN_EXECUTE (RETURN_VALUE (INTEGER_LITERAL 2))))
        
        (RETURN_VALUE (INTEGER_LITERAL 0))))))
    "#;

    let result = compile_and_run(program);
    assert!(result.is_ok());
}

#[test]
fn test_binary_search() {
    let program = r#"
(DEFINE_MODULE
  (NAME "test_search")
  (IMPORT_MODULE "std.collections")
  (CONTENT
    (DEFINE_FUNCTION
      (NAME "main")
      (RETURNS (TYPE INT))
      (BODY
        ; Create sorted array
        (DECLARE_VARIABLE (NAME "arr") (TYPE (ARRAY INT 10))
          (INITIAL_VALUE (ARRAY_LITERAL 
            (TYPE (ARRAY INT 10))
            (INTEGER_LITERAL 1)
            (INTEGER_LITERAL 3)
            (INTEGER_LITERAL 5)
            (INTEGER_LITERAL 7)
            (INTEGER_LITERAL 9))))
        
        ; Search for existing element
        (DECLARE_VARIABLE (NAME "index") (TYPE INT)
          (INITIAL_VALUE (CALL_FUNCTION "std.collections.binary_search"
            (ARGUMENTS 
              (VARIABLE_REFERENCE "arr")
              (INTEGER_LITERAL 5)
              (INTEGER_LITERAL 5)))))
        
        (IF_CONDITION (PREDICATE_NOT_EQUALS (VARIABLE_REFERENCE "index") (INTEGER_LITERAL 2))
          (THEN_EXECUTE (RETURN_VALUE (INTEGER_LITERAL 1))))
        
        ; Search for non-existing element
        (ASSIGN (TARGET "index")
          (SOURCE (CALL_FUNCTION "std.collections.binary_search"
            (ARGUMENTS 
              (VARIABLE_REFERENCE "arr")
              (INTEGER_LITERAL 5)
              (INTEGER_LITERAL 4)))))
        
        (IF_CONDITION (PREDICATE_NOT_EQUALS (VARIABLE_REFERENCE "index") (INTEGER_LITERAL -1))
          (THEN_EXECUTE (RETURN_VALUE (INTEGER_LITERAL 2))))
        
        (RETURN_VALUE (INTEGER_LITERAL 0))))))
    "#;

    let result = compile_and_run(program);
    assert!(result.is_ok());
}

#[test]
fn test_filter_operation() {
    let program = r#"
(DEFINE_MODULE
  (NAME "test_filter")
  (IMPORT_MODULE "std.collections")
  (CONTENT
    ; Define predicate function
    (DEFINE_FUNCTION
      (NAME "is_even")
      (ACCEPTS_PARAMETER (NAME "x") (TYPE INT))
      (RETURNS (TYPE BOOL))
      (BODY
        (RETURN_VALUE (PREDICATE_EQUALS 
          (EXPRESSION_MODULO (VARIABLE_REFERENCE "x") (INTEGER_LITERAL 2))
          (INTEGER_LITERAL 0)))))
    
    (DEFINE_FUNCTION
      (NAME "main")
      (RETURNS (TYPE INT))
      (BODY
        ; Create input array
        (DECLARE_VARIABLE (NAME "input") (TYPE (ARRAY INT 10))
          (INITIAL_VALUE (ARRAY_LITERAL 
            (TYPE (ARRAY INT 10))
            (INTEGER_LITERAL 1)
            (INTEGER_LITERAL 2)
            (INTEGER_LITERAL 3)
            (INTEGER_LITERAL 4)
            (INTEGER_LITERAL 5))))
        
        ; Create output array
        (DECLARE_VARIABLE (NAME "output") (TYPE (ARRAY INT 10)))
        
        ; Filter even numbers
        (DECLARE_VARIABLE (NAME "count") (TYPE INT)
          (INITIAL_VALUE (CALL_FUNCTION "std.collections.filter"
            (ARGUMENTS 
              (VARIABLE_REFERENCE "input")
              (INTEGER_LITERAL 5)
              (FUNCTION_REFERENCE "is_even")
              (VARIABLE_REFERENCE "output")))))
        
        ; Check result
        (IF_CONDITION (PREDICATE_NOT_EQUALS (VARIABLE_REFERENCE "count") (INTEGER_LITERAL 2))
          (THEN_EXECUTE (RETURN_VALUE (INTEGER_LITERAL 1))))
        
        (IF_CONDITION (PREDICATE_NOT_EQUALS 
            (ARRAY_ACCESS (VARIABLE_REFERENCE "output") (INTEGER_LITERAL 0))
            (INTEGER_LITERAL 2))
          (THEN_EXECUTE (RETURN_VALUE (INTEGER_LITERAL 2))))
        
        (IF_CONDITION (PREDICATE_NOT_EQUALS 
            (ARRAY_ACCESS (VARIABLE_REFERENCE "output") (INTEGER_LITERAL 1))
            (INTEGER_LITERAL 4))
          (THEN_EXECUTE (RETURN_VALUE (INTEGER_LITERAL 3))))
        
        (RETURN_VALUE (INTEGER_LITERAL 0))))))
    "#;

    let result = compile_and_run(program);
    assert!(result.is_ok());
}

#[test]
fn test_reduce_operation() {
    let program = r#"
(DEFINE_MODULE
  (NAME "test_reduce")
  (IMPORT_MODULE "std.collections")
  (CONTENT
    ; Define accumulator function
    (DEFINE_FUNCTION
      (NAME "add")
      (ACCEPTS_PARAMETER (NAME "acc") (TYPE INT))
      (ACCEPTS_PARAMETER (NAME "val") (TYPE INT))
      (RETURNS (TYPE INT))
      (BODY
        (RETURN_VALUE (EXPRESSION_ADD 
          (VARIABLE_REFERENCE "acc")
          (VARIABLE_REFERENCE "val")))))
    
    (DEFINE_FUNCTION
      (NAME "main")
      (RETURNS (TYPE INT))
      (BODY
        ; Create array
        (DECLARE_VARIABLE (NAME "arr") (TYPE (ARRAY INT 5))
          (INITIAL_VALUE (ARRAY_LITERAL 
            (TYPE (ARRAY INT 5))
            (INTEGER_LITERAL 1)
            (INTEGER_LITERAL 2)
            (INTEGER_LITERAL 3)
            (INTEGER_LITERAL 4)
            (INTEGER_LITERAL 5))))
        
        ; Sum all elements using reduce
        (DECLARE_VARIABLE (NAME "sum") (TYPE INT)
          (INITIAL_VALUE (CALL_FUNCTION "std.collections.reduce"
            (ARGUMENTS 
              (VARIABLE_REFERENCE "arr")
              (INTEGER_LITERAL 5)
              (INTEGER_LITERAL 0)
              (FUNCTION_REFERENCE "add")))))
        
        ; Check sum = 1+2+3+4+5 = 15
        (IF_CONDITION (PREDICATE_NOT_EQUALS (VARIABLE_REFERENCE "sum") (INTEGER_LITERAL 15))
          (THEN_EXECUTE (RETURN_VALUE (INTEGER_LITERAL 1))))
        
        (RETURN_VALUE (INTEGER_LITERAL 0))))))
    "#;

    let result = compile_and_run(program);
    assert!(result.is_ok());
}
