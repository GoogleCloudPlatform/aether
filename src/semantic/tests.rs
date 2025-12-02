use super::*;
use crate::error::SourceLocation;

fn create_test_module() -> Module {
    Module {
        name: Identifier::new("test_module".to_string(), SourceLocation::unknown()),
        intent: Some("Test module".to_string()),
        imports: Vec::new(),
        exports: Vec::new(),
        type_definitions: Vec::new(),
        trait_definitions: Vec::new(),
        impl_blocks: Vec::new(), // Add this to match the AST definition
        constant_declarations: vec![ConstantDeclaration {
            name: Identifier::new("PI".to_string(), SourceLocation::unknown()),
            type_spec: Box::new(TypeSpecifier::Primitive {
                type_name: PrimitiveType::Float,
                source_location: SourceLocation::unknown(),
            }),
            value: Box::new(Expression::FloatLiteral {
                value: 3.14159,
                source_location: SourceLocation::unknown(),
            }),
            intent: Some("Mathematical constant PI".to_string()),
            source_location: SourceLocation::unknown(),
        }],
        function_definitions: Vec::new(),
        external_functions: Vec::new(),
        source_location: SourceLocation::unknown(),
    }
}

// Helper to create a SemanticAnalyzer from source code
fn semantic_analyzer_from_source(source: &str) -> SemanticAnalyzer {
    let mut lexer = crate::lexer::v2::Lexer::new(source, "test.aether".to_string());
    let tokens = lexer.tokenize().unwrap();
    let mut parser = crate::parser::v2::Parser::new(tokens);
    let program = parser.parse_program().unwrap();
    let mut analyzer = SemanticAnalyzer::new();
    // Assume a single module for simplicity in tests
    analyzer.analyze_module(&program.modules[0]).unwrap();
    analyzer
}

fn analyze_program_result(source: &str) -> Result<SemanticAnalyzer, Vec<SemanticError>> {
    let mut lexer = crate::lexer::v2::Lexer::new(source, "test.aether".to_string());
    let tokens = lexer.tokenize().unwrap();
    let mut parser = crate::parser::v2::Parser::new(tokens);
    let program = parser.parse_program().unwrap();
    let mut analyzer = SemanticAnalyzer::new();
    match analyzer.analyze_program(&program) {
        Ok(()) => Ok(analyzer),
        Err(errors) => Err(errors),
    }
}

#[test]
fn test_semantic_analyzer_creation() {
    let analyzer = SemanticAnalyzer::new();
    assert!(!analyzer.has_errors());
    assert_eq!(analyzer.get_statistics().modules_analyzed, 0);
}

#[test]
fn test_constant_declaration_analysis() {
    let mut analyzer = SemanticAnalyzer::new();
    let module = create_test_module();

    let result = analyzer.analyze_module(&module);
    assert!(result.is_ok());
    assert_eq!(analyzer.get_statistics().modules_analyzed, 1);
    assert_eq!(analyzer.get_statistics().variables_declared, 1);
}

#[test]
fn test_type_mismatch_detection() {
    let mut analyzer = SemanticAnalyzer::new();

    let mut module = create_test_module();
    // Change the constant to have mismatched type
    module.constant_declarations[0].value = Box::new(Expression::StringLiteral {
        value: "not a float".to_string(),
        source_location: SourceLocation::unknown(),
    });

    let result = analyzer.analyze_module(&module);
    assert!(result.is_err());
}

#[test]
fn test_expression_type_analysis() {
    let mut analyzer = SemanticAnalyzer::new();

    // Test integer literal
    let int_expr = Expression::IntegerLiteral {
        value: 42,
        source_location: SourceLocation::unknown(),
    };
    let int_type = analyzer.analyze_expression(&int_expr).unwrap();
    assert_eq!(int_type, Type::primitive(PrimitiveType::Integer));

    // Test arithmetic expression
    let add_expr = Expression::Add {
        left: Box::new(Expression::IntegerLiteral {
            value: 10,
            source_location: SourceLocation::unknown(),
        }),
        right: Box::new(Expression::IntegerLiteral {
            value: 20,
            source_location: SourceLocation::unknown(),
        }),
        source_location: SourceLocation::unknown(),
    };
    let add_type = analyzer.analyze_expression(&add_expr).unwrap();
    assert_eq!(add_type, Type::primitive(PrimitiveType::Integer));
}

