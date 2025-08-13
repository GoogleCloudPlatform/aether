//! Integration tests for FFI struct passing
//! 
//! Tests that verify correct struct layout, alignment, and passing
//! between Aether and C code.

use std::fs;
use std::path::PathBuf;
use std::process::Command;

#[test]
fn test_basic_struct_passing() {
    let test_program = r#"
(DEFINE_MODULE
  (NAME test_basic_struct)
  (INTENT "Test basic struct passing by value and pointer")
  
  (CONTENT
    ; Define Point2D struct
    (DEFINE_STRUCTURED_TYPE Point2D
      (FIELD (NAME x) (TYPE FLOAT64))
      (FIELD (NAME y) (TYPE FLOAT64))
    )
    
    ; External functions for testing
    (DECLARE_EXTERNAL_FUNCTION
      (NAME point_distance)
      (LIBRARY "aether_runtime")
      (RETURNS FLOAT64)
      (PARAM (NAME "p1") (TYPE Point2D) (PASSING_MODE BY_VALUE))
      (PARAM (NAME "p2") (TYPE Point2D) (PASSING_MODE BY_VALUE)))
    
    (DECLARE_EXTERNAL_FUNCTION
      (NAME point_add)
      (LIBRARY "aether_runtime")
      (RETURNS Point2D)
      (PARAM (NAME "p1") (TYPE Point2D) (PASSING_MODE BY_VALUE))
      (PARAM (NAME "p2") (TYPE Point2D) (PASSING_MODE BY_VALUE)))
    
    (DECLARE_EXTERNAL_FUNCTION
      (NAME point_scale)
      (LIBRARY "aether_runtime")
      (RETURNS VOID)
      (PARAM (NAME "p") (TYPE (POINTER_TO Point2D)) (PASSING_MODE BY_VALUE))
      (PARAM (NAME "factor") (TYPE FLOAT64)))
    
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
        ; Create two points
        (DECLARE_VARIABLE (NAME p1) (TYPE Point2D))
        (ASSIGN_FIELD p1 x 3.0)
        (ASSIGN_FIELD p1 y 4.0)
        
        (DECLARE_VARIABLE (NAME p2) (TYPE Point2D))
        (ASSIGN_FIELD p2 x 0.0)
        (ASSIGN_FIELD p2 y 0.0)
        
        ; Test distance calculation
        (DECLARE_VARIABLE (NAME dist) (TYPE FLOAT64))
        (ASSIGN (TARGET_VARIABLE dist)
          (SOURCE_EXPRESSION (CALL_FUNCTION point_distance p1 p2)))
        
        (CALL_FUNCTION printf "Distance: %f\n" dist)
        
        ; Test point addition
        (DECLARE_VARIABLE (NAME sum) (TYPE Point2D))
        (ASSIGN (TARGET_VARIABLE sum)
          (SOURCE_EXPRESSION (CALL_FUNCTION point_add p1 p2)))
        
        (CALL_FUNCTION printf "Sum: (%f, %f)\n" 
          (FIELD_ACCESS sum x) (FIELD_ACCESS sum y))
        
        ; Test point scaling by pointer
        (DECLARE_VARIABLE (NAME p3) (TYPE Point2D))
        (ASSIGN_FIELD p3 x 2.0)
        (ASSIGN_FIELD p3 y 3.0)
        
        (CALL_FUNCTION point_scale (ADDRESS_OF p3) 2.0)
        
        (CALL_FUNCTION printf "Scaled: (%f, %f)\n"
          (FIELD_ACCESS p3 x) (FIELD_ACCESS p3 y))
        
        (RETURN_VALUE 0)
      )
    )
  )
)
"#;

    // Write test program
    let test_file = PathBuf::from("test_basic_struct.aether");
    fs::write(&test_file, test_program).expect("Failed to write test file");
    
    // Compile the program
    let output = Command::new("target/release/aether-compiler")
        .arg("compile")
        .arg(&test_file)
        .arg("-o")
        .arg("test_basic_struct")
        .output()
        .expect("Failed to run compiler");
    
    if !output.status.success() {
        panic!("Compilation failed: {}", String::from_utf8_lossy(&output.stderr));
    }
    
    // Run the program
    let run_output = Command::new("./test_basic_struct")
        .output()
        .expect("Failed to run compiled program");
    
    assert!(run_output.status.success(), "Test program failed");
    let stdout = String::from_utf8_lossy(&run_output.stdout);
    
    // Verify output
    assert!(stdout.contains("Distance: 5"));
    assert!(stdout.contains("Sum: (3"));
    assert!(stdout.contains("Scaled: (4"));
    
    // Clean up
    fs::remove_file(test_file).ok();
    fs::remove_file("test_basic_struct").ok();
    fs::remove_file("test_basic_struct.o").ok();
}

