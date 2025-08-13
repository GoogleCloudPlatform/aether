// Test program to demonstrate enhanced verification features
use aether::verification::contracts::*;
use aether::ast;
use aether::error::SourceLocation;
use aether::types::Type;
use aether::semantic::metadata::*;

fn main() {
    println!("Testing Enhanced Contract Verification System");
    
    // Create a function contract with LLM-first features
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
    );
    
    // Add enhanced postcondition
    contract.add_enhanced_postcondition(
        "result_correct".to_string(),
        Expression::BinaryOp {
            op: BinaryOp::Eq,
            left: Box::new(Expression::Result),
            right: Box::new(Expression::BinaryOp {
                op: BinaryOp::Div,
                left: Box::new(Expression::Variable("numerator".to_string())),
                right: Box::new(Expression::Variable("denominator".to_string())),
            }),
        },
        SourceLocation::unknown(),
        Some("Result equals numerator divided by denominator".to_string()),
        FailureAction::Abort,
        VerificationHint::SMTSolver,
    );
    
    // Add semantic intent
    contract.set_intent(IntentSpec {
        primary_intent: "Perform safe division with zero check".to_string(),
        business_purpose: Some("Mathematical division operation".to_string()),
        technical_approach: Some("Check denominator before division".to_string()),
        success_criteria: vec![
            "Division completes without error".to_string(),
            "Result is mathematically correct".to_string(),
        ],
        failure_modes: vec![
            FailureMode {
                description: "Division by zero".to_string(),
                probability: 0.1,
                impact: ImpactLevel::Critical,
                mitigation: "Precondition check".to_string(),
            }
        ],
    });
    
    // Add behavioral specification
    contract.set_behavior(BehavioralSpec {
        idempotent: true,
        pure: true,
        side_effects: vec![],
        timeout_ms: Some(100),
        retry_policy: Some(RetryPolicy::None),
        deterministic: true,
        thread_safe: true,
    });
    
    // Generate proof obligations
    let obligations = contract.generate_proof_obligations();
    
    println!("\n=== Contract Summary ===");
    println!("Function: {}", contract.function_name);
    println!("Preconditions: {}", contract.preconditions.len());
    println!("Postconditions: {}", contract.postconditions.len());
    println!("Is Pure: {}", contract.is_pure);
    
    println!("\n=== Proof Obligations ===");
    for (i, obligation) in obligations.iter().enumerate() {
        println!("{}. {} - {}", i + 1, obligation.id, obligation.description);
        println!("   Method: {:?}", obligation.method);
        println!("   Priority: {:?}", obligation.priority);
    }
    
    // Test semantic predicates
    println!("\n=== Semantic Predicates Test ===");
    let email_check = Expression::SemanticPredicate {
        predicate: "is_valid_email".to_string(),
        args: vec![Expression::Variable("email".to_string())],
    };
    println!("Email validation: {}", email_check.to_string());
    
    // Test temporal expressions
    println!("\n=== Temporal Expressions Test ===");
    let invariant = Expression::Temporal {
        op: TemporalOp::Always,
        expr: Box::new(Expression::BinaryOp {
            op: BinaryOp::Gt,
            left: Box::new(Expression::Variable("balance".to_string())),
            right: Box::new(Expression::Constant(ConstantValue::Integer(0))),
        }),
    };
    println!("Invariant: {}", invariant.to_string());
    
    // Test aggregate expressions
    println!("\n=== Aggregate Expressions Test ===");
    let sum_positive = Expression::Aggregate {
        op: AggregateOp::All,
        collection: Box::new(Expression::Variable("accounts".to_string())),
        predicate: Some(Box::new(Expression::BinaryOp {
            op: BinaryOp::Ge,
            left: Box::new(Expression::Variable("balance".to_string())),
            right: Box::new(Expression::Constant(ConstantValue::Integer(0))),
        })),
    };
    println!("All accounts non-negative: {}", sum_positive.to_string());
    
    println!("\n=== Test Complete ===");
}