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

//! Integration tests for variadic function support

use std::fs;
use std::path::PathBuf;
use std::process::Command;

#[test]
fn test_printf_variadic() {
    let test_program = r#"
(DEFINE_MODULE
  (NAME test_printf)
  (INTENT "Test printf variadic function")
  
  (CONTENT
    (DECLARE_EXTERNAL_FUNCTION
      (NAME printf)
      (LIBRARY "libc")
      (RETURNS INTEGER)
      (PARAM (NAME "format") (TYPE STRING))
      (CALLING_CONVENTION "C")
      (VARIADIC true))
    
    (DEFINE_FUNCTION
      (NAME main)
      (RETURNS INTEGER)
      (BODY
        (CALL_FUNCTION printf "Hello %s! The answer is %d\n" "World" 42)
        (RETURN_VALUE 0)
      )
    )
  )
)
"#;

    // Write test program
    let test_file = PathBuf::from("test_printf_variadic.aether");
    fs::write(&test_file, test_program).expect("Failed to write test file");
    
    // Compile the program
    let output = Command::new("target/release/aether-compiler")
        .arg("compile")
        .arg(&test_file)
        .arg("-o")
        .arg("test_printf_variadic")
        .output()
        .expect("Failed to run compiler");
    
    if !output.status.success() {
        panic!("Compilation failed: {}", String::from_utf8_lossy(&output.stderr));
    }
    
    // Run the program
    let run_output = Command::new("./test_printf_variadic")
        .output()
        .expect("Failed to run compiled program");
    
    assert!(run_output.status.success());
    let stdout = String::from_utf8_lossy(&run_output.stdout);
    assert!(stdout.contains("Hello World! The answer is 42"));
    
    // Clean up
    fs::remove_file(test_file).ok();
    fs::remove_file("test_printf_variadic").ok();
    fs::remove_file("test_printf_variadic.o").ok();
}

#[test]
fn test_multiple_variadic_functions() {
    let test_program = r#"
(DEFINE_MODULE
  (NAME test_multiple_variadic)
  (INTENT "Test multiple variadic functions")
  
  (CONTENT
    (DECLARE_EXTERNAL_FUNCTION
      (NAME printf)
      (LIBRARY "libc")
      (RETURNS INTEGER)
      (PARAM (NAME "format") (TYPE STRING))
      (CALLING_CONVENTION "C")
      (VARIADIC true))
    
    (DECLARE_EXTERNAL_FUNCTION
      (NAME sprintf)
      (LIBRARY "libc")
      (RETURNS INTEGER)
      (PARAM (NAME "buffer") (TYPE STRING))
      (PARAM (NAME "format") (TYPE STRING))
      (CALLING_CONVENTION "C")
      (VARIADIC true))
    
    (DEFINE_FUNCTION
      (NAME main)
      (RETURNS INTEGER)
      (BODY
        (CALL_FUNCTION printf "Testing %s %d %f\n" "variadic" 123 3.14)
        (RETURN_VALUE 0)
      )
    )
  )
)
"#;

    // Write test program
    let test_file = PathBuf::from("test_multiple_variadic.aether");
    fs::write(&test_file, test_program).expect("Failed to write test file");
    
    // Compile the program
    let output = Command::new("target/release/aether-compiler")
        .arg("compile")
        .arg(&test_file)
        .arg("-o")
        .arg("test_multiple_variadic")
        .output()
        .expect("Failed to run compiler");
    
    if !output.status.success() {
        panic!("Compilation failed: {}", String::from_utf8_lossy(&output.stderr));
    }
    
    // Run the program
    let run_output = Command::new("./test_multiple_variadic")
        .output()
        .expect("Failed to run compiled program");
    
    assert!(run_output.status.success());
    let stdout = String::from_utf8_lossy(&run_output.stdout);
    assert!(stdout.contains("Testing variadic 123"));
    
    // Clean up
    fs::remove_file(test_file).ok();
    fs::remove_file("test_multiple_variadic").ok();
    fs::remove_file("test_multiple_variadic.o").ok();
}

#[test] 
fn test_non_variadic_external_function() {
    let test_program = r#"
(DEFINE_MODULE
  (NAME test_non_variadic)
  (INTENT "Test non-variadic external function")
  
  (CONTENT
    (DECLARE_EXTERNAL_FUNCTION
      (NAME puts)
      (LIBRARY "libc")
      (RETURNS INTEGER)
      (PARAM (NAME "str") (TYPE STRING))
      (CALLING_CONVENTION "C")
      (VARIADIC false))
    
    (DEFINE_FUNCTION
      (NAME main)
      (RETURNS INTEGER)
      (BODY
        (CALL_FUNCTION puts "Hello from non-variadic function!")
        (RETURN_VALUE 0)
      )
    )
  )
)
"#;

    // Write test program
    let test_file = PathBuf::from("test_non_variadic.aether");
    fs::write(&test_file, test_program).expect("Failed to write test file");
    
    // Compile the program
    let output = Command::new("target/release/aether-compiler")
        .arg("compile")
        .arg(&test_file)
        .arg("-o")
        .arg("test_non_variadic")
        .output()
        .expect("Failed to run compiler");
    
    if !output.status.success() {
        panic!("Compilation failed: {}", String::from_utf8_lossy(&output.stderr));
    }
    
    // Run the program
    let run_output = Command::new("./test_non_variadic")
        .output()
        .expect("Failed to run compiled program");
    
    assert!(run_output.status.success());
    let stdout = String::from_utf8_lossy(&run_output.stdout);
    assert!(stdout.contains("Hello from non-variadic function!"));
    
    // Clean up
    fs::remove_file(test_file).ok();
    fs::remove_file("test_non_variadic").ok();
    fs::remove_file("test_non_variadic.o").ok();
}