#[test]
fn test_variable_initialization_checking() {
    let mut analyzer = SemanticAnalyzer::new();

    // Add an uninitialized variable
    let var_symbol = Symbol::new(
        "x".to_string(),
        Type::primitive(PrimitiveType::Integer),
        SymbolKind::Variable,
        true,
        false,
        SourceLocation::unknown(),
    );

    analyzer.symbol_table.add_symbol(var_symbol).unwrap();

    // Try to use the uninitialized variable
    let var_expr = Expression::Variable {
        name: Identifier::new("x".to_string(), SourceLocation::unknown()),
        source_location: SourceLocation::unknown(),
    };

    let result = analyzer.analyze_expression(&var_expr);
    assert!(result.is_err());
    if let Err(SemanticError::UseBeforeInitialization { .. }) = result {
        // Expected error
    } else {
        panic!("Expected UseBeforeInitialization error");
    }
}

#[test]
fn test_contract_validation_integration() {
    use crate::ast::{
        ComplexityExpectation, ComplexityNotation, ComplexityType, ContractAssertion, Expression,
        FailureAction, FunctionMetadata, PerformanceExpectation, PerformanceMetric, PrimitiveType,
    };
    use crate::contracts::{ContractContext, ContractValidator};
    use crate::error::SourceLocation;
    use crate::types::{Type, TypeChecker};
    use std::collections::HashMap;

    let mut validator = ContractValidator::new();
    let mut parameter_types = HashMap::new();
    parameter_types.insert("x".to_string(), Type::primitive(PrimitiveType::Integer));
    parameter_types.insert("y".to_string(), Type::primitive(PrimitiveType::Integer));

    let context = ContractContext {
        parameter_types,
        return_type: Type::primitive(PrimitiveType::Integer),
        type_checker: Rc::new(RefCell::new(TypeChecker::new())),
    };

    // Test valid metadata
    let valid_metadata = FunctionMetadata {
        preconditions: vec![ContractAssertion {
            condition: Box::new(Expression::BooleanLiteral {
                value: true,
                source_location: SourceLocation::unknown(),
            }),
            failure_action: FailureAction::AssertFail,
            message: Some("Test precondition".to_string()),
            source_location: SourceLocation::unknown(),
            runtime_check: false,
        }],
        postconditions: Vec::new(),
        invariants: Vec::new(),
        algorithm_hint: Some("division".to_string()),
        performance_expectation: Some(PerformanceExpectation {
            metric: PerformanceMetric::LatencyMs,
            target_value: 1.0,
            context: Some("Test latency".to_string()),
        }),
        complexity_expectation: Some(ComplexityExpectation {
            complexity_type: ComplexityType::Time,
            notation: ComplexityNotation::BigO,
            value: "O(1)".to_string(),
        }),
        throws_exceptions: Vec::new(),
        thread_safe: Some(true),
        may_block: Some(false),
    };

    let result = validator.validate_function_metadata(
        &valid_metadata,
        &context,
        "test_function",
        &SourceLocation::unknown(),
    );

    assert!(result.is_ok());
    let validation_result = result.unwrap();
    assert!(validation_result.is_valid);
    assert_eq!(validator.get_stats().functions_processed, 1);
    assert_eq!(validator.get_stats().preconditions_validated, 1);
    assert_eq!(validator.get_stats().performance_expectations_checked, 1);
    assert_eq!(validator.get_stats().complexity_expectations_checked, 1);
}

