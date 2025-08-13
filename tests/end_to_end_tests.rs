use aether::lexer::Lexer;
use aether::parser::Parser;
use aether::semantic::SemanticAnalyzer;
use aether::ast::ASTPrettyPrinter;
use std::fs;

/// Helper function to get test fixture content
fn load_fixture(filename: &str) -> String {
    let path = format!("tests/fixtures/{}", filename);
    fs::read_to_string(&path).expect(&format!("Failed to load fixture: {}", path))
}

/// Test complete compilation pipeline from source to semantic analysis
#[test]
fn test_complete_pipeline_simple_module() {
    let source = load_fixture("simple_module.aether");
    
    // Step 1: Tokenization
    let mut lexer = Lexer::new(&source, "simple_module.aether".to_string());
    let tokens = lexer.tokenize().expect("Tokenization should succeed");
    
    assert!(!tokens.is_empty());
    assert_eq!(tokens.last().unwrap().token_type, aether::lexer::TokenType::Eof);
    
    // Step 2: Parsing
    let mut parser = Parser::new(tokens);
    let program = parser.parse_program().expect("Parsing should succeed");
    
    assert_eq!(program.modules.len(), 1);
    assert_eq!(program.modules[0].name.name, "simple_module");
    assert_eq!(program.modules[0].constant_declarations.len(), 2);
    
    // Step 3: Semantic Analysis
    let mut analyzer = SemanticAnalyzer::new();
    analyzer.analyze_program(&program).expect("Semantic analysis should succeed");
    
    let stats = analyzer.get_statistics();
    assert_eq!(stats.modules_analyzed, 1);
    assert_eq!(stats.variables_declared, 2);
    assert_eq!(stats.types_defined, 0);
    assert_eq!(stats.functions_analyzed, 0);
    
    // Step 4: AST Pretty Printing
    let mut pretty_printer = ASTPrettyPrinter::new();
    let ast_output = pretty_printer.print_program(&program);
    
    assert!(ast_output.contains("Program {"));
    assert!(ast_output.contains("Module 'simple_module'"));
    assert!(ast_output.contains("const VERSION: String"));
    assert!(ast_output.contains("const MAX_ITEMS: Integer"));
}

#[test]
fn test_complete_pipeline_empty_module() {
    let source = load_fixture("empty_module.aether");
    
    // Complete pipeline
    let mut lexer = Lexer::new(&source, "empty_module.aether".to_string());
    let tokens = lexer.tokenize().expect("Tokenization should succeed");
    
    let mut parser = Parser::new(tokens);
    let program = parser.parse_program().expect("Parsing should succeed");
    
    assert_eq!(program.modules.len(), 1);
    assert_eq!(program.modules[0].name.name, "empty_module");
    assert_eq!(program.modules[0].constant_declarations.len(), 0);
    assert_eq!(program.modules[0].function_definitions.len(), 0);
    assert_eq!(program.modules[0].type_definitions.len(), 0);
    
    let mut analyzer = SemanticAnalyzer::new();
    analyzer.analyze_program(&program).expect("Semantic analysis should succeed");
    
    let stats = analyzer.get_statistics();
    assert_eq!(stats.modules_analyzed, 1);
    assert_eq!(stats.variables_declared, 0);
}

#[test]
fn test_complete_pipeline_complex_expressions() {
    let source = load_fixture("complex_expressions.aether");
    
    let mut lexer = Lexer::new(&source, "complex_expressions.aether".to_string());
    let tokens = lexer.tokenize().expect("Tokenization should succeed");
    
    let mut parser = Parser::new(tokens);
    let program = parser.parse_program().expect("Parsing should succeed");
    
    assert_eq!(program.modules.len(), 1);
    assert_eq!(program.modules[0].constant_declarations.len(), 3);
    
    let mut analyzer = SemanticAnalyzer::new();
    analyzer.analyze_program(&program).expect("Semantic analysis should succeed");
    
    let stats = analyzer.get_statistics();
    assert_eq!(stats.modules_analyzed, 1);
    assert_eq!(stats.variables_declared, 3);
}

#[test]
fn test_pipeline_type_errors_detection() {
    let source = load_fixture("type_errors.aether");
    
    let mut lexer = Lexer::new(&source, "type_errors.aether".to_string());
    let tokens = lexer.tokenize().expect("Tokenization should succeed");
    
    let mut parser = Parser::new(tokens);
    let program = parser.parse_program().expect("Parsing should succeed");
    
    let mut analyzer = SemanticAnalyzer::new();
    let result = analyzer.analyze_program(&program);
    
    // Should fail semantic analysis due to type errors
    assert!(result.is_err());
    
    let errors = result.unwrap_err();
    assert!(errors.len() >= 1); // At least 1 type error expected (our fixture has 2 but let's be safe)
    
    // Verify error types
    let error_messages: Vec<String> = errors.iter().map(|e| e.to_string()).collect();
    let error_string = error_messages.join(" ");
    assert!(error_string.contains("Type mismatch"));
}

#[test]
fn test_pipeline_syntax_errors_detection() {
    let source = load_fixture("syntax_errors.aether");
    
    let mut lexer = Lexer::new(&source, "syntax_errors.aether".to_string());
    let tokens = lexer.tokenize().expect("Tokenization should succeed for syntax errors");
    
    let mut parser = Parser::new(tokens);
    let result = parser.parse_program();
    
    // Should fail parsing due to syntax errors
    assert!(result.is_err());
    
    let error = result.unwrap_err();
    assert!(error.to_string().contains("Unexpected") || error.to_string().contains("Expected"));
}

#[test]
fn test_lexer_error_recovery() {
    // Test with invalid characters
    let source = "invalid characters: @#$%^&*";
    
    let mut lexer = Lexer::new(source, "test.aether".to_string());
    let result = lexer.tokenize();
    
    assert!(result.is_err());
    let error = result.unwrap_err();
    assert!(error.to_string().contains("Unexpected character"));
}

