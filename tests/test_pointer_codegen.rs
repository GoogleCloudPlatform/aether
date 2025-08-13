//! Integration tests for pointer operations in LLVM codegen

use aether::Compiler;
use aether::error::CompilerError;
use std::fs;
use std::path::PathBuf;
use tempfile::TempDir;

#[test]
fn test_pointer_address_of() -> Result<(), CompilerError> {
    let source = r#"
(DEFINE_MODULE
    (NAME 'test_pointers')
    (CONTENT
        (DEFINE_FUNCTION
            (NAME 'test_address_of')
            (RETURNS (TYPE INTEGER))
            (BODY
                (DECLARE_VARIABLE (NAME 'x') (TYPE INTEGER) (INITIAL_VALUE (INTEGER_LITERAL 42)))
                (DECLARE_VARIABLE (NAME 'ptr') (TYPE (POINTER INTEGER)) 
                    (INITIAL_VALUE (ADDRESS_OF (VARIABLE_REFERENCE 'x'))))
                (RETURN (DEREFERENCE (VARIABLE_REFERENCE 'ptr')))))))
    "#;
    
    // Create a temporary directory for the test
    let temp_dir = TempDir::new()?;
    let test_file = temp_dir.path().join("test_address_of.aether");
    fs::write(&test_file, source)?;
    
    let compiler = Compiler::new();
    let result = compiler.compile_file(test_file)?;
    
    // For now, just verify compilation succeeds
    // Compilation succeeded if we got here without error
    
    Ok(())
}

#[test]
fn test_pointer_arithmetic() -> Result<(), CompilerError> {
    let source = r#"
        (module test_pointers
            (function test_pointer_add () integer
                (let ((arr (array 1 2 3 4 5))
                      (ptr (ADDRESS_OF arr))
                      (ptr2 (POINTER_ADD ptr 2)))
                    (DEREFERENCE ptr2))))
    "#;
    
    let temp_dir = TempDir::new()?;
    let test_file = temp_dir.path().join("test_pointer_add.aether");
    fs::write(&test_file, source)?;
    
    let compiler = Compiler::new();
    let result = compiler.compile_file(test_file)?;
    
    // Compilation succeeded if we got here without error
    
    Ok(())
}

#[test]
fn test_pointer_dereference() -> Result<(), CompilerError> {
    let source = r#"
        (module test_pointers
            (function test_deref () integer
                (let ((x 100)
                      (ptr (ADDRESS_OF x))
                      (y (DEREFERENCE ptr)))
                    y)))
    "#;
    
    let temp_dir = TempDir::new()?;
    let test_file = temp_dir.path().join("test_deref.aether");
    fs::write(&test_file, source)?;
    
    let compiler = Compiler::new();
    let result = compiler.compile_file(test_file)?;
    
    // Compilation succeeded if we got here without error
    
    Ok(())
}