#[test]
fn test_contract_validation_failures() {
    use crate::ast::{
        ComplexityExpectation, ComplexityNotation, ComplexityType, FunctionMetadata,
        PerformanceExpectation, PerformanceMetric,
    };
    use crate::contracts::{ContractContext, ContractValidator};
    use crate::error::SourceLocation;
    use crate::types::{Type, TypeChecker};
    use std::collections::HashMap;

    let mut validator = ContractValidator::new();
    let context = ContractContext {
        parameter_types: HashMap::new(),
        return_type: Type::primitive(PrimitiveType::Void),
        type_checker: Rc::new(RefCell::new(TypeChecker::new())),
    };

    // Test invalid performance expectation
    let invalid_metadata = FunctionMetadata {
        preconditions: Vec::new(),
        postconditions: Vec::new(),
        invariants: Vec::new(),
        algorithm_hint: None,
        performance_expectation: Some(PerformanceExpectation {
            metric: PerformanceMetric::LatencyMs,
            target_value: -10.0, // Invalid negative value
            context: None,
        }),
        complexity_expectation: Some(ComplexityExpectation {
            complexity_type: ComplexityType::Time,
            notation: ComplexityNotation::BigO,
            value: "O(invalid)".to_string(), // Invalid complexity notation
        }),
        throws_exceptions: Vec::new(),
        thread_safe: None,
        may_block: None,
    };

    let result = validator.validate_function_metadata(
        &invalid_metadata,
        &context,
        "bad_function",
        &SourceLocation::unknown(),
    );

    assert!(result.is_ok());
    let validation_result = result.unwrap();
    assert!(!validation_result.is_valid);
    assert!(!validation_result.errors.is_empty());
    assert_eq!(validator.get_stats().contract_errors, 2); // Performance + complexity errors
}

#[test]
fn test_generic_function_param_resolution() {
    let source = r#"
    module Test {
        func identity<T>(value: T) -> T {
            return value;
        }
    }
    "#;
    let analyzer = semantic_analyzer_from_source(source);

    // Use get_all_symbols() since the module scope has been exited after analysis
    let all_symbols = analyzer.symbol_table.get_all_symbols();
    let func_symbol = all_symbols
        .iter()
        .find(|s| s.name == "identity")
        .expect("Function 'identity' should exist");
    if let Type::Function {
        parameter_types,
        return_type,
        ..
    } = &func_symbol.symbol_type
    {
        assert_eq!(parameter_types.len(), 1);
        assert!(matches!(parameter_types[0], Type::Generic { ref name, .. } if name == "T"));
        assert!(matches!(**return_type, Type::Generic { ref name, .. } if name == "T"));
    } else {
        panic!("Expected function type");
    }

    // Within the function's scope, 'T' should be resolvable as a type
    let func_scope_symbols = analyzer.symbol_table.get_all_symbols();
    let generic_t_symbol = func_scope_symbols
        .iter()
        .find(|s| s.name == "T" && matches!(s.symbol_type, Type::Generic { .. }));
    assert!(generic_t_symbol.is_some());
}

#[test]
fn test_generic_struct_field_resolution() {
    let source = r#"
    module Test {
        struct Box<T> {
            value: T;
        }
    }
    "#;
    let analyzer = semantic_analyzer_from_source(source);

    let struct_def = analyzer.symbol_table.lookup_type_definition("Box").unwrap();
    if let crate::types::TypeDefinition::Struct { fields, .. } = struct_def {
        assert_eq!(fields.len(), 1);
        assert_eq!(&fields[0].0, "value");
        assert!(matches!(&fields[0].1, Type::Generic { ref name, .. } if name == "T"));
    } else {
        panic!("Expected struct definition");
    }
}