#[test]
fn test_nested_struct_passing() {
    let test_program = r#"
(DEFINE_MODULE
  (NAME test_nested_struct)
  (INTENT "Test nested struct passing")
  
  (CONTENT
    ; Define Point2D struct
    (DEFINE_STRUCTURED_TYPE Point2D
      (FIELD (NAME x) (TYPE FLOAT64))
      (FIELD (NAME y) (TYPE FLOAT64))
    )
    
    ; Define Rectangle struct with nested Point2D
    (DEFINE_STRUCTURED_TYPE Rectangle
      (FIELD (NAME top_left) (TYPE Point2D))
      (FIELD (NAME width) (TYPE FLOAT64))
      (FIELD (NAME height) (TYPE FLOAT64))
    )
    
    ; External functions
    (DECLARE_EXTERNAL_FUNCTION
      (NAME rectangle_area)
      (LIBRARY "aether_runtime")
      (RETURNS FLOAT64)
      (PARAM (NAME "rect") (TYPE Rectangle) (PASSING_MODE BY_VALUE)))
    
    (DECLARE_EXTERNAL_FUNCTION
      (NAME rectangle_expand)
      (LIBRARY "aether_runtime")
      (RETURNS Rectangle)
      (PARAM (NAME "rect") (TYPE Rectangle) (PASSING_MODE BY_VALUE))
      (PARAM (NAME "amount") (TYPE FLOAT64)))
    
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
        ; Create a rectangle
        (DECLARE_VARIABLE (NAME rect) (TYPE Rectangle))
        (ASSIGN_FIELD (FIELD_ACCESS rect top_left) x 0.0)
        (ASSIGN_FIELD (FIELD_ACCESS rect top_left) y 0.0)
        (ASSIGN_FIELD rect width 10.0)
        (ASSIGN_FIELD rect height 5.0)
        
        ; Test area calculation
        (DECLARE_VARIABLE (NAME area) (TYPE FLOAT64))
        (ASSIGN (TARGET_VARIABLE area)
          (SOURCE_EXPRESSION (CALL_FUNCTION rectangle_area rect)))
        
        (CALL_FUNCTION printf "Area: %f\n" area)
        
        ; Test expansion
        (DECLARE_VARIABLE (NAME expanded) (TYPE Rectangle))
        (ASSIGN (TARGET_VARIABLE expanded)
          (SOURCE_EXPRESSION (CALL_FUNCTION rectangle_expand rect 1.0)))
        
        (CALL_FUNCTION printf "Expanded top-left: (%f, %f)\n"
          (FIELD_ACCESS (FIELD_ACCESS expanded top_left) x)
          (FIELD_ACCESS (FIELD_ACCESS expanded top_left) y))
        
        (CALL_FUNCTION printf "Expanded size: %f x %f\n"
          (FIELD_ACCESS expanded width)
          (FIELD_ACCESS expanded height))
        
        (RETURN_VALUE 0)
      )
    )
  )
)
"#;

    // Write test program
    let test_file = PathBuf::from("test_nested_struct.aether");
    fs::write(&test_file, test_program).expect("Failed to write test file");
    
    // Compile the program
    let output = Command::new("target/release/aether-compiler")
        .arg("compile")
        .arg(&test_file)
        .arg("-o")
        .arg("test_nested_struct")
        .output()
        .expect("Failed to run compiler");
    
    if !output.status.success() {
        panic!("Compilation failed: {}", String::from_utf8_lossy(&output.stderr));
    }
    
    // Run the program
    let run_output = Command::new("./test_nested_struct")
        .output()
        .expect("Failed to run compiled program");
    
    assert!(run_output.status.success(), "Test program failed");
    let stdout = String::from_utf8_lossy(&run_output.stdout);
    
    // Verify output
    assert!(stdout.contains("Area: 50"));
    assert!(stdout.contains("Expanded top-left: (-1"));
    assert!(stdout.contains("Expanded size: 12"));
    
    // Clean up
    fs::remove_file(test_file).ok();
    fs::remove_file("test_nested_struct").ok();
    fs::remove_file("test_nested_struct.o").ok();
}

