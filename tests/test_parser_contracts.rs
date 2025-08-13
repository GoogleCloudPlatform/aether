#[cfg(test)]
mod parser_contract_tests {
    use aether::parser::Parser;
    use aether::lexer::Lexer;
    use aether::ast::*;
    use aether::error::SourceLocation;
    
    #[test]
    fn test_parser_function_with_contracts() {
        let test_file_path = "tests/fixtures/function_with_contracts.aether";
        let source = std::fs::read_to_string(test_file_path)
            .expect("Failed to read test file");
        
        let mut lexer = Lexer::new(&source, test_file_path.to_string());
        let tokens = match lexer.tokenize() {
            Ok(tokens) => tokens,
            Err(e) => panic!("Lexer error: {:?}", e),
        };
        
        let mut parser = Parser::new(tokens);
        let module = match parser.parse_module() {
            Ok(module) => module,
            Err(e) => panic!("Parser error: {:?}", e),
        };
        
        // Verify module was parsed
        assert_eq!(module.name.name, "function_with_contracts");
        assert_eq!(module.intent, Some("Test module with function contracts and metadata".to_string()));
        
        // Check that we have the test_division function
        assert_eq!(module.function_definitions.len(), 1);
        let function = &module.function_definitions[0];
        assert_eq!(function.name.name, "test_division");
        
        // Check metadata
        assert!(!function.metadata.preconditions.is_empty());
        assert!(!function.metadata.postconditions.is_empty());
        assert_eq!(function.metadata.algorithm_hint, Some("division".to_string()));
        assert!(function.metadata.performance_expectation.is_some());
        assert!(function.metadata.complexity_expectation.is_some());
        
        println!("✓ Successfully parsed function with contracts");
    }
    
    #[test]
    fn test_parser_invalid_contracts() {
        let test_file_path = "tests/fixtures/invalid_contracts.aether";
        let source = std::fs::read_to_string(test_file_path)
            .expect("Failed to read test file");
        
        let mut lexer = Lexer::new(&source, test_file_path.to_string());
        let tokens = match lexer.tokenize() {
            Ok(tokens) => tokens,
            Err(e) => panic!("Lexer error: {:?}", e),
        };
        
        let mut parser = Parser::new(tokens);
        let module = match parser.parse_module() {
            Ok(module) => module,
            Err(e) => panic!("Parser error: {:?}", e),
        };
        
        // Verify module was parsed
        assert_eq!(module.name.name, "invalid_contracts");
        
        // Check that we have the bad_performance function
        assert_eq!(module.function_definitions.len(), 1);
        let function = &module.function_definitions[0];
        assert_eq!(function.name.name, "bad_performance");
        
        // Check metadata was parsed (even though values are invalid)
        assert!(function.metadata.performance_expectation.is_some());
        assert!(function.metadata.complexity_expectation.is_some());
        
        println!("✓ Successfully parsed function with invalid contracts (validation happens later)");
    }
}