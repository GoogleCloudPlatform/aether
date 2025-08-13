//! Integration tests for string runtime functions

use std::fs;
use std::path::PathBuf;
use std::process::Command;

#[test]
fn test_string_runtime_functions() {
    let test_program = r#"
(DEFINE_MODULE
  (NAME test_string_runtime)
  (INTENT "Test runtime string functions")
  
  (CONTENT
    ; External function declarations
    (DECLARE_EXTERNAL_FUNCTION
      (NAME printf)
      (LIBRARY "libc")
      (RETURNS INTEGER)
      (PARAM (NAME "format") (TYPE STRING))
      (CALLING_CONVENTION "C")
      (VARIADIC true))
    
    (DECLARE_EXTERNAL_FUNCTION
      (NAME string_index_of)
      (LIBRARY "aether_runtime")
      (RETURNS INTEGER)
      (PARAM (NAME "haystack") (TYPE STRING))
      (PARAM (NAME "needle") (TYPE STRING)))
    
    (DECLARE_EXTERNAL_FUNCTION
      (NAME string_starts_with)
      (LIBRARY "aether_runtime")
      (RETURNS INTEGER)
      (PARAM (NAME "str") (TYPE STRING))
      (PARAM (NAME "prefix") (TYPE STRING)))
    
    (DECLARE_EXTERNAL_FUNCTION
      (NAME string_ends_with)
      (LIBRARY "aether_runtime")
      (RETURNS INTEGER)
      (PARAM (NAME "str") (TYPE STRING))
      (PARAM (NAME "suffix") (TYPE STRING)))
    
    (DECLARE_EXTERNAL_FUNCTION
      (NAME parse_float)
      (LIBRARY "aether_runtime")
      (RETURNS FLOAT)
      (PARAM (NAME "str") (TYPE STRING)))
    
    (DECLARE_EXTERNAL_FUNCTION
      (NAME float_to_string)
      (LIBRARY "aether_runtime")
      (RETURNS STRING)
      (PARAM (NAME "value") (TYPE FLOAT)))
    
    (DEFINE_FUNCTION
      (NAME main)
      (RETURNS INTEGER)
      (BODY
        ; Test string_index_of
        (DECLARE_VARIABLE (NAME test_str) (TYPE STRING))
        (ASSIGN (TARGET_VARIABLE test_str) (SOURCE_EXPRESSION "Hello, World!"))
        
        (DECLARE_VARIABLE (NAME index) (TYPE INTEGER))
        (ASSIGN (TARGET_VARIABLE index) 
          (SOURCE_EXPRESSION (CALL_FUNCTION string_index_of test_str "World")))
        
        ; Expected: 7
        (IF_CONDITION
          (PREDICATE_NOT_EQUALS index 7)
          (THEN_EXECUTE
            (CALL_FUNCTION printf "FAIL: string_index_of returned %d, expected 7\n" index)
            (RETURN_VALUE 1)
          )
        )
        
        ; Test string_starts_with
        (DECLARE_VARIABLE (NAME starts) (TYPE INTEGER))
        (ASSIGN (TARGET_VARIABLE starts)
          (SOURCE_EXPRESSION (CALL_FUNCTION string_starts_with test_str "Hello")))
        
        ; Expected: 1 (true)
        (IF_CONDITION
          (PREDICATE_NOT_EQUALS starts 1)
          (THEN_EXECUTE
            (CALL_FUNCTION printf "FAIL: string_starts_with returned %d, expected 1\n" starts)
            (RETURN_VALUE 1)
          )
        )
        
        ; Test string_ends_with
        (DECLARE_VARIABLE (NAME ends) (TYPE INTEGER))
        (ASSIGN (TARGET_VARIABLE ends)
          (SOURCE_EXPRESSION (CALL_FUNCTION string_ends_with test_str "World!")))
        
        ; Expected: 1 (true)
        (IF_CONDITION
          (PREDICATE_NOT_EQUALS ends 1)
          (THEN_EXECUTE
            (CALL_FUNCTION printf "FAIL: string_ends_with returned %d, expected 1\n" ends)
            (RETURN_VALUE 1)
          )
        )
        
        ; Test parse_float
        (DECLARE_VARIABLE (NAME float_str) (TYPE STRING))
        (ASSIGN (TARGET_VARIABLE float_str) (SOURCE_EXPRESSION "3.14"))
        
        (DECLARE_VARIABLE (NAME parsed) (TYPE FLOAT))
        (ASSIGN (TARGET_VARIABLE parsed)
          (SOURCE_EXPRESSION (CALL_FUNCTION parse_float float_str)))
        
        ; Test float_to_string
        (DECLARE_VARIABLE (NAME float_back) (TYPE STRING))
        (ASSIGN (TARGET_VARIABLE float_back)
          (SOURCE_EXPRESSION (CALL_FUNCTION float_to_string parsed)))
        
        (CALL_FUNCTION printf "All string runtime tests passed!\n")
        (RETURN_VALUE 0)
      )
    )
  )
)
"#;

    // Write test program
    let test_file = PathBuf::from("test_string_runtime.aether");
    fs::write(&test_file, test_program).expect("Failed to write test file");
    
    // Compile the program
    let output = Command::new("target/release/aether-compiler")
        .arg("compile")
        .arg(&test_file)
        .arg("-o")
        .arg("test_string_runtime")
        .output()
        .expect("Failed to run compiler");
    
    if !output.status.success() {
        panic!("Compilation failed: {}", String::from_utf8_lossy(&output.stderr));
    }
    
    // Run the program
    let run_output = Command::new("./test_string_runtime")
        .output()
        .expect("Failed to run compiled program");
    
    assert!(run_output.status.success(), "Test program failed");
    let stdout = String::from_utf8_lossy(&run_output.stdout);
    assert!(stdout.contains("All string runtime tests passed!"));
    
    // Clean up
    fs::remove_file(test_file).ok();
    fs::remove_file("test_string_runtime").ok();
    fs::remove_file("test_string_runtime.o").ok();
}