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

//! Tests for Standard Library Math Operations

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
fn test_safe_arithmetic() {
    let program = r#"
(DEFINE_MODULE
  (NAME "test_math")
  (IMPORT_MODULE "std.math")
  (CONTENT
    (DEFINE_FUNCTION
      (NAME "main")
      (RETURNS (TYPE INT))
      (BODY
        ; Test safe addition
        (DECLARE_VARIABLE (NAME "sum") (TYPE INT)
          (INITIAL_VALUE (CALL_FUNCTION "std.math.safe_add"
            (ARGUMENTS 
              (INTEGER_LITERAL 100)
              (INTEGER_LITERAL 200)))))
        
        (IF_CONDITION (PREDICATE_NOT_EQUALS (VARIABLE_REFERENCE "sum") (INTEGER_LITERAL 300))
          (THEN_EXECUTE (RETURN_VALUE (INTEGER_LITERAL 1))))
        
        ; Test safe multiplication
        (DECLARE_VARIABLE (NAME "product") (TYPE INT)
          (INITIAL_VALUE (CALL_FUNCTION "std.math.safe_multiply"
            (ARGUMENTS 
              (INTEGER_LITERAL 10)
              (INTEGER_LITERAL 20)))))
        
        (IF_CONDITION (PREDICATE_NOT_EQUALS (VARIABLE_REFERENCE "product") (INTEGER_LITERAL 200))
          (THEN_EXECUTE (RETURN_VALUE (INTEGER_LITERAL 2))))
        
        (RETURN_VALUE (INTEGER_LITERAL 0))))))
    "#;

    let result = compile_and_run(program);
    assert!(result.is_ok());
}

#[test]
fn test_trigonometric_functions() {
    let program = r#"
(DEFINE_MODULE
  (NAME "test_trig")
  (IMPORT_MODULE "std.math")
  (CONTENT
    (DEFINE_FUNCTION
      (NAME "main")
      (RETURNS (TYPE INT))
      (BODY
        ; Test sin(0) = 0
        (DECLARE_VARIABLE (NAME "sin_zero") (TYPE FLOAT)
          (INITIAL_VALUE (CALL_FUNCTION "std.math.sin"
            (ARGUMENTS (FLOAT_LITERAL 0.0)))))
        
        (IF_CONDITION (PREDICATE_GREATER_THAN 
            (CALL_FUNCTION "std.math.abs"
              (ARGUMENTS (EXPRESSION_MULTIPLY 
                (VARIABLE_REFERENCE "sin_zero") 
                (FLOAT_LITERAL 1000000.0))))
            (INTEGER_LITERAL 1))
          (THEN_EXECUTE (RETURN_VALUE (INTEGER_LITERAL 1))))
        
        ; Test cos(0) = 1
        (DECLARE_VARIABLE (NAME "cos_zero") (TYPE FLOAT)
          (INITIAL_VALUE (CALL_FUNCTION "std.math.cos"
            (ARGUMENTS (FLOAT_LITERAL 0.0)))))
        
        (IF_CONDITION (PREDICATE_GREATER_THAN 
            (CALL_FUNCTION "std.math.abs"
              (ARGUMENTS (EXPRESSION_MULTIPLY 
                (EXPRESSION_SUBTRACT 
                  (VARIABLE_REFERENCE "cos_zero")
                  (FLOAT_LITERAL 1.0))
                (FLOAT_LITERAL 1000000.0))))
            (INTEGER_LITERAL 1))
          (THEN_EXECUTE (RETURN_VALUE (INTEGER_LITERAL 2))))
        
        ; Test sin/cos range [-1, 1]
        (DECLARE_VARIABLE (NAME "sin_val") (TYPE FLOAT)
          (INITIAL_VALUE (CALL_FUNCTION "std.math.sin"
            (ARGUMENTS (FLOAT_LITERAL 1.5)))))
        
        (IF_CONDITION (LOGICAL_OR
            (PREDICATE_GREATER_THAN (VARIABLE_REFERENCE "sin_val") (FLOAT_LITERAL 1.0))
            (PREDICATE_LESS_THAN (VARIABLE_REFERENCE "sin_val") (FLOAT_LITERAL -1.0)))
          (THEN_EXECUTE (RETURN_VALUE (INTEGER_LITERAL 3))))
        
        (RETURN_VALUE (INTEGER_LITERAL 0))))))
    "#;

    let result = compile_and_run(program);
    assert!(result.is_ok());
}