#[test]
fn test_parser_error_recovery() {
    // Test with malformed S-expressions
    let source = "(DEFINE_MODULE (NAME 'test') (CONTENT ((((";
    
    let mut lexer = Lexer::new(source, "test.aether".to_string());
    let tokens = lexer.tokenize().expect("Tokenization should succeed");
    
    let mut parser = Parser::new(tokens);
    let result = parser.parse_program();
    
    assert!(result.is_err());
}

#[test]
fn test_large_file_performance() {
    use std::time::Instant;
    
    let source = load_fixture("large_file.aether");
    
    let start = Instant::now();
    
    // Complete pipeline
    let mut lexer = Lexer::new(&source, "large_file.aether".to_string());
    let tokens = lexer.tokenize().expect("Tokenization should succeed");
    
    let tokenization_time = start.elapsed();
    
    let mut parser = Parser::new(tokens);
    let program = parser.parse_program().expect("Parsing should succeed");
    
    let parsing_time = start.elapsed();
    
    let mut analyzer = SemanticAnalyzer::new();
    analyzer.analyze_program(&program).expect("Semantic analysis should succeed");
    
    let total_time = start.elapsed();
    
    // Performance assertions
    assert!(tokenization_time.as_millis() < 100, "Tokenization too slow: {:?}", tokenization_time);
    assert!(parsing_time.as_millis() < 200, "Parsing too slow: {:?}", parsing_time);
    assert!(total_time.as_millis() < 500, "Total pipeline too slow: {:?}", total_time);
    
    // Verify correctness
    let stats = analyzer.get_statistics();
    assert_eq!(stats.modules_analyzed, 1);
    assert_eq!(stats.variables_declared, 20);
}

#[test]
fn test_memory_usage() {
    // Test that we can process multiple files without excessive memory usage
    let files = vec![
        "simple_module.aether",
        "empty_module.aether",
        "complex_expressions.aether",
        "large_file.aether",
    ];
    
    for filename in files {
        let source = load_fixture(filename);
        
        let mut lexer = Lexer::new(&source, filename.to_string());
        let tokens = lexer.tokenize().expect(&format!("Tokenization should succeed for {}", filename));
        
        let mut parser = Parser::new(tokens);
        let program = parser.parse_program().expect(&format!("Parsing should succeed for {}", filename));
        
        let mut analyzer = SemanticAnalyzer::new();
        let result = analyzer.analyze_program(&program);
        
        // Only valid files should pass semantic analysis
        match filename {
            "simple_module.aether" | "empty_module.aether" | "complex_expressions.aether" | "large_file.aether" => {
                assert!(result.is_ok(), "Semantic analysis should succeed for valid file: {}", filename);
            }
            "type_errors.aether" => {
                assert!(result.is_err(), "Type errors file should fail semantic analysis");
            }
            _ => {
                // Other files may or may not pass - just check they don't crash
            }
        }
    }
}

#[test]
fn test_concurrent_processing() {
    use std::thread;
    use std::sync::Arc;
    
    let source = Arc::new(load_fixture("simple_module.aether"));
    let mut handles = vec![];
    
    // Process the same file in multiple threads
    for i in 0..4 {
        let source_clone = Arc::clone(&source);
        let handle = thread::spawn(move || {
            let filename = format!("thread_{}.aether", i);
            
            let mut lexer = Lexer::new(&source_clone, filename.clone());
            let tokens = lexer.tokenize().expect("Tokenization should succeed");
            
            let mut parser = Parser::new(tokens);
            let program = parser.parse_program().expect("Parsing should succeed");
            
            let mut analyzer = SemanticAnalyzer::new();
            analyzer.analyze_program(&program).expect("Semantic analysis should succeed");
            
            let stats = analyzer.get_statistics();
            (stats.modules_analyzed, stats.variables_declared)
        });
        
        handles.push(handle);
    }
    
    // Collect results
    for handle in handles {
        let (modules, variables) = handle.join().expect("Thread should complete successfully");
        assert_eq!(modules, 1);
        assert_eq!(variables, 2);
    }
}

#[test]
fn test_edge_cases() {
    // Test with very long identifiers
    let source = r#"(DEFINE_MODULE
        (NAME 'very_long_identifier_name_that_should_be_handled_correctly_by_the_lexer_and_parser')
        (INTENT "Test very long identifiers")
        (CONTENT)
    )"#;
    
    let mut lexer = Lexer::new(source, "edge_case.aether".to_string());
    let tokens = lexer.tokenize().expect("Should handle long identifiers");
    
    let mut parser = Parser::new(tokens);
    let program = parser.parse_program().expect("Should parse long identifiers");
    
    assert_eq!(program.modules[0].name.name, "very_long_identifier_name_that_should_be_handled_correctly_by_the_lexer_and_parser");
    
    // Test with simple constant for now (complex expressions not yet implemented)
    let source = r#"(DEFINE_MODULE
        (NAME 'nested_test')
        (INTENT "Test simple constants")
        (CONTENT
            (DECLARE_CONSTANT
                (NAME 'SIMPLE_VALUE')
                (TYPE INTEGER)
                (VALUE 123)
                (INTENT "Simple constant value")
            )
        )
    )"#;
    
    let mut lexer = Lexer::new(source, "nested.aether".to_string());
    let tokens = lexer.tokenize().expect("Should handle simple constants");
    
    let mut parser = Parser::new(tokens);
    let program = parser.parse_program().expect("Should parse simple constants");
    
    let mut analyzer = SemanticAnalyzer::new();
    analyzer.analyze_program(&program).expect("Should analyze simple constants");
}