use proptest::prelude::*;
use aether::lexer::{Lexer, TokenType};
use aether::parser::Parser;
use aether::semantic::SemanticAnalyzer;

/// Generate valid AetherScript identifiers
fn valid_identifier() -> impl Strategy<Value = String> {
    prop::string::string_regex(r"[a-zA-Z_][a-zA-Z0-9_]{0,100}").unwrap()
}

/// Generate valid string literals  
fn valid_string_literal() -> impl Strategy<Value = String> {
    prop::string::string_regex(r#""[^"]*""#).unwrap()
}

/// Generate valid integer literals
fn valid_integer() -> impl Strategy<Value = i64> {
    prop::num::i64::ANY
}

/// Generate valid float literals
fn valid_float() -> impl Strategy<Value = f64> {
    prop::num::f64::POSITIVE | prop::num::f64::NEGATIVE
}

/// Generate random strings for fuzzing
fn fuzz_string() -> impl Strategy<Value = String> {
    prop::collection::vec(any::<u8>(), 0..1000)
        .prop_map(|bytes| String::from_utf8_lossy(&bytes).to_string())
}

/// Property test: Valid identifiers should always tokenize successfully
proptest! {
    #[test]
    fn test_valid_identifiers_tokenize(identifier in valid_identifier()) {
        let source = format!("({})", identifier);
        let mut lexer = Lexer::new(&source, "test.aether".to_string());
        
        let result = lexer.tokenize();
        prop_assert!(result.is_ok());
        
        let tokens = result.unwrap();
        prop_assert!(tokens.len() >= 3); // LeftParen, Identifier, RightParen, Eof
        
        // Check that identifier is properly tokenized
        let identifier_token = &tokens[1];
        if let TokenType::Identifier(name) = &identifier_token.token_type {
            prop_assert_eq!(name, &identifier);
        } else {
            prop_assert!(false, "Expected identifier token");
        }
    }
}

/// Property test: Valid integers should always tokenize and parse correctly
proptest! {
    #[test]
    fn test_valid_integers_parse(value in valid_integer()) {
        let source = format!(r#"(DEFINE_MODULE
            (NAME 'test')
            (INTENT "Test")
            (CONTENT
                (DECLARE_CONSTANT
                    (NAME 'TEST_INT')
                    (TYPE INTEGER)
                    (VALUE {})
                    (INTENT "Test integer")
                )
            )
        )"#, value);
        
        let mut lexer = Lexer::new(&source, "test.aether".to_string());
        let tokens = lexer.tokenize();
        prop_assert!(tokens.is_ok());
        
        let mut parser = Parser::new(tokens.unwrap());
        let program = parser.parse_program();
        prop_assert!(program.is_ok());
        
        // Verify semantic analysis passes for valid integer
        let mut analyzer = SemanticAnalyzer::new();
        let result = analyzer.analyze_program(&program.unwrap());
        prop_assert!(result.is_ok());
    }
}

/// Property test: Valid floats should always tokenize and parse correctly
proptest! {
    #[test]
    fn test_valid_floats_parse(value in -1000.0f64..1000.0f64) {
        let source = format!(r#"(DEFINE_MODULE
            (NAME 'test')
            (INTENT "Test")
            (CONTENT
                (DECLARE_CONSTANT
                    (NAME 'TEST_FLOAT')
                    (TYPE FLOAT)
                    (VALUE {})
                    (INTENT "Test float")
                )
            )
        )"#, value);
        
        let mut lexer = Lexer::new(&source, "test.aether".to_string());
        let tokens = lexer.tokenize();
        prop_assert!(tokens.is_ok());
        
        let mut parser = Parser::new(tokens.unwrap());
        let program = parser.parse_program();
        prop_assert!(program.is_ok());
        
        // Verify semantic analysis passes for valid float
        let mut analyzer = SemanticAnalyzer::new();
        let result = analyzer.analyze_program(&program.unwrap());
        prop_assert!(result.is_ok());
    }
}

/// Property test: String literals should always tokenize correctly
proptest! {
    #[test]
    fn test_string_literals_tokenize(content in "[a-zA-Z0-9 ]{0,20}") {
        let string_literal = format!(r#""{}""#, content);
        let source = format!("({})", string_literal);
        
        let mut lexer = Lexer::new(&source, "test.aether".to_string());
        let result = lexer.tokenize();
        prop_assert!(result.is_ok());
        
        let tokens = result.unwrap();
        prop_assert!(tokens.len() >= 3); // LeftParen, String, RightParen, Eof
        
        // Check that string is properly tokenized
        if let TokenType::String(value) = &tokens[1].token_type {
            prop_assert_eq!(value, &content);
        } else {
            prop_assert!(false, "Expected string token");
        }
    }
}

/// Property test: Well-formed modules should always parse successfully
proptest! {
    #[test]
    fn test_well_formed_modules_parse(
        module_name in valid_identifier(),
        intent in "[a-zA-Z0-9 ]{0,30}",
        const_name in valid_identifier(),
        const_value in valid_integer()
    ) {
        let source = format!(r#"(DEFINE_MODULE
            (NAME '{}')
            (INTENT "{}")
            (CONTENT
                (DECLARE_CONSTANT
                    (NAME '{}')
                    (TYPE INTEGER)
                    (VALUE {})
                    (INTENT "Generated constant")
                )
            )
        )"#, module_name, intent, const_name, const_value);
        
        let mut lexer = Lexer::new(&source, "test.aether".to_string());
        let tokens = lexer.tokenize();
        prop_assert!(tokens.is_ok());
        
        let mut parser = Parser::new(tokens.unwrap());
        let program = parser.parse_program();
        prop_assert!(program.is_ok());
        
        let prog = program.unwrap();
        prop_assert_eq!(prog.modules.len(), 1);
        prop_assert_eq!(&prog.modules[0].name.name, &module_name);
        prop_assert_eq!(prog.modules[0].constant_declarations.len(), 1);
        prop_assert_eq!(&prog.modules[0].constant_declarations[0].name.name, &const_name);
    }
}

/// Property test: Lexer should never crash on any input
proptest! {
    #[test]
    fn test_lexer_never_crashes(input in fuzz_string()) {
        let mut lexer = Lexer::new(&input, "fuzz.aether".to_string());
        
        // The lexer should never panic, even on invalid input
        let _result = lexer.tokenize();
        
        // We don't care if it succeeds or fails, just that it doesn't crash
        prop_assert!(true);
    }
}

/// Property test: Parser should never crash on valid tokens
proptest! {
    #[test]
    fn test_parser_never_crashes_on_tokens(
        tokens in prop::collection::vec(
            prop::sample::select(vec![
                TokenType::LeftParen,
                TokenType::RightParen,
                TokenType::Keyword("NAME".to_string()),
                TokenType::Keyword("DEFINE_MODULE".to_string()),
                TokenType::Keyword("CONTENT".to_string()),
                TokenType::Identifier("test".to_string()),
                TokenType::String("test".to_string()),
                TokenType::Integer(42),
                TokenType::Eof,
            ]),
            0..100
        )
    ) {
        // Create tokens with dummy locations
        let test_tokens: Vec<aether::lexer::Token> = tokens.into_iter().map(|token_type| {
            aether::lexer::Token {
                token_type,
                lexeme: "test".to_string(),
                location: aether::error::SourceLocation::unknown(),
            }
        }).collect();
        
        let mut parser = Parser::new(test_tokens);
        
        // Parser should never panic, even on malformed token streams
        let _result = parser.parse_program();
        
        // We don't care if it succeeds or fails, just that it doesn't crash
        prop_assert!(true);
    }
}

/// Property test: Semantic analyzer should never crash on valid ASTs
proptest! {
    #[test]
    fn test_semantic_analyzer_never_crashes(
        module_name in valid_identifier(),
        const_name in valid_identifier()
    ) {
        // Create a minimal valid AST
        let module = aether::ast::Module {
            name: aether::ast::Identifier::new(module_name, aether::error::SourceLocation::unknown()),
            intent: Some("Test module".to_string()),
            imports: Vec::new(),
            exports: Vec::new(),
            type_definitions: Vec::new(),
            constant_declarations: vec![
                aether::ast::ConstantDeclaration {
                    name: aether::ast::Identifier::new(const_name, aether::error::SourceLocation::unknown()),
                    type_spec: Box::new(aether::ast::TypeSpecifier::Primitive {
                        type_name: aether::ast::PrimitiveType::Integer,
                        source_location: aether::error::SourceLocation::unknown(),
                    }),
                    value: Box::new(aether::ast::Expression::IntegerLiteral {
                        value: 42,
                        source_location: aether::error::SourceLocation::unknown(),
                    }),
                    intent: Some("Test constant".to_string()),
                    source_location: aether::error::SourceLocation::unknown(),
                }
            ],
            function_definitions: Vec::new(),
            external_functions: Vec::new(),
            source_location: aether::error::SourceLocation::unknown(),
        };
        
        let program = aether::ast::Program {
            modules: vec![module],
            source_location: aether::error::SourceLocation::unknown(),
        };
        
        let mut analyzer = SemanticAnalyzer::new();
        
        // Semantic analyzer should never panic
        let _result = analyzer.analyze_program(&program);
        
        // We don't care if it succeeds or fails, just that it doesn't crash
        prop_assert!(true);
    }
}

/// Property test: Type checking should be consistent
proptest! {
    #[test]
    fn test_type_checking_consistency(
        const_name in valid_identifier(),
        int_value in valid_integer(),
        _string_value in "[a-zA-Z0-9 ]{0,10}"
    ) {
        // Test that integer constants with integer values always pass type checking
        let int_source = format!(r#"(DEFINE_MODULE
            (NAME 'test')
            (INTENT "Test")
            (CONTENT
                (DECLARE_CONSTANT
                    (NAME '{}')
                    (TYPE INTEGER)
                    (VALUE {})
                    (INTENT "Integer constant")
                )
            )
        )"#, const_name, int_value);
        
        let mut lexer = Lexer::new(&int_source, "test.aether".to_string());
        let tokens = lexer.tokenize().unwrap();
        let mut parser = Parser::new(tokens);
        let program = parser.parse_program().unwrap();
        let mut analyzer = SemanticAnalyzer::new();
        let result = analyzer.analyze_program(&program);
        
        prop_assert!(result.is_ok(), "Integer constant with integer value should pass type checking");
        
        // Test that string constants with integer values always fail type checking
        let mixed_source = format!(r#"(DEFINE_MODULE
            (NAME 'test')
            (INTENT "Test")
            (CONTENT
                (DECLARE_CONSTANT
                    (NAME '{}')
                    (TYPE STRING)
                    (VALUE {})
                    (INTENT "String constant with int value - should fail")
                )
            )
        )"#, const_name, int_value);
        
        let mut lexer = Lexer::new(&mixed_source, "test.aether".to_string());
        let tokens = lexer.tokenize().unwrap();
        let mut parser = Parser::new(tokens);
        let program = parser.parse_program().unwrap();
        let mut analyzer = SemanticAnalyzer::new();
        let result = analyzer.analyze_program(&program);
        
        prop_assert!(result.is_err(), "String constant with integer value should fail type checking");
    }
}

/// Property test: Simple integer constants should be consistent
proptest! {
    #[test]
    fn test_simple_integer_consistency(
        value in 1i64..1000
    ) {
        let source = format!(r#"(DEFINE_MODULE
            (NAME 'test')
            (INTENT "Test")
            (CONTENT
                (DECLARE_CONSTANT
                    (NAME 'SIMPLE_VALUE')
                    (TYPE INTEGER)
                    (VALUE {})
                    (INTENT "Simple integer value")
                )
            )
        )"#, value);
        
        let mut lexer = Lexer::new(&source, "test.aether".to_string());
        let tokens = lexer.tokenize();
        prop_assert!(tokens.is_ok());
        
        let mut parser = Parser::new(tokens.unwrap());
        let program = parser.parse_program();
        prop_assert!(program.is_ok());
        
        let mut analyzer = SemanticAnalyzer::new();
        let result = analyzer.analyze_program(&program.unwrap());
        prop_assert!(result.is_ok(), "Simple integer constants should always type check correctly");
    }
}

/// Property test: Module names should be preserved through the entire pipeline
proptest! {
    #[test]
    fn test_module_name_preservation(module_name in valid_identifier()) {
        let source = format!(r#"(DEFINE_MODULE
            (NAME '{}')
            (INTENT "Test module name preservation")
            (CONTENT)
        )"#, module_name);
        
        let mut lexer = Lexer::new(&source, "test.aether".to_string());
        let tokens = lexer.tokenize().unwrap();
        
        let mut parser = Parser::new(tokens);
        let program = parser.parse_program().unwrap();
        
        prop_assert_eq!(program.modules.len(), 1);
        prop_assert_eq!(&program.modules[0].name.name, &module_name);
        
        let mut analyzer = SemanticAnalyzer::new();
        let result = analyzer.analyze_program(&program);
        prop_assert!(result.is_ok());
        
        // Module name should still be preserved after semantic analysis
        prop_assert_eq!(&program.modules[0].name.name, &module_name);
    }
}