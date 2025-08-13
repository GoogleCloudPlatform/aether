// Integration tests for the AetherScript compiler
// This file serves as the main integration test entry point

use aether::Compiler;

mod integration;

#[test]
fn test_compiler_creation() {
    let _compiler = Compiler::new();
    // Basic test to ensure the compiler can be instantiated
}

#[test]
fn test_compiler_default() {
    let _compiler = Compiler::default();
    // Test default implementation
}

// Additional integration tests are in separate files:
// - cli_integration_tests.rs: CLI interface testing
// - end_to_end_tests.rs: Complete compilation pipeline testing  
// - property_based_tests.rs: Property-based and fuzz testing
// - integration/: LLM-first feature integration tests

#[test]
fn test_integration_test_setup() {
    // Verify that test fixtures exist
    assert!(std::path::Path::new("tests/fixtures").exists());
    assert!(std::path::Path::new("tests/fixtures/simple_module.aether").exists());
    assert!(std::path::Path::new("tests/fixtures/type_errors.aether").exists());
    assert!(std::path::Path::new("tests/fixtures/complex_expressions.aether").exists());
    assert!(std::path::Path::new("tests/fixtures/syntax_errors.aether").exists());
    assert!(std::path::Path::new("tests/fixtures/empty_module.aether").exists());
    assert!(std::path::Path::new("tests/fixtures/large_file.aether").exists());
}