#[test]
fn test_generic_enum_variant_resolution() {
    let source = r#"
    module Test {
        enum Option<T> {
            case Some(T);
            case None;
        }
    }
    "#;
    let analyzer = semantic_analyzer_from_source(source);

    let enum_def = analyzer
        .symbol_table
        .lookup_type_definition("Option")
        .unwrap();
    if let crate::types::TypeDefinition::Enum { variants, .. } = enum_def {
        assert_eq!(variants.len(), 2);
        assert_eq!(&variants[0].name, "Some");
        assert_eq!(variants[0].associated_types.len(), 1);
        assert!(
            matches!(&variants[0].associated_types[0], Type::Generic { ref name, .. } if name == "T")
        );
        assert_eq!(&variants[1].name, "None");
        assert_eq!(variants[1].associated_types.len(), 0);
    } else {
        panic!("Expected enum definition");
    }
}

#[test]
fn test_undefined_generic_parameter_in_function() {
    let source = r#"
    module Test {
        func foo<T>(value: U) -> T { // U is undefined
            return value;
        }
    }
    "#;
    let mut lexer = crate::lexer::v2::Lexer::new(source, "test.aether".to_string());
    let tokens = lexer.tokenize().unwrap();
    let mut parser = crate::parser::v2::Parser::new(tokens);
    let program = parser.parse_program().unwrap();
    let mut analyzer = SemanticAnalyzer::new();
    let analysis_result = analyzer.analyze_module(&program.modules[0]);
    assert!(analysis_result.is_err());
    assert!(
        matches!(analysis_result.unwrap_err(), SemanticError::UndefinedSymbol { symbol, .. } if symbol == "U")
    );
}

#[test]
fn test_trait_method_resolution_for_concrete_type() {
    let source = r#"
    module test;

    trait Printable {
        func print(self: &Self) -> Int;
    }

    struct Label {
        value: String;
    }

    impl Printable for Label {
        func print(self: &Self) -> Int {
            return 42;
        }
    }

    func demo(label: Label) -> Int {
        return label.print();
    }
    "#;

    let analyzer = analyze_program_result(source).expect("semantic analysis should succeed");
    let dispatch = analyzer.get_trait_dispatch_table();
    let key = TraitMethodKey {
        receiver: Type::named("Label".to_string(), Some("test".to_string())),
        method_name: "print".to_string(),
    };

    assert!(
        dispatch.contains_key(&key),
        "expected dispatch entry for Label.print"
    );

    let info = dispatch.get(&key).unwrap();
    assert_eq!(info.trait_name, "Printable");
    assert_eq!(info.return_type, Type::primitive(PrimitiveType::Integer));
}

#[test]
fn test_trait_method_resolution_for_generic_param() {
    let source = r#"
    module test;

    trait Displayable {
        func display(self: &Self) -> String;
    }

    func render<T>(value: T) -> String where T: Displayable {
        return value.display();
    }
    "#;

    let analyzer = analyze_program_result(source).expect("semantic analysis should succeed");
    let symbol_table = analyzer.get_symbol_table();
    let render_fn = symbol_table
        .lookup_symbol_any_scope("render")
        .expect("render function should be registered");

    if let Type::Function { return_type, .. } = &render_fn.symbol_type {
        assert_eq!(**return_type, Type::primitive(PrimitiveType::String));
    } else {
        panic!("render should be a function");
    }
}

#[test]
fn test_trait_method_resolution_missing_impl_errors() {
    let source = r#"
    module test;

    trait Serializable {
        func serialize(self: &Self) -> String;
    }

    struct Plain {}

    func demo(value: Plain) -> String {
        return value.serialize();
    }
    "#;

    let result = analyze_program_result(source);
    assert!(result.is_err(), "expected error for missing trait impl");
    let errors = result.err().unwrap();
    assert!(
        errors.iter().any(|e| matches!(
            e,
            SemanticError::InvalidOperation { .. }
                | SemanticError::UndefinedSymbol { .. }
                | SemanticError::TypeMismatch { .. }
        )),
        "expected a semantic error about trait method resolution, got {:?}",
        errors
    );
}