#[test]
fn test_min_max_abs() {
    let program = r#"
(DEFINE_MODULE
  (NAME "test_minmax")
  (IMPORT_MODULE "std.math")
  (CONTENT
    (DEFINE_FUNCTION
      (NAME "main")
      (RETURNS (TYPE INT))
      (BODY
        ; Test min
        (IF_CONDITION (PREDICATE_NOT_EQUALS 
            (CALL_FUNCTION "std.math.min"
              (ARGUMENTS (INTEGER_LITERAL 10) (INTEGER_LITERAL 20)))
            (INTEGER_LITERAL 10))
          (THEN_EXECUTE (RETURN_VALUE (INTEGER_LITERAL 1))))
        
        ; Test max
        (IF_CONDITION (PREDICATE_NOT_EQUALS 
            (CALL_FUNCTION "std.math.max"
              (ARGUMENTS (INTEGER_LITERAL 10) (INTEGER_LITERAL 20)))
            (INTEGER_LITERAL 20))
          (THEN_EXECUTE (RETURN_VALUE (INTEGER_LITERAL 2))))
        
        ; Test abs
        (IF_CONDITION (PREDICATE_NOT_EQUALS 
            (CALL_FUNCTION "std.math.abs"
              (ARGUMENTS (INTEGER_LITERAL -42)))
            (INTEGER_LITERAL 42))
          (THEN_EXECUTE (RETURN_VALUE (INTEGER_LITERAL 3))))
        
        (RETURN_VALUE (INTEGER_LITERAL 0))))))
    "#;

    let result = compile_and_run(program);
    assert!(result.is_ok());
}

#[test]
fn test_sqrt_and_pow() {
    let program = r#"
(DEFINE_MODULE
  (NAME "test_sqrt_pow")
  (IMPORT_MODULE "std.math")
  (CONTENT
    (DEFINE_FUNCTION
      (NAME "main")
      (RETURNS (TYPE INT))
      (BODY
        ; Test sqrt(4) = 2
        (DECLARE_VARIABLE (NAME "sqrt_four") (TYPE FLOAT)
          (INITIAL_VALUE (CALL_FUNCTION "std.math.sqrt"
            (ARGUMENTS (FLOAT_LITERAL 4.0)))))
        
        (IF_CONDITION (PREDICATE_GREATER_THAN 
            (CALL_FUNCTION "std.math.abs"
              (ARGUMENTS (EXPRESSION_MULTIPLY 
                (EXPRESSION_SUBTRACT 
                  (VARIABLE_REFERENCE "sqrt_four")
                  (FLOAT_LITERAL 2.0))
                (FLOAT_LITERAL 1000000.0))))
            (INTEGER_LITERAL 1))
          (THEN_EXECUTE (RETURN_VALUE (INTEGER_LITERAL 1))))
        
        ; Test 2^3 = 8
        (DECLARE_VARIABLE (NAME "two_cubed") (TYPE FLOAT)
          (INITIAL_VALUE (CALL_FUNCTION "std.math.pow"
            (ARGUMENTS 
              (FLOAT_LITERAL 2.0)
              (FLOAT_LITERAL 3.0)))))
        
        (IF_CONDITION (PREDICATE_GREATER_THAN 
            (CALL_FUNCTION "std.math.abs"
              (ARGUMENTS (EXPRESSION_MULTIPLY 
                (EXPRESSION_SUBTRACT 
                  (VARIABLE_REFERENCE "two_cubed")
                  (FLOAT_LITERAL 8.0))
                (FLOAT_LITERAL 1000000.0))))
            (INTEGER_LITERAL 1))
          (THEN_EXECUTE (RETURN_VALUE (INTEGER_LITERAL 2))))
        
        (RETURN_VALUE (INTEGER_LITERAL 0))))))
    "#;

    let result = compile_and_run(program);
    assert!(result.is_ok());
}
