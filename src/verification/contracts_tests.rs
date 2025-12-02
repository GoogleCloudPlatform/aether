use super::*;

#[test]
fn test_contract_creation() {
    let mut contract = FunctionContract::new("test_func".to_string());

    // Add a precondition: x > 0
    contract.add_precondition(
        "positive_x".to_string(),
        Expression::BinaryOp {
            op: BinaryOp::Gt,
            left: Box::new(Expression::Variable("x".to_string())),
            right: Box::new(Expression::Constant(ConstantValue::Integer(0))),
        },
        SourceLocation::unknown(),
    );

    assert_eq!(contract.preconditions.len(), 1);
    assert_eq!(contract.preconditions[0].name, "positive_x");
}

#[test]
fn test_enhanced_contract_creation() {
    let mut contract = FunctionContract::new("safe_divide".to_string());

    // Add enhanced precondition with proof hint
    contract.add_enhanced_precondition(
        "non_zero_denominator".to_string(),
        Expression::BinaryOp {
            op: BinaryOp::Ne,
            left: Box::new(Expression::Variable("denominator".to_string())),
            right: Box::new(Expression::Constant(ConstantValue::Float(0.0))),
        },
        SourceLocation::unknown(),
        Some("denominator != 0 is required for division".to_string()),
        FailureAction::ThrowException("Division by zero".to_string()),
        VerificationHint::SMTSolver,
        None,
    );

    assert_eq!(contract.preconditions.len(), 1);
    assert_eq!(contract.preconditions[0].name, "non_zero_denominator");
    assert!(contract.preconditions[0].proof_hint.is_some());
    assert!(matches!(
        contract.preconditions[0].failure_action,
        FailureAction::ThrowException(_)
    ));
}

#[test]
fn test_expression_to_string() {
    // Test: x + 1
    let expr = Expression::BinaryOp {
        op: BinaryOp::Add,
        left: Box::new(Expression::Variable("x".to_string())),
        right: Box::new(Expression::Constant(ConstantValue::Integer(1))),
    };

    assert_eq!(expr.to_string(), "(x + 1)");

    // Test: forall x. x > 0
    let quantified = Expression::Quantifier {
        kind: QuantifierKind::Forall,
        variables: vec![(
            "x".to_string(),
            Type::primitive(crate::ast::PrimitiveType::Integer),
        )],
        body: Box::new(Expression::BinaryOp {
            op: BinaryOp::Gt,
            left: Box::new(Expression::Variable("x".to_string())),
            right: Box::new(Expression::Constant(ConstantValue::Integer(0))),
        }),
    };

    assert_eq!(quantified.to_string(), "forall x. (x > 0)");
}

#[test]
fn test_semantic_predicate() {
    let expr = Expression::SemanticPredicate {
        predicate: "is_valid_email".to_string(),
        args: vec![Expression::Variable("email".to_string())],
    };

    assert_eq!(expr.to_string(), "is_valid_email(email)");
}

#[test]
fn test_temporal_expressions() {
    // Test: always (x > 0)
    let temporal = Expression::Temporal {
        op: TemporalOp::Always,
        expr: Box::new(Expression::BinaryOp {
            op: BinaryOp::Gt,
            left: Box::new(Expression::Variable("x".to_string())),
            right: Box::new(Expression::Constant(ConstantValue::Integer(0))),
        }),
    };

    assert_eq!(temporal.to_string(), "always (x > 0)");
}

#[test]
fn test_aggregate_expressions() {
    // Test: sum(array)
    let sum_expr = Expression::Aggregate {
        op: AggregateOp::Sum,
        collection: Box::new(Expression::Variable("array".to_string())),
        predicate: None,
    };

    assert_eq!(sum_expr.to_string(), "sum(array)");

    // Test: all(array | x > 0)
    let all_expr = Expression::Aggregate {
        op: AggregateOp::All,
        collection: Box::new(Expression::Variable("array".to_string())),
        predicate: Some(Box::new(Expression::BinaryOp {
            op: BinaryOp::Gt,
            left: Box::new(Expression::Variable("x".to_string())),
            right: Box::new(Expression::Constant(ConstantValue::Integer(0))),
        })),
    };

    assert_eq!(all_expr.to_string(), "all(array | (x > 0))");
}

#[test]
fn test_proof_obligation_generation() {
    let mut contract = FunctionContract::new("test_func".to_string());

    contract.add_enhanced_precondition(
        "pre1".to_string(),
        Expression::BinaryOp {
            op: BinaryOp::Gt,
            left: Box::new(Expression::Variable("x".to_string())),
            right: Box::new(Expression::Constant(ConstantValue::Integer(0))),
        },
        SourceLocation::unknown(),
        None,
        FailureAction::ThrowException("x must be positive".to_string()),
        VerificationHint::SMTSolver,
        None,
    );

    contract.add_enhanced_postcondition(
        "post1".to_string(),
        Expression::BinaryOp {
            op: BinaryOp::Ge,
            left: Box::new(Expression::Result),
            right: Box::new(Expression::Constant(ConstantValue::Integer(0))),
        },
        SourceLocation::unknown(),
        Some("Result is non-negative".to_string()),
        FailureAction::Abort,
        VerificationHint::SMTSolver,
        None,
    );

    let obligations = contract.generate_proof_obligations();

    assert_eq!(obligations.len(), 2);
    assert!(obligations[0].id.contains("pre"));
    assert!(obligations[1].id.contains("post"));
    assert_eq!(obligations[1].assumptions.len(), 1);
}