#[test]
fn test_struct_with_small_fields() {
    let test_program = r#"
(DEFINE_MODULE
  (NAME test_struct_alignment)
  (INTENT "Test struct alignment with small fields")
  
  (CONTENT
    ; Define Color struct with byte fields
    (DEFINE_STRUCTURED_TYPE Color
      (FIELD (NAME r) (TYPE INTEGER8))
      (FIELD (NAME g) (TYPE INTEGER8))
      (FIELD (NAME b) (TYPE INTEGER8))
      (FIELD (NAME a) (TYPE INTEGER8))
    )
    
    ; External function
    (DECLARE_EXTERNAL_FUNCTION
      (NAME color_blend)
      (LIBRARY "aether_runtime")
      (RETURNS Color)
      (PARAM (NAME "c1") (TYPE Color) (PASSING_MODE BY_VALUE))
      (PARAM (NAME "c2") (TYPE Color) (PASSING_MODE BY_VALUE))
      (PARAM (NAME "ratio") (TYPE FLOAT32)))
    
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
        ; Create two colors
        (DECLARE_VARIABLE (NAME red) (TYPE Color))
        (ASSIGN_FIELD red r 255)
        (ASSIGN_FIELD red g 0)
        (ASSIGN_FIELD red b 0)
        (ASSIGN_FIELD red a 255)
        
        (DECLARE_VARIABLE (NAME blue) (TYPE Color))
        (ASSIGN_FIELD blue r 0)
        (ASSIGN_FIELD blue g 0)
        (ASSIGN_FIELD blue b 255)
        (ASSIGN_FIELD blue a 255)
        
        ; Test blending
        (DECLARE_VARIABLE (NAME purple) (TYPE Color))
        (ASSIGN (TARGET_VARIABLE purple)
          (SOURCE_EXPRESSION (CALL_FUNCTION color_blend red blue 0.5)))
        
        (CALL_FUNCTION printf "Blended color: R=%d G=%d B=%d A=%d\n"
          (CAST_AS (FIELD_ACCESS purple r) INTEGER)
          (CAST_AS (FIELD_ACCESS purple g) INTEGER)
          (CAST_AS (FIELD_ACCESS purple b) INTEGER)
          (CAST_AS (FIELD_ACCESS purple a) INTEGER))
        
        (RETURN_VALUE 0)
      )
    )
  )
)
"#;

    // Write test program
    let test_file = PathBuf::from("test_struct_alignment.aether");
    fs::write(&test_file, test_program).expect("Failed to write test file");
    
    // Compile the program
    let output = Command::new("target/release/aether-compiler")
        .arg("compile")
        .arg(&test_file)
        .arg("-o")
        .arg("test_struct_alignment")
        .output()
        .expect("Failed to run compiler");
    
    if !output.status.success() {
        panic!("Compilation failed: {}", String::from_utf8_lossy(&output.stderr));
    }
    
    // Run the program
    let run_output = Command::new("./test_struct_alignment")
        .output()
        .expect("Failed to run compiled program");
    
    assert!(run_output.status.success(), "Test program failed");
    let stdout = String::from_utf8_lossy(&run_output.stdout);
    
    // Verify output - blending red and blue should give purple
    assert!(stdout.contains("R=127") || stdout.contains("R=128")); // Allow for rounding
    assert!(stdout.contains("G=0"));
    assert!(stdout.contains("B=127") || stdout.contains("B=128"));
    assert!(stdout.contains("A=255"));
    
    // Clean up
    fs::remove_file(test_file).ok();
    fs::remove_file("test_struct_alignment").ok();
    fs::remove_file("test_struct_alignment.o").ok();
}

