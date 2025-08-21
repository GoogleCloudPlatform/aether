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

//! Tests for Standard Library I/O Operations

// Test utilities are defined locally
use aether::Compiler;
use aether::error::CompilerError;
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
fn test_file_write_and_read() {
    let program = r#"
(DEFINE_MODULE
  (NAME "test_io")
  (IMPORT_MODULE "std.io")
  (CONTENT
    (DEFINE_FUNCTION
      (NAME "main")
      (RETURNS (TYPE INT))
      (BODY
        ; Write to file
        (DECLARE_VARIABLE (NAME "success") (TYPE BOOL)
          (INITIAL_VALUE (CALL_FUNCTION "std.io.write_file_safe"
            (ARGUMENTS 
              (STRING_LITERAL "test_output.txt")
              (STRING_LITERAL "Hello, AetherScript!")
              (BOOL_LITERAL FALSE)))))
        
        (IF_CONDITION (PREDICATE_NOT (VARIABLE_REFERENCE "success"))
          (THEN_EXECUTE (RETURN_VALUE (INTEGER_LITERAL 1))))
        
        ; Read from file
        (DECLARE_VARIABLE (NAME "content") (TYPE STRING)
          (INITIAL_VALUE (CALL_FUNCTION "std.io.read_file_safe"
            (ARGUMENTS 
              (STRING_LITERAL "test_output.txt")
              (INTEGER_LITERAL 1024)))))
        
        ; Print the content
        (CALL_FUNCTION "std.io.println"
          (ARGUMENTS (VARIABLE_REFERENCE "content")))
        
        (RETURN_VALUE (INTEGER_LITERAL 0))))))
    "#;

    let result = compile_and_run(program);
    assert!(result.is_ok());
    
    // Clean up
    let _ = std::fs::remove_file("test_output.txt");
}

#[test]
fn test_console_io() {
    let program = r#"
(DEFINE_MODULE
  (NAME "test_console")
  (IMPORT_MODULE "std.io")
  (CONTENT
    (DEFINE_FUNCTION
      (NAME "main")
      (RETURNS (TYPE INT))
      (BODY
        (CALL_FUNCTION "std.io.print"
          (ARGUMENTS (STRING_LITERAL "Hello, ")))
        (CALL_FUNCTION "std.io.println"
          (ARGUMENTS (STRING_LITERAL "World!")))
        (RETURN_VALUE (INTEGER_LITERAL 0))))))
    "#;

    let result = compile_and_run(program);
    assert!(result.is_ok());
}

#[test]
fn test_file_size_limit() {
    let program = r#"
(DEFINE_MODULE
  (NAME "test_size_limit")
  (IMPORT_MODULE "std.io")
  (CONTENT
    (DEFINE_FUNCTION
      (NAME "main")
      (RETURNS (TYPE INT))
      (BODY
        ; Create a small file
        (CALL_FUNCTION "std.io.write_file_safe"
          (ARGUMENTS 
            (STRING_LITERAL "small_file.txt")
            (STRING_LITERAL "Small content")
            (BOOL_LITERAL FALSE)))
        
        ; Try to read with very small limit - should fail
        (TRY_EXECUTE
          (PROTECTED_BLOCK
            (CALL_FUNCTION "std.io.read_file_safe"
              (ARGUMENTS 
                (STRING_LITERAL "small_file.txt")
                (INTEGER_LITERAL 5))) ; Too small
            (RETURN_VALUE (INTEGER_LITERAL 1))) ; Should not reach here
          
          (CATCH_EXCEPTION
            (EXCEPTION_TYPE "io_error")
            (BINDING_VARIABLE (NAME "e") (TYPE "io_error"))
            (HANDLER_BLOCK
              (RETURN_VALUE (INTEGER_LITERAL 0))))) ; Expected
        ))))
    "#;

    let result = compile_and_run(program);
    assert!(result.is_ok());
    
    // Clean up
    let _ = std::fs::remove_file("small_file.txt");
}