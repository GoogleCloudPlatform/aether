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
        ComplexityExpectation, ComplexityNotation, ComplexityType, ContractAssertion,
        Expression, FailureAction, FunctionMetadata, PerformanceExpectation, PerformanceMetric,
        PrimitiveType,
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