#[test]
fn test_struct_with_string_field() {
    let test_program = r#"
(DEFINE_MODULE
  (NAME test_struct_string)
  (INTENT "Test struct with string field")
  
  (CONTENT
    ; Define Person struct
    (DEFINE_STRUCTURED_TYPE Person
      (FIELD (NAME name) (TYPE STRING))
      (FIELD (NAME age) (TYPE INTEGER))
      (FIELD (NAME height) (TYPE FLOAT64))
    )
    
    ; External functions
    (DECLARE_EXTERNAL_FUNCTION
      (NAME person_create)
      (LIBRARY "aether_runtime")
      (RETURNS (POINTER_TO Person))
      (PARAM (NAME "name") (TYPE STRING))
      (PARAM (NAME "age") (TYPE INTEGER))
      (PARAM (NAME "height") (TYPE FLOAT64)))
    
    (DECLARE_EXTERNAL_FUNCTION
      (NAME person_free)
      (LIBRARY "aether_runtime")
      (RETURNS VOID)
      (PARAM (NAME "person") (TYPE (POINTER_TO Person))))
    
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
        ; Create a person
        (DECLARE_VARIABLE (NAME person_ptr) (TYPE (POINTER_TO Person)))
        (ASSIGN (TARGET_VARIABLE person_ptr)
          (SOURCE_EXPRESSION (CALL_FUNCTION person_create "Alice" 30 165.5)))
        
        ; Access fields through pointer
        (DECLARE_VARIABLE (NAME person) (TYPE Person))
        (ASSIGN (TARGET_VARIABLE person) (SOURCE_EXPRESSION (DEREFERENCE person_ptr)))
        
        (CALL_FUNCTION printf "Person: %s, age %d, height %.1f\n"
          (FIELD_ACCESS person name)
          (FIELD_ACCESS person age)
          (FIELD_ACCESS person height))
        
        ; Clean up
        (CALL_FUNCTION person_free person_ptr)
        
        (RETURN_VALUE 0)
      )
    )
  )
)
"#;

    // Write test program
    let test_file = PathBuf::from("test_struct_string.aether");
    fs::write(&test_file, test_program).expect("Failed to write test file");
    
    // Compile the program
    let output = Command::new("target/release/aether-compiler")
        .arg("compile")
        .arg(&test_file)
        .arg("-o")
        .arg("test_struct_string")
        .output()
        .expect("Failed to run compiler");
    
    if !output.status.success() {
        panic!("Compilation failed: {}", String::from_utf8_lossy(&output.stderr));
    }
    
    // Run the program
    let run_output = Command::new("./test_struct_string")
        .output()
        .expect("Failed to run compiled program");
    
    assert!(run_output.status.success(), "Test program failed");
    let stdout = String::from_utf8_lossy(&run_output.stdout);
    
    // Verify output
    assert!(stdout.contains("Person: Alice, age 30, height 165.5"));
    
    // Clean up
    fs::remove_file(test_file).ok();
    fs::remove_file("test_struct_string").ok();
    fs::remove_file("test_struct_string.o").ok();
}